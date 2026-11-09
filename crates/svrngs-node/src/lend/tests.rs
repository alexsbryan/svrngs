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
        kind: Program::parse("k").unwrap(),
        audience: Audience::Selected(vec!["house".to_string()]),
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
    shelf.reads = vec![
        Program::parse("films").unwrap(),
        Program::parse("music").unwrap(),
    ];
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

/// A declaration that names no groups is legacy, needing selection, never
/// all groups; a selection is read back and reaches only its IDs. Planted:
/// a declaration without `groups` read as reaching every group.
#[test]
fn a_lend_with_no_selection_reaches_nothing_and_a_selection_reaches_its_ids() {
    let root = tempfile::tempdir().unwrap();
    let name = LendName::parse("jellyfin").unwrap();
    std::fs::create_dir_all(lends_dir(root.path())).unwrap();
    let file = lends_dir(root.path()).join("jellyfin.json");
    std::fs::write(&file, r#"{"port": 8096, "kind": "k", "refused": []}"#).unwrap();
    let legacy = read(root.path(), &name).unwrap().unwrap();
    assert_eq!(legacy.audience, Audience::Legacy);
    assert!(
        !legacy.audience.reaches("house") && !legacy.audience.reaches("trip"),
        "a legacy declaration reaches no group"
    );
    std::fs::write(
        &file,
        r#"{"port": 8096, "kind": "k", "refused": [], "groups": ["house"]}"#,
    )
    .unwrap();
    let scoped = read(root.path(), &name).unwrap().unwrap();
    assert_eq!(scoped.audience, Audience::Selected(vec!["house".into()]));
    assert!(scoped.audience.reaches("house"));
    assert!(!scoped.audience.reaches("trip"));
    assert_eq!(
        scoped.audience.ids(),
        Some(["house".to_string()].as_slice())
    );
}

/// An audience-only edit fixes the groups and keeps every other field —
/// the declared password, the sign-in client, the refused paths; a limit
/// edit through the same path keeps the audience, so a limit change never
/// restores a grant another narrowed (LENDING_SCOPE §"Updates"). Planted:
/// the limit edit rewriting the audience.
#[test]
fn an_audience_edit_keeps_the_rest_and_a_limit_edit_keeps_the_audience() {
    let root = tempfile::tempdir().unwrap();
    let mut jellyfin = lend("jellyfin", 8096);
    jellyfin.audience = Audience::Selected(vec!["house".into()]);
    jellyfin.upload = Some(Upload::from_mbit(NonZeroU32::new(30).unwrap()));
    declare(root.path(), &jellyfin, Some(("authorization", "s3cret"))).unwrap();
    let file = lends_dir(root.path()).join("jellyfin.json");

    set_audience(
        root.path(),
        &jellyfin.name,
        vec!["house".into(), "trip".into()],
    )
    .unwrap();
    let after = read(root.path(), &jellyfin.name).unwrap().unwrap();
    assert_eq!(
        after.audience,
        Audience::Selected(vec!["house".into(), "trip".into()])
    );
    assert_eq!(after.upload, jellyfin.upload, "the upload is kept");
    assert_eq!(after.kind, jellyfin.kind, "the kind is kept");
    assert_eq!(
        crate::declared::read_declared_in(&secret_dir(root.path(), &jellyfin.name)),
        vec![("authorization".to_string(), "s3cret".to_string())],
        "the declared password is kept"
    );
    let bytes_before = std::fs::read_to_string(&file).unwrap();
    assert!(bytes_before.contains("trip"), "the audience is written");

    limit(root.path(), &jellyfin.name, NonZeroU16::new(5)).unwrap();
    let after_limit = read(root.path(), &jellyfin.name).unwrap().unwrap();
    assert_eq!(
        after_limit.audience,
        Audience::Selected(vec!["house".into(), "trip".into()]),
        "a limit edit preserves the audience"
    );
    assert_eq!(after_limit.limit, NonZeroU16::new(5));
}

/// What a lend is lent to is disclosed by `listed`: a declared lend only
/// through a selected ID, a legacy one and a paused one never; a carried
/// app only in its own group. Planted: a legacy lend listed to everyone.
#[test]
fn a_lend_is_listed_only_through_a_group_it_is_lent_to() {
    let house = |g: &str| g == "house";
    let offered = |groups: Option<Vec<String>>, carried: Option<&str>| Offered {
        name: LendName::parse("jellyfin").unwrap(),
        kind: Program::parse("k").unwrap(),
        limit: None,
        using: 0,
        upload: None,
        reads: Vec::new(),
        asks: None,
        group: carried.map(str::to_string),
        groups,
        raw: None,
    };
    assert!(offered(Some(vec!["house".into()]), None).listed(house));
    assert!(!offered(Some(vec!["house".into()]), None).listed(|g| g == "trip"));
    assert!(
        !offered(None, None).listed(|_| true),
        "a legacy declaration is not listed even to a member of every group"
    );
    assert!(
        !offered(Some(Vec::new()), None).listed(|_| true),
        "a paused lend is not listed"
    );
    assert!(offered(Some(vec!["house".into()]), Some("house")).listed(house));
    assert!(!offered(Some(vec!["house".into()]), Some("house")).listed(|g| g == "trip"));
    // Listed, it tells only the groups the asker shares, never the rest of
    // its audience. Planted: the whole selection disclosed.
    let mut both = offered(Some(vec!["house".into(), "trip".into()]), None);
    assert!(both.listed(house));
    assert_eq!(both.groups, Some(vec!["house".into()]), "trip is not told");
}

/// origin/main's `LendFile` (crates/svrngs-node/src/lend.rs:282-306 at
/// d6d2a120), verbatim but its name: the parser of an svrngs before
/// selected groups, which a downgrade would run on these files.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct OldLendFile {
    port: NonZeroU16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    raw: Option<Raw>,
    kind: Program,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    limit: Option<NonZeroU16>,
    #[serde(default)]
    refused: Vec<String>,
    #[serde(
        default,
        rename = "upload_bps",
        skip_serializing_if = "Option::is_none"
    )]
    upload: Option<Upload>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    reads: Vec<Program>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sign_in_with_svrngs: Option<SignInWithSvrngs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    asks: Option<String>,
}

