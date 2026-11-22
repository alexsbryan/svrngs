// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/api.rs@e1a43bcd2
//! The loopback routes a shim author touches, and nothing else.
//!
//! The ring rail's doors, `/v1/rail/{append,log,live,people}`, in [`crate::rail`];
//! beside them (svrngs, u5-1) the groups this computer holds and what their
//! people lend, `/v1/{groups,lends}`, in [`crate::surface`]. The address it
//! is served on is written to [`API_FILE`] in the data dir.
//! (svrngs: the mesh, media, publish, origin, membership, store and ledger
//! doors beside them stay behind; PLAN §4, U3's inventory.)
//!
//! **Loopback is not the auth; the token is** (give-token). The bind
//! refuses any address that is not loopback, and every request to the
//! record asks with a code the token in [`TOKEN_FILE`] makes for its own
//! connection ([`ask`], [`holds_the_token`]), never with the token, which
//! is refused from any program ([`as_whom`]; "Never send the key",
//! V1.13.3). A container on a Mac running podman reaches the Mac's
//! `127.0.0.1` (svrn-docs `rings/current/GIVE.md` §5), so loopback alone
//! let a program meant to be contained write as the person; a program the
//! person runs reads the token as it could read their key, so it still
//! acts as them (WHAT_CAN_ACT_AS_YOU §4, kept for their own programs).

use std::net::{Ipv6Addr, SocketAddr};

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::serve::IncomingStream;
use axum::{Json, Router};
use n0_future::task::AbortOnDropHandle;
use tokio::net::TcpListener;

use crate::rail::RailApi;
use crate::share::{self, ANSWERED_WITHIN};

/// Why the API did not start. (Upstream's were two variants of the daemon's
/// `Refusal`, `lib.rs:122-123`, which did not come over; named here as the
/// crate names its errors, since `Refusal` is the command's, cli.rs.)
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0} is not a loopback address; the record is this computer's alone")]
    NotLoopback(SocketAddr),
    #[error("cannot listen on {0}: {1}")]
    Listen(SocketAddr, std::io::Error),
    #[error("another program on this computer holds {0}")]
    Taken(SocketAddr),
    #[error("cannot read or write where the API listens, {0}: {1}")]
    AddressFile(std::path::PathBuf, std::io::Error),
    #[error("{0} names no address svrngs listens at: delete it, and svrngs takes a new one as it starts")]
    NoAddress(std::path::PathBuf),
    #[error("cannot read or keep this computer's token, {0}: {1}")]
    TokenFile(std::path::PathBuf, std::io::Error),
    #[error("{0} holds no token svrngs made: delete it, and svrngs makes a new one as it starts")]
    NoToken(std::path::PathBuf),
}

/// The file in the data dir, beside [`API_FILE`] and owner-only (0600),
/// holding the token a program keys its asks of svrngs with ([`ask`]),
/// never sending it: made at the first start and kept ([`kept_token`]).
pub const TOKEN_FILE: &str = "token";

/// This computer's token: named for how a program showed it before
/// V1.13.3, `Authorization: Bearer`, which svrngs now refuses ([`as_whom`]).
#[derive(Clone)]
pub struct Bearer(std::sync::Arc<str>);

impl Bearer {
    /// What `request` asks with, compared in constant time by the sign-in
    /// provider's `same`: the token itself, `Authorization: Bearer
    /// <token>`, refused as [`Unasked::Bearer`]; a code, `Authorization:
    /// Svrngs <code>`, taken only where it is [`ask`]'s for the nonce its
    /// connection's last proof gave ([`Line`]) and that connection's two
    /// ends, else refused saying why; or none of these.
    fn asked(&self, request: &Request) -> Option<Result<(), Unasked>> {
        let (same, secret) = (crate::oidc::provider::same, self.secret().as_bytes());
        let shown = request.headers().get_all(axum::http::header::AUTHORIZATION);
        let shown: Vec<_> = shown.iter().map(|h| h.as_bytes()).collect();
        let token = |t: &&[u8]| t.strip_prefix(b"Bearer ").is_some_and(|t| same(t, secret));
        match shown[..] {
            _ if shown.iter().any(token) => return Some(Err(Unasked::Bearer)),
            [_, _, ..] => return Some(Err(Unasked::Twice)),
            _ => {}
        }
        let code = shown.first()?.strip_prefix(b"Svrngs ")?;
        let line = request.extensions().get::<Line>();
        let nonce = line.and_then(|l| l.nonce.borrow().clone());
        let ends = crate::session::peer(request).zip(line.and_then(|l| l.server));
        let Some((nonce, (client, server))) = nonce.zip(ends) else {
            return Some(Err(Unasked::Unproved));
        };
        let made = same(code, ask(self.secret(), &nonce, client, server).as_bytes());
        Some(made.then_some(()).ok_or(Unasked::Elsewhere))
    }

