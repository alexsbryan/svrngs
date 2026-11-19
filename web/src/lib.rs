// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/apps/ring-runtime/src/lib.rs@e1a43bcd2
//! The page's dial — the wasm half of the page a group's link,
//! `<origin>#<token>`, opens on a phone (DESIGN §3, "Joining").
//!
//! A browser cannot open a UDP socket, so it cannot speak iroh natively. This
//! module IS an iroh endpoint compiled to wasm (relay-only): the page dials a
//! computer in the group by its key, through a relay, on the members'
//! protocol the node already accepts, as the person whose key stands in the
//! group, and speaks HTTP/1.1 over the connection, one bi-stream per
//! request, by hyper's client (`http.rs`, u4-4). The
//! relay forwards ciphertext and nothing proxies HTTP — there is no TLS to
//! terminate and no door to expose (PLAN §4, U4's inventory).
//!
//! Proven upstream at these pins: `browser-dial` bd-1 measured every layer
//! below answering against a live node (commonwealth-ai `ralph/DECISIONS.md`,
//! browser-dial-2): `HTTP/1.1 200 OK` carrying a real rail row.
//!
//! Every layer is emitted to the page's `__trace` hook as it happens, never
//! only returned at the end — a layer that hangs must still be named, which is
//! the whole reason the trace exists (ARCH §1, glassbox).

#![deny(unsafe_code)]

mod http;
mod join;
// The node's relay adapter, compiled here too: every relay URL the page
// makes or reads without the DNS root's dot, which WebKit refuses (u4-9).
// Formatted by its own crate's rustfmt.toml, not by this one's. Its n0
// relays and lookup go unused: the page dials none of n0's (relay-default).
#[path = "../../crates/svrngs-node/src/relay.rs"]
#[rustfmt::skip]
#[allow(dead_code)]
mod relay;
// How long a dial to a lender's computer waits to be answered, the node's
// one constant, so a phone and a computer are told an asleep one alike
// (events-asleep).
#[path = "../../crates/svrngs-node/src/dial.rs"]
mod dial;
// This build's version, and the one check of it, the node's (arch-version);
// the page carries no answer, so reads no `VERSION`.
#[path = "../../crates/svrngs-node/src/version.rs"]
#[allow(dead_code)]
mod version;

use iroh::RelayUrl;
use svrngs_core::alpn::Alpn;
use svrngs_core::link::Token;
use svrngs_core::pass::{Label, Pass};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Push one layer line to the page's `__trace` collector, if the page defined
/// one. Best-effort: a missing collector must not break the dial.
fn emit(line: &str) {
    if let Some(f) = get(&js_sys::global(), "__trace") {
        if let Ok(f) = f.dyn_into::<js_sys::Function>() {
            let _ = f.call1(&JsValue::UNDEFINED, &JsValue::from_str(line));
        }
    }
}

/// The property `what` of `of`, when it can be read: how the page reads
/// what the browser holds.
fn get(of: &JsValue, what: &str) -> Option<JsValue> {
    js_sys::Reflect::get(of, &JsValue::from_str(what)).ok()
}

/// iroh's own events for `targets` (tracing-subscriber's `Targets`, as
/// `iroh::socket=trace`), sent to the page's trace: opt-in, by the page's
/// `svrngs-iroh` storage key (web/app.js), so a dial's stall is seen.
#[wasm_bindgen]
pub fn trace_iroh(targets: String) -> Result<(), JsError> {
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;
    let targets: tracing_subscriber::filter::Targets = targets.parse().map_err(|e| JsError::new(&format!("{e}")))?;
    let fmt = tracing_subscriber::fmt::layer().without_time().with_ansi(false);
    let fmt = fmt.with_writer(|| Emitted(Vec::new()));
    let set = tracing_subscriber::registry().with(fmt).with(targets).try_init();
    set.map_err(|e| JsError::new(&e.to_string()))
}

/// One event's line, emitted whole when written.
struct Emitted(Vec<u8>);

