// SPDX-License-Identifier: AGPL-3.0-or-later
//! The holes U7 closes in the core (DESIGN §6: "forks void nothing",
//! "Removal has one decider", "A removal names the removed key's acts by
//! hash", "U7 also closes two holes"), each planted red at u7-removal's base.
use super::*;
use crate::tests_support::{after, applied, key, record, signed, NS};
use crate::{actor_of, admit, Admission, Ed25519Verifier, RailGap};

/// A seed of alex alone, as `svrngs new` writes one: the maker's key.
fn alex_alone() -> Roster {
    let mut m = BTreeMap::new();
    m.insert(Person::parse("alex").unwrap(), vec![actor_of(&key(1))]);
    Roster::new(m)
}

fn admitted_under(seed: &Roster, ops: &[Op<SignedOp>]) -> Admission {
    admit(&crate::ActSet::of(ops), &[], seed, NS, &Ed25519Verifier)
}

fn admit_of(person: &str, joining: u8) -> RailAct {
    RailAct::Admit {
        person: Person::parse(person).unwrap(),
        key: actor_of(&key(joining)),
        invite: None,
    }
}

/// `leaving`'s removal, having seen `through` of their acts.
fn remove_of(leaving: u8, through: &[&Op<SignedOp>]) -> RailAct {
    RailAct::Remove {
        key: actor_of(&key(leaving)),
        through: through.iter().map(|op| op.id.clone()).collect(),
    }
}

fn standing(a: &Admission, k: u8) -> bool {
    a.membership().standing.contains(&actor_of(&key(k)))
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

/// **A second act at an `Admit`'s seq takes nothing back** (DESIGN §6, "Two
/// acts on one parent are a fork, and forks void nothing"). Alex admits dee
/// at seq 1, then signs a second act on the same parent. Both branches are
/// held and each act weighed as itself: dee stands, dee's act counts, and
/// the other branch's acts count too. At the base two acts on one seq
/// excluded both from what an app folds (admit.rs's per-actor audit), so an
/// author took back their own act on every copy; `membership`, folded
/// before the audit, kept the `Admit`.
#[test]
fn a_second_act_at_an_admits_seq_takes_nothing_back() {
    let one = signed(&key(1), 100, 0, record("one"));
    let let_dee_in = after(&key(1), &one, 101, admit_of("dee", 4));
    let back = after(&key(1), &one, 102, record("back"));
    let on_back = after(&key(1), &back, 103, record("on-back"));
    let dee_writes = signed(&key(4), 104, 0, record("dee-writes"));
    let a = admitted_under(
        &alex_alone(),
        &[one, let_dee_in.clone(), back, on_back, dee_writes.clone()],
    );
    assert!(standing(&a, 4), "dee stands: {:?}", a.gaps());
    assert!(a.membership().counts(&let_dee_in.id), "the Admit counts");
    assert!(
        a.membership().counts(&dee_writes.id),
        "and so does dee's act"
    );
    assert_eq!(
        sorted(applied(&a)),
        sorted(
            ["back", "dee-writes", "on-back", "one"]
                .map(String::from)
                .to_vec()
        ),
        "the other branch's acts count beside it"
    );
    assert!(a.is_complete(), "nothing is missing: {:?}", a.gaps());
}

/// **A member's `Remove` cuts nobody until the keeper sequences it**
/// (DESIGN §6, "Removal has one decider"). Alex made the group; bo, a
/// member, asks to remove cy. Nobody sequenced it, so cy stands and cy's
/// later act counts. At the base the fold's `Remove` arm counted any
/// standing key's cut (membership.rs:323-326).
#[test]
fn a_members_remove_with_no_keeper_sequencing_it_cuts_nobody() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let cy_in = after(&key(1), &bo_in, 101, admit_of("cy", 3));
    let ask = signed(&key(2), 102, 0, remove_of(3, &[]));
    let cy_writes = signed(&key(3), 103, 0, record("cy-writes"));
    let a = admitted_under(&alex_alone(), &[bo_in, cy_in, ask, cy_writes.clone()]);
    assert!(standing(&a, 3), "cy stands: {:?}", a.gaps());
    assert!(a.membership().counts(&cy_writes.id), "and cy's act counts");
}

