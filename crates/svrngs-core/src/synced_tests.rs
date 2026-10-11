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
    assert_eq!(SignedOp::UNDERSTANDS, SYNCED_LINE_VERSION);
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
