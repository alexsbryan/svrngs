// SPDX-License-Identifier: AGPL-3.0-or-later
//! The link in the record (svrngs, U2; DESIGN §6 "A link admits each person
//! once"): the line versions an `Invite` and the `Admit`s that redeem it
//! ship at. Whether they count is the interpreter's
//! (`tests_source_graph.rs`).
use super::*;
use crate::actor_of;
use crate::tests_support::{after, key, signed, NS};

/// The link's key: the `Invite` names its public half.
const LINK: u8 = 9;

fn invite(until: i64) -> RailAct {
    RailAct::Invite {
        key: actor_of(&key(LINK)),
        until,
        device_of: None,
    }
}

/// `joining` redeems `inv` as `person`, its new key held by `device`.
fn redeem_by(inv: &Op<SignedOp>, person: &str, joining: u8, device: Device) -> RailAct {
    let person = Person::parse(person).unwrap();
    RailAct::Admit {
        invite: Some(crate::Redemption::sign(
            &key(joining),
            NS,
            &inv.id,
            &person,
            device,
        )),
        person,
        key: actor_of(&key(joining)),
    }
}

fn redeem(inv: &Op<SignedOp>, person: &str, joining: u8) -> RailAct {
    redeem_by(inv, person, joining, Device::Computer)
}

fn redeem_browser(inv: &Op<SignedOp>, person: &str, joining: u8) -> RailAct {
    redeem_by(inv, person, joining, Device::Browser)
}

/// The link's two kinds ship at their own line version; an `Admit` naming
/// no `Invite` keeps the membership cohort's; a browser's redemption ships
/// at its own (u4-1), and a computer's keeps the link's.
#[test]
fn the_link_ships_at_its_own_line_version() {
    let inv = signed(&key(1), 100, 0, invite(200));
    let dee = after(&key(1), &inv, 101, redeem(&inv, "dee", 4));
    let phone = after(&key(1), &dee, 103, redeem_browser(&inv, "fay", 6));
    assert_eq!(phone.v, crate::BROWSER_LINE_VERSION);
    let plain = signed(
        &key(1),
        102,
        0,
        RailAct::Admit {
            person: Person::parse("eve").unwrap(),
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
