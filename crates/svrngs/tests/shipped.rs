//! Where the command as shipped meets its groups: at n0's relays and
//! address lookup, chosen, until the operator flips the switch to svrngs's
//! relay (relay-park (b); DESIGN §9, §10, the operator, 2026-10-13). A
//! group that keeps no relay is met at n0's; a group that keeps its own
//! (`new --relay`) is met there, and once the computer holds a group that
//! keeps none, n0's relays are named beside it, so that group still
//! reaches it — a non-empty `relay_urls` replaces n0's set
//! (`svrngs_node::guest::RelayConfig`). svrngs's own posture is
//! `tests/default_relay.rs`'s, a binary apart: its traces must name no n0
//! host, and this one's computers ask n0. Planted: `cli::SHIPPED` left at
//! `MeetAt::Svrngs`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs::cli::{self, Verb};
use svrngs::group;
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

fn argv(s: &str) -> Vec<String> {
    std::iter::once("svrngs".to_string())
        .chain(s.split_whitespace().map(str::to_string))
        .collect()
}

/// n0's relays as the command writes them.
fn n0() -> Vec<String> {
    let urls: Vec<RelayUrl> = svrngs_node::relay::n0_relays().urls();
    urls.iter().map(ToString::to_string).collect()
}

#[test]
fn the_command_as_shipped_meets_a_group_that_keeps_no_relay_at_n0_s() {
    let dir = tempfile::tempdir().unwrap();
    let shipped = cli::reach(dir.path(), &Verb::Run, cli::SHIPPED).unwrap();
    assert!(shipped.n0_services, "n0's lookup is on");
    assert_eq!(shipped.relay_urls, n0(), "at n0's relays");
    let joined = cli::parse(argv("new house --as Sam")).unwrap();
    let theirs = cli::reach(dir.path(), &joined, cli::SHIPPED).unwrap();
    assert_eq!(theirs.relay_urls, n0(), "a new group with no relay too");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_group_that_keeps_its_own_relay_is_met_there_with_n0_s_beside_it() {
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    let relay = Server::spawn(config).await.expect("a relay on 127.0.0.1");
    let url = format!("http://{}/", relay.http_addr().unwrap());
    let dir = tempfile::tempdir().unwrap();

    let made = cli::parse(argv(&format!("new house --as Sam --relay {url}"))).unwrap();
    let ours: RelayConfig = cli::reach(dir.path(), &made, cli::SHIPPED).unwrap();
    assert!(ours.n0_services, "n0's lookup is on");
    assert_eq!(ours.relay_urls, [url.clone()], "house's relay alone");
    let sam = Node::start(
        dir.path().to_path_buf(),
        SigningKey::from_bytes(&[51; 32]),
        &ours,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a computer starts");
    assert!(
        !sam.endpoint().address_lookup().unwrap().is_empty(),
        "Sam's computer asks n0's lookup"
    );
    let Verb::New {
        name,
        person,
        relay: kept,
        ..
    } = made
    else {
        unreachable!("new parses as new")
    };
    cli::new_group(&sam, name, person.unwrap(), None, kept, &ours)
        .await
        .expect("Sam makes house");
    let home = sam.endpoint().addr().relay_urls().next().cloned();
    assert_eq!(
        home.map(|h| h.to_string()).as_deref(),
        Some(url.as_str()),
        "Sam's computer is met at house's relay"
    );
    let next = cli::reach(dir.path(), &Verb::Run, cli::SHIPPED).unwrap();
    assert_eq!(
        next.relay_urls,
        [url.clone()],
        "and meets house there again"
    );

    let work = cli::parse(argv("new work --as Sam")).unwrap();
    let Verb::New { name, person, .. } = work else {
        unreachable!("new parses as new")
    };
    cli::new_group(&sam, name, person.unwrap(), None, None, &ours)
        .await
        .expect("Sam makes work, which keeps no relay");
    let both = cli::reach(dir.path(), &Verb::Run, cli::SHIPPED).unwrap();
    let beside: Vec<String> = std::iter::once(url).chain(n0()).collect();
    assert!(both.n0_services, "n0's lookup is on");
    assert_eq!(
        both.relay_urls, beside,
        "house at its own relay, work at n0's, beside it"
    );
    sam.endpoint().close().await;
}
