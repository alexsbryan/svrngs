// SPDX-License-Identifier: AGPL-3.0-or-later
//! A folder synced both ways (sy-1): given / when / then over `admit`.
//! alex is key 1, bo key 2, cy key 3 (`tests_support::ring`).

use crate::tests_support::*;
use crate::*;

fn now(fill: u8) -> Option<(ContentHash, u64)> {
    Some((ContentHash::from_bytes([fill; 32]), 4096))
}

fn entry(k: u8, seq: u64, path: &str) -> Option<Entry> {
    Some(Entry {
        key: actor_of(&key(k)),
        seq,
        path: SyncPath::parse(path).unwrap(),
    })
}

fn synced(path: &str, now: Option<(ContentHash, u64)>, base: Option<Entry>) -> RailAct {
    RailAct::Synced {
        folder: "Shared".into(),
        changes: vec![Change {
            path: SyncPath::parse(path).unwrap(),
            now,
            base,
        }],
    }
}

/// What the record places: each file's path and hash fill, beside or not.
fn placed(ops: &[Op<SignedOp>]) -> Vec<(String, u8, bool)> {
    let mut out: Vec<(String, u8, bool)> = admitted(ops)
        .layout()
        .into_iter()
        .map(|p| (p.path.into(), p.hash.as_bytes()[0], p.beside))
        .collect();
    out.sort();
    out
}

fn at(path: &str, fill: u8) -> (String, u8, bool) {
    (path.into(), fill, false)
}

#[test]
fn an_add_then_an_edit_by_another_then_a_delete_by_a_third() {
    let mut ops = run(&key(1), 1_000, [synced("a.txt", now(7), None)]);
    assert_eq!(placed(&ops), vec![at("a.txt", 7)]);

    ops.extend(run(
        &key(2),
        1_010,
        [synced("a.txt", now(8), entry(1, 0, "a.txt"))],
    ));
    assert_eq!(
        placed(&ops),
        vec![at("a.txt", 8)],
        "the edit replaces what it names"
    );

    ops.extend(run(
        &key(3),
        1_020,
        [synced("a.txt", None, entry(2, 0, "a.txt"))],
    ));
    assert_eq!(placed(&ops), vec![], "the delete replaces the edit");
}

/// Ana's edit (alex, 8) and Bo's (bo, 9) of alex's first version, neither
/// seeing the other; which goes under the name is the key order's.
fn two_edits(path: &str) -> (Vec<Op<SignedOp>>, bool) {
    let mut ops = run(&key(1), 1_000, [synced(path, now(7), None)]);
    let ana = after(
        &key(1),
        &ops[0],
        1_010,
        synced(path, now(8), entry(1, 0, path)),
    );
    ops.push(ana);
    ops.extend(run(
        &key(2),
        1_005,
        [synced(path, now(9), entry(1, 0, path))],
    ));
    (ops, actor_of(&key(1)) < actor_of(&key(2)))
}

#[test]
fn two_edits_of_one_version_are_both_kept_the_second_beside_as_a_conflict() {
    let (ops, alex_first) = two_edits("d/a.txt");
    let want = match alex_first {
        true => vec![("d/a (conflict bo).txt".into(), 9, true), at("d/a.txt", 8)],
        false => vec![
            ("d/a (conflict alex).txt".into(), 8, true),
            at("d/a.txt", 9),
        ],
    };
    assert_eq!(
        placed(&ops),
        want,
        "neither edit saw the other: both are kept, the first by key under the name"
    );
}

#[test]
fn a_delete_beside_an_edit_of_the_same_version_keeps_the_edit() {
    let mut ops = run(&key(1), 1_000, [synced("a", now(7), None)]);
    ops.extend(run(&key(2), 1_010, [synced("a", now(8), entry(1, 0, "a"))]));
    ops.extend(run(&key(3), 1_010, [synced("a", None, entry(1, 0, "a"))]));
    assert_eq!(placed(&ops), vec![at("a", 8)]);
}

#[test]
fn deleting_the_conflict_s_copy_ends_the_conflict() {
    let (mut ops, alex_first) = two_edits("a.txt");
    let (copy, its, kept) = match alex_first {
        true => ("a (conflict bo).txt", entry(2, 0, "a.txt"), 8),
        false => ("a (conflict alex).txt", entry(1, 1, "a.txt"), 9),
    };
    ops.extend(run(&key(3), 1_020, [synced(copy, None, its)]));
    assert_eq!(placed(&ops), vec![at("a.txt", kept)]);
}

