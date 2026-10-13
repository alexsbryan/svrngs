//! svrngs's relay the command's default, n0 none of its path
//! (relay-default (a); DESIGN §10, the operator, 2026-10-12). A computer
//! built as the command builds it (`cli::reach`, which `src/main.rs` calls)
//! meets its groups at svrngs's relay, or at the one a group keeps, with no
//! address lookup — n0's or any. A group made with `new --relay` keeps it,
//! every computer its link lets in meets the group there too, and each
//! reaches each by key and that relay alone; no trace of any names an n0
//! host. Planted: `cli::reach` answering the command's old posture,
//! `RelayConfig::default()`'s n0 relays and lookup.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs::cli::{self, Verb};
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
    let dir = tempfile::tempdir().unwrap();
    let ours = cli::reach(dir.path(), &Verb::Run).unwrap();
    assert!(!ours.n0_services, "n0's relays and lookup are off");
    assert_eq!(ours.relay_urls, ["https://relay.svrnme.sh/"]);
    let asked = cli::parse(argv("new house --relay relay.example.org")).unwrap();
    let theirs = cli::reach(dir.path(), &asked).unwrap();
    assert!(!theirs.n0_services, "n0's relays and lookup are off");
    assert_eq!(theirs.relay_urls, ["https://relay.example.org/"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_group_s_own_relay_meets_every_member_and_n0_never() {
    traced();
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    let relay = Server::spawn(config).await.expect("a relay on 127.0.0.1");
    let url = format!("http://{}/", relay.http_addr().unwrap());
    let dirs = [(); 3].map(|()| tempfile::tempdir().unwrap());

    let made = cli::parse(argv(&format!("new house --as Sam --relay {url}"))).unwrap();
    let ours = cli::reach(dirs[0].path(), &made).unwrap();
    let sam = computer(dirs[0].path(), 41, &ours).await;
    assert!(
        sam.endpoint().address_lookup().unwrap().is_empty(),
        "Sam's computer looks no key up, at n0 or anywhere"
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
    let (seed, printed) = cli::new_group(&sam, name, person.unwrap(), None, kept, &ours)
        .await
        .expect("Sam makes house");
    let link = Link::parse(&printed.client).unwrap();
    let carried = link.token.relay.map(|r| RelayUrl::from(r).to_string());
    assert_eq!(
        carried.as_deref(),
        Some(url.as_str()),
        "the link carries it"
    );

    let mut people = vec![sam];
    for (dir, (seed_byte, who)) in dirs[1..].iter().zip([(42, "Alex"), (43, "Bo")]) {
        let join = cli::parse(argv(&format!("join {} --as {who}", printed.client))).unwrap();
        let theirs = cli::reach(dir.path(), &join).unwrap();
        assert_eq!(theirs.relay_urls, [url.clone()], "{who} meets it there");
        let node = computer(dir.path(), seed_byte, &theirs).await;
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
    for (node, dir) in people.iter().zip(&dirs) {
        assert_eq!(home(node).await, url, "every computer is reached there");
        let next = cli::reach(dir.path(), &Verb::Run).unwrap();
        assert_eq!(next.relay_urls, [url.clone()], "and meets it there again");
    }
    // Each reaches each by its key and the group's relay alone: Alex hears
    // of Bo from Bo first, so Bo stands in Alex's copy when Bo dials.
    let url: RelayUrl = url.parse().unwrap();
    for (from, to) in [(1, 2), (2, 1), (0, 2), (1, 0)] {
        let at = EndpointAddr::new(people[to].endpoint().id()).with_relay_url(url.clone());
        let synced = people[from].sync_with(&seed.id(), &at).await;
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
