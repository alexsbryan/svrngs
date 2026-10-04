// SPDX-License-Identifier: AGPL-3.0-or-later
// The identity tests: ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/iroh_identity_forward.rs@e1a43bcd2
//! The member door, through it (DESIGN §3; §8 U3's inventory; the u3-4
//! row). A lends a service on its loopback; B, standing in A's group, dials
//! A by key and asks through B's own loopback bridge, as a member's browser
//! will. Nodes in one process, no relay.
//!
//! The identity tests are upstream's `rewrite_head` tests
//! (iroh_identity_forward.rs:584-702), re-asserted through the door: each
//! sends upstream's head, raw, and reads what the service received. Every
//! byte B receives is scanned for A's secret ([`clean`]).

#[path = "../common/mod.rs"]
mod common;
mod origins;

use std::net::SocketAddr;
use std::time::Duration;

use common::loopback_sockets;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use svrngs_node::iroh::{Contact, RelayConfig};
use svrngs_node::lend::{self, LendName};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, declared, Ed25519Verifier, Person, RailAct, Roster, SigningKey};

const NS: &str = "house";

/// A's credential for its own service: Jellyfin 12's form, an inner space
/// and two quotes (iroh_identity_forward.rs:664-687).
const SECRET: &str = r#"MediaBrowser Token="a-key-1234""#;

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// No link is asked about here.
fn no_seed(_: &std::path::Path) -> Result<Option<String>, String> {
    Ok(None)
}

/// A lender and a member of its group, and the member's bridge to it.
struct Pair {
    a: Node,
    _b: Node,
    dir: tempfile::TempDir,
    _db: tempfile::TempDir,
    bridge: SocketAddr,
}

impl Pair {
    /// A founds the group as ada and admits bo, B's key.
    async fn new() -> Self {
        let (dir, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let start = |seed: u8, d: std::path::PathBuf| async move {
            Node::start(
                d,
                signing(seed),
                &RelayConfig::from_parts(Vec::new(), Some("none")).unwrap(),
                no_seed,
                "127.0.0.1:0".parse().unwrap(),
            )
            .await
            .expect("a node starts")
        };
        let a = start(1, dir.path().to_path_buf()).await;
        let b = start(2, db.path().to_path_buf()).await;
        let seed = Roster::new(
            [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
                .into_iter()
                .collect(),
        );
        let journal = a.rail().journal(NS).unwrap();
        journal.set_roster(&seed).unwrap();
        let author = svrngs_node::Author {
            signer: a.rail().signer(),
            roster: &seed,
            namespace: NS,
            verifier: &Ed25519Verifier,
            now: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };
        let fixture = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse("bo").unwrap(),
                key: actor_of(&signing(2)),
                invite: None,
            },
            on_behalf_of: None,
        };
        svrngs_node::commit(&*journal, &fixture, &author)
            .unwrap()
            .unwrap();
        let base = b
            .transport()
            .base_url(&Contact {
                key: a.endpoint().id(),
                relay_url: None,
                iroh_direct_addrs: loopback_sockets(a.endpoint()),
            })
            .await
            .expect("a contact with an address resolves");
        let bridge = base.trim_start_matches("http://").parse().unwrap();
        Pair {
            a,
            _b: b,
            dir,
            _db: db,
            bridge,
        }
    }

    /// A declares the lend `name` on `port`, refusing `refused`.
    fn lend(&self, name: &str, port: u16, refused: &[&str]) {
        let dir = lend::lends_dir(self.dir.path());
        std::fs::create_dir_all(&dir).unwrap();
        let body = serde_json::json!({ "port": port, "refused": refused });
        std::fs::write(dir.join(format!("{name}.json")), body.to_string()).unwrap();
    }

    /// A keeps `value` as the lend's secret, sent as `header`.
    fn secret(&self, name: &str, header: &str, value: &str) {
        let dir = lend::secret_dir(self.dir.path(), &LendName::parse(name).unwrap());
        declared::write_declared_in(&dir, header, value).unwrap();
    }

    /// One raw request through B's bridge; the whole reply, scanned.
    async fn raw(&self, request: &[u8]) -> Reply {
        let mut sock = tokio::net::TcpStream::connect(self.bridge).await.unwrap();
        sock.write_all(request).await.unwrap();
        let mut reply = Vec::new();
        tokio::time::timeout(Duration::from_secs(30), sock.read_to_end(&mut reply))
            .await
            .expect("the door answers")
            .unwrap();
        Reply::of(reply)
    }

