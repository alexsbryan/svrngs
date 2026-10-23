// SPDX-License-Identifier: AGPL-3.0-or-later
//! The member door (DESIGN §3; PLAN §4, U3's inventory): a route on the members'
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
//! request before the door sees it (PLAN §4, U1's inventory).
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
//! **How many at once, and each one's share** (u3-6; DESIGN §3, "Many
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
//! **What it lends, asked** (u3-5; DESIGN §3, "Lends are asked, not
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
//! computer's key. A pass a guest's visit key signed lets its holder in as
//! that guest, their host's, never as the host ([`guest_of`], guests-door).
//!
//! **What the door refuses**, by name: a request not on an iroh stream; a
//! dialer standing in no group this computer holds; a path that can step
//! out of its lend — a `..` segment, an encoded `.` or `/` — as
//! `route_by_prefix` refuses one (`iroh_routed_forward.rs:318-337`); a lend
//! this computer does not declare; a path the lend refuses
//! ([`crate::lend::RefusedPath::covers`]); and a newcomer past the lend's
//! limit. Each refusal, and the answer for a service that does not answer,
//! is marked as svrngs's, which ([`LendRefusal`]); the service's own answers never are.
//! A browser's tab asking for a page is shown svrngs's own answer as one ([`paged`]).

use std::num::NonZeroU16;
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
use crate::lend::{self, Lend, LendName, Offered, Source};
use crate::network::Whereabouts;
use crate::ring_routes::err;
use crate::{register::Visiting, Person, RingRail};
use limit::{Counts, Verdict};
use svrngs_core::pass::{Label, Pass};
use svrngs_core::token::Group;

pub mod choices;
pub mod limit;

/// svrngs's own header prefix: every header under it on a request through
/// the door is the door's. A client's is stripped whole.
pub const IDENTITY_PREFIX: &str = "x-svrngs-";

/// The name to show for the person asking, as the register names them:
/// their name in the first group this computer holds, in the groups'
/// order, that they stand in. A name is a claim (bash-person): two people
/// may share one, so nothing is keyed on it — an account is
/// [`USER_HEADER`]'s.
pub const PERSON_HEADER: &str = "x-svrngs-person";

/// The person asking, by their first key in the group, as the record
/// spells an actor: its lowercase hex. The same from every device of
/// theirs (bash-devices), so an app keys a person on it; an id, not a key
/// to seal to — `/v1/rail/people` lists every key of each person.
pub const KEY_HEADER: &str = "x-svrngs-key";

/// The device asking: the key it stands in the group by, as the record
/// spells an actor — one of the keys `/v1/rail/people` lists for the
/// person, on the device they joined first the same as [`KEY_HEADER`]
/// (bash-two-at-once). An app that holds something to a device, as
/// Jellyfin holds a session to the device a token was signed in under,
/// keys it on this; what is a person's, on [`KEY_HEADER`].
pub const DEVICE_HEADER: &str = "x-svrngs-device";

/// The name of the account an app makes for the person asking:
/// [`user_name`]'s, unique by their key.
pub const USER_HEADER: &str = "x-svrngs-user";

/// What the door tells a lend of who asks.
pub type Identity = [(HeaderName, HeaderValue); 4];

/// The account name for `person` asking with `key`: their name kept to
/// ASCII letters, digits, `-` and `.`, or `person` if none is left, then
/// `-` and the first 16 hex of the key. So two people of one name are two
/// accounts, an app's own accounts (its lender's `ana`) are never a
/// person's, and the same key finds the same account after any restart.
/// The characters are ones Jellyfin, Navidrome and Calibre-Web each take
/// in a user name and none reads as a pattern (`_` and `%` are SQL `LIKE`'s).
pub fn user_name(person: &Person, key: &str) -> String {
    let kept: String = (person.as_str().chars())
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
        .collect();
    let kept = if kept.is_empty() { "person" } else { &kept };
    format!("{kept}-{}", &key[..key.len().min(16)])
}

/// The name and account the door tells a lend about `person` asking with
/// `actor`: a guest as their host's guest, with an account of their own
/// ([`user_name`], then `.guest`) (GUESTS §3.5). One string beside the
/// headers and the sign-in with svrngs' claims.
fn told_as(person: &Person, host: Option<&Person>, actor: &str) -> (String, String) {
    let told = host.map_or(person.to_string(), |h| format!("{person} ({h}'s guest)"));
    let user = user_name(person, actor) + if host.is_some() { ".guest" } else { "" };
    (told, user)
}

/// The app asking: the label of the pass's `app`, the lend whose pages
/// hold it, under the person's signature (sup-1) — the lend the person's
/// page opened (`web/app.js`, `open`), or a computer's page as its origin
/// names it ([`crate::origin`]). Absent with no pass and no app named, so a
/// service reads its absence as no app named (su-1, the scoped-use spike).
pub const APP_HEADER: &str = "x-svrngs-app";

/// The lend's own label (events-links): the first name of its origin on a
/// computer, and the `<label>` of `<page>#open=<label>&at=<path>`, the
/// address at which a group's page opens the lend at a path on a phone
/// (`web/app.js`), so an app writes a link into itself.
pub const LABEL_HEADER: &str = "x-svrngs-label";

