// SPDX-License-Identifier: AGPL-3.0-or-later
// The identity tests: ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/iroh_identity_forward.rs@e1a43bcd2
//! The member door, through it (DESIGN §3; §8 U3's inventory; the u3-4
//! row). A lends a service on its loopback; B, standing in A's group, dials
//! A by key and asks through B's own loopback bridge, as a member's browser
//! will. Nodes in one process, no relay, each listening on 127.0.0.1 alone:
//! on a wildcard, iroh may carry B's requests on this computer's private
//! address, and the door guesses a viewer there is home and counts no one
//! (ralph/REVIEW_FINDINGS.md #33).
//!
//! The identity tests are upstream's `rewrite_head` tests
//! (iroh_identity_forward.rs:584-702), re-asserted through the door: each
//! sends upstream's head, raw, and reads what the service received. Every
//! byte B receives is scanned for A's secret ([`clean`]).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod choices;
#[path = "../common/mod.rs"]
mod common;
mod limit;
mod newcomer;
mod origins;
mod sign_in;

use std::net::SocketAddr;
use std::time::Duration;

use common::{bridge_as, loopback_sockets};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use svrngs_node::iroh::{Contact, RelayConfig};
use svrngs_node::lend::{self, LendName};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, declared, Person, RailAct, Roster, SigningKey};

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

