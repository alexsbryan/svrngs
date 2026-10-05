// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/iroh.rs@e1a43bcd2
//! EXPERIMENTAL dial-by-key transport over iroh (QUIC by Ed25519
//! key, hole-punching, optional relays). Upstream feature-gated it behind
//! `iroh`; in svrngs it is the only transport, so nothing gates it.
//!
//! ## Shape: a localhost byte-tunnel to dial, hyper on the stream to serve
//!
//! [`IrohTransport::base_url`] resolves a base URL, and every call
//! site keeps its existing `reqwest` client — so the dialling half
//! terminates iroh QUIC locally and hands HTTP a plain TCP socket:
//!
//! - Client side ([`IrohTransport`]): per-peer localhost
//!   `TcpListener`; each accepted TCP connection opens one iroh
//!   bi-stream to the peer (dialed by its Ed25519 key — the
//!   [`Contact`]'s `key`) and copies bytes both ways.
//! - Server side ([`IrohAcceptor`]): admits or closes each iroh
//!   connection on its ALPN and the dialer's verified key, and serves
//!   every bi-stream of an admitted one with hyper's HTTP/1 server and
//!   the axum router its admit decision names — no loopback listener, and the dialer
//!   reaches handlers as a [`Dialer`] request extension, never a header.
//!
//! Known upgrade path (deliberately NOT in the spike): serve hyper
//! directly on iroh streams (drop the double-copy), carry the
//! traffic class in the ALPN or a stream header so one acceptor can
//! route to both daemon ports, and a tunnel-proxy sidecar for the
//! raw-TCP `rpc-server` tensor traffic that this transport
//! intentionally does not cover. (Upstream's words. The server side
//! has taken the first; svrngs has one ALPN and no tensor traffic.)
//!
//! Upstream's `PeerTransport` port, `PeerContact`, `PeerEndpoint` and
//! `TrafficClass` did not come over: there is one transport, so it is a
//! concrete type (RUST_ARCH: a trait only at a real boundary), a contact is a
//! key with optional dial hints, and there is one ALPN ([`ALPN`]).
//!
//! ## Spike limitations (documented, intentional)
//!
//! - Peer iroh socket addresses come from the [`Contact`] or an explicitly
//!   seeded map ([`IrohTransport::add_known_peer`]); a bare key with neither
//!   is not dialled. (svrngs: unless the endpoint has an address lookup,
//!   which resolves it — DESIGN §8, "Four facts".)

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

use std::sync::Arc;
use std::time::Duration;

pub use crate::alpn::{Alpn, ALPN};
// The guest dialer and the one HTTP bridge live in `crate::guest`, as they
// lived in `mesh_reach::guest` upstream (pb-reach-guest, phase-b-35);
// re-exported so callers spell them `svrngs_node::iroh::`.
pub use crate::guest::{
    build_relayed_endpoint, configured_proxy_redacted, parse_dial_string, ring_crypto_provider,
    HttpBridge, RelayConfig,
};

// Re-exported so callers build endpoints without declaring their own iroh
// dependency — keeps the version pin in exactly one place.
pub use iroh::endpoint::presets;
pub use iroh::endpoint::Builder as EndpointBuilder;
pub use iroh::{Endpoint, EndpointAddr, PublicKey, RelayUrl, SecretKey};
// svrngs: iroh's address book in memory, so a test tells a joiner the
// loopback addresses a link's bare key resolves to (u2-8).
pub use iroh::address_lookup::MemoryLookup;

/// Render an endpoint's current dial info as a pairing string
/// ([`parse_dial_string`]'s inverse). Relay URLs first (stable),
/// then direct addresses. `None` while the endpoint has no
/// reachable address yet.
pub fn format_dial_string(addr: &EndpointAddr) -> Option<String> {
    let mut targets: Vec<String> = addr.relay_urls().map(|r| r.to_string()).collect();
    targets.extend(addr.ip_addrs().map(|a| a.to_string()));
    if targets.is_empty() {
        return None;
    }
    Some(format!(
        "{}@{}",
        hex::encode(addr.id.as_bytes()),
        targets.join(",")
    ))
}

/// A peer to dial: its key, and whatever dial hints are known. Upstream's
/// `PeerContact` was the mesh's gossiped record — a `NodeId`, IP addresses
/// and an optional key; here the key is the peer and the hints are optional.
#[derive(Debug, Clone)]
pub struct Contact {
    /// The peer's Ed25519 key — what the QUIC handshake verifies.
    pub key: PublicKey,
    /// The peer's home relay, if known.
    pub relay_url: Option<String>,
    /// The peer's iroh UDP socket addresses, if known.
    pub iroh_direct_addrs: Vec<SocketAddr>,
}

