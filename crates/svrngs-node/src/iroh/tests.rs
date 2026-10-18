// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/iroh.rs@e1a43bcd2
use super::*;
use crate::guest::ring_crypto_provider;
use iroh::endpoint::Builder as EndpointBuilder;

/// What a test's subscriber writes, kept to be read: one copy, where
/// blob_tests.rs, relay_tests.rs and guest/tests.rs each wrote out a sink
/// (REVIEW-audit-arch-auto-1).
#[derive(Clone, Default)]
#[expect(clippy::disallowed_types, reason = "a test's trace sink")]
pub(crate) struct Said(pub(crate) std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for Said {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// What keeps a router served on the stream: its acceptor, and the
/// dialer's transport and the contact it reaches the router by.
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
/// `dialer` reaches it through a test's splice
/// ([`crate::node::splice`]). For a route that decides on
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

fn contact_for(server: &Endpoint, addrs: Vec<SocketAddr>) -> Contact {
    Contact {
        key: server.id(),
        relay_url: None,
        iroh_direct_addrs: addrs,
    }
}

/// A bare key — no relay and no address, in the contact or seeded — is
/// dialled when the endpoint has an address lookup, and the lookup's
/// answer is where the request lands; with no lookup it is not dialled
/// (PLAN §4, U1's "A bare key is dialled"). A memory lookup stands in for n0's DNS.
/// (svrngs, arch-peer (d): asked by a request on the stream, where the
/// bridge's test read a marker through its loopback port.)
#[tokio::test]
async fn a_bare_key_is_dialled_through_the_endpoints_address_lookup() {
    let (member, _) = served(35, true).await;
    let bare = contact_for(&member, Vec::new());

    let without = IrohTransport::new(hermetic_endpoint(36, vec![]).await);
    let said = without.fetch(&bare, get("/who")).await;
    assert!(
        matches!(said, Err(Unsent::NoPath)),
        "with no lookup a bare key is not dialled: {said:?}"
    );

    let lookup = iroh::address_lookup::MemoryLookup::new();
    let mut found = iroh::EndpointAddr::new(member.id());
    for a in loopback_sockets(&member) {
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
    let answer = with
        .fetch(&bare, get("/who"))
        .await
        .expect("with a lookup a bare key is dialled");
    assert_eq!(
        *answer.body(),
        key_actor(&SecretKey::from_bytes(&[37; 32]).public()),
        "the request lands where the lookup said"
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
        matches!(&said, Ok(Err(e @ Unsent::Asleep(_))) if e.to_string().starts_with("not answered within 4s")),
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

/// **Which connection a request went on, traced** (#178): two requests to
/// a member, the first said dialled and the second said riding the
/// connection kept to it, each with its protocol. Red with the plant the
/// finding names, the kept connection's request untraced (`kept` as at
/// 5d5cb1f6): no line says which.
#[tokio::test]
async fn which_connection_a_request_went_on_is_traced() {
    let said = Said::default();
    let writer = said.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("transport=debug")
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let _said = tracing::subscriber::set_default(subscriber);
    // A second dispatcher kept alive, as blob_tests.rs's trace test keeps
    // one: tracing-core 0.1.36 (callsite.rs:545) caches a callsite another
    // test's thread reached first unheard.
    let _two = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
    let (member, _) = served(49, true).await;
    let transport = IrohTransport::new(hermetic_endpoint(50, vec![]).await);
    let contact = contact_for(&member, loopback_sockets(&member));
    for _ in 0..2 {
        transport
            .fetch(&contact, get("/who"))
            .await
            .expect("an answer");
    }
    let log = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let which: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("iroh: a request on"))
        .map(|l| {
            if l.contains("kept=true") {
                "kept"
            } else {
                "dialled"
            }
        })
        .collect();
    assert_eq!(which, ["dialled", "kept"], "{log}");
    assert!(
        log.lines()
            .filter(|l| l.contains("iroh: a request on"))
            .all(|l| l.contains("alpn=Member")),
        "{log}"
    );
}