    pub(crate) fn secret(&self) -> &str {
        &self.0
    }
}

/// The token kept in `data_dir`, or, before the first start, a fresh one
/// written there owner-only, as the sign-in key is kept (`oidc.rs`). A file
/// that cannot be read, or holds no token svrngs made, is refused, never
/// replaced: a program that read it would be refused in silence.
#[expect(clippy::disallowed_methods, reason = "draws the record's token")]
pub fn kept_token(data_dir: &std::path::Path) -> Result<Bearer, ApiError> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let path = data_dir.join(TOKEN_FILE);
    let text = match std::fs::read_to_string(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut secret = [0u8; 32];
            // As identity.rs: getrandom failing is a host with no entropy.
            getrandom::fill(&mut secret).expect("failed to draw the record's token");
            let text = URL_SAFE_NO_PAD.encode(secret);
            crate::declared::write_secret(&path, text.as_bytes())
                .map_err(|(at, e)| ApiError::TokenFile(at, e))?;
            tracing::debug!(target: "rails", path = %path.display(), "api: made this computer's token, once, owner-only");
            text
        }
        read => read.map_err(|e| ApiError::TokenFile(path.clone(), e))?,
    };
    let made = URL_SAFE_NO_PAD
        .decode(text.trim())
        .is_ok_and(|b| b.len() == 32);
    tracing::debug!(target: "rails", path = %path.display(), made, "api: read this computer's token");
    made.then(|| Bearer(text.trim().into()))
        .ok_or(ApiError::NoToken(path))
}

/// The file in the data dir naming where the API listens, as a URL and a
/// newline (`http://127.0.0.1:<port>`), so an app the node did not start
/// finds it (u5-1), and the command listens there again at its next start
/// ([`kept_address`]; the first takes any free port, `svrngs::cli::API`).
/// A node that has stopped leaves it naming a port that refuses, which is
/// what an app asking it then hears.
pub const API_FILE: &str = "api";

/// Where [`write_address`] last said the API listens in `data_dir`, so it
/// listens there again (start-address: a bookmark and a program's address
/// survive a restart, `demo/runs/start/2026-10-13/README.md` §4); `None`
/// before the first start. A file naming no address is refused, never read
/// as none, which would move the address in silence.
pub fn kept_address(data_dir: &std::path::Path) -> Result<Option<SocketAddr>, ApiError> {
    let path = data_dir.join(API_FILE);
    let text = match std::fs::read_to_string(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        read => read.map_err(|e| ApiError::AddressFile(path.clone(), e))?,
    };
    let kept = text
        .trim()
        .strip_prefix("http://")
        .and_then(|a| a.parse().ok());
    tracing::debug!(target: "rails", path = %path.display(), ?kept, "api: read where it listened, to listen there again");
    kept.map(Some).ok_or(ApiError::NoAddress(path))
}

/// Write `addr` to [`API_FILE`] in `data_dir`, through a file renamed into
/// place, so an app never reads half an address.
pub fn write_address(data_dir: &std::path::Path, addr: SocketAddr) -> Result<(), ApiError> {
    let path = data_dir.join(API_FILE);
    crate::declared::write_whole(&path, format!("http://{addr}\n").as_bytes())
        .map_err(|(_, e)| ApiError::AddressFile(path.clone(), e))?;
    tracing::debug!(target: "rails", path = %path.display(), %addr, "api: wrote where it listens, for apps it did not start");
    Ok(())
}

