// SPDX-License-Identifier: AGPL-3.0-or-later
//! A relay's URL as every browser reaches it (u4-9; DESIGN §11, risk 5).
//!
//! iroh spells each of n0's relays with the DNS root's trailing dot,
//! `https://use1-1.relay.n0.iroh.link./` (`iroh-1.0.2/src/defaults.rs:27-33`,
//! made into URLs at `:48`), so that a resolver never tries the host's
//! search domains first (iroh PR #3487). WebKit, every iOS browser's engine,
//! refuses TLS to any host so spelled, and reaches the same host without
//! the dot (`ralph/REVIEW_FINDINGS.md` #52; iroh issue #4519). So every
//! relay URL the command, the node and the page make or read passes
//! through [`dotless`] where it enters: n0's relays as iroh lists them, a
//! link's relay as its token is read ([`crate::link::Token::decode`]), a
//! relay a computer is told, and an address iroh's lookup answers. A
//! computer's own relay comes from the first of those, so the address it
//! advertises is dotless too. Pure: a URL in, a URL out.

use alloc::string::String;

use url::{Host, Url};

/// `url` with its host's trailing dot dropped, else `url` unchanged: a
/// host that is an address, or a domain written without one.
pub fn dotless(mut url: Url) -> Url {
    let Some(Host::Domain(host)) = url.host() else {
        tracing::debug!(target: "relay", %url, "relay: not a domain — kept as written");
        return url;
    };
    let Some(bare) = host.strip_suffix('.').filter(|h| !h.is_empty()) else {
        tracing::debug!(target: "relay", %url, "relay: no trailing dot — kept as written");
        return url;
    };
    let bare = String::from(bare);
    match url.set_host(Some(&bare)) {
        Ok(()) => tracing::debug!(target: "relay", %url, "relay: the host's trailing dot dropped"),
        // Unreachable for a domain that parsed with the dot; kept as
        // written, said, rather than guessed at.
        Err(e) => {
            tracing::debug!(target: "relay", %url, error = %e, "relay: the host without its dot did not parse — kept as written")
        }
    }
    url
}

#[cfg(test)]
#[path = "relay_tests.rs"]
mod tests;
