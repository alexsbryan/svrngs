//! The record over loopback (DESIGN §3, §8 U3's inventory, the u3-2 row): an
//! app on a member's computer appends to its group, reads the log and the
//! people, on a listener bound to 127.0.0.1, and nothing else reaches it.
//! Nodes in one process, no relay; peers are reached by seeded addresses,
//! as `group_not_mesh.rs` does.

use std::net::SocketAddr;

use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::{Node, NodeError};
use svrngs_node::SigningKey;

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
