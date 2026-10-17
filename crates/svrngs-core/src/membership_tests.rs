// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/membership.rs@e1a43bcd2
use super::*;
use crate::tests_support::{key, record, signed, with_ids, NS};
use crate::{actor_of, Ed25519Verifier};

fn seed_of(seeds: &[(u8, &str)]) -> Roster {
    let mut m = BTreeMap::new();
    for (k, name) in seeds {
        m.insert(Person::parse(*name).unwrap(), vec![actor_of(&key(*k))]);
    }
    Roster::new(m)
}

fn admit_act(by: u8, ts: i64, seq: u64, person: &str, joining: u8) -> Op<SignedOp> {
    signed(
        &key(by),
        ts,
        seq,
        RailAct::Admit {
            person: Person::parse(person).unwrap(),
            key: actor_of(&key(joining)),
            invite: None,
        },
    )
}

/// `by` removes (or asks to remove) `leaving`, having seen `through` of
/// their acts.
fn remove_act(by: u8, ts: i64, seq: u64, leaving: u8, through: &[&Op<SignedOp>]) -> Op<SignedOp> {
    signed(
        &key(by),
        ts,
        seq,
        RailAct::Remove {
            key: actor_of(&key(leaving)),
            through: through.iter().map(|op| op.id.clone()).collect(),
        },
    )
}

fn void(by: u8, ts: i64, seq: u64, target: &Op<SignedOp>) -> Op<SignedOp> {
    signed(
        &key(by),
        ts,
        seq,
        RailAct::Correct {
            corrects: target.id.clone(),
            replacement: None,
        },
    )
}

/// **Leg 1 — the permutation property, at membership level.** The rail's
/// order is content-derived, so arrival order must not reach the answer:
/// every permutation of a set with a mutual removal and an Admit racing
/// its own void produces the identical Membership. Watched red against a
/// fold that walks arrival order (the pre-registration's named defect).
#[test]
fn every_permutation_of_a_membership_set_agrees() {
    let seed = seed_of(&[(1, "alex")]);
    let admit_dee = admit_act(1, 104, 2, "dee", 4);
    let ops = vec![
        admit_act(1, 100, 0, "bo", 2),
        admit_act(1, 101, 1, "cy", 3),
        // A mutual removal: bo and cy cut each other.
        remove_act(2, 102, 0, 3, &[]),
        remove_act(3, 103, 0, 2, &[]),
        // An Admit racing its own void.
        admit_dee.clone(),
        signed(
            &key(1),
            105,
            3,
            RailAct::Correct {
                corrects: admit_dee.id.clone(),
                replacement: None,
            },
        ),
        signed(&key(2), 106, 1, record("x")),
    ];
    let base = membership(with_ids(ops.iter()), &seed, NS);
    // Every arrival order is the same answer.
    for rotation in 1..ops.len() {
        let mut rotated: Vec<&Op<SignedOp>> = ops.iter().collect();
        rotated.rotate_left(rotation);
        assert_eq!(
            membership(with_ids(rotated), &seed, NS),
            base,
            "rotation {rotation} changed the answer"
        );
    }
    // And the two named conflict cases. Since U7 (DESIGN §6, "Removal has
    // one decider") neither bo nor cy is the keeper, so the mutual removal is
    // two asks that cut nobody, and a Correct of an Admit voids nothing, so
    // dee stands. Before, the rail's order picked one cut and the void
    // unseated dee.
    assert!(
        base.counts(&ops[2].id) && base.counts(&ops[3].id),
        "both asks count as acts"
    );
    for k in [2, 3, 4] {
        assert!(base.standing.contains(&actor_of(&key(k))), "key {k} stands");
    }
}

