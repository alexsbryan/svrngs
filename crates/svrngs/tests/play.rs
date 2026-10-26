// SPDX-License-Identifier: AGPL-3.0-or-later
//! Play (give-play; svrn-docs rings/current/GIVE.md §3 scene 2, §6): Sam
//! lends a UDP port and a TCP port, and Alex's computer, asked at
//! `/v1/play` as the home page's Play and Stop ask it, opens each on
//! `127.0.0.1` at the lend's own port (`room::port_of`, by its label), so
//! a program there reaches Sam's as if it ran here. Two nodes in one
//! process, no relay, each dialled on 127.0.0.1. The port is the same once
//! Alex's svrngs starts again, so a program's saved address keeps working.
//! What is refused, and why: while Play is off nothing listens at that
//! port, since a raw port carries no token and anything on Alex's computer
//! reaching it reaches Sam's lend as Alex; and once Alex stops standing in
//! the group, the listener closes, so a person removed keeps no way in.
//! Each test lends under names of its own, so each one's port is its own.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs::cli;
use svrngs::group::{Name, Seed};
use svrngs_core::pass::Label;
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, RelayConfig};
use svrngs_node::link::Link;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, SigningKey};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::time::timeout;

const WAIT: Duration = Duration::from_secs(30);

fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: true,
    }
}

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

async fn start(seed: u8, dir: &Path) -> Node {
    Node::start(
        dir.to_path_buf(),
        signing(seed),
        &no_relay(),
        svrngs::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// Where `node` is reached on 127.0.0.1.
fn at(node: &Node) -> EndpointAddr {
    let mut addr = EndpointAddr::new(node.endpoint().id());
    for mut a in node.endpoint().bound_sockets() {
        if a.ip().is_unspecified() {
            a.set_ip(Ipv4Addr::LOCALHOST.into());
        }
        addr = addr.with_ip_addr(a);
    }
    addr
}

/// `node` told where `other` listens, as an address lookup would.
fn knows(node: &Node, other: &Node) {
    let book = MemoryLookup::new();
    book.add_endpoint_info(at(other));
    node.endpoint().address_lookup().unwrap().add(book);
}

/// Sam (key 1) makes house and lets Alex (key 2) in by Sam's link.
async fn house(sam: &Path, alex: &Path) -> (Node, Node, Seed) {
    let sam = start(1, sam).await;
    let relay = no_relay();
    let made = cli::new_group(&sam, name("house"), name("Sam"), None, None, &relay);
    let (seed, printed) = made.await.expect("Sam makes house");
    let alex = start(2, alex).await;
    knows(&alex, &sam);
    knows(&sam, &alex);
    let link = Link::parse(&printed.client).unwrap();
    let joined = cli::join_group(&alex, &signing(2), &link, &name("Alex")).await;
    joined.expect("Alex is let in");
    (sam, alex, seed)
}

/// A UDP program on Sam's computer that answers each packet with itself.
async fn udp_echo() -> u16 {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut buf = vec![0; 65_535];
        while let Ok((n, from)) = socket.recv_from(&mut buf).await {
            let _ = socket.send_to(&buf[..n], from).await;
        }
    });
    port
}

/// A TCP program on Sam's computer that answers each connection with what
/// it is sent.
async fn tcp_echo() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut c, _)) = listener.accept().await {
            tokio::spawn(async move {
                let (mut from, mut to) = c.split();
                let _ = tokio::io::copy(&mut from, &mut to).await;
            });
        }
    });
    port
}

/// Sam lends `port` as `name`, a raw port of the way `raw` says, as
/// `svrngs share <port> --udp` or `--tcp` declares it.
fn lend(sam: &Path, lend: &str, port: u16, raw: &str) {
    let lends = svrngs_node::lend::lends_dir(sam);
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "port": port, "raw": raw, "kind": "echo", "limit": 3 });
    std::fs::write(lends.join(format!("{lend}.json")), body.to_string()).unwrap();
}

/// Where Alex's computer opens Sam's `lend`: the room's port for its label.
fn kept(lend: &str) -> SocketAddr {
    let label = Label::of(signing(1).verifying_key().as_bytes(), lend);
    (Ipv4Addr::LOCALHOST, svrngs_node::room::port_of(&label)).into()
}

