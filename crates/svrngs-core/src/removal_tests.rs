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

fn remove_of(leaving: u8, through_seq: u64) -> RailAct {
    RailAct::Remove {
        key: actor_of(&key(leaving)),
        through_seq,
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
    let ask = signed(&key(2), 102, 0, remove_of(3, 0));
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
/// membership.rs:323-326), so the backdated act counted.
#[test]
fn a_backdated_act_does_not_slip_under_a_removal() {
    let bo_in = signed(&key(1), 100, 0, admit_of("bo", 2));
    let kept = signed(&key(2), 101, 0, record("kept"));
    let cut = after(&key(1), &bo_in, 110, remove_of(2, 0));
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
    let cut = after(&key(1), &bo_in, 110, remove_of(2, 0));
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
    let a = admitted_under(&alex_alone(), &[bo_in, cy_in, undo, cy_writes.clone()]);
    assert!(standing(&a, 3), "cy stands: {:?}", a.gaps());
    assert!(a.membership().counts(&cy_writes.id), "and cy's act counts");
}
