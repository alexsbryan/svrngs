// SPDX-License-Identifier: AGPL-3.0-or-later
//! Removal, as this computer's person asks it (U7; DESIGN §6 "Removal has
//! one decider"; PLAN §3, 2026-10-09: removing someone stays in the command).
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
//! A cut is told to the removed person's computers once, as it lands, which
//! their node takes since this computer stands in their copy of the
//! register; a leave to the leaver's own others (#179), and by the round to
//! every other computer of the group. Each is told the cut with what it
//! covers (#211): asked its digest, the acts its answer carries never taken,
//! then sent the signer's acts through the cut and the hand-ons it lacks
//! (#184), the selection `ring_routes.rs` `told_with` makes, until its
//! digest covers the cut. A cut computer that is off as it lands is told by
//! the first computer it asks to sync the group, by that selection, and
//! nothing after (`ring_routes.rs`, `told_cut`); every lender's door refuses
//! it all the same.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::authority::Unlawful;
use crate::iroh::{actor_key, Contact};
use crate::node::Answered;
use crate::node::{Asking, Host};
use crate::peer_wire::RingSyncRequest;
use crate::ring_routes::told_with;
use crate::ring_sync::{ExchangeStop, To};
use crate::{
    register::{Member, Register},
    Command, Device, HandOnRefusal, HandOnTo, Joined, Leave, Office, Person, Refused, Removal,
    RemovalRefusal, Take, NO_BUDGET,
};

/// Where the command asks a removal.
pub const REMOVALS_ROUTE: &str = "/v1/removals";

/// Where the command asks a hand-on.
pub const HANDOVERS_ROUTE: &str = "/v1/handovers";

/// Where the command and the home page ask this computer's person's leave
/// (groups-leave), of the group a [`LeaveAsk`] names.
pub const LEAVES_ROUTE: &str = "/v1/leaves";

/// Where the command and the home page ask this computer to take the order
/// of removals a hand-on left waiting, of the group a [`LeaveAsk`] names.
pub const ACCEPTS_ROUTE: &str = "/v1/accepts";

/// Where the command and the home page ask its keeper's Move of the group
/// a [`LeaveAsk`] names (svrn-docs EUDOXIA_JUDGE §2e §4).
pub const ADOPTS_ROUTE: &str = "/v1/adopts";

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
    /// The person decides removals once they accept it on their computer.
    Offered,
    /// This computer's person decides removals now: they accepted it, or
    /// took back their own hand-on.
    Taken,
    /// No hand-on waits for this computer to accept or take back.
    NotOffered,
    /// The group's record would not count it, so nothing was written.
    Uncounted,
    /// This computer's person left the group.
    Left,
    /// This computer's person left a sources-v1 group whose order of
    /// removals they kept: nobody keeps it now, until the others recover it.
    LeftVacant,
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
    /// The group moved to its new decider: a hand-on waits for its Accept.
    Moved,
    /// `person`'s computer answered with an older svrngs, so nothing
    /// moved: it updates first.
    Behind { person: Person },
    /// `person`'s computer, or one not named, did not answer, get ready or
    /// hold what this one holds in time, so nothing moved: Move again.
    Unready { person: Option<Person> },
    /// The record cannot move as it is (a keeper's fork, an unsettled or
    /// pruned history): only making the group again does.
    Stuck,
}

impl IntoResponse for Outcome {
    fn into_response(self) -> Response {
        let status = match self {
            Outcome::Out
            | Outcome::Asked { .. }
            | Outcome::Handed
            | Outcome::Offered
            | Outcome::Taken
            | Outcome::Left
            | Outcome::LeftVacant
            | Outcome::Moved => StatusCode::OK,
            Outcome::Fault => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::CONFLICT,
        };
        (status, Json(self)).into_response()
    }
}

