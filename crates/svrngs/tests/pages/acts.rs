//! page-acts's test (main.rs): a line the phone's page left unsent, kept
//! in its store across a real reopen, is carried on the next open. The
//! rest of page-acts (d), on both pages, is web/tests/pages/again.test.mjs.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;
use tokio::io::AsyncWriteExt;

use crate::drive::drive;
use crate::harness::{relay, relay_url, Engine};

/// What ada's shelf says it asks of an app.
const ASKS: &str = "the page reads the shelf";

/// Ada's computer, `signing(1)`'s, holding house from its seed,
/// lending `page`, which reads kind `k`, and `shelf`, of
/// kind `k`, which asks, bo let in; her home page's address, and house's id.
async fn ada(dir: &Path, relay: &iroh_relay::server::Server) -> (Node, String, String) {
    use svrngs_node::group::{draw, Name};
    let key = SigningKey::from_bytes(&[1; 32]);
    let config = RelayConfig {
        relay_urls: vec![relay_url(relay)],
        n0_services: false,
        loopback_only: true,
    };
    let node = Node::start(
        dir.to_path_buf(),
        key.clone(),
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
        key.verifying_key(),
    );
    node.host().keep_group(&seed, None, &[]).unwrap();
    crate::admit(&node, &seed.id(), "bo", 7);
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    for (name, more) in [
        ("page", json!({ "reads": ["k"] })),
        ("shelf", json!({ "asks": ASKS })),
    ] {
        let mut body = json!({ "port": serving().await, "kind": "k", "limit": 3, "refused": [], "groups": [seed.id()] });
        body.as_object_mut()
            .unwrap()
            .extend(more.as_object().unwrap().clone());
        std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
    }
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let home = format!("http://svrngs.localhost:{}/", node.api_addr().port());
    (node, home, seed.id())
}

/// Bo's phone, in house through ada's computer (pages/index.rs
/// `bo_s_phone`): the page.mjs `driving` args for group `id`.
fn bo_s_phone(relay: &iroh_relay::server::Server, id: &str) -> Value {
    let token = svrngs_core::link::Token {
        issuer: SigningKey::from_bytes(&[1; 32]).verifying_key(),
        secret: [0; 16],
        relays: vec![relay_url(relay).parse().expect("a URL")],
    }
    .encode();
    let bo = hex::encode(SigningKey::from_bytes(&[7; 32]).to_bytes());
    json!({ "site": crate::harness::site(), "key": bo, "token": token, "id": id })
}

/// A page on a port of this host, each request answered with one.
async fn serving() -> u16 {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut head = [0u8; 8192];
                let _ = tokio::io::AsyncReadExt::read(&mut sock, &mut head).await;
                let html = "<!doctype html><title>lent</title>";
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

/// Each wrong one of `checks` said, so one run says all a page did not do.
fn none_wrong(checks: &[(bool, &str)], seen: &Value) {
    let wrong: Vec<&str> = (checks.iter())
        .filter_map(|(failed, what)| failed.then_some(*what))
        .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
}

in_webkit!(
    /// **A line left unsent is carried on the next open** (page-acts (d)).
    /// The seam is the claim: store.js's IndexedDB keeping the line across
    /// a real reopen, and empty once it is carried; the carry's decision is
    /// web/tests/pages/acts.test.mjs's. Bo's phone keeps house with a line
    /// of his own signed and left unsent, as a carry that failed leaves it
    /// (app.js@19b63b2b:506); the page opened again reads house, carries
    /// the line to ada's computer, and keeps none unsent. Watched failing on v1 (`UI` flipped), which
    /// carries unsent lines only at the next yes.
    a_line_left_unsent_is_carried_on_the_next_open
);
async fn a_line_left_unsent_is_carried_on_the_next_open(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, _, id) = ada(dir.path(), &relay).await;
    let payload = svrngs_core::Payload::new(json!({ "kind": "planted" })).unwrap();
    let act = svrngs_node::RailAct::Record { payload };
    let bo = SigningKey::from_bytes(&[7; 32]);
    let now = chrono::Local::now().timestamp();
    let line = svrngs_core::sign_act(&bo, &id, now, 0, None, act, None, None, None);
    let mut args = bo_s_phone(&relay, &id);
    let plant = json!({ "plant": line, "line": { "seq": 0, "id": line.id }, "carried": "PAGE: acted (1 line(s) unsent, and carried)", "within": 60_000 });
    args.as_object_mut()
        .unwrap()
        .extend(plant.as_object().unwrap().clone());
    let seen = drive(engine, "pages/acts.mjs", args, None).await;
    none_wrong(
        &[
            (seen["went"] != true, "the open carries the line"),
            (seen["unsent"] != 0, "and keeps none unsent"),
        ],
        &seen,
    );
    drop(ada);
}
