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
    /// The set of `ops`, however ordered or repeated. Collecting keeps the
    /// last of identical lines where `insert` keeps the first: the same
    /// choice, since identical keys are identical serialised lines.
    pub fn of<'a>(ops: impl IntoIterator<Item = &'a Op<SignedOp>>) -> Self {
        let keyed = ops.into_iter().map(|op| (line_key(op), op.clone()));
        Self(keyed.collect())
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