/// A legacy declaration — no `groups` key — on a computer in no group is
/// stamped `"groups": null` once, its limit and sign-in client kept, still
/// read as needing selection, and left byte for byte on a second start; the
/// old parser refuses the stamped file rather than lend it to every group.
/// In groups, the same file and a `null` are lent to those, once, and a
/// list, empty or not, is never rewritten. A stamp that cannot be written
/// leaves the file as it was, legacy and listed to nobody. Planted: no
/// stamp, the file left for an old binary to lend.
#[cfg(unix)]
#[test]
fn a_legacy_declaration_is_stamped_once_so_an_old_parser_refuses_it() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let lends = lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let client = r#"{"client_id":"svrngs-media","client_secret":"s3cret"}"#;
    let legacy =
        format!(r#"{{"port":8096,"kind":"media","limit":2,"sign_in_with_svrngs":{client}}}"#);
    std::fs::write(lends.join("media.json"), &legacy).unwrap();
    assert!(
        serde_json::from_str::<OldLendFile>(&legacy).is_ok(),
        "the old parser reads legacy"
    );
    stamp(dir.path(), &[]);
    let stamped = std::fs::read_to_string(lends.join("media.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stamped).unwrap();
    assert_eq!(json["groups"], serde_json::Value::Null, "{stamped}");
    assert!(
        json.as_object().unwrap().contains_key("groups"),
        "{stamped}"
    );
    assert!(
        serde_json::from_str::<OldLendFile>(&stamped).is_err(),
        "an old svrngs refuses it"
    );
    let name = LendName::parse("media").unwrap();
    let media = read(dir.path(), &name).unwrap().unwrap();
    assert_eq!(media.audience, Audience::Legacy);
    assert_eq!(
        (media.limit, media.sign_in_with_svrngs.is_some()),
        (NonZeroU16::new(2), true)
    );
    stamp(dir.path(), &[]);
    assert_eq!(
        std::fs::read_to_string(lends.join("media.json")).unwrap(),
        stamped,
        "once"
    );

    std::fs::write(lends.join("pad.json"), r#"{"port":9,"kind":"pad"}"#).unwrap();
    std::fs::set_permissions(&lends, std::fs::Permissions::from_mode(0o555)).unwrap();
    stamp(dir.path(), &[]);
    std::fs::set_permissions(&lends, std::fs::Permissions::from_mode(0o755)).unwrap();
    let pad = std::fs::read_to_string(lends.join("pad.json")).unwrap();
    assert_eq!(
        pad, r#"{"port":9,"kind":"pad"}"#,
        "an unwritable stamp leaves it"
    );
    let pad = read(dir.path(), &LendName::parse("pad").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(pad.audience, Audience::Legacy, "and it reaches nobody");

    let house = vec!["house".to_string()];
    stamp(dir.path(), &house);
    let audience = |name: &str| {
        let name = LendName::parse(name).unwrap();
        read(dir.path(), &name).unwrap().unwrap().audience
    };
    assert_eq!(audience("media"), Audience::Selected(house.clone()));
    assert_eq!(audience("pad"), Audience::Selected(house.clone()));
    let kept = read(dir.path(), &LendName::parse("media").unwrap()).unwrap();
    assert_eq!(kept.unwrap().limit, NonZeroU16::new(2), "the rest kept");
    let later = vec!["house".to_string(), "later".to_string()];
    stamp(dir.path(), &later);
    assert_eq!(audience("media"), Audience::Selected(house), "a snapshot");
    std::fs::write(
        lends.join("still.json"),
        r#"{"port":9,"kind":"pad","groups":[]}"#,
    )
    .unwrap();
    stamp(dir.path(), &later);
    assert_eq!(
        audience("still"),
        Audience::Selected(Vec::new()),
        "paused stays paused"
    );
}

/// A withdrawal is serialized with every other change to the lend: one
/// asked while a change holds its lock is busy, so the change's write can
/// never bring back a lend just withdrawn. Planted: `withdraw` outside the
/// lock, which removed the file mid-change and let the change write it back.
#[test]
fn a_withdrawal_while_a_change_holds_the_lend_is_busy_never_undone() {
    let root = tempfile::tempdir().unwrap();
    let shelf = lend("shelf", 9);
    declare(root.path(), &shelf, None).unwrap();
    let mut during = None;
    let changed = mutate(root.path(), &shelf.name, |l| {
        during = Some(withdraw(root.path(), &shelf.name));
        l.limit = NonZeroU16::new(5);
    });
    assert!(changed.unwrap(), "the change is written");
    let said = during.unwrap();
    let back = read(root.path(), &shelf.name).unwrap().is_some();
    assert!(
        !(matches!(said, Ok(true)) && back),
        "withdraw said it withdrew, and the change wrote the lend back"
    );
    assert!(matches!(said, Err(LendError::Busy { .. })), "{said:?}");
    assert!(
        withdraw(root.path(), &shelf.name).unwrap(),
        "taken once let go"
    );
    assert_eq!(
        read(root.path(), &shelf.name).unwrap(),
        None,
        "and it stays withdrawn"
    );
}

/// A re-declaration is serialized with every other change to the lend
/// (audit F4): a `share --group` narrowing asked while a limit change holds
/// the lock is busy, so the change's stale write can never undo it, and
/// once let go the narrowing is taken and kept. Planted: `declare` outside
/// the lock, which the change's write then undid.
#[test]
fn a_redeclaration_while_a_change_holds_the_lend_is_busy_never_undone() {
    let root = tempfile::tempdir().unwrap();
    let both = Audience::Selected(vec!["house".into(), "trip".into()]);
    let shelf = Lend {
        audience: both,
        ..lend("shelf", 9)
    };
    declare(root.path(), &shelf, None).unwrap();
    let narrowed = Lend {
        audience: Audience::Selected(vec!["house".into()]),
        ..shelf.clone()
    };
    let mut during = None;
    let changed = mutate(root.path(), &shelf.name, |l| {
        during = Some(declare(root.path(), &narrowed, None));
        l.limit = NonZeroU16::new(5);
    });
    assert!(changed.unwrap(), "the change is written");
    let said = during.unwrap();
    let now = read(root.path(), &shelf.name).unwrap().unwrap().audience;
    assert!(
        !(said.is_ok() && now != narrowed.audience),
        "declare said it narrowed, and the change wrote the old audience back"
    );
    assert!(matches!(said, Err(LendError::Busy { .. })), "{said:?}");
    declare(root.path(), &narrowed, None).unwrap();
    let kept = read(root.path(), &shelf.name).unwrap().unwrap().audience;
    assert_eq!(kept, narrowed.audience, "taken once let go");
}
