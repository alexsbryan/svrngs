// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bytes an app places in a group, by hash, with a lifetime (onair-blob;
//! svrn-docs `rings/current/ON_AIR.md` §4.1): `PUT /v1/blob?group=&keep=`
//! and `GET /v1/blob/<hash>?group=` on the record API. The moving is the
//! folders' (folder.rs): the same store, the same provider and its one
//! rule, [`Folders::serves`], taught what an app placed, and the same
//! verified fetch, [`Folders::hold`]. What is new is that the app names the
//! hash, not a synced directory, and that the hash lapses `keep` seconds
//! after it was put, on every computer holding it: a member asked for it
//! first asks the group's other computers ([`ASKED`]) who holds it and for
//! how long yet, fetches it from those, and holds it no longer than they
//! do. What an app placed is kept in this process's memory alone: not the
//! record, not a folder, and nothing of it past a restart, at which the
//! store's sweep drops its bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use iroh_blobs::Hash;
use serde_json::{json, Map, Value};

use super::Folders;
use crate::iroh::{Contact, Dialer};
use crate::node::{Host, ANSWER_WITHIN};
use crate::register::Member;
use crate::ring_routes::err;

/// Where a member asks another whether it holds a hash an app placed in a
/// group, and for how long yet: on the members' protocol, answered to a
/// dialer standing in the group alone.
pub(crate) const ASKED: &str = "/internal/ring/blob/{hash}";

/// Each hash an app placed in a group, and when it lapses there. Kept
/// past its lapse, so one that lapsed is said apart from one never held.
pub(super) type Carried = BTreeMap<(Hash, String), Instant>;

/// What a computer says of a hash in a group: a closed set (AGENTS 9),
/// the answer at [`ASKED`] and the `kind` of a refused get.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "held", rename_all = "snake_case")]
pub(super) enum Holds {
    For { left_ms: u64 },
    Lapsed,
    NeverHeld,
}

/// A blob route's query: the group, always, and how long a put keeps.
#[derive(serde::Deserialize)]
pub(super) struct BlobQuery {
    group: String,
    keep: Option<u64>,
}

impl Folders {
    /// What this computer says of `hash` in `group` at `now`.
    pub(super) fn held(&self, hash: Hash, group: &str, now: Instant) -> Holds {
        match self.carried.borrow().get(&(hash, group.to_string())) {
            None => Holds::NeverHeld,
            Some(until) if *until <= now => Holds::Lapsed,
            Some(until) => Holds::For {
                left_ms: (*until - now).as_millis() as u64,
            },
        }
    }

    /// Every hash an app placed, in any group, not lapsed at `now`: what
    /// the store's sweep keeps of them.
    pub(super) fn carrying(&self, now: Instant) -> BTreeSet<Hash> {
        let carried = self.carried.borrow();
        let live = carried.iter().filter(|(_, until)| **until > now);
        live.map(|((hash, _), _)| *hash).collect()
    }

    /// `hash` held in `group` until `until`, or `None` to forget it there;
    /// a later lapse it had there kept.
    fn carry(&self, hash: Hash, group: &str, until: Option<Instant>) {
        let at = (hash, group.to_string());
        self.carried.send_modify(|c| match until {
            Some(until) => _ = c.insert(at.clone(), until.max(*c.get(&at).unwrap_or(&until))),
            None => _ = c.remove(&at),
        });
    }

    /// The people of `group`, if this computer stands in it.
    fn standing_in(&self, group: &str) -> Result<BTreeMap<String, Member>, Response> {
        let people = crate::rail::standing_of(self.host.rail(), group)?;
        if people.contains_key(self.host.actor()) {
            return Ok(people);
        }
        tracing::debug!(target: "blob", group, "blob: refused — this computer does not stand in the group");
        let why = format!("this computer does not stand in {group}");
        Err(err(StatusCode::FORBIDDEN, why))
    }

