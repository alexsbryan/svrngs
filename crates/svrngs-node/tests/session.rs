//! Browser authority is port-scoped and proved per request. These are integration
//! specifications; the security pass runs only the pure session/core tests.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[allow(dead_code)]
#[path = "door/origins.rs"]
mod origins;
#[path = "common/pair.rs"]
mod pair;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use pair::{alex_lets_sam_in, signing};
use std::{net::SocketAddr, time::Duration};
use svrngs_core::pass::Label;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct Got {
    status: u16,
    head: String,
    body: String,
}
impl Got {
    fn header(&self, name: &str) -> Option<&str> {
        self.head
            .lines()
            .filter_map(|l| l.split_once(':'))
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.trim())
    }
}
async fn ask(
    api: SocketAddr,
    method: &str,
    host: &str,
    path: &str,
    headers: &str,
    body: &str,
) -> Got {
    let mut sock = tokio::net::TcpStream::connect(api).await.unwrap();
    sock.write_all(format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\n{headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
    let mut bytes = Vec::new();
    tokio::time::timeout(Duration::from_secs(30), sock.read_to_end(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    let reply = String::from_utf8(bytes).unwrap();
    let (head, body) = reply.split_once("\r\n\r\n").unwrap();
    Got {
        status: head[9..12].parse().unwrap(),
        head: head.into(),
        body: body.into(),
    }
}
fn path_of(url: &str) -> String {
    format!(
        "/{}",
        url.strip_prefix("http://")
            .unwrap()
            .split_once('/')
            .unwrap()
            .1
    )
}
const TYPED: &str =
    "Sec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-Dest: document\r\n";

async fn enroll(api: SocketAddr, token: &str, name: &str) -> Vec<u8> {
    let body = if name == "svrngs" {
        "{}".into()
    } else {
        serde_json::json!({ "lend": name }).to_string()
    };
    let opened = ask(
        api,
        "POST",
        &api.to_string(),
        "/v1/open",
        &format!("Authorization: Bearer {token}\r\n"),
        &body,
    )
    .await;
    assert_eq!(opened.status, 200);
    let json: serde_json::Value = serde_json::from_str(&opened.body).unwrap();
    let path = path_of(json["open"].as_str().unwrap());
    let host = format!("{name}.localhost:{}", api.port());
    let shell = ask(api, "GET", &host, &path, TYPED, "").await;
    assert!(shell.header("set-cookie").is_none());
    assert!(shell.body.contains("svrngs-session.js"));
    let headers = format!("Origin: http://{host}\r\nSec-Fetch-Site: same-origin\r\n");
    let reply = ask(api, "POST", &host, &path, &headers, "").await;
    assert_eq!(reply.status, 200);
    assert!(reply.header("set-cookie").is_none());
    let replay = ask(api, "POST", &host, &path, &headers, "").await;
    assert_eq!(replay.status, 401);
    let json: serde_json::Value = serde_json::from_str(&reply.body).unwrap();
    URL_SAFE_NO_PAD
        .decode(json["key"].as_str().unwrap())
        .unwrap()
}

async fn proved(
    api: SocketAddr,
    key: &[u8],
    host: &str,
    method: &str,
    path: &str,
    body: &str,
) -> String {
    let epoch = ask(api, "GET", host, "/svrngs-session-epoch", TYPED, "").await;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut nonce = [0; 16];
    getrandom::fill(&mut nonce).unwrap();
    let stamp = format!("{now}.{}", hex::encode(nonce));
    let digest = hex::encode(ring::digest::digest(&ring::digest::SHA256, body.as_bytes()).as_ref());
    let message = format!(
        "{host}\n{method}\n{path}\n{stamp}\n{digest}\nyes\n{}",
        epoch.body
    );
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    let tag = URL_SAFE_NO_PAD.encode(ring::hmac::sign(&key, message.as_bytes()));
    format!("Origin: http://{host}\r\nSec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nX-Svrngs-Browser: {stamp}.{tag}\r\nX-Svrngs-Body: {digest}\r\nX-Svrngs-Intent: yes\r\n")
}

#[tokio::test]
async fn enrollment_is_one_use_and_proofs_are_bound_to_requests_not_cookies() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, _sam) = alex_lets_sam_in(a.path(), b.path()).await;
    let api = alex.api_addr();
    let host = format!("svrngs.localhost:{}", api.port());
    let cookie = ask(
        api,
        "GET",
        &host,
        "/v1/groups",
        &format!("{TYPED}Cookie: svrngs=stolen\r\n"),
        "",
    )
    .await;
    assert_eq!(cookie.status, 401);
    let key = enroll(api, &alex.token(), "svrngs").await;
    let headers = proved(api, &key, &host, "GET", "/v1/groups", "").await;
    assert_eq!(
        ask(api, "GET", &host, "/lent", &headers, "").await.status,
        401
    );
    assert_eq!(
        ask(api, "GET", &host, "/v1/groups", &headers, "")
            .await
            .status,
        200
    );
    assert_eq!(
        ask(api, "GET", &host, "/v1/groups", &headers, "")
            .await
            .status,
        401
    );
    let headers = proved(api, &key, &host, "POST", "/lent", "{}").await;
    let altered = ask(
        api,
        "POST",
        &host,
        "/lent",
        &headers,
        "{\"lend\":\"changed\"}",
    )
    .await;
    assert_eq!(
        altered.status, 403,
        "an authenticated header cannot authorize a different body"
    );
}

#[tokio::test]
async fn a_lend_has_its_own_key_and_no_proof_reaches_the_service() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, _sam) = alex_lets_sam_in(a.path(), b.path()).await;
    let (service, heads) = origins::head_origin().await;
    let dir = svrngs_node::lend::lends_dir(a.path());
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("night.json"),
        serde_json::json!({"port":service.port(),"kind":"night","limit":3,"groups":["house"]}).to_string(),
    )
    .unwrap();
    let name = Label::of(&signing(1).verifying_key().to_bytes(), "night").to_string();
    let api = alex.api_addr();
    let host = format!("{name}.localhost:{}", api.port());
    let key = enroll(api, &alex.token(), &name).await;
    let headers = proved(api, &key, &host, "GET", "/films?x=1", "").await;
    let got = ask(api, "GET", &host, "/films?x=1", &headers, "").await;
    assert_eq!(got.status, 200);
    let sent = heads.lock().unwrap();
    assert!(sent.iter().any(|h| h.starts_with("GET /films?x=1")));
    assert!(!sent
        .iter()
        .any(|h| h.to_ascii_lowercase().contains("x-svrngs-browser")));
}

