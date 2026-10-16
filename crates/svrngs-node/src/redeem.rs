// SPDX-License-Identifier: AGPL-3.0-or-later
//! Redeeming a link (DESIGN §6, "A link admits each person once"; §8, U2's
//! inventory, its found/join/internal and acceptor rows). New code, in the
//! shape of upstream's mesh join (`commonwealth/crates/commonwealth-rails/src/
//! join.rs:116-229` and `internal.rs:222-295` at `e1a43bcd2`): the joiner
//! dials the issuer by key and proves it holds both the link's key and its
//! own; the issuer checks, persists, then answers.
//!
//! A link carries only the issuer's key and the link's secret (u2-8), so
//! the joiner first asks what it cannot know: an [`AskRequest`], the link's
//! key and the joiner's, signed by the link's key, which the issuer answers
//! ([`Asked`]) with the group its own `Invite` naming that key is in, the
//! group's seed as the command keeps it ([`SeedText`]: the seed is the
//! command's, DESIGN §5, so the node answers it as read and never parses
//! it), and the `Invite`'s id. Whether that `Invite` still lets anyone in
//! is not asked here: the redemption's fold decides it, once.
//!
//! The joiner then presents a [`RedeemRequest`] on [`Alpn::Redeem`], the one
//! protocol a key standing in no group may speak, serving [`router`]'s two
//! routes: the group's id, the `Invite`'s id, the person's name, its new key,
//! that key's consent ([`crate::Redemption`]) and what holds the key, a computer
//! or a browser ([`crate::Device`], u4-1), all signed by the link's secret key. The issuer's node checks that the dialer is the new key, that the
//! link's key signed, that the name parses ([`Name::kept`], the core's one
//! rule; u2-9), that it holds the group, that the `Invite` is its own
//! and names the link's key; then, under the door's write lock, that the
//! new key does not stand already, and that `membership` over the journal
//! with the redeeming `Admit` in it counts that `Admit` — the one decider of
//! a lapsed, closed or voided link, and of the consent (svrngs-core
//! membership.rs). Only then is the `Admit` written, the round woken so
//! the group's other computers hear of the newcomer now, and the answer
//! sent.
//!
//! Every refusal is answered in one sentence, [`REFUSAL`], so the answer
//! never says which groups the node holds or why — but a name that does not
//! parse, answered with why it is not a name, which says nothing of groups,
//! and a name a standing person of the link's group goes by
//! ([`RedeemRefusal::NameTaken`], bash-person), answered `409` with that
//! person's name alone as its `error`, which the link's holder may know: the joiner words
//! it and asks another name. The trace at debug says which check it was. A key that already stands is answered success and
//! nothing is written, so a joiner whose answer was lost may open the link
//! again.
//!
//! The two requests, their signing and the check of the link's signature
//! are svrngs-core's (`svrngs_core::joining`, u4-2), so a browser's page
//! signs them with the command's body; they are re-exported here.

use std::path::Path;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::iroh::{Alpn, Dialer, Endpoint, EndpointAddr};
use crate::register::Standing;
use crate::ring_routes::err;
use crate::ring_sync::AskRound;
use crate::{actor_of, Name, NotAName, OpId, Person, RailError, RingRail};
use svrngs_core::{RedeemRefusal, Refused as CommitRefused};

pub use svrngs_core::joining::{AskRequest, Asked, RedeemRequest};

/// The redemption's path.
const ROUTE: &str = "/redeem";

/// The ask's path.
const ASK_ROUTE: &str = "/ask";

/// How the issuer's node reads a group's seed, in the group's directory,
/// to answer an ask: `Ok(None)` when it keeps none there. The command's
/// (`svrngs::group::seed_text`), handed to [`crate::node::Node::start`].
pub type SeedText = fn(&Path) -> Result<Option<String>, String>;

/// The one answer to every refusal: it names no group and no reason.
pub const REFUSAL: &str = "this link lets nobody in";

/// The answer when the issuer's node could not judge — its own journal
/// unreadable — which is not the link's fault and is not a refusal.
pub const UNJUDGED: &str = "the computer that sent this link cannot let anyone in just now";

/// What a redemption did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Redeemed {
    /// The `Admit` this node wrote.
    Admitted(OpId),
    /// The key stood already; nothing was written.
    AlreadyIn,
}