/// The routes, for [`crate::api::router`] to serve behind its check.
pub(crate) fn router(asking: Asking) -> Router {
    Router::new()
        .route(REMOVALS_ROUTE, post(remove))
        .route(HANDOVERS_ROUTE, post(hand_over))
        .route(LEAVES_ROUTE, post(leave))
        .route(ACCEPTS_ROUTE, post(take))
        .route(ADOPTS_ROUTE, post(adopt))
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
/// A cut is told once, with what it covers ([`tell`]), to the computers
/// that would not hear of it by the round: a removal's to the removed person's, a leave's to
/// every other computer, the leaver's own others too (#179), since the
/// leaver's round no longer reaches the group (the operator, 2026-10-14:
/// "Record only").
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
                Refused::Uncounted(_) => Outcome::Uncounted,
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
    if cut.is_empty() {
        return Outcome::Asked {
            decides: decides(&after),
        };
    }
    // The computers it cut, told now, each with the relay the record named
    // before the cut (relay-in-record): for a leave, the leaver's others.
    // The rest hear it by the round: the keeper's exchange, or for a leave,
    // which leaves this computer standing for nobody, its round's telling
    // until each has answered (#190).
    let computers = (before.members.into_iter())
        .filter(|(k, m)| m.device == Device::Computer && *k != at.host.actor && cut.contains(k))
        .map(|(k, m)| (k, m.relay))
        .collect();
    let (host, group) = (at.host.clone(), group.to_string());
    tokio::spawn(async move { tell(&host, &group, computers).await });
    if left && after.office == Office::Vacant {
        Outcome::LeftVacant
    } else if left {
        Outcome::Left
    } else {
        Outcome::Out
    }
}

/// Tell each of `computers` the cut this computer signed, with what it
/// covers ([`told`]), all at once; traced, and the computers that answered —
/// hold it, or refused it — returned, so the round tells a leave again only
/// to the rest (#190).
pub(crate) async fn tell(
    host: &Host,
    group: &str,
    computers: Vec<(String, Option<svrngs_core::relay::Url>)>,
) -> Vec<String> {
    let told = computers.into_iter().filter_map(|(actor, relay)| {
        let Some(key) = actor_key(&actor) else {
            tracing::debug!(target: "removal", group, actor, "removal: the removed key is no computer's address, so it is not told");
            return None;
        };
        Some(async move {
            let contact = Contact::named(key, &host.meeting(group), relay.as_ref());
            let told = told(host, group, (&host.transport, &contact)).await;
            told.inspect(|how| tracing::debug!(target: "removal", group, actor, how, "removal: a computer told the cut"))
                .inspect_err(|e| tracing::warn!(target: "removal", group, actor, error = e, "removal: a computer not told the cut with what it covers, so it is asked again, never sent the cut alone"))
                .is_ok()
                .then_some(actor)
        })
    });
    let answered = n0_future::join_all(told.collect::<Vec<_>>()).await;
    answered.into_iter().flatten().collect()
}