impl std::io::Write for Emitted {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for Emitted {
    fn drop(&mut self) {
        emit(String::from_utf8_lossy(&self.0).trim_end());
    }
}

/// A refusal's line is emitted, and is the error.
macro_rules! refuse {
    ($($a:tt)*) => {{
        let line = format!($($a)*);
        emit(&line);
        return Err(JsError::new(&line));
    }};
}

macro_rules! t {
    ($($a:tt)*) => {
        emit(&format!($($a)*))
    };
}

/// A browser endpoint bound with the person's key, online at every relay
/// the page may take, and the token's issuer, whom it dials when no other
/// computer is named.
struct Bound {
    ep: iroh::Endpoint,
    issuer: Option<iroh::PublicKey>,
    relays: Vec<RelayUrl>,
}

/// The computer whose key is `bytes`, refused where they name no key.
fn computer(bytes: &[u8; 32]) -> Result<iroh::PublicKey, JsError> {
    iroh::PublicKey::from_bytes(bytes)
        .or_else(|e| refuse!("LAYER parse: REFUSED — node id is not a valid public key: {e}"))
}

/// Whether this page's policy upgrades insecure requests: the tests' page
/// carries the line; the published page (`web/index.html`, `connect-src
/// 'self' wss: https:`) has no such line, and a worker no document to read.
fn upgrades_insecure() -> bool {
    let document = get(&js_sys::global(), "document");
    let query = document.as_ref().and_then(|d| get(d, "querySelector"));
    let meta = query.and_then(|q| q.dyn_into::<js_sys::Function>().ok()).and_then(|q| {
        let at = JsValue::from_str("meta[http-equiv=\"Content-Security-Policy\"]");
        document.as_ref().and_then(|d| q.call1(d, &at).ok())
    });
    meta.and_then(|m| get(&m, "content"))
        .and_then(|c| c.as_string())
        .is_some_and(|c| c.contains("upgrade-insecure-requests"))
}

/// Whether this page may take `relay`: one over plain HTTP is a route only
/// a page that is not https itself, or one whose policy upgrades insecure
/// requests, may reach. The published page's `connect-src 'self' wss:
/// https:` refuses it, and the mixed content would too, so the route is
/// never tried there — no blocked request is issued for it.
fn this_page_takes(relay: &svrngs_core::relay::Url) -> bool {
    let https_page = get(&js_sys::global(), "location")
        .and_then(|l| get(&l, "protocol"))
        .and_then(|p| p.as_string())
        .is_some_and(|p| p == "https:");
    relay.scheme() == "https" || !https_page || upgrades_insecure()
}

/// The ways to a group: the relays its link carries, today's way in, then
/// `meets`, those its keeper names as the page last read them, each once and
/// each one this page's policy takes.
fn ways_in(link: Vec<svrngs_core::relay::Url>, meets: &[String]) -> Vec<RelayUrl> {
    let named = meets.iter().filter_map(|one| relay::parse(one).ok());
    let mut ways: Vec<RelayUrl> = Vec::new();
    for way in link.into_iter().map(RelayUrl::from).chain(named) {
        if this_page_takes(&way) && !ways.contains(&way) {
            ways.push(way);
        }
    }
    ways
}

/// The relay `via` names, parsed, or none when it names none: refused
/// when it is not one.
fn named(via: &Option<String>) -> Result<Option<RelayUrl>, JsError> {
    (via.as_deref().map(relay::parse).transpose())
        .or_else(|e| refuse!("LAYER parse: REFUSED — {via:?} is not a relay: {e}"))
}

/// Every relay `relays` names, parsed, refused at the first that is not
/// one: what a pass's holder is handed.
fn handed(relays: &[String]) -> Result<Vec<RelayUrl>, JsError> {
    let parsed = relays.iter().map(|relay| relay::parse(relay)).collect::<Result<_, _>>();
    parsed.map_err(|e| JsError::new(&format!("{relays:?} is not a relay: {e}")))
}

/// [`bind`] for the group whose link's token is `token`: its issuer, and
/// every relay it carries this page's policy takes — the public fallback
/// beside the group's own HTTP relay, where that one is one the page may
/// reach (a phone binds one endpoint, relay fallbacks) — refused when it
/// carries none: from a computer that reached no relay, with n0's lookup
/// or without it (`cli::link_relay`, relay-park (c)), or one naming only
/// routes the published policy refuses.
async fn bind_for(key: Vec<u8>, token: &str, meets: &[String], held: bool) -> Result<Bound, JsError> {
    let Some(link) = Token::decode(token) else {
        refuse!("LAYER parse: REFUSED — the fragment is not a link's token: {token}")
    };
    let issuer = computer(link.issuer.as_bytes())?;
    t!("LAYER parse: ok — issuer {issuer}");
    let relays = ways_in(link.relays, meets);
    if relays.is_empty() {
        refuse!("LAYER parse: REFUSED — the link names no relay this page's policy takes")
    }
    bind(key, Some(issuer), relays, held, None).await
}

/// Bind an endpoint as the person whose key is `key` — or a pass's holder —
/// reaching the network through `relays` alone, and wait until it is online,
/// or [`dial::LINK_WITHIN`] passes with no relay answering, when the endpoint
/// is closed and the bind refused. `issuer` is whom it dials when no other
/// computer is named. One that dials lenders is `held` as [`dial::held`]
/// says; the one a link's dial binds for itself waits as iroh does.
///
/// `key` is the person's key, the 32 bytes the page's store holds (DESIGN
/// §3, call 1): iroh signs its handshake with it synchronously, so it is
/// bytes here, never a WebCrypto key. The relays are the group's link's
/// token's, the fragment of `<origin>#<token>`, parsed by svrngs-core as
/// the command parses it (call 5), each the page's policy takes; iroh
/// chooses among them the fastest that answers, and a dial may be tried at
/// every one. A pass's holder (self-host) presents `token` — the pass
/// itself — as the token a group's own relay admits it by, since the fresh
/// key a lend's page dials from stands in no group; the group's page's own
/// endpoint presents none. No bearer rides a request to a lend: the node
/// knows the person by the key that dialled, or the pass its holder shows.
async fn bind(
    key: Vec<u8>,
    issuer: Option<iroh::PublicKey>,
    relays: Vec<RelayUrl>,
    held: bool,
    token: Option<String>,
) -> Result<Bound, JsError> {
    t!("LAYER page: wasm module loaded and dial entered (iroh =1.4.2)");

    // ── LAYER 1: the person's key, then the link's token ───────────────────
    let secret = match <[u8; 32]>::try_from(key.as_slice()) {
        Ok(bytes) => iroh::SecretKey::from_bytes(&bytes),
        Err(_) => refuse!(
            "LAYER key: REFUSED — the page holds {} bytes, not a key's 32",
            key.len()
        ),
    };
    t!("LAYER key: ok — the key {}", secret.public());
    // The relays the link carries, or a pass's holder is handed, and no
    // address lookup: n0's relays and lookup are none of a phone's path
    // (relay-default), as upstream refused a dial string with no relay
    // (ring-runtime/src/lib.rs:91-99). Dotless, as WebKit reaches them.
    relays.iter().for_each(|relay| t!("LAYER parse: ok — relay {relay}"));
    // A pass's holder presents the pass to every relay it was handed: the
    // group's own admits it by the pass (self-host); the build's fallback
    // and n0's ignore the token.
    let map: iroh::RelayMap = relays.iter().cloned().collect();
    let mode = iroh::RelayMode::Custom(match token {
        Some(token) => map.with_auth_token(token),
        None => map,
    });
    let builder = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
        .secret_key(secret)
        .relay_mode(mode)
        .transport_config(if held { dial::held(None) } else { Default::default() });

    // ── LAYER 2: wasm bindgen / iroh endpoint construction ─────────────────
    t!("LAYER wasm-bindgen/iroh-endpoint: binding a browser endpoint (relay-only, held: {held})…");
    // It offers no protocol: a browser is never dialled (DESIGN §3, call 2).
    let ep = match builder.bind().await {
        Ok(e) => e,
        Err(e) => {
            refuse!("LAYER wasm-bindgen/iroh-endpoint: REFUSED — endpoint bind failed: {e}")
        }
    };
    t!(
        "LAYER wasm-bindgen/iroh-endpoint: ok — local endpoint {} bound",
        ep.id()
    );

    // ── LAYER 3: relay connect ─────────────────────────────────────────────
    t!("LAYER relay-connect: waiting for the relay handshake (endpoint.online())…");
    let within = dial::LINK_WITHIN;
    if n0_future::time::timeout(within, ep.online()).await.is_err() {
        ep.close().await;
        refuse!("LAYER relay-connect: REFUSED — no relay answered within {within:?}, and the endpoint is closed")
    }
    t!("LAYER relay-connect: ok — endpoint reports online (relay handshake done)");
    Ok(Bound { ep, issuer, relays })
}

/// Dial `to`, a computer's key in 64 hex, else the token's issuer, from
/// `bound` on `alpn`: through `via`, the relay its group names for it
/// (relay-in-record), when this page may take that one, else every relay
/// the link carries, all tried at once, so a route that is stalled or
/// cannot reach the computer does not delay the one that answers. A pass's
/// holder has no document to judge the page's policy by, so the relay the
/// page handed it for a lender is taken as handed. A connection carries
/// the key dialled and is authenticated as that computer's, so a route is
/// chosen only after reaching it. A dial to a lender is given up when not
/// answered `within` (events-asleep), a member's to the computer its link
/// names within [`dial::LINK_WITHIN`] ([`Endpoint::dial_link`]); a
/// joiner's to it waits as long as iroh does.
async fn reach(
    bound: &Bound,
    to: Option<String>,
    via: Option<String>,
    alpn: Alpn,
    within: Option<std::time::Duration>,
) -> Result<Dialled, JsError> {
    // The issuer's dial is held as iroh holds one, a lender's as
    // `dial::held` says, from whichever endpoint: a lender's 4 s of silence
    // would end the issuer's before its cap did (v1ui-race (a)).
    let held = to.as_ref().map_or_else(Default::default, |_| dial::held(None));
    let node_id = match to {
        None => match bound.issuer {
            Some(issuer) => issuer,
            None => refuse!("LAYER parse: REFUSED — no computer named to dial"),
        },
        Some(to) => match <[u8; 32]>::try_from(hex::decode(&to).unwrap_or_default().as_slice()) {
            Ok(bytes) => computer(&bytes)?,
            Err(_) => refuse!("LAYER parse: REFUSED — {to:?} is not a computer's key"),
        },
    };
    let named = named(&via)?;
    // A pass's holder has no document to judge the page's policy by: every
    // relay it was handed was taken by the page that handed it
    // ([`relays_of`], `relay_taken`), so the one named for a lender is
    // taken as handed. A page judges its own.
    let mut routes = bound.relays.clone();
    match named {
        Some(relay) if bound.issuer.is_none() || this_page_takes(&relay) => {
            t!("LAYER parse: ok — {node_id}'s group names its relay {relay}, tried beside every way this endpoint knows");
            if !routes.contains(&relay) {
                routes.push(relay);
            }
        }
        Some(relay) => t!("LAYER parse: {node_id}'s group names {relay}, which this page's policy does not take, so the link's routes are tried"),
        None => t!("LAYER parse: no relay named for {node_id} — its group names none, or the page refused it — so every relay the link carries is tried"),
    }

    // The relay this endpoint is at, named beside the one that answers.
    let at = bound.ep.addr().relay_urls().next().cloned();
    let opts = iroh::endpoint::ConnectOptions::new().with_transport_config(held);
    use n0_future::StreamExt as _;
    let mut tries = n0_future::FuturesUnordered::new();
    for relay in &routes {
        let (ep, opts, alpn) = (bound.ep.clone(), opts.clone(), alpn.bytes());
        let addr = iroh::EndpointAddr::from_parts(node_id, [iroh::TransportAddr::Relay(relay.clone())]);
        tries.push(async move {
            t!("LAYER tls/alpn+relay: dialing {node_id} on the {alpn:?} protocol through {relay}…");
            let connect =
                async { Ok::<_, iroh::endpoint::ConnectError>(ep.connect_with_opts(addr, alpn, opts).await?.await?) };
            match within {
                Some(within) => (relay, n0_future::time::timeout(within, connect).await),
                None => (relay, Ok(connect.await)),
            }
        });
    }
    let (mut timed_out, mut failed) = (false, None);
    while let Some((relay, dialled)) = tries.next().await {
        match dialled {
            Ok(Ok(conn)) => {
                let own = Some(relay) == at.as_ref();
                t!("LAYER relay chosen: {relay}, this page's own: {own}");
                t!("LAYER tls/alpn+relay: ok — QUIC connection established to {node_id} on the {alpn:?} protocol");
                t!(
                    "LAYER tls/alpn: negotiated ALPN = {:?}",
                    String::from_utf8_lossy(conn.alpn())
                );
                // The HTTP rides a bi-stream per request, opened by
                // `Dialled::fetch`. The tries still in flight are dropped
                // with the set, so no losing route is left dialling.
                return Ok(Dialled { ep: None, conn });
            }
            Ok(Err(e)) => {
                t!("LAYER tls/alpn+relay: a route failed: {e}");
                failed = Some(e);
            }
            Err(_) => timed_out = true,
        }
    }
    match failed {
        Some(e) if !timed_out => {
            refuse!("LAYER tls/alpn+relay: REFUSED — connect() failed on every route: {e}")
        }
        _ => refuse!("LAYER tls/alpn+relay: REFUSED — not answered in {within:?}: asleep"),
    }
}

/// A computer the page has dialled: each request to it is one bi-stream of
/// the connection, spoken by hyper's HTTP/1 client ([`http`]).
#[wasm_bindgen]
pub struct Dialled {
    /// The endpoint the dial bound for itself, closed with it; none when
    /// it was dialled from an [`Endpoint`], which outlives it.
    ep: Option<iroh::Endpoint>,
    conn: iroh::endpoint::Connection,
}

#[wasm_bindgen]
impl Dialled {
    /// Ask the computer `method path` with `headers`, an array of `[name,
    /// value]` sent as named, and `body`, a `ReadableStream` of bytes or
    /// null: a promise of the page's `Response`, its body streamed.
    pub fn fetch(
        &self,
        method: String,
        path: String,
        headers: js_sys::Array,
        body: Option<web_sys::ReadableStream>,
    ) -> js_sys::Promise {
        let conn = self.conn.clone();
        let open = conn.close_reason().is_none();
        wasm_bindgen_futures::future_to_promise(async move {
            match http::fetch(conn.clone(), method.clone(), path.clone(), headers, body).await {
                Ok(response) => Ok(response.into()),
                Err(e) => {
                    // Given up while asked, its computer silent for the whole
                    // idle timeout (`dial::held`): asleep, which the worker
                    // says without dialling again. One given up before it
                    // was asked is dialled again.
                    let silent = open && matches!(conn.close_reason(), Some(iroh::endpoint::ConnectionError::TimedOut));
                    let e = if silent { format!("asleep: {e}") } else { e.to_string() };
                    emit(&format!("HTTP: {method} {path} — REFUSED — {e}"));
                    Err(js_sys::Error::new(&e).into())
                }
            }
        })
    }

