// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `svrngs` command (DESIGN §5; §8, U2's inventory): a computer's device
//! key ([`identity`]), a group's seed and id ([`group`]) and its link
//! ([`link`]), and the verbs ([`cli`]).

pub mod cli;
pub mod group;
pub mod identity;
pub mod link;
pub mod share;
