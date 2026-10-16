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
    register::Register, Command, Device, HandOnRefusal, HandOnTo, Joined, Leave, Person, Refused,
    Removal, RemovalRefusal,
};

/// Where the command asks a removal.
pub const REMOVALS_ROUTE: &str = "/v1/removals";

/// Where the command asks a hand-on.
pub const HANDOVERS_ROUTE: &str = "/v1/handovers";

/// Where the command and the home page ask this computer's person's leave
/// (groups-leave), of the group a [`LeaveAsk`] names.
pub const LEAVES_ROUTE: &str = "/v1/leaves";

/// What a leave asks: the group, by its id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaveAsk {
    pub group: String,
}

/// What the command asks: the group by its id, and a person by name and,
/// of two people of one name, by when they joined (bash-same-name).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovalAsk {
    pub group: String,
    pub person: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joined: Option<Joined>,
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
    /// This computer's person left the group.
    Left,
    /// Nobody of that name is in the group.
    NotInGroup,
    /// The ask names no one person — more than one in the group goes by
    /// the name (bash-devices), or none of them joined when it says — so
    /// nothing is written: when each of them joined, which names one
    /// (bash-same-name).
    Which { joined: Vec<Joined> },
    /// The person decides removals, so is not removed: they hand that on
    /// first.
    Decides,
    /// The person is the one asking: a leave, not a removal.
    Own,
    /// Only `decides` hands deciding on.
    NotDecider { decides: Option<Person> },
    /// Only whoever decides removals hands deciding on, and who that is
    /// could not be read here: said at warn, never as a group nobody
    /// decides (ralph/REVIEW_FINDINGS.md #123).
    NotDeciderUnread,
    /// This computer is not in the group.
    NotIn,
    /// The group's record could not be read or written here, or who is in
    /// it cannot be judged, its removals unsettled (#124): said at warn.
    Fault,
}

