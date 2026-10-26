// SPDX-License-Identifier: AGPL-3.0-or-later
//! Joining a group by its link, in one place for `svrngs join` and the
//! home page (arch-command (a)). Moved from the command, behaviour kept:
//! [`Host::join`] was svrngs's cli.rs `join_group`, and [`key`] and
//! [`addr`] its issuer.rs, which iroh dials and the core cannot name. The
//! command keeps what is its own: the wait for a relay, and its words.

use crate::group::{self, kept_relay, Name, Seed};
use crate::iroh::{EndpointAddr, PublicKey, RelayUrl};
use crate::link::{Link, Token};
use crate::node::Host;
use crate::redeem::{ask, redeem, AskRequest, Asked, RedeemError, RedeemRequest, Redeemed};
use crate::{register::Standing, Device, Person};

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

/// A join that let this computer in, or found it in already.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entered {
    /// Let in by the link's issuer, as `person`.
    In { seed: Seed, person: Name },
    /// This computer holds the group already; nothing was redeemed.
    AlreadyIn(Seed),
}

/// Why a join did not let this computer in, in the home page's words
/// (arch-command (c)); the command says each in its own (cli.rs `Refusal`).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotJoined {
    #[error("This link lets nobody new in: ask whoever sent it for a new one.")]
    LinkRefused,
    #[error("Someone in {group} already goes by {holder}, so you were not let in as them. If that’s you, press “Add your phone or another computer” on a computer already in {group}, and paste that link here. Else join under another name.")]
    NameTaken { holder: Name, group: Name },
    #[error("The computer that sent this link cannot be reached. It may be off: try again later.")]
    IssuerAway,
    #[error("This computer cannot keep {0} on its disk.")]
    Keep(Name),
    /// This computer left the group, or was removed, and its one key never
    /// stands again (groups-leave; DESIGN §6, "Leaving a group").
    #[error("You’re no longer in {0} on this computer, and you can’t join it again from here yet. Join from your phone or another computer instead.")]
    Left(Name),
}

impl Host {
    /// What `link` is for, asked of its issuer, proving the link's key: its
    /// answer, and the seed answered, believed only when it hashes to the
    /// group named (`group::answered`). The home page says it before it
    /// joins.
    pub async fn asked(&self, link: &Link) -> Result<(Asked, Seed), NotJoined> {
        let request = AskRequest::sign(&link.token.key(), self.rail.signer());
        let asked = ask(&self.endpoint, addr(&link.token), &request).await;
        let asked = asked.map_err(|e| not_let_in(e, None))?;
        let Some(seed) = group::answered(&asked.group, &asked.seed) else {
            tracing::debug!(target: "join", group = %asked.group, "join: the issuer's answer is not believed, so nothing is redeemed");
            return Err(NotJoined::LinkRefused);
        };
        match self.rail.standing(&seed.id()) {
            Ok(Standing::Left(_)) => {
                tracing::debug!(target: "join", group = %seed.id(), "join: refused — this computer's key was cut in the group, so nothing is redeemed");
                return Err(NotJoined::Left(seed.name().clone()));
            }
            Err(e) => {
                tracing::debug!(target: "join", group = %seed.id(), error = %e, "join: whether this computer was cut in the group cannot be read here, so the issuer judges it (#181)")
            }
            Ok(_) => {}
        }
        Ok((asked, seed))
    }

    /// What `link` is for ([`Self::asked`]); redeem it there as `person`,
    /// signing with this computer's key; keep the group's seed once let in, and the
    /// origin of an https link; sync once from the issuer, whom this
    /// computer's copy of the register may not know yet (PLAN §4, U2).
    /// Nothing is written until the issuer has let this computer in, so a
    /// refused link leaves nothing behind.
    pub async fn join(&self, link: &Link, person: &Name) -> Result<Entered, NotJoined> {
        let key = self.rail.signer();
        let (issuer, link_key) = (addr(&link.token), link.token.key());
        let (asked, seed) = self.asked(link).await?;
        let id = seed.id();
        if group::groups(self.rail.root()).iter().any(|s| s.id() == id) {
            tracing::debug!(target: "join", group = %id, "join: this computer holds the group already, so nothing is redeemed");
            return Ok(Entered::AlreadyIn(seed));
        }
        // A device link's person is who this computer joins as, asked no name
        // (bash-devices); else the name given.
        let person = asked.person.unwrap_or_else(|| Person::from(person.clone()));
        tracing::debug!(target: "join", group = %id, %person, "join: redeeming as this person");
        let request = RedeemRequest::sign(
            &link_key,
            key,
            &id,
            &asked.invite,
            &person,
            Device::Computer,
        );
        let redeemed = redeem(&self.endpoint, issuer.clone(), &request)
            .await
            .map_err(|e| not_let_in(e, Some(seed.name())))?;
        tracing::debug!(target: "join", group = %id, redeemed = ?redeemed, "join: let in");
        // An address lookup is n0's, built only with `n0_services` (svrngs-node
        // guest.rs `relayed_endpoint_builder`): this computer meets its groups at n0's.
        let n0 = crate::iroh::has_lookup(&self.endpoint);
        let relay = kept_relay(link.token.relay.clone().map(RelayUrl::from), n0);
        // Kept as the node keeps a group, as `new` keeps it (Host::keep_group).
        let kept = self.keep_group(&seed, link.origin.as_ref(), relay.as_ref());
        kept.map_err(|why| {
            tracing::debug!(target: "join", group = %id, why, "join: refused — the group cannot be kept here");
            NotJoined::Keep(seed.name().clone())
        })?;
        if link.origin.is_none() {
            tracing::debug!(target: "join", group = %id, "join: a client's link, so no origin is kept");
        }
        let synced = self.sync_with(&id, &issuer).await;
        match &synced.stop {
            None => {
                tracing::debug!(target: "join", group = %id, pulled = synced.pulled, "join: synced once from the issuer")
            }
            Some(stop) => {
                tracing::warn!(target: "join", group = %id, stop = ?stop, pulled = synced.pulled, "join: let in, and the first sync stopped; the round carries on")
            }
        }
        if matches!(redeemed, Redeemed::AlreadyIn) {
            tracing::debug!(target: "join", group = %id, "join: the issuer says this key stood already; this computer now keeps its copy");
        }
        let person = person.name().clone();
        Ok(Entered::In { seed, person })
    }
}

/// The issuer did not let this computer in: it refused, or never answered.
fn not_let_in(e: RedeemError, group: Option<&Name>) -> NotJoined {
    tracing::debug!(target: "join", error = %e, "join: not let in");
    match e {
        RedeemError::Refused(_) => NotJoined::LinkRefused,
        RedeemError::NameTaken(holder) => match group {
            Some(group) => NotJoined::NameTaken {
                holder,
                group: group.clone(),
            },
            // Only a redemption answers that a name is taken: to an ask
            // it is an answer this build cannot read.
            None => NotJoined::IssuerAway,
        },
        // Only a redemption answers that the key was cut (#181).
        RedeemError::Cut => group.map_or(NotJoined::IssuerAway, |g| NotJoined::Left(g.clone())),
        RedeemError::Unjudged | RedeemError::NoAnswer(_) => NotJoined::IssuerAway,
    }
}
