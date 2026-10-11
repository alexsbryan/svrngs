//! Removal through the node, as the command asks it (U7; DESIGN §6
//! "Removal has one decider"): `POST /v1/removals` and `/v1/handovers` on
//! the record API, three nodes in one process, no relay, peers reached by
//! seeded addresses as `group_not_mesh.rs` reaches them.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::time::Duration;

use svrngs_node::iroh::{PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::removal::{Asked, Outcome, HANDOVERS_ROUTE, REMOVALS_ROUTE};
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};

const NS: &str = "trip";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from_bytes(&signing(seed).verifying_key().to_bytes()).unwrap()
}

fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
}

fn no_seed(_: &std::path::Path) -> Result<Option<String>, String> {
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

/// Sam's group: Sam's key (1) its seed, so Sam decides removals.
fn sams() -> Roster {
    Roster::new(
        [(Person::parse("sam").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    )
}

fn admit(person: &str, joining: u8) -> RailAct {
    RailAct::Admit {
        person: Person::parse(person).unwrap(),
        key: actor_of(&signing(joining)),
        invite: None,
    }
}

/// What the command asks `node` at `route`, and the node's answer.
async fn ask(node: &Node, route: &str, person: &str) -> (u16, Outcome) {
    let answer = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("http://{}{route}", node.api_addr()))
        .json(&Asked {
            group: NS.into(),
            person: person.into(),
        })
        .send()
        .await
        .unwrap();
    let status = answer.status().as_u16();
    (status, answer.json().await.unwrap())
}

fn stands(node: &Node, seed: u8) -> bool {
    node.rail()
        .register(NS)
        .unwrap()
        .members
        .contains_key(&actor_of(&signing(seed)))
}

/// Sam, Ana and Bo, each a computer holding the group, Sam's copy the one
/// that admitted the others; each reaches the others by address.
async fn three(dirs: &[tempfile::TempDir; 3]) -> [Node; 3] {
    let nodes = [
        start(1, &dirs[0]).await,
        start(2, &dirs[1]).await,
        start(3, &dirs[2]).await,
    ];
    for n in &nodes {
        n.rail().set_seed(NS, &sams()).unwrap();
    }
    let fixture = |act| svrngs_core::tests_support::Batch(vec![act]);
    for (person, k) in [("ana", 2), ("bo", 3)] {
        nodes[0]
            .rail()
            .commit(NS, &fixture(admit(person, k)))
            .unwrap()
            .unwrap();
    }
    for (i, n) in nodes.iter().enumerate() {
        for (j, m) in nodes.iter().enumerate() {
            if i != j {
                n.transport()
                    .add_known_peer(key(j as u8 + 1), loopback_sockets(m.endpoint()));
            }
        }
    }
    for n in &nodes[1..] {
        tokio::time::timeout(Duration::from_secs(30), n.round())
            .await
            .unwrap();
    }
    nodes
}

/// **Only the person who decides makes a removal count, and the removed
/// computer is told** (U7 (e)). Ana asks to remove Bo: written, Bo stands,
/// the answer names Sam as who decides. Sam asks the same: Bo is out, and
/// Bo's own computer, told the cut alone, holds Bo out too; then Bo's
/// removal of Ana is refused there, as Bo is not in the group. Red with the
/// fold's keeper check out (membership.rs, `let keeps = true`): Ana's ask
/// is the cut.
#[tokio::test]
async fn the_keeper_removes_and_the_removed_computer_is_told() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [sam, ana, bo] = three(&dirs).await;
    assert!(
        stands(&ana, 3) && stands(&bo, 3),
        "control: bo stands on every copy"
    );

    let (status, asked) = ask(&ana, REMOVALS_ROUTE, "bo").await;
    assert_eq!(status, 200);
    assert_eq!(
        asked,
        Outcome::Asked {
            decides: Some(Person::parse("sam").unwrap())
        }
    );
    assert!(stands(&ana, 3), "Ana's ask cuts nobody");

    assert_eq!(ask(&sam, REMOVALS_ROUTE, "bo").await, (200, Outcome::Out));
    assert!(!stands(&sam, 3), "Bo is out on Sam's copy");
    let told = tokio::time::timeout(Duration::from_secs(10), async {
        while stands(&bo, 3) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(told.is_ok(), "Bo's computer was told its cut within 10 s");
    assert!(bo
        .rail()
        .register(NS)
        .unwrap()
        .removed
        .contains_key(&actor_of(&signing(3))));
    assert_eq!(ask(&bo, REMOVALS_ROUTE, "ana").await, (409, Outcome::NotIn));
    assert!(stands(&sam, 2), "Ana stands");
}

/// **Only the person who decides hands deciding on** (U7 (e)). Ana's
/// hand-on is refused, naming Sam; Sam's to Ana is done, and then Ana's
/// removal of Sam is the cut. A person not in the group is said so.
#[tokio::test]
async fn only_the_keeper_hands_on() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [sam, ana, _bo] = three(&dirs).await;
    assert_eq!(
        ask(&ana, HANDOVERS_ROUTE, "ana").await,
        (
            409,
            Outcome::NotDecider {
                decides: Some(Person::parse("sam").unwrap())
            }
        )
    );
    assert_eq!(
        ask(&sam, REMOVALS_ROUTE, "dee").await,
        (409, Outcome::NotInGroup)
    );
    assert_eq!(
        ask(&sam, REMOVALS_ROUTE, "sam").await,
        (409, Outcome::Decides)
    );
    assert_eq!(
        ask(&sam, HANDOVERS_ROUTE, "ana").await,
        (200, Outcome::Handed)
    );
    tokio::time::timeout(Duration::from_secs(30), ana.round())
        .await
        .unwrap();
    assert_eq!(ask(&ana, REMOVALS_ROUTE, "sam").await, (200, Outcome::Out));
}
