// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/iroh.rs@e1a43bcd2
//! EXPERIMENTAL dial-by-key transport over iroh (QUIC by Ed25519
//! key, hole-punching, optional relays). Upstream feature-gated it behind
//! `iroh`; in svrngs it is the only transport, so nothing gates it.
//!
//! ## Shape: localhost byte-tunnels, not a new HTTP stack
//!
//! [`IrohTransport::base_url`] resolves a base URL, and every call
//! site keeps its existing `reqwest` client — so this transport
//! terminates iroh QUIC locally and hands HTTP a plain TCP socket:
//!
//! - Client side ([`IrohTransport`]): per-peer localhost
//!   `TcpListener`; each accepted TCP connection opens one iroh
//!   bi-stream to the peer (dialed by its Ed25519 key — the
//!   [`Contact`]'s `key`) and copies bytes both ways.
//! - Server side ([`IrohAcceptor`]): accepts iroh bi-streams and
//!   copies each into a fresh TCP connection to the daemon's
//!   existing localhost listener — unmodified axum router,
//!   unmodified middleware.
//!
//! Known upgrade path (deliberately NOT in the spike): serve hyper
//! directly on iroh streams (drop the double-copy), carry the
//! traffic class in the ALPN or a stream header so one acceptor can
//! route to both daemon ports, and a tunnel-proxy sidecar for the
//! raw-TCP `rpc-server` tensor traffic that this transport
//! intentionally does not cover.
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
//!   is not dialled.

use std::collections::HashMap;
use std::net::SocketAddr;

use std::sync::Arc;

pub use crate::alpn::ALPN;
// The guest dialer and the one HTTP bridge live in `crate::guest`, as they
// lived in `mesh_reach::guest` upstream (pb-reach-guest, phase-b-35);
// re-exported so callers spell them `svrngs_node::iroh::`. The acceptor below
// splices through the same `pump`.
pub use crate::guest::{
    build_relayed_endpoint, configured_proxy_redacted, parse_dial_string, ring_crypto_provider,
    HttpBridge, RelayConfig,
};
use crate::guest::{pump, PumpSide};

