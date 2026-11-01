//! page-join's v2-only tests (main.rs): on the home page, what is
//! refused is said once.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

use crate::drive::drive;
use crate::harness::{relay, relay_url, Engine};

/// Ada's computer holding house from its seed (browser/main.rs `house`),
/// on `crate::UI`, and her home page's address.
async fn house(dir: &Path, relay: &iroh_relay::server::Server) -> (Node, String) {
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
    let seed = draw(
        Name::parse("house").unwrap(),
        Name::parse("ada").unwrap(),
        ada.verifying_key(),
    );
    node.host().keep_group(&seed, None, None).unwrap();
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let home = format!("http://svrngs.localhost:{}/", node.api_addr().port());
    (node, home)
}

/// `press` pressed twice on ada's home page, each read from `read` once
/// the page traced a line starting with `line` (web/tests/v2/index.mjs,
/// page-index's driver), after `fill` typed: how many times each read
/// says `words`.
async fn twice(
    engine: Engine,
    fill: Value,
    press: &str,
    read: &str,
    line: &str,
    words: &str,
) -> (Vec<usize>, Value) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = house(dir.path(), &relay).await;
    let press = json!([press, null, line]);
    let steps = [fill, press.clone(), press]
        .into_iter()
        .filter(|s| !s.is_null())
        .collect::<Vec<_>>();
    let args = json!({ "home": home, "read": read, "steps": steps });
    let seen = drive(engine, "v2/index.mjs", args, None).await;
    let said = (seen["after"].as_array().into_iter().flatten())
        .map(|a| a["words"].to_string().matches(words).count())
        .collect();
    drop(ada);
    (said, seen)
}

in_chromium!(
    /// **A group not made is said once** (page-join (d)'s first fix). Ada
    /// presses "Make a group" twice with no name: the node refuses each,
    /// and the section says why once, beside the button, the second in the
    /// first's place. Watched failing on v1 (`UI` flipped), which appends
    /// it at each press (home.js:421).
    a_group_not_made_is_said_once
);
async fn a_group_not_made_is_said_once(engine: Engine) {
    let (said, seen) = twice(
        engine,
        Value::Null,
        "#make button:has-text('Make a group')",
        "#make p",
        "MAKE: refused",
        "a name cannot be empty",
    )
    .await;
    assert_eq!(said, [1, 1], "said once, at each press: {seen:#}");
}

in_chromium!(
    /// **A link to nothing lent is said once** (page-join (f)'s fix). Ada
    /// pastes a link that opens nothing lent in her groups and presses
    /// "Open it" twice: "That is not a link to something lent in your
    /// groups." is said once. Watched failing on v1 (`UI` flipped), which
    /// appends it at each press (home.js:397).
    a_link_to_nothing_lent_is_said_once
);
async fn a_link_to_nothing_lent_is_said_once(engine: Engine) {
    let fill = json!(["#open input", "http://page.example/#open=no-such"]);
    let words = "That is not a link to something lent in your groups.";
    let (said, seen) = twice(
        engine,
        fill,
        "#open button:has-text('Open it')",
        "#open p",
        "OPEN: open",
        words,
    )
    .await;
    assert_eq!(said, [1, 1], "said once, at each press: {seen:#}");
}
