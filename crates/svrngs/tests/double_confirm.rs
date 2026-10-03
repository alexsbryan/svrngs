//! double-confirm (DESIGN D9 E0; `scripts/e0_demo.py` TEST_BARS): a Confirm
//! retried across a host restart produces one act. Driven through the bridge
//! append on loopback, as the chat adapter calls it.

use std::net::SocketAddr;

use serde_json::{Value, json};
use svrngs::{Home, Host, store};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const INTERACTION: &str = "1290000000000000001";

async fn post(addr: SocketAddr, token: Option<&str>, body: &str) -> (u16, Value) {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let auth = token
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    let req = format!(
        "POST /append HTTP/1.1\r\nHost: localhost\r\n{auth}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
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

fn confirm(interaction: Option<&str>) -> String {
    let mut on_behalf_of = json!({"speaker": "discord:80351110224678912"});
    if let Some(i) = interaction {
        on_behalf_of["interaction"] = json!(i);
    }
    json!({"app": "expenses", "op": {"kind": "confirm", "of": "the pizza"}, "on_behalf_of": on_behalf_of}).to_string()
}

/// The journal's lines, read as a person would read the file.
fn lines(home: &Home) -> Vec<Value> {
    let group = home.the_group().unwrap().unwrap();
    let text = std::fs::read_to_string(home.group_dir(&group).join(store::JOURNAL)).unwrap();
    text.lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}

fn answering(home: &Home, interaction: &str) -> usize {
    lines(home)
        .iter()
        .filter(|l| l["body"]["on_behalf_of"]["interaction"] == interaction)
        .count()
}

fn new_home() -> (tempfile::TempDir, Home) {
    let dir = tempfile::tempdir().unwrap();
    let home = Home::new(dir.path());
    home.create_group("the house", svrngs::now()).unwrap();
    (dir, home)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_confirm_retried_across_a_host_restart_makes_one_act() {
    let (_dir, home) = new_home();
    let token = home.token().unwrap();

    let host = Host::start(&home, 0).await.unwrap();
    let (status, first) = post(host.addr(), Some(token.as_str()), &confirm(Some(INTERACTION))).await;
    assert_eq!((status, &first["new"]), (200, &json!(true)), "{first}");
    host.stop().await.unwrap();

    let host = Host::start(&home, 0).await.unwrap();
    let (status, retry) = post(host.addr(), Some(token.as_str()), &confirm(Some(INTERACTION))).await;
    host.stop().await.unwrap();

    assert_eq!(status, 200, "{retry}");
    assert_eq!(
        answering(&home, INTERACTION),
        1,
        "the retry made a second act: {:#?}",
        lines(&home)
    );
    assert_eq!(retry, json!({"id": first["id"], "new": false}));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_confirm_with_no_interaction_id_is_refused() {
    let (_dir, home) = new_home();
    let token = home.token().unwrap();
    let host = Host::start(&home, 0).await.unwrap();
    let (status, body) = post(host.addr(), Some(token.as_str()), &confirm(None)).await;
    host.stop().await.unwrap();
    assert_eq!(status, 422, "{body}");
    assert!(body["error"].as_str().unwrap().contains("interaction"), "{body}");
    assert_eq!(lines(&home).len(), 1, "only the group's first act: nothing was written");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_append_without_the_capability_token_is_refused() {
    let (_dir, home) = new_home();
    let host = Host::start(&home, 0).await.unwrap();
    let (none, _) = post(host.addr(), None, &confirm(Some(INTERACTION))).await;
    let wrong = "0".repeat(64);
    let (bad, _) = post(host.addr(), Some(&wrong), &confirm(Some(INTERACTION))).await;
    host.stop().await.unwrap();
    assert_eq!((none, bad), (401, 401));
    assert_eq!(answering(&home, INTERACTION), 0);
}
