//! **Removal wins.** A person's Removes of their own keys never void each
//! other (Eudoxia `community/src/history.rs` `exits`, from 560b0c2): a
//! device removed by another device of its person, or cut by the keeper,
//! stays out after signing a back-dated Remove of its remover, and two
//! Leaves of one person at once leave every key of theirs out. Red at
//! e995acd, the pin before: three of these four stood the device again.
//! Through the public path a computer folds with: `admit`.
use std::collections::BTreeMap;

use svrngs_core::{
    actor_of, admit, sign_act, ActSet, Admission, Ed25519Verifier, Op, Person, RailAct, Roster,
    SignedOp, SigningKey,
};

const NS: &str = "counter-remove";

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}
fn actor(n: u8) -> String {
    actor_of(&key(n))
}
/// ana keeps the group (key 1); bo has two computers (2 and 4); cy has 3.
fn roster() -> Roster {
    Roster::new(BTreeMap::from([
        (Person::parse("ana").unwrap(), vec![actor(1)]),
        (Person::parse("bo").unwrap(), vec![actor(2), actor(4)]),
        (Person::parse("cy").unwrap(), vec![actor(3)]),
    ]))
}
/// `by`'s Remove of `of` at `seq` after `prev`, covering `through`.
fn remove(
    by: u8,
    seq: u64,
    prev: Option<&Op<SignedOp>>,
    ts: i64,
    of: u8,
    through: &[&Op<SignedOp>],
) -> Op<SignedOp> {
    let through = through.iter().map(|o| o.id.clone()).collect();
    let act = RailAct::Remove {
        key: actor(of),
        through,
    };
    sign_act(
        &key(by),
        NS,
        ts,
        seq,
        prev.map(|p| p.id.clone()),
        act,
        None,
        None,
        None,
    )
}
fn fold(ops: &[&Op<SignedOp>]) -> Admission {
    admit(
        &ActSet::of(ops.iter().copied()),
        &[],
        &roster(),
        NS,
        &Ed25519Verifier,
    )
}
fn stands(a: &Admission, n: u8) -> bool {
    a.membership().standing.contains(&actor(n))
}
fn show(what: &str, a: &Admission) {
    let who = |n: u8| format!("{n}:{}", if stands(a, n) { "stands" } else { "out" });
    let all: Vec<_> = (1..=4).map(who).collect();
    eprintln!(
        "{what}: {all:?} cut={:?}",
        a.membership().cut.keys().collect::<Vec<_>>()
    );
}

/// A person's own removal, as svrngs's Leave writes it from bo's computer 2:
/// a Remove of 4, then of 2. Computer 4, which never saw it, signs a Remove
/// of 2 on its own chain.
#[test]
fn a_device_its_person_removed_stays_out_after_a_back_dated_remove_of_its_remover() {
    let other = remove(2, 0, None, 10, 4, &[]);
    let own = remove(2, 1, Some(&other), 11, 2, &[]);
    let left = fold(&[&other, &own]);
    show("own: bo's Leave from 2", &left);
    assert!(!stands(&left, 4) && !stands(&left, 2), "control: bo left");
    let back = remove(4, 0, None, 5, 2, &[]);
    let after = fold(&[&other, &own, &back]);
    show("own: + 4's back-dated Remove of 2", &after);
    assert!(!stands(&after, 4), "4 stands again");
}

/// A keeper cut, as svrngs's Removal writes it: every standing key of bo.
/// Computer 4 signs back-dated Removes of the keeper and of bo's other key.
#[test]
fn a_keeper_s_cut_holds_against_the_cut_device_s_back_dated_removes() {
    let cut = remove(1, 0, None, 10, 4, &[]);
    let cut_two = remove(1, 1, Some(&cut), 11, 2, &[]);
    let removed = fold(&[&cut, &cut_two]);
    show("cut: the keeper removes bo", &removed);
    assert!(
        !stands(&removed, 4) && !stands(&removed, 2),
        "control: bo cut"
    );
    let of_keeper = remove(4, 0, None, 5, 1, &[]);
    let of_own = remove(4, 1, Some(&of_keeper), 6, 2, &[]);
    let after = fold(&[&cut, &cut_two, &of_keeper, &of_own]);
    show("cut: + 4's back-dated Removes of 1 and 2", &after);
    assert!(stands(&after, 1), "the keeper stands");
    assert!(!stands(&after, 4) && !stands(&after, 2), "bo stands again");
}

/// bo's computer 2 removes 4; the keeper then cuts 2, covering that Remove.
/// Computer 4 signs a back-dated Remove of 2.
#[test]
fn a_device_removed_by_a_device_the_keeper_cut_stays_out_after_removing_it() {
    let other = remove(2, 0, None, 10, 4, &[]);
    let cut = remove(1, 0, None, 11, 2, &[&other]);
    let before = fold(&[&other, &cut]);
    show("mixed: 2 removes 4, keeper cuts 2", &before);
    assert!(
        !stands(&before, 4) && !stands(&before, 2),
        "control: both out"
    );
    let back = remove(4, 0, None, 5, 2, &[]);
    let after = fold(&[&other, &cut, &back]);
    show("mixed: + 4's back-dated Remove of 2", &after);
    assert!(!stands(&after, 4), "4 stands again");
}

/// bo leaves from both computers at once, each by svrngs's Leave: a Remove
/// of the other computer, then of itself. No line is crafted.
#[test]
fn two_leaves_of_one_person_at_once_leave_every_key_of_theirs_out() {
    let two_other = remove(2, 0, None, 10, 4, &[]);
    let two_own = remove(2, 1, Some(&two_other), 11, 2, &[]);
    let four_other = remove(4, 0, None, 10, 2, &[]);
    let four_own = remove(4, 1, Some(&four_other), 11, 4, &[]);
    let both = fold(&[&two_other, &two_own, &four_other, &four_own]);
    show("two leaves", &both);
    assert!(
        !stands(&both, 2) && !stands(&both, 4),
        "bo stands after two leaves"
    );
}
