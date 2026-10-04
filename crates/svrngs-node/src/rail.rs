// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/rail.rs@e1a43bcd2
//! The ring rail's doors — the surface a ring app writes its own state to.
//!
//! `POST /v1/rail/append`, `GET /v1/rail/log`, `POST+GET /v1/rail/live`,
//! (svrngs: and `GET /v1/rail/people`, the register's answer)
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
//! (svrngs: append, log and the live lane came over, DESIGN §8, U3's
//! inventory; the sync doors and `attest` stay behind — one process per
//! computer reads its own journals.)
//!
//! **The journals live under THIS process's data root**, never the daemon's
//! (§4 rule 1 — one data directory, one owner; a second process never opens
//! the daemon's dir, and the daemon's journals migrate in fp-54's commit, so
//! there is no dual-writer window). The signer is THIS process's node key.
//! (svrngs: upstream's census of rails' and the daemon's two keys went with
//! the mesh; a node here has one key, DESIGN §8, "One key signs and dials".)
//!
//! **The live lane is the drain half only.** The daemon's `live_push` fans
//! each payload out to every online peer over sovereign-mesh's fabric,
//! which this daemon deliberately does not carry (package-closure clean is
//! its own Cargo description). The buffer, the caps and the answer shape
//! are the daemon's (`peers: []`, `delivered: 0` — the fan-out's absence is
//! reported in the body, not papered over); peer delivery arrives with
//! fp-54's dial.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

// rail.rs's refusal body came over with the sync route (u1-3), so there is one.
use crate::register::{standing_in, Standing};
use crate::ring_routes::err;
use crate::{
    admit, Compaction, Ed25519Verifier, Person, RailAct, RailError, RingJournal, RingRail,
};

