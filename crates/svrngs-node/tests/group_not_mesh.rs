//! A group is not a mesh (DESIGN §8, U1): who may connect, who is synced and
//! who may write come from the group's register — its seed, the founder's
//! key alone, and the `Admit` acts written through its door — never a mesh.
//! Nodes in one process, no relay; peers are reached by seeded addresses.
//! The shape of upstream's
//! `commonwealth/crates/commonwealth-rails/tests/ring_round.rs`, rewritten:
//! upstream's is built on the mesh daemon.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::time::Duration;

use svrngs_node::iroh::{Contact, PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::register::Standing;
use svrngs_node::{
    actor_of, Device, Digest, Op, Payload, Person, RailAct, Redemption, Roster, SignedOp,
    SigningKey,
};

const NS: &str = "house";
const OTHER: &str = "work";

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

/// No link is asked about here: groups are seeded by hand, with no seed
/// the command would keep.
fn no_seed(_: &std::path::Path) -> Result<Option<String>, String> {
    Ok(None)
}

/// A group's seed: its founder's key alone (DESIGN §6).
fn founded_by(founder: u8) -> Roster {
    Roster::new(
        [(
            Person::parse("founder").unwrap(),
            vec![actor_of(&signing(founder))],
        )]
        .into_iter()
        .collect(),
    )
}

/// `node` holds `ns`, seeded with `founder`'s key.
fn hold(node: &Node, ns: &str, founder: u8) {
    node.rail().set_seed(ns, &founded_by(founder)).unwrap();
}

/// Sign `act` onto `ns` through `node`'s door, against the group's seed.
fn append(node: &Node, ns: &str, act: RailAct) -> Op<SignedOp> {
    let fixture = svrngs_core::tests_support::Fixture {
        act,
        on_behalf_of: None,
    };
    node.rail()
        .commit(ns, &fixture)
        .unwrap()
        .unwrap_or_else(|e| panic!("the door refused: {e:?}"))
        .lines()[0]
        .clone()
}

fn admit(person: &str, joining: u8) -> RailAct {
    RailAct::Admit {
        person: Person::parse(person).unwrap(),
        key: actor_of(&signing(joining)),
        invite: None,
    }
}

fn record(n: u32) -> RailAct {
    RailAct::Record {
        payload: Payload::new(serde_json::json!({ "n": n })).unwrap(),
    }
}

fn digest(node: &Node, ns: &str) -> Digest {
    node.rail().digest(ns).unwrap()
}

/// `dialer`'s base URL for `served`, through its bridge.
async fn base_url(dialer: &Node, served: &Node) -> String {
    dialer
        .transport()
        .base_url(&Contact {
            key: served.endpoint().id(),
            relay_url: None,
            iroh_direct_addrs: loopback_sockets(served.endpoint()),
        })
        .await
        .expect("a contact with an address resolves")
}

async fn round(node: &Node) -> svrngs_node::ring_sync::RoundOutcome {
    tokio::time::timeout(Duration::from_secs(30), node.round())
        .await
        .expect("a round finishes")
}

/// A founds the group and admits B through its door; B holds only the seed.
/// B's round dials A — the one key standing in B's copy — and A's acceptor
/// and route serve B, who stands by A's `Admit`: B converges onto A's
/// journal. B then writes through its own door, standing by the `Admit` it
/// pulled, and its next round carries the act to A. The failing inputs:
/// standing read as the seed alone (A refuses B), and the door asking the
/// seed (B's door refuses B).
#[tokio::test]
async fn a_member_the_founder_admits_syncs_and_writes() {
    let (da, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (start(1, &da).await, start(2, &db).await);
    hold(&a, NS, 1);
    append(&a, NS, admit("bo", 2));
    append(&a, NS, record(0));
    hold(&b, NS, 1);
    assert_ne!(
        digest(&a, NS),
        digest(&b, NS),
        "control: B holds none of A's acts"
    );
    b.transport()
        .add_known_peer(key(1), loopback_sockets(a.endpoint()));

    let pulled = round(&b).await;
    assert_eq!(
        (pulled.peers_reached, pulled.peers_refused),
        (1, 0),
        "{pulled:?}"
    );
    assert_eq!(digest(&a, NS), digest(&b, NS), "B converged onto A");

    // No wait: B writes at once, and its key sorts before A's, so in the
    // rail's order, (ts_unix, actor, seq, id), an act stamped in its
    // `Admit`'s second would sort before that `Admit` and not count (DESIGN
    // §6). B's door signs one second past the newest act it holds, so it
    // never is. At the wall clock this failed whenever B wrote in that second.
    assert!(
        actor_of(&signing(2)) < actor_of(&signing(1)),
        "control: the new key sorts first"
    );
    let bos = append(&b, NS, record(1));
    let pushed = round(&b).await;
    // Which round moved the act is not asserted: the round each node runs at
    // start may race this one. That A now holds and counts it is.
    assert_eq!(pushed.peers_reached, 1, "{pushed:?}");
    let held = a.rail().admission(NS).unwrap();
    assert!(held.is_complete(), "gaps: {:?}", held.gaps());
    let ops = svrngs_core::tests_support::wire_ops(&held);
    let counted = ops
        .iter()
        .filter(|o| o.applies())
        .find(|o| o.id == bos.id)
        .expect("A holds and counts B's act");
    assert_eq!(counted.person, Person::parse("bo").unwrap());
}

/// A browser's key dials and is never dialled (u4-1; DESIGN §3 "Joining",
/// call 2). A founds the group, admits B's computer, and admits P's key by
/// a redemption naming a browser; B and P hold the group's seed. A's round
/// dials B alone, and A's list of what the group's people lend asks B alone,
/// naming nobody whose computer did not answer. P — a node standing in for
/// a page — dials A, and A's acceptor and sync route serve it, as they
/// would a computer. The failing input: the round dialling every standing
/// key, which leaves P unreachable and named unanswered.
#[tokio::test]
async fn a_round_dials_a_computer_and_never_a_browser() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let (a, b, p) = (
        start(1, &dirs[0]).await,
        start(2, &dirs[1]).await,
        start(3, &dirs[2]).await,
    );
    for node in [&a, &b, &p] {
        hold(node, NS, 1);
    }
    append(&a, NS, admit("bo", 2));
    let link = signing(9);
    let invite = append(
        &a,
        NS,
        RailAct::Invite {
            key: actor_of(&link),
            until: i64::MAX,
        },
    );
    let pia = Person::parse("pia").unwrap();
    append(
        &a,
        NS,
        RailAct::Admit {
            invite: Some(Redemption::sign(
                &signing(3),
                NS,
                &invite.id,
                &pia,
                Device::Browser,
            )),
            person: pia,
            key: actor_of(&signing(3)),
        },
    );
    let Standing::Members(standing) = a.rail().standing(NS).unwrap() else {
        panic!("A holds the group, seeded")
    };
    let devices: Vec<_> = [2, 3]
        .map(|k| standing.get(&actor_of(&signing(k))).map(|m| m.device))
        .into();
    assert_eq!(
        devices,
        [Some(Device::Computer), Some(Device::Browser)],
        "control: B stands from a computer and P from a browser"
    );
    a.transport()
        .add_known_peer(key(2), loopback_sockets(b.endpoint()));

    let round_a = round(&a).await;
    assert_eq!(
        (round_a.peers_reached, round_a.peers_unreachable),
        (1, 0),
        "A dials B and not P: {round_a:?}"
    );
    let lends = tokio::time::timeout(Duration::from_secs(30), a.lends())
        .await
        .expect("the ask finishes");
    assert_eq!(lends.unanswered, Vec::<Person>::new(), "A asks B alone");

    p.transport()
        .add_known_peer(key(1), loopback_sockets(a.endpoint()));
    let round_p = round(&p).await;
    assert_eq!(
        (round_p.peers_reached, round_p.peers_refused),
        (1, 0),
        "A's acceptor and sync route serve P: {round_p:?}"
    );
    assert_eq!(digest(&a, NS), digest(&p, NS), "P converged onto A");
}

/// C, admitted to nothing, has its connection closed by A's acceptor with no
/// HTTP answer. D, admitted only to A's second group, is served that group
/// and refused (403) the first — a header naming A makes D no more A — and
/// a group A does not hold is refused in the same words, so the answer says
/// nothing of what A holds. The failing inputs: the acceptor admitting any
/// dialer (C answered), and the route skipping its group check (D served
/// the first group).
#[tokio::test]
async fn a_stranger_is_closed_and_a_member_of_another_group_is_refused() {
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let a = start(1, &dirs[0]).await;
    let c = start(3, &dirs[1]).await;
    let d = start(4, &dirs[2]).await;
    hold(&a, NS, 1);
    hold(&a, OTHER, 1);
    append(&a, OTHER, admit("dee", 4));
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let sync = |base: &str, ns: &str, claim: Option<String>| {
        let mut req = http
            .post(format!("{base}/internal/ring/sync"))
            .json(&serde_json::json!({ "namespace": ns, "digest": Digest::default(), "ops": [] }));
        if let Some(c) = claim {
            req = req.header("X-Mesh-Pubkey", c);
        }
        tokio::time::timeout(Duration::from_secs(20), req.send())
    };

    let to_c = base_url(&c, &a).await;
    let answer = sync(&to_c, NS, None)
        .await
        .expect("a closed connection is an error, not a hang");
    assert!(
        answer.is_err(),
        "C stands nowhere, so gets no HTTP answer, got {:?}",
        answer.map(|r| r.status())
    );

    let to_d = base_url(&d, &a).await;
    let served = sync(&to_d, OTHER, None).await.unwrap().unwrap();
    assert_eq!(
        served.status(),
        200,
        "control: D is served the group it is in"
    );
    let mut refusals = Vec::new();
    for (ns, claim) in [
        (NS, None),
        (NS, Some(actor_of(&signing(1)))),
        ("elsewhere", None),
    ] {
        let answer = sync(&to_d, ns, claim.clone()).await.unwrap().unwrap();
        assert_eq!(answer.status(), 403, "{ns}, claiming {claim:?}");
        let body = answer.text().await.unwrap();
        refusals.push(body.replace(ns, "<group>"));
    }
    assert!(
        refusals.iter().all(|r| *r == refusals[0]),
        "one refusal, whether or not A holds the group: {refusals:?}"
    );
}