/// `body` posted to `route` of svrngs at `api`, as [`API_FILE`] names it,
/// or `route` got where there is none, asked as the person holding
/// `token` on one connection, and only once svrngs has proved on it that
/// it holds the token: a fresh challenge asked of `POST /v1/proof`, its
/// answer checked against this connection's own two ends ([`proven`]),
/// and then asked with the code made for the nonce that answer gave this
/// connection ([`ask`]). So the token is sent nowhere, and what can't prove
/// it holds it is sent no code either: a program holding svrngs's address
/// while svrngs is down, of any account, in a container or a sandbox or
/// not, or one carrying the challenge to svrngs on a connection of its
/// own; else why not. The one way svrngs's command asks svrngs (cli.rs
/// `Here::ask`, `svrngs ask`).
#[expect(
    clippy::disallowed_methods,
    reason = "dial: svrngs's own API, proved before it is asked"
)]
pub async fn asked(api: &str, route: &str, token: &str, body: Option<String>) -> Asked {
    let away = |_: std::io::Error| Withheld::Away;
    let at = api.trim().trim_start_matches("http://");
    let at: SocketAddr = at.parse().map_err(|_| Withheld::Away)?;
    let connected = tokio::time::timeout(PROVE_WITHIN, tokio::net::TcpStream::connect(at));
    let stream = connected
        .await
        .map_err(|_| Withheld::Unanswered)?
        .map_err(away)?;
    let ends = crate::iroh::ends(&stream).map_err(away)?;
    let challenge = Challenge::drawn();
    let mut on = share::http_on(stream).await.ok_or(Withheld::Away)?;
    let post = |route| axum::http::Request::post(route).header("content-type", "application/json");
    let asked = format!(r#"{{"challenge":"{}"}}"#, challenge.0);
    let answer = share::sent::<ANSWERED_WITHIN>(&mut on, at.port(), post("/v1/proof"), asked);
    let answer = tokio::time::timeout(PROVE_WITHIN, answer).await.ok();
    let proved = proven(answer, &proof(token, &challenge, ends.local, ends.peer));
    tracing::debug!(target: "rails", %at, client = %ends.local, proved = ?proved.as_ref().map(|_| ()), "api: whether svrngs proved it holds the token on this connection and gave it a nonce, asked on it only if so");
    let code = ask(token, &proved?, ends.local, ends.peer);
    // hyper takes a second request only once the first has let it go.
    on.ready().await.map_err(|_| Withheld::Away)?;
    let method = if body.is_some() { "POST" } else { "GET" };
    let asked = post(route).method(method);
    let asked = asked.header("authorization", format!("Svrngs {code}"));
    // svrngs proved, so its answer is read whole, never cut short.
    let answer = share::sent::<{ usize::MAX }>(&mut on, at.port(), asked, body.unwrap_or_default());
    answer.await.ok_or(Withheld::Away)
}

/// What svrngs answered an ask ([`asked`]), its status and body, or why
/// it was sent none of the token.
pub type Asked = Result<(u16, Vec<u8>), Withheld>;

/// Why svrngs's command sent none of its token to what answers at the
/// address svrngs wrote ([`asked`]): a closed set (AGENTS 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Withheld {
    /// Nothing takes a connection there, or answers on it: svrngs isn't
    /// running.
    Away,
    /// What answers has no proof to give (404): a svrngs older than this
    /// command.
    Older,
    /// Nothing answered the challenge within [`PROVE_WITHIN`].
    Unanswered,
    /// What answers is not svrngs proving, on this connection, that it
    /// holds the token.
    Unproven,
}

/// Why svrngs answers a program not as its person ([`as_whom`],
/// [`holds_the_token`]): a closed set (AGENTS 9), each said in words that
/// say what to do instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Unasked {
    /// svrngs's token itself, `Authorization: Bearer <token>`, or another
    /// Bearer at the record.
    #[error("svrngs doesn't take its token from a program any more: ask through `svrngs ask <route>`, or apps/client/svrngs.mjs, as `svrngs guide` shows")]
    Bearer,
    /// svrngs's token itself at a lend's address, where a program that
    /// can't ask with a code uses the lend by Play (`crate::play`).
    #[error("svrngs doesn't take its token from a program any more. To use what's lent here from a program, press Play on it on your home page, and give the program any text as its key; or ask through `svrngs ask <route>`, or apps/client/svrngs.mjs")]
    BearerAtALend,
    /// More than one `Authorization`, so none of them is taken.
    #[error("this request sends more than one Authorization header, so svrngs takes none of them: send one, the code for this connection, as `svrngs ask` and apps/client/svrngs.mjs do")]
    Twice,
    /// No code, or a credential of another kind, at the record.
    #[error("svrngs answers a program here only as you: ask through `svrngs ask <route>`, or apps/client/svrngs.mjs, each of which asks with a code made for its own connection")]
    Nothing,
    /// A code on a connection that nothing was proved on.
    #[error("nothing was proved on this connection, so svrngs takes no code on it: POST /v1/proof on it first, then ask on it, as `svrngs ask` and apps/client/svrngs.mjs do")]
    Unproved,
    /// A code made for another connection, or for this one before its
    /// last proof.
    #[error("this code was made for another connection, or before svrngs's last proof on this one: ask with the code for this connection's last proof, as `svrngs ask` and apps/client/svrngs.mjs do")]
    Elsewhere,
}

