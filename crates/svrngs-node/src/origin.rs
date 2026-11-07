// SPDX-License-Identifier: AGPL-3.0-or-later
//! Every lend at an origin of its own on this computer's loopback (u5-2;
//! DESIGN §3, "Lent apps", call 1): `http://<label>.localhost:<port>/`, on
//! the record API's listener and behind its one check, the port its own.
//! New: upstream reached every lend of a lender through one bridge port
//! (PLAN §4, U5's inventory), so one lent page could read every other's
//! cookies and storage. A browser resolves `*.localhost` to loopback and
//! keeps each host's cookies, storage and workers apart — Chromium, WebKit
//! and Safari alike (`demo/runs/u5/2026-10-05-localhost/`).
//!
//! The lend the host's label names ([`svrngs_core::pass::Label`]) is
//! reached by its lender's key: this computer's own through its door's rules
//! — its secret injected and never let out, its limit counted — in the
//! door's one body ([`door::lend_for`], as [`door::Who::Own`]); another
//! computer's through the dial to its door, as this computer's key, its
//! answer and any WebSocket carried back whole. A label naming a key that
//! stands in no group held here reaches nothing, so no page can make this
//! computer dial a stranger.
//!
//! **Which app asks** (sup-1; DESIGN §3, "Lent apps", call 6) is read from
//! what the browser says of the page asking, which no page's script can set
//! ([`app_of`]), and the page is kept to its own lend and the lends of the
//! kinds its lender declared it reads ([`crate::surface::reaches`]), as a phone's pass
//! keeps it. The app is carried as the door's [`Who::Own`] to this
//! computer's own lend, and to another computer's in a pass this computer
//! signs for itself ([`own_pass`]), so the far door reads it from the one
//! carrier a phone's request brings too. A page stops its own app using a
//! lend, `POST /svrngs-reads/<label>/svrngs-stop` at its origin (sup-2),
//! carried the same way ([`crate::home::stop`]).

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::{self, HeaderMap, HeaderValue};
use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Router;
use serde_json::Value;
use std::collections::BTreeMap;

use svrngs_core::pass::{Label, Pass, Whose};

use crate::door::{self, Door, LendRefusal, Who, LEND_ROUTE, PASS_HEADER, REFUSED_HEADER};
use crate::iroh::Contact;
use crate::lend::{Kind, Offered};
use crate::node::Asking;
use crate::oidc::provider::same;
use crate::register::{Register, RemovalSaid};
use crate::session::{Shown, AS_PATH};
use crate::surface::{seed_of, STOP_SUFFIX};

/// Whether a lent app's answer may write entries as the person asking
/// ([`APPEND_HEADER`]), by what the request proved: decided by the door
/// that took it and handed to [`reach`] by each, never defaulted.
#[derive(Clone, Debug)]
pub(crate) enum Appending {
    /// This origin's own page, its session's proof carrying the person's
    /// intent (`x-svrngs-intent: yes`, which the proof signs).
    Granted,
    /// A program with the token, accepting entries of the one kind it named.
    Program(Kind),
    /// No entry is written, and why.
    Denied(Unconsented),
}

/// Why a request lets no lent app write entries as the person asking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unconsented {
    /// A program with the token that named no kind to accept.
    NoKind,
    /// A browser's request without the person's intent: a tab's read, a
    /// frame, a page that is not this origin's own.
    NoIntent,
    /// A page's read by the capability in its address: reach as that app,
    /// never a yes to sign the read lend's reply.
    Capability,
    /// The room door (`crate::room`): it checks only that the caller's
    /// address is on this computer's network, which proves no person.
    NetworkOnly,
}

impl Appending {
    fn for_request(shown: Shown, headers: &HeaderMap) -> Result<Self, Response> {
        let intent = headers.get("x-svrngs-intent").is_some_and(|h| h == "yes");
        let appending = match (shown, headers.get(APPEND_HEADER)) {
            (Shown::Token, Some(kind)) => {
                let kind = kind.to_str().ok().and_then(|k| Kind::parse(k).ok());
                Self::Program(kind.ok_or_else(|| {
                    tracing::debug!(target: "origin", "origin: refused — a program named no kind it accepts entries of that svrngs reads");
                    LendRefusal::Closed.say(
                        StatusCode::BAD_REQUEST,
                        "name the app's kind when accepting its entries",
                    )
                })?)
            }
            (Shown::Token, None) => Self::Denied(Unconsented::NoKind),
            (Shown::Session, _) if intent => Self::Granted,
            _ => Self::Denied(Unconsented::NoIntent),
        };
        tracing::debug!(target: "origin", ?shown, ?appending, "origin: whether a lent app's answer may write entries as the person asking");
        Ok(appending)
    }
    pub(crate) fn permits(&self, kind: &Kind) -> bool {
        match self {
            Self::Granted => true,
            Self::Program(wanted) => wanted == kind,
            Self::Denied(_) => false,
        }
    }
}

