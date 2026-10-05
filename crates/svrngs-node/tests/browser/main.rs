//! The page in a browser (DESIGN §3, "Joining"; §8, U4): the built page,
//! in Playwright's headless Chromium, dials a node by key as a person who
//! stands in its group. Everything runs on this computer: iroh's relay
//! server on 127.0.0.1, plain HTTP, and a node whose only relay it is;
//! the page is served on 127.0.0.1 and its link carries that relay. The
//! browser half is `web/tests/drive.mjs`, which fails naming the install
//! command when Playwright or its Chromium is absent — it never skips.
//!
//! Each test builds the page first with `scripts/build-web.sh`, once per
//! run, so no test reads a stale build.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
// The door's tests' services: this binary lends the Range origin, the
// streaming one and the upload one, and the rest go unused.
#[allow(dead_code)]
#[path = "../door/origins.rs"]
mod origins;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use svrngs_core::link::Token;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};
use tokio::io::AsyncWriteExt;

const NS: &str = "house";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// No link is asked about here.
fn no_seed(_: &Path) -> Result<Option<String>, String> {
    Ok(None)
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The page, built by `scripts/build-web.sh` once per run of this binary.
fn site() -> &'static Path {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let out = repo().join("target/web/site");
        let built = std::process::Command::new(repo().join("scripts/build-web.sh"))
            .arg("--out")
            .arg(&out)
            .output()
            .expect("scripts/build-web.sh runs");
        assert!(
            built.status.success(),
            "scripts/build-web.sh failed, so the page cannot be tested:\n{}",
            String::from_utf8_lossy(&built.stderr)
        );
        out
    })
}

/// The relay's address, `http://127.0.0.1:<port>/`.
fn relay_url(relay: &Server) -> String {
    format!(
        "http://{}/",
        relay.http_addr().expect("the relay serves HTTP")
    )
}

/// iroh's relay server on 127.0.0.1, plain HTTP: no TLS, no QUIC.
async fn relay() -> Server {
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    Server::spawn(config)
        .await
        .expect("a relay starts on 127.0.0.1")
}

/// A node whose one relay is `relay`, listening on 127.0.0.1 alone, its
/// group founded by ada (its own key) and holding `admitted` as bo's key,
/// written directly as U1's tests write an `Admit`. Returned once it is
/// online at the relay.
async fn node(dir: &Path, relay: &Server, admitted: &SigningKey) -> Node {
    let node = Node::start(
        dir.to_path_buf(),
        signing(1),
        &RelayConfig::from_parts(vec![relay_url(relay)], Some("none"))
            .unwrap()
            .on_loopback_only(),
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    let seed = Roster::new(
        [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    node.rail().set_seed(NS, &seed).unwrap();
    let fixture = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("bo").unwrap(),
            key: actor_of(admitted),
            invite: None,
        },
        on_behalf_of: None,
    };
    node.rail().commit(NS, &fixture).unwrap().unwrap();
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    node
}

/// A link's token naming `issuer`, reached through `relay`. Its secret
/// redeems nothing here: a member's dial needs only the issuer and the way
/// to it.
fn token(issuer: &SigningKey, relay: &Server) -> String {
    Token {
        issuer: issuer.verifying_key(),
        secret: [0; 16],
        relay: Some(relay_url(relay).parse().expect("a URL")),
    }
    .encode()
}

/// Run the browser half on `args`, and read the JSON line it printed. When
/// `release` fires, the browser half is told `go` (its upload's tail, u4-4).
async fn drive(args: Value, release: Option<tokio::sync::oneshot::Receiver<()>>) -> Value {
    let mut child = tokio::process::Command::new("node")
        .arg(repo().join("web/tests/drive.mjs"))
        .arg(args.to_string())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs: the browser tests need Node 18 or later");
    let mut stdin = child.stdin.take().expect("stdin is piped");
    let told = tokio::spawn(async move {
        if let Some(Ok(())) = match release {
            Some(release) => Some(release.await),
            None => None,
        } {
            let _ = stdin.write_all(b"go\n").await;
            let _ = stdin.flush().await;
        }
        // Held open until the browser half is done: it reads stdin.
        std::future::pending::<()>().await;
    });
    let out = child
        .wait_with_output()
        .await
        .expect("the browser half ran");
    told.abort();
    assert!(
        out.status.success(),
        "the browser half failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.lines().last().expect("the browser half printed");
    serde_json::from_str(line).expect("the browser half printed JSON")
}

/// The page holds bo's key, opens the link, and dials the node by key
/// through the relay the token carries; the node admits bo on the members'
/// protocol and its door answers `GET /lends` 200. The failing input is
/// upstream's one path fact (commonwealth-ai `ralph/DECISIONS.md:9977-9983`):
/// the send half finished before the response.
#[tokio::test(flavor = "multi_thread")]
async fn a_page_holding_a_standing_key_dials_the_node_and_gets_its_lends() {
    let site = site();
    let dir = tempfile::tempdir().unwrap();
    let relay = relay().await;
    let bo = signing(7);
    let node = node(dir.path(), &relay, &bo).await;

    let seen = drive(
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
        }),
        None,
    )
    .await;

    let lines: Vec<&str> = seen["lines"]
        .as_array()
        .expect("the page kept its layers")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let bound = format!(
        "LAYER wasm-bindgen/iroh-endpoint: ok — local endpoint {} bound",
        hex::encode(bo.verifying_key().as_bytes())
    );
    assert!(
        lines.contains(&bound.as_str()),
        "the page's endpoint must be bo's key, the one its store holds:\n{seen:#}"
    );
    assert!(
        lines.contains(&"HTTP: GET /lends answered 200"),
        "the node must answer the page 200; the page saw:\n{seen:#}"
    );
    assert_eq!(seen["status"], "You’re in", "{seen:#}");
    // The page shows the answer's body, parsed by hyper (web/src/http.rs).
    assert_eq!(seen["answer"], r#"{"lends":[]}"#, "{seen:#}");
    drop(node);
}

const FILE_BYTES: u64 = 64 * 1024 * 1024;
const POST_BYTES: u64 = 8 * 1024 * 1024;
const MIB: u64 = 1024 * 1024;

/// The upload's byte at `i`, as the page makes it (web/tests/drive.mjs).
fn posted_at(i: u64) -> u8 {
    ((i * 7 + (i >> 10)) & 255) as u8
}

fn sha(bytes: impl Iterator<Item = u8>) -> String {
    let mut sha = Sha256::new();
    sha.update(bytes.collect::<Vec<u8>>());
    hex::encode(sha.finalize())
}

/// `node` lends `name` on `port`, as the door's tests declare a lend.
fn lend(dir: &Path, name: &str, port: u16) {
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "port": port, "kind": "k", "limit": 3, "refused": [] });
    std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// HTTP over the dial (u4-4): the page, holding bo's key, speaks hyper's
