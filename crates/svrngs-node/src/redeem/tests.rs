// SPDX-License-Identifier: AGPL-3.0-or-later
//! The redemption's parsing and the ask's decision, judged on values alone:
//! no journal, no endpoint, no directory. The redemption's decider is
//! svrngs-core's `Redeem`, tested there. `three_people_open_one_link_and_each_is_let_in_once`
//! (`crates/svrngs/tests/join_by_link.rs`) drives the same checks through the
//! door over iroh.

use super::*;
use crate::{derive_id, sign_act, Op, RailAct, SignedOp};

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

    assert!(matches!(
        parse_redeem(None, &req),
        Err(NotAdmitted::NoDialer)
    ));
    let other = actor_of(&key(9));
    assert!(matches!(
        parse_redeem(Some(&other), &req),
        Err(NotAdmitted::NotTheDialer)
    ));
    let mut forged = req.clone();
    forged.sig = req.cosig.clone();
    assert!(matches!(
        parse_redeem(Some(&me), &forged),
        Err(NotAdmitted::LinkUnsigned)
    ));
    let mut unnamed = RedeemRequest::sign(&link, &joining, GROUP, &req.invite, &hal());
    unnamed.person = String::new();
    unnamed.sig = sign_message(&link, &unnamed.message());
    assert!(matches!(
        parse_redeem(Some(&me), &unnamed),
        Err(NotAdmitted::NotAName(_))
    ));
    assert_eq!(parse_redeem(Some(&me), &req).unwrap(), hal());
}

#[test]
fn an_ask_is_parsed_before_anything_is_read() {
    let link = key(2);
    let joining = key(3);
    let ask = AskRequest::sign(&link, &joining);
    assert!(parse_ask(Some(&actor_of(&joining)), &ask).is_ok());
    assert!(matches!(parse_ask(None, &ask), Err(NotAdmitted::NoDialer)));
    let mut forged = ask.clone();
    forged.link = actor_of(&key(7));
    assert!(matches!(
        parse_ask(Some(&actor_of(&joining)), &forged),
        Err(NotAdmitted::LinkUnsigned)
    ));
}
