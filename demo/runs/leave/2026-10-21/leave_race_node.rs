// SPDX-License-Identifier: AGPL-3.0-or-later
//! #211 through three computers in one process (no relay, peers by seeded
//! address, as `tests/removal.rs` reaches them). Each computer's round is a
//! day apart (`Node::start_every`), so only a wake or a round the test asks
//! moves anything: no sleep decides what a copy holds, only polls for what
//! a wake already sent.
//!
//! Not in the crate: `demo/runs/leave/2026-10-21/repro.sh node` copies it
//! into `crates/svrngs-node/tests/`, runs it, and removes it.
//!
//! - `a_leave_reaches_every_copy_with_the_acts_it_covers` asserts what a
//!   fix must make true, and is red at HEAD.
//! - the other two assert what HEAD does, and pass.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::time::Duration;

use svrngs_core::tests_support::{applied, record, Batch};
use svrngs_node::iroh::{PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::removal::{LeaveAsk, Outcome, RemovalAsk, LEAVES_ROUTE, REMOVALS_ROUTE};
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};

const NS: &str = "oak";
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from_bytes(&signing(seed).verifying_key().to_bytes()).unwrap()
}

fn no_seed(_: &std::path::Path) -> Result<Option<String>, String> {
    Ok(None)
}

async fn start(seed: u8, dir: &tempfile::TempDir) -> Node {
    let relay = RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    };
    let api = "127.0.0.1:0".parse().unwrap();
    Node::start_every(dir.path().to_path_buf(), signing(seed), &relay, no_seed, api, DAY)
        .await
        .expect("a node starts")
}

/// Oak: Ana's key (1) its seed, so Ana keeps it; Kim (2) and Bo (3)
/// admitted by Ana; every copy holding all three, by one round each.
async fn oak(dirs: &[tempfile::TempDir; 3]) -> [Node; 3] {
    let nodes = [start(1, &dirs[0]).await, start(2, &dirs[1]).await, start(3, &dirs[2]).await];
    let seed = Roster::new([(Person::parse("ana").unwrap(), vec![actor_of(&signing(1))])].into());
    for n in &nodes {
        n.rail().set_seed(NS, &seed).unwrap();
    }
    for (person, k) in [("kim", 2u8), ("bo", 3)] {
        let admit = RailAct::Admit {
            person: Person::parse(person).unwrap(),
            key: actor_of(&signing(k)),
            invite: None,
        };
        nodes[0].rail().commit(NS, &Batch(vec![admit])).unwrap().unwrap();
    }
    for (i, n) in nodes.iter().enumerate() {
        for (j, m) in nodes.iter().enumerate() {
            if i != j {
                n.transport().add_known_peer(key(j as u8 + 1), loopback_sockets(m.endpoint()));
            }
        }
    }
    for n in &nodes[1..] {
        tokio::time::timeout(Duration::from_secs(30), n.round()).await.unwrap();
    }
    nodes
}

/// `node` writes a money entry, `what`, and the lines it wrote; no round
/// is woken (`RingRail::commit` wakes none, as a Visit's write wakes none,
/// #212).
fn writes(node: &Node, what: &str) -> Vec<svrngs_node::Op<svrngs_node::SignedOp>> {
    let done = node.rail().commit(NS, &Batch(vec![record(what)])).unwrap().unwrap();
    done.lines().to_vec()
}

fn http() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

async fn leaves(node: &Node) -> Outcome {
    let answer = http()
        .post(format!("http://{}{LEAVES_ROUTE}", node.api_addr()))
        .json(&LeaveAsk { group: NS.into() })
        .send()
        .await
        .unwrap();
    answer.json().await.unwrap()
}

async fn removes(node: &Node, person: &str) -> Outcome {
    let ask = RemovalAsk { group: NS.into(), person: person.into(), joined: None };
    let answer = http()
        .post(format!("http://{}{REMOVALS_ROUTE}", node.api_addr()))
        .json(&ask)
        .send()
        .await
        .unwrap();
    answer.json().await.unwrap()
}