impl Outcome {
    fn status(&self) -> StatusCode {
        match self {
            Outcome::Out | Outcome::Asked { .. } | Outcome::Handed | Outcome::Left => {
                StatusCode::OK
            }
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
        .route(LEAVES_ROUTE, post(leave))
        .with_state(asking)
}

/// The person who decides removals in `register`.
fn decides(register: &Register) -> Option<Person> {
    let keeper = register.keeper.as_ref()?;
    register.members.get(keeper).map(|m| m.person.clone())
}

async fn remove(State(at): State<Asking>, Json(asked): Json<RemovalAsk>) -> Outcome {
    let Ok(person) = Person::parse(&asked.person) else {
        tracing::debug!(target: "removal", ?asked, "removal: refused — the name asked is no name, so nobody of it is in the group");
        return Outcome::NotInGroup;
    };
    let joined = asked.joined;
    cut(at, &asked.group, &Removal { person, joined }).await
}

/// This computer's person leaves (groups-leave): svrngs-core's [`Leave`].
/// From then on this computer holds the group and nobody stands in it here
/// ([`crate::register::Standing::Left`]): no round, no sync, no lend
/// reaching it. Nothing on this computer's disk is deleted.
async fn leave(State(at): State<Asking>, Json(asked): Json<LeaveAsk>) -> Outcome {
    let me = Leave(at.host.actor.clone());
    cut(at, &asked.group, &me).await
}

/// `command`, a removal or a leave, written in `group`, and what happened.
/// A cut is told once, the cut alone, to the computers that would not hear
/// of it by the round: a removal's to the removed person's, a leave's to
/// everyone else's, since the leaver's round no longer reaches the group
/// (the operator, 2026-10-14: "Record only").
async fn cut<C: Command<Refusal = RemovalRefusal>>(
    at: Asking,
    group: &str,
    command: &C,
) -> Outcome {
    let rail = &at.host.rail;
    let Ok(before) = rail.register(group) else {
        tracing::warn!(target: "removal", group, "removal: the group's register cannot be read");
        return Outcome::Fault;
    };
    let recorded = match rail.commit(group, command) {
        Ok(Ok(recorded)) => recorded,
        Ok(Err(refused)) => {
            tracing::debug!(target: "removal", group, ?refused, "removal: refused");
            return match refused {
                Refused::NoStanding => Outcome::NotIn,
                Refused::Unsettled => Outcome::Fault,
                Refused::Command(RemovalRefusal::NotInGroup) => Outcome::NotInGroup,
                Refused::Command(RemovalRefusal::Which(joined)) => Outcome::Which { joined },
                Refused::Command(RemovalRefusal::Keeps) => Outcome::Decides,
                Refused::Command(RemovalRefusal::Own) => Outcome::Own,
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
    let left = cut.contains(&at.host.actor);
    tracing::debug!(target: "removal", group, written = recorded.lines().len(), ?cut, left,
        "removal: written — the cut when this computer decides or leaves, else an ask; the round woken");
    // Now, not at the next round: every other lender refuses the removed
    // person only once its computer holds the cut, and the keeper sees an
    // ask only once it holds it (demo/runs/u7/2026-10-11, take 1: 43.3 s).
    at.door.ring.wake();
    if cut.is_empty() {
        return Outcome::Asked {
            decides: decides(&after),
        };
    }
    // Each with the relay the record named before the cut (relay-in-record).
    let computers = (before.members.into_iter())
        .filter(|(k, m)| {
            m.device == Device::Computer && *k != at.host.actor && left != cut.contains(k)
        })
        .map(|(k, m)| (k, m.relay))
        .collect();
    tokio::spawn(tell(
        at,
        group.to_string(),
        computers,
        recorded.lines().to_vec(),
    ));
    if left {
        Outcome::Left
    } else {
        Outcome::Out
    }
}

/// Tell each of `computers` the cut, `lines` and nothing else of the
/// group, one sync request each; traced, never retried.
async fn tell(
    at: Asking,
    group: String,
    computers: Vec<(String, Option<svrngs_core::relay::Url>)>,
    lines: Vec<crate::Op<crate::SignedOp>>,
) {
    let Ok(digest) = at.host.rail.digest(&group) else {
        tracing::warn!(target: "removal", group, "removal: no digest, so the removed computers are not told");
        return;
    };
    for (actor, relay) in computers {
        let Some(key) = actor_key(&actor) else {
            tracing::debug!(target: "removal", group, actor, "removal: the removed key is no computer's address, so it is not told");
            continue;
        };
        let contact = Contact::named(key, relay.as_ref());
        let request = RingSyncRequest {
            namespace: group.clone(),
            digest: digest.clone(),
            ops: lines.clone(),
        };
        let told = crate::ring_sync::post((&at.host.transport, &contact), &request).await;
        tracing::debug!(target: "removal", group, actor, told = told.is_ok(), error = ?told.err(),
            "removal: the removed computer told its cut, or why not");
    }
}

async fn hand_over(State(at): State<Asking>, Json(asked): Json<RemovalAsk>) -> Outcome {
    let Ok(person) = Person::parse(&asked.person) else {
        tracing::debug!(target: "removal", ?asked, "removal: refused — the name asked is no name, so nobody of it is in the group");
        return Outcome::NotInGroup;
    };
    let rail = &at.host.rail;
    let group = asked.group.as_str();
    let joined = asked.joined;
    let outcome = match rail.commit(group, &HandOnTo { person, joined }) {
        Ok(Ok(_)) => {
            at.door.ring.wake();
            Outcome::Handed
        }
        Ok(Err(Refused::NoStanding)) => Outcome::NotIn,
        Ok(Err(Refused::Unsettled)) => Outcome::Fault,
        Ok(Err(Refused::Command(HandOnRefusal::NotInGroup))) => Outcome::NotInGroup,
        Ok(Err(Refused::Command(HandOnRefusal::Which(joined)))) => Outcome::Which { joined },
        Ok(Err(Refused::Command(HandOnRefusal::NotKeeper))) => match rail.register(group) {
            Ok(register) => Outcome::NotDecider {
                decides: decides(&register),
            },
            Err(e) => {
                tracing::warn!(target: "removal", group, error = %e,
                    "removal: a hand-on refused, and the register that names who decides cannot be read");
                Outcome::NotDeciderUnread
            }
        },
        Err(e) => {
            tracing::warn!(target: "removal", group, error = %e, "removal: the group's record cannot be written");
            Outcome::Fault
        }
    };
    tracing::debug!(target: "removal", group, ?outcome, "removal: a hand-on asked; the round woken when written");
    outcome
}
