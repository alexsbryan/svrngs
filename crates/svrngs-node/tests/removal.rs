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
use svrngs_node::removal::{Outcome, RemovalAsk, HANDOVERS_ROUTE, REMOVALS_ROUTE};
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
        .json(&RemovalAsk {
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

/// Whether `node`'s copy holds `seed`'s key cut, within `within`.
async fn holds_cut(node: &Node, seed: u8, within: Duration) -> bool {
    tokio::time::timeout(within, async {
        while !node
            .rail()
            .register(NS)
            .unwrap()
            .removed
            .contains_key(&actor_of(&signing(seed)))
        {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .is_ok()
}

/// **A removal goes now, not at the next round** (U7 (e); u7-removal (f)'s
/// run, demo/runs/u7/2026-10-11/README.md, take 1: Ana's lend refused Bo
/// 43.3 s after Sam's command, at Ana's next round). Ana's ask reaches Sam,
/// Sam's cut reaches Ana, and Sam's hand-on to Ana reaches her, each within
/// 5 s, with every round 60 s apart and none asked for. Red with the
/// removal route waking no round (removal.rs at 7412588).
#[tokio::test]
async fn a_removal_reaches_every_computer_without_waiting_for_a_round() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [sam, ana, _bo] = three(&dirs).await;
    let soon = Duration::from_secs(5);
    // Ana's ask is her first act, so any act of hers held is it.
    let asks = |node: &Node| {
        node.rail()
            .digest(NS)
            .unwrap()
            .get(&actor_of(&signing(2)))
            .is_some()
    };

    assert_eq!(
        ask(&ana, REMOVALS_ROUTE, "bo").await.1,
        Outcome::Asked {
            decides: Some(Person::parse("sam").unwrap())
        }
    );
    let reached = tokio::time::timeout(soon, async {
        while !asks(&sam) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(
        reached.is_ok(),
        "Ana's ask reached Sam's computer within 5 s"
    );

    assert_eq!(ask(&sam, REMOVALS_ROUTE, "bo").await, (200, Outcome::Out));
    assert!(
        holds_cut(&ana, 3, soon).await,
        "Sam's cut reached Ana's computer within 5 s"
    );

    assert_eq!(
        ask(&sam, HANDOVERS_ROUTE, "ana").await,
        (200, Outcome::Handed)
    );
    let handed = tokio::time::timeout(soon, async {
        while ana.rail().register(NS).unwrap().keeper != Some(actor_of(&signing(2))) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(
        handed.is_ok(),
        "Sam's hand-on reached Ana's computer within 5 s"
    );
}

/// `node`'s copy of the group's seed, the file the register reads first.
fn seed_path(node: &Node) -> std::path::PathBuf {
    node.rail().group_dir(NS).unwrap().join("roster.json")
}

/// The plant for #123: a layer that makes `seed` unreadable as the core
/// gives its hand-on verdict, so the commit reads the register and the
/// read after it cannot.
struct Unreadable(std::path::PathBuf);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Unreadable {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        use std::os::unix::fs::PermissionsExt;
        struct Message(String);
        impl tracing::field::Visit for Message {
            fn record_debug(&mut self, f: &tracing::field::Field, v: &dyn std::fmt::Debug) {
                if f.name() == "message" {
                    self.0 = format!("{v:?}");
                }
            }
        }
        let mut m = Message(String::new());
        event.record(&mut m);
        if m.0 == "hand-on: the fold's verdict" {
            std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o000)).unwrap();
        }
    }
}

/// **A register that cannot be read is said as itself** (REVIEW_FINDINGS
/// #123). Ana's hand-on is refused, and her computer cannot read the
/// register to name who decides: she is told so, not told the words of a
/// group nobody decides. The seed is made unreadable between the core's
/// refusal and the read that names who decides (the plant, [`Unreadable`]).
#[tokio::test]
async fn an_unreadable_register_is_said_on_a_hand_on() {
    use std::os::unix::fs::PermissionsExt;
    use tracing_subscriber::layer::SubscriberExt;
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [_sam, ana, _bo] = three(&dirs).await;
    let seed = seed_path(&ana);
    let readable = || std::fs::set_permissions(&seed, std::fs::Permissions::from_mode(0o644));

    let planted = tracing_subscriber::registry().with(Unreadable(seed.clone()));
    let after = {
        let _on = tracing::subscriber::set_default(planted);
        ask(&ana, HANDOVERS_ROUTE, "ana").await
    };
    let unread = ana.rail().register(NS).is_err();
    readable().unwrap();
    assert!(
        unread,
        "the plant: Ana's register turned unreadable after the refusal"
    );
    assert_eq!(after, (409, Outcome::NotDeciderUnread));
}