// Re-exported so callers build endpoints without declaring their own iroh
// dependency — keeps the version pin in exactly one place.
pub use iroh::endpoint::presets;
pub use iroh::endpoint::Builder as EndpointBuilder;
pub use iroh::{Endpoint, EndpointAddr, PublicKey, RelayUrl, SecretKey};

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
    known_addrs: std::sync::Mutex<HashMap<[u8; 32], Vec<SocketAddr>>>,
    /// One localhost bridge per peer, on the one [`ALPN`]. Each entry
    /// remembers the dial info it was built with (`dial_key`) so a
    /// dial-info change RETARGETS the bridge — a frozen target
    /// kept dialing a restarted peer's dead ephemeral port forever
    /// (the 2026-07-19 dual-restart heal deadlock's transport half).
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
    /// `None` when there's no usable path (no relay AND no address) — a
    /// bare key isn't dialable without one.
    fn endpoint_addr_for(&self, peer: &Contact) -> Option<iroh::EndpointAddr> {
        let mut ea = iroh::EndpointAddr::new(peer.key);
        let mut has_path = false;
        if let Some(relay) = peer.relay_url.as_deref() {
            match relay.parse::<RelayUrl>() {
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
        has_path.then_some(ea)
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
                 — not dialable"
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

/// Server half: accept iroh bi-streams and forward each to the
/// daemon's existing localhost HTTP listener.
pub struct IrohAcceptor {
    task: tokio::task::JoinHandle<()>,
}

impl Drop for IrohAcceptor {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl IrohAcceptor {
    /// Spawn the accept loop forwarding EVERY accepted bi-stream to a
    /// single local listener, regardless of negotiated ALPN. Right for
    /// a single-ALPN endpoint (Track M: `sovereign-server` binds only
    /// `cwth/client/0` → its HTTP listener).
    ///
    /// Admits any dialer. Correct where the local listener authenticates
    /// for itself (`sovereign-server` requires a bearer of every caller);
    /// NOT correct where it trusts loopback — see
    /// [`spawn_admitting`](Self::spawn_admitting).
    pub fn spawn(endpoint: iroh::Endpoint, forward_to: SocketAddr) -> Self {
        Self::run(
            endpoint,
            move |_alpn, _dialer| async move { Some(forward_to) },
        )
    }

    /// Spawn the accept loop routing each connection to a local
    /// listener chosen by its **negotiated ALPN** — the W1 capability
    /// that lets one daemon endpoint serve both the internal router
    /// (`cwth/http/0`) and the client router (`cwth/client/0`) without
    /// a port (the class chose the ALPN). A connection whose ALPN is
    /// not in `routes` is closed with a loud log, never misrouted.
    pub fn spawn_routed(endpoint: iroh::Endpoint, routes: HashMap<Vec<u8>, SocketAddr>) -> Self {
        Self::run(endpoint, move |alpn, _dialer| {
            let target = routes.get(&alpn).copied();
            async move { target }
        })
    }

    /// Spawn the accept loop routing each connection by its negotiated ALPN
    /// **and by who dialed it**.
    ///
    /// # Why the dialer has to be part of the routing decision
    ///
    /// An iroh endpoint accepts anyone. The dial string that reaches it is
    /// public by design — it rides in every mesh invite's `dial=` and is
    /// gossiped as `MemberRecord.node_pubkey` — so "holds the dial string"
    /// is not a credential and must never be treated as one. But the
    /// acceptor forwards by `TcpStream::connect`ing a loopback listener,
    /// and a listener that trusts loopback (correctly, for the local user)
    /// cannot tell that hop from a real local caller. Route on ALPN alone
    /// and every dial-string holder inherits whatever that listener grants
    /// its own machine.
    ///
    /// What the connection DOES carry is the dialer's Ed25519 public key,
    /// verified by the QUIC handshake — the same key a mesh gossips as
    /// `node_pubkey`. `resolve` receives it, so a caller can send members
    /// and strangers to different listeners (or send a stranger nowhere)
    /// on evidence rather than on transport.
    ///
    /// `resolve` returning `None` closes the connection. It is async so the
    /// decision can consult live state — membership changes between dials.
    pub fn spawn_admitting<F, Fut>(endpoint: iroh::Endpoint, resolve: F) -> Self
    where
        F: Fn(Vec<u8>, PublicKey) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Option<SocketAddr>> + Send + 'static,
    {
        Self::run(endpoint, resolve)
    }

    /// Shared accept loop. `resolve` maps a connection's negotiated ALPN
    /// and its verified dialer key to the local TCP target its bi-streams
    /// forward to; `None` closes the connection. Both are read once per
    /// connection, then every bi-stream on it is pumped to that target.
    fn run<F, Fut>(endpoint: iroh::Endpoint, resolve: F) -> Self
    where
        F: Fn(Vec<u8>, PublicKey) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Option<SocketAddr>> + Send + 'static,
    {
        let resolve = Arc::new(resolve);
        let task = tokio::spawn(async move {
            while let Some(incoming) = endpoint.accept().await {
                let resolve = resolve.clone();
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
                    // certificate). Route this connection's streams by both.
                    let alpn = conn.alpn().to_vec();
                    let dialer = conn.remote_id();
                    let Some(forward_to) = resolve(alpn.clone(), dialer).await else {
                        tracing::warn!(
                            target: "transport",
                            alpn = %String::from_utf8_lossy(&alpn),
                            dialer = %hex::encode(dialer.as_bytes()),
                            "iroh acceptor: no local forward for this (ALPN, dialer) — closing connection"
                        );
                        return;
                    };
                    // Ends when the connection closes.
                    while let Ok((send, recv)) = conn.accept_bi().await {
                        // The pump's zero-answer warn names the ALPN;
                        // clone it so the next accepted stream still
                        // has it.
                        let alpn = alpn.clone();
                        tokio::spawn(async move {
                            match tokio::net::TcpStream::connect(forward_to).await {
                                Ok(tcp) => {
                                    // Same Nagle × delayed-ACK stall as the
                                    // bridge side — see HttpBridge::spawn.
                                    tcp.set_nodelay(true).ok();
                                    pump(
                                        tcp,
                                        send,
                                        recv,
                                        PumpSide::Acceptor,
                                        hex::encode(dialer.as_bytes()),
                                        &alpn,
                                    )
                                    .await
                                }
                                Err(e) => tracing::warn!(
                                    target: "transport",
                                    error = %e,
                                    forward_to = %forward_to,
                                    "iroh acceptor: local forward connect failed"
                                ),
                            }
                        });
                    }
                });
            }
        });
        Self { task }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Bind an empty (no-relay, deterministic) endpoint serving `alpns`.
    async fn hermetic_endpoint(seed: u8, alpns: Vec<Vec<u8>>) -> Endpoint {
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
    fn loopback_sockets(endpoint: &Endpoint) -> Vec<SocketAddr> {
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

    /// The acceptor splices through the leaf's one `pump` (pb-reach-guest):
    /// the name it calls resolves to `crate::guest::pump`, so a local
    /// copy of the pump in this file turns this red.
    #[test]
    fn the_acceptor_pumps_through_the_leafs_one_pump() {
        assert_eq!(
            std::any::type_name_of_val(&pump),
            "svrngs_node::guest::pump"
        );
    }
}
