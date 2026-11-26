//! The cases driven by hand, each on copies a caller builds and judged under
//! a rule for a set above a missing re-add (`rule.rs`):
//! - `minimal`: the minimal case of the split (svrn-docs APP_STATE.md §8,
//!   "Question 9, in full");
//! - `two_removals`: the re-add is another device's, and the device that
//!   moved the cat after it is removed too;
//! - `defaults`: the minimal case on a column with a DEFAULT.

use std::collections::BTreeSet;

use crate::change::{ChangeSet, RowKey, SetId, Site, State, Val, pack};
use crate::live::{kept_ids, named};
use crate::reference;
use crate::replica::{Op, Replica};
use crate::rule::Rule;
use crate::sim::TABLES;

/// The copies of the cases.
pub const A: usize = 0;
pub const B: usize = 1;
pub const C: usize = 2;

/// The cases, to run each in turn.
#[derive(Clone, Copy, Debug)]
pub enum Case {
    Minimal,
    TwoRemovals,
    Defaults,
}

impl Case {
    pub const ALL: [Case; 3] = [Case::Minimal, Case::TwoRemovals, Case::Defaults];

    pub fn name(self) -> &'static str {
        match self {
            Case::Minimal => "the minimal case",
            Case::TwoRemovals => "two removals",
            Case::Defaults => "column defaults",
        }
    }

    /// The copies standing at its end, each by its index and name.
    pub fn standing(self) -> &'static [(usize, &'static str)] {
        match self {
            Case::Minimal | Case::Defaults => &[(A, "A"), (B, "B")],
            Case::TwoRemovals => &[(B, "B")],
        }
    }

    /// The case on copies `make` builds, judged under `rule`: the copies,
    /// what each ends on, and W1's verdict.
    pub fn run<R: Replica>(self, rule: Rule, make: &dyn Fn(Site) -> R) -> (Vec<R>, String, Result<(), String>) {
        match self {
            Case::Minimal => minimal(rule, make),
            Case::TwoRemovals => two_removals(rule, make),
            Case::Defaults => defaults(rule, make),
        }
    }
}

pub fn cat() -> Vec<Val> {
    vec![Val::text("cat")]
}

pub fn tile() -> Vec<Val> {
    vec![Val::text("tile")]
}

/// The row a case moves, by its table and key, and its name on the board.
pub struct Thing {
    row: RowKey,
    name: &'static str,
}

impl Thing {
    pub fn cat() -> Thing {
        Thing {
            row: ("magnets".to_owned(), pack(&cat())),
            name: "cat",
        }
    }

    pub fn tile() -> Thing {
        Thing {
            row: ("tiles".to_owned(), pack(&tile())),
            name: "tile",
        }
    }
}

/// A case's thing as a person sees it on the board: `{pos 5, word milk}`,
/// `{}` for one with no column set, or `no cat`.
fn board(state: &State, thing: &Thing) -> String {
    let Some(row) = state.get(&thing.row) else {
        return format!("no {}", thing.name);
    };
    let cols: Vec<String> = row
        .iter()
        .map(|(col, v)| match v {
            Val::Integer(i) => format!("{col} {i}"),
            Val::Text(t) => format!("{col} {t}"),
            other => format!("{col} {other:?}"),
        })
        .collect();
    format!("{{{}}}", cols.join(", "))
}

fn add_cat() -> Op {
    Op::insert(
        "magnets",
        cat(),
        vec![("pos", Val::Integer(1)), ("word", Val::text("milk"))],
    )
}

fn delete_cat() -> Op {
    Op::Delete {
        table: "magnets",
        key: cat(),
    }
}

fn move_cat(pos: i64) -> Op {
    Op::Update {
        table: "magnets",
        key: cat(),
        set: vec![("pos", Val::Integer(pos))],
    }
}

/// Each removed device's computer is cut off, and every copy is told.
fn remove<R: Replica>(copies: &mut [R], gone: &[usize]) {
    for &g in gone {
        let site = copies[g].site();
        for copy in copies.iter_mut() {
            copy.removed(site);
        }
    }
}

/// The standing copies meet in full: each takes every set the other keeps,
/// then adopts its marks, until none changes.
fn meet<R: Replica>(copies: &mut [R], standing: &[usize]) {
    let now = |copies: &[R]| -> Vec<_> {
        standing
            .iter()
            .map(|&c| (copies[c].state(), kept_ids(&copies[c]), copies[c].marks()))
            .collect()
    };
    for round in 0.. {
        let before = now(copies);
        for &from in standing {
            for &to in standing.iter().filter(|&&to| to != from) {
                for set in copies[from].kept() {
                    copies[to].take(&set);
                }
                let marks = copies[from].marks();
                copies[to].adopt(&marks);
            }
        }
        if now(copies) == before {
            break;
        }
        assert!(round < 10, "the meet settles");
    }
}