/// `/v1/play` on `node`, as the home page asks it: `ask` posted, or the
/// Plays read; its status and its answer.
async fn play(node: &Node, ask: Option<Value>) -> (u16, Value) {
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let url = format!("http://{}/v1/play", node.api_addr());
    let asked = match &ask {
        Some(ask) => http
            .post(url)
            .header("content-type", "application/json")
            .body(ask.to_string()),
        None => http.get(url),
    };
    let answer = asked.bearer_auth(node.token().trim()).send().await.unwrap();
    let status = answer.status().as_u16();
    let body = answer.text().await.unwrap();
    (
        status,
        serde_json::from_str(&body).unwrap_or(Value::String(body)),
    )
}

/// Play turned `on` (or `off`) for Sam's `lend` on Alex's computer.
fn turn(on: &str, lend: &str) -> Option<Value> {
    Some(json!({ on: { "key": actor_of(&signing(1)), "name": lend } }))
}

/// A packet sent to `to` from a program on Alex's computer, and what comes
/// back within five seconds.
async fn udp_asks(to: SocketAddr, said: &[u8]) -> Option<Vec<u8>> {
    let program = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut buf = vec![0; 65_535];
    for _ in 0..10 {
        program.send_to(said, to).await.ok()?;
        let heard = timeout(Duration::from_millis(500), program.recv_from(&mut buf));
        if let Ok(Ok((n, _))) = heard.await {
            return Some(buf[..n].to_vec());
        }
    }
    None
}

/// What a TCP program on Alex's computer hears back from `to`.
async fn tcp_asks(to: SocketAddr, said: &[u8]) -> Option<Vec<u8>> {
    let mut program = TcpStream::connect(to).await.ok()?;
    program.write_all(said).await.ok()?;
    let mut heard = vec![0; said.len()];
    timeout(Duration::from_secs(5), program.read_exact(&mut heard))
        .await
        .ok()?
        .ok()?;
    Some(heard)
}

/// Whether nothing listens at `to`, UDP and TCP: each binds there.
async fn nothing_at(to: SocketAddr) -> bool {
    UdpSocket::bind(to).await.is_ok() && TcpListener::bind(to).await.is_ok()
}

