// SPDX-License-Identifier: AGPL-3.0-or-later
//! Every lend at an origin of its own on this computer's loopback (u5-2;
//! DESIGN §3, "Lent apps", call 1): `http://<label>.localhost:<port>/`, on
//! the record API's listener and behind its one check, the port its own.
//! New: upstream reached every lend of a lender through one bridge port
//! (DESIGN §8, U5's inventory), so one lent page could read every other's
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
//! kinds its lender declared it reads ([`kept_to_reads`]), as a phone's pass
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

use crate::door::{self, Door, LendRefusal, Who, LEND_ROUTE, PASS_HEADER};
use crate::iroh::{Contact, PublicKey};
use crate::lend::Offered;
use crate::node::Asking;

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

/// The app asking for the lend `host` names, by DESIGN §3's table, from
/// what the browser says of the page asking: a navigation, a typed address
/// or a request from the lend's own origin is its own page's; one from
/// another lend's page is that page's, by its `Origin` at this listener's
/// port; one from another page with no `Origin` (an `<img>`) is nobody's,
/// `None`. A request with no `Sec-Fetch-*` is a program's, not a page's, and
/// is the lend's own, as loopback is trusted (DESIGN §9).
fn app_of(headers: &HeaderMap, host: &Label) -> Option<Label> {
    let said = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    let (site, mode) = (said("sec-fetch-site"), said("sec-fetch-mode"));
    if site.is_none() && mode.is_none() {
        tracing::debug!(target: "origin", %host, "origin: no Sec-Fetch-*, a program on this computer, so the lend's own");
        return Some(host.clone());
    }
    if mode == Some("navigate") || matches!(site, Some("none" | "same-origin")) {
        tracing::debug!(target: "origin", %host, ?site, ?mode, "origin: a navigation, a typed address or its own origin, so the lend's own page");
        return Some(host.clone());
    }
    let port = at(said(header::HOST.as_str())?)?.1;
    let from = said(header::ORIGIN.as_str())
        .and_then(|o| o.strip_prefix("http://"))
        .and_then(at)
        .filter(|(_, p)| *p == port)
        .map(|(label, _)| label);
    tracing::debug!(target: "origin", %host, ?site, ?mode, origin = ?said(header::ORIGIN.as_str()), app = ?from.as_ref().map(Label::as_str),
        "origin: from another page, so the app its Origin names, or none");
    from
}

/// Whether the page of the lend `app` may reach the lend `label`: both
/// lent by people of a group this computer stands in, `label`'s of a kind
/// `app`'s lender declared it reads ([`Offered::reads_one`]), as
/// `/svrngs-reads` answers it ([`crate::surface::reads`]). Each lender is
/// asked what it lends now; one that did not answer is could-not-judge,
/// refused `away`, never read as lending nothing.
async fn kept_to_reads(at: &Asking, app: &Label, label: &Label) -> Result<(), Response> {
    let (reader, read) = (hex::encode(app.lender()), hex::encode(label.lender()));
    let this = at.host.actor();
    let (groups, _) = at.host.shared_with(&reader)?;
    if !groups
        .iter()
        .any(|(_, s)| s.contains_key(this) && s.contains_key(&read))
    {
        tracing::debug!(target: "origin", %app, %label, "origin: refused — the page's lender and the lend's share no group with this computer");
        return Err(not_read());
    }
    let named = |offered: Option<Vec<Offered>>, l: &Label| {
        offered.map(|o| {
            o.into_iter()
                .find(|o| Label::of(&l.lender(), o.name.as_str()) == *l)
        })
    };
    let (reading, lent) = tokio::join!(at.offered_by(&reader), at.offered_by(&read));
    match (named(reading, app), named(lent, label)) {
        (Some(Some(reading)), Some(Some(lent))) if reading.reads_one(&lent) => {
            tracing::debug!(target: "origin", %app, %label, kind = %lent.kind, "origin: the page reads the lend's kind, so it reaches it");
            Ok(())
        }
        (None, _) | (_, None) => {
            tracing::debug!(target: "origin", %app, %label, "origin: refused — a lender did not answer, so what the page reads cannot be judged");
            Err(away())
        }
        (reading, lent) => {
            tracing::debug!(target: "origin", %app, %label, reads = ?reading.flatten().map(|r| r.reads), kind = ?lent.flatten().map(|l| l.kind),
                "origin: refused — the page does not read the lend, or one of them is not lent now");
            Err(not_read())
        }
    }
}

