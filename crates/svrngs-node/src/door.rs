// SPDX-License-Identifier: AGPL-3.0-or-later
//! The member door (DESIGN §3; §8 U3's inventory): a route on the members'
//! protocol that carries a request for a lend to its service on this
//! computer's loopback, and the answer back.
//!
//! `/lend/<lend>/<path>` reaches the lend's port as `/<path>`, the query
//! kept — the first segment names the service and is taken off, as
//! upstream's `split_app_name` did (`iroh_identity_forward.rs:181`).
//!
//! **Who is asking** is set by the door and nobody else. Every header a
//! client typed under [`IDENTITY_PREFIX`], and every header the lend's
//! secret is about to fill, is stripped, compared as upstream's
//! `rewrite_head` compared (`iroh_identity_forward.rs:297-328`: written as
//! the wire writes it, then lowercased, so case, padding and bytes the wire
//! drops dodge nothing); then the person and key are set from the
//! connection's verified [`Dialer`] and the register; then the lend's
//! secret ([`crate::declared`]). Exactly one of each reaches the service, and
//! it is the door's. The rules are upstream's, the code is not: upstream
//! rewrote request heads on a byte splice, and here hyper has parsed the
//! request before the door sees it (DESIGN §8, U1's inventory).
//!
//! **Bodies stream both ways**, neither buffered: a request's body is read
//! as the service reads it, a response's as the member reads it, so a large
//! upload or a whole file costs the door no more memory than a chunk.
//! `Range`, `If-Range` and `Content-Range` pass untouched, as every header
//! but the hop-by-hop ones does, so a seek is the service's to answer.
//! Upstream streamed a GET's response and passed `Range` alone
//! (`serve_client.rs:616-655`, `forward_stream`); it streamed no request body
//! anywhere.
//!
//! **A live socket** (u5-2): a request asking to switch protocols — a
//! WebSocket's `Upgrade` — is asked again of the service, and once it
//! answers 101 the two connections are spliced, byte for byte, the person
//! counted on the lend while it is open.
//!
//! **The secret never goes back.** A response header holding the lend's
//! secret is dropped, named at warn by its header, never its value.
//!
//! **A lend's sign-in** (mwp-1, [`sign_in`]): a lend declaring one has no
//! one secret; the door asks the lender's program on each person's first
//! request, sets that person's headers on each of their requests, and swaps
//! a stand-in for their secret both ways, and puts their script first in
//! each page answered to them (mwp-2). A request it cannot sign in is
//! refused, `502`, never carried unsigned.
//!
//! **How many at once, and each one's share** (u3-6; DESIGN §8, "Many
//! people, one library"): before a request is carried, the door asks
//! [`limit`] whether its person is let in — counted while a request of
//! theirs is in flight or within [`limit::WINDOW`] of one, a viewer whose
//! path ends at a private address guessed home and not counted. A newcomer
//! past the lend's limit is refused, `503`, with the reason and how many
//! are using it. A person let in is told their share of the upload the lend
//! declares, the declared upload over the people counted, in
//! [`SHARE_HEADER`] on the request the service receives and on the answer
//! the member receives; a lend declaring none tells no share. Each verdict
//! is traced at info with the path the door observed ([`Via`]) — its bytes,
//! RTT and whether it is relayed — so a watcher of the lender's log reads
//! every share without the debug level.
//!
//! **What it lends, asked** (u3-5; DESIGN §8, "Lends are asked, not
//! stored"): `GET /lends` answers a dialer standing in a group this
//! computer holds every lend declared here now, read from the declarations
//! at each ask, each with how many people use it now ([`Using`]): the
//! people [`limit`] counts on it.
//!
//! **A pass's holder** (u5-2; DESIGN §3, "Lent apps", call 6): on the
//! pass holders' protocol ([`holders_router`]), which any key may speak, a
//! request for a lend carries a pass in [`PASS_HEADER`], and the door lets
//! its holder in as the person who signed it only when the pass opens this
//! lend to the key that dialled, now, and its signer stands in a group
//! held here ([`svrngs_core::pass::Pass::check`]); then it is that person's
//! request, counted, told its share and carried as any member's. A lend is
//! named by its name or by its label ([`find`]), which names this
//! computer's key.
//!
//! **What the door refuses**, by name: a request not on an iroh stream; a
//! dialer standing in no group this computer holds; a path that can step
//! out of its lend — a `..` segment, an encoded `.` or `/` — as
//! `route_by_prefix` refuses one (`iroh_routed_forward.rs:318-337`); a lend
//! this computer does not declare; a path the lend refuses
//! ([`crate::lend::RefusedPath::covers`]); and a newcomer past the lend's
//! limit. Each refusal, and the answer for a service that does not answer,
//! is marked as svrngs's, which ([`LendRefusal`]); the service's own answers never are.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::{self, HeaderMap, HeaderName, HeaderValue};
use axum::http::{StatusCode, Version};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::{Extension, Json};
use http_body_util::BodyExt;
use tokio::sync::{mpsc, oneshot};

use crate::declared::read_declared_in;
use crate::iroh::{Dialer, Via};
use crate::lend::{self, Lend, LendName, Offered};
use crate::ring_routes::err;
use crate::{Person, RingRail};
use limit::{Counts, Verdict, Whereabouts};
use svrngs_core::pass::{Label, Pass};

pub mod choices;
pub mod limit;
pub mod sign_in;
use sign_in::Signing;

/// svrngs's own header prefix: every header under it on a request through
/// the door is the door's. A client's is stripped whole.
pub const IDENTITY_PREFIX: &str = "x-svrngs-";

/// The person asking, as the register names them: their name in the first
/// group this computer holds, in the groups' order, that they stand in.
pub const PERSON_HEADER: &str = "x-svrngs-person";

/// The key the connection verified, as the record spells an actor: its
/// lowercase hex.
pub const KEY_HEADER: &str = "x-svrngs-key";

/// The app asking: the label of the pass's `app`, the lend whose pages
/// hold it, under the person's signature (sup-1) — the lend the person's
/// page opened (`web/app.js`, `open`), or a computer's page as its origin
/// names it ([`crate::origin`]). Absent with no pass and no app named, so a
/// service reads its absence as no app named (su-1, the scoped-use spike).
pub const APP_HEADER: &str = "x-svrngs-app";

