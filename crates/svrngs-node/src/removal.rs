// SPDX-License-Identifier: AGPL-3.0-or-later
//! Removal, as this computer's person asks it (U7; DESIGN §6 "Removal has
//! one decider"; §10, 2026-10-09: removing someone stays in the command).
//! `svrngs remove` and `svrngs handover` ask the node running here, the one
//! process that writes this computer's acts, as `share --folder` does
//! ([`crate::folder`]): `POST /v1/removals` commits svrngs-core's
//! [`Removal`], `POST /v1/handovers` its [`HandOnTo`], and each answers
//! what happened as an [`Outcome`] the command words for a person. The
//! fold decides whether a removal is the cut or an ask; this module only
//! carries it.
//!
//! Each removal or hand-on written wakes the round, as a folder's write
//! does, so every computer of the group holds it now rather than at the
//! next round, up to a minute on (demo/runs/u7/2026-10-11, take 1).
//!
//! A cut is told to the removed person's computers once, as it lands: one
//! sync request carrying the cut and nothing else of the group, which their
//! node takes since this computer stands in their copy of the register.
//! From then on every computer of the group closes theirs at the connection
//! (`node.rs`, `admit_dialer`), so a removed computer that is off as it
//! lands is not told by svrngs; every lender's door refuses it all the same.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::iroh::{actor_key, Contact};
use crate::node::Asking;
use crate::peer_wire::RingSyncRequest;
use crate::{
    register::Register, Device, HandOnRefusal, HandOnTo, Person, Refused, Removal, RemovalRefusal,
};

/// Where the command asks a removal.
pub const REMOVALS_ROUTE: &str = "/v1/removals";

/// Where the command asks a hand-on.
pub const HANDOVERS_ROUTE: &str = "/v1/handovers";

/// What the command asks: the group by its id, and a person by name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovalAsk {
    pub group: String,
    pub person: String,
}

/// What happened, for the command to say.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Outcome {
    /// The cut: the person is out.
    Out,
    /// An ask: `decides`, the person who decides removals, may make it.
    Asked { decides: Option<Person> },
    /// The person now decides removals.
    Handed,
    /// Nobody of that name is in the group.
    NotInGroup,
    /// The person decides removals, so is not removed: they hand that on
    /// first.
    Decides,
    /// Only `decides` hands deciding on.
    NotDecider { decides: Option<Person> },
    /// This computer is not in the group.
    NotIn,
    /// The group's record could not be read or written here: said at warn.
    Fault,
}

impl Outcome {
    fn status(&self) -> StatusCode {
        match self {
            Outcome::Out | Outcome::Asked { .. } | Outcome::Handed => StatusCode::OK,
            Outcome::Fault => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::CONFLICT,
        }
    }
}

impl IntoResponse for Outcome {
    fn into_response(self) -> Response {
        (self.status(), Json(self)).into_response()
    }
}

/// The routes, for [`crate::api::router`] to serve behind its check.
pub(crate) fn router(asking: Asking) -> Router {
    Router::new()
        .route(REMOVALS_ROUTE, post(remove))
        .route(HANDOVERS_ROUTE, post(hand_over))
        .with_state(asking)
}

/// The person who decides removals in `register`.
fn decides(register: &Register) -> Option<Person> {
    let keeper = register.keeper.as_ref()?;
    register.members.get(keeper).map(|m| m.person.clone())
}

