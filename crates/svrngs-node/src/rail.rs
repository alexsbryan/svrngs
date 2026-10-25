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
//! paths, same server-assigned fields, same gap
//! sentences — so a page written against one daemon behaves the same against
//! the other (ARCH §10.6). What is deliberately NOT mirrored is the guest
//! half: there are no grants and no sessions here (loopback IS the auth),
//! so the namespace is always named explicitly.
//! (svrngs: append, log and the live lane came over, PLAN §4, U3's
//! inventory; the sync doors and `attest` stay behind — one process per
//! computer reads its own journals.)
//!
//! **The journals live under THIS process's data root**, never the daemon's
//! (§4 rule 1 — one data directory, one owner; a second process never opens
//! the daemon's dir, and the daemon's journals migrate in fp-54's commit, so
//! there is no dual-writer window). The signer is THIS process's node key.
//! (svrngs: upstream's census of rails' and the daemon's two keys went with
//! the mesh; a node here has one key, PLAN §4, U1's "One key signs and dials".)
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
use crate::iroh::IrohTransport;
use crate::live::{push_ephemeral, LivePeer};
use crate::node::dialled;
use crate::register::{Member, Standing};
use crate::ring_routes::err;
use crate::{actor_of, Device, Joined, Person, RailAct, RailError, RingRail};
use svrngs_core::{Append, AppendRefusal};

