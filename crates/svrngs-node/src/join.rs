// SPDX-License-Identifier: AGPL-3.0-or-later
//! Joining a group by its link, in one place for `svrngs join` and the
//! home page (arch-command (a)). Moved from the command, behaviour kept:
//! [`Asking::join`] was svrngs's cli.rs `join_group`, and [`key`] and
//! [`addr`] its issuer.rs, which iroh dials and the core cannot name. The
//! command keeps what is its own: the wait for a relay, and its words.

use crate::group::{self, kept_relay, Name, Seed};
use crate::iroh::{EndpointAddr, PublicKey, RelayUrl};
use crate::link::{Link, Token};
use crate::node::{Asking, Node};
use crate::redeem::{ask, redeem, AskRequest, RedeemError, RedeemRequest, Redeemed};
use crate::{Device, Person};

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

/// Why a join did not let this computer in.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotJoined {
    #[error("the link lets nobody new in")]
    LinkRefused,
    #[error("someone in {group} already goes by {holder}")]
    NameTaken { holder: Name, group: Name },
    #[error("the computer that sent the link cannot be reached")]
    IssuerAway,
    #[error("{0} cannot be kept on this computer")]
    Keep(Name),
}

impl Node {
    /// [`Asking::join`], asked of this node.
    pub async fn join(&self, link: &Link, person: &Name) -> Result<Entered, NotJoined> {
        self.asking.join(link, person).await
    }
}

impl Asking {
    /// Ask `link`'s issuer what it is for, proving the link's key; believe
    /// the seed answered only when it hashes to the group named
    /// (`group::answered`); redeem the link there as `person`, signing with
    /// this computer's key; keep the group's seed once let in, and the
    /// origin of an https link; sync once from the issuer, whom this
    /// computer's copy of the register may not know yet (DESIGN §8, U2).
    /// Nothing is written until the issuer has let this computer in, so a
    /// refused link leaves nothing behind.
    pub(crate) async fn join(&self, link: &Link, person: &Name) -> Result<Entered, NotJoined> {
        let (host, key) = (&self.host, &*self.key);
        let issuer = addr(&link.token);
        let link_key = link.token.key();
        let asked = ask(
            &host.endpoint,
            issuer.clone(),
            &AskRequest::sign(&link_key, key),
        )
        .await
        .map_err(|e| not_let_in(e, None))?;
        let Some(seed) = group::answered(&asked.group, &asked.seed) else {
            tracing::debug!(target: "join", group = %asked.group, "join: the issuer's answer is not believed, so nothing is redeemed");
            return Err(NotJoined::LinkRefused);
        };
        let id = seed.id();
        if group::groups(host.rail.root()).iter().any(|s| s.id() == id) {
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
        let redeemed = redeem(&host.endpoint, issuer.clone(), &request)
            .await
            .map_err(|e| not_let_in(e, Some(seed.name())))?;
        tracing::debug!(target: "join", group = %id, redeemed = ?redeemed, "join: let in");
        // An address lookup is n0's, built only with `n0_services` (svrngs-node
        // guest.rs `relayed_endpoint_builder`): this computer meets its groups at n0's.
        let n0 = crate::iroh::has_lookup(&host.endpoint);
        let relay = kept_relay(link.token.relay.clone().map(RelayUrl::from), n0);
        // Kept as the node keeps a group, as `new` keeps it (Host::keep_group).
        let kept = host.keep_group(&seed, link.origin.as_ref(), relay.as_ref());
        kept.map_err(|why| {
            tracing::debug!(target: "join", group = %id, why, "join: refused — the group cannot be kept here");
            NotJoined::Keep(seed.name().clone())
        })?;
        if link.origin.is_none() {
            tracing::debug!(target: "join", group = %id, "join: a client's link, so no origin is kept");
        }
        let synced = host.sync_with(&id, &issuer).await;
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
        RedeemError::Unjudged | RedeemError::NoAnswer(_) => NotJoined::IssuerAway,
    }
}
