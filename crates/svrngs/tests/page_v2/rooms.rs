//! page-home-rooms's v2-only tests (main.rs): what is open to this WiFi
//! and what has Play on, each one region on the home page, so a press in
//! one of its lists is drawn in every other, with no reload.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;
use tokio::io::AsyncWriteExt;

use crate::drive::drive;
use crate::harness::{relay, relay_url, Engine};

/// Ada's computer, on `crate::UI`, holding a group from each name in
/// `groups`, each from her own seed, and her home page's address.
async fn ada(dir: &Path, relay: &iroh_relay::server::Server, groups: &[&str]) -> (Node, String) {
    use svrngs_node::group::{draw, Name};
    let ada = SigningKey::from_bytes(&[1; 32]);
    let config = RelayConfig {
        relay_urls: vec![relay_url(relay)],
        n0_services: false,
        loopback_only: true,
    };
    let node = Node::start(
        dir.to_path_buf(),
        ada.clone(),
        &config,
        svrngs_node::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
    .ui(crate::UI);
    crate::drive::opens_at(node.api_addr().port(), &node.token());
    for group in groups {
        let seed = draw(
            Name::parse(group).unwrap(),
            Name::parse("ada").unwrap(),
            ada.verifying_key(),
        );
        node.host().keep_group(&seed, None, None).unwrap();
    }
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let home = format!("http://svrngs.localhost:{}/", node.api_addr().port());
    (node, home)
}

/// A page on a port of this host, each request answered with one titled
/// `title`, lent from `dir` as `name`.
async fn lends_a_page(dir: &Path, name: &str, title: &'static str) {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
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
    let body = json!({ "port": port, "kind": "k", "limit": 3 });
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
    /// again in place alone (home.js:77), the other left as it was read.
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
            !lines.iter().any(|l| l.starts_with("ROOM: turn: read -> turning")),
            "each turn is ROOM's",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}
