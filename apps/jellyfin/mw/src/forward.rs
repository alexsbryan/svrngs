// SPDX-License-Identifier: AGPL-3.0-or-later
//! The forward: every request the door sends to this port, passed to the
//! lender's Jellyfin signed in as its person, bodies streamed both ways and
//! a WebSocket carried. Written new on hyper, as mw-0's inventory found
//! upstream has no forward that reads answers (README §1); one connection
//! to Jellyfin per request, nothing pooled or retried.

use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::Incoming;
use hyper::header::{self, HeaderMap, HeaderName, HeaderValue};
use hyper::{Method, Request, Response, StatusCode, Version};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};

use crate::page::{self, Server};
use crate::people::{On, People};
use crate::person::{Device, DeviceFrom, NotTheDoors, Who, KEY_HEADER, PERSON_HEADER};
use crate::swap;
use crate::Plant;

pub(crate) type Body = BoxBody<Bytes, hyper::Error>;

/// Every header a client may carry a Jellyfin credential in. The forward
/// sets the first itself and drops the rest, so what reaches Jellyfin is
/// the person's own credential and nothing else (Jellyfin 12 takes one
/// only from the first, DESIGN §8 U5's inventory :924).
const CREDENTIALS: [&str; 4] = [
    "authorization",
    "x-emby-authorization",
    "x-emby-token",
    "x-mediabrowser-token",
];

/// Headers that belong to one hop (RFC 9110 §7.6.1), not passed on;
/// `connection` and `upgrade` are put back for a protocol switch.
const HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-connection",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "transfer-encoding",
    "upgrade",
];

/// What every forward holds: where Jellyfin is, the server its page names,
/// the owner of the accounts, the token the page's stored sign-in holds
/// ([`swap::STAND_IN`], but under a plant), and the plant this run
/// carries, if any.
#[derive(Clone)]
pub(crate) struct At {
    pub(crate) jellyfin: SocketAddr,
    pub(crate) server: Server,
    pub(crate) people: People,
    pub(crate) page_token: String,
    pub(crate) plant: Option<Plant>,
}

/// Serve `listener` until the process ends.
pub(crate) async fn serve(listener: TcpListener, at: At) {
    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "mw: a connection could not be accepted");
                continue;
            }
        };
        let at = at.clone();
        tokio::spawn(async move {
            let service = hyper::service::service_fn(move |req| {
                let at = at.clone();
                async move { Ok::<_, std::convert::Infallible>(forward(at, req).await) }
            });
            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .with_upgrades()
                .await
            {
                tracing::debug!(%peer, error = %e, "mw: a connection ended with an error");
            }
        });
    }
}

fn said(status: StatusCode, why: impl Into<String>) -> Response<Body> {
    let mut r = Response::new(
        Full::new(Bytes::from(why.into()))
            .map_err(|never| match never {})
            .boxed(),
    );
    *r.status_mut() = status;
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    r
}

/// The person a request is from: the door's word, or, under a plant, what
/// the plant makes of it.
fn who(headers: &HeaderMap, plant: Option<Plant>) -> Result<Who, NotTheDoors> {
    let everyone = || Who {
        key: crate::person::PersonKey::parse(&"e".repeat(64)).expect("64 hex"),
        name: "everyone".into(),
    };
    match (Who::of(headers), plant) {
        (Ok(_), Some(Plant::OneUser | Plant::OneDevice)) => Ok(everyone()),
        (Err(NotTheDoors::NoKey), Some(Plant::LetKeyless)) => Ok(everyone()),
        (w, _) => w,
    }
}

/// The protocol a message asks to switch to, if it does.
fn upgrade_of(headers: &HeaderMap) -> Option<HeaderValue> {
    let asks = headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .any(|t| t.trim().eq_ignore_ascii_case("upgrade"));
    asks.then(|| headers.get(header::UPGRADE).cloned()).flatten()
}

fn drop_hop(headers: &mut HeaderMap, switching: Option<HeaderValue>) {
    for h in HOP {
        headers.remove(h);
    }
    if let Some(protocol) = switching {
        headers.insert(header::CONNECTION, HeaderValue::from_static("upgrade"));
        headers.insert(header::UPGRADE, protocol);
    }
}