/// What the doors read: the journals, the nudge that wakes the round
/// after an append ([`crate::ring_sync::append`]), and the live lane's
/// buffer, which the live route fills ([`crate::ring_routes::ring_live`]).
/// (Upstream's handlers read them from `RailsDaemon`, which did not come
/// over, DESIGN §8.)
#[derive(Clone)]
pub struct RailApi {
    pub rail: Arc<RingRail>,
    pub nudge: Arc<tokio::sync::Notify>,
    pub live: Arc<LiveBuffer>,
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
///
/// (svrngs: then the register is asked, before the journal is opened, so a
/// group this computer does not hold leaves no trace: it is refused
/// `kind: "not_held"`, and a group held with no seed `kind: "no_seed"`,
/// since nobody can be shown to stand in it — DESIGN §8, U3's inventory,
/// "the roster becomes the register". Upstream opened any namespace that
/// was a name.)
fn journal_of(rail: &RingRail, namespace: &str) -> Result<Arc<RingJournal>, Response> {
    standing_of(rail, namespace)?;
    rail.journal(namespace)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// The people standing in `namespace`, or the refusal [`journal_of`] makes
/// of a namespace that is not one, a group not held, or one with no seed.
fn standing_of(rail: &RingRail, namespace: &str) -> Result<BTreeMap<String, Person>, Response> {
    if !crate::valid_namespace(namespace) {
        tracing::debug!(target: "rails", namespace, "rail: refused a namespace that is not one");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": RailError::BadNamespace(namespace.to_string()).to_string(),
                "kind": "bad_namespace",
                "namespace": namespace,
            })),
        )
            .into_response());
    }
    let refused = |status: StatusCode, kind: &str, why: String| {
        (
            status,
            Json(serde_json::json!({ "error": why, "kind": kind, "namespace": namespace })),
        )
            .into_response()
    };
    match rail.standing(namespace, &Ed25519Verifier) {
        Ok(Standing::Members(people)) => {
            tracing::debug!(target: "rails", namespace, people = people.len(), "rail: the group is held here, and its register read");
            Ok(people)
        }
        Ok(Standing::NotHeld) => {
            tracing::debug!(target: "rails", namespace, "rail: refused — this computer holds no such group");
            Err(refused(
                StatusCode::NOT_FOUND,
                "not_held",
                format!("this computer is in no group {namespace}"),
            ))
        }
        Ok(Standing::NoSeed) => {
            tracing::warn!(target: "rails", namespace, "rail: refused — the group is held with no seed, so nobody can be shown to be in it");
            Err(refused(
                StatusCode::CONFLICT,
                "no_seed",
                format!("nobody can be shown to be in {namespace} on this computer"),
            ))
        }
        Err(e) => {
            tracing::warn!(target: "rails", namespace, error = %e, "rail: refused — the group's register is unreadable");
            Err(err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// The people standing in a group as an app reads them: each person, with
/// the key they stand by, its lowercase hex — a key an app may seal to
/// (DESIGN §3). The one shape every door that names the people answers.
pub(crate) fn people_json(standing: &BTreeMap<String, Person>) -> serde_json::Value {
    standing
        .iter()
        .map(|(key, person)| serde_json::json!({ "person": person, "key": key }))
        .collect()
}

/// POST /v1/rail/append — sign and append one act to the named namespace.
///
/// The body is the act alone. `seq`, the signature, the timestamp and the id
/// are all this daemon's to assign; an app that could choose its own sequence
/// number or actor could write as somebody else.
///
/// (svrngs: upstream dropped a bare `on_behalf_of`, since a caller could
/// stamp only with a signed `GuestAttestation`, which went with `attest`.
/// Here an app serving someone else names them, and the act is signed as
/// this computer with `on_behalf_of` naming that person — whom the member
/// door verified (DESIGN §3, §8 "Localhost is the trust boundary"). The
/// name must parse as a person and be a person standing in the group;
/// anything else is refused and nothing is written, never dropped.)
pub(crate) async fn append_act(
    rail: &RingRail,
    journal: &Arc<RingJournal>,
    body: serde_json::Value,
) -> Response {
    let stamp = match on_behalf_of(rail, journal.namespace(), &body) {
        Ok(stamp) => stamp,
        Err(refusal) => return refusal,
    };
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
        if let Some(person) = &stamp {
            tracing::debug!(target: "rails", namespace = journal.namespace(), %person, "rail: a seal is signed as this computer alone, whoever was named");
        }
        journal
            .seal(rail.signer(), &roster, &Ed25519Verifier)
            .map(|done| (done.op, Some(retire(&done.retired))))
    } else {
        journal
            .append(
                act,
                rail.signer(),
                &roster,
                stamp.as_ref().map(Person::as_str),
                &Ed25519Verifier,
            )
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
            // roster's origin; here the register decides, and the sentence
            // an app may show a person says so in DESIGN §1's words.)
            let (actor, namespace) = match &e {
                RailError::NotInRoster { actor, namespace } => (actor.clone(), namespace.clone()),
                _ => (String::new(), String::new()),
            };
            tracing::debug!(target: "rails", namespace = %namespace, actor = %actor, "rail: refused — this computer's key does not stand in the group");
            let refusal = serde_json::json!({
                "error": format!(
                    "this computer is not in {namespace}, so nothing it writes would count; \
                     someone in it must let it in"
                ),
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

/// The person an append is on behalf of: `None` when the body names
/// nobody, so the act is this computer's own; else the person it names,
/// parsed ([`Person::parse`]) and standing in `namespace` now, as the
/// register says ([`RingRail::standing`]). A name that is not one, or names
/// nobody in the group, is refused with a reason, typed. The register is
/// read before the door's write lock is taken, so a person removed between
/// the two is still named; their removal is in the record either way.
fn on_behalf_of(
    rail: &RingRail,
    namespace: &str,
    body: &serde_json::Value,
) -> Result<Option<Person>, Response> {
    let refused = |kind: &str, why: String| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": why, "kind": kind, "namespace": namespace })),
        )
            .into_response()
    };
    let raw = match body.get("on_behalf_of") {
        None | Some(serde_json::Value::Null) => {
            tracing::debug!(target: "rails", namespace, "rail: the act is this computer's own — it names nobody");
            return Ok(None);
        }
        Some(serde_json::Value::String(raw)) => raw,
        Some(other) => {
            tracing::debug!(target: "rails", namespace, named = %other, "rail: refused — on_behalf_of is not text");
            return Err(refused(
                "not_a_name",
                "on_behalf_of names a person, as text".into(),
            ));
        }
    };
    let person = Person::parse(raw).map_err(|why| {
        tracing::debug!(target: "rails", namespace, raw, why = %why, "rail: refused — on_behalf_of is not a name");
        refused("not_a_name", format!("on_behalf_of is not a name: {why}"))
    })?;
    match rail.standing(namespace, &Ed25519Verifier) {
        Ok(Standing::Members(people)) if people.values().any(|p| *p == person) => {
            tracing::debug!(target: "rails", namespace, %person, "rail: the act is on behalf of a person in the group");
            Ok(Some(person))
        }
        Ok(_) => {
            tracing::debug!(target: "rails", namespace, %person, "rail: refused — on_behalf_of names nobody in the group");
            Err(refused(
                "not_in_group",
                format!("{person} is not in {namespace}"),
            ))
        }
        Err(e) => {
            tracing::warn!(target: "rails", namespace, error = %e, "rail: refused — the group's register is unreadable");
            Err(err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// GET /v1/rail/log — the namespace's acts, in one order, and its gaps:
/// admitted ops, each gap's own sentence (the rail's `Display`, so the page
/// and the terminal say the same words), the held count, completeness, and
/// the roster the ops were admitted against. (svrngs: the people standing
/// after those ops, from the register over the same read, in place of the
/// seed they were admitted from.)
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
    let people = match standing_in(&ops, &roster, journal.namespace(), &Ed25519Verifier) {
        Ok(p) => p,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
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
        "people": people_json(&people),
    }))
    .into_response()
}

/// GET /v1/rail/people — the people standing in the group, each with the
/// key they stand by, from the register (svrngs, DESIGN §3: an app needs
/// no second source for who is in its group, nor for a key to seal to).
/// New: upstream's apps read the mesh's roster door instead.
fn people_answer(rail: &RingRail, namespace: &str) -> Response {
    match standing_of(rail, namespace) {
        Ok(people) => Json(serde_json::json!({
            "namespace": namespace,
            "people": people_json(&people),
        }))
        .into_response(),
        Err(refusal) => refusal,
    }
}

/// GET /v1/rail/people.
pub async fn people(State(api): State<RailApi>, Query(q): Query<RailQuery>) -> Response {
    let namespace = match namespace_of(&q) {
        Ok(ns) => ns,
        Err(refusal) => return refusal,
    };
    people_answer(&api.rail, &namespace)
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

// ── The live lane ────────────────────────────────────────────

/// How many payloads ONE namespace's buffer holds before the oldest is
/// evicted — the daemon's cap. Bounded because this is memory an untrusted
/// local process can fill.
const LIVE_BUFFER_CAPACITY: usize = 256;

/// The largest live payload, in wire bytes — the daemon's cap, and ONE
/// number on this daemon too: read by the push door that accepts and by no
/// one else, so a payload this daemon would accept is exactly the one it
/// keeps.
pub(crate) const LIVE_PAYLOAD_MAX_BYTES: usize = 4096;

/// The arrived-payload buffer: bounded, in memory, and the only place a
/// live payload ever sits. Keyed by namespace, the way append and log are.
#[derive(Debug, Default)]
pub struct LiveBuffer {
    inner: Mutex<HashMap<String, LiveBufferInner>>,
}

#[derive(Debug, Default)]
struct LiveBufferInner {
    payloads: VecDeque<String>,
    /// Payloads evicted because the buffer was full, since the last drain.
    /// Eviction is a real loss and it is REPORTED, never silent.
    dropped: usize,
}

impl LiveBuffer {
    /// Take one arrived payload for `namespace`. Evicts that namespace's
    /// oldest when full.
    pub fn push(&self, namespace: &str, payload: String) {
        let mut buffers = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let inner = buffers.entry(namespace.to_string()).or_default();
        if inner.payloads.len() == LIVE_BUFFER_CAPACITY {
            inner.payloads.pop_front();
            inner.dropped += 1;
            tracing::warn!(
                target: "rails",
                namespace,
                capacity = LIVE_BUFFER_CAPACITY,
                dropped = inner.dropped,
                "rail live: buffer full, evicted the oldest payload — nobody \
                 is draining GET /v1/rail/live fast enough"
            );
        }
        inner.payloads.push_back(payload);
    }

    /// Empty `namespace`'s buffer, returning what was in it and how many
    /// were lost. Every other namespace's buffer is untouched.
    pub(crate) fn drain(&self, namespace: &str) -> (Vec<String>, usize) {
        let mut buffers = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match buffers.remove(namespace) {
            Some(inner) => (inner.payloads.into(), inner.dropped),
            None => (Vec::new(), 0),
        }
    }
}

/// POST /v1/rail/live — take one ephemeral payload for the named namespace.
///
/// Nothing is written anywhere, and nothing is fanned out: the peer round
/// runs over sovereign-mesh's fabric, which this daemon does not carry, so
/// the answer reports the absence honestly — `peers: []`, `delivered: 0` —
/// rather than a success that would read as "everyone saw it".
fn push_answer(live: &LiveBuffer, namespace: &str, body: axum::body::Bytes) -> Response {
    if body.len() > LIVE_PAYLOAD_MAX_BYTES {
        tracing::debug!(
            target: "rails",
            namespace,
            bytes = body.len(),
            "rail live: refused an oversize payload"
        );
        return err(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "live payload is {} bytes; the lane carries at most {} — \
                 send presence, not state",
                body.len(),
                LIVE_PAYLOAD_MAX_BYTES
            ),
        );
    }
    let payload = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!(target: "rails", namespace, error = %e, "rail live: refused a non-UTF-8 payload");
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!(
                    "live payload is not UTF-8 ({e}) — the lane carries text \
                     (base64 your bytes, as a rail act does)"
                ),
            );
        }
    };
    live.push(namespace, payload.to_string());
    tracing::debug!(
        target: "rails",
        namespace,
        bytes = body.len(),
        "rail live: buffered one payload (no peer fan-out on this daemon)"
    );
    Json(serde_json::json!({
        "bytes": body.len(),
        "peers": [],
        "delivered": 0,
    }))
    .into_response()
}

/// GET /v1/rail/live — drain the caller's namespace since the last call.
fn drain_answer(live: &LiveBuffer, namespace: &str) -> Response {
    let (payloads, dropped) = live.drain(namespace);
    if dropped > 0 {
        tracing::warn!(
            target: "rails",
            namespace,
            dropped,
            drained = payloads.len(),
            "rail live: drain reports evicted payloads — the presence map has \
             a hole in it"
        );
    }
    Json(serde_json::json!({
        "payloads": payloads,
        "dropped": dropped,
    }))
    .into_response()
}

/// POST /v1/rail/live.
pub async fn live_push(
    State(api): State<RailApi>,
    Query(q): Query<RailQuery>,
    body: axum::body::Bytes,
) -> Response {
    let namespace = match namespace_of(&q) {
        Ok(ns) => ns,
        Err(refusal) => return refusal,
    };
    push_answer(&api.live, &namespace, body)
}

/// GET /v1/rail/live.
pub async fn live_drain(State(api): State<RailApi>, Query(q): Query<RailQuery>) -> Response {
    let namespace = match namespace_of(&q) {
        Ok(ns) => ns,
        Err(refusal) => return refusal,
    };
    drain_answer(&api.live, &namespace)
}

#[cfg(test)]
mod tests;
