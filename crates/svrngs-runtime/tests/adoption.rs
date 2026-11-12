// SPDX-License-Identifier: AGPL-3.0-or-later
//! An adopted group on a computer's journal (svrn-docs
//! `rings/current/EUDOXIA_JUDGE.md` §2e §6): one decider per copy, read
//! through the register as the node reads it, and the same decider whatever
//! order the lines arrived in.

use std::sync::Arc;

use svrngs_runtime::authority::AuthorityChange;
use svrngs_runtime::seed::Seed;
use svrngs_runtime::{
    actor_of, sign_act, Adopt, Decided, Device, Digest, HandOnTo, MintInvite, Name, Office, Op, Person, RailAct, Ready,
    Redeem, Redemption, RingRail, SignedOp, SigningKey, NO_BUDGET,
};

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn all(rail: &RingRail, ns: &str) -> Vec<Op<SignedOp>> {
    rail.ops_missing_from_within(ns, &Digest::default(), NO_BUDGET)
        .unwrap()
        .0
}

/// Sam's old group on his computer, Ana let in by his link, a note of his,
/// and both computers ready: Ana's Ready signed on her own key.
fn pier(dir: &std::path::Path) -> (RingRail, String) {
    let name = |n: &str| Name::parse(n).unwrap();
    let seed = Seed::from_parts(name("Pier"), name("Sam"), key(1).verifying_key(), [4; 16]);
    let (rail, ns) = (RingRail::new(dir, Arc::new(key(1))), seed.id());
    rail.set_seed(&ns, &seed.roster()).unwrap();
    let link = key(30);
    let mint = MintInvite {
        key: actor_of(&link),
        until: i64::MAX,
        device_of: None,
    };
    let invite = rail.commit(&ns, &mint).unwrap().unwrap().lines()[0].id.clone();
    let person = Person::parse("Ana").unwrap();
    let consent = Redemption::sign(&key(2), &ns, &invite, &person, Device::Computer, None);
    let redeem = Redeem {
        invite,
        person,
        key: actor_of(&key(2)),
        cosig: consent.sig,
        device: Device::Computer,
        link: actor_of(&link),
        issuer: actor_of(&key(1)),
    };
    rail.commit(&ns, &redeem).unwrap().unwrap();
    let note = RailAct::from_json(serde_json::json!({ "op": "record", "payload": { "kind": "notes/item" } }));
    rail.commit(&ns, &svrngs_runtime::Append { act: note.unwrap() })
        .unwrap()
        .unwrap();
    let tips = all(&rail, &ns).iter().map(|op| op.id.clone()).collect();
    let change = AuthorityChange::Ready {
        tips,
        keeper: actor_of(&key(1)),
    };
    let ready = sign_act(
        &key(2),
        &ns,
        1,
        0,
        None,
        RailAct::Authority { change },
        None,
        None,
        None,
    );
    rail.ingest_all(&ns, &[ready]).unwrap();
    rail.commit(&ns, &Ready).unwrap().unwrap();
    (rail, ns)
}

/// Plant: `folded` reads a held Adopt's group as the old fold does.
#[test]
fn after_adoption_a_hand_on_waits_on_this_copys_register() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, ns) = pier(dir.path());
    let drawn = rail.adoption(&ns).unwrap().unwrap();
    let adopt = rail.commit(&ns, &Adopt(drawn)).unwrap().unwrap().lines()[0].id.clone();
    assert_eq!(rail.register(&ns).unwrap().decided, Decided::Adopted(adopt.clone()));
    let pinned = rail.group_dir(&ns).unwrap().join(format!("adoption-{adopt}.json"));
    assert!(pinned.exists(), "the verified pin is kept beside the group");
    let to = HandOnTo {
        person: Person::parse("Ana").unwrap(),
        joined: None,
    };
    rail.commit(&ns, &to).unwrap().unwrap();
    let register = rail.register(&ns).unwrap();
    assert_eq!(register.keeper, None);
    assert!(
        matches!(register.office, Office::Pending { .. }),
        "{:?}",
        register.office
    );
}

/// Plant: the decider takes the Adopt it judged first, by arrival.
#[test]
fn the_order_lines_arrive_in_never_picks_the_decider() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, ns) = pier(dir.path());
    let drawn = rail.adoption(&ns).unwrap().unwrap();
    let base = all(&rail, &ns);
    let ready = base
        .iter()
        .filter(|op| op.actor == actor_of(&key(1)))
        .max_by_key(|op| op.kind.seq)
        .unwrap();
    // Two Adopts on one place of Sam's chain: each lawful alone.
    let adopt = |ts| {
        let change = AuthorityChange::Adopt {
            adoption: drawn.clone(),
        };
        let act = RailAct::Authority { change };
        sign_act(
            &key(1),
            &ns,
            ts,
            ready.kind.seq + 1,
            Some(ready.id.clone()),
            act,
            None,
            None,
            None,
        )
    };
    let (one, two) = (adopt(2_000_000_000), adopt(2_000_000_001));
    let copy = |first: &Op<SignedOp>, then: &Op<SignedOp>, at: &std::path::Path| {
        let other = RingRail::new(at, Arc::new(key(2)));
        other.set_seed(&ns, &seed_of(&rail, &ns)).unwrap();
        other.ingest_all(&ns, &base).unwrap();
        other.ingest_all(&ns, std::slice::from_ref(first)).unwrap();
        let alone = other.register(&ns).unwrap().decided;
        assert!(alone.interpreted(), "{alone:?}");
        other.ingest_all(&ns, std::slice::from_ref(then)).unwrap();
        other.register(&ns).unwrap()
    };
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (copy(&one, &two, a.path()), copy(&two, &one, b.path()));
    assert_eq!(a.decided, b.decided);
    assert_eq!(a.decided, Decided::Adopted(one.id.clone().min(two.id.clone())));
    assert_eq!((a.office, b.office), (Office::Frozen, Office::Frozen));
}

fn seed_of(rail: &RingRail, ns: &str) -> svrngs_runtime::Roster {
    let roster = std::fs::read(rail.group_dir(ns).unwrap().join("roster.json")).unwrap();
    serde_json::from_slice(&roster).unwrap()
}
