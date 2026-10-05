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

use svrngs_core::link::Token;
use wasm_bindgen::prelude::*;

/// The members' protocol, spelled where the node spells it (svrngs-core's
/// `Alpn`, since u4-2): a person's page dials as a member, by the key that
/// stands in the group, where upstream's dialled as a guest with a bearer.
const MEMBER_ALPN: &[u8] = svrngs_core::alpn::Alpn::Member.bytes();

/// Push one layer line to the page's `__trace` collector, if the page defined
/// one. Best-effort: a missing collector must not break the dial.
fn emit(line: &str) {
    if let Ok(f) = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("__trace")) {
        if let Ok(f) = f.dyn_into::<js_sys::Function>() {
            let _ = f.call1(&JsValue::UNDEFINED, &JsValue::from_str(line));
        }
    }
}

/// Dial the computer that issued `token`, by its key, as the person whose
/// key is `key`, on the members' protocol: the dialled computer, which
/// answers HTTP on a bi-stream per request ([`Dialled::fetch`]), or the
/// layer that refused, every layer emitted as it happens.
///
/// `key` is the person's key, the 32 bytes the page's store holds (DESIGN
/// §3, call 1): iroh signs its handshake with it synchronously, so it is
/// bytes here, never a WebCrypto key. `token` is the group's link's token,
/// the fragment of `<origin>#<token>`, parsed by svrngs-core as the command
/// parses it (call 5). No bearer rides a request: the node knows the person
/// by the key that dialled.
#[wasm_bindgen]
pub async fn connect(key: Vec<u8>, token: String) -> Result<Dialled, JsError> {
    macro_rules! t {
        ($($a:tt)*) => {
            emit(&format!($($a)*))
        };
    }
    // A refusal's line is emitted, and is the error.
    macro_rules! refuse {
        ($($a:tt)*) => {{
            let line = format!($($a)*);
            emit(&line);
            return Err(JsError::new(&line));
        }};
    }

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
    t!("LAYER key: ok — the person's key {}", secret.public());
    let Some(link) = Token::decode(&token) else {
        refuse!("LAYER parse: REFUSED — the fragment is not a link's token: {token}")
    };
    let node_id = match iroh::PublicKey::from_bytes(link.issuer.as_bytes()) {
        Ok(p) => p,
        Err(e) => {
            refuse!("LAYER parse: REFUSED — node id is not a valid public key: {e}")
        }
    };
    // The relay the token carries, from an issuer run without address
    // lookup; else iroh's lookup, the resolver the N0 preset adds in a
    // browser (iroh-1.0.2/src/endpoint/presets.rs:122-128), with n0's
    // relays. Upstream refused a dial string with no relay
    // (ring-runtime/src/lib.rs:91-99); a bare key is a link's usual form.
    let builder = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal).secret_key(secret);
    let (builder, addr) = match link.relay {
        Some(relay) => {
            let relay = iroh::RelayUrl::from(relay);
            t!("LAYER parse: ok — node id {node_id}, relay {relay} from the token");
            let addr = iroh::EndpointAddr::from_parts(node_id, [iroh::TransportAddr::Relay(relay.clone())]);
            (builder.relay_mode(iroh::RelayMode::Custom(relay.into())), addr)
        }
        None => {
            t!("LAYER parse: ok — node id {node_id}, a bare key: found by n0's address lookup");
            let lookup = iroh::address_lookup::PkarrResolver::n0_dns();
            let addr = iroh::EndpointAddr::from_parts(node_id, []);
            (
                builder.relay_mode(iroh::RelayMode::Default).address_lookup(lookup),
                addr,
            )
        }
    };

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

    t!("LAYER tls/alpn+relay-hop: dialing {node_id} on the member protocol through a relay…");
    let conn = match ep.connect(addr, MEMBER_ALPN).await {
        Ok(c) => c,
        Err(e) => {
            refuse!("LAYER tls/alpn+relay-hop: REFUSED — connect() failed: {e}")
        }
    };
    t!("LAYER tls/alpn+relay-hop: ok — QUIC connection established to {node_id} on the member protocol");

    t!(
        "LAYER tls/alpn: negotiated ALPN = {:?}",
        String::from_utf8_lossy(conn.alpn())
    );
    // The HTTP rides a bi-stream per request, opened by `Dialled::fetch`.
    Ok(Dialled { ep, conn })
}

/// A computer the page has dialled: each request to it is one bi-stream of
/// the connection, spoken by hyper's HTTP/1 client ([`http`]).
#[wasm_bindgen]
pub struct Dialled {
    ep: iroh::Endpoint,
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

    /// Close the endpoint, and every stream with it.
    pub fn close(&self) -> js_sys::Promise {
        let ep = self.ep.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            ep.close().await;
            Ok(JsValue::UNDEFINED)
        })
    }
}
