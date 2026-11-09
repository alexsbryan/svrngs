// SPDX-License-Identifier: AGPL-3.0-or-later
//! The shelf embedded (v1ui-shelf (c)): each app src/shelf.txt lists, its
//! files read into the binary from apps/ at build, written to
//! `$OUT_DIR/shelf.rs` as the list carried.rs's `SHELF` is, so svrngs's own
//! code names no app (scripts/vocab_check.py) and the shelf stays data. And
//! the programs svrngs ships terms for: each file of `src/programs/` in
//! file-name order, `$OUT_DIR/programs.rs` as share.rs's `PROGRAMS` is.

use std::path::Path;

fn main() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let listed = std::fs::read_to_string(here.join("src/shelf.txt")).expect("shelf.txt is read");
    println!("cargo::rerun-if-changed=src/shelf.txt");
    let file = |at: &Path, f: &str| format!("({f:?}, include_str!({:?}))", at.join(f));
    let look = file(&here.join("src/home"), "look.css");
    let mut shelf = String::from("&[");
    let lines = listed
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty());
    for fields in lines.map(|l| l.split('\t').collect::<Vec<_>>()) {
        let [name, app, beside, words] = fields[..] else {
            panic!("a shelf line is four fields: {fields:?}");
        };
        let (app, beside) = (here.join("../../apps").join(app), beside.split(' '));
        let files = Vec::from_iter(["index.html", "index.js"].into_iter().chain(beside));
        let files: Vec<_> = files.into_iter().map(|f| file(&app, f)).collect();
        shelf += &format!("({name:?}, {words:?}, &[{}, {look}]),", files.join(", "));
    }
    let out = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR");
    let out = Path::new(&out);
    std::fs::write(out.join("shelf.rs"), shelf + "]").expect("the shelf is written");

    // By name, never the order the filesystem lists them in. A file holding
    // another program's row is refused here: a row answers by the word it
    // holds, so it would be silently that program's terms.
    println!("cargo::rerun-if-changed=src/programs");
    let listed = std::fs::read_dir(here.join("src/programs")).expect("src/programs is read");
    let paths: std::collections::BTreeSet<_> = listed.map(|e| e.expect("a file").path()).collect();
    let mut programs = String::from("&[");
    for path in paths {
        let word = path.file_stem().and_then(|s| s.to_str()).expect("a name");
        let text = std::fs::read_to_string(&path).expect("a program's file is read");
        let own = |line: &str| {
            let row = line.split('#').next().unwrap_or("").trim();
            row.is_empty() || row.split_whitespace().nth(1) == Some(word)
        };
        assert!(text.lines().all(own), "{path:?}: a row of another program");
        programs += &format!("({word:?}, include_str!({path:?})),");
    }
    std::fs::write(out.join("programs.rs"), programs + "]").expect("programs are written");
}
