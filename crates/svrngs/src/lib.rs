// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `svrngs` command (DESIGN §5; §8, U2's inventory): a computer's device
//! key ([`identity`]), a group's seed and id ([`group`], the node's), the
//! relay anyone runs ([`relay`]), and the verbs ([`cli`]). A link's issuer,
//! as iroh dials it, is the node's `join` since arch-command (a).

pub mod cli;
pub use svrngs_node::group;
pub mod identity;
pub mod relay;
pub mod share;
