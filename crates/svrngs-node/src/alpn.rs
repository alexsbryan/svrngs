// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-reach/src/alpn.rs@e1a43bcd2
//! The protocol a node's endpoint speaks. Upstream named five here, one per
//! class of traffic (`RPC_ALPN`, `CLIENT_ALPN`, `GUEST_ALPN`, `OFFER_ALPN`
//! beside this one); svrngs carries one, and an enum when a second protocol
//! comes (docs/DESIGN.md §8, U1's inventory).

/// ALPN for mesh-internal HTTP-over-iroh tunnels. Version-suffixed so
/// a future class-aware protocol can coexist during migration.
pub const ALPN: &[u8] = b"cwth/http/0";