/// The page of each group held here that the device asking stands in,
/// `<group>=<page>` apart by a space: the `<page>` of [`LABEL_HEADER`]'s
/// address, the group's kept page ([`crate::invite::kept_page`]), `none`
/// when it keeps none and `unread` when that cannot be read. Every group,
/// as no request says which it is for: a lend is lent to every group held
/// here, and a computer's key stands in each of them; a phone's, in one.
pub const PAGES_HEADER: &str = "x-svrngs-pages";

/// A person's share of the upload a lend declares, in bits a second, on
/// each request through the door and its answer: a declared figure, as the
/// name says (DESIGN §3, "The upload is declared"), never a measured one.
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

/// Said to a computer in none of this one's groups, by the door and the
/// node alike, in the asker's words: this computer is the one they reached.
pub const LENDS_NOTHING: &str = "the computer you reached lends nothing to you";

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
    /// The computer that lends it cannot read its own state.
    Fault,
    /// The lend asks, and its person said no yes to the app asking (sup-2).
    Unasked,
    /// The lend asks, and its person stopped the app asking.
    Stopped,
    /// The asker was removed from the group (U7; DESIGN §6).
    Removed,
    /// The asker's visit has ended: at its time, by its host's Stop, or as
    /// its host left (guests-door).
    Ended,
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
            LendRefusal::Fault => "fault",
            LendRefusal::Unasked => "unasked",
            LendRefusal::Stopped => "stopped",
            LendRefusal::Removed => "removed",
            LendRefusal::Ended => "ended",
        }
    }

    /// svrngs's answer, `status` and `words`, marked as its own.
    pub(crate) fn say(self, status: StatusCode, words: impl Into<String>) -> Response {
        self.said(status, serde_json::json!({ "error": words.into() }))
    }

    /// svrngs's answer, `status` and `body`, its kind in the body and the
    /// header, its words kept for [`paged`].
    fn said(self, status: StatusCode, mut body: serde_json::Value) -> Response {
        body["kind"] = self.as_str().into();
        let words = Words(body["error"].as_str().unwrap_or_default().to_string());
        let mut answer = (status, Json(body)).into_response();
        answer.extensions_mut().insert(words);
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

/// The words of svrngs's own answer, kept on it for [`paged`].
#[derive(Clone)]
struct Words(String);

/// Whose computer did not answer, kept on an answer `away` for [`paged`],
/// which draws their window on the door, dark: home/away.html.
#[derive(Clone)]
pub(crate) struct Dark(pub(crate) Person);
const AWAY: &str = include_str!("home/away.html");

/// svrngs's answer drawn as a page when a browser's tab asked for one, by
/// `Accept`; a program asking anything else is given its JSON (v1ui-polish
/// (a)): the routes a person's tab reaches a lend by are layered by it.
pub(crate) async fn paged(req: Request, next: axum::middleware::Next) -> Response {
    let accept = req.headers().get(header::ACCEPT).map(HeaderValue::as_bytes);
    let tab = accept.is_some_and(|a| String::from_utf8_lossy(a).contains("text/html"));
    let mut answer = next.run(req).await;
    let dark = answer.extensions_mut().remove::<Dark>();
    let Some(Words(words)) = answer.extensions_mut().remove::<Words>().filter(|_| tab) else {
        return answer;
    };
    tracing::debug!(target: "door", words, dark = ?dark.as_ref().map(|Dark(p)| p.as_str()), "door: svrngs's own answer to a browser's tab, drawn as a page: whose window is dark, when a computer did not answer");
    // The words, a sentence begun lower-case, are drawn as one, with nothing in them markup.
    let words = words.trim_end_matches('.').replace('<', "&lt;");
    let page = match dark {
        Some(Dark(p)) => AWAY.replace("{who}", &p.as_str().replace('<', "&lt;")).replace("{what}", "this"),
        None => format!("<!doctype html><meta charset=utf-8><meta name=viewport content=width=device-width><style>p::first-letter{{text-transform:uppercase}}</style><p>{words}."),
    };
    let (mut parts, _) = answer.into_parts();
    parts.headers.remove(header::CONTENT_TYPE);
    (parts, axum::response::Html(page)).into_response()
}

/// svrngs's answer to someone removed from the group (U7), in their words.
pub(crate) fn gone() -> Response {
    LendRefusal::Removed.say(
        StatusCode::FORBIDDEN,
        "You are no longer in this group, so nothing of it is open to you.",
    )
}

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
    /// Each person's yes and stop to an app, written through one task.
    pub choices: choices::Choices,
    /// The groups' folders, which carry apps ([`Source::Carried`]).
    pub folders: crate::folder::Folders,
    /// The lender's sign-in key and the codes this run minted, shared with
    /// the provider on the record API ([`crate::oidc`]): one key, one code
    /// store, whichever channel asks.
    pub(crate) oidc: crate::oidc::Provider,
}

/// The door's routes, for the members' protocol's router: a lend, every
/// method, and the list of what this computer lends.
pub fn router(door: Door) -> axum::Router {
    axum::Router::new()
        .route("/lend/{*rest}", any(through))
        .route(LENDS_ROUTE, get(lends))
        .route(CHOICES, get(choices::choices_of).post(choices::choose))
        .layer(axum::middleware::from_fn(paged))
        .with_state(door)
}

