//! The canaries (DESIGN §8): small apps in `apps/`, run against the infra on
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

/// Sam's computer lets `person`'s key in, as a test admits a member by
/// hand (DESIGN §8, U1).
fn admit(sam: &Node, person: &str, joining: u8) {
    let fixture = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse(person).unwrap(),
            key: actor_of(&signing(joining)),
            invite: None,
        },
        on_behalf_of: None,
    };
    sam.rail().commit(GROUP, &fixture).unwrap().unwrap();
}

/// Sam's computer and Alex's, one group: Sam founds it and lets in Alex,
/// and Bo and Cy, whose keys are on no computer here; Alex's computer holds
/// the seed, and one round of its own brings it the rest of the register.
/// Each is told the other's loopback addresses, so a round an append wakes
/// on either reaches the other.
async fn two_computers(ds: &tempfile::TempDir, da: &tempfile::TempDir) -> (Node, Node) {
    let (sam, alex) = (start(1, ds).await, start(2, da).await);
    sam.rail().set_seed(GROUP, &founded_by_sam()).unwrap();
    for (person, seed) in [("alex", 2), ("bo", 3), ("cy", 4)] {
        admit(&sam, person, seed);
    }
    alex.rail().set_seed(GROUP, &founded_by_sam()).unwrap();
    sam.transport()
        .add_known_peer(key(2), loopback_sockets(alex.endpoint()));
    alex.transport()
        .add_known_peer(key(1), loopback_sockets(sam.endpoint()));
    tokio::time::timeout(Duration::from_secs(30), alex.round())
        .await
        .expect("Alex's round finishes");
    assert_eq!(
        sam.rail().digest(GROUP).unwrap(),
        alex.rail().digest(GROUP).unwrap(),
        "control: Alex's computer holds Sam's register before any app writes"
    );
    (sam, alex)
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

/// **Expenses** (DESIGN §8, the u3-7 row): no server, all state in the
/// record. Apps on Sam's computer and Alex's write — for their own person,
/// and for Bo and Cy by name — a correction voids, and both copies fold to
/// the same balances, the ones the acts say (`apps/expenses/canary.mjs`).
#[tokio::test]
async fn expenses() {
    let (ds, da) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (sam, alex) = two_computers(&ds, &da).await;
    let at = |n: &Node| format!("http://{}", n.api_addr());
    canary("expenses", &[GROUP.to_string(), at(&sam), at(&alex)]).await;
    for n in [&sam, &alex] {
        n.endpoint().close().await;
    }
}
