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
// The page built, and a relay on 127.0.0.1: shared with the command's
// joining test (crates/svrngs/tests/join_page.rs).
#[macro_use]
mod harness;

use std::path::Path;
use std::time::Duration;

use harness::{relay, relay_url, repo, site, Engine};
use iroh_relay::server::Server;
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
        &RelayConfig::from_parts(vec![relay_url(relay)], Some("none"))
            .unwrap()
            .on_loopback_only(),
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
async fn page_origin(html: String) -> std::net::SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let html = html.clone();
            tokio::spawn(async move {
                let mut head = [0u8; 8192];
                let _ = tokio::io::AsyncReadExt::read(&mut sock, &mut head).await;
                let answer = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n{html}",
                    html.len()
                );
                let _ = sock.write_all(answer.as_bytes()).await;
            });
        }
    });
    addr
}

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

in_both_engines!(
    /// The runtime (u4-6; DESIGN §3, "Joining", calls 3 and 4): ada's computer
    /// lends a page and a film, an H.264 mp4 from upstream's Range origin, and
    /// cy's, another computer holding bo's key, lends a file. The phone, holding
    /// bo's key, opens ada's page from its list: since u5-2 it opens at an
    /// origin of its own, `<label>.localhost`, apart from the page that
    /// holds bo's key, and that origin's worker serves it, its `<video>` and
    /// every byte range from ada's door, with the pass the page signed. The film plays and seeks three times,
    /// each playing again within five seconds (Demo A's seek bar); the browser
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
    let page = one_origin(page_origin(html.to_string()).await, film_origin).await;
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
    for seek in seeks {
        assert!(
            seek["ms"].as_u64().is_some_and(|ms| ms <= 5_000),
            "each seek must play again within five seconds:\n{seen:#}"
        );
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
    let page = one_origin(page_origin(html.to_string()).await, film_origin).await;
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
    let page = page_origin("<!doctype html><title>ada's page</title>".to_string()).await;
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
