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

// The dialling half of sync, forked in U1 (docs/PLAN.md §4, U1's
// inventory): the sync bodies (`sovereign-peer-wire`), the one ALPN, the
// relayed endpoint (`mesh-reach::guest`; its loopback bridge cut, arch-peer), and the
// client side of the dial-by-key transport (`commonwealth-transport::iroh`).
// The ALPNs are svrngs-core's since u4-2, so a browser's page spells each as
// the node does; re-exported, so `crate::alpn` reads as before.
pub use svrngs_core::alpn;
// How long a dial to a lend's computer waits (events-asleep); the page
// compiles it too.
pub mod dial;
pub mod guest;
pub mod iroh;
pub mod peer_wire;
// Every relay URL without the DNS root's dot, so WebKit reaches it (u4-9).
pub mod relay;

// Ring sync and its route, forked in U1 (docs/PLAN.md §4, U1's inventory):
// the round and the exchange (`commonwealth-rails`' `ring_sync`), and the sync
// route (`ring_routes.rs`), which a caller serves as an `axum::Router`.
pub mod ring_routes;
pub mod ring_sync;

// The register, U1's (docs/DESIGN.md §6): who stands in a group, the one
// answer the round, the acceptor, the sync route and the door ask.

// Redeeming a link, U2's (docs/DESIGN.md §6): the route served on the
// second ALPN, and the joiner's call to it.
pub mod redeem;

// The record over loopback, U3's (docs/PLAN.md §4, U3's inventory): the
// rail's append, log and live doors (`commonwealth-rails`' `rail.rs`), on a
// listener that refuses any address but loopback (its `api.rs`), and the
// live lane's fan-out (`sovereign-daemon`'s `routes_rail_live.rs`).
pub mod api;
pub mod live;
pub mod rail;
// What an app asks beside the record, u5-1's (docs/DESIGN.md §3): the
// groups this computer holds, and what their people lend, on the same
// loopback listener.
pub mod surface;
// Every lend at an origin of its own on this computer's loopback, u5-2's
// (docs/DESIGN.md §3, "Lent apps", call 1), on the same listener.
/// The home page at `svrngs.localhost` (sup-2).
pub mod home;
pub mod origin;
// A browser told from a program at both: an open's session, or the token.
mod session;
// Where this computer is, and on which network: the router's hardware
// address, this computer's address there, a caller's (the room; the door).
pub mod network;
// The room, u6-1's (docs/DESIGN.md §3, "The room"): the lends a person
// turns on, served by the running node on the network each was turned on at.
pub mod room;
// Play, give-play's (svrn-docs rings/current/GIVE.md §6): a port a member
// lends, opened on this computer's 127.0.0.1 while its person has Play on.
pub mod play;
// What a lend is decided from: the probe, the well-known table (v1ui-lend (a)).
pub mod share;

// The member door, U3's (docs/PLAN.md §4, U3's inventory): the route that
// carries a member's request to a lend's service, the lend's declaration,
// and the lender's secret for its own service, kept on its own computer
// (`commonwealth-media`'s `declared.rs`).
pub mod declared;
pub mod door;
pub mod lend;

// The node, U1's (docs/PLAN.md §4): the rail, the endpoint, the accept loop
// serving the sync route and the round, in place of upstream's `RailsDaemon`.
pub mod folder;
// A group's seed kept on this computer, and the group kept (start-group).
pub mod group;
pub mod invite;
// Joining a group by its link (arch-command (a)).
pub mod join;
pub mod node;
// Sign in with svrngs: the lender's ES256 key and the tokens it signs.
pub mod oidc;
// Removal as this computer's person asks it (U7, DESIGN §6).
pub mod removal;
// This build's version, which every answer a page reads carries, and the
// one check made of it (arch-version); the page compiles it too.
pub mod version;