    async fn get(&self, path: &str, headers: &str) -> Reply {
        self.raw(
            format!("GET {path} HTTP/1.1\r\nHost: local\r\n{headers}Connection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
    }

    async fn close(self) {
        self.a.endpoint().close().await;
    }
}

/// What B received: its status, and the bytes, every one of which has been
/// scanned for A's secret.
struct Reply {
    status: u16,
    text: String,
}

impl Reply {
    fn of(bytes: Vec<u8>) -> Self {
        clean(&bytes);
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let status = text
            .split_whitespace()
            .nth(1)
            .and_then(|c| c.parse().ok())
            .unwrap_or_else(|| panic!("no status line in the reply: {text:?}"));
        Reply { status, text }
    }
}

/// No byte a member receives holds the secret, or its token alone.
fn clean(bytes: &[u8]) {
    for needle in [SECRET.as_bytes(), b"a-key-1234", b"the-holders-key"] {
        assert!(
            !bytes.windows(needle.len()).any(|w| w == needle),
            "a member received the lender's secret: {:?}",
            String::from_utf8_lossy(bytes)
        );
    }
}

fn header_values<'a>(head: &'a str, name: &str) -> Vec<&'a str> {
    head.lines()
        .filter_map(|l| l.split_once(':'))
        .filter(|(n, _)| n.trim().eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim())
        .collect()
}

/// A, lending the head-keeping origin as `shelf` with [`SECRET`].
async fn lending_heads() -> (Pair, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
    let pair = Pair::new().await;
    let (origin, heads) = origins::head_origin().await;
    pair.lend("shelf", origin.port(), &["/Auth/Keys"]);
    pair.secret("shelf", "authorization", SECRET);
    (pair, heads)
}

fn last(heads: &std::sync::Mutex<Vec<String>>) -> String {
    heads
        .lock()
        .unwrap()
        .last()
        .cloned()
        .expect("the service was asked")
}

/// THE failing input (:584): a client that types the identity headers
/// itself. Exactly one person and one key reach the service, and they are
/// the door's — bo, and B's key — however the client cased them; the rest
/// of the prefix is gone. Planted: the door's strip skipped.
#[tokio::test]
async fn a_forged_identity_header_is_replaced_by_the_verified_one() {
    let (pair, heads) = lending_heads().await;
    let reply = pair
        .get(
            "/lend/shelf/whoami",
            "x-svrngs-person: forged\r\nX-SVRNGS-KEY: forged\r\nX-Svrngs-Anything: nope\r\n",
        )
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let head = last(&heads);
    assert!(head.starts_with("GET /whoami HTTP/1.1\r\n"), "{head}");
    assert_eq!(header_values(&head, "x-svrngs-person"), vec!["bo"]);
    assert_eq!(
        header_values(&head, "x-svrngs-key"),
        vec![actor_of(&signing(2))]
    );
    assert!(
        header_values(&head, "x-svrngs-anything").is_empty(),
        "{head}"
    );
    pair.close().await;
}

/// :601 and :689 — the request line and every header the door does not set
/// pass byte for byte: `Range` is a seek, `If-Range` its condition, and a
/// cookie is the client's business. Planted: `Range` dropped.
#[tokio::test]
async fn range_if_range_and_cookie_pass_untouched() {
    let (pair, heads) = lending_heads().await;
    let reply = pair
        .get(
            "/lend/shelf/library/title.bin?x=1",
            "Range: bytes=10-19\r\nIf-Range: \"etag-1\"\r\nCookie: c=1\r\n",
        )
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let head = last(&heads);
    assert!(
        head.starts_with("GET /library/title.bin?x=1 HTTP/1.1\r\n"),
        "{head}"
    );
    assert_eq!(header_values(&head, "range"), vec!["bytes=10-19"]);
    assert_eq!(header_values(&head, "if-range"), vec!["\"etag-1\""]);
    assert_eq!(header_values(&head, "cookie"), vec!["c=1"]);
    pair.close().await;
}

/// :628 and :664 — the lend's credential displaces the client's copy, and
/// reaches the service verbatim, its inner space and quotes kept: one on
/// the wire, and it is the lender's.
#[tokio::test]
async fn a_declared_credential_displaces_the_client_s_and_arrives_verbatim() {
    let (pair, heads) = lending_heads().await;
    let reply = pair
        .get(
            "/lend/shelf/Items",
            "Authorization: MediaBrowser Token=\"the-viewers\"\r\n",
        )
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(header_values(&last(&heads), "authorization"), vec![SECRET]);
    pair.close().await;
}

