//! page-index's tests (main.rs): the phone's index drawn again from each
//! read, after a bring and on coming back, with no reload. The home page's
//! index and "What you lend", read again after its own writes, are the
//! home page twin's laws (web/tests/pages/again.test.mjs).

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
/// her home page's address, and the group's id.
async fn house(dir: &Path, relay: &iroh_relay::server::Server) -> (Node, String, String) {
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
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let home = format!("http://svrngs.localhost:{}/", node.api_addr().port());
    (node, home, seed.id())
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

/// Ada's computer lending `shelf`, a page of its own, as `body` declares it
/// (browser/main.rs's lend tests), her home page's address, and the
/// group's id.
async fn lending(
    dir: &Path,
    relay: &iroh_relay::server::Server,
    body: Value,
) -> (Node, String, String) {
    let (ada, home, id) = house(dir, relay).await;
    let mut body = body;
    body["port"] = json!(serving(22000, "ada's shelf").await);
    body["groups"] = json!([id]);
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    std::fs::write(lends.join("shelf.json"), body.to_string()).unwrap();
    (ada, home, id)
}

/// What web/tests/pages/index.mjs read after step `i`, its first line.
fn first(seen: &Value, i: usize) -> String {
    seen["after"][i]["words"][0]
        .as_str()
        .unwrap_or_default()
        .into()
}

/// Bo's phone, in house through ada's computer (pages/shelf.rs): the
/// page.mjs `driving` args for group `id`, reading `read`.
fn bo_s_phone(relay: &iroh_relay::server::Server, id: &str, read: &str) -> Value {
    let token = svrngs_core::link::Token {
        issuer: SigningKey::from_bytes(&[1; 32]).verifying_key(),
        secret: [0; 16],
        relays: vec![relay_url(relay).parse().expect("a URL")],
    }
    .encode();
    let bo = hex::encode(SigningKey::from_bytes(&[7; 32]).to_bytes());
    json!({ "site": crate::harness::site(), "key": bo, "token": token, "id": id, "read": read })
}

/// House as `house` holds it, bo let in, and house's own folder kept,
/// where the shelf's apps are brought (pages/shelf.rs `house`).
async fn kept(dir: &Path, relay: &iroh_relay::server::Server) -> (Node, String, String) {
    let (node, home, id) = house(dir, relay).await;
    crate::admit(&node, &id, "bo", 7);
    let folder = dir.join("house");
    std::fs::create_dir(&folder).unwrap();
    let ask = svrngs_node::folder::FolderAsk {
        dir: folder,
        group: id.clone(),
    };
    let asked = reqwest::Client::builder()
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
    assert!(asked.status().is_success(), "{:?}", asked.text().await);
    (node, home, id)
}

in_webkit!(
    /// **A bring is in the phone's index with no reload** (page-index
    /// (c)). Bo's phone, in house through ada's computer, ticks Money and
    /// brings it: `#lent` lists Money with the page loaded once, read
    /// again by the page's `reread`. Watched failing on v1 (`UI` flipped).
    a_bring_is_in_the_phone_s_index_with_no_reload
);
async fn a_bring_is_in_the_phone_s_index_with_no_reload(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, _, id) = kept(dir.path(), &relay).await;
    let mut args = bo_s_phone(&relay, &id, "#lent tr");
    args["steps"] = json!([
        ["#shelf label:has-text('Money') input", true],
        [
            "#shelf button:has-text('Bring the ticked ones')",
            null,
            "PAGE: reread: ",
            "Money"
        ]
    ]);
    let seen = drive(engine, "pages/index.mjs", args, None).await;
    let wrong: Vec<&str> = [
        (
            !first(&seen, 0).starts_with("Money"),
            "the index lists Money",
        ),
        (
            seen["after"][0]["how"] != "settled",
            "read again by the page",
        ),
        (seen["loads"] != 1, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

in_webkit!(
    /// **A lend filled while shown reads "Full now." on coming back**
    /// (page-index (c)). Ada lends `shelf` to one person at once; bo's
    /// phone shows it, open to him. While it is shown, cy opens it from
    /// his own computer, through ada's door, so it is full (door/limit.rs
    /// `ask`, counted for `WINDOW`); bo's phone is hidden and shown again,
    /// as he comes back to its tab, and reads `shelf` "Full now.", with
    /// the page loaded once. Watched failing on v1 (`UI` flipped).
    a_lend_filled_while_shown_reads_full_now_on_coming_back
);
async fn a_lend_filled_while_shown_reads_full_now_on_coming_back(engine: Engine) {
    use svrngs_node::iroh::Contact;
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let body = json!({ "kind": "books", "limit": 1 });
    let (ada, _, id) = lending(dir.path(), &relay, body).await;
    crate::admit(&ada, &id, "bo", 7);
    crate::admit(&ada, &id, "cy", 8);
    let ready = dir.path().join("ready");
    let mut args = bo_s_phone(&relay, &id, "#lent tr");
    args["steps"] = json!([]);
    args["ready"] = json!(ready);
    args["visible"] = json!(["PAGE: read", "Full now."]);
    let (go, release) = tokio::sync::oneshot::channel();
    let phone = tokio::spawn(drive(engine, "pages/index.mjs", args, Some(release)));
    tokio::time::timeout(Duration::from_secs(60), async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("bo's phone shows what house lends");
    // Cy's computer, dialling ada's over loopback, which the door counts
    // as away (network.rs `guess`).
    let cy_dir = tempfile::tempdir().unwrap();
    let cy = Node::start(
        cy_dir.path().to_path_buf(),
        SigningKey::from_bytes(&[8; 32]),
        &RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: true,
        },
        |_| Ok(None),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("cy's node starts");
    let to = Contact {
        key: ada.endpoint().id(),
        relays: Default::default(),
        iroh_direct_addrs: (ada.endpoint().bound_sockets().into_iter())
            .map(|mut a| {
                if a.ip().is_unspecified() {
                    a.set_ip(
                        if a.is_ipv4() { "127.0.0.1" } else { "::1" }
                            .parse()
                            .unwrap(),
                    );
                }
                a
            })
            .collect(),
    };
    let base = cy.transport().spliced(&to).await.expect("cy reaches ada");
    let opened = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .get(format!("{base}/lend/shelf/"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        opened.status(),
        200,
        "cy opens shelf: {:?}",
        opened.text().await
    );
    go.send(()).unwrap();
    let seen = phone.await.unwrap();
    let shown = seen["shown"].to_string();
    let later = seen["later"]["words"].to_string();
    let wrong: Vec<&str> = [
        (
            !shown.contains("shelf") || shown.contains("Full now."),
            "shelf shown, open to bo",
        ),
        (
            seen["later"]["how"] != "settled" || !later.contains("Full now."),
            "full now, on coming back",
        ),
        (seen["loads"] != 1, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop((ada, cy));
}
