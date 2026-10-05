// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail/src/lib.rs@e1a43bcd2
//! The ring journal on disk — `<root>/rings/<namespace>/ring_oplog.jsonl`.
//!
//! [`svrngs_core`] is the FOLD: vocabulary, signing, admission,
//! the sync digest, and not one line of I/O. This crate is the half that
//! touches a filesystem — one writer per namespace, the door that signs, and
//! the peer-ingest path. Everything the core exports is re-exported here, so
//! an application names one crate and a peer that only needs the fold (canon)
//! names the other.
//!
//! # The door is strict about what the rail knows, and silent about the rest
//!
//! [`RingJournal::append`] refuses what the rail can judge: a payload with no
//! canonical form (see [`Payload`]), and an attempt to author under a key
//! that holds no standing in the ring (svrngs: the register's — the seed and
//! its `Admit` and `Remove` acts, [`register`] — where upstream asked whether
//! the ring's roster carried the key). It says nothing about whether an amount
//! is positive or a borrower exists, because it cannot — those are the app's,
//! and the app owns one validator that its own door and its own reducer both
//! call, exactly as this module used to.
//!
//! The second of those is [`RailError::NotInRoster`] rather than a sentence,
//! so a renderer says it in its own words by matching the variant, never the
//! prose. (Upstream's reason was that the fix depended on whether the roster
//! was a file or DERIVED, `RingRail::roster_origin`; svrngs derives no
//! roster, and the fix is an `Admit`.)
//!
//! The journal is **truth**; the mesh store is only a transport buffer. In
//! production `MeshStore` is `in_memory()`, so anything treating it as
//! durable loses the log on restart.

pub use svrngs_runtime::*;

// The v1 checkpoint document's one composer (moved from sovereign-mesh,
// phase-b pb-rails-parity).
pub mod ring_checkpoint;

// The dialling half of sync, forked in U1 (docs/DESIGN.md §8, U1's
// inventory): the sync bodies (`sovereign-peer-wire`), the one ALPN, the
// relayed endpoint and the loopback bridge (`mesh-reach::guest`), and the
// client side of the dial-by-key transport (`commonwealth-transport::iroh`).
// The ALPNs are svrngs-core's since u4-2, so a browser's page spells each as
// the node does; re-exported, so `crate::alpn` reads as before.
pub use svrngs_core::alpn;
pub mod guest;
pub mod iroh;
pub mod peer_wire;

// Ring sync and its route, forked in U1 (docs/DESIGN.md §8, U1's inventory):
// the round and the exchange (`commonwealth-rails`' `ring_sync`), and the sync
// route (`ring_routes.rs`), which a caller serves as an `axum::Router`.
pub mod ring_routes;
pub mod ring_sync;

// The register, U1's (docs/DESIGN.md §6): who stands in a group, the one
// answer the round, the acceptor, the sync route and the door ask.

// Redeeming a link, U2's (docs/DESIGN.md §6): the route served on the
// second ALPN, and the joiner's call to it.
pub mod redeem;

// The record over loopback, U3's (docs/DESIGN.md §8, U3's inventory): the
// rail's append, log and live doors (`commonwealth-rails`' `rail.rs`), on a
// listener that refuses any address but loopback (its `api.rs`), and the
// live lane's fan-out (`sovereign-daemon`'s `routes_rail_live.rs`).
pub mod api;
pub mod live;
pub mod rail;

// The member door, U3's (docs/DESIGN.md §8, U3's inventory): the route that
// carries a member's request to a lend's service, the lend's declaration,
// and the lender's secret for its own service, kept on its own computer
// (`commonwealth-media`'s `declared.rs`).
pub mod declared;
pub mod door;
pub mod lend;

// The node, U1's (docs/DESIGN.md §8): the rail, the endpoint, the accept loop
// serving the sync route and the round, in place of upstream's `RailsDaemon`.
pub mod node;
