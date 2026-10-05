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

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use serde_json::{json, Value};
use svrngs_core::link::Token;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};

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

/// Run the browser half on `args`, and read the JSON line it printed.
async fn drive(args: Value) -> Value {
    let out = tokio::process::Command::new("node")
        .arg(repo().join("web/tests/drive.mjs"))
        .arg(args.to_string())
        .output()
        .await
        .expect("node runs: the browser tests need Node 18 or later");
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

    let seen = drive(json!({
        "site": site,
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
    }))
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
        lines.contains(&"RESPONSE-STATUS: HTTP/1.1 200 OK"),
        "the node must answer the page 200; the page saw:\n{seen:#}"
    );
    assert_eq!(seen["status"], "You’re in", "{seen:#}");
    // The page shows the response as the dial read it, head and body; its
    // HTTP is parsed in u4-4.
    let answer = seen["answer"].as_str().unwrap();
    let body = answer.split_once("\r\n\r\n").map(|(_, body)| body);
    assert_eq!(body, Some(r#"{"lends":[]}"#), "{seen:#}");
    drop(node);
}
