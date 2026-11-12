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
use tokio::io::AsyncWriteExt;

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

/// Ada's computer, holding a group from each name in `groups`, and her
/// home page's address.
async fn ada(dir: &Path, relay: &iroh_relay::server::Server, groups: &[&str]) -> (Node, String) {
    ada_in(dir, relay, &self::groups(groups)).await
}

/// [`ada`], holding `seeds`.
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

/// A page on a port of this host, each request answered with one titled
/// `title`, lent from `dir` as `name`.
async fn lends_a_page(dir: &Path, name: &str, title: &'static str) {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut head = [0u8; 8192];
                let _ = tokio::io::AsyncReadExt::read(&mut sock, &mut head).await;
                let html = format!("<!doctype html><title>{title}</title>");
                let answer = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n{html}",
                    html.len()
                );
                let _ = sock.write_all(answer.as_bytes()).await;
            });
        }
    });
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "port": port, "kind": "k", "limit": 3, "groups": svrngs::group::groups(dir).iter().map(|s| s.id()).collect::<Vec<_>>() });
    std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// The page's trace lines (`window.__lines`).
fn lines(seen: &Value) -> Vec<&str> {
    (seen["lines"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .collect()
}

/// Whether this host has a private address to open a lend to its WiFi on
/// (browser/main.rs's WiFi tests): could not judge, said, when it has none.
fn on_a_wifi() {
    match svrngs_node::network::route() {
        Some(std::net::IpAddr::V4(v4)) if v4.is_private() => {}
        other => panic!(
            "could not judge: this host's route leaves by {other:?}, no private address to serve a room on"
        ),
    }
}

in_chromium!(
    /// **Opened to this WiFi in one group's foot, said open in the other's**
    /// (page-home-rooms (a)'s fix). Ada is in house and flat, and lends
    /// `shelf`, so each group's foot lists it. She opens it to this WiFi in
    /// the first, and both lines say it is open, from ROOM's one answer,
    /// with no reload; she closes it in the second, and both say it is
    /// closed. Watched failing on v1 (`UI` flipped), whose line is drawn
    /// again in place alone (home.js@19b63b2b:77), the other left as it was read.
    opened_in_one_foot_is_said_open_in_every_list
);
async fn opened_in_one_foot_is_said_open_in_every_list(engine: Engine) {
    on_a_wifi();
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = ada(dir.path(), &relay, &["house", "flat"]).await;
    lends_a_page(dir.path(), "shelf", "ada's shelf").await;
    let row = "#home section ul.wifi li";
    let press = |nth: usize, words: &str| {
        json!([
            format!("#home section:nth-of-type({nth}) ul.wifi li button:has-text('{words}')"),
            null,
            true,
        ])
    };
    let args = json!({
        "ask": home,
        "all": row,
        "do": [press(1, "Open it to this WiFi"), press(2, "Close it to this WiFi")],
    });
    let seen = drive(engine, "home.mjs", args, None).await;
    let said = |i: usize| {
        (seen["after"][i].as_array().into_iter().flatten())
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
    };
    let (opened, closed) = (said(0), said(1));
    let open = "Open to this WiFi: type http://";
    let shut = "Closed to this WiFi: nobody on it can open it through this computer now.";
    let lines = lines(&seen);
    let wrong: Vec<&str> = [
        (
            seen["all"].as_array().map_or(0, Vec::len) != 2,
            "two groups' feet, each listing ada’s shelf",
        ),
        (
            opened.len() != 2 || !opened.iter().all(|t| t.contains(open)),
            "opened in the first foot, both lines say it is open",
        ),
        (
            closed.len() != 2 || !closed.iter().all(|t| t.contains(shut) && !t.contains(open)),
            "closed in the second, both say it is closed",
        ),
        (seen["loads"] != 1, "with no reload"),
        (
            !lines
                .iter()
                .any(|l| l.starts_with("ROOM: turn: read -> turning")),
            "each turn is ROOM's",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

in_chromium!(
    /// **A lend's address, copied twice, is said copied once**
    /// (page-home-rooms (a)'s second fix). Ada lends `shelf`; she presses
    /// "Copy its address for a program" in its line at her group's foot
    /// twice, and the line says it copied the address once, whether the
    /// clipboard took it or not. Watched failing on v1 (`UI` flipped),
    /// which appends what it says at each press (home.js@19b63b2b:202).
    a_copied_address_is_said_once
);
async fn a_copied_address_is_said_once(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = ada(dir.path(), &relay, &["house"]).await;
    lends_a_page(dir.path(), "shelf", "ada's shelf").await;
    let press = json!([
        "#home section ul.wifi li button:has-text('Copy its address for a program')",
        null,
        // The clipboard refuses here (web/tests/pages/index.mjs grants it
        // no permission), so the answer each press waits for is this one.
        "COPY: unwritten"
    ]);
    let args = json!({ "home": home, "read": "#home section ul.wifi li", "steps": [press, press] });
    let seen = drive(engine, "pages/index.mjs", args, None).await;
    let line = seen["after"][1]["words"][0].as_str().unwrap_or_default();
    let said = line.matches(" Copied: ").count() + line.matches(" Copy it from here: ").count();
    let wrong: Vec<&str> = [
        (said != 1, "said copied once, after two presses"),
        (
            seen["after"][1]["how"] != "settled",
            "each press answered by COPY",
        ),
        (seen["loads"] != 1, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
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
