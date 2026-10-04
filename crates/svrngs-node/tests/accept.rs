//! The accept loop admits on the dialer's verified key and serves the caller's
//! router on the stream (DESIGN §8, U1's inventory, the iroh row): two
//! hermetic endpoints in one process, no relay, B dialling A through the
//! loopback bridge with reqwest.

mod common;
use common::loopback_sockets;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::{routing::get, Extension, Router};
use svrngs_node::iroh::{
    ring_crypto_provider, Admit, Contact, Dialer, Endpoint, EndpointBuilder, IrohAcceptor,
    IrohTransport, PublicKey, SecretKey, ALPN,
};

/// Bind an empty (no-relay, deterministic) endpoint serving `alpns` — iroh.rs
/// tests' `hermetic_endpoint`.
async fn hermetic_endpoint(seed: u8, alpns: Vec<Vec<u8>>) -> Endpoint {
    EndpointBuilder::empty()
        .crypto_provider(ring_crypto_provider())
        .secret_key(SecretKey::from_bytes(&[seed; 32]))
        .alpns(alpns)
        .bind()
        .await
        .expect("hermetic endpoint bind")
}

fn key(seed: u8) -> PublicKey {
    SecretKey::from_bytes(&[seed; 32]).public()
}

/// A, serving one route that answers the actor its request's [`Dialer`]
/// names, and counts the requests it served. It admits `admitted` alone.
async fn spawn_a(admitted: PublicKey) -> (Endpoint, IrohAcceptor, Arc<AtomicUsize>) {
    let a = hermetic_endpoint(1, vec![ALPN.to_vec()]).await;
    let served = Arc::new(AtomicUsize::new(0));
    let count = served.clone();
    let router = Router::new().route(
        "/whoami",
        get(move |Extension(dialer): Extension<Dialer>| {
            count.fetch_add(1, Ordering::SeqCst);
            async move { dialer.actor() }
        }),
    );
    let admit = move |alpn: &[u8], dialer: &PublicKey| {
        if alpn == ALPN && *dialer == admitted {
            Admit::Serve(router.clone())
        } else {
            Admit::Refuse
        }
    };
    let acceptor = IrohAcceptor::spawn(a.clone(), admit);
    (a, acceptor, served)
}

/// `seed`'s base URL for A, through u1-1's bridge, which lives as long as the
/// transport returned beside it.
async fn dial(seed: u8, a: &Endpoint) -> (IrohTransport, String) {
    let transport = IrohTransport::new(hermetic_endpoint(seed, vec![]).await);
    let base = transport
        .base_url(&Contact {
            key: a.id(),
            relay_url: None,
            iroh_direct_addrs: loopback_sockets(a),
        })
        .await
        .expect("a contact with an address resolves");
    (transport, base)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// B's request reaches A's handler with B's key in the extension, and a
/// header naming C changes nothing the handler sees. The failing inputs are
/// an extension filled from a request header, or none at all.
#[tokio::test]
async fn a_handler_reads_the_dialers_verified_key_and_no_header_names_another() {
    let (b, c) = (key(2), key(3));
    let (a, _acceptor, served) = spawn_a(b).await;
    let (_b, base) = dial(2, &a).await;
    let url = format!("{base}/whoami");

    let plain = tokio::time::timeout(Duration::from_secs(20), client().get(&url).send())
        .await
        .expect("no timeout")
        .expect("B is admitted and answered");
    assert_eq!(plain.status(), 200);
    assert_eq!(plain.text().await.unwrap(), hex::encode(b.as_bytes()));

    let claiming = tokio::time::timeout(
        Duration::from_secs(20),
        client()
            .get(&url)
            .header("X-Mesh-Pubkey", hex::encode(c.as_bytes()))
            .send(),
    )
    .await
    .expect("no timeout")
    .expect("B is admitted and answered");
    assert_eq!(claiming.status(), 200);
    assert_eq!(
        claiming.text().await.unwrap(),
        hex::encode(b.as_bytes()),
        "a header naming C must not change the dialer the handler sees"
    );
    assert_eq!(served.load(Ordering::SeqCst), 2);
}

/// A dialer the admit decision refuses has its connection closed: reqwest
/// gets no HTTP answer at all, and A's handler never runs. The failing input
/// is an acceptor that serves whatever the decision says.
#[tokio::test]
async fn a_refused_dialer_gets_its_connection_closed_with_no_http_answer() {
    let (a, _acceptor, served) = spawn_a(key(2)).await;
    let (_d, base) = dial(4, &a).await;
    let answer = tokio::time::timeout(
        Duration::from_secs(20),
        client().get(format!("{base}/whoami")).send(),
    )
    .await
    .expect("a closed connection is an error, not a hang");
    assert!(
        answer.is_err(),
        "a refused dialer must get no HTTP answer, got {:?}",
        answer.map(|r| r.status())
    );
    assert_eq!(served.load(Ordering::SeqCst), 0);
}
