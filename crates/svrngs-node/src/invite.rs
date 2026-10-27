// SPDX-License-Identifier: AGPL-3.0-or-later
//! A link to a group, made in one place for `svrngs link` and the home page
//! (start-home): its token drawn, the relay it carries, and its `Invite`
//! written in the group's record. Moved from the command, behaviour kept:
//! [`draw`] was svrngs's `issuer::mint`, [`carried`] its `cli::link_relay`,
//! [`Host::mint`] and [`Host::own_person`] its `cli::mint` and
//! `cli::own_person`, asked through [`crate::node::Node::host`], and
//! [`kept_page`] its group.rs `read_origin`; [`Host::link`] its `cli::link_to`
//! and `cli::mint` (arch-command-relay (b)). The command keeps what is its
//! own: the group a name finds, the wait for a relay, and its words.

use std::path::Path;

use svrngs_core::link::{Origin, Token, SECRET_LEN};

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

/// Where this computer keeps the links it made for a guest (guests-link;
/// GUESTS §5), under its journals' root, one file a link named by the
/// link's key: off the record, as nobody but this computer redeems one.
pub const VISIT_LINKS: &str = "visit-links";

/// A link made for a guest, as kept until it is redeemed or lapses: its
/// group, how long the visit lasts from when the guest opens it, in
/// seconds, and when it was made.
pub type VisitLink = (String, i64, i64);

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

/// A link made: its token, the page its https link is on when the group
/// keeps one, and, a device link, whose computer or phone it adds.
#[derive(Debug)]
pub struct Linked {
    pub token: Token,
    pub page: Option<Origin>,
    pub device: Option<Person>,
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
    let configured = || (relay.relay_urls.first()).and_then(|r| crate::relay::parse(r).ok());
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

    /// A link to `group`, for the home page (start-home) and `svrngs link`
    /// and `new` alike: the https one on `page`, kept from now on, when
    /// given (`link --origin`), else on the group's kept page, else on this
    /// computer's; with `device`, a device link, and whose; with `visit`, a
    /// guest's link for
    /// a visit that long, kept here ([`VisitLink`]) in place of an
    /// `Invite`, so it lets nobody join, and on the group's page alone. Its
    /// relay is the one reached now, with no wait: a running computer has
    /// reached it, the command waits first, or [`carried`] says why.
    pub fn link(
        &self,
        group: &str,
        device: bool,
        page: Option<&Origin>,
        visit: Option<i64>,
    ) -> Result<Linked, Unminted> {
        let _minting = (self.minting.lock()).unwrap_or_else(std::sync::PoisonError::into_inner);
        self.link_held(group, device, page, visit)
    }

    /// `seed`'s group kept ([`Host::keep_group`]) and its first link made,
    /// with no link minted between, `new`'s: a link the home page makes the
    /// moment the group shows waits for this one, so it is the newer and the
    /// open one. The keeping's refusal, else the link or why none was made.
    pub fn keep_linked(
        &self,
        seed: &crate::group::Seed,
        page: Option<&Origin>,
        relay: Option<&RelayUrl>,
    ) -> Result<Result<Linked, Unminted>, String> {
        let _minting = (self.minting.lock()).unwrap_or_else(std::sync::PoisonError::into_inner);
        self.keep_group(seed, page, relay)?;
        tracing::debug!(target: "link", group = %seed.id(), "link: the group kept; its first link minted before any other");
        Ok(self.link_held(&seed.id(), false, None, None))
    }

    /// The page `group`'s https links print on: its own, else the page this
    /// computer came from, which install.sh keeps and the home page's groups
    /// keep (Asking::found), since a joiner by the client's link keeps none
    /// of the group's and a phone opens only an https link
    /// (capability-scenes-found (b)); a carried app is told it too
    /// (give-page-host). The group's own is read first, so a computer's
    /// unreadable page never hides it.
    pub(crate) fn printed_page(&self, group: &str) -> Result<Option<Origin>, String> {
        let dir = self.rail.group_dir(group).map_err(|e| e.to_string())?;
        kept_page(&dir)?.map_or_else(|| kept_page(self.rail.root()), |p| Ok(Some(p)))
    }