/// **A backdated act does not slip under a removal's cut** (DESIGN §6, "A
/// removal names the removed key's acts by hash, never by position or
/// time"). Alex, the keeper, removes bo having seen bo's seq 0. Bo then
/// signs on, timed before the removal: an act above the cut, and a second
/// act beside the one alex saw. Neither counts; the act alex saw does. At
/// the base the cut was placed by the `Remove`'s place in the order and
/// `through_seq` went unread (`lib.rs`, `RailAct::Remove`;
/// membership.rs:323-326), so the backdated act counted. (`Remove` then
/// carried `through_seq`; it names acts by id since (c).)
#[test]
fn a_backdated_act_does_not_slip_under_a_removal() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let kept = signed(&key(2), 101, 0, record("kept"));
    let cut = after(&key(1), &bo_in, 110, remove_of(2, &[&kept]));
    let above = after(&key(2), &kept, 105, record("above-the-cut"));
    let beside = signed(&key(2), 102, 0, record("beside-the-cut"));
    let a = admitted_under(
        &alex_alone(),
        &[bo_in, kept.clone(), cut, above.clone(), beside.clone()],
    );
    assert!(!standing(&a, 2), "bo is out");
    assert!(a.membership().counts(&kept.id), "what alex saw counts");
    assert!(
        !a.membership().counts(&above.id),
        "an act above the cut does not"
    );
    assert!(
        !a.membership().counts(&beside.id),
        "nor one beside what alex saw"
    );
    assert_eq!(applied(&a), vec!["kept"]);
}

/// **An act refused inside its author's chain is reported as what it is,
/// not a hole** (DESIGN §6, "U7 also closes two holes"). Alex's seq 1
/// redeems a link this copy does not hold, so it is refused; seq 0 and 2
/// count. The one gap is the refusal. At the base the per-actor audit
/// counted admitted acts only, so seq 1 was also a `SequenceHole` and the
/// copy never read as complete for an act it holds.
#[test]
fn an_act_refused_inside_its_authors_chain_is_not_a_hole() {
    let unheld = signed(
        &key(1),
        90,
        0,
        RailAct::Invite {
            key: actor_of(&key(9)),
            until: 500,
        },
    );
    let person = Person::parse("dee").unwrap();
    let one = signed(&key(1), 100, 0, record("one"));
    let redeems = after(
        &key(1),
        &one,
        101,
        RailAct::Admit {
            invite: Some(Redemption::sign(
                &key(4),
                NS,
                &unheld.id,
                &person,
                Device::Computer,
            )),
            person,
            key: actor_of(&key(4)),
        },
    );
    let two = after(&key(1), &redeems, 102, record("two"));
    let a = admitted_under(&alex_alone(), &[one, redeems.clone(), two]);
    assert_eq!(
        a.gaps(),
        vec![RailGap::RefusedRedemption {
            id: redeems.id.clone(),
            actor: redeems.actor.clone(),
            fault: InviteFault::Unheld,
        }],
        "the refusal, and no hole beside it"
    );
    assert_eq!(applied(&a), vec!["one", "two"]);
}

fn correct_of(target: &Op<SignedOp>) -> RailAct {
    RailAct::Correct {
        corrects: target.id.clone(),
        replacement: None,
    }
}

/// **A removed key cannot take back its removal** (DESIGN §6, "A removed
/// key cannot remove"; "There is no recovery inside a group"). Alex removes
/// bo; bo signs a `Correct` of alex's `Remove`. Bo stays out and bo's later
/// act counts nowhere. At the base the void layer bound every key an
/// unvoided `Admit` named, cut or not (membership.rs:214-245), so bo's
/// correction voided the cut and restored bo.
#[test]
fn a_removed_key_cannot_take_back_its_removal() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let kept = signed(&key(2), 101, 0, record("kept"));
    let cut = after(&key(1), &bo_in, 110, remove_of(2, &[&kept]));
    let undo = after(&key(2), &kept, 111, correct_of(&cut));
    let later = after(&key(2), &undo, 112, record("later"));
    let a = admitted_under(&alex_alone(), &[bo_in, kept, cut, undo, later.clone()]);
    assert!(!standing(&a, 2), "bo stays out: {:?}", a.gaps());
    assert!(
        !a.membership().counts(&later.id),
        "and bo's later act counts nowhere"
    );
}

/// **A member's `Correct` of an `Admit` unseats nobody** (DESIGN §6,
/// "Removal has one decider"). Alex admits cy; bo, a member, signs a
/// `Correct` of that `Admit`. Cy stands: voiding an `Admit` was a removal
/// no keeper sequenced. At the base any bound key's `Correct` voided any
/// act, an `Admit` included, and every key it admitted fell with it
/// (membership.rs:214-252).
#[test]
fn a_members_correct_of_an_admit_unseats_nobody() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let cy_in = after(&key(1), &bo_in, 101, admit_of("cy", 3));
    let undo = signed(&key(2), 102, 0, correct_of(&cy_in));
    let cy_writes = signed(&key(3), 103, 0, record("cy-writes"));
    let a = admitted_under(
        &alex_alone(),
        &[bo_in, cy_in.clone(), undo, cy_writes.clone()],
    );
    assert!(standing(&a, 3), "cy stands: {:?}", a.gaps());
    assert!(a.membership().counts(&cy_writes.id), "and cy's act counts");
    assert!(a.membership().counts(&cy_in.id), "and the Admit, unvoided");
}