/// A person's share of the upload a lend declares, in bits a second, on
/// each request through the door and its answer: a declared figure, as the
/// name says (DESIGN §8, "The upload is declared"), never a measured one.
pub const SHARE_HEADER: &str = "x-svrngs-declared-share-bps";

/// The pass a holder presents with every request on the pass holders'
/// protocol (u5-2): [`Pass::encode`]'s form. Under [`IDENTITY_PREFIX`], so
/// no service ever receives it.
pub const PASS_HEADER: &str = "x-svrngs-pass";

/// On every answer svrngs makes in a lend's place, never on the lend's
/// own (a service's is dropped, [`forward`]): which of [`LendRefusal`] it is,
/// so an app tells svrngs's word from the lend's by more than a status
/// that both use (mnp-1; `demo/runs/mn/2026-10-08/README.md` §9, 2).
pub const REFUSED_HEADER: &str = "x-svrngs-refused";

/// Why svrngs answered rather than the lend: a closed set (AGENTS 9),
/// each kind spelt once in [`REFUSED_HEADER`] and the body's `kind`, as the
/// lend's worker spells it too (`web/lend.js`, `say`).
#[derive(Debug, Clone, Copy)]
pub enum LendRefusal {
    /// The lender's computer, or the lend's service on it, did not answer.
    Away,
    /// The pass presented does not open the lend now: ask the page for another.
    Pass,
    /// Not open to this asker, or not from where it asks.
    Closed,
    /// Nothing is lent here by that name.
    Unlent,
    /// A path the lend refuses, or that no lend can be asked.
    Path,
    /// As many people as the lend's limit are using it.
    Full,
    /// The lend's sign-in did not sign the person in, or its answer cannot be read.
    SignIn,
    /// The computer that lends it cannot read its own state.
    Fault,
    /// The lend asks, and its person said no yes to the app asking (sup-2).
    Unasked,
    /// The lend asks, and its person stopped the app asking.
    Stopped,
}

impl LendRefusal {
    pub const fn as_str(self) -> &'static str {
        match self {
            LendRefusal::Away => "away",
            LendRefusal::Pass => "pass",
            LendRefusal::Closed => "closed",
            LendRefusal::Unlent => "unlent",
            LendRefusal::Path => "path",
            LendRefusal::Full => "full",
            LendRefusal::SignIn => "sign-in",
            LendRefusal::Fault => "fault",
            LendRefusal::Unasked => "unasked",
            LendRefusal::Stopped => "stopped",
        }
    }

    /// svrngs's answer, `status` and `words`, marked as its own.
    pub(crate) fn say(self, status: StatusCode, words: impl Into<String>) -> Response {
        self.said(status, serde_json::json!({ "error": words.into() }))
    }

    /// svrngs's answer, `status` and `body`, its kind in the body and the header.
    fn said(self, status: StatusCode, mut body: serde_json::Value) -> Response {
        body["kind"] = self.as_str().into();
        let mut answer = (status, Json(body)).into_response();
        answer
            .headers_mut()
            .insert(REFUSED_HEADER, HeaderValue::from_static(self.as_str()));
        answer
    }
}

/// How long the door waits on the group's ring for a pass's unknown
/// signer: a member that knows them answers well under a second on a relay
/// and ends the wait, whoever else is asleep (asleep-1-none-held); with
/// none awake that knows them the pass is refused as before.
const ASK_WITHIN: Duration = Duration::from_secs(10);

/// Where a request for a lend starts.
pub const LEND_ROUTE: &str = "/lend/";

/// Where a member asks what this computer lends.
pub const LENDS_ROUTE: &str = "/lends";

/// Where a lend's choices are said and asked ([`choices`]).
const CHOICES: &str = "/choices/{lend}";

/// How long the door waits for a lend's service to take a connection; once
/// connected, a body takes as long as it takes, as upstream's
/// `forward_stream` bounded only its connect.
const CONNECT_WITHIN: Duration = Duration::from_secs(10);

/// Hop-by-hop headers (RFC 9110 §7.6.1), never carried across the door in
/// either direction, beside any a message's `Connection` names: the strip
/// lists of `sovereign-pods/src/worker_inference_proxy.rs:74-93`, less
/// `host` (replaced, [`own_address`]) and `authorization` (a lend's secret displaces a client's,
/// and a lend declaring none leaves the client's alone) and
/// `content-length` (kept, so a `HEAD` and a `206` keep their length).
/// A message asking to switch protocols asks again on the far side
/// ([`upgrade_of`], u5-2), so a WebSocket crosses the door.
const HOP_BY_HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authorization",
    "proxy-connection",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// What the door reads: the register, where the lends are declared, and
/// who is using each now.
#[derive(Clone)]
pub struct Door {
    pub rail: Arc<RingRail>,
    /// A round of the group's ring, asked for a pass's unknown signer.
    pub ring: crate::ring_sync::AskRound,
    /// This computer's key, which its lends' labels name.
    pub key: [u8; 32],
    pub data_dir: PathBuf,
    pub using: Using,
    pub sign_ins: sign_in::SignIns,
    /// Each person's yes and stop to an app, written through one task.
    pub choices: choices::Choices,
}

/// The door's routes, for the members' protocol's router: a lend, every
/// method, and the list of what this computer lends.
pub fn router(door: Door) -> axum::Router {
    axum::Router::new()
        .route("/lend/{*rest}", any(through))
        .route(LENDS_ROUTE, get(lends))
        .route(CHOICES, get(choices::choices_of).post(choices::choose))
        .with_state(door)
}

/// The door's route on the pass holders' protocol ([`crate::alpn::Alpn::Lend`]):
/// a lend, every method, and nothing else.
pub fn holders_router(door: Door) -> axum::Router {
    axum::Router::new()
        .route("/lend/{*rest}", any(through_pass))
        .route(CHOICES, axum::routing::post(choices::stop_with_pass))
        .with_state(door)
}

/// Who is using each lend now, as [`limit::Counts`] decides it: the
/// door's own state, owned by one task and told by messages (RUST_ARCH 3,
/// 4), which reads the clock as each message arrives. A request let in
/// begins a use, and its answer, dropped once streamed or abandoned, ends
/// it ([`InFlight`]). Never in the record.
#[derive(Clone)]
pub struct Using(mpsc::UnboundedSender<Use>);

