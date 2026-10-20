// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/publish_cmd.rs@e1a43bcd2
//! `svrngs share` (PLAN §4, U3's inventory, its publish_cmd row): the
//! verb's shape as upstream had it, `resolve_target` from `svrn publish`
//! (publish_cmd.rs:409-434, its tests in `share/tests.rs`). The probe, the
//! well-known table and the lend's decision are the node's
//! ([`svrngs_node::share`], moved there in v1ui-lend (a) so the home page
//! asks them too), asked here by the names they had.

use std::num::NonZeroU16;

pub use svrngs_node::share::*;

/// `5000` → port 5000 on this computer. (svrngs: a port alone, where
/// upstream also took a loopback `host:port`; the door dials 127.0.0.1.)
///
/// Loopback-only is not a limitation to relax later: the point of publishing
/// is that the app stays bound to this machine and is reached by mesh key. A
/// `0.0.0.0` target would already be on the LAN to anyone, with no identity
/// attached, which is the thing the mesh replaces.
pub fn resolve_target(port: &str) -> Result<NonZeroU16, String> {
    match port.parse::<u16>().ok().and_then(NonZeroU16::new) {
        Some(p) => Ok(p),
        None => Err(format!(
            "`{port}` is not a port on this computer: a number from 1 to 65535"
        )),
    }
}

#[cfg(test)]
mod tests;
