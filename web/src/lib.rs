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
//! terminate and no door to expose (DESIGN §8, U4's inventory).
//!
//! Proven upstream at these pins: `browser-dial` bd-1 measured every layer
//! below answering against a live node (commonwealth-ai `ralph/DECISIONS.md`,
//! browser-dial-2): `HTTP/1.1 200 OK` carrying a real rail row.
//!
//! Every layer is emitted to the page's `__trace` hook as it happens, never
//! only returned at the end — a layer that hangs must still be named, which is
//! the whole reason the trace exists (ARCH §1, glassbox).

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

use svrngs_core::alpn::Alpn;
use svrngs_core::link::Token;
use svrngs_core::pass::{Label, Pass};
use wasm_bindgen::prelude::*;

/// Push one layer line to the page's `__trace` collector, if the page defined
/// one. Best-effort: a missing collector must not break the dial.
fn emit(line: &str) {
    if let Ok(f) = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("__trace")) {
        if let Ok(f) = f.dyn_into::<js_sys::Function>() {
            let _ = f.call1(&JsValue::UNDEFINED, &JsValue::from_str(line));
        }
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

/// A browser endpoint bound with the person's key, online at the relay
/// the link's token names, and the token's issuer, whom it dials when no
/// other computer is named.
struct Bound {
    ep: iroh::Endpoint,
    issuer: Option<iroh::PublicKey>,
    relay: iroh::RelayUrl,
}

/// [`bind`] for the group whose link's token is `token`: its issuer, and
/// its relay, refused when it carries none — from a computer that reached
/// no relay, with n0's lookup or without it (`cli::link_relay`, relay-park
/// (c)): the page asks no lookup (relay-default).
async fn bind_for(key: Vec<u8>, token: &str) -> Result<Bound, JsError> {
    let Some(link) = Token::decode(token) else {
        refuse!("LAYER parse: REFUSED — the fragment is not a link's token: {token}")
    };
    let issuer = match iroh::PublicKey::from_bytes(link.issuer.as_bytes()) {
        Ok(p) => p,
        Err(e) => {
            refuse!("LAYER parse: REFUSED — node id is not a valid public key: {e}")
        }
    };
    t!("LAYER parse: ok — issuer {issuer}");
    let Some(relay) = link.relay else {
        refuse!("LAYER parse: REFUSED — the link names no relay, and the page asks n0's lookup for none")
    };
    bind(key, Some(issuer), relay.into()).await
}

/// Bind an endpoint as the person whose key is `key` — or a pass's holder —
/// reaching the network through `relay` alone, and wait until it is online. `issuer` is whom it dials when no other
/// computer is named.
///
/// `key` is the person's key, the 32 bytes the page's store holds (DESIGN
/// §3, call 1): iroh signs its handshake with it synchronously, so it is
/// bytes here, never a WebCrypto key. The relay is the group's link's
/// token's, the fragment of `<origin>#<token>`, parsed by svrngs-core as
/// the command parses it (call 5). No bearer rides a request: the node
/// knows the person by the key that dialled, or the pass its holder shows.
async fn bind(key: Vec<u8>, issuer: Option<iroh::PublicKey>, relay: iroh::RelayUrl) -> Result<Bound, JsError> {
    t!("LAYER page: wasm module loaded and dial entered (iroh =1.0.2)");

    // ── LAYER 1: the person's key, then the link's token ───────────────────
    let secret = match <[u8; 32]>::try_from(key.as_slice()) {
        Ok(bytes) => iroh::SecretKey::from_bytes(&bytes),
        Err(_) => {
            refuse!(
                "LAYER key: REFUSED — the page holds {} bytes, not a key's 32",
                key.len()
            )
        }
    };
    t!("LAYER key: ok — the key {}", secret.public());
    // The relay the link carries, or a pass's holder is handed, and no
    // address lookup: n0's relays and lookup are none of a phone's path
    // (relay-default), as upstream refused a dial string with no relay
    // (ring-runtime/src/lib.rs:91-99). Dotless, as WebKit reaches it.
    t!("LAYER parse: ok — relay {relay}");
    let builder = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
        .secret_key(secret)
        .relay_mode(iroh::RelayMode::Custom(relay.clone().into()));

    // ── LAYER 2: wasm bindgen / iroh endpoint construction ─────────────────
    t!("LAYER wasm-bindgen/iroh-endpoint: binding a browser endpoint (relay-only)…");
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
    ep.online().await;
    t!("LAYER relay-connect: ok — endpoint reports online (relay handshake done)");
    Ok(Bound { ep, issuer, relay })
}

/// Dial `to`, a computer's key in 64 hex, else the token's issuer, from
/// `bound` on `alpn`: through `via`, the relay its group names for it
/// (relay-in-record), else the relay the token carries, said. A dial to a
/// lender is given up when not answered `within` (events-asleep); the
/// dial to the computer a link names waits as long as iroh does.
async fn reach(
    bound: &Bound,
    to: Option<String>,
    via: Option<String>,
    alpn: Alpn,
    within: Option<std::time::Duration>,
) -> Result<iroh::endpoint::Connection, JsError> {
    let node_id = match to {
        None => match bound.issuer {
            Some(issuer) => issuer,
            None => refuse!("LAYER parse: REFUSED — no computer named to dial"),
        },
        Some(to) => match <[u8; 32]>::try_from(hex::decode(&to).unwrap_or_default().as_slice()) {
            Ok(bytes) => match iroh::PublicKey::from_bytes(&bytes) {
                Ok(p) => p,
                Err(e) => {
                    refuse!("LAYER parse: REFUSED — node id is not a valid public key: {e}")
                }
            },
            Err(_) => refuse!("LAYER parse: REFUSED — {to:?} is not a computer's key"),
        },
    };
    let named = match via.as_deref().map(relay::parse) {
        None => None,
        Some(Ok(url)) => Some(url),
        Some(Err(e)) => refuse!("LAYER parse: REFUSED — {via:?} is not a relay: {e}"),
    };
    match &named {
        Some(relay) => t!("LAYER parse: ok — {node_id}'s group names its relay {relay}"),
        None => t!("LAYER parse: the group names no relay for {node_id}, so it is dialled through the link's"),
    }
    let relay = named.unwrap_or_else(|| bound.relay.clone());
    let addr = iroh::EndpointAddr::from_parts(node_id, [iroh::TransportAddr::Relay(relay)]);

    t!("LAYER tls/alpn+relay-hop: dialing {node_id} on the {alpn:?} protocol through a relay…");
    let connect = bound.ep.connect(addr, alpn.bytes());
    let dialled = match within {
        Some(within) => n0_future::time::timeout(within, connect).await,
        None => Ok(connect.await),
    };
    let conn = match dialled {
        Ok(Ok(c)) => c,
        Err(_) => refuse!("LAYER tls/alpn+relay-hop: REFUSED — not answered in {within:?}: asleep"),
        Ok(Err(e)) => refuse!("LAYER tls/alpn+relay-hop: REFUSED — connect() failed: {e}"),
    };
    t!("LAYER tls/alpn+relay-hop: ok — QUIC connection established to {node_id} on the {alpn:?} protocol");

    t!(
        "LAYER tls/alpn: negotiated ALPN = {:?}",
        String::from_utf8_lossy(conn.alpn())
    );
    // The HTTP rides a bi-stream per request, opened by `Dialled::fetch`.
    Ok(conn)
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
        wasm_bindgen_futures::future_to_promise(async move {
            match http::fetch(conn, method.clone(), path.clone(), headers, body).await {
                Ok(response) => Ok(response.into()),
                Err(e) => {
                    emit(&format!("HTTP: {method} {path} — REFUSED — {e}"));
                    Err(js_sys::Error::new(&e.to_string()).into())
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
        let ep = self.ep.clone();
        let conn = self.conn.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            match ep {
                Some(ep) => ep.close().await,
                None => conn.close(0u32.into(), b"done"),
            }
            Ok(JsValue::UNDEFINED)
        })
    }
}

/// Dial the computer that issued `token`, by its key, as the person whose
/// key is `key`, on the members' protocol: the dialled computer, which
/// answers HTTP on a bi-stream per request ([`Dialled::fetch`]), or the
/// layer that refused, every layer emitted as it happens.
#[wasm_bindgen]
pub async fn connect(key: Vec<u8>, token: String) -> Result<Dialled, JsError> {
    dial(key, token, None, Alpn::Member).await
}

/// Dial the computer that issued `token` on the redeem protocol, the one a
/// key standing in no group may speak, to ask what the link is for and
/// redeem it ([`join`]'s bodies).
#[wasm_bindgen]
pub async fn connect_to_join(key: Vec<u8>, token: String) -> Result<Dialled, JsError> {
    dial(key, token, None, Alpn::Redeem).await
}

/// Dial `to`, else the computer that issued `token`, on `alpn`, from an
/// endpoint of its own: the dial's [`Dialled::close`] closes it.
async fn dial(key: Vec<u8>, token: String, to: Option<String>, alpn: Alpn) -> Result<Dialled, JsError> {
    let bound = bind_for(key, &token).await?;
    let conn = reach(&bound, to, None, alpn, None).await?;
    Ok(Dialled {
        ep: Some(bound.ep),
        conn,
    })
}

/// Bind one endpoint as the person whose key is `key`, in the group whose
/// link's token is `token`, to dial each lender from ([`Endpoint::dial`]):
/// the service worker's (DESIGN §3, call 3). A relay keeps one endpoint
/// per key and sets the older aside when another binds
/// (iroh-relay-1.0.2/src/server/clients.rs:80-92), so the worker holds one
/// for every lender it reaches, never one per dial.
#[wasm_bindgen]
pub async fn endpoint(key: Vec<u8>, token: String) -> Result<Endpoint, JsError> {
    Ok(Endpoint {
        bound: bind_for(key, &token).await?,
    })
}

/// Bind one endpoint as a pass's holder, whose key is `key`, reaching the
/// group through `relay` — the one the group's link carries — refused
/// with none: a lend's origin's worker (u5-2; DESIGN §3, "Lent apps", call 6).
/// It is handed no link's token, which would let it join the group.
#[wasm_bindgen]
pub async fn holder_endpoint(key: Vec<u8>, relay: Option<String>) -> Result<Endpoint, JsError> {
    let relay = match relay.as_deref().map(relay::parse) {
        None => refuse!("LAYER parse: REFUSED — no relay handed, and the page asks n0's lookup for none"),
        Some(Ok(url)) => url,
        Some(Err(e)) => refuse!("LAYER parse: REFUSED — {relay:?} is not a relay: {e}"),
    };
    Ok(Endpoint {
        bound: bind(key, None, relay).await?,
    })
}

/// The relay a link's `token` carries, dotless, or none: what a pass's
/// holder is handed in place of the token (u5-2).
#[wasm_bindgen]
pub fn relay_of(token: String) -> Result<Option<String>, JsError> {
    let link = Token::decode(&token).ok_or_else(|| JsError::new("not a link's token"))?;
    Ok(link.relay.map(|r| iroh::RelayUrl::from(r).to_string()))
}

/// The label of the lend `name` lent by the computer whose key is
/// `lender`, 64 hex: one DNS label, its origin's name (u5-2).
#[wasm_bindgen]
pub fn label(lender: String, name: String) -> Result<String, JsError> {
    let label = Label::of_hex(&lender, &name).ok_or_else(|| JsError::new("not a computer's key"))?;
    Ok(label.to_string())
}

/// The person whose key is `person` lets the key `holder` act as them for
/// the app `app` toward it and the lends `reads` until `until`, seconds
/// since 1970: the pass, as the door reads it from a request (u5-2, sup-1;
/// svrngs-core's `Pass`).
#[wasm_bindgen]
pub fn pass(person: Vec<u8>, holder: Vec<u8>, app: String, reads: Vec<String>, until: f64) -> Result<String, JsError> {
    let key = |bytes: Vec<u8>| {
        <[u8; 32]>::try_from(bytes.as_slice())
            .map(|b| svrngs_core::SigningKey::from_bytes(&b))
            .map_err(|_| JsError::new("a key is 32 bytes"))
    };
    let parse = |l: &String| Label::parse(l).ok_or_else(|| JsError::new(&format!("{l} is not a lend's label")));
    let app = parse(&app)?;
    let reads = reads.iter().map(parse).collect::<Result<Vec<_>, _>>()?;
    let holder = svrngs_core::actor_of(&key(holder)?);
    Ok(Pass::sign(&key(person)?, &holder, (app, reads), until as i64).encode())
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
        let conn = reach(&self.bound, Some(lender), relay, Alpn::Member, Some(dial::DIAL_WITHIN)).await?;
        Ok(Dialled { ep: None, conn })
    }

    /// Dial the computer whose key is `lender` through `relay`, as
    /// [`Self::dial`], on the pass holders' protocol, as a pass's holder:
    /// every request presents the pass (u5-2).
    pub async fn dial_lend(&self, lender: String, relay: Option<String>) -> Result<Dialled, JsError> {
        let conn = reach(&self.bound, Some(lender), relay, Alpn::Lend, Some(dial::DIAL_WITHIN)).await?;
        Ok(Dialled { ep: None, conn })
    }

    /// Close the endpoint, and every dial from it.
    pub async fn close(&self) {
        self.bound.ep.close().await;
    }
}
