// SPDX-License-Identifier: AGPL-3.0-or-later
//! The set's own laws, which every convergence test assumes and none pinned
// (the test audit of 2026-11-16, demo/runs/test-audit/): what `ActSet`
//! promises in its doc comment — arrival order and repetition cannot be
//! written down — is a property of this type, not of admission, and a
//! regression here would reach every rail at once through `admit`.

use crate::*;

use crate::tests_support::*;

/// **The property the type exists for.** The same lines, given in another
/// order or said twice, are the same set: no permutation or repetition can
/// be written down. If this fails, two nodes gossiping in different orders
/// hold different journals and every convergence test below inherits the
/// fault without naming it.
#[test]
fn neither_arrival_order_nor_repetition_can_be_written_down() {
    let one = signed(&key(1), 100, 0, record("one"));
    let two = signed(&key(2), 101, 0, record("two"));
    let three = after(&key(1), &one, 102, record("three"));
    let plain = ActSet::of([&one, &two, &three]);
    for order in [
        vec![&three, &one, &two],
        vec![&two, &three, &one],
        // The same lines, each said twice, as a redelivery would.
        vec![&one, &two, &one, &three, &two, &three],
    ] {
        assert_eq!(
            ActSet::of(order),
            plain,
            "a different telling is the same set"
        );
    }
    assert_eq!(plain.len(), 3);
}

/// Two copies merge by union: a line either copy held survives the merge,
/// and a line both held is held once. `insert` says `false` only for the
/// exact line already there — never for a line that merely resembles it.
#[test]
fn two_copies_merge_by_union_and_only_the_exact_line_is_a_duplicate() {
    let ours = signed(&key(1), 100, 0, record("ours"));
    let yours = signed(&key(2), 101, 0, record("yours"));
    let shared = signed(&key(3), 102, 0, record("shared"));
    let mut a = ActSet::of([&ours, &shared]);
    let b = ActSet::of([&yours, &shared]);

    assert!(a.insert(yours.clone()), "a line only they held is added");
    assert!(
        !a.insert(shared.clone()),
        "the exact line already held is not"
    );
    // The union from either side: each copy's lines inserted into the other.
    let from_ours = {
        let mut m = b.clone();
        for op in a.iter() {
            m.insert(op.clone());
        }
        m
    };
    assert_eq!(a, from_ours, "the merge is the union");
    assert_eq!(a.len(), 3, "the shared line is held once");
}
