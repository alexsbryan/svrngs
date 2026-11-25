//! svrngs-keeper: a card kept running on this computer (svrn-docs
//! rings/current/HOSTING_AND_GIVE_BOUNDARY.md). The card an app is written as
//! ([`card`]), the runtime that keeps one in rootless podman ([`runtime`],
//! over [`podman`]'s command line and, on a Mac, its [`machine`]), and the
//! author's [`check`] of a card; the user unit [`systemd`] keeps one in; and
//! the command a program already running here was started with ([`promote`]).
//! It knows no group and names no svrngs type: Give's keeper asks it to run
//! the group's app.
#![deny(unsafe_code)]

pub mod card;
pub mod check;
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub mod host;
pub mod kept;
#[cfg(target_os = "macos")]
pub mod launchd;
pub mod machine;
pub mod podman;
pub mod promote;
pub mod runtime;
pub mod unit;
// Forked from commonwealth-ai's host-kit, whose rustfmt policy is stock and
// whose edition is 2021 (DESIGN §4: a fork keeps upstream's format and lint
// allowances). The one allowance: edition 2024's let-chains make clippy ask
// to collapse an `if let` that 2021 could not.
#[rustfmt::skip]
#[allow(clippy::collapsible_if)]
pub mod systemd;