/// How long a pass this computer signs for its own page lasts: one
/// request's, signed afresh for each, so none is kept.
const OWN_PASS_FOR: i64 = 300;

/// The suffix every lend's host on this computer ends in.
const LOCALHOST: &str = ".localhost";

/// The lend a request's `Host` names, `<label>.localhost[:<port>]`, or
/// `None` for any other host: the record API's.
pub(crate) fn label_of(host: Option<&HeaderValue>) -> Option<Label> {
    at(host?.to_str().ok()?).map(|(label, _)| label)
}

/// `<label>.localhost[:<port>]` read: its label and its port as written.
fn at(host: &str) -> Option<(Label, Option<String>)> {
    let host = host.to_ascii_lowercase();
    let (name, port) = match host.rsplit_once(':') {
        Some((name, port)) => (name, Some(port.to_string())),
        None => (host.as_str(), None),
    };
    Some((Label::parse(name.strip_suffix(LOCALHOST)?)?, port))
}

/// The app asking for the lend `host` names, by DESIGN §3, "Lent apps", call 6, from
/// what the browser says of the page asking: a typed address or a request
/// from the lend's own origin is its own page's, and so is a navigation of
/// the whole tab, a link the person follows, unless its `Origin` names
/// another lend (a form that page posted); one from another lend's page is
/// that page's, by its `Origin` at this listener's port, or a frame's, which
/// sends none, by its `Referer`, whatever its `Sec-Fetch-Site` says (a
/// worker's fetch on the page's behalf says its own); one from another page naming no lend (an
/// `<img>`, a frame with its `Referer` withheld) is nobody's, `None`. So a
/// page's frame or form is held to what its fetch is (capability-navigation).
/// Asked only of a browser's request, with its session here
/// ([`Shown::Session`]), so with `Sec-Fetch-Site`; a
/// program's, which shows the token, is the lend's own ([`lent`]).
fn app_of(headers: &HeaderMap, host: &Label) -> Option<Label> {
    let ([site, mode, origin, at_host], own) = fetch(headers);
    let port = at(at_host?)?.1;
    let dest = headers.get("sec-fetch-dest").and_then(|v| v.to_str().ok());
    let framed = mode == Some("navigate") && dest.is_some_and(|d| d != "document");
    // A frame's navigation sends no `Origin`: who framed it says its `Referer`.
    let sent = [header::ORIGIN, header::REFERER][usize::from(framed)].clone();
    let from = (headers.get(sent).and_then(|v| v.to_str().ok()))
        .and_then(|o| o.strip_prefix("http://")?.split('/').next())
        .and_then(at)
        .filter(|(_, p)| *p == port)
        .map(|(label, _)| label);
    let another = from.as_ref().is_some_and(|l| l != host);
    // Another lend named outweighs `same-origin`: a service worker's fetch
    // of a navigation it was shown no `Origin` of (WebKit) says its own
    // site, the browser still sending the page's true `Origin`.
    if own && !another && (matches!(site, Some("none" | "same-origin")) || !framed) {
        tracing::debug!(target: "origin", %host, ?site, ?mode, ?dest, "origin: a typed address, its own origin, or the whole tab going there that no other lend's page sent, so the lend's own page");
        return Some(host.clone());
    }
    tracing::debug!(target: "origin", %host, ?site, ?mode, ?dest, ?origin, app = ?from.as_ref().map(Label::as_str),
        "origin: from another page, a fetch, a frame or a form, so the app its Origin names (a frame's, its Referer), or none");
    from
}

/// What the browser says of the page asking, which no page's script can
/// set — its `Sec-Fetch-Site`, `Sec-Fetch-Mode`, `Origin` and the `Host`
/// it asked — and whether it is a navigation, a typed address or the page's
/// own origin: read once for a lend's origin ([`app_of`]) and the home page
/// (`crate::home`'s `only_itself`), each deciding the rest by its own rule.
pub(crate) fn fetch(headers: &HeaderMap) -> ([Option<&str>; 4], bool) {
    let [site, mode, origin, host] = ["sec-fetch-site", "sec-fetch-mode", "origin", "host"]
        .map(|name| headers.get(name).and_then(|v| v.to_str().ok()));
    let own = mode == Some("navigate") || matches!(site, Some("none" | "same-origin"));
    ([site, mode, origin, host], own)
}

