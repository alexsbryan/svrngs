// SPDX-License-Identifier: AGPL-3.0-or-later
//! The link in the record (svrngs, U2; DESIGN §6 "A link admits each person
//! once"): an `Invite`, and the `Admit`s that redeem it, judged by
//! `membership` and named by `admit`.
use super::*;
use crate::tests_support::{after, key, ring, signed, standing, NS};
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
    redeem_by(inv, person, joining, consents, Device::Computer)
}

/// [`redeem_as`], the new key held by `device`.
fn redeem_by(
    inv: &Op<SignedOp>,
    person: &str,
    joining: u8,
    consents: u8,
    device: Device,
) -> RailAct {
    let person = Person::parse(person).unwrap();
    RailAct::Admit {
        invite: Some(Redemption::sign(
            &key(consents),
            NS,
            &inv.id,
            &person,
            device,
        )),
        person,
        key: actor_of(&key(joining)),
    }
}

/// `joining` redeems `inv` as `person` from a browser.
fn redeem_browser(inv: &Op<SignedOp>, person: &str, joining: u8) -> RailAct {
    redeem_by(inv, person, joining, joining, Device::Browser)
}

fn redeem(inv: &Op<SignedOp>, person: &str, joining: u8) -> RailAct {
    redeem_as(inv, person, joining, joining)
}

fn admitted(ops: &[Op<SignedOp>]) -> Admission {
    admit(&crate::ActSet::of(ops), &[], &ring(), NS, &Ed25519Verifier)
}

/// The one gap `a` names, which must refuse `refused` for `fault`, and the
/// joining keys that must not stand.
fn refused_for(a: &Admission, refused: &Op<SignedOp>, fault: InviteFault, keys: &[u8]) {
    assert_eq!(
        a.gaps(),
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
    assert_eq!(a.gaps(), vec![], "nothing is refused");
    for k in [4, 5, 6] {
        assert!(standing(&a, k), "key {k} stands");
    }
    assert!(a.membership().counts(&dee_writes.id));
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

/// A `Correct` of the `Invite` voided the link until U7, unseating every
/// key it admitted: a removal no keeper sequenced. Since U7 a `Correct` of
/// an act that moves membership voids nothing, so the redemption counts
/// (DESIGN §6, "Removal has one decider"); a link closes by its issuer's
/// next `Invite`. The name is the rule before U7, kept so u7-removal's
/// check follows every test by name.
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
    let a = admitted(&[inv, close, dee]);
    assert_eq!(a.gaps(), vec![], "nothing is refused");
    assert!(standing(&a, 4), "dee stands");
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
    let m = a.membership();
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
    assert!(early.membership().counts(&two.id));
    let late = admitted(&[one, two, gus]);
    assert_eq!(late.gaps(), vec![], "nothing is refused");
    assert!(standing(&late, 7), "gus stands");
}

/// The generator for `fixtures/redemption_before_browsers.jsonl`: alex's
/// `Invite`, and dee's `Admit` redeeming it, as a journal held them at line
/// version 3, before a redemption named a browser (u4-1). Kept so the
/// fixture is reproducible.
#[test]
#[ignore = "fixture generator — run with --ignored --nocapture"]
fn emit_redemption_fixture() {
    let inv = signed(&key(1), 1_700_000_000, 0, invite(1_800_000_000));
    let dee = after(&key(1), &inv, 1_700_000_001, redeem(&inv, "dee", 4));
    for op in [inv, dee] {
        println!("{}", serde_json::to_string(&op).unwrap());
    }
}

/// The fixture's lines, as a journal at line version 3 held them.
fn before_browsers() -> Vec<Op<SignedOp>> {
    include_str!("fixtures/redemption_before_browsers.jsonl")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("the fixture is a journal line"))
        .collect()
}

