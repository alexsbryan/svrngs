// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai kernel-types/src/hash.rs@e1a43bcd2
use super::*;

#[test]
fn hex_round_trips() {
    let h = ContentHash::of_str("the quick brown fox");
    assert_eq!(ContentHash::from_hex(&h.to_hex()), Some(h));
}

#[test]
fn to_hex_is_64_lowercase_hex() {
    let h = ContentHash::of_str("x");
    assert_eq!(h.to_hex().len(), 64);
    assert!(h
        .to_hex()
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
}

#[test]
fn matches_the_encoding_corpus_engine_already_writes() {
    // corpus-engine/src/engine/mod.rs:2732 is
    //   `blake3::hash(s.as_bytes()).to_hex().to_string()`
    // Pinning it here means adopting ContentHash on an existing
    // `content_hash: String` column is a type change, not a data
    // migration. If this test ever fails, stored rows have been orphaned.
    let expected = blake3::hash(b"hello").to_hex().to_string();
    assert_eq!(ContentHash::of(b"hello").to_hex(), expected);
}

#[test]
fn short_is_the_16_char_prefix_three_modules_hand_rolled() {
    // enrichment/atlas/atoms.rs, enrichment/governance.rs and
    // enrichment/code_intel/mod.rs each computed `full[..16]`.
    let h = ContentHash::of_str("atom");
    assert_eq!(h.short().len(), 16);
    assert_eq!(h.short(), h.to_hex()[..16]);
}

#[test]
fn a_truncated_hash_is_refused_not_zero_extended() {
    let h = ContentHash::of_str("anything");
    assert_eq!(ContentHash::from_hex(&h.short()), None);
}

#[test]
fn a_sha256_hex_string_is_not_silently_adopted_as_blake3() {
    // Same width, different value space. from_hex CANNOT detect this —
    // which is exactly why `of()` is the only way to mint one from
    // content, and why the doc forbids converting a SHA-256 digest.
    // What this test pins is the weaker, checkable claim: BLAKE3 and
    // SHA-256 of the same input do not collide, so a mixed store is
    // detectable as garbage rather than as agreement.
    let blake = ContentHash::of(b"hello").to_hex();
    let sha256_of_hello = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
    assert_ne!(blake, sha256_of_hello);
}

#[test]
fn serde_wire_form_is_a_plain_hex_string() {
    // Upstream asserts this through kernel-types' `wire` module, which
    // the fork does not carry; the claim is the same, asserted directly.
    let h = ContentHash::of_str("wire");
    let json = serde_json::to_string(&h).unwrap();
    assert_eq!(json, format!("\"{}\"", h.to_hex()));
    assert_eq!(serde_json::from_str::<ContentHash>(&json).unwrap(), h);
}

#[test]
fn deserializing_a_non_hash_string_errors_rather_than_defaulting() {
    assert!(serde_json::from_str::<ContentHash>("\"not-a-hash\"").is_err());
}
