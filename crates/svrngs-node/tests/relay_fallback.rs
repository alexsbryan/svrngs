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
mod common;
#[path = "common/on_wire.rs"]
mod on_wire;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs_core::link::{client_link, Link};
use svrngs_core::tests_support::{record, Batch};
use svrngs_node::group::{self, Name};
use svrngs_node::iroh::RelayUrl;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, SigningKey};

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

/// Waits for `node`'s home to be `want`, up to the asks that leave the
/// fallback and one more, and iroh's 10 s report.
async fn homed_at(node: &Node, want: &str) {
    let (t, tiers) = (Instant::now(), svrngs_node::tiers::LEFT_AFTER + 1);
    while t.elapsed() < svrngs_node::tiers::ASKED_EVERY * tiers + Duration::from_secs(10) {
        if home(node).as_deref() == Some(want) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!(
        "not homed at {want} in {:?}; home is {:?}",
        t.elapsed(),
        home(node)
    );
}

/// Whether `node`'s journal holds a record whose payload says `what`.
fn holds(node: &Node, group: &str, what: &str) -> bool {
    let rail = node.rail();
    let journal = rail.group_dir(group).unwrap().join("ring_oplog.jsonl");
    std::fs::read_to_string(journal)
        .unwrap_or_default()
        .contains(what)
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
/// home moves to R_own; a link minted then carries R_own and R_pub beside
/// it, and Bo holding R_own alone is let in.
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
        vec![r_own.clone(), r_pub.clone()],
        "the second link carries the local primary, and the fallback beside it (review F2)"
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

/// **A fresh contribution syncs after the public reach is cut** (the
/// relay-fallback charter: a native computer's local recovery when the
/// internet disappears). Ada's group is met at A, its own relay; her
/// build's fallback is B, where she and Bo are homed while it answers. B is
/// stopped: both move to A (the move `relay_tiers` measures), and a record
/// Ada writes then reaches Bo through A. Both nodes are loopback-only, so
/// every dial is relay-carried; neither has connected before the cut, so
/// no direct path is cached. Planted, watched failing: `Meeting::decide`
/// keeping the build's fallback when it does not answer, so neither home
/// moves and nothing is reached (red: not homed at A within 30 s).
#[tokio::test(flavor = "multi_thread")]
async fn a_fresh_contribution_syncs_after_the_public_reach_is_cut() {
    cut_and_contribute().await;
}

async fn cut_and_contribute() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_own, a), (build, b)) = (relay(any).await, relay(any).await);
    let at_a: svrngs_core::relay::Url = a.parse().unwrap();
    let (_ada_dir, ada) = node(5, &[b.clone()], any).await;
    let house = group::draw(
        Name::parse("house").unwrap(),
        Name::parse("ada").unwrap(),
        SigningKey::from_bytes(&[5; 32]).verifying_key(),
    );
    ada.host()
        .keep_group(&house, None, &[at_a.clone()])
        .expect("Ada keeps house, met at A");
    let (_bo_dir, bo) = node(6, &[b.clone()], any).await;
    bo.host()
        .keep_group(&house, None, &[at_a.clone()])
        .expect("Bo keeps house, met at A");
    // Bo let in by a link Ada's computer mints: the group is sources-v1.
    let (ns, rail) = (house.id(), ada.rail());
    svrngs_core::tests_support::let_in(
        (&ns, &actor_of(rail.signer())),
        (
            &SigningKey::from_bytes(&[6; 32]),
            "bo",
            svrngs_node::Device::Computer,
        ),
        |mint| {
            rail.commit(&ns, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(rail.commit(&ns, redeem).unwrap().unwrap()),
    );
    homed_at(&ada, &b).await;
    homed_at(&bo, &b).await;

    // The public reach is cut, and both computers move to the group's own.
    build.shutdown().await.unwrap();
    homed_at(&ada, &a).await;
    homed_at(&bo, &a).await;

    // A fresh contribution reaches Bo through A.
    ada.rail()
        .commit(&house.id(), &Batch(vec![record("after the cut")]))
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(30), ada.round())
        .await
        .expect("Ada's round finishes");
    assert!(
        holds(&bo, &house.id(), "after the cut"),
        "the group's own relay carried a fresh contribution with the public reach cut"
    );

    ada.endpoint().close().await;
    bo.endpoint().close().await;
}

