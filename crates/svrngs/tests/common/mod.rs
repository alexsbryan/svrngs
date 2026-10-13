//! What the command's tests start alike, written once (ralph/REVIEW_FINDINGS.md
//! #31): included by each test binary that starts Sam's computer, beside
//! the browser harness it reaches the relay through.

use std::time::Duration;

use iroh_relay::server::Server;
use svrngs::group;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

use crate::harness;

/// Sam's computer, its key from `seed`, online through a relay on
/// 127.0.0.1: the relay and the folder are kept alive by the caller.
pub async fn sams_computer(seed: u8) -> (Server, RelayConfig, tempfile::TempDir, Node) {
    let relay = harness::relay().await;
    let ours = RelayConfig {
        relay_urls: vec![harness::relay_url(&relay)],
        n0_services: false,
        loopback_only: true,
    };
    let dir = tempfile::tempdir().unwrap();
    let sam = Node::start(
        dir.path().to_path_buf(),
        SigningKey::from_bytes(&[seed; 32]),
        &ours,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("Sam's computer starts");
    tokio::time::timeout(Duration::from_secs(20), sam.endpoint().online())
        .await
        .expect("Sam's computer reaches the relay on 127.0.0.1");
    (relay, ours, dir, sam)
}