/// What the doors read: the journals, the nudge that wakes the round
/// after an append ([`crate::ring_sync::append`]), and the live lane's
/// buffer, which the live route fills ([`crate::ring_routes::ring_live`]).
/// (Upstream's handlers read them from `RailsDaemon`, which did not come
/// over, PLAN §4, U1's inventory.)
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
/// since nobody can be shown to stand in it — PLAN §4, U3's inventory,
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
        Ok(Standing::Members(people) | Standing::Left(people)) => {
            tracing::debug!(target: "rails", namespace, people = people.len(), "rail: the group is held here, left or not, and its register read");
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
/// to (DESIGN §3) — the person's first key, which every key of theirs
/// shares and the door tells a lend as `x-svrngs-key` (bash-devices), and
/// when the person joined, which tells two of one name apart
/// (bash-same-name), and what holds the key — `computer` or `browser`, so a
/// page draws a phone as one (v1ui-door (a)). The one shape every door that
/// names the people answers.
pub(crate) fn people_json<'a>(
    standing: impl IntoIterator<Item = (&'a String, &'a Person, &'a str, Joined, Device)>,
) -> serde_json::Value {
    (standing.into_iter())
        .map(|(key, person, first, joined, device)| serde_json::json!({ "person": person, "key": key, "person_key": first, "joined": joined, "device": device }))
        .collect()
}

/// The people standing, the register's rows, in [`people_json`]'s shape:
/// the people door's, and the members' `/people/lends` beside what they
/// lend, so a phone's page draws the door (v1ui-door (d)).
pub(crate) fn standing_json(s: &BTreeMap<String, Member>) -> serde_json::Value {
    let rows = s.iter();
    people_json(rows.map(|(k, m)| (k, &m.person, m.person_key.as_str(), m.joined, m.device)))
}

/// Who is visiting `r`'s group now, by this computer's clock, from the
/// fold's visits alone (svrngs-core's `Guest::live`), as the pages show
/// them (guests-pages; GUESTS §5 "The pages"): the visit's id, which its
/// host's Stop names; the guest; their host by name; until when; and
/// whether `me` hosts them, so may stop it. `None` when the clock cannot
/// be read, never read as nobody.
pub(crate) fn visiting(r: &crate::register::Register, me: &str) -> Option<Vec<serde_json::Value>> {
    let now = crate::door::unix_now()?;
    let live = (r.visits.iter()).filter(|(_, g)| g.live(&r.cut, now));
    let on: Vec<_> = live.map(|(id, g)| serde_json::json!({ "id": id, "guest": g.visit.guest, "host": r.members.get(&g.host).map(|m| &m.person), "until": g.visit.until, "yours": g.host == me })).collect();
    tracing::debug!(target: "rails", visits = r.visits.len(), on = on.len(), "rail: who is visiting now, the fold's visits live at this computer's time");
    Some(on)
}

/// POST /v1/rail/append — sign and append one act to the named namespace.
///
/// The body is the act alone. `seq`, the signature, the timestamp and the id
/// are all this daemon's to assign; an app that could choose its own sequence
/// number or actor could write as somebody else.
///
/// (svrngs: a program on this computer writes as this computer's own person,
/// "your computer is you" (svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md`
/// §4): a body naming that person in `on_behalf_of` is its own act, the name
/// dropped, and a body naming anyone else is refused, `not_yours`, since
/// another person's entry is signed on their own device (capability-signed).)
pub(crate) async fn append_act(
    rail: &RingRail,
    namespace: &str,
    body: serde_json::Value,
) -> Response {
    if let Err(refusal) = stamp(rail, namespace, &body) {
        return refusal;
    }
    // Taken as a `Value` and converted here rather than as `Json<RailAct>`,
    // so a refusal is the rail's own sentence instead of axum's rejection
    // prose (ARCH §10.6).
    let act = match RailAct::from_json(body) {
        Ok(act) => act,
        Err(e) => return err(StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
    };
    // A seal is an act like any other here: `Append` refuses it as not an
    // app's (arch-seal (b); WHO_KNOWS_WHOM §3).
    let append = Append { act };
    let appended = match rail.commit_said(namespace, &append) {
        Ok(Ok(recorded)) => Ok(recorded.lines()[0].clone()),
        Ok(Err(AppendRefusal::NotAnAppAct)) => {
            tracing::debug!(target: "rails", namespace, "rail: refused — the register's acts and a seal are not an app's to write");
            return refused(
                namespace,
                "not_an_app_act",
                "an app writes records and corrections; admitting, removing, inviting and sealing are not an app's".into(),
            );
        }
        Err(e) => Err(e),
    };
    match appended {
        Ok(op) => {
            // The whole op rides beside the flat fields, so a dialing client
            // that speaks the rail's TYPES (the mesh round's port, since the
            // journals moved here) reads back an `Op<SignedOp>` instead of
            // re-deriving one from the render (ARCH §10.6). The flat fields
            // stay: they are what a page renders, and the daemon's append
            // door answers with exactly them.
            let out = serde_json::json!({
                "id": op.id,
                "seq": op.kind.seq,
                "actor": op.actor,
                "ts_unix": op.ts_unix,
                "namespace": namespace,
                "op": op,
            });
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

/// Whether the body names only this computer's own person, if anyone: a
/// name that is not one is `not_a_name`, and anyone else's `not_yours`.
fn stamp(rail: &RingRail, namespace: &str, body: &serde_json::Value) -> Result<(), Response> {
    let raw = match body.get("on_behalf_of") {
        None | Some(serde_json::Value::Null) => return Ok(()),
        Some(serde_json::Value::String(raw)) => raw,
        Some(other) => {
            tracing::debug!(target: "rails", namespace, named = %other, "rail: refused — on_behalf_of is not text");
            let why = "on_behalf_of names a person, as text".into();
            return Err(refused(namespace, "not_a_name", why));
        }
    };
    let named = Person::parse(raw).map_err(|why| {
        tracing::debug!(target: "rails", namespace, raw, why = %why, "rail: refused — on_behalf_of is not a name");
        refused(namespace, "not_a_name", format!("on_behalf_of is not a name: {why}"))
    })?;
    let read = rail.admission(namespace).ok();
    let me = actor_of(rail.signer());
    let own = (read.as_ref()).and_then(|a| Some(a.membership().person_of(&me)?.1.clone()));
    tracing::debug!(target: "rails", namespace, %named, own = ?own, "rail: a program here writes as this computer's own person, or nobody's");
    match own {
        Some(own) if own == named => Ok(()),
        own => Err(refused(namespace, "not_yours", format!(
            "This computer writes only as {}. Another person's entry is signed on their own phone or computer.",
            own.as_ref().map_or("its own person", Person::as_str)
        ))),
    }
}

/// `POST /carry`, `[group, line]`: a line a device of the group signed
/// itself, a phone's own act, carried unchanged into this computer's copy
/// as the round stores a peer's (capability-signed; DESIGN.md §3, "The
/// computer's part"). It writes no line of its own. Refused, in words, when
/// the line does not verify, or whose standing it counts under
/// ([`svrngs_core::Op::stands_as`]: its signer's, or a holder's pass's
/// signer's, that pass theirs and naming the holder) stands nowhere in the
/// group; answered with whether the fold counts it now. Members' and
/// holders' protocols both serve it.
pub(crate) async fn carry(
    State(at): State<crate::node::Asking>,
    by: Option<axum::Extension<crate::iroh::Dialer>>,
    Json((group, line)): Json<(String, svrngs_core::Op<svrngs_core::SignedOp>)>,
) -> Response {
    use svrngs_core::RingVerifier;
    let rail = &at.host.rail;
    let id = svrngs_core::derive_id(&line, &group);
    let signed =
        id == line.id && svrngs_core::Ed25519Verifier.verify(&line.actor, &id, &line.kind.sig);
    let read = rail.admission(&group).ok();
    let held = (line.kind.grant.as_ref()).is_none_or(|p| p.holder == line.actor && p.signed());
    let stands =
        (read.as_ref()).is_some_and(|a| a.membership().standing.contains(line.stands_as()));
    tracing::debug!(target: "rails", group, %id, signer = %line.actor, as_ = line.stands_as(), by = ?by.map(|d| d.actor()), signed, held, stands, "rail: a line a device signed, carried here unchanged, or why not");
    if !(signed && held && stands) {
        let why = format!("This entry was not signed by anyone in {group}.");
        return refused(&group, "not_signed", why);
    }
    if let Err(e) = rail.ingest_all(&group, std::slice::from_ref(&line)) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    at.folders.nudge.notify_one();
    let counted = rail
        .admission(&group)
        .is_ok_and(|a| a.membership().counts(&id));
    Json(serde_json::json!({ "id": id, "counted": counted })).into_response()
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
    let register = admission.membership();
    let people: Vec<(&String, &Person, &str, Joined, Device)> = (register.standing.iter())
        .filter_map(|key| {
            let ((first, person), device) = register.person_of(key).zip(register.device(key))?;
            Some((key, person, first, register.joined(first), device))
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
            "people": standing_json(&people),
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

/// One arrived payload and who sent it: the dialer's key, verified by the
/// acceptor, and their person's first key, as `/v1/rail/people` names them
/// (ON_AIR §4.3), so an app knows who spoke without signing every payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Live {
    pub payload: String,
    pub key: String,
    pub person_key: String,
}

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
    payloads: VecDeque<Live>,
    /// Payloads evicted because the buffer was full, since the last drain.
    /// Eviction is a real loss and it is REPORTED, never silent.
    dropped: usize,
}

impl LiveBuffer {
    /// Take one arrived payload for `namespace`. Evicts that namespace's
    /// oldest when full.
    pub fn push(&self, namespace: &str, payload: Live) {
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
    pub(crate) fn drain(&self, namespace: &str) -> (Vec<Live>, usize) {
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
    let peers = live_peers(&actor_of(api.rail.signer()), namespace, &standing);
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
/// standing in it but `this` node's, each by its key and the relay its
/// group names (relay-in-record), named by its person: the keys the round dials ([`crate::node::dialled`]), so neither
/// a browser's key (u4-1) nor one that is not an iroh key is offered it.
/// New (u3-3): upstream walked the mesh's Online members.
fn live_peers(this: &str, namespace: &str, standing: &BTreeMap<String, Member>) -> Vec<LivePeer> {
    let mut peers = Vec::new();
    let relay = |a: &str| standing.get(a)?.relay.as_ref();
    for (actor, person, contact) in dialled(this, namespace, standing, relay) {
        tracing::debug!(target: "rails", namespace, actor, %person, "rail live: a person stands in the group, to offer the payload");
        peers.push(LivePeer {
            node: actor.clone(),
            name: Some(person.to_string()),
            contact,
        });
    }
    peers
}

/// GET /v1/rail/live — drain the caller's namespace since the last call,
/// each payload with its sender ([`Live`]): `{payload, key, person_key}`.
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
