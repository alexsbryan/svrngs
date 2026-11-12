//! page-shelf's tests (main.rs): the bring on both pages, its
//! answer beside the button, the button held while it brings, and the
//! group read again after it.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_core::link::Token;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

use crate::drive::drive;
use crate::harness::{relay, relay_url, site, Engine};

/// Ada's computer holding house from its seed, as production holds a group
/// (browser/main.rs `house`), bo let in, and house's own folder kept, where
/// the shelf's apps are brought (carried.rs `brought`).
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
    .expect("a node starts");
    crate::drive::opens_at(node.api_addr().port(), &node.token());
    let seed = draw(
        Name::parse("house").unwrap(),
        Name::parse("ada").unwrap(),
        ada.verifying_key(),
    );
    node.host().keep_group(&seed, None, &[]).unwrap();
    crate::admit(&node, &seed.id(), "bo", 7);
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

/// Whether the page asked its group again after SHELF's `brought`: the
/// phone's `reread` (web/page/enter.js), the home's `again`, HOME's then
/// each GROUP's (home/home.js, group.js).
fn read_after(lines: &[&str], home: bool) -> bool {
    let at = |start: &str| lines.iter().position(|l| l.starts_with(start));
    let asked: &[&str] = if home {
        &["HOME: again: ", "GROUP: again: "]
    } else {
        &["PAGE: reread: "]
    };
    let brought = at("SHELF: brought (");
    brought.is_some() && asked.iter().all(|a| at(a) > brought)
}

/// What `seen` (web/tests/pages/shelf.mjs) shows of the bring, each wrong
/// one said, so one run says all a page did not do. The button pressed
/// twice at once: held while it brings, then given back, and the second
/// press refused by SHELF, so the home page posts /shelf once; then
/// Money, brought, offered no more, and the group read again, which says
/// Money is on the table, with no reload.
fn brought_on_v2(seen: &Value, home: bool) {
    let next = seen["next"].as_str().unwrap_or_default();
    let lines: Vec<&str> = (seen["lines"].as_array().into_iter().flatten())
        .filter_map(|l| l.as_str())
        .collect();
    let traced = |start: &str| lines.iter().filter(|l| l.starts_with(start)).count();
    let posts = (seen["sent"].as_array().into_iter().flatten())
        .filter(|r| *r == "POST /shelf")
        .count();
    let wrong: Vec<&str> = [
        (
            !next.contains(
                "You brought Money to house: everyone in it can open it in a few seconds.",
            ),
            "the answer is the button's next sibling",
        ),
        (
            seen["busy"] != json!([true, false]),
            "the button is held while it brings, then given back",
        ),
        (
            traced("SHELF: bring: idle -> bringing") != 1
                || traced("SHELF: bring refused in bringing") != 1,
            "the second press is refused while the first brings",
        ),
        (home && posts != 1, "the home page posts /shelf once"),
        (
            !(seen["table"] == true && seen["reloads"] == 0 && read_after(&lines, home)),
            "then the group is read again, Money on the table, with no reload",
        ),
        (
            seen["shelf"]
                .as_str()
                .unwrap_or_default()
                .contains("Money: who owes"),
            "Money, brought, is offered no more",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
}

in_webkit!(
    /// **The bring on a phone, on v2** (page-shelf (b)). Bo's phone, in
    /// house through ada's computer, ticks Money and brings it. Watched
    /// failing on v1 (`UI` flipped), whose answer replaces the words above
    /// the boxes (question.js@19b63b2b:207).
    the_phone_s_bring_is_answered_beside_its_button
);
async fn the_phone_s_bring_is_answered_beside_its_button(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, id) = house(dir.path(), &relay).await;
    let token = Token {
        issuer: SigningKey::from_bytes(&[1; 32]).verifying_key(),
        secret: [0; 16],
        relays: vec![relay_url(&relay).parse().expect("a URL")],
    }
    .encode();
    let bo = hex::encode(SigningKey::from_bytes(&[7; 32]).to_bytes());
    let args = json!({ "site": site(), "key": bo, "token": token, "id": id, "apps": ["Money"], "table": "Money is on the table." });
    let seen = drive(engine, "pages/shelf.mjs", args, None).await;
    brought_on_v2(&seen, false);
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
    let seen = drive(engine, "pages/shelf.mjs", args, None).await;
    brought_on_v2(&seen, true);
    drop(ada);
}
