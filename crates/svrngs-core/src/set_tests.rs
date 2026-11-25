// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::tests_support::key;
use crate::{actor_of, verify_hex};

/// Whether `sig` is the set's device's over it and its `changes`' hash, as
/// a copy checks a set it takes (svrngs-db `sync.rs`, `signed`).
fn checks(at: &At<'_>, changes: &[u8; 32], sig: &str) -> bool {
    verify_hex(at.device, &at.message(changes), sig)
}

/// A set's signature checks for that set alone: another group, app,
/// device, database, seq, schema or change fails it. Plant: the message
/// leaving out the nonce (a set lifted into another database of the same
/// device checks).
#[test]
fn a_set_s_signature_checks_for_that_set_alone() {
    let (k, other) = (key(1), key(2));
    let (device, someone) = (actor_of(&k), actor_of(&other));
    let (nonce, schema) = (
        [7; 16],
        self::schema("CREATE TABLE t (id TEXT PRIMARY KEY);"),
    );
    let at = At {
        group: "house",
        app: "magnets",
        device: &device,
        nonce: &nonce,
        seq: 3,
        schema: &schema,
    };
    let sig = at.sign(&k, &changes(b"rows"));
    assert!(checks(&at, &changes(b"rows"), &sig));
    assert!(!checks(&at, &changes(b"rowz"), &sig), "another change");
    let (n2, s2) = (
        [8; 16],
        self::schema("CREATE TABLE u (id TEXT PRIMARY KEY);"),
    );
    let lifted = [
        At { group: "lab", ..at },
        At { app: "notes", ..at },
        At {
            device: &someone,
            ..at
        },
        At { nonce: &n2, ..at },
        At { seq: 4, ..at },
        At { schema: &s2, ..at },
    ];
    for at in lifted {
        assert!(!checks(&at, &changes(b"rows"), &sig), "{at:?}");
    }
    assert!(
        !checks(&at, &changes(b"rows"), &at.sign(&other, &changes(b"rows"))),
        "another key's"
    );
}

/// A site is its database's: one device's two databases, two devices', and
/// two apps' or groups' with one nonce, each have their own, and each is
/// derived alike everywhere. Plant: the site of the device alone (a
/// database made again under its key keeps its old site).
#[test]
fn a_site_is_its_database_s_own() {
    let (a, b) = (actor_of(&key(1)), actor_of(&key(2)));
    let s = site("house", "magnets", &a, &[1; 16]);
    assert_eq!(s, site("house", "magnets", &a, &[1; 16]), "derived alike");
    for other in [
        site("house", "magnets", &a, &[2; 16]),
        site("house", "magnets", &b, &[1; 16]),
        site("house", "notes", &a, &[1; 16]),
        site("lab", "magnets", &a, &[1; 16]),
    ] {
        assert_ne!(s, other);
    }
}

/// A schema's hash is its text's: another text, another hash.
#[test]
fn a_schema_s_hash_is_its_text_s() {
    assert_eq!(schema("a"), schema("a"));
    assert_ne!(schema("a"), schema("a "));
}
