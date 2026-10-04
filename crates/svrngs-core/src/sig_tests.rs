// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/sig.rs@e1a43bcd2
use super::*;

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

#[test]
fn a_signature_verifies_under_the_key_that_made_it() {
    let k = key(1);
    let sig = sign_ring_op(&k, "house", 100, 1, r#"{"op":"record"}"#);
    assert!(verify_ring_op(
        &actor_of(&k),
        "house",
        100,
        1,
        r#"{"op":"record"}"#,
        &sig
    ));
}

/// Every field is bound. Flip any one and the signature must fail —
/// otherwise that field is forgeable after the fact.
#[test]
fn every_signed_field_is_actually_bound() {
    let k = key(1);
    let a = actor_of(&k);
    let body = r#"{"op":"record"}"#;
    let sig = sign_ring_op(&k, "house", 100, 1, body);
    assert!(
        !verify_ring_op(&a, "lending", 100, 1, body, &sig),
        "namespace"
    );
    assert!(!verify_ring_op(&a, "house", 101, 1, body, &sig), "ts_unix");
    assert!(!verify_ring_op(&a, "house", 100, 2, body, &sig), "seq");
    assert!(
        !verify_ring_op(&a, "house", 100, 1, r#"{"op":"correct"}"#, &sig),
        "body"
    );
    assert!(
        !verify_ring_op(&actor_of(&key(2)), "house", 100, 1, body, &sig),
        "actor"
    );
}

/// The reason for length prefixes, stated as a test: two different
/// (namespace, actor) splits of the same characters must not sign the
/// same bytes.
#[test]
fn field_boundaries_cannot_bleed() {
    assert_ne!(
        ring_op_message("ab", 1, "cd", 1, "x"),
        ring_op_message("a", 1, "bcd", 1, "x"),
    );
}

#[test]
fn malformed_input_is_false_never_a_panic_and_never_an_admit() {
    let k = key(1);
    let sig = sign_ring_op(&k, "house", 100, 1, "{}");
    let a = actor_of(&k);
    assert!(!verify_ring_op("not-hex", "house", 100, 1, "{}", &sig));
    assert!(
        !verify_ring_op("ab", "house", 100, 1, "{}", &sig),
        "short key"
    );
    assert!(!verify_ring_op(&a, "house", 100, 1, "{}", "not-hex"));
    assert!(
        !verify_ring_op(&a, "house", 100, 1, "{}", "abcd"),
        "short sig"
    );
    assert!(!verify_ring_op(&a, "house", 100, 1, "{}", ""));
}

/// A ring-op signature must not be reusable as a dial-info signature.
/// The domain separator is the whole defence, so pin that it is present
/// and different.
#[test]
fn the_domain_separator_prefixes_the_message() {
    let msg = ring_op_message("house", 1, "aa", 1, "{}");
    assert!(msg.starts_with(RING_OP_DOMAIN));
    assert_ne!(RING_OP_DOMAIN, b"cwth-dial-info-binding:");
}
