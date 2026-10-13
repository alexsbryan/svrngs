// SPDX-License-Identifier: AGPL-3.0-or-later
//! A link's edge, the command's since u4-2 moved the token into svrngs-core
//! (`svrngs_core::link`): the issuer's key and relay as iroh dials them,
//! which the core cannot name (it has no iroh). It came from `link.rs` as
//! it was; drawing a fresh token's secret, its other edge, is the node's
//! since start-home (`svrngs_node::invite::draw`), so the home page makes
//! a link as `svrngs link` does.

use svrngs_node::iroh::{EndpointAddr, PublicKey, RelayUrl};
use svrngs_node::link::Token;

/// The issuer's computer, as iroh names it.
pub fn key(token: &Token) -> PublicKey {
    PublicKey::from_bytes(token.issuer.as_bytes())
        .expect("the token's key was checked as iroh checks one (iroh-base key.rs:122-127)")
}

/// How the issuer's computer is dialled: its key, and its relay when
/// the token carries one.
pub fn addr(token: &Token) -> EndpointAddr {
    let addr = EndpointAddr::new(key(token));
    match &token.relay {
        Some(relay) => addr.with_relay_url(RelayUrl::from(relay.clone())),
        None => addr,
    }
}