/// HTTP/1 through the node's door to the door's tests' services
/// (tests/door/origins.rs). A range of a 64 MiB file arrives sha-exact and
/// in chunks, never whole; a 206 and a 416 come back whole, status and
/// `Content-Range`; a chunked answer that never ends is decoded as it
/// streams; and an 8 MiB upload reaches the service while the page holds at
/// most 1 MiB of it, then arrives sha-exact. Planted: the response
/// buffered whole; `Range` dropped.
#[tokio::test(flavor = "multi_thread")]
async fn a_page_speaks_http_through_the_door_streaming_both_ways() {
    let site = site();
    let dir = tempfile::tempdir().unwrap();
    let relay = relay().await;
    let bo = signing(7);
    let node = node(dir.path(), &relay, &bo).await;
    let file = dir.path().join("title.bin");
    std::fs::write(
        &file,
        (0..FILE_BYTES).map(origins::byte_at).collect::<Vec<u8>>(),
    )
    .unwrap();
    lend(
        dir.path(),
        "shelf",
        origins::range_origin(file, FILE_BYTES).await.port(),
    );
    let (tap, _) = origins::streaming_origin().await;
    lend(dir.path(), "tap", tap.port());
    let (sink, first) = origins::upload_origin().await;
    lend(dir.path(), "sink", sink.port());

    let (from, to) = (1_000, 9_000_000 - 1);
    let seen = drive(
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "http": {
                "range": { "path": "/lend/shelf/title.bin", "header": format!("bytes={from}-{to}") },
                "past": { "path": "/lend/shelf/title.bin", "header": format!("bytes={FILE_BYTES}-") },
                "chunked": { "path": "/lend/tap/stream", "want": 15 },
                "post": { "path": "/lend/sink/up", "size": POST_BYTES, "chunk": 64 * 1024, "ahead": MIB },
            },
        }),
        Some(first),
    )
    .await;
    let http = &seen["http"];

    let range = &http["range"];
    assert_eq!(
        range["status"], 206,
        "the range must come back 206:\n{http:#}"
    );
    assert_eq!(
        range["contentRange"],
        format!("bytes {from}-{to}/{FILE_BYTES}"),
        "{http:#}"
    );
    assert_eq!(range["bytes"], to - from + 1, "{http:#}");
    assert_eq!(
        range["sha"],
        sha((from..to + 1).map(origins::byte_at)),
        "{http:#}"
    );
    assert!(
        range["chunks"].as_u64().unwrap() > 1 && range["largest"].as_u64().unwrap() <= MIB,
        "the page must be handed the body as it streams, never whole:\n{http:#}"
    );

    let past = &http["past"];
    assert_eq!(past["status"], 416, "{http:#}");
    assert_eq!(
        past["contentRange"],
        format!("bytes */{FILE_BYTES}"),
        "{http:#}"
    );
    assert_eq!(past["bytes"], 0, "{http:#}");

    assert_eq!(http["chunked"]["status"], 200, "{http:#}");
    assert!(
        http["chunked"]["text"]
            .as_str()
            .unwrap()
            .starts_with("hellohellohello"),
        "the chunked answer must be decoded as it streams:\n{http:#}"
    );

    let post = &http["post"];
    assert_eq!(post["status"], 200, "{http:#}");
    assert!(
        post["held"].as_u64().unwrap() <= MIB,
        "the service must have the upload's first byte while the page holds at most 1 MiB:\n{http:#}"
    );
    assert_eq!(
        post["text"],
        format!("{POST_BYTES} {}", sha((0..POST_BYTES).map(posted_at))),
        "{http:#}"
    );
    drop(node);
}