enum Use {
    /// A person, at a guessed whereabouts, asks for a lend; the verdict is
    /// answered with the hold of a use let in, which ends it when dropped
    /// — so an asker gone before hearing ends its own.
    Ask(
        Box<Lend>,
        String,
        Whereabouts,
        oneshot::Sender<(Verdict, Option<InFlight>)>,
    ),
    Ended(LendName, String),
    People(LendName, oneshot::Sender<u32>),
}

#[expect(
    clippy::disallowed_methods,
    reason = "the door's clock: when a use of a lend begins and ends"
)]
fn now() -> Instant {
    Instant::now()
}

impl Using {
    /// The task that keeps the count, ending when the last handle drops.
    pub fn spawn() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Use>();
        let hold = tx.downgrade();
        tokio::spawn(async move {
            let mut counts = Counts::default();
            while let Some(u) = rx.recv().await {
                let now = now();
                match u {
                    Use::Ask(lend, actor, whereabouts, answer) => {
                        let verdict = counts.ask(&lend, &actor, whereabouts, now);
                        let held = match (verdict, hold.upgrade()) {
                            (Verdict::In { .. }, Some(using)) => Some(InFlight {
                                using,
                                lend: lend.name.clone(),
                                actor,
                            }),
                            (Verdict::In { .. }, None) => {
                                counts.end(&lend.name, &actor, now);
                                continue;
                            }
                            _ => None,
                        };
                        // Unheard, the hold drops with the answer and ends the use.
                        let _ = answer.send((verdict, held));
                    }
                    Use::Ended(lend, actor) => counts.end(&lend, &actor, now),
                    Use::People(lend, answer) => {
                        let _ = answer.send(counts.people(&lend, now));
                    }
                }
            }
        });
        Self(tx)
    }

    /// `actor`, at `whereabouts`, asks for `lend`: the verdict, and the
    /// hold of a use let in; `None` when the count's task is gone, which is
    /// no verdict at all.
    async fn ask(
        &self,
        lend: &Lend,
        actor: &str,
        whereabouts: Whereabouts,
    ) -> Option<(Verdict, Option<InFlight>)> {
        let (answer, told) = oneshot::channel();
        self.0
            .send(Use::Ask(
                Box::new(lend.clone()),
                actor.to_string(),
                whereabouts,
                answer,
            ))
            .ok()?;
        told.await.ok()
    }

    /// How many people use `lend` now; `None` when the count's task is
    /// gone, which is not a count of none.
    pub async fn people(&self, lend: &LendName) -> Option<u32> {
        let (answer, counted) = oneshot::channel();
        self.0.send(Use::People(lend.clone(), answer)).ok()?;
        counted.await.ok()
    }
}

/// One request's use of a lend, ended when dropped.
struct InFlight {
    using: mpsc::UnboundedSender<Use>,
    lend: LendName,
    actor: String,
}

impl Drop for InFlight {
    fn drop(&mut self) {
        let _ = self.using.send(Use::Ended(
            self.lend.clone(),
            std::mem::take(&mut self.actor),
        ));
    }
}

/// `GET /lends`: what this computer lends, to a dialer standing in a group
/// it holds, read from the declarations now. A declaration that cannot be
/// read is left out, named at warn; a lend whose count cannot be had is
/// left out too, never answered as used by none.
async fn lends(
    State(door): State<Door>,
    dialer: Option<Extension<Dialer>>,
) -> Result<Response, Response> {
    let Some(Extension(dialer)) = dialer else {
        tracing::warn!(target: "door", "door: lends refused — no verified dialer, so the request did not come on an iroh stream");
        return Err(err(
            StatusCode::FORBIDDEN,
            "what a computer lends is asked only from another computer of its group",
        ));
    };
    let actor = dialer.actor();
    if door.rail.standing_anywhere(&actor).is_none() {
        tracing::debug!(target: "door", actor, "door: lends refused — the dialer stands in no group held here");
        return Err(err(
            StatusCode::FORBIDDEN,
            "this computer lends nothing to you",
        ));
    }
    let offered = offered(&door).await?;
    tracing::debug!(target: "door", actor, lends = offered.len(), "door: lends — answered what this computer lends");
    Ok(Json(serde_json::json!({ "lends": offered })).into_response())
}

/// Every lend this computer declares now, each with how many people use
/// it ([`Using`]); or, when the declarations cannot be read, the refusal
/// every route listing lends answers, a 500 in words. A declaration that
/// cannot be read is left out, named at warn; a lend whose count cannot be
/// had is left out too, never answered as used by none.
pub(crate) async fn offered(door: &Door) -> Result<Vec<Offered>, Response> {
    let declared = match lend::all(&door.data_dir) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(target: "door", error = %e, "door: lends — the declarations cannot be read");
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "what this computer lends cannot be read on it",
            ));
        }
    };
    let mut offered = Vec::new();
    for lend in declared {
        let lend = match lend {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!(target: "door", error = %e, "door: lends — a declaration cannot be read, so it is left out");
                continue;
            }
        };
        let Some(using) = door.using.people(&lend.name).await else {
            tracing::warn!(target: "door", lend = %lend.name, "door: lends — no count of who uses it, so it is left out");
            continue;
        };
        offered.push(Offered {
            name: lend.name,
            kind: lend.kind,
            limit: lend.limit,
            using,
            upload: lend.upload,
            reads: lend.reads,
            asks: lend.asks,
        });
    }
    Ok(offered)
}

/// A name as upstream's `rewrite_head` compared it (`compare_name`,
/// `iroh_identity_forward.rs:274`): visible ASCII only, trimmed, lowercased.
fn compare_name(name: &str) -> String {
    name.chars()
        .filter(|c| (' '..='~').contains(c))
        .collect::<String>()
        .trim()
        .to_ascii_lowercase()
}

/// Whether `path` can step out of the lend it names: a `..` segment, or an
/// encoded `.` or `/` anywhere (`route_by_prefix`, upstream's guard).
fn can_step_out(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    path.split('/').any(|seg| seg == "..") || lower.contains("%2e") || lower.contains("%2f")
}

/// The lend a request's path names, and the path its service is asked for.
fn split_lend(path: &str) -> Option<(&str, String)> {
    let rest = path.strip_prefix(LEND_ROUTE)?;
    let (name, tail) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let tail = if tail.is_empty() { "/" } else { tail };
    Some((name, tail.to_string()))
}