/// The door's route on the pass holders' protocol ([`crate::alpn::Alpn::Lend`]):
/// a lend, every method, and nothing else.
pub fn holders_router(door: Door) -> axum::Router {
    axum::Router::new()
        .route("/lend/{*rest}", any(through_pass))
        .route(CHOICES, axum::routing::post(choices::stop_with_pass))
        .layer(axum::middleware::from_fn(paged))
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
            "you can ask what the computer you reached lends only from a computer in its group",
        ));
    };
    let actor = dialer.actor();
    if door.rail.standing_anywhere(&actor).is_none() {
        tracing::debug!(target: "door", actor, "door: lends refused — the dialer stands in no group held here");
        return Err(err(StatusCode::FORBIDDEN, LENDS_NOTHING));
    }
    let mut offered = offered(&door).await?;
    let stands = |g: &String| door.rail.standing(g).is_ok_and(|s| s.names(&actor));
    offered.retain(|o| o.listed(stands));
    tracing::debug!(target: "door", actor, lends = offered.len(), "door: lends — answered what this computer lends, each carried app only to its own group");
    let version = crate::version::VERSION;
    Ok(Json(serde_json::json!({ "lends": offered, "version": version })).into_response())
}

/// Every lend this computer declares now, and every app its groups'
/// folders carry here, each with how many people use it ([`Using`]); or,
/// when the declarations cannot be read, the refusal
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
                "the computer you reached cannot read what it lends",
            ));
        }
    };
    let mut offered = Vec::new();
    let carried = door.folders.carried().into_iter().map(Ok);
    for lend in declared.into_iter().chain(carried) {
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
            group: match lend.source {
                Source::Carried(group) => Some(group),
                Source::Port(_) => None,
            },
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
    let named: Vec<String> = connection(headers).collect();
    remove_counted(headers, |name| {
        HOP_BY_HOP.contains(&name) || named.iter().any(|n| n == name)
    })
}

/// Remove every header whose name, as [`compare_name`] reads it, `goes`;
/// answer how many values went.
fn remove_counted(headers: &mut HeaderMap, goes: impl Fn(&str) -> bool) -> usize {
    let gone: Vec<HeaderName> = headers
        .keys()
        .filter(|n| goes(&compare_name(n.as_str())))
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
    let named = connection(headers).any(|t| t == "upgrade");
    named.then(|| headers.get(header::UPGRADE).cloned())?
}

/// Each name a message's `Connection` lists, as [`compare_name`] reads it.
fn connection(headers: &HeaderMap) -> impl Iterator<Item = String> + '_ {
    let values = headers.get_all(header::CONNECTION).iter();
    let tokens = values.filter_map(|v| v.to_str().ok());
    tokens.flat_map(|v| v.split(',')).map(compare_name)
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
    identity: Identity,
    secret: &[(HeaderName, HeaderValue)],
) -> usize {
    strip_hop_by_hop(headers);
    let declared: Vec<String> = secret
        .iter()
        .map(|(n, _)| compare_name(n.as_str()))
        .collect();
    let displaced = remove_counted(headers, |name| {
        name.starts_with(IDENTITY_PREFIX) || declared.iter().any(|d| d == name)
    });
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
/// its lender on that computer; and [`LendPath`] turns a redirect to this
/// address back into a path the browser reads on the lend's origin. Until
/// u5-2 the door passed the browser's `Host`, so the service built its
/// links for the address the browser used, one `<lender's bridge>/lend/<name>/`
/// for every lend.
fn own_address(port: NonZeroU16) -> HeaderValue {
    HeaderValue::from_str(&format!("localhost:{port}")).expect("a host and a port are a header")
}

/// The scheme the browser reached a lend with, from the `Host` it asked:
/// `http` for a `*.localhost` host or an address, `https` for any other
/// name. svrngs serves a lend itself only in plain http: at
/// `<label>.localhost` on a computer ([`crate::origin`]) and at the room's
/// `http://<private address>:<port>/`, the device's `Host` passed through
/// ([`crate::room`]). https reaches a lend only at a page host, a name
/// (`web/store.js`, `lendOrigin`). It is what the door tells a service as
/// `X-Forwarded-Proto` ([`tell_origin`]) and what the front channel reads
/// back as the lend's own origin (oidc-jellyfin).
pub(crate) fn scheme_of(host: &str) -> &'static str {
    // The host without its `:<port>`, which an IPv6 address's last colon,
    // inside its `[]`, never is.
    let unported = host.trim_end_matches(|c: char| c.is_ascii_digit());
    let name = unported.strip_suffix(':').unwrap_or(host);
    let address = name.trim_matches(['[', ']']).parse::<std::net::IpAddr>();
    if address.is_ok() || name.to_ascii_lowercase().ends_with(".localhost") {
        "http"
    } else {
        "https"
    }
}

/// Tell the service the origin the browser used, as a reverse proxy does
/// (oidc-jellyfin): `X-Forwarded-Host` is the `Host` the browser asked and
/// `X-Forwarded-Proto` its scheme ([`scheme_of`]). Any of either a client
/// typed is displaced, so only the door's reach the service. The service is
/// still asked at its own address ([`own_address`]); this is the one fact
/// it may need to build a page on the lend's own origin, as the Jellyfin
/// SSO plugin's callback does when `KnownProxies` names this computer
/// (`demo/runs/oidc/2026-10-21-jellyfin-2/`).
fn tell_origin(headers: &mut HeaderMap) {
    let (forwarded_host, forwarded_proto) = (
        HeaderName::from_static("x-forwarded-host"),
        HeaderName::from_static("x-forwarded-proto"),
    );
    headers.remove(&forwarded_host);
    headers.remove(&forwarded_proto);
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .map(str::to_owned)
    else {
        tracing::debug!(target: "door", "door: the browser's Host cannot be read, so the service is told no forwarded origin");
        return;
    };
    let Ok(value) = HeaderValue::from_str(&host) else {
        return;
    };
    let scheme = scheme_of(&host);
    tracing::debug!(target: "door", forwarded_host = %host, forwarded_proto = scheme,
        "door: the browser's origin, told to the service as a proxy's forwarded headers");
    headers.insert(forwarded_host, value);
    headers.insert(forwarded_proto, HeaderValue::from_static(scheme));
}