/// Why the issuer's node did not admit the joiner — one per check, each
/// traced, and answered as [`REFUSAL`] but for [`NotAdmitted::Unjudged`].
#[derive(Debug)]
enum NotAdmitted {
    /// The request did not come on an iroh stream.
    NoDialer,
    /// The key presented is not the key that dialled.
    NotTheDialer,
    /// The link's key did not sign the request.
    LinkUnsigned,
    /// The person's name does not parse; answered with why.
    NotAName(NotAName),
    /// This node holds no such group, or holds it with no seed.
    NotHeld,
    /// Asked, no group held here holds this node's `Invite` naming the link's key.
    NoInvite,
    /// The redemption's decider refused it ([`svrngs_core::Redeem`]).
    Decided(RedeemRefusal),
    /// This node could not read its own journal.
    Unjudged(RailError),
}

/// What the routes read: this node's journals, how it reads a seed, and
/// the round, woken once an `Admit` is written.
#[derive(Clone)]
struct Issuer {
    rail: Arc<RingRail>,
    seeds: SeedText,
    ring: AskRound,
}

/// The ask and redemption routes, served on [`Alpn::Redeem`] to any dialer.
pub fn router(rail: Arc<RingRail>, seeds: SeedText, ring: AskRound) -> axum::Router {
    axum::Router::new()
        .route(ASK_ROUTE, post(ask_route))
        .route(ROUTE, post(redeem_route))
        .with_state(Issuer { rail, seeds, ring })
}

async fn ask_route(
    State(issuer): State<Issuer>,
    dialer: Option<Extension<Dialer>>,
    Json(req): Json<AskRequest>,
) -> Response {
    let asked = serve_ask(&issuer, dialer.map(|Extension(d)| d.actor()), &req);
    answer("ask", &req.key, asked)
}

async fn redeem_route(
    State(issuer): State<Issuer>,
    dialer: Option<Extension<Dialer>>,
    Json(req): Json<RedeemRequest>,
) -> Response {
    let redeemed = serve_redeem(&issuer.rail, dialer.map(|Extension(d)| d.actor()), &req);
    // The group's other computers hear of the newcomer now, not at the
    // round's next interval: until they hold the `Admit` each refuses the
    // newcomer's computer, and waiting on the interval was 56 of a
    // newcomer's 57 s (`demo/runs/first-minute/2026-10-12/README.md`).
    if let Ok(Redeemed::Admitted(admit)) = &redeemed {
        tracing::debug!(target: "redeem", key = %req.key, admit = %admit,
            "redeem: the round woken, so the group's computers hear of the newcomer now");
        issuer.ring.wake();
    }
    answer("redeem", &req.key, redeemed)
}

/// A decision as the wire carries it: what was done, [`UNJUDGED`], or the
/// one [`REFUSAL`].
fn answer<T: Serialize>(route: &str, key: &str, decided: Result<T, NotAdmitted>) -> Response {
    match decided {
        Ok(done) => (StatusCode::OK, Json(done)).into_response(),
        Err(NotAdmitted::Unjudged(e)) => {
            tracing::warn!(target: "redeem", route, error = %e,
                "redeem: could not judge — this node's own journal is unreadable");
            err(StatusCode::SERVICE_UNAVAILABLE, UNJUDGED)
        }
        Err(NotAdmitted::NotAName(why)) => {
            tracing::debug!(target: "redeem", route, key, why = ?why,
                "redeem: refused — the name is not one; answered with why, naming no group");
            err(StatusCode::FORBIDDEN, why.to_string())
        }
        Err(NotAdmitted::Decided(RedeemRefusal::NameTaken(holder))) => {
            tracing::debug!(target: "redeem", route, key, %holder,
                "redeem: refused — a standing person goes by the name; answered 409 with theirs");
            err(StatusCode::CONFLICT, holder.to_string())
        }
        Err(NotAdmitted::Decided(why)) => {
            tracing::debug!(target: "redeem", route, key, why = ?why,
                "redeem: the decider refused, answered in the one sentence every refusal shares");
            err(StatusCode::FORBIDDEN, REFUSAL)
        }
        Err(why) => {
            tracing::debug!(target: "redeem", route, key, why = ?why,
                "redeem: refused, in the one sentence every refusal shares");
            err(StatusCode::FORBIDDEN, REFUSAL)
        }
    }
}

