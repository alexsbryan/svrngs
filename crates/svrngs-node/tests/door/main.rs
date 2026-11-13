// SPDX-License-Identifier: AGPL-3.0-or-later
// The identity tests: ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/iroh_identity_forward.rs@e1a43bcd2
//! The member door, through it (DESIGN §3; PLAN §4, U3's inventory; the u3-4
//! row): the door working across QUIC, each test a contained one
//! (docs/CHECKS.md, "Seam migration ledger", unit 2). A lends a service on
//! its loopback; a key standing in A's group dials A by key and asks through
//! a loopback bridge, as a member's browser will. Nodes in one process, no
//! relay, each listening on 127.0.0.1 alone: on a wildcard, iroh may carry
//! the requests on this computer's private address, and the door guesses a
//! viewer there is home and counts no one (ralph/REVIEW_FINDINGS.md #33).
//!
//! Each rule the door applies is judged on its own from explicit inputs in
//! `src/door/tests.rs`, `src/oidc/tests.rs`, `src/door/limit/tests.rs`,
//! `src/door/choices/tests.rs` and `src/lend/tests.rs`; here, only that the
//! door applies them to a verified dialer's request and refuses before the
//! service is asked. `docs/seams.tsv` names where each retired test's
//! assertions now are. Every byte a member receives is scanned for A's
//! secret ([`clean`]).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../common/mod.rs"]
mod common;
mod lists;
mod live;
mod newcomer;
mod oidc;
mod origins;
mod refusals;

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
            loopback_only: true,
        },
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
        svrngs_core::tests_support::let_in_by(NS, &signing(2), "bo", |act| {
            let act = svrngs_core::tests_support::Fixture { act };
            a.rail().commit(NS, &act).unwrap().unwrap().lines()[0]
                .id
                .clone()
        });
        Self::dialling(a, b, dir, db).await
    }

    /// B's bridge to A, once A stands with it.
    async fn dialling(a: Node, b: Node, dir: tempfile::TempDir, db: tempfile::TempDir) -> Self {
        let base = b
            .transport()
            .spliced(&Contact {
                key: a.endpoint().id(),
                relays: Default::default(),
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
        let body = serde_json::json!({ "port": port, "kind": "k", "limit": 3, "refused": refused, "groups": [NS] });
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

/// Ada's computer, A (key 1), and the groups it holds, each seeded before it
/// starts: the house (ada, and bo on two devices, keys 2 and 6), trip (ada
/// and bo), yard (ada and cy, key 3), bos (bo alone, so ada stands not in it)
/// and gone (cy alone). `before` writes declarations A finds as it starts.
struct World {
    a: Node,
    dir: tempfile::TempDir,
}

impl World {
    async fn new(before: impl FnOnce(&std::path::Path)) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let rail = svrngs_runtime::RingRail::new(dir.path(), std::sync::Arc::new(signing(1)));
        let people = |list: &[(&str, &[u8])]| {
            Roster::new(
                list.iter()
                    .map(|(p, keys)| {
                        let keys = keys.iter().map(|k| actor_of(&signing(*k))).collect();
                        (Person::parse(p).unwrap(), keys)
                    })
                    .collect(),
            )
        };
        let ada_bo: &[(&str, &[u8])] = &[("ada", &[1]), ("bo", &[2])];
        for (group, roster) in [
            (NS, people(&[("ada", &[1]), ("bo", &[2, 6])])),
            ("trip", people(ada_bo)),
            ("yard", people(&[("ada", &[1]), ("cy", &[3])])),
            ("bos", people(&[("bo", &[2])])),
            ("gone", people(&[("cy", &[3])])),
        ] {
            rail.set_seed(group, &roster).unwrap();
        }
        drop(rail);
        before(dir.path());
        let a = start(1, dir.path()).await;
        World { a, dir }
    }

    /// A lends `name` as `body` declares it.
    fn lend(&self, name: &str, body: serde_json::Value) {
        let dir = lend::lends_dir(self.dir.path());
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{name}.json")), body.to_string()).unwrap();
    }

    /// A keeps `value` as the lend's secret, sent as `header`.
    fn secret(&self, name: &str, header: &str, value: &str) {
        let dir = lend::secret_dir(self.dir.path(), &LendName::parse(name).unwrap());
        declared::write_declared_in(&dir, header, value).unwrap();
    }

    /// A port that dials A as `key` on `alpn`.
    async fn as_key(&self, key: u8, alpn: svrngs_core::alpn::Alpn) -> SocketAddr {
        bridge_as(&signing(key), &self.a, alpn).await.local_addr()
    }

    /// The label of A's lend `name`.
    fn label(&self, name: &str) -> svrngs_core::pass::Label {
        svrngs_core::pass::Label::of(self.a.endpoint().id().as_bytes(), name)
    }

    /// `person`, key `seed`, let into `ns` by A's keeper.
    fn let_in(&self, ns: &str, seed: u8, person: &str) {
        svrngs_core::tests_support::let_in_by(ns, &signing(seed), person, |act| {
            let act = svrngs_core::tests_support::Fixture { act };
            self.a.rail().commit(ns, &act).unwrap().unwrap().lines()[0]
                .id
                .clone()
        });
    }

    /// Ada has Sam (visit key 9) over until tomorrow, opening `opens` of
    /// hers: the visit's id.
    fn visit(&self, opens: &[&str]) -> svrngs_node::OpId {
        let mut named = std::collections::BTreeMap::new();
        named.insert(
            actor_of(&signing(1)),
            opens.iter().map(|o| o.to_string()).collect(),
        );
        let act = RailAct::Visit(svrngs_node::Visit {
            guest: Person::parse("Sam").unwrap(),
            key: actor_of(&signing(SAM)),
            opens: named,
            until: unix_now() + 24 * 3600,
        });
        let fixture = svrngs_core::tests_support::Fixture { act };
        let recorded = self.a.rail().commit(NS, &fixture).unwrap().unwrap();
        recorded.lines()[0].id.clone()
    }
}

/// Sam's visit key, which Sam's page makes fresh for a visit.
const SAM: u8 = 9;

/// `by`'s pass to `holder` for the app `app`, reading `reads`, until
/// `until`, as the header a request carries.
fn pass(
    by: u8,
    holder: u8,
    app: &svrngs_core::pass::Label,
    reads: &[&svrngs_core::pass::Label],
    until: i64,
) -> String {
    let reads = reads.iter().map(|l| (*l).clone()).collect();
    let pass = svrngs_core::pass::Pass::sign(
        &signing(by),
        &actor_of(&signing(holder)),
        (app.clone(), reads),
        until,
    );
    format!("x-svrngs-pass: {}\r\n", pass.encode())
}

/// `method` of `path` through the bridge at `at`, `headers` added, with no
/// body.
async fn ask(at: SocketAddr, method: &str, path: &str, headers: &str) -> Reply {
    let body = if method == "GET" || method == "HEAD" {
        ""
    } else {
        "Content-Length: 0\r\n"
    };
    raw_to(
        at,
        format!(
            "{method} {path} HTTP/1.1\r\nHost: local\r\n{headers}{body}Connection: close\r\n\r\n"
        ),
    )
    .await
}

/// The kind svrngs marked an answer with, if it is its own.
fn kind(reply: &Reply) -> Option<String> {
    header_values(&reply.text, "x-svrngs-refused")
        .first()
        .map(|k| k.to_string())
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

/// Bytes cross the door over QUIC both ways, sha-exact: upstream's Range
/// origin over a 64 MiB file, the whole file 200, a range 206 with its
/// `Content-Range`, an unsatisfiable range 416; and a 64 MiB upload, which
/// the service reads the first byte of while the member still holds its
/// tail back, then whole (this is the test's memory bound: a door that holds
/// a body whole never lets the first byte through). Planted: `Range` dropped
/// (the 206 is a 200); the request body buffered whole.
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

    let (sink, first) = origins::upload_origin().await;
    pair.lend("sink", sink.port(), &[]);
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

/// A, its computer lending `name` on `port`, as [`Pair::lend`] declares one.
fn lends_on(dir: &std::path::Path, name: &str, port: u16) {
    let lends = lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    let body =
        serde_json::json!({ "port": port, "kind": "k", "limit": 3, "refused": [], "groups": [NS] });
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
    svrngs_core::tests_support::let_in_by(ns, &signing(seed), person, |act| {
        let act = svrngs_core::tests_support::Fixture { act };
        pair.a.rail().commit(ns, &act).unwrap().unwrap().lines()[0]
            .id
            .clone()
    });
    pair.a
        .transport()
        .add_known_peer(node.endpoint().id(), loopback_sockets(node.endpoint()));
    (node, dir)
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

/// **What a lend's service and a member receive through the door**, in
/// one crossing (a trust-floor gate, scripts/trust_floor.txt): THE failing
/// input (iroh_identity_forward.rs:584), a client that types the identity
/// headers itself, is answered by exactly one person and one key, the
/// door's, however cased, the rest of the prefix gone; and every other rule
/// of the head that `src/door/tests.rs` judges on its own is shown applied
/// by `forward` and `lend_for` to a verified dialer's request. Each step
/// names the test it came from, retired at its move (docs/seams.tsv).
/// Planted, each watched failing when its test was its own: the door's
/// strip skipped; `Range` dropped; the door's drop of an echoed secret
/// skipped; `Host` passed as the client sent it; `tell_origin` telling
/// every lend; every group bo stands in told (`pages` unfiltered); the
/// door taking a pass for any lend; the door reading no link; the lend's
/// own secret sent in place of none; `Admit::Readers` dropping the key;
/// the door naming the pass's first read; the guest's key put in
/// `Standing`.
#[tokio::test]
async fn a_forged_identity_header_is_replaced_by_the_verified_one() {
    use svrngs_core::alpn::Alpn;
    let world = World::new(|_| {}).await;
    let (origin, heads) = origins::head_origin().await;
    let port = origin.port();
    let shelf_json = serde_json::json!({ "port": port, "kind": "k", "refused": ["/Auth/Keys"], "groups": [NS, "yard"] });
    world.lend("shelf", shelf_json.clone());
    world.secret("shelf", "authorization", SECRET);
    let shelf = world.label("shelf");
    let (bo, bo6) = (
        world.as_key(2, Alpn::Member).await,
        world.as_key(6, Alpn::Member).await,
    );
    let actor = |seed: u8| actor_of(&signing(seed));

    // THE failing input (main.rs:233-241 at 5e20d0b3).
    let forged = "x-svrngs-person: forged\r\nX-SVRNGS-KEY: forged\r\nX-Svrngs-Anything: nope\r\n";
    let reply = ask(bo, "GET", "/lend/shelf/whoami", forged).await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let head = last(&heads);
    assert!(head.starts_with("GET /whoami HTTP/1.1\r\n"), "{head}");
    assert_eq!(header_values(&head, "x-svrngs-person"), vec!["bo"]);
    assert_eq!(header_values(&head, "x-svrngs-key"), vec![actor(2)]);
    assert!(
        header_values(&head, "x-svrngs-anything").is_empty(),
        "{head}"
    );
    // the_service_is_told_only_the_groups_the_lend_reaches (scope.rs:101-106).
    let pages = header_values(&head, "x-svrngs-pages").join(" ");
    assert!(pages.starts_with(&format!("{NS}=")), "{pages}");
    assert!(
        !pages.contains("trip") && !pages.contains("yard"),
        "{pages}"
    );
    let told = |head: &str| {
        let one = |n| header_values(head, n).join(",");
        (
            one("x-svrngs-key"),
            one("x-svrngs-user"),
            one("x-svrngs-device"),
        )
    };
    let from_2 = told(&head);

    // one_persons_two_devices_are_one_account_to_a_lend (main.rs:355-358).
    let reply = ask(bo6, "GET", "/lend/shelf/whoami", "").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let from_6 = told(&last(&heads));
    assert_eq!(
        (&from_6.0, &from_6.1),
        (&from_2.0, &from_2.1),
        "one key, one account"
    );
    assert_eq!(
        [from_2.2.as_str(), from_6.2.as_str()],
        [actor(2), actor(6)],
        "each device"
    );

    // two_keys_of_one_name_are_two_accounts_to_a_lend (main.rs:266-273).
    world.let_in(NS, 4, "bo");
    let other_bo = world.as_key(4, Alpn::Member).await;
    let reply = ask(other_bo, "GET", "/lend/shelf/whoami", "").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let head = last(&heads);
    assert_eq!(header_values(&head, "x-svrngs-person"), vec!["bo"]);
    let user = header_values(&head, "x-svrngs-user").join(",");
    assert!(
        !user.is_empty() && user != from_2.1,
        "each key its own account: {user}"
    );

    // range_if_range_and_cookie_pass_untouched (main.rs:376-382).
    let seek = "Range: bytes=10-19\r\nIf-Range: \"etag-1\"\r\nCookie: c=1\r\n";
    let reply = ask(bo, "GET", "/lend/shelf/library/title.bin?x=1", seek).await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let head = last(&heads);
    assert!(
        head.starts_with("GET /library/title.bin?x=1 HTTP/1.1\r\n"),
        "{head}"
    );
    assert_eq!(header_values(&head, "range"), vec!["bytes=10-19"]);
    assert_eq!(header_values(&head, "if-range"), vec!["\"etag-1\""]);
    assert_eq!(header_values(&head, "cookie"), vec!["c=1"]);

    // a_declared_credential_displaces_the_client_s_and_arrives_verbatim (main.rs:399).
    let viewer = "Authorization: MediaBrowser Token=\"the-viewers\"\r\n";
    assert_eq!(
        ask(bo, "GET", "/lend/shelf/Items", viewer).await.status,
        200
    );
    assert_eq!(header_values(&last(&heads), "authorization"), vec![SECRET]);

    // three_client_spellings_never_reach_the_service (main.rs:414-425).
    let asked = heads.lock().unwrap().len();
    let reply = ask(bo, "GET", "/lend/shelf/", "AUTHORIZATION: viewer-upper\r\n").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(header_values(&last(&heads), "authorization"), vec![SECRET]);
    for spelling in [
        "  authorization  : viewer-pad\r\n",
        "Authoriz\u{7f}ation: viewer-ctl\r\n",
    ] {
        let reply = ask(bo, "GET", "/lend/shelf/", spelling).await;
        assert_eq!(reply.status, 400, "{spelling:?}: {}", reply.text);
    }
    assert_eq!(
        heads.lock().unwrap().len(),
        asked + 1,
        "only the first reached the service"
    );
    assert!(!heads.lock().unwrap().iter().any(|h| h.contains("viewer-")));

    // a_secret_the_service_echoes_never_reaches_the_member (main.rs:437-446):
    // the service echoes it in x-seen-authorization, and [`clean`] scans.
    let reply = ask(bo, "GET", "/lend/shelf/Users", "").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(
        header_values(&last(&heads), "authorization"),
        vec![SECRET],
        "control: sent"
    );
    assert!(
        header_values(&reply.text, "x-seen-authorization").is_empty(),
        "{}",
        reply.text
    );

    // a_picky_service_answers_401_without_the_secret_and_200_with_it (main.rs:459-461).
    let picky = origins::picky_origin("x-emby-token", "the-holders-key").await;
    world.lend(
        "picky",
        serde_json::json!({ "port": picky.port(), "kind": "k", "groups": [NS] }),
    );
    let viewers = "X-Emby-Token: the-viewers-key\r\n";
    assert_eq!(
        ask(bo, "GET", "/lend/picky/Items", viewers).await.status,
        401
    );
    world.secret("picky", "X-Emby-Token", "the-holders-key");
    assert_eq!(
        ask(bo, "GET", "/lend/picky/Items", viewers).await.status,
        200
    );

    // the_service_is_asked_on_its_own_address_and_its_redirect_comes_back_a_path
    // (main.rs:1646-1651) and a_services_redirect_off_its_own_origin_is_never_made_a_path
    // (main.rs:1667-1680).
    let guarded = origins::host_origin().await.port();
    world.lend(
        "guarded",
        serde_json::json!({ "port": guarded, "kind": "k", "groups": [NS] }),
    );
    for (path, location) in [
        ("/lend/guarded/go", "/landed?x=1".to_string()),
        (
            "/lend/guarded/away",
            format!("http://localhost:{guarded}//evil.example/cb"),
        ),
        (
            "/lend/guarded/away-back",
            format!("http://localhost:{guarded}/\\evil.example/cb"),
        ),
    ] {
        let reply = ask(bo, "GET", path, "").await;
        assert_eq!(reply.status, 302, "{path}: {}", reply.text);
        assert_eq!(
            header_values(&reply.text, "location"),
            vec![location.as_str()],
            "{path}"
        );
    }

    // a_service_is_told_one_address_for_itself (main.rs:1712-1727).
    let host = format!("{shelf}.localhost:6473");
    let request = format!(
        "GET /lend/shelf/whoami HTTP/1.1\r\nHost: {host}\r\nOrigin: http://{host}\r\n\
         x-forwarded-host: evil.example\r\nX-Forwarded-Proto: https\r\nConnection: close\r\n\r\n"
    );
    assert_eq!(raw_to(bo, request).await.status, 200);
    let head = last(&heads);
    assert!(
        header_values(&head, "x-forwarded-host").is_empty(),
        "{head}"
    );
    assert!(
        header_values(&head, "x-forwarded-proto").is_empty(),
        "{head}"
    );
    let own = format!("localhost:{port}");
    assert_eq!(header_values(&head, "host"), vec![own.as_str()], "{head}");
    assert_eq!(
        header_values(&head, "origin"),
        vec![format!("http://{own}").as_str()],
        "{head}"
    );

    // the_browser_origin_is_told_to_the_service_as_forwarded_headers (oidc.rs:632-645)
    // and a_rooms_address_is_told_http_and_its_origin_passes_authorize (oidc.rs:669).
    let (media, media_heads) = origins::head_origin().await;
    world.lend(
        "media",
        serde_json::json!({ "port": media.port(), "kind": "k", "groups": [NS],
            "sign_in_with_svrngs": { "client_id": "svrngs-media", "client_secret": "media-secret_2026.10" } }),
    );
    let label = world.label("media");
    for (host, proto) in [
        (format!("{label}.localhost:6473"), "http"),
        (format!("{label}.svrngs.example"), "https"),
        ("192.168.1.20:7123".to_string(), "http"),
    ] {
        let request = format!(
            "GET /lend/media/whoami HTTP/1.1\r\nHost: {host}\r\nx-forwarded-host: evil.example\r\n\
             X-Forwarded-Proto: https\r\nConnection: close\r\n\r\n"
        );
        assert_eq!(raw_to(bo, request).await.status, 200, "{host}");
        let head = last(&media_heads);
        assert_eq!(
            header_values(&head, "x-forwarded-host"),
            vec![host.as_str()],
            "{head}"
        );
        assert_eq!(
            header_values(&head, "x-forwarded-proto"),
            vec![proto],
            "{head}"
        );
        let own = format!("localhost:{}", media.port());
        assert_eq!(header_values(&head, "host"), vec![own.as_str()], "{head}");
    }

    // a_pass_opens_only_the_lend_it_names_to_the_key_holding_it (main.rs:1153-1172).
    let (holder, endpoint) = (
        world.as_key(19, Alpn::Lend).await,
        world.as_key(10, Alpn::Lend).await,
    );
    let bos = pass(2, 19, &shelf, &[], unix_now() + 3600);
    let reply = ask(holder, "GET", &format!("/lend/{shelf}/whoami"), &bos).await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let head = last(&heads);
    assert!(head.starts_with("GET /whoami HTTP/1.1\r\n"), "{head}");
    assert_eq!(
        header_values(&head, "x-svrngs-person"),
        vec!["bo"],
        "{head}"
    );
    assert_eq!(
        header_values(&head, "x-svrngs-key"),
        vec![actor(2)],
        "{head}"
    );
    assert!(header_values(&head, "x-svrngs-pass").is_empty(), "{head}");
    assert_eq!(
        header_values(&head, "authorization"),
        vec![SECRET],
        "{head}"
    );

    // a_link_carries_a_pass_to_the_key_dialling_and_no_further (main.rs:1278-1287).
    let carried = svrngs_core::pass::Pass::sign(
        &signing(2),
        &actor(19),
        (shelf.clone(), vec![]),
        unix_now() + 3600,
    );
    let mut link = carried.link(&actor(10));
    link.sig = svrngs_core::sign_message(&signing(19), &link.message());
    let linked = format!(
        "x-svrngs-pass: {}\r\nx-svrngs-link: {}\r\n",
        carried.encode(),
        link.encode()
    );
    let reply = ask(endpoint, "GET", &format!("/lend/{shelf}/whoami"), &linked).await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let head = last(&heads);
    assert_eq!(
        header_values(&head, "x-svrngs-person"),
        vec!["bo"],
        "{head}"
    );
    assert!(header_values(&head, "x-svrngs-link").is_empty(), "{head}");
    assert!(header_values(&head, "x-svrngs-pass").is_empty(), "{head}");

    // an_app_reads_a_lend_bare_and_never_writes_or_upgrades_it (main.rs:1359-1375).
    let picks = world.label("picks");
    for (how, at, pass) in [
        (
            "a phone's pass",
            holder,
            pass(2, 19, &picks, &[&shelf], unix_now() + 3600),
        ),
        (
            "a computer's own pass",
            bo,
            pass(2, 2, &picks, &[&shelf], unix_now() + 3600),
        ),
    ] {
        let path = format!("/lend/{shelf}/whoami");
        let get = ask(at, "GET", &path, &pass).await;
        assert_eq!(get.status, 200, "{how}: {}", get.text);
        assert!(
            header_values(&last(&heads), "authorization").is_empty(),
            "{how}: own secret sent"
        );
        let upgrade = format!("{pass}Upgrade: websocket\r\n");
        let raw = ask(at, "GET", &path, &upgrade).await;
        assert_eq!(raw.status, 200, "{how}: {}", raw.text);
        assert!(
            header_values(&last(&heads), "upgrade").is_empty(),
            "{how}: only that GET"
        );
    }

    // a_pass_s_app_is_told_and_a_client_s_own_stripped (main.rs:1559-1612).
    let forged = format!(
        "{}X-Svrngs-App: forged\r\n",
        pass(2, 19, &picks, &[&shelf], unix_now() + 3600)
    );
    assert_eq!(
        ask(holder, "GET", &format!("/lend/{shelf}/whoami"), &forged)
            .await
            .status,
        200
    );
    let head = last(&heads);
    assert_eq!(
        header_values(&head, "x-svrngs-app"),
        vec![picks.as_str()],
        "{head}"
    );
    assert_eq!(
        header_values(&head, "x-svrngs-person"),
        vec!["bo"],
        "{head}"
    );
    assert_eq!(
        ask(bo, "GET", "/lend/shelf/whoami", "X-Svrngs-App: forged\r\n")
            .await
            .status,
        200
    );
    assert!(header_values(&last(&heads), "x-svrngs-app").is_empty());
    let own = pass(2, 2, &picks, &[&shelf], unix_now() + 300);
    assert_eq!(ask(bo, "GET", "/lend/shelf/whoami", &own).await.status, 200);
    let head = last(&heads);
    assert_eq!(
        header_values(&head, "x-svrngs-app"),
        vec![picks.as_str()],
        "{head}"
    );

    // a_send_is_a_carried_app_s_from_its_person_s_own_computer (main.rs:1837-1844).
    let mut sends = shelf_json.clone();
    sends["sends"] = serde_json::json!(["POST /whoami"]);
    world.lend("shelf", sends);
    let taste = svrngs_core::pass::Label::carried("Taste", NS);
    let own = pass(2, 2, &taste, &[&shelf], unix_now() + 300);
    let sent = ask(bo, "POST", &format!("/lend/{shelf}/whoami"), &own).await;
    assert_eq!(sent.status, 200, "{}", sent.text);
    let head = last(&heads);
    assert!(head.starts_with("POST /whoami HTTP/1.1\r\n"), "{head}");
    assert!(
        header_values(&head, "authorization").is_empty(),
        "as no one: {head}"
    );
    world.lend("shelf", shelf_json);

    // a_readers_key_goes_in_its_own_header_only_on_another_app_s_read
    // (main.rs:1428-1442) and the_readers_secret_goes_only_to_another_app_s_get
    // (main.rs:1477-1519), one readers' account holding both.
    const KEY: &str = "immich-api-key-for-apps";
    const READERS: &str = r#"MediaBrowser Token="the-group""#;
    let readers = lend::readers_dir(world.dir.path(), &LendName::parse("shelf").unwrap());
    declared::write_declared_in(&readers, "x-api-key", KEY).unwrap();
    declared::write_declared_in(&readers, "authorization", READERS).unwrap();
    for (why, app, method, sent, key) in [
        ("another app's GET", &picks, "GET", READERS, true),
        ("another app's HEAD", &picks, "HEAD", READERS, true),
        ("the lend's own page", &shelf, "GET", SECRET, false),
        ("the lend's own page's POST", &shelf, "POST", SECRET, false),
    ] {
        // Another app's page sends its own key, which never reaches the service.
        let page_s_own = if key {
            "x-api-key: the-page-s-own\r\n"
        } else {
            ""
        };
        let headers = format!(
            "{}{page_s_own}",
            pass(2, 19, app, &[&shelf], unix_now() + 3600)
        );
        let reply = ask(holder, method, &format!("/lend/{shelf}/whoami"), &headers).await;
        assert_eq!(reply.status, 200, "{why}: {}", reply.text);
        let head = last(&heads);
        assert_eq!(header_values(&head, "authorization"), vec![sent], "{why}");
        let want: Vec<&str> = if key { vec![KEY] } else { vec![] };
        assert_eq!(header_values(&head, "x-api-key"), want, "{why}: {head}");
        assert!(!head.contains("the-page-s-own"), "{why}: {head}");
    }
    for (why, method) in [("a member's GET", "GET"), ("a member's own POST", "POST")] {
        let member = ask(bo, method, "/lend/shelf/whoami", "").await;
        assert_eq!(member.status, 200, "{why}: {}", member.text);
        assert_eq!(
            header_values(&last(&heads), "authorization"),
            vec![SECRET],
            "{why}"
        );
    }

    // sam_opens_kims_jellyfin_as_kims_guest (guests.rs:109-127), the guest
    // ada's here, and Sam's open told no group's page
    // (sams_visit_key_syncs_nothing_and_reads_no_person_presence_or_record, guests.rs:316-319).
    world.visit(&["shelf"]);
    let sams_holder = world.as_key(11, Alpn::Lend).await;
    let sams = pass(SAM, 11, &shelf, &[], unix_now() + 3600);
    let opened = ask(sams_holder, "GET", &format!("/lend/{shelf}/whoami"), &sams).await;
    assert_eq!(opened.status, 200, "{}", opened.text);
    let head = last(&heads);
    let sam = actor(SAM);
    assert_eq!(
        header_values(&head, "x-svrngs-person"),
        ["Sam (ada's guest)"],
        "{head}"
    );
    assert_eq!(
        header_values(&head, "x-svrngs-key"),
        [sam.as_str()],
        "{head}"
    );
    assert_eq!(
        header_values(&head, "x-svrngs-device"),
        [sam.as_str()],
        "{head}"
    );
    let user = format!("Sam-{}.guest", &sam[..16]);
    assert_eq!(
        header_values(&head, "x-svrngs-user"),
        [user.as_str()],
        "{head}"
    );
    assert!(
        !head.contains(&actor(1)),
        "ada's key reached the service: {head}"
    );
    let pages = header_values(&head, "x-svrngs-pages").join(" ");
    assert!(
        !pages.contains(NS),
        "Sam's open was told the group's page: {pages}"
    );
    world.a.endpoint().close().await;
}