async fn forward(at: At, mut req: Request<Incoming>) -> Response<Body> {
    let path = req.uri().path().to_string();
    let who = match who(req.headers(), at.plant) {
        Ok(w) => w,
        Err(why) => {
            tracing::info!(%path, %why, "mw: refused — not a request svrngs's door forwarded");
            return said(
                StatusCode::FORBIDDEN,
                format!("This port answers only what svrngs's door forwards: {why}.\n"),
            );
        }
    };
    let device = match at.plant {
        Some(Plant::OneDevice) => Some((Device::one(), DeviceFrom::Planted)),
        _ => Device::of(req.headers(), req.uri().query(), &who),
    };
    tracing::debug!(%path, key = %who.key.as_str(), device = ?device.as_ref().map(|(d, from)| (&d.id, from)), "mw: asked");
    let on = match device {
        Some((d, _)) => On::Device(d),
        None if swap::asks_sign_in(req.uri().query()) => On::Last,
        None => On::Nobody,
    };
    let signed = match at.people.signed(who.clone(), on).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(%path, key = %who.key.as_str(), error = %e,
                "mw: refused — the person could not be signed in to Jellyfin");
            return said(
                StatusCode::BAD_GATEWAY,
                format!("Jellyfin did not sign {} in: {e}\n", who.name),
            );
        }
    };
    // jellyfin-web's page, which carries the stored sign-in (page.rs): asked
    // whole every time, never answered "not modified" from a copy that was
    // written for this browser before.
    let page_asked =
        req.method() == Method::GET && path.starts_with("/web/") && (path.ends_with('/') || path.ends_with(".html"));

    let switching = upgrade_of(req.headers());
    let asker_side = switching.as_ref().map(|_| hyper::upgrade::on(&mut req));
    let (mut parts, body) = req.into_parts();
    for h in CREDENTIALS {
        parts.headers.remove(h);
    }
    parts.headers.remove(KEY_HEADER);
    parts.headers.remove(PERSON_HEADER);
    // Every answer comes as Jellyfin wrote it, so a token in it can be read
    // (swap.rs); a compressed one would carry it unseen.
    parts.headers.remove(header::ACCEPT_ENCODING);
    if page_asked {
        parts.headers.remove(header::IF_NONE_MATCH);
        parts.headers.remove(header::IF_MODIFIED_SINCE);
    }
    drop_hop(&mut parts.headers, switching.clone());
    let token = signed.on.as_ref().map(|(_, t)| t.as_str());
    match &signed.on {
        Some((device, token)) => {
            let Ok(signed) = HeaderValue::from_str(&device.signed(token)) else {
                tracing::warn!(%path, "mw: refused — the person's device cannot be put in a header");
                return said(
                    StatusCode::BAD_REQUEST,
                    "the device this asks from cannot be named to Jellyfin\n",
                );
            };
            parts.headers.insert(HeaderName::from_static("authorization"), signed);
        }
        None => tracing::debug!(%path, key = %who.key.as_str(),
            "mw: the request names no device — passed on signed in as nobody, its credentials dropped"),
    }
    parts.version = Version::HTTP_11;
    let asked = parts
        .uri
        .path_and_query()
        .map_or("/".to_string(), |p| p.as_str().to_string());
    let target = match swap::inward(&asked, token) {
        Some(t) => {
            tracing::debug!(%path, key = %who.key.as_str(), signed_in = token.is_some(),
                "mw: the query's credential set to the person's own, or dropped with none");
            t
        }
        None => asked,
    };
    let Ok(uri) = target.parse() else {
        return said(StatusCode::BAD_REQUEST, "the request target is not a path\n");
    };
    parts.uri = uri;
    let request = Request::from_parts(parts, body);

    let Some(mut answer) = send(at.jellyfin, request).await else {
        return said(
            StatusCode::BAD_GATEWAY,
            "Jellyfin is not answering on the lender's computer\n",
        );
    };
    let switched = upgrade_of(answer.headers()).filter(|_| answer.status() == StatusCode::SWITCHING_PROTOCOLS);
    let service_side = switched.as_ref().map(|_| hyper::upgrade::on(&mut answer));
    let (mut parts, body) = answer.into_parts();
    drop_hop(&mut parts.headers, switched.clone());
    if let (Some(asker), Some(service)) = (asker_side, service_side) {
        tracing::debug!(%path, key = %who.key.as_str(), "mw: Jellyfin switched protocols; splicing the two");
        tokio::spawn(splice(asker, service, path.clone()));
        return Response::from_parts(parts, body.boxed());
    }
    if let Some(token) = token {
        swap_headers(&mut parts.headers, token, &path);
    }
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let page = (page_asked && parts.status == StatusCode::OK && content_type.starts_with("text/html"))
        .then_some(())
        .and_then(|()| match at.plant {
            Some(Plant::NoSignIn) => {
                tracing::warn!(%path, "mw: planted — the page passed on without its stored sign-in");
                None
            }
            _ => Some(page::script(&at.server, &signed.user, &at.page_token)),
        });
    let swapping = token.filter(|_| swap::textual(&content_type));
    if page.is_none() && swapping.is_none() {
        tracing::debug!(%path, key = %who.key.as_str(), status = parts.status.as_u16(), "mw: answered; streaming it back");
        return Response::from_parts(parts, body.boxed());
    }
    let read = match body.collect().await {
        Ok(b) => b.to_bytes(),
        Err(e) => {
            tracing::warn!(%path, error = %e, "mw: Jellyfin's answer could not be read whole");
            return said(StatusCode::BAD_GATEWAY, "Jellyfin's answer broke off\n");
        }
    };
    let mut bytes = read.to_vec();
    if let Some(script) = page {
        match page::written(&bytes, &script) {
            Some(written) => {
                tracing::debug!(%path, key = %who.key.as_str(), user = %signed.user,
                    "mw: jellyfin-web's page carries the person's stored sign-in");
                bytes = written;
                parts.headers.remove(header::ETAG);
                parts.headers.remove(header::LAST_MODIFIED);
                parts
                    .headers
                    .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            }
            None => tracing::warn!(%path, "mw: jellyfin-web's page is not text — passed on without a stored sign-in"),
        }
    }
    if let Some(token) = swapping {
        match swap::outward(&bytes, token) {
            Some((out, n)) => {
                tracing::debug!(%path, key = %who.key.as_str(), swapped = n,
                    "mw: the person's token swapped for the stand-in in Jellyfin's answer");
                bytes = out;
            }
            None => tracing::debug!(%path, "mw: a textual answer read; it holds no token"),
        }
    }
    parts
        .headers
        .insert(header::CONTENT_LENGTH, HeaderValue::from(bytes.len()));
    tracing::debug!(%path, key = %who.key.as_str(), status = parts.status.as_u16(), bytes = bytes.len(), "mw: answered, read whole");
    Response::from_parts(
        parts,
        Full::new(Bytes::from(bytes)).map_err(|never| match never {}).boxed(),
    )
}

