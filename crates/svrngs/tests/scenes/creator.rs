//! Creator -> friend through the existing carried app, phone signer and version
//! gates. Contract and evidence limits: demo/creator-to-friend.md, C1-C5.
use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};

use crate::harness::{relay_url, Engine};
use crate::people::{late, sees, Browsers, Person, Scene};

const APP: &str = "Garden-rota";
const HTML: &str = include_str!("creator/index.html");
const SCRIPT: &str = include_str!("creator/index.js");

fn publish(to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    std::fs::write(to.join("index.html"), HTML).unwrap();
    std::fs::write(to.join("index.js"), SCRIPT).unwrap();
}

async fn steps(browsers: &mut Browsers, who: &str, steps: Value) -> Value {
    let got = browsers
        .tell(json!({ "do": "steps", "who": who, "steps": steps }))
        .await;
    assert_ne!(got["failed"], true, "{who}'s browser: {got:#}");
    assert_eq!(got["errors"], json!([]), "{who}'s page errors: {got:#}");
    got
}

fn words(got: &Value) -> &str {
    got["done"].as_array().unwrap().last().unwrap()["text"]
        .as_str()
        .unwrap()
}

fn record(scene: &Scene, report: &mut Value, bar: &str, observations: Value) {
    report["bars"][bar] = json!({ "verdict": "passed", "observations": observations });
    std::fs::write(
        scene.scratch("report.json"),
        serde_json::to_vec_pretty(report).unwrap(),
    )
    .unwrap();
    eprintln!("creator {bar}: passed");
}

async fn opened(who: &Person, group: &str, name: &str) -> Value {
    sees(
        Engine::Chromium,
        "home.mjs",
        json!({
            "ask": format!("{}?group={group}", who.home_page()),
            "heading": false, "b": true, "after": ["body", format!("Open {name}")], "open": name,
        }),
    )
    .await
}