/// Client half: resolves a peer's key to a localhost base
/// URL bridged over iroh.
#[derive(Debug)]
pub struct IrohTransport {
    endpoint: iroh::Endpoint,
    /// Out-of-band dial hints: pubkey → iroh UDP socket addresses.
    /// **Fallback only.** Dial info comes first from the [`Contact`] —
    /// relay URL + direct addrs.
    /// Retained so hermetic tests can seed addresses directly; when a
    /// contact carries its own addrs, both are merged.
    #[expect(clippy::disallowed_types, reason = "the transport's address cache")]
    known_addrs: std::sync::Mutex<HashMap<[u8; 32], Vec<SocketAddr>>>,
    /// One localhost bridge per peer, on the one [`ALPN`]. Each entry
    /// remembers the dial info it was built with (`dial_key`) so a
    /// dial-info change RETARGETS the bridge — a frozen target
    /// kept dialing a restarted peer's dead ephemeral port forever
    /// (the 2026-07-19 dual-restart heal deadlock's transport half).
    #[expect(clippy::disallowed_types, reason = "the transport's bridge cache")]
    bridges: tokio::sync::Mutex<HashMap<[u8; 32], CachedBridge>>,
}

/// A cached bridge plus the normalized contact dial-info it targets.
#[derive(Debug)]
struct CachedBridge {
    bridge: Arc<HttpBridge>,
    dial_key: DialKey,
}

/// Normalized (relay_url, sorted direct addrs) — the parts of a
/// [`Contact`] that determine where a bridge's iroh dials go.
type DialKey = (Option<String>, Vec<SocketAddr>);

fn dial_key_for(peer: &Contact) -> DialKey {
    let mut addrs = peer.iroh_direct_addrs.clone();
    addrs.sort();
    addrs.dedup();
    (peer.relay_url.clone(), addrs)
}

