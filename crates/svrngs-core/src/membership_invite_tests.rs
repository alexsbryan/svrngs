// SPDX-License-Identifier: AGPL-3.0-or-later
//! The link in the record (svrngs, U2; DESIGN §6 "A link admits each person
//! once"): an `Invite`, and the `Admit`s that redeem it, judged by
//! `membership` and named by `admit`.
use super::*;
use crate::tests_support::{after, key, ring, signed, NS};
use crate::{actor_of, admit, Admission, Ed25519Verifier, RailGap};

/// The link's key: the `Invite` names its public half.
const LINK: u8 = 9;
/// The key of the issuer's second link.
const LINK_TWO: u8 = 10;

fn invite(until: i64) -> RailAct {
    invite_for(LINK, until)
}

fn invite_for(link: u8, until: i64) -> RailAct {
    RailAct::Invite {
        key: actor_of(&key(link)),
        until,
    }
}

/// `joining` redeems `inv` as `person`, consenting with `consents`' key.
fn redeem_as(inv: &Op<SignedOp>, person: &str, joining: u8, consents: u8) -> RailAct {
    let person = Person::from(person);
    RailAct::Admit {
        invite: Some(Redemption::sign(&key(consents), NS, &inv.id, &person)),
        person,
        key: actor_of(&key(joining)),
    }
}

fn redeem(inv: &Op<SignedOp>, person: &str, joining: u8) -> RailAct {
    redeem_as(inv, person, joining, joining)
}

fn admitted(ops: &[Op<SignedOp>]) -> Admission {
    admit(ops, &[], &ring(), NS, &Ed25519Verifier)
}

fn standing(a: &Admission, k: u8) -> bool {
    a.membership
        .as_ref()
        .unwrap()
        .standing
        .contains(&actor_of(&key(k)))
}

/// The one gap `a` names, which must refuse `refused` for `fault`, and the
/// joining keys that must not stand.
fn refused_for(a: &Admission, refused: &Op<SignedOp>, fault: InviteFault, keys: &[u8]) {
    assert_eq!(
        a.gaps,
        vec![RailGap::RefusedRedemption {
            id: refused.id.clone(),
            actor: refused.actor.clone(),
            fault,
        }],
        "the redemption is refused by name, and nothing else is"
    );
    for k in keys {
        assert!(!standing(a, *k), "key {k} gains no standing");
    }
}

/// One link, three people: each redeeming `Admit` under one `Invite` gives
/// its key standing, and the newcomer's own act counts.
#[test]
fn three_redemptions_under_one_invite_each_give_their_key_standing() {
    let inv = signed(&key(1), 100, 0, invite(200));
    let dee = after(&key(1), &inv, 101, redeem(&inv, "dee", 4));
    let eve = after(&key(1), &dee, 102, redeem(&inv, "eve", 5));
    let fay = after(&key(1), &eve, 103, redeem(&inv, "fay", 6));
    let dee_writes = signed(&key(4), 104, 0, crate::tests_support::record("hi"));
    let a = admitted(&[inv, dee, eve, fay, dee_writes.clone()]);
    assert_eq!(a.gaps, vec![], "nothing is refused");
    for k in [4, 5, 6] {
        assert!(standing(&a, k), "key {k} stands");
    }
    assert!(a.membership.as_ref().unwrap().counts(&dee_writes.id));
}

/// An `Admit` timed at the `Invite`'s `until` is lapsed. Watched failing
/// with the lapse unchecked.
#[test]
fn a_redemption_under_a_lapsed_invite_is_refused() {
    let inv = signed(&key(1), 100, 0, invite(102));
    let dee = after(&key(1), &inv, 102, redeem(&inv, "dee", 4));
    let a = admitted(&[inv, dee.clone()]);
    refused_for(&a, &dee, InviteFault::Lapsed, &[4]);
}

/// A `Correct` of the `Invite` voids the link, and is not how it closes
/// (u2-4b). Watched failing with the void ignored.
#[test]
fn a_redemption_under_a_voided_invite_is_refused() {
    let inv = signed(&key(1), 100, 0, invite(200));
    let close = after(
        &key(1),
        &inv,
        101,
        RailAct::Correct {
            corrects: inv.id.clone(),
            replacement: None,
        },
    );
    let dee = after(&key(1), &close, 102, redeem(&inv, "dee", 4));
    let a = admitted(&[inv, close, dee.clone()]);
    refused_for(&a, &dee, InviteFault::Voided, &[4]);
}

/// Only the `Invite`'s signer redeems it: bo stands, but alex made the
/// link. Watched failing with the signer unchecked.
#[test]
fn a_redemption_by_someone_who_did_not_sign_the_invite_is_refused() {
    let inv = signed(&key(1), 100, 0, invite(200));
    let dee = signed(&key(2), 101, 0, redeem(&inv, "dee", 4));
    let a = admitted(&[inv, dee.clone()]);
    refused_for(&a, &dee, InviteFault::NotIssuer, &[4]);
}