/// Within [`WAIT`], `until` holds.
async fn within<F: std::future::Future<Output = bool>>(mut until: impl FnMut() -> F) -> bool {
    timeout(WAIT, async {
        while !until().await {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .is_ok()
}

/// **Play opens a lent port at its kept address, UDP or TCP as it is
/// lent**: before Play nothing listens there; pressed, Alex's computer
/// says where a program connects, and a program's packets and bytes reach
/// Sam's and come back. Red at HEAD: no `/v1/play`.
#[tokio::test(flavor = "multi_thread")]
async fn play_opens_a_lent_port_at_its_kept_address_udp_or_tcp() {
    let (sam_dir, alex_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (_sam, alex, _) = house(sam_dir.path(), alex_dir.path()).await;
    lend(sam_dir.path(), "pudp", udp_echo().await, "udp");
    lend(sam_dir.path(), "ptcp", tcp_echo().await, "tcp");
    let (udp, tcp) = (kept("pudp"), kept("ptcp"));
    assert!(
        nothing_at(udp).await && nothing_at(tcp).await,
        "nothing listens before Play"
    );

    let (status, on) = play(&alex, turn("on", "pudp")).await;
    assert_eq!(status, 200, "{on}");
    let said = format!("Connect to {udp}");
    assert!(on.to_string().contains(&said), "{on}");
    assert_eq!(
        udp_asks(udp, b"hello by UDP").await.as_deref(),
        Some(&b"hello by UDP"[..])
    );

    let (status, on) = play(&alex, turn("on", "ptcp")).await;
    assert_eq!(status, 200, "{on}");
    assert!(
        on.to_string().contains(&format!("Connect to {tcp}")),
        "{on}"
    );
    assert_eq!(
        tcp_asks(tcp, b"hello by TCP").await.as_deref(),
        Some(&b"hello by TCP"[..])
    );
}

/// **The port is the same after a restart**: Play stays on across Alex's
/// svrngs starting again, at the same address, before anyone presses it
/// again. Red at HEAD: no `/v1/play`.
#[tokio::test(flavor = "multi_thread")]
async fn play_stays_on_at_the_same_port_across_a_restart() {
    let (sam_dir, alex_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (sam, alex, _) = house(sam_dir.path(), alex_dir.path()).await;
    lend(sam_dir.path(), "rudp", udp_echo().await, "udp");
    let (status, on) = play(&alex, turn("on", "rudp")).await;
    assert_eq!(status, 200, "{on}");
    let to = kept("rudp");
    assert!(udp_asks(to, b"before").await.is_some(), "Play is on: {on}");

    drop(alex);
    assert!(
        within(|| nothing_at(to)).await,
        "nothing listens while svrngs is off"
    );
    // Started again from a copy of its data dir, key, record and Plays
    // alike: a node dropped in this process leaves a blocking task its
    // runtime waits on, so a start on the same dir hangs here (sampled:
    // `BlockingPool::shutdown` under `block_in_place`); the command's own
    // restarts are processes (crates/svrngs/tests/share.rs).
    let again = tempfile::tempdir().unwrap();
    let copied = std::process::Command::new("cp")
        .arg("-R")
        .arg(alex_dir.path().join("."))
        .arg(again.path())
        .status();
    assert!(copied.unwrap().success());
    let alex = start(2, again.path()).await;
    knows(&alex, &sam);
    let again = within(|| async { udp_asks(to, b"after").await.as_deref() == Some(&b"after"[..]) });
    assert!(
        again.await,
        "Play is on again at {to}: {:?}",
        play(&alex, None).await
    );
}

/// **Stop unbinds it, and nothing listens while it is off**: a raw port
/// carries no token, so a listener left on would let any program on this
/// computer reach Sam's lend as Alex. Red at HEAD: no `/v1/play`.
#[tokio::test(flavor = "multi_thread")]
async fn stop_closes_it_and_nothing_listens_while_it_is_off() {
    let (sam_dir, alex_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (_sam, alex, _) = house(sam_dir.path(), alex_dir.path()).await;
    lend(sam_dir.path(), "sudp", udp_echo().await, "udp");
    lend(sam_dir.path(), "stcp", tcp_echo().await, "tcp");
    for lend in ["sudp", "stcp"] {
        let (status, on) = play(&alex, turn("on", lend)).await;
        assert_eq!(status, 200, "{on}");
    }
    assert!(tcp_asks(kept("stcp"), b"on").await.is_some());
    for lend in ["sudp", "stcp"] {
        let (status, off) = play(&alex, turn("off", lend)).await;
        assert_eq!(status, 200, "{off}");
        assert!(!off.to_string().contains(lend), "{lend} is off: {off}");
        assert!(
            nothing_at(kept(lend)).await,
            "nothing listens at {lend}'s port once off"
        );
    }
    let (_, read) = play(&alex, None).await;
    assert_eq!(read, json!([]), "no Play on");
}

/// **A member who stops standing loses the listener**: Sam removes Alex
/// from house, and Alex's computer closes the port once it learns it, and
/// says why. Red at HEAD: no `/v1/play`.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_who_stops_standing_loses_the_listener() {
    let (sam_dir, alex_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (sam, alex, seed) = house(sam_dir.path(), alex_dir.path()).await;
    lend(sam_dir.path(), "xtcp", tcp_echo().await, "tcp");
    let (status, on) = play(&alex, turn("on", "xtcp")).await;
    assert_eq!(status, 200, "{on}");
    let to = kept("xtcp");
    assert!(tcp_asks(to, b"in").await.is_some(), "Play is on: {on}");

    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let removed = http
        .post(format!("http://{}/v1/removals", sam.api_addr()))
        .bearer_auth(sam.token().trim())
        .header("content-type", "application/json")
        .body(json!({ "group": seed.id(), "person": "Alex" }).to_string())
        .send()
        .await
        .unwrap();
    assert!(removed.status().is_success(), "{:?}", removed.text().await);
    let told = within(|| async {
        let register = alex.rail().register(&seed.id());
        register.is_ok_and(|r| r.removed.contains_key(&actor_of(&signing(2))))
    });
    assert!(told.await, "Alex's computer is told its cut");
    assert!(
        within(|| nothing_at(to)).await,
        "the listener closes once Alex stands nowhere with Sam"
    );
    let (_, read) = play(&alex, None).await;
    assert!(read.to_string().contains("no group together"), "{read}");
}
