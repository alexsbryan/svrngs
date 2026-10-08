// SPDX-License-Identifier: AGPL-3.0-or-later
//! The sign-in's answer to svrngs's door (DESIGN §3, "The room", call 6,
//! the contract; crates/svrngs-node/src/door/sign_in.rs, its asker): asked
//! `POST /` with the person's `x-svrngs-key` and `x-svrngs-person` and
//! nothing else, it answers `200` and `{secret, headers, script}` — the
//! person's Jellyfin token, the `authorization` signing each of their
//! requests in on their own device, and the script writing jellyfin-web's
//! stored sign-in. Anything else it refuses, and the door refuses the
//! person's request in turn, never carrying it unsigned.

use bytes::Bytes;
use http_body_util::Full;
use hyper::header::{self, HeaderValue};
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::page::{self, Server};
use crate::people::People;
use crate::person::{NotTheDoors, PersonKey, Who, STAND_IN_HEADER};
use crate::Plant;

/// What every answer holds: the server the stored sign-in names, the owner
/// of the accounts, the token the stored sign-in holds under a plant (else
/// the door's stand-in, as its ask names it), and the plant this run
/// carries, if any.
#[derive(Clone)]
pub(crate) struct At {
    pub(crate) server: Server,
    pub(crate) people: People,
    pub(crate) page_token: Option<String>,
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
                async move { Ok::<_, std::convert::Infallible>(answer(at, req).await) }
            });
            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .await
            {
                tracing::debug!(%peer, error = %e, "mw: a connection ended with an error");
            }
        });
    }
}

fn said(status: StatusCode, kind: &'static str, body: String) -> Response<Full<Bytes>> {
    let mut r = Response::new(Full::new(Bytes::from(body)));
    *r.status_mut() = status;
    r.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(kind));
    r
}

/// The person an ask is for: the door's word, or, under a plant, what the
/// plant makes of it.
fn who(req: &Request<hyper::body::Incoming>, plant: Option<Plant>) -> Result<Who, NotTheDoors> {
    let everyone = || Who {
        key: PersonKey::parse(&"e".repeat(64)).expect("64 hex"),
        name: "everyone".into(),
    };
    match (Who::of(req.headers()), plant) {
        (Ok(_), Some(Plant::OneUser | Plant::OneDevice)) => Ok(everyone()),
        (Err(NotTheDoors::NoKey), Some(Plant::LetKeyless)) => Ok(everyone()),
        (w, _) => w,
    }
}

async fn answer(at: At, req: Request<hyper::body::Incoming>) -> Response<Full<Bytes>> {
    let text = "text/plain; charset=utf-8";
    if req.method() != Method::POST || req.uri().path() != "/" {
        tracing::info!(method = %req.method(), path = %req.uri().path(), "mw: refused — not the door's ask");
        return said(
            StatusCode::NOT_FOUND,
            text,
            "This port answers svrngs's door's POST / alone.\n".into(),
        );
    }
    let who = match who(&req, at.plant) {
        Ok(w) => w,
        Err(why) => {
            tracing::info!(%why, "mw: refused — not an ask svrngs's door made");
            return said(
                StatusCode::FORBIDDEN,
                text,
                format!("This port answers only svrngs's door: {why}.\n"),
            );
        }
    };
    let Some(stand_in) = req.headers().get(STAND_IN_HEADER).and_then(|v| v.to_str().ok()) else {
        tracing::info!("mw: refused — the ask names no stand-in");
        return said(
            StatusCode::FORBIDDEN,
            text,
            format!("This port answers only svrngs's door: no {STAND_IN_HEADER}.\n"),
        );
    };
    let page_token = at.page_token.clone().unwrap_or(stand_in.to_string());
    tracing::info!(key = %who.key.as_str(), person = %who.name, "mw: the door asks who a person is to Jellyfin");
    let signed = match at.people.signed(who.clone()).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(key = %who.key.as_str(), error = %e, "mw: refused — the person could not be signed in to Jellyfin");
            return said(
                StatusCode::BAD_GATEWAY,
                text,
                format!("Jellyfin did not sign {} in: {e}\n", who.name),
            );
        }
    };
    let script = match at.plant {
        Some(Plant::NoSignIn) => {
            tracing::warn!("mw: planted — answered with no script, so no stored sign-in");
            None
        }
        _ => Some(page::script(&at.server, &signed.user, &page_token, &signed.device.id)),
    };
    tracing::info!(key = %who.key.as_str(), user = %signed.user, device = %signed.device.id,
        "mw: answered the door — the person's token, its authorization and the stored sign-in");
    let body = serde_json::json!({
        "secret": signed.token,
        "headers": { "authorization": signed.device.signed(&signed.token) },
        "script": script,
    });
    said(StatusCode::OK, "application/json", body.to_string())
}
