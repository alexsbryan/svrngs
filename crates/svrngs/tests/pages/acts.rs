//! page-acts's tests (main.rs): what v2 does of "What acts as
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
    let bo = svrngs_core::tests_support::Fixture {
        act: svrngs_node::RailAct::Admit {
            person: svrngs_node::Person::parse("bo").unwrap(),
            key: svrngs_node::actor_of(&SigningKey::from_bytes(&[7; 32])),
            invite: None,
        },
    };
    node.rail().commit(&seed.id(), &bo).unwrap().unwrap();
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

in_chromium!(
    /// **Two refusals on /ask are said once** (page-acts (d)). Ada's home
    /// page asks whether her page may use her shelf; her session lapses
    /// (its key dropped), and she presses Yes, then, once that is
    /// refused, Yes again: the section says the refusal once, the second
    /// in place of the first. Watched failing on v1 (`UI` flipped), whose
    /// `ask` appends each refusal (home.js@19b63b2b:343).
    two_refusals_on_ask_are_said_once
);
async fn two_refusals_on_ask_are_said_once(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home, _) = ada(dir.path(), &relay).await;
    let ask = format!(
        "{home}ask?lend={}&app={}&back=/",
        label("shelf"),
        label("page")
    );
    let yes = "#home section button:text-is('Yes')";
    let press = |n: u8| json!({ "at": yes, "traced": ["ASK: unsaid", n] });
    let args = json!({ "home": ask, "shown": "#home section h2", "forget": true, "press": [press(1), press(2)], "at": "#home section", "within": 15_000 });
    let seen = drive(engine, "pages/acts.mjs", args, None).await;
    let texts: Vec<&str> = (seen["texts"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .collect();
    let first = texts
        .first()
        .and_then(|t| t.split("Not now").nth(1))
        .unwrap_or_default();
    none_wrong(
        &[
            (seen["went"] != json!([true, true]), "each Yes is refused"),
            (
                first.trim().is_empty(),
                "the refusal is said after the buttons",
            ),
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
    /// (`UI` flipped), whose Stop reloads the page (home.js@19b63b2b:221).
    a_stop_on_what_apps_use_is_read_again_in_place
);
async fn a_stop_on_what_apps_use_is_read_again_in_place(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let shelf = svrngs_node::lend::LendName::try_from("shelf".to_string()).unwrap();
    kept_yes(
        &svrngs_node::lend::choices_path(dir.path(), &shelf),
        &label("page"),
    );
    let (ada, home, _) = ada(dir.path(), &relay).await;
    let stop = "#acts p:has-text('uses') button:text-is('Stop')";
    let args = json!({ "home": home, "shown": "#acts summary", "press": ["#acts summary", stop], "said": "was stopped using", "at": "#acts", "within": 30_000 });
    let seen = drive(engine, "pages/members.mjs", args, None).await;
    none_wrong(
        &[
            (
                seen["went"] != true,
                "after the Stop, the section says it was stopped",
            ),
            (seen["reloads"] != 0, "with no reload"),
        ],
        &seen,
    );
    drop(ada);
}

in_chromium!(
    /// **A refused Stop on what apps use is said beside it**. Ada said yes
    /// to her page using her shelf, which asks; her computer can read
    /// that yes but keep nothing new beside it (its choices' directory
    /// made read-only), so her Stop is refused, "this could not be kept on
    /// the computer that lends it" (door/choices.rs `keep`). The line
    /// with that Stop says so after it. Watched failing at 4b7160c0,
    /// whose refusal carried no lend and no app (ui/machine.js `run`
    /// makes a failed effect's message `{type, n, why}`), so ACTS kept it
    /// under "undefined undefined", which `Uses` never draws.
    a_refused_stop_on_what_apps_use_is_said_beside_it
);
async fn a_refused_stop_on_what_apps_use_is_said_beside_it(engine: Engine) {
    use std::os::unix::fs::PermissionsExt;
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let shelf = svrngs_node::lend::LendName::try_from("shelf".to_string()).unwrap();
    let choices = svrngs_node::lend::choices_path(dir.path(), &shelf);
    kept_yes(&choices, &label("page"));
    let (ada, home, _) = ada(dir.path(), &relay).await;
    let held = choices.parent().unwrap();
    std::fs::set_permissions(held, std::fs::Permissions::from_mode(0o555)).unwrap();
    let line = "#acts p:has-text('uses')";
    let stop =
        json!({ "at": format!("{line} button:text-is('Stop')"), "traced": ["ACTS: unstopped", 1] });
    let args = json!({ "home": home, "shown": "#acts summary", "press": [{ "at": "#acts summary" }, stop], "at": line, "within": 30_000 });
    let seen = drive(engine, "pages/acts.mjs", args, None).await;
    // Writable again, so the directory is removed with the rest.
    std::fs::set_permissions(held, std::fs::Permissions::from_mode(0o755)).unwrap();
    let said = seen["texts"][1].as_str().unwrap_or_default();
    none_wrong(
        &[
            (seen["went"][1] != true, "the Stop is refused"),
            (
                !said.contains("Stop")
                    || !said.contains("this could not be kept on the computer that lends it"),
                "the refusal is said on the line of its Stop",
            ),
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
    /// opens nothing yet (a yes the file kept names no lend), with no reload. Watched failing on v1
    /// (`UI` flipped), which reads /reaching once a load (acts.mjs:60
    /// reloads it).
    a_yes_in_another_tab_is_shown_on_visible
);
async fn a_yes_in_another_tab_is_shown_on_visible(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home, _) = ada(dir.path(), &relay).await;
    let hold = dir.path().join("hold");
    let args = json!({ "home": home, "shown": "#acts summary", "press": ["#acts summary"], "hold": hold, "said": "opens nothing of your friends’ as you", "at": "#acts", "within": 30_000 });
    let driven = tokio::spawn(drive(engine, "pages/members.mjs", args, None));
    let shown = hold.with_extension("shown");
    let told = tokio::time::timeout(Duration::from_secs(90), async {
        while !shown.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    assert!(
        told.await.is_ok(),
        "ada's page shows what acts as her within 90 s"
    );
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

in_webkit!(
    /// **A refused yes keeps the ask shown, with why** (page-acts (d)).
    /// Bo's phone asks, at `#ask=`, whether ada's page may use her shelf;
    /// once it asks, ada stops lending the shelf, and bo presses Yes: her
    /// computer refuses it, and the page says so in its heading with the
    /// ask still shown, so bo can say it again. Watched failing on v1
    /// (`UI` flipped), whose form is hidden on Yes and left so (app.js@19b63b2b:316).
    a_refused_yes_keeps_the_ask_shown
);
async fn a_refused_yes_keeps_the_ask_shown(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, _, id) = ada(dir.path(), &relay).await;
    let hold = dir.path().join("hold");
    let mut args = bo_s_phone(&relay, &id);
    let ask = format!("ask={}&app={}&at=/", label("shelf"), label("page"));
    args.as_object_mut().unwrap().extend(
        json!({ "ask": ask, "hold": hold, "within": 60_000 })
            .as_object()
            .unwrap()
            .clone(),
    );
    let driven = tokio::spawn(drive(engine, "pages/acts.mjs", args, None));
    let shown = hold.with_extension("shown");
    let told = tokio::time::timeout(Duration::from_secs(120), async {
        while !shown.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    assert!(told.await.is_ok(), "bo's page asks within 120 s");
    std::fs::remove_file(svrngs_node::lend::lends_dir(dir.path()).join("shelf.json")).unwrap();
    std::fs::write(&hold, "").unwrap();
    let seen = driven.await.unwrap();
    none_wrong(
        &[
            (
                !seen["asked"]
                    .as_str()
                    .is_some_and(|t| t.starts_with("Let ")),
                "bo is asked",
            ),
            (seen["went"] != true, "the refusal is said in the heading"),
            (seen["hidden"] != false, "the ask stays shown"),
        ],
        &seen,
    );
    drop(ada);
}

in_webkit!(
    /// **A line left unsent is carried on the next open** (page-acts (d)).
    /// Bo's phone keeps house with a line of his own signed and left
    /// unsent, as a carry that failed leaves it (app.js@19b63b2b:506); the page
    /// opened again reads house, carries the line to ada's computer, and
    /// keeps none unsent. Watched failing on v1 (`UI` flipped), which
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
