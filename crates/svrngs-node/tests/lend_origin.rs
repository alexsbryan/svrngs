//! Every lend at an origin of its own on a computer's loopback (DESIGN §3,
//! "Lent apps", call 1; the u5-2 row): `http://<label>.localhost:<api
//! port>/`, served on the record API's listener. Alex founds a group and
//! lets Sam in; both lend, each with a secret. Two nodes in one process, no
//! relay, each listening on 127.0.0.1 alone (#33's fix), Sam reached by
//! seeded addresses. The browser halves of the same claims run in
//! `tests/conformance.rs`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[allow(dead_code)]
#[path = "door/origins.rs"]
mod origins;

use std::net::SocketAddr;
use std::time::Duration;

use common::loopback_sockets;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use svrngs_core::pass::Label;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend::{self, LendName};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, declared, Person, RailAct, Roster, SigningKey};

const NS: &str = "house";
const SAMS: &str = r#"MediaBrowser Token="sams-secret""#;
const ALEXS: &str = "Bearer alexs-secret";

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
            loopback_only: true,
        },
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// `dir`'s computer lends `name` on `port`, with `secret` as its
/// `authorization`.
fn lends(dir: &std::path::Path, name: &str, port: u16, secret: &str) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let body = serde_json::json!({ "port": port, "kind": "app", "limit": 3 });
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
    let secrets = lend::secret_dir(dir, &LendName::parse(name).unwrap());
    declared::write_declared_in(&secrets, "authorization", secret).unwrap();
}

