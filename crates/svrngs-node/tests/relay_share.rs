//! **A relay with a tiny rate throttles a lender for real** (share-refusal;
//! RELAY_COMMONS design 3, "The signal" and "The refusal"): an open relay on
//! 127.0.0.1 held to a few hundred kilobytes a second, a member pulling a
//! range through it, so the lender's own endpoint is the connection the
//! relay reports rate limited (iroh's `relay_conns_ratelimited`, read by
//! `Host::watch_share`); the door then answers the next range over that
//! relay `429`, `x-svrngs-refused: share`, in the design's words, while a
//! request under the door's line passes. The member's endpoint is
//! relay-only, so the range's bytes go up through the relay, and the pull
//! is what spends the share. Planted, watched red: `Net::ratelimited`
//! answering zero, the report never seen, where the range after the pull
//! was served again and the poll timed out.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use iroh_relay::server::{ClientRateLimit, RelayConfig as RelayServer, Server, ServerConfig};
use svrngs_node::iroh::{EndpointAddr, RelayConfig, RelayUrl, SecretKey};
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

/// A plain-HTTP relay on 127.0.0.1 at `at`, holding each client to `rate`
/// bytes a second with a `burst` (relay-share's `ClientRateLimit`, at
/// numbers a test can trip), and its URL.
async fn relay(at: SocketAddr, rate: u32, burst: u32) -> (Server, String) {
    let mut config = RelayServer::new(at);
    let mut share = ClientRateLimit::new(NonZeroU32::new(rate).unwrap());
    share.max_burst_bytes = Some(NonZeroU32::new(burst).unwrap());
    config.limits.client_rx = Some(share);
    let mut server_config = ServerConfig::default();
    server_config.relay = Some(config);
    let server = Server::spawn(server_config)
        .await
        .expect("a relay on 127.0.0.1");
    let url = format!("http://{}/", server.http_addr().unwrap());
    (server, url)
}

fn at(url: &str) -> RelayConfig {
    RelayConfig {
        relay_urls: vec![url.into()],
        n0_services: false,
        loopback_only: false,
    }
}

/// A server on 127.0.0.1 that answers a request with `Range:` a 206 of
/// `big` bytes and any other with 200 and a few: what the check pulls
/// through the relay and then asks past it.
async fn serving(big: usize) -> String {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((mut asked, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = asked.read(&mut buf).await.unwrap_or(0);
            let head = String::from_utf8_lossy(&buf[..n]).to_ascii_lowercase();
            let (status, body) = if head.contains("range:") {
                ("206 Partial Content", vec![b'r'; big])
            } else {
                ("200 OK", b"ok".to_vec())
            };
            let answer = format!(
                "HTTP/1.1 {status}\r\ncontent-length: {}\r\ncontent-type: application/octet-stream\r\nconnection: close\r\n\r\n",
                body.len()
            );
            let _ = asked.write_all(answer.as_bytes()).await;
            let _ = asked.write_all(&body).await;
        }
    });
    url
}

/// The port of `url`, as the lend's declaration names it.
fn port_of(url: &str) -> u16 {
    url.rsplit(':')
        .next()
        .unwrap()
        .trim_end_matches('/')
        .parse()
        .unwrap()
}

/// The asks that leave the fallback and two more, and iroh's slowest
/// re-check (relay_tiers.rs `homing_by`).
fn homing_by() -> Duration {
    svrngs_node::tiers::ASKED_EVERY * (svrngs_node::tiers::LEFT_AFTER + 2) + Duration::from_secs(26)
}