    /// Ask the computer to switch `path` to another protocol, `headers` an
    /// array of `[name, value]` naming it (a WebSocket's `Upgrade`, u5-2): a
    /// promise of `{status, headers, readable, writable}`, the raw bytes of
    /// the switched stream on the last two once `status` is 101. A service
    /// worker sees no WebSocket, so the lend's worker speaks its frames over
    /// these.
    pub fn socket(&self, path: String, headers: js_sys::Array) -> js_sys::Promise {
        let conn = self.conn.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            http::socket(conn, &path, &headers).await.map_err(|e| {
                emit(&format!("HTTP: a socket at {path} — REFUSED — {e}"));
                js_sys::Error::new(&e.to_string()).into()
            })
        })
    }

    /// Close the dial, and every stream with it: its endpoint when it
    /// bound one for itself, else its connection.
    pub fn close(&self) -> js_sys::Promise {
        let (ep, conn) = (self.ep.clone(), self.conn.clone());
        wasm_bindgen_futures::future_to_promise(async move {
            match ep {
                Some(ep) => ep.close().await,
                None => conn.close(0u32.into(), b"done"),
            }
            Ok(JsValue::UNDEFINED)
        })
    }
}

/// Dial the computer that issued `token` on the redeem protocol, the one a
/// key standing in no group may speak, to ask what the link is for and
/// redeem it ([`join`]'s bodies), from an endpoint of its own, waiting as
/// iroh does: the dial's [`Dialled::close`] closes it. (A member dials the
/// link's computer by [`Endpoint::dial_link`]; #172 took the second way.)
#[wasm_bindgen]
pub async fn connect_to_join(key: Vec<u8>, token: String) -> Result<Dialled, JsError> {
    let bound = bind_for(key, &token, &[], false).await?;
    let conn = reach(&bound, None, None, Alpn::Redeem, None).await?.conn;
    let ep = Some(bound.ep);
    Ok(Dialled { ep, conn })
}