fn hand_on(to: u8) -> RailAct {
    RailAct::HandOn {
        key: actor_of(&key(to)),
    }
}

fn gaps_named(a: &Admission, f: impl Fn(&RailGap) -> bool) -> usize {
    a.gaps().iter().filter(|g| f(g)).count()
}

/// **The keeper sequences an ask, and only then it cuts** (U7 (c)). Bo
/// asks to remove cy; alex, the keeper, sequences it by removing cy,
/// having seen cy's one act. Cy is out; cy's act alex saw still counts; a
/// later act of cy's is `Removed`, beside a complete answer.
#[test]
fn the_keeper_sequences_an_ask_and_only_then_it_cuts() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let cy_in = after(&key(1), &bo_in, 101, admit_of("cy", 3));
    let cy_writes = signed(&key(3), 102, 0, record("cy-writes"));
    let ask = signed(&key(2), 103, 0, remove_of(3, &[&cy_writes]));
    let cut = after(&key(1), &cy_in, 104, remove_of(3, &[&cy_writes]));
    let cy_later = after(&key(3), &cy_writes, 105, record("cy-later"));
    let a = admitted_under(
        &alex_alone(),
        &[
            bo_in,
            cy_in,
            cy_writes.clone(),
            ask.clone(),
            cut.clone(),
            cy_later,
        ],
    );
    assert!(!standing(&a, 3), "cy is out");
    assert!(a.membership().counts(&ask.id) && a.membership().counts(&cut.id));
    assert_eq!(a.membership().cut[&actor_of(&key(3))].by, cut.id);
    assert_eq!(applied(&a), vec!["cy-writes"]);
    assert_eq!(gaps_named(&a, |g| matches!(g, RailGap::Removed { .. })), 1);
    assert!(a.is_complete(), "{:?}", a.gaps());
}

/// **The keeper is handed on only by the keeper** (U7 (c)). Cy, not the
/// keeper, hands on to cy: nothing moves. Alex hands on to
/// bo; after it alex's `Remove` of cy is an ask, and bo's cuts.
#[test]
fn a_hand_on_moves_the_keeper_and_only_the_keepers_counts() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let cy_in = after(&key(1), &bo_in, 101, admit_of("cy", 3));
    let cy_hands = signed(&key(3), 102, 0, hand_on(3));
    let alex_hands = after(&key(1), &cy_in, 103, hand_on(2));
    let alex_asks = after(&key(1), &alex_hands, 104, remove_of(3, &[&cy_hands]));
    let a = admitted_under(
        &alex_alone(),
        &[
            bo_in.clone(),
            cy_in.clone(),
            cy_hands.clone(),
            alex_hands.clone(),
            alex_asks.clone(),
        ],
    );
    assert_eq!(a.membership().keeper, Some(actor_of(&key(2))), "bo keeps");
    assert!(standing(&a, 3), "alex's Remove after the hand-on is an ask");
    let bo_cuts = signed(&key(2), 105, 0, remove_of(3, &[&cy_hands]));
    let a = admitted_under(
        &alex_alone(),
        &[bo_in, cy_in, cy_hands, alex_hands, alex_asks, bo_cuts],
    );
    assert!(!standing(&a, 3), "bo's Remove cuts");
}

/// **A removed key cannot remove, ask or hand on, and the keeper cannot
/// remove itself** (U7 (c)). Bo, cut, then removes cy and hands on: both
/// count nowhere. Alex's `Remove` of alex cuts nobody.
#[test]
fn a_removed_key_cannot_remove_ask_or_hand_on() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let cy_in = after(&key(1), &bo_in, 101, admit_of("cy", 3));
    let cut = after(&key(1), &cy_in, 102, remove_of(2, &[]));
    let bo_removes = signed(&key(2), 103, 0, remove_of(3, &[]));
    let bo_hands = after(&key(2), &bo_removes, 104, hand_on(3));
    let alex_alone_out = after(&key(1), &cut, 105, remove_of(1, &[]));
    let a = admitted_under(
        &alex_alone(),
        &[
            bo_in,
            cy_in,
            cut,
            bo_removes,
            bo_hands,
            alex_alone_out.clone(),
        ],
    );
    assert!(standing(&a, 3) && standing(&a, 1), "cy and alex stand");
    assert_eq!(a.membership().keeper, Some(actor_of(&key(1))));
    assert_eq!(gaps_named(&a, |g| matches!(g, RailGap::Removed { .. })), 2);
    assert!(
        a.membership().counts(&alex_alone_out.id),
        "an act, cutting nobody"
    );
}

