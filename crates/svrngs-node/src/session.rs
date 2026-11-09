// SPDX-License-Identifier: AGPL-3.0-or-later
//! A browser told from a program (give-session; svrn-docs
//! `rings/current/GIVE.md` §5): the home page and each lend's origin answer
//! a browser only with a proof by the key this computer gave that origin,
//! and a program only with its token ([`crate::api::Bearer`]). Loopback
//! alone let in a container on a Mac running podman, which reaches the
//! Mac's `127.0.0.1` through gvproxy (GIVE.md §5, measured 2026-10-25).
//!
//! **An open** is a code, single-use and gone in a minute (the sign-in
//! provider's `Codes`), minted by `POST /v1/open` behind the token
//! (`svrngs open`, install.sh's open) or by the home page for a lend it
//! opens (`crate::home`'s `/to`), and redeemed at [`OPEN_PATH`] on the host
//! it names by a `POST` from that origin's own page (`home/session.js`),
//! answered with the origin's key: the token's HMAC over the host's first
//! name and this computer's port. None is kept, a restart keeps every
//! browser opened, and one origin's key opens no other, another port's
//! included. The page keeps the key in that origin's IndexedDB, never to be
//! read back out, and registers a worker (`home/session-worker.js`) that
//! signs each of the origin's own requests with it.
//!
//! **A proof** (`x-svrngs-browser`, [`Sessions::shown`]) is
//! `<stamp>.<nonce>.<tag>`: the key's HMAC over the host, the method, the
//! path and query, the stamp and a random nonce, the body's SHA-256 where
//! the home page asks one (`x-svrngs-body`, [`Sessions::body_matches`]),
//! the person's intent (`x-svrngs-intent`, `yes` only from the origin's
//! own page), and this run's epoch, drawn at start and read at
//! `/svrngs-session-epoch`, so no proof outlives a restart. A stamp a
//! minute or more from now is refused, and each nonce is spent once, in
//! its origin's own window ([`Nonces`]). A WebSocket, whose handshake
//! carries no header a page sets, opens with a single-use code a proved
//! fetch mints (`/svrngs-socket`, [`Sessions::socket`]).
//!
//! **A redirect's onward request** is proved by the proof that asked for it: a
//! page's `fetch` follows a redirect itself, sending the same headers on,
//! and the worker cannot read an opaque redirect's `Location` to prove the
//! next request afresh. So where a proved request's answer is a redirect to
//! this origin, that proof is taken once more, at the address the answer
//! named, by the method the redirect leaves, while its stamp is fresh
//! ([`Onwards`]); at any other address, or twice, it is refused as before.
//!
//! A cookie opens nothing: one a browser kept from before is taken off,
//! with the proof, the token and the intent, before a lend's service sees
//! the request ([`Sessions::strip`]).
use crate::{api::Bearer, oidc::provider::Codes};
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::post,
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
#[expect(
    clippy::disallowed_types,
    reason = "the nonce store is one process's map, read and written by its own routes"
)]
use std::sync::Mutex;
use svrngs_core::pass::Label;

pub(crate) const OPEN_PATH: &str = "/svrngs-open";
pub(crate) const AS_PATH: &str = "/svrngs-as/";
pub(crate) const HOME: &str = "svrngs";
const PROOF: &str = "x-svrngs-browser";

/// How far a proof's stamp may be from now, either way, in seconds.
const FRESH: i64 = 60;
/// The most proofs one origin shows within [`FRESH`] of now: some 68 a
/// second, each kept until its stamp is stale.
const PER_ORIGIN: usize = 4096;
/// The most origins holding fresh proofs at once: every lend a person has
/// opened, and the home page, working within the same minute.
const ORIGINS: usize = 64;
/// The most redirects waiting for their onward request at once, every origin's:
/// one a proved request's answer named, each kept while its stamp is fresh.
const ONWARDS: usize = 4096;
/// Kept by no cache: the session's epoch, scripts, browser key and refusal page.
const NO_STORE: (header::HeaderName, &str) = (header::CACHE_CONTROL, "no-store");