/// A node of this binary: no relay, listening on 127.0.0.1 alone (#33).
async fn start(seed: u8, dir: &std::path::Path) -> Node {
    Node::start(
        dir.to_path_buf(),
        signing(seed),
        &RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: false,
        }
        .on_loopback_only(),
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// A lender and a member of its group, and the member's bridge to it.
struct Pair {
    a: Node,
    b: Node,
    dir: tempfile::TempDir,
    _db: tempfile::TempDir,
    bridge: SocketAddr,
}

impl Pair {
    /// A founds the group as ada and admits bo, B's key.
    async fn new() -> Self {
        let (dir, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let a = start(1, dir.path()).await;
        let b = start(2, db.path()).await;
        let seed = Roster::new(
            [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
                .into_iter()
                .collect(),
        );
        a.rail().set_seed(NS, &seed).unwrap();
        let fixture = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse("bo").unwrap(),
                key: actor_of(&signing(2)),
                invite: None,
            },
            on_behalf_of: None,
        };
        a.rail().commit(NS, &fixture).unwrap().unwrap();
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
            b,
            dir,
            _db: db,
            bridge,
        }
    }

    /// A declares the lend `name` on `port`, refusing `refused`.
    fn lend(&self, name: &str, port: u16, refused: &[&str]) {
        let dir = lend::lends_dir(self.dir.path());
        std::fs::create_dir_all(&dir).unwrap();
        let body = serde_json::json!({ "port": port, "kind": "k", "limit": 3, "refused": refused });
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

/// Two keys of one name, told apart (bash-person; the charter's "The
/// name"): a second computer admitted as `bo` asks the same lend as B, and
/// each request carries an `x-svrngs-user` of its own, so an app that makes
/// an account from it makes two. Red at bash-person's base: no
/// `x-svrngs-user`, and both told `x-svrngs-person: bo`, the one header an
/// app could key an account on (`demo/runs/c/dry-2026-10-12/data/name.json`).
#[tokio::test]
async fn two_keys_of_one_name_are_two_accounts_to_a_lend() {
    use svrngs_core::alpn::Alpn;
    let (pair, heads) = lending_heads().await;
    let (second, _dir) = beside(&pair, NS, 3, "bo").await;
    let ask = "GET /lend/shelf/whoami HTTP/1.1\r\nHost: local\r\nConnection: close\r\n\r\n";
    let mut users = Vec::new();
    for bridge in [
        pair.bridge,
        bridge_as(&signing(3), &pair.a, Alpn::Member)
            .await
            .local_addr(),
    ] {
        let reply = raw_to(bridge, ask.into()).await;
        assert_eq!(reply.status, 200, "{}", reply.text);
        let head = last(&heads);
        assert_eq!(header_values(&head, "x-svrngs-person"), vec!["bo"]);
        users.push(header_values(&head, "x-svrngs-user").join(","));
    }
    assert!(
        users.iter().all(|u| !u.is_empty()) && users[0] != users[1],
        "each key its own account: {users:?}"
    );
    second.endpoint().close().await;
    pair.close().await;
}

/// A lending as ada, and bo's two devices — keys 2 and 6, one row of the
/// seed, so one person to the fold before device links and after — each a
/// bridge dialling A on the members' protocol (bash-devices).
struct Bos {
    a: Node,
    dir: tempfile::TempDir,
    devices: [svrngs_node::iroh::HttpBridge; 2],
}

impl Bos {
    async fn new() -> Self {
        use svrngs_core::alpn::Alpn;
        let dir = tempfile::tempdir().unwrap();
        let a = start(1, dir.path()).await;
        let seed = Roster::new(
            [
                (Person::parse("ada").unwrap(), vec![actor_of(&signing(1))]),
                (
                    Person::parse("bo").unwrap(),
                    vec![actor_of(&signing(2)), actor_of(&signing(6))],
                ),
            ]
            .into_iter()
            .collect(),
        );
        a.rail().set_seed(NS, &seed).unwrap();
        let devices = [
            bridge_as(&signing(2), &a, Alpn::Member).await,
            bridge_as(&signing(6), &a, Alpn::Member).await,
        ];
        Bos { a, dir, devices }
    }

    /// A lends `name` as `body` says.
    fn lend(&self, name: &str, body: serde_json::Value) {
        let dir = lend::lends_dir(self.dir.path());
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{name}.json")), body.to_string()).unwrap();
    }

    /// Bo's device `i` asks `path` through A's door, `headers` added.
    async fn get(&self, i: usize, path: &str, headers: &str) -> Reply {
        let ask =
            format!("GET {path} HTTP/1.1\r\nHost: local\r\n{headers}Connection: close\r\n\r\n");
        raw_to(self.devices[i].local_addr(), ask).await
    }
}

/// **One person's two devices are one account to a lend** (bash-devices
/// (c)): bo asks from each, and the service is told one `x-svrngs-key`,
/// bo's first, and one `x-svrngs-user` both times, so an app keyed on
/// either gives bo one account; and each device's own key in
/// `x-svrngs-device`, so an app tells bo's devices apart (bash-two-at-once
/// (b)). Red at bash-devices' base: each device told its own key and its
/// own account name (door.rs, `PersonKey::resolved` is the key itself);
/// red at bash-two-at-once's: no device told.
#[tokio::test]
async fn one_persons_two_devices_are_one_account_to_a_lend() {
    let bos = Bos::new().await;
    let (origin, heads) = origins::head_origin().await;
    bos.lend(
        "shelf",
        serde_json::json!({ "port": origin.port(), "kind": "k" }),
    );
    let (mut told, mut devices) = (Vec::new(), Vec::new());
    for i in 0..2 {
        let reply = bos.get(i, "/lend/shelf/whoami", "").await;
        assert_eq!(reply.status, 200, "{}", reply.text);
        let head = last(&heads);
        told.push((
            header_values(&head, "x-svrngs-key").join(","),
            header_values(&head, "x-svrngs-user").join(","),
        ));
        devices.push(header_values(&head, "x-svrngs-device").join(","));
    }
    assert_eq!(told[0].0, actor_of(&signing(2)), "bo's first key");
    assert_eq!(told[0], told[1], "(key, account) from each device");
    let each = [actor_of(&signing(2)), actor_of(&signing(6))];
    assert_eq!(devices, each, "each device by its own key");
    bos.a.endpoint().close().await;
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

/// svrngs's word told from the lend's (mnp-1; `demo/runs/mn/2026-10-08/`
/// README §9, 2): each answer the door makes in a lend's place carries
/// `x-svrngs-refused`, naming which — a lend not declared, a path the lend
/// refuses, a service not answering — and a 403 the lend's own service
/// answers carries none, the one it forged dropped, its body its own; nor
/// does an answer let through. Planted: the header off.
#[tokio::test]
async fn svrngs_s_refusals_are_marked_its_own_and_a_lend_s_are_not() {
    let (pair, _heads) = lending_heads().await;
    pair.lend("strict", origins::refusing_origin().await.port(), &[]);
    let gone = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    pair.lend("gone", gone.local_addr().unwrap().port(), &[]);
    drop(gone);
    for (path, status, refused) in [
        ("/lend/nothing/x", 404, Some("unlent")),
        ("/lend/shelf/Auth/Keys", 403, Some("path")),
        ("/lend/shelf/../x", 400, Some("path")),
        ("/lend/gone/x", 502, Some("away")),
        ("/lend/strict/x", 403, None),
        ("/lend/shelf/x", 200, None),
    ] {
        let reply = pair.get(path, "").await;
        assert_eq!(reply.status, status, "{path}: {}", reply.text);
        let (head, body) = reply.text.split_once("\r\n\r\n").unwrap();
        assert_eq!(
            header_values(head, "x-svrngs-refused").first().copied(),
            refused,
            "{path}: {}",
            reply.text
        );
        if let Some(kind) = refused {
            assert!(
                body.contains(&format!(r#""kind":"{kind}""#)),
                "{path}: {body}"
            );
        }
    }
    let strict = pair.get("/lend/strict/x", "").await;
    assert!(strict.text.contains("the app's own no"), "{}", strict.text);
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

/// An origin that answers a request's head and first chunk, then holds the
/// rest of its answer until `release` fires.
async fn held_origin() -> (SocketAddr, tokio::sync::oneshot::Sender<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (release, released) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut head = Vec::new();
        while !head.ends_with(b"\r\n\r\n") {
            let mut byte = [0u8; 1];
            sock.read_exact(&mut byte).await.unwrap();
            head.push(byte[0]);
        }
        sock.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n")
            .await
            .unwrap();
        let _ = released.await;
        sock.write_all(b"0\r\n\r\n").await.unwrap();
    });
    (addr, release)
}

/// What A lends, as B asks it now: `(name, kind, limit, using)`.
async fn lends_of(pair: &Pair) -> Vec<(String, String, u64, u64)> {
    let reply = pair.get("/lends", "").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let body = reply.text.split("\r\n\r\n").nth(1).unwrap();
    let doc: serde_json::Value = serde_json::from_str(body).unwrap();
    doc["lends"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["name"].as_str().unwrap().to_string(),
                l["kind"].as_str().unwrap().to_string(),
                l["limit"].as_u64().unwrap(),
                l["using"].as_u64().unwrap(),
            )
        })
        .collect()
}

/// A member asks what A lends, and how many use each now (u3-5): read
/// from the declarations at each ask, so a lend withdrawn is gone at the
/// next; and a person whose answer is still streaming is using it, and is
/// still counted once it ends, until the window passes (u3-6;
/// `door::limit::WINDOW`, crossed in `limit.rs`). The secret is in no
/// answer ([`clean`]). Planted: the
/// declarations read once, when the door starts (the withdrawn lend still
/// listed); the use ended when the service answers rather than when the
/// answer ends (none using while it streams).
#[tokio::test]
async fn a_member_asks_what_is_lent_and_how_many_use_it_now() {
    let (pair, _heads) = lending_heads().await;
    let (origin, release) = held_origin().await;
    pair.lend("slow", origin.port(), &[]);
    assert_eq!(
        lends_of(&pair).await,
        vec![
            ("shelf".to_string(), "k".to_string(), 3, 0),
            ("slow".to_string(), "k".to_string(), 3, 0),
        ]
    );

    let mut sock = tokio::net::TcpStream::connect(pair.bridge).await.unwrap();
    sock.write_all(b"GET /lend/slow/x HTTP/1.1\r\nHost: local\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut got = Vec::new();
    while !got.windows(5).any(|w| w == b"hello") {
        let mut buf = [0u8; 256];
        let n = tokio::time::timeout(Duration::from_secs(20), sock.read(&mut buf))
            .await
            .expect("the first chunk comes through")
            .unwrap();
        assert!(n > 0, "the answer ended early: {got:?}");
        got.extend_from_slice(&buf[..n]);
    }
    let slow = |l: &Vec<(String, String, u64, u64)>| l.iter().find(|x| x.0 == "slow").unwrap().3;
    assert_eq!(
        slow(&lends_of(&pair).await),
        1,
        "bo is using it while it streams"
    );

    release.send(()).unwrap();
    sock.read_to_end(&mut got).await.unwrap();
    assert_eq!(
        slow(&lends_of(&pair).await),
        1,
        "the answer ended, and bo is counted until the window passes"
    );

    lend::withdraw(pair.dir.path(), &LendName::parse("slow").unwrap()).unwrap();
    assert_eq!(
        lends_of(&pair).await,
        vec![("shelf".to_string(), "k".to_string(), 3, 0)],
        "a lend withdrawn is gone at the next ask"
    );
    pair.close().await;
}

/// B's own computer asks what the people of its group lend (u3-5;
/// `Node::lends`): ada's shelf, with the address on B's loopback that
/// reaches it, its origin there (u5-2) — through which bo reaches the service as bo — and cy, whose
/// computer is nowhere, named as not answering rather than lending nothing.
/// Asked again once ada withdraws the shelf, it is gone.
#[tokio::test]
async fn a_member_s_computer_lists_what_is_lent_with_where_to_reach_it() {
    let (pair, heads) = lending_heads().await;
    let ghost = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("cy").unwrap(),
            key: actor_of(&signing(9)),
            invite: None,
        },
        on_behalf_of: None,
    };
    pair.a.rail().commit(NS, &ghost).unwrap().unwrap();
    let seed = Roster::new(
        [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    pair.b.rail().set_seed(NS, &seed).unwrap();
    let a_addrs = loopback_sockets(pair.a.endpoint());
    let mut addr = svrngs_node::iroh::EndpointAddr::new(pair.a.endpoint().id());
    for a in &a_addrs {
        addr = addr.with_ip_addr(*a);
    }
    let synced = pair.b.sync_with(NS, &addr).await;
    assert!(synced.stop.is_none(), "{synced:?}");

    let lends = pair.b.lends().await;
    assert_eq!(
        lends.unlisted.unanswered,
        vec![Person::parse("cy").unwrap()]
    );
    assert_eq!(lends.lent.len(), 1, "{lends:?}");
    let shelf = &lends.lent[0];
    assert_eq!(shelf.person, Person::parse("ada").unwrap());
    assert_eq!(shelf.offered.name.as_str(), "shelf");
    assert_eq!(shelf.offered.kind.as_str(), "k");
    assert_eq!(shelf.offered.using, 0);
    // Its origin on B's loopback (u5-2), which B's record API serves.
    let label = svrngs_core::pass::Label::of(pair.a.endpoint().id().as_bytes(), "shelf");
    let host = format!("{label}.localhost");
    assert_eq!(
        shelf.address,
        format!("http://{host}:{}/", pair.b.api_addr().port())
    );

    let http = reqwest::Client::builder()
        .no_proxy()
        .resolve(&host, pair.b.api_addr())
        .build()
        .unwrap();
    let reply = http
        .get(format!("{}whoami", shelf.address))
        .send()
        .await
        .unwrap();
    assert_eq!(reply.status(), 200);
    clean(reply.text().await.unwrap().as_bytes());
    assert_eq!(header_values(&last(&heads), "x-svrngs-person"), vec!["bo"]);

    lend::withdraw(pair.dir.path(), &LendName::parse("shelf").unwrap()).unwrap();
    assert!(
        pair.b.lends().await.lent.is_empty(),
        "a lend withdrawn is gone at the next ask"
    );
    pair.close().await;
}

/// A, its computer lending `name` on `port`, as [`Pair::lend`] declares one.
fn lends_on(dir: &std::path::Path, name: &str, port: u16) {
    let lends = lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    let body = serde_json::json!({ "port": port, "kind": "k", "limit": 3, "refused": [] });
    std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// `node` stands in `ns`, founded by ada, and A knows where it listens.
async fn beside(pair: &Pair, ns: &str, seed: u8, person: &str) -> (Node, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let node = start(seed, dir.path()).await;
    let roster = Roster::new(
        [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    node.rail().set_seed(ns, &roster).unwrap();
    if ns != NS {
        pair.a.rail().set_seed(ns, &roster).unwrap();
    }
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse(person).unwrap(),
            key: actor_of(&signing(seed)),
            invite: None,
        },
        on_behalf_of: None,
    };
    pair.a.rail().commit(ns, &admit).unwrap().unwrap();
    pair.a
        .transport()
        .add_known_peer(node.endpoint().id(), loopback_sockets(node.endpoint()));
    (node, dir)
}

/// A member asks the computer it dials what the people of its groups lend
/// (u4-5; DESIGN §3, call 4): A answers its own shelf as ada's, with A's
/// key, and cy's pad, asked of cy's computer now, with cy's key — so the
/// asker dials each lender itself — and dan, whose computer is nowhere, as
/// not answering. Eve stands with ada in another group bo is not in, and
/// what eve lends is not bo's to see. Each lend carries the relay its
/// group names for its lender (relay-in-record), `null` for none. Planted:
/// A's own lends left out; the groups the asker does not stand in asked
/// too; no relay given.
#[tokio::test]
async fn a_member_asks_what_the_group_s_people_lend_each_with_its_lender_s_key() {
    let (pair, _heads) = lending_heads().await;
    let (cy, cy_dir) = beside(&pair, NS, 3, "cy").await;
    lends_on(cy_dir.path(), "pad", 1);
    let (eve, eve_dir) = beside(&pair, "work", 4, "eve").await;
    lends_on(eve_dir.path(), "desk", 1);
    let ghost = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("dan").unwrap(),
            key: actor_of(&signing(9)),
            invite: None,
        },
        on_behalf_of: None,
    };
    pair.a.rail().commit(NS, &ghost).unwrap().unwrap();
    // Ada's relay in the record, with the root's dot (relay-in-record).
    let at = svrngs_core::RelayAt {
        key: actor_of(&signing(1)),
        url: svrngs_core::relay::parse("https://relay.example./"),
    };
    pair.a.rail().commit(NS, &at).unwrap().unwrap();

    let reply = pair.get(svrngs_node::node::PEOPLE_LENDS_ROUTE, "").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let body = reply.text.split("\r\n\r\n").nth(1).unwrap();
    let doc: serde_json::Value = serde_json::from_str(body).unwrap();
    let lent: Vec<(&str, &str, &str)> = doc["lent"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["person"].as_str().unwrap(),
                l["key"].as_str().unwrap(),
                l["name"].as_str().unwrap(),
            )
        })
        .collect();
    let (ada, cy_key) = (actor_of(&signing(1)), actor_of(&signing(3)));
    assert_eq!(
        lent,
        vec![
            ("ada", ada.as_str(), "shelf"),
            ("cy", cy_key.as_str(), "pad")
        ],
        "{doc:#}"
    );
    assert_eq!(doc["unanswered"], serde_json::json!(["dan"]), "{doc:#}");
    // Each lend with the relay its group names for its lender, dotless;
    // `null`, said, for cy's, whose computer names none.
    let relays: Vec<&serde_json::Value> = doc["lent"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| &l["relay"])
        .collect();
    let want = [
        serde_json::json!("https://relay.example/"),
        serde_json::Value::Null,
    ];
    assert_eq!(relays, want.iter().collect::<Vec<_>>(), "{doc:#}");
    for node in [cy, eve] {
        node.endpoint().close().await;
    }
    pair.close().await;
}

