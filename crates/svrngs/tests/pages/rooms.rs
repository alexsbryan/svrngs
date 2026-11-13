//! page-home-rooms's tests (main.rs): what is open to this WiFi
//! and what has Play on, each one region on the home page, so a press in
//! one of its lists is drawn in every other, with no reload.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_node::group::Seed;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

use crate::drive::drive;
use crate::harness::{relay, relay_url, Engine};

/// A group of ada's, `signing(1)`'s, for each name in `names`, each from
/// a seed of its own.
fn groups(names: &[&str]) -> Vec<Seed> {
    use svrngs_node::group::{draw, Name};
    let ada = SigningKey::from_bytes(&[1; 32]).verifying_key();
    (names.iter())
        .map(|g| draw(Name::parse(g).unwrap(), Name::parse("ada").unwrap(), ada))
        .collect()
}

/// A computer of the key drawn from `me`, holding `seeds`, online at
/// `relay`.
async fn computer(dir: &Path, relay: &iroh_relay::server::Server, me: u8, seeds: &[Seed]) -> Node {
    let config = RelayConfig {
        relay_urls: vec![relay_url(relay)],
        n0_services: false,
        loopback_only: true,
    };
    let node = Node::start(
        dir.to_path_buf(),
        SigningKey::from_bytes(&[me; 32]),
        &config,
        svrngs_node::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    crate::drive::opens_at(node.api_addr().port(), &node.token());
    for seed in seeds {
        node.host().keep_group(seed, None, &[]).unwrap();
    }
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    node
}

/// Ada's computer, holding `seeds`, and her home page's address.
async fn ada_in(dir: &Path, relay: &iroh_relay::server::Server, seeds: &[Seed]) -> (Node, String) {
    let node = computer(dir, relay, 1, seeds).await;
    let home = format!("http://svrngs.localhost:{}/", node.api_addr().port());
    (node, home)
}

/// Cy's computer, the key drawn from 44, let into each of `seeds` by ada
/// and her record of each synced to it once, so ada's page lists what cy
/// lends in each group (browser/main.rs `joined`).
async fn cy(dir: &Path, relay: &iroh_relay::server::Server, ada: &Node, seeds: &[Seed]) -> Node {
    for seed in seeds {
        crate::admit(ada, &seed.id(), "cy", 44);
    }
    let node = computer(dir, relay, 44, seeds).await;
    for seed in seeds {
        let synced = node
            .host()
            .sync_with(&seed.id(), &ada.endpoint().addr())
            .await;
        assert!(synced.stop.is_none(), "cy syncs ada's record: {synced:?}");
    }
    let addrs = node.endpoint().addr().ip_addrs().copied().collect();
    ada.transport().add_known_peer(node.endpoint().id(), addrs);
    node
}

/// A UDP program on this host that answers each packet with itself, lent
/// from `dir` as `name`, a raw port (browser/play.rs `udp_echo`).
async fn lends_a_udp_port(dir: &Path, name: &str) {
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut buf = vec![0; 65_535];
        while let Ok((n, from)) = socket.recv_from(&mut buf).await {
            let _ = socket.send_to(&buf[..n], from).await;
        }
    });
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "port": port, "raw": "udp", "kind": "echo", "limit": 3, "groups": svrngs::group::groups(dir).iter().map(|s| s.id()).collect::<Vec<_>>() });
    std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// The page's trace lines (`window.__lines`).
fn lines(seen: &Value) -> Vec<&str> {
    (seen["lines"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .collect()
}

in_chromium!(
    /// **Play in one group's index, said on in the other's, and its Stop
    /// there drawn in the first** (page-home-rooms (b)'s fix). Cy is in
    /// ada's house and flat and lends `played`, a UDP port, so each group's
    /// index lists it with Play. Ada presses Play in house's, and flat's
    /// says "Connect to", from PLAY's one answer, with no reload; she
    /// presses Stop in flat's, and house's offers Play again. Watched
    /// failing on v1 (`UI` flipped), whose cell is drawn again in place
    /// alone (home.js@19b63b2b:91), the other left as it was read.
    play_in_one_index_is_said_on_in_every_list
);
async fn play_in_one_index_is_said_on_in_every_list(engine: Engine) {
    let relay = relay().await;
    let (ada_dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let seeds = groups(&["house", "flat"]);
    let (ada, home) = ada_in(ada_dir.path(), &relay, &seeds).await;
    let cy = cy(cy_dir.path(), &relay, &ada, &seeds).await;
    lends_a_udp_port(cy_dir.path(), "played").await;
    // The cell with Play or Stop, a port's alone.
    let row = "#home section table.index td:has(button)";
    // Each press read once the cell pressed says what it answered.
    let press = |nth: usize, words: &str, until: &str| {
        json!([
            format!("#home section:nth-of-type({nth}) table.index tr:has-text('played') button:has-text('{words}')"),
            null,
            true,
            until,
        ])
    };
    let args = json!({
        "ask": home,
        "all": row,
        "do": [press(1, "Play", "Connect to"), press(2, "Stop", "Play opens it")],
    });
    let seen = drive(engine, "home.mjs", args, None).await;
    let said = |i: usize| {
        (seen["after"][i].as_array().into_iter().flatten())
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
    };
    let (on, off) = (said(0), said(1));
    let at = "Connect to 127.0.0.1:";
    let lines = lines(&seen);
    let wrong: Vec<&str> = [
        (
            on.len() != 2 || !on.iter().all(|t| t.contains(at) && t.ends_with("Stop")),
            "Play in house's: both groups' cells say where to connect, with Stop",
        ),
        (
            off.len() != 2 || !off.iter().all(|t| !t.contains(at) && t.ends_with("Play")),
            "Stop in flat's, both offer Play again",
        ),
        (seen["loads"] != 1, "with no reload"),
        (
            !lines
                .iter()
                .any(|l| l.starts_with("PLAY: turn: read -> turning")),
            "each turn is PLAY's",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop((ada, cy));
}