#[tokio::test]
async fn a_forged_read_capability_is_not_a_browser_session() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, _sam) = alex_lets_sam_in(a.path(), b.path()).await;
    let name = Label::of(&signing(1).verifying_key().to_bytes(), "night");
    let page = Label::of(&signing(1).verifying_key().to_bytes(), "page");
    let host = format!("{name}.localhost:{}", alex.api_addr().port());
    let extra = format!("Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{page}.localhost:{}\r\n", alex.api_addr().port());
    assert_eq!(
        ask(
            alex.api_addr(),
            "GET",
            &host,
            "/svrngs-as/forged/films",
            &extra,
            ""
        )
        .await
        .status,
        401
    );
}

/// **A same-origin redirect's onward request is proved by the proof that asked
/// it, once, and only at the address the answer named.** A browser's fetch
/// follows a redirect itself, sending the same headers on (the session
/// worker cannot read an opaque redirect's `Location`), so the proof bound
/// to `/redirect` arrives again at `/landed?x=1`. The node, which saw its
/// own answer name that address, takes it there once (`session.rs`
/// `Onwards`), through as many redirects as a browser follows; the same proof
/// at another path, or past those, is refused. The service is the conformance canary's Host check, which
/// redirects every path it is asked to `/landed?x=1`. Red before `Onwards`:
/// the onward answered 401, as the canary's `fetch('redirect')` did in both
/// engines.
#[tokio::test]
async fn a_same_origin_redirect_is_followed_once_by_the_proof_that_asked_it() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, _sam) = alex_lets_sam_in(a.path(), b.path()).await;
    let service = origins::host_origin().await;
    let dir = svrngs_node::lend::lends_dir(a.path());
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("canary.json"),
        serde_json::json!({"port":service.port(),"kind":"canary","limit":3,"groups":["house"]}).to_string(),
    )
    .unwrap();
    let name = Label::of(&signing(1).verifying_key().to_bytes(), "canary").to_string();
    let api = alex.api_addr();
    let host = format!("{name}.localhost:{}", api.port());
    let key = enroll(api, &alex.token(), &name).await;
    let (said, _guards) = common::heard("session=debug,origin=debug");

    let headers = proved(api, &key, &host, "GET", "/redirect", "").await;
    let first = ask(api, "GET", &host, "/redirect", &headers, "").await;
    assert_eq!(first.status, 302, "{}", first.head);
    assert_eq!(
        first.header("location"),
        Some("/landed?x=1"),
        "made a path by the door"
    );
    let onward = ask(api, "GET", &host, "/landed?x=1", &headers, "").await;
    let heard = || String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    assert_eq!(
        onward.status,
        302,
        "the onward reaches the service:\n{}\n{}",
        onward.head,
        heard()
    );
    // The service redirects `/landed?x=1` to itself, so each onward names the
    // next: followed through 20 redirects in all, the fetch standard's
    // limit, and the 21st refused.
    let mut taken = 1;
    while ask(api, "GET", &host, "/landed?x=1", &headers, "")
        .await
        .status
        == 302
    {
        taken += 1;
        assert!(taken <= 20, "a proof followed past 20 redirects");
    }
    assert_eq!(
        taken, 20,
        "a proof follows as many redirects as a browser does"
    );

    let headers = proved(api, &key, &host, "GET", "/redirect", "").await;
    assert_eq!(
        ask(api, "GET", &host, "/redirect", &headers, "")
            .await
            .status,
        302
    );
    let elsewhere = ask(api, "GET", &host, "/elsewhere", &headers, "").await;
    assert_eq!(
        elsewhere.status, 401,
        "an onward request only where the answer pointed"
    );
}
