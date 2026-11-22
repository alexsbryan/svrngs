#![allow(clippy::disallowed_methods, reason = "a test dials what it runs")]
//! Direct connections on this WiFi (webrtc-network; DESIGN §3): the home
//! page turns them on for the network this computer is on now, and the
//! file keeps them, so svrngs answers an offer on that network alone. The
//! offer's own gate is judged in `src/webrtc/tests.rs`; this asks the
//! route as the home page asks it, on a host whose network can be read
//! (could-not-judge otherwise).

use std::path::Path;

use serde_json::{json, Value};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::network::{self, Network};
use svrngs_node::node::Node;
use svrngs_node::webrtc::direct::DIRECT_FILE;
use svrngs_node::SigningKey;

fn no_seed(_: &Path) -> Result<Option<String>, String> {
    Ok(None)
}

async fn start(dir: &Path) -> Node {
    Node::start(
        dir.to_path_buf(),
        SigningKey::from_bytes(&[71; 32]),
        &RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: true,
        },
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// The setting on `node`, as `GET /v1/direct` says it, as the home page
/// reads it.
async fn on(node: &Node) -> Value {
    let (_, text) = node.ask_as_you("/v1/direct", None).await;
    serde_json::from_str(&text).unwrap_or_else(|_| panic!("the setting said {text:?}"))
}

/// The setting on `node` after `body`, as `POST /v1/direct`: its status
/// and its answer, as the home page's button posts it.
async fn asked(node: &Node, body: Value) -> (u16, Value) {
    let (status, text) = node.ask_as_you("/v1/direct", Some(body)).await;
    let body = serde_json::from_str(&text)
        .unwrap_or_else(|_| panic!("the setting answered {status}, not JSON: {text:?}"));
    (status, body)
}

/// This host's network, as the route reads it (`network::here`):
/// could-not-judge on a host with none.
fn here() -> Network {
    network::here().network.unwrap_or_else(|why| {
        panic!("could not judge: this host's network cannot be read ({why:?})")
    })
}

/// **The lender turns direct connections on for this network, and the file
/// keeps them** (webrtc-network; the operator's UDP decision, 2026-11-19):
/// off before the first press, on after it, the network's own name in
/// `direct.json`, and off again. Planted, watched red: a press writing
/// nothing (`turned` keeping `vec![]`), and the read-back said off.
#[tokio::test]
async fn the_lender_turns_direct_connections_on_for_this_network_and_the_file_keeps_them() {
    let dir = tempfile::tempdir().unwrap();
    let node = start(dir.path()).await;
    assert_eq!(on(&node).await["on"].as_bool(), Some(false));
    let (status, turned) = asked(&node, json!({ "on": true })).await;
    assert_eq!(
        (status, turned["on"].as_bool()),
        (200, Some(true)),
        "{turned}"
    );
    assert_eq!(on(&node).await["on"].as_bool(), Some(true));
    let kept: Vec<Network> =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join(DIRECT_FILE)).unwrap())
            .unwrap();
    assert_eq!(kept, vec![here()]);
    let (status, off) = asked(&node, json!({ "on": false })).await;
    assert_eq!((status, off["on"].as_bool()), (200, Some(false)), "{off}");
    node.net().close().await;
}
