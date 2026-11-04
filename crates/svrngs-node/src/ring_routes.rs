// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_routes.rs@e1a43bcd2
// `err`: ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/rail.rs@e1a43bcd2
//! The ring's peer routes on `cwth/http/0` (phase-b pb-rails-parity):
//! `POST /internal/ring/sync`, `POST /internal/ring/live` and
//! `GET /internal/ring/checkpoint/{ns}` — what the daemon's
//! `routes_internal::{ring_sync, ring_live, ring_checkpoint}` answer today, so
//! the flip turns nothing off. (svrngs: the sync route came over in U1 and
//! the live route in U3; the checkpoint route waits, PLAN §4 and §2.)
//!
//! Upstream mounted them on the internal listener beside gossip and join,
//! under the standing `/internal/ring` prefix the endpoint registers for
//! MEMBERS (`origins::stand_own`): a dialer the roster does not name never
//! reached them. In svrngs [`router`] is an `axum::Router` its caller serves.
//! What a member may then read is each ring's own question —
//! the sync route (and, svrngs u3-3, the live route) asks the ring's
//! register whether the dialer's key stands in it ([`RingRail::standing`], the one answer the sender also asks), the
//! key the acceptor ([`crate::iroh::IrohAcceptor`]) puts on every request it
//! serves as a [`Dialer`] extension. No header names the asker. (svrngs:
//! upstream asked whether the ring's seed named the key,
//! `ring_sync::roster_names`; a ring this node does not hold is refused in
//! the same words, before anything is opened, so the answer never says which
//! rings a node holds — PLAN §4, U1.) A request
//! with no [`Dialer`] did not come on an iroh stream, and is refused: the
//! route is served only there. (Upstream read the key from an
//! `X-Mesh-Pubkey` header its acceptor stamped, and served a request with
//! none as a process on its machine; svrngs has no loopback listener for
//! such a process to reach, PLAN §4, U1's inventory.)

use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json};

use crate::iroh::Dialer;
use crate::peer_wire::{
    LiveEnvelope, RingSyncRequest, MAX_REQUEST_BODY_BYTES, RING_SYNC_OPS_BUDGET_BYTES,
};
use crate::rail::{Live, LiveBuffer, LIVE_PAYLOAD_MAX_BYTES};
use crate::register::{Member, Standing};
use crate::ring_sync::answer;
use crate::{ops_missing_from_within, Op, RailAct, RingRail, SignedOp};

// rail.rs's refusal body, forked with the route (PLAN §4, U1's inventory);
// each one traced, so a refusal its route does not trace still leaves one
// (#149).
pub(crate) fn err(status: StatusCode, msg: impl Into<String>) -> Response {
    let msg = msg.into();
    tracing::debug!(target: "rails", %status, msg, "refused");
    (status, Json(serde_json::json!({ "error": msg }))).into_response()
}

/// What the ring routes read: the journals and the live lane's buffer.
/// (Upstream's also held the origin registry, whose registrations named the
/// live namespaces; it did not come over.) And the round's nudge, which a
/// push landing its asker's own relay wakes (relay-in-record).
#[derive(Clone)]
pub struct RingInbound {
    pub rail: Arc<RingRail>,
    pub live: Arc<LiveBuffer>,
    pub wake: Arc<tokio::sync::Notify>,
}

/// The sync and live routes, as a router (upstream's was a `RouteBundle`
/// for the internal listener). The sync route takes the peer-wire body
/// limit, because its exchange budget is half of it.
pub fn router(state: RingInbound) -> axum::Router {
    axum::Router::new()
        .route(
            crate::ring_sync::SYNC_ROUTE,
            post(ring_sync).layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES)),
        )
        .route("/internal/ring/live", post(ring_live))
        .with_state(state)
}

