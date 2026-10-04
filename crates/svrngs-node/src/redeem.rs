// SPDX-License-Identifier: AGPL-3.0-or-later
//! Redeeming a link (DESIGN §6, "A link admits each person once"; §8, U2's
//! inventory, its found/join/internal and acceptor rows). New code, in the
//! shape of upstream's mesh join (`commonwealth/crates/commonwealth-rails/src/
//! join.rs:116-229` and `internal.rs:222-295` at `e1a43bcd2`): the joiner
//! dials the issuer by key and proves it holds both the link's key and its
//! own; the issuer checks, persists, then answers.
//!
//! The joiner presents a [`RedeemRequest`] on [`Alpn::Redeem`], the one
//! protocol a key standing in no group may speak, serving [`router`]'s one
//! route: the group's id, the `Invite`'s id, the person's name, its new key
//! and that key's consent ([`Redemption`]), all signed by the link's secret
//! key. The issuer's node checks that the dialer is the new key, that the
//! link's key signed, that it holds the group, that the `Invite` is its own
//! and names the link's key; then, under the door's write lock, that the
//! new key does not stand already, and that `membership` over the journal
//! with the redeeming `Admit` in it counts that `Admit` — the one decider of
//! a lapsed, closed or voided link, and of the consent (svrngs-core
//! membership.rs). Only then is the `Admit` written, and the answer sent.
//!
//! Every refusal is answered in one sentence, [`REFUSAL`], so the answer
//! never says which groups the node holds or why; the trace at debug says
//! which check it was. A key that already stands is answered success and
//! nothing is written, so a joiner whose answer was lost may open the link
//! again.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use crate::iroh::{Alpn, Dialer, Endpoint, EndpointAddr, HttpBridge};
use crate::register::{membership_in, Standing};
use crate::ring_routes::err;
use crate::ring_sync::peer_client;
use crate::{
    actor_of, derive_id, field, sign_message, verify_hex, Ed25519Verifier, InviteFault, OpId,
    Person, RailAct, RailError, Redemption, RingRail, SigningKey,
};

/// The route's one path.
const ROUTE: &str = "/redeem";

/// Domain separator for the link key's signature over a request, so it is
/// never valid as an act's signature or a consent, nor either as it.
const REQUEST_DOMAIN: &[u8] = b"svrngs-redeem-request:";

/// The one answer to every refusal: it names no group and no reason.
pub const REFUSAL: &str = "this link lets nobody in";

/// The answer when the issuer's node could not judge — its own journal
/// unreadable — which is not the link's fault and is not a refusal.
pub const UNJUDGED: &str = "the computer that sent this link cannot let anyone in just now";

/// What a joiner presents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedeemRequest {
    /// The group's id, as the seed hashes it.
    pub group: String,
    /// The id of the issuer's `Invite`, as the link carries it.
    pub invite: OpId,
    /// The name the joiner goes by in the group.
    pub person: Person,
    /// The new key, actor hex: the key the joiner dials with.
    pub key: String,
    /// The new key's consent, as a redeeming `Admit` carries it.
    pub cosig: String,
    /// The link's public key, actor hex.
    pub link: String,
    /// The link's secret key's signature over everything above.
    pub sig: String,
}

impl RedeemRequest {
    /// `joining` asks to join `group` as `person`, through the link whose
    /// secret is `link` and whose `Invite` is `invite`.
    pub fn sign(
        link: &SigningKey,
        joining: &SigningKey,
        group: &str,
        invite: &OpId,
        person: &Person,
    ) -> Self {
        let consent = Redemption::sign(joining, group, invite, person);
        let mut req = Self {
            group: group.to_string(),
            invite: invite.clone(),
            person: person.clone(),
            key: actor_of(joining),
            cosig: consent.sig,
            link: actor_of(link),
            sig: String::new(),
        };
        req.sig = sign_message(link, &req.message());
        req
    }

    /// `DOMAIN || group || invite[32] || person || key || cosig || link`,
    /// every variable field preceded by its `u32` big-endian length.
    fn message(&self) -> Vec<u8> {
        let mut msg = REQUEST_DOMAIN.to_vec();
        field(&mut msg, self.group.as_bytes());
        field(&mut msg, self.invite.as_bytes());
        field(&mut msg, self.person.as_str().as_bytes());
        field(&mut msg, self.key.as_bytes());
        field(&mut msg, self.cosig.as_bytes());
        field(&mut msg, self.link.as_bytes());
        msg
    }

