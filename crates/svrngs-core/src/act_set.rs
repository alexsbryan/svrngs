// SPDX-License-Identifier: AGPL-3.0-or-later
//! The acts a computer holds for one group, as a set: arrival order and
//! repetition cannot be written down, and two copies merge by union.

use alloc::collections::BTreeMap;
use alloc::string::String;

use crate::hash::ContentHash;
use crate::op::Op;
use crate::SignedOp;

/// A grow-only set of journal lines, keyed by the hash of each whole line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActSet(BTreeMap<ContentHash, Op<SignedOp>>);

impl ActSet {
    /// The set of `ops`, however ordered or repeated.
    pub fn of<'a>(ops: impl IntoIterator<Item = &'a Op<SignedOp>>) -> Self {
        let mut set = Self::default();
        for op in ops {
            set.insert(op.clone());
        }
        set
    }

    /// Adds `op`; `false` when the set already held that exact line.
    pub fn insert(&mut self, op: Op<SignedOp>) -> bool {
        let key = line_key(&op);
        if self.0.contains_key(&key) {
            return false;
        }
        self.0.insert(key, op);
        true
    }

    /// The union of `self` and `other`.
    pub fn merge(&mut self, other: ActSet) {
        for (key, op) in other.0 {
            self.0.entry(key).or_insert(op);
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Op<SignedOp>> {
        self.0.values()
    }
}

fn line_key(op: &Op<SignedOp>) -> ContentHash {
    let line: String =
        serde_json::to_string(op).expect("a journal line serialises: it was read from or for one");
    ContentHash::of_str(&line)
}
