// SPDX-License-Identifier: AGPL-3.0-or-later
//! A browser told from a program (give-session; svrn-docs
//! `rings/current/GIVE.md` §5): the home page and each lend's origin answer
//! a browser only with this computer's session for that host, and a program
//! only with its token ([`crate::api::Bearer`]). Loopback alone let in a
//! container on a Mac running podman, which reaches the Mac's `127.0.0.1`
//! through gvproxy (GIVE.md §5, measured 2026-10-25).
//!
//! **An open** is a code, single-use and gone in a minute (the sign-in
//! provider's `Codes`), minted by `POST /v1/open` behind the token
//! (`svrngs open`, install.sh's open) or by the home page for a lend it
//! opens (`crate::home`'s `/to`), and redeemed at [`OPEN_PATH`] on the host
//! it names, where it becomes that host's cookie, the browser coming back
//! there once to show it kept it before going on. A host's cookie is the
//! token's HMAC over the host's first name: none is kept, a restart keeps
//! every browser opened (a bookmark keeps working), and one host's cookie
//! opens no other. Neither the cookie nor the token is carried on to a
//! lend's service ([`Sessions::strip`]).
//!
//! A cookie names a host, not a port (RFC 6265 §8.5), so a page made to
//! visit `svrngs.localhost` at another port sends that port's listener the
//! home page's cookie: accepted, as a program there could read the person's
//! files anyway; the check is against programs that cannot.

use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::post;
use axum::{Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use std::sync::Arc;
use svrngs_core::pass::Label;

use crate::api::Bearer;
use crate::oidc::provider::{same, Codes};

/// Where an open's code is redeemed, on the host it opens: a path no app
/// serves, as the reads' (`crate::surface::READS_PATH`).
pub(crate) const OPEN_PATH: &str = "/svrngs-open";

/// Why a browser that came back to [`OPEN_PATH`] showing no session has
/// none ([`Sessions::redeem`]), in the person's words.
const UNKEPT: &str = "<p>This browser didn’t keep what svrngs needs to open this page: it keeps no cookies for sites on localhost. Let it keep them, then open this again.</p>";

/// Where a lend's page reads another lend, `<AS_PATH><cap>/<path>` at that
/// lend's origin ([`Sessions::cap`]; DESIGN §3, "Lent apps", call 6).
pub(crate) const AS_PATH: &str = "/svrngs-as/";

/// The home page's first name, as its host says it (`crate::home`).
pub(crate) const HOME: &str = "svrngs";

/// The cookie's name.
const COOKIE: &str = "svrngs";

/// How long a browser keeps its session: 400 days, the most a browser
/// keeps a cookie (RFC 6265bis §5.5's cap, which Chromium holds to).
const KEPT_FOR: u32 = 400 * 24 * 3600;

/// Where install.sh put the opener, in the person's words.
const OPENER: &str = if cfg!(target_os = "macos") {
    "Applications"
} else {
    "applications menu"
};

/// What a request showed, of what this computer gave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shown {
    /// The token: a program the person runs.
    Token,
    /// The host's cookie, from a browser, whose page `crate::origin`'s
    /// `app_of` judges by what it sends of it: its `Sec-Fetch-*`, else its
    /// `Origin`, as a WebSocket's handshake in Chromium sends no
    /// `Sec-Fetch-Site`; one sending neither reaches nothing.
    Session,
    Neither,
}

/// This computer's opens and sessions, one for every host it answers.
#[derive(Clone)]
pub(crate) struct Sessions {
    token: Bearer,
    key: ring::hmac::Key,
    codes: Arc<Codes<(String, String)>>,
    port: u16,
}