/// An `Origin` that is the lend's own — its first name the lend's label, as
/// every lend's is (DESIGN §7) — told to the service
/// as its own address, as the Host is ([`own_address`]), so an app that
/// takes a post only from its own address, as Django's origin check does
/// (Paperless-ngx; u5-catalogue-dry), takes the lend's own pages' posts.
/// Any other origin is left as it is, so the service still refuses a post
/// from a page that is not the lend's.
fn own_origin(headers: &mut HeaderMap, label: &Label, (lend, port): (&Lend, NonZeroU16)) {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|o| o.to_str().ok()) else {
        return;
    };
    let host = origin.split_once("://").map_or("", |(_, rest)| rest);
    let first = host.split(['.', ':']).next().unwrap_or_default();
    if !first.eq_ignore_ascii_case(&label.to_string()) {
        tracing::debug!(target: "door", lend = %lend.name, origin, "door: the request's origin is not the lend's own; left as it is");
        return;
    }
    let own = HeaderValue::from_bytes(&[b"http://", own_address(port).as_bytes()].concat())
        .expect("a host and a port are a header");
    tracing::debug!(target: "door", lend = %lend.name, "door: the request's origin is the lend's own; told to the service as its own address");
    headers.insert(header::ORIGIN, own);
}

/// The hosts a service answers at on its own computer: the one set by which
/// a URL on the service's own address is read as a path on the lend
/// ([`LendPath`]).
const OWN_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// A path on a lend's own origin: what is left of a URL on the service's
/// own address (`http://<one of OWN_HOSTS>:<port>`) or on the browser's
/// origin for the lend once that is taken off, case ignored, `/` for
/// nothing. Parsed once, so a `Location` built from it is read on whatever
/// origin the browser reached the lend at, and never as another host's: it
/// starts with `/`, `?` or `#` but not `//`, and holds no `\`, which a
/// browser reads as `/`. It holds no control byte either, so it is a
/// header. The door's answer to a service's redirect ([`lend_for`]) and the
/// front channel's to an app's ([`crate::oidc::authorize`]) are built from
/// one.
pub(crate) struct LendPath(String);