/// The relay `node` is homed at, once it is `want`, waited for up to
/// [`homing_by`].
async fn homed(node: &Node, want: &str) {
    let (t, by) = (Instant::now(), homing_by());
    while t.elapsed() < by {
        let home = node
            .endpoint()
            .addr()
            .relay_urls()
            .next()
            .map(ToString::to_string);
        if home.as_deref() == Some(want) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("not homed at {want} within {by:?}");
}

/// `GET /lend/<label>/` over `conn`, `extra` a header line (or none), the
/// answer read whole: a member's request through the door, on the stream.
async fn ask(
    conn: &iroh::endpoint::Connection,
    label: &svrngs_core::pass::Label,
    extra: &str,
) -> String {
    let (mut send, mut recv) = conn.open_bi().await.unwrap();
    let head =
        format!("GET /lend/{label}/ HTTP/1.1\r\nHost: x\r\n{extra}Connection: close\r\n\r\n");
    send.write_all(head.as_bytes()).await.unwrap();
    String::from_utf8_lossy(&recv.read_to_end(1 << 23).await.unwrap()).into_owned()
}

#[tokio::test]
async fn a_relay_with_a_tiny_rate_throttles_a_lender_and_the_door_refuses_a_range() {
    let (said, _heard) = common::heard("share=debug");
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    // A megabyte a second with a 64 KiB burst: the pull passes a few
    // hundred kilobytes and spends the bucket on the way.
    let (_relay, url) = relay(any, 1_000_000, 64 * 1024).await;
    let (dir, key) = (
        tempfile::tempdir().unwrap(),
        SigningKey::from_bytes(&[3; 32]),
    );
    let node = Node::start(
        dir.path().to_path_buf(),
        key.clone(),
        &at(&url),
        |_| Ok(None),
        any,
    )
    .await
    .unwrap();
    let name = |n: &str| svrngs_node::group::Name::parse(n).unwrap();
    let seed = svrngs_node::group::draw(name("house"), name("Ada"), key.verifying_key());
    node.host()
        .keep_group(&seed, None, &[])
        .expect("Ada keeps house");
    homed(&node, &url).await;
    let group = node.rail().namespaces().unwrap().remove(0);
    let (sam, rail) = (SigningKey::from_bytes(&[9; 32]), node.rail());
    let ada = svrngs_node::actor_of(rail.signer());
    svrngs_core::tests_support::let_in(
        (&group, &ada),
        (&sam, "sam", svrngs_node::Device::Computer),
        |mint| {
            rail.commit(&group, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(rail.commit(&group, redeem).unwrap().unwrap()),
    );
    let service = serving(256 * 1024).await;
    let lends = svrngs_node::lend::lends_dir(node.rail().root());
    std::fs::create_dir_all(&lends).unwrap();
    let lent = serde_json::json!({ "port": port_of(&service), "kind": "app", "limit": 3, "groups": [group] });
    std::fs::write(lends.join("telly.json"), lent.to_string()).unwrap();

    let map = iroh::RelayMap::from(url.parse::<RelayUrl>().unwrap());
    let dialer = iroh::endpoint::Builder::new(iroh::endpoint::presets::Minimal)
        .crypto_provider(svrngs_node::guest::ring_crypto_provider())
        .secret_key(SecretKey::from_bytes(&sam.to_bytes()))
        .relay_mode(iroh::RelayMode::Custom(map))
        .clear_ip_transports()
        .bind()
        .await
        .unwrap();
    let to = EndpointAddr::new(node.endpoint().id()).with_relay_url(url.parse().unwrap());
    let conn = dialer.connect(to, svrngs_node::alpn::ALPN).await.unwrap();
    let label = svrngs_core::pass::Label::of(node.endpoint().id().as_bytes(), "telly");

    // The pull: a range served, its bytes going up through the relay until
    // the relay's bucket is spent and it reports this connection rate
    // limited.
    let pulled = ask(&conn, &label, "Range: bytes=0-\r\n").await;
    assert!(pulled.starts_with("HTTP/1.1 206"), "{pulled}");
    assert!(
        pulled.len() > 256 * 1024,
        "the range arrived whole: {}",
        pulled.len()
    );

    // The next range over the same relay is refused `share`, in the
    // design's words, once the report has been read (the watch reads it
    // every second).
    let refused = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let answered = ask(&conn, &label, "Range: bytes=0-\r\n").await;
            if answered.starts_with("HTTP/1.1 429") {
                return answered;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
    .await
    .expect("the range over the spent relay was refused");
    assert!(refused.contains("x-svrngs-refused: share"), "{refused}");
    assert!(
        refused.contains("has used its share of the shared relay for now"),
        "{refused}"
    );

    // A small request passes: no range, and under the door's line.
    let small = ask(&conn, &label, "").await;
    assert!(small.starts_with("HTTP/1.1 200"), "{small}");
    assert!(small.ends_with("ok"), "{small}");

    // The signal was the node's own relay's report: the count of
    // connections it reported rate limited rose while this computer held
    // the build's relay.
    let lines = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    assert!(
        lines
            .lines()
            .any(|l| l.contains("count=1") && l.contains("at_build=true")),
        "the relay's report was seen: {lines}"
    );
    dialer.close().await;
    node.endpoint().close().await;
}
