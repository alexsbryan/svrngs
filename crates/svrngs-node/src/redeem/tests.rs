// SPDX-License-Identifier: AGPL-3.0-or-later
//! The redemption's decisions, judged on values alone: no journal, no
//! endpoint, no directory. `three_people_open_one_link_and_each_is_let_in_once`
//! (`crates/svrngs/tests/join_by_link.rs`) drives the same checks through the
//! door over iroh.

use super::*;
use crate::{sign_act, Op, SignedOp};

const GROUP: &str = "house";

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

/// `issuer`'s `Invite` naming `link`'s key, as its journal would hold it.
fn invite(issuer: &SigningKey, seq: u64, link: &SigningKey) -> Op<SignedOp> {
    let act = RailAct::Invite {
        key: actor_of(link),
        until: i64::MAX,
    };
    sign_act(
        issuer,
        GROUP,
        1_000 + seq as i64,
        seq,
        None,
        act,
        None,
        None,
    )
}

fn hal() -> Person {
    Person::parse("Hal").unwrap()
}

fn request(link: &SigningKey, joining: &SigningKey, invite: &Op<SignedOp>) -> RedeemRequest {
    RedeemRequest::sign(link, joining, GROUP, &derive_id(invite, GROUP), &hal())
}

#[test]
fn a_redemption_is_parsed_before_anything_is_read() {
    let (link, joining) = (key(2), key(3));
    let req = request(&link, &joining, &invite(&key(1), 0, &link));
    let me = actor_of(&joining);

    assert!(matches!(parse_redeem(None, &req), Err(Refused::NoDialer)));
    let other = actor_of(&key(9));
    assert!(matches!(
        parse_redeem(Some(&other), &req),
        Err(Refused::NotTheDialer)
    ));
    let mut forged = req.clone();
    forged.sig = req.cosig.clone();
    assert!(matches!(
        parse_redeem(Some(&me), &forged),
        Err(Refused::LinkUnsigned)
    ));
    let mut unnamed = RedeemRequest::sign(&link, &joining, GROUP, &req.invite, &hal());
    unnamed.person = String::new();
    unnamed.sig = sign_message(&link, &unnamed.message());
    assert!(matches!(
        parse_redeem(Some(&me), &unnamed),
        Err(Refused::NotAName(_))
    ));
    assert_eq!(parse_redeem(Some(&me), &req).unwrap(), hal());
}

#[test]
fn the_admit_is_decided_from_the_groups_acts_alone() {
    let (issuer, link, joining) = (key(1), key(2), key(3));
    let me = actor_of(&issuer);
    let held = invite(&issuer, 0, &link);
    let req = request(&link, &joining, &held);

    let act = decide_admit(std::slice::from_ref(&held), &me, &req, hal()).unwrap();
    assert_eq!(
        act,
        RailAct::Admit {
            person: hal(),
            key: actor_of(&joining),
            invite: Some(Redemption {
                id: req.invite.clone(),
                sig: req.cosig.clone(),
            }),
        }
    );

    assert!(matches!(
        decide_admit(&[], &me, &req, hal()),
        Err(Refused::NoInvite)
    ));
    let theirs = invite(&key(4), 0, &link);
    let req_theirs = request(&link, &joining, &theirs);
    assert!(matches!(
        decide_admit(&[theirs], &me, &req_theirs, hal()),
        Err(Refused::NotOwn)
    ));
    let elsewhere = invite(&issuer, 0, &key(5));
    let req_elsewhere = request(&link, &joining, &elsewhere);
    assert!(matches!(
        decide_admit(&[elsewhere], &me, &req_elsewhere, hal()),
        Err(Refused::OtherLink)
    ));
}

#[test]
fn an_ask_finds_this_nodes_newest_invite_naming_the_link() {
    let (issuer, link) = (key(1), key(2));
    let me = actor_of(&issuer);
    let ops = [
        invite(&issuer, 0, &link),
        invite(&key(4), 0, &link),
        invite(&issuer, 1, &link),
        invite(&issuer, 2, &key(5)),
    ];
    let found = own_invite(&ops, &me, &actor_of(&link)).unwrap();
    assert_eq!(found.id, ops[2].id);
    assert!(own_invite(&ops, &me, &actor_of(&key(6))).is_none());

    let joining = key(3);
    let ask = AskRequest::sign(&link, &joining);
    assert!(parse_ask(Some(&actor_of(&joining)), &ask).is_ok());
    assert!(matches!(parse_ask(None, &ask), Err(Refused::NoDialer)));
    let mut forged = ask.clone();
    forged.link = actor_of(&key(7));
    assert!(matches!(
        parse_ask(Some(&actor_of(&joining)), &forged),
        Err(Refused::LinkUnsigned)
    ));
}
