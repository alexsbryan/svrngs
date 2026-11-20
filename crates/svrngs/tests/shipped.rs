//! Where the command as shipped meets its groups: at svrngs's own relay
//! with no address lookup, and at n0's public relays only while ours does
//! not answer, the emergency behind it (the operator's switch; PLAN §2,
//! §3; n0-emergency; DESIGN §3, "Where a computer meets its groups"). No
//! socket: the build's relays as `reach(SHIPPED)` makes them, and a
//! computer built with them moved on one wire as ours goes silent and
//! comes back. svrngs's relay is the test's own spelling, apart from
//! `cli::DEFAULT_RELAY`. A group that keeps its own relay is
//! `tests/default_relay.rs`'s, a binary apart. Planted: `cli::SHIPPED`
//! left at `MeetAt::N0`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::time::Duration;

use svrngs::cli;
use svrngs::group;
use svrngs_node::iroh::RelayUrl;
use svrngs_node::node::{Node, Relaying, Wire};
use svrngs_node::SigningKey;

/// svrngs's relay, as the operator hosts it.
const OURS: &str = "https://relay.svrnme.sh/";

#[test]
fn the_command_as_shipped_meets_its_groups_at_svrngs_s_relay_with_no_lookup() {
    let shipped = cli::reach(cli::SHIPPED);
    assert!(!shipped.n0_services, "no address lookup, n0's or any");
    assert_eq!(shipped.relay_urls, [OURS], "at svrngs's relay alone");
}

/// The relay `node` is homed at on the wire, once it is `want`, waited for
/// on the test's clock as long as relay_tiers.rs `homing_by` waits.
async fn homed(node: &Node, want: &RelayUrl) {
    let tiers = (
        svrngs_node::tiers::ASKED_EVERY,
        svrngs_node::tiers::LEFT_AFTER,
    );
    let by = tiers.0 * (tiers.1 + 2) + Duration::from_secs(26);
    let t = tokio::time::Instant::now();
    while t.elapsed() < by {
        if node.net().addr().relay_urls().next() == Some(want) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("not homed at {want} within {by:?}");
}

/// **The command as shipped meets at n0's only while ours does not
/// answer**: Sam's computer, built with `reach(SHIPPED)`, keeps a group
/// that names no relay of its own. While svrngs's relay answers he is
/// homed there and holds none of n0's; with it silent, at n0's; with it
/// back, at ours again, n0's out of his endpoint.
#[tokio::test(start_paused = true)]
async fn the_command_as_shipped_meets_at_n0_s_only_while_ours_does_not_answer() {
    let wire = Wire::default();
    let ours: RelayUrl = OURS.parse().unwrap();
    let n0 = svrngs_node::relay::n0_relays()
        .urls::<Vec<RelayUrl>>()
        .remove(0);
    wire.relay(&ours, Relaying::Up);
    wire.relay(&n0, Relaying::Up);
    let (dir, key) = (
        tempfile::tempdir().unwrap(),
        SigningKey::from_bytes(&[52; 32]),
    );
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let at = (dir.path().to_path_buf(), key.clone());
    let sam = Node::on_wire_relayed(&wire, at, &cli::reach(cli::SHIPPED), &[], every).await;
    let sam = sam.expect("Sam's computer starts on the wire");
    let name = |n: &str| group::Name::parse(n).unwrap();
    let house = group::draw(name("house"), name("Sam"), key.verifying_key());
    sam.host()
        .keep_group(&house, None, &[])
        .expect("Sam keeps house, which names no relay");
    let n0_held = |sam: &Node| sam.relays_held().contains(&n0.to_string());

    homed(&sam, &ours).await;
    assert!(
        !n0_held(&sam),
        "n0's held while ours answers: {:?}",
        sam.relays_held()
    );
    wire.relay(&ours, Relaying::Down);
    homed(&sam, &n0).await;
    wire.relay(&ours, Relaying::Up);
    homed(&sam, &ours).await;
    assert!(
        !n0_held(&sam),
        "n0's held once ours answers again: {:?}",
        sam.relays_held()
    );
}
