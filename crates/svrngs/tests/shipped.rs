//! Where the command as shipped meets its groups: at n0's relays and
//! address lookup, chosen, until the operator flips the switch to svrngs's
//! relay (relay-park (b); PLAN §2, §3, the operator, 2026-10-13). A
//! group that keeps no relay is met at n0's; a group that keeps its own
//! (`new --relay`) is met there and at n0's beside it, its fallback
//! (relay fallbacks). svrngs's own posture is
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

fn name_of(s: &str) -> group::Name {
    group::Name::parse(s).unwrap()
}

/// n0's relays as the command writes them.
fn n0() -> Vec<String> {
    let urls: Vec<RelayUrl> = svrngs_node::relay::n0_relays().urls();
    urls.iter().map(ToString::to_string).collect()
}

#[test]
fn the_command_as_shipped_meets_a_group_that_keeps_no_relay_at_n0_s() {
    let shipped = cli::reach(cli::SHIPPED);
    assert!(shipped.n0_services, "n0's lookup is on");
    assert_eq!(shipped.relay_urls, n0(), "at n0's relays");
}

/// A group that keeps its own relay is met there and at the build's beside
/// it, a second relay on 127.0.0.1 standing in for n0's so the test reaches
/// none off this computer (relay fallbacks); a group that keeps none, at
/// the build's alone.
#[tokio::test(flavor = "multi_thread")]
async fn a_group_that_keeps_its_own_relay_is_met_there_with_n0_s_beside_it() {
    let relay_here = || async {
        let mut config = ServerConfig::default();
        config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
        let relay = Server::spawn(config).await.expect("a relay on 127.0.0.1");
        let url = format!("http://{}/", relay.http_addr().unwrap());
        (relay, url)
    };
    let ((_own, url), (_build, build)) = (relay_here().await, relay_here().await);
    let dir = tempfile::tempdir().unwrap();

    let made = cli::parse(argv(&format!("new house --as Sam --relay {url}"))).unwrap();
    let ours = RelayConfig {
        relay_urls: vec![build.clone()],
        ..cli::reach(cli::SHIPPED)
    };
    assert!(ours.n0_services, "n0's lookup is on");
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
        relays: kept,
        ..
    } = made
    else {
        unreachable!("new parses as new")
    };
    let (seed, printed) = cli::new_group(&sam, name, person.unwrap(), None, &kept, &ours)
        .await
        .expect("Sam makes house");
    let home = sam.endpoint().addr().relay_urls().next().cloned();
    let home = home
        .map(|h| h.to_string())
        .expect("Sam's computer is met at one");
    assert!([&url, &build].contains(&&home), "{home}");
    let link = svrngs_node::link::Link::parse(&printed.client).unwrap();
    let carried: Vec<String> = link.token.relays.iter().map(ToString::to_string).collect();
    assert!(
        carried.contains(&url),
        "its link, lookup on, says so: {carried:?}"
    );
    let said = |id: &str| {
        let meets = sam.rail().register(id).unwrap().meets;
        meets.map(|m| m.iter().map(ToString::to_string).collect::<Vec<_>>())
    };
    assert_eq!(
        said(&seed.id()),
        Some(vec![url.clone()]),
        "house names its own"
    );

    // A group that keeps no relay names none, and is met at the build's.
    let founder = ed25519_dalek::VerifyingKey::from_bytes(sam.endpoint().id().as_bytes()).unwrap();
    let work = group::draw(name_of("work"), name_of("Sam"), founder);
    sam.host()
        .keep_group(&work, None, &[])
        .expect("Sam keeps work, which keeps no relay");
    assert_eq!(said(&work.id()), Some(vec![]), "work names none");
    sam.endpoint().close().await;
}