    /// Whether the link's key signed this request. `false` on anything
    /// malformed.
    fn signed_by_link(&self) -> bool {
        verify_hex(&self.link, &self.message(), &self.sig)
    }
}

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
/// traced, and answered as [`REFUSAL`] but for [`Refused::Unjudged`].
#[derive(Debug)]
enum Refused {
    /// The request did not come on an iroh stream.
    NoDialer,
    /// The key presented is not the key that dialled.
    NotTheDialer,
    /// The link's key did not sign the request.
    LinkUnsigned,
    /// This node holds no such group, or holds it with no seed.
    NotHeld,
    /// No `Invite` with that id is held here.
    NoInvite,
    /// The `Invite` is someone else's: only its issuer's node redeems it.
    NotOwn,
    /// The `Invite` names another key than the link's.
    OtherLink,
    /// `membership` refused the `Admit`: a lapsed, closed or voided link,
    /// or a consent that does not verify.
    Fault(InviteFault),
    /// `membership` did not count the `Admit`, nor say why.
    Uncounted,
    /// The key stood already, seen under the write lock.
    Stands,
    /// This node could not read its own journal.
    Unjudged(RailError),
}

/// The redemption route, served on [`Alpn::Redeem`] to any dialer.
pub fn router(rail: Arc<RingRail>) -> axum::Router {
    axum::Router::new()
        .route(ROUTE, post(redeem_route))
        .with_state(rail)
}

async fn redeem_route(
    State(rail): State<Arc<RingRail>>,
    dialer: Option<Extension<Dialer>>,
    Json(req): Json<RedeemRequest>,
) -> Response {
    match decide(&rail, dialer.map(|Extension(d)| d), &req) {
        Ok(redeemed) => (StatusCode::OK, Json(redeemed)).into_response(),
        Err(Refused::Unjudged(e)) => {
            tracing::warn!(target: "redeem", group = %req.group, error = %e,
                "redeem: could not judge — this node's own journal is unreadable");
            err(StatusCode::SERVICE_UNAVAILABLE, UNJUDGED)
        }
        Err(why) => {
            tracing::debug!(target: "redeem", group = %req.group, key = %req.key, why = ?why,
                "redeem: refused, in the one sentence every refusal shares");
            err(StatusCode::FORBIDDEN, REFUSAL)
        }
    }
}

/// The issuer's checks, in the module docs' order.
fn decide(
    rail: &RingRail,
    dialer: Option<Dialer>,
    req: &RedeemRequest,
) -> Result<Redeemed, Refused> {
    let dialer = dialer.ok_or(Refused::NoDialer)?;
    if dialer.actor() != req.key {
        return Err(Refused::NotTheDialer);
    }
    if !req.signed_by_link() {
        return Err(Refused::LinkUnsigned);
    }
    let group = req.group.as_str();
    match rail.standing(group, &Ed25519Verifier) {
        Ok(Standing::Members(_)) => {}
        Ok(_) => return Err(Refused::NotHeld),
        Err(e) => return Err(Refused::Unjudged(e)),
    }
    let journal = rail.journal(group).map_err(Refused::Unjudged)?;
    let seed = journal
        .seed()
        .map_err(Refused::Unjudged)?
        .ok_or(Refused::NotHeld)?;
    let (ops, _) = journal.read().map_err(Refused::Unjudged)?;
    let invite = ops
        .iter()
        .find(|op| {
            derive_id(op, group) == req.invite && matches!(op.kind.act, RailAct::Invite { .. })
        })
        .ok_or(Refused::NoInvite)?;
    let me = rail.signer().actor();
    if invite.actor != me {
        return Err(Refused::NotOwn);
    }
    if !matches!(&invite.kind.act, RailAct::Invite { key, .. } if *key == req.link) {
        return Err(Refused::OtherLink);
    }
    tracing::debug!(target: "redeem", group, key = %req.key, invite = %req.invite,
        "redeem: the dialer holds the link and its key, and the Invite is this node's");
    let act = RailAct::Admit {
        person: req.person.clone(),
        key: req.key.clone(),
        invite: Some(Redemption {
            id: req.invite.clone(),
            sig: req.cosig.clone(),
        }),
    };
    let kept = journal
        .append_if(
            act,
            rail.signer(),
            &seed,
            &Ed25519Verifier,
            |held, admit| judge(held, admit, &seed, group, &req.key),
        )
        .map_err(Refused::Unjudged)?;
    match kept {
        Ok(op) => {
            tracing::debug!(target: "redeem", group, key = %req.key, admit = %op.id,
                "redeem: admitted — the Admit is written");
            Ok(Redeemed::Admitted(op.id))
        }
        Err(Refused::Fault(fault)) => {
            tracing::debug!(target: "redeem", group, key = %req.key, fault = %fault,
                "redeem: membership would not count the Admit, so it is not written");
            Err(Refused::Fault(fault))
        }
        Err(Refused::Stands) => {
            tracing::debug!(target: "redeem", group, key = %req.key,
                "redeem: the key stands already — answered success, nothing written");
            Ok(Redeemed::AlreadyIn)
        }
        Err(why) => Err(why),
    }
}