/// Why a request shows no session ([`Sessions::shown`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unproven {
    /// No proof header, or one that is not text.
    Absent,
    /// Not `<stamp>.<nonce>.<tag>`, a 32-hex-digit nonce and a stamp in seconds.
    Malformed,
    /// Its stamp a minute or more from this computer's clock.
    Stale,
    /// Asked at another host than the origin's own, port included.
    OtherHost,
    /// Its tag is not this origin's key's over this request.
    Unsigned,
    /// Signed, and its nonce not spent now.
    Spent(Spent),
}

/// Why a signed proof's nonce is not spent ([`Nonces::spend`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spent {
    /// Shown before, while its stamp was fresh: a replay.
    Replayed,
    /// This origin has shown [`PER_ORIGIN`] fresh proofs already.
    OriginFull,
    /// [`ORIGINS`] other origins hold fresh proofs already.
    TooManyOrigins,
}

/// Each origin's spent nonces, with their stamps, while a stamp is fresh:
/// one window per origin, so a busy origin fills only its own. Bounded by
/// [`PER_ORIGIN`] in each of at most [`ORIGINS`] origins: 262,144 entries
/// of 32 bytes, a stamp and a 128-bit nonce, about 8 MiB and under 16 MiB
/// with the tree's own nodes, however many proofs are shown. Stale ones
/// are dropped at each spend, as a stale stamp is refused before its nonce
/// is read.
#[derive(Default)]
#[expect(
    clippy::disallowed_types,
    reason = "the nonce store is one process's map, read and written by its own routes"
)]
pub(crate) struct Nonces(Mutex<BTreeMap<String, BTreeSet<(i64, u128)>>>);

impl Nonces {
    /// `nonce`, stamped `stamp`, spent for `origin` at `now`: once, in
    /// that origin's window; or why not.
    pub(crate) fn spend(
        &self,
        origin: &str,
        stamp: i64,
        nonce: u128,
        now: i64,
    ) -> Result<(), Spent> {
        let mut seen = self.0.lock().expect("the nonce store's lock");
        let fresh = (now - FRESH + 1, 0);
        seen.retain(|_, window| {
            *window = window.split_off(&fresh);
            !window.is_empty()
        });
        if !seen.contains_key(origin) && seen.len() >= ORIGINS {
            return Err(Spent::TooManyOrigins);
        }
        let window = seen.entry(origin.to_string()).or_default();
        if window.contains(&(stamp, nonce)) {
            return Err(Spent::Replayed);
        }
        if window.len() >= PER_ORIGIN {
            return Err(Spent::OriginFull);
        }
        window.insert((stamp, nonce));
        Ok(())
    }
}

/// A redirect's onward request, waiting for the proof that asked for it
/// ([`Onwards`]): at which origin, address (none once taken) and method, that
/// proof's stamp, and how many redirects it has been followed through.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Onward {
    name: String,
    at: Option<String>,
    method: axum::http::Method,
    stamp: i64,
    followed: u8,
}

/// Each redirect a proved request's answer named, keyed by that request's
/// whole proof, `<stamp>.<nonce>.<tag>`, which only the origin's key makes:
/// taken once, at that address and by that method, while the stamp is
/// fresh, and through at most [`FOLLOWS`] redirects, as a browser's fetch
/// follows at most 20. Bounded by [`ONWARDS`]; past it a redirect is not
/// followed, and says so.
#[derive(Default)]
#[expect(
    clippy::disallowed_types,
    reason = "the onward store is one process's map, read and written by its own routes"
)]
pub(crate) struct Onwards(Mutex<BTreeMap<String, Onward>>);

/// The most redirects one proof is followed through: the fetch standard's
/// limit, past which a browser fails the fetch itself.
const FOLLOWS: u8 = 20;

