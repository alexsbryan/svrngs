//! Give: a group keeps an app on one member's computer at a time
//! (svrn-docs rings/current/GIVE.md). This is the half that runs on the
//! computer that keeps it: the card an app is written as ([`card`]), the
//! runtime that keeps one in rootless podman ([`runtime`], over [`podman`]'s
//! command line and, on a Mac, its [`machine`]), and the author's
//! [`check`] of a card. None of it names svrngs.

pub mod card;
pub mod check;
pub mod machine;
pub mod podman;
pub mod runtime;