/// Bind one endpoint as the person whose key is `key`, in the group whose
/// link's token is `token`, to dial each lender from ([`Endpoint::dial`]):
/// the service worker's (DESIGN §3, call 3). A relay keeps one endpoint
/// per key and sets the older aside when another binds
/// (iroh-relay-1.0.2/src/server/clients.rs:80-92), so the worker holds one
/// for every lender it reaches, never one per dial.
#[wasm_bindgen]
pub async fn endpoint(key: Vec<u8>, token: String, meets: Option<Vec<String>>) -> Result<Endpoint, JsError> {
    Ok(Endpoint {
        bound: bind_for(key, &token, &meets.unwrap_or_default(), true).await?,
    })
}

/// Bind one endpoint as a pass's holder, whose key is `key`, reaching the
/// group through `relays` — every relay the group's link carries that this
/// page's policy takes, its public fallback among them — refused with
/// none: a lend's origin's worker (u5-2; DESIGN §3, "Lent apps", call 6).
/// It is handed no link's token, which would let it join the group, and
/// `pass` is presented at the relays it is handed, since a group's own
/// relay admits the holder by it, the fresh key standing in no group
/// (self-host).
#[wasm_bindgen]
pub async fn holder_endpoint(key: Vec<u8>, relays: Vec<String>, pass: String) -> Result<Endpoint, JsError> {
    let parsed = handed(&relays)?;
    if parsed.is_empty() {
        refuse!("LAYER parse: REFUSED — no relay handed, and the page asks n0's lookup for none")
    }
    Ok(Endpoint {
        bound: bind(key, None, parsed, true, Some(pass)).await?,
    })
}

/// The first relay a link's `token` carries, dotless, or none: whether a
/// link is one this build can read at all (u5-2).
#[wasm_bindgen]
pub fn relay_of(token: String) -> Result<Option<String>, JsError> {
    let link = Token::decode(&token).ok_or_else(|| JsError::new("not a link's token"))?;
    Ok(link.relays.first().map(|r| RelayUrl::from(r.clone()).to_string()))
}

/// Every relay a link's `token` carries that this page's policy takes,
/// dotless, in the order the link carries them: what a pass's holder is
/// handed in place of the token, so an app does not revert to a route the
/// page refuses (u5-2; relay fallbacks).
#[wasm_bindgen]
pub fn relays_of(token: String, meets: Option<Vec<String>>) -> Result<Vec<String>, JsError> {
    let link = Token::decode(&token).ok_or_else(|| JsError::new("not a link's token"))?;
    let ways = ways_in(link.relays, &meets.unwrap_or_default());
    Ok(ways.iter().map(ToString::to_string).collect())
}

/// Whether this page's policy takes `relay`, a URL: what a page asks
/// before handing a lend's worker the relay its record names for a lender,
/// so the worker is never handed one the page refuses (relay fallbacks).
#[wasm_bindgen]
pub fn relay_taken(relay: String) -> bool {
    relay::parse(&relay).is_ok_and(|url| this_page_takes(&url))
}

/// The key of the computer that made the link `token`, 64 hex: the one
/// [`Endpoint::dial_link`] dials (v1ui-reach (c)).
#[wasm_bindgen]
pub fn issuer_of(token: String) -> Result<String, JsError> {
    let link = Token::decode(&token).ok_or_else(|| JsError::new("not a link's token"))?;
    Ok(hex::encode(link.issuer.as_bytes()))
}

