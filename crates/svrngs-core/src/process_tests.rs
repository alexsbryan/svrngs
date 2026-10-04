// SPDX-License-Identifier: AGPL-3.0-or-later
//! The process managers' door: an effect is due only from acts that stand,
//! and a seal is the cause of the retirement it sets.

use super::sealed::Sealed;
use crate::tests_support::*;
use crate::*;

/// A manager that names whatever causes it was built with.
struct Names(Vec<OpId>);

impl Sealed for Names {}

impl ProcessManager for Names {
    type Effect = ();

    fn react(&self, _: &Admission) -> Option<((), Vec<OpId>)> {
        Some(((), self.0.clone()))
    }
}

#[test]
fn an_effect_is_due_only_from_acts_that_stand_in_the_record() {
    let kept = signed(&key(1), 100, 0, record("kept"));
    let wrong = after(&key(1), &kept, 101, record("wrong"));
    let correct = after(
        &key(1),
        &wrong,
        102,
        RailAct::Correct {
            corrects: wrong.id.clone(),
            replacement: None,
        },
    );
    let a = admitted(&[kept.clone(), wrong.clone(), correct]);
    assert!(a.is_complete(), "{:?}", a.gaps());

    let due = a
        .react(&Names(vec![kept.id.clone()]))
        .unwrap()
        .expect("control: an act that stands causes an effect");
    assert_eq!(due.causes(), [kept.id.clone()]);
    assert_eq!(
        a.react(&Names(Vec::new())).unwrap_err(),
        Uncaused(Vec::new()),
        "no cause"
    );
    assert!(
        a.react(&Names(vec![stray_id(9)])).is_err(),
        "a cause the record does not hold"
    );
    assert!(
        a.react(&Names(vec![wrong.id.clone()])).is_err(),
        "a cause a correction voided"
    );
}

#[test]
fn nothing_sealed_is_nothing_due() {
    let a = admitted(&[signed(&key(1), 100, 0, record("a"))]);
    assert!(matches!(a.react(&Compact), Ok(None)));
}

/// The seal at seq 2 sets the floor: seq 1 retires, the line a correction
/// above the floor names stays, and so do the seal and what follows it.
#[test]
fn a_seal_is_the_cause_of_the_retirement_it_sets() {
    let a0 = signed(&key(1), 100, 0, record("named"));
    let b1 = after(&key(1), &a0, 101, record("retired"));
    let seal = after(&key(1), &b1, 102, RailAct::Seal);
    let fix = after(
        &key(1),
        &seal,
        103,
        RailAct::Correct {
            corrects: a0.id.clone(),
            replacement: None,
        },
    );
    let lines = [a0, b1, seal.clone(), fix];
    let due = admitted(&lines)
        .react(&Compact)
        .unwrap()
        .expect("a seal is due a retirement");
    assert_eq!(due.causes(), [seal.id.clone()]);
    let kept: Vec<u64> = lines
        .iter()
        .filter(|l| due.effect().keeps(l, NS))
        .map(|l| l.kind.seq)
        .collect();
    assert_eq!(kept, [0, 2, 3]);
}
