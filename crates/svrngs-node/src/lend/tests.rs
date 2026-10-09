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
    std::fs::write(
        &file,
        r#"{"port": 8096, "kind": "k", "limit": 3, "refusd": ["/admin"]}"#,
    )
    .unwrap();
    assert!(matches!(
        read(root.path(), &name),
        Err(LendError::Malformed { .. })
    ));
    std::fs::write(&file, r#"{"port": 0, "kind": "k", "limit": 3}"#).unwrap();
    assert!(read(root.path(), &name).is_err(), "port 0 is no port");
    for unsaid in [
        r#"{"port": 8096, "limit": 3}"#,
        r#"{"port": 8096, "kind": "k"}"#,
        r#"{"port": 8096, "kind": "k", "limit": 0}"#,
        r#"{"port": 8096, "kind": "a/b", "limit": 3}"#,
    ] {
        std::fs::write(&file, unsaid).unwrap();
        assert!(
            read(root.path(), &name).is_err(),
            "a kind or limit absent or not one is refused, never defaulted: {unsaid}"
        );
    }
    std::fs::write(
        &file,
        r#"{"port": 8096, "kind": "k", "limit": 3, "refused": ["/admin"]}"#,
    )
    .unwrap();
    let lend = read(root.path(), &name).unwrap().unwrap();
    assert_eq!(lend.port.get(), 8096);
    assert_eq!(
        lend.refusing("/Admin/x").map(RefusedPath::as_str),
        Some("/admin")
    );
}

fn lend(name: &str, port: u16) -> Lend {
    Lend {
        name: LendName::parse(name).unwrap(),
        port: NonZeroU16::new(port).unwrap(),
        kind: Kind::parse("k").unwrap(),
        limit: NonZeroU16::new(3).unwrap(),
        refused: vec![refused("/admin")],
        upload: None,
        reads: Vec::new(),
        sign_in: None,
        asks: None,
    }
}

/// What `declare` writes, `read` and `all` read back; `withdraw` takes it
/// back, its secret with it, and says when there was nothing to take.
#[test]
fn a_lend_declared_is_read_back_and_withdrawn_whole() {
    let root = tempfile::tempdir().unwrap();
    let mut shelf = lend("shelf", 8096);
    shelf.upload = Some(Upload::from_mbit(NonZeroU32::new(30).unwrap()));
    // The kinds it reads (u5-2) come back too; planted: not written.
    shelf.reads = vec![Kind::parse("films").unwrap(), Kind::parse("music").unwrap()];
    declare(root.path(), &shelf, Some(("authorization", "s3cret"))).unwrap();
    declare(root.path(), &lend("pad", 9000), None).unwrap();
    assert_eq!(read(root.path(), &shelf.name).unwrap(), Some(shelf.clone()));
    let names: Vec<String> = all(root.path())
        .unwrap()
        .into_iter()
        .map(|l| l.unwrap().name.to_string())
        .collect();
    assert_eq!(names, ["pad", "shelf"], "every lend, by name, nothing else");
    let secrets = secret_dir(root.path(), &shelf.name);
    assert_eq!(
        crate::declared::read_declared_in(&secrets),
        vec![("authorization".to_string(), "s3cret".to_string())]
    );

    declare(root.path(), &shelf, None).unwrap();
    assert!(
        crate::declared::read_declared_in(&secrets).is_empty(),
        "declared again with no secret, it keeps none"
    );
    assert!(withdraw(root.path(), &shelf.name).unwrap());
    assert!(read(root.path(), &shelf.name).unwrap().is_none());
    assert!(!secrets.exists());
    assert!(
        !withdraw(root.path(), &shelf.name).unwrap(),
        "nothing left to take"
    );
}

/// A sign-in (mwp-1) is kept in the declaration and read back; with a
/// secret beside it, refused and nothing written; and a lend with none is
/// written with no word of it, as before.
#[test]
fn a_sign_in_is_declared_alone_and_a_lend_without_one_as_before() {
    let root = tempfile::tempdir().unwrap();
    let mut signed = lend("signed", 8096);
    signed.sign_in = NonZeroU16::new(9000);
    assert!(matches!(
        declare(root.path(), &signed, Some(("authorization", "s3cret"))),
        Err(LendError::SecretAndSignIn(_))
    ));
    assert_eq!(read(root.path(), &signed.name).unwrap(), None);
    declare(root.path(), &signed, None).unwrap();
    assert_eq!(read(root.path(), &signed.name).unwrap(), Some(signed));
    declare(root.path(), &lend("pad", 9001), None).unwrap();
    let text = std::fs::read_to_string(lends_dir(root.path()).join("pad.json")).unwrap();
    assert!(!text.contains("sign_in"), "{text}");
}

/// A lend's upload is its lender's to state: absent is none declared, never
/// a default; zero is no upload, refused; one stated reads back in bits a
/// second, and prints as a speed test states it.
#[test]
fn an_upload_is_declared_or_absent_never_defaulted() {
    let root = tempfile::tempdir().unwrap();
    let name = LendName::parse("shelf").unwrap();
    std::fs::create_dir_all(lends_dir(root.path())).unwrap();
    let file = lends_dir(root.path()).join("shelf.json");
    std::fs::write(&file, r#"{"port": 8096, "kind": "k", "limit": 3}"#).unwrap();
    assert_eq!(read(root.path(), &name).unwrap().unwrap().upload, None);
    std::fs::write(
        &file,
        r#"{"port": 8096, "kind": "k", "limit": 3, "upload_bps": 0}"#,
    )
    .unwrap();
    assert!(read(root.path(), &name).is_err(), "no upload is not one");
    std::fs::write(
        &file,
        r#"{"port": 8096, "kind": "k", "limit": 3, "upload_bps": 30000000}"#,
    )
    .unwrap();
    let upload = read(root.path(), &name).unwrap().unwrap().upload.unwrap();
    assert_eq!(upload.bps(), 30_000_000);
    assert_eq!(upload.to_string(), "30 Mbit/s");
    assert_eq!(
        Upload::new(NonZeroU64::new(1_500_000).unwrap()).to_string(),
        "1500000 bit/s"
    );
}