impl Unasked {
    /// Its refusal, 401 in the record's one shape ([`refuse`]): `no_bearer`
    /// for a Bearer, `unasked` for the rest.
    fn refused(self) -> Response {
        let bearer = matches!(self, Self::Bearer | Self::BearerAtALend);
        let kind = if bearer { "no_bearer" } else { "unasked" };
        refuse(StatusCode::UNAUTHORIZED, kind, self)
    }
}

/// How long svrngs's command waits for svrngs's address to take its
/// connection, and then for svrngs to prove itself on it ([`asked`]): past svrngs's start, which holds
/// its address well before it takes a connection there (node.rs
/// `start_on`: the endpoint and the folders between, some 300 ms on a Mac
/// with its groups, its log said, 2026-11-21), and past a pause of a loaded
/// computer as long, many times over. Waiting gives nothing away, as
/// nothing without the token can prove it holds it; and it is waited only
/// where something took the connection, since nothing listening refuses at
/// once. As long as a share's port has to take a connection
/// (`door::bound::CONNECT_WITHIN`) and Give gives each call (apps/give).
pub const PROVE_WITHIN: std::time::Duration = std::time::Duration::from_secs(10);

/// 32 random bytes as 64 lowercase hex characters, made fresh: a client's
/// challenge for svrngs's proof, for each connection ([`proof`],
/// [`asked`]), and the nonce svrngs gives that connection with it, which
/// its code is made for ([`ask`], [`proved`]); parsed, so svrngs refuses
/// anything else before it proves, and its command any other nonce.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct Challenge(String);

impl Challenge {
    /// Drawn fresh.
    #[expect(clippy::disallowed_methods, reason = "a challenge or a nonce drawn")]
    fn drawn() -> Self {
        let mut drawn = [0u8; 32];
        // As kept_token: getrandom failing is a host with no entropy.
        getrandom::fill(&mut drawn).expect("failed to draw a challenge or a nonce");
        Self(hex::encode(drawn))
    }
}

impl TryFrom<String> for Challenge {
    type Error = &'static str;
    fn try_from(hex: String) -> Result<Self, Self::Error> {
        let fits = hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        fits.then_some(Self(hex)).ok_or("not 64 lowercase hex")
    }
}

/// svrngs's proof that it holds `token`, for `challenge` on the connection
/// from `client` to `server`, each as Rust's `SocketAddr` prints it
/// (`127.0.0.1:50000`, `[::1]:50001`): HMAC-SHA256 under the token,
/// trimmed, over `svrngs proof v1\n<challenge>\n<client>\n<server>`, as 64
/// lowercase hex characters. The one decider of it: `POST /v1/proof` makes
/// it ([`proved`]), the command checks it ([`asked`]), and Give keeps a
/// copy (apps/give/src/record.rs `proof`), held to the same vectors
/// (api_tests.rs). Bound to the connection's two ends, so a proof svrngs
/// made for another connection, one a relay opened to it, proves nothing
/// on this one.
pub fn proof(token: &str, challenge: &Challenge, client: SocketAddr, server: SocketAddr) -> String {
    mac(token, "proof", challenge, (client, server))
}

/// The code a program asks svrngs with as the person holding `token`, on
/// the connection from `client` to `server` that svrngs gave `nonce` with
/// its last proof ([`proved`]), each end as [`proof`] prints it: HMAC-SHA256
/// under the token, trimmed, over `svrngs ask v1\n<nonce>\n<client>\n<server>`,
/// as 64 lowercase hex characters, sent as `Authorization: Svrngs <code>`.
/// The one decider of it, beside the proof's: svrngs checks it
/// ([`Bearer::asked`]), the command makes it ([`asked`]), and Give keeps a
/// copy (apps/give/src/record.rs `ask`), held to the same vectors
/// (api_tests.rs). Useless on any other connection, and on this one after
/// another proof, which gives it another nonce.
pub fn ask(token: &str, nonce: &Challenge, client: SocketAddr, server: SocketAddr) -> String {
    mac(token, "ask", nonce, (client, server))
}

/// HMAC-SHA256 under `token`, trimmed, over `svrngs <what> v1\n<fresh>\n<client>\n<server>`,
/// in lowercase hex: a [`proof`] or an [`ask`]'s code, whose first lines
/// differ, so neither is the other (session_tests.rs `no_mac_…`).
fn mac(token: &str, what: &str, fresh: &Challenge, ends: (SocketAddr, SocketAddr)) -> String {
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, token.trim().as_bytes());
    let message = format!("svrngs {what} v1\n{}\n{}\n{}", fresh.0, ends.0, ends.1);
    hex::encode(ring::hmac::sign(&key, message.as_bytes()))
}

