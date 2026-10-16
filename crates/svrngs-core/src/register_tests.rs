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
            device_of: None,
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
        let register = admit(&held, &[], &ring(), NS, &Ed25519Verifier)
            .membership()
            .clone();
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
    let ins = run(&key(1), 100, [admit("bo", 2), admit("cy", 3)]);
    // Bo's second computer joins by a device link of bo's (bash-devices):
    // before, alex admitted it as `bo` by hand, which now makes another
    // person of that name.
    let (link, second) = device_link(&key(2), &key(5), 105);
    let bo_again = signed(&key(5), 111, 0, record("bo-again"));
    let store = Memory::holding(&[ins, vec![link, second, bo_again]].concat());
    (seed, store)
}

/// `of`'s device link, made at `ts`, and the `Admit` redeeming it for
/// `new`, a computer, as `of`'s computer writes them.
fn device_link(of: &SigningKey, new: &SigningKey, ts: i64) -> (Op<SignedOp>, Op<SignedOp>) {
    let act = RailAct::Invite {
        key: actor_of(&key(8)),
        until: 10_000,
        device_of: Some(actor_of(of)),
    };
    let link = signed(of, ts, 0, act);
    let id = derive_id(&link, NS);
    let admit = RailAct::Admit {
        person: p("bo"),
        key: actor_of(new),
        invite: Some(Redemption::sign(new, NS, &id, &p("bo"), Device::Computer)),
    };
    let redeemed = after(of, &link, ts + 1, admit);
    (link, redeemed)
}

fn register(store: &Memory, seed: &Roster) -> Membership {
    admit(&store.0.borrow(), &[], seed, NS, &Ed25519Verifier)
        .membership()
        .clone()
}