/// Remove the hop-by-hop headers from `headers`, and any its `Connection`
/// names; answer how many went.
pub(crate) fn strip_hop_by_hop(headers: &mut HeaderMap) -> usize {
    let named: Vec<String> = headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .map(compare_name)
        .collect();
    let gone: Vec<HeaderName> = headers
        .keys()
        .filter(|n| {
            let name = compare_name(n.as_str());
            HOP_BY_HOP.contains(&name.as_str()) || named.contains(&name)
        })
        .cloned()
        .collect();
    let mut n = 0;
    for name in gone {
        n += headers.get_all(&name).iter().count();
        headers.remove(&name);
    }
    n
}

/// The protocol a message asks to switch to: its `Upgrade`, when its
/// `Connection` names `upgrade` (RFC 9110 §7.8). Read before the hop-by-hop
/// headers go, so the door carries a WebSocket's upgrade (u5-2), where it
/// stripped `Upgrade` with the rest.
pub(crate) fn upgrade_of(headers: &HeaderMap) -> Option<HeaderValue> {
    let named = headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .any(|t| compare_name(t) == "upgrade");
    named.then(|| headers.get(header::UPGRADE).cloned())?
}

/// Ask for `protocol` again on a head whose hop-by-hop headers went.
pub(crate) fn keep_upgrade(headers: &mut HeaderMap, protocol: HeaderValue) {
    headers.insert(header::CONNECTION, HeaderValue::from_static("upgrade"));
    headers.insert(header::UPGRADE, protocol);
}

/// Once both sides have switched, copy the bytes each writes to the other
/// until either ends: a WebSocket, across the door. `held` lasts as long,
/// so a person counts on the lend while their socket is open.
pub(crate) fn splice<H: Send + 'static>(
    asker: hyper::upgrade::OnUpgrade,
    service: hyper::upgrade::OnUpgrade,
    held: H,
    lend: String,
) {
    tokio::spawn(async move {
        let _held = held;
        let (asker, service) = match tokio::try_join!(asker, service) {
            Ok(both) => both,
            Err(e) => {
                tracing::debug!(target: "door", lend, error = %e, "door: an upgrade did not complete on both sides");
                return;
            }
        };
        let (mut a, mut s) = (
            hyper_util::rt::TokioIo::new(asker),
            hyper_util::rt::TokioIo::new(service),
        );
        match tokio::io::copy_bidirectional(&mut a, &mut s).await {
            Ok((up, down)) => {
                tracing::debug!(target: "door", lend, up, down, "door: an upgraded connection ended")
            }
            Err(e) => {
                tracing::debug!(target: "door", lend, error = %e, "door: an upgraded connection ended in error")
            }
        }
    });
}

/// The request's head as the service is to see it: the client's headers
/// but the hop-by-hop ones, those under [`IDENTITY_PREFIX`] and those a
/// secret fills; then who is asking; then the secret. Answers how many the
/// client typed that the door displaced.
fn rewrite_headers(
    headers: &mut HeaderMap,
    identity: [(HeaderName, HeaderValue); 2],
    secret: &[(HeaderName, HeaderValue)],
) -> usize {
    strip_hop_by_hop(headers);
    let declared: Vec<String> = secret
        .iter()
        .map(|(n, _)| compare_name(n.as_str()))
        .collect();
    let typed: Vec<HeaderName> = headers
        .keys()
        .filter(|n| {
            let name = compare_name(n.as_str());
            name.starts_with(IDENTITY_PREFIX) || declared.contains(&name)
        })
        .cloned()
        .collect();
    let mut displaced = 0;
    for name in typed {
        displaced += headers.get_all(&name).iter().count();
        headers.remove(&name);
    }
    for (name, value) in identity.into_iter().chain(secret.iter().cloned()) {
        headers.append(name, value);
    }
    displaced
}

/// The lend's secret as headers. A declared name or value that cannot be a
/// header is skipped, named at warn by its header, never its value.
fn secret_headers(door: &Door, lend: &LendName) -> Vec<(HeaderName, HeaderValue)> {
    read_declared_in(&lend::secret_dir(&door.data_dir, lend))
        .into_iter()
        .filter_map(|(name, value)| {
            match (
                HeaderName::try_from(name.as_str()),
                HeaderValue::try_from(value),
            ) {
                (Ok(n), Ok(v)) => Some((n, v)),
                _ => {
                    tracing::warn!(target: "door", lend = %lend, header = %name,
                        "door: a declared secret cannot be a header, so it is not sent");
                    None
                }
            }
        })
        .collect()
}

/// The address a service is asked on, as on its own computer:
/// `localhost:<its port>` (u5-2). Every lend has an origin of its own, which
/// the service cannot know, so an app that refuses a `Host` it does not
/// know — as one guarding against DNS rebinding does — answers as it would
/// its lender on that computer; and [`as_path`] turns a redirect to this
/// address back into a path the browser reads on the lend's origin. Until
/// u5-2 the door passed the browser's `Host`, so the service built its
/// links for the address the browser used, one `<lender's bridge>/lend/<name>/`
/// for every lend.
fn own_address(lend: &Lend) -> HeaderValue {
    HeaderValue::from_str(&format!("localhost:{}", lend.port))
        .expect("a host and a port are a header")
}

/// An `Origin` that is the lend's own — its first name the lend's label, as
/// every lend's is (DESIGN §7) — told to the service
/// as its own address, as the Host is ([`own_address`]), so an app that
/// takes a post only from its own address, as Django's origin check does
/// (Paperless-ngx; u5-catalogue-dry), takes the lend's own pages' posts.
/// Any other origin is left as it is, so the service still refuses a post
/// from a page that is not the lend's.
fn own_origin(headers: &mut HeaderMap, label: &Label, lend: &Lend) {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|o| o.to_str().ok()) else {
        return;
    };
    let host = origin.split_once("://").map_or("", |(_, rest)| rest);
    let first = host.split(['.', ':']).next().unwrap_or_default();
    if !first.eq_ignore_ascii_case(&label.to_string()) {
        tracing::debug!(target: "door", lend = %lend.name, origin, "door: the request's origin is not the lend's own; left as it is");
        return;
    }
    let own = HeaderValue::from_bytes(&[b"http://", own_address(lend).as_bytes()].concat())
        .expect("a host and a port are a header");
    tracing::debug!(target: "door", lend = %lend.name, "door: the request's origin is the lend's own; told to the service as its own address");
    headers.insert(header::ORIGIN, own);
}

