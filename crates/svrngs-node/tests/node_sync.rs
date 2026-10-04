//! Two nodes in one process, no relay, sync a group's journal over iroh, and a
//! third the group's seed does not name is refused by the sync route (DESIGN
//! §8, U1). Until u1-5 the seed — the group's `roster.json` — says who is in
//! the group.

use std::net::SocketAddr;
use std::time::Duration;

use svrngs_node::iroh::{Contact, Endpoint, PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::{
    actor_of, Digest, Ed25519Verifier, Payload, Person, RailAct, Roster, SigningKey,
};

const NS: &str = "house";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from_bytes(&signing(seed).verifying_key().to_bytes()).unwrap()
}

/// No relay and no address lookup: peers are reached by seeded addresses.
fn no_relay() -> RelayConfig {
    RelayConfig::from_parts(Vec::new(), Some("none")).unwrap()
}

async fn start(seed: u8, dir: &tempfile::TempDir) -> Node {
    Node::start(dir.path().to_path_buf(), signing(seed), &no_relay())
        .await
        .expect("a node starts")
}

/// The seed naming `seeds`, each as its own person.
fn seed_of(seeds: &[u8]) -> Roster {
    Roster::new(
        seeds
            .iter()
            .map(|s| (Person::from(format!("p{s}")), vec![actor_of(&signing(*s))]))
            .collect(),
    )
}

/// iroh binds the wildcard; rewrite to loopback so the address is dialable
/// in-process.
fn loopback_sockets(endpoint: &Endpoint) -> Vec<SocketAddr> {
    endpoint
        .bound_sockets()
        .into_iter()
        .map(|mut a| {
            if a.ip().is_unspecified() {
                let ip = if a.is_ipv4() { "127.0.0.1" } else { "::1" };
                a.set_ip(ip.parse().unwrap());
            }
            a
        })
        .collect()
}

fn digest(node: &Node) -> Digest {
    node.rail()
        .journal(NS)
        .unwrap()
        .digest(&Ed25519Verifier)
        .unwrap()
}

/// A's group is seeded with A's and B's keys and A writes three acts; B holds
/// the same seed and nothing else. One round of B's pulls A's journal over
/// iroh and the two converge. The failing input is a round that dials nobody.
#[tokio::test]
async fn a_member_the_seed_names_pulls_the_groups_journal_over_iroh() {
    let (da, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (start(1, &da).await, start(2, &db).await);
    let seed = seed_of(&[1, 2]);
    let ja = a.rail().journal(NS).unwrap();
    ja.set_roster(&seed).unwrap();
    b.rail().journal(NS).unwrap().set_roster(&seed).unwrap();
    for n in 0..3 {
        let payload = Payload::new(serde_json::json!({ "n": n })).unwrap();
        ja.append(
            RailAct::Record { payload },
            a.rail().signer(),
            &seed,
            None,
            &Ed25519Verifier,
        )
        .unwrap();
    }
    assert_ne!(digest(&a), digest(&b), "control: B holds none of A's acts");

    b.transport()
        .add_known_peer(key(1), loopback_sockets(a.endpoint()));
    let outcome = tokio::time::timeout(Duration::from_secs(30), b.round())
        .await
        .expect("a round finishes");
    assert_eq!(outcome.peers_reached, 1, "{outcome:?}");
    assert_eq!(outcome.peers_refused, 0, "{outcome:?}");
    assert_eq!(digest(&a), digest(&b), "two nodes, one journal");
    let held = b
        .rail()
        .journal(NS)
        .unwrap()
        .admit(&seed, &Ed25519Verifier)
        .unwrap();
    assert_eq!(held.ops.len(), 3);
    assert!(held.is_complete(), "gaps: {:?}", held.gaps);
}

/// C, whom the seed does not name, dials A and asks for the group: the sync
/// route answers 403, and a header naming B makes C no more B. The failing
/// input is a route that reads the asker from the header.
#[tokio::test]
async fn a_dialer_the_seed_does_not_name_is_refused_whatever_its_header_claims() {
    let (da, dc) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, c) = (start(1, &da).await, start(3, &dc).await);
    a.rail()
        .journal(NS)
        .unwrap()
        .set_roster(&seed_of(&[1, 2]))
        .unwrap();
    let base = c
        .transport()
        .base_url(&Contact {
            key: key(1),
            relay_url: None,
            iroh_direct_addrs: loopback_sockets(a.endpoint()),
        })
        .await
        .expect("a contact with an address resolves");
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let body = serde_json::json!({ "namespace": NS, "digest": Digest::default(), "ops": [] });
    for claim in [None, Some(actor_of(&signing(2)))] {
        let mut req = http.post(format!("{base}/internal/ring/sync")).json(&body);
        if let Some(b) = &claim {
            req = req.header("X-Mesh-Pubkey", b);
        }
        let answer = tokio::time::timeout(Duration::from_secs(20), req.send())
            .await
            .expect("no timeout")
            .expect("C is admitted, and answered");
        assert_eq!(answer.status(), 403, "claiming {claim:?}");
        assert!(answer
            .text()
            .await
            .unwrap()
            .contains("not on house's roster"));
    }
}