impl Sessions {
    /// The sessions for hosts at `port`, each derived from `token`.
    pub(crate) fn new(token: Bearer, port: u16) -> Self {
        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, token.secret().as_bytes());
        let codes = Arc::new(Codes::new());
        Self {
            token,
            key,
            codes,
            port,
        }
    }

    /// The cookie's value at the host first named `name`.
    fn value(&self, name: &str) -> String {
        URL_SAFE_NO_PAD.encode(ring::hmac::sign(&self.key, name.as_bytes()))
    }

    /// What the page of the lend `app` shows in its address for the lend
    /// `read`, whose origin its fetch sends no cookie: the token's HMAC
    /// over the pair, whose `/` is in no host's first name, so no host's
    /// cookie is one. Kept nowhere, it lasts as the token does.
    pub(crate) fn cap(&self, app: &Label, read: &Label) -> String {
        self.value(&format!("{app}/{read}"))
    }

    /// What `headers` showed at the host first named `name`.
    pub(crate) fn shown(&self, headers: &HeaderMap, name: &str) -> Shown {
        let want = self.value(name);
        let ours = |c: &str| {
            c.strip_prefix(COOKIE)?
                .strip_prefix('=')
                .map(str::to_string)
        };
        let session = cookies(headers)
            .filter_map(ours)
            .any(|v| same(v.as_bytes(), want.as_bytes()));
        let shown = match (self.token.shown(headers), session) {
            (true, _) => Shown::Token,
            (false, true) => Shown::Session,
            _ => Shown::Neither,
        };
        tracing::debug!(target: "session", name, ?shown, "session: what the caller showed, the token, this host's session, or neither");
        shown
    }

    /// The address that opens the host first named `name` at `at`, a path
    /// there ("/" unless one is given), with a code for that host alone.
    pub(crate) fn open(&self, name: &str, at: Option<&str>) -> String {
        let at = at.filter(|a| local(a));
        let code = self.codes.mint((name.into(), at.unwrap_or("/").into()));
        tracing::debug!(target: "session", name, ?at, "session: an open minted, single-use");
        format!(
            "http://{name}.localhost:{}{OPEN_PATH}?code={code}",
            self.port
        )
    }

    /// The code in `query` redeemed at the host first named `name`: its
    /// session set, and back here, `?kept=<path>`, to see `req` show it
    /// before going on to the path it was minted for, or say why not: a
    /// browser keeping no cookie for a lend would go from it to the home
    /// page's `/to` and back without end (`crate::origin`'s `homeward`). A
    /// code spent, lapsed or minted for another host refused.
    pub(crate) fn redeem(&self, name: &str, req: &axum::extract::Request) -> Response {
        let query = req.uri().query().unwrap_or_default();
        if let Some((_, at)) = form_urlencoded::parse(query.as_bytes()).find(|(k, _)| k == "kept") {
            let kept = self.shown(req.headers(), name) != Shown::Neither;
            tracing::debug!(target: "session", name, kept, "session: an open back to see its session kept, on to its path if so, else said why");
            let on = Redirect::to(Some(at.as_ref()).filter(|a| local(a)).unwrap_or("/"));
            return (kept.then(|| on.into_response())).unwrap_or_else(|| self.refused(UNKEPT));
        }
        let code = query.split('&').find_map(|p| p.strip_prefix("code="));
        let redeemed = self.codes.redeem(code.unwrap_or_default());
        let at = match redeemed {
            Ok((host, at)) if host == name => at,
            other => {
                tracing::debug!(target: "session", name, ?other, "session: refused — no open, or one minted for another host, and spent");
                return self.refused("");
            }
        };
        tracing::debug!(target: "session", name, at, "session: opened, its cookie set, back here to see it kept");
        let back: String = form_urlencoded::byte_serialize(at.as_bytes()).collect();
        let set = format!(
            "{COOKIE}={}; Path=/; Max-Age={KEPT_FOR}; HttpOnly; SameSite=Lax",
            self.value(name)
        );
        let mut answer = Redirect::to(&format!("{OPEN_PATH}?kept={back}")).into_response();
        let set = HeaderValue::from_str(&set).expect("base64 is a header's");
        answer.headers_mut().insert(header::SET_COOKIE, set);
        answer
    }

    /// `headers` with this computer's cookie and token taken out, before a
    /// lend's service is asked: neither is the service's.
    pub(crate) fn strip(&self, headers: &mut HeaderMap) {
        if self.token.shown(headers) {
            headers.remove(header::AUTHORIZATION);
        }
        let others: Vec<String> = cookies(headers)
            .filter(|c| !c.starts_with(&format!("{COOKIE}=")))
            .map(Into::into)
            .collect();
        headers.remove(header::COOKIE);
        if let Ok(kept) = HeaderValue::from_str(&others.join("; ")) {
            (!others.is_empty()).then(|| headers.insert(header::COOKIE, kept));
        }
    }

    /// The one answer to a caller that showed neither: the person told to
    /// open svrngs from their computer, by the opener install.sh put in
    /// place, which needs no terminal, after `why` when there is one; a
    /// program told where the token is.
    pub(crate) fn refused(&self, why: &str) -> Response {
        let page = format!(
            "<!doctype html><meta charset=\"utf-8\"><title>Open svrngs from your computer</title>\
             <h1>Open svrngs from your computer</h1>{why}\
             <p>This page opens only from svrngs on this computer. Open svrngs from your {OPENER}, and it opens here.</p>\
             <p>A program on this computer shows the token in {} instead.</p>",
            self.token.file().display()
        );
        (StatusCode::UNAUTHORIZED, Html(page)).into_response()
    }
}

/// Whether `at` is a path on the host it is said at, never another's.
fn local(at: &str) -> bool {
    at.starts_with('/') && !at.starts_with("//") && !at.contains('\\')
}

/// Each cookie a request sent, `name=value`.
fn cookies(headers: &HeaderMap) -> impl Iterator<Item = &str> {
    let sent = headers.get_all(header::COOKIE).into_iter();
    let sent = sent.filter_map(|v| v.to_str().ok());
    sent.flat_map(|v| v.split(';')).map(str::trim)
}

/// What a program asks to open: the home page, or a lend at a path there.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Opening {
    lend: Option<Label>,
    at: Option<String>,
}

/// `POST /v1/open`, behind the token, `{}` or `{"lend", "at"}`: an
/// address that opens the home page, or that lend at that path, in this
/// computer's browser (`svrngs open`).
async fn open(State(s): State<Sessions>, Json(o): Json<Opening>) -> Json<serde_json::Value> {
    let name = o.lend.as_ref().map_or(HOME, Label::as_str);
    Json(serde_json::json!({ "open": s.open(name, o.at.as_deref()) }))
}

/// `/v1/open`, for the record API's routes behind the token.
pub(crate) fn router(sessions: Sessions) -> Router {
    Router::new()
        .route("/v1/open", post(open))
        .with_state(sessions)
}