/// `location` with the service's own address taken off, so `http://localhost:<port>/x`
/// is `/x`, read on whatever origin the asker reached the lend at; `None`
/// when it points anywhere else, which is left as it is.
fn as_path(location: &str, lend: &Lend) -> Option<String> {
    let rest = ["localhost", "127.0.0.1", "[::1]"]
        .iter()
        .find_map(|host| {
            let own = format!("http://{host}:{}", lend.port);
            let head = location.get(..own.len())?;
            head.eq_ignore_ascii_case(&own)
                .then(|| &location[own.len()..])
        })?;
    match rest.chars().next() {
        None => Some("/".into()),
        Some('/' | '?' | '#') => Some(rest.into()),
        Some(_) => None,
    }
}

/// Drop every response header holding one of `secret`'s values; answer the
/// names dropped.
fn drop_echoed_secret(
    headers: &mut HeaderMap,
    secret: &[(HeaderName, HeaderValue)],
) -> Vec<String> {
    let holds = |v: &HeaderValue| {
        secret.iter().any(|(_, s)| {
            let s = s.as_bytes();
            !s.is_empty() && v.as_bytes().windows(s.len()).any(|w| w == s)
        })
    };
    let echoed: Vec<HeaderName> = headers
        .iter()
        .filter(|(_, v)| holds(v))
        .map(|(n, _)| n.clone())
        .collect();
    for name in &echoed {
        headers.remove(name);
    }
    echoed.iter().map(|n| n.to_string()).collect()
}

/// Who asks for a lend through the door: a closed set (AGENTS 9), each
/// shown to be a person in a group held here its own way.
#[derive(Debug, Clone)]
pub(crate) enum Who {
    /// A dialer on the members' protocol, whose key is the person's; the
    /// app its own pass names, when it presents one ([`own_app`]).
    Member(Dialer),
    /// A dialer on the pass holders' protocol, whose key holds the pass it
    /// presents, signed by the person (u5-2).
    Holder(Dialer),
    /// This computer, opening its own lend at the lend's origin on its
    /// loopback ([`crate::origin`], u5-2): the person this computer's key
    /// is, counted as any other, for the app its origin names (sup-1).
    Own(Label),
}

/// The person the door resolves a request to (ONE_PERSON_MANY_DEVICES.md
/// §4): today the one key they have, a member's dialler or a pass's signer.
/// Made only here ([`PersonKey::resolved`]), so nothing kept about a person
/// — a sign-in's answer, a choice — is keyed by the raw key a request came
/// with, and another device for the same person changes that one function.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct PersonKey(String);

impl PersonKey {
    /// The person `key`, actor hex, is: the key itself, until a person
    /// spans devices.
    fn resolved(key: String) -> Self {
        Self(key)
    }
}
svrngs_core::text_newtype!(PersonKey);

/// The person a request for a lend is carried as.
struct Asker {
    actor: PersonKey,
    group: String,
    person: Person,
    /// The pass's app; none for a key of the group's with no pass.
    app: Option<Label>,
}

/// Who asks, decided before the lend is found or after: a member by the
/// key it dials with, a holder only by a pass for the lend it names.
enum Pending {
    Known(Asker),
    Pass(Dialer),
}

/// One request through the door on the pass holders' protocol.
async fn through_pass(
    State(door): State<Door>,
    dialer: Option<Extension<Dialer>>,
    via: Option<Extension<Via>>,
    req: Request,
) -> Result<Response, Response> {
    let Some(Extension(dialer)) = dialer else {
        tracing::warn!(target: "door", "door: refused — no verified dialer, so the request did not come on an iroh stream");
        return Err(LendRefusal::Closed.say(
            StatusCode::FORBIDDEN,
            "a lend is reached only through its group",
        ));
    };
    lend_for(&door, Who::Holder(dialer), via.map(|Extension(v)| v), req).await
}

