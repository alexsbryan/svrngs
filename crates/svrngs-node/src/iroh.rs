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

use n0_future::task::AbortOnDropHandle;
use std::sync::Arc;
use std::time::Duration;

pub use crate::alpn::{Alpn, ALPN};
// The guest dialer and the one HTTP bridge live in `crate::guest`, as they
// lived in `mesh_reach::guest` upstream (pb-reach-guest, phase-b-35);
// re-exported so callers spell them `svrngs_node::iroh::`.
pub use crate::guest::{build_relayed_endpoint, HttpBridge, RelayConfig};

// Re-exported so callers build endpoints without declaring their own iroh
// dependency — keeps the version pin in exactly one place.
pub use iroh::{Endpoint, EndpointAddr, PublicKey, RelayUrl, SecretKey};
// svrngs: iroh's address book in memory, so a test tells a joiner the
// loopback addresses a link's bare key resolves to (u2-8).
pub use iroh::address_lookup::MemoryLookup;

/// Render an endpoint's current dial info as a pairing string
/// (`<64-hex-endpoint-id>@<target>[,<target>...]`). Relay URLs first (stable),
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

impl Contact {
    /// A peer known by its key alone, no hint: how svrngs dials a member,
    /// since nothing gossips addresses (DESIGN §8, "A bare key is dialled").
    pub fn by_key(key: PublicKey) -> Self {
        Self {
            key,
            relay_url: None,
            iroh_direct_addrs: Vec::new(),
        }
    }
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

    /// Whether `peer`, dialled on the members' protocol now, closes the
    /// connection as [`NOT_ADMITTED`]: it answered, and does not know this
    /// computer stands in any group it holds. Asked once a request through
    /// the bridge has failed, which reads that close as no answer
    /// (`demo/runs/first-minute/2026-10-12/README.md`). A dial that fails, or
    /// a connection still open after [`CLOSED_WITHIN`], is not that.
    pub async fn closes_unadmitted(&self, peer: &Contact) -> bool {
        let label = &peer.key.to_string()[..8];
        let closed = match self.endpoint_addr_for(peer) {
            None => None,
            Some(target) => match self.endpoint.connect(target, ALPN).await {
                Ok(conn) => tokio::time::timeout(CLOSED_WITHIN, conn.closed())
                    .await
                    .ok(),
                Err(e) => {
                    tracing::debug!(target: "transport", peer = %label, error = %e, "iroh: the peer did not answer a dial");
                    None
                }
            },
        };
        let unadmitted = matches!(&closed, Some(iroh::endpoint::ConnectionError::ApplicationClosed(c))
            if c.reason.as_ref() == NOT_ADMITTED);
        tracing::debug!(target: "transport", peer = %label, unadmitted, ?closed,
            "iroh: whether the peer closes this computer's connection as not admitted");
        unadmitted
    }
}

/// The reason the acceptor closes a connection it does not admit with,
/// which [`IrohTransport::closes_unadmitted`] reads: the one answer a
/// stranger is given, at the connection and never in HTTP.
pub const NOT_ADMITTED: &[u8] = b"not admitted";

/// How long [`IrohTransport::closes_unadmitted`] waits for the close: the
/// acceptor closes as soon as the handshake is done.
const CLOSED_WITHIN: Duration = Duration::from_secs(1);

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
    /// Hand the connection to iroh-blobs' provider (gf-1): a shared
    /// folder's bytes by hash, on its own protocol.
    Blobs(iroh_blobs::BlobsProtocol),
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
    _task: AbortOnDropHandle<()>,
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
                            conn.close(0u32.into(), NOT_ADMITTED);
                            return;
                        }
                        Admit::Blobs(blobs) => {
                            use iroh::protocol::ProtocolHandler;
                            tracing::debug!(target: "transport", dialer = %dialer.actor(), "iroh acceptor: dialer admitted — its connection handed to the blobs provider");
                            blobs.accept(conn).await.ok();
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
        Self {
            _task: AbortOnDropHandle::new(task),
        }
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
pub(crate) mod tests;
