// SPDX-License-Identifier: AGPL-3.0-or-later
//! The provider's back channel: discovery, the keys, the token and
//! userinfo, on the record API's listener (SIGN_IN_WITH_SVRNGS §5 "The back
//! channel", §5a "What the provider must do").
//!
//! **One issuer string, exact**: [`issuer`] spells it from the API's port
//! and the lend's name, and discovery and every ID token's `iss` call it, so
//! they are byte-equal. No trailing slash.
//!
//! **A code stays in this run's memory** ([`Codes`]): one use, expiring
//! after [`CODE_LIFETIME`], bound to the person, the lend, the client, the
//! redirect and the PKCE challenge ([`Code`]). Tokens are never in the
//! record and never on disk.
//!
//! **The lender's declaration and key are read from one place**: the client
//! id and secret from [`crate::lend::SignInWithSvrngs`], the ES256 key from
//! [`super::Key`]. Discovery omits a PAR endpoint and lists `ES256` alone;
//! the token route accepts `client_secret_basic`, `client_secret_post` and a
//! public client with none; PKCE S256 is checked when a challenge was bound
//! and none is required; a nonce is echoed when one was sent.
//!
//! Every refusal traces at debug, naming the fact that refused it.

use std::collections::HashMap;
use std::path::PathBuf;
#[expect(
    clippy::disallowed_types,
    reason = "the code store is one process's map, read and written by its own routes"
)]
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use ring::rand::SystemRandom;
use serde::Deserialize;
use svrngs_core::token::{Claims, Group};

use crate::lend::{self, Lend, LendName, SignInWithSvrngs, Source};

use super::{Key, KeyFailure};

/// Where the person's browser is sent to sign in: on the lend's own origin,
/// which the door answers for every lend (SIGN_IN_WITH_SVRNGS §5 "The front
/// channel"). The provider names it in discovery; the door's `lend_for`
/// answers it.
pub const AUTHORIZE_PATH: &str = "/.svrngs/oidc/authorize";

/// How long a one-time code may be redeemed after it is minted.
pub(crate) const CODE_LIFETIME: i64 = 60;

/// How long the ID and access tokens live.
pub(crate) const TOKEN_LIFETIME: i64 = 600;

/// The lender's one issuer string for the lend `name`, on the API at
/// `port`: `http://127.0.0.1:<port>/oidc/<name>`, no trailing slash. The one
/// home of the string discovery and every ID token say.
pub fn issuer(port: u16, name: &str) -> String {
    format!("http://127.0.0.1:{port}/oidc/{name}")
}

/// Seconds since the epoch, the clock the codes and tokens are held to.
#[expect(
    clippy::disallowed_methods,
    reason = "the provider is the adapter that reads the clock for a code and a token"
)]
pub(crate) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// The provider's state: the lender's key, the codes kept in this run's
/// memory, and where the lends' declarations are read.
#[derive(Clone)]
pub struct Provider {
    key: Arc<Key>,
    codes: Arc<Codes>,
    data_dir: PathBuf,
    port: u16,
}

impl Provider {
    /// The provider for the API at `port`, with the lender's key read or
    /// made once in `data_dir` (the key's one home).
    pub fn new(data_dir: PathBuf, port: u16) -> Result<Self, KeyFailure> {
        let key = super::load_or_create(&data_dir)?;
        tracing::debug!(target: "oidc", port, "oidc: the provider's back channel is ready");
        Ok(Self {
            key: Arc::new(key),
            codes: Arc::new(Codes::new()),
            data_dir,
            port,
        })
    }