/// W1 on a case's standing copies, said by the thing it moves: they agree,
/// each is the reference's under `rule` over the sets they hold, and a
/// newcomer synced from each, its marks adopted and its sets in kept order
/// and reversed, ends equal to it and to the reference.
fn judged<R: Replica>(
    rule: Rule,
    make: &dyn Fn(Site) -> R,
    copies: &[R],
    (thing, standing): (&Thing, &[(usize, &str)]),
    (newcomer, site): (&str, Site),
    (over, admissible): (&str, &[&ChangeSet]),
) -> (String, Result<(), String>) {
    let shown = |state: &State| board(state, thing);
    let want = reference::under(rule, TABLES, admissible.iter().copied());
    let names = |ids: &BTreeSet<SetId>| ids.iter().map(|id| named(*id)).collect::<Vec<_>>();
    let mut said = vec![format!("  the reference over {over}: {}", shown(&want))];
    let mut wrong = Vec::new();
    for &(c, name) in standing {
        let (state, kept) = (copies[c].state(), kept_ids(&copies[c]));
        let held = copies[c].held();
        let holding = match held.is_empty() {
            true => String::new(),
            false => format!(", holding {:?}", names(&held)),
        };
        said.push(format!(
            "  {name}: {}, keeping {:?}{holding}, its clock rows naming {:?}",
            shown(&state),
            names(&kept),
            names(&copies[c].standing())
        ));
        if state != want {
            wrong.push(format!("{name} is not the reference's"));
        }
        let mut sets = copies[c].kept();
        for order in ["kept order", "reversed"] {
            let mut fresh = make(site);
            fresh.adopt(&copies[c].marks());
            for set in &sets {
                fresh.take(set);
            }
            let got = fresh.state();
            said.push(format!("    {newcomer} synced from {name}, {order}: {}", shown(&got)));
            if got != state {
                wrong.push(format!("{newcomer} synced from {name}, {order}, is not equal to it"));
            }
            if got != want {
                wrong.push(format!(
                    "{newcomer} synced from {name}, {order}, is not the reference's"
                ));
            }
            sets.reverse();
        }
    }
    if let [(first, one), rest @ ..] = standing {
        for &(c, other) in rest {
            if copies[c].state() != copies[*first].state() {
                wrong.insert(0, format!("{one} and {other} split"));
            }
        }
    }
    let verdict = if wrong.is_empty() {
        Ok(())
    } else {
        Err(wrong.join("; "))
    };
    (said.join("\n"), verdict)
}

/// The minimal case of the split (svrn-docs APP_STATE.md §8, "Question 9, in
/// full"), driven by hand on copies `make` builds. A (site 1) adds the cat
/// with pos 1 and word milk, and C (site 3) takes it. C deletes the cat, adds
/// it back with pos 1, and moves it to 5. A takes the move; B (site 2) takes
/// the move, then A's add. C is removed, so its delete and re-add never reach
/// the group, and A and B meet in full. Returns the copies, what each ends
/// on, and W1's verdict under `rule` (`judged`), over the sets A and B hold,
/// A's add and C's move.
pub fn minimal<R: Replica>(rule: Rule, make: &dyn Fn(Site) -> R) -> (Vec<R>, String, Result<(), String>) {
    let mut copies: Vec<R> = (1..=3).map(|i| make(Site::of(i))).collect();
    let a1 = copies[A].write(&[add_cat()]).expect("A adds the cat");
    copies[C].take(&a1);
    copies[C].write(&[delete_cat()]).expect("C deletes it");
    let readd = Op::insert("magnets", cat(), vec![("pos", Val::Integer(1))]);
    copies[C].write(&[readd]).expect("C adds it back");
    let c3 = copies[C].write(&[move_cat(5)]).expect("C moves it");
    copies[A].take(&c3);
    copies[B].take(&c3);
    copies[B].take(&a1);
    remove(&mut copies, &[C]);
    meet(&mut copies, &[A, B]);
    let (said, verdict) = judged(
        rule,
        make,
        &copies,
        (&Thing::cat(), Case::Minimal.standing()),
        ("a newcomer", Site::of(250)),
        ("A's add and C's move", &[&a1, &c3]),
    );
    (copies, said, verdict)
}

