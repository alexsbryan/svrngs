// SPDX-License-Identifier: AGPL-3.0-or-later
//! The shelf embedded (v1ui-shelf (c)): each app src/shelf.txt lists, its
//! files read into the binary from apps/ at build, written to
//! `$OUT_DIR/shelf.rs` as the list carried.rs's `SHELF` is, so svrngs's own
//! code names no app (scripts/vocab_check.py) and the shelf stays data.
//! So too each sign-in helper's steps well_known.txt names (v1ui-polish
//! (d)), as share.rs's `HELPERS`.

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
        let (name, app, beside, words) = (
            fields[0],
            here.join("../../apps").join(fields[1]),
            fields[2].split(' '),
            fields[3],
        );
        let files = Vec::from_iter(["index.html", "index.js"].into_iter().chain(beside));
        let files: Vec<_> = files.into_iter().map(|f| file(&app, f)).collect();
        shelf += &format!("({name:?}, {words:?}, &[{}, {look}]),", files.join(", "));
    }
    let out = Path::new(&std::env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("shelf.rs");
    std::fs::write(&out, shelf + "]").expect("the shelf is written");
    // Each sign-in helper's steps well_known.txt names, `helper <kind> <port> <steps>`.
    let table = std::fs::read_to_string(here.join("src/well_known.txt")).expect("the table");
    println!("cargo::rerun-if-changed=src/well_known.txt");
    let (root, mut helpers) = (here.join("../.."), String::from("&["));
    for l in table.lines() {
        if let ["helper", kind, _, steps] = l.split_whitespace().collect::<Vec<_>>()[..] {
            helpers += &format!("({kind:?}, include_str!({:?})),", root.join(steps));
        }
    }
    std::fs::write(out.with_file_name("helpers.rs"), helpers + "]").expect("helpers.rs");
}
