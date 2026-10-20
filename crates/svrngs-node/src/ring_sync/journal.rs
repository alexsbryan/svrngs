// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_sync/journal.rs@e1a43bcd2
//! The round's seam: one round's membership. A sibling of `ring_sync.rs` so
//! that file stays under the 800-line approach band (ARCH §3.1); re-exported
//! there. (svrngs: upstream's seam also held the roster a ring is offered
//! and served by, and the one roster test both directions decided on,
//! `roster_names`; both directions now ask the register,
//! [`crate::RingRail::standing`].)
//!
//! Upstream's seam also held two traits: `RingSyncHost`, what the round reads
//! from the node that runs it, and `RingSyncJournal`, the half of a rail one
//! round reads and writes. svrngs has one of each — the node's
//! [`crate::node::Host`] and [`crate::RingRail`] — so the round and the route call
//! them directly (RUST_ARCH 7).

use crate::iroh::{Contact, PublicKey};

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
    /// The members a round dials, each a `(name, contact)`: its key and
    /// the relay its group names (relay-in-record). With no presence
    /// (PLAN §4, U1's "No presence") none is skipped as offline: an offline
    /// member costs its dial.
    pub fn of(members: Vec<(String, Contact)>) -> Self {
        let exchanged_with = members.iter().map(|(name, _)| name.clone()).collect();
        let peers = members
            .into_iter()
            .map(|(name, contact)| OnlinePeer {
                pubkey: Some(contact.key),
                contact,
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
    /// `unidentified` count), and the round offers it nothing. (svrngs: every
    /// member is a key, so this is always `Some`.)
    pub(super) pubkey: Option<PublicKey>,
    pub(super) name: String,
}
