// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/rail.rs@e1a43bcd2
//! The ring rail's doors — the surface a ring app writes its own state to.
//!
//! `POST /v1/rail/append`, `GET /v1/rail/log`, `POST+GET /v1/rail/live`,
//! mounted beside the mesh routes in [`crate::api`], and — since fp-54's
//! serve half — the sync doors the ring round reads its journals through
//! (`digest`, `missing`, `ingest`, `roster`, `read`, `admit`, `compact`,
//! `namespaces`, `actor`), all rail-core-typed. The append and log
//! bodies mirror the inference daemon's `routes_rail` door for door — same
//! paths, same server-assigned fields, same retire rendering, same gap
//! sentences — so a page written against one daemon behaves the same against
//! the other (ARCH §10.6). What is deliberately NOT mirrored is the guest
//! half: there are no grants and no sessions here (loopback IS the auth),
//! so the namespace is always named explicitly.
//! (svrngs: append and log came over, DESIGN §8, U3's inventory; the live
//! lane is the next unit's, and the sync doors and `attest` stay behind —
//! one process per computer reads its own journals.)
//!
//! **The journals live under THIS process's data root**, never the daemon's
//! (§4 rule 1 — one data directory, one owner; a second process never opens
//! the daemon's dir, and the daemon's journals migrate in fp-54's commit, so
//! there is no dual-writer window). The signer is THIS process's node key.
//! (svrngs: upstream's census of rails' and the daemon's two keys went with
//! the mesh; a node here has one key, DESIGN §8, "One key signs and dials".)

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

// rail.rs's refusal body came over with the sync route (u1-3), so there is one.
use crate::ring_routes::err;
use crate::{admit, Compaction, Ed25519Verifier, RailAct, RailError, RingJournal, RingRail};

/// What the doors read: the journals. (Upstream's handlers read them from
/// `RailsDaemon`, which did not come over, DESIGN §8.)
#[derive(Clone)]
pub struct RailApi {
    pub rail: Arc<RingRail>,
}

// ── The doors ────────────────────────────────────────────────

/// Query parameters common to every rail route here. There are no grants on
/// a rails daemon, so the namespace is always the operator's explicit one.
#[derive(Debug, Deserialize)]
pub struct RailQuery {
    #[serde(default)]
    pub namespace: Option<String>,
}

/// Render what [`RingJournal::seal`]'s prune did, for the append body — the
/// daemon's rendering verbatim: a seal and the prune it authorises are one
/// act to the caller, and a refused prune is not a failed seal, so the
/// outcome is reported in the body instead of a status code.
fn retire(retired: &Result<Compaction, RailError>) -> serde_json::Value {
    match retired {
        Ok(done) => serde_json::json!({
            "removed": done.removed,
            "kept": done.kept,
            "gaps_cleared": done.gaps_cleared,
            "floors": done.floors,
        }),
        Err(e) => serde_json::json!({ "refused": e.to_string() }),
    }
}

fn namespace_of(q: &RailQuery) -> Result<String, Response> {
    q.namespace.clone().ok_or_else(|| {
        err(
            StatusCode::BAD_REQUEST,
            "the namespace must be named explicitly — pass ?namespace=<id>",
        )
    })
}

/// A namespace that is not one is refused with `kind: "bad_namespace"`
/// beside the sentence, so a dialing client hands its caller the same typed
/// refusal a local rail would (`RailError::BadNamespace`), never prose to
/// match (ARCH principle 9).
fn journal_of(rail: &RingRail, namespace: &str) -> Result<Arc<RingJournal>, Response> {
    rail.journal(namespace).map_err(|e| match e {
        RailError::BadNamespace(ns) => {
            tracing::debug!(target: "rails", namespace = %ns, "rail: refused a namespace that is not one");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": RailError::BadNamespace(ns.clone()).to_string(),
                    "kind": "bad_namespace",
                    "namespace": ns,
                })),
            )
                .into_response()
        }
        e => err(StatusCode::BAD_REQUEST, e.to_string()),
    })
}