/// **The keeper's removal cuts every key of the person, through what the
/// keeper holds** (U7 (e)). Alex removes bo: a `Remove` of each of bo's two
/// keys, each naming that key's newest act, and bo stands nowhere.
#[test]
fn the_keepers_removal_cuts_every_key_of_the_person() {
    let (seed, store) = alexs_group();
    let done = commit(
        &store,
        &Removal {
            person: p("bo"),
            joined: None,
        },
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
        &Removal {
            person: p("cy"),
            joined: None,
        },
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
        &Removal {
            person: p("cy"),
            joined: None,
        },
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
        &Removal {
            person: p("bo"),
            joined: None,
        },
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
        commit(
            &store,
            &Removal {
                person: p("alex"),
                joined: None
            },
            &as_bo
        )
        .unwrap(),
        Err(Refused::Command(RemovalRefusal::Keeps))
    );
    let held = store.0.borrow().len();
    assert_eq!(
        commit(
            &store,
            &HandOnTo {
                person: p("cy"),
                joined: None
            },
            &as_bo
        )
        .unwrap(),
        Err(Refused::Command(HandOnRefusal::NotKeeper))
    );
    assert_eq!(store.0.borrow().len(), held, "nothing written");
    assert_eq!(
        commit(
            &store,
            &Removal {
                person: p("dee"),
                joined: None
            },
            &as_bo
        )
        .unwrap(),
        Err(Refused::Command(RemovalRefusal::NotInGroup))
    );
    commit(
        &store,
        &HandOnTo {
            person: p("bo"),
            joined: None,
        },
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
        &Removal {
            person: p("alex"),
            joined: None,
        },
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

// ── a name binds once (bash-person; the charter's "The name") ───────

/// `name`, redeemed by `joiner` through the keeper's link in
/// [`alexs_group`], as the keeper's computer writes it.
fn redeem_in_alexs_group(store: &Memory, seed: &Roster, name: &str, joiner: &SigningKey) -> bool {
    let invite = invite_by(&key(1), &link(), 10_000);
    store.0.borrow_mut().insert(invite.clone());
    let id = derive_id(&invite, NS);
    let asked = Redeem {
        invite: id.clone(),
        person: p(name),
        key: actor_of(joiner),
        cosig: Redemption::sign(joiner, NS, &id, &p(name), Device::Computer).sig,
        device: Device::Computer,
        link: actor_of(&link()),
        issuer: actor_of(&key(1)),
    };
    commit(store, &asked, &author(&key(1), seed, 1_000))
        .unwrap()
        .is_ok()
}

/// **A link does not let a stranger in under a name a standing person
/// holds**, as written or in another case: cy stands, and a new key
/// redeeming the keeper's link as `cy`, then as `CY`, is refused and
/// nothing is written. Red at bash-person's base: both admitted
/// (register.rs `Redeem::decide` never reads the name).
#[test]
fn a_link_refuses_a_name_a_standing_person_holds_in_any_case() {
    let let_in: Vec<(&str, bool, usize)> = ["cy", "CY"]
        .into_iter()
        .map(|name| {
            let (seed, store) = alexs_group();
            redeem_in_alexs_group(&store, &seed, name, &joining());
            let stands = register(&store, &seed)
                .standing
                .contains(&actor_of(&joining()));
            let held = store.0.borrow().len();
            (name, stands, held)
        })
        .collect();
    assert_eq!(
        let_in,
        [("cy", false, 6), ("CY", false, 6)],
        "(name, the stranger stands, acts held): nothing past the Invite"
    );
}

/// **No key the keeper cannot remove stands under the keeper's name.** A
/// new key redeems the keeper's link as `alex`; the keeper then asks to
/// remove `alex`'s other keys, as `svrngs remove alex` would: whatever the
/// verdicts, the stranger's key does not stand. Red at bash-person's base:
/// the stranger is admitted as alex and the removal refused `Keeps`
/// (register.rs `Removal::decide`).
#[test]
fn a_stranger_under_the_keepers_name_does_not_stand() {
    let (seed, store) = alexs_group();
    redeem_in_alexs_group(&store, &seed, "alex", &joining());
    let _ = commit(
        &store,
        &Removal {
            person: p("alex"),
            joined: None,
        },
        &author(&key(1), &seed, 1_001),
    )
    .unwrap();
    assert!(
        !register(&store, &seed)
            .standing
            .contains(&actor_of(&joining())),
        "the stranger stands as alex"
    );
}

/// **A hand-on to a person reaches that person's computer, never a
/// stranger's of the same name.** A new computer key that sorts before bo's
/// two redeems the keeper's link as `bo`; the keeper hands on to bo, and
/// one of bo's computers keeps. Red at bash-person's base: the stranger
/// keeps (register.rs `HandOnTo::key`, the first computer key of the name).
#[test]
fn a_hand_on_to_a_name_never_reaches_a_strangers_computer() {
    let (seed, store) = alexs_group();
    let bos = [actor_of(&key(2)), actor_of(&key(5))];
    let stranger = (10..=60)
        .map(key)
        .find(|k| bos.iter().all(|b| actor_of(k) < *b))
        .expect("a key sorting before bo's");
    redeem_in_alexs_group(&store, &seed, "bo", &stranger);
    let _ = commit(
        &store,
        &HandOnTo {
            person: p("bo"),
            joined: None,
        },
        &author(&key(1), &seed, 1_001),
    )
    .unwrap();
    let keeper = register(&store, &seed).keeper.unwrap();
    assert!(bos.contains(&keeper), "a stranger keeps: {keeper}");
}

/// **A removed person's name is free again**: a stranger redeeming as `Cy`
/// is refused, naming cy as cy stands; the keeper removes cy, and the same
/// redemption admits the stranger as `Cy`.
#[test]
fn a_removed_persons_name_is_free_again() {
    let (seed, store) = alexs_group();
    let invite = invite_by(&key(1), &link(), 10_000);
    store.0.borrow_mut().insert(invite.clone());
    let id = derive_id(&invite, NS);
    let asked = Redeem {
        invite: id.clone(),
        person: p("Cy"),
        key: actor_of(&joining()),
        cosig: Redemption::sign(&joining(), NS, &id, &p("Cy"), Device::Computer).sig,
        device: Device::Computer,
        link: actor_of(&link()),
        issuer: actor_of(&key(1)),
    };
    let alex = key(1);
    let keeper = author(&alex, &seed, 1_000);
    assert_eq!(
        commit(&store, &asked, &keeper).unwrap(),
        Err(Refused::Command(RedeemRefusal::NameTaken(p("cy"))))
    );
    commit(
        &store,
        &Removal {
            person: p("cy"),
            joined: None,
        },
        &keeper,
    )
    .unwrap()
    .unwrap();
    commit(&store, &asked, &author(&alex, &seed, 1_001))
        .unwrap()
        .unwrap();
    let m = register(&store, &seed);
    assert!(m.standing.contains(&actor_of(&joining())));
    assert_eq!(
        m.person_of(&actor_of(&joining())).map(|(_, n)| n),
        Some(&p("Cy"))
    );
}

// ── a person is their first key (bash-devices) ──────────────────────

/// **Two people a race gave one name are two people.** Alex lets dee in
/// on one computer and bo lets another dee in on theirs, each by hand,
/// neither having seen the other; the keeper's removal of `dee` is refused,
/// since two people hold the name, and both stand; so is a hand-on to
/// `dee`, and alex keeps. Red at bash-devices' base: the fold binds a key to a
/// name (membership.rs, `bindings`), so both keys are one dee: the removal
/// cuts both (register.rs `Removal::decide`, `keys_of`), and the hand-on
/// goes to the first of them (`HandOnTo::key`).
#[test]
fn two_people_a_race_gave_one_name_are_not_taken_as_one() {
    let seed = Roster::new([(p("alex"), vec![actor_of(&key(1))])].into_iter().collect());
    let admit = |person: &str, k: u8| RailAct::Admit {
        person: p(person),
        key: actor_of(&key(k)),
        invite: None,
    };
    let alexs = run(&key(1), 100, [admit("bo", 2), admit("dee", 6)]);
    let bos = run(&key(2), 120, [admit("dee", 8)]);
    let raced = || Memory::holding(&[alexs.clone(), bos.clone()].concat());
    let alex = key(1);
    let as_alex = author(&alex, &seed, 200);
    let (removing, handing) = (raced(), raced());
    let removed = commit(
        &removing,
        &Removal {
            person: p("dee"),
            joined: None,
        },
        &as_alex,
    )
    .unwrap();
    let handed = commit(
        &handing,
        &HandOnTo {
            person: p("dee"),
            joined: None,
        },
        &as_alex,
    )
    .unwrap();
    let (r, h) = (register(&removing, &seed), register(&handing, &seed));
    let stand = [6, 8].map(|k| r.standing.contains(&actor_of(&key(k))));
    assert_eq!(
        (removed.is_err(), handed.is_err(), stand, h.keeper),
        (true, true, [true, true], Some(actor_of(&key(1)))),
        "(removal refused, hand-on refused, each dee stands, the keeper)"
    );
}

/// **A person's second device stands as them by a link of theirs, never by
/// typing their name.** Cy's computer makes a device link and a new key
/// redeems it under another name: it stands as cy, by cy's first key and
/// cy's name, and the keeper's removal of cy takes both. Red at
/// bash-devices' base: no link said "as me", and the name was refused
/// `NameTaken` (bash-person), so a second device had no way in as cy. (The
/// link is a device link and the name another since (b): at (a) neither
/// could be written.)
#[test]
fn a_persons_second_device_stands_as_them_by_a_link_of_theirs() {
    let (seed, store) = alexs_group();
    let cy = key(3);
    let invite = signed(
        &cy,
        200,
        0,
        RailAct::Invite {
            key: actor_of(&link()),
            until: 10_000,
            device_of: Some(actor_of(&cy)),
        },
    );
    assert_eq!(invite.kind.line_version(), DEVICE_LINE_VERSION);
    store.0.borrow_mut().insert(invite.clone());
    let id = derive_id(&invite, NS);
    let asked = Redeem {
        invite: id.clone(),
        person: p("whoever"),
        key: actor_of(&joining()),
        cosig: Redemption::sign(&joining(), NS, &id, &p("whoever"), Device::Computer).sig,
        device: Device::Computer,
        link: actor_of(&link()),
        issuer: actor_of(&cy),
    };
    let redeemed = commit(&store, &asked, &author(&cy, &seed, 1_000)).unwrap();
    assert!(redeemed.is_ok(), "cy's second device let in: {redeemed:?}");
    let (cys, joins) = (actor_of(&cy), actor_of(&joining()));
    assert_eq!(
        register(&store, &seed).person_of(&joins),
        Some((cys.as_str(), &p("cy")))
    );
    commit(
        &store,
        &Removal {
            person: p("cy"),
            joined: None,
        },
        &author(&key(1), &seed, 1_001),
    )
    .unwrap()
    .unwrap();
    let m = register(&store, &seed);
    assert!(
        !m.standing.contains(&cys) && !m.standing.contains(&joins),
        "cy's two devices go together"
    );
}

/// **A device link counts only from a key standing as its person.** Cy's
/// computer makes a link adding a device of bo's; its redemption is refused
/// and the key does not stand, as bo's or anyone's. Planted: the fold's
/// check out (membership.rs, `theirs` always true) — the key stands as bo.
#[test]
fn a_device_link_counts_only_from_a_key_standing_as_its_person() {
    let (seed, store) = alexs_group();
    let (forged, _) = device_link(&key(2), &joining(), 100);
    let RailAct::Invite { device_of, .. } = &forged.kind.act else {
        panic!("an Invite")
    };
    let cy = key(3);
    let act = RailAct::Invite {
        key: actor_of(&link()),
        until: 10_000,
        device_of: device_of.clone(),
    };
    let invite = signed(&cy, 200, 0, act);
    store.0.borrow_mut().insert(invite.clone());
    let id = derive_id(&invite, NS);
    let asked = Redeem {
        invite: id.clone(),
        person: p("bo"),
        key: actor_of(&joining()),
        cosig: Redemption::sign(&joining(), NS, &id, &p("bo"), Device::Computer).sig,
        device: Device::Computer,
        link: actor_of(&link()),
        issuer: actor_of(&cy),
    };
    let redeemed = commit(&store, &asked, &author(&cy, &seed, 1_000)).unwrap();
    let m = register(&store, &seed);
    assert_eq!(
        (redeemed, m.standing.contains(&actor_of(&joining()))),
        (
            Err(Refused::Command(RedeemRefusal::Fault(
                InviteFault::Uncounted
            ))),
            false
        ),
        "(the redemption, the key stands)"
    );
}

/// **A plain redemption of a name is a new person, never a merge.** A
/// stranger joins as `dee` through alex's link while no dee stands, then
/// the keeper's hand lets another key in as `dee`: two people, each their
/// own first key, and removal by the name is refused `Which` in the
/// fold's words, cutting neither.
#[test]
fn a_plain_redemption_is_a_new_person_keyed_by_its_own_key() {
    let (seed, store) = alexs_group();
    redeem_in_alexs_group(&store, &seed, "dee", &joining());
    let by_hand = RailAct::Admit {
        person: p("dee"),
        key: actor_of(&key(6)),
        invite: None,
    };
    let alexs = (store.0.borrow().iter())
        .filter(|o| o.actor == actor_of(&key(1)))
        .max_by_key(|o| o.kind.seq)
        .cloned()
        .unwrap();
    store
        .0
        .borrow_mut()
        .insert(after(&key(1), &alexs, 1_002, by_hand));
    let m = register(&store, &seed);
    let firsts = [actor_of(&joining()), actor_of(&key(6))]
        .map(|k| m.person_of(&k).map(|(f, n)| (f.to_string(), n.clone())));
    assert_eq!(
        firsts,
        [
            Some((actor_of(&joining()), p("dee"))),
            Some((actor_of(&key(6)), p("dee")))
        ]
    );
    assert_eq!(
        commit(
            &store,
            &Removal {
                person: p("dee"),
                joined: None
            },
            &author(&key(1), &seed, 1_003)
        )
        .unwrap(),
        Err(Refused::Command(RemovalRefusal::Which(vec![
            Joined::At(1_000),
            Joined::At(1_002)
        ])))
    );
}

// ── two people of one name, told apart (bash-same-name) ─────────────

/// **The keeper removes one of two people of one name, by when they
/// joined.** dee joins through alex's link at 1,000 and another dee by
/// alex's hand at 1,002. By the name alone, removal and hand-on are refused
/// with when each dee joined; asked of a time neither joined at, the same;
/// of 1,002, the removal cuts that dee alone and the other stands, and a
/// hand-on of 1,000 reaches the other. Of a seed row's person, as having
/// started the group. Red at this row's base: no way to name one dee.
#[test]
fn the_keeper_names_one_of_two_people_of_one_name_by_when_they_joined() {
    let (seed, store) = alexs_group();
    redeem_in_alexs_group(&store, &seed, "dee", &joining());
    let alexs = (store.0.borrow().iter())
        .filter(|o| o.actor == actor_of(&key(1)))
        .max_by_key(|o| o.kind.seq)
        .cloned()
        .unwrap();
    let by_hand = RailAct::Admit {
        person: p("dee"),
        key: actor_of(&key(6)),
        invite: None,
    };
    let by_hand = after(&key(1), &alexs, 1_002, by_hand);
    store.0.borrow_mut().insert(by_hand);
    let alexs_key = key(1);
    let as_alex = author(&alexs_key, &seed, 1_003);
    let each = vec![Joined::At(1_000), Joined::At(1_002)];
    let dee = |joined| Removal {
        person: p("dee"),
        joined,
    };
    let to_dee = |joined| HandOnTo {
        person: p("dee"),
        joined,
    };
    assert_eq!(
        (
            commit(&store, &dee(None), &as_alex).unwrap(),
            commit(&store, &dee(Some(Joined::At(1_001))), &as_alex).unwrap(),
            commit(&store, &to_dee(None), &as_alex).unwrap(),
        ),
        (
            Err(Refused::Command(RemovalRefusal::Which(each.clone()))),
            Err(Refused::Command(RemovalRefusal::Which(each.clone()))),
            Err(Refused::Command(HandOnRefusal::Which(each))),
        ),
        "(by name alone, by a time neither joined at, a hand-on by name alone)"
    );
    assert!(commit(&store, &dee(Some(Joined::At(1_002))), &as_alex)
        .unwrap()
        .is_ok());
    let m = register(&store, &seed);
    assert_eq!(
        [actor_of(&joining()), actor_of(&key(6))].map(|k| m.standing.contains(&k)),
        [true, false],
        "the dee who joined at 1,002 is out, the other stands"
    );
    let alex = || Removal {
        person: p("alex"),
        joined: Some(Joined::Started),
    };
    assert_eq!(
        commit(&store, &alex(), &as_alex).unwrap(),
        Err(Refused::Command(RemovalRefusal::Keeps)),
        "alex, who started the group, named so"
    );
    assert!(commit(&store, &to_dee(Some(Joined::At(1_000))), &as_alex)
        .unwrap()
        .is_ok());
    assert_eq!(register(&store, &seed).keeper, Some(actor_of(&joining())));
}

/// relay-in-record (b): a computer's relay is written when the record
/// says otherwise — moved, then withdrawn — and nothing when it says the
/// same, so a round asking each time writes once per move.
#[test]
fn a_computers_relay_is_written_once_per_move() {
    let (seed, store) = alexs_group();
    let url = |u: &str| Some(relay::Url::parse(u).unwrap());
    let at = |u: Option<relay::Url>| RelayAt {
        key: actor_of(&key(3)),
        url: u,
    };
    for (now, asked, writes) in [
        (1_000, url("https://cy.example/"), 1),
        (1_001, url("https://cy.example/"), 0),
        (1_002, url("https://moved.example/"), 1),
        (1_003, None, 1),
        (1_004, None, 0),
    ] {
        let done = commit(&store, &at(asked.clone()), &author(&key(3), &seed, now))
            .unwrap()
            .unwrap();
        assert_eq!(done.lines().len(), writes, "at {now}");
        assert_eq!(
            register(&store, &seed).relay(&actor_of(&key(3))),
            asked,
            "at {now}"
        );
    }
}

// ── leaving (groups-leave) ──────────────────────────────────────

/// **A person's leave cuts every key of theirs, from either computer, by
/// them alone** (groups-leave (a)). Bo leaves from each of bo's two
/// computers in turn, in a fresh copy of alex's group: a `Remove` of each
/// key, the signer's last, and bo stands nowhere while alex and cy stay.
/// Planted with the signer's key first (`[&self.0].into_iter().chain(..)`
/// in `Leave::decide`), one of the two left bo's other computer standing.
#[test]
fn a_persons_leave_cuts_every_key_of_theirs() {
    for from in [key(2), key(5)] {
        let (seed, store) = alexs_group();
        let done = commit(&store, &Leave(actor_of(&from)), &author(&from, &seed, 200));
        let done = done.unwrap().unwrap();
        assert_eq!(done.lines().len(), 2, "one Remove per key");
        let m = register(&store, &seed);
        for k in [key(2), key(5)] {
            assert!(!m.standing.contains(&actor_of(&k)), "bo is out: {m:?}");
        }
        assert!(m.standing.contains(&actor_of(&key(1))) && m.standing.contains(&actor_of(&key(3))));
    }
}

/// **The keeper's leave is refused until they hand on, and writes nothing**
/// (groups-leave (a)). Alex leaves: `Keeps`, nothing written. Alex hands on
/// to bo and leaves: alex is out. Bo removing bo with `svrngs remove` is
/// refused `Own`, not read as half a leave.
#[test]
fn the_keepers_leave_waits_for_the_hand_on() {
    let (seed, store) = alexs_group();
    let alex = Leave(actor_of(&key(1)));
    let held = store.0.borrow().len();
    assert_eq!(
        commit(&store, &alex, &author(&key(1), &seed, 200)).unwrap(),
        Err(Refused::Command(RemovalRefusal::Keeps))
    );
    assert_eq!(store.0.borrow().len(), held, "nothing written");
    let to_cy = HandOnTo {
        person: p("cy"),
        joined: None,
    };
    commit(&store, &to_cy, &author(&key(1), &seed, 201))
        .unwrap()
        .unwrap();
    commit(&store, &alex, &author(&key(1), &seed, 202))
        .unwrap()
        .unwrap();
    assert!(
        !register(&store, &seed)
            .standing
            .contains(&actor_of(&key(1))),
        "alex left"
    );
    for from in [key(2), key(5)] {
        let bo = Removal {
            person: p("bo"),
            joined: None,
        };
        let refused = commit(&store, &bo, &author(&from, &seed, 203)).unwrap();
        assert_eq!(refused, Err(Refused::Command(RemovalRefusal::Own)));
    }
}
