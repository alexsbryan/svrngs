// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_sync/journal.rs@e1a43bcd2
//! The round's seam: what it reads from the node that runs it (a
//! [`RingSyncHost`]), the journal half it exchanges over (a
//! [`RingSyncJournal`]), one round's membership, and the one roster test both
//! directions of ring sync decide on. A sibling of `ring_sync.rs` so that file
//! stays under the 800-line approach band (ARCH §3.1); re-exported there.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::iroh::{Contact, IrohTransport, PublicKey};
use crate::{Compaction, Digest, Op, RailError, RingRail, Roster, SignedOp};

/// A journal call, boxed so [`RingSyncJournal`] stays object-safe. The same
/// shape as the daemon's rail port future, so a port forwards without a wrap.
pub type JournalFut<'a, T> = Pin<Box<dyn Future<Output = Result<T, RailError>> + Send + 'a>>;

/// The half of a ring rail one round reads and writes: which rings this node
/// holds, each ring's roster, and the digest exchange over its journal.
///
/// Implemented here for the local [`RingRail`] (cw-rails' own journals). The
/// daemon's rail-port implementation retired with pb-mesh-exit-transport, when
/// cw-rails became the one host of the round.
pub trait RingSyncJournal: Send + Sync {
    /// Every namespace this node holds a journal for.
    fn namespaces(&self) -> JournalFut<'_, Vec<String>>;
    /// The namespace's roster, through the rail's one reader.
    fn roster(&self, namespace: &str) -> JournalFut<'_, Roster>;
    /// The per-actor contiguous high-water marks of this node's journal.
    fn journal_digest(&self, namespace: &str) -> JournalFut<'_, Digest>;
    /// Ingest a peer's ops, keyed on their content-addressed ids.
    fn journal_ingest_all(&self, namespace: &str, ops: &[Op<SignedOp>]) -> JournalFut<'_, usize>;
    /// One budget of what `theirs` says the peer lacks, and whether more
    /// remains.
    fn journal_ops_missing_from_within(
        &self,
        namespace: &str,
        theirs: &Digest,
        budget_bytes: usize,
    ) -> JournalFut<'_, (Vec<Op<SignedOp>>, bool)>;
    /// Retire what a seal below the floor authorises.
    fn journal_compact(&self, namespace: &str, roster: &Roster) -> JournalFut<'_, Compaction>;
}

impl RingSyncJournal for RingRail {
    fn namespaces(&self) -> JournalFut<'_, Vec<String>> {
        Box::pin(async move { RingRail::namespaces(self) })
    }

    fn roster(&self, namespace: &str) -> JournalFut<'_, Roster> {
        let namespace = namespace.to_string();
        Box::pin(async move {
            let journal = self.journal(&namespace)?;
            RingRail::roster(self, &journal).await
        })
    }

    fn journal_digest(&self, namespace: &str) -> JournalFut<'_, Digest> {
        let namespace = namespace.to_string();
        Box::pin(async move { self.journal(&namespace)?.digest(&crate::Ed25519Verifier) })
    }

    fn journal_ingest_all(&self, namespace: &str, ops: &[Op<SignedOp>]) -> JournalFut<'_, usize> {
        let namespace = namespace.to_string();
        let ops = ops.to_vec();
        Box::pin(async move { self.journal(&namespace)?.ingest_all(&ops) })
    }

    fn journal_ops_missing_from_within(
        &self,
        namespace: &str,
        theirs: &Digest,
        budget_bytes: usize,
    ) -> JournalFut<'_, (Vec<Op<SignedOp>>, bool)> {
        let namespace = namespace.to_string();
        let theirs = theirs.clone();
        Box::pin(async move {
            self.journal(&namespace)?.ops_missing_from_within(
                &crate::Ed25519Verifier,
                &theirs,
                budget_bytes,
            )
        })
    }

    fn journal_compact(&self, namespace: &str, roster: &Roster) -> JournalFut<'_, Compaction> {
        let namespace = namespace.to_string();
        let roster = roster.clone();
        Box::pin(async move {
            self.journal(&namespace)?
                .compact(&roster, &crate::Ed25519Verifier)
        })
    }
}

/// What a round reads from the node that runs it: its journals, its peer
/// HTTP client, this round's membership and the transport that turns a
/// member's contact into addresses. Upstream's `RailsDaemon` was one host,
/// and since pb-mesh-exit-transport the only production one; in svrngs the
/// node is (U1, DESIGN §8).
pub trait RingSyncHost: Send + Sync {
    /// This node's journals, or `None` on a node with no ring storage.
    fn journal(&self) -> Option<Arc<dyn RingSyncJournal>>;
    /// The one client every peer call rides — one pool, one timeout policy.
    fn http(&self) -> Result<&reqwest::Client, &str>;
    /// This round's membership, read once under the host's lock, or `None`
    /// when the host has none to round with. Upstream read it from the mesh,
    /// which svrngs does not fork; until the register answers (U1, DESIGN
    /// §8) nothing builds a [`RoundMembers`], so every host answers `None`
    /// and the round dials nobody.
    fn members(&self) -> Pin<Box<dyn Future<Output = Option<RoundMembers>> + Send + '_>>;
    /// The transport a member's contact is resolved through.
    fn transport(&self) -> Arc<IrohTransport>;
}

/// One round's view of the membership: the Online members it may dial, and
/// the two name lists its glassbox line prints. Upstream's `RoundMembers::of`
/// built it from a walk of the mesh, and minted the mesh proof each dial
/// carried; neither came over, so nothing builds one yet.
pub struct RoundMembers {
    pub(super) peers: Vec<OnlinePeer>,
    pub(super) exchanged_with: Vec<String>,
    pub(super) skipped_offline: Vec<String>,
}

/// An Online member of this mesh, as one round sees it.
///
/// Carries the key the roster filter decides on alongside the contact the
/// transport dials, because the two are read from the same `MemberRecord` in
/// the same pass and the filter runs per namespace, after the membership lock
/// is gone.
pub(super) struct OnlinePeer {
    pub(super) contact: Contact,
    /// `None` for a member on a pre-identity build. Such a member is absent
    /// from every DERIVED roster by construction (`ring_roster`'s
    /// `unidentified` count), and [`roster_names`] gives
    /// it the same answer against a hand-written one.
    pub(super) pubkey: Option<PublicKey>,
    pub(super) name: String,
}

/// Does `roster` name `pubkey` — the ONE membership test BOTH directions of
/// ring sync decide on.
///
/// The sender asks it of every Online peer before it offers a namespace
/// (`run_one_round`); the serving routes ask it of the verified asker
/// before they answer one (cw-rails' `ring_routes::ring_sync`). One function
/// so the
/// two cannot drift on how a key is rendered — a roster's actor is iroh
/// `PublicKey`'s lowercase hex `Display`, which is what [`crate::actor_of`]
/// writes and what `roster.json` holds.
///
/// **A member with no `node_pubkey` answers `false`**, which is exactly what a
/// DERIVED roster already does with it — `MeshRoster::derive` counts such a
/// member `unidentified` and writes no row, on purpose. A key the ring cannot
/// name cannot be on the ring: under-share, never over-share.
pub fn roster_names(roster: &Roster, pubkey: Option<PublicKey>) -> bool {
    pubkey.is_some_and(|k| roster.person_for(&k.to_string()).is_some())
}