/// A computer whose declarations cannot be read says so to a member who
/// asks what it lends, on either route (`GET /lends`, u3-5; the people's
/// lends, u4-5), with one refusal: a 500 in words, never an empty list.
/// Planted: the unreadable declarations answered as lending nothing.
#[tokio::test]
async fn declarations_that_cannot_be_read_are_refused_on_both_routes_never_lent_as_none() {
    let pair = Pair::new().await;
    std::fs::write(lend::lends_dir(pair.dir.path()), "not a directory").unwrap();
    for route in ["/lends", svrngs_node::node::PEOPLE_LENDS_ROUTE] {
        let reply = pair.get(route, "").await;
        assert_eq!(reply.status, 500, "{route}: {}", reply.text);
        assert!(
            reply
                .text
                .contains("what this computer lends cannot be read on it"),
            "{route}: {}",
            reply.text
        );
    }
    pair.close().await;
}

/// A WebSocket's upgrade is carried both ways (u5-2; DESIGN §3, "What an
/// app cannot yet ask", a live socket): B asks A's lend to switch to
/// `websocket`, the service answers 101, and the bytes B writes after come
/// back echoed through the door, the person set on the head as on any
/// request. Planted: the door stripping `Upgrade` as hop-by-hop
/// (door.rs:128 before u5-2), which the service answers 426.
#[tokio::test]
async fn a_websocket_upgrade_is_carried_both_ways() {
    let pair = Pair::new().await;
    let (origin, heads) = origins::upgrade_origin().await;
    pair.lend("live", origin.port(), &[]);
    let mut sock = tokio::net::TcpStream::connect(pair.bridge).await.unwrap();
    sock.write_all(
        b"GET /lend/live/socket HTTP/1.1\r\nHost: local\r\nConnection: Upgrade\r\n\
          Upgrade: websocket\r\nSec-WebSocket-Version: 13\r\n\r\n",
    )
    .await
    .unwrap();
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        let n = tokio::time::timeout(Duration::from_secs(20), sock.read(&mut byte))
            .await
            .expect("the door answers the upgrade")
            .unwrap();
        assert!(
            n == 1,
            "the door closed before its answer ended: {:?}",
            String::from_utf8_lossy(&head)
        );
        head.push(byte[0]);
    }
    let head = String::from_utf8_lossy(&head).into_owned();
    assert!(
        head.starts_with("HTTP/1.1 101"),
        "the upgrade must be answered 101: {head}"
    );
    assert_eq!(header_values(&head, "upgrade"), vec!["websocket"], "{head}");
    for word in ["ping", "and again"] {
        sock.write_all(word.as_bytes()).await.unwrap();
        let mut echo = vec![0u8; word.len()];
        tokio::time::timeout(Duration::from_secs(20), sock.read_exact(&mut echo))
            .await
            .expect("the service's echo comes back")
            .unwrap();
        assert_eq!(echo, word.as_bytes());
    }
    let asked = last(&heads);
    assert_eq!(
        header_values(&asked, "x-svrngs-person"),
        vec!["bo"],
        "{asked}"
    );
    pair.close().await;
}