impl IrohTransport {
    #[expect(clippy::disallowed_types, reason = "the transport's caches")]
    pub fn new(endpoint: iroh::Endpoint) -> Self {
        Self {
            endpoint,
            known_addrs: std::sync::Mutex::new(HashMap::new()),
            bridges: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Seed dial hints for a peer. Test/fallback surface — a caller with a
    /// [`Contact`] passes its hints there instead.
    pub fn add_known_peer(&self, pubkey: PublicKey, addrs: Vec<SocketAddr>) {
        if let Ok(mut map) = self.known_addrs.lock() {
            map.insert(*pubkey.as_bytes(), addrs);
        }
    }

    /// Build the dial target from the contact's iroh info (relay
    /// URL + direct addrs), merging any test-seeded `known_addrs`.
    /// `None` when there's no usable path (no relay AND no address) and the
    /// endpoint has no address lookup to resolve the key. (svrngs: upstream
    /// refused every bare key, since its mesh gossiped addresses; with no
    /// gossip a member is dialled by key, and the endpoint's own lookup —
    /// n0's DNS under `RelayConfig::n0_services` — finds it, DESIGN §8,
    /// "Four facts".)
    fn endpoint_addr_for(&self, peer: &Contact) -> Option<iroh::EndpointAddr> {
        let mut ea = iroh::EndpointAddr::new(peer.key);
        let mut has_path = false;
        if let Some(relay) = peer.relay_url.as_deref() {
            match crate::relay::parse(relay) {
                Ok(url) => {
                    ea = ea.with_relay_url(url);
                    has_path = true;
                }
                Err(e) => tracing::warn!(
                    target: "transport",
                    transport = "iroh",
                    relay = %relay,
                    error = %e,
                    "iroh: peer relay_url did not parse — ignoring"
                ),
            }
        }
        let seeded = self
            .known_addrs
            .lock()
            .ok()
            .and_then(|m| m.get(peer.key.as_bytes()).cloned())
            .unwrap_or_default();
        let mut seen = std::collections::HashSet::new();
        for a in peer.iroh_direct_addrs.iter().copied().chain(seeded) {
            if seen.insert(a) {
                ea = ea.with_ip_addr(a);
                has_path = true;
            }
        }
        if has_path {
            return Some(ea);
        }
        let lookup = self
            .endpoint
            .address_lookup()
            .is_ok_and(|services| !services.is_empty());
        tracing::debug!(
            target: "transport",
            transport = "iroh",
            peer = %&peer.key.to_string()[..8],
            lookup,
            "iroh: a bare key — dialled through the endpoint's address lookup when it has one"
        );
        lookup.then_some(ea)
    }

    /// Get or create the localhost TCP bridge for the peer,
    /// dialing the target resolved from `peer`.
    async fn bridge_for(&self, peer: &Contact) -> Option<SocketAddr> {
        let key = *peer.key.as_bytes();
        let dial_key = dial_key_for(peer);
        let mut bridges = self.bridges.lock().await;
        if let Some(cached) = bridges.get(&key) {
            if cached.dial_key == dial_key {
                return Some(cached.bridge.local_addr());
            }
        }
        let Some(target) = self.endpoint_addr_for(peer) else {
            // The fresh contact has no dialable path at all (no relay, no
            // direct addrs). Nothing to retarget TO — drop any stale bridge
            // rather than keep tunneling at an address the peer has left.
            bridges.remove(&key);
            return None;
        };
        if let Some(cached) = bridges.get_mut(&key) {
            // The peer's dial info changed (typical: it
            // restarted and its ephemeral iroh port moved). A frozen
            // bridge would keep dialing the dead target forever — point
            // this one at the fresh contact instead. Retargeting rather
            // than rebuilding keeps the loopback port stable, which is
            // what plain-TCP clients holding that address depend on (see
            // `HttpBridge::retarget`); in-flight tunnels to the stale
            // target were doomed either way.
            cached.bridge.retarget(target);
            cached.dial_key = dial_key;
            let local_addr = cached.bridge.local_addr();
            tracing::info!(
                target: "transport",
                peer = %hex::encode(&key[..4]),
                bridge = %local_addr,
                "iroh bridge: peer dial info changed — retargeted in place (port held)"
            );
            return Some(local_addr);
        }
        let bridge = HttpBridge::spawn_preferring(
            self.endpoint.clone(),
            target,
            ALPN,
            Some(preferred_bridge_port(&peer.key, ALPN)),
        )
        .await
        .ok()?;
        let local_addr = bridge.local_addr();
        bridges.insert(
            key,
            CachedBridge {
                bridge: Arc::new(bridge),
                dial_key,
            },
        );
        Some(local_addr)
    }

    /// The peer's loopback base URL, bridged over iroh — upstream's
    /// `PeerTransport::endpoints`, which answered at most one candidate on
    /// this transport. `None` when the contact has no dialable path.
    pub async fn base_url(&self, peer: &Contact) -> Option<String> {
        let label = &peer.key.to_string()[..8];
        let Some(local) = self.bridge_for(peer).await else {
            tracing::debug!(
                target: "transport",
                transport = "iroh",
                peer = %label,
                "iroh: no dialable path in contact (no relay_url / iroh_direct_addrs) \
                 and no address lookup — not dialable"
            );
            return None;
        };
        tracing::debug!(
            target: "transport",
            transport = "iroh",
            peer = %label,
            first = %format!("iroh:{local}→{label}"),
            "transport: resolved"
        );
        Some(format!("http://{local}"))
    }
}

/// The loopback port a peer's bridge prefers, from its key and the ALPN —
/// identity from essence (ARCH §7.5), never a counter or whatever the kernel
/// handed out last time. 20000..=32767 sits below Linux's default ephemeral
/// range (32768–60999), so a derived port is never one an unrelated socket
/// was just given; a collision between two peers is a 1-in-12768 event per
/// pair and is handled by the named fallback in
/// [`HttpBridge::spawn_preferring`].
pub fn preferred_bridge_port(pubkey: &PublicKey, alpn: &[u8]) -> u16 {
    // FNV-1a: spread, not secrecy. The inputs are public.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in pubkey.as_bytes().iter().chain(alpn.iter()) {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    20_000 + (h % 12_768) as u16
}

/// The acceptor's answer to a connection, decided once per connection on its
/// negotiated ALPN and the key its handshake verified.
#[derive(Debug, Clone)]
pub enum Admit {
    /// Serve every bi-stream on the connection with this router: the
    /// routes its protocol reaches, and no others.
    Serve(axum::Router),
    /// Close the connection before any stream is read: no HTTP answer.
    Refuse,
}

/// The key iroh's handshake verified for the connection a request came on.
/// [`IrohAcceptor`] inserts it as an extension into every request it serves,
/// and nothing outside this crate can build one, so a handler that extracts
/// `Extension<Dialer>` reads the connection — never a header a client typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dialer(PublicKey);

impl Dialer {
    /// The verified key.
    pub fn key(&self) -> PublicKey {
        self.0
    }

    /// The record's actor: the key's lowercase hex, as
    /// [`actor_of`](svrngs_core::actor_of) spells a signer.
    pub fn actor(&self) -> String {
        key_actor(&self.0)
    }
}

/// The connection a request came on, beside its [`Dialer`]: what the door
/// reads of the path it travels (svrngs, u3-6). Inserted by
/// [`IrohAcceptor`] as an extension, as the dialer is.
#[derive(Debug, Clone)]
pub struct Via(iroh::endpoint::Connection);

/// What a connection's selected path looks like now, as iroh's
/// `Connection::paths()` and `PathStats` report it: use, never capacity
/// (DESIGN §8, U3's inventory, its iroh_path row).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Observed {
    /// The address the path ends at, `None` when it rides a relay.
    pub addr: Option<IpAddr>,
    pub relayed: bool,
    pub rtt: Duration,
    /// Bytes this computer has sent on the path, and received.
    pub sent: u64,
    pub received: u64,
}

impl Via {
    /// The selected path — the one iroh sends on — or `None` when none is.
    /// Upstream classified a peer's addresses instead, and found a direct
    /// leg and a relay both active read as relayed while bytes went direct
    /// (`commonwealth-transport/src/iroh_path.rs:56-68`, `Mixed`); the
    /// selected path is the one carrying them.
    pub fn observe(&self) -> Option<Observed> {
        let paths = self.0.paths();
        let path = paths.iter().find(|p| p.is_selected())?;
        let stats = path.stats();
        let addr = match path.remote_addr() {
            iroh::TransportAddr::Ip(a) => Some(a.ip()),
            _ => None,
        };
        Some(Observed {
            addr,
            relayed: path.is_relay(),
            rtt: stats.rtt,
            sent: stats.udp_tx.bytes,
            received: stats.udp_rx.bytes,
        })
    }
}

/// The record's actor for an iroh key: its lowercase hex, as
/// [`actor_of`](svrngs_core::actor_of) spells a signer.
pub fn key_actor(key: &PublicKey) -> String {
    hex::encode(key.as_bytes())
}

/// The iroh key an actor's lowercase hex spells, or `None` — the inverse of
/// [`key_actor`], for a key the register names that the node dials.
pub fn actor_key(actor: &str) -> Option<PublicKey> {
    let bytes: [u8; 32] = hex::decode(actor).ok()?.try_into().ok()?;
    PublicKey::from_bytes(&bytes).ok()
}

/// Server half: accept iroh connections, and serve each bi-stream of an
/// admitted one with hyper's HTTP/1 server and the caller's router.
pub struct IrohAcceptor {
    task: tokio::task::JoinHandle<()>,
}

impl Drop for IrohAcceptor {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl IrohAcceptor {
    /// Spawn the accept loop. `admit` sees each connection's negotiated ALPN
    /// and the dialer's key, verified by the QUIC handshake, once per
    /// connection; [`Admit::Refuse`] closes the connection. Every bi-stream
    /// of an admitted connection is one HTTP/1 connection served by the
    /// router [`Admit::Serve`] names, each request carrying the dialer as a
    /// [`Dialer`] extension.
    ///
    /// The dial string that reaches an endpoint is public, so holding it is
    /// not a credential; the verified key is the evidence `admit` decides
    /// on. It is called again for every connection, so a decision that
    /// consults live state sees membership change between dials.
    pub fn spawn<F>(endpoint: iroh::Endpoint, admit: F) -> Self
    where
        F: Fn(&[u8], &PublicKey) -> Admit + Send + Sync + 'static,
    {
        let admit = Arc::new(admit);
        let task = tokio::spawn(async move {
            while let Some(incoming) = endpoint.accept().await {
                let admit = admit.clone();
                tokio::spawn(async move {
                    let conn = match incoming.await {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::debug!(
                                target: "transport",
                                error = %e,
                                "iroh acceptor: handshake failed"
                            );
                            return;
                        }
                    };
                    // A fully-accepted connection has a negotiated ALPN and
                    // a verified dialer key (from the peer's TLS
                    // certificate). Admit this connection's streams by both.
                    let alpn = conn.alpn().to_vec();
                    let dialer = Dialer(conn.remote_id());
                    let router = match admit(&alpn, &dialer.0) {
                        Admit::Refuse => {
                            tracing::warn!(
                                target: "transport",
                                alpn = %String::from_utf8_lossy(&alpn),
                                dialer = %dialer.actor(),
                                "iroh acceptor: dialer not admitted — closing connection"
                            );
                            conn.close(0u32.into(), b"not admitted");
                            return;
                        }
                        Admit::Serve(router) => {
                            tracing::debug!(
                                target: "transport",
                                alpn = %String::from_utf8_lossy(&alpn),
                                dialer = %dialer.actor(),
                                "iroh acceptor: dialer admitted — serving its streams"
                            );
                            router
                        }
                    };
                    // Ends when the connection closes.
                    while let Ok((send, recv)) = conn.accept_bi().await {
                        let via = Via(conn.clone());
                        tokio::spawn(serve_stream(send, recv, dialer, via, router.clone()));
                    }
                });
            }
        });
        Self { task }
    }
}

/// One bi-stream is one HTTP/1 connection: hyper parses every request on it,
/// the dialer is inserted as the request's [`Dialer`], its connection as
/// its [`Via`], and `router` answers.
async fn serve_stream(
    send: iroh::endpoint::SendStream,
    recv: iroh::endpoint::RecvStream,
    dialer: Dialer,
    via: Via,
    router: axum::Router,
) {
    use tower::ServiceExt;
    let io = hyper_util::rt::TokioIo::new(tokio::io::join(recv, send));
    let service =
        hyper::service::service_fn(move |mut req: hyper::Request<hyper::body::Incoming>| {
            req.extensions_mut().insert(dialer);
            req.extensions_mut().insert(via.clone());
            router.clone().oneshot(req)
        });
    // svrngs (u5-2): with upgrades, so the door carries a WebSocket.
    if let Err(e) = hyper::server::conn::http1::Builder::new()
        .serve_connection(io, service)
        .with_upgrades()
        .await
    {
        tracing::debug!(
            target: "transport",
            dialer = %dialer.actor(),
            error = %e,
            "iroh acceptor: a stream's HTTP connection ended in error"
        );
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// What keeps a router served on the stream: the serving endpoint, its
    /// acceptor, and the dialer's transport, whose bridge the URL names.
    pub(crate) struct OnStream {
        _acceptor: IrohAcceptor,
        _transport: IrohTransport,
    }

    /// `router` served by an acceptor that admits every dialer, on a
    /// hermetic endpoint; and the base URL at which a dialer holding
    /// `dialer` reaches it through the bridge. For a route that decides on
    /// the [`Dialer`] itself, so the admit decision is not what a test of it
    /// watches.
    pub(crate) async fn serve_on_stream(
        router: axum::Router,
        dialer: SecretKey,
    ) -> (String, OnStream) {
        let served = hermetic_endpoint(0xEE, vec![ALPN.to_vec()]).await;
        let addrs = loopback_sockets(&served);
        let acceptor =
            IrohAcceptor::spawn(served.clone(), move |_, _| Admit::Serve(router.clone()));
        let dialling = EndpointBuilder::empty()
            .crypto_provider(ring_crypto_provider())
            .secret_key(dialer)
            .bind()
            .await
            .expect("hermetic endpoint bind");
        let transport = IrohTransport::new(dialling);
        let base = transport
            .base_url(&Contact {
                key: served.id(),
                relay_url: None,
                iroh_direct_addrs: addrs,
            })
            .await
            .expect("a contact with an address resolves");
        (
            base,
            OnStream {
                _acceptor: acceptor,
                _transport: transport,
            },
        )
    }

    /// Bind an empty (no-relay, deterministic) endpoint serving `alpns`.
    pub(crate) async fn hermetic_endpoint(seed: u8, alpns: Vec<Vec<u8>>) -> Endpoint {
        EndpointBuilder::empty()
            .crypto_provider(ring_crypto_provider())
            .secret_key(SecretKey::from_bytes(&[seed; 32]))
            .alpns(alpns)
            .bind()
            .await
            .expect("hermetic endpoint bind")
    }

    /// iroh binds the wildcard; rewrite to loopback so the address is
    /// dialable in-process (mirrors the spike e2e's `dialable_sockets`).
    pub(crate) fn loopback_sockets(endpoint: &Endpoint) -> Vec<SocketAddr> {
        endpoint
            .bound_sockets()
            .into_iter()
            .map(|mut a| {
                if a.ip().is_unspecified() {
                    let ip = if a.is_ipv4() { "127.0.0.1" } else { "::1" };
                    a.set_ip(ip.parse().unwrap());
                }
                a
            })
            .collect()
    }

    /// The same peer and protocol always prefer the same port, a different
    /// protocol on the same peer prefers a different one, and every derived
    /// port sits below the kernel's ephemeral range. The failing inputs are
    /// a port outside 20000..=32767 or two calls disagreeing.
    #[test]
    fn a_peers_bridge_port_is_derived_from_its_key_and_alpn() {
        let a = SecretKey::from_bytes(&[7u8; 32]).public();
        let b = SecretKey::from_bytes(&[8u8; 32]).public();
        // Upstream's CLIENT_ALPN bytes: a second protocol, which svrngs
        // does not carry, still derives a different port.
        let other: &[u8] = b"cwth/client/0";
        let p = preferred_bridge_port(&a, ALPN);
        assert_eq!(p, preferred_bridge_port(&a, ALPN));
        assert_ne!(p, preferred_bridge_port(&a, other));
        assert_ne!(p, preferred_bridge_port(&b, ALPN));
        for port in [p, preferred_bridge_port(&b, ALPN)] {
            assert!((20_000..=32_767).contains(&port), "{port}");
        }
    }

    /// When the derived port is already bound, the bridge still comes up —
    /// on another port, named at warn — rather than failing the dial.
    #[tokio::test]
    async fn a_taken_derived_port_falls_back_to_an_ephemeral_one_and_still_serves() {
        let server_ep = hermetic_endpoint(61, vec![ALPN.to_vec()]).await;
        let client_ep = hermetic_endpoint(62, vec![]).await;
        let taken = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = taken.local_addr().unwrap().port();
        let mut target = EndpointAddr::new(server_ep.id());
        for sock in loopback_sockets(&server_ep) {
            target = target.with_ip_addr(sock);
        }
        let bridge = HttpBridge::spawn_preferring(client_ep, target, ALPN, Some(port))
            .await
            .expect("a taken preferred port is not a failed bridge");
        assert_ne!(bridge.local_addr().port(), port);
        assert!(bridge.local_addr().ip().is_loopback());
    }

    /// A trivial TCP "service": every accepted connection is answered
    /// with a fixed marker, then closed. Stands in for one of the
    /// daemon's two local HTTP listeners — the marker is the witness
    /// that a stream reached THIS listener and not the other. It also
    /// drains the client's bytes so the close is a clean FIN, not a
    /// RST that could truncate the marker in flight on loopback.
    async fn spawn_marker_listener(marker: &'static [u8]) -> SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let _ = sock.write_all(marker).await;
                    let _ = sock.shutdown().await;
                    let mut drain = Vec::new();
                    let _ = sock.read_to_end(&mut drain).await;
                });
            }
        });
        addr
    }

    /// Splice every stream `endpoint` accepts into `forward_to`, through the
    /// one [`pump`](crate::guest::pump) — what upstream's
    /// `IrohAcceptor::spawn_routed` did for these tests with a one-entry
    /// table. These are the bridge's tests, which carry bytes, so a byte
    /// splice shaped as `guest/tests.rs`'s `spawn_lender` stands in, whatever
    /// the acceptor serves.
    fn spawn_splicing_acceptor(endpoint: Endpoint, forward_to: SocketAddr) {
        use crate::guest::{pump, PumpSide};
        tokio::spawn(async move {
            while let Some(incoming) = endpoint.accept().await {
                tokio::spawn(async move {
                    let Ok(conn) = incoming.await else { return };
                    while let Ok((send, recv)) = conn.accept_bi().await {
                        let tcp = tokio::net::TcpStream::connect(forward_to).await.unwrap();
                        tokio::spawn(pump(
                            tcp,
                            send,
                            recv,
                            PumpSide::Acceptor,
                            "peer".to_string(),
                            ALPN,
                        ));
                    }
                });
            }
        });
    }

    #[test]
    fn relay_config_from_parts_maps_discovery() {
        // Default / "n0" / absent / empty → n0 services on.
        assert!(RelayConfig::default().n0_services);
        assert!(
            RelayConfig::from_parts(vec![], None)
                .expect("absent is n0")
                .n0_services
        );
        assert!(
            RelayConfig::from_parts(vec![], Some("n0"))
                .expect("n0")
                .n0_services
        );
        assert!(
            RelayConfig::from_parts(vec![], Some(""))
                .expect("empty is n0")
                .n0_services
        );
        // The one sovereignty spelling → n0 severed.
        let c = RelayConfig::from_parts(vec![], Some("none")).expect("none");
        assert!(!c.n0_services);
        // relay_urls passes through.
        let c = RelayConfig::from_parts(vec!["https://r.example:443".into()], Some("none"))
            .expect("none");
        assert_eq!(c.relay_urls, vec!["https://r.example:443".to_string()]);
        assert!(!c.n0_services);
    }

    /// **C2: a typo'd value refuses to load** (ROOT_CAUSE_FIXES C2). The old
    /// shape warned and kept n0 — the substitution this door exists to stop.
    /// Watched failing first, with the warn-and-keep planted back.
    #[test]
    fn an_unknown_discovery_value_refuses_to_load() {
        let err = RelayConfig::from_parts(vec![], Some("carrier-pigeon")).unwrap_err();
        assert!(err.contains("carrier-pigeon"), "the value is named: {err}");
        assert!(
            err.contains("accepted"),
            "and the accepted set is named: {err}"
        );
    }

    /// **C2: `self`/`local` are refused naming `none`** — they were never
    /// implemented, and aliasing them to `none` would keep the lie in the
    /// vocabulary. Watched failing first with the same plant.
    #[test]
    fn self_and_local_refuse_naming_none() {
        for d in ["self", "local"] {
            let err = RelayConfig::from_parts(vec![], Some(d)).unwrap_err();
            assert!(err.contains("never implemented"), "{d}: {err}");
            assert!(
                err.contains("`none`"),
                "{d}: the honest spelling must be named: {err}"
            );
        }
    }

    #[tokio::test]
    async fn build_sovereign_endpoint_binds_without_n0() {
        // Sovereign mode (n0 severed, no custom relay = direct-addr
        // only): must still bind a real endpoint. This is the
        // air-gapped-LAN posture — Minimal preset, relays Disabled, no
        // n0 DNS. Nothing here should touch the network at all.
        let ep = build_relayed_endpoint(
            SecretKey::from_bytes(&[78u8; 32]),
            vec![ALPN.to_vec()],
            &RelayConfig {
                relay_urls: vec![],
                n0_services: false,
                loopback_only: false,
            },
        )
        .await
        .expect("sovereign (no-n0) endpoint must bind");
        assert!(
            !ep.bound_sockets().is_empty(),
            "endpoint must bind a socket"
        );
    }

    /// Read the marker a bridge's loopback port forwards to. Same handshake as
    /// upstream's `read_marker_over` (send first so the peer's `accept_bi`
    /// fires), but against an ALREADY-BOUND bridge port rather than a fresh
    /// bridge — the point being that the port outlives a retarget.
    async fn read_marker_from_port(port: SocketAddr) -> Vec<u8> {
        let mut tcp = tokio::net::TcpStream::connect(port)
            .await
            .expect("connect to bridge port");
        let _ = tcp.write_all(b"ping").await;
        let _ = tcp.shutdown().await;
        let mut buf = Vec::new();
        let read = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tcp.read_to_end(&mut buf),
        )
        .await;
        read.expect("read did not time out").expect("read ok");
        buf
    }

    fn contact_for(server: &Endpoint, addrs: Vec<SocketAddr>) -> Contact {
        Contact {
            key: server.id(),
            relay_url: None,
            iroh_direct_addrs: addrs,
        }
    }

    /// A peer's dial info changing must RETARGET the bridge, not
    /// rebuild it: the loopback port is what plain-TCP clients hold (ggml's
    /// rpc-server list, via the mesh's discovered worker endpoint string), and
    /// minting a new one made an unmoved peer read downstream as a stream of
    /// different workers (2026-07-25: 34207 → 40043 → 39419 → 34133 → 40021).
    /// The tunnel must still land on the NEW target — that's what the rebuild
    /// was originally protecting (the 2026-07-19 dual-restart heal deadlock), so
    /// this asserts both halves: same port, fresh destination.
    #[tokio::test]
    async fn bridge_retargets_in_place_when_peer_dial_info_changes() {
        let worker_addr = spawn_marker_listener(b"WORKER").await;
        let server_ep = hermetic_endpoint(31, vec![ALPN.to_vec()]).await;
        spawn_splicing_acceptor(server_ep.clone(), worker_addr);

        let transport = IrohTransport::new(hermetic_endpoint(32, vec![]).await);

        // Tick 1: the peer's address is stale (nothing listens there).
        // The bridge binds a port; nothing is dialed until a client connects.
        let stale: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let first = transport
            .base_url(&contact_for(&server_ep, vec![stale]))
            .await
            .expect("a contact with an address resolves");
        let port_before = first.clone();

        // Tick 2: the contact carries the peer's real address. Different dial key.
        let live = loopback_sockets(&server_ep);
        let second = transport
            .base_url(&contact_for(&server_ep, live))
            .await
            .expect("a contact with an address resolves");
        assert_eq!(
            second, port_before,
            "a dial-info change must hold the loopback port, not mint a new one"
        );

        // …and the tunnel now reaches the peer at its NEW address, proving the
        // held port is not a frozen bridge still dialing the dead one.
        let bridge_port: SocketAddr = second.strip_prefix("http://").unwrap().parse().unwrap();
        assert_eq!(
            read_marker_from_port(bridge_port).await,
            b"WORKER",
            "the retargeted bridge must reach the peer's current address"
        );
    }

    /// An unchanged contact re-resolves to the SAME bridge with no rebind. This
    /// is what makes the mesh's per-tick re-mint of a known bridged worker
    /// (`sovereign_daemon::daemon::reaffirm_plan` → `Rebridge`) free: it replaces
    /// a `/status` probe that rode the same congested iroh path as the tunnel.
    #[tokio::test]
    async fn repeated_resolution_of_an_unchanged_peer_reuses_the_bridge() {
        let server_ep = hermetic_endpoint(33, vec![ALPN.to_vec()]).await;
        let transport = IrohTransport::new(hermetic_endpoint(34, vec![]).await);
        let contact = contact_for(&server_ep, loopback_sockets(&server_ep));
        let a = transport.base_url(&contact).await;
        let b = transport.base_url(&contact).await;
        assert!(a.is_some(), "a contact with an address resolves");
        assert_eq!(a, b, "cached bridge must be reused");
    }

    /// A bare key — no relay and no address, in the contact or seeded — is
    /// dialled when the endpoint has an address lookup, and the lookup's
    /// answer is where the tunnel lands; with no lookup it is not dialled
    /// (DESIGN §8, "Four facts"). A memory lookup stands in for n0's DNS.
    #[tokio::test]
    async fn a_bare_key_is_dialled_through_the_endpoints_address_lookup() {
        let worker_addr = spawn_marker_listener(b"WORKER").await;
        let server_ep = hermetic_endpoint(35, vec![ALPN.to_vec()]).await;
        spawn_splicing_acceptor(server_ep.clone(), worker_addr);
        let bare = contact_for(&server_ep, Vec::new());

        let without = IrohTransport::new(hermetic_endpoint(36, vec![]).await);
        assert_eq!(
            without.base_url(&bare).await,
            None,
            "with no lookup a bare key is not dialled"
        );

        let lookup = iroh::address_lookup::MemoryLookup::new();
        let mut found = iroh::EndpointAddr::new(server_ep.id());
        for a in loopback_sockets(&server_ep) {
            found = found.with_ip_addr(a);
        }
        lookup.add_endpoint_info(found);
        let dialling = EndpointBuilder::empty()
            .crypto_provider(ring_crypto_provider())
            .secret_key(SecretKey::from_bytes(&[37; 32]))
            .address_lookup(lookup)
            .bind()
            .await
            .expect("hermetic endpoint bind");
        let with = IrohTransport::new(dialling);
        let base = with
            .base_url(&bare)
            .await
            .expect("with a lookup a bare key is dialled");
        let port: SocketAddr = base.strip_prefix("http://").unwrap().parse().unwrap();
        assert_eq!(
            read_marker_from_port(port).await,
            b"WORKER",
            "the tunnel lands where the lookup said"
        );
    }

    #[test]
    fn dial_key_normalizes_addr_order_and_dupes() {
        let a: SocketAddr = "10.0.0.1:1000".parse().unwrap();
        let b: SocketAddr = "10.0.0.2:2000".parse().unwrap();
        let mk = |addrs: Vec<SocketAddr>| Contact {
            key: SecretKey::from_bytes(&[1u8; 32]).public(),
            relay_url: Some("https://r.example/".into()),
            iroh_direct_addrs: addrs,
        };
        // Same set, different order / dup → SAME key: a reordered contact
        // must NOT churn the bridge.
        assert_eq!(
            super::dial_key_for(&mk(vec![a, b])),
            super::dial_key_for(&mk(vec![b, a, b]))
        );
        // A changed port (peer restarted) → DIFFERENT key → rebuild.
        let b2: SocketAddr = "10.0.0.2:2001".parse().unwrap();
        assert_ne!(
            super::dial_key_for(&mk(vec![a, b])),
            super::dial_key_for(&mk(vec![a, b2]))
        );
    }
}
