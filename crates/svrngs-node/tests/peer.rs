//! A computer's loopback reaches no friend's door (arch-peer; AUDIT §2.1):
//! a program on the computer that holds no key, or a page in its browser,
//! is never carried to another computer's door as this computer's person.
//! Alex founds a group and lets Sam in, two nodes in one process, no relay,
//! each listening on 127.0.0.1 alone; Sam lends `pad`, which asks nobody,
//! and `book`, which asks before an app uses it. Alex's computer dials
//! Sam's as it does on its own — a round, and Sam's pad opened at its
//! origin — and then every TCP port this process listens on is asked, as
//! `lsof` lists them: Alex's, Sam's and the services'. A test binary of its
//! own, its tests one at a time ([`ONE_AT_A_TIME`]), so no other test's
//! port is asked.
//!
//! A lend's origin on Alex's computer reaches Sam's door by design, and is
//! not one of these: it answers only a lend's host, and carries the app
//! that host names in a pass (`origin.rs`).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[allow(dead_code)]
#[path = "door/origins.rs"]
mod origins;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::loopback_sockets;
use serde_json::json;
use svrngs_core::pass::Label;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend::{self, LendName};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const NS: &str = "house";

/// Held by each test for its whole run: every test's Alex and Sam have the
/// same keys, so two at once would ask each other's ports, and Sam's
/// bridge port would be the first test's.
static ONE_AT_A_TIME: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn no_seed(_: &Path) -> Result<Option<String>, String> {
    Ok(None)
}

async fn start(seed: u8, dir: &Path) -> Node {
    let relay = RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: true,
    };
    Node::start(
        dir.to_path_buf(),
        signing(seed),
        &relay,
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// `dir`'s computer lends `name` on `port`, asking before an app uses it
/// when `asks`.
fn lends(dir: &Path, name: &str, port: u16, asks: Option<&str>) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let body = json!({ "port": port, "kind": "app", "limit": 3, "asks": asks });
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// The app nobody named, which a probe says yes to.
fn nobody() -> Label {
    Label::of(&[7; 32], "nobody")
}

/// Alex's computer and Sam's, Sam lending `pad` and `book`, once Alex's has
/// dialled Sam's as it does on its own.
struct Dialled {
    alex: Node,
    sam: Node,
    sam_dir: tempfile::TempDir,
    _alex_dir: tempfile::TempDir,
    pad_heads: Arc<Mutex<Vec<String>>>,
}

impl Dialled {
    async fn new() -> Self {
        let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let alex = start(1, alex_dir.path()).await;
        let sam = start(2, sam_dir.path()).await;
        let seed = Roster::new(
            [(Person::parse("alex").unwrap(), vec![actor_of(&signing(1))])]
                .into_iter()
                .collect(),
        );
        for node in [&alex, &sam] {
            node.rail().set_seed(NS, &seed).unwrap();
        }
        let admit = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse("sam").unwrap(),
                key: actor_of(&signing(2)),
                invite: None,
            },
            on_behalf_of: None,
        };
        alex.rail().commit(NS, &admit).unwrap().unwrap();
        alex.transport()
            .add_known_peer(sam.endpoint().id(), loopback_sockets(sam.endpoint()));
        let (pad, pad_heads) = origins::head_origin().await;
        lends(sam_dir.path(), "pad", pad.port(), None);
        let (book, _) = origins::head_origin().await;
        lends(sam_dir.path(), "book", book.port(), Some("your books"));

        tokio::time::timeout(Duration::from_secs(30), alex.round())
            .await
            .expect("Alex's round finishes");
        let label = Label::of(sam.endpoint().id().as_bytes(), "pad");
        let host = format!("{label}.localhost:{}", alex.api_addr().port());
        let opened = plain(alex.api_addr().port(), &host, "GET / HTTP/1.1\r\n", "").await;
        assert!(
            opened
                .as_deref()
                .is_some_and(|o| o.starts_with("HTTP/1.1 200")),
            "Sam's pad at its origin on Alex's computer: {opened:?}"
        );
        Self {
            alex,
            sam,
            sam_dir,
            _alex_dir: alex_dir,
            pad_heads,
        }
    }

    /// What reached Sam's door, read at the far end: each port whose probe
    /// Sam's pad was told, as whom; and a yes kept on Sam's book for
    /// [`nobody`].
    fn reached(&self, ports: &[u16]) -> Vec<String> {
        let heads = self.pad_heads.lock().unwrap();
        let mut reached: Vec<String> = ports
            .iter()
            .filter_map(|port| {
                let seen = heads
                    .iter()
                    .find(|h| h.starts_with(&format!("GET /probe-{port} ")))?;
                let person = header_values(seen, "x-svrngs-person").join(",");
                Some(format!(
                    "{port}: Sam's pad, its service told the request is {person}'s"
                ))
            })
            .collect();
        let book = lend::choices_path(self.sam_dir.path(), &LendName::parse("book").unwrap());
        let kept = std::fs::read_to_string(book).unwrap_or_default();
        if kept.contains(nobody().as_str()) {
            reached.push(format!(
                "a yes kept on Sam's book for an app nobody named: {kept}"
            ));
        }
        reached
    }

    async fn close(self) {
        self.sam.endpoint().close().await;
        self.alex.endpoint().close().await;
    }
}

