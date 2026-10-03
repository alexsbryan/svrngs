//! The bridge append (DESIGN C2): on loopback only, `POST /append` with the
//! capability token as a bearer, writing `{app, op, on_behalf_of}` as an act
//! signed by the Bridge-role key. The chat adapter is the only caller.

use std::net::Ipv4Addr;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde_json::json;
use tokio::net::TcpListener;
use tracing::{debug, warn};

use crate::journal::{AppendError, Appended, BridgeAppend, JournalHandle};

/// The port the bridge append listens on unless told otherwise.
pub const DEFAULT_PORT: u16 = 7570;
pub const APPEND_PATH: &str = "/append";

/// The bridge's capability token: 64 hex characters.
#[derive(Clone)]
pub struct Token(String);

impl Token {
    pub fn parse(s: &str) -> Result<Self, String> {
        if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
            Ok(Self(s.to_string()))
        } else {
            Err("a capability token is 64 hex characters".into())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Constant-time in the presented token's bytes, so its timing says nothing.
    fn admits(&self, presented: &str) -> bool {
        presented.len() == self.0.len()
            && presented
                .bytes()
                .zip(self.0.bytes())
                .fold(0, |acc, (a, b)| acc | (a ^ b))
                == 0
    }
}

/// Loopback, always: the address is not a parameter.
pub async fn bind(port: u16) -> std::io::Result<TcpListener> {
    TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await
}

#[derive(Clone)]
struct AppState {
    journal: JournalHandle,
    token: Token,
}

pub fn router(journal: JournalHandle, token: Token) -> Router {
    Router::new()
        .route(APPEND_PATH, post(append))
        .with_state(AppState { journal, token })
}

async fn append(State(s): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    let presented = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    if !presented.is_some_and(|t| s.token.admits(t)) {
        warn!(presented = presented.is_some(), "bridge:unauthorized");
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({"error": "a capability token is required"})),
        )
            .into_response();
    }
    let req: BridgeAppend = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            debug!(error = %e, "bridge:bad_append");
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                axum::Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };
    match s.journal.append(req).await {
        Ok(Appended::New(id)) => axum::Json(json!({"id": id, "new": true})).into_response(),
        Ok(Appended::Taken(id)) => axum::Json(json!({"id": id, "new": false})).into_response(),
        Err(e @ AppendError::Refused(_)) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            axum::Json(json!({"error": e.to_string()})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
