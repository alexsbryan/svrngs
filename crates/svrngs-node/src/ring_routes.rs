// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_routes.rs@e1a43bcd2
// `err`: ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/rail.rs@e1a43bcd2
//! The ring's peer routes on `cwth/http/0` (phase-b pb-rails-parity):
//! `POST /internal/ring/sync`, `POST /internal/ring/live` and
//! `GET /internal/ring/checkpoint/{ns}` — what the daemon's
//! `routes_internal::{ring_sync, ring_live, ring_checkpoint}` answer today, so
//! the flip turns nothing off. (svrngs: the sync route came over in U1 and
//! the live route in U3; the checkpoint route waits, DESIGN §8 and §9.)
//!
//! Upstream mounted them on the internal listener beside gossip and join,
//! under the standing `/internal/ring` prefix the endpoint registers for
//! MEMBERS (`origins::stand_own`): a dialer the roster does not name never
//! reached them. In svrngs [`router`] is an `axum::Router` its caller serves.
//! What a member may then read is each ring's own question —
//! the sync route asks the ring's register whether the dialer's key stands
//! in it ([`RingRail::standing`], the one answer the sender also asks), the
//! key the acceptor ([`crate::iroh::IrohAcceptor`]) puts on every request it
//! serves as a [`Dialer`] extension. No header names the asker. (svrngs:
//! upstream asked whether the ring's seed named the key,
//! `ring_sync::roster_names`; a ring this node does not hold is refused in
//! the same words, before anything is opened, so the answer never says which
//! rings a node holds — DESIGN §8, U1.) A request
//! with no [`Dialer`] did not come on an iroh stream, and is refused: the
//! route is served only there. (Upstream read the key from an
//! `X-Mesh-Pubkey` header its acceptor stamped, and served a request with
//! none as a process on its machine; svrngs has no loopback listener for
//! such a process to reach, DESIGN §8, U1's inventory.)

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
use crate::rail::{LiveBuffer, LIVE_PAYLOAD_MAX_BYTES};
use crate::register::Standing;
use crate::ring_sync::answer;
use crate::{Ed25519Verifier, RingRail};

// rail.rs's refusal body, forked with the route (DESIGN §8, U1's inventory).
pub(crate) fn err(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "error": msg.into() }))).into_response()
}

/// What the ring routes read: the journals and the live lane's buffer.
/// (Upstream's also held the origin registry, whose registrations named the
/// live namespaces; it did not come over.)
#[derive(Clone)]
pub struct RingInbound {
    pub rail: Arc<RingRail>,
    pub live: Arc<LiveBuffer>,
}

/// The sync and live routes, as a router (upstream's was a `RouteBundle`
/// for the internal listener). The sync route takes the peer-wire body
/// limit, because its exchange budget is half of it.
pub fn router(state: RingInbound) -> axum::Router {
    axum::Router::new()
        .route(
            "/internal/ring/sync",
            post(ring_sync).layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES)),
        )
        .route("/internal/ring/live", post(ring_live))
        .with_state(state)
}

/// Why this asker may not have this namespace, or `None` to serve it.
fn standing_refusal(rail: &RingRail, dialer: Option<Dialer>, namespace: &str) -> Option<String> {
    let Some(dialer) = dialer else {
        tracing::warn!(
            target: "rails",
            namespace,
            "ring sync: refused — no verified dialer on the request, so it did not \
             come on an iroh stream, and this route is served only there"
        );
        return Some(format!(
            "{namespace} is served to its roster over iroh, and this request did not come on an iroh stream"
        ));
    };
    let asker = dialer.actor();
    // One sentence for every refusal below, so the answer never says whether
    // this node holds the ring; the log says which it was.
    let refusal = format!("{namespace} is not served to {asker} here");
    match rail.standing(namespace, &Ed25519Verifier) {
        Ok(standing) if standing.names(&asker) => {
            tracing::debug!(target: "rails", namespace, asker, "ring sync: the asker stands in the ring");
            None
        }
        Ok(Standing::NotHeld) => {
            tracing::debug!(target: "rails", namespace, asker,
                "ring sync: refused — this node holds no such ring, answered as a stranger is");
            Some(refusal)
        }
        Ok(_) => {
            tracing::debug!(target: "rails", namespace, asker,
                "ring sync: refused — the asker holds no standing in this ring");
            Some(refusal)
        }
        Err(e) => {
            tracing::warn!(target: "rails", namespace, asker, error = %e,
                "ring sync: refused — this ring's register is unreadable, so nobody can be shown to stand in it");
            Some(refusal)
        }
    }
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
    if let Some(refusal) = standing_refusal(&state.rail, dialer, &req.namespace) {
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
/// namespace its envelope names. A namespace no live registration on this
/// endpoint holds is REFUSED: nobody here could drain it (the daemon's rule,
/// with a registration where the daemon reads a grant). An oversize payload
/// is refused against the one cap the push door keeps.
pub async fn ring_live(State(state): State<RingInbound>, body: axum::body::Bytes) -> Response {
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
    // svrngs: a namespace the rail holds, where upstream asked the origin
    // registry's live registrations, which did not come over: any app on
    // this computer may drain a group the node holds.
    let held = match state.rail.namespaces() {
        Ok(names) => names,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    if !held.iter().any(|n| n == &namespace) {
        tracing::warn!(target: "rails", namespace,
            "ring live: refused a payload for a namespace no live registration here holds");
        return err(
            StatusCode::FORBIDDEN,
            format!(
                "no live registration on this endpoint holds namespace '{namespace}' — \
                 nobody here could drain it, so it is not buffered"
            ),
        );
    }
    let bytes = payload.len();
    state.live.push(&namespace, payload);
    tracing::debug!(target: "rails", namespace, bytes, "ring live: buffered");
    Json(serde_json::json!({ "buffered": bytes })).into_response()
}

#[cfg(test)]
mod tests;
