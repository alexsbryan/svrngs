// SPDX-License-Identifier: AGPL-3.0-or-later
//! A link to a group, made in one place for `svrngs link` and the home page
//! (start-home): its token drawn, the relay it carries, and its `Invite`
//! written in the group's record. Moved from the command, behaviour kept:
//! [`draw`] was svrngs's `issuer::mint`, [`carried`] its `cli::link_relay`,
//! [`Host::mint`] and [`Host::own_person`] its `cli::mint` and
//! `cli::own_person`, asked through [`crate::node::Node::host`], and
//! [`kept_page`] its group.rs `read_origin`. The command keeps what is its own: the clock, the
//! wait for a relay, and its words.

use std::path::Path;

use svrngs_core::link::{client_link, https_link, Origin, Token, SECRET_LEN};

use crate::iroh::{PublicKey, RelayConfig, RelayUrl};
use crate::node::Host;
use crate::{actor_of, MintInvite, Person, RailError, Refused};

/// How long a link lets people in: upstream's one invite lapse
/// (`commonwealth-discovery/src/membership.rs:53` at `e1a43bcd2`, a day;
/// the core decides any lapse it is handed, u2-4b).
pub const LINK_LIFE_SECS: i64 = 24 * 60 * 60;

/// How many secrets [`draw`] draws before answering one whose token spells
/// a kept word: the issuer's key is fixed, so its part may spell one that
/// no secret changes.
const DRAWS: usize = 256;

/// The one spelling of the file in a group's directory that keeps the page
/// its https links print on (u2-7; group.rs `write_origin` writes it).
pub const ORIGIN_FILE: &str = "origin";

/// The page `dir`, a group's directory, keeps for its https links, or
/// `None` when it keeps none; why it cannot be read, never an absence
/// (ARCH 6). Moved from svrngs's group.rs `read_origin`.
pub fn kept_page(dir: &Path) -> Result<Option<Origin>, String> {
    match std::fs::read_to_string(dir.join(ORIGIN_FILE)) {
        Ok(raw) => Origin::parse(&raw)
            .map(Some)
            .ok_or_else(|| "not an http(s) address without a '#'".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(target: "group", dir = %dir.display(), "group: no origin kept");
            Ok(None)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Why no link was made.
#[derive(Debug, thiserror::Error)]
pub enum Unminted {
    #[error("this computer does not stand in the group")]
    NotIn,
    #[error("{0}")]
    Unkept(String),
}

/// A fresh token for `issuer`, reached through `relay` when given: its
/// secret drawn again, up to [`DRAWS`] times, while the token spells a
/// word DESIGN §1 keeps from people.
#[expect(clippy::disallowed_methods, reason = "draws a link's secret")]
pub fn draw(issuer: PublicKey, relay: Option<RelayUrl>) -> Token {
    let mut token = Token {
        issuer: ed25519_dalek::VerifyingKey::from_bytes(issuer.as_bytes())
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

/// The relay a link carries, given how this computer reaches the network
/// and the home relay its endpoint reports: `home`, or the first relay
/// configured while the endpoint reports none, in either posture.
pub fn carried(relay: &RelayConfig, home: Option<RelayUrl>) -> Option<RelayUrl> {
    let configured = || {
        relay
            .relay_urls
            .first()
            .and_then(|r| crate::relay::parse(r).ok())
    };
    let carried = home.or_else(configured);
    match &carried {
        Some(url) => {
            tracing::debug!(target: "link", relay = %url, n0 = relay.n0_services, "link: the link carries this computer's relay")
        }
        None => {
            tracing::warn!(target: "link", n0 = relay.n0_services, "link: no relay — the link carries the key alone, and no phone's page can open it")
        }
    }
    carried
}

impl Host {
    /// A link to `group` carrying `relay`: an `Invite` for a fresh token's
    /// key that lapses [`LINK_LIFE_SECS`] after `now`, signed by this
    /// computer, adding a device of the person `device_of` when given; the
    /// token. The issuer's newer `Invite` closes their last one from its
    /// place in the order (u2-4b, `membership`).
    pub fn mint(
        &self,
        group: &str,
        relay: Option<RelayUrl>,
        device_of: Option<String>,
        now: i64,
    ) -> Result<Token, Unminted> {
        let token = draw(self.endpoint.id(), relay);
        // Where this computer is reached, in the record before a joiner
        // pulls it, so the joiner can dial it back (relay-in-record).
        self.say_relay();
        let mint = MintInvite {
            key: actor_of(&token.key()),
            until: now + LINK_LIFE_SECS,
            device_of,
        };
        let recorded = match self.rail.commit(group, &mint) {
            Ok(Ok(recorded)) => recorded,
            Ok(Err(Refused::NoStanding)) => {
                tracing::debug!(target: "link", group, "link: refused — this computer does not stand in the group");
                return Err(Unminted::NotIn);
            }
            Ok(Err(Refused::Unsettled)) => {
                return Err(Unminted::Unkept(
                    RailError::Unsettled(group.to_string()).to_string(),
                ))
            }
            Ok(Err(Refused::Command(never))) => match never {},
            Err(e) => return Err(Unminted::Unkept(e.to_string())),
        };
        tracing::debug!(target: "link", group, invite = %recorded.lines()[0].id, until = mint.until, "link: minted");
        Ok(token)
    }

    /// A link to `group` made where its svrngs runs, for the home page
    /// (start-home), as `svrngs link` makes one (`cli::link_to`): the
    /// client's link, and the https one on the group's kept page; with
    /// `device`, a device link. Its relay is the one reached now, with no
    /// wait: a running computer has reached it, or [`carried`] says why.
    #[expect(
        clippy::disallowed_methods,
        reason = "the page's clock: a link's life starts now"
    )]
    pub fn link(&self, group: &str, device: bool) -> Result<(String, Option<String>), Unminted> {
        let unkept = |e: &dyn std::fmt::Display| Unminted::Unkept(e.to_string());
        // Held first, so no journal is opened for a group it does not hold.
        let held = self.rail.namespaces().map_err(|e| unkept(&e))?;
        if !held.iter().any(|g| g == group) {
            return Err(Unminted::NotIn);
        }
        let dir = self.rail.group_dir(group).map_err(|e| unkept(&e))?;
        let page = kept_page(&dir).map_err(|e| unkept(&e))?;
        let device_of = device.then(|| self.own_person(group)).transpose()?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
        let now = now.map_err(|e| unkept(&e))?.as_secs() as i64;
        let home = self.endpoint.addr().relay_urls().next().cloned();
        let relay = carried(&self.relay, home);
        let token = self.mint(group, relay, device_of.map(|p| p.0), now)?;
        tracing::debug!(target: "link", group, device, page = ?page.as_ref().map(Origin::as_str), "link: made for the home page");
        Ok((client_link(&token), page.map(|p| https_link(&p, &token))))
    }

    /// This computer's person in `group`, by their first key and name, as
    /// the register names them (bash-devices): whom a device link adds a
    /// device of. A computer that stands there as nobody makes none.
    pub fn own_person(&self, group: &str) -> Result<(String, Person), Unminted> {
        let me = self.rail.signer().actor();
        let standing = (self.rail.standing(group)).map_err(|e| Unminted::Unkept(e.to_string()))?;
        let own = standing
            .member(&me)
            .map(|m| (m.person_key.clone(), m.person.clone()));
        tracing::debug!(target: "link", group, ?own, "link: a device link — this computer's person");
        own.ok_or(Unminted::NotIn)
    }
}

#[cfg(test)]
#[path = "invite_tests.rs"]
mod tests;