async fn entry(who: &Person, group: &str, when: &str) -> Value {
    for _ in 0..120 {
        let read = who.get(&format!("/v1/rail/log?namespace={group}")).await;
        if let Some(op) = read["ops"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["payload"]["when"] == when)
        {
            return op.clone();
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    late(format!(
        "the actual journal did not hold {when} within 60 s"
    ))
    .await
}

async fn app_request(who: &Person, origin: &str, body: Option<Value>) -> (u16, Value) {
    let url = reqwest::Url::parse(origin).unwrap();
    let client = reqwest::Client::builder()
        .no_proxy()
        .resolve(
            url.host_str().unwrap(),
            ([127, 0, 0, 1], url.port().unwrap()).into(),
        )
        .build()
        .unwrap();
    let token = std::fs::read_to_string(who.home.join(".svrngs/token")).unwrap();
    let url = url.join("svrngs/acts").unwrap();
    let ask = match body {
        Some(body) => client.post(url).json(&body),
        None => client.get(url),
    };
    let reply = ask.bearer_auth(token.trim()).send().await.unwrap();
    let status = reply.status().as_u16();
    (status, reply.json().await.unwrap())
}

scene!(creator);
async fn creator(engine: Engine) {
    let scene = Scene::new("creator", engine).await;
    let mut report = json!({ "scene": "creator-to-friend", "kind": "scripted-browser",
        "human_newcomer": "never-ran", "bars": {} });
    let (page, _served) = scene.page().await;
    let relay = relay_url(&scene.relay);
    let jo = scene
        .start(
            "Jo",
            &[
                "new", "Jo test", "--as", "Jo", "--origin", &page, "--relay", &relay,
            ],
        )
        .await;
    let test = jo.group_named("Jo test").await;
    let private = jo.folder(&test).await.join("apps/Try-rota");
    publish(&private);
    let checked = scene
        .command(&jo, &["check", private.to_str().unwrap()], "")
        .await;
    let mut browsers = Browsers::driven(
        "demo/runs/canon/2026-10-17-house/people.mjs",
        engine,
        &[("jo", Engine::Chromium), ("kim", Engine::Webkit)],
    )
    .await;
    jo.opens(&test, "Try-rota").await;
    let initial = opened(&jo, &test, "Try rota").await;
    let trial_url = initial["opened"]["url"].as_str().unwrap();
    steps(
        &mut browsers,
        "jo",
        json!([
            { "goto": trial_url }, { "wait": "You are Jo" },
            { "fill": "#when", "value": "TEST ONLY: Tuesday" },
            { "click": "Save my availability", "until": "Jo: TEST ONLY: Tuesday" }
        ]),
    )
    .await;
    entry(&jo, &test, "TEST ONLY: Tuesday").await;

    let jo = scene
        .again(
            jo,
            &[
                "new", "Garden", "--as", "Jo", "--origin", &page, "--relay", &relay,
            ],
        )
        .await;
    let garden = jo.group_named("Garden").await;
    let invite = jo
        .on_home_page("/link", &json!({ "group": garden, "device": false }))
        .await;
    let joined = browsers
        .tell(json!({ "do": "join", "who": "kim", "link": invite["https"], "name": "Kim" }))
        .await;
    assert!(
        joined["status"].as_str().unwrap().contains("in Garden"),
        "{joined:#}"
    );
    let published = jo.folder(&garden).await.join(format!("apps/{APP}"));
    std::fs::create_dir_all(&published).unwrap();
    for file in ["index.html", "index.js"] {
        std::fs::copy(private.join(file), published.join(file)).unwrap();
    }
    jo.opens(&garden, APP).await;
    let log = jo.get(&format!("/v1/rail/log?namespace={garden}")).await;
    assert!(!log.to_string().contains("TEST ONLY: Tuesday"));
    record(
        &scene,
        &mut report,
        "C1",
        json!({ "check": checked, "test_entries_copied": false }),
    );

    let computer = opened(&jo, &garden, "Garden rota").await;
    let origin = computer["opened"]["url"].as_str().unwrap().to_string();
    assert!(
        computer["lines"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line == "HOME: layout b"),
        "the creator scene must use the current grouped layout"
    );
    assert!(
        computer["later"][0]
            .as_str()
            .unwrap()
            .contains("as you in Garden."),
        "{computer:#}"
    );
    let phone = page.to_string();
    let shown = steps(
        &mut browsers,
        "kim",
        json!([
        { "goto": phone }, { "wait": "Garden rota" }
        ]),
    )
    .await;
    assert!(words(&shown).contains("read and write only its own entries"));
    record(
        &scene,
        &mut report,
        "C2",
        json!({ "group": "Garden", "computer_and_phone_named_scope": true }),
    );

    let saved = steps(
        &mut browsers,
        "kim",
        json!([
            { "lent": "Garden rota", "until": "You are Kim" },
            { "fill": "#when", "value": "Saturday morning" },
            { "click": "Save my availability", "until": "Kim: Saturday morning" },
            { "read": true }
        ]),
    )
    .await;
    assert!(words(&saved).contains("Original version"));
    let bookmarked = saved["done"].as_array().unwrap().last().unwrap()["url"]
        .as_str()
        .unwrap()
        .to_string();
    let kim = entry(&jo, &garden, "Saturday morning").await;
    assert_eq!(kim["person"], "Kim");
    let log = jo.get(&format!("/v1/rail/log?namespace={garden}")).await;
    assert_eq!(
        log["ops"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|o| o["payload"]["when"] == "Saturday morning")
            .count(),
        1,
        "one save creates one entry"
    );
    let people = log["people"].as_array().unwrap();
    let kim_key = &people.iter().find(|p| p["person"] == "Kim").unwrap()["person_key"];
    let jo_key = &people.iter().find(|p| p["person"] == "Jo").unwrap()["person_key"];
    assert_eq!(&kim["person_key"], kim_key);
    assert_ne!(&kim["actor"], jo_key, "the host did not sign Kim's entry");
    let view = steps(
        &mut browsers,
        "jo",
        json!([
            { "goto": origin }, { "wait": "Kim: Saturday morning" }
        ]),
    )
    .await;
    assert!(!words(&view).contains("TEST ONLY"));
    let (made, other) = jo.post(&format!("/v1/rail/append?namespace={garden}"),
        &json!({ "op": "record", "payload": { "kind": "Other/note", "words": "Leave this alone" } })).await;
    assert!(
        made < 300,
        "the other app's entry exists before the probe: {other}"
    );
    let (readable, read) = app_request(&jo, &origin, None).await;
    assert_eq!(readable, 200, "the probe can read this app: {read}");
    assert!(read["ops"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["payload"]["when"] == "Saturday morning"));
    let (status, why) = app_request(
        &jo,
        &origin,
        Some(json!({ "corrects": other["id"], "op": null })),
    )
    .await;
    assert_eq!(status, 403, "another app's correction: {why}");
    assert_eq!(why["error"], "an app corrects only its own acts");
    let log = jo.get(&format!("/v1/rail/log?namespace={garden}")).await;
    let other = log["ops"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["payload"]["kind"] == "Other/note")
        .unwrap();
    assert_eq!(other["voided"], false);
    record(
        &scene,
        &mut report,
        "C3",
        json!({ "phone_person": kim["person"],
        "host_did_not_sign": true, "other_app_refusal": status, "entry": kim["id"] }),
    );

    std::fs::write(
        published.join("index.html"),
        HTML.replace("Original version", "Updated version"),
    )
    .unwrap();
    let mut updated = false;
    for _ in 0..120 {
        let shown = sees(engine, "carried.mjs", json!({ "url": origin })).await;
        if shown["said"].as_str().unwrap().contains("Updated version") {
            updated = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    assert!(updated, "Jo's own changes run for Jo");
    let before = steps(
        &mut browsers,
        "kim",
        json!([
            { "goto": bookmarked }, { "wait": "Kim: Saturday morning" }
        ]),
    )
    .await;
    assert!(
        words(&before).contains("Original version"),
        "no premature acceptance: {before:#}"
    );
    let changes = steps(
        &mut browsers,
        "kim",
        json!([
            { "goto": phone }, { "wait": "Garden rota" },
            { "summary": "What acts as you" }, { "summary": "See what’s new" }, { "read": true }
        ]),
    )
    .await;
    assert!(
        words(&changes).contains("Jo changed its files"),
        "{changes:#}"
    );
    let accepted = steps(
        &mut browsers,
        "kim",
        json!([
            { "click": "Open the new version", "until": "new version for you now" },
            { "lent": "Garden rota", "until": "Updated version" },
            { "wait": "Kim: Saturday morning" }
        ]),
    )
    .await;
    assert!(words(&accepted).contains("Updated version"));
    record(
        &scene,
        &mut report,
        "C4",
        json!({ "before": "Original version", "after": "Updated version",
        "changed_by": "Jo", "contribution_retained": true }),
    );

    std::fs::remove_dir_all(&published).unwrap();
    let mut withdrawn = false;
    for _ in 0..120 {
        let listed = jo.get(&format!("/v1/lends?namespace={garden}")).await;
        if !listed["lent"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["kind"] == APP)
        {
            withdrawn = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    assert!(withdrawn, "removal withdraws the carried app");
    let (status, _) = app_request(&jo, &origin, None).await;
    assert!(
        status >= 400,
        "the old app route must no longer offer entries"
    );
    let retained = entry(&jo, &garden, "Saturday morning").await;
    assert_eq!(retained["id"], kim["id"]);
    record(
        &scene,
        &mut report,
        "C5",
        json!({ "withdrawn": true, "old_route_status": status,
        "shared_entry_retained": true, "withdrawal_action": "removed published folder" }),
    );
}
