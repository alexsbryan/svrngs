// SPDX-License-Identifier: AGPL-3.0-or-later
//! Joining a group by its link, in one place for `svrngs join` and the
//! home page (arch-command (a)). Moved from the command, behaviour kept:
//! [`Host::join`] was svrngs's cli.rs `join_group`, and [`key`] and
//! [`addr`] its issuer.rs, which iroh dials and the core cannot name. The
//! command keeps what is its own: the wait for a relay, and its words.
//! A computer whose svrngs runs from login joins through it too: the
//! command asks the running one at [`JOINS_ROUTE`] (join-while-running),
//! as it asks `room` and `leave`.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::group::{self, Name, Seed};
use crate::iroh::{EndpointAddr, PublicKey};
use crate::link::{Link, Token};
use crate::node::{Asking, Host};
use crate::redeem::{ask, redeem, AskRequest, Asked, RedeemError, RedeemRequest, Redeemed};
use crate::{register::Standing, Device, Person};
use iroh::TransportAddr;

/// The issuer's computer, as iroh names it.
pub fn key(token: &Token) -> PublicKey {
    PublicKey::from_bytes(token.issuer.as_bytes())
        .expect("the token's key was checked as iroh checks one (iroh-base key.rs:122-127)")
}

/// How the issuer's computer is dialled: its key, and every relay the
/// token carries (relay fallbacks).
pub fn addr(token: &Token) -> EndpointAddr {
    let relays = token
        .relays
        .iter()
        .map(|r| TransportAddr::Relay(r.clone().into()));
    EndpointAddr::from_parts(key(token), relays)
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
        self.meet_at(&link.token.relays).await;
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
        // Kept as the node keeps a group, as `new` keeps it (Host::keep_group),
        // the link's relays where an old group meets until its keeper names them.
        let kept = self.keep_group(&seed, link.origin.as_ref(), &link.token.relays);
        kept.map_err(|why| {
            tracing::debug!(target: "join", group = %id, why, "join: refused — the group cannot be kept here");
            NotJoined::Keep(seed.name().clone())
        })?;
        if link.origin.is_none() {
            tracing::debug!(target: "join", group = %id, "join: a client's link, so no origin is kept");
        }
        let synced = self.sync_with(&id, &issuer).await;
        // Where the group meets, from the record the sync brought, decided
        // now: until then this computer knew only the link's relays, and
        // its next round is a minute away (the run's take 2, relay
        // fallbacks).
        self.refresh();
        self.decide(None).await;
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

/// Where the command asks a join of the svrngs running here, as it asks
/// `room`, `leave` and a folder (join-while-running): `POST /v1/joins`,
/// served behind the record's checks ([`crate::api::router`]).
pub const JOINS_ROUTE: &str = "/v1/joins";

/// What the command asks: the whole link, and the person it joins as.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinAsk {
    pub link: String,
    pub person: String,
}

/// What came of a join the svrngs running here was asked, for the command
/// to say in its own words (cli.rs `Said`, `Refusal`): let in, or the
/// group held already, with the seed as `seed.json` holds it, whom this
/// computer joined as, and where its folder was placed or why not — or
/// why nobody was let in ([`NotJoined`], each named as the command's own
/// refusal is).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Outcome {
    In {
        seed: String,
        person: String,
        dir: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        placed: Option<String>,
    },
    AlreadyIn {
        seed: String,
        dir: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        placed: Option<String>,
    },
    LinkRefused,
    NameTaken {
        holder: String,
        group: String,
    },
    IssuerAway,
    Keep {
        group: String,
    },
    Left {
        group: String,
    },
}

impl Outcome {
    /// [`NotJoined`], as the wire says it.
    fn of(why: NotJoined) -> Self {
        match why {
            NotJoined::LinkRefused => Outcome::LinkRefused,
            NotJoined::NameTaken { holder, group } => Outcome::NameTaken {
                holder: holder.to_string(),
                group: group.to_string(),
            },
            NotJoined::IssuerAway => Outcome::IssuerAway,
            NotJoined::Keep(group) => Outcome::Keep {
                group: group.to_string(),
            },
            NotJoined::Left(group) => Outcome::Left {
                group: group.to_string(),
            },
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Outcome::In { .. } | Outcome::AlreadyIn { .. } => StatusCode::OK,
            _ => StatusCode::CONFLICT,
        }
    }
}

impl IntoResponse for Outcome {
    fn into_response(self) -> Response {
        (self.status(), Json(self)).into_response()
    }
}

/// The route, for [`crate::api::router`] to serve behind its checks.
pub(crate) fn router(asking: Asking) -> Router {
    Router::new()
        .route(JOINS_ROUTE, post(joined_here))
        .route(crate::home::LINKS_ROUTE, post(crate::home::link))
        .with_state(asking)
}

/// An ask the record cannot take, in the record's one refusal shape.
fn not_taken(status: StatusCode, kind: &str, error: String) -> Response {
    tracing::debug!(target: "join", kind, "join: an ask the record cannot take");
    crate::api::refuse(status, kind, error)
}

/// `POST /v1/joins`: the group the link names joined by the svrngs running
/// here, as the home page joins it ([`Host::join`]) — nothing written
/// until the link's issuer has let this computer in — and its folder
/// placed as a joiner's is ([`crate::folder::Folders::place`]).
async fn joined_here(State(at): State<Asking>, Json(asked): Json<JoinAsk>) -> Response {
    let Some(link) = Link::parse(&asked.link) else {
        return not_taken(
            StatusCode::BAD_REQUEST,
            "no_link",
            "That is not a whole link: paste all of it, to its end.".into(),
        );
    };
    let person = match Name::parse(&asked.person) {
        Ok(name) => name,
        Err(why) => return not_taken(StatusCode::BAD_REQUEST, "no_name", why.to_string()),
    };
    let entered = match at.host.join(&link, &person).await {
        Ok(entered) => entered,
        Err(why) => {
            tracing::debug!(target: "join", why = %why, "join: not let in, for the command");
            return Outcome::of(why).into_response();
        }
    };
    let (seed, person) = match entered {
        Entered::In { seed, person } => (seed, Some(person.to_string())),
        Entered::AlreadyIn(seed) => (seed, None),
    };
    tracing::debug!(target: "join", group = %seed.id(), ?person, "join: answered the command's join, by the svrngs running here");
    let (dir, placed) = at.folders.place(&seed);
    let (dir, placed) = (dir.display().to_string(), placed.err());
    let seed = seed.to_text();
    match person {
        Some(person) => Outcome::In {
            seed,
            person,
            dir,
            placed,
        },
        None => Outcome::AlreadyIn { seed, dir, placed },
    }
    .into_response()
}
