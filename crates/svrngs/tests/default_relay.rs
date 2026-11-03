//! svrngs's relay, where a computer meets its groups once the build ships
//! `MeetAt::Svrngs`, n0 none of its path (relay-default (a); PLAN §3,
//! the operator, 2026-10-12; parked at the gate by relay-park, 2026-10-13,
//! the build shipping n0's until the switch, `tests/shipped.rs`). A
//! computer built as the command builds it there (`cli::reach`, which
//! `src/main.rs` calls) meets its groups at svrngs's relay, and at the ones
//! a group keeps beside it (relay fallbacks), with no address lookup —
//! n0's or any. A group made with `new --relay` keeps it in its record,
//! every computer its link lets in meets the group there too, and each
//! reaches each by key at the group's relays; no trace of any names an n0
//! host. Planted: `cli::reach` answering the command's old posture,
//! `RelayConfig::default()`'s n0 relays and lookup.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs::cli::{self, MeetAt, Verb};
use svrngs::group;
use svrngs_node::iroh::{EndpointAddr, RelayConfig, RelayUrl};
use svrngs_node::link::Link;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

/// n0's hosts: its relays' and its address lookup's domains (iroh-1.0.2
/// `defaults.rs`, `address_lookup/dns.rs`, `address_lookup/pkarr.rs`).
const N0: [&str; 2] = ["iroh.link", "n0.computer"];

/// Every line this binary traces, at debug, iroh's included.
static TRACES: Mutex<Vec<u8>> = Mutex::new(Vec::new());

struct Kept;

impl Write for Kept {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        TRACES.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn traced() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new("debug"))
            .with_writer(|| Kept)
            .with_ansi(false)
            .init();
    });
}

fn argv(s: &str) -> Vec<String> {
    std::iter::once("svrngs".to_string())
        .chain(s.split_whitespace().map(str::to_string))
        .collect()
}

/// A computer over `dir`, its key from `seed`, reaching the network as
/// `relay` says.
async fn computer(dir: &std::path::Path, seed: u8, relay: &RelayConfig) -> Node {
    Node::start(
        dir.to_path_buf(),
        SigningKey::from_bytes(&[seed; 32]),
        relay,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a computer starts")
}

/// The relay `node` is reached at, once it reaches one.
async fn home(node: &Node) -> String {
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the computer reaches its relay on 127.0.0.1");
    let url = node.endpoint().addr().relay_urls().next().cloned();
    url.expect("a home relay").to_string()
}

#[test]
fn the_command_meets_svrngs_s_relay_and_never_n0_s() {
    let ours = cli::reach(MeetAt::Svrngs);
    assert!(!ours.n0_services, "n0's relays and lookup are off");
    assert_eq!(ours.relay_urls, ["https://relay.svrnme.sh/"]);
}

/// A relay on 127.0.0.1, and its URL.
async fn relay_here() -> (Server, String) {
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    let relay = Server::spawn(config).await.expect("a relay on 127.0.0.1");
    let url = format!("http://{}/", relay.http_addr().unwrap());
    (relay, url)
}

/// The group's own relay and the build's, a second relay on 127.0.0.1
/// standing in for svrngs's so nothing leaves this computer (relay
/// fallbacks): every computer meets the group at both, iroh making either
/// its home, and n0 never.
#[tokio::test(flavor = "multi_thread")]
async fn a_group_s_own_relay_meets_every_member_and_n0_never() {
    traced();
    let ((_own, url), (_build, build)) = (relay_here().await, relay_here().await);
    let dirs = [(); 3].map(|()| tempfile::tempdir().unwrap());
    let ours = RelayConfig {
        relay_urls: vec![build.clone()],
        ..cli::reach(MeetAt::Svrngs)
    };

    let made = cli::parse(argv(&format!("new house --as Sam --relay {url}"))).unwrap();
    let sam = computer(dirs[0].path(), 41, &ours).await;
    assert!(
        sam.endpoint().address_lookup().unwrap().is_empty(),
        "Sam's computer looks no key up, at n0 or anywhere"
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
    let link = Link::parse(&printed.client).unwrap();
    let carried: Vec<String> = link.token.relays.iter().map(ToString::to_string).collect();
    assert!(carried.contains(&url), "the link carries it: {carried:?}");

    let mut people = vec![sam];
    for (dir, (seed_byte, who)) in dirs[1..].iter().zip([(42, "Alex"), (43, "Bo")]) {
        let join = cli::parse(argv(&format!("join {} --as {who}", printed.client))).unwrap();
        let node = computer(dir.path(), seed_byte, &ours).await;
        assert!(node.endpoint().address_lookup().unwrap().is_empty());
        let Verb::Join { link, person, .. } = join else {
            unreachable!("join parses as join")
        };
        let key = SigningKey::from_bytes(&[seed_byte; 32]);
        cli::join_group(&node, &key, &link, &person.unwrap())
            .await
            .unwrap_or_else(|e| panic!("{who} is let in: {e}"));
        people.push(node);
    }
    for node in &people {
        let home = home(node).await;
        assert!(
            [&url, &build].contains(&&home),
            "every computer is reached at one: {home}"
        );
        let meets = node.rail().register(&seed.id()).unwrap().meets;
        let said = meets.map(|m| m.iter().map(ToString::to_string).collect::<Vec<_>>());
        assert_eq!(
            said,
            Some(vec![url.clone()]),
            "and its record says the group meets there"
        );
    }
    // Each reaches each by its key at the group's relays.
    let (url, build): (RelayUrl, RelayUrl) = (url.parse().unwrap(), build.parse().unwrap());
    for (from, to) in [(1, 2), (2, 1), (0, 2), (1, 0)] {
        let at = EndpointAddr::new(people[to].endpoint().id())
            .with_relay_url(url.clone())
            .with_relay_url(build.clone());
        let synced = people[from].host().sync_with(&seed.id(), &at).await;
        assert!(
            synced.stop.is_none(),
            "{from} reaches {to}: {:?}",
            synced.stop
        );
    }
    let traces = String::from_utf8_lossy(&TRACES.lock().unwrap()).into_owned();
    assert!(
        traces.contains(url.as_str()),
        "the traces are read: {}",
        traces.len()
    );
    for host in N0 {
        let seen: Vec<_> = traces
            .lines()
            .filter(|l| l.contains(host))
            .take(3)
            .collect();
        assert!(seen.is_empty(), "no computer names n0's {host}: {seen:#?}");
    }
    for node in &people {
        node.endpoint().close().await;
    }
}
