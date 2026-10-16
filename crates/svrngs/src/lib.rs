// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `svrngs` command (DESIGN §5; §8, U2's inventory): a computer's device
//! key ([`identity`]), a group's seed and id ([`group`], the node's), its link's
//! issuer, as iroh dials it ([`issuer`], the node's since arch-command (a); the
//! token is `svrngs_core::link`'s), the
//! relay anyone runs ([`relay`]), and the verbs ([`cli`]).

pub mod cli;
pub use svrngs_node::group;
pub mod identity;
pub use svrngs_node::join as issuer;
pub mod relay;
pub mod share;
