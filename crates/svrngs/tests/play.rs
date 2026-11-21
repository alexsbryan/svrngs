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

#[path = "common/made.rs"]
mod made;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
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
    let made = made::new_group(&sam, name("house"), name("Sam"), None, &[]);
    let (seed, printed) = made.await.expect("Sam makes house");
    let alex = start(2, alex).await;
    knows(&alex, &sam);
    knows(&sam, &alex);
    let link = Link::parse(&printed.client).unwrap();
    let joined = made::join_group(&alex, &link, &name("Alex")).await;
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
/// `svrngs share <port> --udp` or `--tcp` declares it, lent to the groups
/// Sam holds.
fn lend(sam: &Path, lend: &str, port: u16, raw: &str) {
    let lends = svrngs_node::lend::lends_dir(sam);
    std::fs::create_dir_all(&lends).unwrap();
    let groups: Vec<_> = svrngs::group::groups(sam).iter().map(|s| s.id()).collect();
    let body = json!({ "port": port, "raw": raw, "kind": "echo", "limit": 3, "groups": groups });
    std::fs::write(
        lends.join(format!("{lend}.json")),
        svrngs_node::node::approving(&body),
    )
    .unwrap();
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

/// An HTTP program on Sam's computer: each request's head kept, and
/// answered `ok`.
async fn http_ok() -> (u16, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let heads: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();
    let kept = heads.clone();
    tokio::spawn(async move {
        while let Ok((mut c, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut byte = [0; 1];
                while !head.ends_with(b"\r\n\r\n") && c.read_exact(&mut byte).await.is_ok() {
                    head.push(byte[0]);
                }
                // svrngs's own look at whether its program answers sends
                // nothing; only a request's head is kept.
                if !head.is_empty() {
                    let head = String::from_utf8_lossy(&head).into_owned();
                    kept.lock().unwrap().push(head);
                }
                let ok = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
                let _ = c.write_all(ok.as_bytes()).await;
            });
        }
    });
    (port, heads)
}

/// Sam lends `port` as `name`, a lend at an address, to the groups Sam
/// holds.
fn lend_at_an_address(sam: &Path, name: &str, port: u16) {
    let lends = svrngs_node::lend::lends_dir(sam);
    std::fs::create_dir_all(&lends).unwrap();
    let groups: Vec<_> = svrngs::group::groups(sam).iter().map(|s| s.id()).collect();
    let body = json!({ "port": port, "kind": "echo", "limit": 3, "groups": groups });
    let declared = svrngs_node::node::approving(&body);
    std::fs::write(lends.join(format!("{name}.json")), declared).unwrap();
}

