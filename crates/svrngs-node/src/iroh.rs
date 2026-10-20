// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/iroh.rs@e1a43bcd2
//! EXPERIMENTAL dial-by-key transport over iroh (QUIC by Ed25519
//! key, hole-punching, optional relays). Upstream feature-gated it behind
//! `iroh`; in svrngs it is the only transport, so nothing gates it.
//!
//! ## Shape: hyper on the stream, both ways
//!
//! - Client side ([`IrohTransport::send`]): a request on a bi-stream of the
//!   connection kept to a member, dialled by its Ed25519 key — the
//!   [`Contact`]'s `key` — and read with hyper's HTTP/1 client, as the
//!   phone's page reads one (`web/src/http.rs`).
//! - Server side ([`IrohAcceptor`]): admits or closes each iroh
//!   connection on its ALPN and the dialer's verified key, and serves
//!   every bi-stream of an admitted one with hyper's HTTP/1 server and
//!   the axum router its admit decision names — no loopback listener, and the dialer
//!   reaches handlers as a [`Dialer`] request extension, never a header.
//!
//! (svrngs, arch-peer: upstream's dialling half was a loopback port per
//! peer, every call site's reqwest client pointed at it, and its "known
//! upgrade path" was this. The port accepted any program on the computer
//! and carried it to the peer as this computer's key, `tests/peer.rs`.)
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
//!   which resolves it — PLAN §4, U1's "A bare key is dialled".)

use std::collections::HashMap;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};

use iroh::endpoint::Connection;
use tokio::time::timeout;

use n0_future::task::AbortOnDropHandle;
use std::sync::Arc;
use std::time::Duration;

pub use crate::alpn::{Alpn, ALPN};
// The guest dialer lives in `crate::guest`, as it lived in
// `mesh_reach::guest` upstream (pb-reach-guest, phase-b-35); re-exported so
// callers spell it `svrngs_node::iroh::`.
pub use crate::guest::{build_relayed_endpoint, RelayConfig};

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

/// Whether `endpoint` resolves a bare key by an address lookup: built only
/// with `n0_services` (`guest::relayed_endpoint_builder`) or one a test adds.
/// A closed endpoint has none.
pub fn has_lookup(endpoint: &Endpoint) -> bool {
    endpoint.address_lookup().is_ok_and(|l| !l.is_empty())
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
    /// since nothing gossips addresses (PLAN §4, U1's "A bare key is dialled").
    pub fn by_key(key: PublicKey) -> Self {
        Self {
            key,
            relay_url: None,
            iroh_direct_addrs: Vec::new(),
        }
    }

    /// A member by its key and the relay its group's record names for it
    /// (relay-in-record; DESIGN §6). With none named its key alone, said:
    /// only an address lookup finds it then, and with none it is not dialled.
    pub fn named(key: PublicKey, relay: Option<&svrngs_core::relay::Url>) -> Self {
        if relay.is_none() {
            tracing::debug!(target: "transport", peer = %key.fmt_short(),
                "transport: the group names no relay for this computer — its key alone, which only an address lookup finds");
        }
        Self {
            relay_url: relay.map(ToString::to_string),
            ..Self::by_key(key)
        }
    }
}

/// Client half: a request to a member, dialled by its key.
#[derive(Debug)]
pub struct IrohTransport {
    pub(crate) endpoint: iroh::Endpoint,
    /// Out-of-band dial hints: pubkey → iroh UDP socket addresses.
    /// **Fallback only.** Dial info comes first from the [`Contact`] —
    /// relay URL + direct addrs.
    /// Retained so hermetic tests can seed addresses directly; when a
    /// contact carries its own addrs, both are merged.
    #[expect(clippy::disallowed_types, reason = "the transport's address cache")]
    known_addrs: std::sync::Mutex<HashMap<[u8; 32], Vec<SocketAddr>>>,
    /// One connection per computer and protocol, kept while it stays open
    /// ([`Self::kept`]), the one rule for a member's requests and a folder's
    /// fetches (#177): each request is a bi-stream of it, as the phone's
    /// page sends one (`web/src/http.rs`), so a round dials a member once,
    /// not once a request (the ceiling's first round, every member dialled,
    /// cost 0.19 s of processor at 965b52b, the next 0.024 s). One that
    /// closes, or fails a request, is let go, so a member that restarted at
    /// another address is dialled anew where its contact says.
    pub(crate) conns: tokio::sync::watch::Sender<Connections>,
}