/// The asker's key and its row of the register, to serve it; or why it may
/// not have this namespace. Asked by the sync route and (svrngs, u3-3) the
/// live route, named by `route` in the trace.
pub(crate) fn standing_asker(
    rail: &RingRail,
    dialer: Option<Dialer>,
    route: &'static str,
    namespace: &str,
) -> Result<(String, Member), String> {
    let Some(dialer) = dialer else {
        tracing::warn!(
            target: "rails",
            route,
            namespace,
            "ring: refused — no verified dialer on the request, so it did not \
             come on an iroh stream, and this route is served only there"
        );
        return Err(format!(
            "{namespace} is served to its roster over iroh, and this request did not come on an iroh stream"
        ));
    };
    let asker = dialer.actor();
    // One sentence for every refusal below, so the answer never says whether
    // this node holds the ring; the log says which it was.
    let refusal = format!("{namespace} is not served to {asker} here");
    let standing = rail.standing(namespace);
    if let Some(row) = standing.as_ref().ok().and_then(|s| s.member(&asker)) {
        tracing::debug!(target: "rails", route, namespace, asker, "ring: the asker stands in the ring");
        return Ok((asker, row.clone()));
    }
    match standing {
        Ok(Standing::NotHeld) => {
            tracing::debug!(target: "rails", route, namespace, asker,
                "ring: refused — this node holds no such ring, answered as a stranger is");
            Err(refusal)
        }
        Ok(_) => {
            tracing::debug!(target: "rails", route, namespace, asker,
                "ring: refused — the asker holds no standing in this ring");
            Err(refusal)
        }
        Err(e) => {
            tracing::warn!(target: "rails", route, namespace, asker, error = %e,
                "ring: refused — this ring's register is unreadable, so nobody can be shown to stand in it");
            Err(refusal)
        }
    }
}

/// What a cut is told with (#179, #211), the one selection both the at-once
/// tell (`removal.rs` `told`) and [`told_cut`] make: of `ops`, what a
/// computer lacks, `by`'s acts through its highest `Remove`, so a leave goes
/// with the signing key's own, which svrngs-core's `Leave` writes last, and
/// every `HandOn`, how the fold reads a keeper the seed did not name
/// (membership.rs `HandOn`, #184). One budget of them, by actor and seq, so a
/// cut never lands ahead of the acts it covers (DESIGN §6). `None` where it
/// lacks no `Remove` by `by`, which each caller traces.
pub fn told_with(mut ops: Vec<Op<SignedOp>>, by: &str, ns: &str) -> Option<Vec<Op<SignedOp>>> {
    let cut = |op: &&Op<SignedOp>| op.actor == by && matches!(op.kind.act, RailAct::Remove { .. });
    let seq = ops.iter().filter(cut).map(|op| op.kind.seq).max()?;
    let keeping = |act: &RailAct| matches!(act, RailAct::HandOn { .. });
    ops.retain(|op| op.actor == by && op.kind.seq <= seq || keeping(&op.kind.act));
    let (none, verify) = (crate::Digest::default(), &crate::Ed25519Verifier);
    let (ops, more) = ops_missing_from_within(&ops, ns, verify, &none, RING_SYNC_OPS_BUDGET_BYTES);
    tracing::debug!(target: "rails", namespace = ns, by, seq, told = ops.len(), more, "ring sync: a cut told with its signer's acts through its highest Remove and the hand-ons lacked, one budget of them");
    Some(ops)
}

/// To an asker that does not stand in the group, a `Remove` naming it which
/// its digest says it lacks, told with what it covers ([`told_with`]), and
/// nothing else; none of the asker's are taken. So a computer off as its
/// cut landed, by a leave or a removal, reads itself out once it asks,
/// never shown the group while every lender refuses it. The digest answered
/// is the asker's own, so it sends nothing back. `None` where it lacks no
/// `Remove` naming it, or what it lacks cannot be read here, said at warn.
fn told_cut(rail: &RingRail, req: &RingSyncRequest, asker: Option<&str>) -> Option<Response> {
    let (ns, digest, asker) = (req.namespace.clone(), req.digest.clone(), asker?);
    let lacks = rail.ops_missing_from_within(&ns, &digest, crate::NO_BUDGET);
    let (ops, _) = lacks
        .inspect_err(|e| tracing::warn!(target: "rails", namespace = ns, asker, error = %e, "ring sync: what an asker not standing lacks cannot be read here, so no cut is told it"))
        .ok()?;
    let cut = ops.iter().find(|op| op.kind.act.names() == Some(asker));
    let by = cut.map(|op| op.actor.clone())?;
    let ops = told_with(ops, &by, &ns)?;
    tracing::debug!(target: "rails", namespace = ns, asker, by, told = ops.len(), "ring sync: an asker not standing, told a Remove naming it with what it covers");
    let told = crate::peer_wire::RingSyncResponse {
        namespace: ns,
        digest,
        ops,
        ingested: 0,
    };
    Some(Json(told).into_response())
}