impl Onwards {
    /// `proof`'s onward request at origin `name`, address `at`, by `method`, kept
    /// until taken or stale at `now`: how many redirects it has followed;
    /// or why not, the proof followed through [`FOLLOWS`] already, or the
    /// store full.
    fn keep(
        &self,
        proof: &str,
        name: &str,
        at: &str,
        method: axum::http::Method,
        stamp: i64,
        now: i64,
    ) -> Result<u8, &'static str> {
        let mut kept = self.0.lock().expect("the onward store's lock");
        kept.retain(|_, h| now.abs_diff(h.stamp) < FRESH.unsigned_abs());
        let followed = kept.get(proof).map_or(0, |h| h.followed) + 1;
        if followed > FOLLOWS {
            return Err("followed through as many redirects as a browser follows");
        }
        if kept.len() >= ONWARDS && !kept.contains_key(proof) {
            return Err("the onward store is full");
        }
        let onward = Onward {
            name: name.into(),
            at: Some(at.into()),
            method,
            stamp,
            followed,
        };
        kept.insert(proof.to_string(), onward);
        Ok(followed)
    }

    /// Whether `proof` names an onward waiting at origin `name`, address `at`,
    /// by `method`, its stamp fresh at `now`: taken, once, if so.
    fn take(
        &self,
        proof: &str,
        name: &str,
        at: &str,
        method: &axum::http::Method,
        now: i64,
    ) -> bool {
        let mut kept = self.0.lock().expect("the onward store's lock");
        let onward = kept.get_mut(proof).filter(|h| {
            h.name == name
                && h.at.as_deref() == Some(at)
                && h.method == method
                && now.abs_diff(h.stamp) < FRESH.unsigned_abs()
        });
        onward.map(|h| h.at = None).is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shown {
    Token,
    Session,
    Neither,
}

#[derive(Clone)]
pub(crate) struct Sessions {
    token: Bearer,
    key: ring::hmac::Key,
    codes: Arc<Codes<(String, String)>>,
    nonces: Arc<Nonces>,
    onwards: Arc<Onwards>,
    sockets: Arc<Codes<(String, String)>>,
    port: u16,
    epoch: String,
}

impl Sessions {
    #[expect(
        clippy::disallowed_methods,
        reason = "the session adapter draws a boot challenge to prevent cross-restart replay"
    )]
    pub(crate) fn new(token: Bearer, port: u16) -> Self {
        let mut epoch = [0; 16];
        getrandom::fill(&mut epoch).expect("failed to draw the browser session's boot challenge");
        Self {
            key: ring::hmac::Key::new(ring::hmac::HMAC_SHA256, token.secret().as_bytes()),
            token,
            codes: Arc::new(Codes::new()),
            nonces: Arc::new(Nonces::default()),
            onwards: Arc::new(Onwards::default()),
            sockets: Arc::new(Codes::new()),
            port,
            epoch: hex::encode(epoch),
        }
    }

    /// The host first named `name` at this computer's port: an origin's
    /// whole authority, as its port is part of it.
    fn host(&self, name: &str) -> String {
        format!("{name}.localhost:{}", self.port)
    }

    fn browser_key(&self, name: &str) -> Vec<u8> {
        ring::hmac::sign(
            &self.key,
            format!("browser/{name}/{}", self.port).as_bytes(),
        )
        .as_ref()
        .to_vec()
    }

    /// The capability a read's address carries (`/svrngs-as/<cap>/`): the
    /// token's HMAC over the pair, the page's lend and the read lend.
    pub(crate) fn cap(&self, app: &Label, read: &Label) -> String {
        let pair = format!("{app}/{read}");
        URL_SAFE_NO_PAD.encode(ring::hmac::sign(&self.key, pair.as_bytes()))
    }

    /// What `req` shows at the origin first named `name`: the token, a
    /// fresh proof by that origin's key ([`Unproven`] says why not), or
    /// neither.
    pub(crate) fn shown(&self, req: &Request, name: &str) -> Shown {
        if self.token.shown(req.headers()) {
            return Shown::Token;
        }
        let shown = match self.proved(req, name) {
            Ok(()) => Shown::Session,
            Err(why) => {
                tracing::debug!(target: "session", name, ?why, "session: no fresh proof by this origin's key; a cookie is never authority");
                Shown::Neither
            }
        };
        tracing::debug!(target: "session", name, ?shown, "session: a fresh request proof, never cookie authority");
        shown
    }

    /// `req`'s proof checked as the module doc says, its nonce spent last.
    fn proved(&self, req: &Request, name: &str) -> Result<(), Unproven> {
        let headers = req.headers();
        let proof = (headers.get(PROOF).and_then(|h| h.to_str().ok())).ok_or(Unproven::Absent)?;
        let (stamp, rest) = proof.split_once('.').ok_or(Unproven::Malformed)?;
        let (nonce, signature) = rest.split_once('.').ok_or(Unproven::Malformed)?;
        let at = stamp.parse::<i64>().map_err(|_| Unproven::Malformed)?;
        let hex = nonce.len() == 32 && nonce.bytes().all(|b| b.is_ascii_hexdigit());
        let spent = (hex.then(|| u128::from_str_radix(nonce, 16).ok()).flatten())
            .ok_or(Unproven::Malformed)?;
        let now = crate::door::unix_now().ok_or(Unproven::Stale)?;
        if now.abs_diff(at) >= FRESH.unsigned_abs() {
            return Err(Unproven::Stale);
        }
        let host = self.host(name);
        if headers.get(header::HOST).and_then(|h| h.to_str().ok()) != Some(host.as_str()) {
            return Err(Unproven::OtherHost);
        }
        let text = |h: &str| headers.get(h).map_or("", |v| v.to_str().unwrap_or(""));
        let (digest, intent) = (text("x-svrngs-body"), text("x-svrngs-intent"));
        let message = format!(
            "{host}\n{}\n{}\n{stamp}.{nonce}\n{digest}\n{intent}\n{}",
            req.method(),
            req.uri(),
            self.epoch
        );
        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &self.browser_key(name));
        let signature = URL_SAFE_NO_PAD
            .decode(signature)
            .map_err(|_| Unproven::Malformed)?;
        if ring::hmac::verify(&key, message.as_bytes(), &signature).is_err() {
            let uri = req.uri().to_string();
            let onward = self.onwards.take(proof, name, &uri, req.method(), now);
            tracing::debug!(target: "session", name, onward, "session: a proof not over this request is taken only as a redirect's onward request its own answer named, once");
            return onward.then_some(()).ok_or(Unproven::Unsigned);
        }
        (self.nonces.spend(name, at, spent, now)).map_err(Unproven::Spent)
    }

    /// The proof `headers` show, read before they are stripped, for
    /// [`Self::onward`].
    pub(crate) fn proof_of(headers: &HeaderMap) -> Option<String> {
        (headers.get(PROOF)?.to_str().ok()).map(str::to_string)
    }

    /// Where `answer`, to a request asked by `method` with `proof` at the
    /// origin first named `name`, redirects to this origin, its onward request
    /// is kept for that proof ([`Onwards`]): by the method a browser follows
    /// it with, GET after a 303, or after a 301 or 302 to a POST, else the
    /// same. Any other answer, or a redirect elsewhere, keeps none.
    pub(crate) fn onward(
        &self,
        name: &str,
        proof: &str,
        method: &axum::http::Method,
        answer: &Response,
    ) {
        use axum::http::Method;
        let status = answer.status().as_u16();
        if !matches!(status, 301 | 302 | 303 | 307 | 308) {
            return;
        }
        let own = format!("http://{}", self.host(name));
        let location = answer
            .headers()
            .get(header::LOCATION)
            .and_then(|l| l.to_str().ok());
        let at = location.map(|l| l.strip_prefix(own.as_str()).unwrap_or(l));
        let Some(at) = at.filter(|a| local(a)) else {
            tracing::debug!(target: "session", name, status, ?location, "session: a redirect to another origin, whose onward request no proof here carries");
            return;
        };
        let next = match (status, method) {
            (303, &Method::HEAD) => Method::HEAD,
            (303, _) | (301 | 302, &Method::POST) => Method::GET,
            _ => method.clone(),
        };
        let stamp = proof.split_once('.').and_then(|(s, _)| s.parse().ok());
        let now = crate::door::unix_now();
        let (Some(stamp), Some(now)) = (stamp, now) else {
            return;
        };
        let kept = self.onwards.keep(proof, name, at, next.clone(), stamp, now);
        tracing::debug!(target: "session", name, status, at, %next, ?kept, "session: a proved request's redirect to its own origin, its onward request kept for that proof, once (refused past 20 redirects, or when the store is full)");
    }

    pub(crate) fn open(&self, name: &str, at: Option<&str>) -> String {
        let at = at.filter(|a| local(a)).unwrap_or("/");
        let code = self.codes.mint((name.into(), at.into()));
        tracing::debug!(target: "session", name, "session: a single-use enrollment open minted");
        format!("http://{}{OPEN_PATH}?code={code}", self.host(name))
    }

    /// Only fixed bootstrap code is public. It carries no person's data or authority.
    pub(crate) fn public(&self, name: &str, req: &Request) -> Option<Response> {
        if req.uri().path() == "/svrngs-session-epoch" && req.method() == axum::http::Method::GET {
            return Some(([NO_STORE], self.epoch.clone()).into_response());
        }
        if req.uri().path() == "/svrngs-socket" && req.method() == axum::http::Method::POST {
            let at = form_urlencoded::parse(req.uri().query().unwrap_or_default().as_bytes())
                .find(|(k, _)| k == "at")
                .map(|(_, v)| v.into_owned());
            let at = at.filter(|a| local(a));
            let local_at = at.is_some();
            let proved = local_at && name != HOME && self.shown(req, name) == Shown::Session;
            return Some(match at.filter(|_| proved) {
                Some(at) => {
                    tracing::debug!(target: "session", name, "session: a WebSocket's single-use open minted for a proved fetch");
                    Json(serde_json::json!({ "code": self.sockets.mint((name.into(), at)) }))
                        .into_response()
                }
                None => {
                    tracing::debug!(target: "session", name, proved, local_at, "session: refused — a WebSocket open asked without this origin's proof, at the home page, or for a path not on this origin");
                    StatusCode::FORBIDDEN.into_response()
                }
            });
        }
        if req.method() != axum::http::Method::GET {
            return None;
        }
        let file = match req.uri().path() {
            "/svrngs-session.js" => include_str!("home/session.js"),
            "/svrngs-session-worker.js" => include_str!("home/session-worker.js"),
            "/svrngs-session-client.js" => include_str!("home/session-client.js"),
            "/svrngs-session-ready" if self.shown(req, name) == Shown::Session => {
                return Some((StatusCode::OK, "ready").into_response())
            }
            _ => return None,
        };
        let headers = [(header::CONTENT_TYPE, "text/javascript"), NO_STORE];
        Some((headers, file).into_response())
    }

    /// Browser WebSockets cannot set a proof header: a proved fetch mints one use.
    pub(crate) fn socket(&self, name: &str, req: &mut Request) -> bool {
        let host = self.host(name);
        let headers = req.headers();
        let websocket =
            |h: &axum::http::HeaderValue| h.as_bytes().eq_ignore_ascii_case(b"websocket");
        if req.method() != axum::http::Method::GET
            || headers.get(header::HOST).is_none_or(|h| h != host.as_str())
            || !headers.get(header::UPGRADE).is_some_and(websocket)
        {
            return false;
        }
        let Some(query) = req.uri().query() else {
            return false;
        };
        let code = query
            .split('&')
            .find_map(|p| p.strip_prefix("__svrngs_socket="));
        let Some(code) = code.map(str::to_string) else {
            return false;
        };
        let rest = query
            .split('&')
            .filter(|p| !p.starts_with("__svrngs_socket="))
            .collect::<Vec<_>>()
            .join("&");
        let at = format!(
            "{}{}{}",
            req.uri().path(),
            if rest.is_empty() { "" } else { "?" },
            rest
        );
        let valid = self
            .sockets
            .redeem(&code)
            .is_ok_and(|(host, path)| host == name && path == at);
        if let Ok(uri) = at.parse() {
            *req.uri_mut() = uri;
        }
        tracing::debug!(target: "session", name, valid, "session: a WebSocket's single-use open");
        valid
    }

    pub(crate) fn redeem(&self, name: &str, req: &Request) -> Response {
        if req.method() == axum::http::Method::GET {
            tracing::debug!(target: "session", name, "session: an open's address read, which enrolls nothing until its page posts it");
            return self.refused("");
        }
        let host = self.host(name);
        let headers = req.headers();
        let own = (headers.get(header::ORIGIN))
            .is_some_and(|h| h == format!("http://{host}").as_str())
            && headers
                .get(header::HOST)
                .is_some_and(|h| h == host.as_str());
        if req.method() != axum::http::Method::POST || !own {
            tracing::debug!(target: "session", name, own, "session: enrollment refused outside its own origin");
            return StatusCode::FORBIDDEN.into_response();
        }
        let query = req.uri().query().unwrap_or_default();
        let code = query
            .split('&')
            .find_map(|p| p.strip_prefix("code="))
            .unwrap_or_default();
        match self.codes.redeem(code) {
            Ok((host, at)) if host == name => {
                tracing::debug!(target: "session", name, "session: origin enrolled once, no cookie issued");
                ([NO_STORE], Json(serde_json::json!({ "key": URL_SAFE_NO_PAD.encode(self.browser_key(name)), "at": at }))).into_response()
            }
            Ok((host, _)) => {
                tracing::debug!(target: "session", name, minted_for = host, "session: refused — an open minted for another origin");
                self.refused("This open was spent or expired.")
            }
            Err(why) => {
                tracing::debug!(target: "session", name, why, "session: refused — an open spent, lapsed or never minted");
                self.refused("This open was spent or expired.")
            }
        }
    }

    pub(crate) fn strip(&self, headers: &mut HeaderMap) {
        headers.remove(PROOF);
        headers.remove("x-svrngs-body");
        headers.remove("x-svrngs-intent");
        if self.token.shown(headers) {
            headers.remove(header::AUTHORIZATION);
        }
        // Retire legacy cookies as well: old browsers may continue sending them.
        let others = headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|h| h.to_str().ok())
            .flat_map(|h| h.split(';'))
            .map(str::trim)
            .filter(|c| !c.starts_with("svrngs="))
            .collect::<Vec<_>>()
            .join("; ");
        headers.remove(header::COOKIE);
        if !others.is_empty() {
            if let Ok(h) = others.parse() {
                headers.insert(header::COOKIE, h);
            }
        }
    }

    /// Control-plane writes authenticate the body before any effect is performed.
    pub(crate) async fn body_matches(&self, req: &mut Request) -> bool {
        if matches!(
            *req.method(),
            axum::http::Method::GET | axum::http::Method::HEAD
        ) {
            return true;
        }
        let wanted = req
            .headers()
            .get("x-svrngs-body")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();
        let body = std::mem::replace(req.body_mut(), axum::body::Body::empty());
        let bytes = match axum::body::to_bytes(body, 2 * 1024 * 1024).await {
            Ok(bytes) => bytes,
            Err(why) => {
                tracing::debug!(target: "session", %why, "session: refused — a control-plane write's body unread, or over 2 MiB, so not checked");
                return false;
            }
        };
        let digest = ring::digest::digest(&ring::digest::SHA256, &bytes);
        let matches =
            crate::oidc::provider::same(wanted.as_bytes(), hex::encode(digest.as_ref()).as_bytes());
        tracing::debug!(target: "session", matches, "session: a control-plane write's proved body checked before any effect");
        *req.body_mut() = axum::body::Body::from(bytes);
        matches
    }

    pub(crate) fn refused(&self, why: &str) -> Response {
        let file = self.token.file().display().to_string();
        let file = file.replace('&', "&amp;").replace('<', "&lt;");
        let opener = if cfg!(target_os = "macos") {
            "Applications"
        } else {
            "applications menu"
        };
        let page = format!("<!doctype html><meta charset=utf-8><title>Open svrngs from your computer</title><h1>Open svrngs from your computer</h1><p>{why}</p><p>Open svrngs from your {opener}, and it opens in your usual browser. A browser comes in only when this computer lets it, so that nothing else here can act as you.</p><p>To use this browser too, open the Terminal app, type <code>svrngs open --print</code>, and open the address it shows here within a minute.</p><p>A program, not a browser, sends the token in {file}.</p><p id=svrngs-session-state></p><script type=module src=/svrngs-session.js></script>");
        (
            StatusCode::UNAUTHORIZED,
            [NO_STORE, (header::REFERRER_POLICY, "no-referrer")],
            Html(page),
        )
            .into_response()
    }
}

fn local(at: &str) -> bool {
    at.starts_with('/')
        && !at.starts_with("//")
        && !at.contains('\\')
        && !at.chars().any(char::is_control)
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Opening {
    lend: Option<Label>,
    at: Option<String>,
}

async fn open(State(s): State<Sessions>, Json(o): Json<Opening>) -> Json<serde_json::Value> {
    Json(
        serde_json::json!({ "open": s.open(o.lend.as_ref().map_or(HOME, Label::as_str), o.at.as_deref()) }),
    )
}

pub(crate) fn router(sessions: Sessions) -> Router {
    Router::new()
        .route("/v1/open", post(open))
        .with_state(sessions)
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