/// Whether a computer whose answer carries `version`, none when it carries
/// none, runs an older svrngs than this page: the node's one rule
/// ([`version::older`]), so a page and a computer judge it alike.
#[wasm_bindgen]
pub fn older(version: Option<u32>) -> bool {
    version::older(version)
}

/// Whether a computer whose answer carries `version` takes a pass carried
/// by its holder's link: the node's rule ([`version::links`]).
#[wasm_bindgen]
pub fn links(version: Option<u32>) -> bool {
    version::links(version)
}

/// This build's version, as a discovery ask carries it: the one constant
/// the page sends in `x-svrngs-version` at the phone's discovery edge
/// (SCOPE_PARALLEL.md), so a page and a computer agree.
#[wasm_bindgen]
pub fn current_version() -> u32 {
    version::VERSION
}

/// The label of the lend `name` of the kind `kind` lent by the computer
/// whose key is `lender`, 64 hex, in `group` when its group carries it, as
/// a computer of `version` lists it: one DNS label, its origin's name
/// (u5-2; svrngs-core's `Label::lent`, the computer's rule too).
#[wasm_bindgen]
pub fn label(
    lender: String,
    name: String,
    kind: String,
    group: Option<String>,
    version: Option<u32>,
) -> Result<String, JsError> {
    let key: [u8; 32] = (hex::decode(&lender).ok().and_then(|k| k.try_into().ok()))
        .ok_or_else(|| JsError::new("not a computer's key"))?;
    let carried = group.as_deref().filter(|_| version::carried_once(version));
    Ok(Label::lent(&key, &name, carried.map(|g| (kind.as_str(), g))).to_string())
}

/// Whether `raw` is a lend's label, as svrngs-core reads one
/// (`Label::parse`): a host whose first name is one is a lend's origin, so
/// the group's page there is one the lend's worker is not serving
/// (web/app.js `cold`).
#[wasm_bindgen]
pub fn is_label(raw: String) -> bool {
    Label::parse(&raw).is_some()
}

/// The label a port's app lent by `lender`, 64 hex, writes the kind
/// `kind`'s part of `group` under: what its write pass names, so the fold
/// lets its holder write as its person (svrngs-core's `grant.rs` `carries`;
/// capability-signed). A carried app's own label already does.
#[wasm_bindgen]
pub fn carried_label(lender: String, kind: String, group: String) -> Result<String, JsError> {
    let name = svrngs_core::pass::carried_name(&kind, &group);
    Label::of_hex(&lender, &name)
        .map(|l| l.to_string())
        .ok_or_else(|| JsError::new("not a computer's key"))
}

/// The person whose key is `person` lets the key `holder`, 64 hex, act as
/// them for the app `app` toward it and the lends `reads` until `until`,
/// seconds since 1970: the pass, as the door reads it from a request (u5-2,
/// sup-1; svrngs-core's `Pass`).
#[wasm_bindgen]
pub fn pass(
    person: Vec<u8>,
    holder: String,
    app: String,
    reads: Vec<String>,
    until: f64,
    approver: Option<String>,
) -> Result<String, JsError> {
    let parse = |l: &String| Label::parse(l).ok_or_else(|| JsError::new(&format!("{l} is not a lend's label")));
    let app = parse(&app)?;
    let reads = reads.iter().map(parse).collect::<Result<Vec<_>, _>>()?;
    Ok(Pass::approvable(&signing(person)?, &holder, (app, reads), until as i64, approver).encode())
}

/// The key of 32 bytes `key`, as the record spells an actor: a holder's,
/// which a pass names.
#[wasm_bindgen]
pub fn actor(key: Vec<u8>) -> Result<String, JsError> {
    Ok(svrngs_core::actor_of(&signing(key)?))
}

fn signing(key: Vec<u8>) -> Result<svrngs_core::SigningKey, JsError> {
    let bytes = <[u8; 32]>::try_from(key.as_slice()).map_err(|_| JsError::new("a key is 32 bytes"))?;
    Ok(svrngs_core::SigningKey::from_bytes(&bytes))
}

/// The bytes the holder of the pass `pass` signs for its link to the key
/// `to`, 64 hex, and, given its signature `sig`, the link as the door reads
/// it beside the pass (`x-svrngs-link`; svrngs-core's `authority::resolve`):
/// each worker run's endpoint key, named by a holder no page can copy
/// (capability-signed; DESIGN §3).
#[wasm_bindgen]
pub fn link(pass: String, to: String, sig: Option<Vec<u8>>) -> Result<Vec<u8>, JsError> {
    let mut link = Pass::decode(&pass).ok_or_else(|| JsError::new("not a pass"))?.link(&to);
    match sig {
        None => Ok(link.message()),
        Some(sig) => {
            link.sig = hex::encode(sig);
            Ok(link.encode().into_bytes())
        }
    }
}

/// The line the key `key` signs for `act`, JSON, in the group `group`, at
/// `seq` after the line whose id is `prev`, at `ts` seconds since 1970: a
/// phone's own yes, which any computer carries unchanged (capability-signed;
/// svrngs-core's `sign_act`, the one place an act is made).
#[wasm_bindgen]
pub fn sign_line(
    key: Vec<u8>,
    group: String,
    ts: f64,
    seq: f64,
    prev: Option<String>,
    act: String,
) -> Result<String, JsError> {
    let key = signing(key)?;
    let mut line = unsigned(svrngs_core::actor_of(&key), group, (ts, seq, prev), act, None)?;
    line.kind.sig = svrngs_core::sign_ring_op(&key, &line.id);
    Ok(serde_json::to_string(&line)?)
}

/// [`sign_line`]'s line for the key `actor`, 64 hex, before its signature,
/// under the pass `grant`, and the bytes its holder signs: a phone's act in
/// an app it opened, signed by a holder no page can copy (svrngs-core's
/// `unsigned_act`).
#[wasm_bindgen]
pub fn unsigned_line(
    actor: String,
    group: String,
    ts: f64,
    seq: f64,
    prev: Option<String>,
    act: String,
    grant: String,
) -> Result<js_sys::Array, JsError> {
    let grant = Pass::decode(&grant).ok_or_else(|| JsError::new("not a pass"))?;
    let line = unsigned(actor, group, (ts, seq, prev), act, Some(grant))?;
    let message = js_sys::Uint8Array::from(svrngs_core::ring_op_message(&line.id).as_slice());
    Ok(js_sys::Array::of2(&serde_json::to_string(&line)?.into(), &message))
}

