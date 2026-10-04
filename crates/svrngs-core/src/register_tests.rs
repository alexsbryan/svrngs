// SPDX-License-Identifier: AGPL-3.0-or-later
//! Redeeming a link, given / when / then on the decider, and end to end
//! through `commit` against a store held in memory.

use crate::tests_support::*;
use crate::*;

fn issuer() -> SigningKey {
    key(1)
}
fn link() -> SigningKey {
    key(9)
}
fn joining() -> SigningKey {
    key(7)
}

fn invite_by(by: &SigningKey, names: &SigningKey, until: i64) -> Op<SignedOp> {
    signed(
        by,
        100,
        0,
        RailAct::Invite {
            key: actor_of(names),
            until,
        },
    )
}

fn redeem(invite: &Op<SignedOp>) -> Redeem {
    let id = derive_id(invite, NS);
    Redeem {
        invite: id.clone(),
        person: p("hal"),
        key: actor_of(&joining()),
        cosig: Redemption::sign(&joining(), NS, &id, &p("hal")).sig,
        link: actor_of(&link()),
        issuer: actor_of(&issuer()),
    }
}

fn author<'a>(signer: &'a SigningKey, roster: &'a Roster, now: i64) -> Author<'a> {
    Author {
        signer,
        roster,
        namespace: NS,
        verifier: &Ed25519Verifier,
        now,
    }
}

fn stands(store: &Memory, k: &SigningKey) -> bool {
    let held = store.0.borrow();
    admit(&held, &[], &ring(), NS, &Ed25519Verifier)
        .membership()
        .standing
        .contains(&actor_of(k))
}

#[test]
fn given_an_invite_a_redemption_writes_one_admit_and_the_key_stands() {
    let invite = invite_by(&issuer(), &link(), 10_000);
    let store = Memory::holding(&[invite.clone()]);
    let roster = ring();
    let recorded = commit(&store, &redeem(&invite), &author(&issuer(), &roster, 1_000))
        .unwrap()
        .unwrap();
    assert_eq!(recorded.lines().len(), 1);
    assert!(stands(&store, &joining()));
}

#[test]
fn redeeming_again_writes_nothing() {
    let invite = invite_by(&issuer(), &link(), 10_000);
    let store = Memory::holding(&[invite.clone()]);
    let roster = ring();
    let signer = issuer();
    let a = author(&signer, &roster, 1_000);
    commit(&store, &redeem(&invite), &a).unwrap().unwrap();
    let again = commit(&store, &redeem(&invite), &a).unwrap().unwrap();
    assert!(again.is_empty());
    assert_eq!(store.0.borrow().len(), 2);
}

#[test]
fn the_decider_refuses_what_the_invite_does_not_allow() {
    let invite = invite_by(&issuer(), &link(), 10_000);
    let none = admitted(&[]);
    assert_eq!(redeem(&invite).decide(&none), Err(RedeemRefusal::NoInvite));

    let theirs = invite_by(&key(2), &link(), 10_000);
    let mut asked = redeem(&theirs);
    asked.issuer = actor_of(&issuer());
    assert_eq!(
        asked.decide(&admitted(&[theirs])),
        Err(RedeemRefusal::NotOwn)
    );

    let elsewhere = invite_by(&issuer(), &key(5), 10_000);
    assert_eq!(
        redeem(&elsewhere).decide(&admitted(&[elsewhere.clone()])),
        Err(RedeemRefusal::OtherLink)
    );
}

#[test]
fn a_lapsed_invite_is_refused_by_the_register_and_nothing_is_written() {
    let invite = invite_by(&issuer(), &link(), 500);
    let store = Memory::holding(&[invite.clone()]);
    let roster = ring();
    let refused = commit(&store, &redeem(&invite), &author(&issuer(), &roster, 1_000)).unwrap();
    assert_eq!(
        refused,
        Err(Refused::Command(RedeemRefusal::Fault(InviteFault::Lapsed)))
    );
    assert_eq!(store.0.borrow().len(), 1);
    assert!(!stands(&store, &joining()));
}

#[test]
fn a_signer_without_standing_writes_nothing() {
    let invite = invite_by(&issuer(), &link(), 10_000);
    let store = Memory::holding(&[invite.clone()]);
    let roster = ring();
    let stranger = key(8);
    let refused = commit(&store, &redeem(&invite), &author(&stranger, &roster, 1_000)).unwrap();
    assert_eq!(refused, Err(Refused::NoStanding));
    assert_eq!(store.0.borrow().len(), 1);
}
