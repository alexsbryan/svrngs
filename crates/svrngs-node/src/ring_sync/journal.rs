// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_sync/journal.rs@e1a43bcd2
//! The round's seam: the roster a ring is offered and served by, one
//! round's membership, and the one roster test both directions of ring sync
//! decide on. A sibling of `ring_sync.rs` so that file stays under the
//! 800-line approach band (ARCH §3.1); re-exported there.
//!
//! Upstream's seam also held two traits: `RingSyncHost`, what the round reads
//! from the node that runs it, and `RingSyncJournal`, the half of a rail one
//! round reads and writes. svrngs has one of each — the node's
//! [`crate::node::Host`] and [`RingRail`] — so the round and the route call
//! them directly (RUST_ARCH 7), and [`roster_of`] is what remains of the
//! second.

use crate::iroh::{Contact, PublicKey};
use crate::{RailError, RingRail, Roster};

/// The namespace's roster, through the rail's one reader
/// ([`RingRail::roster`]).
pub async fn roster_of(rail: &RingRail, namespace: &str) -> Result<Roster, RailError> {
    let journal = rail.journal(namespace)?;
    rail.roster(&journal).await
}

/// One round's view of the membership: the Online members it may dial, and
/// the two name lists its glassbox line prints. Upstream's `RoundMembers::of`
/// built it from a walk of the mesh, and minted the mesh proof each dial
/// carried; neither came over. svrngs builds it from named keys.
pub struct RoundMembers {
    pub(super) peers: Vec<OnlinePeer>,
    pub(super) exchanged_with: Vec<String>,
    pub(super) skipped_offline: Vec<String>,
}

impl RoundMembers {
    /// The members a round dials, each a `(name, key)`, dialled by its key
    /// alone. With no presence (DESIGN §8, "Four facts") none is skipped as
    /// offline: an offline member costs its dial.
    pub fn of(members: Vec<(String, PublicKey)>) -> Self {
        let exchanged_with = members.iter().map(|(name, _)| name.clone()).collect();
        let peers = members
            .into_iter()
            .map(|(name, key)| OnlinePeer {
                contact: Contact {
                    key,
                    relay_url: None,
                    iroh_direct_addrs: Vec::new(),
                },
                pubkey: Some(key),
                name,
            })
            .collect();
        Self {
            peers,
            exchanged_with,
            skipped_offline: Vec::new(),
        }
    }
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