/// The signer's approval of `line`, the app's entry in `group`, at `now`
/// by its own clock (web/sign.js; HOLDER_APPROVAL §3.D): the grant with the
/// entry named, as JSON, and the bytes the approver `approver` signs — the
/// signer's key never leaves WebCrypto, so it signs them there and puts
/// the signature in the grant's `approval`. Only while the open's `reach` pass is live
/// for the line's holder, `grant` the one the line carries and one the open
/// issued, both naming `approver`, of one lender's app, and the line
/// stamped within five minutes of `now`.
#[wasm_bindgen]
pub fn approve_line(
    approver: String,
    group: String,
    reach: String,
    grant: String,
    line: String,
    now: f64,
) -> Result<js_sys::Array, JsError> {
    let reach = Pass::decode(&reach).ok_or_else(|| JsError::new("not a reach pass"))?;
    let issued = Pass::decode(&grant).ok_or_else(|| JsError::new("not an issued grant"))?;
    let line: svrngs_core::Op<svrngs_core::SignedOp> = serde_json::from_str(&line)?;
    let pass = svrngs_core::approve_in_open(&approver, &group, &reach, &issued, &line, now as i64)
        .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(js_sys::Array::of2(
        &serde_json::to_string(&pass)?.into(),
        &js_sys::Uint8Array::from(pass.approval_message().as_slice()).into(),
    ))
}

type Stamp = (f64, f64, Option<String>);

fn unsigned(
    actor: String,
    group: String,
    (ts, seq, prev): Stamp,
    act: String,
    grant: Option<Pass>,
) -> Result<svrngs_core::Op<svrngs_core::SignedOp>, JsError> {
    let kind = svrngs_core::SignedOp {
        act: svrngs_core::RailAct::from_json(serde_json::from_str(&act)?).map_err(|e| JsError::new(&e.to_string()))?,
        prev: prev.map(|p| serde_json::from_value(p.into())).transpose()?,
        seq: seq as u64,
        sig: String::new(),
        on_behalf_of: None,
        view: None,
        grant,
    };
    Ok(svrngs_core::unsigned_act(actor, &group, ts as i64, kind))
}

/// The act, JSON, an app `kind`'s page asks for with `body`, as the door
/// shapes it (svrngs-core's `app_act`); none for a body that is not one.
#[wasm_bindgen]
pub fn app_act(kind: String, body: Vec<u8>) -> Option<String> {
    svrngs_core::grant::app_act(&kind, &body).map(|act| act.to_string())
}

/// A browser endpoint bound with the person's key, online at a relay.
#[wasm_bindgen]
pub struct Endpoint {
    bound: Bound,
}

#[wasm_bindgen]
impl Endpoint {
    /// Dial the computer whose key is `lender`, 64 hex, through `relay`,
    /// the one its group names, on the members' protocol: a phone dials
    /// whoever lends a service itself (DESIGN §3, call 4). The dial's
    /// [`Dialled::close`] closes only its connection.
    pub async fn dial(&self, lender: String, relay: Option<String>) -> Result<Dialled, JsError> {
        reach(&self.bound, Some(lender), relay, Alpn::Member, Some(dial::DIAL_WITHIN)).await
    }

    /// Dial the computer that made the group's link, on the members'
    /// protocol, given up when not answered within [`dial::LINK_WITHIN`],
    /// from this endpoint, so the page dials it and the lenders at once
    /// from the one endpoint a relay keeps per key (v1ui-race (a)).
    pub async fn dial_link(&self) -> Result<Dialled, JsError> {
        reach(&self.bound, None, None, Alpn::Member, Some(dial::LINK_WITHIN)).await
    }

    /// Dial the computer whose key is `lender` through `relay`, as
    /// [`Self::dial`], on the pass holders' protocol, as a pass's holder:
    /// every request presents the pass (u5-2).
    pub async fn dial_lend(&self, lender: String, relay: Option<String>) -> Result<Dialled, JsError> {
        reach(&self.bound, Some(lender), relay, Alpn::Lend, Some(dial::DIAL_WITHIN)).await
    }

    /// Dial the computer whose key is `lender` through `relay` on the
    /// WebRTC protocol, as a phone offering a direct path does (webrtc-page;
    /// `crates/svrngs-node/src/webrtc.rs` `OFFER_ROUTE`): the offer and its
    /// answer cross on this connection, the pass presented with it.
    pub async fn dial_webrtc(&self, lender: String, relay: Option<String>) -> Result<Dialled, JsError> {
        reach(&self.bound, Some(lender), relay, Alpn::WebRtc, Some(dial::DIAL_WITHIN)).await
    }

    /// Close the endpoint, and every dial from it.
    pub async fn close(&self) {
        self.bound.ep.close().await;
    }
}

// ── the direct path (webrtc-page; RELAY_COMMONS design 4) ───────────────
//
// The lend worker's fresh data endpoint, ported from the spike's `Data`
// (svrn-docs `rings/current/webrtc-transport-spike/page/src/lib.rs`, the
// port map): its own key and the wire whose datagrams the lend's window
// carries — a service worker has no RTCPeerConnection, so the window holds
// the channel and passes each datagram here over a port. It dials WebRTC
// alone, then again with the relay beside it, since iroh takes a custom
// path only on a connection whose first path is not the relay (the spike's
// Q1); the offer naming this endpoint's key crosses on the run's own dial
// (`Endpoint::dial_webrtc`).

// The wire, the node's file compiled here too (the port map). Formatted
// by its own crate's rustfmt.toml, not by this one's, as `relay` above is.
#[path = "../../crates/svrngs-node/src/webrtc/wire.rs"]
#[rustfmt::skip]
mod wire;
// The path timing both ends carry, the node's file too (webrtc-page; the
// spike's README, "The pre-requisites" 1), formatted as the node's.
#[path = "../../crates/svrngs-node/src/webrtc/paths.rs"]
#[rustfmt::skip]
mod paths;