/// A pass this computer signs for its own page of `app` to reach `label`
/// at another computer's door: its key the person and the holder, so that
/// door, dialled as this key, reads the app from it (`door.rs`'s `own_app`).
fn own_pass(at: &Asking, app: &Label, label: &Label) -> Option<HeaderValue> {
    let reads = Vec::from_iter((app != label).then(|| label.clone()));
    let until = door::unix_now()? + OWN_PASS_FOR;
    let (key, actor) = (at.host.rail.signer(), &at.host.actor);
    let pass = Pass::sign(key, actor, (app.clone(), reads), until);
    HeaderValue::from_str(&pass.encode()).ok()
}

/// The person sent to the home page's `/to`, the lend `label` at `uri`,
/// which opens it with a code for its origin only if the home page's own
/// session or rule lets it (`crate::home`'s `only_itself`): a navigation,
/// with the home page's session, which a lent page's fetch or frame never
/// sends (DESIGN §7, "On a computer"). It says no yes.
fn homeward(at: &Asking, label: &Label, uri: &Uri) -> Response {
    let tail: String = form_urlencoded::byte_serialize(uri.to_string().as_bytes()).collect();
    let home = crate::home::address(at.host.api_port) + "to?lend=" + label.as_str();
    Redirect::to(&(home + "&at=" + &tail)).into_response()
}

/// Where this computer's browser opens the lend `label`, with the record
/// API listening on `port`.
pub fn origin(label: &Label, port: u16) -> String {
    format!("http://{label}{LOCALHOST}:{port}/")
}

/// The routes every lend's origin is served by, for [`crate::api::router`].
pub(crate) fn router(asking: Asking) -> Router {
    let paged = axum::middleware::from_fn(door::paged);
    Router::new().fallback(lent).layer(paged).with_state(asking)
}