/// What svrngs's `answer` to a challenge says, `wanted` the proof made for
/// this connection's own ends: proved, and the nonce it gave this
/// connection; older, with no proof to give (404), or proved with no nonce,
/// as a svrngs before it stopped taking the token; nothing answering, or
/// nothing in time; or not svrngs proving itself. Compared in constant
/// time (the sign-in provider's `same`).
fn proven(answer: Option<Option<(u16, Vec<u8>)>>, wanted: &str) -> Result<Challenge, Withheld> {
    let (status, said) = answer.ok_or(Withheld::Unanswered)?.ok_or(Withheld::Away)?;
    let said: serde_json::Value = serde_json::from_slice(&said).unwrap_or_default();
    let shown = said["proof"].as_str().unwrap_or_default().as_bytes();
    (status != 404).then_some(()).ok_or(Withheld::Older)?;
    let proved = status == 200 && crate::oidc::provider::same(shown, wanted.as_bytes());
    proved.then_some(()).ok_or(Withheld::Unproven)?;
    let nonce = serde_json::from_value(said["nonce"].clone());
    nonce.map_err(|_| Withheld::Older)
}

/// `POST /v1/proof`, `{"challenge": "<hex>"}`: svrngs's [`proof`] that it
/// holds the token, for a client's fresh challenge on this connection, and
/// a nonce drawn fresh for this connection alone, which its asks' code is
/// made for ([`ask`]), as `{"proof": "<hex>", "nonce": "<hex>"}`; a later
/// proof on the connection gives it another in its place ([`Line`]). Asked
/// with nothing before a client asks anything else ([`asked`]). It tells no
/// one anything of the token: a keyed proof over a challenge the client
/// made, bound to the connection's two ends. Refused to a page in a browser
/// as every route is ([`not_from_a_page`]), and refused 400 a challenge
/// that isn't one, or a connection whose two ends are unknown, which
/// [`serve`] tells each request.
async fn proved(State(token): State<Bearer>, request: Request) -> Response {
    let line = request.extensions().get::<Line>().cloned();
    let server = line.as_ref().and_then(|l| l.server);
    let ends = crate::session::peer(&request).zip(server);
    let body = axum::body::to_bytes(request.into_body(), 1024).await;
    let asked = serde_json::from_slice::<Challenged>(&body.unwrap_or_default());
    let asked = asked.ok().zip(ends).zip(line);
    tracing::debug!(target: "rails", ?ends, proved = asked.is_some(), "api: a proof that svrngs holds the token asked, made for this connection's two ends with a nonce for it alone in place of any before, or refused");
    let Some(((Challenged { challenge }, (client, server)), line)) = asked else {
        let why = "send {\"challenge\": \"<hex>\"}, 32 bytes you made for this connection as 64 lowercase hex characters";
        return refuse(StatusCode::BAD_REQUEST, "no_challenge", why);
    };
    let nonce = Challenge::drawn();
    line.nonce.send_replace(Some(nonce.clone()));
    let proof = proof(token.secret(), &challenge, client, server);
    Json(serde_json::json!({ "proof": proof, "nonce": nonce.0 })).into_response()
}

/// What `POST /v1/proof` is asked.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Challenged {
    challenge: Challenge,
}

/// One connection to svrngs's API, as [`serve`] tells each request on it:
/// the end it reached, svrngs's own, which a proof and a code name as
/// their server, none where it can't be read, and then nothing is proved;
/// and the nonce its last proof gave it ([`proved`]), kept for this
/// connection alone, so a code made for it ([`ask`]) asks nothing on any
/// other. Dropped with the connection.
#[derive(Clone)]
pub struct Line {
    server: Option<SocketAddr>,
    nonce: tokio::sync::watch::Sender<Option<Challenge>>,
}

impl Line {
    /// A connection that reached svrngs at `server`, nothing proved on it
    /// yet: one for each [`serve`] takes, and for each a test makes asking
    /// routes in this process.
    pub fn new(server: Option<SocketAddr>) -> Self {
        let nonce = tokio::sync::watch::Sender::new(None);
        Self { server, nonce }
    }
}

/// What [`as_whom`] marks a request asked with its own connection's code:
/// asked as the person, as the record ([`holds_the_token`]) and a lend's
/// origin (`crate::session::Sessions::shown`) take it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AsYou;

