//! The record over loopback (DESIGN §3, §8 U3's inventory, the u3-2 row): an
//! app on a member's computer appends to its group, reads the log and the
//! people, on a listener bound to 127.0.0.1, and nothing else reaches it.
//! Nodes in one process, no relay; peers are reached by seeded addresses,
//! as `group_not_mesh.rs` does.

mod common;
use common::loopback_sockets;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use svrngs_node::iroh::{PublicKey, RelayConfig};
use svrngs_node::node::{Node, NodeError};
use svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
use svrngs_node::{actor_of, Ed25519Verifier, Person, RailAct, Roster, SigningKey};

const NS: &str = "house";

/// How long B is watched for an act after A's app appends it: a sixth of
/// the round's interval, so only a round the append itself woke can carry
/// the act inside it.
const ONE_ROUND: Duration = Duration::from_secs(10);

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// No relay and no address lookup: peers are reached by seeded addresses.
fn no_relay() -> RelayConfig {
    RelayConfig::from_parts(Vec::new(), Some("none")).unwrap()
}

/// No link is asked about here.
fn no_seed(_: &std::path::Path) -> Result<Option<String>, String> {
    Ok(None)
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from_bytes(&signing(seed).verifying_key().to_bytes()).unwrap()
}

async fn start(seed: u8, dir: &tempfile::TempDir) -> Node {
    start_on(seed, dir, "127.0.0.1:0".parse().unwrap())
        .await
        .expect("a node starts")
}

/// A group's seed: its founder's key alone (DESIGN §6).
fn founded_by(founder: u8) -> Roster {
    Roster::new(
        [(
            Person::parse("ada").unwrap(),
            vec![actor_of(&signing(founder))],
        )]
        .into_iter()
        .collect(),
    )
}

/// `node` holds `ns`, seeded with `founder`'s key.
fn hold(node: &Node, ns: &str, founder: u8) {
    node.rail()
        .journal(ns)
        .unwrap()
        .set_roster(&founded_by(founder))
        .unwrap();
}

/// The founder admits `joining` as `person` through its door, as a test
/// admits a member by hand (DESIGN §8, U1).
fn admit(node: &Node, founder: u8, person: &str, joining: u8) {
    node.rail()
        .journal(NS)
        .unwrap()
        .append(
            RailAct::Admit {
                person: Person::parse(person).unwrap(),
                key: actor_of(&signing(joining)),
                invite: None,
            },
            node.rail().signer(),
            &founded_by(founder),
            None,
            &Ed25519Verifier,
        )
        .unwrap();
}

/// An app's HTTP client: loopback, never a proxy.
fn app() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// Whether `node` holds the act `id` in `ns`.
fn holds(node: &Node, ns: &str, id: &str) -> bool {
    let (ops, _) = node.rail().journal(ns).unwrap().read().unwrap();
    ops.iter().any(|o| o.id.to_string() == id)
}

async fn start_on(seed: u8, dir: &tempfile::TempDir, api: SocketAddr) -> Result<Node, NodeError> {
    Node::start(
        dir.path().to_path_buf(),
        signing(seed),
        &no_relay(),
        no_seed,
        api,
    )
    .await
}

/// **The API is served on loopback alone** — upstream's bind guard
/// (`api.rs:167`, `check_loopback`): asked to listen on an address that is
/// not loopback, the node refuses to start, naming the address, and binds
/// nothing. Asked for loopback, it serves there.
#[tokio::test]
async fn a_listener_not_on_loopback_is_refused_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let wide: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let refused = match start_on(1, &dir, wide).await {
        Ok(_) => panic!("a node must not serve the record on {wide}"),
        Err(e) => e.to_string(),
    };
    assert!(
        refused.contains("0.0.0.0:0") && refused.contains("not a loopback address"),
        "{refused}"
    );

    let node = start_on(1, &dir, "127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    assert!(node.api_addr().ip().is_loopback(), "{}", node.api_addr());
    assert_ne!(node.api_addr().port(), 0, "the port bound is answered");
    node.endpoint().close().await;
}

