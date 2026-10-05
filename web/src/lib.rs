// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/apps/ring-runtime/src/lib.rs@e1a43bcd2
//! The page's dial — the wasm half of the page a group's link,
//! `<origin>#<token>`, opens on a phone (DESIGN §3, "Joining").
//!
//! A browser cannot open a UDP socket, so it cannot speak iroh natively. This
//! module IS an iroh endpoint compiled to wasm (relay-only): the page dials a
//! computer in the group by its key, through a relay, on the members'
//! protocol the node already accepts, as the person whose key stands in the
//! group, and speaks one HTTP/1.1 request over the resulting bi-stream. The
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
/// key is `key`; send `GET <path>` over one iroh bi-stream on the members'
/// protocol, and return the full trace (every layer, plus the response
/// verbatim).
///
/// `key` is the person's key, the 32 bytes the page's store holds (DESIGN
/// §3, call 1): iroh signs its handshake with it synchronously, so it is
/// bytes here, never a WebCrypto key. `token` is the group's link's token,
/// the fragment of `<origin>#<token>`, parsed by svrngs-core as the command
/// parses it (call 5). `path` is the member route asked (`/lends`). No
/// bearer rides the request: the node knows the person by the key that
/// dialled.
#[wasm_bindgen]
pub async fn dial(key: Vec<u8>, token: String, path: String) -> String {
    let mut trace = String::new();
    macro_rules! t {
        ($($a:tt)*) => {{
            let line = format!($($a)*);
            emit(&line);
            trace.push_str(&line);
            trace.push('\n');
        }};
    }

    t!("LAYER page: wasm module loaded and dial entered (iroh =1.0.2)");

    // ── LAYER 1: the person's key, then the link's token ───────────────────
    let secret = match <[u8; 32]>::try_from(key.as_slice()) {
        Ok(bytes) => iroh::SecretKey::from_bytes(&bytes),
        Err(_) => {
            t!("LAYER key: REFUSED — the page holds {} bytes, not a key's 32", key.len());
            return trace;
        }
    };
    t!("LAYER key: ok — the person's key {}", secret.public());
    let Some(link) = Token::decode(&token) else {
        t!("LAYER parse: REFUSED — the fragment is not a link's token: {token}");
        return trace;
    };
    let node_id = match iroh::PublicKey::from_bytes(link.issuer.as_bytes()) {
        Ok(p) => p,
        Err(e) => {
            t!("LAYER parse: REFUSED — node id is not a valid public key: {e}");
            return trace;
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
            (builder.relay_mode(iroh::RelayMode::Default).address_lookup(lookup), addr)
        }
    };

    // ── LAYER 2: wasm bindgen / iroh endpoint construction ─────────────────
    t!("LAYER wasm-bindgen/iroh-endpoint: binding a browser endpoint (relay-only)…");
    // It offers no protocol: a browser is never dialled (DESIGN §3, call 2).
    let ep = match builder.bind().await {
        Ok(e) => e,
        Err(e) => {
            t!("LAYER wasm-bindgen/iroh-endpoint: REFUSED — endpoint bind failed: {e}");
            return trace;
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
            t!("LAYER tls/alpn+relay-hop: REFUSED — connect() failed: {e}");
            return trace;
        }
    };
    t!("LAYER tls/alpn+relay-hop: ok — QUIC connection established to {node_id} on the member protocol");

    // ── LAYER 4: the HTTP request over one bi-stream ───────────────────────
    t!(
        "LAYER tls/alpn: negotiated ALPN = {:?}",
        String::from_utf8_lossy(conn.alpn())
    );
    let (mut send, mut recv) = match conn.open_bi().await {
        Ok(s) => s,
        Err(e) => {
            t!("LAYER open-bi: REFUSED — {e}");
            return trace;
        }
    };
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: svrngs\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    );
    if let Err(e) = send.write_all(req.as_bytes()).await {
        t!("LAYER request-write: REFUSED — {e}");
        return trace;
    }
    t!(
        "LAYER request-write: ok — {} bytes of HTTP/1.1 sent on the bi-stream (send half kept open until the response, as the native bridge does)",
        req.len()
    );

    let body = match recv.read_to_end(512 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            t!("LAYER response-read: REFUSED — {e}");
            return trace;
        }
    };
    // Close our half now the exchange is done — the native bridge finishes the
    // send stream only after the local TCP side is finished, never before.
    let _ = send.finish();
    let text = String::from_utf8_lossy(&body).to_string();
    let status = text
        .lines()
        .next()
        .unwrap_or("<no status line>")
        .to_string();
    t!(
        "LAYER member-admission: response received, {} bytes",
        body.len()
    );
    t!("RESPONSE-STATUS: {status}");
    t!("RESPONSE-BODY: {text}");

    ep.close().await;
    trace
}
