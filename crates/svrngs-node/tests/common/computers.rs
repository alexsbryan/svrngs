//! Ana's, Ben's and Cy's computers in one process, no relay, each reached
//! by the addresses seeded in the others' lookup: one copy, where
//! folders.rs and carried.rs each wrote it out (REVIEW-audit-v1ui-auto-2).
//! Only those two load it (`#[path]`); both meet their computers on a wire
//! (on_wire.rs) but folders.rs's Q test, which starts them here.

use std::path::Path;
use std::time::Duration;
use svrngs_node::iroh::Net as _;

use svrngs_node::iroh::{EndpointAddr, MemoryLookup, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::seed::Seed;
use svrngs_node::{Name, SigningKey};

use crate::common::loopback_sockets;

pub fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

#[allow(
    dead_code,
    reason = "folders.rs's Q test alone starts its computers over QUIC"
)]
fn seed_file(dir: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(dir.join("seed.json")) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[allow(
    dead_code,
    reason = "folders.rs's Q test alone starts its computers over QUIC"
)]
pub async fn start(seed: u8, dir: &Path) -> Node {
    let relay = RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: true,
    };
    Node::start(
        dir.to_path_buf(),
        signing(seed),
        &relay,
        seed_file,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// The seed of a group Ana founds, named `name`, salted by `salt`.
pub fn seed(name: &str, salt: u8) -> Seed {
    Seed::from_parts(
        Name::parse(name).unwrap(),
        Name::parse("ana").unwrap(),
        signing(1).verifying_key(),
        [salt; 16],
    )
}

/// A group Ana founds, named `name`, salted by `salt`, with each of
/// `others` (Ben 2, Cy 3) admitted: its id, on every node named.
pub fn found(name: &str, salt: u8, ana: &Node, others: &[(&Node, u8)]) -> String {
    let seed = seed(name, salt);
    let id = seed.id();
    for node in std::iter::once(ana).chain(others.iter().map(|(n, _)| *n)) {
        node.rail().set_seed(&id, &seed.roster()).unwrap();
        let dir = node.rail().group_dir(&id).unwrap();
        std::fs::write(dir.join("seed.json"), seed.to_text()).unwrap();
    }
    for &(_, n) in others {
        let person = ["", "", "ben", "cy"][n as usize];
        svrngs_core::tests_support::let_in_by(&id, &signing(n), person, |act| {
            let act = svrngs_core::tests_support::Fixture { act };
            ana.rail().commit(&id, &act).unwrap().unwrap().lines()[0]
                .id
                .clone()
        });
    }
    id
}

/// Each of `nodes` told where each listens, for the round and for a dial
/// by key alike.
#[allow(
    dead_code,
    reason = "folders.rs's Q test alone starts its computers over QUIC"
)]
pub fn known(nodes: &[&Node]) {
    for who in nodes {
        let book = MemoryLookup::new();
        for other in nodes {
            let mut addr = EndpointAddr::new(other.endpoint().id());
            for a in loopback_sockets(other.endpoint()) {
                addr = addr.with_ip_addr(a);
            }
            book.add_endpoint_info(addr);
            who.transport()
                .add_known_peer(other.endpoint().id(), loopback_sockets(other.endpoint()));
        }
        who.endpoint().address_lookup().unwrap().add(book);
    }
}

pub async fn round(node: &Node) {
    tokio::time::timeout(Duration::from_secs(30), node.round())
        .await
        .expect("a round finishes");
}

/// Up to 30 s, rounds run on `nodes`: whether `done` came true.
#[allow(dead_code, reason = "kept.rs's restart waits on its nodes' own rounds")]
pub async fn until_on(nodes: &[&Node], done: impl Fn() -> bool) -> bool {
    for _ in 0..30 {
        if done() {
            return true;
        }
        for node in nodes {
            round(node).await;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    done()
}

/// A client of a real port: carried.rs's llama-server; a node's API is
/// asked in-process (on_wire.rs `http`).
#[allow(dead_code, reason = "carried.rs alone asks a real port")]
pub fn http() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}
