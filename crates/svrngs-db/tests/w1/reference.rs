//! The reference model: the table state that a set of admissible change
//! sets, taken in any order, merges to. Written apart from any svrngs-db
//! code, from the rule as APP_STATE §4 states it ("How it merges"; "A
//! removed device", "Why"), with Fly's fork read where the rule leaves a case
//! unsaid (`changes_vtab_write.rs`, at the pin `change.rs` names):
//!
//! - **A row's causal length is the highest any change of it carries.** Odd,
//!   the row exists; even, it was deleted. Parity decides, whatever the
//!   change's column (`:515`), and a change at a lower length decides
//!   nothing (`:511-513`).
//! - **A column's value is its winning change's, among the row's changes at
//!   that length:** the highest `col_version` (`:58-62`), then the larger
//!   value in cr-sqlite's order (`:93`, `:108`; `Val::cmp_crsql`), then the
//!   larger site (`:95-106`, with merge-equal-values on, as svrngs sets it).
//!   The site only names which of two equal values' sets stands.
//! - **A column no change at that length reaches is NULL,** as on a copy
//!   that creates the row from a set at that length: it inserts the key
//!   alone (`:620-640`; `tableinfo.rs:391-406`).
//!
//! So the state is a join: per row, the pair (causal length, each column's
//! highest change at it), where a longer length replaces the columns and an
//! equal one takes each column's maximum. It is commutative, associative and
//! idempotent (`laws.rs`).
//!
//! **Where cr-sqlite's merge equals it.** A copy that held a row alive keeps
//! its old values when a change at a longer odd length arrives, each at
//! `col_version` 0 (`tableinfo.rs:440-454`, called at
//! `changes_vtab_write.rs:273`), and only a change at the new length replaces
//! one; a copy without the row makes it from the key alone. So a copy ends on
//! this model's state when it has taken, at the row's length, a change of
//! every non-key column. The set that re-adds a row carries one of each: a
//! local insert clocks every non-key column (`local_writes/mod.rs`,
//! `mark_locally_inserted`).

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::change::{Change, ChangeSet, RowKey, SENTINEL, State, Val};

/// The join of every change taken so far.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Join {
    rows: BTreeMap<RowKey, Line>,
}

/// One row's part of the join: its causal length, and each non-key column's
/// winning change at that length.
#[derive(Clone, Debug, PartialEq)]
struct Line {
    cl: i64,
    cells: BTreeMap<String, Change>,
}

impl Line {
    fn of(c: &Change) -> Line {
        let mut cells = BTreeMap::new();
        if c.cid != SENTINEL && c.cl % 2 == 1 {
            cells.insert(c.cid.clone(), c.clone());
        }
        Line { cl: c.cl, cells }
    }

    /// This line joined with another: the longer causal length's columns, or
    /// at one length each column's higher change.
    fn join(&mut self, other: &Line) {
        match other.cl.cmp(&self.cl) {
            Ordering::Less => {}
            Ordering::Greater => *self = other.clone(),
            Ordering::Equal => {
                for (cid, c) in &other.cells {
                    match self.cells.get(cid) {
                        Some(held) if !outranks(c, held) => {}
                        _ => {
                            self.cells.insert(cid.clone(), c.clone());
                        }
                    }
                }
            }
        }
    }
}

/// Whether `a` beats `b` for one column at one causal length: the most
/// edits, then the larger value, then the larger site.
fn outranks(a: &Change, b: &Change) -> bool {
    a.col_version
        .cmp(&b.col_version)
        .then_with(|| a.val.cmp_crsql(&b.val))
        .then_with(|| a.site.cmp(&b.site))
        == Ordering::Greater
}

impl Join {
    /// The join of these sets' changes.
    pub fn of<'a>(sets: impl IntoIterator<Item = &'a ChangeSet>) -> Join {
        let mut join = Join::default();
        for set in sets {
            join.add(set);
        }
        join
    }

    /// This join, having also taken `set`.
    pub fn add(&mut self, set: &ChangeSet) {
        for c in set.changes() {
            let line = Line::of(c);
            match self.rows.get_mut(&(c.table.clone(), c.pk.clone())) {
                Some(held) => held.join(&line),
                None => {
                    self.rows.insert((c.table.clone(), c.pk.clone()), line);
                }
            }
        }
    }

    /// This join joined with another.
    pub fn join(mut self, other: &Join) -> Join {
        for (key, line) in &other.rows {
            match self.rows.get_mut(key) {
                Some(held) => held.join(line),
                None => {
                    self.rows.insert(key.clone(), line.clone());
                }
            }
        }
        self
    }

    /// The table state: the rows at an odd causal length, each with its
    /// winning values that are not NULL.
    pub fn state(&self) -> State {
        self.rows
            .iter()
            .filter(|(_, line)| line.cl % 2 == 1)
            .map(|(key, line)| {
                let values = line
                    .cells
                    .iter()
                    .filter(|(_, c)| c.val != Val::Null)
                    .map(|(cid, c)| (cid.clone(), c.val.clone()))
                    .collect();
                (key.clone(), values)
            })
            .collect()
    }
}

/// The reference model: the state these admissible sets merge to, in any
/// order.
pub fn state<'a>(sets: impl IntoIterator<Item = &'a ChangeSet>) -> State {
    Join::of(sets).state()
}
