// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai oplog-types/src/lib.rs@e1a43bcd2
//! The pure envelope of an append-only journal — the types, no I/O.
//!
//! Split from `oplog` on 2026-09-23 at the type/writer boundary
//! (ROOT_CAUSE_FIXES B4): `Op`, `OpId`, `SkippedLine` and [`Journaled`] live
//! here because a fold-only consumer — `commonwealth-rail-core`, the ring
//! rail's canon — must LINK NO FILESYSTEM CODE AT ALL, and the `Oplog<K>`
//! writer in `oplog` is filesystem code. `oplog` re-exports everything here,
//! so its consumers' imports did not change.
//!
//! The purity is a contract three ways, not a hope: `purity-gate` censuses
//! these sources for fs/net/clock reads and keeps the closure clean,
//! `quality/ARCH_LAYERS.toml` forbids `commonwealth-rail-core -> oplog` as a
//! dependency edge, and this crate carries NO `workspace-hack` — which would
//! drag the whole workspace's feature union (and its IO crates) straight
//! back into the closure.
//!
//! Everything else in the module you are holding is verbatim from `oplog`'s
//! split day; its history and the three-tenant story are `oplog`'s module
//! docs.

use alloc::string::String;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::hash::ContentHash;

/// What a tenant must declare to get a journal: where its lines live, which
/// line format the current writer emits, and the prefix its ids wear.
///
/// A trait rather than three constructor arguments so the three facts cannot
/// be supplied inconsistently at two call sites of the same log (ARCH §10.6),
/// and so `Oplog::<K>::new(dir)` needs nothing but the directory.
pub trait Journaled: Serialize + DeserializeOwned {
    /// Basename of the JSONL file, joined onto the directory given to
    /// `Oplog::new`.
    const FILE: &'static str;
    /// Short, stable id prefix — `"gov"`, `"recon"`, `"bridge"`. Part of the
    /// hashed input, so ids from two tenants can never collide even if the
    /// same body were written at the same second by the same actor.
    const ID_PREFIX: &'static str;
    /// Line format version this build writes for entries in the base
    /// format. Bump only when a reader must opt in to new semantics;
    /// [`Oplog::read_all`] skips lines declaring a higher `v` rather than
    /// silently misreading them.
    const VERSION: u32 = 1;
    /// The highest line format version this build understands on read.
    /// Lines declaring more are skipped as [`SkippedLine::NewerVersion`]
    /// BEFORE their body is parsed — an act whose KIND this build has never
    /// seen must be named NewerVersion, never Malformed (leg 5 of
    /// `ra-membership-is-order-free`).
    const UNDERSTANDS: u32 = Self::VERSION;
    /// The version stamp this entry carries. A kind whose semantics came
    /// after the base format returns the bumped version, so an un-upgraded
    /// node names it NewerVersion and nothing else changes byte.
    fn line_version(&self) -> u32 {
        Self::VERSION
    }
    /// Short label for tracing and error text (`"governance_oplog"`).
    const LABEL: &'static str;
}

// ── Identity ─────────────────────────────────────────────────

/// Stable, content-addressed id for one op.
///
/// svrngs (U2, DESIGN §6 "Ids cover everything signed"): the WHOLE BLAKE3
/// hash, spelled as 64 hex on the wire, where upstream kept a `ring-` prefix
/// and 64 bits of it — a prefix `ContentHash::short` itself calls "never an
/// identity", and two acts one writer grinds to share it (about 2^32 hashes)
/// were held one or the other by whichever a copy saw first. A string that is
/// not 64 hex does not parse, so a line carrying one is unreadable rather
/// than an act with a second spelling of identity.
///
/// A ring act's id is minted by [`crate::act_id`]; [`Op::new`] is the
/// generic envelope's, for the journal's own tests. `from_hex` reads an id
/// back off a line or an argument, and is named so a reader sees no hashing
/// happened here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OpId(ContentHash);

impl OpId {
    pub(crate) fn from_hash(h: ContentHash) -> Self {
        Self(h)
    }
    /// `None` on anything that is not exactly 64 hex characters.
    pub fn from_hex(s: &str) -> Option<Self> {
        ContentHash::from_hex(s).map(Self)
    }
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

impl core::fmt::Display for OpId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

// ── The envelope ─────────────────────────────────────────────

/// One line in an `Oplog` — an act (`kind`) plus its provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "K: DeserializeOwned"))]
pub struct Op<K> {
    /// Content-addressed op id (see [`OpId`]). Stable across replays.
    pub id: OpId,
    /// Line format version. Always written; the read-side gate skips lines
    /// declaring a version this build does not understand.
    pub v: u32,
    /// When the act happened (Unix seconds).
    pub ts_unix: i64,
    /// Who performed it. `human:<name>` for an adjudication a person made,
    /// a machine label (`"ingest"`, `"reconcile:multi-origin"`) otherwise.
    /// Tenants that require human attribution enforce it themselves — see
    /// `governance::first_unattended_act`.
    pub actor: String,
    #[serde(flatten)]
    pub kind: K,
}

/// A line present in the journal that this build did not turn into an [`Op`].
///
/// Not an error — the read still succeeds, and every other line is returned.
/// It is the *reportable absence*: the caller now holds the evidence that its
/// answer is derived from a subset, and can decide whether that matters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkippedLine {
    /// This build could not parse the line at all.
    Malformed { line: u64, error: String },
    /// The line declares a format version newer than `K::VERSION`, so reading
    /// it would be guessing at semantics this build does not have.
    NewerVersion { line: u64, v: u32 },
}