/// The routes this API serves: the rail's, and `apps` — the routes an app
/// asks beside the record ([`crate::surface::router`]) — each asked for
/// `token` ([`holds_the_token`]); `signin`, which keeps its own client
/// secret ([`crate::oidc::provider`]); svrngs's proof that it holds the
/// token, which a program asks before it sends it ([`proved`]); all refused
/// to a page in a browser
/// ([`not_from_a_page`]); and, for a request whose `Host` names the home
/// page or a lend, `pages`: the home page's routes and the lends' origins
/// ([`crate::origin`], u5-2), which answer no route of the record's. All
/// behind the one loopback check.
pub fn router(api: RailApi, apps: Router, signin: Router, pages: Pages, token: Bearer) -> Router {
    let held = axum::middleware::from_fn(holds_the_token);
    Router::new()
        // The ring rail's doors (FIVE_PROGRAMS fp-44): durable append and
        // the log, and the live lane's local buffer half. The bodies mirror
        // the daemon's `routes_rail` doors.
        .route("/v1/rail/append", post(crate::rail::append))
        .route("/v1/rail/log", get(crate::rail::log))
        .route(
            "/v1/rail/live",
            post(crate::rail::live_push).get(crate::rail::live_drain),
        )
        // svrngs: the group's people with their keys, from the register
        // (DESIGN §3), so an app needs no second source.
        .route("/v1/rail/people", get(crate::rail::people))
        .with_state(api)
        .merge(apps)
        .layer(held)
        .merge(signin)
        // svrngs: asked before a program sends the token, so asked without.
        .route("/v1/proof", post(proved).with_state(token.clone()))
        .layer(axum::middleware::from_fn(not_from_a_page))
        .layer(axum::middleware::from_fn_with_state(pages, to_a_lend))
        .layer(axum::middleware::from_fn_with_state(token, as_whom))
        // svrngs: every request's caller is asked too, so a listener that
        // ever served beyond loopback would still answer no one else.
        .layer(axum::middleware::from_fn(from_this_computer))
}

/// The home page's routes and the lends' origins ([`crate::home::router`]).
pub type Pages = (Router, Router);

/// A request whose `Host` is the home page's, `svrngs.localhost`, goes to
/// the home page (sup-2, [`crate::home`]); one whose `Host` is a lend's,
/// `<label>.localhost`, to the lend's origin (u5-2); neither, to the
/// record. The one dispatch by host.
async fn to_a_lend(State((home, lends)): State<Pages>, request: Request, next: Next) -> Response {
    use tower::ServiceExt;
    let host = request.headers().get(axum::http::header::HOST);
    let to = match () {
        _ if crate::home::named(host) => home,
        _ if crate::origin::label_of(host).is_some() => lends,
        _ => return next.run(request).await,
    };
    let Ok(answered) = to.oneshot(request).await;
    answered
}

/// The record is served to this computer's apps, never to a page in a
/// browser (u5-2): a request carrying `Origin` or `Sec-Fetch-Site`, which
/// a browser sends and an app's own HTTP client does not, is refused 403
/// before any door runs. So a lent page, at its origin on this same
/// listener, cannot write the record, nor can any site this computer's
/// browser opens: loopback alone let both in, since a browser on this
/// computer is a caller on loopback.
async fn not_from_a_page(request: Request, next: Next) -> Response {
    let Some(page) = page_of(request.headers()) else {
        return next.run(request).await;
    };
    tracing::warn!(target: "rails", page = ?page, path = %request.uri().path(), "api: refused — a page in a browser asked the record");
    let why = "the record is served to this computer's apps, not to a page in a browser";
    refuse(StatusCode::FORBIDDEN, "from_a_page", why)
}

/// What says a request is a page's in a browser, which a program's own
/// HTTP client doesn't send: its `Origin`, or its `Sec-Fetch-Site`, which a
/// browser sends with every request it makes here (Fetch Metadata; this
/// computer's addresses are potentially trustworthy). Node's `fetch` sends
/// `Sec-Fetch-Mode` alone, so it is a program's. The one decider, for the
/// record ([`not_from_a_page`]) and Play at a lend's address
/// (`crate::play::shown`).
pub(crate) fn page_of(headers: &axum::http::HeaderMap) -> Option<&axum::http::HeaderValue> {
    let origin = headers.get(axum::http::header::ORIGIN);
    origin.or_else(|| headers.get("sec-fetch-site"))
}

/// A refusal in the record's one shape beside the group's
/// ([`crate::rail::refused_as`]): its sentence, and its `kind`.
pub(crate) fn refuse(status: StatusCode, kind: &str, error: impl ToString) -> Response {
    let said = serde_json::json!({ "error": error.to_string(), "kind": kind });
    (status, Json(said)).into_response()
}

