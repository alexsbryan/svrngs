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
//! routes: the group's id, the `Invite`'s id, the person's name, its new key
//! and that key's consent ([`Redemption`]), all signed by the link's secret
//! key. The issuer's node checks that the dialer is the new key, that the
//! link's key signed, that the name parses ([`Name::kept`], the core's one
//! rule; u2-9), that it holds the group, that the `Invite` is its own
//! and names the link's key; then, under the door's write lock, that the
//! new key does not stand already, and that `membership` over the journal
//! with the redeeming `Admit` in it counts that `Admit` — the one decider of
//! a lapsed, closed or voided link, and of the consent (svrngs-core
//! membership.rs). Only then is the `Admit` written, and the answer sent.
//!
//! Every refusal is answered in one sentence, [`REFUSAL`], so the answer
//! never says which groups the node holds or why — but a name that does not
//! parse, answered with why it is not a name, which says nothing of groups; the trace at debug says
//! which check it was. A key that already stands is answered success and
//! nothing is written, so a joiner whose answer was lost may open the link
//! again.

use std::path::Path;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::iroh::{Alpn, Dialer, Endpoint, EndpointAddr, HttpBridge};
use crate::register::Standing;
use crate::ring_routes::err;
use crate::ring_sync::peer_client;
use crate::{
    actor_of, derive_id, field, sign_message, verify_hex, Ed25519Verifier, Name, NotAName, OpId,
    Person, RailAct, RailError, Redemption, RingRail, SigningKey,
};
use svrngs_core::{commit, Author, RedeemRefusal, Refused as CommitRefused};

/// The redemption's path.
const ROUTE: &str = "/redeem";

/// The ask's path.
const ASK_ROUTE: &str = "/ask";

/// Domain separator for the link key's signature over a request, so it is
/// never valid as an act's signature or a consent, nor either as it.
const REQUEST_DOMAIN: &[u8] = b"svrngs-redeem-request:";

/// Domain separator for the link key's signature over an ask.
const ASK_DOMAIN: &[u8] = b"svrngs-redeem-ask:";

/// How the issuer's node reads a group's seed, in the group's directory,
/// to answer an ask: `Ok(None)` when it keeps none there. The command's
/// (`svrngs::group::seed_text`), handed to [`crate::node::Node::start`].
pub type SeedText = fn(&Path) -> Result<Option<String>, String>;

/// What a joiner holding only a link asks its issuer: the link's public
/// key and the joiner's, actor hex, signed by the link's secret key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskRequest {
    /// The link's public key.
    pub link: String,
    /// The key the joiner dials with.
    pub key: String,
    /// The link's secret key's signature over the two.
    pub sig: String,
}

impl AskRequest {
    /// `joining` asks about the link whose secret key is `link`.
    pub fn sign(link: &SigningKey, joining: &SigningKey) -> Self {
        let mut req = Self {
            link: actor_of(link),
            key: actor_of(joining),
            sig: String::new(),
        };
        req.sig = sign_message(link, &req.message());
        req
    }

    /// `DOMAIN || link || key`, each preceded by its `u32` big-endian length.
    fn message(&self) -> Vec<u8> {
        let mut msg = ASK_DOMAIN.to_vec();
        field(&mut msg, self.link.as_bytes());
        field(&mut msg, self.key.as_bytes());
        msg
    }
}