/// Whether `node`'s copy holds `seed`'s key cut, polled for up to 10 s:
/// the cut was already sent, by the wake its write made.
async fn holds_cut(node: &Node, seed: u8) -> bool {
    let cut = || node.rail().register(NS).unwrap().removed.contains_key(&actor_of(&signing(seed)));
    tokio::time::timeout(Duration::from_secs(10), async {
        while !cut() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .is_ok()
}

/// Every copy rounds, three times over: each exchange there is to have.
async fn rounds(nodes: &[&Node]) {
    for _ in 0..3 {
        for n in nodes {
            tokio::time::timeout(Duration::from_secs(30), n.round()).await.unwrap();
        }
    }
}

/// What a copy's person reads of the record: `GET /v1/rail/log`'s
/// completeness and each gap's sentence, and the money entries that apply.
async fn says(name: &str, node: &Node) -> Vec<String> {
    let log: serde_json::Value = http()
        .get(format!("http://{}/v1/rail/log?namespace={NS}", node.api_addr()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let gaps: Vec<&str> = (log["gaps"].as_array().unwrap().iter())
        .map(|g| g["message"].as_str().unwrap())
        .collect();
    let entries = applied(&node.rail().admission(NS).unwrap());
    eprintln!("  {name:<4} entries {entries:?}; complete {}; gaps {gaps:?}", log["complete"]);
    entries
}

/// **A leave reaches every copy with the acts it covers** — what a fix must
/// make true; red at HEAD. Kim's first entry reaches everyone by a round;
/// her last is written with no round, and she leaves. Her leave reaches
/// Ana's and Bo's copies by her round's telling, the `Remove` alone
/// (removal.rs `tell`; ring_sync.rs `leave_told`), and nothing after it
/// carries her last entry: her round tells only the leave, nobody dials a
/// key cut (ring_sync.rs, `standing.names`), and the sync route takes none
/// of a cut asker's acts (ring_routes.rs `told_cut`).
#[tokio::test]
async fn a_leave_reaches_every_copy_with_the_acts_it_covers() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [ana, kim, bo] = oak(&dirs).await;
    writes(&kim, "kim-paid");
    rounds(&[&kim]).await;
    writes(&kim, "kim-last");
    assert_eq!(leaves(&kim).await, Outcome::Left);
    assert!(holds_cut(&ana, 2).await && holds_cut(&bo, 2).await, "the leave reached both");
    rounds(&[&ana, &bo, &kim]).await;
    eprintln!("after Kim's leave and three rounds of every computer:");
    let kims = says("Kim", &kim).await;
    let (anas, bos) = (says("Ana", &ana).await, says("Bo", &bo).await);
    assert_eq!(kims, ["kim-paid", "kim-last"], "control: Kim's copy");
    assert_eq!(anas, kims, "Ana's copy agrees with Kim's");
    assert_eq!(bos, kims, "Bo's copy agrees with Kim's");
}

/// **It heals when a copy that stands holds the act** (HEAD; passes). Bo's
/// copy took Kim's last entry before her leave; Ana's did not. Ana's next
/// round with Bo carries it (sync.rs: author-blind), so Ana's copy agrees.
#[tokio::test]
async fn a_leave_heals_where_a_standing_copy_holds_its_last_act() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [ana, kim, bo] = oak(&dirs).await;
    writes(&kim, "kim-paid");
    rounds(&[&kim]).await;
    let last = writes(&kim, "kim-last");
    bo.rail().ingest_all(NS, &last).unwrap();
    assert_eq!(leaves(&kim).await, Outcome::Left);
    assert!(holds_cut(&ana, 2).await, "the leave reached Ana");
    eprintln!("before Ana's round with Bo:");
    let before = says("Ana", &ana).await;
    assert!(before.is_empty(), "Kim's entries count nowhere on Ana's copy");
    rounds(&[&ana]).await;
    eprintln!("after it:");
    assert_eq!(says("Ana", &ana).await, ["kim-paid", "kim-last"]);
    assert_eq!(says("Kim", &kim).await, ["kim-paid", "kim-last"]);
}

/// **A keeper's removal carries what it names** (HEAD; passes). Ana keeps
/// Oak and holds Kim's entry `kim-seen`; Bo's copy does not; `kim-unseen`
/// reached nobody. Ana removes Kim: her round's exchange gives Bo the cut
/// and what it names together, and `kim-unseen`, above what the cut names,
/// counts nowhere on any copy, Kim's included. The copies agree.
#[tokio::test]
async fn a_keepers_removal_reaches_every_copy_with_what_it_names() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [ana, kim, bo] = oak(&dirs).await;
    let seen = writes(&kim, "kim-seen");
    ana.rail().ingest_all(NS, &seen).unwrap();
    writes(&kim, "kim-unseen");
    assert_eq!(removes(&ana, "kim").await, Outcome::Out);
    assert!(holds_cut(&bo, 2).await && holds_cut(&kim, 2).await, "the cut reached both");
    rounds(&[&ana, &bo, &kim]).await;
    eprintln!("after Ana removed Kim and three rounds of every computer:");
    for (name, node) in [("Ana", &ana), ("Bo", &bo), ("Kim", &kim)] {
        assert_eq!(says(name, node).await, ["kim-seen"], "{name}'s copy");
    }
}
