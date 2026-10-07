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

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::{self, HeaderValue};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::Router;
use svrngs_core::pass::Label;

use crate::door::{self, Door, Who, LEND_ROUTE};
use crate::iroh::{key_actor, Contact, PublicKey};
use crate::node::Asking;
use crate::ring_routes::err;
use crate::Ed25519Verifier;

/// The suffix every lend's host on this computer ends in.
const LOCALHOST: &str = ".localhost";

/// The lend a request's `Host` names, `<label>.localhost[:<port>]`, or
/// `None` for any other host: the record API's.
pub(crate) fn label_of(host: Option<&HeaderValue>) -> Option<Label> {
    let host = host?.to_str().ok()?.to_ascii_lowercase();
    let name = host
        .rsplit_once(':')
        .map_or(host.as_str(), |(name, _)| name);
    Label::parse(name.strip_suffix(LOCALHOST)?)
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
async fn lent(State(at): State<Asking>, req: Request) -> Response {
    let Some(label) = label_of(req.headers().get(header::HOST)) else {
        tracing::debug!(target: "origin", host = ?req.headers().get(header::HOST), "origin: refused — no lend's host");
        return err(StatusCode::NOT_FOUND, "nothing is lent here");
    };
    reach(&at, label, req).await
}

/// `req` carried to the lend `label`, as this computer's person: by the
/// `Host` at its origin on loopback, or by the port at the room door
/// (u6-1), one body for both (DESIGN §3, "The room",
/// call 3).
pub(crate) async fn reach(at: &Asking, label: Label, req: Request) -> Response {
    let tail = req.uri().path_and_query().map_or("/", |p| p.as_str());
    let target = format!("{LEND_ROUTE}{label}{tail}");
    let lender = label.lender();
    let door: &Door = at.door();
    if lender == door.key {
        tracing::debug!(target: "origin", %label, "origin: this computer's own lend, through its door's rules");
        let (mut parts, body) = req.into_parts();
        let Ok(uri) = target.parse::<Uri>() else {
            return err(
                StatusCode::BAD_REQUEST,
                "this is not an address a lend can be asked",
            );
        };
        parts.uri = uri;
        return door::lend_for(door, Who::Own, None, Request::from_parts(parts, body)).await;
    }
    let actor = hex::encode(lender);
    let key = PublicKey::from_bytes(&lender).ok();
    let stands = door
        .rail
        .standing_anywhere(&actor, &Ed25519Verifier)
        .is_some();
    let Some(key) = key.filter(|_| stands) else {
        tracing::debug!(target: "origin", %label, actor, "origin: refused — the label names no computer standing in a group held here");
        return err(StatusCode::NOT_FOUND, "nothing is lent here");
    };
    let contact = Contact {
        key,
        relay_url: None,
        iroh_direct_addrs: Vec::new(),
    };
    let Some(base) = at.host.transport().base_url(&contact).await else {
        tracing::debug!(target: "origin", %label, actor, "origin: no dialable path to the lender's computer");
        return away();
    };
    let Some(bridge) = base
        .strip_prefix("http://")
        .and_then(|a| a.parse::<std::net::SocketAddr>().ok())
    else {
        return away();
    };
    tracing::debug!(target: "origin", %label, lender = %key_actor(&key), %bridge, "origin: another computer's lend, through the dial to its door");
    carried(req, bridge, &target, label.as_str()).await
}

/// The lender's computer could not be reached.
fn away() -> Response {
    err(
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
) -> Response {
    let upgrade = door::upgrade_of(req.headers());
    let asker = upgrade.as_ref().map(|_| hyper::upgrade::on(&mut req));
    let (mut parts, body) = req.into_parts();
    door::strip_hop_by_hop(&mut parts.headers);
    if let Some(protocol) = upgrade {
        door::keep_upgrade(&mut parts.headers, protocol);
    }
    let Ok(uri) = target.parse::<Uri>() else {
        return err(
            StatusCode::BAD_REQUEST,
            "this is not an address a lend can be asked",
        );
    };
    let mut request = axum::http::Request::new(body);
    *request.uri_mut() = uri;
    *request.method_mut() = parts.method;
    *request.headers_mut() = parts.headers;
    let Some(mut answer) = door::send_to(bridge, request, label).await else {
        return away();
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
    response.into_response()
}