/// What a program asks svrngs with, decided once, before any route runs
/// ([`Bearer::asked`]; "Never send the key", V1.13.3): svrngs's token
/// itself, wherever it is shown, and a code not made for this connection's
/// last proof, are refused 401 saying what to do, so the token's bytes
/// reach no handler and no log, the words at a lend's address leading to
/// Play, which a program that insists on a key uses with any text as its
/// key; its own connection's code is marked [`AsYou`]; and anything else
/// goes on as it came, another program's key that Play takes off among it
/// (`crate::play::shown`).
async fn as_whom(State(token): State<Bearer>, mut request: Request, next: Next) -> Response {
    let host = request.headers().get(axum::http::header::HOST);
    let lend = !crate::home::named(host) && crate::origin::label_of(host).is_some();
    let asked = token.asked(&request);
    tracing::debug!(target: "rails", ?asked, lend, path = %request.uri().path(), "api: what a program asked with: svrngs's token itself, refused; a code, taken as you only on the connection it was made for; or neither");
    match asked {
        Some(Ok(())) => drop(request.extensions_mut().insert(AsYou)),
        Some(Err(Unasked::Bearer)) if lend => return Unasked::BearerAtALend.refused(),
        Some(Err(why)) => return why.refused(),
        None => {}
    }
    next.run(request).await
}

/// Serve the record only to a program asking as the person (give-token):
/// with the code for its own connection ([`as_whom`]). Any other is
/// refused 401, saying how to ask, before any door runs.
async fn holds_the_token(request: Request, next: Next) -> Response {
    let asked = request.extensions().get::<AsYou>().is_some();
    let bearer = crate::oidc::provider::bearer(request.headers()).is_ok();
    tracing::debug!(target: "rails", asked, bearer, path = %request.uri().path(), "api: whether the caller asked as you, with its own connection's code");
    match (asked, bearer) {
        (true, _) => next.run(request).await,
        (false, true) => Unasked::Bearer.refused(),
        (false, false) => Unasked::Nothing.refused(),
    }
}

/// Serve a request only from a caller on this computer: the bind guard's
/// one decision ([`check_loopback`]) asked of the caller's address, which
/// [`serve`] hands every request. A caller elsewhere, or one whose address
/// is unknown, is refused 403 naming why, before any door runs. (svrngs:
/// upstream relied on the bind alone; DESIGN §3, "Localhost is the trust
/// boundary".)
async fn from_this_computer(request: Request, next: Next) -> Response {
    let refused = |why: &str| refuse(StatusCode::FORBIDDEN, "not_loopback", why);
    let Some(addr) = crate::session::peer(&request) else {
        tracing::warn!(target: "rails", path = %request.uri().path(), "api: refused — the caller's address is unknown, so it cannot be shown to be on this computer");
        return refused(
            "the caller's address is unknown; the record is served to this computer's apps alone",
        );
    };
    if let Err(refusal) = check_loopback(addr) {
        tracing::warn!(target: "rails", caller = %addr, path = %request.uri().path(), "api: refused — the caller is not on this computer");
        return refused(&refusal.to_string());
    }
    tracing::debug!(target: "rails", caller = %addr, path = %request.uri().path(), "api: a caller on this computer");
    next.run(request).await
}

/// The bind guard, as a function so it has a failing input that can be
/// named without standing an iroh endpoint up. The record is this
/// computer's alone; served anywhere else, it would be published.
pub fn check_loopback(listen: SocketAddr) -> Result<(), ApiError> {
    // svrngs: canonical first, so `::ffff:127.0.0.1` reads as the loopback
    // it is, where `Ipv6Addr::is_loopback` alone says no.
    let loopback = listen.ip().to_canonical().is_loopback();
    loopback.then_some(()).ok_or(ApiError::NotLoopback(listen))
}

/// The API being served, on the address it is bound to. Dropping it stops
/// each listener. (svrngs: upstream's `serve` answered the task alone, for a
/// port its config named; the node binds whatever port the command asks,
/// `0` among them, so the address bound is answered too.)
pub struct Serving {
    /// Where an app on this computer reaches the API.
    pub addr: SocketAddr,
    pub(crate) _tasks: Vec<AbortOnDropHandle<()>>,
}

/// A listener bound on loopback, not yet served. (svrngs: upstream's
/// `serve` bound and served in one call; the node binds before its
/// endpoint, so a refused address leaves no endpoint open, and serves once
/// the endpoint's transport exists, which the live lane dials through.)
pub struct Bound {
    /// The address asked's, then, for 127.0.0.1's, ::1's at its port.
    listeners: Vec<tokio::net::TcpListener>,
    /// Where it listens.
    pub addr: SocketAddr,
}

