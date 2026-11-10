// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// What `check` says of an app of `files`, each finding as `file:line`
/// and what.
fn said(files: &[(&str, &str)]) -> Vec<(String, Refused)> {
    let dir = tempfile::tempdir().unwrap();
    for (name, text) in files {
        std::fs::write(dir.path().join(name), text).unwrap();
    }
    let found = check(dir.path()).expect("an index.html");
    (found.into_iter())
        .map(|(file, line, r)| (format!("{file}:{line}"), r))
        .collect()
}

/// **What an assistant writes first is found, by file and line** (svrn-docs
/// `rings/current/EXTENSION_POINT.md` §5, "Inline script or a CDN link,
/// what assistants write first"; §7, G3): a font and a style sheet from
/// elsewhere, a script from a CDN, a tag laid out over lines as a formatter
/// lays it, a script written in the page, a handler and a `javascript:`
/// address, each said in the words the operator approved. Planted, watched
/// failing: `inline` never set, and the script on line 11 is not said.
#[test]
fn what_an_assistant_writes_first_is_found_by_file_and_line() {
    let page = r#"<!doctype html>
<html>
<head>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter">
<link rel=stylesheet href=https://cdn.jsdelivr.net/npm/water.css>
<script src="https://unpkg.com/preact@10/dist/preact.min.js"></script>
<link
  rel="stylesheet"
  href="//cdn.example.org/site.css"
/>
<script type="module">
  document.title = 'hi';
</script>
</head>
<body><button onclick="sign()">Sign</button> <a href="javascript:go()">go</a></body>
</html>
"#;
    let font = Refused::Font("fonts.googleapis.com".into());
    assert_eq!(
        said(&[("index.html", page)]),
        [
            ("index.html:4".to_string(), font.clone()),
            (
                "index.html:5".to_string(),
                Refused::Style("cdn.jsdelivr.net".into())
            ),
            (
                "index.html:6".to_string(),
                Refused::Script("unpkg.com".into())
            ),
            (
                "index.html:7".to_string(),
                Refused::Style("cdn.example.org".into())
            ),
            ("index.html:11".to_string(), Refused::Inline),
            ("index.html:15".to_string(), Refused::Inline),
        ]
    );
    assert_eq!(
        font.words(),
        "This page asks fonts.googleapis.com for a font, and apps load only their own files. That way, opening an app tells nobody outside your group you were here. To fix it, save the .woff2 beside index.html and point @font-face at it."
    );
    assert_eq!(
        Refused::Script("unpkg.com".into()).words(),
        "This page asks unpkg.com for a script, and apps load only their own files. That way, opening an app tells nobody outside your group you were here. To fix it, save the script beside index.html and load it from there."
    );
    assert_eq!(
        Refused::Inline.words(),
        "This page has a script written inside it, and apps run scripts only from their own files. Move it to a .js file beside index.html."
    );
}

/// **A page of its own files is told nothing**: its script and style sheet
/// beside it, a font of its own, an inline style, an image written into
/// the page or named by a style sheet, JSON data in a script tag, a link
/// out, and a module imported from beside it. Planted, watched failing:
/// every `<script>` read as written inside the page, and `index.html:3` is
/// said.
#[test]
fn a_page_of_its_own_files_is_told_nothing() {
    let page = r#"<!doctype html><title>ok</title>
<link rel="stylesheet" href="./look.css?v=2">
<script type="module" src="index.js"></script>
<script type="application/json" id="data">{"a": 1}</script>
<style>body { font: 16px Own; }</style>
<p style="color: red"><img src="data:image/png;base64,AAAA" alt="">
<a href="https://example.org/">a link out is a navigation, not a load</a>
"#;
    let css = "@font-face { font-family: Own; src: url(own.woff2); }\nbody { background: url(https://example.org/bg.png); }\n";
    let js = "// import('https://example.org/x.js') is never run\nimport { x } from './x.js';\n";
    let files = [("index.html", page), ("look.css", css), ("index.js", js)];
    assert_eq!(said(&files), []);
}

/// **The style sheets and scripts beside it are read too**: a style sheet
/// importing a font and a style sheet from elsewhere, a style written in
/// the page naming a font elsewhere, and a script importing a module from
/// a CDN, each by its own file and line; and a folder with no `index.html`
/// is none, never nothing found. Planted, watched failing: the files beside
/// the page never read, and only `index.html:2` is said.
#[test]
fn the_style_sheets_and_scripts_beside_it_are_read_too() {
    let page = "<!doctype html>\n<style>@font-face { src: url(https://fonts.gstatic.com/s/inter.woff2) }</style>\n";
    let css = "body { margin: 0 }\n@import url(\"https://fonts.googleapis.com/css2?family=Inter\");\n@import 'https://cdn.example.org/a.css';\n";
    let js = "import { h } from 'https://esm.sh/preact';\nconst m = await import(\"https://cdn.example.org/m.js\");\n";
    assert_eq!(
        said(&[("index.html", page), ("look.css", css), ("app.mjs", js)]),
        [
            ("app.mjs:1".to_string(), Refused::Script("esm.sh".into())),
            (
                "app.mjs:2".to_string(),
                Refused::Script("cdn.example.org".into())
            ),
            (
                "index.html:2".to_string(),
                Refused::Font("fonts.gstatic.com".into())
            ),
            (
                "look.css:2".to_string(),
                Refused::Font("fonts.googleapis.com".into())
            ),
            (
                "look.css:3".to_string(),
                Refused::Style("cdn.example.org".into())
            ),
        ]
    );
    let none = tempfile::tempdir().unwrap();
    std::fs::write(none.path().join("app.js"), js).unwrap();
    assert_eq!(check(none.path()), None);
}
