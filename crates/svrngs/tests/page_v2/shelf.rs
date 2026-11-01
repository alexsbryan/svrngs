//! page-shelf's v2-only tests (main.rs): the bring on both pages, its
//! answer beside the button, the button held while it brings, and the
//! group read again after it.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_core::link::Token;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, RailAct, SigningKey};

use crate::drive::drive;
use crate::harness::{relay, relay_url, site_of, Engine};

/// Ada's computer holding house from its seed, as production holds a group
/// (browser/main.rs `house`), bo let in, and house's own folder kept, where
/// the shelf's apps are brought (carried.rs `brought`); on `crate::UI`.
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
    let bo = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("bo").unwrap(),
            key: actor_of(&SigningKey::from_bytes(&[7; 32])),
            invite: None,
        },
    };
    node.rail().commit(&seed.id(), &bo).unwrap().unwrap();
    let folder = dir.join("house");
    std::fs::create_dir(&folder).unwrap();
    let ask = svrngs_node::folder::FolderAsk {
        dir: folder,
        group: seed.id(),
    };
    let kept = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("http://{}/v1/folders", node.api_addr()))
        .bearer_auth(node.token())
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&ask).unwrap())
        .send()
        .await
        .unwrap();
    assert!(kept.status().is_success(), "{:?}", kept.text().await);
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    (node, seed.id())
}

/// What `seen` (web/tests/v2/shelf.mjs) shows of the bring, each wrong
/// one said, so one run says all a page did not do.
fn brought_on_v2(seen: &Value) {
    let next = seen["next"].as_str().unwrap_or_default();
    let wrong: Vec<&str> = [(
        !next.contains("You brought Money to house: everyone in it can open it in a few seconds."),
        "the answer is the button's next sibling",
    )]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
}

in_webkit!(
    /// **The bring on a phone, on v2** (page-shelf (b)). Bo's phone, in
    /// house through ada's computer, ticks Money and brings it. Watched
    /// failing on v1 (`UI` flipped), whose answer replaces the words above
    /// the boxes (question.js:207).
    the_phone_s_bring_is_answered_beside_its_button
);
async fn the_phone_s_bring_is_answered_beside_its_button(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, id) = house(dir.path(), &relay).await;
    let token = Token {
        issuer: SigningKey::from_bytes(&[1; 32]).verifying_key(),
        secret: [0; 16],
        relay: Some(relay_url(&relay).parse().expect("a URL")),
    }
    .encode();
    let bo = hex::encode(SigningKey::from_bytes(&[7; 32]).to_bytes());
    let args = json!({ "site": site_of(crate::UI), "key": bo, "token": token, "id": id, "apps": ["Money"], "table": "Money is on the table." });
    let seen = drive(engine, "v2/shelf.mjs", args, None).await;
    brought_on_v2(&seen);
    drop(ada);
}

in_chromium!(
    /// **The bring on the home page, on v2** (page-shelf (b)): ada's home
    /// page, Money ticked and brought, as on the phone. Watched failing
    /// on v1 (`UI` flipped), as on the phone.
    the_home_page_s_bring_is_answered_beside_its_button
);
async fn the_home_page_s_bring_is_answered_beside_its_button(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, _) = house(dir.path(), &relay).await;
    let home = format!("http://svrngs.localhost:{}/", ada.api_addr().port());
    let args = json!({ "home": home, "apps": ["Money"], "table": "Money is on the table." });
    let seen = drive(engine, "v2/shelf.mjs", args, None).await;
    brought_on_v2(&seen);
    drop(ada);
}
