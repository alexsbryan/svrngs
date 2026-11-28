// SPDX-License-Identifier: AGPL-3.0-or-later
//! The shelf embedded (v1ui-shelf (c)): each app src/shelf.txt lists, its
//! files read into the binary from apps/ at build, written to
//! `$OUT_DIR/shelf.rs` as the list carried.rs's `SHELF` is, so svrngs's own
//! code names no app (scripts/vocab_check.py) and the shelf stays data. And
//! the programs svrngs ships terms for: each file of `src/programs/` in
//! file-name order, `$OUT_DIR/programs.rs` as share.rs's `PROGRAMS` is.

use std::path::{Path, PathBuf};

fn main() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let listed = std::fs::read_to_string(here.join("src/shelf.txt")).expect("shelf.txt is read");
    println!("cargo::rerun-if-changed=src/shelf.txt");
    // A file include_str! reads rustc watches itself, so this script reruns
    // only when the list changes.
    let file = |at: &Path, f: &str| format!("({f:?}, include_str!({:?}))", at.join(f));
    let look = file(&here.join("src/home"), "look.css");
    let mut shelf = String::from("&[");
    let lines = listed
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty());
    for fields in lines.map(|l| l.split('\t').collect::<Vec<_>>()) {
        let [name, app, beside]: [&str; 3] = fields.try_into().expect("three shelf fields");
        let (app, beside) = (here.join("../../apps").join(app), beside.split(' '));
        let files = Vec::from_iter(["index.html", "index.js"].into_iter().chain(beside));
        let files: Vec<_> = files.into_iter().map(|f| file(&app, f)).collect();
        shelf += &format!("({name:?}, &[{}, {look}]),", files.join(", "));
    }
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
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

    // update.rs's `BUILT`, the floor every release is held to: when this
    // svrngs's commit was made, the later of SOURCE_DATE_EPOCH, as a builder
    // says it, and git's time for HEAD, read in this package's folder, where
    // a build script runs. Neither known stops the build: no floor would take
    // any release ever signed, and this build's clock is a time nobody chose.
    //
    // Only SOURCE_DATE_EPOCH reruns this script, never a commit: a commit
    // here, docs alone included, would rebuild svrngs-node and all above it
    // for every worker's next cargo run. A release is exact all the same, as
    // scripts/build-release.sh says its own commit's time there. A dev build
    // keeps git's time from when this script last ran, a floor at or below
    // its commit, which only lets it take a release it would anyway: no one
    // installs a dev build from a page.
    println!("cargo::rerun-if-env-changed=SOURCE_DATE_EPOCH");
    let mut ask = std::process::Command::new("git");
    let said = ask.args(["log", "-1", "--format=%ct"]).output().ok();
    let said = said.filter(|said| said.status.success());
    let git = said.and_then(|said| String::from_utf8(said.stdout).ok());
    let times = [std::env::var("SOURCE_DATE_EPOCH").ok(), git];
    let time = |t: &String| t.trim().parse().expect("SOURCE_DATE_EPOCH is seconds");
    let built: Option<u32> = times.iter().flatten().map(time).max();
    let built = built.expect("svrngs builds in its git checkout, or with SOURCE_DATE_EPOCH saying when its commit was made");
    std::fs::write(out.join("built.rs"), built.to_string()).expect("the floor is written");
}