/// One raw request to `addr`; the whole reply, scanned.
async fn raw_to(addr: SocketAddr, request: String) -> Reply {
    let mut sock = tokio::net::TcpStream::connect(addr).await.unwrap();
    sock.write_all(request.as_bytes()).await.unwrap();
    let mut reply = Vec::new();
    tokio::time::timeout(Duration::from_secs(30), sock.read_to_end(&mut reply))
        .await
        .expect("the door answers")
        .unwrap();
    Reply::of(reply)
}

#[allow(clippy::disallowed_methods)]
fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// A pass's holder (u5-2; DESIGN §3, "Lent apps", call 6): a fresh key
/// standing nowhere dials A on the pass holders' protocol. With bo's pass
/// for `shelf`, named by its label, it is let in as bo — the service is
/// told bo and bo's key, never the holder's, and never the pass — and its
/// secret is injected as for any member. The same pass at `other`, by label
/// or by name, opens nothing; nor does no pass, a lapsed one, one signed by
/// a key standing nowhere, or one held by another key. Planted: the door
/// taking a pass for any lend (`PassFault::NotNamed` ignored).
#[tokio::test]
async fn a_pass_opens_only_the_lend_it_names_to_the_key_holding_it() {
    use svrngs_core::alpn::Alpn;
    use svrngs_core::pass::{Label, Pass};

    let (pair, heads) = lending_heads().await;
    let (other, other_heads) = origins::head_origin().await;
    pair.lend("other", other.port(), &[]);
    let a = *pair.a.endpoint().id().as_bytes();
    let (shelf, elsewhere) = (Label::of(&a, "shelf"), Label::of(&a, "other"));
    let holder = signing(9);
    let bridge = bridge_as(&holder, &pair.a, Alpn::Lend).await;
    let ask = |path: String, pass: Option<&Pass>| {
        let pass = pass.map_or(String::new(), |p| {
            format!("x-svrngs-pass: {}\r\n", p.encode())
        });
        raw_to(
            bridge.local_addr(),
            format!("GET {path} HTTP/1.1\r\nHost: local\r\n{pass}Connection: close\r\n\r\n"),
        )
    };
    let until = unix_now() + 3600;
    let bos = Pass::sign(
        &signing(2),
        &actor_of(&holder),
        (shelf.clone(), vec![]),
        until,
    );

    let opened = ask(format!("/lend/{shelf}/whoami"), Some(&bos)).await;
    assert_eq!(opened.status, 200, "{}", opened.text);
    let head = last(&heads);
    assert!(head.starts_with("GET /whoami HTTP/1.1\r\n"), "{head}");
    assert_eq!(
        header_values(&head, "x-svrngs-person"),
        vec!["bo"],
        "{head}"
    );
    assert_eq!(
        header_values(&head, "x-svrngs-key"),
        vec![actor_of(&signing(2))],
        "{head}"
    );
    assert!(header_values(&head, "x-svrngs-pass").is_empty(), "{head}");
    assert_eq!(
        header_values(&head, "authorization"),
        vec![SECRET],
        "{head}"
    );

    let lapsed = Pass::sign(
        &signing(2),
        &actor_of(&holder),
        (shelf.clone(), vec![]),
        unix_now() - 1,
    );
    let nobodys = Pass::sign(
        &signing(5),
        &actor_of(&holder),
        (shelf.clone(), vec![]),
        until,
    );
    let anothers = Pass::sign(
        &signing(2),
        &actor_of(&signing(8)),
        (shelf.clone(), vec![]),
        until,
    );
    for (why, path, pass) in [
        (
            "another lend, by its label",
            format!("/lend/{elsewhere}/x"),
            Some(&bos),
        ),
        (
            "another lend, by its name",
            "/lend/other/x".to_string(),
            Some(&bos),
        ),
        ("no pass", format!("/lend/{shelf}/x"), None),
        ("a lapsed pass", format!("/lend/{shelf}/x"), Some(&lapsed)),
        (
            "a pass signed by a key standing nowhere",
            format!("/lend/{shelf}/x"),
            Some(&nobodys),
        ),
        (
            "a pass held by another key",
            format!("/lend/{shelf}/x"),
            Some(&anothers),
        ),
    ] {
        let refused = ask(path, pass).await;
        assert_eq!(refused.status, 403, "{why}: {}", refused.text);
        assert!(
            refused.text.contains(r#""kind":"pass""#),
            "{why}: {}",
            refused.text
        );
    }
    assert_eq!(
        heads.lock().unwrap().len(),
        1,
        "only the opened request reached shelf"
    );
    assert!(
        other_heads.lock().unwrap().is_empty(),
        "nothing reached other"
    );
    pair.close().await;
}

/// The app a pass's holder asks for (su-1, `demo/runs/su/2026-10-09/`
/// §5; sup-1): the service is told `x-svrngs-app`, the pass's `app` — the
/// lend the person's page opened — whichever lend is asked, and the
/// client's own is stripped; a key of the group's, asking with no pass, is
/// told no app, its own stripped too. A pass from before the app had a
/// field of its own, `lends` and no `app`, is refused `pass`, so the
/// lend's worker asks the page for another. A key of the group's
/// presenting its own pass, as a computer's origin signs one for its page
/// (sup-1), is told that pass's app; its pass naming another lend, or
/// signed by another key, is refused `pass`. Planted: the door naming the
/// pass's first read, the lend asked here; the client's header let
/// through; a member's pass read whoever signed it.
#[tokio::test]
async fn a_pass_s_app_is_told_and_a_client_s_own_stripped() {
    use svrngs_core::alpn::Alpn;
    use svrngs_core::pass::{Label, Pass};

    let (pair, heads) = lending_heads().await;
    let a = *pair.a.endpoint().id().as_bytes();
    let (shelf, app) = (Label::of(&a, "shelf"), Label::of(&a, "picks"));
    let holder = signing(9);
    let bridge = bridge_as(&holder, &pair.a, Alpn::Lend).await;
    let pass = Pass::sign(
        &signing(2),
        &actor_of(&holder),
        (app.clone(), vec![shelf.clone()]),
        unix_now() + 3600,
    );
    let opened = raw_to(
        bridge.local_addr(),
        format!(
            "GET /lend/{shelf}/whoami HTTP/1.1\r\nHost: local\r\nx-svrngs-pass: {}\r\nX-Svrngs-App: forged\r\nConnection: close\r\n\r\n",
            pass.encode()
        ),
    )
    .await;
    assert_eq!(opened.status, 200, "{}", opened.text);
    let head = last(&heads);
    assert_eq!(
        header_values(&head, "x-svrngs-app"),
        vec![app.as_str()],
        "{head}"
    );
    assert_eq!(
        header_values(&head, "x-svrngs-person"),
        vec!["bo"],
        "{head}"
    );

    let mut before = serde_json::to_value(&pass).unwrap();
    let fields = before.as_object_mut().unwrap();
    fields.remove("app");
    let lends = fields.remove("reads").unwrap();
    fields.insert("lends".into(), serde_json::json!([app.as_str(), lends[0]]));
    use base64::Engine as _;
    let before = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&before).unwrap());
    let refused = raw_to(
        bridge.local_addr(),
        format!("GET /lend/{shelf}/whoami HTTP/1.1\r\nHost: local\r\nx-svrngs-pass: {before}\r\nConnection: close\r\n\r\n"),
    )
    .await;
    assert_eq!(refused.status, 403, "{}", refused.text);
    assert!(
        refused.text.contains(r#""kind":"pass""#),
        "{}",
        refused.text
    );

    let member = pair
        .get("/lend/shelf/whoami", "X-Svrngs-App: forged\r\n")
        .await;
    assert_eq!(member.status, 200, "{}", member.text);
    let head = last(&heads);
    assert!(header_values(&head, "x-svrngs-app").is_empty(), "{head}");

    let bo = actor_of(&signing(2));
    let own = |by: u8, reads: Vec<Label>| {
        let pass = Pass::sign(&signing(by), &bo, (app.clone(), reads), unix_now() + 300);
        format!("x-svrngs-pass: {}\r\n", pass.encode())
    };
    let member = pair
        .get("/lend/shelf/whoami", &own(2, vec![shelf.clone()]))
        .await;
    assert_eq!(member.status, 200, "{}", member.text);
    let head = last(&heads);
    assert_eq!(
        header_values(&head, "x-svrngs-app"),
        vec![app.as_str()],
        "a member's own pass names its page's app: {head}"
    );
    let asked = heads.lock().unwrap().len();
    for (row, pass) in [
        ("naming another lend", own(2, vec![])),
        ("signed by another key", own(5, vec![shelf.clone()])),
    ] {
        let refused = pair.get("/lend/shelf/whoami", &pass).await;
        assert_eq!(refused.status, 403, "{row}: {}", refused.text);
        assert!(
            refused.text.contains(r#""kind":"pass""#),
            "{row}: {}",
            refused.text
        );
    }
    assert_eq!(
        heads.lock().unwrap().len(),
        asked,
        "no refused pass reached shelf"
    );
    pair.close().await;
}

/// The service is asked as on its own computer (u5-2): the door names it
/// `localhost:<its port>` in `Host`, whatever address the member's browser
/// used, so an app that refuses a host it does not know answers; and a
/// redirect to that address comes back as a path, which the browser reads
/// on the lend's own origin. Planted: `Host` passed as the client sent it
/// (the door before u5-2), which the service refuses 403.
#[tokio::test]
async fn the_service_is_asked_on_its_own_address_and_its_redirect_comes_back_a_path() {
    let pair = Pair::new().await;
    pair.lend("guarded", origins::host_origin().await.port(), &[]);
    let reply = pair.get("/lend/guarded/go", "").await;
    assert_eq!(reply.status, 302, "{}", reply.text);
    assert_eq!(
        header_values(&reply.text, "location"),
        vec!["/landed?x=1"],
        "{}",
        reply.text
    );
    pair.close().await;
}

/// **A removed person is refused as removed, by pass and by dial** (U7
/// (e); DESIGN §6; demo/c.md, "Removed"). Ada keeps the group and removes
/// bo. A pass bo signed for `shelf`, presented by its holder, and bo's own
/// computer dialling on the members' protocol are each answered 403 with
/// svrngs's refusal `removed` and its words; neither reaches the service.
/// Planted, watched failing: the pass's removed check out (refused `pass`,
/// after the ring was asked) and a removed dialer closed like a stranger
/// (no reply).
#[tokio::test]
async fn a_removed_person_is_refused_as_removed_by_pass_and_by_dial() {
    use svrngs_core::alpn::Alpn;
    use svrngs_core::pass::{Label, Pass};

    let (pair, heads) = lending_heads().await;
    let shelf = Label::of(pair.a.endpoint().id().as_bytes(), "shelf");
    let bo = svrngs_node::Removal {
        person: Person::parse("bo").unwrap(),
        joined: None,
    };
    pair.a.rail().commit(NS, &bo).unwrap().unwrap();
    let holder = signing(9);
    let pass = Pass::sign(
        &signing(2),
        &actor_of(&holder),
        (shelf.clone(), vec![]),
        unix_now() + 3600,
    );
    let (as_holder, as_bo) = (
        bridge_as(&holder, &pair.a, Alpn::Lend).await,
        bridge_as(&signing(2), &pair.a, Alpn::Member).await,
    );
    let by_pass = raw_to(
        as_holder.local_addr(),
        format!(
            "GET /lend/{shelf}/x HTTP/1.1\r\nHost: local\r\nx-svrngs-pass: {}\r\nConnection: close\r\n\r\n",
            pass.encode()
        ),
    )
    .await;
    let by_dial = raw_to(
        as_bo.local_addr(),
        "GET /lend/shelf/x HTTP/1.1\r\nHost: local\r\nConnection: close\r\n\r\n".into(),
    )
    .await;
    for (how, reply) in [("by pass", by_pass), ("by dial", by_dial)] {
        assert_eq!(reply.status, 403, "{how}: {}", reply.text);
        assert!(
            reply.text.contains("x-svrngs-refused: removed"),
            "{how}: {}",
            reply.text
        );
        assert!(
            reply.text.contains("You are no longer in this group"),
            "{how}: {}",
            reply.text
        );
    }
    assert!(
        heads.lock().unwrap().is_empty(),
        "the service was never asked"
    );
    pair.close().await;
}
