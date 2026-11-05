//! A group's advertised public fallback reaches its issuer while a local
//! primary relay exists (relay fallbacks; CHARTER.md "Reuse and feasibility"):
//! the smallest existing-transport way, with the **actual authenticated ask**
//! (`Host::join`'s ask and redeem over `Alpn::Redeem`), not a `/ping`.
//!
//! What pinned iroh holds (inventory): an `Endpoint`'s `RelayMap` takes a
//! set, `insert_relay` adds one (`iroh-1.0.2/src/endpoint.rs:982`), and a
//! relay client connection is kept to every relay in the map while it is
//! used — but only the **home** relay's connection is kept for good
//! (`iroh-1.0.2/src/socket/transports/relay/actor.rs:5`, "Unless it is for
//! the home relay, this one never exits"); a non-home connection closes
//! after 60 s idle (`:64-65`), and the relay server drops a packet to any
//! endpoint with no connected client (`iroh-relay-1.0.2/src/server/
//! clients.rs:200-210`). So the browser-compatible way is the charter's:
//! the public fallback is this computer's **home** while online, and the
//! local primary sits beside it in the map and takes over when the
//! fallback stops answering (iroh's net report re-homes, the same move
//! `tests/relay_tiers.rs` measures).
//!
//! Planted (watched failing first): the issuer relieved of the fallback
//! before the join leaves the fallback-holding joiner `IssuerAway` — so
//! the fallback, not the joiner's own reach, is what carried the ask.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs_core::link::{client_link, Link};
use svrngs_node::group::{self, Name};
use svrngs_node::iroh::RelayUrl;
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

/// A fresh node of `seed`, loopback only, reaching the network through
/// `relays` alone, its data directory kept alive beside it.
async fn node(seed: u8, relays: &[String], any: SocketAddr) -> (tempfile::TempDir, Node) {
    let dir = tempfile::tempdir().unwrap();
    let at = svrngs_node::iroh::RelayConfig {
        relay_urls: relays.to_vec(),
        n0_services: false,
        loopback_only: true,
    };
    let node = Node::start(
        dir.path().to_path_buf(),
        SigningKey::from_bytes(&[seed; 32]),
        &at,
        svrngs_node::group::seed_text,
        any,
    )
    .await
    .expect("a node starts");
    (dir, node)
}

/// The relay `node` is homed at now, if any: iroh reports one home relay
/// (`iroh-base-1.0.2/src/endpoint_addr.rs:145-150`).
fn home(node: &Node) -> Option<String> {
    node.endpoint()
        .addr()
        .relay_urls()
        .next()
        .map(ToString::to_string)
}

