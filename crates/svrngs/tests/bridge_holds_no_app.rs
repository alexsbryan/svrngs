//! bridge-holds-no-app (DESIGN C5, "It never loads app code"; D9 E0;
//! `scripts/e0_demo.py` TEST_BARS): no app module is in the bridge's
//! dependency graph. In every source under `bridge/`:
//!
//! - a relative import resolves inside `bridge/`;
//! - a built-in is imported as `node:<name>`, never one that loads code
//!   (`vm`, `module`, `child_process`, `worker_threads`, `wasi`);
//! - any other import is a package `bridge/package.json` takes from the
//!   registry, never from a path;
//! - nothing loads code at run time: no `import(`, `require(`, `eval(`,
//!   `Function(`;
//! - nothing defines or calls an app's exports or a fold — `init`, `reduce`,
//!   `propose`, `say`, `due`, `fold` — as a function of its own, so app logic
//!   cannot be pasted in either (a method such as `list.reduce(` is not one).

use std::fs;
use std::path::{Component, Path, PathBuf};

const LOADS_CODE: &[&str] = &["vm", "module", "child_process", "worker_threads", "wasi"];
const RUNS_CODE: &[&str] = &["import", "require", "eval", "Function"];
const FOLD_WORDS: &[&str] = &["init", "reduce", "propose", "say", "due", "fold"];

fn bridge_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bridge")
}

/// `text` with comments removed, and quoted string contents kept or blanked:
/// a specifier is read from a string, and a call never is. A template
/// literal is kept either way, since `${...}` in it is code.
fn strip(text: &str, keep_strings: bool) -> String {
    let b: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            '/' if b.get(i + 1) == Some(&'/') => {
                while i < b.len() && b[i] != '\n' {
                    i += 1;
                }
            }
            '/' if b.get(i + 1) == Some(&'*') => {
                i += 2;
                while i < b.len() && !(b[i] == '*' && b.get(i + 1) == Some(&'/')) {
                    i += 1;
                }
                i += 2;
            }
            q @ ('\'' | '"' | '`') => {
                out.push(q);
                i += 1;
                while i < b.len() && b[i] != q {
                    let escaped = b[i] == '\\';
                    for _ in 0..if escaped { 2 } else { 1 } {
                        if i < b.len() {
                            out.push(if keep_strings || q == '`' { b[i] } else { ' ' });
                        }
                        i += 1;
                    }
                }
                out.push(q);
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Each place `word` stands alone in `code`, with what precedes it (skipping
/// spaces) and what follows it.
fn words<'a>(code: &'a str, word: &str) -> Vec<(Option<char>, &'a str)> {
    let mut found = Vec::new();
    for (at, _) in code.match_indices(word) {
        let before = code[..at].chars().next_back();
        let after = &code[at + word.len()..];
        if before.is_some_and(is_ident) || after.chars().next().is_some_and(is_ident) {
            continue;
        }
        found.push((code[..at].trim_end().chars().next_back(), after.trim_start()));
    }
    found
}

/// The quoted specifier at the start of `s`, if one is there.
fn quoted(s: &str) -> Option<&str> {
    let q = s.chars().next().filter(|c| matches!(c, '\'' | '"'))?;
    s[1..].split(q).next()
}

/// Every static import specifier in `code`: `import 'x'`, `... from 'x'`.
fn specifiers(code: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for w in ["from", "import"] {
        out.extend(
            words(code, w)
                .into_iter()
                .filter_map(|(_, after)| quoted(after).map(String::from)),
        );
    }
    out
}

/// `base/rel`, with `.` and `..` folded, without touching the disk: a planted
/// import of a file that does not exist resolves too.
fn resolve(base: &Path, rel: &str) -> PathBuf {
    let mut out = PathBuf::new();
    for c in base.join(rel).components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

/// What `file` (whose text is `text`) does that the bridge must not.
fn violations(root: &Path, file: &Path, text: &str, deps: &[String]) -> Vec<String> {
    let code = strip(text, false);
    let mut out = Vec::new();
    for spec in specifiers(&strip(text, true)) {
        if spec.starts_with('.') || spec.starts_with('/') {
            let to = resolve(file.parent().unwrap(), &spec);
            if !to.starts_with(root) {
                out.push(format!("imports {spec}, outside bridge/"));
            }
        } else if let Some(builtin) = spec.strip_prefix("node:") {
            if LOADS_CODE.contains(&builtin) {
                out.push(format!("imports {spec}, which loads code"));
            }
        } else {
            let package = match spec.split('/').collect::<Vec<_>>()[..] {
                [scope, name, ..] if scope.starts_with('@') => format!("{scope}/{name}"),
                [name, ..] => name.to_string(),
                [] => spec.clone(),
            };
            if !deps.contains(&package) {
                out.push(format!(
                    "imports {spec}, which bridge/package.json does not take from the registry"
                ));
            }
        }
    }
    for w in RUNS_CODE {
        for (before, after) in words(&code, w) {
            if after.starts_with('(') && before != Some('.') {
                out.push(format!("calls {w}(, which loads code at run time"));
            }
        }
    }
    for w in FOLD_WORDS {
        for (before, after) in words(&code, w) {
            if after.starts_with('(') && before != Some('.') {
                out.push(format!("defines or calls {w}(, an app's export"));
            }
        }
    }
    out
}

/// The packages `package.json` takes from the registry; one taken from a path,
/// a link or git is a violation in its own right.
fn registry_deps(manifest: &str) -> (Vec<String>, Vec<String>) {
    let json: serde_json::Value = serde_json::from_str(manifest).unwrap();
    let (mut deps, mut bad) = (Vec::new(), Vec::new());
    for field in [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        for (name, version) in json[field].as_object().into_iter().flatten() {
            let v = version.as_str().unwrap_or_default();
            if v.chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit() || "^~=<>*".contains(c))
            {
                deps.push(name.clone());
            } else {
                bad.push(format!("package.json takes {name} from {v}, not the registry"));
            }
        }
    }
    (deps, bad)
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            if p.file_name() != Some("node_modules".as_ref()) {
                sources(&p, out);
            }
        } else if p.extension().is_some_and(|x| x == "js" || x == "mjs" || x == "cjs") {
            out.push(p);
        }
    }
}