/// `POST /internal/ring/sync` — one anti-entropy exchange for one namespace.
/// The body is raw bytes so the gauge reads the direction with a limit.
pub async fn ring_sync(
    State(state): State<RingInbound>,
    dialer: Option<Extension<Dialer>>,
    body: axum::body::Bytes,
) -> Response {
    let request_bytes = body.len();
    let req: RingSyncRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return err(StatusCode::BAD_REQUEST, format!("malformed body: {e}")),
    };
    let dialer = dialer.map(|Extension(d)| d);
    let asker = dialer.map(|d| d.actor());
    if let Err(refusal) = standing_asker(&state.rail, dialer, "sync", &req.namespace) {
        if let Some(told) = told_cut(&state.rail, &req, asker.as_deref()) {
            return told;
        }
        return err(StatusCode::FORBIDDEN, refusal);
    }
    let (response, more_for_caller) = match answer(state.rail.as_ref(), &req) {
        Ok(pair) => pair,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    let bytes = match serde_json::to_vec(&response) {
        Ok(b) => b,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    tracing::debug!(
        target: "rails",
        namespace = %req.namespace,
        request_bytes,
        offered = req.ops.len(),
        ingested = response.ingested,
        sending = response.ops.len(),
        more_for_caller,
        payload_bytes = bytes.len(),
        "ring sync: exchange"
    );
    // A newcomer says where it is reached in its join's second exchange,
    // after the redemption's wake has told the others of it: wake the round
    // again, so they hear its relay now, not at their next interval
    // (demo/runs/relay-in-record/2026-10-12/, take 2: 57 s).
    let own_relay = |op: &&crate::Op<crate::SignedOp>| {
        Some(&op.actor) == asker.as_ref() && matches!(op.kind.act, crate::RailAct::Relay { .. })
    };
    if response.ingested > 0 && req.ops.iter().any(|op| own_relay(&op)) {
        tracing::debug!(target: "rails", namespace = %req.namespace, asker, "ring sync: the asker's own relay landed, so the round is woken");
        state.wake.notify_one();
    }
    if request_bytes >= RING_SYNC_OPS_BUDGET_BYTES {
        tracing::warn!(
            target: "rails",
            namespace = %req.namespace,
            request_bytes,
            budget_bytes = RING_SYNC_OPS_BUDGET_BYTES,
            limit_bytes = MAX_REQUEST_BODY_BYTES,
            "ring sync: a caller filled its whole exchange budget — this ring is \
             carrying more history than one exchange holds"
        );
    }
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        bytes,
    )
        .into_response()
}

/// `POST /internal/ring/live` — buffer one peer's ephemeral payload under the
/// namespace its envelope names. An oversize payload is refused against the
/// one cap the push door keeps; then a dialer not standing in that group,
/// or a group this node does not hold, is REFUSED as the sync route
/// refuses one (svrngs, u3-3; upstream refused a namespace no live
/// registration held, the daemon's rule).
pub async fn ring_live(
    State(state): State<RingInbound>,
    dialer: Option<Extension<Dialer>>,
    body: axum::body::Bytes,
) -> Response {
    let LiveEnvelope { namespace, payload } = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(target: "rails", error = %e, "ring live: refused a malformed envelope");
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("live envelope is not {{\"namespace\", \"payload\"}} JSON ({e})"),
            );
        }
    };
    if payload.len() > LIVE_PAYLOAD_MAX_BYTES {
        tracing::warn!(target: "rails", namespace, bytes = payload.len(),
            max = LIVE_PAYLOAD_MAX_BYTES, "ring live: refused an oversize payload");
        return err(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "live payload is {} bytes; this lane carries at most {}",
                payload.len(),
                LIVE_PAYLOAD_MAX_BYTES
            ),
        );
    }
    // svrngs: the dialer must stand in the group its envelope names, the
    // sync route's one question (`standing_asker`), where upstream asked
    // whether a live registration held the namespace and let any member
    // its acceptor admitted fill any ring's buffer. A group this node does
    // not hold is refused in the same words.
    // onair-live: and the key it dialled with is kept with the payload,
    // with its person's first key, so a drain says who spoke (ON_AIR §4.3).
    let dialer = dialer.map(|Extension(d)| d);
    let (key, member) = match standing_asker(&state.rail, dialer, "live", &namespace) {
        Ok(asker) => asker,
        Err(refusal) => return err(StatusCode::FORBIDDEN, refusal),
    };
    let bytes = payload.len();
    let live = Live {
        payload,
        key,
        person_key: member.person_key,
    };
    tracing::debug!(target: "rails", namespace, bytes, live.key, live.person_key, "ring live: buffered, with its sender");
    state.live.push(&namespace, live);
    Json(serde_json::json!({ "buffered": bytes })).into_response()
}

#[cfg(test)]
mod tests;