fn header_values<'a>(head: &'a str, name: &str) -> Vec<&'a str> {
    head.lines()
        .filter_map(|l| l.split_once(':'))
        .filter(|(n, _)| n.trim().eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim())
        .collect()
}

/// Every TCP port this process listens on, as `lsof` lists them. `lsof`
/// missing or failing panics, so a listing that could not be made is never
/// read as no port.
fn listening() -> Vec<u16> {
    let out = std::process::Command::new("lsof")
        .args(["-nP", "-a", "-iTCP", "-sTCP:LISTEN", "-Fn", "-p"])
        .arg(std::process::id().to_string())
        .output()
        .expect("lsof runs: the test lists this process's ports by it");
    assert!(out.status.success(), "lsof: {out:?}");
    let mut ports: Vec<u16> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.strip_prefix('n')?.rsplit_once(':')?.1.parse().ok())
        .collect();
    ports.sort_unstable();
    ports.dedup();
    assert!(!ports.is_empty(), "lsof listed no port: {out:?}");
    ports
}

/// One raw request to `127.0.0.1:port` naming `host`, as a program holding
/// no key sends it: the whole reply, or `None` when the port answers
/// nothing within 10 s.
async fn plain(port: u16, host: &str, head: &str, body: &str) -> Option<String> {
    let mut sock = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .ok()?;
    let ask = format!(
        "{head}Host: {host}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    sock.write_all(ask.as_bytes()).await.ok()?;
    let mut reply = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), sock.read_to_end(&mut reply))
        .await
        .ok()?
        .ok()?;
    Some(String::from_utf8_lossy(&reply).into_owned())
}

/// **No loopback port reaches a friend's door** (arch-peer (a)). A program
/// on Alex's computer holding no key, speaking no svrngs, naming no lend's
/// host and carrying no pass sends to every port: `GET
/// /lend/pad/probe-<port>`, `GET /choices/book`, and `POST /choices/book`
/// saying yes for an app nobody named. None reaches Sam's door: Sam's pad is
/// told no probe, no port answers Alex's choices, and no yes is kept.
/// Planted: the bridge as it was at 965b52b (`guest.rs` `HttpBridge`, at
/// `iroh.rs` `preferred_bridge_port`), which carried all three as Alex
/// through port 30029 (`demo/runs/arch/2026-10-15-peer/a-red.out`), until
/// arch-peer (d) took it out.
#[tokio::test]
async fn no_loopback_port_reaches_a_friends_door() {
    let _one = ONE_AT_A_TIME.lock().await;
    let two = Dialled::new().await;
    let yes = json!({ "app": nobody(), "said": "yes" }).to_string();
    let ports = listening();
    let mut answered = Vec::new();
    for &port in &ports {
        let host = format!("127.0.0.1:{port}");
        plain(
            port,
            &host,
            &format!("GET /lend/pad/probe-{port} HTTP/1.1\r\n"),
            "",
        )
        .await;
        let read = plain(port, &host, "GET /choices/book HTTP/1.1\r\n", "").await;
        if let Some(read) =
            read.filter(|r| r.starts_with("HTTP/1.1 200") && r.contains("\"choices\""))
        {
            let said = read.lines().last().unwrap_or_default();
            answered.push(format!(
                "{port}: Alex's choices on Sam's book, read: {said}"
            ));
        }
        let head = "POST /choices/book HTTP/1.1\r\nContent-Type: text/plain\r\n";
        if let Some(said) = plain(port, &host, head, &yes)
            .await
            .filter(|r| r.contains("\"said\":\"yes\""))
        {
            let said = said.lines().last().unwrap_or_default();
            answered.push(format!("{port}: Sam's door kept a yes as Alex's: {said}"));
        }
    }
    let reached = [two.reached(&ports), answered].concat();
    assert!(
        reached.is_empty(),
        "a program holding no key reached Sam's door through a port on this computer:\n{}",
        reached.join("\n")
    );
    two.close().await;
}

