//! The room (DESIGN §3, "The room", call 3): the lends Alex's computer
//! turns on for the room, served to its own network by the running node,
//! and closed on another network, saying why. A restart is a process's,
//! judged with the command's (crates/svrngs/tests/share.rs,
//! `room_asks_svrngs_running_here_and_keeps_it_across_a_restart`). Two nodes in one process, no relay, each dialled
//! on 127.0.0.1 (#33's fix): Alex founds a group and lets Sam in, and Sam
//! lends the conformance canary (`apps/web-conformance`, u5-2). The room is
//! asked at `/v1/room` as the home page and `svrngs room` ask it, and a
//! client on this host's private address — not on loopback — asks the room
//! what the canary asks of a lend on a computer. Each test's Sam has a key
//! of its own, so each lend's port (`room::port_of`) is its own. The pure
//! decisions — the address, the network, a caller off it — are judged with
//! values in `src/room_tests.rs`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use common::loopback_sockets;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use svrngs_core::pass::Label;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend::{self, LendName};
use svrngs_node::node::Node;
use svrngs_node::room;
use svrngs_node::{actor_of, declared, Person, RailAct, Roster, SigningKey};

const NS: &str = "house";
const SAMS: &str = "Bearer sams-secret-61";
const ALEX: u8 = 61;
const WAIT: Duration = Duration::from_secs(30);

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn no_seed(_: &Path) -> Result<Option<String>, String> {
    Ok(None)
}

async fn start(seed: u8, dir: &Path) -> Node {
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

/// This host's private address, where the room serves; could-not-judge
/// on a host with none.
fn here() -> Ipv4Addr {
    match svrngs_node::network::route() {
        Some(IpAddr::V4(v4)) if v4.is_private() => v4,
        other => panic!(
            "could not judge: this host's route leaves by {other:?}, no private address to serve a room on"
        ),
    }
}

/// Alex's computer, started over `dir`, its group holding Sam's key `sam`.
async fn alex(dir: &Path, sam: u8) -> Node {
    let alex = start(ALEX, dir).await;
    founded(&alex);
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("sam").unwrap(),
            key: actor_of(&signing(sam)),
            invite: None,
        },
        on_behalf_of: None,
    };
    alex.rail().commit(NS, &admit).unwrap().unwrap();
    alex
}

/// `node`'s group, founded by Alex.
fn founded(node: &Node) {
    let seed = Roster::new(
        [(
            Person::parse("alex").unwrap(),
            vec![actor_of(&signing(ALEX))],
        )]
        .into_iter()
        .collect(),
    );
    node.rail().set_seed(NS, &seed).unwrap();
}

/// Sam's computer, key `sam`, over `dir`, lending the canary as `app`,
/// and known to Alex's at its loopback sockets.
async fn sam(sam: u8, dir: &Path, alex: &Node) -> (Node, tokio::process::Child, u16) {
    let node = start(sam, dir).await;
    founded(&node);
    let (canary, port) = canary("sam").await;
    lends(dir, "app", port);
    alex.transport()
        .add_known_peer(node.endpoint().id(), loopback_sockets(node.endpoint()));
    (node, canary, port)
}

/// The canary, as `who`, and the port it listens on.
async fn canary(who: &str) -> (tokio::process::Child, u16) {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut child = tokio::process::Command::new("node")
        .arg(repo.join("apps/web-conformance/serve.mjs"))
        .arg(who)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs the canary: Node 18 or later");
    let mut lines = tokio::io::BufReader::new(child.stdout.take().unwrap()).lines();
    let first = lines.next_line().await.unwrap().expect("its port");
    let port = serde_json::from_str::<Value>(&first).unwrap()["port"]
        .as_u64()
        .unwrap() as u16;
    (child, port)
}

/// `dir`'s computer lends `name` on `port`, with Sam's secret.
fn lends(dir: &Path, name: &str, port: u16) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let body = serde_json::json!({ "port": port, "kind": "app", "limit": 3 });
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
    let secrets = lend::secret_dir(dir, &LendName::parse(name).unwrap());
    declared::write_declared_in(&secrets, "authorization", SAMS).unwrap();
}

