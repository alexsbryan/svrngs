//! The page in a browser (DESIGN §3, "Joining"; PLAN §4, U4): the built page,
//! in Playwright's headless Chromium and in its WebKit as an iPhone (u4-7),
//! each test once in each, dials a node by key as a person who stands in
//! its group. Everything runs on this computer: iroh's relay
//! server on 127.0.0.1, plain HTTP, and a node whose only relay it is;
//! the page is served on 127.0.0.1 and its link carries that relay. The
//! browser halves are `web/tests/drive.mjs` and, for the runtime (u4-6),
//! `web/tests/runtime.mjs`, which fail naming the install command when
//! Playwright or its browsers are absent — they never skip; the runtime's
//! film is made by ffmpeg, whose absence fails naming its install too.
//!
//! Each test builds the page first with `scripts/build-web.sh`, once per
//! run, so no test reads a stale build.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
// The door's tests' services: this binary lends the Range origin, the
// streaming one and the upload one, and the rest go unused.
#[allow(dead_code)]
#[path = "../door/origins.rs"]
mod origins;
// The page built, and a relay on 127.0.0.1: shared with the command's
// joining test (crates/svrngs/tests/join_page.rs).
#[macro_use]
mod harness;
// The host's load and the seek bar's, the conformance canary's too.
mod load;
use load::seeks_judged;
use svrngs_node::iroh::Net as _;
// An app the group carries (v1ui-shelf).
mod carried;
// The group's page in two tabs (groups-tabs).
mod tabs;
// A visit made with a link (guests-link).
mod guests;
// A navigation into a lend, judged as a fetch is (capability-navigation).
mod navigations;
// Opening an app is the yes to what it does as you (capability-open).
mod opens;

mod grouped;
mod keep_going;
// The one place a lent film is played (scripts/check.sh `playback`).
mod playback;
mod publication;
// A browser half run and its JSON read: the demo scenes' too
// (crates/svrngs/tests/scenes).
mod drive;

use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use drive::drive;
use harness::{relay, relay_url, repo, site, Engine};
use iroh_relay::server::Server;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use svrngs_core::link::Token;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, Roster, SigningKey};
use tokio::io::AsyncWriteExt;

const NS: &str = "house";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// No link is asked about here.
fn no_seed(_: &Path) -> Result<Option<String>, String> {
    Ok(None)
}

/// The published page reaches nothing on this computer (v1patch-loopback;
/// svrn-docs WHAT_RUNS_AS_YOU.md §2): the `index.html` `scripts/build-web.sh`
/// writes, the build `scripts/deploy-page.sh` ships, holds no `127.0.0.1`
/// and no `localhost` in its `connect-src`.
#[test]
fn the_published_page_allows_no_loopback() {
    let page = std::fs::read_to_string(site().join("index.html")).expect("the built page");
    let policy = page
        .split("Content-Security-Policy\" content=\"")
        .nth(1)
        .and_then(|p| p.split('"').next())
        .expect("the page's policy");
    let connect = policy
        .split(';')
        .map(str::trim)
        .find(|d| d.starts_with("connect-src "))
        .expect("its connect-src");
    let near: Vec<&str> = connect
        .split_whitespace()
        .filter(|t| t.contains("127.0.0.1") || t.contains("localhost"))
        .collect();
    assert!(
        near.is_empty(),
        "the published page may reach this computer by {near:?}, in `{connect}`"
    );
}

/// A node whose one relay is `relay`, listening on 127.0.0.1 alone, its
/// group founded by ada (its own key) and holding `admitted` as bo's key,
/// written directly as U1's tests write an `Admit`. Returned once it is
/// online at the relay.
async fn node(dir: &Path, relay: &Server, admitted: &SigningKey) -> Node {
    computer(dir, relay, admitted, 1, "ada").await
}

/// [`node`], its key `signing(me)` and its founder `founder`.
async fn computer(
    dir: &Path,
    relay: &Server,
    admitted: &SigningKey,
    me: u8,
    founder: &str,
) -> Node {
    computer_on(dir, relay_url(relay), admitted, me, founder).await
}