/// Sam's `lend` at an address, with Play on for it on Alex's computer: the
/// address Alex's home page copies, and the heads Sam's program was sent.
async fn played_at_an_address(
    lend: &str,
) -> (
    String,
    std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    [Node; 2],
    [tempfile::TempDir; 2],
) {
    let (sam_dir, alex_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (sam, alex, _) = house(sam_dir.path(), alex_dir.path()).await;
    let (port, heads) = http_ok().await;
    lend_at_an_address(sam_dir.path(), lend, port);
    let (status, on) = play(&alex, turn("on", lend)).await;
    assert_eq!(status, 200, "{on}");
    let address = on[0]["at"].as_str().unwrap().to_string();
    let label = Label::of(signing(1).verifying_key().as_bytes(), lend);
    let api = alex.api_addr().port();
    assert_eq!(address, format!("http://{label}.localhost:{api}/"), "{on}");
    assert_eq!(on[0]["said"], format!("Connect to {address}."), "{on}");
    (address, heads, [sam, alex], [sam_dir, alex_dir])
}

/// **A lend's address answers a program here with Play on, on either
/// loopback**: Sam lends a program at an address; with Play on, a program
/// on Alex's computer, sending any text as its key, is answered at the
/// address on 127.0.0.1 and on ::1, the first address a Mac and Linux
/// give `<label>.localhost`, its socket's owner read by this system's own reader
/// (libproc on a Mac, sock_diag on Linux); no key reaches Sam's program, in
/// any header a model's client sends one in (`Authorization`, `X-Api-Key`,
/// `Api-Key`, `X-Goog-Api-Key`), whatever its case. Planted, watched
/// failing: the API listening on 127.0.0.1 alone (::1 refused its
/// connection); Play taking off `Authorization` alone (Sam's program was
/// sent `X-Api-Key`).
#[tokio::test(flavor = "multi_thread")]
async fn play_lets_a_program_in_at_a_lend_s_address_on_either_loopback() {
    let (address, heads, [_sam, alex], _dirs) = played_at_an_address("chat").await;
    let host = address
        .trim_start_matches("http://")
        .split(':')
        .next()
        .unwrap()
        .to_string();
    let port = alex.api_addr().port();
    for at in [
        SocketAddr::from((Ipv4Addr::LOCALHOST, port)),
        (std::net::Ipv6Addr::LOCALHOST, port).into(),
    ] {
        let http = reqwest::Client::builder()
            .no_proxy()
            .resolve(&host, at)
            .build()
            .unwrap();
        let asked = http
            .get(format!("{address}v1/models"))
            .bearer_auth("any-text-at-all")
            .header("X-Api-Key", "an-anthropic-key")
            .header("api-key", "an-azure-key")
            .header("X-GOOG-API-KEY", "a-google-key");
        let answer = asked.send().await.unwrap_or_else(|e| panic!("{at}: {e}"));
        let status = answer.status().as_u16();
        assert_eq!(
            (status, answer.text().await.unwrap()),
            (200, "ok".into()),
            "at {at}"
        );
    }
    let heads = heads.lock().unwrap();
    assert_eq!(heads.len(), 2, "{heads:?}");
    for key in [
        "any-text-at-all",
        "an-anthropic-key",
        "an-azure-key",
        "a-google-key",
    ] {
        assert!(!heads.iter().any(|h| h.contains(key)), "{key}: {heads:?}");
    }
}

/// **Play lets a program reach the lend, and none of svrngs's own
/// addresses at it**: with Play on, a program here asking the lend's
/// address for what svrngs answers there in the lend's place — the lends
/// its page reads (`/svrngs-reads`, whose read addresses outlive Play),
/// its stop, its opener, its reads through svrngs and by a read's address
/// — is refused in words that say so, and Sam's program is sent nothing;
/// the lend's own path, beside them, answers. Planted, watched failing:
/// V1.13.3's candidate answered `/svrngs-reads` 200 with the lends it reads.
#[tokio::test(flavor = "multi_thread")]
async fn play_reaches_the_lend_and_none_of_svrngs_s_own_addresses_at_it() {
    let (address, heads, [_sam, alex], _dirs) = played_at_an_address("own").await;
    let host = address.trim_start_matches("http://").split(':').next();
    let host = host.unwrap().to_string();
    let http = reqwest::Client::builder()
        .no_proxy()
        .resolve(&host, alex.api_addr())
        .build()
        .unwrap();
    let label = host.trim_end_matches(".localhost");
    for (path, post) in [
        ("svrngs-reads".to_string(), false),
        (format!("svrngs-reads/{label}/svrngs-stop"), true),
        ("svrngs-to?lender=a&lend=b".into(), false),
        (format!("svrngs-lend/{label}/v1/models"), false),
        ("svrngs-as/a-capability/v1/models".into(), false),
    ] {
        let url = format!("{address}{path}");
        let asked = if post {
            http.post(&url)
        } else {
            http.get(&url)
        };
        let answer = asked.send().await.unwrap();
        let (status, said) = (answer.status().as_u16(), answer.text().await.unwrap());
        assert_eq!(status, 403, "{path}: {said}");
        assert!(said.contains("svrngs’s own address"), "{path}: {said}");
    }
    assert!(
        heads.lock().unwrap().is_empty(),
        "Sam's program was sent one"
    );
    let models = http
        .get(format!("{address}v1/models"))
        .send()
        .await
        .unwrap();
    assert_eq!(models.text().await.unwrap(), "ok", "the lend's own path");
}

/// **curl and Node reach a lend's address with Play on**, each resolving
/// `<label>.localhost` as it does on this computer: getaddrinfo names ::1
/// first on a Mac and on Linux, and curl names both itself. By hand: the
/// programs on PATH.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "by hand: curl and node on PATH"]
async fn curl_and_node_reach_a_lend_s_address_with_play_on() {
    let (address, _heads, _nodes, _dirs) = played_at_an_address("talk").await;
    let url = format!("{address}v1/models");
    let curl = tokio::process::Command::new("curl")
        .args([
            "-sS",
            "--noproxy",
            "*",
            "-H",
            "Authorization: Bearer none",
            &url,
        ])
        .output();
    let curl = curl.await.expect("curl runs");
    let said = String::from_utf8_lossy(&curl.stdout);
    assert_eq!(
        said,
        "ok",
        "curl: {}",
        String::from_utf8_lossy(&curl.stderr)
    );
    let script = "fetch(process.argv[1], { headers: { authorization: 'Bearer none' } }).then(r => r.text()).then(t => process.stdout.write(t))";
    let node = tokio::process::Command::new("node")
        .args(["-e", script, &url])
        .output();
    let node = node.await.expect("node runs");
    let said = String::from_utf8_lossy(&node.stdout);
    assert_eq!(
        said,
        "ok",
        "node: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    eprintln!("RESULT: curl and node reached {url}");
}

/// **Another account's program is refused at a lend's address with Play
/// on**, on a real socket of another uid: a uid of this account's own
/// subordinate range (/etc/subuid), as `unshare --map-auto` gives it
/// without root, read by sock_diag. By hand on Linux: it needs
/// newuidmap and a range in /etc/subuid.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "by hand on Linux: unshare --map-auto and a range in /etc/subuid"]
async fn another_account_s_program_is_refused_at_a_lend_s_address() {
    let (address, heads, [_sam, alex], _dirs) = played_at_an_address("fond").await;
    let host = address
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string();
    let script = "import sys, urllib.request as u\nr = u.Request('http://127.0.0.1:' + sys.argv[1] + '/v1/models', headers={'Host': sys.argv[2], 'Authorization': 'Bearer none'})\ntry:\n    print(u.urlopen(r).status, flush=True)\nexcept Exception as e:\n    print(e.code, e.read().decode(), flush=True)";
    let port = alex.api_addr().port().to_string();
    let argv = [
        "--map-auto",
        "--map-root-user",
        "setpriv",
        "--reuid=1",
        "--regid=1",
        "--clear-groups",
        "python3",
        "-I",
        "-c",
        script,
        &port,
        &host,
    ];
    let theirs = tokio::process::Command::new("unshare").args(argv).output();
    let theirs = theirs.await.expect("unshare runs");
    let said = String::from_utf8_lossy(&theirs.stdout);
    eprintln!(
        "another account's program: {said} {}",
        String::from_utf8_lossy(&theirs.stderr)
    );
    assert!(said.starts_with("403"), "{said}");
    assert!(said.contains("this came from another account"), "{said}");
    assert!(
        heads.lock().unwrap().is_empty(),
        "another account's program reached Sam's"
    );
    let http = reqwest::Client::builder()
        .no_proxy()
        .resolve(host.split(':').next().unwrap(), alex.api_addr())
        .build()
        .unwrap();
    let ours = http
        .get(format!("{address}v1/models"))
        .send()
        .await
        .unwrap();
    assert_eq!(ours.status(), 200, "this account's program, beside it");
    eprintln!("RESULT: another account's program refused at {address}, this account's answered");
}
