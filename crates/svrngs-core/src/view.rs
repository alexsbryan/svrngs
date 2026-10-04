// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/view.rs@e1a43bcd2
//! The [`View`] commitment — a bounded proof of *which ops a node holds*.
//!
//! One primitive, three mountings (ARCH §10.6): the sync digest's per-actor
//! entry (so two replicas that diverged at one seq can SEE it and exchange),
//! an act's `view` field (so an equivocation or a cut is weighable from the
//! act itself — `RING_APP_LIBRARY.md` §19 step 0), and the checkpoint
//! document's completeness claim (it carries the digest). The definition
//! below is the wire contract: the golden vectors in the tests are frozen
//! from this implementation and a reimplementation must reproduce them
//! byte-for-byte.
//!
//! The chain is over DERIVED ids (`admit::derived_id` — identity from
//! essence), ids at one seq sorted and deduped, so a tampered `id` field and
//! a second line for the same content fold identically. Known limit,
//! inherited not introduced: `OpId` is 64-bit-truncated BLAKE3 (oplog's
//! `ContentHash::short`), so the chain is as strong as the ids — the same
//! limit every correction target already rides.

use std::collections::{BTreeMap, BTreeSet};

use crate::hash::ContentHash;
use crate::op::Op;
use serde::{Deserialize, Serialize};

use crate::SignedOp;

/// Domain separation and definition version for the fold. A second
/// definition gets a second label; old heads stay meaningful.
const VIEW_DOMAIN: &[u8] = b"cwth/view/1";

/// One actor's commitment: the contiguous run `[from, mark]` being claimed
/// (`from` is the sealed floor, `mark` the contiguous high-water — the same
/// run the mark alone describes) and `head`, the fold of everything held in
/// it. Equal heads over an equal `(from, mark)` mean equal held sets; the
/// preimage resistance of the hash is the whole argument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    pub from: u64,
    pub mark: u64,
    pub head: String,
}

impl View {
    /// Fold the ops held for `actor` in `[from, mark]`.
    ///
    /// A caller evaluating SOMEONE ELSE's claim may hold more or less in the
    /// window (fork branches, holes) — the fold is a pure function of the set
    /// presented, so equal sets fold equal regardless of who computes it.
    pub fn of<'a>(
        actor: &str,
        from: u64,
        mark: u64,
        ops: impl IntoIterator<Item = &'a Op<SignedOp>>,
    ) -> Self {
        let pairs: Vec<(u64, String)> = ops
            .into_iter()
            .filter(|op| op.actor == actor && op.kind.seq >= from && op.kind.seq <= mark)
            .map(|op| (op.kind.seq, crate::admit::derived_id(op).to_string()))
            .collect();
        Self::fold_pairs(actor, from, mark, &pairs)
    }

    /// The fold itself, over `(seq, derived-id)` pairs — the formula the
    /// module docs state and the surface `fixtures/view_golden.json` pins
    /// for a second implementation. Ids at one seq are sorted and deduped;
    /// pairs outside `[from, mark]` are ignored. `of` derives the ids and
    /// delegates here, so the spec formula has exactly one home.
    pub fn fold_pairs(actor: &str, from: u64, mark: u64, pairs: &[(u64, String)]) -> Self {
        let mut by_seq: BTreeMap<u64, BTreeSet<String>> = BTreeMap::new();
        for (seq, id) in pairs {
            if *seq >= from && *seq <= mark {
                by_seq.entry(*seq).or_default().insert(id.clone());
            }
        }
        let mut h = ContentHash::of(&[VIEW_DOMAIN, b"\0", actor.as_bytes()].concat());
        for (seq, ids) in &by_seq {
            for id in ids {
                let mut step = Vec::with_capacity(32 + 8 + 2 + id.len());
                step.extend_from_slice(h.as_bytes());
                step.push(0);
                step.extend_from_slice(&seq.to_be_bytes());
                step.push(0);
                step.extend_from_slice(id.as_bytes());
                h = ContentHash::of(&step);
            }
        }
        View {
            from,
            mark,
            head: h.to_hex(),
        }
    }
}

/// The sync digest: `{actor → View}`, versioned on the wire.
///
/// The version is not decoration (ARCH §18.3): a marks-only digest cannot
/// prove content, so v1 is REFUSED BY NAME rather than read as if it carried
/// a commitment, and an unknown `v` is refused naming the number. Silence and
/// defaults are how a commitment quietly becomes a suggestion.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Digest(BTreeMap<String, View>);

/// The one digest envelope this build speaks.
pub const DIGEST_V: u32 = 2;

impl Digest {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert(&mut self, actor: String, view: View) {
        self.0.insert(actor, view);
    }

    pub fn get(&self, actor: &str) -> Option<&View> {
        self.0.get(actor)
    }

    pub fn contains_key(&self, actor: &str) -> bool {
        self.0.contains_key(actor)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &View)> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn entries(&self) -> &BTreeMap<String, View> {
        &self.0
    }
}

impl From<BTreeMap<String, View>> for Digest {
    fn from(entries: BTreeMap<String, View>) -> Self {
        Self(entries)
    }
}

impl Serialize for Digest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("v", &DIGEST_V)?;
        map.serialize_entry("entries", &self.0)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let raw = serde_json::Value::deserialize(deserializer)?;
        match raw.get("v") {
            Some(v) if v.as_u64() == Some(u64::from(DIGEST_V)) => {
                let entries = raw
                    .get("entries")
                    .cloned()
                    .ok_or_else(|| D::Error::custom("digest v2 carries no `entries`"))?;
                let entries: BTreeMap<String, View> =
                    serde_json::from_value(entries).map_err(D::Error::custom)?;
                Ok(Digest(entries))
            }
            Some(v) => Err(D::Error::custom(format!(
                "digest v{} is not spoken here (this node speaks v{DIGEST_V})",
                v.as_u64()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| v.to_string())
            ))),
            None if raw.as_object().is_some_and(|o| {
                !o.is_empty() && o.values().all(serde_json::Value::is_u64)
            }) =>
            {
                Err(D::Error::custom(
                    "digest v1 (marks-only) is refused: it cannot prove content — both ends must speak v2",
                ))
            }
            None => Err(D::Error::custom("unrecognised digest envelope")),
        }
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
