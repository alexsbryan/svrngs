//! svrngs-keeper: a card kept running on this computer (svrn-docs
//! rings/current/HOSTING_AND_GIVE_BOUNDARY.md). The card an app is written as
//! ([`card`]), the runtime that keeps one in rootless podman ([`runtime`],
//! over [`podman`]'s command line and, on a Mac, its [`machine`]), and the
//! author's [`check`] of a card. It knows no group and names no svrngs type:
//! Give's keeper asks it to run the group's app, and svrngs calls its program.

pub mod card;
pub mod check;
pub mod machine;
pub mod podman;
pub mod runtime;