/// POST /v1/rail/append — sign and append one act to the named namespace.
///
/// The body is the act alone. `seq`, the signature, the timestamp and the id
/// are all this daemon's to assign; an app that could choose its own sequence
/// number or actor could write as somebody else.
///
/// There are no sessions on a rails daemon, so a caller cannot stamp by
/// saying so: a bare `on_behalf_of` is dropped HERE, before anything is
/// signed, and warned — the same drop the daemon's `stamp_from` makes for
/// its session-less callers. (svrngs: upstream honoured instead a signed
/// `GuestAttestation`, which its `attest` door minted for svrn's guest
/// door; `attest` stayed behind, DESIGN §8, U3's inventory, so the branch
/// went with it.)
pub(crate) async fn append_act(
    rail: &RingRail,
    journal: &Arc<RingJournal>,
    body: serde_json::Value,
) -> Response {
    if let Some(claimed) = body.get("on_behalf_of").and_then(|v| v.as_str()) {
        tracing::warn!(
            target: "rails",
            namespace = journal.namespace(),
            claimed,
            "rail: dropped an on_behalf_of — this door signs as the node"
        );
    }
    // Taken as a `Value` and converted here rather than as `Json<RailAct>`,
    // so a refusal is the rail's own sentence instead of axum's rejection
    // prose (ARCH §10.6).
    let act = match RailAct::from_json(body) {
        Ok(act) => act,
        Err(e) => return err(StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
    };
    // Through the rail's ONE roster reader — never the file directly.
    // (svrngs: upstream's `RingRail::roster` went in U1 with the derived
    // rosters; the file it read is the group's seed.)
    let roster = match journal.roster_file() {
        Ok(r) => r,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    let sealed = matches!(act, RailAct::Seal);
    let appended = if sealed {
        // A seal takes no stamp: it is delivery, not words.
        journal
            .seal(rail.signer(), &roster, &Ed25519Verifier)
            .map(|done| (done.op, Some(retire(&done.retired))))
    } else {
        journal
            .append(act, rail.signer(), &roster, None, &Ed25519Verifier)
            .map(|op| (op, None))
    };
    match appended {
        Ok((op, retired)) => {
            // The whole op rides beside the flat fields, so a dialing client
            // that speaks the rail's TYPES (the mesh round's port, since the
            // journals moved here) reads back an `Op<SignedOp>` instead of
            // re-deriving one from the render (ARCH §10.6). The flat fields
            // stay: they are what a page renders, and the daemon's append
            // door answers with exactly them.
            let mut out = serde_json::json!({
                "id": op.id,
                "seq": op.kind.seq,
                "actor": op.actor,
                "ts_unix": op.ts_unix,
                "namespace": journal.namespace(),
                "op": op,
            });
            if let Some(retired) = retired {
                out["retired"] = retired;
            }
            Json(out).into_response()
        }
        Err(e @ RailError::NotInRoster { .. }) => {
            // `kind` carries the refusal TYPED, not only as prose: the KV
            // pump's defer-on-not-in-roster is a real decision that must
            // survive the dial, and a client matching on the sentence would
            // be prose matching (ARCH principle 9). `actor` names whose key
            // was refused. (svrngs: upstream chose the sentence by the
            // roster's origin; with the derived origin gone, the file's
            // sentence, the rail's own, is the one.)
            let (actor, namespace) = match &e {
                RailError::NotInRoster { actor, namespace } => (actor.clone(), namespace.clone()),
                _ => (String::new(), String::new()),
            };
            let refusal = serde_json::json!({
                "error": e.to_string(),
                "kind": "not_in_roster",
                "actor": actor,
                "namespace": namespace,
            });
            (StatusCode::UNPROCESSABLE_ENTITY, Json(refusal)).into_response()
        }
        Err(RailError::Rejected(why)) => err(StatusCode::UNPROCESSABLE_ENTITY, why),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// GET /v1/rail/log — the namespace's acts, in one order, and its gaps:
/// admitted ops, each gap's own sentence (the rail's `Display`, so the page
/// and the terminal say the same words), the held count, completeness, and
/// the roster the ops were admitted against.
async fn log_answer(_rail: &RingRail, journal: &Arc<RingJournal>) -> Response {
    let roster = match journal.roster_file() {
        Ok(r) => r,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    // ONE read, admitted from those exact bytes — reading the journal twice
    // would let a write land between and ship an answer that does not match
    // the ops beside it.
    let (ops, skipped) = match journal.read() {
        Ok(pair) => pair,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    let admission = admit(
        &ops,
        &skipped,
        &roster,
        journal.namespace(),
        &Ed25519Verifier,
    );
    let gaps: Vec<serde_json::Value> = admission
        .gaps
        .iter()
        .map(|g| {
            let mut v = serde_json::to_value(g).unwrap_or_default();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("message".into(), serde_json::Value::String(g.to_string()));
            }
            v
        })
        .collect();
    let ops_json: Vec<serde_json::Value> = admission
        .ops
        .iter()
        .map(|op| serde_json::to_value(op).unwrap_or_default())
        .collect();
    Json(serde_json::json!({
        "namespace": journal.namespace(),
        "ops": ops_json,
        "gaps": gaps,
        "held": admission.held,
        "complete": admission.is_complete(),
        "roster": roster,
    }))
    .into_response()
}

/// POST /v1/rail/append.
pub async fn append(
    State(api): State<RailApi>,
    Query(q): Query<RailQuery>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let namespace = match namespace_of(&q) {
        Ok(ns) => ns,
        Err(refusal) => return refusal,
    };
    let journal = match journal_of(&api.rail, &namespace) {
        Ok(j) => j,
        Err(refusal) => return refusal,
    };
    append_act(&api.rail, &journal, body).await
}

/// GET /v1/rail/log.
pub async fn log(State(api): State<RailApi>, Query(q): Query<RailQuery>) -> Response {
    let namespace = match namespace_of(&q) {
        Ok(ns) => ns,
        Err(refusal) => return refusal,
    };
    let journal = match journal_of(&api.rail, &namespace) {
        Ok(j) => j,
        Err(refusal) => return refusal,
    };
    log_answer(&api.rail, &journal).await
}

#[cfg(test)]
mod tests;
