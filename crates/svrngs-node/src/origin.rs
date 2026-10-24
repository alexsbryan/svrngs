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
use axum::response::{IntoResponse, Response};
use axum::Router;
use svrngs_core::pass::{Label, Pass};

use crate::door::{self, Door, LendRefusal, Who, LEND_ROUTE, PASS_HEADER, REFUSED_HEADER};
use crate::iroh::Contact;
use crate::node::Asking;
use crate::register::{Register, RemovalSaid};
use crate::surface::{seed_of, STOP_SUFFIX};

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
/// sends none, by its `Referer`; one from another page naming no lend (an
/// `<img>`, a frame with its `Referer` withheld) is nobody's, `None`. So a
/// page's frame or form is held to what its fetch is (capability-navigation).
/// A request with no `Sec-Fetch-Site` is a program's, not a page's
/// — a browser that sends any `Sec-Fetch-*` sends it, and Node's `fetch`
/// sends a mode alone (start-measure, `demo/runs/start/2026-10-13/` §3 (a)
/// 4), as the record reads a page by it too (`crate::api`'s
/// `not_from_a_page`) — and is the lend's own, as loopback is trusted
/// (PLAN §2).
fn app_of(headers: &HeaderMap, host: &Label) -> Option<Label> {
    let ([site, mode, origin, at_host], own) = fetch(headers);
    if site.is_none() {
        tracing::debug!(target: "origin", %host, ?mode, "origin: no Sec-Fetch-Site, a program on this computer, so the lend's own");
        return Some(host.clone());
    }
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
    if own && (matches!(site, Some("none" | "same-origin")) || !(framed || another)) {
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
    let reads = if app == label {
        vec![]
    } else {
        vec![label.clone()]
    };
    let until = door::unix_now()? + OWN_PASS_FOR;
    let (key, actor) = (at.host.rail.signer(), &at.host.actor);
    let pass = Pass::sign(key, actor, (app.clone(), reads), until);
    HeaderValue::from_str(&pass.encode()).ok()
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
async fn lent(State(at): State<Asking>, req: Request) -> Result<Response, Response> {
    let Some(label) = label_of(req.headers().get(header::HOST)) else {
        tracing::debug!(target: "origin", host = ?req.headers().get(header::HOST), "origin: refused — no lend's host");
        return Err(LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here"));
    };
    if req.uri().path() == crate::surface::READS_PATH {
        tracing::debug!(target: "origin", %label, "origin: the lend's page asks the lends it reads, answered from the record");
        return crate::surface::reads(&at, &label).await;
    }
    let Some(app) = app_of(req.headers(), &label) else {
        tracing::debug!(target: "origin", %label, "origin: refused — from another page that names no origin");
        return Err(LendRefusal::Closed.say(
            StatusCode::FORBIDDEN,
            "a page reaches another lend only by fetch, which says where it is from",
        ));
    };
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
        let unasked = |r: &Response| {
            r.headers()
                .get(REFUSED_HEADER)
                .is_some_and(|k| k == "unasked")
        };
        let reached = match crate::surface::reaches(&at, &app, &label).await {
            Ok(()) => reach(&at, label, app, req).await,
            Err(refused) if unasked(&refused) => Err(refused),
            Err(refused) => return Err(refused),
        };
        return Ok(readable(reached, page));
    }
    reach(&at, label, app, req).await
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

/// `req` carried to the lend `label`, as this computer's person, for the
/// page of the lend `app`: by the `Host` at its origin on loopback, or by
/// the port at the room door (u6-1), one body for both (DESIGN §3, "The
/// room", call 3).
pub(crate) async fn reach(
    at: &Asking,
    label: Label,
    app: Label,
    mut req: Request,
) -> Result<Response, Response> {
    let tail = req.uri().path_and_query().map_or("/", |p| p.as_str());
    let target = format!("{LEND_ROUTE}{label}{tail}");
    let lender = label.lender();
    let door: &Door = &at.door;
    if lender == door.key {
        tracing::debug!(target: "origin", %label, "origin: this computer's own lend, through its door's rules");
        let (mut parts, body) = req.into_parts();
        parts.uri = uri_of(&target)?;
        return door::lend_for(door, Who::Own(app), None, Request::from_parts(parts, body)).await;
    }
    let contact = dialled(at, &label)?;
    let Some(pass) = own_pass(at, &app, &label) else {
        return Err(LendRefusal::Fault.say(
            StatusCode::INTERNAL_SERVER_ERROR,
            "this computer could not say which app asks",
        ));
    };
    req.headers_mut().insert(PASS_HEADER, pass);
    tracing::debug!(target: "origin", %label, %app, lender = hex::encode(lender), "origin: another computer's lend, through the dial to its door, the app in this computer's own pass");
    carried(at, req, &contact, &target, label.as_str()).await
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
pub(crate) fn dialled(at: &Asking, label: &Label) -> Result<Contact, Response> {
    let actor = hex::encode(label.lender());
    at.host
        .contact(&actor)
        .ok_or_else(|| refused(at, label, &actor))
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
        door::splice(asker, service, (), label.to_string());
    }
    let mut response = Response::new(Body::new(body));
    *response.status_mut() = parts.status;
    *response.headers_mut() = parts.headers;
    Ok(response.into_response())
}