/// A request at a lend's origin, carried to the lend.
async fn lent(State(at): State<Asking>, mut req: Request) -> Result<Response, Response> {
    let Some(label) = label_of(req.headers().get(header::HOST)) else {
        tracing::debug!(target: "origin", host = ?req.headers().get(header::HOST), "origin: refused — no lend's host");
        return Err(LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here"));
    };
    if req.uri().path() == crate::session::OPEN_PATH {
        return Ok(at.sessions.redeem(label.as_str(), &req));
    }
    if let Some(answer) = at.sessions.public(label.as_str(), &req) {
        return Ok(answer);
    }
    // A page's read of this lend, by the capability in its address
    // (`crate::surface::reads`), is taken off before anything is decided.
    let cap = capped(&mut req)?;
    // A browser with this origin's session, else a program with the token,
    // which is the lend's own; neither is carried on to the lend.
    let mut shown = if at.sessions.socket(label.as_str(), &mut req) {
        Shown::Session
    } else {
        at.sessions.shown(&req, label.as_str())
    };
    let mut own_proof = shown == Shown::Session;
    let mut appending = Appending::for_request(shown, req.headers())?;
    // The proof, before it is stripped: a redirect its answer names is
    // followed with it, once (`crate::session::Onwards`).
    let proof = crate::session::Sessions::proof_of(req.headers());
    at.sessions.strip(req.headers_mut());
    if let Some(cap) = cap.filter(|_| shown != Shown::Token) {
        own_proof = false;
        let app = app_of(req.headers(), &label);
        if app.as_ref() == Some(&label) {
            tracing::debug!(target: "origin", %label, "origin: the person going to a read's address, opened by the home page, whose session decides; a capability is never a session");
            return Ok(homeward(&at, &label, req.uri()));
        }
        let held = app
            .as_ref()
            .is_some_and(|a| same(cap.as_bytes(), at.sessions.cap(a, &label).as_bytes()));
        tracing::debug!(target: "origin", %label, ?app, held, "origin: a page's read by the capability in its address, refused unless this computer's for that page");
        shown = if held { Shown::Session } else { Shown::Neither };
        // A read capability can be held by the app's server too. It proves
        // reach as that app, never authority to sign a target's reply.
        appending = Appending::Denied(Unconsented::Capability);
    }
    // A bookmark in a browser that never opened this lend: the whole tab,
    // and only it, goes by the home page's `/to`, as a read's address does.
    let dest = (req.headers().get("sec-fetch-dest")).and_then(|v| v.to_str().ok());
    let tab = (fetch(req.headers()).0[1], dest) == (Some("navigate"), Some("document"));
    if shown == Shown::Neither && tab && req.method() == Method::GET {
        tracing::debug!(target: "origin", %label, "origin: the whole tab here with neither this origin's session nor the token, so on to the home page's /to, whose session decides");
        return Ok(homeward(&at, &label, req.uri()));
    }
    if shown == Shown::Neither {
        tracing::debug!(target: "origin", %label, "origin: refused — neither this origin's session nor the token, and no whole tab's navigation");
        return Err(at.sessions.refused(""));
    }
    if own_proof
        && matches!(appending, Appending::Denied(_))
        && !matches!(*req.method(), Method::GET | Method::HEAD)
    {
        tracing::debug!(target: "origin", %label, method = %req.method(), ?appending, "origin: refused — a write with this origin's proof but not from its own page's intent");
        return Err(LendRefusal::Closed.say(
            StatusCode::FORBIDDEN,
            "only this origin's client may submit a write",
        ));
    }
    let program = (shown == Shown::Token).then(|| label.clone());
    let Some(app) = program.or_else(|| app_of(req.headers(), &label)) else {
        tracing::debug!(target: "origin", %label, "origin: refused — from another page that names no origin");
        return Err(LendRefusal::Closed.say(
            StatusCode::FORBIDDEN,
            "a page reaches another lend only by fetch, which says where it is from",
        ));
    };
    if own_proof && app != label {
        tracing::debug!(target: "origin", %label, %app, "origin: refused — this origin's proof, from a page its fetch names as another app's");
        return Err(LendRefusal::Closed.say(
            StatusCode::FORBIDDEN,
            "an origin's proof cannot name another app",
        ));
    }
    if req.uri().path() == crate::surface::READS_PATH {
        if app != label {
            tracing::debug!(target: "origin", %label, %app, "origin: refused — another app's page asks the lends this one reads");
            return Err(LendRefusal::Closed.say(
                StatusCode::FORBIDDEN,
                "only an app's own origin may ask its capabilities",
            ));
        }
        tracing::debug!(target: "origin", %label, "origin: its own page asks the lends it reads; reaching an app grants no authority over its capabilities");
        return crate::surface::reads(&at, &label).await;
    }
    let stop = req.uri().path().strip_prefix(crate::surface::READS_PATH);
    let stop = stop.and_then(|p| p.strip_prefix('/')?.strip_suffix(STOP_SUFFIX));
    if let Some(lend) = stop
        .and_then(Label::parse)
        .filter(|_| req.method() == Method::POST)
    {
        tracing::debug!(target: "origin", %label, %app, %lend, "origin: a page stops its app using a lend");
        let pass = own_pass(&at, &app, &lend);
        return Ok(crate::home::say(&at, &lend, app, crate::door::choices::Said::Stop, pass).await);
    }
    if app != label {
        // A page that reads the lend is told its person's yes is missing,
        // so it can say what to do (capability-open); one that does not
        // read it is told nothing of it.
        let page = req.headers().get(header::ORIGIN).cloned();
        let unasked =
            |r: &Response| (r.headers().get(REFUSED_HEADER)).is_some_and(|k| k == "unasked");
        let reached = match crate::surface::reaches(&at, &app, &label).await {
            Ok(()) => reach(&at, label, app, appending, req).await,
            Err(refused) if unasked(&refused) => Err(refused),
            Err(refused) => return Err(refused),
        };
        return Ok(readable(reached, page));
    }
    let (method, name) = (req.method().clone(), label.clone());
    let reached = reach(&at, label, app, appending, req).await;
    if let (Some(proof), Ok(answer)) = (proof.filter(|_| own_proof), &reached) {
        at.sessions.onward(name.as_str(), &proof, &method, answer);
    }
    reached
}

/// svrngs's own answer to the page of another lend that reads this one —
/// an answer marked [`REFUSED_HEADER`], which no service's carries (the
/// door displaces a service's, mnp-1) — made readable by that page: its
/// origin, `page`, alone allowed, never `*`, and the mark exposed (DESIGN
/// §3, "What an app gets"). A preflight is a request like any other here,
/// under the same rule. A service's own answer is left as it sent it.
fn readable(answer: Result<Response, Response>, page: Option<HeaderValue>) -> Response {
    let mut answer = answer.unwrap_or_else(|refused| refused);
    let kind = answer.headers().get(REFUSED_HEADER).cloned();
    let (Some(kind), Some(page)) = (kind, page) else {
        tracing::debug!(target: "origin", status = answer.status().as_u16(), "origin: the lend's own answer to a page that reads it, its headers its own");
        return answer;
    };
    tracing::debug!(target: "origin", ?page, ?kind, "origin: svrngs's own answer, readable by the page that reads the lend");
    let headers = answer.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, page);
    let exposed = HeaderValue::from_static(REFUSED_HEADER);
    headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, exposed);
    headers.append(header::VARY, HeaderValue::from_static("origin"));
    answer
}

