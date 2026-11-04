//! Give: a group keeps an app on one member's computer at a time
//! (svrn-docs rings/current/GIVE.md). This is the half that runs on a
//! computer: the card an app is written as ([`card`]), the runtime that
//! keeps one in rootless podman ([`runtime`], over [`podman`]'s command line
//! and, on a Mac, its [`machine`]), the author's [`check`] of a card, and the
//! keeper program ([`keep`]), which folds who keeps what ([`give_fold`]),
//! reads and writes the group's record ([`record`]), lends through svrngs's
//! command ([`lend`]), and places give's carried page in a group's folder
//! ([`page`]). Only [`record`], [`lend`], [`page`] and [`keep`] know svrngs,
//! and only by its public surface.

pub mod card;
pub mod check;
pub mod keep;
pub mod lend;
pub mod machine;
pub mod page;
pub mod podman;
pub mod record;
pub mod runtime;
pub mod shelf;