/// A node of key `signing(me)`, its one relay `relay`, a URL, listening on
/// 127.0.0.1 alone, its group seeded with `founder` holding
/// `signing(founder_key)`.
async fn seeded(dir: &Path, relay: String, me: u8, founder: &str, founder_key: u8) -> Node {
    let node = Node::start(
        dir.to_path_buf(),
        signing(me),
        &RelayConfig {
            relay_urls: vec![relay],
            n0_services: false,
            loopback_only: true,
        },
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    drive::opens_at(node.api_addr().port(), &node.token());
    let seed = Roster::new(
        [(
            Person::parse(founder).unwrap(),
            vec![actor_of(&signing(founder_key))],
        )]
        .into_iter()
        .collect(),
    );
    node.rail().set_seed(NS, &seed).unwrap();
    node
}

/// [`computer`], its relay reached at `relay`, a URL.
async fn computer_on(
    dir: &Path,
    relay: String,
    admitted: &SigningKey,
    me: u8,
    founder: &str,
) -> Node {
    let node = seeded(dir, relay, me, founder, me).await;
    svrngs_core::tests_support::let_in_by(NS, admitted, "bo", |act| {
        let act = svrngs_core::tests_support::Fixture { act };
        node.rail().commit(NS, &act).unwrap().unwrap().lines()[0]
            .id
            .clone()
    });
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    node
}

/// A computer of ada's group: its key `signing(me)` admitted as `person`
/// by `ada`, and ada's record synced to it once, so each lists and lets in
/// the other's people (u5-2: the page lists cy's lends, and a pass names
/// them, only when cy is in the group the phone opened).
async fn joined(dir: &Path, relay: &Server, ada: &Node, me: u8, person: &str) -> Node {
    svrngs_core::tests_support::let_in_by(NS, &signing(me), person, |act| {
        let act = svrngs_core::tests_support::Fixture { act };
        ada.rail().commit(NS, &act).unwrap().unwrap().lines()[0]
            .id
            .clone()
    });
    standing(dir, relay, ada, me, person).await
}

/// The computer of `person`, whose key `signing(me)` ada's group already
/// holds: started, and ada's record synced to it once, as [`joined`] does.
async fn standing(dir: &Path, relay: &Server, ada: &Node, me: u8, person: &str) -> Node {
    standing_under(dir, relay, ada, me, person, "ada").await
}

/// [`standing`], the group's founder, `signing(1)`, named `founder`.
async fn standing_under(
    dir: &Path,
    relay: &Server,
    ada: &Node,
    me: u8,
    person: &str,
    founder: &str,
) -> Node {
    standing_on(dir, relay_url(relay), ada, me, person, founder).await
}

/// [`standing_under`], its relay reached at `relay`, a URL.
async fn standing_on(
    dir: &Path,
    relay: String,
    ada: &Node,
    me: u8,
    person: &str,
    founder: &str,
) -> Node {
    let node = seeded(dir, relay, me, founder, 1).await;
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let synced = node.host().sync_with(NS, &ada.endpoint().addr()).await;
    assert!(
        synced.stop.is_none(),
        "{person} syncs ada's record: {synced:?}"
    );
    // No address lookup here: ada is told where the new computer listens.
    let addrs = node.endpoint().addr().ip_addrs().copied().collect();
    ada.transport().add_known_peer(node.endpoint().id(), addrs);
    node
}

/// A link's token naming `issuer`, reached through `relay`. Its secret
/// redeems nothing here: a member's dial needs only the issuer and the way
/// to it.
fn token(issuer: &SigningKey, relay: &Server) -> String {
    Token {
        issuer: issuer.verifying_key(),
        secret: [0; 16],
        relays: vec![relay_url(relay).parse().expect("a URL")],
    }
    .encode()
}

/// Drive `home.mjs` in `engine` at `at`'s home page, as `args` say.
async fn home_page(engine: Engine, at: &Node, mut args: Value) -> Value {
    args["ask"] = json!(format!("http://svrngs.localhost:{}/", at.api_addr().port()));
    drive(engine, "home.mjs", args, None).await
}

/// `folder` kept as group `NS`'s on `at`, as `share --folder` asks it.
async fn keep_folder(at: &Node, folder: &Path) {
    keep_folder_in(at, folder, NS).await;
}

/// A node as [`node`] makes it, but its group a real one, as production
/// holds it: house's seed drawn by ada, `signing(1)`, kept and read back
/// by `seed_text`, so the node knows the group's name and its own folder's;
/// `admitted` let in as bo. The node, once online, and the group's id. No
/// seed can have `NS` as its id, so a test that needs the group's name
/// holds this group, not `NS`.
async fn house(dir: &Path, relay: &Server, admitted: &SigningKey) -> (Node, String) {
    use svrngs_node::group::{draw, Name};
    let node = Node::start(
        dir.to_path_buf(),
        signing(1),
        &RelayConfig {
            relay_urls: vec![relay_url(relay)],
            n0_services: false,
            loopback_only: true,
        },
        svrngs_node::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    drive::opens_at(node.api_addr().port(), &node.token());
    let ada = Name::parse("ada").unwrap();
    let seed = draw(
        Name::parse("house").unwrap(),
        ada,
        signing(1).verifying_key(),
    );
    node.host().keep_group(&seed, None, &[]).unwrap();
    // bo let in by a link ada's computer mints: house is sources-v1.
    let (ns, rail) = (seed.id(), node.rail());
    svrngs_core::tests_support::let_in(
        (&ns, &actor_of(&signing(1))),
        (admitted, "bo", svrngs_node::Device::Computer),
        |mint| {
            rail.commit(&ns, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(rail.commit(&ns, redeem).unwrap().unwrap()),
    );
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    (node, seed.id())
}

/// `folder` kept as `group`'s on `at`, as `share --folder` asks it.
async fn keep_folder_in(at: &Node, folder: &Path, group: &str) {
    let kept = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("http://{}/v1/folders", at.api_addr()))
        .bearer_auth(at.token())
        .json(&svrngs_node::folder::FolderAsk {
            dir: folder.to_path_buf(),
            group: group.into(),
        })
        .send()
        .await
        .unwrap();
    assert!(kept.status().is_success(), "{:?}", kept.text().await);
}

in_webkit!(
    /// The page holds bo's key, opens the link, and dials the node by key
    /// through the relay the token carries; the node admits bo on the members'
    /// protocol and answers `GET /people/lends` 200 (u4-5; `GET /lends` until
    /// then). The failing input is
    /// upstream's one path fact (commonwealth-ai `ralph/DECISIONS.md:9977-9983`):
    /// the send half finished before the response.
    a_page_holding_a_standing_key_dials_the_node_and_gets_its_lends
);
async fn a_page_holding_a_standing_key_dials_the_node_and_gets_its_lends(engine: Engine) {
    let site = site();
    let dir = tempfile::tempdir().unwrap();
    let relay = relay().await;
    let bo = signing(7);
    let node = node(dir.path(), &relay, &bo).await;

    let seen = drive(
        engine,
        "drive.mjs",
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
        lines.contains(&"HTTP: GET /people/lends answered 200"),
        "the node must answer the page 200; the page saw:\n{seen:#}"
    );
    assert_eq!(seen["status"], "You’re in house, as bo", "{seen:#}");
    // The page shows the answer, parsed by hyper (web/src/http.rs): ada
    // lends nothing.
    assert_eq!(seen["lent"], json!([]), "{seen:#}");
    assert_eq!(
        seen["answer"], "Nobody in house lends anything yet.",
        "{seen:#}"
    );
    // Every address off this computer refused it: n0 unreachable, and
    // never asked (relay-default (b)).
    assert_eq!(seen["away"], json!([]), "{seen:#}");
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
    lend_reading(dir, name, port, &[]);
}

/// [`lend`], its pages reading the group's lends of each kind in `reads`
/// (u5-2, `svrngs share --reads`).
fn lend_reading(dir: &Path, name: &str, port: u16, reads: &[&str]) {
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "groups": [NS], "port": port, "kind": "k", "limit": 3, "refused": [], "reads": reads });
    std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// svrngs's question for the app lent as `app` and the lend `lend`, each
/// `[person, name, kind]`, as its one source words it
/// (`crates/svrngs-node/src/home/question.js`, which the home page and
/// the group's page both load), asked of node (lean-4 (c)).
fn question(app: [&str; 3], lend: [&str; 3]) -> String {
    let as_lend =
        |[person, name, kind]: [&str; 3]| json!({ "person": person, "name": name, "kind": kind });
    let said = in_question("q.question(...args)", json!([as_lend(app), as_lend(lend)]));
    said.as_str().unwrap_or_default().to_string()
}

/// `call`, an expression of question.js as `q` and of `args`, answered by
/// node as JSON: the one source both pages draw their words from.
fn in_question(call: &str, args: Value) -> Value {
    let source = repo().join("crates/svrngs-node/src/home/question.js");
    let script = format!(
        "const q = await import({:?}); const args = JSON.parse(process.argv[1]); console.log(JSON.stringify({call}));",
        format!("file://{}", source.display())
    );
    let out = std::process::Command::new("node")
        .args(["--input-type=module", "-e", &script])
        .arg(args.to_string())
        .output()
        .expect("node runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("node answers JSON")
}

// Pure row/alias/source contracts are web/tests/question.test.mjs's.

/// What the home page and the group's page say of an ask naming an app or
/// a lend not lent there, or not asking: one wording for both, chosen in
/// onair-economize (d) (ralph/REVIEW_FINDINGS.md #113), where the home page
/// said "groups" and the group's page "group".
const UNLENT: &str = "That app, or what it asks to use, is not lent here now.";

/// Where the phone opens `name`, lent by the computer whose key is `seed`'s:
/// an origin of its own, `https://<label>.<page host>:<port>/` (u5-2;
/// harness.rs `PAGE_HOST`).
fn label(seed: u8, name: &str) -> String {
    svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name).to_string()
}

/// Whether `url` is the lend `label`'s own origin: on a computer, or under
/// the page's host on a phone.
fn at_origin(url: &Value, label: &str) -> bool {
    let own = [
        format!("http://{label}.localhost:"),
        format!("https://{label}.{}:", harness::PAGE_HOST),
    ];
    url.as_str()
        .is_some_and(|u| own.iter().any(|o| u.starts_with(o)))
}

in_webkit!(
    /// HTTP over the dial (u4-4): bo's page speaks hyper's HTTP/1, as cy,
    /// through the node's door to the door's tests' services
    /// (tests/door/origins.rs). A range of a 64 MiB file arrives sha-exact and
    /// in chunks, never whole; a 206 and a 416 come back whole, status and
    /// `Content-Range`; a chunked answer that never ends is decoded as it
    /// streams; and an 8 MiB upload reaches the service while the page holds at
    /// most 1 MiB of it, then arrives sha-exact. Planted: the response
    /// buffered whole; `Range` dropped.
    a_page_speaks_http_through_the_door_streaming_both_ways
);
async fn a_page_speaks_http_through_the_door_streaming_both_ways(engine: Engine) {
    let site = site();
    let dir = tempfile::tempdir().unwrap();
    let relay = relay().await;
    let bo = signing(7);
    let node = node(dir.path(), &relay, &bo).await;
    // The checks dial as cy, a second person of the group, under a key of
    // their own: the page already holds an endpoint as bo (web/app.js
    // `enter`), and a relay keeps one client per key, so a second as bo
    // was cut when the page's took the key back, 16-24 s on (arch-stream
    // (a), demo/runs/stream/2026-10-15/README.md, 1).
    let cy = signing(2);
    svrngs_core::tests_support::let_in_by(NS, &cy, "cy", |act| {
        let act = svrngs_core::tests_support::Fixture { act };
        node.rail().commit(NS, &act).unwrap().unwrap().lines()[0]
            .id
            .clone()
    });
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
        engine,
        "drive.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "http": {
                "key": hex::encode(cy.to_bytes()),
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
    // Each step's answer and the page's lines with their times, for a run to
    // read where it stopped (arch-stream (a)).
    eprintln!("streaming::{}: {http}", engine.as_str());

    let range = &http["range"];
    // Its time against drive.mjs's 60 s, passed or failed, for a run to read
    // (`-- --nocapture`; v1ui-lend-ask (b)).
    eprintln!(
        "streaming::{}: the 9 MB range in {} ms of its 60,000",
        engine.as_str(),
        range["ms"]
    );
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

/// A film for `<video>`: 60 s of H.264 at about 4 Mb/s, 29 MB — noise keeps
/// the encoder from shrinking it, so a seek needs bytes not yet fetched —
/// with a keyframe each second and its index first. Made by ffmpeg, which
/// this test needs.
fn film(path: &Path) -> u64 {
    let made = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-f", "lavfi"])
        .args(["-i", "testsrc2=duration=60:size=640x360:rate=24"])
        .args(["-vf", "noise=alls=12:allf=t", "-c:v", "libx264"])
        .args(["-preset", "ultrafast", "-crf", "27", "-pix_fmt", "yuv420p"])
        .args(["-g", "24", "-movflags", "+faststart"])
        .arg(path)
        .output()
        .unwrap_or_else(|e| panic!("ffmpeg makes the test's film — run: brew install ffmpeg\n{e}"));
    assert!(
        made.status.success(),
        "ffmpeg with libx264 makes the test's film — run: brew install ffmpeg\n{}",
        String::from_utf8_lossy(&made.stderr)
    );
    std::fs::metadata(path).unwrap().len()
}

/// A lent page: `html` for every request.
async fn page_origin(html: &'static str) -> std::net::SocketAddr {
    kept_page(html).await.0
}

/// A lent page, `html` for every request, each request's head kept.
async fn kept_page(html: &'static str) -> (std::net::SocketAddr, KeptHeads) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    kept_at(listener, html)
}

/// [`kept_page`] on `listener`, wherever it listens.
fn kept_at(
    listener: tokio::net::TcpListener,
    html: &'static str,
) -> (std::net::SocketAddr, KeptHeads) {
    let addr = listener.local_addr().unwrap();
    let heads = KeptHeads::default();
    let kept = heads.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
                let mut head = [0u8; 8192];
                let n = tokio::io::AsyncReadExt::read(&mut sock, &mut head)
                    .await
                    .unwrap_or(0);
                kept.lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&head[..n]).into_owned());
                // A page is HTML; a body that isn't is data a reader may have.
                let kind = if html.starts_with('<') {
                    "text/html; charset=utf-8"
                } else {
                    "text/plain"
                };
                let answer = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: {kind}\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n{html}",
                    html.len()
                );
                let _ = sock.write_all(answer.as_bytes()).await;
            });
        }
    });
    (addr, heads)
}

type KeptHeads = std::sync::Arc<std::sync::Mutex<Vec<String>>>;

/// A lent page and its film at one origin (u5-2: each lend an origin of
/// its own, as a lent app serves its pages and its media): `/` answered by
/// `page`, every other path by `film`, a request head read to choose and
/// the rest spliced.
async fn one_origin(
    page: std::net::SocketAddr,
    film: std::net::SocketAddr,
) -> std::net::SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut byte = [0u8; 1];
                while !head.ends_with(b"\r\n") {
                    if tokio::io::AsyncReadExt::read(&mut sock, &mut byte)
                        .await
                        .unwrap_or(0)
                        == 0
                    {
                        return;
                    }
                    head.push(byte[0]);
                }
                let to = if head.starts_with(b"GET / ") {
                    page
                } else {
                    film
                };
                let Ok(mut there) = tokio::net::TcpStream::connect(to).await else {
                    return;
                };
                let _ = there.write_all(&head).await;
                let _ = tokio::io::copy_bidirectional(&mut sock, &mut there).await;
            });
        }
    });
    addr
}

/// The `x-svrngs-key` each request head names, as the door set it.
fn keys(heads: &std::sync::Mutex<Vec<String>>) -> Vec<String> {
    heads
        .lock()
        .unwrap()
        .iter()
        .map(|head| {
            head.lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("x-svrngs-key:")
                        .map(|k| k.trim().to_string())
                })
                .unwrap_or_default()
        })
        .collect()
}

/// The byte range each request head asked, `bytes=a-b` as `(a, Some(b))`
/// and `bytes=a-` as `(a, None)`; a head with no range, none.
fn ranges(heads: &std::sync::Mutex<Vec<String>>) -> Vec<(u64, Option<u64>)> {
    heads
        .lock()
        .unwrap()
        .iter()
        .filter_map(|head| {
            let line = head.lines().find_map(|l| {
                l.to_ascii_lowercase()
                    .strip_prefix("range: bytes=")
                    .map(str::to_string)
            })?;
            let (from, to) = line.trim().split_once('-')?;
            Some((from.parse().ok()?, to.parse().ok()))
        })
        .collect()
}