async fn remove(State(at): State<Asking>, Json(asked): Json<RemovalAsk>) -> Outcome {
    let Ok(person) = Person::parse(&asked.person) else {
        return Outcome::NotInGroup;
    };
    let rail = at.host.rail();
    let group = asked.group.as_str();
    let Ok(before) = rail.register(group) else {
        tracing::warn!(target: "removal", group, "removal: the group's register cannot be read");
        return Outcome::Fault;
    };
    let recorded = match rail.commit(group, &Removal { person }) {
        Ok(Ok(recorded)) => recorded,
        Ok(Err(refused)) => {
            tracing::debug!(target: "removal", group, ?refused, "removal: refused");
            return match refused {
                Refused::NoStanding => Outcome::NotIn,
                Refused::Command(RemovalRefusal::NotInGroup) => Outcome::NotInGroup,
                Refused::Command(RemovalRefusal::Keeps) => Outcome::Decides,
            };
        }
        Err(e) => {
            tracing::warn!(target: "removal", group, error = %e, "removal: the group's record cannot be written");
            return Outcome::Fault;
        }
    };
    let Ok(after) = rail.register(group) else {
        tracing::warn!(target: "removal", group, "removal: written, and the register cannot be read back");
        return Outcome::Fault;
    };
    let cut: Vec<String> = after
        .removed
        .keys()
        .filter(|k| !before.removed.contains_key(*k))
        .cloned()
        .collect();
    tracing::debug!(target: "removal", group, written = recorded.lines().len(), ?cut,
        "removal: written — the cut when this computer decides, else an ask; the round woken");
    // Now, not at the next round: every other lender refuses the removed
    // person only once its computer holds the cut, and the keeper sees an
    // ask only once it holds it (demo/runs/u7/2026-10-11, take 1: 43.3 s).
    at.door().ring.wake();
    if cut.is_empty() {
        return Outcome::Asked {
            decides: decides(&after),
        };
    }
    let computers = cut
        .into_iter()
        .filter(|k| {
            before
                .members
                .get(k)
                .is_some_and(|m| m.device == Device::Computer)
        })
        .collect();
    tokio::spawn(tell(
        at,
        group.to_string(),
        computers,
        recorded.lines().to_vec(),
    ));
    Outcome::Out
}

/// Tell each of `computers` the cut, `lines` and nothing else of the
/// group, one sync request each; traced, never retried.
async fn tell(
    at: Asking,
    group: String,
    computers: Vec<String>,
    lines: Vec<crate::Op<crate::SignedOp>>,
) {
    let (Ok(http), Ok(digest)) = (at.host.http(), at.host.rail().digest(&group)) else {
        tracing::warn!(target: "removal", group, "removal: no client or digest, so the removed computers are not told");
        return;
    };
    for actor in computers {
        let Some(key) = actor_key(&actor) else {
            continue;
        };
        let contact = Contact {
            key,
            relay_url: None,
            iroh_direct_addrs: Vec::new(),
        };
        let Some(base) = at.host.transport().base_url(&contact).await else {
            tracing::debug!(target: "removal", group, actor, "removal: no path to the removed computer, so it is not told");
            continue;
        };
        let request = RingSyncRequest {
            namespace: group.clone(),
            digest: digest.clone(),
            ops: lines.clone(),
        };
        let told =
            crate::ring_sync::post(http, &format!("{base}/internal/ring/sync"), &request).await;
        tracing::debug!(target: "removal", group, actor, told = told.is_ok(),
            "removal: the removed computer told its cut");
    }
}

async fn hand_over(State(at): State<Asking>, Json(asked): Json<RemovalAsk>) -> Outcome {
    let Ok(person) = Person::parse(&asked.person) else {
        return Outcome::NotInGroup;
    };
    let rail = at.host.rail();
    let group = asked.group.as_str();
    let outcome = match rail.commit(group, &HandOnTo { person }) {
        Ok(Ok(_)) => {
            at.door().ring.wake();
            Outcome::Handed
        }
        Ok(Err(Refused::NoStanding)) => Outcome::NotIn,
        Ok(Err(Refused::Command(HandOnRefusal::NotInGroup))) => Outcome::NotInGroup,
        Ok(Err(Refused::Command(HandOnRefusal::NotKeeper))) => Outcome::NotDecider {
            decides: rail.register(group).ok().as_ref().and_then(decides),
        },
        Err(e) => {
            tracing::warn!(target: "removal", group, error = %e, "removal: the group's record cannot be written");
            Outcome::Fault
        }
    };
    tracing::debug!(target: "removal", group, ?outcome, "removal: a hand-on asked; the round woken when written");
    outcome
}