#[test]
fn a_synced_change_ships_at_its_own_line_version() {
    let op = &run(&key(1), 1_000, [synced("a", now(7), None)])[0];
    assert_eq!(op.kind.line_version(), SYNCED_LINE_VERSION);
    assert_eq!(
        SignedOp::UNDERSTANDS,
        crate::VISIT_LINE_VERSION,
        "this build reads it, and removal's, a device link's, a relay's, a yes to a carried app's version and a visit after"
    );
}

/// **A path under a folder is named parts, and nothing else** (syp-4 c):
/// each shape that could leave the folder, or that another system reads
/// as a separator, is refused by its name; a path of named parts is taken
/// as written. Red with the parse taking any text.
#[test]
fn a_path_under_a_folder_is_named_parts_and_nothing_else() {
    let refused = [
        ("", NotASyncPath::Empty),
        ("/etc/passwd", NotASyncPath::EmptyPart("/etc/passwd".into())),
        ("a//b", NotASyncPath::EmptyPart("a//b".into())),
        ("a/", NotASyncPath::EmptyPart("a/".into())),
        (".", NotASyncPath::Climbs(".".into())),
        ("..", NotASyncPath::Climbs("..".into())),
        ("../outside", NotASyncPath::Climbs("../outside".into())),
        ("a/./b", NotASyncPath::Climbs("a/./b".into())),
        ("a/../../b", NotASyncPath::Climbs("a/../../b".into())),
        ("a\\..\\b", NotASyncPath::Separator("a\\..\\b".into())),
        ("a\0b", NotASyncPath::Separator("a\0b".into())),
        ("C:/Windows", NotASyncPath::Prefix("C:/Windows".into())),
        ("c:", NotASyncPath::Prefix("c:".into())),
        ("a:b/c", NotASyncPath::Prefix("a:b/c".into())),
    ];
    for (raw, why) in refused {
        assert_eq!(SyncPath::parse(raw), Err(why), "{raw:?}");
    }
    for raw in ["a", "d/notes.txt", "a b/.hidden/c..d", "...", "ab:c/d"] {
        assert_eq!(
            SyncPath::parse(raw).map(String::from),
            Ok(raw.into()),
            "{raw:?}"
        );
    }
}

/// **An act naming a path outside the folder is no act** (syp-4 c): read
/// by any copy, a change naming `../outside` is a `MalformedLine` saying
/// why, and nothing is placed. Red with the parse taking any text: the
/// file placed at `../outside`.
#[test]
fn an_act_naming_a_path_outside_the_folder_is_a_malformed_line() {
    let op = &run(&key(1), 1_000, [synced("PLACEHOLDER", now(7), None)])[0];
    let line = serde_json::to_string(op)
        .unwrap()
        .replace("PLACEHOLDER", "../outside");
    let read = serde_json::from_str::<Op<SignedOp>>(&line);
    let why = read.map(|_| ()).unwrap_err().to_string();
    assert!(
        why.contains("`../outside` is not under the folder"),
        "{why}"
    );
}

/// **A conflict's name makes no part of its writer's person** (syp-4 c):
/// a person named with a `/` is written beside the file with the `/`
/// in no part of its own. Red with the person's name written as it is.
#[test]
fn a_conflict_s_name_makes_no_part_of_its_writer_s_person() {
    let path = SyncPath::parse("d/a.txt").unwrap();
    let named = super::beside(&path, &Person::parse("x/../../y").unwrap());
    assert_eq!(named.as_str(), "d/a (conflict x∕..∕..∕y).txt");
    assert_eq!(SyncPath::parse(named.as_str()), Ok(named));
}

/// alex brings Money, its page and its script in two acts, and a note
/// beside it; bo changes its script.
fn brought_then_changed() -> Vec<Op<SignedOp>> {
    let mut ops = run(
        &key(1),
        1_000,
        [
            synced("apps/Money/index.html", now(7), None),
            synced("apps/Money/app.js", now(8), None),
            synced("notes.txt", now(5), None),
        ],
    );
    let base = entry(1, 1, "apps/Money/app.js");
    ops.extend(run(
        &key(2),
        1_010,
        [synced("apps/Money/app.js", now(9), base)],
    ));
    let at = |p: &str, f| at(&format!("apps/Money/{p}"), f);
    assert_eq!(
        placed(&ops),
        [
            at("app.js", 9),
            at("index.html", 7),
            ("notes.txt".into(), 5, false)
        ]
    );
    ops
}