impl LendPath {
    /// `url` as a path on the lend whose service is at `port`, its browser
    /// at the origin `browser`; else why not, worded to follow the URL's
    /// name, as the front channel says it: "the redirect_uri {why}".
    pub(crate) fn parse(
        url: &str,
        port: NonZeroU16,
        browser: Option<&str>,
    ) -> Result<Self, &'static str> {
        let own = OWN_HOSTS.map(|host| format!("http://{host}:{port}"));
        let rest = (own.iter().map(String::as_str).chain(browser))
            .find_map(|origin| {
                let rest = url
                    .get(..origin.len())?
                    .eq_ignore_ascii_case(origin)
                    .then(|| &url[origin.len()..])?;
                (rest.is_empty() || rest.starts_with(['/', '?', '#'])).then_some(rest)
            })
            .ok_or("is not on this lend's own address")?;
        if rest.starts_with("//") || rest.contains('\\') {
            return Err("would send you to another address");
        }
        if rest.contains(|c: char| c.is_ascii_control()) {
            return Err("holds a character no address holds");
        }
        Ok(Self(if rest.is_empty() { "/" } else { rest }.into()))
    }

    /// The `Location` that sends the browser here, `query`'s pairs added
    /// form-encoded. It is a header by construction: the path holds no
    /// control byte, the only bytes a header refuses, and form-encoding
    /// writes none.
    pub(crate) fn location<'a>(
        mut self,
        query: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> HeaderValue {
        let pairs = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(query)
            .finish();
        if !pairs.is_empty() {
            self.0.push(if self.0.contains('?') { '&' } else { '?' });
            self.0.push_str(&pairs);
        }
        HeaderValue::from_str(&self.0)
            .expect("a path with no control byte, and form-encoded pairs, are a header")
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
/// §4): their first key in the group the register finds the key in, the
/// same from every device of theirs (bash-devices), whether a member's
/// dialler or a pass's signer asks. Made only here ([`PersonKey::resolved`]),
/// so nothing kept about a person — a lend's count, a choice — is keyed by
/// the raw key a request came with.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct PersonKey(String);

impl PersonKey {
    /// The group `key`, actor hex, stands in here, and who they are there:
    /// their name and person; `None` when they stand in no group held here.
    fn resolved(door: &Door, key: &str) -> Option<(String, Person, Self)> {
        let (group, member) = door.rail.standing_anywhere(key)?;
        tracing::debug!(target: "door", key, person_key = member.person_key, "door: the person the key is");
        Some((group, member.person, Self(member.person_key)))
    }
}
svrngs_core::text_newtype!(PersonKey);

/// The device a request for a lend comes from ([`DEVICE_HEADER`]): the key
/// that stands in a group held here — a member's dialler's, a pass's
/// signer's, this computer's at its own origin; told to the lend so an app
/// may hold what it signs in to one
/// (demo/runs/two-at-once/2026-10-12/README.md, finding 1).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceKey(String);
svrngs_core::text_newtype!(DeviceKey);

/// The person a request for a lend is carried as.
struct Asker {
    actor: PersonKey,
    device: DeviceKey,
    group: String,
    person: Person,
    /// The pass's app; none for a key of the group's with no pass.
    app: Option<Label>,
    /// Whose guest they are, for a visit's key ([`guest_of`]); none for a
    /// person of the group.
    host: Option<Person>,
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
    let dialer = dialed(dialer, "you can open this only through your group")?;
    lend_for(&door, Who::Holder(dialer), via.map(|Extension(v)| v), req).await
}

/// The key a request was dialled with, which only an iroh stream carries;
/// without one it is refused in `words`, the protocol's own.
fn dialed(dialer: Option<Extension<Dialer>>, words: &str) -> Result<Dialer, Response> {
    dialer.map(|Extension(d)| d).ok_or_else(|| {
        tracing::warn!(target: "door", "door: refused — no verified dialer, so the request did not come on an iroh stream");
        LendRefusal::Closed.say(StatusCode::FORBIDDEN, words)
    })
}

/// The lend `raw` names here: by its name, or by its label when that names
/// this computer's key (u5-2). `None` when this computer lends none so.
fn find(door: &Door, raw: &str) -> Result<Option<Lend>, lend::LendError> {
    let carried = || door.folders.carried().into_iter();
    if let Some(label) = Label::parse(raw).filter(|l| l.lender() == door.key) {
        let found = (lend::all(&door.data_dir)?
            .into_iter()
            .filter_map(Result::ok))
        .chain(carried())
        .find(|l| Label::of(&door.key, l.name.as_str()) == label);
        tracing::debug!(target: "door", label = raw, found = ?found.as_ref().map(|l| l.name.to_string()), "door: a lend named by its label");
        return Ok(found);
    }
    match LendName::parse(raw) {
        Ok(name) => {
            Ok(lend::read(&door.data_dir, &name)?.or_else(|| carried().find(|l| l.name == name)))
        }
        Err(_) => Ok(None),
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "the door's clock: whether a pass has lapsed, and until when a computer's own lasts"
)]
pub(crate) fn unix_now() -> Option<i64> {
    unix_secs(std::time::SystemTime::now())
}

/// `t` in Unix seconds; `None`, said at warn, for a time before 1970: a
/// clock that cannot be read, never read as 0, which opened every lapsed
/// pass (`0 >= until` is false; #203).
fn unix_secs(t: std::time::SystemTime) -> Option<i64> {
    let secs = t.duration_since(std::time::UNIX_EPOCH).ok();
    if secs.is_none() {
        tracing::warn!(target: "door", "door: this computer's clock reads before 1970, so nothing asking the time is judged");
    }
    secs.map(|d| d.as_secs() as i64)
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
    let Some(now) = unix_now() else {
        return Err(refused("this computer's clock cannot be read"));
    };
    if let Err(fault) = pass.check(&key, &label, now) {
        return Err(refused(&fault.to_string()));
    }
    let mut standing = PersonKey::resolved(door, &pass.person);
    // A key the keeper cut never stands again (U7), so it is told so at once
    // rather than after the ring is asked.
    if standing.is_none() && door.rail.removed_anywhere(&pass.person).is_some() {
        tracing::debug!(target: "door", holder = key, signer = pass.person, lend = %lend.name,
            "door: refused — the pass's signer was removed from a group held here");
        return Err(gone());
    }
    if standing.is_none() {
        if let Some(visit) = door.rail.visit_anywhere(&pass.person) {
            return guest_of(&hex::encode(door.key), visit, pass, lend, now);
        }
        tracing::debug!(target: "door", holder = key, signer = pass.person, lend = %lend.name,
            "door: the pass's signer stands in no group held here, so the group's ring is asked once first");
        let known = || door.rail.standing_anywhere(&pass.person).is_some();
        let in_time = door.ring.until(ASK_WITHIN, known).await;
        standing = PersonKey::resolved(door, &pass.person);
        tracing::debug!(target: "door", signer = pass.person, in_time, known = standing.is_some(),
            "door: the ring asked — whether a member that knows the signer answered, or a round ended, in time, and whether the signer stands now");
    }
    let Some((group, person, actor)) = standing else {
        return Err(refused("its signer stands in no group held here"));
    };
    tracing::debug!(target: "door", holder = key, person = person.as_str(), lend = %lend.name, until = pass.until,
        "door: a pass opens this lend, so its holder asks as the person who signed it");
    Ok(Asker {
        app: Some(pass.app),
        device: DeviceKey(pass.person),
        actor,
        group,
        person,
        host: None,
    })
}

/// The guest whose visit's key signed `pass` (guests-door; GUESTS §3.4-§3.6,
/// §5 "The door"): let in as themselves, their host's guest, only while the
/// fold says the visit is live ([`svrngs_core::Guest::live`]) and it names
/// this lend, its host's, this computer's own, and never a carried app, the
/// group's. Each refusal says which of those refused it: `ended` for the
/// visit's end, `closed` for the rest. A visit begun later than now, which
/// only this computer's clock stepping back makes, is said as one ended.
fn guest_of(me: &str, v: Visiting, pass: Pass, lend: &Lend, now: i64) -> Result<Asker, Response> {
    let ((group, register, guest), name) = (v, lend.name.as_str());
    let host = register.members.get(&guest.host).map(|m| &m.person);
    let Some(host) = host.or(register.removed.get(&guest.host)).cloned() else {
        tracing::debug!(target: "door", guest = pass.person, host = guest.host, "door: refused — a visit whose host is bound to nobody here");
        return Err(LendRefusal::Closed.say(StatusCode::FORBIDDEN, "nobody here hosts your visit"));
    };
    let live = guest.live(&register.cut, now);
    let gone = register.cut.get(&guest.host).is_some_and(|c| c.at <= now);
    let stopped = guest.stopped.is_some_and(|s| s <= now);
    // A carried app is its group's, never its host's own.
    let own = guest.host == me && matches!(lend.source, Source::Port(_));
    let named = guest.visit.opens.get(me).is_some_and(|l| l.contains(name));
    let ended = |w| Some((LendRefusal::Ended, w));
    let closed = |w| Some((LendRefusal::Closed, w));
    let refused = match () {
        _ if gone => ended(format!("{host} is no longer a member, so your visit ended")),
        _ if stopped => ended(format!("{host} ended your visit")),
        _ if !live => ended(format!("your visit with {host} is over: its time is up")),
        _ if !own => closed(format!("{name} is not {host}'s to open to you")),
        _ if !named => closed(format!("{host} did not open {name} to you")),
        _ => None,
    };
    tracing::debug!(target: "door", guest = pass.person, %host, lend = name, live, own, named, ?refused, "door: a visit's key — let in as its host's guest only while the visit is live and names this lend, its host's own, never a carried app");
    if let Some((kind, words)) = refused {
        return Err(kind.say(StatusCode::FORBIDDEN, words));
    }
    Ok(Asker {
        app: Some(pass.app),
        actor: PersonKey(pass.person.clone()),
        device: DeviceKey(pass.person),
        group,
        person: guest.visit.guest,
        host: Some(host),
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
        Some(p) => unix_now().map_or(Some("this computer's clock cannot be read".into()), |now| {
            p.check(&key, &label, now).err().map(|f| f.to_string())
        }),
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
    let dialer = dialed(
        dialer,
        "you can open this only from a computer in its group",
    )?;
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
            "the computer you reached refuses an address that could step outside what it lends",
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
            let Some((group, person, whose)) = PersonKey::resolved(door, &actor) else {
                tracing::debug!(target: "door", actor, "door: refused — the dialer stands in no group held here");
                return Err(LendRefusal::Closed.say(StatusCode::FORBIDDEN, LENDS_NOTHING));
            };
            Pending::Known(Asker {
                actor: whose,
                device: DeviceKey(actor.clone()),
                group,
                person,
                app: match &who {
                    Who::Own(app) => Some(app.clone()),
                    _ => None,
                },
                host: None,
            })
        }
        Who::Holder(dialer) => Pending::Pass(*dialer),
    };
    let lend = choices::found(door, raw_name)?;
    let Asker {
        actor: whose,
        device,
        group,
        person,
        mut app,
        host,
    } = match pending {
        Pending::Known(asker) => asker,
        Pending::Pass(holder) => holder_of(door, &holder, req.headers(), &lend).await?,
    };
    let actor = whose.as_str();
    let port = match &lend.source {
        Source::Port(port) => *port,
        Source::Carried(group) => {
            tracing::debug!(target: "door", actor, lend = %lend.name, group, path = service_path, "door: a carried app, answered from this computer's store");
            let opened = (door.folders).open(&lend, (group, device.as_str(), &service_path), req);
            return Ok(opened.await);
        }
    };
    if service_path == crate::oidc::AUTHORIZE_PATH {
        let Some(client) = lend.sign_in_with_svrngs.as_ref() else {
            tracing::debug!(target: "door", actor, lend = %lend.name, "door: refused — the authorize path on a lend that does not sign people in with svrngs");
            return Err(LendRefusal::Unlent.say(
                StatusCode::NOT_FOUND,
                format!("{} does not sign people in with svrngs", lend.name),
            ));
        };
        let (name, username) = told_as(&person, host.as_ref(), actor);
        let group = if host.is_some() {
            Group::Guest
        } else {
            Group::Member
        };
        let who = svrngs_core::token::Subject {
            sub: actor.to_string(),
            name,
            username,
            group,
        };
        tracing::debug!(target: "door", actor, lend = %lend.name, ?group,
            "door: the authorize path on the lend's own origin, answered by the door and never carried");
        let browser = req
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .map(|host| format!("{}://{host}", scheme_of(host)));
        return Ok(crate::oidc::authorize(
            &door.oidc,
            &lend,
            client,
            port,
            browser.as_deref(),
            who,
            req.uri().query(),
        ));
    }
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
    let whereabouts = crate::network::guess(observed.and_then(|o| o.addr));
    let Some((verdict, held)) = door.using.ask(&lend, actor, whereabouts).await else {
        tracing::warn!(target: "door", lend = %lend.name, "door: refused — no count of who uses the lend, so none can be let in");
        return Err(LendRefusal::Fault.say(
            StatusCode::INTERNAL_SERVER_ERROR,
            "the computer that lends this cannot count who is using it now, so it lets nobody in",
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
        Verdict::Full { using, limit } => {
            tracing::info!(target: "door", person = person.as_str(), lend = %lend.name, using, limit,
                "door: refused — as many people as the lend's limit are using it");
            return Err(busy(&lend, using, limit));
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
    tracing::debug!(target: "door", actor, person = person.as_str(), host = ?host.as_ref().map(Person::as_str), group, lend = %lend.name, app = ?app.as_ref().map(Label::as_str),
        method = %req.method(), path = service_path, "door: carrying a request to the lend's service, the app the pass's or none, a guest's host's or none");
    let told = Told { share, held };
    forward(
        door,
        (&lend, port),
        (&whose, &device, &person, app.as_ref()),
        host.as_ref(),
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

/// The refusal of a newcomer past `lend`'s `limit`, `using` people counted:
/// the reason, in a person's words — the number its lender's choice, which
/// they can change (bash-limit) — and the figures, for an app to turn into
/// a choice (DESIGN §3, "Too many at once").
fn busy(lend: &Lend, using: u32, limit: std::num::NonZeroU16) -> Response {
    let people = |n: u32| if n == 1 { "person" } else { "people" };
    let limit = limit.get();
    let reason = format!(
        "{} is lent to {limit} {} at once, and {using} {} using it now: try again when one stops. \
        That number is the choice of the person who lends it, and they can change it.",
        lend.name,
        people(u32::from(limit)),
        if using == 1 { "is" } else { "are" },
    );
    let body = serde_json::json!({ "error": reason, "using": using, "limit": limit });
    LendRefusal::Full.said(StatusCode::SERVICE_UNAVAILABLE, body)
}

/// A connection to `to`, `lend`'s service, when it takes one within
/// [`CONNECT_WITHIN`]; `None`, named at warn with why, when it does not —
/// what [`send_to`] carries a request on.
async fn connected(to: std::net::SocketAddr, lend: &str) -> Option<tokio::net::TcpStream> {
    match tokio::time::timeout(CONNECT_WITHIN, tokio::net::TcpStream::connect(to)).await {
        Ok(Ok(tcp)) => Some(tcp),
        Ok(Err(e)) => {
            tracing::warn!(target: "door", lend, %to, error = %e, "door: the lend's service refused a connection");
            None
        }
        Err(_) => {
            tracing::warn!(target: "door", lend, %to, within = ?CONNECT_WITHIN, "door: the lend's service took no connection in time");
            None
        }
    }
}

/// Send `request` on a new HTTP/1 connection to `to`, driven with
/// upgrades, and answer the response; `None`, named at warn by `lend`,
/// when nothing there takes the connection ([`connected`]) or answers.
async fn send_to(
    to: std::net::SocketAddr,
    request: axum::http::Request<Body>,
    lend: &str,
) -> Option<axum::http::Response<hyper::body::Incoming>> {
    let tcp = connected(to, lend).await?;
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

/// [`PAGES_HEADER`]'s value for `device`; `None`, named at warn, when the
/// groups held cannot be listed, and at debug when a page cannot be a header.
fn pages(door: &Door, device: &DeviceKey) -> Option<HeaderValue> {
    let stands = |group: &String| {
        let standing = door.rail.standing(group);
        tracing::debug!(target: "door", group, read = standing.is_ok(), "door: whether the device stands in a group, for its page; an unreadable register tells none");
        standing.is_ok_and(|s| s.names(device.as_str()))
    };
    let held = door.rail.namespaces().inspect_err(|e| tracing::warn!(target: "door", error = %e, "door: the groups held cannot be listed, so no page is told"));
    let told: Vec<String> = (held.ok()?.into_iter().filter(stands)).map(|group| {
        let dir = door.rail.group_dir(&group).map_err(|e| e.to_string());
        let page: String = dir.and_then(|d| crate::invite::kept_page(&d)).map_or_else(|why| {
            tracing::warn!(target: "door", group, why, "door: a group's page cannot be read, so it is told unread");
            "unread".into()
        }, |page| page.map_or("none".into(), |p| p.as_str().into()));
        format!("{group}={page}")
    }).collect();
    let value = HeaderValue::from_str(&told.join(" ")).ok();
    tracing::debug!(target: "door", device = device.as_str(), ?told, header = value.is_some(), "door: the pages of the groups the device stands in, none told when one cannot be a header");
    value
}

/// The share, when the door tells one, on the head of a request carried to
/// the service and of its answer ([`SHARE_HEADER`]).
fn tell_share(headers: &mut HeaderMap, share: Option<u64>) {
    if let Some(share) = share {
        headers.insert(
            HeaderName::from_static(SHARE_HEADER),
            HeaderValue::from(share),
        );
    }
}

/// Carry `req` to `lend`'s service as `path` (with its query), the head
/// rewritten, and stream the answer back.
async fn forward(
    door: &Door,
    (lend, port): (&Lend, NonZeroU16),
    (whose, device, person, app): (&PersonKey, &DeviceKey, &Person, Option<&Label>),
    host: Option<&Person>,
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
    // A guest is told as their host's guest, with an account of their own: a
    // member's ends in 16 hex ([`user_name`]), and `.guest` in none (GUESTS §3.5).
    let (told, user) = told_as(person, host, actor);
    let (Ok(person_value), Ok(key_value), Ok(user_value), Ok(device_value)) = (
        HeaderValue::from_bytes(told.as_bytes()),
        HeaderValue::from_str(actor),
        HeaderValue::from_str(&user),
        HeaderValue::from_str(device.as_str()),
    ) else {
        tracing::warn!(target: "door", actor, "door: refused — the person's name cannot be a header");
        return Err(LendRefusal::Fault.say(
            StatusCode::INTERNAL_SERVER_ERROR,
            "the computer that lends this cannot tell it who you are",
        ));
    };
    let identity = [
        (HeaderName::from_static(PERSON_HEADER), person_value),
        (HeaderName::from_static(KEY_HEADER), key_value),
        (HeaderName::from_static(USER_HEADER), user_value),
        (HeaderName::from_static(DEVICE_HEADER), device_value),
    ];
    tracing::debug!(target: "door", actor, user = ?identity[2].1, "door: the account the lend is told");
    let (me, name) = (hex::encode(door.key), &lend.name);
    let lender = door.rail.standing_anywhere(&me).map(|(_, m)| m.person);
    tracing::debug!(target: "door", actor, lender = ?lender.as_ref().map(|p| p.as_str()), "door: the lend's lender, by whom a refusal names it; none here names this computer, so \"Its lender\"");
    let lender = lender.as_ref().map_or("Its lender", |p| p.as_str());
    let away = format!("{lender}’s {name} is not answering now: try again later");
    let unanswered = || LendRefusal::Away.say(StatusCode::BAD_GATEWAY, away.clone());
    let origin = std::net::SocketAddr::from(([127, 0, 0, 1], port.get()));
    let secret = secret_headers(door, &lend.name);
    let target = parts
        .uri
        .query()
        .map_or(path.clone(), |q| format!("{path}?{q}"));
    let displaced = rewrite_headers(&mut parts.headers, identity, &secret);
    if let Some(app) = app {
        let app = HeaderValue::from_str(app.as_str()).expect("a label is base32, so a header");
        parts
            .headers
            .insert(HeaderName::from_static(APP_HEADER), app);
    }
    let label = Label::of(&door.key, lend.name.as_str());
    let named = HeaderValue::from_str(label.as_str()).expect("a label is base32, so a header");
    let pages = pages(door, device).map(|p| (HeaderName::from_static(PAGES_HEADER), p));
    let told = [(HeaderName::from_static(LABEL_HEADER), named)];
    parts.headers.extend(told.into_iter().chain(pages));
    if displaced > 0 {
        tracing::info!(target: "door", actor, lend = %lend.name, displaced,
            "door: dropped headers the client typed in the door's or the secret's name before setting them");
    }
    tell_origin(&mut parts.headers);
    parts.headers.insert(header::HOST, own_address(port));
    own_origin(&mut parts.headers, &label, (lend, port));
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
                "the computer that lends this cannot read that address",
            ));
        }
    };
    tell_share(&mut parts.headers, share);
    *request.headers_mut() = parts.headers;
    // The use began when the person was let in, and lasts until the member
    // has the whole answer or leaves it: the answer's body holds it.

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
    let sent = (parts.headers.get(header::LOCATION)).and_then(|l| l.to_str().ok());
    match sent.map(|l| LendPath::parse(l, port, None)) {
        Some(Ok(path)) => {
            let path = path.location([]);
            tracing::debug!(target: "door", lend = %lend.name, ?path, "door: a redirect to the service's own address, answered as a path");
            parts.headers.insert(header::LOCATION, path);
        }
        Some(Err(why)) => {
            tracing::debug!(target: "door", lend = %lend.name, location = sent, why, "door: the service's redirect, left as it sent it: the address")
        }
        None => {}
    }
    // The share on the answer is the door's, and so is a refusal's mark: a
    // service's own of either is displaced.
    parts.headers.remove(SHARE_HEADER);
    parts.headers.remove(REFUSED_HEADER);
    tell_share(&mut parts.headers, share);
    tracing::debug!(target: "door", actor, lend = %lend.name, status = parts.status.as_u16(),
        "door: the lend's service answered; streaming it back");
    let length = parts
        .headers
        .get(header::CONTENT_LENGTH)
        .and_then(|l| l.to_str().ok()?.parse().ok());
    let echoed = drop_echoed_secret(&mut parts.headers, &secret);
    if !echoed.is_empty() {
        tracing::warn!(target: "door", lend = %lend.name, headers = ?echoed,
            "door: the lend's service answered with its secret in a header — dropped, never sent on");
    }
    carriage.answered = Some((parts.status.as_u16(), length));
    tracing::debug!(target: "carriage", lend = %lend.name, range = ?carriage.range, status = parts.status.as_u16(),
        content_range = ?parts.headers.get(header::CONTENT_RANGE), ms = carriage.began.elapsed().as_millis(), "door: answered");
    let body = body.map_frame(move |frame| {
        let _ = &held;
        let carried = &mut carriage;
        carried.sent += frame.data_ref().map_or(0, |d| d.len() as u64);
        frame
    });
    let mut response = Response::new(Body::new(body));
    *response.status_mut() = parts.status;
    *response.headers_mut() = parts.headers;
    Ok(response.into_response())
}

#[cfg(test)]
mod tests;
