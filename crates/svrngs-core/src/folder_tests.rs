// SPDX-License-Identifier: AGPL-3.0-or-later
//! A shared folder in the record (gf-1): given / when / then over `admit`.

use crate::tests_support::*;
use crate::*;

fn file(path: &str, fill: u8) -> SharedFile {
    SharedFile {
        path: path.into(),
        size: 47_000_000,
        hash: FileHash([fill; 32]),
    }
}

fn folder(name: &str, files: Vec<SharedFile>) -> RailAct {
    RailAct::Folder {
        name: name.into(),
        files,
    }
}

#[test]
fn a_folder_names_its_files_by_hash_and_whose_they_are() {
    let ops = run(&key(1), 1_000, [folder("films", vec![file("a.mp4", 7)])]);
    let shared = admitted(&ops).folders();
    assert_eq!(shared.len(), 1);
    assert_eq!(shared[0].key, actor_of(&key(1)));
    assert_eq!(shared[0].person, p("alex"));
    assert_eq!(shared[0].files, vec![file("a.mp4", 7)]);
}

#[test]
fn a_folder_shared_again_is_its_newest_list_and_an_empty_one_is_its_stop() {
    let ops = run(
        &key(1),
        1_000,
        [
            folder("films", vec![file("a.mp4", 7), file("a.mkv", 8)]),
            folder("films", vec![file("a.mp4", 7)]),
        ],
    );
    let shared = admitted(&ops).folders();
    assert_eq!(shared.len(), 1);
    assert_eq!(shared[0].files, vec![file("a.mp4", 7)]);

    let mut stopped = ops.clone();
    stopped.push(after(&key(1), &ops[1], 1_010, folder("films", vec![])));
    assert!(admitted(&stopped).folders().is_empty());
}

#[test]
fn two_people_holding_one_hash_are_two_holders_of_it() {
    let mut ops = run(&key(1), 1_000, [folder("films", vec![file("a.mp4", 7)])]);
    ops.extend(run(
        &key(3),
        1_000,
        [folder("copy", vec![file("same.mp4", 7)])],
    ));
    let holders = admitted(&ops).holders(&FileHash([7; 32]));
    assert_eq!(holders, vec![actor_of(&key(1)), actor_of(&key(3))]);
    assert!(admitted(&ops).holders(&FileHash([9; 32])).is_empty());
}

#[test]
fn a_hash_is_spelled_as_hex_and_nothing_else_parses() {
    let h = FileHash([0xab; 32]);
    let json = serde_json::to_string(&h).unwrap();
    assert_eq!(json, format!("\"{}\"", "ab".repeat(32)));
    assert_eq!(serde_json::from_str::<FileHash>(&json).unwrap(), h);
    assert!(serde_json::from_str::<FileHash>("\"abcd\"").is_err());
    assert_eq!(FileHash::parse(&"ab".repeat(32)), Some(h));
    assert_eq!(FileHash::parse("zz"), None);
}

#[test]
fn a_folder_ships_at_its_own_line_version() {
    let op = &run(&key(1), 1_000, [folder("films", vec![file("a.mp4", 7)])])[0];
    assert_eq!(op.kind.line_version(), FOLDER_LINE_VERSION);
    assert_eq!(SignedOp::UNDERSTANDS, FOLDER_LINE_VERSION);
}
