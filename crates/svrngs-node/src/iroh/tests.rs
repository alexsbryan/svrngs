// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/iroh.rs@e1a43bcd2
use super::*;
use crate::guest::ring_crypto_provider;
use iroh::endpoint::Builder as EndpointBuilder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// What keeps a router served on the stream: the serving endpoint, its
/// acceptor, and the dialer's transport, whose bridge the URL names.
pub(crate) struct OnStream {
    _acceptor: IrohAcceptor,
    transport: IrohTransport,
    contact: Contact,
}

impl OnStream {
    /// The served router as a member a request is sent to, from the
    /// dialer's transport (arch-peer (c)).
    pub(crate) fn to(&self) -> (&IrohTransport, &Contact) {
        (&self.transport, &self.contact)
    }
}

/// `router` served by an acceptor that admits every dialer, on a
/// hermetic endpoint; and the base URL at which a dialer holding
/// `dialer` reaches it through the bridge. For a route that decides on
/// the [`Dialer`] itself, so the admit decision is not what a test of it
/// watches.
pub(crate) async fn serve_on_stream(router: axum::Router, dialer: SecretKey) -> (String, OnStream) {
    let served = hermetic_endpoint(0xEE, vec![ALPN.to_vec()]).await;
    let addrs = loopback_sockets(&served);
    let acceptor = IrohAcceptor::spawn(served.clone(), move |_, _| Admit::Serve(router.clone()));
    let dialling = EndpointBuilder::empty()
        .crypto_provider(ring_crypto_provider())
        .secret_key(dialer)
        .bind()
        .await
        .expect("hermetic endpoint bind");
    let transport = IrohTransport::new(dialling);
    let contact = Contact {
        key: served.id(),
        relay_url: None,
        iroh_direct_addrs: addrs,
    };
    let base = transport
        .spliced(&contact)
        .await
        .expect("a contact with an address resolves");
    (
        base,
        OnStream {
            _acceptor: acceptor,
            transport,
            contact,
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
    let bridge = HttpBridge::spawn_preferring(client_ep, target, ALPN, Some(port), None)
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
/// splice stands in, whatever the acceptor serves.
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
                        PumpSide::Bridge,
                        "peer".to_string(),
                        ALPN,
                    ));
                }
            });
        }
    });
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

/// The dialer an iroh stream from `key`'s holder is verified as, for a
/// test that asks a route's decision with no stream (#146).
pub(crate) fn dialer(key: &crate::SigningKey) -> Dialer {
    Dialer(SecretKey::from_bytes(&key.to_bytes()).public())
}

/// A router answering `/who` with the key its request came from, and
/// `/never` never, served on a hermetic endpoint that admits a dialer or
/// refuses it as `admits` says; the endpoint, and how many connections it
/// was asked to admit.
async fn served(seed: u8, admits: bool) -> (Endpoint, Arc<std::sync::atomic::AtomicUsize>) {
    use axum::routing::get;
    let router = axum::Router::new()
        .route(
            "/who",
            get(|axum::Extension(d): axum::Extension<Dialer>| async move { d.actor() }),
        )
        .route("/never", get(std::future::pending::<String>));
    let endpoint = hermetic_endpoint(seed, vec![ALPN.to_vec()]).await;
    let asked = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted = asked.clone();
    let acceptor = IrohAcceptor::spawn(endpoint.clone(), move |_, _| {
        counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if admits {
            Admit::Serve(router.clone())
        } else {
            Admit::Refuse
        }
    });
    std::mem::forget(acceptor);
    (endpoint, asked)
}

fn get(path: &str) -> Request {
    axum::http::Request::get(path)
        .body(axum::body::Body::empty())
        .unwrap()
}

/// **A request on the stream** (arch-peer (b)): [`IrohTransport::fetch`]
/// reaches the member's router as this computer's key, read whole, and a
/// second request rides a stream of the same connection, so the member is
/// dialled, and asked to admit, once. Planted: a fresh dial per request
/// (the kept connection not read), which the member admits twice.
#[tokio::test]
async fn a_request_rides_a_stream_of_the_one_connection_kept_to_the_member() {
    let (member, asked) = served(41, true).await;
    let transport = IrohTransport::new(hermetic_endpoint(42, vec![]).await);
    let contact = contact_for(&member, loopback_sockets(&member));
    for _ in 0..2 {
        let answer = transport
            .fetch(&contact, get("/who"))
            .await
            .expect("an answer");
        assert_eq!(answer.status(), axum::http::StatusCode::OK);
        assert_eq!(
            *answer.body(),
            key_actor(&SecretKey::from_bytes(&[42; 32]).public())
        );
    }
    assert_eq!(
        asked.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "one connection, two requests"
    );
}

/// **Not admitted, from the first dial** (arch-peer (b)): a member that
/// does not know this computer stands in any group it holds closes the
/// connection as [`NOT_ADMITTED`], and the request is said
/// [`Unsent::NotAdmitted`], the member dialled once. Planted: the close
/// reason not read, which says [`Unsent::Failed`].
#[tokio::test]
async fn a_member_that_does_not_admit_this_computer_is_said_so_from_the_first_dial() {
    let (member, asked) = served(43, false).await;
    let transport = IrohTransport::new(hermetic_endpoint(44, vec![]).await);
    let contact = contact_for(&member, loopback_sockets(&member));
    let refused = transport.send(&contact, get("/who")).await;
    assert!(matches!(refused, Err(Unsent::NotAdmitted)), "{refused:?}");
    assert_eq!(
        asked.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "dialled once"
    );
}

/// **Asleep within the cap** (arch-peer (b)): a computer that answers no
/// dial is said not answered within [`crate::dial::DIAL_WITHIN`],
/// and a bare key with no lookup is [`Unsent::NoPath`], never dialled.
/// Planted: the dial uncapped, which iroh gives up only after 30 s.
#[tokio::test]
async fn a_computer_that_answers_no_dial_is_asleep_within_the_cap() {
    let asleep = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let key = SecretKey::from_bytes(&[45; 32]).public();
    let transport = IrohTransport::new(hermetic_endpoint(46, vec![]).await);
    let contact = Contact {
        key,
        relay_url: None,
        iroh_direct_addrs: vec![asleep.local_addr().unwrap()],
    };
    let said = tokio::time::timeout(
        crate::dial::DIAL_WITHIN * 2,
        transport.send(&contact, get("/who")),
    )
    .await;
    assert!(
        matches!(&said, Ok(Err(Unsent::Failed(why))) if why.starts_with("not answered within 4s")),
        "{said:?}"
    );
    let said = transport.send(&Contact::by_key(key), get("/who")).await;
    assert!(matches!(said, Err(Unsent::NoPath)), "{said:?}");
}

/// **A whole answer within ten seconds** (arch-peer (b)): a member that
/// takes the request and never answers it is said to have no whole answer at
/// [`ANSWERED_WITHIN`], the round's reqwest client's bound, kept.
/// Planted: no bound, which waits as long as the member does.
#[tokio::test]
async fn an_answer_not_read_whole_within_ten_seconds_is_slow() {
    let (member, _) = served(47, true).await;
    let transport = IrohTransport::new(hermetic_endpoint(48, vec![]).await);
    let contact = contact_for(&member, loopback_sockets(&member));
    let said = tokio::time::timeout(
        ANSWERED_WITHIN * 2,
        transport.fetch(&contact, get("/never")),
    )
    .await;
    assert!(
        matches!(&said, Ok(Err(Unsent::Failed(why))) if why.starts_with("no whole answer within 10s")),
        "{said:?}"
    );
}