/// The capability `req`'s path names under [`AS_PATH`], taken off its path.
fn capped(req: &mut Request) -> Result<Option<String>, Response> {
    let path = req.uri().to_string();
    let Some((cap, tail)) = (path.strip_prefix(AS_PATH)).and_then(|p| p.split_once('/')) else {
        return Ok(None);
    };
    *req.uri_mut() = uri_of(&format!("/{tail}"))?;
    Ok(Some(cap.into()))
}

/// `req` carried to the lend `label`, as this computer's person, for the
/// page of the lend `app`: by the `Host` at its origin on loopback, or by
/// the port at the room door (u6-1), one body for both (DESIGN §3, "The
/// room", call 3). Each door says what its caller proved of entries the
/// answer names, `appending`; none is assumed.
pub(crate) async fn reach(
    at: &Asking,
    label: Label,
    app: Label,
    appending: Appending,
    mut req: Request,
) -> Result<Response, Response> {
    let tail = req.uri().path_and_query().map_or("/", |p| p.as_str());
    let target = format!("{LEND_ROUTE}{label}{tail}");
    let door: &Door = &at.door;
    let Serving::At(contact, lender) = serving(at, &label).await? else {
        tracing::debug!(target: "origin", %label, "origin: this computer's own lend, through its door's rules");
        let (mut parts, body) = req.into_parts();
        parts.uri = uri_of(&target)?;
        let answer = door::lend_for(door, Who::Own(app), None, Request::from_parts(parts, body));
        return appended(at, &label, &Serving::Own, answer.await?, appending).await;
    };
    let Some(pass) = own_pass(at, &app, &label) else {
        return Err(LendRefusal::Fault.say(
            StatusCode::INTERNAL_SERVER_ERROR,
            "this computer could not say which app asks",
        ));
    };
    req.headers_mut().insert(PASS_HEADER, pass);
    tracing::debug!(target: "origin", %label, %app, lender, "origin: another computer's lend, through the dial to its door, the app in this computer's own pass");
    let answer = carried(at, req, &contact, &target, label.as_str()).await;
    let answer = answer.inspect_err(|_| at.lenders.away(&label, &lender))?;
    appended(at, &label, &Serving::At(contact, lender), answer, appending).await
}

/// The header in which a lent app's answer names, as a JSON array, the
/// acts it writes as the person asking: each `{group, op}` or `{group,
/// corrects, op}`, as a carried app's page posts one (capability-signed,
/// F1; DESIGN §3, "Lent apps").
pub(crate) const APPEND_HEADER: &str = "x-svrngs-append";

/// Why a lent app's act is not written as the person asking, as they are
/// told it after the app's name.
#[derive(Debug, thiserror::Error)]
enum Unwritten {
    #[error("answered with an entry svrngs cannot read")]
    Unread,
    #[error("wrote in a group you and its lender are not both in")]
    NotBoth,
    #[error("wrote in a group it is not lent to")]
    NotLent,
    #[error("writes only its own part of the group’s record")]
    OtherPart,
}