    /// [`Host::link`], its sequencer held.
    #[expect(clippy::disallowed_methods, reason = "a link's life starts now")]
    fn link_held(
        &self,
        group: &str,
        device: bool,
        page: Option<&Origin>,
        visit: Option<i64>,
    ) -> Result<Linked, Unminted> {
        let unkept = |e: &dyn std::fmt::Display| Unminted::Unkept(e.to_string());
        // Held first, so no journal is opened for a group it does not hold.
        let held = self.rail.namespaces().map_err(|e| unkept(&e))?;
        if !held.iter().any(|g| g == group) {
            tracing::debug!(target: "link", group, "link: refused — this computer does not hold the group");
            return Err(Unminted::NotIn);
        }
        let dir = self.rail.group_dir(group).map_err(|e| unkept(&e))?;
        if let Some(given) = page {
            crate::group::write_origin(&dir, given).map_err(|e| unkept(&e))?;
            tracing::debug!(target: "link", group, origin = given.as_str(), "link: the group's origin changed");
        }
        let page = self.printed_page(group).map_err(|e| unkept(&e))?;
        let (key, device) = device.then(|| self.own_person(group)).transpose()?.unzip();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
        let now = now.map_err(|e| unkept(&e))?.as_secs() as i64;
        let home = self.endpoint.addr().relay_urls().next().cloned();
        let relay = carried(&self.relay, home);
        let token = match visit {
            None => self.mint(group, relay, key, now)?,
            Some(_) if page.is_none() => {
                tracing::debug!(target: "link", group, "link: refused — a guest's link opens in a browser, and this group keeps no page");
                return Err(Unminted::Unkept(
                    "this group keeps no page for a guest to open".into(),
                ));
            }
            Some(length) => {
                let (token, dir) = (
                    draw(self.endpoint.id(), relay),
                    self.rail.root().join(VISIT_LINKS),
                );
                let kept = serde_json::to_vec(&(group, length, now)).map_err(|e| unkept(&e))?;
                let at = dir.join(actor_of(&token.key()));
                (std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(at, kept)))
                    .map_err(|e| unkept(&e))?;
                tracing::debug!(target: "link", group, length, "link: a guest's link, kept here and in no record");
                token
            }
        };
        tracing::debug!(target: "link", group, ?device, page = ?page.as_ref().map(Origin::as_str), "link: made, on the group's kept page too, else this computer's, when it keeps one");
        Ok(Linked {
            token,
            page,
            device,
        })
    }

    /// The guest's link whose key is `link`, claimed: removed as it is
    /// read, so of two redeeming it at once one has it, and it lets one
    /// guest in, once (guests-link). None when this computer never made it,
    /// it was claimed, or it lapsed [`LINK_LIFE_SECS`] after it was made.
    pub(crate) fn claim_visit(&self, link: &str, now: i64) -> Option<VisitLink> {
        let at = self.rail.root().join(VISIT_LINKS).join(link);
        let kept = (std::fs::read(&at).ok()).filter(|_| std::fs::remove_file(&at).is_ok());
        let kept = kept.and_then(|k| serde_json::from_slice::<VisitLink>(&k).ok());
        let lapsed = kept.as_ref().map(|k| now > k.2 + LINK_LIFE_SECS);
        tracing::debug!(target: "link", link, held = kept.is_some(), ?lapsed, "link: a guest's link claimed, if this computer made it and nobody claimed it first, and in time");
        kept.filter(|_| lapsed == Some(false))
    }

    /// This computer's person in `group`, by their first key and name, as
    /// the register names them (bash-devices): whom a device link adds a
    /// device of. A computer that stands there as nobody makes none.
    pub fn own_person(&self, group: &str) -> Result<(String, Person), Unminted> {
        let me = actor_of(self.rail.signer());
        let standing = (self.rail.standing(group)).map_err(|e| Unminted::Unkept(e.to_string()))?;
        let own = (standing.member(&me)).map(|m| (m.person_key.clone(), m.person.clone()));
        tracing::debug!(target: "link", group, ?own, "link: a device link — this computer's person");
        own.ok_or(Unminted::NotIn)
    }
}

#[cfg(test)]
#[path = "invite_tests.rs"]
mod tests;
