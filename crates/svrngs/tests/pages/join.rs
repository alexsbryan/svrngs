//! page-join's tests (main.rs): a group joined by the running svrngs,
//! not the page, is drawn when the page is shown again. A group joined or
//! made from the page, drawn with no reload and said read until it is, are
//! the home page twin's laws (web/tests/pages/home.test.mjs).

use std::path::Path;
use std::time::Duration;

use serde_json::json;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

use crate::drive::drive;
use crate::harness::{relay, relay_url, Engine};

/// Ada's computer holding house from its seed (browser/main.rs `house`),
/// and her home page's address.
async fn house(dir: &Path, relay: &iroh_relay::server::Server) -> (Node, String) {
    let (node, home, _) = computer(dir, relay, 1, "house", "ada").await;
    (node, home)
}

/// The computer of key `me`, holding `group`, made by `person`: it, its
/// home page's address, and the group's id.
async fn computer(
    dir: &Path,
    relay: &iroh_relay::server::Server,
    me: u8,
    group: &str,
    person: &str,
) -> (Node, String, String) {
    use svrngs_node::group::{draw, Name};
    let ada = SigningKey::from_bytes(&[me; 32]);
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
    .expect("a node starts");
    crate::drive::opens_at(node.api_addr().port(), &node.token());
    let seed = draw(
        Name::parse(group).unwrap(),
        Name::parse(person).unwrap(),
        ada.verifying_key(),
    );
    node.host().keep_group(&seed, None, &[]).unwrap();
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let home = format!("http://svrngs.localhost:{}/", node.api_addr().port());
    (node, home, seed.id())
}

in_chromium!(
    /// **A group joined by the running svrngs, not the page, is on the
    /// page when it is shown again** (join-while-running): ada's home page
    /// is open on house; through her running svrngs — asked as `svrngs
    /// join` now asks it, at the record's `/v1/joins` — she joins Sam's
    /// Trip as Dee; the page, hidden and shown again as a person coming
    /// back to its tab, reads its groups and draws Trip's section under
    /// house's, each of its own, with no reload. Planted, watched failing
    /// with the join made after the page was shown again: the page that
    /// read its groups once drew house alone.
    a_group_joined_by_the_running_svrngs_is_on_the_page_when_shown_again
);
async fn a_group_joined_by_the_running_svrngs_is_on_the_page_when_shown_again(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = house(dir.path(), &relay).await;
    let sam_dir = tempfile::tempdir().unwrap();
    let (sam, _, trip) = computer(sam_dir.path(), &relay, 2, "Trip", "Sam").await;
    let made = sam
        .host()
        .link(&trip, false, None, None)
        .expect("Sam makes a link to Trip");
    let link = svrngs_node::link::client_link(&made.token);
    let ready = dir.path().join("ready");
    let args = json!({
        "home": home,
        "read": "#home section h2",
        "steps": [],
        "ready": ready,
        "visible": ["HOME: visible:", "Trip"],
    });
    let (send, release) = tokio::sync::oneshot::channel::<()>();
    let driven = tokio::spawn(drive(engine, "pages/index.mjs", args, Some(release)));
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(90), async {
            while !ready.exists() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
        .await
        .is_ok(),
        "ada's page shows house within 90 s"
    );
    // Through the running svrngs, as `svrngs join` asks it: the join.
    let token = std::fs::read_to_string(dir.path().join(svrngs_node::api::TOKEN_FILE)).unwrap();
    let asked = serde_json::json!({ "link": link, "person": "Dee" });
    let joined: serde_json::Value = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("http://{}/v1/joins", ada.api_addr()))
        .header("content-type", "application/json")
        .bearer_auth(token.trim())
        .body(asked.to_string())
        .send()
        .await
        .expect("the running svrngs answers a join")
        .json()
        .await
        .unwrap_or(serde_json::Value::Null);
    assert_eq!(
        joined["outcome"], "in",
        "the running svrngs let ada in: {joined}"
    );
    send.send(()).expect("the page is shown again");
    let seen = driven.await.expect("the page was driven");
    let headings: Vec<String> = seen["later"]["words"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|w| w.as_str().unwrap_or_default().to_string())
        .collect();
    let wrong: Vec<&str> = [
        (
            !headings.iter().any(|h| h.contains("house")),
            "house still drawn",
        ),
        (
            !headings.iter().any(|h| h.contains("Trip")),
            "Trip drawn after the show",
        ),
        (seen["later"]["how"] != "settled", "the read settled"),
        (seen["loads"] != 1, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop((ada, sam));
}