/// Under the write lock: the key does not stand in `held`, and `membership`
/// over `held` with `admit` in it counts `admit` and gives the key standing.
fn judge(
    held: &[crate::Op<crate::SignedOp>],
    admit: &crate::Op<crate::SignedOp>,
    seed: &crate::Roster,
    group: &str,
    key: &str,
) -> Result<(), Refused> {
    let before = membership_in(held, seed, group, &Ed25519Verifier).map_err(Refused::Unjudged)?;
    if before.standing.contains(key) {
        return Err(Refused::Stands);
    }
    let with: Vec<_> = held.iter().chain([admit]).cloned().collect();
    let after = membership_in(&with, seed, group, &Ed25519Verifier).map_err(Refused::Unjudged)?;
    let id = derive_id(admit, group);
    if let Some(fault) = after.refused.get(&id) {
        return Err(Refused::Fault(fault.clone()));
    }
    if !after.counts(&id) || !after.standing.contains(key) {
        return Err(Refused::Uncounted);
    }
    Ok(())
}

/// Why a joiner was not let in.
#[derive(Debug, thiserror::Error)]
pub enum RedeemError {
    /// The issuer's node answered, and refused, in its words: [`REFUSAL`]
    /// from a node of this build.
    #[error("refused: {0}")]
    Refused(String),
    /// The issuer's node answered that it could not judge: [`UNJUDGED`].
    #[error("the issuer could not judge")]
    Unjudged,
    /// No answer, or one this build cannot read.
    #[error("no answer: {0}")]
    NoAnswer(String),
}

/// Present `req` to the issuer's node at `issuer`, dialling from `endpoint`
/// on [`Alpn::Redeem`].
pub async fn redeem(
    endpoint: &Endpoint,
    issuer: EndpointAddr,
    req: &RedeemRequest,
) -> Result<Redeemed, RedeemError> {
    let bridge = HttpBridge::spawn(endpoint.clone(), issuer, Alpn::Redeem.bytes())
        .await
        .map_err(|e| RedeemError::NoAnswer(format!("no bridge: {e}")))?;
    let http = peer_client().map_err(|e| RedeemError::NoAnswer(e.to_string()))?;
    let url = format!("http://{}{ROUTE}", bridge.local_addr());
    tracing::debug!(target: "redeem", group = %req.group, url, "redeem: dialling the issuer by key");
    let response = http
        .post(&url)
        .json(req)
        .send()
        .await
        .map_err(|e| RedeemError::NoAnswer(e.to_string()))?;
    let status = response.status();
    tracing::debug!(target: "redeem", group = %req.group, status = status.as_u16(), "redeem: the issuer answered");
    match status {
        StatusCode::OK => response
            .json()
            .await
            .map_err(|e| RedeemError::NoAnswer(format!("an answer this build cannot read: {e}"))),
        StatusCode::FORBIDDEN => Err(RedeemError::Refused(
            response.text().await.unwrap_or_default(),
        )),
        StatusCode::SERVICE_UNAVAILABLE => Err(RedeemError::Unjudged),
        other => Err(RedeemError::NoAnswer(format!("HTTP {other}"))),
    }
}