/// The part of a lent film every test but the playback step's has the
/// lend's page fetch (page.mjs `fetched`), its first MiB: what it asks.
const FILM_PART: (u64, u64) = (0, MIB - 1);

/// `FILM_PART` of the film `mp4` as the lend's page fetched it (page.mjs
/// `fetched`, `json!({"path": "title.mp4", ...})` from `film_args`), through
/// the lend's worker: 206, the film's type, the part named of the whole, and
/// the file's bytes; the page's `<video>` sourced from the same file. What
/// a test asserts of a lent film in place of the browser decoding it: that
/// a browser plays it is the playback step's (playback.rs; demo/runs/
/// flakes/2026-11-12), and decoding varies with the host's load and media
/// stack where the bytes do not.
fn film_answered(answer: &Value, mp4: &Path, what: &str, shown: &Value) {
    let film = std::fs::read(mp4).unwrap();
    let (from, to) = FILM_PART;
    let want = json!({
        "status": 206,
        "type": "video/mp4",
        "contentRange": format!("bytes {from}-{to}/{}", film.len()),
        "bytes": to - from + 1,
        "sha": sha(film[from as usize..=to as usize].iter().copied()),
    });
    let got = json!({
        "status": answer["status"],
        "type": answer["type"],
        "contentRange": answer["contentRange"],
        "bytes": answer["bytes"],
        "sha": answer["sha"],
    });
    assert_eq!(
        got, want,
        "{what}: the film's part, through the lend's worker:\n{shown:#}"
    );
    assert!(
        answer["src"]
            .as_str()
            .is_some_and(|s| s.ends_with("/title.mp4")),
        "{what}: the page's <video> is the film's:\n{shown:#}"
    );
}

/// What page.mjs `fetched` takes for `FILM_PART` of the lent page's film.
fn film_args() -> Value {
    let (from, to) = FILM_PART;
    json!({ "path": "title.mp4", "range": format!("bytes={from}-{to}") })
}

/// Chromium's report of its worker: stopped through DevTools, then running
/// again, its endpoint opened when it started and again after the stop.
fn stopped_and_ran_again(seen: &Value, shown: &Value) {
    assert!(
        seen["runsAtStop"]
            .as_array()
            .unwrap()
            .contains(&json!("stopped")),
        "the browser must have stopped the worker:\n{shown:#}"
    );
    let opened = seen["logs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| {
            l.as_str()
                .is_some_and(|l| l.contains("RUNTIME: opening the endpoint"))
        })
        .count();
    assert_eq!(
        opened, 2,
        "the worker opens its endpoint when it starts, and again after it was stopped:\n{seen:#}"
    );
    let runs: Vec<&str> = seen["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let stop = runs.iter().position(|r| *r == "stopped").unwrap();
    assert!(
        runs[stop..].contains(&"running"),
        "the worker must run again after it was stopped:\n{shown:#}"
    );
}

/// A check this test could not judge, said on its output and added to
/// `target/could-not-judge/browser.txt`, which scripts/check.sh prints apart:
/// never a pass (AGENTS.md 6).
fn could_not_judge(line: &str) {
    use std::io::Write;
    eprintln!("could-not-judge: {line}");
    let kept = repo().join("target/could-not-judge");
    std::fs::create_dir_all(&kept).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(kept.join("browser.txt"))
        .unwrap();
    writeln!(file, "{line}").unwrap();
}

// Calibration cases moved unchanged to tests/seek_judgment.rs; the actual timed
// seeks consume this instrument against real playback in playback.rs alone.

/// The measurement an open relay's per-client rate is set from (relay-verb
/// (b); `demo/runs/relay-rate/`): ada's computer lends a film at Demo A's
/// bitrate, 3,730,555 bit/s (Jellyfin's conversion for WebKit,
/// `demo/runs/a/2026-10-11/README.md:38`), and a phone's page in Chromium
/// plays it through a relay on 127.0.0.1 and seeks three times, as Demo A's
/// people do. The relay's bytes received from its clients
/// (`iroh-relay` `bytes_recv`) are sampled every 50 ms; their most in any
/// one second bounds the most any one client sent in it. Written to
/// `$RELAY_RATE_OUT`; a measurement, so ignored by `cargo test`:
///   RELAY_RATE_OUT=<file> cargo test -p svrngs-node --test browser -- --ignored a_films_peak_through_a_relay
#[tokio::test(flavor = "multi_thread")]
#[ignore = "relay-verb's measurement, run by demo/runs/relay-rate/"]
async fn a_films_peak_through_a_relay() {
    let out = std::env::var("RELAY_RATE_OUT").expect("RELAY_RATE_OUT names where it is written");
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let ada_dir = tempfile::tempdir().unwrap();
    let ada = node(ada_dir.path(), &relay, &bo).await;
    let mp4 = ada_dir.path().join("title.mp4");
    let made = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-f", "lavfi"])
        .args(["-i", "testsrc2=duration=120:size=1280x720:rate=24"])
        .args([
            "-vf",
            "noise=alls=12:allf=t",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
        ])
        .args(["-b:v", "3730k", "-maxrate", "3730k", "-bufsize", "3730k"])
        .args(["-pix_fmt", "yuv420p", "-g", "24", "-movflags", "+faststart"])
        .arg(&mp4)
        .output()
        .unwrap();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let size = std::fs::metadata(&mp4).unwrap().len();
    let (film_origin, _) = origins::kept_range_origin(mp4, size).await;
    let html = "<!doctype html><title>a page</title>\
                <video src=\"title.mp4\" playsinline preload=\"auto\"></video>";
    let page = one_origin(page_origin(html).await, film_origin).await;
    lend_reading(ada_dir.path(), "page", page.port(), &["k"]);

    let metrics = relay.metrics().server.clone();
    let (stop, mut stopped) = tokio::sync::oneshot::channel::<()>();
    let sampler = tokio::spawn(async move {
        let t0 = std::time::Instant::now();
        let mut samples = Vec::new();
        loop {
            samples.push(json!([
                t0.elapsed().as_millis() as u64,
                metrics.bytes_recv.get(),
                metrics.bytes_sent.get()
            ]));
            tokio::select! {
                _ = &mut stopped => return samples,
                _ = tokio::time::sleep(Duration::from_millis(50)) => {}
            }
        }
    });
    let seen = drive(
        Engine::Chromium,
        "runtime.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "seeks": [40.0, 20.0, 100.0],
            "other": { "lend": label(1, "page"), "path": "title.mp4", "range": "bytes=0-1023" },
        }),
        None,
    )
    .await;
    stop.send(()).unwrap();
    let samples = sampler.await.unwrap();
    let at = |i: usize, k: usize| samples[i][k].as_u64().unwrap();
    // The most received in any window of 1,000 ms, from each sample on.
    let (mut peak, mut peak_at) = (0, 0);
    for i in 0..samples.len() {
        if let Some(j) = (i..samples.len()).find(|&j| at(j, 0) >= at(i, 0) + 1000) {
            let got = at(j, 1) - at(i, 1);
            if got > peak {
                (peak, peak_at) = (got, at(i, 0));
            }
        }
    }
    let last = samples.len() - 1;
    let written = json!({
        "film_bytes": size,
        "film_seconds": 120,
        "ms": at(last, 0),
        "bytes_recv": at(last, 1),
        "bytes_sent": at(last, 2),
        "peak_bytes_in_a_second": peak,
        "peak_from_ms": peak_at,
        "first": seen["first"],
        "seeks": seen["seeks"],
        "samples": samples,
    });
    std::fs::write(&out, written.to_string()).unwrap();
    eprintln!(
        "the film {size} bytes; the relay received {} in {} ms, at most {peak} in one second (from {peak_at} ms); first play {}, seeks {}",
        at(last, 1),
        at(last, 0),
        seen["first"],
        seen["seeks"]
    );
    assert!(
        seen["first"]["ms"].is_u64(),
        "the film must play:\n{seen:#}"
    );
    drop(ada);
}

/// What the worker traced, read at `/trace` by the browser half.
fn trace_of<'a>(seen: &'a Value, key: &str) -> &'a str {
    seen[key].as_str().unwrap_or_default()
}

