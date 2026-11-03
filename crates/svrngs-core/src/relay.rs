// SPDX-License-Identifier: AGPL-3.0-or-later
//! A relay's URL as every browser reaches it (u4-9; DESIGN §8, risk 5).
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

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use url::Host;
pub use url::{ParseError, Url};

/// A relay as written — a link's, one the record names
/// ([`crate::RailAct::Relay`]), one a person or a peer tells a computer —
/// read dotless; its [`ParseError`], said, when it is no URL. The one
/// reading of a relay a person or a peer wrote: svrngs-node's
/// `relay::parse` wraps it as iroh's `RelayUrl` (#131).
pub fn parse(raw: &str) -> Result<Url, ParseError> {
    let parsed = raw.parse::<Url>().map(dotless);
    if let Err(e) = &parsed {
        tracing::debug!(target: "relay", raw, error = %e, "relay: refused — not a URL");
    }
    parsed
}

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

/// Where a group meets (relay fallbacks; svrn-docs
/// `rings/current/GROUP_RELAYS.md` §7, the invariant the operator agreed
/// 2026-11-02, as tiers): two tiers, the relays its keeper names in its
/// record ([`crate::Membership::meets_at`]), and the build's, which the
/// record never names (svrngs's `cli::SHIPPED`). Each tier a set, so no
/// order within it; the one order is between them, decided by
/// [`Meeting::decide`]. Made only by [`Meeting::of`], which always carries
/// the build's tier, so a group with no fallback is not written: its parts
/// are private.
///
/// ```compile_fail
/// // A meeting by hand, the build's tier left out, does not compile.
/// let _ = svrngs_core::relay::Meeting { own: Default::default(), build: Default::default(), unowned: false };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meeting {
    own: BTreeSet<Url>,
    build: BTreeSet<Url>,
    /// Whether a group it was made of names no relay of its own, so the
    /// build's tier is held whatever answers ([`Tier::Unowned`]).
    unowned: bool,
}

/// Which tier a computer meets its groups at (relay fallbacks): a closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// Its groups' own relays alone, iroh choosing among them by speed.
    Own,
    /// The build's too: none of its groups' own relays answers.
    Unanswered,
    /// The build's too: a group it holds names none of its own, and one
    /// endpoint, so one home, serves every group (§7's first slice).
    Unowned,
}

impl Meeting {
    /// The one function: the group's `own` relays as one tier, the
    /// `build`'s as the other, each read dotless.
    pub fn of(own: impl IntoIterator<Item = Url>, build: &[Url]) -> Self {
        let own: BTreeSet<Url> = own.into_iter().map(dotless).collect();
        let build = build.iter().cloned().map(dotless).collect();
        let meeting = Self {
            unowned: own.is_empty(),
            own,
            build,
        };
        tracing::debug!(target: "relay", own = ?meeting.own, build = ?meeting.build, "relay: where a group meets, its own relays as one tier and the build's as the other");
        meeting
    }

    /// Every relay of both tiers, once: where a member is dialled.
    pub fn relays(&self) -> impl Iterator<Item = &Url> {
        self.own.union(&self.build)
    }

    /// The group's own tier: the relays svrngs asks whether they answer.
    pub fn own(&self) -> &BTreeSet<Url> {
        &self.own
    }

    /// Where a computer of both groups meets: each tier joined.
    pub fn join(mut self, other: Meeting) -> Self {
        self.own.extend(other.own);
        self.build.extend(other.build);
        self.unowned |= other.unowned;
        self
    }

    /// The one decision (§7): the tier a computer meets its groups at, and
    /// the relays its endpoint holds for it, given whether any of the own
    /// tier `answers`. The build's tier only while none answers, or while
    /// a group names none of its own.
    pub fn decide(&self, answers: bool) -> (Tier, BTreeSet<Url>) {
        let tier = match (self.unowned, answers) {
            (true, _) => Tier::Unowned,
            (false, false) => Tier::Unanswered,
            (false, true) => Tier::Own,
        };
        let build = self.build.iter().filter(|_| tier != Tier::Own);
        let held: BTreeSet<Url> = self.own.iter().chain(build).cloned().collect();
        tracing::debug!(target: "relay", ?tier, answers, held = held.len(), "relay: the tier a computer meets its groups at — its own relays while any answers, the build's beside them only when none does or a group names none");
        (tier, held)
    }

    /// The at most two relays a link carries: `home`, the issuer's, first
    /// when it is the meeting's, then the group's own, else the build's
    /// first, so a group with none of its own carries one relay, as before.
    pub fn carried(&self, home: Option<&Url>) -> Vec<Url> {
        let home = home.filter(|h| self.relays().any(|r| r == *h));
        let own = self.own.iter().filter(|r| Some(*r) != home);
        let mut carried: Vec<&Url> = home.into_iter().chain(own).take(2).collect();
        if carried.is_empty() {
            carried.extend(self.build.first());
        }
        tracing::debug!(target: "relay", ?carried, "relay: a link carries the issuer's home first, then the group's own");
        carried.into_iter().cloned().collect()
    }
}

#[cfg(test)]
#[path = "relay_tests.rs"]
mod tests;
