// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/attest.rs@e1a43bcd2
use super::*;
use crate::tests_support::{key, ring};

// `ring()` binds keys 1..; key 99 is claimed by nobody.
fn att(seed: u8) -> GuestAttestation {
    GuestAttestation::sign(&key(seed), "guest-ana", "house", 1_000)
}

#[test]
fn a_member_signed_attestation_round_trips_and_verifies() {
    let a = att(1);
    let wire: GuestAttestation = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
    assert_eq!(wire.verify(&ring(), "house", 999), Ok(()));
}

#[test]
fn a_flipped_byte_is_forged() {
    let mut a = att(1);
    a.name = "guest-anb".into();
    assert_eq!(a.verify(&ring(), "house", 999), Err(AttestRefusal::Forged));
    let mut b = att(1);
    let flipped = if b.sig.starts_with('0') { "1" } else { "0" };
    b.sig.replace_range(0..1, flipped);
    assert_eq!(b.verify(&ring(), "house", 999), Err(AttestRefusal::Forged));
}

#[test]
fn past_expiry_is_expired() {
    assert_eq!(
        att(1).verify(&ring(), "house", 1_000),
        Err(AttestRefusal::Expired)
    );
}

#[test]
fn another_namespace_is_wrong_namespace() {
    assert_eq!(
        att(1).verify(&ring(), "lending", 999),
        Err(AttestRefusal::WrongNamespace)
    );
}

#[test]
fn a_key_absent_from_the_roster_is_signer_not_in_roster() {
    assert_eq!(
        att(99).verify(&ring(), "house", 999),
        Err(AttestRefusal::SignerNotInRoster)
    );
}