/// The lend `raw` names here: by its name, or by its label when that names
/// this computer's key (u5-2). `None` when this computer lends none so.
fn find(door: &Door, raw: &str) -> Result<Option<Lend>, lend::LendError> {
    if let Some(label) = Label::parse(raw).filter(|l| l.lender() == door.key) {
        let found = lend::all(&door.data_dir)?
            .into_iter()
            .filter_map(Result::ok)
            .find(|l| Label::of(&door.key, l.name.as_str()) == label);
        tracing::debug!(target: "door", label = raw, found = ?found.as_ref().map(|l| l.name.to_string()), "door: a lend named by its label");
        return Ok(found);
    }
    match LendName::parse(raw) {
        Ok(name) => lend::read(&door.data_dir, &name),
        Err(_) => Ok(None),
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "the door's clock: whether a pass has lapsed, and until when a computer's own lasts"
)]
pub(crate) fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// The person whose pass `holder` presents in `headers` for `lend`: the
/// pass must open it to that key now, and its signer stand in a group held
/// here. A refusal names no group, and says only that this is not open to
/// the asker, with `kind: "pass"`, so a runtime holding the pass knows to
/// ask its person's page for another. A signer standing in no group held
/// here is asked of the group's ring once, within [`ASK_WITHIN`], until a
/// member that knows them answers or the round ends, before the refusal: a
/// person admitted at another computer is known here only after a round
/// (mnp-1).
async fn holder_of(
    door: &Door,
    holder: &Dialer,
    headers: &HeaderMap,
    lend: &Lend,
) -> Result<Asker, Response> {
    let key = holder.actor();
    let refused = |why: &str| {
        tracing::debug!(target: "door", holder = key, lend = %lend.name, why, "door: refused — a pass that does not open this lend");
        LendRefusal::Pass.say(
            StatusCode::FORBIDDEN,
            format!(
                "{} is not open to you here; open it again from your group's page",
                lend.name
            ),
        )
    };
    let Some(pass) = headers
        .get(PASS_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(Pass::decode)
    else {
        return Err(refused("no pass, or none that reads"));
    };
    let label = Label::of(&door.key, lend.name.as_str());
    if let Err(fault) = pass.check(&key, &label, unix_now()) {
        return Err(refused(&fault.to_string()));
    }
    let mut standing = door.rail.standing_anywhere(&pass.person);
    if standing.is_none() {
        tracing::debug!(target: "door", holder = key, signer = pass.person, lend = %lend.name,
            "door: the pass's signer stands in no group held here, so the group's ring is asked once first");
        let known = || door.rail.standing_anywhere(&pass.person).is_some();
        let in_time = door.ring.until(ASK_WITHIN, known).await;
        standing = door.rail.standing_anywhere(&pass.person);
        tracing::debug!(target: "door", signer = pass.person, in_time, known = standing.is_some(),
            "door: the ring asked — whether a member that knows the signer answered, or a round ended, in time, and whether the signer stands now");
    }
    let Some((group, person)) = standing else {
        return Err(refused("its signer stands in no group held here"));
    };
    tracing::debug!(target: "door", holder = key, person = person.as_str(), lend = %lend.name, until = pass.until,
        "door: a pass opens this lend, so its holder asks as the person who signed it");
    Ok(Asker {
        app: Some(pass.app),
        actor: PersonKey::resolved(pass.person),
        group,
        person,
    })
}

/// The app a member's own pass names (sup-1): a computer's origin signs
/// one for itself, its key the person and the holder, naming the app its
/// page is ([`crate::origin`]), so this door reads the app from the one
/// carrier a phone's request brings too. `None` with no pass; a pass not
/// the dialer's own, or not opening this lend to it now, is refused
/// `pass`, never read as no app.
fn own_app(
    door: &Door,
    dialer: &Dialer,
    headers: &HeaderMap,
    lend: &Lend,
) -> Result<Option<Label>, Response> {
    let Some(raw) = headers.get(PASS_HEADER) else {
        return Ok(None);
    };
    let key = dialer.actor();
    let label = Label::of(&door.key, lend.name.as_str());
    let pass = raw.to_str().ok().and_then(Pass::decode);
    let fault = match &pass {
        None => Some("none that reads".to_string()),
        Some(p) if p.person != key => Some("signed by another key than the one dialling".into()),
        Some(p) => p
            .check(&key, &label, unix_now())
            .err()
            .map(|f| f.to_string()),
    };
    if let Some(why) = fault {
        tracing::debug!(target: "door", actor = key, lend = %lend.name, why, "door: refused — a member's own pass that does not open this lend");
        return Err(LendRefusal::Pass.say(
            StatusCode::FORBIDDEN,
            format!("{} is not open to that app here", lend.name),
        ));
    }
    let app = pass.map(|p| p.app);
    tracing::debug!(target: "door", actor = key, lend = %lend.name, app = ?app.as_ref().map(Label::as_str),
        "door: a member's own pass names the app its page is");
    Ok(app)
}

/// One request through the door on the members' protocol.
async fn through(
    State(door): State<Door>,
    dialer: Option<Extension<Dialer>>,
    via: Option<Extension<Via>>,
    req: Request,
) -> Result<Response, Response> {
    let Some(Extension(dialer)) = dialer else {
        tracing::warn!(target: "door", "door: refused — no verified dialer, so the request did not come on an iroh stream");
        return Err(LendRefusal::Closed.say(
            StatusCode::FORBIDDEN,
            "a lend is reached only from another computer of its group",
        ));
    };
    lend_for(&door, Who::Member(dialer), via.map(|Extension(v)| v), req).await
}

/// One request for a lend, from `who`: refused by name, or carried to the
/// lend's service and its answer streamed back.
pub(crate) async fn lend_for(
    door: &Door,
    who: Who,
    via: Option<Via>,
    req: Request,
) -> Result<Response, Response> {
    let actor = match &who {
        Who::Member(dialer) | Who::Holder(dialer) => dialer.actor(),
        Who::Own(_) => hex::encode(door.key),
    };
    let path = req.uri().path().to_string();
    if can_step_out(&path) {
        tracing::debug!(target: "door", actor, path, "door: refused — a path that can step out of its lend");
        return Err(LendRefusal::Path.say(
            StatusCode::BAD_REQUEST,
            "a path that can step outside a lend is refused",
        ));
    }
    let Some((raw_name, service_path)) = split_lend(&path) else {
        tracing::debug!(target: "door", actor, path, "door: refused — no lend named in the path");
        return Err(
            LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here by that name")
        );
    };
    let pending = match &who {
        Who::Member(_) | Who::Own(_) => {
            let Some((group, person)) = door.rail.standing_anywhere(&actor) else {
                tracing::debug!(target: "door", actor, "door: refused — the dialer stands in no group held here");
                return Err(LendRefusal::Closed
                    .say(StatusCode::FORBIDDEN, "this computer lends nothing to you"));
            };
            Pending::Known(Asker {
                actor: PersonKey::resolved(actor.clone()),
                group,
                person,
                app: match &who {
                    Who::Own(app) => Some(app.clone()),
                    _ => None,
                },
            })
        }
        Who::Holder(dialer) => Pending::Pass(*dialer),
    };
    let lend = choices::found(door, raw_name)?;
    let Asker {
        actor: whose,
        group,
        person,
        mut app,
    } = match pending {
        Pending::Known(asker) => asker,
        Pending::Pass(holder) => holder_of(door, &holder, req.headers(), &lend).await?,
    };
    let actor = whose.as_str();
    if let Who::Member(dialer) = &who {
        app = own_app(door, dialer, req.headers(), &lend)?;
    }
    if let Some(refused) = choices::check(door, &lend, &whose, app.as_ref()) {
        return Err(refused);
    }
    if let Some(refused) = lend.refusing(&service_path) {
        tracing::debug!(target: "door", actor, lend = %lend.name, path = service_path, refused = refused.as_str(),
            "door: refused — the lend refuses this path");
        return Err(LendRefusal::Path.say(
            StatusCode::FORBIDDEN,
            format!("{} is lent without {}", lend.name, refused.as_str()),
        ));
    }
    let observed = via.and_then(|v| v.observe());
    if observed.is_none() {
        tracing::debug!(target: "door", actor, lend = %lend.name, "door: no path observed, so the person is taken to be away");
    }
    let whereabouts = limit::guess(observed.and_then(|o| o.addr));
    let Some((verdict, held)) = door.using.ask(&lend, actor, whereabouts).await else {
        tracing::warn!(target: "door", lend = %lend.name, "door: refused — no count of who uses the lend, so none can be let in");
        return Err(LendRefusal::Fault.say(
            StatusCode::INTERNAL_SERVER_ERROR,
            "who is using this lend cannot be counted on the computer that lends it",
        ));
    };
    let (relayed, rtt_ms, sent, received) = observed.map_or((None, None, None, None), |o| {
        (
            Some(o.relayed),
            Some(o.rtt.as_millis()),
            Some(o.sent),
            Some(o.received),
        )
    });
    let share = match verdict {
        Verdict::Full { using } => {
            tracing::info!(target: "door", person = person.as_str(), lend = %lend.name, using, limit = lend.limit,
                "door: refused — as many people as the lend's limit are using it");
            return Err(busy(&lend, using));
        }
        Verdict::Home => {
            tracing::info!(target: "door", person = person.as_str(), lend = %lend.name, addr = ?observed.and_then(|o| o.addr),
                "door: guessed in this computer's home — the path ends at a private address — so not counted and told no share");
            None
        }
        Verdict::In { people, share } => {
            tracing::info!(target: "door", person = person.as_str(), lend = %lend.name, people, share_bps = ?share,
                declared_bps = ?lend.upload.map(lend::Upload::bps), relayed = ?relayed, rtt_ms = ?rtt_ms,
                sent = ?sent, received = ?received, "door: let in, and told their share of the declared upload");
            if share.is_none() {
                tracing::debug!(target: "door", lend = %lend.name, "door: the lend declares no upload, so no share is told");
            }
            share
        }
    };
    tracing::debug!(target: "door", actor, person = person.as_str(), group, lend = %lend.name, app = ?app.as_ref().map(Label::as_str),
        method = %req.method(), path = service_path, "door: carrying a request to the lend's service, the app the pass's or none");
    let told = Told { share, held };
    forward(
        door,
        &lend,
        (&whose, &person, app.as_ref()),
        service_path,
        req,
        told,
    )
    .await
}

/// What the door decided for a request let in: the share it is told, and
/// the use it holds until its answer is whole or abandoned (none for a
/// person guessed home, who is not counted).
struct Told {
    share: Option<u64>,
    held: Option<InFlight>,
}

/// The refusal of a newcomer past `lend`'s limit, `using` people counted:
/// the reason, in a person's words, and the figures, for an app to turn
/// into a choice (DESIGN §8, "Too many at once").
fn busy(lend: &Lend, using: u32) -> Response {
    let people = |n: u32| if n == 1 { "person" } else { "people" };
    let limit = lend.limit.get();
    let reason = format!(
        "{} is lent to {limit} {} at once, and {using} {} using it now: try again when one stops",
        lend.name,
        people(u32::from(limit)),
        if using == 1 { "is" } else { "are" },
    );
    let body = serde_json::json!({ "error": reason, "using": using, "limit": limit });
    LendRefusal::Full.said(StatusCode::SERVICE_UNAVAILABLE, body)
}

/// Send `request` on a new HTTP/1 connection to `to`, driven with
/// upgrades, and answer the response; `None`, named at warn by `lend`,
/// when nothing there takes the connection in [`CONNECT_WITHIN`] or
/// answers.
pub(crate) async fn send_to(
    to: std::net::SocketAddr,
    request: axum::http::Request<Body>,
    lend: &str,
) -> Option<axum::http::Response<hyper::body::Incoming>> {
    let tcp = match tokio::time::timeout(CONNECT_WITHIN, tokio::net::TcpStream::connect(to)).await {
        Ok(Ok(tcp)) => tcp,
        Ok(Err(e)) => {
            tracing::warn!(target: "door", lend, %to, error = %e, "door: the lend's service refused a connection");
            return None;
        }
        Err(_) => {
            tracing::warn!(target: "door", lend, %to, within = ?CONNECT_WITHIN, "door: the lend's service took no connection in time");
            return None;
        }
    };
    let (mut sender, conn) = match hyper::client::conn::http1::handshake(
        hyper_util::rt::TokioIo::new(tcp),
    )
    .await
    {
        Ok(pair) => pair,
        Err(e) => {
            tracing::warn!(target: "door", lend, error = %e, "door: no HTTP/1 connection to the lend's service");
            return None;
        }
    };
    // Drives the connection until the response body is read to its end.
    let name = lend.to_string();
    tokio::spawn(async move {
        if let Err(e) = conn.with_upgrades().await {
            tracing::debug!(target: "door", lend = %name, error = %e, "door: the connection to the lend's service ended in error");
        }
    });
    match sender.send_request(request).await {
        Ok(a) => Some(a),
        Err(e) => {
            tracing::warn!(target: "door", lend, error = %e, "door: the lend's service did not answer");
            None
        }
    }
}

/// One request's carriage, traced at debug under `carriage` from asked to
/// its end (u5-11): its `Range`, the answer's status, and whether the
/// answer went whole, was cut off when the asker stopped reading it, or
/// the asker left before the service answered — what a seek in a lent
/// player decides, which no other line shows.
struct Carriage {
    lend: String,
    range: Option<String>,
    began: Instant,
    answered: Option<(u16, Option<u64>)>,
    sent: u64,
}

impl Drop for Carriage {
    fn drop(&mut self) {
        let (lend, range, sent) = (&self.lend, &self.range, self.sent);
        let ms = self.began.elapsed().as_millis();
        match self.answered {
            None => tracing::debug!(target: "carriage", lend, ?range, ms,
                "door: no answer carried — the asker left before the service answered, or a line above says why"),
            Some((status, Some(length))) if sent < length => {
                tracing::debug!(target: "carriage", lend, ?range, status, sent, length, ms,
                "door: the answer cut off — the asker stopped reading it")
            }
            Some((status, length)) => {
                tracing::debug!(target: "carriage", lend, ?range, status, sent, ?length, ms, "door: the answer sent to its end")
            }
        }
    }
}

/// Carry `req` to `lend`'s service as `path` (with its query), the head
/// rewritten, and stream the answer back.
async fn forward(
    door: &Door,
    lend: &Lend,
    (whose, person, app): (&PersonKey, &Person, Option<&Label>),
    path: String,
    req: Request,
    Told { share, held }: Told,
) -> Result<Response, Response> {
    let actor = whose.as_str();
    let mut req = req;
    let range = req.headers().get(header::RANGE);
    let mut carriage = Carriage {
        lend: lend.name.to_string(),
        range: range.and_then(|r| r.to_str().ok()).map(str::to_string),
        began: now(),
        answered: None,
        sent: 0,
    };
    tracing::debug!(target: "carriage", actor, lend = %lend.name, range = ?carriage.range, "door: asked");
    let upgrade = upgrade_of(req.headers());
    let asker_side = upgrade.as_ref().map(|_| hyper::upgrade::on(&mut req));
    let (mut parts, body) = req.into_parts();
    let get = parts.method == axum::http::Method::GET;
    let (Ok(person_value), Ok(key_value)) = (
        HeaderValue::from_bytes(person.as_str().as_bytes()),
        HeaderValue::from_str(actor),
    ) else {
        tracing::warn!(target: "door", actor, "door: refused — the person's name cannot be a header");
        return Err(LendRefusal::Fault.say(
            StatusCode::INTERNAL_SERVER_ERROR,
            "who is asking cannot be told to this lend",
        ));
    };
    let identity = [
        (HeaderName::from_static(PERSON_HEADER), person_value),
        (HeaderName::from_static(KEY_HEADER), key_value),
    ];
    let signing = match lend.sign_in {
        None => Signing::Secret(secret_headers(door, &lend.name)),
        Some(port) => match door
            .sign_ins
            .of(&lend.name, port, whose, identity.clone())
            .await
        {
            Ok(signed) => Signing::Person(port, signed),
            Err(why) => {
                tracing::warn!(target: "door", actor, lend = %lend.name, %why,
                    "door: refused — the lend's sign-in did not sign the person in, so the request is not carried");
                return Err(LendRefusal::SignIn.say(
                    StatusCode::BAD_GATEWAY,
                    format!("{} could not sign you in", lend.name),
                ));
            }
        },
    };
    let (target, secret) = signing.inward(&mut parts.headers, path, parts.uri.query());
    let displaced = rewrite_headers(&mut parts.headers, identity, &secret);
    if let Some(app) = app {
        let app = HeaderValue::from_str(app.as_str()).expect("a label is base32, so a header");
        parts
            .headers
            .insert(HeaderName::from_static(APP_HEADER), app);
    }
    if displaced > 0 {
        tracing::info!(target: "door", actor, lend = %lend.name, displaced,
            "door: dropped headers the client typed in the door's or the secret's name before setting them");
    }
    parts.headers.insert(header::HOST, own_address(lend));
    own_origin(
        &mut parts.headers,
        &Label::of(&door.key, lend.name.as_str()),
        lend,
    );
    if let Some(protocol) = upgrade.clone() {
        tracing::debug!(target: "door", actor, lend = %lend.name, ?protocol, "door: the request asks to switch protocols, carried");
        keep_upgrade(&mut parts.headers, protocol);
    }
    let mut request = match axum::http::Request::builder()
        .method(parts.method)
        .uri(&target)
        .version(Version::HTTP_11)
        .body(body)
    {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!(target: "door", actor, error = %e, "door: refused — the request target is not a path");
            return Err(LendRefusal::Path.say(
                StatusCode::BAD_REQUEST,
                "the request is not one a lend can be asked",
            ));
        }
    };
    if let Some(share) = share {
        parts.headers.insert(
            HeaderName::from_static(SHARE_HEADER),
            HeaderValue::from(share),
        );
    }
    *request.headers_mut() = parts.headers;
    // The use began when the person was let in, and lasts until the member
    // has the whole answer or leaves it: the answer's body holds it.

    let unanswered = || {
        LendRefusal::Away.say(
            StatusCode::BAD_GATEWAY,
            format!(
                "{} is not answering on the computer that lends it",
                lend.name
            ),
        )
    };
    let origin = std::net::SocketAddr::from(([127, 0, 0, 1], lend.port.get()));
    let Some(answer) = send_to(origin, request, lend.name.as_str()).await else {
        let unanswered = unanswered();
        carriage.answered = Some((unanswered.status().as_u16(), None));
        return Err(unanswered);
    };
    let mut answer = answer;
    let switched =
        upgrade_of(answer.headers()).filter(|_| answer.status() == StatusCode::SWITCHING_PROTOCOLS);
    let service_side = switched.as_ref().map(|_| hyper::upgrade::on(&mut answer));
    let (mut parts, body) = answer.into_parts();
    strip_hop_by_hop(&mut parts.headers);
    let held = match (asker_side, service_side, switched) {
        (Some(asker), Some(service), Some(protocol)) => {
            tracing::debug!(target: "door", actor, lend = %lend.name, ?protocol, "door: the service switched protocols; splicing the two");
            keep_upgrade(&mut parts.headers, protocol);
            splice(asker, service, held, lend.name.to_string());
            None
        }
        _ => held,
    };
    let own = parts
        .headers
        .get(header::LOCATION)
        .and_then(|l| l.to_str().ok())
        .and_then(|l| as_path(l, lend));
    if let Some(path) = own.and_then(|p| HeaderValue::from_str(&p).ok()) {
        tracing::debug!(target: "door", lend = %lend.name, ?path, "door: a redirect to the service's own address, answered as a path");
        parts.headers.insert(header::LOCATION, path);
    }
    // The share on the answer is the door's, and so is a refusal's mark: a
    // service's own of either is displaced.
    parts.headers.remove(SHARE_HEADER);
    parts.headers.remove(REFUSED_HEADER);
    if let Some(share) = share {
        parts.headers.insert(
            HeaderName::from_static(SHARE_HEADER),
            HeaderValue::from(share),
        );
    }
    tracing::debug!(target: "door", actor, lend = %lend.name, status = parts.status.as_u16(),
        "door: the lend's service answered; streaming it back");
    let length = parts
        .headers
        .get(header::CONTENT_LENGTH)
        .and_then(|l| l.to_str().ok()?.parse().ok());
    let swapping = signing.outward(&mut parts, get, &door.sign_ins, (&lend.name, whose))?;
    carriage.answered = Some((parts.status.as_u16(), length));
    tracing::debug!(target: "carriage", lend = %lend.name, range = ?carriage.range, status = parts.status.as_u16(),
        content_range = ?parts.headers.get(header::CONTENT_RANGE), ms = carriage.began.elapsed().as_millis(), "door: answered");
    let body = body.map_frame(move |frame| {
        let _ = &held;
        let carried = &mut carriage;
        carried.sent += frame.data_ref().map_or(0, |d| d.len() as u64);
        frame
    });
    let body = match swapping {
        Some((secrets, script)) => Body::new(sign_in::Swapping::new(body, secrets, script)),
        None => Body::new(body),
    };
    let mut response = Response::new(body);
    *response.status_mut() = parts.status;
    *response.headers_mut() = parts.headers;
    Ok(response.into_response())
}

#[cfg(test)]
mod tests;