/// The computer at `to` told the cut this computer signed in `group` with
/// what it covers (#211): asked its digest first, the acts each answer
/// carries never taken, so the leaver's computer takes nothing more of the
/// group; then sent what it lacks of the cut's chain, one budget at a time
/// (`ring_routes.rs` `told_with`), until its digest covers the cut. How it
/// ended once it does, or it refuses; else why not, its digest not moving
/// among them, so a chain past the budget is never taken for told.
async fn told(host: &Host, group: &str, to: To<'_>) -> Result<&'static str, String> {
    let (mut ops, mut theirs) = (Vec::new(), None);
    loop {
        let request = RingSyncRequest {
            namespace: group.to_string(),
            digest: host.rail.digest(group).map_err(|e| e.to_string())?,
            ops,
        };
        let answer = match crate::ring_sync::post(to, &request).await {
            Ok(answer) => answer.digest,
            Err(ExchangeStop::Refused { .. }) => return Ok("it refused this computer"),
            Err(ExchangeStop::Failed(e)) => return Err(e),
        };
        tracing::debug!(target: "removal", group, sent = request.ops.len(), "removal: a computer asked its digest, or sent the cut's chain it lacked");
        if theirs.as_ref() == Some(&answer) {
            return Err("its digest did not move with the acts sent".into());
        }
        let lacks = host.rail.ops_missing_from_within(group, &answer, NO_BUDGET);
        let Some(next) = told_with(lacks.map_err(|e| e.to_string())?.0, &host.actor, group) else {
            return Ok("its digest covers the cut");
        };
        (ops, theirs) = (next, Some(answer));
    }
}

async fn hand_over(State(at): State<Asking>, Json(asked): Json<RemovalAsk>) -> Outcome {
    let Ok(person) = Person::parse(&asked.person) else {
        tracing::debug!(target: "removal", ?asked, "removal: refused — the name asked is no name, so nobody of it is in the group");
        return Outcome::NotInGroup;
    };
    let group = asked.group.as_str();
    let joined = asked.joined;
    // A sources-v1 group's hand-on waits on its recipient's Accept.
    let written = |r: Register| match r.office {
        Office::Pending { .. } => Outcome::Offered,
        _ => Outcome::Handed,
    };
    handing(&at, group, &HandOnTo { person, joined }, written)
}

/// This computer takes the order of removals a hand-on left waiting: its
/// person's own yes to a hand-on to it, or its taking back its own
/// (svrngs-core's [`Take`]). Asked only by this computer's person.
async fn take(State(at): State<Asking>, Json(asked): Json<LeaveAsk>) -> Outcome {
    let me = Take(at.host.actor.clone());
    handing(&at, &asked.group, &me, |_| Outcome::Taken)
}

/// `command`, a hand-on or a take, written in `group`, and what happened:
/// `written` of the register after it, once written.
fn handing<C: Command<Refusal = HandOnRefusal>>(
    at: &Asking,
    group: &str,
    command: &C,
    written: impl FnOnce(Register) -> Outcome,
) -> Outcome {
    let rail = &at.host.rail;
    let outcome = match rail.commit(group, command) {
        Ok(Ok(_)) => match rail.register(group) {
            Ok(register) => written(register),
            Err(e) => {
                tracing::warn!(target: "removal", group, error = %e, "removal: written, and the register cannot be read back");
                Outcome::Fault
            }
        },
        Ok(Err(Refused::NoStanding)) => Outcome::NotIn,
        Ok(Err(Refused::Unsettled)) => Outcome::Fault,
        Ok(Err(Refused::Uncounted(_))) => Outcome::Uncounted,
        Ok(Err(Refused::Command(HandOnRefusal::NotInGroup))) => Outcome::NotInGroup,
        Ok(Err(Refused::Command(HandOnRefusal::Which(joined)))) => Outcome::Which { joined },
        Ok(Err(Refused::Command(HandOnRefusal::NotOffered))) => Outcome::NotOffered,
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
    tracing::debug!(target: "removal", group, ?outcome, "removal: a hand-on or a take asked; the round woken when written");
    outcome
}

/// Whether a computer's answer lets its keeper's Move go on: it reads line
/// version 15 ([`crate::version::moves`]). One older, or silent, is waited
/// for, since a line it cannot read cuts its sync off for the whole group.
pub(crate) fn moves(answer: &Answered) -> bool {
    matches!(answer, Answered::Lends(_, v) if crate::version::moves(Some(*v)))
}

/// The keeper's Move (§2e §4), asked only by this computer's person: every
/// other standing computer reads line version 15, holds what this one
/// holds and signs a Ready; then this computer signs the Adopt over all of
/// it and gives it to each. Nothing is signed by this computer until every
/// other one answered, and a stop says whose computer stopped it.
async fn adopt(State(at): State<Asking>, Json(asked): Json<LeaveAsk>) -> Outcome {
    let (host, group) = (&at.host, asked.group.as_str());
    let Ok(register) = host.rail.register(group) else {
        tracing::warn!(target: "removal", group, "move: the group's register cannot be read");
        return Outcome::Fault;
    };
    tracing::debug!(target: "removal", group, decided = ?register.decided, keeper = ?register.keeper, "move: asked; only the keeper of a group on its old fold moves it");
    if register.decided.interpreted() {
        return Outcome::Moved;
    }
    if register.keeper.as_ref() != Some(&host.actor) {
        return Outcome::NotDecider {
            decides: decides(&register),
        };
    }
    let (meeting, t) = (host.meeting(group), &*host.transport);
    let mine = |(k, m): &(&String, &Member)| m.device == Device::Computer && **k != host.actor;
    let unready = |person: &Person| Outcome::Unready {
        person: Some(person.clone()),
    };
    let mut others = Vec::new();
    for (key, m) in register.members.iter().filter(mine) {
        let at = actor_key(key).map(|k| Contact::named(k, &meeting, m.relay.as_ref()));
        let answer = match &at {
            Some(at) => crate::node::ask_lends(t, key, at.clone()).await,
            None => Answered::Unanswered,
        };
        tracing::debug!(target: "removal", group, key, moves = moves(&answer), "move: whether a computer reads line version 15");
        let person = m.person.clone();
        match at.filter(|_| moves(&answer)) {
            Some(at) => others.push((person, at)),
            // Off, or not reached: it is asked to be on, not to update.
            None if !matches!(answer, Answered::Lends(..)) => return unready(&person),
            None => return Outcome::Behind { person },
        }
    }
    for (person, at) in &others {
        if !synced(host, group, (t, at)).await || !readied((t, at), group).await {
            return unready(person);
        }
    }
    if !matches!(host.rail.commit(group, &crate::Ready), Ok(Ok(_))) {
        return Outcome::Fault;
    }
    // Twice round: every Ready reaches every computer, then each is shown
    // to hold all of it.
    for (person, at) in others.iter().chain(&others) {
        if !synced(host, group, (t, at)).await {
            return unready(person);
        }
    }
    let drawn = host.rail.adoption(group);
    tracing::debug!(target: "removal", group, drawn = ?drawn.as_ref().map(|d| d.as_ref().err()), "move: the Adopt this computer may sign");
    let adopted = match drawn {
        Ok(Ok(drawn)) => host.rail.commit(group, &crate::Adopt(drawn)),
        Ok(Err(why)) => Ok(Err(Refused::Command(why))),
        Err(e) => Err(e),
    };
    match adopted {
        Ok(Ok(_)) => {}
        Ok(Err(Refused::Command(Unlawful::Missing | Unlawful::Ready))) => {
            return Outcome::Unready { person: None }
        }
        Ok(Err(_)) => return Outcome::Stuck,
        Err(_) => return Outcome::Fault,
    }
    for (_, at) in &others {
        synced(host, group, (t, at)).await;
    }
    Outcome::Moved
}

/// One exchange with the computer at `to`, and whether it ended holding
/// every line this computer holds of `group`.
async fn synced(host: &Host, group: &str, to: To<'_>) -> bool {
    let stop = crate::ring_sync::exchange(to, &host.rail, group).await.stop;
    let mine = host.rail.digest(group).ok().filter(|_| stop.is_none());
    let request = mine.map(|digest| RingSyncRequest {
        namespace: group.into(),
        digest,
        ops: Vec::new(),
    });
    let theirs = match request {
        Some(request) => crate::ring_sync::post(to, &request).await.ok(),
        None => None,
    };
    let lacks = theirs.and_then(|t| {
        host.rail
            .ops_missing_from_within(group, &t.digest, NO_BUDGET)
            .ok()
    });
    let holds = lacks.is_some_and(|(ops, _)| ops.is_empty());
    tracing::debug!(target: "removal", group, holds, ?stop, "move: a computer exchanged with, and whether it holds every line here");
    holds
}

/// Whether the computer at `to` signed its Ready in `group` when asked
/// (ring_routes.rs `ready`).
async fn readied(to: To<'_>, group: &str) -> bool {
    let body = serde_json::json!({ "group": group }).to_string();
    let asked = crate::iroh::posted(crate::ring_routes::READY_ROUTE, body);
    let answer = to.0.fetch(to.1, asked).await;
    tracing::debug!(target: "removal", group, status = ?answer.as_ref().map(|r| r.status().as_u16()).ok(), "move: a computer asked to sign its Ready");
    answer.is_ok_and(|r| r.status().is_success())
}