in_webkit!(
    /// Safari's worker (u4-10; the operator's iPhone, 2026-10-05, in
    /// demo/runs/u4/2026-10-05-phones/): it read nothing its page had kept,
    /// and never dialled. Since u5-2 the worker that dials is the lend's, at
    /// the lend's own origin, and it reads its origin's store; here it is
    /// planted to read that store empty, its pages not
    /// (web/tests/handover.mjs). The page opens ada's lent page from its
    /// list; the lend's start page hands its worker the pass in memory, and
    /// the worker dials with it: the film's bytes come. The browser stops
    /// the worker and the lend's address is opened again: the worker holds no
    /// pass and reads none, so it sends the page to its start page, which
    /// reads the pass, hands it over and opens the address again, and the
    /// film's bytes come again (`film_answered`); before that, a fetch from
    /// a page it serves is answered by the worker itself, 503, marked svrngs's, a pass refused (mnp-1).
    /// The group's page's own unseal still opens bo's key after, and the
    /// lend's origin keeps the holder as a key, never its bytes. Planted:
    /// the handed pass ignored (lend.js reading only its store); the
    /// worker's mark off. Safari's worker on an iPhone 2026-10-30, which
    /// loses every CryptoKey handed it (`messageerror` in its place,
    /// demo/runs/ios/2026-10-30-observe/keyprobe/phone.log), its start page
    /// handing the holder's bytes instead and the group's page signing to
    /// a new holder after a stop, is web/tests/lend-worker.test.mjs's.
    a_lend_worker_that_reads_no_store_dials_with_the_pass_its_start_page_hands_it
);
async fn a_lend_worker_that_reads_no_store_dials_with_the_pass_its_start_page_hands_it(
    engine: Engine,
) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &bo).await;

    let mp4 = dir.path().join("title.mp4");
    let size = film(&mp4);
    let (film_origin, film_heads) = origins::kept_range_origin(mp4.clone(), size).await;
    let html = "<!doctype html><title>a page</title>\
                <video src=\"title.mp4\" playsinline preload=\"auto\"></video>";
    let page = one_origin(page_origin(html).await, film_origin).await;
    lend(dir.path(), "page", page.port());

    let seen = drive(
        engine,
        "handover.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "film": film_args(),
        }),
        None,
    )
    .await;
    let mut shown = seen.clone();
    shown["logs"] = json!(format!(
        "{} lines",
        seen["logs"].as_array().map_or(0, Vec::len)
    ));
    eprintln!("the hand-over's run: {seen:#}");

    assert_eq!(
        seen["unsealedAfter"],
        hex::encode(bo.to_bytes()),
        "the page's own unseal must still open bo's key: no worker puts a sealing key over the page's:\n{shown:#}"
    );
    assert!(at_origin(&seen["opened"], &label(1, "page")), "{shown:#}");
    assert!(
        trace_of(&seen, "handedTrace").contains("the page handed over the pass"),
        "the lend's worker must dial with the pass its start page handed it:\n{shown:#}"
    );
    film_answered(
        &seen["answered"],
        &mp4,
        "the lend's worker must dial with the handed pass",
        &shown,
    );
    assert_eq!(
        seen["unheld"],
        json!({ "status": 503, "refused": "pass" }),
        "a worker holding no pass must answer a fetch itself, marked svrngs's:\n{shown:#}"
    );
    let stopped = trace_of(&seen, "stoppedTrace");
    let logged = |words: &str| {
        seen["logs"].as_array().is_some_and(|l| {
            l.iter()
                .any(|m| m.as_str().is_some_and(|m| m.contains(words)))
        })
    };
    assert!(
        stopped.contains("is not kept here"),
        "the restarted worker must read nothing of its origin's store (the plant):\n{shown:#}"
    );
    assert!(
        stopped.contains("its start page reads it again")
            && stopped.contains("the page handed over the pass"),
        "the lend's address must go through its start page, which hands the pass over:\n{shown:#}"
    );
    assert!(
        !logged("no bytes of it are here: the group’s page signs a pass to a new holder"),
        "the restarted worker's start page, its kept key taken, must not ask the group's page for a new holder:\n{shown:#}"
    );
    assert_eq!(seen["reopened"], seen["opened"], "{shown:#}");
    film_answered(
        &seen["answeredAgain"],
        &mp4,
        "the restarted worker must dial after the hand-over page",
        &shown,
    );
    let bytes = "and its holder’s bytes, imported here and kept nowhere";
    for (key, words) in [
        ("handedTrace", "the first"),
        ("stoppedTrace", "the stopped"),
    ] {
        assert!(
            !trace_of(&seen, key).contains(bytes),
            "{words} worker must take the holder's bytes only where the key was lost:\n{shown:#}"
        );
    }
    assert!(
        !logged("STORE: the worker said a message was lost"),
        "the start page must hear that its key was lost only when it was:\n{shown:#}"
    );
    assert_eq!(
        seen["kept"]["key"], "CryptoKey",
        "the lend's origin must keep the holder as a key:\n{shown:#}"
    );
    assert!(
        seen["kept"]["fields"]
            .as_array()
            .is_some_and(|f| !f.contains(&json!("seed"))),
        "the lend's origin must never keep the holder's bytes:\n{shown:#}"
    );
    assert!(
        keys(&film_heads).iter().all(|k| *k == actor_of(&bo)),
        "ada's door must name bo's key"
    );
    drop(ada);
}

in_webkit!(
    /// A lent page told the lends it reads, and one opened at a path (mnp-2;
    /// DESIGN §3, "Lent apps", call 6). Ada lends a page reading kind `k`,
    /// cy a shelf of it; dee stands in the group and her computer never
    /// answers. The phone opens ada's page from its list, and at its own
    /// origin the page asks `/svrngs-reads`: cy's shelf, its kind, name and
    /// lender, at an address under the page's own origin, and dee as not
    /// having answered — neither the pass nor its holder key, which the
    /// origin keeps beside them. A fetch at that address reaches cy's shelf
    /// as bo; the page navigated to a path under it lands on cy's shelf's
    /// own origin at that path, its query and fragment kept — through the
    /// group's page, which signs its pass, and its start page, which goes
    /// there — though this phone never opened the shelf, and one Back
    /// returns to ada's page. The phone's open is its yes to the page
    /// reaching cy's shelf as bo (capability-open), and its button says so
    /// (red at 392be359: the name alone). Planted: the whole handoff answered (the pass
    /// in it); the unanswered answered none; the start page going to the
    /// lend's root, the path dropped.
    a_lent_page_is_told_the_lends_it_reads_and_opens_one_at_a_path
);
async fn a_lent_page_is_told_the_lends_it_reads_and_opens_one_at_a_path(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let (ada_dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(ada_dir.path(), &relay, &bo).await;
    let cy = joined(cy_dir.path(), &relay, &ada, 2, "cy").await;
    // Dee stands in the group; no computer of hers ever answers.
    svrngs_core::tests_support::let_in_by(NS, &signing(3), "dee", |act| {
        let act = svrngs_core::tests_support::Fixture { act };
        ada.rail().commit(NS, &act).unwrap().unwrap().lines()[0]
            .id
            .clone()
    });
    let page = page_origin("<!doctype html><title>ada's page</title>").await;
    lend_reading(ada_dir.path(), "page", page.port(), &["k"]);
    let (shelf, heads) = kept_page("<!doctype html><title>cy's shelf</title>").await;
    lend(cy_dir.path(), "shelf", shelf.port());

    let seen = drive(
        engine,
        "reads.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "ask": "probe?n=1",
            "go": "deep/path?n=2#/details?id=3",
            "title": "cy's shelf",
        }),
        None,
    )
    .await;
    eprintln!("the reads' run: {seen:#}");

    assert!(at_origin(&seen["opened"], &label(1, "page")), "{seen:#}");
    assert_eq!(
        seen["words"], "ada’s page (k) Reads cy’s k. They can see it’s you, and what it asks for. Opening it lets it write its own part of house’s record, as you.",
        "the phone's open is its yes, and what it does is said beside it (capability-open):\n{seen:#}"
    );
    let told = &seen["told"];
    assert_eq!(told["status"], 200, "{seen:#}");
    let body = told["body"].as_str().unwrap();
    for kept in ["pass", "holder"] {
        let secret = told[kept]
            .as_str()
            .expect("the origin keeps the pass and its holder");
        assert!(
            !body.contains(secret),
            "the page must be told neither the pass nor its holder key, here its {kept}: {body}"
        );
    }
    let answer: Value = serde_json::from_str(body).unwrap();
    let address = answer["lends"][0]["address"].as_str().unwrap_or_default();
    assert_eq!(
        answer,
        json!({
            "lends": [{ "kind": "k", "name": "shelf", "lender": "cy", "address": address }],
            "unanswered": ["dee"],
        }),
        "{seen:#}"
    );
    assert!(
        at_origin(&json!(address), &label(1, "page"))
            && address.ends_with(&format!("/svrngs-reads/{}/", label(2, "shelf"))),
        "the address is under the page's own origin: {address}"
    );
    // Asked by fetch, the shelf's HTML is refused at the page's own origin
    // (web/lend.js `inert`), though cy's shelf is asked (`asked`, below).
    assert_eq!(seen["asked"]["status"], 403, "{seen:#}");
    let landed = &seen["landed"];
    let shelf_at = landed["url"].as_str().unwrap_or_default();
    assert!(
        at_origin(&landed["url"], &label(2, "shelf"))
            && shelf_at.ends_with("/deep/path?n=2#/details?id=3")
            && landed["title"] == "cy's shelf",
        "the shelf must open at the path asked, at its own origin:\n{seen:#}"
    );
    let asked: Vec<String> = heads
        .lock()
        .unwrap()
        .iter()
        .filter_map(|h| h.lines().next().map(str::to_string))
        .collect();
    assert!(
        asked.iter().any(|l| l.starts_with("GET /probe?n=1 "))
            && asked.iter().any(|l| l.starts_with("GET /deep/path?n=2 ")),
        "cy's shelf must be asked each path: {asked:?}"
    );
    assert!(
        keys(&heads).iter().all(|k| *k == actor_of(&bo)),
        "cy's door must name bo"
    );
    assert!(
        at_origin(&seen["backAt"], &label(1, "page")),
        "one Back returns to ada's page:\n{seen:#}"
    );
    drop((ada, cy));
}

/// The value of the header `name` in the request head `head`, if told.
fn told_in(head: &str, name: &str) -> Option<String> {
    head.lines().find_map(|l| {
        let (n, v) = l.split_once(':')?;
        n.eq_ignore_ascii_case(name).then(|| v.trim().to_string())
    })
}

/// A lent app that writes a link into itself (events-links (a)): `/`
/// answers a page holding `<page>#open=<label>&at=/at/a/path?n=1`, its
/// page the house's in `x-svrngs-pages` and its label `x-svrngs-label`, or
/// no link when either is not told; any other path a page titled "the app
/// at a path". Each request's head kept.
async fn linking_app() -> (std::net::SocketAddr, KeptHeads) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let heads = KeptHeads::default();
    let kept = heads.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
                let mut head = [0u8; 8192];
                let n = tokio::io::AsyncReadExt::read(&mut sock, &mut head)
                    .await
                    .unwrap_or(0);
                let head = String::from_utf8_lossy(&head[..n]).into_owned();
                let html = if head.starts_with("GET / ") {
                    kept.lock().unwrap().push(head.clone());
                    let page = told_in(&head, "x-svrngs-pages").and_then(|p| {
                        let house = p
                            .split(' ')
                            .find_map(|g| g.strip_prefix(&format!("{NS}=")))?;
                        Some(house.to_string())
                    });
                    let link = page
                        .zip(told_in(&head, "x-svrngs-label"))
                        .map(|(page, label)| format!("{page}#open={label}&at=/at/a/path?n=1"));
                    let a = link.map_or(String::new(), |l| {
                        format!("<a id=link href=\"{l}\">the link</a>")
                    });
                    format!("<!doctype html><title>the app</title>{a}")
                } else {
                    "<!doctype html><title>the app at a path</title>".into()
                };
                // A page is HTML; a body that isn't is data a reader may have.
                let kind = if html.starts_with('<') {
                    "text/html; charset=utf-8"
                } else {
                    "text/plain"
                };
                let answer = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: {kind}\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n{html}",
                    html.len()
                );
                let _ = sock.write_all(answer.as_bytes()).await;
            });
        }
    });
    (addr, heads)
}