/// `answer` from the lend `label`, its lender's computer at `contact`
/// (none for this computer's own), with each act it names in
/// [`APPEND_HEADER`] written by this computer's key as its own person,
/// under the kind the lend declares, in a group its lender stands in too,
/// and carried to the lender before the page sees the answer, which never
/// carries the header on. One refused, none is written: the answer is
/// withheld and the person told why.
async fn appended(
    at: &Asking,
    label: &Label,
    serving: &Serving,
    mut answer: Response,
    appending: Appending,
) -> Result<Response, Response> {
    let Some(named) = answer.headers_mut().remove(APPEND_HEADER) else {
        return Ok(answer);
    };
    if let Appending::Denied(why) = appending {
        tracing::debug!(target: "origin", %label, ?why, "origin: refused — a lent app's answer named entries this request did not consent to, so none is written and the answer withheld");
        return Err(LendRefusal::Closed.say(
            StatusCode::FORBIDDEN,
            "this request did not authorize entries from the app",
        ));
    }
    let (rail, (contact, lender)) = (&at.host.rail, serving.dialled(&at.host.actor));
    let lend = declared(at, label, serving).await;
    if let Some(l) = lend.as_ref().filter(|l| !appending.permits(&l.kind)) {
        tracing::debug!(target: "origin", %label, kind = %l.kind.as_str(), ?appending, "origin: refused — the lend's declared kind is not the one the program accepts entries of");
        return Err(LendRefusal::Closed.say(
            StatusCode::FORBIDDEN,
            "the app's kind changed; this program authorized no entry of that kind",
        ));
    }
    let who = contact.and_then(|_| rail.standing_anywhere(lender));
    let whose = who.map_or("Your".into(), |(_, m)| format!("{}’s", m.person));
    let name = lend.as_ref().map_or("app", |l| l.name.as_str());
    let what = format!("{whose} {name}");
    let acts: Vec<Value> = serde_json::from_slice(named.as_bytes()).map_err(|why| {
        tracing::debug!(target: "origin", %label, %why, "origin: refused a malformed append declaration; no success-shaped substitution");
        LendRefusal::Closed.say(StatusCode::UNPROCESSABLE_ENTITY, format!("{what} answered with entries svrngs cannot read, so they were not saved."))
    })?;
    let judged = |act: &Value| {
        let kind = lend.as_ref().ok_or(Unwritten::Unread)?.kind.as_str();
        let group = act["group"].as_str();
        let read = rail.admission(group.ok_or(Unwritten::Unread)?).ok();
        let both =
            |s: &std::collections::BTreeSet<_>| s.contains(lender) && s.contains(&at.host.actor);
        let read = read
            .filter(|a| both(&a.membership().standing))
            .ok_or(Unwritten::NotBoth)?;
        // Only a group the lend reaches, never another both stand in.
        let reached = lend
            .as_ref()
            .is_some_and(|l| l.clone().listed(|g| Some(g) == group));
        reached.then_some(()).ok_or(Unwritten::NotLent)?;
        let body = serde_json::to_vec(act).unwrap_or_default();
        let shaped = svrngs_core::grant::app_act(kind, &body).ok_or(Unwritten::Unread)?;
        let ops = serde_json::to_value(read.wire()).unwrap_or_default();
        let ops = ops.as_array().map_or(&[][..], Vec::as_slice);
        match shaped.get("corrects") {
            Some(id) if !crate::folder::carried::of_kind(ops, kind, id) => {
                Err(Unwritten::OtherPart)
            }
            _ => Ok((group.unwrap_or_default().to_string(), shaped)),
        }
    };
    let acts = acts.iter().map(judged).collect::<Result<Vec<_>, _>>().map_err(|why| {
        tracing::debug!(target: "origin", %label, ?why, ?named, "origin: refused — a lent app's act, not written as the person asking, and its answer withheld");
        LendRefusal::Closed.say(StatusCode::FORBIDDEN, format!("{what} {why}, so it was not saved."))
    })?;
    for (group, act) in acts {
        let written = crate::rail::append_act(rail, &group, act).await;
        tracing::debug!(target: "origin", %label, group, status = written.status().as_u16(), carried = contact.is_some(), "origin: a lent app's act, written by this computer as its own person, then carried to the lender");
        if !written.status().is_success() {
            return Err(written);
        }
        at.folders.nudge.notify_one();
        let Some(contact) = contact else { continue };
        let line = axum::body::to_bytes(written.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        let line = serde_json::from_slice::<Value>(&line).map_or(Value::Null, |l| l["op"].clone());
        let asked = axum::http::Request::post(crate::rail::CARRY_ROUTE)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::json!([group, line]).to_string().into())
            .expect("a fixed route");
        let taken = at.host.transport.fetch(contact, asked).await;
        let taken: Option<Value> = taken
            .ok()
            .and_then(|t| serde_json::from_slice(t.body()).ok());
        tracing::debug!(target: "origin", %label, group, ?taken, "origin: the lender's computer took the line, or why not");
        if taken.is_none_or(|t| t["counted"] != true) {
            let said =
                format!("Saved on this computer. {what} shows it once {whose} computer has it.");
            return Err(LendRefusal::Away.say(StatusCode::BAD_GATEWAY, said));
        }
    }
    Ok(answer)
}

/// The lend `label` as its lender declares it: this computer's own from
/// its declarations, another computer's asked of it now.
async fn declared(at: &Asking, label: &Label, serving: &Serving) -> Option<Offered> {
    let (contact, lender) = serving.dialled(&at.host.actor);
    let (lends, version) = match contact {
        None => (door::offered(&at.door).await.ok()?, crate::version::VERSION),
        Some(c) => {
            match crate::node::ask_lends(&at.host.transport, label.as_str(), c.clone()).await {
                crate::node::Answered::Lends(lends, version) => (lends, version),
                _ => return None,
            }
        }
    };
    lends
        .into_iter()
        .find(|o| o.label(lender, version).as_ref() == Some(label))
}

