//! The cases driven by hand, each on copies a caller builds:
//! - `minimal`: the minimal case of the split (svrn-docs APP_STATE.md §8,
//!   "Question 9, in full").

use std::collections::BTreeSet;

use crate::change::{SetId, Site, State, Val, pack};
use crate::live::{kept_ids, named};
use crate::reference;
use crate::replica::{Op, Replica};

/// The copies of the minimal case.
pub const A: usize = 0;
pub const B: usize = 1;
pub const C: usize = 2;

pub fn cat() -> Vec<Val> {
    vec![Val::text("cat")]
}

/// The cat's row in a state, said.
pub fn cat_row(state: &State) -> String {
    match state.get(&("magnets".to_owned(), pack(&cat()))) {
        Some(row) => format!("{row:?}"),
        None => "no row".to_owned(),
    }
}

/// The minimal case of the split (svrn-docs APP_STATE.md §8, "Question 9, in
/// full"), driven by hand on copies `make` builds. A (site 1) adds the cat
/// with pos 1 and word milk, and C (site 3) takes it. C deletes the cat, adds
/// it back with pos 1, and moves it to 5. A takes the move; B (site 2) takes
/// the move, then A's add. C is removed, so its delete and re-add never reach
/// the group, and A and B meet in full. Returns the copies, what each ends
/// on, and W1's verdict: A and B agree, each is the reference's over the
/// sets they hold, A's add and C's move, and a newcomer synced from each,
/// its sets in kept order and reversed, ends equal to it.
pub fn minimal<R: Replica>(make: &dyn Fn(Site) -> R) -> (Vec<R>, String, Result<(), String>) {
    let mut copies: Vec<R> = (1..=3).map(|i| make(Site::of(i))).collect();
    let add = Op::insert(
        "magnets",
        cat(),
        vec![("pos", Val::Integer(1)), ("word", Val::text("milk"))],
    );
    let a1 = copies[A].write(&[add]).expect("A adds the cat");
    copies[C].take(&a1);
    let delete = Op::Delete {
        table: "magnets",
        key: cat(),
    };
    copies[C].write(&[delete]).expect("C deletes it");
    let readd = Op::insert("magnets", cat(), vec![("pos", Val::Integer(1))]);
    copies[C].write(&[readd]).expect("C adds it back");
    let mv = Op::Update {
        table: "magnets",
        key: cat(),
        set: vec![("pos", Val::Integer(5))],
    };
    let c3 = copies[C].write(&[mv]).expect("C moves it");
    copies[A].take(&c3);
    copies[B].take(&c3);
    copies[B].take(&a1);
    let gone = copies[C].site();
    for copy in &mut copies {
        copy.removed(gone);
    }
    let now = |copies: &[R]| [A, B].map(|c| (copies[c].state(), kept_ids(&copies[c])));
    for round in 0.. {
        let before = now(&copies);
        for (from, to) in [(A, B), (B, A)] {
            for set in copies[from].kept() {
                copies[to].take(&set);
            }
        }
        if now(&copies) == before {
            break;
        }
        assert!(round < 10, "the meet settles");
    }
    let want = reference::state([&a1, &c3]);
    let mut said = vec![format!("  the reference over A's add and C's move: {}", cat_row(&want))];
    let mut wrong = Vec::new();
    for (c, name) in [(A, "A"), (B, "B")] {
        let (state, kept) = (copies[c].state(), kept_ids(&copies[c]));
        let names = |ids: &BTreeSet<SetId>| ids.iter().map(|id| named(*id)).collect::<Vec<_>>();
        said.push(format!(
            "  {name}: {}, keeping {:?}, its clock rows naming {:?}",
            cat_row(&state),
            names(&kept),
            names(&copies[c].standing())
        ));
        if state != want {
            wrong.push(format!("{name} is not the reference's"));
        }
        let mut sets = copies[c].kept();
        for order in ["kept order", "reversed"] {
            let mut newcomer = make(Site::of(250));
            for set in &sets {
                newcomer.take(set);
            }
            said.push(format!(
                "    a newcomer synced from {name}, {order}: {}",
                cat_row(&newcomer.state())
            ));
            if newcomer.state() != state {
                wrong.push(format!("a newcomer synced from {name}, {order}, is not equal to it"));
            }
            sets.reverse();
        }
    }
    if copies[A].state() != copies[B].state() {
        wrong.insert(0, "A and B split".to_owned());
    }
    let verdict = if wrong.is_empty() {
        Ok(())
    } else {
        Err(wrong.join("; "))
    };
    (copies, said.join("\n"), verdict)
}