in_both_engines!(
    /// **A lend's address opens cold in a browser that joined** (svrn-docs
    /// `rings/current/IN_THE_BROWSER.md` §4, the cold bookmark; DESIGN §3,
    /// "Lent apps", call 1). Bo's browser holds the house and has opened
    /// no lend, so no lend's worker is in it: its address, as a bookmark
    /// opens it, at a path, `/?n=1`, reaches the page's host, whose page
    /// sends the browser to the group's page at `#open=<label>&at=<path>`,
    /// which signs a pass and comes back: the app, at that path, at its own
    /// origin. With the worker taken away again at once, its address opened
    /// a second time is said, and never sent to the group's page twice
    /// within the minute. A deep path of it, which the page's host has no
    /// file for, opens the same way in a tab of its own, as the site's
    /// 404.html is the page (scripts/build-web.sh); at the group's own
    /// address such a path says there is no page there. Red on 07eafbd7,
    /// the address's page saying "Open this page from your group’s link.";
    /// the deep path red on 597a8d70, answered 404 with nothing (the
    /// operator's iPhone and curl at group.svrnme.sh: Vercel's NOT_FOUND).
    a_lends_address_opens_cold_in_a_browser_that_joined
);
async fn a_lends_address_opens_cold_in_a_browser_that_joined(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &bo).await;
    let (app, _) = linking_app().await;
    lend(dir.path(), "app", app.port());
    let label = label(1, "app");
    let args = json!({
        "site": site,
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "cold": label,
        "missing": true,
    });
    let seen = drive(engine, "cold.mjs", args, None).await;
    eprintln!("the cold address's run: {seen:#}");
    let cold = &seen["cold"];
    assert!(
        at_origin(&cold["url"], &label)
            && cold["url"].as_str().unwrap().ends_with("/?n=1")
            && cold["title"] == "the app at a path",
        "a lend's address opens cold, at its path, at its own origin:\n{seen:#}"
    );
    let again = &seen["again"];
    assert_eq!(
        (&again["title"], &again["status"]),
        (
            &json!("You’re invited to a group"),
            &json!("This could not be opened. Open it again from your group’s page.")
        ),
        "a second cold open within the minute is said, never sent on again:\n{seen:#}"
    );
    assert!(at_origin(&again["url"], &label), "{seen:#}");
    let deep = &seen["deep"];
    assert!(
        at_origin(&deep["url"], &label)
            && deep["url"]
                .as_str()
                .unwrap()
                .ends_with("/at/a/deep/path?n=3")
            && deep["title"] == "the app at a path",
        "a deep path the page's host has no file for opens cold too, by its 404.html:\n{seen:#}"
    );
    assert_eq!(
        seen["missing"]["status"],
        "There’s no page at this address. Open your group’s link to reach your group.",
        "a path of the group's own address that is no page is said so:\n{seen:#}"
    );
    drop(ada);
}

in_both_engines!(
    /// **A lend's alias opens it from the group's page** (svrn-docs
    /// `rings/current/IN_THE_BROWSER.md`, "Decided"; DESIGN §3, "Lent
    /// apps", call 1). Two people called ada each lend `app` in the house,
    /// the second let in by hand. Bo's page opens each by its alias at
    /// `#open=<alias>`, the name at whose it is, told apart as the page
    /// tells the two adas apart, by when each joined: `app@ada (started
    /// the group)` the founder's app, `app@ada (joined <day>)` the
    /// other's, each at its own origin. The bare `app@ada`, which names
    /// both, and `nothing@ada`, which names nothing, are said in the
    /// page's words, never opened as a nearest match. In a computer's
    /// browser each row gives its alias as the address to keep, "Bookmark
    /// this.", and each opened lands on its own app, the first from the
    /// group's page itself; "How this reaches you" is there, in the page's
    /// words. On a phone neither: it keeps the group's page on its Home
    /// Screen. Red on 3587c98f, where
    /// `#open=` read a label alone, so every alias opened nothing and said
    /// nothing; the keeping red on a27fd4a5, which gave no address.
    a_lends_alias_opens_it_from_the_groups_page
);
async fn a_lends_alias_opens_it_from_the_groups_page(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let (dir, other_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(dir.path(), &relay, &bo).await;
    let other = joined(other_dir.path(), &relay, &ada, 2, "ada").await;
    let (app, _) = linking_app().await;
    lend(dir.path(), "app", app.port());
    let second = page_origin("<!doctype html><title>the app of the other ada</title>").await;
    lend(other_dir.path(), "app", second.port());
    let today = chrono::Utc::now().format("%-d %b").to_string();
    let theirs = format!("app@ada (joined {today})");
    let aliases = [
        "app@ada (started the group)",
        theirs.as_str(),
        "app@ada",
        "nothing@ada",
    ];
    let args = json!({
        "site": site,
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "aliases": aliases,
        "keep": true,
    });
    let seen = drive(engine, "cold.mjs", args, None).await;
    eprintln!("the aliases' run: {seen:#}");
    // Each row's address to keep, its alias, and the one line for it
    // (IN_THE_BROWSER §4, "Saying it"); each opened lands on its own app.
    // On a phone none: it keeps the group's page on its Home Screen.
    let line = " Bookmark this.";
    let kept: Vec<String> = serde_json::from_value(seen["keep"].clone()).unwrap_or_default();
    let mut shown: Vec<String> = kept
        .iter()
        .map(|k| k.strip_suffix(line).unwrap_or(k).to_string())
        .collect();
    shown.sort();
    let at = |a: &str| {
        a.split_once("/#open=")
            .map(|(_, a)| a.to_string())
            .unwrap_or_default()
    };
    if matches!(engine, Engine::Webkit) {
        assert_eq!(seen["keep"], json!([null, null]), "{seen:#}");
    } else {
        assert_eq!(
            shown.iter().map(|a| at(a)).collect::<Vec<_>>(),
            [
                "app@ada+(joined+".to_string() + &today.replace(' ', "+") + ")",
                "app@ada+(started+the+group)".into()
            ],
            "each row's alias to keep, told apart, with its line:\n{seen:#}"
        );
        assert!(kept.iter().all(|k| k.ends_with(line)), "{seen:#}");
        let mut opened: Vec<(bool, Value)> =
            (seen["kept"].as_array().cloned().unwrap_or_default().iter())
                .map(|k| (at_origin(&k["url"], &label(1, "app")), k["title"].clone()))
                .collect();
        opened.sort_by_key(|(first, _)| *first);
        assert_eq!(
            opened,
            [
                (false, json!("the app of the other ada")),
                (true, json!("the app"))
            ],
            "each address kept opens its own app:\n{seen:#}"
        );
    }
    // How this reaches you, for someone at a computer, in the page's words
    // (DESIGN §1); on a phone, out of Ana's first look.
    let how = &seen["how"];
    assert_eq!(how["hidden"], matches!(engine, Engine::Webkit), "{seen:#}");
    let told = how["text"].as_str().unwrap_or_default().to_lowercase();
    let universe = [
        "key", "member", "ring", "sync", "journal", "node", "relay", "vpn", "tunnel",
    ];
    assert!(
        told.contains("how this reaches you") && words_said(&told, &universe).is_empty(),
        "{seen:#}"
    );
    let landed = |i: usize| &seen["aliases"][i];
    assert!(
        at_origin(&landed(0)["url"], &label(1, "app")) && landed(0)["title"] == "the app",
        "the founder's app by its alias:\n{seen:#}"
    );
    assert!(
        at_origin(&landed(1)["url"], &label(2, "app"))
            && landed(1)["title"] == "the app of the other ada",
        "the other ada's app by its alias, told apart by when she joined:\n{seen:#}"
    );
    for (i, alias) in [(2, "app@ada"), (3, "nothing@ada")] {
        let status = landed(i)["status"].as_str().unwrap_or_default();
        assert!(
            status.starts_with("You’re in house, as bo")
                && landed(i)["said"] == format!("Nothing in house goes by {alias} now."),
            "{alias} opens nothing, and is said:\n{seen:#}"
        );
    }
    drop((ada, other));
}

// Alias collisions and time zones, a link's token with what it opens, and a
// fragment the page cannot read are Node laws (question.test.mjs,
// pages/enter.test.mjs); the alias and cold-open tests above still land in
// the browser.

in_webkit!(
    /// A lent page asks its person through the group's page, and is stopped
    /// (sup-3; DESIGN §3, "What an app gets"). Ada lends a page reading kind
    /// `k`, cy a shelf of it that asks, in cy's words. At its origin ada's
    /// page is told the shelf's words and an `ask` address, `…/svrngs-ask?
    /// lend=<shelf>&app=<page>`, and is refused `unasked`. Sent there with
    /// `&back=`, the group's page asks "Let ada's page use cy's shelf?"
    /// around cy's words; its Yes is posted to cy's door as bo, and the page
    /// opens again at the path it gave, carried now, cy's shelf told the app.
    /// The group's page's "What apps use" says so, and its Stop refuses the
    /// page `stopped`; said yes again, the page's own stop, sent by the
    /// worker with the pass, refuses it again. The group's page framed by
    /// the lent page is not shown in the frame, and its origin keeps no
    /// text that is its pass's holder key, only a key object that cannot
    /// be copied (capability-signed (H); a pass's own yes is refused in
    /// tests/door/refusals.rs). Asked of an app and a lend not lent, the group's page
    /// says [`UNLENT`], as the home page does (onair-economize (d); red at
    /// a20ed46: "... in your group now."). Planted, each watched failing: a
    /// holder's yes accepted at the door; the group's page framed (no
    /// `frame-ancestors`); the ask's `back` dropped.
    a_lent_page_asks_its_person_through_the_group_s_page_and_is_stopped
);
async fn a_lent_page_asks_its_person_through_the_group_s_page_and_is_stopped(engine: Engine) {
    const WORDS: &str = "the page receives what is on the shelf; nothing leaves cy's computer";
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let (ada_dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(ada_dir.path(), &relay, &bo).await;
    let cy = joined(cy_dir.path(), &relay, &ada, 2, "cy").await;
    let page = page_origin("<!doctype html><title>ada's page</title>").await;
    lend_reading(ada_dir.path(), "page", page.port(), &["k"]);
    let (shelf, heads) = kept_page("what is on the shelf").await;
    let lends = svrngs_node::lend::lends_dir(cy_dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "groups": [NS], "port": shelf.port(), "kind": "k", "limit": 3, "refused": [], "asks": WORDS });
    std::fs::write(lends.join("shelf.json"), body.to_string()).unwrap();
    let (page_label, shelf_label) = (label(1, "page"), label(2, "shelf"));
    // Bo stopped 200 other apps on the shelf before: what apps use is then
    // about 19 KB, past the page's first read of an answer (hyper's 8 KiB),
    // so its body comes after its head on every run, as a lost or late
    // packet makes it come on a busy host or a phone's link
    // (bash-film-load).
    let stopped: serde_json::Map<String, Value> = (0..200)
        .map(|i| {
            (
                label(9, &format!("app-{i}")),
                json!({ "said": "stop", "at": 1 }),
            )
        })
        .collect();
    let shelf_name = svrngs_node::lend::LendName::try_from("shelf".to_string()).unwrap();
    let choices = svrngs_node::lend::choices_path(cy_dir.path(), &shelf_name);
    std::fs::create_dir_all(choices.parent().unwrap()).unwrap();
    let kept = json!({ actor_of(&bo): stopped });
    std::fs::write(&choices, kept.to_string()).unwrap();

    let seen = drive(
        engine,
        "ask.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "probe": "probe?n=1",
            "back": "/after?n=1",
        }),
        None,
    )
    .await;
    eprintln!("the ask's run: {seen:#}");
    let refused = |step: &str| (seen[step]["status"].clone(), seen[step]["refused"].clone());

    let told = &seen["told"]["lends"][0];
    assert_eq!(told["asks"], WORDS, "{seen:#}");
    let ask = told["ask"].as_str().unwrap_or_default();
    assert!(
        at_origin(&json!(ask), &page_label)
            && ask.ends_with(&format!("/svrngs-ask?lend={shelf_label}&app={page_label}")),
        "the ask address, at the page's own origin: {ask}"
    );
    let stop = told["stop"].as_str().unwrap_or_default();
    assert!(
        at_origin(&json!(stop), &page_label)
            && stop.ends_with(&format!("/svrngs-reads/{shelf_label}/svrngs-stop")),
        "the stop address, beside ask, at the page's own origin: {stop}"
    );
    assert_eq!(
        refused("before"),
        (json!(403), json!("unasked")),
        "{seen:#}"
    );
    for asked in ["asked", "again"] {
        let asked = &seen[asked];
        assert_eq!(
            asked["heading"],
            question(["ada", "page", "k"], ["cy", "shelf", "k"]),
            "{seen:#}"
        );
        assert_eq!(asked["words"], WORDS, "{seen:#}");
        let landed = asked["landed"].as_str().unwrap_or_default();
        assert!(
            at_origin(&asked["landed"], &page_label) && landed.ends_with("/after?n=1"),
            "the page opens again at the path it gave: {landed}"
        );
    }
    for yes in ["yes", "yesAgain"] {
        assert_eq!(refused(yes), (json!(200), json!(null)), "{yes}: {seen:#}");
    }
    let last = heads.lock().unwrap().last().cloned().unwrap_or_default();
    assert!(
        last.to_lowercase()
            .contains(&format!("x-svrngs-app: {page_label}")),
        "cy's shelf is told the app: {last}"
    );
    let used = seen["used"].as_str().unwrap_or_default();
    assert!(
        used.contains("ada’s page (k) uses cy’s shelf (k)"),
        "{seen:#}"
    );
    let unused = seen["unused"].as_str().unwrap_or_default();
    assert!(
        unused.contains("ada’s page (k) was stopped using cy’s shelf (k)"),
        "{seen:#}"
    );
    assert_eq!(seen["appStop"]["status"], 200, "{seen:#}");
    for stopped in ["pageStopped", "appStopped"] {
        assert_eq!(
            refused(stopped),
            (json!(403), json!("stopped")),
            "{stopped}: {seen:#}"
        );
    }
    assert_eq!(seen["unlent"], UNLENT, "{seen:#}");
    assert_eq!(
        seen["framed"],
        json!([]),
        "the group's page must never show in a lent page's frame:\n{seen:#}"
    );
    // What its origin keeps (capability-signed (H); DESIGN §3): no text
    // kept with its pass is the key the pass names as its holder, so no
    // page there copies it to act as bo elsewhere, and the key object kept
    // in its place gives up no bytes.
    let kept = &seen["kept"];
    let pass = svrngs_core::pass::Pass::decode(kept["pass"].as_str().unwrap_or_default())
        .unwrap_or_else(|| panic!("the origin keeps its pass: {seen:#}"));
    let texts = kept["texts"].as_array().into_iter().flatten();
    let copied = texts
        .filter_map(|t| <[u8; 32]>::try_from(hex::decode(t.as_str()?).ok()?).ok())
        .any(|b| actor_of(&SigningKey::from_bytes(&b)) == pass.holder);
    assert!(
        !copied,
        "a page at the lend's origin reads its holder's key as bytes: {seen:#}"
    );
    assert_eq!(
        kept["exported"],
        json!(["InvalidAccessError"]),
        "the holder kept as one key that cannot be copied: {seen:#}"
    );
    drop((ada, cy));
}