/// Each connection kept, by the computer's key and the protocol's bytes.
pub(crate) type Connections = std::collections::BTreeMap<([u8; 32], &'static [u8]), Connection>;

impl IrohTransport {
    #[expect(clippy::disallowed_types, reason = "the transport's address cache")]
    pub fn new(endpoint: iroh::Endpoint) -> Self {
        Self {
            endpoint,
            known_addrs: std::sync::Mutex::new(HashMap::new()),
            conns: tokio::sync::watch::Sender::new(Connections::new()),
        }
    }

    /// The connection kept to `peer` on `alpn` while it stays open, else
    /// one [`dialled`] within `within`, each dial's own cap, and kept.
    pub(crate) async fn kept(
        &self,
        peer: &Contact,
        alpn: Alpn,
        within: Duration,
    ) -> Result<Connection, Unsent> {
        let at = (*peer.key.as_bytes(), alpn.bytes());
        let kept = self.conns.borrow().get(&at).cloned();
        let kept = kept.filter(|c| c.close_reason().is_none());
        tracing::debug!(target: "transport", peer = %peer.key.fmt_short(), ?alpn, kept = kept.is_some(), "iroh: a request on the connection kept to it, else on one dialled (#178)");
        if let Some(conn) = kept {
            return Ok(conn);
        }
        // Which of the key's addresses iroh dials: one in use is the only
        // one it sends to (syp-6 b).
        let known = self.endpoint.remote_info(peer.key).await;
        let known: Vec<String> = (known.iter().flat_map(|i| i.addrs()))
            .map(|a| format!("{:?} {:?}", a.addr(), a.usage()))
            .collect();
        tracing::debug!(target: "transport", peer = %peer.key.fmt_short(), ?alpn, ?known, "iroh: dialled by key, iroh holding these addresses for it");
        let to = self.endpoint_addr_for(peer).ok_or(Unsent::NoPath)?;
        let conn = dialled(&self.endpoint, to, alpn.bytes(), Some(within)).await?;
        self.conns.send_modify(|m| drop(m.insert(at, conn.clone())));
        Ok(conn)
    }

    /// The connection kept to `peer` on `alpn` let go, a request on it
    /// failed: the next dials afresh.
    pub(crate) fn let_go(&self, peer: &PublicKey, alpn: Alpn) {
        self.conns
            .send_if_modified(|m| m.remove(&(*peer.as_bytes(), alpn.bytes())).is_some());
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
    /// gossip a member is dialled by key and the relay its group names,
    /// relay-in-record, else the endpoint's own lookup — n0's DNS under
    /// `RelayConfig::n0_services` — finds it, PLAN §4, U1's "A bare key is dialled".)
    pub(crate) fn endpoint_addr_for(&self, peer: &Contact) -> Option<iroh::EndpointAddr> {
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
        let lookup = has_lookup(&self.endpoint);
        tracing::debug!(
            target: "transport",
            transport = "iroh",
            peer = %&peer.key.to_string()[..8],
            lookup,
            "iroh: a bare key — dialled through the endpoint's address lookup when it has one"
        );
        lookup.then_some(ea)
    }
}

impl IrohTransport {
    /// `req` sent to the member `peer` names, on the members' protocol, on
    /// a bi-stream of the connection kept to it, or of one dialled within
    /// [`crate::dial::DIAL_WITHIN`]: its answer's head, its body still to
    /// read, so a stream or a WebSocket is carried whole. A connection that
    /// fails a request is let go, and the next request dials afresh.
    pub async fn send(&self, peer: &Contact, req: Request) -> Result<Streamed, Unsent> {
        let conn = self
            .kept(peer, Alpn::Member, crate::dial::DIAL_WITHIN)
            .await?;
        let answered = on_stream(&conn, req).await;
        if answered.is_err() {
            self.let_go(&peer.key, Alpn::Member);
        }
        answered
    }

    /// [`Self::send`], its answer read whole within [`ANSWERED_WITHIN`].
    pub async fn fetch(&self, peer: &Contact, req: Request) -> Result<Whole, Unsent> {
        whole(self.send(peer, req)).await
    }
}

/// How long a request read whole waits for its answer, the dial included:
/// the ten seconds the round's reqwest client was built with
/// (`ring_sync.rs` `peer_client` at 965b52b, cut with the bridge), kept.
pub const ANSWERED_WITHIN: Duration = Duration::from_secs(10);

/// A request on the stream: axum's, as the door and the origin carry one.
pub type Request = axum::http::Request<axum::body::Body>;
/// An answer's head, its body read as it comes.
pub type Streamed = axum::http::Response<hyper::body::Incoming>;
/// An answer read whole.
pub type Whole = axum::http::Response<axum::body::Bytes>;

/// Why a request to a computer has no answer, each traced at debug where
/// it is decided.
#[derive(Debug, thiserror::Error)]
pub enum Unsent {
    #[error("no dialable path: no relay, no address and no lookup")]
    NoPath,
    #[error("it answered, and does not know this computer stands in any group it holds")]
    NotAdmitted,
    /// A dial not answered within its cap: a computer taken as asleep.
    #[error("not answered within {0:?}")]
    Asleep(Duration),
    #[error("{0}")]
    Failed(String),
}

/// `answer` read whole, within [`ANSWERED_WITHIN`].
pub(crate) async fn whole(
    answer: impl Future<Output = Result<Streamed, Unsent>>,
) -> Result<Whole, Unsent> {
    use http_body_util::BodyExt;
    let read = async {
        let (head, body) = answer.await?.into_parts();
        let body = body.collect().await.map_err(|e| e.to_string());
        Ok(Whole::from_parts(
            head,
            body.map_err(Unsent::Failed)?.to_bytes(),
        ))
    };
    let slow = Unsent::Failed(format!("no whole answer within {ANSWERED_WITHIN:?}"));
    timeout(ANSWERED_WITHIN, read).await.unwrap_or(Err(slow))
}

/// A connection to `to` on `alpn`, dialled within `within` when given, and
/// then held as a phone's page holds its dial ([`crate::dial::held`]), so a
/// computer that stops answering a connection kept to it is taken as asleep
/// within the cap too, as a fresh dial would be.
pub(crate) async fn dialled(
    e: &Endpoint,
    to: EndpointAddr,
    alpn: &'static [u8],
    within: Option<Duration>,
) -> Result<Connection, Unsent> {
    let (peer, cap) = (to.id.fmt_short(), within.unwrap_or(Duration::MAX));
    // Held only when capped: held, iroh gives up a handshake silent for the
    // cap too, and a joiner's dial is not capped (`crate::dial`).
    let held = within.map_or_else(Default::default, |_| crate::dial::held());
    let held = iroh::endpoint::ConnectOptions::new().with_transport_config(held);
    let connect = async move {
        let connecting = e.connect_with_opts(to, alpn, held).await;
        let failed = |e: &dyn std::fmt::Display| Unsent::Failed(e.to_string());
        connecting
            .map_err(|e| failed(&e))?
            .await
            .map_err(|e| failed(&e))
    };
    let dialled = timeout(cap, connect)
        .await
        .unwrap_or(Err(Unsent::Asleep(cap)));
    tracing::debug!(target: "transport", %peer, ?within, error = ?dialled.as_ref().err(), "iroh: a dial, answered or not; one not answered in time is a computer taken as asleep");
    dialled
}

/// `req` on a new bi-stream of `conn`, one HTTP/1 connection driven with
/// upgrades, a `Host` named when it names none, as the page names one
/// (`web/src/http.rs`); a failure read as [`Unsent::NotAdmitted`] when the
/// far end closed the connection as [`NOT_ADMITTED`].
pub(crate) async fn on_stream(conn: &Connection, mut req: Request) -> Result<Streamed, Unsent> {
    let host = req.headers_mut().entry(axum::http::header::HOST);
    host.or_insert(axum::http::HeaderValue::from_static("svrngs"));
    let sent = async {
        let (send, recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
        let io = hyper_util::rt::TokioIo::new(tokio::io::join(recv, send));
        let shake = hyper::client::conn::http1::handshake(io).await;
        let (mut sender, driver) = shake.map_err(|e| e.to_string())?;
        tokio::spawn(driver.with_upgrades());
        sender.send_request(req).await.map_err(|e| e.to_string())
    };
    let failed = match sent.await {
        Ok(answer) => return Ok(answer),
        Err(failed) => failed,
    };
    let unadmitted = closed_unadmitted(conn).await;
    tracing::debug!(target: "transport", peer = %conn.remote_id().fmt_short(), unadmitted, failed, "iroh: a request had no answer");
    if unadmitted {
        return Err(Unsent::NotAdmitted);
    }
    Err(Unsent::Failed(failed))
}

/// Whether the far end of `conn` closes it as [`NOT_ADMITTED`] within
/// [`CLOSED_WITHIN`]: it answered, and does not know this computer stands
/// in any group it holds.
async fn closed_unadmitted(conn: &Connection) -> bool {
    let closed = timeout(CLOSED_WITHIN, conn.closed()).await.ok();
    let unadmitted = matches!(&closed, Some(iroh::endpoint::ConnectionError::ApplicationClosed(c)) if c.reason.as_ref() == NOT_ADMITTED);
    tracing::debug!(target: "transport", peer = %conn.remote_id().fmt_short(), unadmitted, ?closed, "iroh: whether the peer closes this computer's connection as not admitted");
    unadmitted
}

/// The reason the acceptor closes a connection it does not admit with,
/// which a request on the stream reads ([`Unsent::NotAdmitted`]): the one answer a
/// stranger is given, at the connection and never in HTTP.
pub const NOT_ADMITTED: &[u8] = b"not admitted";

/// How long a request with no answer waits for its connection's close: the
/// acceptor closes as soon as the handshake is done.
const CLOSED_WITHIN: Duration = Duration::from_secs(1);

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
/// (PLAN §4, U3's inventory, its iroh_path row).
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