/// A page reaching a lend it does not read.
fn not_read() -> Response {
    LendRefusal::Closed.say(
        StatusCode::FORBIDDEN,
        "this page does not read that; only what its lender declared it reads",
    )
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
    let until = door::unix_now() + OWN_PASS_FOR;
    let pass = Pass::sign(&at.key, at.host.actor(), (app.clone(), reads), until);
    HeaderValue::from_str(&pass.encode()).ok()
}

/// Where this computer's browser opens the lend `label`, with the record
/// API listening on `port`.
pub fn origin(label: &Label, port: u16) -> String {
    format!("http://{label}{LOCALHOST}:{port}/")
}

/// The routes every lend's origin is served by, for [`crate::api::router`].
pub(crate) fn router(asking: Asking) -> Router {
    Router::new().fallback(lent).with_state(asking)
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
    let stop = stop.and_then(|p| p.strip_prefix('/')?.strip_suffix("/svrngs-stop"));
    if let Some(lend) = stop
        .and_then(Label::parse)
        .filter(|_| req.method() == Method::POST)
    {
        tracing::debug!(target: "origin", %label, %app, %lend, "origin: a page stops its app using a lend");
        let pass = own_pass(&at, &app, &lend);
        return Ok(crate::home::say(&at, &lend, app, crate::door::choices::Said::Stop, pass).await);
    }
    if app != label {
        kept_to_reads(&at, &app, &label).await?;
    }
    reach(&at, label, app, req).await
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
    let door: &Door = at.door();
    if lender == door.key {
        tracing::debug!(target: "origin", %label, "origin: this computer's own lend, through its door's rules");
        let (mut parts, body) = req.into_parts();
        parts.uri = uri_of(&target)?;
        return door::lend_for(door, Who::Own(app), None, Request::from_parts(parts, body)).await;
    }
    let base = dialled(at, &label).await?;
    let Some(bridge) = base
        .strip_prefix("http://")
        .and_then(|a| a.parse::<std::net::SocketAddr>().ok())
    else {
        return Err(away());
    };
    let Some(pass) = own_pass(at, &app, &label) else {
        return Err(LendRefusal::Fault.say(
            StatusCode::INTERNAL_SERVER_ERROR,
            "this computer could not say which app asks",
        ));
    };
    req.headers_mut().insert(PASS_HEADER, pass);
    tracing::debug!(target: "origin", %label, %app, lender = hex::encode(lender), %bridge, "origin: another computer's lend, through the dial to its door, the app in this computer's own pass");
    carried(req, bridge, &target, label.as_str()).await
}

/// `target` as the address a lend is asked at, or refused as no path.
fn uri_of(target: &str) -> Result<Uri, Response> {
    target.parse().map_err(|_| {
        LendRefusal::Path.say(
            StatusCode::BAD_REQUEST,
            "this is not an address a lend can be asked",
        )
    })
}

/// The bridge to the door of the computer lending `label`, dialled as this
/// computer's key; refused for a lender standing in no group held here, so
/// no page makes this computer dial a stranger, and `away` with no path.
pub(crate) async fn dialled(at: &Asking, label: &Label) -> Result<String, Response> {
    let actor = hex::encode(label.lender());
    let key = PublicKey::from_bytes(&label.lender()).ok();
    let stands = at.door().rail.standing_anywhere(&actor).is_some();
    let Some(key) = key.filter(|_| stands) else {
        tracing::debug!(target: "origin", %label, actor, "origin: refused — the label names no computer standing in a group held here");
        return Err(LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here"));
    };
    let contact = Contact {
        key,
        relay_url: None,
        iroh_direct_addrs: Vec::new(),
    };
    let Some(base) = at.host.transport().base_url(&contact).await else {
        tracing::debug!(target: "origin", %label, actor, "origin: no dialable path to the lender's computer");
        return Err(away());
    };
    Ok(base)
}

/// The lender's computer could not be reached.
pub(crate) fn away() -> Response {
    LendRefusal::Away.say(
        StatusCode::BAD_GATEWAY,
        "the computer that lends this did not answer; try again later",
    )
}

/// `req` carried to the door at `bridge` as `target`, its answer back
/// whole, a WebSocket spliced once both sides switch.
async fn carried(
    mut req: Request,
    bridge: std::net::SocketAddr,
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
    let Some(mut answer) = door::send_to(bridge, request, label).await else {
        return Err(away());
    };
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