/// Ada's computer removes `who` from house as `svrngs remove` asks it.
async fn removal(ada: &Node, who: &str) -> svrngs_node::removal::Outcome {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!(
            "http://{}{}",
            ada.api_addr(),
            svrngs_node::removal::REMOVALS_ROUTE
        ))
        .bearer_auth(ada.token())
        .json(&svrngs_node::removal::RemovalAsk {
            group: NS.into(),
            person: who.into(),
            joined: None,
        })
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

in_chromium!(
    /// The group's link on a computer (v1-onboarding (c); DESIGN §7): the
    /// page offers the one line that gets svrngs, composed from the
    /// link's `#` part, and a button that copies it, which puts it on the
    /// clipboard; on a phone it offers none. No request either page makes,
    /// the worker's and the relay's included, carries the token, and the
    /// page host's answer to `/` previews the link as an invitation,
    /// naming no group. The link here lets nobody in, so each page settles
    /// on its refusal. Watched failing: the page before the offer, and the
    /// line sent in a request. In Chromium alone, whose clipboard a test
    /// may read: the line, its words, Copy and a phone's none are
    /// web/tests/pages/group-page.test.mjs's ('a computer is offered one
    /// line'), for both engines.
    the_page_offers_a_computer_one_line_and_sends_the_token_nowhere
);
async fn the_page_offers_a_computer_one_line_and_sends_the_token_nowhere(engine: Engine) {
    let site = site();
    let dir = tempfile::tempdir().unwrap();
    let relay = relay().await;
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let token = token(&signing(1), &relay);
    let seen = drive(
        engine,
        "computer.mjs",
        json!({ "site": site, "token": token }),
        None,
    )
    .await;
    let origin = seen["origin"].as_str().unwrap();
    let line = format!("curl -fsSL {origin}install.sh | sh -s -- '{origin}#{token}'");
    let computer = &seen["computer"];
    assert_eq!(computer["offered"], true, "{computer:#}");
    assert_eq!(computer["line"], line, "{computer:#}");
    assert_eq!(computer["button"], "Copied", "{computer:#}");
    assert_eq!(computer["written"], line, "{computer:#}");
    assert_eq!(computer["copied"], line, "{computer:#}");
    assert_eq!(seen["phone"]["offered"], false, "{:#}", seen["phone"]);
    let requests = seen["requests"].as_array().unwrap();
    eprintln!(
        "offered on a computer: {computer:#}; on a phone: {}; {} requests from the pages, {} at the host",
        seen["phone"]["offered"],
        requests.len(),
        seen["asked"].as_array().unwrap().len()
    );
    assert!(!requests.is_empty(), "no request was seen: {seen:#}");
    for r in requests.iter().chain(seen["asked"].as_array().unwrap()) {
        assert!(
            !r.to_string().contains(&token),
            "the token left the page: {r:#}"
        );
    }
    let served = seen["served"].as_str().unwrap();
    for meta in [
        "<title>You’re invited to a group</title>",
        r#"<meta property="og:title" content="You’re invited to a group">"#,
        r#"<meta name="description" content="#,
        r#"<meta property="og:description" content="#,
    ] {
        assert!(served.contains(meta), "the preview lacks {meta}");
    }
    assert!(!served.contains("house") && !served.contains(&token));
    drop(ada);
}

/// A computer in no group: a node with no seed and nothing in its record.
async fn alone(dir: &Path, relay: &Server) -> Node {
    let node = Node::start(
        dir.to_path_buf(),
        signing(3),
        &RelayConfig {
            relay_urls: vec![relay_url(relay)],
            n0_services: false,
            loopback_only: true,
        },
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    drive::opens_at(node.api_addr().port(), &node.token());
    node
}

in_chromium!(
    /// **The home page in no group, as a person first sees it**
    /// (start-home; the operator, 2026-10-26), the home page's one test in
    /// a real browser (docs/CHECKS.md, seam B): it loads under its policy
    /// (home.rs `POLICY`) with no page error and no refusal of the policy's,
    /// and what only layout decides holds. One look in June's forty words
    /// (demo/xenia/people/june.md), in xenia's order, things to tap first,
    /// starts at the box for the link someone sent her (page.mjs
    /// `firstLook`; the fifth set's June read 80-119 words there,
    /// demo/xenia/findings/wordy-home.md); and look.css hides "Done in a
    /// terminal", as nobody is there to remove. Its words, and what the
    /// page draws in no group, are web/tests/pages/home.test.mjs's ('the
    /// page says what it is for'). Planted, watched failing: index.html
    /// and home.js as c6c2212 left them; red at 5c6e3b0d, making first.
    the_home_page_says_what_it_is_for
);
async fn the_home_page_says_what_it_is_for(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let dee = alone(dir.path(), &relay).await;
    let args = json!({ "all": "#home h2", "look": 40, "text": true });
    let seen = home_page(engine, &dee, args).await;
    let look = seen["look"].as_str().unwrap_or_default();
    assert!(
        look.starts_with("The link someone sent you: Your name in it:"),
        "{seen:#}"
    );
    let text = seen["text"].as_str().unwrap_or_default();
    assert!(!text.contains("Done in a terminal"), "{seen:#}");
    let logs = seen["logs"].as_array().cloned().unwrap_or_default();
    let faults = (logs.iter().filter_map(Value::as_str))
        .filter(|l| l.starts_with("pageerror") || l.contains("Content Security Policy"))
        .collect::<Vec<_>>();
    assert!(faults.is_empty(), "{faults:#?}");
    drop(dee);
}

/// Each of `words`, or its plural, that `text` holds as a whole word.
fn words_said<'a>(text: &str, words: &[&'a str]) -> Vec<&'a str> {
    let said: BTreeSet<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    (words.iter().copied())
        .filter(|w| said.contains(w) || said.contains(format!("{w}s").as_str()))
        .collect()
}