/// The admitted key must consent: a co-signature by another key admits
/// neither. Watched failing with the co-signature unchecked.
#[test]
fn a_redemption_co_signed_by_another_key_is_refused() {
    let inv = signed(&key(1), 100, 0, invite(200));
    let dee = after(&key(1), &inv, 101, redeem_as(&inv, "dee", 4, 7));
    let a = admitted(&[inv, dee.clone()]);
    refused_for(&a, &dee, InviteFault::BadCosignature, &[4, 7]);
}

/// A refused redemption binds nothing, so its key's `Correct` voids
/// nothing. Watched failing with the void layer binding every `Admit`.
#[test]
fn a_refused_redemptions_key_voids_nothing() {
    let inv = signed(&key(1), 100, 0, invite(102));
    let dee = after(&key(1), &inv, 102, redeem(&inv, "dee", 4));
    let bo_writes = signed(&key(2), 103, 0, crate::tests_support::record("x"));
    let dee_voids = signed(
        &key(4),
        104,
        0,
        RailAct::Correct {
            corrects: bo_writes.id.clone(),
            replacement: None,
        },
    );
    let m = membership(
        [&inv, &dee, &bo_writes, &dee_voids]
            .into_iter()
            .map(|op| (&op.id, op)),
        &ring(),
        NS,
    );
    assert!(m.voided.is_empty(), "dee's void lands nowhere: {m:?}");
    assert!(m.counts(&bo_writes.id));
}

/// The link's two kinds ship at their own line version; an `Admit` naming
/// no `Invite` keeps the membership cohort's.
#[test]
fn the_link_ships_at_its_own_line_version() {
    let inv = signed(&key(1), 100, 0, invite(200));
    let dee = after(&key(1), &inv, 101, redeem(&inv, "dee", 4));
    let plain = signed(
        &key(1),
        102,
        0,
        RailAct::Admit {
            person: Person::from("eve"),
            key: actor_of(&key(5)),
            invite: None,
        },
    );
    assert_eq!(
        (inv.v, dee.v, plain.v),
        (
            crate::LINK_LINE_VERSION,
            crate::LINK_LINE_VERSION,
            crate::MEMBERSHIP_LINE_VERSION
        )
    );
}

/// A new link closes the old one from its place in the order (u2-4b, DESIGN
/// §8): dee, eve and fay redeem link one; alex appends link two; gus, under
/// link one after it, is refused `Closed` by link two, while the three keep
/// standing and their own acts count, and hal under link two stands.
/// Watched failing with the older `Invite` voided (the three lose standing)
/// and with closing ignored (gus stands).
#[test]
fn a_newer_link_closes_the_older_and_keeps_whom_it_admitted() {
    let one = signed(&key(1), 100, 0, invite(300));
    let dee = after(&key(1), &one, 101, redeem(&one, "dee", 4));
    let eve = after(&key(1), &dee, 102, redeem(&one, "eve", 5));
    let fay = after(&key(1), &eve, 103, redeem(&one, "fay", 6));
    let two = after(&key(1), &fay, 105, invite_for(LINK_TWO, 300));
    let hal = after(&key(1), &two, 106, redeem(&two, "hal", 8));
    // gus last in alex's chain: a refused act is not admitted, so one
    // mid-chain is also a `SequenceHole` (admit's audit), not this test's.
    let gus = after(&key(1), &hal, 107, redeem(&one, "gus", 7));
    let mut writes = Vec::new();
    for k in [4, 5, 6] {
        writes.push(signed(&key(k), 108, 0, crate::tests_support::record("hi")));
    }
    let mut ops = vec![one, dee, eve, fay, two.clone(), gus.clone(), hal];
    ops.extend(writes.iter().cloned());
    let a = admitted(&ops);
    refused_for(&a, &gus, InviteFault::Closed { by: two.id }, &[7]);
    for k in [4, 5, 6, 8] {
        assert!(standing(&a, k), "key {k} stands");
    }
    let m = a.membership.as_ref().unwrap();
    for w in &writes {
        assert!(m.counts(&w.id), "{} still counts", w.actor);
    }
}

/// A redemption under link one timed before link two counts wherever it
/// arrives: a copy that holds link two first does not count gus, and once
/// gus's `Admit` arrives, last, gus stands. Watched failing with the older
/// `Invite` voided.
#[test]
fn a_redemption_timed_before_the_newer_link_counts_when_it_arrives_late() {
    let one = signed(&key(1), 100, 0, invite(300));
    let gus = after(&key(1), &one, 104, redeem(&one, "gus", 7));
    let two = after(&key(1), &gus, 105, invite_for(LINK_TWO, 300));
    let early = admitted(&[one.clone(), two.clone()]);
    assert!(!standing(&early, 7), "gus is not yet held");
    assert!(early.membership.as_ref().unwrap().counts(&two.id));
    let late = admitted(&[one, two, gus]);
    assert_eq!(late.gaps, vec![], "nothing is refused");
    assert!(standing(&late, 7), "gus stands");
}
