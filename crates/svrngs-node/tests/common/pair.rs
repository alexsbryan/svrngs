//! Alex's computer and Sam's in one process, no relay, each on 127.0.0.1
//! alone, Sam reached by seeded addresses: one copy, where home.rs and
//! lend_origin.rs each wrote it out and peer.rs a third time
//! (REVIEW-audit-arch-auto-1). Only those three load it (`#[path]`), so
//! each test binary that compiles it uses all of it.

use std::path::Path;

use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};

use crate::common::loopback_sockets;

pub const NS: &str = "house";

pub fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn no_seed(_: &Path) -> Result<Option<String>, String> {
    Ok(None)
}

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
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// Alex's computer and Sam's: Alex founds a group and lets Sam in, and
/// Alex's knows where Sam's listens.
pub async fn alex_lets_sam_in(alex_dir: &Path, sam_dir: &Path) -> (Node, Node) {
    let alex = start(1, alex_dir).await;
    let sam = start(2, sam_dir).await;
    let seed = Roster::new(
        [(Person::parse("alex").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    for node in [&alex, &sam] {
        node.rail().set_seed(NS, &seed).unwrap();
    }
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("sam").unwrap(),
            key: actor_of(&signing(2)),
            invite: None,
        },
    };
    alex.rail().commit(NS, &admit).unwrap().unwrap();
    alex.transport()
        .add_known_peer(sam.endpoint().id(), loopback_sockets(sam.endpoint()));
    (alex, sam)
}
