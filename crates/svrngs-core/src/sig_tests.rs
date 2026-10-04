// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/sig.rs@e1a43bcd2
use super::*;
use crate::tests_support::{admitted, after, record, signed, NS};

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

const BODY: &str = r#"{"op":"record"}"#;

/// The id of a fixed act in `house`: actor key(1), seq 1, ts 100.
fn id_with(prev: Option<&OpId>) -> OpId {
    act_id("house", &actor_of(&key(1)), 1, prev, 100, BODY)
}

#[test]
fn a_signature_verifies_under_the_key_that_made_it() {
    let k = key(1);
    let id = id_with(None);
    let sig = sign_ring_op(&k, &id);
    assert!(verify_ring_op(&actor_of(&k), &id, &sig));
}

/// svrngs (U2): an id is the whole BLAKE3 hash — 64 hex on the line and in
/// every rendering, never upstream's `ring-` prefix and 64 bits. The vector
/// is frozen from this implementation: a second one must reproduce it.
#[test]
fn an_id_is_64_hex() {
    let id = id_with(None);
    let shown = id.to_string();
    assert_eq!(shown.len(), 64, "{shown}");
    assert!(shown
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    assert_eq!(serde_json::to_string(&id).unwrap(), format!("\"{shown}\""));
    let line = serde_json::to_value(signed(&key(1), 100, 0, record("x"))).unwrap();
    assert_eq!(line["id"].as_str().map(str::len), Some(64), "{line}");
    assert_eq!(
        shown,
        "43a44de7ac4885e4cf4ced80c0e137bb1b3381562da7f65da2f7608ca4c5d013"
    );
}

/// Every field is in the id. Flip any one and the id moves — otherwise that
/// field is forgeable after the fact — and the signature over the old id
/// does not verify over the new one.
#[test]
fn every_field_is_in_the_id() {
    let k = key(1);
    let a = actor_of(&k);
    let base = id_with(None);
    let sig = sign_ring_op(&k, &base);
    let parent = act_id("house", &a, 0, None, 99, BODY);
    let other_parent = act_id("house", &a, 0, None, 98, BODY);
    let moved = [
        ("namespace", act_id("lending", &a, 1, None, 100, BODY)),
        (
            "actor",
            act_id("house", &actor_of(&key(2)), 1, None, 100, BODY),
        ),
        ("seq", act_id("house", &a, 2, None, 100, BODY)),
        ("prev named", id_with(Some(&parent))),
        ("ts_unix", act_id("house", &a, 1, None, 101, BODY)),
        (
            "body",
            act_id("house", &a, 1, None, 100, r#"{"op":"correct"}"#),
        ),
    ];
    for (field, id) in &moved {
        assert_ne!(id, &base, "{field}");
        assert!(!verify_ring_op(&a, id, &sig), "{field}");
    }
    assert_ne!(
        id_with(Some(&parent)),
        id_with(Some(&other_parent)),
        "which previous act"
    );
}

/// The reason for length prefixes, stated as a test: two different
/// (namespace, actor) splits of the same characters must not hash the same
/// bytes.
#[test]
fn field_boundaries_cannot_bleed() {
    assert_ne!(
        act_id("ab", "cd", 1, None, 1, "x"),
        act_id("a", "bcd", 1, None, 1, "x"),
    );
}

#[test]
fn malformed_input_is_false_never_a_panic_and_never_an_admit() {
    let k = key(1);
    let id = id_with(None);
    let sig = sign_ring_op(&k, &id);
    let a = actor_of(&k);
    assert!(!verify_ring_op("not-hex", &id, &sig));
    assert!(!verify_ring_op("ab", &id, &sig), "short key");
    assert!(!verify_ring_op(&a, &id, "not-hex"));
    assert!(!verify_ring_op(&a, &id, "abcd"), "short sig");
    assert!(!verify_ring_op(&a, &id, ""));
}

/// A ring-op signature must not be reusable as a dial-info signature.
/// The domain separator is the whole defence, so pin that it is present
/// and different.
#[test]
fn the_domain_separator_prefixes_the_message() {
    let msg = ring_op_message(&id_with(None));
    assert!(msg.starts_with(RING_OP_DOMAIN));
    assert_ne!(RING_OP_DOMAIN, b"cwth-dial-info-binding:");
}

/// svrngs (U2): the signature is over the id and outside it, so one act
/// carrying two valid signatures — Ed25519 allows any nonce — is one act:
/// one id, admitted once, no fork. Upstream's id hashed the line, signature
/// included, so the two lines were two acts at one seq, and both were
/// excluded.
#[test]
fn one_act_carrying_two_valid_signatures_has_one_id() {
    use ed25519_dalek::hazmat::{raw_sign, ExpandedSecretKey};
    let k = key(1);
    let first = signed(&k, 100, 0, record("x"));
    let one = after(&k, &first, 101, record("y"));
    let mut esk = ExpandedSecretKey::from(&k.to_bytes());
    esk.hash_prefix = [7; 32];
    let resigned =
        raw_sign::<ed25519_dalek::Sha512>(&esk, &ring_op_message(&one.id), &k.verifying_key());
    let mut two = one.clone();
    two.kind.sig = hex::encode(resigned.to_bytes());
    assert_ne!(one.kind.sig, two.kind.sig);
    assert!(verify_ring_op(&one.actor, &one.id, &two.kind.sig));
    assert_eq!(derive_id(&one, NS), derive_id(&two, NS));
    let a = admitted(&[first, one.clone(), two]);
    assert!(a.gaps().is_empty(), "{:?}", a.gaps());
    assert_eq!(a.ops().len(), 2);
    assert_eq!(a.ops()[1].id, one.id);
}