/// The issuer's answer to an ask. The joiner believes `seed` only if it
/// hashes to `group`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asked {
    /// The id of the group the `Invite` is in.
    pub group: String,
    /// That group's seed, as [`SeedText`] read it.
    pub seed: String,
    /// The id of the issuer's `Invite` naming the link's key.
    pub invite: OpId,
}

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
    /// The name the joiner goes by in the group, as sent: the issuer parses
    /// it ([`Name::kept`]) before anything is written.
    pub person: String,
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
            person: person.as_str().to_string(),
            key: actor_of(joining),
            cosig: consent.sig,
            link: actor_of(link),
            sig: String::new(),
        };
        req.sig = sign_message(link, &req.message());
        req
    }

    /// The bytes the link's key signs: `DOMAIN || group || invite[32] ||
    /// person || key || cosig || link`, every variable field preceded by its
    /// `u32` big-endian length. Public so a test can sign a request naming
    /// a person no `Person` holds, as a hand-built client could.
    pub fn message(&self) -> Vec<u8> {
        let mut msg = REQUEST_DOMAIN.to_vec();
        field(&mut msg, self.group.as_bytes());
        field(&mut msg, self.invite.as_bytes());
        field(&mut msg, self.person.as_bytes());
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

/// What the routes read: this node's journals, and how it reads a seed.
#[derive(Clone)]
struct Issuer {
    rail: Arc<RingRail>,
    seeds: SeedText,
}

/// The ask and redemption routes, served on [`Alpn::Redeem`] to any dialer.
pub fn router(rail: Arc<RingRail>, seeds: SeedText) -> axum::Router {
    axum::Router::new()
        .route(ASK_ROUTE, post(ask_route))
        .route(ROUTE, post(redeem_route))
        .with_state(Issuer { rail, seeds })
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
    answer("redeem", &req.key, redeemed)
}

/// A decision as the wire carries it: what was done, [`UNJUDGED`], or the
/// one [`REFUSAL`].
fn answer<T: Serialize>(route: &str, key: &str, decided: Result<T, Refused>) -> Response {
    match decided {
        Ok(done) => (StatusCode::OK, Json(done)).into_response(),
        Err(Refused::Unjudged(e)) => {
            tracing::warn!(target: "redeem", route, error = %e,
                "redeem: could not judge — this node's own journal is unreadable");
            err(StatusCode::SERVICE_UNAVAILABLE, UNJUDGED)
        }
        Err(Refused::NotAName(why)) => {
            tracing::debug!(target: "redeem", route, key, why = ?why,
                "redeem: refused — the name is not one; answered with why, naming no group");
            err(StatusCode::FORBIDDEN, why.to_string())
        }
        Err(Refused::Decided(why)) => {
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

/// The ask, as a door: parse ([`parse_ask`]), then for each group held
/// here read its acts and decide ([`own_invite`]); the seed is read only for
/// the group the decision names, and answered as kept.
fn serve_ask(issuer: &Issuer, dialer: Option<String>, req: &AskRequest) -> Result<Asked, Refused> {
    parse_ask(dialer.as_deref(), req)?;
    let me = issuer.rail.signer().actor();
    for group in issuer.rail.namespaces().map_err(Refused::Unjudged)? {
        let journal = issuer.rail.journal(&group).map_err(Refused::Unjudged)?;
        let (ops, _) = journal.read().map_err(Refused::Unjudged)?;
        let Some(invite) = own_invite(&ops, &me, &req.link) else {
            continue;
        };
        let invite = derive_id(invite, &group);
        let seed = (issuer.seeds)(journal.dir())
            .map_err(|e| Refused::Unjudged(RailError::Io(e)))?
            .ok_or(Refused::NotHeld)?;
        tracing::debug!(target: "redeem", group, key = %req.key, invite = %invite,
            "redeem: asked — the link's key is named by this node's Invite; answering its group's seed");
        return Ok(Asked {
            group,
            seed,
            invite,
        });
    }
    Err(Refused::NoInvite)
}

/// An ask judged on what arrived alone: the dialer is the key asking, and
/// the link's key signed. Reads nothing, so a refused ask opens no journal.
fn parse_ask(dialer: Option<&str>, req: &AskRequest) -> Result<(), Refused> {
    let dialer = dialer.ok_or(Refused::NoDialer)?;
    if dialer != req.key {
        return Err(Refused::NotTheDialer);
    }
    if !verify_hex(&req.link, &req.message(), &req.sig) {
        return Err(Refused::LinkUnsigned);
    }
    Ok(())
}

/// The ask's decision over one group's acts: `me`'s `Invite` naming the
/// link's key — the newest, should two. Reads nothing.
fn own_invite<'a>(
    ops: &'a [crate::Op<crate::SignedOp>],
    me: &str,
    link: &str,
) -> Option<&'a crate::Op<crate::SignedOp>> {
    ops.iter().rev().find(|op| {
        op.actor == me && matches!(&op.kind.act, RailAct::Invite { key, .. } if key == link)
    })
}

/// The redemption, as an adapter: parse ([`parse_redeem`]); check this node
/// holds the group; hand [`svrngs_core::Redeem`] to [`commit`], which decides
/// it under the journal's write lock.
fn serve_redeem(
    rail: &RingRail,
    dialer: Option<String>,
    req: &RedeemRequest,
) -> Result<Redeemed, Refused> {
    let person = parse_redeem(dialer.as_deref(), req)?;
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
    let command = svrngs_core::Redeem {
        invite: req.invite.clone(),
        person,
        key: req.key.clone(),
        cosig: req.cosig.clone(),
        link: req.link.clone(),
        issuer: rail.signer().actor(),
    };
    let author = Author {
        signer: rail.signer(),
        roster: &seed,
        namespace: group,
        verifier: &Ed25519Verifier,
        now: crate::journal::now_unix(group).map_err(Refused::Unjudged)?,
        on_behalf_of: None,
    };
    match commit(&*journal, &command, &author).map_err(Refused::Unjudged)? {
        Ok(recorded) if recorded.is_empty() => {
            tracing::debug!(target: "redeem", group, key = %req.key,
                "redeem: the key stands already — answered success, nothing written");
            Ok(Redeemed::AlreadyIn)
        }
        Ok(recorded) => {
            let admit = recorded.lines()[0].id.clone();
            tracing::debug!(target: "redeem", group, key = %req.key, admit = %admit,
                "redeem: admitted — the Admit is written");
            Ok(Redeemed::Admitted(admit))
        }
        Err(CommitRefused::NoStanding) => Err(Refused::Unjudged(RailError::NotInRoster {
            actor: rail.signer().actor(),
            namespace: group.to_string(),
        })),
        Err(CommitRefused::Command(why)) => Err(Refused::Decided(why)),
    }
}

/// A redemption judged on what arrived alone: the dialer is the new key,
/// the link's key signed, and the name parses. Reads nothing, so a refused
/// redemption opens no journal.
fn parse_redeem(dialer: Option<&str>, req: &RedeemRequest) -> Result<Person, Refused> {
    let dialer = dialer.ok_or(Refused::NoDialer)?;
    if dialer != req.key {
        return Err(Refused::NotTheDialer);
    }
    if !req.signed_by_link() {
        return Err(Refused::LinkUnsigned);
    }
    Name::kept(req.person.clone())
        .map(Person::from)
        .map_err(Refused::NotAName)
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

/// Post `body` to `route` on the issuer's node, and read its answer.
async fn call<T: DeserializeOwned>(
    endpoint: &Endpoint,
    issuer: EndpointAddr,
    route: &str,
    body: &impl Serialize,
) -> Result<T, RedeemError> {
    let bridge = HttpBridge::spawn(endpoint.clone(), issuer, Alpn::Redeem.bytes())
        .await
        .map_err(|e| RedeemError::NoAnswer(format!("no bridge: {e}")))?;
    let http = peer_client().map_err(|e| RedeemError::NoAnswer(e.to_string()))?;
    let url = format!("http://{}{route}", bridge.local_addr());
    tracing::debug!(target: "redeem", url, "redeem: dialling the issuer by key");
    let response = http
        .post(&url)
        .json(body)
        .send()
        .await
        .map_err(|e| RedeemError::NoAnswer(e.to_string()))?;
    let status = response.status();
    tracing::debug!(target: "redeem", route, status = status.as_u16(), "redeem: the issuer answered");
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

#[cfg(test)]
mod tests;
