// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-reach/src/alpn.rs@e1a43bcd2
//! The protocols a node's endpoint speaks. Upstream named five here, one per
//! class of traffic (`RPC_ALPN`, `CLIENT_ALPN`, `GUEST_ALPN`, `OFFER_ALPN`
//! beside this one); svrngs carries two, the enum this file promised when a
//! second protocol came (docs/DESIGN.md §8, U2's inventory, the acceptor
//! row): the members' protocol, and the one a key standing in no group may
//! speak, to redeem a link. It lives in svrngs-core since u4-2, so a page
//! dialling from a browser spells each name as the node does (DESIGN §8,
//! U4's inventory, the `kernel-types/src/alpn.rs` row).

/// A protocol the node speaks, by the ALPN its connection negotiated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alpn {
    /// Sync and every route a member reaches: served only to a dialer
    /// standing in a group the node holds.
    Member,
    /// Redeeming a link: the one route a key standing nowhere may reach.
    Redeem,
}

impl Alpn {
    /// Every protocol, as the endpoint advertises them.
    pub const ALL: [Alpn; 2] = [Alpn::Member, Alpn::Redeem];

    /// The bytes negotiated on the wire.
    pub const fn bytes(self) -> &'static [u8] {
        match self {
            // Upstream's, kept: version-suffixed so a future class-aware
            // protocol can coexist during migration.
            Alpn::Member => b"cwth/http/0",
            Alpn::Redeem => b"svrngs/redeem/0",
        }
    }

    /// The protocol `bytes` names, or `None` for one this node does not
    /// speak.
    pub fn of(bytes: &[u8]) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.bytes() == bytes)
    }
}

/// ALPN for mesh-internal HTTP-over-iroh tunnels: the members' protocol,
/// as the forked dialling code spells it.
pub const ALPN: &[u8] = Alpn::Member.bytes();