/// Who serves the lend `label` to this computer: its own door, or the
/// computer at `.0`, whose key is `.1`, dialled as this computer's key.
pub(crate) enum Serving {
    Own,
    At(Contact, String),
}

impl Serving {
    /// The contact dialled, none for this computer's own, and the lender's
    /// key, `own` for this computer's.
    fn dialled<'a>(&'a self, own: &'a str) -> (Option<&'a Contact>, &'a str) {
        match self {
            Self::Own => (None, own),
            Self::At(contact, lender) => (Some(contact), lender),
        }
    }
}

/// Who serves `label` ([`Serving`]): a computer's lend its lender; a
/// carried app this computer's own copy when it carries one, else the
/// first computer [`Lenders`] names that is a contact here, so the app is
/// one origin whichever computer answers (COMMUNITY_APPS §7). Not heard of
/// yet, every group held here is asked what it lends first.
pub(crate) async fn serving(at: &Asking, label: &Label) -> Result<Serving, Response> {
    let key = match label.whose() {
        Whose::Computer(key) => hex::encode(key),
        Whose::Carried => match at.lenders.of(label) {
            Some(keys) => lender_of(at, label, keys)?,
            None => {
                let (groups, own) = at.host.shared_with(&at.host.actor)?;
                if let Some(own) = own {
                    at.lent(&groups, &own, None).await?;
                }
                lender_of(at, label, at.lenders.of(label).unwrap_or_default())?
            }
        },
    };
    if key == at.host.actor {
        return Ok(Serving::Own);
    }
    let contact = at
        .host
        .contact(&key)
        .ok_or_else(|| refused(at, label, &key))?;
    Ok(Serving::At(contact, key))
}