/// No byte Alex receives holds either secret.
fn clean(bytes: &[u8]) {
    for needle in ["sams-secret", "alexs-secret"] {
        assert!(
            !bytes.windows(needle.len()).any(|w| w == needle.as_bytes()),
            "Alex received a lender's secret: {:?}",
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

fn last(heads: &std::sync::Mutex<Vec<String>>) -> String {
    heads
        .lock()
        .unwrap()
        .last()
        .cloned()
        .expect("the service was asked")
}

/// One raw request to `api` naming `host`; the reply's head, and the
/// socket for what follows when it answered 101.
async fn ask(api: SocketAddr, host: &str, rest: &str) -> (String, tokio::net::TcpStream) {
    let mut sock = tokio::net::TcpStream::connect(api).await.unwrap();
    sock.write_all(format!("{rest}Host: {host}\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let mut reply = Vec::new();
    let mut byte = [0u8; 1];
    while !reply.ends_with(b"\r\n\r\n") {
        let n = tokio::time::timeout(Duration::from_secs(20), sock.read(&mut byte))
            .await
            .expect("an answer")
            .unwrap();
        assert_eq!(
            n,
            1,
            "closed before the head ended: {:?}",
            String::from_utf8_lossy(&reply)
        );
        reply.push(byte[0]);
    }
    clean(&reply);
    (String::from_utf8_lossy(&reply).into_owned(), sock)
}

/// A whole reply's body after its head, read to the close.
async fn body(mut sock: tokio::net::TcpStream) -> String {
    let mut rest = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(20), sock.read_to_end(&mut rest)).await;
    clean(&rest);
    String::from_utf8_lossy(&rest).into_owned()
}

/// Alex's computer and Sam's: Alex founds a group and lets Sam in, and
/// Alex's knows where Sam's listens.
async fn alex_lets_sam_in(alex_dir: &std::path::Path, sam_dir: &std::path::Path) -> (Node, Node) {
    let alex = start(1, alex_dir).await;
    let sam = start(2, sam_dir).await;
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
    (alex, sam)
}

/// **Each lend at an origin of its own** (the u5-2 row). Alex's
/// `/v1/lends` answers each lend's origin, `http://<label>.localhost:<api
/// port>/`. At Sam's, Alex reaches Sam's service through the dial to Sam's
/// door, as Alex — Sam's door names Alex's key — with Sam's secret
/// injected; at Alex's own, Alex's service through Alex's door's rules,
/// Alex's secret injected and Alex counted. A WebSocket opened at Sam's
/// origin is carried through both doors and echoes. No byte Alex receives
/// holds either secret. A label naming a key that stands nowhere reaches
/// nothing. Planted: this computer's own lend sent through a dial to
/// itself (the label's lender not compared with this computer's key),
/// which no member dials, so it does not answer.
#[tokio::test]
async fn each_lend_opens_at_an_origin_of_its_own_on_this_computer() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    let (pad, pad_heads) = origins::head_origin().await;
    lends(sam_dir.path(), "pad", pad.port(), SAMS);
    let (live, live_heads) = origins::upgrade_origin().await;
    lends(sam_dir.path(), "live", live.port(), SAMS);
    let (telly, telly_heads) = origins::head_origin().await;
    lends(alex_dir.path(), "telly", telly.port(), ALEXS);

    let api = alex.api_addr();
    let port = api.port();
    let label = |node: &Node, name: &str| Label::of(node.endpoint().id().as_bytes(), name);
    let host = |l: &Label| format!("{l}.localhost:{port}");

    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let listed: serde_json::Value = http
        .get(format!("http://{api}/v1/lends?namespace={NS}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let origin_of = |name: &str| {
        listed["lent"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["name"] == name)
            .unwrap_or_else(|| panic!("{name} is listed: {listed:#}"))["origin"]
            .clone()
    };
    for (node, name) in [(&sam, "pad"), (&sam, "live"), (&alex, "telly")] {
        assert_eq!(
            origin_of(name),
            format!("http://{}/", host(&label(node, name))),
            "{listed:#}"
        );
    }
    let bare = alex.lends().await;
    assert!(
        bare.lent
            .iter()
            .any(|l| l.address == format!("http://{}/", host(&label(&sam, "pad")))),
        "the bare command's addresses are the lends' origins: {bare:?}"
    );

    let (head, sock) = ask(
        api,
        &host(&label(&sam, "pad")),
        "GET /whoami?x=1 HTTP/1.1\r\nConnection: close\r\n",
    )
    .await;
    assert!(
        head.starts_with("HTTP/1.1 200"),
        "Sam's pad at its origin: {head}{}",
        body(sock).await
    );
    let seen = last(&pad_heads);
    assert!(seen.starts_with("GET /whoami?x=1 HTTP/1.1\r\n"), "{seen}");
    assert_eq!(
        header_values(&seen, "x-svrngs-key"),
        vec![actor_of(&signing(1))],
        "Sam's door names Alex's key: {seen}"
    );
    assert_eq!(
        header_values(&seen, "x-svrngs-person"),
        vec!["alex"],
        "{seen}"
    );
    assert_eq!(header_values(&seen, "authorization"), vec![SAMS], "{seen}");
    assert_eq!(
        header_values(&seen, "host"),
        vec![format!("localhost:{}", pad.port())],
        "{seen}"
    );

    let (head, sock) = ask(
        api,
        &host(&label(&alex, "telly")),
        "GET /mine HTTP/1.1\r\nConnection: close\r\n",
    )
    .await;
    assert!(
        head.starts_with("HTTP/1.1 200"),
        "Alex's own telly at its origin: {head}{}",
        body(sock).await
    );
    let seen = last(&telly_heads);
    assert!(seen.starts_with("GET /mine HTTP/1.1\r\n"), "{seen}");
    assert_eq!(
        header_values(&seen, "x-svrngs-key"),
        vec![actor_of(&signing(1))],
        "{seen}"
    );
    assert_eq!(header_values(&seen, "authorization"), vec![ALEXS], "{seen}");

    let (head, mut sock) = ask(
        api,
        &host(&label(&sam, "live")),
        "GET /socket HTTP/1.1\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\n",
    )
    .await;
    assert!(
        head.starts_with("HTTP/1.1 101"),
        "a WebSocket at Sam's origin: {head}"
    );
    for word in ["ping", "pong"] {
        sock.write_all(word.as_bytes()).await.unwrap();
        let mut echo = vec![0u8; 4];
        tokio::time::timeout(Duration::from_secs(20), sock.read_exact(&mut echo))
            .await
            .expect("the echo comes back through both doors")
            .unwrap();
        assert_eq!(echo, word.as_bytes());
    }
    assert_eq!(
        header_values(&last(&live_heads), "x-svrngs-key"),
        vec![actor_of(&signing(1))]
    );

    let stranger = Label::of(&signing(9).verifying_key().to_bytes(), "pad");
    let (head, _) = ask(
        api,
        &host(&stranger),
        "GET / HTTP/1.1\r\nConnection: close\r\n",
    )
    .await;
    assert!(
        head.starts_with("HTTP/1.1 404"),
        "a label naming a key that stands nowhere: {head}"
    );
    sam.endpoint().close().await;
    alex.endpoint().close().await;
}

/// **The record answers no lend's origin, and no page** (the u5-2 row): a
/// request at a lend's host for a record route reaches the lend, never the
/// record; a request carrying a browser's `Origin` or `Sec-Fetch-Site` —
/// a lent page's POST to the record API — is refused 403 `from_a_page`,
/// and the same request from an app's own client is answered. Planted: the
/// record's routes before the host is asked (`to_a_lend` not layered),
/// which answers the lend's host with the groups; and no check for a page.
#[tokio::test]
async fn the_record_answers_no_lend_s_origin_and_refuses_a_page() {
    let dir = tempfile::tempdir().unwrap();
    let alex = start(1, dir.path()).await;
    let seed = Roster::new(
        [(Person::parse("alex").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    alex.rail().set_seed(NS, &seed).unwrap();
    let (telly, heads) = origins::head_origin().await;
    lends(dir.path(), "telly", telly.port(), ALEXS);
    let api = alex.api_addr();
    let label = Label::of(alex.endpoint().id().as_bytes(), "telly");
    let lend_host = format!("{label}.localhost:{}", api.port());

    let (head, sock) = ask(
        api,
        &lend_host,
        "GET /v1/groups HTTP/1.1\r\nConnection: close\r\n",
    )
    .await;
    let answer = body(sock).await;
    assert!(head.starts_with("HTTP/1.1 200"), "{head}{answer}");
    assert!(
        !answer.contains("\"groups\""),
        "the record answered a lend's host: {answer}"
    );
    assert!(
        last(&heads).starts_with("GET /v1/groups HTTP/1.1\r\n"),
        "the lend was asked"
    );

    let append =
        serde_json::json!({ "namespace": NS, "body": { "kind": "note", "n": 1 } }).to_string();
    let post = |extra: &str| {
        format!(
            "POST /v1/rail/append HTTP/1.1\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n{extra}",
            append.len()
        )
    };
    for page in [
        format!("Origin: http://{lend_host}\r\n"),
        "Sec-Fetch-Site: same-site\r\n".to_string(),
    ] {
        let mut sock = tokio::net::TcpStream::connect(api).await.unwrap();
        sock.write_all(
            format!(
                "{}Host: 127.0.0.1:{}\r\n\r\n{append}",
                post(&page),
                api.port()
            )
            .as_bytes(),
        )
        .await
        .unwrap();
        let reply = body(sock).await;
        assert!(reply.starts_with("HTTP/1.1 403"), "{page}: {reply}");
        assert!(reply.contains("from_a_page"), "{page}: {reply}");
    }
    let mut sock = tokio::net::TcpStream::connect(api).await.unwrap();
    sock.write_all(
        format!("{}Host: 127.0.0.1:{}\r\n\r\n{append}", post(""), api.port()).as_bytes(),
    )
    .await
    .unwrap();
    let reply = body(sock).await;
    assert!(
        !reply.contains("from_a_page"),
        "an app's own client is answered: {reply}"
    );
    alex.endpoint().close().await;
}

/// `dir`'s computer lends `name` on `port`, of `kind`, its pages reading
/// `reads`.
fn declare(dir: &std::path::Path, name: &str, port: u16, kind: &str, reads: &[&str]) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let body = serde_json::json!({ "port": port, "kind": kind, "limit": 3, "reads": reads });
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
    let secrets = lend::secret_dir(dir, &LendName::parse(name).unwrap());
    declared::write_declared_in(&secrets, "authorization", ALEXS).unwrap();
}

/// `GET /x` at `host` with `extra` header lines: the status, the refusal's
/// kind, and the app the service behind it was told — `None` when the
/// service was not asked.
async fn judged(
    api: SocketAddr,
    host: &str,
    extra: &str,
    heads: &std::sync::Mutex<Vec<String>>,
) -> (u16, Option<String>, Option<Vec<String>>) {
    let before = heads.lock().unwrap().len();
    let (head, sock) = ask(
        api,
        host,
        &format!("GET /x HTTP/1.1\r\nConnection: close\r\n{extra}"),
    )
    .await;
    body(sock).await;
    let status = head[9..12].parse().unwrap();
    let refused = header_values(&head, "x-svrngs-refused")
        .first()
        .map(|k| k.to_string());
    let after = heads.lock().unwrap();
    let told = (after.len() > before).then(|| {
        header_values(after.last().unwrap(), "x-svrngs-app")
            .into_iter()
            .map(str::to_string)
            .collect()
    });
    (status, refused, told)
}

/// **A computer's origin names the app asking, and keeps a page to its
/// reads** (sup-1; DESIGN §3, "Lent apps", call 7, and its table). Alex
/// lends a page, `night`, reading kind `media`, a `telly` of it and a
/// `diary`; Sam a `films` and a `notes`. At Alex's computer, each row of
/// the table: a program (no `Sec-Fetch-*`), a typed address, a navigation
/// (whatever its `Origin`) and a request from the lend's own origin are the
/// lend's own page, its service told its own label; a `fetch` from
/// `night`'s page, by its `Origin`, is `night`'s, at Alex's own `telly` and
/// at Sam's `films` alike, Sam's door told it from the pass Alex's computer
/// signs; Sam's page at its origin is Sam's `films` at Sam's door. Refused
/// `closed`, its service never asked: `night`'s page at `diary` and at
/// Sam's `notes`, which it does not read; a request from another page with
/// no `Origin` (an `<img>`); an `Origin` at another port; a site's.
/// Node's `fetch` says `Sec-Fetch-Mode: cors` and no `Sec-Fetch-Site`,
/// which a browser never leaves out (start-address (b); start-measure,
/// `demo/runs/start/2026-10-13/README.md` §3 (a) 4): a program's.
/// Planted, each watched failing: the origin taking the `Host` for a
/// cross-origin `fetch`; a page reaching a lend it does not read; Alex's
/// computer presenting no pass of its own to Sam's door; a program with a
/// mode and no site refused (`app_of` as at a39c8d4).
#[tokio::test]
async fn a_computer_s_origin_names_the_app_and_keeps_a_page_to_its_reads() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    let mut heads = std::collections::BTreeMap::new();
    for (dir, name, kind, reads) in [
        (alex_dir.path(), "night", "night", &["media"][..]),
        (alex_dir.path(), "telly", "media", &[]),
        (alex_dir.path(), "diary", "notes", &[]),
        (sam_dir.path(), "films", "media", &[]),
        (sam_dir.path(), "notes", "notes", &[]),
    ] {
        let (at, kept) = origins::head_origin().await;
        declare(dir, name, at.port(), kind, reads);
        heads.insert(name, kept);
    }
    let api = alex.api_addr();
    let port = api.port();
    let label = |seed: u8, name: &str| Label::of(&signing(seed).verifying_key().to_bytes(), name);
    let night = label(1, "night");
    let from = |l: &Label| format!("Origin: http://{l}.localhost:{port}\r\n");
    let fetch = "Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: cors\r\n";
    let told = |l: &Label| (200, None, Some(vec![l.to_string()]));
    let closed = (403, Some("closed".to_string()), None);
    let rows = [
        ("a program", 1, "night", String::new(), told(&night)),
        (
            "Node's fetch, a program saying a mode and no site",
            1,
            "night",
            "Sec-Fetch-Mode: cors\r\n".into(),
            told(&night),
        ),
        (
            "a typed address",
            1,
            "night",
            "Sec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\n".into(),
            told(&night),
        ),
        (
            "a navigation from another page",
            1,
            "telly",
            format!(
                "Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: navigate\r\n{}",
                from(&night)
            ),
            told(&label(1, "telly")),
        ),
        (
            "its own origin",
            1,
            "night",
            "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\n".into(),
            told(&night),
        ),
        (
            "night's fetch at Alex's telly",
            1,
            "telly",
            format!("{fetch}{}", from(&night)),
            told(&night),
        ),
        (
            "night's fetch at Sam's films",
            2,
            "films",
            format!("{fetch}{}", from(&night)),
            told(&night),
        ),
        (
            "Sam's page at its origin",
            2,
            "films",
            "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\n".into(),
            told(&label(2, "films")),
        ),
        (
            "night's fetch at Alex's diary, unread",
            1,
            "diary",
            format!("{fetch}{}", from(&night)),
            closed.clone(),
        ),
        (
            "night's fetch at Sam's notes, unread",
            2,
            "notes",
            format!("{fetch}{}", from(&night)),
            closed.clone(),
        ),
        (
            "another page's <img>",
            1,
            "telly",
            "Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: no-cors\r\n".into(),
            closed.clone(),
        ),
        (
            "an Origin at another port",
            1,
            "telly",
            format!("{fetch}Origin: http://{night}.localhost:1\r\n"),
            closed.clone(),
        ),
        (
            "a site's",
            1,
            "telly",
            "Sec-Fetch-Site: cross-site\r\nSec-Fetch-Mode: cors\r\nOrigin: https://example.com\r\n"
                .into(),
            closed.clone(),
        ),
    ];
    for (row, seed, name, extra, want) in rows {
        let host = format!("{}.localhost:{port}", label(seed, name));
        let got = judged(api, &host, &extra, &heads[name]).await;
        eprintln!("{row}: {got:?}");
        assert_eq!(got, want, "{row}");
    }
    sam.endpoint().close().await;
    alex.endpoint().close().await;
}

/// A path on loopback to `to` that holds every datagram `delay` each way,
/// so a computer reached through it answers its dial as late as the
/// slowest dial measured (`demo/runs/asleep/2026-10-14/README.md`, 1).
async fn slow_path(to: SocketAddr, delay: Duration) -> SocketAddr {
    use std::sync::{Arc, Mutex};
    use tokio::net::UdpSocket;
    let front = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
    let back = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
    let at = front.local_addr().unwrap();
    let dialler = Arc::new(Mutex::new(None::<SocketAddr>));
    let (f, b, d) = (front.clone(), back.clone(), dialler.clone());
    tokio::spawn(async move {
        let mut buf = vec![0u8; 65536];
        while let Ok((n, from)) = f.recv_from(&mut buf).await {
            *d.lock().unwrap() = Some(from);
            let (b, bytes) = (b.clone(), buf[..n].to_vec());
            tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                let _ = b.send_to(&bytes, to).await;
            });
        }
    });
    tokio::spawn(async move {
        let mut buf = vec![0u8; 65536];
        while let Ok((n, _)) = back.recv_from(&mut buf).await {
            let Some(dialler) = *dialler.lock().unwrap() else {
                continue;
            };
            let (f, bytes) = (front.clone(), buf[..n].to_vec());
            tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                let _ = f.send_to(&bytes, dialler).await;
            });
        }
    });
    at
}

/// **A lend's computer asleep is said away within the cap; one that answers
/// as late as the slowest dial measured is not** (events-asleep (b)). Alex
/// lets Sam and Cy in. Sam's computer is reached only through a path that
/// holds each datagram 750 ms each way, so its dial is answered no sooner
/// than 1.5 s, past the slowest working dial `demo/runs/asleep/2026-10-14/`
/// measured, 1,013 ms: Sam's pad, at its origin on Alex's computer, answers
/// 200. Cy's computer is asleep, a socket nobody reads (`common::never_answers`):
/// Cy's lend at its origin is answered 502 `away` within
/// [`svrngs_node::dial::DIAL_WITHIN`] and a second. Planted, each watched
/// failing: the dial uncapped, as before this row (`guest.rs`'s bridge
/// awaiting `connect` to its own end), which Cy's lend waits out; and the
/// cap below the slow path's dial, 1 s, which says Sam's lend away.
#[tokio::test]
async fn a_lend_s_computer_asleep_is_said_away_within_the_cap() {
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
    let delay = Duration::from_millis(750);
    let slow = slow_path(loopback_sockets(sam.endpoint())[0], delay).await;
    alex.transport()
        .add_known_peer(sam.endpoint().id(), vec![slow]);
    let (_asleep, at_cy) = common::never_answers();
    let cy =
        svrngs_node::iroh::PublicKey::from_bytes(signing(3).verifying_key().as_bytes()).unwrap();
    alex.transport().add_known_peer(cy, at_cy);
    for (person, seed) in [("sam", 2), ("cy", 3)] {
        let admit = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse(person).unwrap(),
                key: actor_of(&signing(seed)),
                invite: None,
            },
            on_behalf_of: None,
        };
        alex.rail().commit(NS, &admit).unwrap().unwrap();
    }
    let (pad, _) = origins::head_origin().await;
    lends(sam_dir.path(), "pad", pad.port(), SAMS);
    let api = alex.api_addr();
    let host =
        |key: &[u8; 32], name: &str| format!("{}.localhost:{}", Label::of(key, name), api.port());
    let within = svrngs_node::dial::DIAL_WITHIN + Duration::from_secs(1);

    let asked = std::time::Instant::now();
    let (head, sock) = ask(
        api,
        &host(sam.endpoint().id().as_bytes(), "pad"),
        "GET / HTTP/1.1\r\nConnection: close\r\n",
    )
    .await;
    let took = asked.elapsed();
    eprintln!("Sam's pad, through the slow path, answered in {took:?}: {head}");
    assert!(
        head.starts_with("HTTP/1.1 200"),
        "a lend that answers is never said away: {head}{}",
        body(sock).await
    );
    assert!(
        took >= 2 * delay,
        "the dial went through the slow path, at least a round trip: {took:?}"
    );

    let asked = std::time::Instant::now();
    let (head, sock) = ask(
        api,
        &host(&signing(3).verifying_key().to_bytes(), "pad"),
        "GET / HTTP/1.1\r\nConnection: close\r\n",
    )
    .await;
    let took = asked.elapsed();
    eprintln!("Cy's pad, Cy's computer asleep, answered in {took:?}: {head}");
    assert!(
        head.starts_with("HTTP/1.1 502"),
        "{head}{}",
        body(sock).await
    );
    assert_eq!(
        header_values(&head, "x-svrngs-refused"),
        vec!["away"],
        "{head}"
    );
    assert!(
        took < within,
        "told Cy's computer is asleep in {took:?}, past the cap and a second, {within:?}"
    );
    sam.endpoint().close().await;
    alex.endpoint().close().await;
}

