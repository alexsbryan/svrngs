//! The room door (DESIGN §3, "The room", call 3; the u6-1 row): Alex's
//! computer serves the lends Alex names to its own network while the
//! command runs. Two nodes in one process, no relay, each dialled on
//! 127.0.0.1 (#33's fix): Alex founds a group and lets Sam in, and Sam
//! lends the conformance canary (`apps/web-conformance`, u5-2) twice, as
//! `app` and `other`. Alex's room door serves `app` alone on this host's
//! private address, and a client there — not on loopback — asks it what
//! the canary asks of a lend on a computer. The picker's planted lists and
//! a caller off the network are judged in `src/room_tests.rs`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::Stdio;
use std::time::Duration;

use common::loopback_sockets;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use svrngs_core::pass::Label;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend::{self, LendName};
use svrngs_node::node::Node;
use svrngs_node::room::{self, RoomError};
use svrngs_node::{actor_of, declared, Person, RailAct, Roster, SigningKey};

const NS: &str = "house";
const SAMS: &str = "Bearer sams-secret-61";
const ALEX: u8 = 61;
const SAM: u8 = 62;
const WAIT: Duration = Duration::from_secs(30);

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn no_seed(_: &std::path::Path) -> Result<Option<String>, String> {
    Ok(None)
}

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
fn lends(dir: &std::path::Path, name: &str, port: u16) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let body = serde_json::json!({ "port": port, "kind": "app", "limit": 3 });
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
    let secrets = lend::secret_dir(dir, &LendName::parse(name).unwrap());
    declared::write_declared_in(&secrets, "authorization", SAMS).unwrap();
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

/// **The room door serves the lends named to this network alone, while it
/// runs** (the u6-1 row). Alex's room door, asked for `app`, serves Sam's
/// canary at `http://<this host's private address>:<its port>/`, and
/// `/v1/lends` answers that address as `app`'s `room`; a client there
/// gets the lend's own root (`/abs.js`, asked from the root), 16 MiB by
/// Range sha-exact, 64 MiB answered as it streams (its first part long
/// before its last), 16 MiB uploaded as it streams (the canary takes the
/// request before its tail is sent) sha-exact with its length told, the
/// WebSocket's echo, and the cookie sign-in read back; Sam's canary is
/// asked as Alex, Sam's secret injected. Loopback at the same port finds
/// nothing, `other`, not named, is not served, and a name nobody lends
/// opens nothing. Once the room is dropped its address refuses, and
/// `/v1/lends` answers no room. Planted, each watched failing: bound to
/// every interface (`Ipv4Addr::UNSPECIFIED` in `room::open`), a lend not
/// named served (the names' filter dropped), still served after stop (no
/// abort on drop).
#[tokio::test(flavor = "multi_thread")]
async fn the_room_door_serves_the_lends_named_to_this_network_while_it_runs() {
    let ip = match room::route() {
        Some(IpAddr::V4(v4)) if v4.is_private() => v4,
        other => panic!(
            "could not judge: this host's route leaves by {other:?}, no private address to serve a room on"
        ),
    };
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let alex = start(ALEX, alex_dir.path()).await;
    let sam = start(SAM, sam_dir.path()).await;
    let seed = Roster::new(
        [(
            Person::parse("alex").unwrap(),
            vec![actor_of(&signing(ALEX))],
        )]
        .into_iter()
        .collect(),
    );
    for node in [&alex, &sam] {
        node.rail().set_seed(NS, &seed).unwrap();
    }
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("sam").unwrap(),
            key: actor_of(&signing(SAM)),
            invite: None,
        },
        on_behalf_of: None,
    };
    alex.rail().commit(NS, &admit).unwrap().unwrap();
    alex.transport()
        .add_known_peer(sam.endpoint().id(), loopback_sockets(sam.endpoint()));
    let (_app, app_port) = canary("sam").await;
    let (_other, other_port) = canary("other").await;
    lends(sam_dir.path(), "app", app_port);
    lends(sam_dir.path(), "other", other_port);
    let label = |name: &str| Label::of(sam.endpoint().id().as_bytes(), name);

    let nothing = room::open(&alex, ip, &[LendName::parse("nothing").unwrap()]).await;
    assert!(
        matches!(nothing, Err(RoomError::NotLent(_))),
        "{:?}",
        nothing.map(|r| r.served.clone())
    );
    let opened = room::open(&alex, ip, &[LendName::parse("app").unwrap()])
        .await
        .expect("the room door opens");
    let at = SocketAddr::new(ip.into(), room::port_of(&label("app")));
    assert_eq!(
        opened
            .served
            .iter()
            .map(|(l, a)| (l.person.as_str(), l.offered.name.as_str(), *a))
            .collect::<Vec<_>>(),
        vec![("sam", "app", at)]
    );
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let rooms = || async {
        let listed: Value = http
            .get(format!(
                "http://{}/v1/lends?namespace={NS}",
                alex.api_addr()
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        listed["lent"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| (l["name"].as_str().unwrap().to_string(), l["room"].clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        rooms().await,
        vec![
            ("app".to_string(), Value::from(format!("http://{at}/"))),
            ("other".to_string(), Value::Null)
        ]
    );
    let loopback = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), at.port());
    assert!(
        !listens(loopback).await,
        "served on loopback too: {loopback}"
    );
    let unnamed = SocketAddr::new(ip.into(), room::port_of(&label("other")));
    assert!(
        !listens(unnamed).await,
        "a lend not named is served: {unnamed}"
    );

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

    let asked: Vec<Value> = http.get(&seen).send().await.unwrap().json().await.unwrap();
    assert!(asked.len() >= 7, "{asked:?}");
    // The test's own asks of `/seen`, straight at the canary, apart.
    for a in asked.iter().filter(|a| a["path"] != "/seen") {
        assert_eq!(a["person"], "alex", "Sam's door names Alex's person: {a}");
        assert_eq!(a["key"], actor_of(&signing(ALEX)), "{a}");
        assert_eq!(a["secret"], true, "{a}");
    }

    drop(opened);
    let stopped = tokio::time::timeout(WAIT, async {
        while listens(at).await {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(stopped.is_ok(), "still served after the room stopped: {at}");
    assert_eq!(
        rooms().await,
        vec![
            ("app".to_string(), Value::Null),
            ("other".to_string(), Value::Null)
        ]
    );
    sam.endpoint().close().await;
    alex.endpoint().close().await;
}