    /// `hash`'s bytes as this computer holds them, naming `from`.
    async fn answer(&self, hash: Hash, from: &str) -> Response {
        match self.store.get_bytes(hash).await {
            Ok(bytes) => ([("x-svrngs-holder", from.to_string())], bytes).into_response(),
            Err(e) => {
                tracing::warn!(target: "blob", %hash, error = %e, "blob: held, but the store cannot read it");
                err(StatusCode::INTERNAL_SERVER_ERROR, format!("the store: {e}"))
            }
        }
    }

    /// Each other computer standing in `group` asked at once, for at most
    /// [`ANSWER_WITHIN`], what it holds of `hash`: its key, and its answer
    /// or why there is none.
    async fn ask_group(&self, hash: Hash, group: &str, people: &People) -> Asked {
        let relay = |a: &str| people.get(a)?.relay.as_ref();
        let path = format!("{}?group={group}", ASKED.replace("{hash}", &hash.to_hex()));
        let mut asks = Vec::new();
        for (actor, _, contact) in crate::node::dialled(self.host.actor(), group, people, relay) {
            let (host, path, actor) = (self.host.clone(), path.clone(), actor.clone());
            asks.push(tokio::spawn(async move {
                let asked = tokio::time::timeout(ANSWER_WITHIN, ask(&host, contact, &path));
                let silent = format!("did not answer within {ANSWER_WITHIN:?}");
                (actor, asked.await.unwrap_or(Err(silent)))
            }));
        }
        let mut answers = Vec::new();
        for (actor, said) in n0_future::join_all(asks).await.into_iter().flatten() {
            tracing::debug!(target: "blob", %hash, group, holder = actor, ?said, "blob: a computer of the group asked what it holds");
            answers.push((actor, said));
        }
        answers
    }
}

type People = BTreeMap<String, Member>;

/// Each computer asked, and what it holds or why it did not say.
type Asked = Vec<(String, Result<Holds, String>)>;

/// `path` asked of the computer at `contact`: what it holds, or why
/// there is no answer.
async fn ask(host: &Host, contact: Contact, path: &str) -> Result<Holds, String> {
    let base = host.transport().base_url(&contact).await;
    let base = base.ok_or("no dialable path to it")?;
    let answer = host.http()?.get(format!("{base}{path}")).send().await;
    let answer = answer.map_err(|e| format!("the ask: {e}"))?;
    if !answer.status().is_success() {
        return Err(format!("refused the ask: {}", answer.status()));
    }
    answer.json().await.map_err(|e| format!("its answer: {e}"))
}

/// A refused get, said by its kind — `lapsed`, `never_held` or
/// `unanswered` — with each computer asked and what it said.
fn refused(hash: Hash, (status, kind, why): (StatusCode, &str, &str), asked: Value) -> Response {
    tracing::debug!(target: "blob", %hash, kind, %asked, "blob: refused — {why}");
    let said = json!({ "error": format!("{hash}: {why}"), "kind": kind, "asked": asked });
    (status, Json(said)).into_response()
}

/// `PUT /v1/blob?group=<id>&keep=<seconds>`: the body's bytes into the
/// store, placed in the group for `keep` seconds; `{"hash": <blake3 hex>}`.
#[expect(
    clippy::disallowed_methods,
    reason = "a put's lapse is read from the clock at the route, its adapter"
)]
pub(super) async fn put(
    State(f): State<Folders>,
    Query(q): Query<BlobQuery>,
    bytes: axum::body::Bytes,
) -> Response {
    if let Err(refusal) = f.standing_in(&q.group) {
        return refusal;
    }
    let until = |k| Instant::now().checked_add(Duration::from_secs(k));
    let Some((keep, Some(until))) = q.keep.map(|k| (k, until(k))) else {
        let why = "how long the bytes are kept must be named, in seconds this computer's clock counts: &keep=<seconds>";
        return err(StatusCode::BAD_REQUEST, why);
    };
    let tag = match f.store.add_bytes(bytes).temp_tag().await {
        Ok(tag) => tag,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, format!("the store: {e}")),
    };
    let hash = tag.hash();
    // Placed before the tag goes, so the sweep never finds it unkept.
    f.carry(hash, &q.group, Some(until));
    drop(tag);
    tracing::debug!(target: "blob", %hash, group = q.group, keep, "blob: put, placed in the group until it lapses");
    Json(json!({ "hash": hash })).into_response()
}