/// **A stopped lend said by its lender, as a page to a tab** (v1ui-polish
/// (a); `demo/runs/v1ui/2026-10-15/README.md` finding 2). Alex's telly and
/// Sam's pad each lent on a port nothing listens on: at each origin on
/// Alex's computer, a tab (`Accept: text/html`) is shown a page naming the
/// lender and the lend, never a brace or the port; a program is given the
/// JSON, `away`. Sam's computer here holds only Alex's founding, so it
/// names its lender by none of its own (`Its lender`), never a host.
/// Planted: HEAD before it, which answered both in braces.
#[tokio::test]
async fn a_stopped_lend_is_said_by_its_lender_as_a_page_to_a_tab_and_json_to_a_program() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    let stopped = common::stopped();
    lends(alex_dir.path(), "telly", stopped.port(), ALEXS);
    lends(sam_dir.path(), "pad", stopped.port(), SAMS);
    let api = alex.api_addr();
    for (node, name, lender) in [(&alex, "telly", "alex"), (&sam, "pad", "Its lender")] {
        let host = format!(
            "{}.localhost:{}",
            Label::of(node.endpoint().id().as_bytes(), name),
            api.port()
        );
        let tab = "GET / HTTP/1.1\r\nAccept: text/html,*/*\r\nSec-Fetch-Mode: navigate\r\nConnection: close\r\n";
        let (head, sock) = ask(api, &host, tab).await;
        let page = format!("{head}{}", body(sock).await);
        assert!(page.starts_with("HTTP/1.1 502"), "{page}");
        assert!(
            page.to_ascii_lowercase()
                .contains("content-type: text/html"),
            "{page}"
        );
        assert!(
            page.contains(&format!("{lender}’s {name} is not answering")),
            "{page}"
        );
        assert!(
            !page.contains("{\"") && !page.contains(&stopped.port().to_string()),
            "{page}"
        );
        let (head, sock) = ask(api, &host, "GET / HTTP/1.1\r\nConnection: close\r\n").await;
        let program = format!("{head}{}", body(sock).await);
        assert!(program.contains(r#""kind":"away""#), "{program}");
    }
}