/// The wire's receive side: the sender the window's channel's datagrams
/// go in through, each attempt's own (the director's finding 2, 3d40e17c:
/// thread-locals here were one per worker, so a second attempt there sent
/// the first one's datagrams to its own window).
type Arrived = tokio::sync::mpsc::UnboundedSender<(wire::PeerKey, Vec<u8>)>;

/// A datagram the window's channel brought, from `peer`, into `arrived`.
/// A free function, not a method: the data endpoint's dial holds its object
/// borrowed while it waits, and wasm-bindgen refuses every other call on
/// it meanwhile, so the handshake's own replies would be thrown away (the
/// spike's README, "A trap on the way"; page/src/lib.rs:210-235).
fn data_push(arrived: &Arrived, peer: wire::PeerKey, datagram: js_sys::Uint8Array) {
    let _ = arrived.send((peer, datagram.to_vec()));
}

/// The fresh data endpoint of a lend's direct path, and the connection it
/// dials: the spike's `Data` (page/src/lib.rs:231-337).
#[wasm_bindgen]
pub struct Direct {
    data: iroh::Endpoint,
    conn: Option<iroh::endpoint::Connection>,
    peer: iroh::EndpointId,
    relay: Option<RelayUrl>,
    arrived: Arrived,
}

/// Bind the data endpoint for the lender whose key is `lender`, 64 hex,
/// reaching the network through `relays` (the pass's, as the run's
/// endpoint is), dialling the lender through `relay` when it is named:
/// every datagram out goes to `out`, and what the window's channel brings
/// comes in through [`data_push`].
#[wasm_bindgen]
pub async fn direct_start(
    lender: String,
    relay: Option<String>,
    relays: Vec<String>,
    out: js_sys::Function,
) -> Result<Direct, JsError> {
    let lender: iroh::EndpointId = lender.parse().map_err(|e| {
        JsError::new(&format!(
            "LAYER parse: REFUSED — {lender:?} is not a computer's key: {e}"
        ))
    })?;
    let named = named(&relay)?;
    let parsed = handed(&relays)?;
    // A pass kept before relay fallbacks names one relay and no `relays`:
    // the relay named for the lend is the data endpoint's way in then
    // (web/lend.js `direct`, as `opened` reads them).
    let parsed = if parsed.is_empty() {
        named.iter().cloned().collect()
    } else {
        parsed
    };
    if parsed.is_empty() {
        refuse!("LAYER parse: REFUSED — no relay handed, and the page asks n0's lookup for none")
    }
    let secret = iroh::SecretKey::generate();
    let me = *secret.public().as_bytes();
    let (arrived, arrivals) = tokio::sync::mpsc::unbounded_channel();
    // The wire's send must be `Send`, which `out` is not: it takes the
    // datagram into this attempt's own queue, and a task of this thread,
    // owning `out`, hands each to the window. The task ends when the wire,
    // and so the endpoint, is dropped.
    let (sent, mut sending) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    wasm_bindgen_futures::spawn_local(async move {
        while let Some(datagram) = sending.recv().await {
            let _ = out.call1(&JsValue::NULL, &js_sys::Uint8Array::from(&datagram[..]));
        }
    });
    let send: wire::Send = std::sync::Arc::new(move |_to: &wire::PeerKey, datagram: &[u8]| {
        let _ = sent.send(datagram.to_vec());
    });
    let data = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
        .secret_key(secret)
        .relay_mode(iroh::RelayMode::custom(parsed.clone()))
        .add_custom_transport(wire::Wire::new(&me, arrivals, send))
        .transport_config(paths::quick_paths())
        .bind()
        .await
        .map_err(|e| {
            JsError::new(&format!(
                "LAYER wasm-bindgen/iroh-endpoint: REFUSED — data endpoint bind failed: {e}"
            ))
        })?;
    Ok(Direct {
        data,
        conn: None,
        peer: lender,
        relay: named.or_else(|| parsed.first().cloned()),
        arrived,
    })
}

#[wasm_bindgen]
impl Direct {
    /// The data endpoint's key, which the offer names.
    pub fn key(&self) -> String {
        self.data.id().to_string()
    }

