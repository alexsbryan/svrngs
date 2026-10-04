// SPDX-License-Identifier: AGPL-3.0-or-later
//! The link in the record (svrngs, U2; DESIGN §6 "A link admits each person
//! once"): an `Invite`, and the `Admit`s that redeem it, judged by
//! `membership` and named by `admit`.
use super::*;
use crate::tests_support::{after, key, ring, signed, NS};
use crate::{actor_of, admit, Admission, Ed25519Verifier, RailGap};

/// The link's key: the `Invite` names its public half.
const LINK: u8 = 9;

fn invite(until: i64) -> RailAct {
    RailAct::Invite {
        key: actor_of(&key(LINK)),
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

/// A `Correct` of the `Invite` closes the link. Watched failing with the
/// void ignored.
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
    refused_for(&a, &dee, InviteFault::Closed, &[4]);
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