/// Bind. Refuses a non-loopback address by name, and one another program
/// holds as that. Bound on 127.0.0.1, it binds ::1 at the same port too: a
/// Mac and Linux both name `<label>.localhost` ::1 first, then 127.0.0.1
/// (getaddrinfo, in Python and in Node, 2026-11-21), so a program given a
/// lend's address asks ::1 first, and one that asks the first address
/// alone reaches nothing else. So ::1 at the port is svrngs's for as long
/// as it runs, and no other account answers there: another program
/// holding it is refused as on 127.0.0.1; Linux holds it even while ::1 is
/// off ([`listening`]), so it is svrngs's as it comes on; and svrngs
/// starts without it only where this computer has no IPv6 at all
/// ([`alone`]), on 127.0.0.1 alone, and says so.
pub async fn bind(listen: SocketAddr) -> Result<Bound, ApiError> {
    check_loopback(listen)?;
    let mut listeners = vec![listening(listen)?];
    let addr = (listeners[0].local_addr()).map_err(|e| ApiError::Listen(listen, e))?;
    let six = SocketAddr::from((Ipv6Addr::LOCALHOST, addr.port()));
    if listen.is_ipv4() {
        match listening(six) {
            Ok(listener) => listeners.push(listener),
            Err(ApiError::Listen(_, e)) if alone(&e) => {
                tracing::warn!(target: "rails", %six, error = %e, "api: this computer has no IPv6, so svrngs listens on 127.0.0.1 alone, and no program here can listen at ::1")
            }
            Err(refused) => return Err(refused),
        }
    }
    Ok(Bound { listeners, addr })
}

/// Whether svrngs starts on 127.0.0.1 alone, ::1 refused with `e`: only
/// where this computer has no IPv6 at all, the family refused
/// (`EAFNOSUPPORT`, Linux's 97 and a Mac's 47, errno.h), so no program here
/// can listen at ::1 either. Any other refusal stops it starting, ::1 off
/// on a Mac among them: another account could take ::1 at svrngs's port
/// once it is on, and answer every `<label>.localhost` first.
pub(crate) fn alone(e: &std::io::Error) -> bool {
    e.raw_os_error() == Some(if cfg!(target_os = "macos") { 47 } else { 97 })
}

/// `listen` bound alone, as tokio's `TcpListener::bind` binds one
/// (`SO_REUSEADDR`, a backlog of 1024), or refused by name. On Linux an
/// IPv6 address is bound even while this computer has it off
/// (`IPV6_FREEBIND`, Linux 4.15), as `sysctl net.ipv6.conf.lo.disable_ipv6`
/// turns ::1 off, so svrngs holds ::1 at its port from the start and no
/// other account can take it there once it comes on.
fn listening(listen: SocketAddr) -> Result<tokio::net::TcpListener, ApiError> {
    let refused = |e: std::io::Error| match e.kind() {
        std::io::ErrorKind::AddrInUse => ApiError::Taken(listen),
        _ => ApiError::Listen(listen, e),
    };
    let socket = match listen {
        SocketAddr::V4(_) => tokio::net::TcpSocket::new_v4(),
        SocketAddr::V6(_) => tokio::net::TcpSocket::new_v6(),
    };
    let socket = socket.map_err(refused)?;
    socket.set_reuseaddr(true).map_err(refused)?;
    #[cfg(target_os = "linux")]
    if listen.is_ipv6() {
        socket2::SockRef::from(&socket)
            .set_freebind_v6(true)
            .map_err(refused)?;
    }
    socket.bind(listen).map_err(refused)?;
    socket.listen(1024).map_err(refused)
}

/// Serve `router` ([`router`]'s) on what [`bind`] bound, each listener,
/// each connection's requests told their connection ([`Line`]) and its
/// caller.
pub fn serve(router: Router, bound: Bound) -> Serving {
    use tower::Layer as _;
    let Bound { listeners, addr } = bound;
    tracing::info!(target: "rails", %addr, listeners = listeners.len(), "api: serving the record to this computer's apps, on 127.0.0.1 and on ::1 where it has one");
    let serving = |listener: tokio::net::TcpListener| {
        let routes = router.clone();
        let made = tower::service_fn(move |conn: IncomingStream<'_, TcpListener>| {
            let line = axum::Extension(Line::new(conn.io().local_addr().ok()));
            let caller = axum::Extension(axum::extract::ConnectInfo(*conn.remote_addr()));
            let routes = caller.layer(line.layer(routes.clone()));
            std::future::ready(Ok::<_, std::convert::Infallible>(routes))
        });
        let served = axum::serve(listener, made);
        AbortOnDropHandle::new(tokio::spawn(async move {
            if let Err(e) = served.await {
                tracing::error!(target: "rails", error = %e, "api: listener stopped");
            }
        }))
    };
    let _tasks = listeners.into_iter().map(serving).collect();
    Serving { addr, _tasks }
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
