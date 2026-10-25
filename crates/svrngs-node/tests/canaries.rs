//! The canaries (DESIGN §5): small apps in `apps/`, run against the infra on
//! every check, each using only what DESIGN §3 says an app gets, so a change
//! that serves one app and breaks another fails the build. This file stands
//! the computers up — nodes in one process, no relay, peers reached by
//! seeded addresses, as `record_api.rs` does — and runs the app's own canary
//! under Node against their loopback record APIs. A canary that cannot run
//! fails: one that never ran has not passed (AGENTS.md 6).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::path::Path;
use std::time::Duration;

use svrngs_node::iroh::{PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};

const GROUP: &str = "house";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from_bytes(&signing(seed).verifying_key().to_bytes()).unwrap()
}

/// No relay and no address lookup: peers are reached by seeded addresses.
fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
}

/// No link is asked about here.
fn no_seed(_: &Path) -> Result<Option<String>, String> {
    Ok(None)
}

async fn start(seed: u8, dir: &tempfile::TempDir) -> Node {
    Node::start(
        dir.path().to_path_buf(),
        signing(seed),
        &no_relay(),
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// The group's seed: Sam's key alone (DESIGN §6).
fn founded_by_sam() -> Roster {
    Roster::new(
        [(Person::parse("sam").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    )
}

/// Sam's computer lets `person`'s key in to `group`, as a test admits a
/// member by hand (PLAN §4, U1).
fn admit(sam: &Node, group: &str, person: &str, joining: u8) {
    let fixture = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse(person).unwrap(),
            key: actor_of(&signing(joining)),
            invite: None,
        },
    };
    sam.rail().commit(group, &fixture).unwrap().unwrap();
}

/// The people of the group, each by the seed of their own computer's key.
const PEOPLE: [(&str, u8); 4] = [("sam", 1), ("alex", 2), ("bo", 3), ("cy", 4)];

/// One computer per person, one group: Sam founds it and lets in Alex, Bo
/// and Cy, each of whom writes on their own computer, since the record API
/// writes only as its own person (DESIGN §3, `not_yours`). Each holds the
/// seed, and one round of its own brings it the rest of the register; each
/// is told every other's loopback addresses, so a round an append wakes on
/// any reaches the rest.
async fn computers(dirs: &[tempfile::TempDir; 4]) -> Vec<Node> {
    let mut nodes = Vec::new();
    for ((_, seed), dir) in PEOPLE.iter().zip(dirs) {
        nodes.push(start(*seed, dir).await);
    }
    nodes[0].rail().set_seed(GROUP, &founded_by_sam()).unwrap();
    for (person, seed) in &PEOPLE[1..] {
        admit(&nodes[0], GROUP, person, *seed);
    }
    for (n, (_, seed)) in nodes.iter().zip(PEOPLE) {
        if seed != 1 {
            n.rail().set_seed(GROUP, &founded_by_sam()).unwrap();
        }
        for (other, (_, theirs)) in nodes.iter().zip(PEOPLE) {
            if theirs != seed {
                n.transport()
                    .add_known_peer(key(theirs), loopback_sockets(other.endpoint()));
            }
        }
    }
    for n in &nodes[1..] {
        tokio::time::timeout(Duration::from_secs(30), n.round())
            .await
            .expect("a round finishes");
        assert_eq!(
            nodes[0].rail().digest(GROUP).unwrap(),
            n.rail().digest(GROUP).unwrap(),
            "control: each computer holds Sam's register before any app writes"
        );
    }
    nodes
}

/// Each computer's loopback record API, in `PEOPLE`'s order.
fn records(nodes: &[Node]) -> Vec<String> {
    nodes
        .iter()
        .map(|n| format!("http://{}", n.api_addr()))
        .collect()
}

/// Run `apps/<app>/canary.mjs` under Node with `args`, and fail with what
/// it said unless it exits 0.
async fn canary(app: &str, args: &[String]) {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps")
        .join(app)
        .join("canary.mjs");
    let out = tokio::process::Command::new("node")
        .arg(&script)
        .args(args)
        .output()
        .await
        .unwrap_or_else(|e| panic!("the {app} canary never ran: `node` did not start ({e})"));
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    println!("{said}");
    assert!(
        out.status.success(),
        "the {app} canary failed ({}):\n{said}",
        out.status
    );
}

/// **Expenses** (DESIGN §5, "The canaries"; the u3-7 row): no server, all state in the
/// record. Apps on each person's own computer write for that person, a
/// correction voids, and every copy folds to the same balances, the ones the
/// acts say (`apps/expenses/canary.mjs`).
#[tokio::test]
async fn expenses() {
    let dirs = [(); 4].map(|_| tempfile::tempdir().unwrap());
    let nodes = computers(&dirs).await;
    let args: Vec<String> = [GROUP.to_string()]
        .into_iter()
        .chain(records(&nodes))
        .collect();
    canary("expenses", &args).await;
    for n in &nodes {
        n.endpoint().close().await;
    }
}

/// **Events** (events-app (c)): the same four computers, and a second group
/// on Sam's alone, Sam and Di in it. Apps on each write for their own person,
/// a correction voids on every copy, another's correction counts for
/// nothing, and every copy folds to the same event, who is coming and who
/// came; the second group's record holds the event and none of the first's
/// answers (`apps/events/canary.mjs`).
#[tokio::test]
async fn events() {
    const SECOND: &str = "dweb";
    let dirs = [(); 4].map(|_| tempfile::tempdir().unwrap());
    let nodes = computers(&dirs).await;
    nodes[0].rail().set_seed(SECOND, &founded_by_sam()).unwrap();
    admit(&nodes[0], SECOND, "di", 5);
    let args: Vec<String> = [GROUP.to_string(), SECOND.to_string()]
        .into_iter()
        .chain(records(&nodes))
        .collect();
    canary("events", &args).await;
    for n in &nodes {
        n.endpoint().close().await;
    }
}