#[test]
fn the_bridge_imports_no_app_code() {
    let root = resolve(&bridge_dir(), ".");
    let (deps, mut broken) = registry_deps(&fs::read_to_string(root.join("package.json")).unwrap());
    let mut files = Vec::new();
    sources(&root, &mut files);
    files.sort();
    let mut seen = Vec::new();
    for f in &files {
        let text = fs::read_to_string(f).unwrap();
        seen.extend(specifiers(&strip(&text, true)));
        let rel = f.strip_prefix(&root).unwrap().display().to_string();
        broken.extend(
            violations(&root, f, &text, &deps)
                .into_iter()
                .map(|v| format!("bridge/{rel} {v}")),
        );
    }
    // The scan read the bridge: the adapter's own import is among what it saw.
    assert!(
        seen.iter().any(|s| s == "discord.js"),
        "the scan saw no import of discord.js: {seen:?}"
    );
    assert!(
        broken.is_empty(),
        "the bridge must never load app code:\n{}",
        broken.join("\n")
    );
}

/// The scan's own failing inputs, kept: each form it refuses, refused.
#[test]
fn the_scan_refuses_every_way_in() {
    let root = PathBuf::from("/r/bridge");
    let file = root.join("bridge.mjs");
    let deps = vec!["discord.js".to_string()];
    let refused = |src: &str| violations(&root, &file, src, &deps);
    for src in [
        "import { reduce } from '../apps/expenses/expenses.js';",
        "import '../apps/expenses/expenses.js';",
        "export * from \"../apps/expenses/expenses.js\";",
        "import vm from 'node:vm';",
        "import x from 'expenses';",
        "const m = await import(name);",
        "const m = require('x');",
        "eval(src);",
        "const f = new Function('a', src);",
        "function reduce(state, act) { return state; }",
        "const view = say(state);",
        "const line = `owed: ${due(state, now)}`;",
    ] {
        assert!(!refused(src).is_empty(), "the scan let through: {src}");
    }
    for src in [
        "import { Client } from 'discord.js'; // import '../apps/x.js'",
        "import { log } from './bridge.mjs'; const n = list.reduce((a, b) => a + b, 0);",
        "const s = 'say(this) and import(that) are words here';",
    ] {
        assert_eq!(refused(src), Vec::<String>::new(), "the scan refused: {src}");
    }
    let (_, bad) = registry_deps(r#"{"dependencies": {"expenses": "file:../apps/expenses"}}"#);
    assert_eq!(bad.len(), 1);
}
