//! Where a computer meets its groups, in tiers (relay fallbacks; svrn-docs
//! `rings/current/GROUP_RELAYS.md` §7): iroh dials a member at a relay this
//! computer's endpoint does not hold, which every dial across tiers rests
//! on; and a computer whose group names a relay of its own meets there,
//! takes the build's relay when its own stops answering, and gives it back
//! once its own answers again (`svrngs_node::tiers`). Each relay plain
//! HTTP on 127.0.0.1, as `svrngs relay --at` serves one, the build's a
//! second such relay standing in for n0's so nothing leaves this computer.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs_core::relay::Url;
use svrngs_node::iroh::{build_relayed_endpoint, EndpointAddr, RelayConfig, RelayUrl, SecretKey};
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

/// A plain-HTTP relay on 127.0.0.1 at `at` (port 0 for any), and its URL.
async fn relay(at: SocketAddr) -> (Server, String) {
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(at));
    let server = Server::spawn(config).await.expect("a relay on 127.0.0.1");
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

/// **A dial reaches a member at a relay this computer does not hold**:
/// Bo's endpoint holds R2 alone, and dials Ada at R1, her home, through
/// R1, in a second. Control: Bo dialling Ada at R2, where she is not, is
/// not answered.
#[tokio::test(flavor = "multi_thread")]
async fn a_dial_reaches_a_member_at_a_relay_this_computer_does_not_hold() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_r1, u1), (_r2, u2)) = (relay(any).await, relay(any).await);
    let alpn = b"tiers/0".to_vec();
    let bind = |seed: u8, url: String| {
        let alpn = vec![alpn.clone()];
        async move { build_relayed_endpoint(SecretKey::from_bytes(&[seed; 32]), alpn, &at(&url)).await }
    };
    let (ada, bo) = (
        bind(1, u1.clone()).await.unwrap(),
        bind(2, u2.clone()).await.unwrap(),
    );
    for e in [&ada, &bo] {
        tokio::time::timeout(Duration::from_secs(10), e.online())
            .await
            .expect("online");
    }
    let taken = tokio::spawn({
        let ada = ada.clone();
        async move { ada.accept().await.unwrap().await.unwrap() }
    });
    let dial =
        |url: &str| EndpointAddr::new(ada.id()).with_relay_url(url.parse::<RelayUrl>().unwrap());
    let missed = tokio::time::timeout(Duration::from_secs(3), bo.connect(dial(&u2), &alpn)).await;
    assert!(!matches!(missed, Ok(Ok(_))), "control: Ada is not at R2");
    let reached = tokio::time::timeout(Duration::from_secs(10), bo.connect(dial(&u1), &alpn)).await;
    assert!(
        matches!(reached, Ok(Ok(_))),
        "Bo reaches Ada at R1: {reached:?}"
    );
    taken.await.unwrap();
}

/// The relay `node` is homed at, once it is `want`, waited for up to
/// three of svrngs's asks and iroh's slowest re-check; the wait in ms.
async fn homed(node: &Node, want: &str) -> u128 {
    let t = Instant::now();
    let by = svrngs_node::tiers::ASKED_EVERY * 3 + Duration::from_secs(26);
    while t.elapsed() < by {
        let home = node
            .endpoint()
            .addr()
            .relay_urls()
            .next()
            .map(ToString::to_string);
        if home.as_deref() == Some(want) {
            return t.elapsed().as_millis();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("not homed at {want} within {by:?}");
}

/// **A group meets at the build's fallback while it answers, and at its
/// own relay when it does not** (relay fallbacks; the relay-fallback
/// charter: a browser may reach this computer only through the fallback):
/// Ada's group names A; her build's relay is B. She is homed at B; B
/// stopped, at A; B back at its address, at B again. Planted: the own tier
/// preferred while the build's answers, and once the fallback stops she
/// never reaches her own relay (red: not homed at A within 56 s).
#[tokio::test(flavor = "multi_thread")]
async fn a_group_meets_at_the_builds_fallback_while_it_answers() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_own, a), (build, b)) = (relay(any).await, relay(any).await);
    let b_at = build.http_addr().unwrap();
    let (dir, key) = (
        tempfile::tempdir().unwrap(),
        SigningKey::from_bytes(&[3; 32]),
    );
    let node = Node::start(
        dir.path().to_path_buf(),
        key.clone(),
        &at(&b),
        |_| Ok(None),
        any,
    )
    .await
    .unwrap();
    let name = |n: &str| svrngs_node::group::Name::parse(n).unwrap();
    let seed = svrngs_node::group::draw(name("house"), name("Ada"), key.verifying_key());
    let at_a = [a.parse::<Url>().unwrap()];
    node.host()
        .keep_group(&seed, None, &at_a)
        .expect("Ada keeps house, met at A");
    eprintln!(
        "TIERS at the build's fallback while it answers: {} ms",
        homed(&node, &b).await
    );
    build.shutdown().await.unwrap();
    eprintln!(
        "TIERS the fallback stopped, at its own: {} ms",
        homed(&node, &a).await
    );
    let _back = relay(b_at).await;
    eprintln!(
        "TIERS the fallback back, at the fallback: {} ms",
        homed(&node, &b).await
    );
    node.endpoint().close().await;
}