    /// The offer `offer` (the window's SDP, JSON) POSTed to the lender
    /// through `dial`, on the WebRTC protocol, with `headers` (the pass
    /// and the link this run's dials present): the lender's answer, JSON.
    pub async fn signal(&self, dial: &Dialled, headers: js_sys::Array, offer: String) -> Result<String, JsError> {
        let body = format!("{{\"key\":\"{}\",\"offer\":{}}}", self.data.id(), offer);
        let pair = |name: &str, value: &str| js_sys::Array::of2(&name.into(), &value.into());
        headers.push(&pair("content-type", "application/json"));
        headers.push(&pair("content-length", &body.len().to_string()));
        let blob = web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&body.into()))
            .map_err(|e| JsError::new(&format!("the offer is not a body: {e:?}")))?;
        let answer = http::fetch(
            dial.conn.clone(),
            "POST".into(),
            paths::OFFER_ROUTE.into(),
            headers,
            Some(blob.stream()),
        )
        .await
        .map_err(|e| JsError::new(&format!("the offer was not answered: {e}")))?;
        let said = wasm_bindgen_futures::JsFuture::from(answer.text().map_err(page)?).await;
        if !answer.ok() {
            let words = said.ok().and_then(|t| t.as_string()).unwrap_or_default();
            refuse!("the offer was refused: {} {words}", answer.status())
        }
        Ok(said.map_err(page)?.as_string().unwrap_or_default())
    }

    /// The two dials of the spike's Q1: WebRTC alone, then again with the
    /// relay beside it, so the relay is backup on a connection whose first
    /// path is not the relay. iroh may establish the second on the relay
    /// alone, the custom address untried (the spike's Q1: dialled with the
    /// relay beside it, iroh picks the relay) — so the first is kept too,
    /// its path the window's channel, and whichever holds the channel
    /// carries the dial: the second when it has it, its relay backup with
    /// it, else the first, whose relay the remote's addresses add.
    pub async fn dial(&mut self) -> Result<(), JsError> {
        let custom = iroh::TransportAddr::Custom(wire::addr_of(self.peer.as_bytes()));
        let first = self
            .data
            .connect(
                iroh::EndpointAddr::from_parts(self.peer, [custom.clone()]),
                Alpn::Lend.bytes(),
            )
            .await
            .map_err(|e| JsError::new(&format!("dial webrtc: {e}")))?;
        let mut at = vec![custom];
        if let Some(relay) = &self.relay {
            at.push(iroh::TransportAddr::Relay(relay.clone()));
        }
        let second = self
            .data
            .connect(iroh::EndpointAddr::from_parts(self.peer, at), Alpn::Lend.bytes())
            .await
            .map_err(|e| JsError::new(&format!("dial both: {e}")))?;
        self.conn = Some(if on_the_channel(&second, false) { second } else { first });
        Ok(())
    }

    /// The data connection, as a dial the worker's fetches go through
    /// (`web/lend.js` `dials`), or none before [`Self::dial`] returned.
    pub fn dialled(&self) -> Option<Dialled> {
        self.conn.clone().map(|conn| Dialled { ep: None, conn })
    }

    /// The direct attempt, driven here so the worker's JavaScript stays
    /// thin (webrtc-page): the window is asked over `port` — the worker's
    /// end of the channel it announced itself on (`{webrtc: 'here'}`,
    /// shim.js) — for the offer, which is signalled to the lender through
    /// `dial` with `headers` (the pass and the link the run's own dials
    /// present); the answer is handed back; and once the window says the
    /// channel is open, the data endpoint dials, its channel's datagrams
    /// from then on going in through [`data_push`]. Answers the data
    /// connection as a dial the worker's fetches go through, or none when
    /// the channel never opened.
    pub async fn go(
        &mut self,
        dial: &Dialled,
        headers: js_sys::Array,
        port: web_sys::MessagePort,
    ) -> Result<Option<Dialled>, JsError> {
        let ask = js_sys::Object::new();
        js_sys::Reflect::set(&ask, &"webrtc".into(), &"ask".into()).map_err(page)?;
        port.post_message(ask.as_ref()).map_err(page)?;
        let offer = told(&port, "offer").await?;
        let answer = self
            .signal(dial, headers, offer.as_string().unwrap_or_default())
            .await?;
        let said = js_sys::Object::new();
        js_sys::Reflect::set(&said, &"answer".into(), &answer.into()).map_err(page)?;
        port.post_message(&said).map_err(page)?;
        told(&port, "open").await?;
        // The channel is open: what it brings goes to the free `data_push`,
        // and the data endpoint dials (WebRTC alone, then with the relay).
        let (arrived, peer) = (self.arrived.clone(), *self.peer.as_bytes());
        each_told(&port, "datagram", move |bytes| {
            data_push(&arrived, peer, js_sys::Uint8Array::new(&bytes))
        });
        self.dial().await?;
        // The direct path is the attempt's point, and iroh may settle the
        // connection on the relay first (the channel is still opening),
        // leaving the film there: wait, bounded, for `webrtc` to be the
        // selected path — the spike's Q1, 100 ms there — for as long as a
        // silent path lives (`paths::PATH_IDLE`), since one not selected by
        // then is given up. The relay carries if it never is, said in the
        // trace.
        let by = n0_future::time::Instant::now() + paths::PATH_IDLE;
        while !self.webrtc_selected() && n0_future::time::Instant::now() < by {
            n0_future::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        if !self.webrtc_selected() {
            t!("RUNTIME: the direct path settled without webrtc: {}", self.paths());
        }
        Ok(self.dialled())
    }

    /// Whether the data connection's selected path is the custom one (the
    /// window's channel), as [`Self::paths`] spells it.
    fn webrtc_selected(&self) -> bool {
        self.conn.as_ref().is_some_and(|conn| on_the_channel(conn, true))
    }

    /// The data connection's paths now: each one's kind, whether it is the
    /// one selected, and its round trip, as JSON (the spike's `paths`).
    pub fn paths(&self) -> String {
        let Some(conn) = self.conn.as_ref() else {
            return "[]".into();
        };
        let list: Vec<serde_json::Value> = conn
            .paths()
            .iter()
            .map(|p| {
                let kind = match p.remote_addr() {
                    iroh::TransportAddr::Relay(_) => "relay",
                    iroh::TransportAddr::Custom(_) => "webrtc",
                    _ => "other",
                };
                serde_json::json!({ "kind": kind, "selected": p.is_selected(), "rtt_ms": p.rtt().as_millis() as u64 })
            })
            .collect();
        serde_json::Value::Array(list).to_string()
    }
}

/// Whether `conn`'s paths hold the custom one (the window's channel), as
/// [`Direct::paths`] spells it, the selected one when `only_selected`: the
/// connection the dial is carried by, and whether the channel carries it.
fn on_the_channel(conn: &iroh::endpoint::Connection, only_selected: bool) -> bool {
    let custom = |addr: &iroh::TransportAddr| matches!(addr, iroh::TransportAddr::Custom(_));
    (conn.paths().iter()).any(|p| (p.is_selected() || !only_selected) && custom(p.remote_addr()))
}

/// `what`'s value in a message `port` brings, as a promise: the window's
/// answer to the direct attempt's ask (webrtc-page). Later messages are
/// let be, and a value the window never sends leaves it waiting.
async fn told(port: &web_sys::MessagePort, what: &str) -> Result<JsValue, JsError> {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        each_told(port, what, move |value| drop(resolve.call1(&JsValue::NULL, &value)));
    });
    wasm_bindgen_futures::JsFuture::from(promise).await.map_err(page)
}

/// Each value at `what` in a message `port` brings, handed to `each`, in
/// place of whatever the port handed its messages to before: how the
/// worker hears the window (webrtc-page).
fn each_told(port: &web_sys::MessagePort, what: &str, mut each: impl FnMut(JsValue) + 'static) {
    let what = what.to_owned();
    let closure = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        if let Some(value) = get(&event.data(), &what).filter(|v| !v.is_undefined()) {
            each(value);
        }
    }) as Box<dyn FnMut(web_sys::MessageEvent)>);
    port.set_onmessage(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}

/// The page's own words for a value that could not be read (the wasm's
/// `JsValue` errors): as [`refuse!`] and [`Dialled::fetch`] say them.
fn page(e: JsValue) -> JsError {
    JsError::new(&format!("{e:?}"))
}