/// The first of `keys` carrying `label` that is this computer or a
/// contact here, or "nothing is lent here".
fn lender_of(at: &Asking, label: &Label, keys: Vec<String>) -> Result<String, Response> {
    let found = keys
        .iter()
        .find(|k| **k == at.host.actor || at.host.contact(k).is_some());
    tracing::debug!(target: "origin", %label, ?keys, found, "origin: a carried app, served by the first computer carrying it that is reached from here");
    found
        .cloned()
        .ok_or_else(|| LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here"))
}

/// Which computers carry each carried app, as the last listing of lends
/// here said ([`Asking::lent`]): this computer's own copy first, then each
/// other's; one that did not answer goes last ([`Self::away`]).
#[derive(Clone)]
pub(crate) struct Lenders(tokio::sync::watch::Sender<BTreeMap<Label, Vec<String>>>);

impl Default for Lenders {
    fn default() -> Self {
        Self(tokio::sync::watch::Sender::new(BTreeMap::new()))
    }
}

impl Lenders {
    /// What a listing of lends said, each carried app's computers in its order.
    pub(crate) fn heard(&self, lent: &[crate::node::Lending]) {
        let mut heard: BTreeMap<Label, Vec<String>> = BTreeMap::new();
        for l in lent.iter().filter(|l| l.offered.group.is_some()) {
            let Some(label) = l.offered.label(&l.key, l.version) else {
                continue;
            };
            heard.entry(label).or_default().push(l.key.clone());
        }
        self.0.send_modify(|all| all.extend(heard));
    }

    fn of(&self, label: &Label) -> Option<Vec<String>> {
        self.0.borrow().get(label).cloned()
    }

    /// `lender` did not answer for `label`: the next request tries it last.
    fn away(&self, label: &Label, lender: &str) {
        self.0.send_if_modified(|all| {
            let Some(keys) = all.get_mut(label) else {
                return false;
            };
            keys.retain(|k| k != lender);
            keys.push(lender.into());
            true
        });
    }
}

/// `target` as the address a lend is asked at, or refused as no path.
fn uri_of(target: &str) -> Result<Uri, Response> {
    target.parse().map_err(|_| {
        LendRefusal::Path.say(
            StatusCode::BAD_REQUEST,
            "your computer cannot read that address",
        )
    })
}

/// The computer lending `label`, to be dialled as this computer's key,
/// or [`refused`].
pub(crate) async fn dialled(at: &Asking, label: &Label) -> Result<Contact, Response> {
    match serving(at, label).await? {
        Serving::At(contact, _) => Ok(contact),
        Serving::Own => Err(LendRefusal::Fault.say(
            StatusCode::INTERNAL_SERVER_ERROR,
            "this computer's own lend is not dialled",
        )),
    }
}

/// A lender standing in no group held here refused, so no page makes this
/// computer dial a stranger; said as this computer's person reads it where
/// a group held here cut this computer and the lender stands in it (#182):
/// "You left Trip on 16 Oct", the day as the command says one (cli.rs
/// `namesakes_said`), or the lenders' own words for a removal
/// ([`door::gone`]). Groups that cannot be listed, or a register unreadable
/// here, show no cut: [`crate::node::Host::contact`] has just said them at
/// warn.
fn refused(at: &Asking, label: &Label, lender: &str) -> Response {
    let (rail, me) = (&at.host.rail, at.host.actor.as_str());
    let cut = |r: &Register| r.removed.contains_key(me) && r.members.contains_key(lender);
    let mut held = rail.namespaces().unwrap_or_default().into_iter();
    let Some((r, id)) = held.find_map(|id| Some((rail.register(&id).ok().filter(cut)?, id))) else {
        tracing::debug!(target: "origin", %label, lender, "origin: refused — the label names no computer standing in a group held here");
        return LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here");
    };
    let left = (r.removals.iter()).find(|x| x.key == me && x.said == RemovalSaid::Left);
    tracing::debug!(target: "origin", %label, group = id, left = left.is_some(), "origin: refused — this computer was cut from a group the lender stands in, so the cut is said");
    let Some(left) = left else {
        return door::gone();
    };
    let name = seed_of(at, &id).map_or("this group".into(), |s| s.name().as_str().to_string());
    let day = chrono::DateTime::from_timestamp(left.at, 0).map(|t| t.with_timezone(&chrono::Local));
    let on = day.map_or(String::new(), |t| t.format(" on %-d %b").to_string());
    LendRefusal::Removed.say(StatusCode::FORBIDDEN, format!("You left {name}{on}"))
}

/// The lender's computer could not be reached.
pub(crate) fn away() -> Response {
    LendRefusal::Away.say(
        StatusCode::BAD_GATEWAY,
        "the computer that lends this did not answer; try again later",
    )
}

/// `req` carried to the door of the computer at `contact` as `target`, on
/// a request on the stream, its answer back whole, a WebSocket spliced once
/// both sides switch; `away` when it has no answer.
async fn carried(
    at: &Asking,
    mut req: Request,
    contact: &Contact,
    target: &str,
    label: &str,
) -> Result<Response, Response> {
    let upgrade = door::upgrade_of(req.headers());
    let asker = upgrade.as_ref().map(|_| hyper::upgrade::on(&mut req));
    let (mut parts, body) = req.into_parts();
    door::strip_hop_by_hop(&mut parts.headers);
    if let Some(protocol) = upgrade {
        door::keep_upgrade(&mut parts.headers, protocol);
    }
    let mut request = axum::http::Request::new(body);
    *request.uri_mut() = uri_of(target)?;
    *request.method_mut() = parts.method;
    *request.headers_mut() = parts.headers;
    let mut answer = at.host.transport.send(contact, request).await.map_err(|e| {
        let whose = at.host.rail.standing_anywhere(&hex::encode(contact.key.as_bytes())).map(|(_, m)| m.person);
        tracing::debug!(target: "origin", label, error = %e, whose = ?whose.as_ref().map(|p| p.as_str()), "origin: the lender's computer did not answer, so a tab is shown their window dark, or svrngs's words where no group held here names them");
        let mut away = away();
        whose.map(|p| away.extensions_mut().insert(door::Dark(p)));
        away
    })?;
    let switched = door::upgrade_of(answer.headers())
        .filter(|_| answer.status() == StatusCode::SWITCHING_PROTOCOLS);
    let service = switched.as_ref().map(|_| hyper::upgrade::on(&mut answer));
    let (mut parts, body) = answer.into_parts();
    door::strip_hop_by_hop(&mut parts.headers);
    if let (Some(asker), Some(service), Some(protocol)) = (asker, service, switched) {
        door::keep_upgrade(&mut parts.headers, protocol);
        door::splice(
            asker,
            service,
            (),
            label.to_string(),
            std::future::pending(),
        );
    }
    let mut response = Response::new(Body::new(body));
    *response.status_mut() = parts.status;
    *response.headers_mut() = parts.headers;
    Ok(response.into_response())
}

#[cfg(test)]
#[path = "origin_tests.rs"]
mod tests;
