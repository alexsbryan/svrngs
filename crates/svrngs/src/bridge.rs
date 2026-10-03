//! The bridge append (DESIGN C2): on loopback only, with the capability token
//! as a bearer, every act signed by the Bridge-role key. The chat bridge
//! (`bridge/`) is the only caller:
//!
//! - `POST /append` writes `{app, op, on_behalf_of}`, refused for a speaker the
//!   guest book has removed;
//! - `POST /guest` writes what the chat reported about a speaker,
//!   `{speaker, name, event}`, into the guest book;
//! - `GET /group` reads the group's name, how many people it has, and whether
//!   anyone has ever entered it from the chat;
//! - `GET /commands` lists the chat commands the apps declare;
//! - `POST /run` hands a command or a tap, `{input, on_behalf_of}`, to the
//!   handler runner and returns what to post, `{text, buttons, private}`;
//! - `POST /due` returns the reminders due now, `[{person, text}]`, each
//!   already fulfilled in the journal.

use std::net::Ipv4Addr;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde_json::json;
use tokio::net::TcpListener;
use tracing::{debug, warn};

use svrngs_core::{Guest, GuestChange};

use crate::journal::{AppendError, Appended, BridgeAppend, GuestWritten, Invocation, JournalHandle};
use crate::runner::Command;

/// The port the bridge append listens on unless told otherwise.
pub const DEFAULT_PORT: u16 = 7570;
pub const APPEND_PATH: &str = "/append";
pub const GUEST_PATH: &str = "/guest";
pub const GROUP_PATH: &str = "/group";
pub const COMMANDS_PATH: &str = "/commands";
pub const RUN_PATH: &str = "/run";
pub const DUE_PATH: &str = "/due";

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
    commands: std::sync::Arc<Vec<Command>>,
}

pub fn router(journal: JournalHandle, token: Token, commands: Vec<Command>) -> Router {
    Router::new()
        .route(APPEND_PATH, post(append))
        .route(GUEST_PATH, post(guest))
        .route(GROUP_PATH, get(group))
        .route(COMMANDS_PATH, get(commands_list))
        .route(RUN_PATH, post(run))
        .route(DUE_PATH, post(due))
        .with_state(AppState {
            journal,
            token,
            commands: std::sync::Arc::new(commands),
        })
}

fn reply(status: StatusCode, body: serde_json::Value) -> Response {
    (status, axum::Json(body)).into_response()
}

/// `None` when the caller presented the capability token; else the refusal.
fn refuse_unless_token(s: &AppState, headers: &HeaderMap) -> Option<Response> {
    let presented = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    if presented.is_some_and(|t| s.token.admits(t)) {
        return None;
    }
    warn!(presented = presented.is_some(), "bridge:unauthorized");
    Some(reply(
        StatusCode::UNAUTHORIZED,
        json!({"error": "a capability token is required"}),
    ))
}

fn bad_request(e: serde_json::Error) -> Response {
    debug!(error = %e, "bridge:bad_request");
    reply(StatusCode::UNPROCESSABLE_ENTITY, json!({"error": e.to_string()}))
}

fn failed(e: AppendError) -> Response {
    let status = match e {
        AppendError::Removed(_) => StatusCode::FORBIDDEN,
        AppendError::Refused(_) => StatusCode::UNPROCESSABLE_ENTITY,
        AppendError::Store(_) | AppendError::Stopped => StatusCode::SERVICE_UNAVAILABLE,
        AppendError::App(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    if status.is_server_error() {
        warn!(error = %e, "bridge:failed");
    }
    let removed = matches!(e, AppendError::Removed(_));
    reply(status, json!({"error": e.to_string(), "removed": removed}))
}

async fn append(State(s): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    if let Some(r) = refuse_unless_token(&s, &headers) {
        return r;
    }
    let req: BridgeAppend = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return bad_request(e),
    };
    match s.journal.append(req).await {
        Ok(Appended::New(id)) => axum::Json(json!({"id": id, "new": true})).into_response(),
        Ok(Appended::Taken(id)) => axum::Json(json!({"id": id, "new": false})).into_response(),
        Err(e) => failed(e),
    }
}

async fn guest(State(s): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    if let Some(r) = refuse_unless_token(&s, &headers) {
        return r;
    }
    let req: Guest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return bad_request(e),
    };
    match s.journal.guest(req).await {
        Ok(GuestWritten { change, id }) => axum::Json(json!({
            "id": id,
            "new": id.is_some(),
            "arrived": change == GuestChange::Arrives,
        }))
        .into_response(),
        Err(e) => failed(e),
    }
}

async fn group(State(s): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(r) = refuse_unless_token(&s, &headers) {
        return r;
    }
    match s.journal.group().await {
        Ok(g) => axum::Json(json!({"name": g.name.as_str(), "people": g.people, "entered": g.entered})).into_response(),
        Err(e) => failed(e),
    }
}

async fn commands_list(State(s): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(r) = refuse_unless_token(&s, &headers) {
        return r;
    }
    axum::Json(s.commands.as_slice()).into_response()
}

async fn run(State(s): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    if let Some(r) = refuse_unless_token(&s, &headers) {
        return r;
    }
    let req: Invocation = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return bad_request(e),
    };
    match s.journal.run(req).await {
        Ok(reply) => axum::Json(reply).into_response(),
        Err(e) => failed(e),
    }
}

async fn due(State(s): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(r) = refuse_unless_token(&s, &headers) {
        return r;
    }
    match s.journal.due().await {
        Ok(reminders) => axum::Json(reminders).into_response(),
        Err(e) => failed(e),
    }
}