/// What holds each standing key (u4-1; DESIGN §3 "Joining", call 2), given
/// / when / then: given the acts, when the register folds them, then it
/// answers this device for this key, and none for a key that does not
/// stand. Watched failing with a line that names no device read as a
/// browser's (the earlier version's row, and the seed's and plain `Admit`'s
/// beside it, answered `Browser`).
#[test]
fn the_register_answers_what_holds_each_standing_key() {
    let inv = signed(&key(1), 100, 0, invite(300));
    let dee = after(&key(1), &inv, 101, redeem(&inv, "dee", 4));
    let eve = after(&key(1), &dee, 102, redeem_browser(&inv, "eve", 5));
    let fay = after(
        &key(1),
        &eve,
        103,
        RailAct::Admit {
            person: Person::parse("fay").unwrap(),
            key: actor_of(&key(6)),
            invite: None,
        },
    );
    let gus = after(&key(1), &fay, 104, redeem_browser(&inv, "gus", 7));
    let gone = after(
        &key(1),
        &gus,
        105,
        RailAct::Remove {
            key: actor_of(&key(7)),
            through: Vec::new(),
        },
    );
    let now = admitted(&[inv, dee, eve, fay, gus, gone]);
    assert_eq!(now.gaps(), vec![], "control: every act counts");
    let old = before_browsers();
    assert!(
        old.iter().all(|o| o.v == crate::LINK_LINE_VERSION),
        "control: the fixture is version 3's"
    );
    let then = admitted(&old);
    assert_eq!(then.gaps(), vec![], "a version 3 line verifies unchanged");

    let rows: [(&str, &Admission, u8, Option<Device>); 7] = [
        ("the seed's key", &now, 1, Some(Device::Computer)),
        ("a computer's redemption", &now, 4, Some(Device::Computer)),
        ("a browser's redemption", &now, 5, Some(Device::Browser)),
        ("an Admit naming no Invite", &now, 6, Some(Device::Computer)),
        ("a browser's key removed", &now, 7, None),
        ("a key never admitted", &now, 8, None),
        (
            "an earlier version's redemption",
            &then,
            4,
            Some(Device::Computer),
        ),
    ];
    for (given, admission, k, then) in rows {
        assert_eq!(
            admission.project::<Membership>().device(&actor_of(&key(k))),
            then,
            "given {given}"
        );
    }
}

/// The property behind every fold, over what holds each key: every arrival
/// order of six acts — a computer's redemption, two browsers', one of them
/// re-admitted after as a computer, and a removal — and every act delivered
/// twice fold to the same register, devices included.
#[test]
fn what_holds_each_key_is_a_function_of_the_set_of_acts() {
    let inv = signed(&key(1), 100, 0, invite(300));
    let dee = after(&key(1), &inv, 101, redeem(&inv, "dee", 4));
    let eve = after(&key(1), &dee, 102, redeem_browser(&inv, "eve", 5));
    let fay = after(&key(1), &eve, 103, redeem_browser(&inv, "fay", 6));
    let fay_again = after(
        &key(1),
        &fay,
        104,
        RailAct::Admit {
            person: Person::parse("fay").unwrap(),
            key: actor_of(&key(6)),
            invite: None,
        },
    );
    let gone = after(
        &key(1),
        &fay_again,
        105,
        RailAct::Remove {
            key: actor_of(&key(4)),
            through: Vec::new(),
        },
    );
    let ops = vec![inv, dee, eve, fay, fay_again, gone];
    let once = admitted(&ops).project::<Membership>();
    let devices = |m: &Membership| [4, 5, 6].map(|k| m.device(&actor_of(&key(k))));
    assert_eq!(
        devices(&once),
        [None, Some(Device::Browser), Some(Device::Computer)],
        "control: dee removed, eve a browser, fay a computer once re-admitted"
    );
    let mut seen = 0;
    crate::tests::permute(&mut (0..ops.len()).collect(), 0, &mut |order| {
        let shuffled: Vec<_> = order.iter().map(|&i| ops[i].clone()).collect();
        assert_eq!(
            admitted(&shuffled).project::<Membership>(),
            once,
            "order {order:?}"
        );
        seen += 1;
    });
    assert_eq!(seen, 720);
    for (i, op) in ops.iter().enumerate() {
        let twice = [ops.clone(), vec![op.clone()]].concat();
        assert_eq!(
            admitted(&twice).project::<Membership>(),
            once,
            "act {i} twice"
        );
    }
}
