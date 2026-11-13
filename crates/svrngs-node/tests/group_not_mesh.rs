//! A group is not a mesh (PLAN §4, U1): who may connect, who is synced and
//! who may write come from the group's register — its seed, the founder's
//! key alone, and the `Admit` acts written through its door — never a mesh.
//! Nodes in one process on one wire (common/on_wire.rs; docs/CHECKS.md,
//! harness 3), no endpoint and no socket: each node's acceptor and routes
//! answer as they decide. That a refused dialer's QUIC connection is
//! closed with no HTTP answer is accept.rs's
//! (`a_refused_dialer_gets_its_connection_closed_with_no_http_answer`).
//! The shape of upstream's
//! `commonwealth/crates/commonwealth-rails/tests/ring_round.rs`, rewritten:
//! upstream's is built on the mesh daemon.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[path = "common/on_wire.rs"]
mod on_wire;

use std::time::Duration;

use svrngs_node::iroh::{Alpn, Unsent};
use svrngs_node::node::{Node, Unlisted};
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

/// `seed`'s computer over `dir` on this test's wire. No link is asked
/// about here: groups are seeded by hand, and no `seed.json` is kept in
/// `dir`, so the seeds it reads are none.
async fn start(seed: u8, dir: &tempfile::TempDir) -> Node {
    on_wire::start(seed, dir.path()).await
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
    let fixture = svrngs_core::tests_support::Fixture { act };
    node.rail()
        .commit(ns, &fixture)
        .unwrap()
        .unwrap_or_else(|e| panic!("the door refused: {e:?}"))
        .lines()[0]
        .clone()
}

/// `joining` let in as `person` by a link `node` mints: no bare `Admit`
/// counts (release 2).
fn admit(node: &Node, ns: &str, person: &str, joining: u8) {
    svrngs_core::tests_support::let_in_by(ns, &signing(joining), person, |act| {
        append(node, ns, act).id
    });
}

fn record(n: u32) -> RailAct {
    RailAct::Record {
        payload: Payload::new(serde_json::json!({ "n": n })).unwrap(),
    }
}

fn digest(node: &Node, ns: &str) -> Digest {
    node.rail().digest(ns).unwrap()
}

/// `dialer`'s base URL for `served`, as a splice gave one: its requests
/// asked of `served` on the members' protocol as `dialer`'s key
/// (on_wire.rs `bridge_as`), decided by `served`'s acceptor and routes.
async fn base_url(dialer: u8, served: &Node) -> String {
    let bridge = on_wire::bridge_as(&signing(dialer), served, Alpn::Member).await;
    format!("http://{}", bridge.local_addr())
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
    admit(&a, NS, "bo", 2);
    append(&a, NS, record(0));
    hold(&b, NS, 1);
    assert_ne!(
        digest(&a, NS),
        digest(&b, NS),
        "control: B holds none of A's acts"
    );

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
    admit(&a, NS, "bo", 2);
    let link = signing(9);
    let invite = append(
        &a,
        NS,
        RailAct::Invite {
            key: actor_of(&link),
            until: i64::MAX,
            device_of: None,
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
    // A knows no address of P's, as a page's key is never one to dial.
    on_wire::part(&a, &p);

    let round_a = round(&a).await;
    assert_eq!(
        (round_a.peers_reached, round_a.peers_unreachable),
        (1, 0),
        "A dials B and not P: {round_a:?}"
    );
    let lends = tokio::time::timeout(Duration::from_secs(30), a.lends())
        .await
        .expect("the ask finishes");
    assert_eq!(lends.unlisted.unanswered, [], "A asks B alone");

    // P reaches A alone, as when it was told A's address and no other.
    on_wire::part(&p, &b);
    let round_p = round(&p).await;
    assert_eq!(
        (round_p.peers_reached, round_p.peers_refused),
        (1, 0),
        "A's acceptor and sync route serve P: {round_p:?}"
    );
    assert_eq!(digest(&a, NS), digest(&p, NS), "P converged onto A");
}

/// A computer that answered is never said not to have answered
/// (bash-first-minute (c); Demo C's dry run, finding 6). A founds the
/// group and admits Bo's, Cy's and Dee's computers. Bo's holds no group, so
/// it has not heard A is in one, and closes A's connection as not admitted;
/// Cy's holds the group and cannot read what it lends, and answers so;
/// Dee's has no address A can reach. A's list names each by how: Dee's did
/// not answer, Bo's has not heard of A, Cy's refused — and A's own ask
/// carries no stranger's answer: Bo's acceptor still closes A with no HTTP
/// answer. Watched failing with every one of the three named unanswered.
#[tokio::test]
async fn a_computer_that_refuses_is_said_apart_from_one_that_does_not_answer() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    // Bo's computer on the wire, holding no group: its acceptor decides.
    let (a, _bo, cy) = (
        start(1, &dirs[0]).await,
        start(2, &dirs[1]).await,
        start(3, &dirs[2]).await,
    );
    for node in [&a, &cy] {
        hold(node, NS, 1);
    }
    for (person, k) in [("bo", 2), ("cy", 3), ("dee", 4)] {
        admit(&a, NS, person, k);
    }
    // Cy's lends cannot be read: where its declarations go is a file.
    std::fs::write(
        svrngs_node::lend::lends_dir(dirs[2].path()),
        "not a directory",
    )
    .unwrap();
    // Dee's computer is on no wire: no path reaches it.
    let lends = tokio::time::timeout(Duration::from_secs(30), a.lends())
        .await
        .expect("the ask finishes");
    let names = |n: &[(&str, u8)]| {
        let named = |(p, k): &(&str, u8)| svrngs_node::node::Named {
            person: Person::parse(p).unwrap(),
            person_key: actor_of(&signing(*k)),
        };
        n.iter().map(named).collect::<Vec<_>>()
    };
    assert_eq!(
        lends.unlisted,
        Unlisted {
            unanswered: names(&[("dee", 4)]),
            unheard: names(&[("bo", 2)]),
            refused: names(&[("cy", 3)]),
            older: vec![],
        },
        "each computer named by how it did not say what it lends, and by its person's first key"
    );
    assert!(lends.lent.is_empty(), "{lends:?}");
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
    let dir = tempfile::tempdir().unwrap();
    let a = start(1, &dir).await;
    // C's and D's keys dial A, as their computers would (on_wire.rs
    // `bridge_as`): A decides by the key alone.
    hold(&a, NS, 1);
    hold(&a, OTHER, 1);
    admit(&a, OTHER, "dee", 4);
    let http = on_wire::http();
    let sync = |base: &str, ns: &str, claim: Option<String>| {
        let mut req = http
            .post(format!("{base}/internal/ring/sync"))
            .json(&serde_json::json!({ "namespace": ns, "digest": Digest::default(), "ops": [] }));
        if let Some(c) = claim {
            req = req.header("x-mesh-pubkey", c);
        }
        tokio::time::timeout(Duration::from_secs(20), req.send())
    };

    let to_c = base_url(3, &a).await;
    let answer = sync(&to_c, NS, None)
        .await
        .expect("a closed connection is an error, not a hang");
    assert!(
        matches!(answer, Err(Unsent::NotAdmitted)),
        "C stands nowhere, so gets no HTTP answer, got {:?}",
        answer.map(|r| r.status())
    );

    let to_d = base_url(4, &a).await;
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