/// The files `fills` name, under `apps/Money/`.
fn money(fills: [(&str, u8); 2]) -> Files {
    let file = |(p, f): (&str, u8)| {
        (
            SyncPath::parse(&format!("apps/Money/{p}")).unwrap(),
            ContentHash::from_bytes([f; 32]),
        )
    };
    fills.map(file).into_iter().collect()
}

/// **An app runs as it was brought** (canon-pin (b)): after bo's change
/// the layout places his script, and what runs of Money is alex's two
/// files, each path as the record first placed it; the note is no app's.
/// Planted: the newest hash at each path — bo's script runs.
#[test]
fn an_app_runs_as_it_was_brought_whoever_changes_it() {
    let runs = admitted(&brought_then_changed()).runs().clone();
    assert_eq!(
        runs.brought["Money"],
        money([("index.html", 7), ("app.js", 8)])
    );
    assert_eq!(runs.brought.len(), 1, "the note is no app's");
}

/// The yeses `ops` hold, by kind and person, each with whether it names
/// `files`; bo's phone (key 4) stands beside his computer.
fn said(ops: &[Op<SignedOp>], files: &Files) -> Vec<(String, String, bool)> {
    let mut seed = ring().members;
    seed.entry(p("bo")).or_default().push(actor_of(&key(4)));
    let a = admit(
        &ActSet::of(ops),
        &[],
        &Roster::new(seed),
        NS,
        &Ed25519Verifier,
    );
    let runs = a.runs().clone();
    (runs.said.iter())
        .map(|((k, who), f)| (k.clone(), who.clone(), f == files))
        .collect()
}

fn yes(files: &Files) -> RailAct {
    RailAct::Runs {
        app: "Money".into(),
        files: files.clone(),
    }
}

/// **Only carry what you signed: a yes counts only from its person's own
/// device** (capability-signed; svrn-docs `rings/current/
/// WHAT_CAN_ACT_AS_YOU.md` §3, shortfall 1). After bo's change, alex says
/// yes to bo's version on his own computer, and cy's computer writes a yes
/// naming bo, `on_behalf_of` him: a remote computer's word that bo chose
/// what runs as him. Refused: it counts nowhere, so Money runs as bo as it
/// was brought, and bo's own yes, signed on his phone and held by any
/// computer, is his. Planted: whose read from `on_behalf_of` again — cy's
/// computer chooses for bo.
#[test]
fn a_yes_another_computer_writes_for_someone_else_counts_nowhere() {
    let mut ops = brought_then_changed();
    let new = money([("index.html", 7), ("app.js", 9)]);
    let alex = ops[2].clone();
    ops.push(after(&key(1), &alex, 1_020, yes(&new)));
    ops.push(signed_for(NS, &key(3), 1_030, 0, yes(&new), Some("bo")));
    let only = |who: &str| (String::from("Money"), String::from(who), true);
    assert_eq!(
        said(&ops, &new),
        [only("alex")],
        "cy's computer said nothing for bo"
    );

    // Kept: bo's phone signs his own yes; whichever computer carries the
    // line, it is the phone's person's.
    ops.push(signed_for(NS, &key(4), 1_040, 0, yes(&new), None));
    assert_eq!(said(&ops, &new), [only("alex"), only("bo")]);
}

/// **A yes names only files a folder act placed** (shortfall 8; `blob.rs`
/// fetches any hash a yes names). cy says yes to a Money whose script is
/// bytes no `Synced` act ever placed at that path: code cy supplied, which
/// would run as cy on every computer that fetched it. Refused: it counts
/// nowhere, and Money runs as cy as it was brought. Kept: a yes to the
/// files the layout places counts. Planted: every file a yes names taken as
/// placed.
#[test]
fn a_yes_naming_files_no_folder_placed_counts_nowhere() {
    let mut ops = brought_then_changed();
    let supplied = money([("index.html", 7), ("app.js", 0x70)]);
    ops.push(signed(&key(3), 1_030, 0, yes(&supplied)));
    let runs = admitted(&ops).runs().clone();
    assert_eq!(runs.said.len(), 0, "no folder placed cy's script");
    assert_eq!(
        runs.of("Money", "cy"),
        Some(&money([("index.html", 7), ("app.js", 8)]))
    );

    let placed = money([("index.html", 7), ("app.js", 9)]);
    ops.pop();
    ops.push(signed(&key(3), 1_030, 0, yes(&placed)));
    assert_eq!(admitted(&ops).runs().of("Money", "cy"), Some(&placed));
}