/// **An ex-keeper's backdated hand-on moves nothing** (U7 (c)). Alex hands
/// on to bo, then signs a second hand-on to cy, timed before the first.
/// It follows alex's hand-on in alex's chain, so it carries no keeper's
/// authority wherever its timestamp puts it: bo keeps.
#[test]
fn an_ex_keepers_backdated_hand_on_moves_nothing() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let cy_in = after(&key(1), &bo_in, 101, admit_of("cy", 3));
    let to_bo = after(&key(1), &cy_in, 110, hand_on(2));
    let to_cy = after(&key(1), &to_bo, 105, hand_on(3));
    let a = admitted_under(&alex_alone(), &[bo_in, cy_in, to_bo, to_cy]);
    assert_eq!(a.membership().keeper, Some(actor_of(&key(2))), "bo keeps");
}

/// **A cut naming an act this copy lacks is a gap until it arrives** (U7
/// (c)). Alex's cut names bo's second act; a copy holding only bo's first
/// names the missing one `UnheldCut` and reads incomplete, and bo's first
/// act, an ancestor of what the cut names, counts once the second arrives.
#[test]
fn a_cut_naming_an_act_this_copy_lacks_is_a_gap_until_it_arrives() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let first = signed(&key(2), 101, 0, record("first"));
    let second = after(&key(2), &first, 102, record("second"));
    let cut = after(&key(1), &bo_in, 103, remove_of(2, &[&second]));
    let lacking = admitted_under(&alex_alone(), &[bo_in.clone(), first.clone(), cut.clone()]);
    assert_eq!(
        gaps_named(
            &lacking,
            |g| matches!(g, RailGap::UnheldCut { missing, .. } if *missing == second.id)
        ),
        1,
        "{:?}",
        lacking.gaps()
    );
    assert!(!lacking.is_complete());
    let whole = admitted_under(&alex_alone(), &[bo_in, first, second, cut]);
    assert_eq!(applied(&whole), vec!["first", "second"]);
    assert!(whole.is_complete(), "{:?}", whole.gaps());
}

/// **Removal is a function of the set** (U7 (c); the permutation contract,
/// `membership.rs`'s module docs). Every arrival order of an ask, a cut, a
/// backdated act, a fork beside what the cut saw, a hand-on, an ex-keeper's
/// backdated hand-on and a removed key's own `Remove` folds to one answer.
#[test]
fn every_arrival_of_a_removal_folds_identically() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let cy_in = after(&key(1), &bo_in, 101, admit_of("cy", 3));
    let kept = signed(&key(2), 102, 0, record("kept"));
    let ask = signed(&key(3), 103, 0, remove_of(2, &[&kept]));
    let cut = after(&key(1), &cy_in, 110, remove_of(2, &[&kept]));
    let above = after(&key(2), &kept, 105, record("above"));
    let beside = signed(&key(2), 104, 0, remove_of(3, &[]));
    let to_cy = after(&key(1), &cut, 111, hand_on(3));
    let back = after(&key(1), &to_cy, 106, hand_on(2));
    let ops = [bo_in, cy_in, kept, ask, cut, above, beside, to_cy, back];
    let once = admitted_under(&alex_alone(), &ops);
    assert_eq!(
        once.membership().keeper,
        Some(actor_of(&key(3))),
        "control: cy keeps"
    );
    assert!(
        !standing(&once, 2) && standing(&once, 3),
        "control: bo out, cy in"
    );
    // A fixed-seed shuffle, as membership_tests.rs's battery: deterministic,
    // no new dependency; nine acts have 362,880 orders, too many to fold each.
    let mut state: u64 = 0x5eed;
    for round in 0..256u64 {
        let mut order: Vec<&Op<SignedOp>> = ops.iter().collect();
        for i in (1..order.len()).rev() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            order.swap(i, (state >> 33) as usize % (i + 1));
        }
        let shuffled: Vec<Op<SignedOp>> = order.into_iter().cloned().collect();
        assert_eq!(
            admitted_under(&alex_alone(), &shuffled),
            once,
            "round {round}"
        );
    }
}
