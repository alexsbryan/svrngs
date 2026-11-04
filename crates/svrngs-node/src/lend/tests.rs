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
        r#"{"port": 8096, "kind": "k", "limit": 0}"#,
        r#"{"port": 8096, "kind": "a/b", "limit": 3}"#,
    ] {
        std::fs::write(&file, unsaid).unwrap();
        assert!(
            read(root.path(), &name).is_err(),
            "a kind absent, or a kind or limit not one, is refused, never defaulted: {unsaid}"
        );
    }
    std::fs::write(
        &file,
        r#"{"port": 8096, "kind": "k", "limit": 3, "refused": ["/admin"]}"#,
    )
    .unwrap();
    let lend = read(root.path(), &name).unwrap().unwrap();
    assert_eq!(lend.source, Source::Port(NonZeroU16::new(8096).unwrap()));
    assert_eq!(
        lend.refusing("/Admin/x").map(RefusedPath::as_str),
        Some("/admin")
    );
}

fn lend(name: &str, port: u16) -> Lend {
    Lend {
        name: LendName::parse(name).unwrap(),
        source: Source::Port(NonZeroU16::new(port).unwrap()),
        kind: Kind::parse("k").unwrap(),
        limit: NonZeroU16::new(3),
        refused: vec![refused("/admin")],
        upload: None,
        reads: Vec::new(),
        sign_in_with_svrngs: None,
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

/// A declaration holding a sign-in of the lender's own is refused as a
/// field this svrngs does not know, never ignored: the hook it named is
/// gone, and sign-in with svrngs is the door's own (SIGN_IN_WITH_SVRNGS
/// §5, §8 9). Red at HEAD, where the field was read and kept.
#[test]
fn an_old_declaration_s_sign_in_is_refused_not_ignored() {
    let root = tempfile::tempdir().unwrap();
    let name = LendName::parse("signed").unwrap();
    std::fs::create_dir_all(lends_dir(root.path())).unwrap();
    let file = lends_dir(root.path()).join("signed.json");
    for unknown in [
        r#"{"port": 8096, "kind": "k", "sign_in": 9000}"#,
        r#"{"port": 8096, "kind": "k", "sign_in_with_svrngs": {}, "bogus": true}"#,
    ] {
        std::fs::write(&file, unknown).unwrap();
        match read(root.path(), &name) {
            Err(LendError::Malformed { .. }) => {}
            other => panic!("a declaration holding an unknown field is read: {other:?}"),
        }
    }
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
        Upload(NonZeroU64::new(1_500_000).unwrap()).to_string(),
        "1500000 bit/s"
    );
}

/// A declaration written before bash-limit is read as it was — its limit
/// of three still lets in three, and refuses a fourth — and one that names
/// no limit, which was refused, now lets in everyone at once: the absence
/// held as itself, never as a number.
#[test]
fn a_limit_declared_is_kept_and_one_absent_is_none() {
    use crate::door::limit::{Counts, Verdict};
    use crate::network::Whereabouts;
    #[expect(
        clippy::disallowed_methods,
        reason = "the time is the counts' argument"
    )]
    let now = std::time::Instant::now();
    let root = tempfile::tempdir().unwrap();
    let name = LendName::parse("shelf").unwrap();
    std::fs::create_dir_all(lends_dir(root.path())).unwrap();
    let file = lends_dir(root.path()).join("shelf.json");
    let fourth = |text: &str| {
        std::fs::write(&file, text).unwrap();
        let lend = read(root.path(), &name).unwrap().unwrap();
        let mut counts = Counts::default();
        for who in ["ana", "bo", "cy"] {
            counts.ask(&lend, who, Whereabouts::Away, now);
        }
        counts.ask(&lend, "dee", Whereabouts::Away, now)
    };
    let before = "{\n  \"port\": 8096,\n  \"kind\": \"k\",\n  \"limit\": 3,\n  \"refused\": []\n}";
    assert!(matches!(fourth(before), Verdict::Full { using: 3, .. }));
    let unnamed = r#"{"port": 8096, "kind": "k", "refused": []}"#;
    assert!(matches!(fourth(unnamed), Verdict::In { .. }));
}

/// A sign-in client is made from the allowed characters, kept in the
/// declaration at 0600, refused on read when it is not, and never kept
/// beside a lend's own password (SIGN_IN_WITH_SVRNGS §5 "Turning it on").
#[cfg(unix)]
#[test]
fn a_sign_in_client_is_checked_written_owner_only_and_never_both() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let mut shelf = lend("shelf", 8096);
    let client = SignInWithSvrngs {
        client_id: "abc.DEF-1_~".into(),
        client_secret: Some("s3cret-._~".into()),
    };
    shelf.sign_in_with_svrngs = Some(client.clone());
    declare(root.path(), &shelf, None).unwrap();
    assert_eq!(read(root.path(), &shelf.name).unwrap(), Some(shelf.clone()));
    let file = lends_dir(root.path()).join("shelf.json");
    assert_eq!(
        std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o600,
        "the declaration holding a secret is owner-only"
    );

    std::fs::write(
        &file,
        r#"{"port":8096,"kind":"k","sign_in_with_svrngs":{"client_id":"a b","client_secret":"x"}}"#,
    )
    .unwrap();
    assert!(
        matches!(read(root.path(), &shelf.name), Err(LendError::NotAClient)),
        "a client outside the alphabet is refused, never repaired"
    );

    assert!(
        matches!(
            declare(root.path(), &shelf, Some(("authorization", "s3cret"))),
            Err(LendError::TwoSecrets)
        ),
        "a lend declares its own password or a sign-in, never both"
    );
}