/// **An app's append goes at once** (the u3-2 row): A founds the group and
/// admits B; an app on A's computer appends a record to A's group over
/// loopback; the round, woken by the append and nothing else, carries it to
/// B inside [`ONE_ROUND`], a sixth of the interval — the test neither
/// sleeps for an interval nor runs a round. Both nodes' start rounds run
/// over no group first, so none is in flight when the app writes. The
/// failing input: the nudge the node builds dropped, as it was before
/// u3-2 (B does not see the act within one round's wait).
#[tokio::test]
async fn an_app_appends_on_loopback_and_the_woken_round_carries_it_to_b() {
    assert!(ONE_ROUND < DEFAULT_RING_SYNC_INTERVAL);
    let (da, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (start(1, &da).await, start(2, &db).await);
    // On this one-thread runtime, a yield runs every ready task once: each
    // node's start round finds no group and returns before any await, and
    // its loop then waits on the interval or the nudge.
    tokio::task::yield_now().await;
    hold(&a, NS, 1);
    admit(&a, 1, "bo", 2);
    hold(&b, NS, 1);
    a.transport()
        .add_known_peer(key(2), loopback_sockets(b.endpoint()));

    let answer = app()
        .post(format!(
            "http://{}/v1/rail/append?namespace={NS}",
            a.api_addr()
        ))
        .json(&serde_json::json!({ "op": "record", "payload": { "n": 1 } }))
        .send()
        .await
        .expect("A's record answers on loopback");
    assert_eq!(answer.status(), reqwest::StatusCode::OK);
    let out: serde_json::Value = answer.json().await.unwrap();
    let id = out["id"].as_str().unwrap().to_string();
    assert_eq!(out["actor"], actor_of(&signing(1)), "A's computer signs");
    assert!(holds(&a, NS, &id), "control: A holds the act");
    assert!(!holds(&b, NS, &id), "control: B does not yet");

    let since = Instant::now();
    while !holds(&b, NS, &id) {
        assert!(
            since.elapsed() < ONE_ROUND,
            "B did not see the app's act within {ONE_ROUND:?}: nothing woke the round"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    for n in [&a, &b] {
        n.endpoint().close().await;
    }
}

/// **A group this computer does not hold is refused** (the u3-2 row): an
/// app's append naming one is answered 404 `not_held`, nothing is opened on
/// disk, and the group A does hold is untouched. The failing input: the
/// register's gate skipped in `journal_of` (crates/svrngs-node/src/rail.rs),
/// which the unit test there watched fail.
#[tokio::test]
async fn an_append_naming_a_group_not_held_is_refused() {
    let da = tempfile::tempdir().unwrap();
    let a = start(1, &da).await;
    hold(&a, NS, 1);
    let answer = app()
        .post(format!(
            "http://{}/v1/rail/append?namespace=elsewhere",
            a.api_addr()
        ))
        .json(&serde_json::json!({ "op": "record", "payload": { "n": 1 } }))
        .send()
        .await
        .unwrap();
    assert_eq!(answer.status(), reqwest::StatusCode::NOT_FOUND);
    let out: serde_json::Value = answer.json().await.unwrap();
    assert_eq!(out["kind"], "not_held", "{out}");
    assert!(!da.path().join("rings/elsewhere").exists());
    assert!(a.rail().journal(NS).unwrap().read().unwrap().0.is_empty());
    a.endpoint().close().await;
}

/// **The people come from the register, with their keys** (the u3-2 row;
/// DESIGN §3): A founds the group as ada and admits B as bo; the people
/// door on A's loopback lists both, each with its key's hex, so an app
/// needs no second source. The failing input: the people read from the
/// seed alone, which the unit test in crates/svrngs-node/src/rail/tests.rs
/// watched fail (bo absent).
#[tokio::test]
async fn the_people_read_lists_a_and_b_with_their_keys() {
    let da = tempfile::tempdir().unwrap();
    let a = start(1, &da).await;
    hold(&a, NS, 1);
    admit(&a, 1, "bo", 2);
    let out: serde_json::Value = app()
        .get(format!(
            "http://{}/v1/rail/people?namespace={NS}",
            a.api_addr()
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mut want = vec![
        serde_json::json!({ "person": "ada", "key": actor_of(&signing(1)) }),
        serde_json::json!({ "person": "bo", "key": actor_of(&signing(2)) }),
    ];
    want.sort_by_key(|p| p["key"].as_str().unwrap().to_string());
    assert_eq!(out["people"], serde_json::Value::from(want), "{out}");
    a.endpoint().close().await;
}
