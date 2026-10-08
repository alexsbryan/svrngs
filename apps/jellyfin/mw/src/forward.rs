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
use hyper::{Request, Response, StatusCode, Version};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};

use crate::people::People;
use crate::person::{Device, NotTheDoors, Who, KEY_HEADER, PERSON_HEADER};
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

/// What every forward holds: where Jellyfin is, the owner of the accounts,
/// and the plant this run carries, if any.
#[derive(Clone)]
pub(crate) struct At {
    pub(crate) jellyfin: SocketAddr,
    pub(crate) people: People,
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
        (Ok(_), Some(Plant::OneUser)) => Ok(everyone()),
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
    let device = Device::of(req.headers(), req.uri().query(), &who);
    tracing::debug!(%path, key = %who.key.as_str(), device = ?device.as_ref().map(|(d, from)| (&d.id, from)), "mw: asked");
    let device = device.map(|(d, _)| d);
    let token = match at.people.token(who.clone(), device.clone()).await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(%path, key = %who.key.as_str(), error = %e,
                "mw: refused — the person could not be signed in to Jellyfin");
            return said(
                StatusCode::BAD_GATEWAY,
                format!("Jellyfin did not sign {} in: {e}\n", who.name),
            );
        }
    };

    let switching = upgrade_of(req.headers());
    let asker_side = switching.as_ref().map(|_| hyper::upgrade::on(&mut req));
    let (mut parts, body) = req.into_parts();
    for h in CREDENTIALS {
        parts.headers.remove(h);
    }
    parts.headers.remove(KEY_HEADER);
    parts.headers.remove(PERSON_HEADER);
    drop_hop(&mut parts.headers, switching.clone());
    match device.zip(token) {
        Some((device, token)) => {
            let Ok(signed) = HeaderValue::from_str(&device.signed(&token)) else {
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
    let target = parts
        .uri
        .path_and_query()
        .map_or("/".to_string(), |p| p.as_str().to_string());
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
    }
    tracing::debug!(%path, key = %who.key.as_str(), status = parts.status.as_u16(), "mw: answered; streaming it back");
    Response::from_parts(parts, body.boxed())
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
