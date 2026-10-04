// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

fn refused(raw: &str) -> RefusedPath {
    RefusedPath::parse(raw).unwrap()
}

/// `covers` at a segment boundary, as upstream's: the path itself and
/// what is under it, never a longer name beside it.
#[test]
fn a_refused_path_covers_itself_and_what_is_under_it() {
    let r = refused("/admin");
    assert!(r.covers("/admin"));
    assert!(r.covers("/admin/users"));
    assert!(!r.covers("/administer"));
    assert!(!r.covers("/"));
    assert!(refused("/admin/").covers("/admin/x"));
}

/// The failing inputs svrngs's comparison exists for: the spellings a
/// service reads as the refused path.
#[test]
fn a_refused_path_is_not_dodged_by_case_encoding_or_slashes() {
    let r = refused("/Auth/Keys");
    for dodge in [
        "/auth/keys",
        "/AUTH/KEYS/1",
        "/%41uth/Keys",
        "//auth//keys",
        "/auth%2Fkeys",
    ] {
        assert!(r.covers(dodge), "{dodge}");
    }
}

#[test]
fn a_lend_name_cannot_express_a_path() {
    assert!(LendName::parse("shelf-1_a").is_ok());
    for bad in ["", "..", "a/b", "a%2fb", "a b", "a\u{7f}", &"x".repeat(65)] {
        assert!(LendName::parse(bad).is_err(), "{bad:?}");
    }
}

/// A field the door does not know is refused, so a misspelt refusal is
/// never read as none; a missing file is no lend, not an error.
#[test]
fn a_declaration_is_read_strictly() {
    let root = tempfile::tempdir().unwrap();
    let name = LendName::parse("shelf").unwrap();
    assert!(read(root.path(), &name).unwrap().is_none());
    std::fs::create_dir_all(lends_dir(root.path())).unwrap();
    let file = lends_dir(root.path()).join("shelf.json");
    std::fs::write(&file, r#"{"port": 8096, "refusd": ["/admin"]}"#).unwrap();
    assert!(matches!(
        read(root.path(), &name),
        Err(LendError::Malformed { .. })
    ));
    std::fs::write(&file, r#"{"port": 0}"#).unwrap();
    assert!(read(root.path(), &name).is_err(), "port 0 is no port");
    std::fs::write(&file, r#"{"port": 8096, "refused": ["/admin"]}"#).unwrap();
    let lend = read(root.path(), &name).unwrap().unwrap();
    assert_eq!(lend.port.get(), 8096);
    assert_eq!(
        lend.refusing("/Admin/x").map(RefusedPath::as_str),
        Some("/admin")
    );
}
