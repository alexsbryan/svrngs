//! page-acts's v2-only tests (main.rs): what v2 does of "What acts as
//! you" and the ask that v1 does not, each part of (d) its own test.

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

/// The label of ada's lend `name` (browser/main.rs `label`).
fn label(name: &str) -> String {
    let ada = SigningKey::from_bytes(&[1; 32]).verifying_key();
    svrngs_core::pass::Label::of(ada.as_bytes(), name).to_string()
}

/// Ada's computer, `signing(1)`'s, holding house from its seed, on
/// `crate::UI`, lending `page`, which reads kind `k`, and `shelf`, of
/// kind `k`, which asks; and her home page's address.
async fn ada(dir: &Path, relay: &iroh_relay::server::Server) -> (Node, String) {
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
    .expect("a node starts")
    .ui(crate::UI);
    crate::drive::opens_at(node.api_addr().port(), &node.token());
    let seed = draw(
        Name::parse("house").unwrap(),
        Name::parse("ada").unwrap(),
        key.verifying_key(),
    );
    node.host().keep_group(&seed, None, None).unwrap();
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    for (name, more) in [("page", json!({ "reads": ["k"] })), ("shelf", json!({ "asks": ASKS }))] {
        let mut body = json!({ "port": serving().await, "kind": "k", "limit": 3, "refused": [] });
        body.as_object_mut().unwrap().extend(more.as_object().unwrap().clone());
        std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
    }
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let home = format!("http://svrngs.localhost:{}/", node.api_addr().port());
    (node, home)
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

in_chromium!(
    /// **Two refusals on /ask are said once** (page-acts (d)). Ada's home
    /// page asks whether her page may use her shelf; her session lapses
    /// (its cookies cleared), and she presses Yes, then, once that is
    /// refused, Yes again: the section says the refusal once, the second
    /// in place of the first. Watched failing on v1 (`UI` flipped), whose
    /// `ask` appends each refusal (home.js:343).
    two_refusals_on_ask_are_said_once
);
async fn two_refusals_on_ask_are_said_once(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = ada(dir.path(), &relay).await;
    let ask = format!("{home}ask?lend={}&app={}&back=/", label("shelf"), label("page"));
    let yes = "#home section button:text-is('Yes')";
    let press = |n: u8| json!({ "at": yes, "traced": ["ASK: unsaid", n] });
    let args = json!({ "home": ask, "shown": "#home section h2", "forget": true, "press": [press(1), press(2)], "at": "#home section", "within": 15_000 });
    let seen = drive(engine, "v2/acts.mjs", args, None).await;
    let texts: Vec<&str> = (seen["texts"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .collect();
    let first = texts.first().and_then(|t| t.split("Not now").nth(1)).unwrap_or_default();
    none_wrong(
        &[
            (seen["went"] != json!([true, true]), "each Yes is refused"),
            (first.trim().is_empty(), "the refusal is said after the buttons"),
            (
                texts.len() != 2 || texts[0] != texts[1],
                "the second refusal said in place of the first",
            ),
            (seen["reloads"] != 0, "with no reload"),
        ],
        &seen,
    );
    drop(ada);
}

/// Ada's yes kept, `{label: {said, at}}` by her key, at `path`, as her
/// computer's door keeps a choice (door/choices.rs `Kept`).
fn kept_yes(path: &Path, app: &str) {
    let ada = svrngs_node::actor_of(&SigningKey::from_bytes(&[1; 32]));
    let at = chrono::Local::now().timestamp();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let kept = json!({ ada: { app: { "said": "yes", "at": at } } });
    std::fs::write(path, kept.to_string()).unwrap();
}

in_chromium!(
    /// **A Stop on what apps use is read again in place** (page-acts (d)).
    /// Ada said yes to her page using her shelf, which asks; her home
    /// page's "What acts as you" says her page uses the shelf, with Stop.
    /// She presses it, and the section, read again, says her page was
    /// stopped using the shelf, with no reload. Watched failing on v1
    /// (`UI` flipped), whose Stop reloads the page (home.js:221).
    a_stop_on_what_apps_use_is_read_again_in_place
);
async fn a_stop_on_what_apps_use_is_read_again_in_place(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let shelf = svrngs_node::lend::LendName::try_from("shelf".to_string()).unwrap();
    kept_yes(&svrngs_node::lend::choices_path(dir.path(), &shelf), &label("page"));
    let (ada, home) = ada(dir.path(), &relay).await;
    let stop = "#acts p:has-text('uses') button:text-is('Stop')";
    let args = json!({ "home": home, "shown": "#acts summary", "press": ["#acts summary", stop], "said": "was stopped using", "at": "#acts", "within": 30_000 });
    let seen = drive(engine, "v2/members.mjs", args, None).await;
    none_wrong(
        &[
            (seen["went"] != true, "after the Stop, the section says it was stopped"),
            (seen["reloads"] != 0, "with no reload"),
        ],
        &seen,
    );
    drop(ada);
}

in_chromium!(
    /// **A yes in another tab is shown on `visible`** (page-acts (d)).
    /// Ada's home page is open, her page said yes to nowhere; she says yes
    /// to it elsewhere (its yes kept, as /to keeps it), and her page,
    /// hidden and shown again, says in "What acts as you" that her page
    /// opens her friends' k as her, with no reload. Watched failing on v1
    /// (`UI` flipped), which reads /reaching once a load (acts.mjs:60
    /// reloads it).
    a_yes_in_another_tab_is_shown_on_visible
);
async fn a_yes_in_another_tab_is_shown_on_visible(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = ada(dir.path(), &relay).await;
    let hold = dir.path().join("hold");
    let args = json!({ "home": home, "shown": "#acts summary", "press": ["#acts summary"], "hold": hold, "said": "opens your friends’ k as you. Since", "at": "#acts", "within": 30_000 });
    let driven = tokio::spawn(drive(engine, "v2/members.mjs", args, None));
    let shown = hold.with_extension("shown");
    let told = tokio::time::timeout(Duration::from_secs(90), async {
        while !shown.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    assert!(told.await.is_ok(), "ada's page shows what acts as her within 90 s");
    kept_yes(&dir.path().join("reaching.json"), &label("page"));
    std::fs::write(&hold, "").unwrap();
    let seen = driven.await.unwrap();
    none_wrong(
        &[
            (seen["went"] != true, "shown again, the yes is said"),
            (seen["reloads"] != 0, "with no reload"),
        ],
        &seen,
    );
    drop(ada);
}