fn http() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// The room on `node` asked `body` at `/v1/room`, as `svrngs room` and
/// the home page ask it: its status and its answer.
async fn asked(node: &Node, body: Value) -> (u16, Value) {
    let answer = http()
        .post(format!("http://{}/v1/room", node.api_addr()))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = answer.status().as_u16();
    let text = answer.text().await.unwrap();
    let body = serde_json::from_str(&text)
        .unwrap_or_else(|_| panic!("the room answered {status}, not JSON: {text:?}"));
    (status, body)
}

/// The room on `node`, as `GET /v1/room` says it.
async fn on(node: &Node) -> Value {
    let answer = http()
        .get(format!("http://{}/v1/room", node.api_addr()))
        .send()
        .await
        .unwrap();
    let text = answer.text().await.unwrap();
    serde_json::from_str(&text).unwrap_or_else(|_| panic!("the room said {text:?}"))
}

/// Sam's lend `name`, key `sam`, as the room names it, served at `at`.
fn sams(sam: u8, name: &str, served: Value) -> Value {
    json!({ "person": "sam", "key": actor_of(&signing(sam)), "name": name, "served": served })
}

/// `room` without each lend's words, which [`said`] reads.
fn bare(room: &Value) -> Value {
    let mut room = room.clone();
    for l in room.as_array_mut().unwrap() {
        l.as_object_mut().unwrap().remove("said");
    }
    room
}

/// Where the room serves Sam's lend `name` of key `sam` on this host.
fn served_at(sam: u8, name: &str) -> SocketAddr {
    let label = Label::of(signing(sam).verifying_key().as_bytes(), name);
    SocketAddr::new(here().into(), room::port_of(&label))
}

/// Byte `i` of the canary's 64 MiB.
fn byte(i: usize) -> u8 {
    ((i * 7 + (i >> 10)) & 255) as u8
}

/// Whether anything listens at `addr`: a connection taken, or refused.
async fn listens(addr: SocketAddr) -> bool {
    matches!(
        tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(addr)).await,
        Ok(Ok(_))
    )
}

