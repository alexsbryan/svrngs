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
//! fp-54's dial. (svrngs: it arrives here: the push door fans each payload
//! out over iroh to the keys standing in the group but this node's
//! ([`crate::live::push_ephemeral`]), and the answer names each and whether
//! it took it. Nothing is buffered on the pushing node; its own apps hear
//! the others, not themselves, as with the daemon's lane.)

use std::collections::{BTreeMap, HashMap, VecDeque};
#[expect(
    clippy::disallowed_types,
    reason = "the live lane's buffer, this computer's only"
)]
use std::sync::{Arc, Mutex};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

// rail.rs's refusal body came over with the sync route (u1-3), so there is one.
use crate::iroh::{Contact, IrohTransport};
use crate::live::{push_ephemeral, LivePeer};
use crate::node::dialled;
use crate::register::{Member, Standing};
use crate::ring_routes::err;
use crate::{Compaction, Membership, Person, RailAct, RailError, RingRail};
use svrngs_core::{Append, AppendRefusal, Refused as CommitRefused};

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
    /// The node's dial-by-key transport, the live lane's fan-out rides it.
    pub transport: Arc<IrohTransport>,
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

pub(crate) fn namespace_of(q: &RailQuery) -> Result<String, Response> {
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
fn group_of(rail: &RingRail, namespace: &str) -> Result<(), Response> {
    standing_of(rail, namespace).map(|_| ())
}

/// The people standing in `namespace`, or the refusal [`group_of`] makes
/// of a namespace that is not one, a group not held, or one with no seed.
pub(crate) fn standing_of(
    rail: &RingRail,
    namespace: &str,
) -> Result<BTreeMap<String, Member>, Response> {
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
    match rail.standing(namespace) {
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

/// The people standing in a group as an app reads them: one row per key,
/// each with its person, the key's lowercase hex — a key an app may seal
/// to (DESIGN §3) — and the person's first key, which every key of theirs
/// shares and the door tells a lend as `x-svrngs-key` (bash-devices). The
/// one shape every door that names the people answers.
pub(crate) fn people_json<'a>(
    standing: impl IntoIterator<Item = (&'a String, &'a Person, &'a str)>,
) -> serde_json::Value {
    (standing.into_iter())
        .map(|(key, person, first)| serde_json::json!({ "person": person, "key": key, "person_key": first }))
        .collect()
}

/// POST /v1/rail/append — sign and append one act to the named namespace.
///
/// The body is the act alone. `seq`, the signature, the timestamp and the id
/// are all this daemon's to assign; an app that could choose its own sequence
/// number or actor could write as somebody else.
///
/// (svrngs: upstream dropped a bare `on_behalf_of`, since a caller could
/// stamp only with a signed `GuestAttestation`, which went with `attest`,
/// and from the core in core-0.
/// Here an app serving someone else names them, and the act is signed as
/// this computer with `on_behalf_of` naming that person — whom the member
/// door verified (DESIGN §3, §8 "Localhost is the trust boundary"). The
/// name must parse as a person and be a person standing in the group;
/// anything else is refused and nothing is written, never dropped.)
pub(crate) async fn append_act(
    rail: &RingRail,
    namespace: &str,
    body: serde_json::Value,
) -> Response {
    let stamp = match stamp(namespace, &body) {
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
    let sealed = matches!(act, RailAct::Seal);
    let appended = if sealed {
        // A seal takes no stamp: it is delivery, not words.
        if let Some(person) = &stamp {
            tracing::debug!(target: "rails", namespace, %person, "rail: a seal is signed as this computer alone, whoever was named");
        }
        rail.seal(namespace)
            .map(|done| (done.op, Some(retire(&done.retired))))
    } else {
        let append = Append {
            act,
            on_behalf_of: stamp,
        };
        match rail.commit(namespace, &append) {
            Ok(Ok(recorded)) => Ok((recorded.lines()[0].clone(), None)),
            Ok(Err(CommitRefused::NoStanding)) => Err(RailError::NotInRoster {
                actor: rail.signer().actor(),
                namespace: namespace.to_string(),
            }),
            Ok(Err(CommitRefused::Unsettled)) => Err(RailError::Unsettled(namespace.to_string())),
            Ok(Err(CommitRefused::Command(AppendRefusal::NotInGroup(person)))) => {
                tracing::debug!(target: "rails", namespace, %person, "rail: refused — on_behalf_of names nobody in the group");
                return refused(
                    namespace,
                    "not_in_group",
                    format!("{person} is not in {namespace}"),
                );
            }
            Ok(Err(CommitRefused::Command(AppendRefusal::NotAnAppAct))) => {
                tracing::debug!(target: "rails", namespace, "rail: refused — the register's acts and the seal are not an app's to write");
                return refused(
                    namespace,
                    "not_an_app_act",
                    "an app writes records and corrections; admitting, removing and inviting are the group's own".into(),
                );
            }
            Err(e) => Err(e),
        }
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
                "namespace": namespace,
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

/// The person an append is on behalf of, parsed: `None` when the body names
/// nobody. Whether that person stands is `Append`'s to decide, under the
/// write lock.
fn stamp(namespace: &str, body: &serde_json::Value) -> Result<Option<Person>, Response> {
    let raw = match body.get("on_behalf_of") {
        None | Some(serde_json::Value::Null) => {
            tracing::debug!(target: "rails", namespace, "rail: the act is this computer's own — it names nobody");
            return Ok(None);
        }
        Some(serde_json::Value::String(raw)) => raw,
        Some(other) => {
            tracing::debug!(target: "rails", namespace, named = %other, "rail: refused — on_behalf_of is not text");
            return Err(refused(
                namespace,
                "not_a_name",
                "on_behalf_of names a person, as text".into(),
            ));
        }
    };
    Person::parse(raw).map(Some).map_err(|why| {
        tracing::debug!(target: "rails", namespace, raw, why = %why, "rail: refused — on_behalf_of is not a name");
        refused(namespace, "not_a_name", format!("on_behalf_of is not a name: {why}"))
    })
}

/// A typed refusal: the sentence, its `kind`, and the group.
fn refused(namespace: &str, kind: &str, why: String) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": why, "kind": kind, "namespace": namespace })),
    )
        .into_response()
}

/// GET /v1/rail/log — the namespace's acts, in one order, and its gaps:
/// admitted ops, each gap's own sentence (the rail's `Display`, so the page
/// and the terminal say the same words), the held count, completeness, and
/// the roster the ops were admitted against. (svrngs: the people standing
/// after those ops, from the register over the same read, in place of the
/// seed they were admitted from.)
async fn log_answer(rail: &RingRail, namespace: &str) -> Response {
    // ONE read, admitted once: the ops, the gaps and the people all come from
    // the same admission, so no write can land between them.
    let admission = match rail.admission(namespace) {
        Ok(a) => a,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    let gaps: Vec<serde_json::Value> = admission
        .gaps()
        .iter()
        .map(|g| {
            let mut v = serde_json::to_value(g).unwrap_or_default();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("message".into(), serde_json::Value::String(g.to_string()));
            }
            v
        })
        .collect();
    let register = admission.project::<Membership>();
    let people: Vec<(&String, &Person, &str)> = (register.standing.iter())
        .filter_map(|key| {
            register
                .person_of(key)
                .map(|(first, person)| (key, person, first))
        })
        .collect();
    Json(serde_json::json!({
        "namespace": namespace,
        "ops": admission.wire(),
        "gaps": gaps,
        "held": admission.held(),
        "complete": admission.is_complete(),
        // Unsettled cuts (#124): who stands is not judged, so nobody is
        // listed as standing; the `unsettled` gap says why.
        "people": (!register.unsettled).then(|| people_json(people)),
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
            "people": people_json(people.iter().map(|(key, m)| (key, &m.person, m.person_key.as_str()))),
        }))
        .into_response(),
        Err(refusal) => refusal,
    }
}

/// GET /v1/rail/people.
pub async fn people(
    State(api): State<RailApi>,
    Query(q): Query<RailQuery>,
) -> Result<Response, Response> {
    Ok(people_answer(&api.rail, &namespace_of(&q)?))
}

/// POST /v1/rail/append.
pub async fn append(
    State(api): State<RailApi>,
    Query(q): Query<RailQuery>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, Response> {
    let namespace = namespace_of(&q)?;
    group_of(&api.rail, &namespace)?;
    Ok(append_act(&api.rail, &namespace, body).await)
}

/// GET /v1/rail/log.
pub async fn log(
    State(api): State<RailApi>,
    Query(q): Query<RailQuery>,
) -> Result<Response, Response> {
    let namespace = namespace_of(&q)?;
    group_of(&api.rail, &namespace)?;
    Ok(log_answer(&api.rail, &namespace).await)
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
    #[expect(
        clippy::disallowed_types,
        reason = "the live lane's buffer, this computer's only"
    )]
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
/// Nothing is written anywhere. (svrngs: and it is fanned out, as the
/// daemon's `live_push` fanned it: offered to every key standing in the
/// group but this node's, each dialled over the node's [`IrohTransport`],
/// and the answer names each peer and whether it took the payload —
/// `delivered` counts them, so a lane half up is never read as "everyone
/// saw it". A group this computer does not hold, or holds with no seed, is
/// refused as the append door refuses it ([`standing_of`]), nothing sent.)
async fn push_answer(api: &RailApi, namespace: &str, body: axum::body::Bytes) -> Response {
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
    let standing = match standing_of(&api.rail, namespace) {
        Ok(people) => people,
        Err(refusal) => return refusal,
    };
    let peers = live_peers(&api.rail.signer().actor(), namespace, &standing);
    let peers = push_ephemeral(&api.transport, peers, namespace, payload).await;
    let delivered = peers.iter().filter(|p| p.delivered).count();
    tracing::debug!(
        target: "rails",
        namespace,
        bytes = body.len(),
        peers = peers.len(),
        delivered,
        "rail live: fanned out one payload"
    );
    Json(serde_json::json!({
        "bytes": body.len(),
        "peers": peers,
        "delivered": delivered,
    }))
    .into_response()
}

/// The peers a live payload in `namespace` is offered to: every key
/// standing in it but `this` node's, each by its key alone, named by its
/// person: the keys the round dials ([`crate::node::dialled`]), so neither
/// a browser's key (u4-1) nor one that is not an iroh key is offered it.
/// New (u3-3): upstream walked the mesh's Online members.
fn live_peers(this: &str, namespace: &str, standing: &BTreeMap<String, Member>) -> Vec<LivePeer> {
    let mut peers = Vec::new();
    for (actor, person, key) in dialled(this, namespace, standing) {
        tracing::debug!(target: "rails", namespace, actor, %person, "rail live: a person stands in the group, to offer the payload");
        peers.push(LivePeer {
            node: actor.clone(),
            name: Some(person.to_string()),
            contact: Contact::by_key(key),
        });
    }
    peers
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
) -> Result<Response, Response> {
    Ok(push_answer(&api, &namespace_of(&q)?, body).await)
}

/// GET /v1/rail/live.
pub async fn live_drain(
    State(api): State<RailApi>,
    Query(q): Query<RailQuery>,
) -> Result<Response, Response> {
    Ok(drain_answer(&api.live, &namespace_of(&q)?))
}

#[cfg(test)]
mod tests;