/// The ask, as an adapter: parse ([`parse_ask`]), ask the runtime for this
/// node's newest `Invite` naming the link's key, and answer that group's seed
/// as the command keeps it.
fn serve_ask(
    issuer: &Issuer,
    dialer: Option<String>,
    req: &AskRequest,
) -> Result<Asked, NotAdmitted> {
    parse_ask(dialer.as_deref(), req)?;
    let (group, invite, person) = issuer
        .rail
        .invite_for(&req.link)
        .map_err(NotAdmitted::Unjudged)?
        .ok_or(NotAdmitted::NoInvite)?;
    let dir = issuer
        .rail
        .group_dir(&group)
        .map_err(NotAdmitted::Unjudged)?;
    let seed = (issuer.seeds)(&dir)
        .map_err(|e| NotAdmitted::Unjudged(RailError::Io(e)))?
        .ok_or(NotAdmitted::NotHeld)?;
    tracing::debug!(target: "redeem", group, key = %req.key, invite = %invite, ?person,
        "redeem: asked — the link's key is named by this node's Invite; answering its group's seed, and a device link's person");
    Ok(Asked {
        group,
        seed,
        invite,
        person,
    })
}

/// An ask judged on what arrived alone: the dialer is the key asking, and
/// the link's key signed. Reads nothing, so a refused ask opens no journal.
fn parse_ask(dialer: Option<&str>, req: &AskRequest) -> Result<(), NotAdmitted> {
    presented(dialer, &req.key, || req.signed_by_link())
}

/// What both routes require of a request before reading it: a dialer, the
/// request's key that dialer's, and the link's key its signer — checked in
/// that order, so `signed` runs only for the key that dialled.
fn presented(
    dialer: Option<&str>,
    key: &str,
    signed: impl FnOnce() -> bool,
) -> Result<(), NotAdmitted> {
    let dialer = dialer.ok_or(NotAdmitted::NoDialer)?;
    if dialer != key {
        return Err(NotAdmitted::NotTheDialer);
    }
    if !signed() {
        return Err(NotAdmitted::LinkUnsigned);
    }
    Ok(())
}

/// The redemption, as an adapter: parse ([`parse_redeem`]); check this node
/// holds the group; hand [`svrngs_core::Redeem`] to the runtime's `commit`,
/// which decides it under the journal's write lock.
fn serve_redeem(
    rail: &RingRail,
    dialer: Option<String>,
    req: &RedeemRequest,
) -> Result<Redeemed, NotAdmitted> {
    let person = parse_redeem(dialer.as_deref(), req)?;
    let group = req.group.as_str();
    match rail.standing(group) {
        Ok(Standing::Members(_)) => {}
        Ok(_) => return Err(NotAdmitted::NotHeld),
        Err(e) => return Err(NotAdmitted::Unjudged(e)),
    }
    let command = svrngs_core::Redeem {
        invite: req.invite.clone(),
        person,
        key: req.key.clone(),
        cosig: req.cosig.clone(),
        device: req.device,
        link: req.link.clone(),
        issuer: actor_of(rail.signer()),
    };
    match rail
        .commit(group, &command)
        .map_err(NotAdmitted::Unjudged)?
    {
        Ok(recorded) if recorded.is_empty() => {
            tracing::debug!(target: "redeem", group, key = %req.key,
                "redeem: the key stands already — answered success, nothing written");
            Ok(Redeemed::AlreadyIn)
        }
        Ok(recorded) => {
            let admit = recorded.lines()[0].id.clone();
            tracing::debug!(target: "redeem", group, key = %req.key, admit = %admit, device = ?req.device,
                "redeem: admitted — the Admit is written, naming the device the request named");
            Ok(Redeemed::Admitted(admit))
        }
        Err(CommitRefused::NoStanding) => Err(NotAdmitted::Unjudged(RailError::NotInRoster {
            actor: actor_of(rail.signer()),
            namespace: group.to_string(),
        })),
        Err(CommitRefused::Unsettled) => Err(NotAdmitted::Unjudged(RailError::Unsettled(
            group.to_string(),
        ))),
        Err(CommitRefused::Command(why)) => Err(NotAdmitted::Decided(why)),
    }
}