/// The browsers a page is asked in: Chromium with the page on loopback,
/// Chromium with the page taken to be on the internet and the person's
/// leave to reach the local network given, and WebKit with the page on
/// loopback — WebKit has no flag that makes a page on loopback count as
/// one on the internet, and no host off this computer is used (fence 8),
/// so a page from the internet in WebKit is could-not-judge.
#[derive(Clone, Copy, Debug)]
enum Browser {
    Chromium,
    ChromiumPublic,
    Webkit,
}

/// **No page in a browser reaches a friend's door** (arch-peer (a)): a
/// page on another origin than any of svrngs's asks every port the
/// browser half is given (`web/tests/bridge.mjs`) — `GET
/// /lend/pad/probe-<port>`, read if the browser lets it be, and `POST
/// /choices/book` saying yes for an app nobody named, sent as a form posts
/// so no preflight asks first — and none reaches Sam's door, read at the
/// far end. Planted: the bridge as it was at 965b52b, which Chromium and
/// WebKit both let a loopback page's requests through to, unread, and
/// Chromium a page on the internet's once the person allows it
/// (`demo/runs/arch/2026-10-15-peer/a-red.out`), until arch-peer (d).
async fn no_page_reaches_a_friends_door(browser: Browser) {
    let _one = ONE_AT_A_TIME.lock().await;
    let two = Dialled::new().await;
    let ports = listening();
    let (engine, public) = match browser {
        Browser::Chromium => ("chromium", false),
        Browser::ChromiumPublic => ("chromium", true),
        Browser::Webkit => ("webkit", false),
    };
    let args = json!({ "engine": engine, "public": public, "ports": ports, "app": nobody() });
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = tokio::process::Command::new("node")
        .arg(repo.join("web/tests/bridge.mjs"))
        .arg(args.to_string())
        .kill_on_drop(true)
        .output()
        .await
        .expect("node runs: the browser tests need Node 18 or later");
    assert!(
        out.status.success(),
        "the browser half failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let page: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("the browser half's JSON line");
    let reached = two.reached(&ports);
    let seen: Vec<_> = page["asked"]
        .as_array()
        .expect("the ports asked")
        .iter()
        .filter(|a| {
            reached
                .iter()
                .any(|r| r.starts_with(&format!("{}:", a["port"])))
        })
        .collect();
    assert!(
        reached.is_empty(),
        "a page at {} in {browser:?} reached Sam's door through a port on this computer:\n{}\n\
         what the page was let see there: {}",
        page["page"],
        reached.join("\n"),
        serde_json::to_string(&seen).unwrap()
    );
    two.close().await;
}

#[tokio::test]
async fn no_page_in_chromium_reaches_a_friends_door() {
    no_page_reaches_a_friends_door(Browser::Chromium).await
}

#[tokio::test]
async fn no_page_from_the_internet_in_chromium_reaches_a_friends_door() {
    no_page_reaches_a_friends_door(Browser::ChromiumPublic).await
}

#[tokio::test]
async fn no_page_in_webkit_reaches_a_friends_door() {
    no_page_reaches_a_friends_door(Browser::Webkit).await
}