/// **Leg 2 — void one Admit, and every key it transitively admitted
/// fell** (p2panda strong-removal rule 4), until U7: a `Correct` of an act
/// that moves membership now voids nothing, since voiding an `Admit` was a
/// removal no keeper sequenced (DESIGN §6, "Removal has one decider"). The
/// name is the rule before U7, kept so u7-removal's check follows every
/// test by name; the stranger and whom they admitted stand.
#[test]
fn voiding_an_admit_drops_what_it_transitively_admitted() {
    let seed = seed_of(&[(1, "alex")]);
    let stranger = admit_act(1, 100, 0, "stranger", 4);
    let stranger_admits_p = admit_act(4, 101, 0, "p", 5);
    let undo = signed(
        &key(1),
        102,
        1,
        RailAct::Correct {
            corrects: stranger.id.clone(),
            replacement: None,
        },
    );
    let m = membership(with_ids([&stranger, &stranger_admits_p, &undo]), &seed, NS);
    assert!(
        m.standing.contains(&actor_of(&key(4))),
        "the stranger stands: the Correct voids nothing"
    );
    assert!(
        m.standing.contains(&actor_of(&key(5))),
        "and so does every key they admitted"
    );
    assert!(m.counts(&undo.id) && m.voided.is_empty());
}

/// **Leg 3 — voiding a Remove restored the member and admitted what they
/// wrote while removed**, until U7: there is no recovery inside a group
/// (DESIGN §6), so a `Correct` of the keeper's cut voids nothing and the
/// member stays out. The name is the rule before U7, kept so u7-removal's
/// check follows every test by name.
#[test]
fn voiding_a_remove_restores_the_member_and_their_in_between_writes() {
    let seed = seed_of(&[(1, "alex"), (2, "bo")]);
    let cut = remove_act(1, 100, 2, 2, &[]);
    let while_out = signed(&key(2), 101, 2, record("written-while-out"));
    let undo = signed(
        &key(1),
        102,
        3,
        RailAct::Correct {
            corrects: cut.id.clone(),
            replacement: None,
        },
    );
    let m = membership(with_ids([&cut, &while_out, &undo]), &seed, NS);
    assert!(
        !m.standing.contains(&actor_of(&key(2))),
        "voiding the Remove restores nobody"
    );
    assert!(
        !m.counts(&while_out.id),
        "and what they wrote while removed counts nowhere"
    );
}

/// The goodhart leg of the pre-registration, at this level: a function
/// that ignores Admit and Remove passes the permutation test perfectly.
/// These assertions are what prove the acts do anything at all.
#[test]
fn the_membership_acts_do_something() {
    let seed = seed_of(&[(1, "alex")]);
    let add = admit_act(1, 100, 0, "bo", 2);
    let write = signed(&key(2), 101, 0, record("bo-writes"));
    let cut = remove_act(1, 102, 1, 2, &[&write]);
    let after_cut = signed(&key(2), 103, 1, record("after-the-cut"));
    let m = membership(with_ids([&add, &write, &cut, &after_cut]), &seed, NS);
    assert!(m.counts(&write.id), "bo counts while admitted");
    assert!(
        !m.counts(&after_cut.id),
        "and not after the cut — position-scoped standing"
    );
    let _ = Ed25519Verifier;
}

/// **The permutation contract at breadth** (leg 1's battery): sixty-four
/// seeded shuffles of a set carrying every interesting pair — a mutual
/// removal, an Admit racing its own void, a re-Admit after a cut — and
/// every shuffle folds identically. The seed is fixed so the battery is
/// deterministic; the assertion is the CONTRACT (arrival never reaches
/// the answer), and the shuffles are the arrival orders a live room
/// produces.
#[test]
fn every_interleaving_of_a_membership_set_folds_identically() {
    let seed = seed_of(&[(1, "alex")]);
    let raced = admit_act(1, 104, 2, "dee", 4);
    let ops = vec![
        admit_act(1, 100, 0, "bo", 2),
        admit_act(1, 101, 1, "cy", 3),
        remove_act(2, 102, 0, 3, &[]),
        remove_act(3, 103, 0, 2, &[]),
        raced.clone(),
        void(1, 105, 3, &raced),
        admit_act(1, 106, 3, "bo", 2),
        signed(&key(2), 107, 1, record("bo-writes")),
        remove_act(1, 108, 4, 2, &[]),
    ];
    let base = membership(with_ids(ops.iter()), &seed, NS);
    let mut state: u64 = 0x5eed;
    for round in 0..64u64 {
        let mut order: Vec<&Op<SignedOp>> = ops.iter().collect();
        // A fixed-seed LCG shuffle — deterministic, no new dependency.
        for i in (1..order.len()).rev() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (state >> 33) as usize % (i + 1);
            order.swap(i, j);
        }
        assert_eq!(
            membership(with_ids(order.iter().copied()), &seed, NS),
            base,
            "interleaving {round} changed the answer — arrival reached the fold"
        );
    }
}