    /// The routes, for [`crate::api::router`] to serve behind its loopback
    /// check and its page guard.
    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/oidc/{lend}/.well-known/openid-configuration",
                get(discovery),
            )
            .route("/oidc/{lend}/keys", get(keys))
            .route("/oidc/{lend}/token", post(token))
            .route("/oidc/{lend}/userinfo", get(userinfo))
            .with_state(self)
    }

    /// The lend `raw` names and the client it declares, when it signs
    /// people in with svrngs.
    fn declared(&self, raw: &str) -> Option<(LendName, Lend, SignInWithSvrngs)> {
        let name = match LendName::parse(raw) {
            Ok(name) => name,
            Err(why) => {
                tracing::debug!(target: "oidc", raw, %why, "oidc: refused — not a lend's name");
                return None;
            }
        };
        match lend::read(&self.data_dir, &name) {
            Ok(Some(mut lend)) => lend
                .sign_in_with_svrngs
                .take()
                .map(|client| (name, lend, client)),
            Ok(None) => {
                tracing::debug!(target: "oidc", lend = %name, "oidc: refused — this lend does not sign people in with svrngs");
                None
            }
            Err(why) => {
                tracing::warn!(target: "oidc", lend = %name, %why, "oidc: refused — its declaration cannot be read");
                None
            }
        }
    }

    /// Mint `code`, answering the one string the token route redeems.
    pub(crate) fn mint(&self, code: Code) -> String {
        self.codes.mint(code)
    }
}

/// A one-time code the door mints at the authorize path and the token route
/// redeems: the person, the lend and client it was made for, the redirect
/// and the PKCE challenge it is bound to, and the nonce to echo.
#[derive(Debug, Clone)]
pub(crate) struct Code {
    pub(crate) lend: String,
    pub(crate) client: String,
    pub(crate) sub: String,
    pub(crate) name: String,
    pub(crate) username: String,
    pub(crate) group: Group,
    pub(crate) redirect: String,
    pub(crate) challenge: Option<String>,
    pub(crate) method: Option<String>,
    pub(crate) nonce: Option<String>,
    expires: i64,
    used: bool,
}

impl Code {
    /// A code for `lend`'s client and the person, bound to `redirect`, the
    /// PKCE `challenge` and the `nonce`, expiring after [`CODE_LIFETIME`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        lend: &str,
        client: &str,
        sub: &str,
        name: &str,
        username: &str,
        group: Group,
        redirect: &str,
        challenge: Option<&str>,
        method: Option<&str>,
        nonce: Option<&str>,
    ) -> Self {
        Self {
            lend: lend.into(),
            client: client.into(),
            sub: sub.into(),
            name: name.into(),
            username: username.into(),
            group,
            redirect: redirect.into(),
            challenge: challenge.map(Into::into),
            method: method.map(Into::into),
            nonce: nonce.map(Into::into),
            expires: now() + CODE_LIFETIME,
            used: false,
        }
    }
}

/// The codes this run has minted, in memory and nowhere else: one use, and
/// expired ones dropped on each mint and redemption.
#[expect(
    clippy::disallowed_types,
    reason = "the code store is one process's map, read and written by its own routes"
)]
struct Codes {
    inner: Mutex<HashMap<String, Code>>,
    rng: SystemRandom,
}

impl Codes {
    #[expect(
        clippy::disallowed_types,
        reason = "the code store is one process's map, read and written by its own routes"
    )]
    fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            rng: SystemRandom::new(),
        }
    }

    fn mint(&self, code: Code) -> String {
        use ring::rand::SecureRandom;
        let mut bytes = [0u8; 32];
        let _ = self.rng.fill(&mut bytes);
        let text = URL_SAFE_NO_PAD.encode(bytes);
        tracing::debug!(target: "oidc", lend = %code.lend, client = %code.client, "oidc: minted a one-time code");
        let mut codes = self.inner.lock().expect("the code store's lock");
        codes.retain(|_, c| c.expires > now());
        codes.insert(text.clone(), code);
        text
    }

    fn redeem(&self, text: &str) -> Result<Code, &'static str> {
        let mut codes = self.inner.lock().expect("the code store's lock");
        codes.retain(|_, c| c.expires > now());
        let Some(code) = codes.get_mut(text) else {
            return Err("no code was minted with that value, or it lapsed");
        };
        if code.used {
            return Err("that code was already redeemed");
        }
        code.used = true;
        Ok(code.clone())
    }
}