/// Waits up to iroh's slowest re-check for `node`'s home to be `want`.
async fn homed_at(node: &Node, want: &str) {
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(30) {
        if home(node).as_deref() == Some(want) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("not homed at {want} within 30 s; home is {:?}", home(node));
}

/// The relays a link carries, as strings.
fn carried(link: &Link) -> Vec<String> {
    link.token.relays.iter().map(ToString::to_string).collect()
}

/// **A joiner holding only the public fallback reaches the issuer through
/// it while the local primary answers, and the local primary carries the
/// join once the fallback is gone.** Sam starts homed at the fallback
/// R_pub alone, keeps an unowned group (so its build tier is held whatever
/// answers), and gets R_own added beside it — `insert_relay`, the call
/// `tiers.rs` already makes. A link minted now carries R_pub alone; Alex
/// holds R_pub alone and is let in. Control: Sam relieved of R_pub, the
/// relay stopped, leaves the fallback-only joiner `IssuerAway`, and Sam's
/// home moves to R_own; a link minted then carries R_own, and Bo holding
/// R_own alone is let in.
#[tokio::test(flavor = "multi_thread")]
async fn a_fallback_home_reaches_the_issuer_and_the_local_primary_takes_over() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_own, r_own), (_pub, r_pub)) = (relay(any).await, relay(any).await);

    // Sam is met at the public fallback alone at build time, so its home
    // is R_pub whatever iroh's probes decide.
    let (_sam_dir, sam) = node(1, &[r_pub.clone()], any).await;
    homed_at(&sam, &r_pub).await;
    // An unowned group: no relay of its own, so `Meeting::decide` holds
    // the build's tier whatever answers (svrngs_core::relay::Meeting).
    let house = group::draw(
        Name::parse("house").unwrap(),
        Name::parse("sam").unwrap(),
        SigningKey::from_bytes(&[1; 32]).verifying_key(),
    );
    sam.host()
        .keep_group(&house, None, &[])
        .expect("Sam keeps house at no relay of its own");
    // The local primary, added beside the fallback home by the pinned call
    // `tiers.rs` makes; the home stays R_pub (iroh keeps where it is while
    // the alternative is not much better).
    let own_url: RelayUrl = r_own.parse().unwrap();
    sam.endpoint()
        .insert_relay(own_url.clone(), Arc::new(own_url.clone().into()))
        .await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(
        home(&sam).as_deref(),
        Some(r_pub.as_str()),
        "the fallback is still home, with the local primary held beside it"
    );

    // A real link from the keeper: the unowned group's meeting carries the
    // build's relay, R_pub (svrngs_core::relay::Meeting::carried).
    let linked = sam.host().link(&house.id(), false, None, None).unwrap();
    let link = Link::parse(&client_link(&linked.token)).expect("the link parses");
    assert_eq!(
        carried(&link),
        vec![r_pub.clone()],
        "the link carries the public fallback alone"
    );

    // Alex holds only the fallback, as a page that cannot use an HTTP-only
    // local primary would.
    let (_alex_dir, alex) = node(2, &[r_pub.clone()], any).await;
    let joined = alex.host().join(&link, &Name::parse("alex").unwrap()).await;
    assert!(
        joined.is_ok(),
        "Alex reaches Sam through the fallback alone, and Sam's ask answers: {joined:?}"
    );

    // Control, the plant: Sam relieved of R_pub, the relay itself stopped
    // (a removed non-home connection lingers up to 60 s, iroh-1.0.2
    // actor.rs:64-65), leaves a fallback-only joiner nowhere to ask.
    let pub_url: RelayUrl = r_pub.parse().unwrap();
    sam.endpoint().remove_relay(&pub_url).await;
    _pub.shutdown().await.unwrap();
    homed_at(&sam, &r_own).await;
    let (_alex2_dir, alex2) = node(3, &[r_pub.clone()], any).await;
    let alone = alex2
        .host()
        .join(&link, &Name::parse("alex2").unwrap())
        .await;
    assert!(
        alone.is_err(),
        "with Sam off R_pub, the fallback-only joiner cannot reach it: {alone:?}"
    );

    // The local primary takes over: the keeper names it where the group
    // meets (`Host::meet`, the one owner of a group's own relays), a link
    // minted now carries R_own, and a joiner holding only R_own is let in.
    sam.host()
        .meet(&house.id(), &[own_url.clone()])
        .expect("the keeper names the local primary");
    let linked = sam.host().link(&house.id(), false, None, None).unwrap();
    let link = Link::parse(&client_link(&linked.token)).expect("the link parses");
    assert_eq!(
        carried(&link),
        vec![r_own.clone()],
        "the second link carries the local primary alone"
    );
    let (_bo_dir, bo) = node(4, &[r_own.clone()], any).await;
    let joined = bo.host().join(&link, &Name::parse("bo").unwrap()).await;
    assert!(
        joined.is_ok(),
        "with the fallback gone, the local primary carries the join: {joined:?}"
    );

    sam.endpoint().close().await;
    alex.endpoint().close().await;
    alex2.endpoint().close().await;
    bo.endpoint().close().await;
}