in_chromium!(
    /// **Open versus files** (docs/APP_ACCESS_AND_STOP.md A1, A2, §5; start-home,
    /// from start-measure's ranked row 12: the folder was a path read as
    /// words, `demo/runs/start/2026-10-13/README.md` §3 (c) 14). Ada lends
    /// her shelf and keeps house's folder. Her page's "Open ada’s shelf
    /// (k)" opens the shelf's own page in a tab, by the keyboard and by
    /// the mouse; "Open shared files in Finder", or "in your file manager"
    /// off a Mac, asks this computer's file browser to open the folder,
    /// and says so. The browser is a planted `open` and `xdg-open`, first
    /// on the test's `PATH`, writing what it was asked to open beside it,
    /// so no window opens, and the open is judged by what it was asked,
    /// never by the page staying still. Planted, watched failing: the
    /// page as ab6ef1f left it, with no such button; and the link saying
    /// only the lend's name, as lend-name.js did before A1.
    the_home_page_opens_the_group_s_folder
);
async fn the_home_page_opens_the_group_s_folder(engine: Engine) {
    use std::os::unix::fs::PermissionsExt;
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let (folder, bin) = (dir.path().join("house"), dir.path().join("bin"));
    std::fs::create_dir(&folder).unwrap();
    std::fs::create_dir(&bin).unwrap();
    // Written beside what it opens, so whichever engine's bin is found
    // first, each test reads its own: both run at once in this process.
    let opened = dir.path().join("house.opened");
    for opener in ["open", "xdg-open"] {
        let at = bin.join(opener);
        std::fs::write(&at, "#!/bin/sh\necho \"$1\" >> \"$1.opened\"\n").unwrap();
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    // The planted bin first: every other program is found as before. Read
    // and set under one lock, else both engines read the same `PATH` and
    // the one set last drops the other's bin: once its own test ends, the
    // other finds the real `open` (REVIEW-audit-v1ui-auto-1's check run).
    static PATH_SET: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let set = PATH_SET.lock().unwrap();
    let path = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(bin.clone()).chain(std::env::split_paths(&path));
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    drop(set);
    keep_folder(&ada, &folder).await;
    let shelf =
        page_origin("<!doctype html><title>ada's shelf</title><body>the shelf itself").await;
    lend(dir.path(), "shelf", shelf.port());
    let seen = home_page(
        engine,
        &ada,
        json!({
            "all": "#home section p",
            "labelled": "Open ada’s shelf (k)",
            "do": [["#home section button:has-text('Open shared files in ')", null, true]],
        }),
    )
    .await;
    eprintln!("the home page's run: {seen:#}");
    let tabs = seen["labelled"].as_array().cloned().unwrap_or_default();
    assert_eq!(
        tabs.len(),
        2,
        "the keyboard and the mouse each open it: {seen:#}"
    );
    for tab in &tabs {
        assert!(at_origin(&tab["url"], &label(1, "shelf")), "{seen:#}");
        assert_eq!(tab["words"], "the shelf itself", "{seen:#}");
    }
    let said = seen["after"][0].to_string();
    let files = [
        "Open shared files in Finder",
        "Open shared files in your file manager",
    ];
    assert!(
        files
            .iter()
            .any(|f| said.contains(&format!("{f} Opened, in a window of its own."))),
        "{seen:#}"
    );
    let asked = std::fs::read_to_string(&opened).unwrap_or_default();
    assert!(
        asked.lines().any(|l| l == folder.display().to_string()),
        "{asked:?}"
    );
    drop(ada);
}

/// V1_UI_BUILD §7's 3.3 bar with margin (v1ui-race (a)): how long the
/// phone may take, with its link's computer stopped, to list the group's
/// things and open one through another person's computer. The bar is 15 s;
/// this is the link's cap, `dial::LINK_WITHIN`, 10 s, as the page dials
/// every computer it knows at once and so never waits that cap out, where
/// dialling one after another it waited all of it before asking the next,
/// and missed 15 s by 347 and 705 ms in a full check
/// (`demo/runs/v1ui-voice/2026-10-15/check-1.out`). Missed on a host past
/// `load::BUSY`, it is could-not-judge, as the film's seeks are.
const REACHED_WITHIN: u64 = svrngs_node::dial::LINK_WITHIN.as_millis() as u64;

/// Bo's phone opens ada's group while every computer answers, then again
/// once each of ada's and cy's that `stops` names is stopped: reach.mjs's
/// run, with the host's load at its start. Ada's computer made bo's link;
/// cy's lends a shelf. What the relay carried over the second visit is
/// said, the dials' cost (v1ui-race (a)).
async fn with_stopped(engine: Engine, stops: [bool; 2]) -> (Value, String) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let (ada_dir, cy_dir, kept) = (
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
    );
    let ada = node(ada_dir.path(), &relay, &bo).await;
    let cy = joined(cy_dir.path(), &relay, &ada, 2, "cy").await;
    let shelf = page_origin("<!doctype html><title>cy's shelf</title>").await;
    lend(cy_dir.path(), "shelf", shelf.port());
    let ready = kept.path().join("ready");
    let (go, release) = tokio::sync::oneshot::channel();
    let args = json!({
        "site": site,
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "ready": ready,
        "open": "shelf",
        "title": "cy's shelf",
    });
    let start = load::load().await;
    let phone = tokio::spawn(drive(engine, "reach.mjs", args, Some(release)));
    tokio::time::timeout(Duration::from_secs(120), async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("the phone opens the group while ada's computer answers");
    let (stopped, awake): (Vec<_>, Vec<_>) =
        [ada, cy].into_iter().zip(stops).partition(|(_, s)| *s);
    for (computer, _) in &stopped {
        computer.endpoint().close().await;
    }
    drop(stopped);
    let carried = || {
        let m = &relay.metrics().server;
        [
            m.bytes_recv.get(),
            m.send_packets_recv.get(),
            m.send_packets_dropped.get(),
        ]
    };
    let before = carried();
    go.send(()).unwrap();
    let seen = phone.await.unwrap();
    let after = carried();
    let [bytes, packets, dropped] = std::array::from_fn(|i| after[i] - before[i]);
    eprintln!("the phone's run: {seen:#}");
    eprintln!(
        "{}: the relay over the second visit, {stops:?} stopped, listed in {} ms as {}: {bytes} bytes, {packets} packets, {dropped} dropped",
        engine.as_str(),
        seen["listed"]["ms"],
        seen["listed"]["status"]
    );
    drop(awake);
    (seen, start)
}

in_webkit!(
    /// The phone reaches whoever's awake (v1ui-reach (b)-(c); V1_UI_BUILD
    /// §3.3). Ada's computer made bo's link; cy's lends a shelf. Bo's phone
    /// opens the group while ada's answers, then again once ada's is
    /// stopped: within the bar it lists cy's shelf through cy's computer,
    /// says so and says ada's did not answer, and opens the shelf; past the
    /// bar on a busy host, could-not-judge, never a pass. Planted:
    /// the page dialling only the link's computer, as before this row,
    /// which waits as iroh does, 30 s, then says it did not answer; and the
    /// page dialling the others only once the link's cap ran out, as before
    /// v1ui-race, past [`REACHED_WITHIN`].
    the_phone_reaches_whoever_s_awake
);
async fn the_phone_reaches_whoever_s_awake(engine: Engine) {
    let (seen, start) = with_stopped(engine, [true, false]).await;
    assert_eq!(
        seen["first"]["status"], "You’re in house, as bo",
        "{seen:#}"
    );
    let listed = &seen["listed"];
    assert_eq!(
        listed["status"], "You’re in house, as bo, through cy’s computer",
        "the page says whose computer it reached the group through:\n{seen:#}"
    );
    assert!(
        listed["said"]
            .as_str()
            .is_some_and(|s| s.contains("ada’s computer did not answer")),
        "{seen:#}"
    );
    let opened = &seen["opened"];
    assert_eq!(opened["title"], "cy's shelf", "{seen:#}");
    let ms = opened["ms"].as_u64().unwrap_or(u64::MAX);
    if ms > REACHED_WITHIN {
        let missed = format!("cy's shelf opened in {ms} ms, past the bar's {REACHED_WITHIN}");
        match load::past_busy(&start, load::cores()) {
            Some(why) => could_not_judge(&format!(
                "the_phone_reaches_whoever_s_awake::{}: {missed}; {why}; {start} … {}",
                engine.as_str(),
                load::load().await
            )),
            None => panic!("{missed}, at {start}:\n{seen:#}"),
        }
    }
}

/// What the page's dials cost with every computer awake (v1ui-race (a)):
/// [`with_stopped`]'s line on the relay over the second visit, stopping
/// nobody, for `demo/runs/race/`. An instrument, not a check.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "v1ui-race (a)'s measurement, run by name"]
async fn the_dials_cost_with_everyone_awake() {
    for engine in [Engine::Chromium, Engine::Webkit] {
        with_stopped(engine, [false, false]).await;
    }
}

/// A way to `relay` that holds each new connection `by` before passing it
/// on, so a dial through it lands that much after one through the relay
/// itself: which computer answers first, made so rather than raced.
async fn slow_way(relay: &Server, by: Duration) -> String {
    let to = harness::relay_at(relay);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let at = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut near, _)) = listener.accept().await {
            tokio::spawn(async move {
                tokio::time::sleep(by).await;
                if let Ok(mut far) = tokio::net::TcpStream::connect(to).await {
                    let _ = tokio::io::copy_bidirectional(&mut near, &mut far).await;
                }
            });
        }
    });
    format!("http://{at}/")
}

/// One test at a time holds 8096, Jellyfin's, on this host: the bar's
/// port in both engines' runs of [`the_home_page_lends_what_runs_here`].
static JELLYFIN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// A port from 20000 up that nothing on this host holds now, inside the
/// probe's span and below the system's own (Linux's from 32768), with its
/// listener: dropped, nothing listens there.
async fn free_here(from: u16) -> tokio::net::TcpListener {
    for port in from..30000 {
        if let Ok(l) = tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            return l;
        }
    }
    panic!("no port free from {from} on this host");
}