/// Wait until `addr` listens, or does not, as `wanted`.
async fn until_listens(addr: SocketAddr, wanted: bool) {
    let waited = tokio::time::timeout(WAIT, async {
        while listens(addr).await != wanted {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(waited.is_ok(), "{addr} listening is not {wanted}");
}

/// A raw request's reply head, read from `sock` to its blank line.
async fn head_of(sock: &mut TcpStream) -> String {
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        let mut b = [0u8; 1];
        let n = tokio::time::timeout(WAIT, sock.read(&mut b))
            .await
            .expect("an answer")
            .unwrap();
        assert_eq!(n, 1, "closed: {:?}", String::from_utf8_lossy(&head));
        head.push(b[0]);
    }
    String::from_utf8(head).unwrap()
}

/// **A lend turned on is served to this network alone, until it is turned
/// off** (the u6-1 row; DESIGN §3, "The room", call 3). Alex's room, asked
/// for `app` by name as `svrngs room app` asks it, serves Sam's canary at
/// `http://<this host's private address>:<its port>/` and says so at
/// `/v1/room`; a client there gets the lend's own root (`/abs.js`, asked
/// from the root), 16 MiB by Range sha-exact, 64 MiB answered as it
/// streams (its first part long before its last), 16 MiB uploaded as it
/// streams (the canary takes the request before its tail is sent)
/// sha-exact with its length told, the WebSocket's echo, and the cookie
/// sign-in read back; Sam's canary is asked as Alex, Sam's secret
/// injected. Loopback at the same port finds nothing, `other`, not named,
/// is not served, and a name nobody lends turns nothing on. Turned off as
/// the home page turns it off, its address refuses and the room is empty.
/// Planted, each watched failing: bound to every interface
/// (`Ipv4Addr::UNSPECIFIED` in `room::listen`), a lend not named served
/// (the names' filter dropped), still served once off (its listener kept).
#[tokio::test(flavor = "multi_thread")]
async fn a_lend_turned_on_is_served_to_this_network_until_it_is_turned_off() {
    const SAM: u8 = 62;
    let at = served_at(SAM, "app");
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let alex = alex(alex_dir.path(), SAM).await;
    let (sam, _app, app_port) = sam(SAM, sam_dir.path(), &alex).await;
    let (_other, other_port) = canary("other").await;
    lends(sam_dir.path(), "other", other_port);

    let (status, nothing) = asked(&alex, json!({ "names": ["nothing"] })).await;
    assert_eq!(status, 409, "{nothing}");
    assert_eq!(
        nothing["error"],
        "nobody in your groups lends nothing to this computer now"
    );
    let (status, turned) = asked(&alex, json!({ "names": ["app"] })).await;
    assert_eq!(status, 200, "{turned}");
    let want = json!([sams(SAM, "app", json!({ "at": at.to_string() }))]);
    assert_eq!(bare(&turned), want);
    assert_eq!(
        turned[0]["said"],
        format!("Open to this WiFi: type http://{at}/ into an app or a browser on a device here, such as your TV, a phone or another computer. Anyone on this WiFi can open it there as you.")
    );
    assert_eq!(bare(&on(&alex).await), want);
    let loopback = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), at.port());
    assert!(
        !listens(loopback).await,
        "served on loopback too: {loopback}"
    );
    let unnamed = served_at(SAM, "other");
    assert!(
        !listens(unnamed).await,
        "a lend not named is served: {unnamed}"
    );

    let http = http();
    let base = format!("http://{at}");
    let get = |path: &str| http.get(format!("{base}{path}"));
    let page = get("/").send().await.unwrap().text().await.unwrap();
    assert!(page.contains(r#"src="/abs.js""#), "{page}");
    let abs = get("/abs.js").send().await.unwrap();
    assert_eq!(abs.status(), 200);
    assert_eq!(abs.text().await.unwrap(), "window.__abs = 'ran';");

    let (from, to) = (1 << 20, (17 << 20) - 1);
    let ranged = get("/big.bin")
        .header("range", format!("bytes={from}-{to}"))
        .send()
        .await
        .unwrap();
    assert_eq!(ranged.status(), 206);
    let got = Sha256::digest(ranged.bytes().await.unwrap());
    let want = Sha256::digest((from..=to).map(byte).collect::<Vec<_>>());
    assert_eq!(hex::encode(got), hex::encode(want), "16 MiB by Range");

    let began = std::time::Instant::now();
    let mut whole = get("/big.wav").send().await.unwrap();
    let (mut first, mut n) = (None, 0);
    while let Some(part) = whole.chunk().await.unwrap() {
        first.get_or_insert(began.elapsed());
        n += part.len();
    }
    let (first, last) = (first.unwrap(), began.elapsed());
    assert_eq!(n, 64 << 20);
    assert!(
        first * 4 < last,
        "the answer came whole, not as it streams: first part at {first:?}, last at {last:?}"
    );
    println!("64 MiB answered: first part at {first:?}, last at {last:?}");

    let body: Vec<u8> = (0..16 << 20).map(byte).collect();
    let mut sock = TcpStream::connect(at).await.unwrap();
    sock.write_all(
        format!(
            "POST /upload HTTP/1.1\r\nHost: {at}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .as_bytes(),
    )
    .await
    .unwrap();
    sock.write_all(&body[..1 << 20]).await.unwrap();
    let seen = format!("http://127.0.0.1:{app_port}/seen");
    tokio::time::timeout(WAIT, async {
        loop {
            let asked: Vec<Value> = http.get(&seen).send().await.unwrap().json().await.unwrap();
            if asked.iter().any(|a| a["path"] == "/upload") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("the canary took no upload before its tail was sent: a door holds the body");
    sock.write_all(&body[1 << 20..]).await.unwrap();
    let head = head_of(&mut sock).await;
    let mut text = String::new();
    sock.read_to_string(&mut text).await.unwrap();
    assert!(head.starts_with("HTTP/1.1 200"), "{head}{text}");
    let told = format!(
        "{} {} {}",
        body.len(),
        hex::encode(Sha256::digest(&body)),
        body.len()
    );
    assert!(text.contains(&told), "{text}");

    let mut ws = TcpStream::connect(at).await.unwrap();
    ws.write_all(format!("GET /ws HTTP/1.1\r\nHost: {at}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let head = head_of(&mut ws).await;
    assert!(head.starts_with("HTTP/1.1 101"), "{head}");
    // A text frame, masked by zeros, so its bytes are the word's.
    ws.write_all(&[0x81, 0x84, 0, 0, 0, 0, b'p', b'i', b'n', b'g'])
        .await
        .unwrap();
    let mut echo = [0u8; 6];
    tokio::time::timeout(WAIT, ws.read_exact(&mut echo))
        .await
        .expect("the echo")
        .unwrap();
    assert_eq!(&echo, &[0x81, 4, b'p', b'i', b'n', b'g']);

    let signed = http.post(format!("{base}/signin")).send().await.unwrap();
    assert_eq!(signed.status(), 200);
    let cookie: Vec<String> = signed
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|c| c.to_str().unwrap().split(';').next().unwrap().to_string())
        .collect();
    assert_eq!(cookie.len(), 2, "{cookie:?}");
    let me: Value = get("/me")
        .header("cookie", cookie.join("; "))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        me["cookie"],
        cookie.join("; "),
        "the cookie sign-in, read back"
    );

    let asked_of: Vec<Value> = http.get(&seen).send().await.unwrap().json().await.unwrap();
    assert!(asked_of.len() >= 7, "{asked_of:?}");
    // The test's own asks of `/seen`, straight at the canary, apart.
    for a in asked_of.iter().filter(|a| a["path"] != "/seen") {
        assert_eq!(a["person"], "alex", "Sam's door names Alex's person: {a}");
        assert_eq!(a["key"], actor_of(&signing(ALEX)), "{a}");
        assert_eq!(a["secret"], true, "{a}");
    }

    let off = json!({ "off": { "key": actor_of(&signing(SAM)), "name": "app" } });
    let (status, room) = asked(&alex, off).await;
    assert_eq!((status, room), (200, json!([])));
    until_listens(at, false).await;
    sam.endpoint().close().await;
    alex.endpoint().close().await;
}

/// **A lend on while its lender is away keeps its address, answers at
/// once, and serves once they answer** (the report, 2: `room` exited when
/// a named lend was not offered right now, which under KeepAlive would be
/// a restart loop). Alex turns Sam's `app` on, as the home page's button
/// does, while Sam's computer is off: the room binds its address by the
/// lend's label alone and says it, and a request there is answered 502,
/// "did not answer", within the dial's cap (`dial::DIAL_WITHIN`, 4 s), not
/// held. Sam's computer starts; the same address serves the canary's root.
/// Planted, watched failing: the room as e01c070a left it, which had no
/// `/v1/room` and asked the lends live.
#[tokio::test(flavor = "multi_thread")]
async fn a_lend_on_while_its_lender_is_away_answers_at_once_then_serves() {
    const SAM: u8 = 63;
    let at = served_at(SAM, "app");
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let alex = alex(alex_dir.path(), SAM).await;
    let on_app = json!({ "on": { "key": actor_of(&signing(SAM)), "name": "app" } });
    let (status, turned) = asked(&alex, on_app).await;
    assert_eq!(status, 200, "{turned}");
    assert_eq!(
        bare(&turned),
        json!([sams(SAM, "app", json!({ "at": at.to_string() }))])
    );
    let http = http();
    let began = std::time::Instant::now();
    let away = http.get(format!("http://{at}/")).send().await.unwrap();
    let took = began.elapsed();
    println!("Sam away: answered {} in {took:?}", away.status());
    assert_eq!(away.status(), 502);
    assert_eq!(away.headers()["x-svrngs-refused"], "away");
    assert!(took < Duration::from_secs(6), "held {took:?}, not answered");

    let (sam, _app, _) = sam(SAM, sam_dir.path(), &alex).await;
    let page = http.get(format!("http://{at}/")).send().await.unwrap();
    assert_eq!(page.status(), 200);
    assert!(page.text().await.unwrap().contains(r#"src="/abs.js""#));
    sam.endpoint().close().await;
    alex.endpoint().close().await;
}

/// **On another network the room stays closed, and says why** (the
/// operator: the room serves only on the network where it was turned on,
/// told apart by its router's hardware address). Alex's room keeps Sam's
/// `app` as turned on at a network whose router is 02:00:5e:00:53:01, a
/// documentation address no router here has; his computer starts: nothing
/// listens at the lend's address, and `/v1/room` says it is off, in the
/// words the home page and `svrngs room` say. Planted, watched failing:
/// the room as e01c070a left it, with no `/v1/room` and nothing kept.
#[tokio::test(flavor = "multi_thread")]
async fn a_lend_on_at_another_network_is_not_served_and_says_why() {
    const SAM: u8 = 65;
    let at = served_at(SAM, "app");
    let alex_dir = tempfile::tempdir().unwrap();
    let kept =
        json!([{ "key": actor_of(&signing(SAM)), "name": "app", "network": "02:00:5e:00:53:01" }]);
    std::fs::write(alex_dir.path().join("room.json"), kept.to_string()).unwrap();
    let alex = alex(alex_dir.path(), SAM).await;
    let room = tokio::time::timeout(WAIT, async {
        loop {
            let room = on(&alex).await;
            if room[0]["person"] == "sam" {
                break room;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("the room says Sam's lend");
    assert_eq!(bare(&room), json!([sams(SAM, "app", json!("elsewhere"))]));
    assert_eq!(
        room[0]["said"],
        "Closed for now: this computer is on another WiFi than the one where you turned it on. It opens again when you are there again."
    );
    assert!(!listens(at).await, "served on another network: {at}");
    alex.endpoint().close().await;
}

/// **A room that cannot be read is said, never read as empty** (AGENTS.md
/// 6). Alex's room.json holds what is not a list of lends; his computer
/// starts: `/v1/room` answers that what is open to this WiFi cannot be read,
/// and why, and serves nothing from a guess. Turning Sam's `app` on again
/// writes the room over, and it is served and kept. A room never turned
/// on, with no room.json, is empty, which is true. Planted, watched
/// failing: room.rs as c0b0bf59 left it, which read it as empty and said
/// so at debug alone.
#[tokio::test(flavor = "multi_thread")]
async fn a_room_that_cannot_be_read_is_said_until_a_lend_is_turned_on_again() {
    const SAM: u8 = 66;
    let at = served_at(SAM, "app");
    let (alex_dir, never) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let kept = alex_dir.path().join("room.json");
    std::fs::write(&kept, r#"{"on": "a TV"}"#).unwrap();
    let alex = alex(alex_dir.path(), SAM).await;
    let unread = http()
        .get(format!("http://{}/v1/room", alex.api_addr()))
        .send()
        .await
        .unwrap();
    assert_eq!(unread.status(), 500);
    let said: Value = unread.json().await.unwrap();
    let words = said["error"].as_str().unwrap_or_default();
    assert!(
        words.starts_with("This computer cannot read what you opened to this WiFi (")
            && words.ends_with("), so nothing is open to it now. Open a lend to this WiFi again, and it starts over from that one."),
        "{said}"
    );
    assert!(!listens(at).await, "served from a guess: {at}");

    let on_app = json!({ "on": { "key": actor_of(&signing(SAM)), "name": "app" } });
    let (status, turned) = asked(&alex, on_app).await;
    assert_eq!(status, 200, "{turned}");
    assert_eq!(
        bare(&turned),
        json!([sams(SAM, "app", json!({ "at": at.to_string() }))])
    );
    assert_eq!(bare(&on(&alex).await), bare(&turned));
    let written: Value = serde_json::from_slice(&std::fs::read(&kept).unwrap()).unwrap();
    assert_eq!(written[0]["name"], "app", "{written}");

    let empty = start(67, never.path()).await;
    assert_eq!(on(&empty).await, json!([]), "a room never turned on");
    empty.endpoint().close().await;
    alex.endpoint().close().await;
}