fn relay_act(by: u8, ts: i64, seq: u64, url: Option<&str>) -> Op<SignedOp> {
    signed(
        &key(by),
        ts,
        seq,
        RailAct::Relay {
            url: url.map(String::from),
        },
    )
}

/// relay-in-record (b): the fold answers each key's relay from the record —
/// its own newest `Relay` winning, in any arrival order, read dotless — and
/// a relay withdrawn, never said, unreadable, or of a key standing nowhere,
/// said as absent. Watched red against a fold that answers none.
#[test]
fn the_fold_answers_each_keys_relay_its_newest_winning() {
    let seed = seed_of(&[(1, "alex")]);
    let ops = vec![
        admit_act(1, 100, 0, "bo", 2),
        admit_act(1, 100, 1, "cy", 3),
        admit_act(1, 100, 2, "dee", 4),
        relay_act(1, 101, 3, Some("https://old.example/")),
        relay_act(2, 101, 0, Some("https://bo.example/")),
        relay_act(4, 101, 0, Some("https://dee.example/")),
        relay_act(9, 101, 0, Some("https://stranger.example/")),
        // alex's moved, spelled with the root's dot; bo's withdrawn; dee's
        // newest is no URL.
        relay_act(1, 102, 4, Some("https://new.example./")),
        relay_act(2, 103, 1, None),
        relay_act(4, 103, 1, Some("not a url")),
    ];
    let actor = |k: u8| actor_of(&key(k));
    let base = membership(with_ids(ops.iter()), &seed, NS);
    assert_eq!(
        base.relay(&actor(1)).map(|u| u.to_string()).as_deref(),
        Some("https://new.example/")
    );
    for k in [2, 3, 4, 9] {
        assert_eq!(base.relay(&actor(k)), None, "key {k}'s relay is absent");
    }
    for rotation in 1..ops.len() {
        let mut rotated: Vec<&Op<SignedOp>> = ops.iter().collect();
        rotated.rotate_left(rotation);
        assert_eq!(
            membership(with_ids(rotated), &seed, NS),
            base,
            "rotation {rotation}"
        );
    }
}

/// **Each person's stretch in the group** (canon-spans (b)): alex founds
/// it with two keys, bo and cy join, alex's second key is cut, cy leaves,
/// and cy comes back by a new key. alex stands from the seed, the cut of
/// one key of two ending nothing; bo from his admission; cy's first
/// stretch ends at her leave, and her second, a new person of her name,
/// stands. Planted: `left` read as none for everyone, and cy reads as
/// standing still; a rejoin read as one stretch, and cy has one.
#[test]
fn the_fold_answers_each_persons_stretch_in_the_group() {
    let mut seed = BTreeMap::new();
    seed.insert(
        Person::parse("alex").unwrap(),
        vec![actor_of(&key(1)), actor_of(&key(5))],
    );
    let ops = vec![
        admit_act(1, 100, 0, "bo", 2),
        admit_act(1, 101, 1, "cy", 3),
        remove_act(1, 102, 2, 5, &[]),
        remove_act(3, 103, 0, 3, &[]),
        admit_act(1, 105, 3, "cy", 4),
    ];
    let m = membership(with_ids(ops.iter()), &Roster::new(seed), NS);
    let mut spans: Vec<_> = m
        .spans()
        .into_iter()
        .map(|s| (s.person.to_string(), s.joined, s.left))
        .collect();
    spans.sort();
    let at = |name: &str, joined, left| (String::from(name), joined, left);
    assert_eq!(
        spans,
        [
            at("alex", None, None),
            at("bo", Some(100), None),
            at("cy", Some(101), Some(103)),
            at("cy", Some(105), None)
        ],
        "{m:?}"
    );
}