/// Every answer header holding the person's token has it swapped for the
/// stand-in (a `Location` Jellyfin built from what it was asked).
fn swap_headers(headers: &mut HeaderMap, token: &str, path: &str) {
    for value in headers.values_mut() {
        let Some((out, _)) = swap::outward(value.as_bytes(), token) else {
            continue;
        };
        match HeaderValue::from_bytes(&out) {
            Ok(v) => {
                tracing::debug!(%path, "mw: the person's token swapped for the stand-in in an answer's header");
                *value = v;
            }
            Err(_) => {
                tracing::warn!(%path, "mw: a header holding the token could not be rewritten — emptied");
                *value = HeaderValue::from_static("");
            }
        }
    }
}

/// One connection to Jellyfin, one request on it: the answer, or `None`
/// (traced) if Jellyfin could not be reached.
async fn send(jellyfin: SocketAddr, request: Request<Incoming>) -> Option<Response<Incoming>> {
    let stream = match TcpStream::connect(jellyfin).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(%jellyfin, error = %e, "mw: Jellyfin could not be reached");
            return None;
        }
    };
    let (mut sender, conn) = match hyper::client::conn::http1::handshake(TokioIo::new(stream)).await {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!(%jellyfin, error = %e, "mw: Jellyfin's connection could not be opened");
            return None;
        }
    };
    tokio::spawn(async move {
        if let Err(e) = conn.with_upgrades().await {
            tracing::debug!(error = %e, "mw: Jellyfin's connection ended with an error");
        }
    });
    match sender.send_request(request).await {
        Ok(a) => Some(a),
        Err(e) => {
            tracing::warn!(%jellyfin, error = %e, "mw: Jellyfin did not answer");
            None
        }
    }
}

/// Copy both ways between the asker's switched connection and Jellyfin's,
/// until either ends.
async fn splice(asker: hyper::upgrade::OnUpgrade, service: hyper::upgrade::OnUpgrade, path: String) {
    match (asker.await, service.await) {
        (Ok(a), Ok(s)) => {
            let (mut a, mut s) = (TokioIo::new(a), TokioIo::new(s));
            match tokio::io::copy_bidirectional(&mut a, &mut s).await {
                Ok((up, down)) => tracing::debug!(%path, up, down, "mw: a switched connection ended"),
                Err(e) => tracing::debug!(%path, error = %e, "mw: a switched connection ended with an error"),
            }
        }
        (a, s) => tracing::debug!(%path, asker = a.is_ok(), jellyfin = s.is_ok(),
            "mw: a protocol switch did not complete on both sides"),
    }
}
