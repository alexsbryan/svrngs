// SPDX-License-Identifier: AGPL-3.0-or-later
//! A link's two edges, the command's since u4-2 moved the token into
//! svrngs-core (`svrngs_core::link`): drawing a fresh token's secret, which
//! the core cannot (it draws nothing), and the issuer's key and relay as
//! iroh dials them, which the core cannot name (it has no iroh). Both came
//! from `link.rs` as they were.

use ed25519_dalek::VerifyingKey;
use svrngs_node::iroh::{EndpointAddr, PublicKey, RelayUrl};
use svrngs_node::link::{Token, SECRET_LEN};

/// How many secrets [`mint`] draws before printing one whose token spells
/// a kept word: the issuer's key is fixed, so its part may spell one that
/// no secret changes.
const DRAWS: usize = 256;

/// A fresh token for `issuer`, reached through `relay` when given: its
/// secret drawn again, up to [`DRAWS`] times, while the token spells a
/// word DESIGN §1 keeps from people.
#[expect(clippy::disallowed_methods, reason = "draws a link's secret")]
pub fn mint(issuer: PublicKey, relay: Option<RelayUrl>) -> Token {
    let mut token = Token {
        issuer: VerifyingKey::from_bytes(issuer.as_bytes())
            .expect("iroh's key is a checked ed25519 key (iroh-base key.rs:122-127)"),
        secret: [0; SECRET_LEN],
        relay: relay.map(Into::into),
    };
    for draw in 1..=DRAWS {
        // As identity.rs: getrandom failing is a host with no entropy.
        getrandom::fill(&mut token.secret).expect("failed to draw a link's secret");
        if !token.spells_a_kept_word() {
            tracing::debug!(target: "link", draw, "link: a token spelling no kept word");
            return token;
        }
    }
    tracing::debug!(target: "link", draws = DRAWS, "link: every draw spelled a kept word — the issuer's key or relay spells it; printed anyway");
    token
}

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

#[cfg(test)]
#[path = "issuer_tests.rs"]
mod tests;