/// A redemption judged on what arrived alone: the dialer is the new key,
/// the link's key signed, and the name parses. Reads nothing, so a refused
/// redemption opens no journal.
fn parse_redeem(dialer: Option<&str>, req: &RedeemRequest) -> Result<Person, NotAdmitted> {
    presented(dialer, &req.key, || req.signed_by_link())?;
    Name::kept(req.person.clone())
        .map(Person::from)
        .map_err(NotAdmitted::NotAName)
}

/// Why a joiner was not let in.
#[derive(Debug, thiserror::Error)]
pub enum RedeemError {
    /// The issuer's node answered, and refused, in its words: [`REFUSAL`]
    /// from a node of this build.
    #[error("refused: {0}")]
    Refused(String),
    /// The issuer's node answered that someone in the group already goes
    /// by the name: theirs, as the group names them.
    #[error("someone in the group already goes by {0}")]
    NameTaken(Name),
    /// The issuer's node answered that it could not judge: [`UNJUDGED`].
    #[error("the issuer could not judge")]
    Unjudged,
    /// No answer, or one this build cannot read.
    #[error("no answer: {0}")]
    NoAnswer(String),
}

/// Ask the issuer's node at `issuer` what `req`'s link is for, dialling
/// from `endpoint` on [`Alpn::Redeem`].
pub async fn ask(
    endpoint: &Endpoint,
    issuer: EndpointAddr,
    req: &AskRequest,
) -> Result<Asked, RedeemError> {
    call(endpoint, issuer, ASK_ROUTE, req).await
}

/// Present `req` to the issuer's node at `issuer`, dialling from `endpoint`
/// on [`Alpn::Redeem`].
pub async fn redeem(
    endpoint: &Endpoint,
    issuer: EndpointAddr,
    req: &RedeemRequest,
) -> Result<Redeemed, RedeemError> {
    call(endpoint, issuer, ROUTE, req).await
}

/// Post `body` to `route` on the issuer's node, and read its answer: on a
/// connection dialled for it alone on [`Alpn::Redeem`], uncapped, the whole
/// within [`crate::iroh::ANSWERED_WITHIN`] (arch-peer (c), where a bridge
/// and the round's reqwest client carried it, with the same 10 s).
async fn call<T: DeserializeOwned>(
    endpoint: &Endpoint,
    issuer: EndpointAddr,
    route: &str,
    body: &impl Serialize,
) -> Result<T, RedeemError> {
    use crate::iroh::{dialled, on_stream, whole};
    let no_answer = |e: String| RedeemError::NoAnswer(e);
    let body = serde_json::to_vec(body).map_err(|e| no_answer(e.to_string()))?;
    let ask = axum::http::Request::post(route)
        .header(axum::http::header::CONTENT_TYPE, "application/json");
    let ask = ask
        .body(body.into())
        .map_err(|e| no_answer(e.to_string()))?;
    tracing::debug!(target: "redeem", route, issuer = %issuer.id.fmt_short(), "redeem: dialling the issuer by key");
    let answered = whole(async {
        on_stream(
            &dialled(endpoint, issuer, Alpn::Redeem.bytes(), None).await?,
            ask,
        )
        .await
    });
    let answer = answered.await.map_err(|e| no_answer(e.to_string()))?;
    let status = answer.status();
    tracing::debug!(target: "redeem", route, status = status.as_u16(), "redeem: the issuer answered");
    let said = answer.body();
    match status {
        StatusCode::OK => serde_json::from_slice(said)
            .map_err(|e| RedeemError::NoAnswer(format!("an answer this build cannot read: {e}"))),
        StatusCode::FORBIDDEN => Err(RedeemError::Refused(
            String::from_utf8_lossy(said).into_owned(),
        )),
        StatusCode::CONFLICT => {
            let said: serde_json::Value = serde_json::from_slice(said).unwrap_or_default();
            let holder = said["error"].as_str().unwrap_or_default().to_string();
            Err(Name::kept(holder).map_or_else(
                |e| RedeemError::NoAnswer(format!("a name this build cannot read: {e}")),
                RedeemError::NameTaken,
            ))
        }
        StatusCode::SERVICE_UNAVAILABLE => Err(RedeemError::Unjudged),
        other => Err(RedeemError::NoAnswer(format!("HTTP {other}"))),
    }
}

#[cfg(test)]
mod tests;