in_chromium!(
    /// **Lend from the page** (v1ui-lend (b); V1_UI_BUILD §3.2 and §7's
    /// 3.2 bar). On ada's computer a stand-in for Jellyfin listens on
    /// 8096 and something svrngs does not know on a port of its own. Her
    /// home page looks for them only once she asks, its look knocking at
    /// every address here; then, under "Already running here", it names
    /// the first by the table with what it needs first (its own accounts,
    /// and how to make one) and lends it in one tap; the second it lends only once she has
    /// given it a name and said what it is, refusing it before. No
    /// terminal: each is declared through the page's `/running`, and the
    /// share line under "Done in a terminal" is gone. Then bo, on his own
    /// computer, opens ada's jellyfin from his home page in a tab. Nothing
    /// is offered where nobody listens: neither a port gone before the
    /// page asked nor the table's Navidrome and Calibre-Web unless they
    /// run on this host. Planted, watched failing: the page offering the
    /// table's ports whether or not anything answers on them; the page
    /// looking as it loads.
    the_home_page_lends_what_runs_here
);
async fn the_home_page_lends_what_runs_here(engine: Engine) {
    let _held = JELLYFIN.lock().await;
    // The other engine's run lets 8096 go as its runtime ends, after the lock.
    let mut bound = tokio::net::TcpListener::bind("127.0.0.1:8096").await;
    for _ in 0..50 {
        if bound.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        bound = tokio::net::TcpListener::bind("127.0.0.1:8096").await;
    }
    let Ok(jellyfin) = bound else {
        return could_not_judge(&format!(
            "the_home_page_lends_what_runs_here::{}: something on this host holds 8096, Jellyfin's",
            engine.as_str()
        ));
    };
    kept_at(jellyfin, "<!doctype html><title>Jellyfin</title>");
    let (chores, _) = kept_at(
        free_here(20000).await,
        "<!doctype html><title>chores</title>",
    );
    let gone = free_here(chores.port() + 1)
        .await
        .local_addr()
        .unwrap()
        .port();
    let relay = relay().await;
    let (ada_dir, bo_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(ada_dir.path(), &relay, &signing(7)).await;
    let bo = standing(bo_dir.path(), &relay, &ada, 7, "bo").await;
    let row = |port: u16| format!("#running li:has-text('localhost:{port}')");
    let press = |port: u16| json!([format!("{} button", row(port)), null, true]);
    let fill = |port: u16, n: &str, v: &str| json!([format!("{} input[name={n}]", row(port)), v]);
    // A person ticks who can open it before lending it (SCOPE_PARALLEL.md:
    // never a default): the row offers HOME's one group.
    let tick = |port: u16| json!([format!("{} input[type=checkbox]", row(port)), true]);
    let look = json!(["#running button:has-text('Look for programs')", null, true]);
    let seen = home_page(
        engine,
        &ada,
        json!({
            "read": "#lent h2",
            "timed": ["#lent h2", "#running li"],
            "all": "#running li, #running p",
            "do": [look, tick(8096), press(8096), tick(chores.port()), press(chores.port()),
                fill(chores.port(), "name", "chores"), fill(chores.port(), "kind", "chore-wheel"),
                press(chores.port())],
            // The running row answers before HOME's lending reread. An old
            // list still matches `#lent h2 ~ *` (grouped-home check.log).
            "after": ["#lent li", "chores: everyone in the groups you chose at once"],
            "text": true,
        }),
    )
    .await;
    eprintln!("ada's home page: {seen:#}");
    let said = |v: &Value, port: u16| -> String {
        let at = format!("localhost:{port}");
        let lines = v.as_array().cloned().unwrap_or_default();
        lines
            .iter()
            .filter_map(Value::as_str)
            .find(|l| {
                (l.match_indices(&at))
                    .any(|(i, _)| !l[i + at.len()..].starts_with(|c: char| c.is_ascii_digit()))
            })
            .unwrap_or("")
            .into()
    };
    // Looked for only once asked: the look knocks at every address here.
    let (unasked, found) = (&seen["all"], &seen["after"][0]);
    assert!(
        unasked.to_string().contains("only when you ask") && said(unasked, 8096).is_empty(),
        "looked for before she asked:\n{seen:#}"
    );
    assert!(
        found.to_string().contains("You looked just now"),
        "that she looked, said:\n{seen:#}"
    );
    let first = said(found, 8096);
    assert!(
        first.contains("jellyfin") && first.contains("Dashboard, Users"),
        "jellyfin named, with how to make each person an account first:\n{seen:#}"
    );
    assert!(said(found, chores.port()).contains("Its name"), "{seen:#}");
    assert!(
        first.contains("Who can open jellyfin?"),
        "the row asks who can open it, a group to tick:\n{seen:#}"
    );
    let quiet = |p: u16| std::net::TcpStream::connect(("127.0.0.1", p)).is_err();
    for port in [gone, ada.api_addr().port()]
        .into_iter()
        .chain([4533, 8083].into_iter().filter(|p| quiet(*p)))
    {
        assert_eq!(
            said(found, port),
            "",
            "offered where nobody listens, or svrngs's own: {port}\n{seen:#}"
        );
    }
    let after = &seen["after"];
    let lent = |i: usize, words: &str| {
        let line = after[i].to_string();
        assert!(line.contains(words), "{words}: {seen:#}");
    };
    lent(1, "You lent jellyfin");
    lent(2, "Give it a name and say which program it is first");
    lent(3, "You lent chores");
    let page = seen["text"].as_str().unwrap_or_default();
    assert!(
        !page.contains("svrngs share"),
        "the terminal's line is gone:\n{page}"
    );
    for lent in ["jellyfin", "chores"] {
        assert!(
            page.contains(&format!("{lent}: everyone in the groups you chose at once")),
            "what this computer lends, drawn again in place: {lent}\n{page}"
        );
    }
    let read = |name: &str| {
        svrngs_node::lend::read(
            ada_dir.path(),
            &svrngs_node::lend::LendName::parse(name).unwrap(),
        )
        .unwrap()
        .map(|l| (l.source, l.kind.to_string()))
    };
    let port = |p: u16| svrngs_node::lend::Source::Port(p.try_into().unwrap());
    assert_eq!(read("jellyfin"), Some((port(8096), "jellyfin".into())));
    assert_eq!(
        read("chores"),
        Some((port(chores.port()), "chore-wheel".into()))
    );
    let name = svrngs_node::lend::LendName::parse("chores").unwrap();
    let lent_to = svrngs_node::lend::read(ada_dir.path(), &name)
        .unwrap()
        .unwrap();
    let house = svrngs_node::lend::Audience::Selected(vec![NS.to_string()]);
    assert_eq!(lent_to.audience, house, "lent to the group ticked");

    let opened = home_page(engine, &bo, json!({ "open": "ada’s jellyfin" })).await;
    eprintln!("bo's home page: {opened:#}");
    let tab = &opened["opened"];
    assert!(
        at_origin(&tab["url"], &label(1, "jellyfin")) && tab["title"] == "Jellyfin",
        "bo opens ada's jellyfin in a tab:\n{opened:#}"
    );
    drop((ada, bo));
}

/// A stand-in for llama-server's chat page (`demo/runs/start/
/// 2026-10-13-story/` 9): `415 Unsupported Media Type` to a request that
/// does not ask for gzip, and to one that does, its page gzipped, as
/// llama-server answers `GET /`. The page is "llama chat", gzip's bytes of
/// `<!doctype html><title>llama chat</title><p>Unpacked on the phone.`.
async fn gzip_only() -> std::net::SocketAddr {
    const PAGE: [u8; 81] = [
        31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 179, 81, 76, 201, 79, 46, 169, 44, 72, 85, 200, 40, 201,
        205, 177, 179, 41, 201, 44, 201, 73, 181, 203, 201, 73, 204, 77, 84, 72, 206, 72, 44, 177,
        209, 135, 136, 216, 20, 216, 133, 230, 21, 36, 38, 103, 167, 166, 40, 228, 231, 41, 148,
        100, 164, 42, 20, 100, 228, 231, 165, 234, 1, 0, 35, 241, 10, 241, 65, 0, 0, 0,
    ];
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut head = [0u8; 8192];
                let n = tokio::io::AsyncReadExt::read(&mut sock, &mut head)
                    .await
                    .unwrap_or(0);
                let head = String::from_utf8_lossy(&head[..n]).to_ascii_lowercase();
                let gzip = head
                    .lines()
                    .any(|l| l.starts_with("accept-encoding:") && l.contains("gzip"));
                let mut answer = if gzip {
                    format!("HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\ncontent-encoding: gzip\r\ncontent-length: {}\r\nconnection: close\r\n\r\n", PAGE.len()).into_bytes()
                } else {
                    b"HTTP/1.1 415 Unsupported Media Type\r\ncontent-type: text/plain\r\ncontent-length: 34\r\nconnection: close\r\n\r\ngzip is not supported by this one".to_vec()
                };
                if gzip {
                    answer.extend_from_slice(&PAGE);
                }
                let _ = sock.write_all(&answer).await;
            });
        }
    });
    addr
}

in_webkit!(
    /// **A phone opens a lend that answers only in gzip** (V1.1.3; the
    /// story run's finding, `demo/runs/start/2026-10-13-story/` 9 and §4):
    /// ada lends a stand-in for llama-server's chat page, which answers
    /// 415 unless asked for gzip, and bo's phone opens it from the group's
    /// list: its page, unpacked, "llama chat". Planted: HEAD before it,
    /// whose worker never asked for gzip, so the phone was shown the 415.
    a_phone_opens_a_lend_that_answers_only_in_gzip
);
async fn a_phone_opens_a_lend_that_answers_only_in_gzip(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &bo).await;
    lend(dir.path(), "chat", gzip_only().await.port());
    let args = json!({
        "site": site,
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "open": "chat",
        "title": "llama chat",
    });
    let seen = drive(engine, "night.mjs", args, None).await;
    eprintln!("the phone's run: {seen:#}");
    let opened = &seen["opened"];
    assert_eq!(opened["title"], "llama chat", "{seen:#}");
    assert!(
        opened["text"]
            .as_str()
            .is_some_and(|t| t.contains("Unpacked on the phone.")),
        "{seen:#}"
    );
    drop(ada);
}