/// `GET /v1/blob/<hash>?group=<id>`: the bytes, from this computer's
/// store, else fetched, verified, from a computer of the group holding
/// them, and held here until they lapse there; the holder named in
/// `x-svrngs-holder`. Refused as [`refused`] says.
#[expect(
    clippy::disallowed_methods,
    reason = "a get's lapse is read from the clock at the route, its adapter"
)]
pub(super) async fn get(
    State(f): State<Folders>,
    Path(hash): Path<Hash>,
    Query(BlobQuery { group, .. }): Query<BlobQuery>,
) -> Response {
    let people = match f.standing_in(&group) {
        Ok(people) => people,
        Err(refusal) => return refusal,
    };
    let lapsed = (StatusCode::GONE, "lapsed", "it lapsed");
    match f.held(hash, &group, Instant::now()) {
        Holds::Lapsed => return refused(hash, lapsed, json!({})),
        Holds::For { .. } if matches!(f.lacks(hash).await, Ok(None)) => {
            tracing::debug!(target: "blob", %hash, group, "blob: held here, not lapsed");
            return f.answer(hash, f.host.actor()).await;
        }
        _ => {}
    }
    let answers = f.ask_group(hash, &group, &people).await;
    let (mut asked, mut holders, mut left) = (Map::new(), Vec::new(), u64::MAX);
    for (key, said) in &answers {
        if let Ok(Holds::For { left_ms }) = said {
            holders.push(key.clone());
            left = left.min(*left_ms);
        }
        let said = said.as_ref().map_or_else(|e| json!(e), |h| json!(h));
        asked.insert(key.clone(), said);
    }
    if holders.is_empty() {
        // Lapsed if any says so; never held only if every one says so.
        let said = |h: Holds| answers.iter().filter(|(_, a)| *a == Ok(h)).count();
        let never = (StatusCode::NOT_FOUND, "never_held", "no computer holds it");
        let kind = match (said(Holds::Lapsed), said(Holds::NeverHeld)) {
            (0, n) if n == answers.len() => never,
            (0, _) => (StatusCode::BAD_GATEWAY, "unanswered", "no holder answered"),
            _ => lapsed,
        };
        return refused(hash, kind, asked.into());
    }
    // Carried while it is fetched, so the sweep never drops it half held.
    let until = Instant::now() + Duration::from_millis(left);
    f.carry(hash, &group, Some(until));
    match f.hold(hash, &holders).await {
        Ok(from) => {
            let from = from.as_deref().unwrap_or(f.host.actor());
            tracing::info!(target: "blob", %hash, group, holder = from, left_ms = left, "blob: held from a computer of the group, carried until it lapses there");
            f.answer(hash, from).await
        }
        Err(why) => {
            f.carry(hash, &group, None);
            asked.extend(why.into_iter().map(|(k, w)| (k, w.into())));
            let unanswered = (StatusCode::BAD_GATEWAY, "unanswered", "no holder answered");
            refused(hash, unanswered, asked.into())
        }
    }
}

/// [`ASKED`]: what this computer holds of a hash in a group, to a dialer
/// standing in it; anyone else refused as the ring's routes refuse them.
#[expect(
    clippy::disallowed_methods,
    reason = "what is left of a lapse is read from the clock at the route, its adapter"
)]
pub(super) async fn asked(
    State(f): State<Folders>,
    dialer: Option<Extension<Dialer>>,
    Path(hash): Path<Hash>,
    Query(BlobQuery { group, .. }): Query<BlobQuery>,
) -> Response {
    let dialer = dialer.map(|Extension(d)| d);
    let rail = f.host.rail();
    if let Err(refusal) = crate::ring_routes::standing_asker(rail, dialer, "blob", &group) {
        return err(StatusCode::FORBIDDEN, refusal);
    }
    let held = f.held(hash, &group, Instant::now());
    tracing::debug!(target: "blob", %hash, group, ?held, "blob: a member asked what this computer holds");
    Json(held).into_response()
}
