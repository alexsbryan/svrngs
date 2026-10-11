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
    redeem_from(invite, Device::Computer)
}

fn redeem_from(invite: &Op<SignedOp>, device: Device) -> Redeem {
    let id = derive_id(invite, NS);
    Redeem {
        invite: id.clone(),
        person: p("hal"),
        key: actor_of(&joining()),
        cosig: Redemption::sign(&joining(), NS, &id, &p("hal"), device).sig,
        device,
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

/// Given an `Invite`, when the issuer is asked to redeem it for a key a
/// browser holds, then the `Admit` names a browser, and the register answers
/// that the key stands as a browser's (u4-1). And the same for a computer.
#[test]
fn a_redemption_writes_the_device_it_names() {
    for device in [Device::Browser, Device::Computer] {
        let invite = invite_by(&issuer(), &link(), 10_000);
        let store = Memory::holding(&[invite.clone()]);
        let roster = ring();
        let recorded = commit(
            &store,
            &redeem_from(&invite, device),
            &author(&issuer(), &roster, 1_000),
        )
        .unwrap()
        .unwrap();
        let written = match &recorded.lines()[0].kind.act {
            RailAct::Admit {
                invite: Some(r), ..
            } => r.device,
            other => panic!("an Admit redeeming the Invite, not {other:?}"),
        };
        assert_eq!(written, device, "the Admit names what the request named");
        let held = store.0.borrow();
        let register = admit(&held, &[], &ring(), NS, &Ed25519Verifier).project::<Membership>();
        assert_eq!(register.device(&actor_of(&joining())), Some(device));
    }
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

// ── removal (U7) ────────────────────────────────────────────────

/// A group alex made, alex keeping it: bo on two computers, cy on one, and
/// an act of bo's on each key.
fn alexs_group() -> (Roster, Memory) {
    let seed = Roster::new([(p("alex"), vec![actor_of(&key(1))])].into_iter().collect());
    let admit = |person: &str, k: u8| RailAct::Admit {
        person: p(person),
        key: actor_of(&key(k)),
        invite: None,
    };
    let ins = run(
        &key(1),
        100,
        [admit("bo", 2), admit("bo", 5), admit("cy", 3)],
    );
    let bo_writes = signed(&key(2), 110, 0, record("bo-writes"));
    let bo_again = signed(&key(5), 111, 0, record("bo-again"));
    let store = Memory::holding(&[ins, vec![bo_writes, bo_again]].concat());
    (seed, store)
}

fn register(store: &Memory, seed: &Roster) -> Membership {
    admit(&store.0.borrow(), &[], seed, NS, &Ed25519Verifier).project::<Membership>()
}

/// **The keeper's removal cuts every key of the person, through what the
/// keeper holds** (U7 (e)). Alex removes bo: a `Remove` of each of bo's two
/// keys, each naming that key's newest act, and bo stands nowhere.
#[test]
fn the_keepers_removal_cuts_every_key_of_the_person() {
    let (seed, store) = alexs_group();
    let done = commit(
        &store,
        &Removal { person: p("bo") },
        &author(&key(1), &seed, 200),
    )
    .unwrap()
    .unwrap();
    assert_eq!(done.lines().len(), 2, "one Remove per key");
    for line in done.lines() {
        let RailAct::Remove { through, .. } = &line.kind.act else {
            panic!("a Remove")
        };
        assert_eq!(through.len(), 1, "the key's one newest act");
    }
    let m = register(&store, &seed);
    assert!(!m.standing.contains(&actor_of(&key(2))) && !m.standing.contains(&actor_of(&key(5))));
    assert!(m.standing.contains(&actor_of(&key(3))), "cy stays");
}

/// **A member's removal is an ask and cuts nobody** (U7 (e)): bo asks to
/// remove cy, the `Remove` is written and counts, and cy stands; then alex
/// asks the same and cy is out. Red with the fold's keeper check out
/// (membership.rs, `let keeps = true`).
#[test]
fn a_members_removal_is_an_ask_until_the_keeper_asks_the_same() {
    let (seed, store) = alexs_group();
    let asked = commit(
        &store,
        &Removal { person: p("cy") },
        &author(&key(2), &seed, 200),
    )
    .unwrap()
    .unwrap();
    assert_eq!(asked.lines().len(), 1, "the ask is written");
    assert!(
        register(&store, &seed)
            .standing
            .contains(&actor_of(&key(3))),
        "cy stands"
    );
    commit(
        &store,
        &Removal { person: p("cy") },
        &author(&key(1), &seed, 201),
    )
    .unwrap()
    .unwrap();
    assert!(
        !register(&store, &seed)
            .standing
            .contains(&actor_of(&key(3))),
        "the keeper's cuts"
    );
    let refused = commit(
        &store,
        &Removal { person: p("bo") },
        &author(&key(3), &seed, 202),
    )
    .unwrap();
    assert_eq!(
        refused,
        Err(Refused::NoStanding),
        "a removed person asks nothing"
    );
}

/// **The keeper is not removed, and only the keeper hands on** (U7 (e)).
/// Bo's removal of alex is refused `Keeps`; bo's hand-on to cy is refused
/// `NotKeeper` and writes nothing; alex hands on to bo, and then bo's
/// removal of alex is the cut. A person not in the group is `NotInGroup`.
#[test]
fn the_keeper_hands_on_and_is_removed_only_after() {
    let (seed, store) = alexs_group();
    let bo = key(2);
    let as_bo = author(&bo, &seed, 200);
    assert_eq!(
        commit(&store, &Removal { person: p("alex") }, &as_bo).unwrap(),
        Err(Refused::Command(RemovalRefusal::Keeps))
    );
    let held = store.0.borrow().len();
    assert_eq!(
        commit(&store, &HandOnTo { person: p("cy") }, &as_bo).unwrap(),
        Err(Refused::Command(HandOnRefusal::NotKeeper))
    );
    assert_eq!(store.0.borrow().len(), held, "nothing written");
    assert_eq!(
        commit(&store, &Removal { person: p("dee") }, &as_bo).unwrap(),
        Err(Refused::Command(RemovalRefusal::NotInGroup))
    );
    commit(
        &store,
        &HandOnTo { person: p("bo") },
        &author(&key(1), &seed, 201),
    )
    .unwrap()
    .unwrap();
    let keeper = register(&store, &seed).keeper.unwrap();
    let bos = [key(2), key(5)];
    let Some(bo_keeps) = bos.iter().find(|k| actor_of(k) == keeper) else {
        panic!("one of bo's computers keeps")
    };
    commit(
        &store,
        &Removal { person: p("alex") },
        &author(bo_keeps, &seed, 202),
    )
    .unwrap()
    .unwrap();
    assert!(
        !register(&store, &seed)
            .standing
            .contains(&actor_of(&key(1))),
        "alex is out"
    );
}
