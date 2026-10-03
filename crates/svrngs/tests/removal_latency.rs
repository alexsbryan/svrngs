//! removal-latency (DESIGN D9 E0; `scripts/e0_demo.py` TEST_BARS): someone
//! banned in the chat cannot have an act admitted sixty seconds later.
//!
//! The ban reaches the host the way the chat delivers it: through the bridge
//! (`bridge/`, driven by its fake adapter), whose leave handler writes it into
//! the guest book. The act is then tried twice: as a stored interaction
//! replayed straight at the loopback append, past the bot, and through the bot
//! as a late tap.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use svrngs::{Home, Host, store};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const AMA: &str = "discord:80351110224678912";
const DEADLINE: Duration = Duration::from_secs(60);

async fn post(addr: SocketAddr, token: &str, body: &str) -> (u16, Value) {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let req = format!(
        "POST /append HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).await.unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).await.unwrap();
    let status = out.split(' ').nth(1).unwrap().parse().unwrap();
    let json = out
        .split_once("\r\n\r\n")
        .map_or(Value::Null, |(_, b)| serde_json::from_str(b).unwrap_or(Value::Null));
    (status, json)
}

/// The append the bridge sends for Ama's Confirm, kept to replay later.
fn confirm(interaction: &str) -> String {
    json!({
        "app": "expenses",
        "op": {"kind": "confirm", "of": "the pizza"},
        "on_behalf_of": {"speaker": AMA, "interaction": interaction},
    })
    .to_string()
}

/// Run the bridge over `steps` with the fake adapter; what it posted and replied.
fn bridge(addr: SocketAddr, home: &Home, steps: Value) -> Value {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bridge");
    let mut child = Command::new("node")
        .arg(dir.join("tests/harness.mjs"))
        .arg(steps.to_string())
        .env("SVRNGS_HOST", format!("http://{addr}"))
        .env("SVRNGS_TOKEN_FILE", home.token_file())
        .env("SVRNGS_OWNER", "Sam")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("node runs the bridge: is Node.js 20 or later on PATH?");
    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(started.elapsed() < DEADLINE, "the bridge took over sixty seconds");
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "the bridge failed: {stderr}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(stdout.lines().last().unwrap_or_default())
        .unwrap_or_else(|e| panic!("{e}: {stdout}\n{stderr}"))
}

fn lines(home: &Home) -> Vec<Value> {
    let group = home.the_group().unwrap().unwrap();
    let text = std::fs::read_to_string(home.group_dir(&group).join(store::JOURNAL)).unwrap();
    text.lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}

fn texts(v: &Value, key: &str) -> Vec<String> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["text"].as_str().unwrap().to_string())
        .collect()
}

/// One chat event, `step` merged over Ama, through the bridge.
async fn as_ama(dir: PathBuf, addr: SocketAddr, step: Value) -> Value {
    tokio::task::spawn_blocking(move || {
        let mut s = json!({"speaker": AMA, "name": "Ama"});
        s.as_object_mut().unwrap().extend(step.as_object().unwrap().clone());
        bridge(addr, &Home::new(dir), json!([s]))
    })
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_banned_speaker_has_no_act_admitted_sixty_seconds_later() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    let home = Home::new(&dir);
    home.create_group("the house", svrngs::now()).unwrap();
    let token = home.token().unwrap();
    let host = Host::start(&home, 0).await.unwrap();
    let addr = host.addr();

    // Ama says something in the channel: the bridge enters her in the guest book.
    let first = as_ama(
        dir.clone(),
        addr,
        json!({"do": "input", "interaction": "m1", "text": "hi"}),
    )
    .await;
    let posts = texts(&first, "posts");
    assert!(posts.iter().any(|p| p.starts_with("Welcome, Ama.")), "{posts:?}");

    // Before the ban her Confirm is admitted: the positive control.
    let (status, before) = post(addr, token.as_str(), &confirm("i-before")).await;
    assert_eq!((status, &before["new"]), (200, &json!(true)), "{before}");

    let banned_at = Instant::now();
    as_ama(dir.clone(), addr, json!({"do": "ban"})).await;

    // A stored interaction replayed at the host, past the bot.
    let (status, replayed) = post(addr, token.as_str(), &confirm("i-after")).await;
    assert!(banned_at.elapsed() <= DEADLINE);
    assert_eq!(
        status,
        403,
        "an act for a speaker banned {:?} ago was admitted: {replayed}",
        banned_at.elapsed()
    );
    assert_eq!(replayed["removed"], json!(true), "{replayed}");

    // A late tap through the bot: refused, told privately, and not welcomed back.
    let late = as_ama(
        dir.clone(),
        addr,
        json!({"do": "input", "kind": "button", "interaction": "i-late"}),
    )
    .await;
    assert!(texts(&late, "posts").is_empty(), "{late}");
    assert!(
        texts(&late, "replies")[0].starts_with("You're no longer part of"),
        "{late}"
    );

    // The tap taken before the ban still answers with its own act.
    let (status, retry) = post(addr, token.as_str(), &confirm("i-before")).await;
    assert_eq!(
        (status, &retry["id"], &retry["new"]),
        (200, &before["id"], &json!(false))
    );
    host.stop().await.unwrap();

    let journal = lines(&home);
    let kinds: Vec<String> = journal
        .iter()
        .filter(|l| l["body"]["on_behalf_of"]["speaker"] == AMA || l["body"]["speaker"] == AMA)
        .map(|l| format!("{}:{}", l["body"]["kind"], l["body"]["event"]).replace('"', ""))
        .collect();
    assert_eq!(
        kinds,
        ["guest:seen", "record:null", "guest:banned"],
        "Ama's acts, in the journal's order: {journal:#?}"
    );
}
