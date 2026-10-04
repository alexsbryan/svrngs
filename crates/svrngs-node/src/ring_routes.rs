// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_routes.rs@e1a43bcd2
// `err`: ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/rail.rs@e1a43bcd2
//! The ring's peer routes on `cwth/http/0` (phase-b pb-rails-parity):
//! `POST /internal/ring/sync`, `POST /internal/ring/live` and
//! `GET /internal/ring/checkpoint/{ns}` — what the daemon's
//! `routes_internal::{ring_sync, ring_live, ring_checkpoint}` answer today, so
//! the flip turns nothing off. (svrngs: the sync route alone came over; the
//! live route is U3's and the checkpoint route waits, DESIGN §8 and §9.)
//!
//! Upstream mounted them on the internal listener beside gossip and join,
//! under the standing `/internal/ring` prefix the endpoint registers for
//! MEMBERS (`origins::stand_own`): a dialer the roster does not name never
//! reached them. In svrngs [`router`] is an `axum::Router` its caller serves.
//! What a member may then read is each ring's own question —
//! the sync route asks the ring's roster about the verified key the acceptor
//! stamped (`X-Mesh-Pubkey`), through [`crate::ring_sync::roster_names`], the
//! one test the sender also applies. A caller with no stamp reached this
//! loopback port without the acceptor in front, so it is a process on this
//! machine, and is served as the daemon serves one.

use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Json;

use crate::iroh::PublicKey;
use crate::peer_wire::{RingSyncRequest, MAX_REQUEST_BODY_BYTES, RING_SYNC_OPS_BUDGET_BYTES};
use crate::ring_sync::{answer, roster_names, RingSyncJournal};
use crate::RingRail;

// rail.rs's refusal body, forked with the route (DESIGN §8, U1's inventory).
pub(crate) fn err(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "error": msg.into() }))).into_response()
}

/// The header the acceptor stamps with the dialer's verified key
/// (`kernel_types::member::verified_headers`); a client-typed one is stripped
/// before the forward.
const PUBKEY_HEADER: &str = "x-mesh-pubkey";

/// What the ring route reads: the journals. (Upstream's also held the live
/// lane's buffer and the origin registry, which only the live route read.)
#[derive(Clone)]
pub struct RingInbound {
    pub rail: Arc<RingRail>,
}

/// The sync route, as a router (upstream's was a `RouteBundle` for the
/// internal listener). It takes the peer-wire body limit, because its
/// exchange budget is half of it.
pub fn router(state: RingInbound) -> axum::Router {
    axum::Router::new()
        .route(
            "/internal/ring/sync",
            post(ring_sync).layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES)),
        )
        .with_state(state)
}

/// Why this asker may not have this namespace, or `None` to serve it.
async fn roster_refusal(rail: &RingRail, headers: &HeaderMap, namespace: &str) -> Option<String> {
    let Some(stamped) = headers.get(PUBKEY_HEADER) else {
        tracing::debug!(
            target: "rails",
            namespace,
            "ring sync: no verified key stamped — a process on this machine, served"
        );
        return None;
    };
    let key = match stamped
        .to_str()
        .ok()
        .and_then(|s| hex::decode(s.trim()).ok())
        .and_then(|b| <[u8; 32]>::try_from(b).ok())
        .and_then(|bytes| PublicKey::from_bytes(&bytes).ok())
    {
        Some(key) => key,
        None => {
            tracing::warn!(target: "rails", namespace, "ring sync: refused — the stamped key is not one");
            return Some(format!(
                "{namespace} is served to its roster, and the asker's key could not be read"
            ));
        }
    };
    let roster = match RingSyncJournal::roster(rail, namespace).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(target: "rails", namespace, error = %e,
                "ring sync: refused — this namespace's roster is unreadable, so nobody can be shown to be on it");
            return Some(format!("{namespace}'s roster is unreadable on this node"));
        }
    };
    if roster_names(&roster, Some(key)) {
        tracing::debug!(target: "rails", namespace, asker = %key, "ring sync: the roster names the asker");
        return None;
    }
    tracing::warn!(target: "rails", namespace, asker = %key,
        "ring sync: refused — this ring's roster does not name the asker");
    Some(format!("{key} is not on {namespace}'s roster"))
}

/// `POST /internal/ring/sync` — one anti-entropy exchange for one namespace.
/// The body is raw bytes so the gauge reads the direction with a limit.
pub async fn ring_sync(
    State(state): State<RingInbound>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let request_bytes = body.len();
    let req: RingSyncRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return err(StatusCode::BAD_REQUEST, format!("malformed body: {e}")),
    };
    if let Some(refusal) = roster_refusal(&state.rail, &headers, &req.namespace).await {
        return err(StatusCode::FORBIDDEN, refusal);
    }
    let (response, more_for_caller) = match answer(state.rail.as_ref(), &req).await {
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

#[cfg(test)]
mod tests;