/// :650 — the three client spellings `rewrite_head` displaced. Upper case
/// reaches the door and is displaced; padding before the colon and a
/// control byte in the name are refused by the door's HTTP parser before
/// the door is asked (RFC 9112 §5.1; hyper answers 400), so neither reaches
/// the service at all. The client's value is on the wire in no case.
#[tokio::test]
async fn three_client_spellings_never_reach_the_service() {
    let (pair, heads) = lending_heads().await;
    let reply = pair
        .get("/lend/shelf/", "AUTHORIZATION: viewer-upper\r\n")
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(header_values(&last(&heads), "authorization"), vec![SECRET]);
    for spelling in [
        "  authorization  : viewer-pad\r\n",
        "Authoriz\u{7f}ation: viewer-ctl\r\n",
    ] {
        let reply = pair.get("/lend/shelf/", spelling).await;
        assert_eq!(reply.status, 400, "{spelling:?}: {}", reply.text);
    }
    let heads = heads.lock().unwrap().clone();
    assert_eq!(heads.len(), 1, "only the first request reached the service");
    assert!(!heads.iter().any(|h| h.contains("viewer-")), "{heads:?}");
    pair.close().await;
}

/// The secret never goes back: the service echoes the credential it
/// received in a response header, and the door drops that header. Planted:
/// the door's drop skipped (and [`clean`] fails the test).
#[tokio::test]
async fn a_secret_the_service_echoes_never_reaches_the_member() {
    let (pair, heads) = lending_heads().await;
    let reply = pair.get("/lend/shelf/Users", "").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(
        header_values(&last(&heads), "authorization"),
        vec![SECRET],
        "control: it was sent"
    );
    assert!(
        header_values(&reply.text, "x-seen-authorization").is_empty(),
        "{}",
        reply.text
    );
    pair.close().await;
}

/// two_daemons.rs:343-350, through the door: the same member, the same
/// picky service, refused before A declares its credential and served
/// after — with no restart, since the door reads the store per request.
#[tokio::test]
async fn a_picky_service_answers_401_without_the_secret_and_200_with_it() {
    let pair = Pair::new().await;
    let origin = origins::picky_origin("x-emby-token", "the-holders-key").await;
    pair.lend("picky", origin.port(), &[]);
    let ask = "X-Emby-Token: the-viewers-key\r\n";
    assert_eq!(pair.get("/lend/picky/Items", ask).await.status, 401);
    pair.secret("picky", "X-Emby-Token", "the-holders-key");
    assert_eq!(pair.get("/lend/picky/Items", ask).await.status, 200);
    pair.close().await;
}

/// The lend's refused paths, however spelled, and any path that can step
/// out of the lend, are refused by name; the service is asked none of
/// them. A lend not declared is not found.
#[tokio::test]
async fn a_refused_path_is_refused() {
    let (pair, heads) = lending_heads().await;
    for (path, status) in [
        ("/lend/shelf/Auth/Keys", 403),
        ("/lend/shelf/auth/keys/1", 403),
        ("/lend/shelf/%41uth/Keys", 403),
        ("/lend/shelf//Auth/Keys", 403),
        ("/lend/shelf/../x", 400),
        ("/lend/shelf/a%2fb", 400),
        ("/lend/shelf/%2e%2e/x", 400),
        ("/lend/nothing/x", 404),
        ("/lend/no..name/x", 404),
    ] {
        let reply = pair.get(path, "").await;
        assert_eq!(reply.status, status, "{path}: {}", reply.text);
    }
    assert!(
        heads.lock().unwrap().is_empty(),
        "the service was asked a refused path"
    );
    let reply = pair.get("/lend/shelf/Auth/Keyshelf", "").await;
    assert_eq!(
        reply.status, 200,
        "control: a path beside the refused one is lent"
    );
    pair.close().await;
}

const FILE_BYTES: u64 = 64 * 1024 * 1024;

/// Read a reply's body off `resp` chunk by chunk: its length and SHA-256,
/// each byte checked against [`origins::byte_at`] from `start`, and scanned.
async fn drain(mut resp: reqwest::Response, start: u64) -> (u64, [u8; 32]) {
    let (mut sha, mut at) = (Sha256::new(), start);
    while let Some(chunk) = resp.chunk().await.unwrap() {
        for (i, b) in chunk.iter().enumerate() {
            assert_eq!(
                *b,
                origins::byte_at(at + i as u64),
                "byte {} is wrong",
                at + i as u64
            );
        }
        clean(&chunk);
        sha.update(&chunk);
        at += chunk.len() as u64;
    }
    (at - start, sha.finalize().into())
}