/// **A group that knows only the public relay pins nobody to it**
/// (KEEP_GOING_AT_HOME 8): the same cut, with Ada and Bo each in a second
/// group naming no relay of its own. Both still move to A, and the record
/// still arrives. On one wire (common/on_wire.rs): each relay the wire's
/// (`Wire::relay`), the clock paused, no relay process or endpoint.
/// Planted: `Meeting::decide` keeping the fallback unanswered, and
/// `Meeting::join` emptying the own tier for a group naming none.
#[tokio::test(start_paused = true)]
async fn a_public_only_group_does_not_pin_a_computer_to_a_relay_that_is_gone() {
    use svrngs_node::node::Relaying;
    let (a, b) = ("http://127.0.0.1:1/", "http://127.0.0.1:2/");
    on_wire::relay(a, Relaying::Up);
    on_wire::relay(b, Relaying::Up);
    let at_a: svrngs_core::relay::Url = a.parse().unwrap();
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let started = |seed: u8| async move {
        let dir = tempfile::tempdir().unwrap();
        let key = SigningKey::from_bytes(&[seed; 32]);
        let node = on_wire::relayed(key, dir.path(), b, &[], every).await;
        (dir, node)
    };
    let (_ada_dir, ada) = started(5).await;
    let house = group::draw(
        Name::parse("house").unwrap(),
        Name::parse("ada").unwrap(),
        SigningKey::from_bytes(&[5; 32]).verifying_key(),
    );
    ada.host()
        .keep_group(&house, None, &[at_a.clone()])
        .expect("Ada keeps house, met at A");
    let (_bo_dir, bo) = started(6).await;
    bo.host()
        .keep_group(&house, None, &[at_a.clone()])
        .expect("Bo keeps house, met at A");
    let (ns, rail) = (house.id(), ada.rail());
    svrngs_core::tests_support::let_in(
        (&ns, &actor_of(rail.signer())),
        (
            &SigningKey::from_bytes(&[6; 32]),
            "bo",
            svrngs_node::Device::Computer,
        ),
        |mint| {
            rail.commit(&ns, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(rail.commit(&ns, redeem).unwrap().unwrap()),
    );
    for (host, seed) in [(ada.host(), 5), (bo.host(), 6)] {
        let away = group::draw(
            Name::parse("away").unwrap(),
            Name::parse("ada").unwrap(),
            SigningKey::from_bytes(&[seed; 32]).verifying_key(),
        );
        host.keep_group(&away, None, &[])
            .expect("a public-only group");
    }
    homed_on(&ada, b).await;
    homed_on(&bo, b).await;

    // The public reach is cut, and both computers move to the group's own.
    on_wire::relay(b, Relaying::Down);
    homed_on(&ada, a).await;
    homed_on(&bo, a).await;

    ada.rail()
        .commit(&house.id(), &Batch(vec![record("after the cut")]))
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(30), ada.round())
        .await
        .expect("Ada's round finishes");
    assert!(
        holds(&bo, &house.id(), "after the cut"),
        "a fresh contribution reached Bo with the public reach cut"
    );
}

/// Up to 60 s on the test's clock, until the relay `node`'s address names
/// is `want`.
async fn homed_on(node: &Node, want: &str) {
    for _ in 0..600 {
        let home = node.net().addr().relay_urls().next().map(|r| r.to_string());
        if home.as_deref() == Some(want) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("not homed at {want} within 60 s");
}