/// Two removals, the re-add another device's. A (site 1) adds the cat with
/// pos 1 and word milk (a1), and B (site 2) and C (site 3) take it. C deletes
/// the cat (c1) and adds it back with pos 2 (c2). A takes both and moves the
/// cat to 5 (a2). Only a2 reaches B, which holds the cat alive from a1. Then
/// A and C are removed, so neither c1 nor c2 ever reaches B, and a newcomer,
/// D (site 4), syncs from B. Returns the copies, what each ends on, and W1's
/// verdict under `rule` (`judged`), over the sets B holds, a1 and a2. A
/// device's own sets carry it through under P: a2 follows a1, A's last, so it
/// is A's next, whatever C's it followed.
pub fn two_removals<R: Replica>(rule: Rule, make: &dyn Fn(Site) -> R) -> (Vec<R>, String, Result<(), String>) {
    let mut copies: Vec<R> = (1..=3).map(|i| make(Site::of(i))).collect();
    let a1 = copies[A].write(&[add_cat()]).expect("A adds the cat");
    copies[B].take(&a1);
    copies[C].take(&a1);
    let c1 = copies[C].write(&[delete_cat()]).expect("C deletes it");
    let readd = Op::insert("magnets", cat(), vec![("pos", Val::Integer(2))]);
    let c2 = copies[C].write(&[readd]).expect("C adds it back");
    copies[A].take(&c1);
    copies[A].take(&c2);
    let a2 = copies[A].write(&[move_cat(5)]).expect("A moves it");
    copies[B].take(&a2);
    remove(&mut copies, &[A, C]);
    let (said, verdict) = judged(
        rule,
        make,
        &copies,
        (&Thing::cat(), Case::TwoRemovals.standing()),
        ("D", Site::of(4)),
        ("a1 and a2", &[&a1, &a2]),
    );
    (copies, said, verdict)
}

/// Column defaults: the minimal case on a tile, whose `size` is NOT NULL
/// with a DEFAULT of 3 (`sim::SCHEMA`), so every insert names it, as the
/// write form asks (APP_STATE §4, "The write form"). A (site 1) adds the
/// tile at pos 1, size 7, and C (site 3) takes it. C deletes the tile, adds
/// it back at pos 1, size 5, and moves it to 5. A takes the move; B (site 2)
/// takes the move, then A's add. C is removed, so its delete and re-add never
/// reach the group, and A and B meet in full. The move brings the tile back
/// where its re-add is missing: B makes it from its key, its size the
/// default (`tableinfo.rs:391-406`). Under N, A clears its earlier life,
/// size 7, to the same; under today's rule A keeps it. C's size 5 went with
/// its re-add, and no copy can show it. Returns the copies, what each ends
/// on, and W1's verdict under `rule` (`judged`), over the sets A and B hold,
/// A's add and C's move.
pub fn defaults<R: Replica>(rule: Rule, make: &dyn Fn(Site) -> R) -> (Vec<R>, String, Result<(), String>) {
    let mut copies: Vec<R> = (1..=3).map(|i| make(Site::of(i))).collect();
    let add = Op::insert(
        "tiles",
        tile(),
        vec![("pos", Val::Integer(1)), ("size", Val::Integer(7))],
    );
    let a1 = copies[A].write(&[add]).expect("A adds the tile");
    copies[C].take(&a1);
    let delete = Op::Delete {
        table: "tiles",
        key: tile(),
    };
    copies[C].write(&[delete]).expect("C deletes it");
    let readd = Op::insert(
        "tiles",
        tile(),
        vec![("pos", Val::Integer(1)), ("size", Val::Integer(5))],
    );
    copies[C].write(&[readd]).expect("C adds it back");
    let to_5 = Op::Update {
        table: "tiles",
        key: tile(),
        set: vec![("pos", Val::Integer(5))],
    };
    let c3 = copies[C].write(&[to_5]).expect("C moves it");
    let own = board(&copies[C].state(), &Thing::tile());
    copies[A].take(&c3);
    copies[B].take(&c3);
    copies[B].take(&a1);
    remove(&mut copies, &[C]);
    meet(&mut copies, &[A, B]);
    let (said, verdict) = judged(
        rule,
        make,
        &copies,
        (&Thing::tile(), Case::Defaults.standing()),
        ("a newcomer", Site::of(250)),
        ("A's add and C's move", &[&a1, &c3]),
    );
    (
        copies,
        format!("  C's own board before its removal: {own}\n{said}"),
        verdict,
    )
}