/// `GET /oidc/{lend}/.well-known/openid-configuration`.
async fn discovery(State(p): State<Provider>, Path(lend): Path<String>) -> Response {
    let Some((name, declared, _client)) = p.declared(&lend) else {
        return missing(&lend);
    };
    let Source::Port(port) = declared.source else {
        tracing::debug!(target: "oidc", lend = %name, "oidc: refused — a carried app names no lend port for the authorize address");
        return missing(name.as_str());
    };
    let issuer = issuer(p.port, name.as_str());
    tracing::debug!(target: "oidc", %issuer, "oidc: discovery answered");
    Json(serde_json::json!({
        "issuer": issuer,
        "authorization_endpoint": format!("http://localhost:{port}{AUTHORIZE_PATH}"),
        "token_endpoint": format!("{issuer}/token"), "userinfo_endpoint": format!("{issuer}/userinfo"),
        "jwks_uri": format!("{issuer}/keys"),
        "response_types_supported": ["code"], "grant_types_supported": ["authorization_code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["ES256"],
        "scopes_supported": ["openid", "profile", "groups"],
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "none"],
        "code_challenge_methods_supported": ["S256"],
        "claims_supported": ["sub", "iss", "aud", "exp", "iat", "name", "preferred_username", "groups"],
    }))
    .into_response()
}

/// `GET /oidc/{lend}/keys`: the JWKS from the key that signs.
async fn keys(State(p): State<Provider>, Path(lend): Path<String>) -> Response {
    if p.declared(&lend).is_none() {
        return missing(&lend);
    }
    ([(header::CONTENT_TYPE, "application/json")], p.key.jwks()).into_response()
}

/// The token route's form.
#[derive(Debug, Deserialize)]
struct TokenForm {
    grant_type: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    code_verifier: Option<String>,
}

/// `POST /oidc/{lend}/token`: the stock authorization-code exchange.
async fn token(
    State(p): State<Provider>,
    Path(lend): Path<String>,
    headers: HeaderMap,
    Form(form): Form<TokenForm>,
) -> Response {
    let Some((name, _declared, config)) = p.declared(&lend) else {
        return missing(&lend);
    };
    if form.grant_type.as_deref() != Some("authorization_code") {
        return refuse(
            StatusCode::BAD_REQUEST,
            "unsupported_grant_type",
            "the only grant is authorization_code",
        );
    }
    let (client_id, secret) = match client_of(&headers, &form) {
        Ok(client) => client,
        Err(why) => return refuse(StatusCode::UNAUTHORIZED, "invalid_client", why),
    };
    if client_id != config.client_id {
        return refuse(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            format!("{client_id} is not this lend's client"),
        );
    }
    if !secret_ok(config.client_secret.as_deref(), secret.as_deref()) {
        return refuse(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "the client's secret is not this lend's",
        );
    }
    let Some(text) = form.code.as_deref() else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "no code was presented",
        );
    };
    let code = match p.codes.redeem(text) {
        Ok(code) => code,
        Err(why) => return refuse(StatusCode::BAD_REQUEST, "invalid_grant", why),
    };
    if code.lend != name.as_str() {
        return refuse(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "the code was minted for another lend",
        );
    }
    if code.client != client_id {
        return refuse(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "the code was minted for another client",
        );
    }
    if form.redirect_uri.as_deref() != Some(code.redirect.as_str()) {
        return refuse(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "the redirect does not match the one the code was minted for",
        );
    }
    if code.challenge.is_some() && code.method.as_deref() != Some("S256") {
        return refuse(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "the code's PKCE method is not S256",
        );
    }
    if let Some(challenge) = &code.challenge {
        let Some(verifier) = form.code_verifier.as_deref() else {
            return refuse(
                StatusCode::BAD_REQUEST,
                "invalid_grant",
                "the code was minted with a PKCE challenge and no verifier was sent",
            );
        };
        if !verifier_matches(challenge, verifier) {
            return refuse(
                StatusCode::BAD_REQUEST,
                "invalid_grant",
                "the code verifier does not answer the code's challenge",
            );
        }
    }
    let issuer = issuer(p.port, name.as_str());
    let mut claims = Claims::new(
        &issuer,
        &code.sub,
        &client_id,
        &code.name,
        &code.username,
        code.group,
        now(),
        TOKEN_LIFETIME,
    );
    claims.nonce = code.nonce;
    let signed = match p.key.token(claims) {
        Ok(signed) => signed,
        Err(why) => {
            tracing::warn!(target: "oidc", %why, "oidc: refused — the token cannot be signed");
            return refuse(
                StatusCode::INTERNAL_SERVER_ERROR,
                "server_error",
                "this computer cannot sign the token",
            );
        }
    };
    tracing::debug!(target: "oidc", %issuer, client = %client_id, sub = %code.sub, "oidc: issued a token");
    Json(serde_json::json!({ "access_token": signed, "token_type": "Bearer", "expires_in": TOKEN_LIFETIME, "id_token": signed }))
        .into_response()
}