fn sha_of(range: std::ops::Range<u64>) -> [u8; 32] {
    let bytes: Vec<u8> = range.map(origins::byte_at).collect();
    Sha256::digest(&bytes).into()
}

/// Upstream's Range origin, through the door, over a 64 MiB file: the whole
/// file 200 and sha-exact, a range 206 with its `Content-Range` and
/// sha-exact, an unsatisfiable range 416. Planted: `Range` dropped (the 206
/// is a 200).
#[tokio::test]
async fn a_range_is_served_206_and_416_through_the_door_sha_exact() {
    let pair = Pair::new().await;
    let file = pair.dir.path().join("title.bin");
    std::fs::write(
        &file,
        (0..FILE_BYTES).map(origins::byte_at).collect::<Vec<u8>>(),
    )
    .unwrap();
    let origin = origins::range_origin(file, FILE_BYTES).await;
    pair.lend("shelf", origin.port(), &[]);
    pair.secret("shelf", "authorization", SECRET);
    let url = format!("http://{}/lend/shelf/title.bin", pair.bridge);
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let scan = |r: &reqwest::Response| {
        for (n, v) in r.headers() {
            clean(n.as_str().as_bytes());
            clean(v.as_bytes());
        }
    };

    let whole = http.get(&url).send().await.unwrap();
    scan(&whole);
    assert_eq!(whole.status(), 200);
    assert_eq!(drain(whole, 0).await, (FILE_BYTES, sha_of(0..FILE_BYTES)));

    let (from, to) = (1_000, 9_000_000 - 1);
    let part = http
        .get(&url)
        .header("Range", format!("bytes={from}-{to}"))
        .send()
        .await
        .unwrap();
    scan(&part);
    assert_eq!(part.status(), 206);
    assert_eq!(
        part.headers()["content-range"],
        format!("bytes {from}-{to}/{FILE_BYTES}").as_str()
    );
    assert_eq!(
        drain(part, from).await,
        (to - from + 1, sha_of(from..to + 1))
    );

    let past = http
        .get(&url)
        .header("Range", format!("bytes={FILE_BYTES}-"))
        .send()
        .await
        .unwrap();
    scan(&past);
    assert_eq!(past.status(), 416);
    assert_eq!(
        past.headers()["content-range"],
        format!("bytes */{FILE_BYTES}").as_str()
    );
    pair.close().await;
}

/// A 64 MiB upload streams through the door: the service reads the body's
/// first byte while the member still holds its tail back, then the whole,
/// sha-exact. This is the test's memory bound — a door that holds a body
/// whole before sending it on never lets the first byte through. Planted:
/// the request body buffered whole.
#[tokio::test]
async fn an_upload_streams_through_the_door() {
    let pair = Pair::new().await;
    let (origin, first) = origins::upload_origin().await;
    pair.lend("sink", origin.port(), &[]);
    let body: Vec<u8> = (0..FILE_BYTES).map(origins::byte_at).collect();
    let (head, tail) = body.split_at(1024 * 1024);

    let mut sock = tokio::net::TcpStream::connect(pair.bridge).await.unwrap();
    sock.write_all(
        format!("POST /lend/sink/up HTTP/1.1\r\nHost: local\r\nContent-Length: {FILE_BYTES}\r\nConnection: close\r\n\r\n")
            .as_bytes(),
    )
    .await
    .unwrap();
    sock.write_all(head).await.unwrap();
    tokio::time::timeout(Duration::from_secs(20), first)
        .await
        .expect("the service saw no byte of the upload before its tail was sent: the door holds the body")
        .unwrap();
    sock.write_all(tail).await.unwrap();
    let mut reply = Vec::new();
    tokio::time::timeout(Duration::from_secs(60), sock.read_to_end(&mut reply))
        .await
        .expect("the service answers the upload")
        .unwrap();
    let reply = Reply::of(reply);
    assert_eq!(reply.status, 200, "{}", reply.text);
    let want = format!("{FILE_BYTES} {}", hex::encode(Sha256::digest(&body)));
    assert!(reply.text.ends_with(&want), "{}", reply.text);
    pair.close().await;
}
