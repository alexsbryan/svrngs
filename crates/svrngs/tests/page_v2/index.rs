//! page-index's v2-only tests (main.rs): the index and "What you lend"
//! drawn again from each read, after the page's own writes, with no reload.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;
use tokio::io::AsyncWriteExt;

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

/// A page at a port from `from` up, inside /running's look (browser/main.rs
/// `free_here`), answering each request with a page titled `title`.
async fn serving(from: u16, title: &'static str) -> u16 {
    let listener = 'free: {
        for port in from..30000 {
            if let Ok(l) = tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
                break 'free l;
            }
        }
        panic!("no port free from {from} on this host");
    };
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
    port
}

/// The page's trace lines (`window.__lines`).
fn lines(seen: &Value) -> Vec<&str> {
    (seen["lines"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .collect()
}

in_chromium!(
    /// **A lend from "Already running here" is in the index at once**
    /// (page-index (d)'s fix). Something listens on a port of ada's
    /// computer; her home page looks, she names it notes and says it is
    /// notes, and lends it: the row says she lent it, and the group's
    /// index lists "ada’s notes" and "What you lend" lists it too, from
    /// HOME's `again`, with no reload. Watched failing on v1 (`UI`
    /// flipped), whose lend redraws only "What you lend" (home.js:247).
    a_lend_from_what_runs_here_is_in_the_index_at_once
);
async fn a_lend_from_what_runs_here_is_in_the_index_at_once(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = house(dir.path(), &relay).await;
    let port = serving(21000, "notes").await;
    let row = format!("#running li:has-text('localhost:{port}')");
    let seen = drive(
        engine,
        "home.mjs",
        json!({
            "ask": home,
            "all": "#running li, #running p",
            "do": [["#running button:has-text('Look for programs')", null, true],
                [format!("{row} input[name=name]"), "notes"], [format!("{row} input[name=kind]"), "notes"],
                [format!("{row} button"), null, true, "You lent notes"]],
            "after": ["#home table.index", "ada’s notes"],
            "text": true,
        }),
        None,
    )
    .await;
    let later = seen["later"].to_string();
    let text = seen["text"].as_str().unwrap_or_default();
    let lines = lines(&seen);
    let at = |start: &str| lines.iter().position(|l| l.starts_with(start));
    let wrong: Vec<&str> = [
        (
            !later.contains("ada’s notes"),
            "the index lists ada’s notes",
        ),
        (
            !text.contains("notes: everyone in your groups at once"),
            "What you lend lists it",
        ),
        (seen["loads"] != 1, "with no reload"),
        (
            !(at("LEND: lent") < at("HOME: again: ") && at("LEND: lent").is_some()),
            "read again by HOME's again, after the lend",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

/// Ada's computer lending `shelf`, a page of its own, as `body` declares it
/// (browser/main.rs's lend tests), and her home page's address.
async fn lending(dir: &Path, relay: &iroh_relay::server::Server, body: Value) -> (Node, String) {
    let (ada, home) = house(dir, relay).await;
    let mut body = body;
    body["port"] = json!(serving(22000, "ada's shelf").await);
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    std::fs::write(lends.join("shelf.json"), body.to_string()).unwrap();
    (ada, home)
}

/// What web/tests/v2/index.mjs read after step `i`, its first line.
fn first(seen: &Value, i: usize) -> String {
    seen["after"][i]["words"][0]
        .as_str()
        .unwrap_or_default()
        .into()
}

in_chromium!(
    /// **Three changes to what ada lends, each read again in place**
    /// (page-index (e)'s fix). Ada lends `shelf` to one person at once;
    /// she sets four, then everyone, then stops lending it, and after each
    /// "What you lend" is read again through HOME's `again`, never a load
    /// of the page. Watched failing on v1 (`UI` flipped), which loads the
    /// page again after each (home.js:283).
    three_changes_to_a_lend_are_read_again_in_place
);
async fn three_changes_to_a_lend_are_read_again_in_place(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let body = json!({ "kind": "books", "limit": 1 });
    let (ada, home) = lending(dir.path(), &relay, body).await;
    let press = |words: &str| {
        json!([
            format!("#lent button:has-text('{words}')"),
            null,
            "LENDING: answered"
        ])
    };
    let steps = json!([
        ["#lent input", "4"],
        press("Let in this many"),
        press("Let everyone in"),
        press("Stop lending")
    ]);
    let args = json!({ "home": home, "read": "#lent li, #lent p", "steps": steps });
    let seen = drive(engine, "v2/index.mjs", args, None).await;
    let wrong: Vec<&str> = [
        (
            !first(&seen, 0).starts_with("shelf: up to 4 people at once"),
            "four, read again",
        ),
        (
            !first(&seen, 1).starts_with("shelf: everyone in your groups at once"),
            "everyone, read again",
        ),
        (
            seen["after"][2]["words"] != json!(["You lend nothing yet."]),
            "stopped, read again",
        ),
        (seen["loads"] != 1, "with no load of the page"),
        (
            !(seen["after"].as_array().into_iter().flatten()).all(|a| a["how"] == "settled"),
            "each read again by LENDING",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

in_chromium!(
    /// **The secret stays shown when the page is read again** (page-index
    /// (e)). Ada turns on signing people in with svrngs for `shelf`; its
    /// secret is shown in the row, then the page is hidden and shown, and
    /// read again: the row now says it signs people in, its button gone,
    /// and still shows the secret, which no request, link or trace line
    /// carries. Watched failing on v1 (`UI` flipped), which never reads
    /// again on coming back.
    the_secret_stays_shown_when_the_page_is_read_again
);
async fn the_secret_stays_shown_when_the_page_is_read_again(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = lending(dir.path(), &relay, json!({ "kind": "books" })).await;
    let sign = json!([
        "#lent button:has-text('Sign people in with svrngs')",
        null,
        "CHANGE: signed"
    ]);
    let args = json!({ "home": home, "read": "#lent li", "steps": [sign], "visible": true });
    let seen = drive(engine, "v2/index.mjs", args, None).await;
    let name = svrngs_node::lend::LendName::parse("shelf").unwrap();
    let secret = (svrngs_node::lend::read(dir.path(), &name).unwrap().unwrap())
        .sign_in_with_svrngs
        .expect("the declaration holds the client")
        .client_secret
        .expect("a secret");
    let later = seen["later"]["words"][0].as_str().unwrap_or_default();
    let wrong: Vec<&str> = [
        (!first(&seen, 0).contains(&secret), "shown once turned on"),
        (
            seen["later"]["how"] != "settled",
            "read again on coming back",
        ),
        (
            !later.contains("Signs people in with svrngs.")
                || later.contains("Sign people in with svrngs"),
            "said as on, its button gone",
        ),
        (!later.contains(&secret), "the secret still shown"),
        (
            [&seen["sent"], &seen["links"], &seen["lines"]]
                .iter()
                .any(|v| v.to_string().contains(&secret)),
            "no request, link or trace line carries it",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}
