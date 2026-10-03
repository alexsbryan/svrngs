//! Demo D1's scene, end to end (DESIGN D7 E0.3; demo/d1-split-the-pizza.md):
//! one person types the pizza, the others tap Confirm, and the bot says who
//! owes whom — through the bridge (its fake adapter), the host's handler
//! runner and Expenses folded by the engine. What the bot posts is what the
//! journal folds to.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use svrngs::runner::Runner;
use svrngs::{Home, Host, journal, store};

const SAM: &str = "discord:101";
const AMA: &str = "discord:102";
const BO: &str = "discord:103";
const CY: &str = "discord:104";

/// Run the bridge over `steps` with the fake adapter; what it posted and replied.
fn bridge(addr: std::net::SocketAddr, home: &Home, steps: Value) -> Value {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bridge");
    let out = Command::new("node")
        .arg(dir.join("tests/harness.mjs"))
        .arg(steps.to_string())
        .env("SVRNGS_HOST", format!("http://{addr}"))
        .env("SVRNGS_TOKEN_FILE", home.token_file())
        .env("SVRNGS_OWNER", "Sam")
        .output()
        .expect("node runs the bridge: is Node.js 20 or later on PATH?");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "the bridge failed: {stderr}");
    serde_json::from_str(stdout.lines().last().unwrap_or_default())
        .unwrap_or_else(|e| panic!("{e}: {stdout}\n{stderr}"))
}

fn tap(speaker: &str, name: &str, interaction: &str, button: &str) -> Value {
    json!({"do": "input", "speaker": speaker, "name": name, "kind": "button", "interaction": interaction, "button": button})
}

fn lines(home: &Home) -> Vec<Value> {
    let group = home.the_group().unwrap().unwrap();
    let text = std::fs::read_to_string(home.group_dir(&group).join(store::JOURNAL)).unwrap();
    text.lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn five_people_split_the_pizza_in_the_chat() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    let home = Home::new(&dir);
    home.create_group("the house", svrngs::now()).unwrap();
    let host = Host::start(&home, 0).await.unwrap();
    let addr = host.addr();
    let run = |steps: Value| {
        let dir = dir.clone();
        tokio::task::spawn_blocking(move || bridge(addr, &Home::new(dir), steps))
    };

    let typed = run(json!([{
        "do": "input", "speaker": SAM, "name": "Sam", "kind": "command", "interaction": "c1",
        "command": "paid", "options": {"amount": "60", "what": "pizza"},
    }]))
    .await
    .unwrap();
    let posted = &typed["replies"][0];
    assert_eq!(
        posted["text"], "Sam paid $60.00 for pizza. If you had some, tap Confirm and I'll split it.",
        "{typed}"
    );
    assert_eq!(posted["private"], false);
    let button = posted["buttons"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(posted["buttons"][0]["label"], "Confirm");

    let tapped = run(json!([
        tap(AMA, "Ama", "b1", &button),
        tap(BO, "Bo", "b2", &button),
        tap(CY, "Cy", "b3", &button),
        tap(CY, "Cy", "b3", &button),
        tap(BO, "Bo", "b4", &button),
        {"do": "input", "speaker": AMA, "name": "Ama", "kind": "command", "interaction": "c2", "command": "owe"},
    ]))
    .await
    .unwrap();
    let replies: Vec<(&str, &str, bool)> = tapped["replies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["to"].as_str().unwrap(),
                r["text"].as_str().unwrap(),
                r["private"].as_bool().unwrap(),
            )
        })
        .collect();
    let owed = "Who owes whom:\nAma owes Sam $15.00\nBo owes Sam $15.00\nCy owes Sam $15.00";
    assert_eq!(replies[2], ("b3", owed, false), "{tapped}");
    assert_eq!(
        replies[3],
        ("b3", owed, false),
        "a retried tap is answered as the first was"
    );
    assert_eq!(replies[4], ("b4", "You're already in on pizza.", true));
    assert_eq!(replies[5], ("c2", owed, false));
    host.stop().await.unwrap();

    // The journal holds one act per tap, the retry made none, and the
    // fold of the journal on disk is what the bot said.
    let acts = lines(&home);
    let by = |i: &str| {
        acts.iter()
            .filter(|l| l["body"]["on_behalf_of"]["interaction"] == i)
            .count()
    };
    assert_eq!(
        (by("c1"), by("b1"), by("b2"), by("b3"), by("b4"), by("c2")),
        (1, 1, 1, 1, 0, 0)
    );
    let group = home.the_group().unwrap().unwrap();
    let (_, journal) = store::GroupDir::open(&home.group_dir(&group), svrngs::now()).unwrap();
    let runner = Runner::built_in().unwrap();
    let app = "expenses".try_into().unwrap();
    assert_eq!(runner.say(&journal, &app, None).unwrap().text, owed);
}

static NOW: AtomicU64 = AtomicU64::new(0);
fn clock() -> u64 {
    NOW.load(Ordering::SeqCst)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_debt_left_alone_is_reminded_once() {
    let tmp = tempfile::tempdir().unwrap();
    let home = Home::new(tmp.path());
    let start = svrngs::now();
    NOW.store(start, Ordering::SeqCst);
    let id = home.create_group("the house", start).unwrap();
    let (dir, journal) = store::GroupDir::open(&home.group_dir(&id), start).unwrap();
    let keys = home.keys(&id).unwrap();
    let (handle, actor) = journal::spawn(dir, journal, keys.bridge, clock, Runner::built_in().unwrap());

    let invoke = |speaker: &str, interaction: &str, input: Value| -> journal::Invocation {
        serde_json::from_value(
            json!({"input": input, "on_behalf_of": {"speaker": speaker, "interaction": interaction}}),
        )
        .unwrap()
    };
    let paid = json!({"kind": "command", "command": "paid", "options": {"amount": "30", "what": "taxi"}});
    let posted = handle.run(invoke(SAM, "c1", paid)).await.unwrap();
    let button = posted.buttons[0].id.clone();
    handle
        .run(invoke(AMA, "b1", json!({"kind": "button", "button": button})))
        .await
        .unwrap();
    assert_eq!(
        handle.due().await.unwrap(),
        vec![],
        "nothing is due the day it happened"
    );

    NOW.store(start + 4 * 24 * 60 * 60, Ordering::SeqCst);
    let due = handle.due().await.unwrap();
    assert_eq!(due.len(), 1, "{due:?}");
    assert_eq!(
        (due[0].person.as_str(), due[0].text.as_str()),
        (AMA, "a reminder: you owe $15.00. Type /owe to see whom to pay.")
    );
    assert_eq!(handle.due().await.unwrap(), vec![], "a reminder is posted once");
    drop(handle);
    tokio::task::spawn_blocking(move || actor.join())
        .await
        .unwrap()
        .unwrap();
}
