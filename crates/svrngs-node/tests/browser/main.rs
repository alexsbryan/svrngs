//! The page in a browser (DESIGN §3, "Joining"; §8, U4): the built page,
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
// The bridge to a door as a pass holder, the door's tests' own.
#[path = "../common/mod.rs"]
mod common;
// The page built, and a relay on 127.0.0.1: shared with the command's
// joining test (crates/svrngs/tests/join_page.rs).
#[macro_use]
mod harness;
// The host's load and the seek bar's, the conformance canary's too.
mod load;

use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use common::bridge_as;
use harness::{relay, relay_url, repo, site, Engine};
use iroh_relay::server::Server;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use svrngs_core::link::{Origin, Token};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Joined, Person, RailAct, Roster, SigningKey};
use tokio::io::AsyncWriteExt;

const NS: &str = "house";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// No link is asked about here.
fn no_seed(_: &Path) -> Result<Option<String>, String> {
    Ok(None)
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
    let node = Node::start(
        dir.to_path_buf(),
        signing(me),
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
    let seed = Roster::new(
        [(
            Person::parse(founder).unwrap(),
            vec![actor_of(&signing(me))],
        )]
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

/// A computer of ada's group: its key `signing(me)` admitted as `person`
/// by `ada`, and ada's record synced to it once, so each lists and lets in
/// the other's people (u5-2: the page lists cy's lends, and a pass names
/// them, only when cy is in the group the phone opened).
async fn joined(dir: &Path, relay: &Server, ada: &Node, me: u8, person: &str) -> Node {
    let fixture = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse(person).unwrap(),
            key: actor_of(&signing(me)),
            invite: None,
        },
        on_behalf_of: None,
    };
    ada.rail().commit(NS, &fixture).unwrap().unwrap();
    let node = Node::start(
        dir.to_path_buf(),
        signing(me),
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
    let seed = Roster::new(
        [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    node.rail().set_seed(NS, &seed).unwrap();
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let synced = node.sync_with(NS, &ada.endpoint().addr()).await;
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
        relay: Some(relay_url(relay).parse().expect("a URL")),
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
    let kept = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("http://{}/v1/folders", at.api_addr()))
        .json(&svrngs_node::folder::FolderAsk {
            dir: folder.to_path_buf(),
            group: NS.into(),
        })
        .send()
        .await
        .unwrap();
    assert!(kept.status().is_success(), "{:?}", kept.text().await);
}

/// Run the browser half `web/tests/<script>` in `engine` on `args`, and
/// read the JSON line it printed. When `release` fires, the browser half is told `go`
/// (its upload's tail, u4-4).
async fn drive(
    engine: Engine,
    script: &str,
    mut args: Value,
    release: Option<tokio::sync::oneshot::Receiver<()>>,
) -> Value {
    args["engine"] = json!(engine.as_str());
    let mut child = tokio::process::Command::new("node")
        .arg(repo().join("web/tests").join(script))
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

in_both_engines!(
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

in_both_engines!(
    /// A link naming no relay — from a computer that had n0's address
    /// lookup — is refused in a person's words, and the page reaches for
    /// no address off this computer, n0's lookup and relays among them,
    /// each refused it (relay-default (b)). Planted, watched failing: the
    /// page before it, which bound with n0's lookup and relays.
    a_link_naming_no_relay_is_refused_and_n0_never_reached
);
async fn a_link_naming_no_relay_is_refused_and_n0_never_reached(engine: Engine) {
    let dir = tempfile::tempdir().unwrap();
    let relay = relay().await;
    let bo = signing(7);
    let node = node(dir.path(), &relay, &bo).await;
    let bare = Token {
        issuer: signing(1).verifying_key(),
        secret: [0; 16],
        relay: None,
    };
    let seen = drive(
        engine,
        "drive.mjs",
        json!({ "site": site(), "key": hex::encode(bo.to_bytes()), "token": bare.encode() }),
        None,
    )
    .await;
    assert_eq!(seen["away"], json!([]), "n0 never asked:\n{seen:#}");
    assert_eq!(
        seen["status"], "This link was made by an older svrngs. Ask whoever sent it for a new one.",
        "{seen:#}"
    );
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
    let body = json!({ "port": port, "kind": "k", "limit": 3, "refused": [], "reads": reads });
    std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// svrngs's question for the app lent as `app` and the lend `lend`, each
/// `[person, name, kind]`, as its one source words it
/// (`crates/svrngs-node/src/home/question.js`, which the home page and
/// the group's page both load), asked of node (lean-4 (c)).
fn question(app: [&str; 3], lend: [&str; 3]) -> String {
    let source = repo().join("crates/svrngs-node/src/home/question.js");
    let script = format!(
        "const q = await import({:?}); const [a, l] = JSON.parse(process.argv[1]); console.log(q.question(a, l));",
        format!("file://{}", source.display())
    );
    let as_lend =
        |[person, name, kind]: [&str; 3]| json!({ "person": person, "name": name, "kind": kind });
    let out = std::process::Command::new("node")
        .args(["--input-type=module", "-e", &script])
        .arg(json!([as_lend(app), as_lend(lend)]).to_string())
        .output()
        .expect("node runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// What the home page and the group's page say of an ask naming an app or
/// a lend not lent there, or not asking: one wording for both, chosen in
/// onair-economize (d) (ralph/REVIEW_FINDINGS.md #113), where the home page
/// said "groups" and the group's page "group".
const UNLENT: &str = "That app, or what it asks to use, is not lent here now.";

/// Where the phone opens `name`, lent by the computer whose key is `seed`'s:
/// an origin of its own, `http://<label>.localhost:<port>/` (u5-2).
fn label(seed: u8, name: &str) -> String {
    svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name).to_string()
}

/// Whether `url` is the lend `label`'s own origin on this computer.
fn at_origin(url: &Value, label: &str) -> bool {
    url.as_str()
        .is_some_and(|u| u.starts_with(&format!("http://{label}.localhost:")))
}

in_both_engines!(
    /// HTTP over the dial (u4-4): the page, holding bo's key, speaks hyper's
    /// HTTP/1 through the node's door to the door's tests' services
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
                let answer = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\n\
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

/// Demo A's seek bar on the film's seeks, each playing again within five
/// seconds, judged as the conformance canary judges it (`load`): `Ok(None)`
/// when each did; when one missed, on a host whose load at the run's start
/// was past `load::BUSY` a core, `Ok(Some(could-not-judge …))`, never a
/// pass; and on a host under it, `Err` naming the misses, which fails.
fn seeks_judged(seeks: &[Value], load: &str, cores: usize) -> Result<Option<String>, String> {
    let missed: Vec<String> = seeks
        .iter()
        .filter(|s| s["ms"].as_u64().is_none_or(|ms| ms > 5_000))
        .map(Value::to_string)
        .collect();
    if missed.is_empty() {
        return Ok(None);
    }
    load::past_busy(load, cores)
        .map(Some)
        .ok_or_else(|| missed.join(", "))
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

/// The film's seeks judged by `seeks_judged`, from `load()`'s lines as
/// macOS and Linux print them, on 12 cores: the bar is 36. The misses are
/// bash-first-minute-1's, the third seek again at 11,424 ms after the
/// worker's stop (ralph/next/bash/ctl/log/sessions/bash-first-minute-1.await.log).
mod film_seeks_by_load {
    use super::*;
    const QUIET: &str =
        "2026-10-12T05:00:00Z 22:00  up 3 days, 2 users, load averages: 35.9 20.1 12.0";
    const BUSY_MAC: &str =
        "2026-10-12T05:00:00Z 22:00  up 3 days, 2 users, load averages: 62.1 40.3 30.1";
    const BUSY_LINUX: &str =
        "2026-10-12T05:00:00Z 05:00:00 up 3 days, 2 users, load average: 62.10, 40.30, 30.10";
    const UNREAD: &str = "2026-10-12T05:00:00Z";

    fn seeks(ms: [Value; 3]) -> Vec<Value> {
        ms.into_iter().map(|ms| json!({ "ms": ms })).collect()
    }

    #[test]
    fn a_seek_missed_at_low_load_fails() {
        let missed = seeks([json!(1853), json!(2701), json!(11424)]);
        let never = seeks([json!(null), json!(null), json!(531)]);
        for load in [QUIET, UNREAD] {
            assert_eq!(
                seeks_judged(&missed, load, 12),
                Err(r#"{"ms":11424}"#.to_string())
            );
            assert_eq!(
                seeks_judged(&never, load, 12),
                Err(r#"{"ms":null}, {"ms":null}"#.to_string())
            );
        }
    }

    #[test]
    fn a_seek_missed_past_the_bar_could_not_judge() {
        let missed = seeks([json!(1853), json!(2701), json!(11424)]);
        for load in [BUSY_MAC, BUSY_LINUX] {
            assert_eq!(
                seeks_judged(&missed, load, 12),
                Ok(Some(
                    "could-not-judge, load 62.1 on 12 cores, past 3 a core".to_string()
                ))
            );
        }
    }

    #[test]
    fn seeks_held_passed_at_any_load() {
        let held = seeks([json!(1853), json!(2701), json!(5000)]);
        for load in [QUIET, BUSY_MAC, UNREAD] {
            assert_eq!(seeks_judged(&held, load, 12), Ok(None));
        }
    }
}

in_both_engines!(
    /// The runtime (u4-6; DESIGN §3, "Joining", calls 3 and 4): ada's computer
    /// lends a page and a film, an H.264 mp4 from upstream's Range origin, and
    /// cy's, another computer holding bo's key, lends a file. The phone, holding
    /// bo's key, opens ada's page from its list: since u5-2 it opens at an
    /// origin of its own, `<label>.localhost`, apart from the page that
    /// holds bo's key, and that origin's worker serves it, its `<video>` and
    /// every byte range from ada's door, with the pass the page signed. The film plays and seeks three times,
    /// each playing again within five seconds (Demo A's seek bar), a miss on
    /// a host past `load::BUSY` a core could-not-judge (`seeks_judged`); the browser
    /// stops the worker before the third, and the next ranges still answer:
    /// Chromium stops it through DevTools, the page left playing, and WebKit,
    /// whose worker Playwright cannot reach, is closed and opened again on its
    /// profile, as a phone ends a browser left in the background, and the page
    /// reopens at the lend's address (web/tests/runtime.mjs). A
    /// range of cy's file, asked at its own origin by ada's page, which reads
    /// lends of its kind (`--reads`, so the pass names cy's file too), comes
    /// back 206 and sha-exact, and cy's door names bo's key, not ada's: no
    /// computer carries another's lend. An open-ended range of cy's file, `bytes=N-`,
    /// comes back a bounded part, a 206 naming it and the whole, sha-exact
    /// (u4-9, web/sw.js `PART`), and every range the film's `<video>` asks
    /// reaches ada's door as a part of at most 2 MiB: Chromium asks
    /// `bytes=N-`, WebKit `bytes=N-<the end>`, and WebKit read such a range
    /// through before it asked for a seek's bytes (ralph/REVIEW_FINDINGS.md
    /// #53). A 403 cy's `strict` answers comes back unmarked, the
    /// `x-svrngs-refused` it forged dropped, and cy's door's refusal of a
    /// path it is lent without comes back marked `path` (mnp-1). Planted: a
    /// range answered 200 with the whole body (the worker dropping `Range`);
    /// the worker's endpoint never reopened (opened once, when it activates);
    /// the open range answered whole (u4-8's worker); a `<video>`'s closed
    /// range to the end answered whole (b6ea0ab's); the door's mark off.
    a_lent_film_plays_and_seeks_under_the_worker_which_reaches_each_lender_itself
);
async fn a_lent_film_plays_and_seeks_under_the_worker_which_reaches_each_lender_itself(
    engine: Engine,
) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let (ada_dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(ada_dir.path(), &relay, &bo).await;
    let cy = joined(cy_dir.path(), &relay, &ada, 2, "cy").await;
    let (ada_key, bo_key) = (actor_of(&signing(1)), actor_of(&bo));

    let mp4 = ada_dir.path().join("title.mp4");
    let size = film(&mp4);
    let (film_origin, film_heads) = origins::kept_range_origin(mp4, size).await;
    let html = "<!doctype html><title>a page</title>\
                <video src=\"title.mp4\" playsinline preload=\"auto\"></video>";
    let page = one_origin(page_origin(html).await, film_origin).await;
    lend_reading(ada_dir.path(), "page", page.port(), &["k"]);
    const OTHER: u64 = 16 * MIB;
    let file = cy_dir.path().join("title.bin");
    std::fs::write(&file, (0..OTHER).map(origins::byte_at).collect::<Vec<u8>>()).unwrap();
    let (shelf, shelf_heads) = origins::kept_range_origin(file, OTHER).await;
    lend(cy_dir.path(), "shelf", shelf.port());
    let strict = origins::refusing_origin().await;
    let lends = svrngs_node::lend::lends_dir(cy_dir.path());
    let body = json!({ "port": strict.port(), "kind": "k", "limit": 3, "refused": ["/private"] });
    std::fs::write(lends.join("strict.json"), body.to_string()).unwrap();

    let (from, to) = (1_000_000, 1_000_000 + 4 * MIB - 1);
    // The host's load as the film starts, which its seeks are judged at.
    let began = load::load().await;
    let seen = drive(
        engine,
        "runtime.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "seeks": [40.0, 20.0, 52.0],
            "other": { "lend": label(2, "shelf"), "path": "title.bin", "range": format!("bytes={from}-{to}") },
            "openEnded": { "lend": label(2, "shelf"), "path": "title.bin", "range": format!("bytes={from}-") },
            "refusals": [
                { "lend": label(2, "strict"), "path": "x" },
                { "lend": label(2, "strict"), "path": "private/x" },
            ],
        }),
        None,
    )
    .await;
    let mut shown = seen.clone();
    shown["logs"] = json!(format!(
        "{} lines",
        seen["logs"].as_array().map_or(0, Vec::len)
    ));
    eprintln!("the runtime's run: {shown}");

    assert!(at_origin(&seen["opened"], &label(1, "page")), "{seen:#}");
    assert!(
        seen["first"]["ms"].is_u64(),
        "the film must play:\n{seen:#}"
    );
    let seeks = seen["seeks"].as_array().unwrap();
    assert_eq!(seeks.len(), 3, "{shown:#}");
    match seeks_judged(seeks, &began, load::cores()) {
        Ok(None) => {}
        Ok(Some(busy)) => could_not_judge(&format!(
            "a_lent_film_plays_and_seeks_under_the_worker_which_reaches_each_lender_itself::{} \
             each seek within five seconds: {busy}, the host at {began}; the seeks played again in {} ms",
            engine.as_str(),
            seeks.iter().map(|s| s["ms"].to_string()).collect::<Vec<_>>().join(", ")
        )),
        Err(missed) => panic!(
            "each seek must play again within five seconds, the host at {began}; \
             missed {missed}:\n{seen:#}"
        ),
    }
    match engine {
        Engine::Chromium => stopped_and_ran_again(&seen, &shown),
        Engine::Webkit => {
            // No worker outlives its browser; WebKit's worker, which Playwright
            // gives no console, shows its new endpoint by the film playing
            // again at the lend's address, which only the worker serves.
            assert_eq!(seen["reopened"], seen["opened"], "{shown:#}");
            assert!(
                seen["replayed"]["ms"].is_u64(),
                "the film must play again once the browser is opened again:\n{shown:#}"
            );
        }
    }

    let film_keys = keys(&film_heads);
    assert!(
        film_heads
            .lock()
            .unwrap()
            .iter()
            .any(|h| h.to_ascii_lowercase().contains("\r\nrange: bytes=")),
        "the film's ranges must reach its service:\n{:#?}",
        film_heads.lock().unwrap()
    );
    assert!(
        film_keys.iter().all(|k| *k == bo_key),
        "ada's door must name bo's key: {film_keys:?}"
    );
    // Each of the film's ranges as the door was asked it: a part, closed
    // and at most web/sw.js's PART, never a range that runs on to the end.
    let asked = ranges(&film_heads);
    let long: Vec<&(u64, Option<u64>)> = asked
        .iter()
        .filter(|(from, to)| to.is_none_or(|to| to + 1 - from > 2 * MIB))
        .collect();
    assert!(
        long.is_empty(),
        "the film's ranges must reach its door as parts of at most 2 MiB, of {size} bytes; \
         these ran on (from, to): {long:?}\nall: {asked:?}"
    );

    let other = &seen["other"];
    assert_eq!(
        other["status"], 206,
        "the range must come back 206:\n{shown:#}"
    );
    assert_eq!(
        other["contentRange"],
        format!("bytes {from}-{to}/{OTHER}"),
        "{shown:#}"
    );
    assert_eq!(other["bytes"], to - from + 1, "{shown:#}");
    assert_eq!(
        other["sha"],
        sha((from..to + 1).map(origins::byte_at)),
        "{shown:#}"
    );
    // The open range: a part from `from`, named by its 206, never the
    // rest of the file.
    let part = &seen["openEnded"];
    let got = part["bytes"].as_u64().unwrap_or(0);
    assert_eq!(
        part["status"], 206,
        "the open range must come back 206:\n{shown:#}"
    );
    assert!(
        got > 0 && got < OTHER - from,
        "the open range must come back a part, not the {} bytes to the end:\n{shown:#}",
        OTHER - from
    );
    assert_eq!(
        part["contentRange"],
        format!("bytes {from}-{}/{OTHER}", from + got - 1),
        "{shown:#}"
    );
    assert_eq!(
        part["sha"],
        sha((from..from + got).map(origins::byte_at)),
        "{shown:#}"
    );
    // Whose word each answer is (mnp-1): the lend's own 403 unmarked, the
    // header it forged dropped; the door's refusal of a path, through the
    // worker, marked svrngs's.
    assert_eq!(
        seen["refusals"],
        json!([
            { "path": "x", "status": 403, "refused": null },
            { "path": "private/x", "status": 403, "refused": "path" },
        ]),
        "{shown:#}"
    );
    let shelf_keys = keys(&shelf_heads);
    assert!(
        !shelf_keys.is_empty() && shelf_keys.iter().all(|k| *k == bo_key),
        "cy's door must name bo's key, the viewer's, never ada's ({ada_key}): {shelf_keys:?}"
    );
    drop((ada, cy));
}

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

in_both_engines!(
    /// Safari's worker (u4-10; the operator's iPhone, 2026-10-05, in
    /// demo/runs/u4/2026-10-05-phones/): it read nothing its page had kept,
    /// and never dialled. Since u5-2 the worker that dials is the lend's, at
    /// the lend's own origin, and it reads its origin's store; here it is
    /// planted to read that store empty, its pages not
    /// (web/tests/handover.mjs). The page opens ada's lent page from its
    /// list; the lend's start page hands its worker the pass in memory, and
    /// the worker dials with it: the film plays. The browser stops the
    /// worker and the lend's address is opened again: the worker holds no
    /// pass and reads none, so it sends the page to its start page, which
    /// reads the pass, hands it over and opens the address again, and the
    /// film plays; before that, a fetch from a page it serves is answered
    /// by the worker itself, 503, marked svrngs's, a pass refused (mnp-1).
    /// The group's page's own unseal still opens bo's key after. Planted:
    /// the handed pass ignored (lend.js reading only its store); the
    /// worker's mark off.
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
    let (film_origin, film_heads) = origins::kept_range_origin(mp4, size).await;
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
    assert!(
        seen["first"]["ms"].is_u64(),
        "the film must play with the handed pass:\n{shown:#}"
    );
    assert_eq!(
        seen["unheld"],
        json!({ "status": 503, "refused": "pass" }),
        "a worker holding no pass must answer a fetch itself, marked svrngs's:\n{shown:#}"
    );
    let stopped = trace_of(&seen, "stoppedTrace");
    assert!(
        stopped.contains("is not kept here"),
        "the restarted worker must read nothing of its origin's store (the plant):\n{shown:#}"
    );
    assert!(
        stopped.contains("its start page reads it again")
            && stopped.contains("the page handed over the pass"),
        "the lend's address must go through its start page, which hands the pass over:\n{shown:#}"
    );
    assert_eq!(seen["reopened"], seen["opened"], "{shown:#}");
    assert!(
        seen["replayed"]["ms"].is_u64(),
        "the film must play after the hand-over page:\n{shown:#}"
    );
    assert!(
        keys(&film_heads).iter().all(|k| *k == actor_of(&bo)),
        "ada's door must name bo's key"
    );
    drop(ada);
}

in_both_engines!(
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
    /// returns to ada's page. Planted: the whole handoff answered (the pass
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
    let dee = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("dee").unwrap(),
            key: actor_of(&signing(3)),
            invite: None,
        },
        on_behalf_of: None,
    };
    ada.rail().commit(NS, &dee).unwrap().unwrap();
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
    assert_eq!(seen["asked"]["status"], 200, "{seen:#}");
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

in_both_engines!(
    /// A lent app writes a link into itself that opens on a phone
    /// (events-links (a); DESIGN §3, "What an app gets"). The door tells
    /// the app its label in `x-svrngs-label` and, in `x-svrngs-pages`, the
    /// page of each group held here that the key asking stands in: bo's
    /// phone the house's alone; ada's computer, at the app's origin on its
    /// loopback, the house's and dweb's, which keeps none. The app writes
    /// `<page>#open=<label>&at=<path>` from them (web/app.js), and the
    /// link, opened in a new tab of bo's phone, lands on the app at that
    /// path. On a computer whose browser keeps no group (events-links (c)),
    /// the group's page says the link opens in svrngs there and shows it to
    /// copy; pasted on the computer's home page under "Open a link from
    /// your group", it opens the app at that path in a tab of its own.
    /// Planted: no headers, as before this test; for the computer, the
    /// page's and the home page's as they were, which said "Open this page
    /// from your group's link." and had no box.
    a_lent_app_writes_a_link_into_itself_that_opens_on_a_phone
);
async fn a_lent_app_writes_a_link_into_itself_that_opens_on_a_phone(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &bo).await;
    let alone = Roster::new(
        [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    ada.rail().set_seed("dweb", &alone).unwrap();
    // The group's page, served by the browser half on a port chosen here.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let page = format!("http://localhost:{port}/");
    let house = ada.rail().group_dir(NS).unwrap();
    svrngs_node::group::write_origin(&house, &Origin::parse(&page).unwrap()).unwrap();
    let (app, heads) = linking_app().await;
    lend(dir.path(), "app", app.port());
    let label = label(1, "app");

    let own = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .get(format!("http://{}/", ada.api_addr()))
        .header(
            "host",
            format!("{label}.localhost:{}", ada.api_addr().port()),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(own.status(), 200, "{:?}", own.text().await);
    let told = |name: &str| {
        let heads = heads.lock().unwrap();
        heads
            .iter()
            .map(|h| told_in(h, name).unwrap_or_default())
            .collect::<Vec<_>>()
    };
    assert_eq!(told("x-svrngs-label"), [label.clone()]);
    assert_eq!(told("x-svrngs-pages"), [format!("dweb=none {NS}={page}")]);

    let seen = drive(
        engine,
        "links.mjs",
        json!({
            "site": site,
            "port": port,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "app",
            "home": format!("http://svrngs.localhost:{}/", ada.api_addr().port()),
        }),
        None,
    )
    .await;
    eprintln!("the link's run: {seen:#}");
    let pages = told("x-svrngs-pages");
    assert_eq!(
        pages[1],
        format!("{NS}={page}"),
        "bo's phone stands in the house alone: {pages:?}"
    );
    assert_eq!(
        seen["link"],
        format!("{page}#open={label}&at=/at/a/path?n=1"),
        "the app writes its link from the door's headers:\n{seen:#}"
    );
    let landed = &seen["landed"];
    assert!(
        at_origin(&landed["url"], &label)
            && landed["url"].as_str().unwrap().ends_with("/at/a/path?n=1")
            && landed["title"] == "the app at a path",
        "the link opens the app at its path on bo's phone:\n{seen:#}"
    );
    let computer = &seen["computer"];
    assert_eq!(
        (&computer["said"], &computer["shown"]),
        (&json!("This link opens in svrngs on your computer."), &seen["link"]),
        "a computer keeping no group is told where the link opens, and shown it to copy:\n{seen:#}"
    );
    let opened = &computer["opened"];
    assert!(
        at_origin(&opened["url"], &label)
            && opened["url"].as_str().unwrap().ends_with("/at/a/path?n=1")
            && opened["title"] == "the app at a path",
        "pasted on the computer's home page, the link opens the app at its path there:\n{seen:#}"
    );
    drop(ada);
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
                let answer = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\n\
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
    /// the lent page is not shown in the frame, and the page's own yes, on
    /// a dial of its own with the pass its origin keeps, is refused and
    /// keeps nothing. Asked of an app and a lend not lent, the group's page
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
    let (shelf, heads) = kept_page("<!doctype html><title>cy's shelf</title>").await;
    let lends = svrngs_node::lend::lends_dir(cy_dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body =
        json!({ "port": shelf.port(), "kind": "k", "limit": 3, "refused": [], "asks": WORDS });
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
    // The app's own yes, with what its origin keeps, at cy's door on the
    // pass holders' protocol: dialled straight, not through the relay,
    // where the worker holds that key.
    let kept = &seen["kept"];
    let (Some(pass), Ok(holder)) = (
        kept["pass"].as_str(),
        hex::decode(kept["holder"].as_str().unwrap_or_default()),
    ) else {
        panic!("the origin keeps the pass and its holder: {seen:#}");
    };
    let holder = SigningKey::from_bytes(&holder.try_into().expect("a key is 32 bytes"));
    let door = bridge_as(&holder, &cy, svrngs_core::alpn::Alpn::Lend).await;
    let at = door.local_addr();
    let asked = |head: String| async move {
        let mut sock = tokio::net::TcpStream::connect(at).await.unwrap();
        sock.write_all(head.as_bytes()).await.unwrap();
        let mut answer = String::new();
        tokio::io::AsyncReadExt::read_to_string(&mut sock, &mut answer)
            .await
            .unwrap();
        answer
    };
    let body = format!(r#"{{"app":"{page_label}","said":"yes"}}"#);
    let yes = asked(format!(
        "POST /choices/{shelf_label} HTTP/1.1\r\nHost: cy\r\nx-svrngs-pass: {pass}\r\n\
         content-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    ))
    .await;
    assert!(
        yes.starts_with("HTTP/1.1 403"),
        "a pass's yes is refused: {yes}"
    );
    let after = asked(format!(
        "GET /lend/{shelf_label}/probe HTTP/1.1\r\nHost: cy\r\nx-svrngs-pass: {pass}\r\nconnection: close\r\n\r\n"
    ))
    .await;
    assert!(
        after.to_lowercase().contains("x-svrngs-refused: stopped"),
        "nothing kept of a pass's yes: {after}"
    );
    drop((ada, cy, door));
}

in_both_engines!(
    /// The home page on a computer asks svrngs's question in the one
    /// wording the group's page asks it (lean-4 (c); sup-4's follow-up 2,
    /// `demo/runs/sup/2026-10-10/README.md`). Ada lends a page reading
    /// kind `k` and a shelf, `shelf` of kind `books`, that asks; her home
    /// page's `/ask?lend=<shelf>&app=<page>` heads its question as
    /// `question.js` words it: the shelf named with its kind, as the bare
    /// command names a lend. Planted, watched failing: the home page with
    /// words of its own (the kind dropped, as before). Asked of an app
    /// and a lend not lent, it says [`UNLENT`], as the group's page does
    /// (onair-economize (d); red at a20ed46: "... in your groups now.").
    the_home_page_asks_in_the_one_wording
);
async fn the_home_page_asks_in_the_one_wording(engine: Engine) {
    let relay = relay().await;
    let ada_dir = tempfile::tempdir().unwrap();
    let ada = node(ada_dir.path(), &relay, &signing(7)).await;
    let page = page_origin("<!doctype html><title>ada's page</title>").await;
    lend_reading(ada_dir.path(), "page", page.port(), &["books"]);
    let (shelf, _) = kept_page("<!doctype html><title>ada's shelf</title>").await;
    let body = json!({ "port": shelf.port(), "kind": "books", "limit": 3, "asks": "the page reads the shelf" });
    std::fs::write(
        svrngs_node::lend::lends_dir(ada_dir.path()).join("shelf.json"),
        body.to_string(),
    )
    .unwrap();
    let ask = format!(
        "http://svrngs.localhost:{}/ask?lend={}&app={}",
        ada.api_addr().port(),
        label(1, "shelf"),
        label(1, "page")
    );
    let seen = drive(engine, "home.mjs", json!({ "ask": ask }), None).await;
    eprintln!("the home page's run: {seen:#}");
    assert_eq!(
        seen["heading"],
        question(["ada", "page", "k"], ["ada", "shelf", "books"]),
        "{seen:#}"
    );
    let ask = format!(
        "http://svrngs.localhost:{}/ask?lend=x&app=y",
        ada.api_addr().port()
    );
    let args = json!({ "ask": ask, "heading": false, "read": "#home p:not(.said)" });
    let seen = drive(engine, "home.mjs", args, None).await;
    assert_eq!(seen["read"], UNLENT, "{seen:#}");
    drop(ada);
}

in_both_engines!(
    /// **The home page changes how many a lend lets in at once, in place**
    /// (bash-limit (b)). Ada lends `shelf` to one person at once; her home
    /// page lists it so, and she sets four, then everyone, then stops
    /// lending it, each a button on the page, each read back from what the
    /// door reads. Planted, watched failing: home.js without its list of
    /// what this computer lends.
    the_home_page_changes_a_lend_s_limit_in_place
);
async fn the_home_page_changes_a_lend_s_limit_in_place(engine: Engine) {
    let relay = relay().await;
    let ada_dir = tempfile::tempdir().unwrap();
    let ada = node(ada_dir.path(), &relay, &signing(7)).await;
    let shelf = page_origin("<!doctype html><title>ada's shelf</title>").await;
    let body = json!({ "port": shelf.port(), "kind": "books", "limit": 1 });
    let lends = svrngs_node::lend::lends_dir(ada_dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    std::fs::write(lends.join("shelf.json"), body.to_string()).unwrap();
    let press = |words: &str| json!([format!("#lent button:has-text('{words}')")]);
    let seen = home_page(
        engine,
        &ada,
        json!({
            "read": "#lent li",
            "all": "#lent li, #lent p",
            "do": [["#lent input", "4"], press("Let in this many"), press("Let everyone in"), press("Stop lending")],
        }),
    )
    .await;
    let first = |v: &Value| v[0].as_str().unwrap_or_default().to_string();
    assert!(
        first(&seen["all"]).starts_with("shelf: up to 1 person at once, 0 using it now."),
        "{seen:#}"
    );
    let after = &seen["after"];
    assert!(
        first(&after[0]).starts_with("shelf: up to 4 people at once"),
        "{seen:#}"
    );
    assert!(
        first(&after[1]).starts_with("shelf: everyone in your groups at once"),
        "{seen:#}"
    );
    assert_eq!(
        after[2],
        json!(["This computer lends nothing."]),
        "{seen:#}"
    );
    let name = svrngs_node::lend::LendName::parse("shelf").unwrap();
    assert!(svrngs_node::lend::read(ada_dir.path(), &name)
        .unwrap()
        .is_none());
    drop(ada);
}

in_both_engines!(
    /// **A removed person's phone is told so in a person's words** (U7
    /// (e); DESIGN §6). Ada removes bo through her computer's
    /// `/v1/removals`; the page holding bo's key opens the group's link,
    /// and ada's computer, which closes a stranger's dial, serves bo's only
    /// to refuse it as removed: the page says "You are no longer in house."
    /// Planted, watched failing: a removed dialer closed like a stranger
    /// (the page said "This browser could not open the page.").
    the_group_s_page_tells_a_removed_person
);
async fn the_group_s_page_tells_a_removed_person(engine: Engine) {
    let site = site();
    let dir = tempfile::tempdir().unwrap();
    let relay = relay().await;
    let bo = signing(7);
    let ada = node(dir.path(), &relay, &bo).await;
    let removed = removal(&ada, "bo").await;
    assert_eq!(removed, svrngs_node::removal::Outcome::Out);
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
    assert_eq!(seen["status"], "You are no longer in house.", "{seen:#}");
    drop(ada);
}

in_both_engines!(
    /// **A removed person's home page tells them so** (U7 (e)). Ada and
    /// bo's computer are in house; ada removes bo through `/v1/removals`,
    /// which tells bo's computer its cut; bo's home page then says bo is no
    /// longer in the group, and lists nothing of it. Planted, watched
    /// failing: the home page without the removed line (it listed the
    /// group's people).
    the_home_page_tells_a_removed_person
);
async fn the_home_page_tells_a_removed_person(engine: Engine) {
    let relay = relay().await;
    let (ada_dir, bo_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    // Bo's computer's key is the one ada's node admits as bo: two keys
    // admitted as `bo` by hand are two people since bash-devices (b), and
    // a removal of `bo` would be refused `shared` (it admitted `signing(9)`).
    let ada = node(ada_dir.path(), &relay, &signing(2)).await;
    let bo = joined(bo_dir.path(), &relay, &ada, 2, "bo").await;
    assert_eq!(
        removal(&ada, "bo").await,
        svrngs_node::removal::Outcome::Out
    );
    let told = tokio::time::timeout(Duration::from_secs(20), async {
        while !bo
            .rail()
            .register(NS)
            .unwrap()
            .removed
            .contains_key(&actor_of(&signing(2)))
        {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(told.is_ok(), "bo's computer is told its cut");
    let seen = home_page(engine, &bo, json!({ "read": "#home section p" })).await;
    assert_eq!(
        seen["read"],
        "You are no longer in this group. What you already have stays on this computer; nothing new reaches it.",
        "{seen:#}"
    );
    drop((ada, bo));
}

in_both_engines!(
    /// **A register that cannot be read is said as itself** (REVIEW_FINDINGS
    /// #123; DESIGN §1). Ada's copy of house's seed is unreadable, so
    /// `/v1/groups` answers `removed: null`; her home page says this
    /// computer could not read who is in the group, never what it lends.
    /// Planted, watched failing: home.js as at 09e1484, which read `null`
    /// as not removed and said "What this group lends cannot be read".
    the_home_page_says_an_unreadable_register
);
async fn the_home_page_says_an_unreadable_register(engine: Engine) {
    use std::os::unix::fs::PermissionsExt;
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(2)).await;
    let roster = ada.rail().group_dir(NS).unwrap().join("roster.json");
    std::fs::set_permissions(&roster, std::fs::Permissions::from_mode(0o000)).unwrap();
    let seen = home_page(engine, &ada, json!({ "read": "#home section p" })).await;
    std::fs::set_permissions(&roster, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        seen["read"], "This computer could not read who is in this group.",
        "{seen:#}"
    );
    drop(ada);
}

in_both_engines!(
    /// **The home page says its own address, as the command and install.sh
    /// say it** (start-address (b)): the same each time svrngs starts, since
    /// it is kept (`api::kept_address`). Planted, watched failing: home.js
    /// as at 82c7dd1, which said no address.
    the_home_page_says_its_address_as_the_command_does
);
async fn the_home_page_says_its_address_as_the_command_does(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(2)).await;
    let seen = home_page(engine, &ada, json!({ "read": "#home > p.address" })).await;
    assert_eq!(
        seen["read"],
        format!(
            "Your home page, the same address each time svrngs starts: {}",
            svrngs_node::home::address(ada.api_addr().port())
        ),
        "{seen:#}"
    );
    drop(ada);
}

in_both_engines!(
    /// **The home page tells two people of one name apart**
    /// (bash-same-name (a)). Ada lets two people into house as cy, a
    /// second apart, each by hand (bash-devices made them two people,
    /// `a1d0ba9`); her home page's people line names each person once,
    /// and no two alike, in a person's words — each cy by when they
    /// joined, as `svrngs remove --joined` takes it, to the instant.
    /// Planted: two people of one name, listed by home.js as "cy, cy"; a
    /// time with no offset (#126).
    the_home_page_tells_two_people_of_one_name_apart
);
async fn the_home_page_tells_two_people_of_one_name_apart(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    for me in [3, 4] {
        let fixture = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse("cy").unwrap(),
                key: actor_of(&signing(me)),
                invite: None,
            },
            on_behalf_of: None,
        };
        ada.rail().commit(NS, &fixture).unwrap().unwrap();
        tokio::time::sleep(Duration::from_millis(1_100)).await;
    }
    let seen = home_page(engine, &ada, json!({ "read": "#home section p" })).await;
    eprintln!("the home page's run: {seen:#}");
    let line = seen["read"].as_str().unwrap_or_default();
    let people: Vec<&str> = (line.strip_prefix("People: "))
        .unwrap_or_else(|| panic!("no people line: {seen:#}"))
        .split(", ")
        .collect();
    let alike = people
        .iter()
        .filter(|p| people.iter().filter(|q| q == p).count() > 1);
    assert_eq!(alike.count(), 0, "two people listed alike: {line}");
    assert_eq!(people.len(), 4, "ada, bo and each cy, once: {line}");
    assert_eq!(
        people.iter().filter(|p| p.starts_with("cy")).count(),
        2,
        "{line}"
    );
    // Each cy by when they joined, in the form `svrngs remove --joined`
    // takes (home.rs `WHEN`), read back to the instant each was let in
    // (#126): 2026-10-12 14:03:22 -0700.
    let admitted: BTreeSet<i64> = (ada.rail().register(NS).unwrap().members.values())
        .filter(|m| m.person.as_str() == "cy")
        .filter_map(|m| match m.joined {
            Joined::At(t) => Some(t),
            Joined::Started => None,
        })
        .collect();
    let read: BTreeSet<i64> = (people.iter().filter(|p| p.starts_with("cy")))
        .map(|cy| {
            let when = (cy
                .strip_prefix("cy (joined ")
                .and_then(|w| w.strip_suffix(')')))
            .unwrap_or_else(|| panic!("{cy} says no time: {line}"));
            chrono::DateTime::parse_from_str(when, svrngs_node::home::WHEN)
                .unwrap_or_else(|e| panic!("{cy}: not in `--joined`'s form ({e}): {line}"))
                .timestamp()
        })
        .collect();
    assert_eq!((read.len(), &read), (2, &admitted), "{line}");
    for kept in ["key", "member", "keeper", "ring", " act", "journal"] {
        assert!(!line.to_lowercase().contains(kept), "says {kept}: {line}");
    }
    drop(ada);
}

/// House's removals, held on ada's computer: cy and dee admitted, cy's
/// ask to remove dee signed on cy's own key and taken in, then ada's cut
/// of dee through `/v1/removals`.
async fn removals_held(ada: &Node) {
    for (person, me) in [("cy", 3), ("dee", 4)] {
        let fixture = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse(person).unwrap(),
                key: actor_of(&signing(me)),
                invite: None,
            },
            on_behalf_of: None,
        };
        ada.rail().commit(NS, &fixture).unwrap().unwrap();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let ask = RailAct::Remove {
        key: actor_of(&signing(4)),
        through: vec![],
    };
    let asked = svrngs_core::tests_support::signed_in(NS, &signing(3), now + 5, 0, ask);
    ada.rail().ingest_all(NS, &[asked]).unwrap();
    assert_eq!(
        removal(ada, "dee").await,
        svrngs_node::removal::Outcome::Out
    );
}

/// Each line of the removals a page lists, up to its time: the time is
/// the browser's own words for it.
fn removals_said(seen: &Value) -> Vec<String> {
    seen["removals"]
        .as_array()
        .unwrap_or_else(|| panic!("the page lists no removals: {seen:#}"))
        .iter()
        .map(|l| {
            l.as_str()
                .unwrap()
                .split(" on ")
                .next()
                .unwrap()
                .to_string()
        })
        .collect()
}

in_both_engines!(
    /// **The home page shows everyone each removal and each ask** (the
    /// operator, 2026-10-12; v1-keeper-office (b)). On ada's computer cy
    /// asks to remove dee and ada removes dee; ada's home page lists both,
    /// by name, with when, from the record alone. Planted, watched failing:
    /// the home page before (b), which listed none.
    the_home_page_shows_every_removal_and_ask
);
async fn the_home_page_shows_every_removal_and_ask(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    removals_held(&ada).await;
    let seen = home_page(engine, &ada, json!({ "all": "#home .removals li" })).await;
    eprintln!("the home page's run: {seen:#}");
    assert_eq!(
        removals_said(&json!({ "removals": seen["all"] })),
        ["cy asked to remove dee", "ada removed dee"],
        "{seen:#}"
    );
    drop(ada);
}

in_both_engines!(
    /// **The group's page shows everyone each removal and each ask**
    /// (v1-keeper-office (b)). The same removals on ada's computer; the
    /// page holding bo's key opens the group's link and lists both, as the
    /// home page does. Planted, watched failing: the page before (b).
    the_group_s_page_shows_every_removal_and_ask
);
async fn the_group_s_page_shows_every_removal_and_ask(engine: Engine) {
    let site = site();
    let dir = tempfile::tempdir().unwrap();
    let relay = relay().await;
    let bo = signing(7);
    let ada = node(dir.path(), &relay, &bo).await;
    removals_held(&ada).await;
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
    eprintln!("the group's page's run: {seen:#}");
    assert_eq!(seen["status"], "You’re in house, as bo", "{seen:#}");
    assert_eq!(
        removals_said(&seen),
        ["cy asked to remove dee", "ada removed dee"],
        "{seen:#}"
    );
    drop(ada);
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

in_both_engines!(
    /// The group's link on a computer (v1-onboarding (c); DESIGN §7): the
    /// page offers the one line that gets svrngs, composed from the
    /// link's `#` part, and a button that copies it; on a phone it offers
    /// none. No request either page makes, the worker's and the relay's
    /// included, carries the token, and the page host's answer to `/`
    /// previews the link as an invitation, naming no group. The link here
    /// lets nobody in, so each page settles on its refusal. Watched
    /// failing: the page before the offer, and the line sent in a request.
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
    // WebKit refuses a test's read of the clipboard (computer.mjs).
    if engine.as_str() == "chromium" {
        assert_eq!(computer["copied"], line, "{computer:#}");
    }
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

in_both_engines!(
    /// **A founder's one line** (start-group (d); DESIGN §7): the group's
    /// page opened with no link, on a computer that holds no group, offers
    /// the line with the page's address where the link goes, saying it
    /// gets svrngs and opens the home page, where a group is made; and a
    /// button that copies it. On a phone it offers none. Planted, watched
    /// failing: the page as a136f60 left it, "Open this page from your
    /// group’s link." alone.
    the_page_opened_with_no_link_offers_a_founder_s_line
);
async fn the_page_opened_with_no_link_offers_a_founder_s_line(engine: Engine) {
    let seen = drive(
        engine,
        "computer.mjs",
        json!({ "site": site(), "token": "" }),
        None,
    )
    .await;
    let origin = seen["origin"].as_str().unwrap();
    let line = format!("curl -fsSL {origin}install.sh | sh -s -- '{origin}'");
    let computer = &seen["computer"];
    eprintln!("offered on a computer: {computer:#}");
    assert_eq!(computer["offered"], true, "{computer:#}");
    assert_eq!(computer["line"], line, "{computer:#}");
    assert_eq!(computer["written"], line, "{computer:#}");
    let words = computer["words"].as_str().unwrap_or_default();
    assert!(words.contains("make your group"), "{computer:#}");
    assert_eq!(seen["phone"]["offered"], false, "{:#}", seen["phone"]);
}

in_both_engines!(
    /// The home page says where the group's folder is on this computer
    /// (v1-onboarding (d)), from `/v1/groups`' `folders`; this node keeps
    /// no seed, so the group is named as the page names a group whose name
    /// it cannot read. Watched failing: home.js before it said so, and
    /// saying "null’s folder".
    the_home_page_says_where_the_groups_folder_is
);
async fn the_home_page_says_where_the_groups_folder_is(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let seen = home_page(engine, &ada, json!({ "read": ".folder" })).await;
    assert_eq!(
        seen["read"],
        format!(
            "The group’s folder is on this computer at {}",
            folder.display()
        ),
        "{seen:#}"
    );
    drop(ada);
}

/// Ada's computer, `admitted` let in as bo, and three more of house who
/// do not say what they lend (bash-first-minute (c)): dee's computer never
/// answers, having none; eve's holds no group of ada's, so it has not
/// heard of ada's and closes it; cy's cannot read what it lends, and
/// answers so. The computers are returned, to be dropped with ada's.
async fn unlisted_held(
    relay: &Server,
    admitted: &SigningKey,
    dirs: &[tempfile::TempDir; 3],
) -> (Node, [Node; 2]) {
    let ada = node(dirs[0].path(), relay, admitted).await;
    for (person, me) in [("dee", 4), ("eve", 5)] {
        let fixture = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse(person).unwrap(),
                key: actor_of(&signing(me)),
                invite: None,
            },
            on_behalf_of: None,
        };
        ada.rail().commit(NS, &fixture).unwrap().unwrap();
    }
    let eve = computer(dirs[1].path(), relay, &signing(9), 5, "eve").await;
    let addrs = eve.endpoint().addr().ip_addrs().copied().collect();
    ada.transport().add_known_peer(eve.endpoint().id(), addrs);
    let cy = joined(dirs[2].path(), relay, &ada, 2, "cy").await;
    std::fs::write(
        svrngs_node::lend::lends_dir(dirs[2].path()),
        "not a directory",
    )
    .unwrap();
    (ada, [eve, cy])
}

in_both_engines!(
    /// **The home page says a refusal as itself** (bash-first-minute (c);
    /// Demo C's dry run, finding 6). On ada's computer, bo's and dee's
    /// computers never answer, eve's has not heard of ada's, and cy's
    /// answers that it cannot say: the home page names each by how, and
    /// says neither eve's nor cy's did not answer. Planted, watched failing:
    /// home.js before (c), which named bo and dee alone.
    the_home_page_says_a_refusal_apart_from_no_answer
);
async fn the_home_page_says_a_refusal_apart_from_no_answer(engine: Engine) {
    let relay = relay().await;
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let (ada, others) = unlisted_held(&relay, &signing(7), &dirs).await;
    let seen = home_page(
        engine,
        &ada,
        json!({ "all": "#home section > ul:not(.removals) li" }),
    )
    .await;
    eprintln!("the home page's run: {seen:#}");
    let mut said: Vec<String> = serde_json::from_value(seen["all"].clone()).unwrap();
    said.sort();
    assert_eq!(
        said,
        [
            "bo's computer did not answer, so what they lend is not listed.",
            "cy's computer answered without saying what they lend, so it is not listed.",
            "dee's computer did not answer, so what they lend is not listed.",
            "eve's computer has not heard of this computer yet, so what they lend is not listed yet.",
        ],
        "{seen:#}"
    );
    drop((ada, others));
}

in_both_engines!(
    /// **The group's page says a refusal as itself**, as the home page
    /// does: the page holding bo's key opens the group's link, and ada's
    /// computer answers for dee's, eve's and cy's. Planted, watched
    /// failing: app.js before (c), which named dee alone, and said nobody
    /// in house lends anything.
    the_group_s_page_says_a_refusal_apart_from_no_answer
);
async fn the_group_s_page_says_a_refusal_apart_from_no_answer(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let (ada, others) = unlisted_held(&relay, &bo, &dirs).await;
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
    eprintln!("the group's page's run: {seen:#}");
    assert_eq!(seen["status"], "You’re in house, as bo", "{seen:#}");
    assert_eq!(
        seen["answer"],
        [
            "dee’s computer did not answer, so what they lend is not listed.",
            "eve’s computer has not heard of your group’s computer yet, so what they lend is not listed yet.",
            "cy’s computer answered without saying what they lend, so it is not listed.",
        ]
        .join("\n"),
        "{seen:#}"
    );
    drop((ada, others));
}

/// A computer in no group: a node with no seed and nothing in its record.
async fn alone(dir: &Path, relay: &Server) -> Node {
    Node::start(
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
    .expect("a node starts")
}

in_both_engines!(
    /// **The home page says at its top what it is for** (start-home, from
    /// start-measure's ranked row 4: with no group, "This computer lends
    /// nothing." and an empty "What apps use" were all it said,
    /// `demo/runs/start/2026-10-13/README.md` §3 (c) 13): on ada's
    /// computer, what can be done on it, in DESIGN §1's words; on a
    /// computer in no group, how to make one (on the page since
    /// start-group) or join one. Planted, watched failing: index.html and
    /// home.js as c6c2212 left them.
    the_home_page_says_what_it_is_for
);
async fn the_home_page_says_what_it_is_for(engine: Engine) {
    let relay = relay().await;
    let dirs = [(); 2].map(|_| tempfile::tempdir().unwrap());
    let ada = node(dirs[0].path(), &relay, &signing(7)).await;
    let seen = home_page(engine, &ada, json!({ "read": "#intro p" })).await;
    let intro = seen["read"].as_str().unwrap_or_default();
    assert!(
        intro.starts_with("This page is yours, on this computer alone."),
        "{seen:#}"
    );
    let dee = alone(dirs[1].path(), &relay).await;
    let seen = home_page(engine, &dee, json!({ "read": "#home > p.none" })).await;
    assert_eq!(
        seen["read"],
        "This computer is in no group yet. Make one below, or join one by the line the page of the link you were sent gives.",
        "{seen:#}"
    );
    drop((ada, dee));
}

in_both_engines!(
    /// **What the page cannot do is named on it, with the one line to
    /// type** (start-home, from start-measure's ranked row 5: no state's
    /// words named a line, `demo/runs/start/2026-10-13/README.md` §3 (c)
    /// 12): lending something and removing someone, which the command does
    /// while svrngs runs; and leaving a group, said as not possible yet
    /// (`ralph/next/start/CHARTER.md`). Planted, watched failing: the page
    /// as 1b9575d left it.
    the_home_page_names_what_the_terminal_does
);
async fn the_home_page_names_what_the_terminal_does(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let seen = home_page(engine, &ada, json!({ "all": "#terminal li, #terminal p" })).await;
    let said = seen["all"].to_string();
    for line in [
        "svrngs share <the number after the colon in its address>",
        "svrngs remove <their name> --group <the group’s name>",
        "Leaving a group is not possible yet",
    ] {
        assert!(said.contains(line), "{line}: {seen:#}");
    }
    // Making a group is the page's own since start-group: the walk after
    // found `svrngs new` refused while svrngs runs from login
    // (demo/runs/start/2026-10-13-home/README.md §3).
    assert!(!said.contains("svrngs new"), "{seen:#}");
    drop(ada);
}

in_both_engines!(
    /// **For a program, the address of each thing lent, with a Copy
    /// button** (start-home, from start-measure's ranked row 6: Bo's tools
    /// reached a friend's model only at the address `/v1/lends` gave,
    /// `demo/runs/start/2026-10-13/README.md` §3 (a) 1-3, which the page
    /// never showed). Ada lends `shelf`; her page's button beside it says
    /// what it copies, and once pressed the page says the address it
    /// copied, which Chromium's clipboard then holds. Planted, watched
    /// failing: the page as f898bfd left it, with no such button.
    the_home_page_copies_a_lend_s_address_for_a_program
);
async fn the_home_page_copies_a_lend_s_address_for_a_program(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let shelf = page_origin("<!doctype html><title>ada's shelf</title>").await;
    lend(dir.path(), "shelf", shelf.port());
    let origin = format!(
        "http://{}.localhost:{}/",
        label(1, "shelf"),
        ada.api_addr().port()
    );
    let seen = home_page(
        engine,
        &ada,
        json!({
            "all": "#home section li",
            "do": [["#home section li button:has-text('Copy its address for a program')", null, true]],
        }),
    )
    .await;
    eprintln!("the home page's run: {seen:#}");
    let said = seen["after"][0].to_string();
    assert!(said.contains(&format!("Copied: {origin}")), "{seen:#}");
    if matches!(engine, Engine::Chromium) {
        assert_eq!(seen["clipboard"], origin, "{seen:#}");
    }
    drop(ada);
}

in_both_engines!(
    /// **Each thing lent opens by a link saying what it opens and whose**
    /// (start-home, from start-measure's ranked row 7: "Sam’s llama" read
    /// as a riddle, `demo/runs/start/2026-10-13/README.md` §3 (c)). Ada
    /// lends `shelf`, of kind `k`; her page's link to it reads "Open ada’s
    /// shelf (k)", the lend named as question.js names it. Planted,
    /// watched failing: the page as aac2340 left it, "ada’s shelf (k)".
    the_home_page_s_links_say_what_they_open
);
async fn the_home_page_s_links_say_what_they_open(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let shelf = page_origin("<!doctype html><title>ada's shelf</title>").await;
    lend(dir.path(), "shelf", shelf.port());
    let seen = home_page(engine, &ada, json!({ "read": "#home section li a" })).await;
    assert_eq!(seen["read"], "Open ada’s shelf (k)", "{seen:#}");
    drop(ada);
}

in_both_engines!(
    /// **Fewer things to puzzle over** (start-home, from start-measure's
    /// ranked row 8, `demo/runs/start/2026-10-13/README.md` §3 (c) 14 and
    /// the words to look up): "What apps use" shown only when an app uses
    /// something; the box's words, what its number is, shown beside it;
    /// and a thing lent opened in a tab of its own, beside the home page,
    /// never in its place. Ada lends `shelf`, nothing asking. Planted,
    /// watched failing: the page as 535e213 left it.
    the_home_page_shows_less_to_puzzle_over
);
async fn the_home_page_shows_less_to_puzzle_over(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let shelf = page_origin("<!doctype html><title>ada's shelf</title>").await;
    lend(dir.path(), "shelf", shelf.port());
    let seen = home_page(
        engine,
        &ada,
        json!({ "read": "#lent label", "all": "#home h2" }),
    )
    .await;
    eprintln!("the home page's run: {seen:#}");
    assert_eq!(seen["read"], "How many people at once: ", "{seen:#}");
    assert!(
        !seen["all"].to_string().contains("What apps use"),
        "{seen:#}"
    );
    let lent = seen["links"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["text"] == "Open ada’s shelf (k)");
    assert_eq!(
        lent.map(|l| &l["target"]),
        Some(&json!("_blank")),
        "{seen:#}"
    );
    drop(ada);
}

in_both_engines!(
    /// **A group is shown as it answers, not held for its slowest
    /// computer** (start-home, from start-measure's ranked row 9: 3.6 s of
    /// "Reading your groups…" with one computer stopped,
    /// `demo/runs/start/2026-10-13/README.md` §3 (c) 16). Ada knows bo's
    /// computer at an address that takes and never answers: her page
    /// shows the group's people at once, and that bo's computer did not
    /// answer only once it has not, more than a second later. Planted,
    /// watched failing: the page as 11190e3 left it, both shown together.
    the_home_page_shows_a_group_before_its_slowest_computer
);
async fn the_home_page_shows_a_group_before_its_slowest_computer(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let silent = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let bo =
        svrngs_node::iroh::PublicKey::from_bytes(signing(7).verifying_key().as_bytes()).unwrap();
    ada.transport()
        .add_known_peer(bo, vec![silent.local_addr().unwrap()]);
    let (people, unanswered) = (
        "#home section p",
        "#home section li:has-text('bo\\'s computer did not answer')",
    );
    let seen = home_page(engine, &ada, json!({ "timed": [people, unanswered] })).await;
    eprintln!("the home page's run: {seen:#}");
    let ms = |s: &str| seen["timed"][s].as_u64().expect("shown");
    assert!(ms(people) + 1000 < ms(unanswered), "{seen:#}");
    drop((ada, silent));
}

in_both_engines!(
    /// **A link to invite someone, made on the page as `svrngs link` makes
    /// it** (start-home, from start-measure's ranked row 10: the act most
    /// asked of a newcomer's page was a terminal command, refused while
    /// svrngs runs — `demo/runs/start/2026-10-13-home/data/before/`,
    /// `svrngs link Lab` exit 1). On ada's page each button says what it
    /// makes; pressed, the page shows the link, which names ada's computer
    /// as its issuer, and says who it is for and for how long; and copies
    /// it by a button saying so. Planted, watched failing: the page as
    /// caafae7 left it, with no such button; the copy, as 4f8aeb7 left it.
    the_home_page_makes_a_link_to_invite_someone
);
async fn the_home_page_makes_a_link_to_invite_someone(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let seen = home_page(
        engine,
        &ada,
        json!({
            "all": "#home section p.link",
            "do": [
                ["#home section button:has-text('Make a link to invite someone to')", null, true],
                ["#home section button:has-text('Make a link to add your phone or another computer of yours')", null, true],
                ["#home section p.link button:has-text('Copy the link')", null, true],
            ],
        }),
    )
    .await;
    eprintln!("the home page's run: {seen:#}");
    for (said, starts) in [
        (
            &seen["after"][0][0],
            "Send this link to the person you are inviting: svrngs://join/",
        ),
        (
            &seen["after"][1][0],
            "Open this link on your other computer or phone; it joins as you",
        ),
    ] {
        let said = said.as_str().unwrap_or_default();
        assert!(said.starts_with(starts), "{seen:#}");
        assert!(
            said.ends_with("It works for a day, while this computer is on. Copy the link"),
            "{seen:#}"
        );
        let at = said.find("svrngs://join/").expect("the link is shown");
        let link = svrngs_core::link::Link::parse(said[at..].split(' ').next().unwrap())
            .expect("the link shown is a link");
        assert_eq!(link.token.issuer, signing(1).verifying_key(), "{seen:#}");
    }
    // The link copied by a button that says so, the walk after finding a
    // 130-character link given no other way off the page
    // (demo/runs/start/2026-10-13-home/README.md §3).
    let copied = seen["after"][2][0].as_str().unwrap_or_default();
    assert!(copied.ends_with("Copy the link Copied."), "{seen:#}");
    if matches!(engine, Engine::Chromium) {
        let shown = seen["after"][1][0].as_str().unwrap_or_default();
        let link = seen["clipboard"].as_str().unwrap_or("none");
        assert!(
            link.starts_with("svrngs://join/") && shown.contains(link),
            "{seen:#}"
        );
    }
    drop(ada);
}

in_both_engines!(
    /// **The group's folder opened from the page** (start-home, from
    /// start-measure's ranked row 12: the folder was a path read as words,
    /// `demo/runs/start/2026-10-13/README.md` §3 (c) 14). Ada keeps house's
    /// folder; her page's "Open this folder" asks this computer's file
    /// browser to open it, and says so. The browser is a planted `open`
    /// and `xdg-open`, first on the test's `PATH`, writing what it was
    /// asked to open beside it, so no window opens. Planted, watched
    /// failing: the page as ab6ef1f left it, with no such button.
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
    // The planted bin first: every other program is found as before.
    let path = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(bin.clone()).chain(std::env::split_paths(&path));
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    keep_folder(&ada, &folder).await;
    let seen = home_page(
        engine,
        &ada,
        json!({
            "all": "#home section p",
            "do": [["#home section button:has-text('Open this folder')", null, true]],
        }),
    )
    .await;
    eprintln!("the home page's run: {seen:#}");
    assert!(
        seen["after"][0]
            .to_string()
            .contains("Open this folder Opened, in a window of its own."),
        "{seen:#}"
    );
    let asked = std::fs::read_to_string(&opened).unwrap_or_default();
    assert!(
        asked.lines().any(|l| l == folder.display().to_string()),
        "{asked:?}"
    );
    drop(ada);
}