/// `GET /oidc/{lend}/userinfo`: the claims the access token carries, when
/// the token is this computer's, for this lend's client, and unexpired.
async fn userinfo(
    State(p): State<Provider>,
    Path(lend): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some((name, _declared, config)) = p.declared(&lend) else {
        return missing(&lend);
    };
    let token = match bearer(&headers) {
        Ok(token) => token,
        Err(why) => return refuse(StatusCode::UNAUTHORIZED, "invalid_token", why),
    };
    let issuer = issuer(p.port, name.as_str());
    let mut claims = match p.key.verified(token, &issuer, &config.client_id, now()) {
        Ok(claims) => claims,
        Err(why) => return refuse(StatusCode::UNAUTHORIZED, "invalid_token", why),
    };
    if let Some(object) = claims.as_object_mut() {
        object.remove("nonce");
    }
    tracing::debug!(target: "oidc", %issuer, client = %config.client_id, "oidc: userinfo answered");
    Json(claims).into_response()
}

/// The client and the secret it presented, by `client_secret_basic` or
/// `client_secret_post`; a client with neither names no secret.
fn client_of(headers: &HeaderMap, form: &TokenForm) -> Result<(String, Option<String>), String> {
    let basic = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Basic "));
    if let Some(basic) = basic {
        let decoded = STANDARD
            .decode(basic)
            .map_err(|_| "the Basic credential is not base64".to_string())?;
        let text = String::from_utf8(decoded)
            .map_err(|_| "the Basic credential is not text".to_string())?;
        let (id, secret) = text
            .split_once(':')
            .ok_or("the Basic credential names no secret")?;
        if form.client_secret.is_some() {
            return Err("two client authentications were sent".into());
        }
        return Ok((id.into(), Some(secret.into())));
    }
    let id = form.client_id.as_ref().ok_or("no client id was sent")?;
    Ok((id.clone(), form.client_secret.clone()))
}

/// Whether `presented` is the secret the declaration keeps: none for a
/// public client, and one compared in constant time.
fn secret_ok(declared: Option<&str>, presented: Option<&str>) -> bool {
    match (declared, presented) {
        (Some(want), Some(got)) => same(want.as_bytes(), got.as_bytes()),
        (None, None) => true,
        _ => false,
    }
}

/// Whether two secrets are the same bytes, comparing every byte so a
/// mismatch costs the same wherever it is.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Whether `verifier` answers the S256 `challenge` the code was bound to.
fn verifier_matches(challenge: &str, verifier: &str) -> bool {
    let digest = ring::digest::digest(&ring::digest::SHA256, verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest.as_ref()) == challenge
}

/// The Bearer token from a userinfo request's `Authorization`, or why none.
fn bearer(headers: &HeaderMap) -> Result<&str, &'static str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or("no Bearer access token was sent")
}

/// `lend` is not one this computer signs people in to, or not one at all.
fn missing(lend: &str) -> Response {
    tracing::debug!(target: "oidc", lend = %lend, "oidc: refused — no lend of that name signs people in with svrngs");
    refuse(
        StatusCode::NOT_FOUND,
        "not_found",
        "no lend of that name signs people in with svrngs",
    )
}

/// A refusal, in an OAuth error's words, traced with the fact that refused.
fn refuse(status: StatusCode, error: &str, why: impl std::fmt::Display) -> Response {
    tracing::debug!(target: "oidc", error, why = %why, "oidc: refused");
    (
        status,
        Json(serde_json::json!({ "error": error, "error_description": why.to_string() })),
    )
        .into_response()
}
