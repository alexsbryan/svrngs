#![allow(clippy::disallowed_methods, reason = "a test dials what it runs")]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! A UDP port is lent (give-udp; svrn-docs rings/current/GIVE.md §6): Sam
//! runs `svrngs share <port> --udp --kind echo`, and a program on Alex's
//! computer sends to a socket there and hears the lent port answer, its
//! packets crossing as QUIC datagrams between two nodes on 127.0.0.1, no
//! relay. A packet larger than one datagram arrives whole. What is refused,
//! and why: a key standing in no group Sam's computer holds is closed at
//! the connection, as on every protocol a member speaks, since a raw port
//! carries no pass; and a person past the lend's limit is told so, the
//! limit counting people, so one person's two programs are one; and a
//! person removed while their program keeps sending is cut by Sam's
//! computer within seconds, since the member's own computer may not cut
//! itself (give-cut).

#[path = "common/made.rs"]
mod made;
use std::net::SocketAddr;
use std::time::Duration;

use svrngs::cli::{self, Said, Verb};
use svrngs::group::{self, Name};
use svrngs::share;
use svrngs_node::door::udp::{self, Carried, Unopened};
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, RelayConfig};
use svrngs_node::lend::{Lend, Raw, Source};
use svrngs_node::link::Link;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, SigningKey};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

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

struct Computer {
    node: Node,
    key: SigningKey,
    dir: tempfile::TempDir,
}

async fn computer(seed: u8) -> Computer {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[seed; 32]);
    let node = Node::start(
        dir.path().to_path_buf(),
        key.clone(),
        &no_relay(),
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    Computer { node, key, dir }
}

/// Where `node` is reached on 127.0.0.1.
fn at(node: &Node) -> EndpointAddr {
    let mut addr = EndpointAddr::new(node.endpoint().id());
    for mut a in node.endpoint().bound_sockets() {
        if a.ip().is_unspecified() {
            a.set_ip(
                if a.is_ipv4() { "127.0.0.1" } else { "::1" }
                    .parse()
                    .unwrap(),
            );
        }
        addr = addr.with_ip_addr(a);
    }
    addr
}

/// Sam makes house, and each of `joiners` is let in by Sam's link.
async fn house(joiners: &[(u8, &str)]) -> (Computer, Vec<Computer>) {
    let sam = computer(1).await;
    let made = made::new_group(&sam.node, name("house"), name("Sam"), None, &[]);
    let (_, printed) = made.await.expect("Sam makes house");
    let mut them = Vec::new();
    for (seed, who) in joiners {
        let c = computer(*seed).await;
        let book = MemoryLookup::new();
        book.add_endpoint_info(at(&sam.node));
        c.node.endpoint().address_lookup().unwrap().add(book);
        let link = Link::parse(&printed.client).unwrap();
        let joined = made::join_group(&c.node, &link, &name(who)).await;
        joined.unwrap_or_else(|e| panic!("{who} is let in: {e:?}"));
        them.push(c);
    }
    (sam, them)
}

/// A program on Sam's computer that answers each packet with itself.
async fn echo() -> SocketAddr {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = socket.local_addr().unwrap();
    tokio::spawn(async move {
        let mut buf = vec![0; 65_535];
        while let Ok((n, from)) = socket.recv_from(&mut buf).await {
            assert!(
                from.ip().is_loopback(),
                "the door sends from 127.0.0.1: {from}"
            );
            let _ = socket.send_to(&buf[..n], from).await;
        }
    });
    addr
}

/// `svrngs share <port> --udp --kind echo [args]` on Sam's computer, as the
/// command parses and declares it.
fn share_udp(sam: &Computer, port: u16, args: &[&str]) -> Lend {
    let argv = [
        "svrngs",
        "share",
        &port.to_string(),
        "--udp",
        "--kind",
        "echo",
    ];
    let argv = argv.iter().chain(args).map(|s| s.to_string());
    let groups = ["--group", "house"].iter().map(|s| s.to_string());
    let Ok(Verb::Share(mut asked, named, _)) = cli::parse(argv.chain(groups)) else {
        panic!("share --udp parses");
    };
    asked.audience = cli::audience_of(sam.dir.path(), asked.port, &named).unwrap();
    assert_eq!(asked.raw, Some(Raw::Udp));
    // A UDP port answers no connection, so it is lent unprobed.
    let mut lend = share::lend_of(&asked, share::well_known_kind(asked.port), "")
        .expect("a UDP port and its kind");
    assert_eq!(lend.source, Source::Raw(Raw::Udp, asked.port));
    svrngs_node::node::approve(&mut lend);
    cli::share_lend(sam.dir.path(), &lend, None).unwrap();
    lend
}

/// Alex's (or Dee's) computer opens Sam's lend, a socket on 127.0.0.1 its
/// programs send to.
async fn open(
    who: &Computer,
    sam: &Computer,
    lend: &Lend,
) -> (Result<Carried, Unopened>, SocketAddr) {
    let local = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = local.local_addr().unwrap();
    let carried = udp::carried(who.node.net(), at(&sam.node), &lend.name, local).await;
    (carried, addr)
}

/// A program sending `packet` to `to` until an answer comes, a datagram
/// being one that may be lost: the answer, or `None` within five tries.
async fn asks(program: &UdpSocket, to: SocketAddr, packet: &[u8]) -> Option<Vec<u8>> {
    let mut buf = vec![0; 65_535];
    for _ in 0..5 {
        program.send_to(packet, to).await.unwrap();
        let heard = tokio::time::timeout(Duration::from_secs(2), program.recv_from(&mut buf));
        if let Ok(Ok((n, _))) = heard.await {
            return Some(buf[..n].to_vec());
        }
    }
    None
}

async fn program() -> UdpSocket {
    UdpSocket::bind("127.0.0.1:0").await.unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn share_udp_lends_a_port_and_a_member_s_datagrams_come_back() {
    let (sam, them) = house(&[(2, "Alex")]).await;
    let port = echo().await.port();
    let lend = share_udp(&sam, port, &[]);
    let said = Said::Shared {
        lend: Box::new(lend.clone()),
        answers: Vec::new(),
        secret: false,
        accounts: None,
        client: None,
        issuer: None,
        terms: Some(Default::default()),
    };
    let said = said.to_string();
    assert!(
        said.contains(&format!("You lend echo (UDP port {port})")),
        "{said}"
    );

    let (carried, local) = open(&them[0], &sam, &lend).await;
    let _carried = carried.expect("Alex's computer opens Sam's echo");
    let heard = asks(&program().await, local, b"hello from Alex").await;
    assert_eq!(heard.as_deref(), Some(&b"hello from Alex"[..]));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_packet_larger_than_a_datagram_arrives_whole() {
    let (sam, them) = house(&[(2, "Alex")]).await;
    let lend = share_udp(&sam, echo().await.port(), &[]);
    let (carried, local) = open(&them[0], &sam, &lend).await;
    let _carried = carried.expect("Alex's computer opens Sam's echo");
    // About eight times what a datagram carries (some 1,200 bytes on a
    // path), and under what macOS sends at once (net.inet.udp.maxdgram, 9,216).
    let packet: Vec<u8> = (0..9_000u32).map(|i| (i % 251) as u8).collect();
    let heard = asks(&program().await, local, &packet).await;
    let heard = heard.expect("the packet comes back");
    assert_eq!(heard.len(), packet.len(), "the packet arrives whole");
    assert!(heard == packet, "the packet arrives whole, in order");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_key_standing_in_no_group_here_is_refused() {
    let (sam, _) = house(&[]).await;
    let lend = share_udp(&sam, echo().await.port(), &[]);
    let stranger = computer(9).await;
    let (carried, _) = open(&stranger, &sam, &lend).await;
    assert!(
        matches!(carried, Err(Unopened::NotAdmitted)),
        "a key standing in no group Sam's computer holds is refused: {:?}",
        carried.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_limit_counts_people_across_flows() {
    let (sam, them) = house(&[(2, "Alex"), (3, "Dee")]).await;
    let lend = share_udp(&sam, echo().await.port(), &["--limit", "1"]);
    let (alex, dee) = (&them[0], &them[1]);

    // Two programs on Alex's computer are two flows and one person.
    let (carried, alex_s) = open(alex, &sam, &lend).await;
    let _alex = carried.expect("Alex's computer opens Sam's echo");
    let (one, two) = (program().await, program().await);
    assert_eq!(
        asks(&one, alex_s, b"one").await.as_deref(),
        Some(&b"one"[..])
    );
    assert_eq!(
        asks(&two, alex_s, b"two").await.as_deref(),
        Some(&b"two"[..])
    );

    // Dee is a second person, past a limit of one: told so, and not carried.
    let (carried, local) = open(dee, &sam, &lend).await;
    let (mut told, _dee) = carried.expect("Dee's computer opens it; a flow is what is counted");
    let theirs = program().await;
    assert_eq!(
        asks(&theirs, local, b"dee").await,
        None,
        "Dee's packet is carried past the limit"
    );
    let told = tokio::time::timeout(Duration::from_secs(5), told.recv()).await;
    let told = told
        .expect("Dee is told within 5s")
        .expect("the lender's words");
    assert!(
        told.starts_with("echo is lent to 1 person at once, and 1 is using it now"),
        "{told}"
    );
    // Alex's flows still cross.
    assert_eq!(
        asks(&one, alex_s, b"again").await.as_deref(),
        Some(&b"again"[..])
    );
}

/// Sam's computer removes `who` from the group they stand in there: the
/// cut `svrngs remove` makes, written on Sam's record.
fn remove(sam: &Computer, who: &Computer) {
    let rail = sam.node.rail();
    let (group, member) = rail
        .standing_anywhere(&actor_of(&who.key))
        .expect("they stand");
    let cut = svrngs_core::Removal {
        person: member.person,
        joined: None,
    };
    rail.commit(&group, &cut).unwrap().unwrap();
}

/// A program on Sam's computer that answers each packet with itself, and
/// tells each packet it is sent.
async fn told_echo() -> (SocketAddr, mpsc::UnboundedReceiver<Vec<u8>>) {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let (addr, (tell, told)) = (socket.local_addr().unwrap(), mpsc::unbounded_channel());
    tokio::spawn(async move {
        let mut buf = vec![0; 65_535];
        while let Ok((n, from)) = socket.recv_from(&mut buf).await {
            let _ = tell.send(buf[..n].to_vec());
            let _ = socket.send_to(&buf[..n], from).await;
        }
    });
    (addr, told)
}

/// A program that sends `packet` to `to` every 100 ms and tells each
/// answer it hears, until the test ends.
fn keeps_sending(
    program: UdpSocket,
    to: SocketAddr,
    packet: &'static [u8],
) -> mpsc::UnboundedReceiver<Vec<u8>> {
    let (tell, told) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut buf = vec![0; 65_535];
        let mut every = tokio::time::interval(Duration::from_millis(100));
        loop {
            tokio::select! {
                _ = every.tick() => drop(program.send_to(packet, to).await),
                Ok((n, _)) = program.recv_from(&mut buf) => drop(tell.send(buf[..n].to_vec())),
            }
        }
    });
    told
}

/// What came on `told` since it was last drained.
fn drained(told: &mut mpsc::UnboundedReceiver<Vec<u8>>) -> Vec<Vec<u8>> {
    std::iter::from_fn(|| told.try_recv().ok()).collect()
}

/// **A member no longer standing is cut** (give-cut). Alex and Dee each
/// have a program that keeps sending to Sam's echo. Sam removes Alex, and
/// within a few seconds none of Alex's packets reaches the lent port and
/// nothing comes back to Alex's program; Dee, still standing, is carried
/// through the same window. Alex's side is the door's own member half, not
/// Play, so it is Sam's computer that cuts: a removal says nothing new
/// reaches the person, and a flow that never idles would otherwise run on.
/// Planted: the lender's sweep that asks standing again, skipped.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_no_longer_standing_is_cut() {
    let (sam, them) = house(&[(2, "Alex"), (3, "Dee")]).await;
    let (echo, mut lent) = told_echo().await;
    let lend = share_udp(&sam, echo.port(), &[]);
    let (alex, dee) = (&them[0], &them[1]);
    let (carried, alex_s) = open(alex, &sam, &lend).await;
    let _alex = carried.expect("Alex's computer opens Sam's echo");
    let (carried, dee_s) = open(dee, &sam, &lend).await;
    let _dee = carried.expect("Dee's computer opens Sam's echo");
    let mut alex_hears = keeps_sending(program().await, alex_s, b"alex");
    let mut dee_hears = keeps_sending(program().await, dee_s, b"dee");
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(
        drained(&mut lent).iter().any(|p| p == b"alex"),
        "control: Alex's packets reach the lent port"
    );
    assert!(
        !drained(&mut alex_hears).is_empty(),
        "control: Alex's program hears the echo"
    );

    remove(&sam, alex);
    tokio::time::sleep(Duration::from_secs(3)).await;
    let _ = (
        drained(&mut lent),
        drained(&mut alex_hears),
        drained(&mut dee_hears),
    );
    tokio::time::sleep(Duration::from_secs(2)).await;
    let reached = drained(&mut lent);
    assert!(
        !reached.iter().any(|p| p == b"alex"),
        "Alex, removed, still reaches the lent port {} times in 2s",
        reached.iter().filter(|p| *p == b"alex").count()
    );
    let heard = drained(&mut alex_hears);
    assert!(
        heard.is_empty(),
        "Alex, removed, still hears {} answers in 2s",
        heard.len()
    );
    assert!(
        reached.iter().any(|p| p == b"dee") && !drained(&mut dee_hears).is_empty(),
        "Dee, still standing, is carried through the same window"
    );
}

/// **A port taken from a group is cut, and no other** (LENDING_SCOPE
/// §"Enforcement and disclosure" 2). Alex keeps sending to Sam's echo and
/// to Sam's other, both lent to the house. Sam lends echo to no group
/// (`[]`): two seconds on, none of Alex's packets reaches echo and none
/// comes back, while other still carries Alex's. Planted: the sweep asking
/// the lend as it was opened, not as it is declared now.
#[tokio::test(flavor = "multi_thread")]
async fn a_port_its_group_is_taken_from_is_cut_and_no_other() {
    let (sam, them) = house(&[(2, "Alex")]).await;
    let (echo, mut lent) = told_echo().await;
    let (other_at, mut others) = told_echo().await;
    let echoes = share_udp(&sam, echo.port(), &[]);
    let other = share_udp(&sam, other_at.port(), &["--name", "other"]);
    let alex = &them[0];
    let (carried, at_echo) = open(alex, &sam, &echoes).await;
    let _echo = carried.expect("Alex's computer opens Sam's echo");
    let (carried, at_other) = open(alex, &sam, &other).await;
    let _other = carried.expect("Alex's computer opens Sam's other");
    let mut hears = keeps_sending(program().await, at_echo, b"echo");
    let _other_hears = keeps_sending(program().await, at_other, b"other");
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(
        drained(&mut lent).iter().any(|p| p == b"echo"),
        "control: echo is reached"
    );

    let paused = svrngs_node::lend::set_audience(sam.dir.path(), &echoes.name, Vec::new());
    assert!(paused.unwrap());
    tokio::time::sleep(Duration::from_secs(2)).await;
    let _ = (
        drained(&mut lent),
        drained(&mut hears),
        drained(&mut others),
    );
    tokio::time::sleep(Duration::from_secs(1)).await;
    let reached = drained(&mut lent);
    assert!(
        !reached.iter().any(|p| p == b"echo"),
        "echo still reached {} times",
        reached.len()
    );
    assert!(drained(&mut hears).is_empty(), "Alex still hears echo");
    assert!(
        drained(&mut others).iter().any(|p| p == b"other"),
        "other stopped"
    );
}

/// **A flow open on a share that is held stops** (SHARE_BINDING_REVIEWS §7
/// #3). Alex keeps sending to Sam's echo. Sam's share of it is held in its
/// file, as another flow's check or Sam's own list holds it: within two
/// seconds none of Alex's packets reaches echo, and none of the flows opened
/// again carries. Planted, watched failing: the flow's tick not reading its
/// lend again (Alex's open flow still reaches echo).
#[tokio::test(flavor = "multi_thread")]
async fn a_flow_open_on_a_share_that_is_held_stops() {
    let (sam, them) = house(&[(2, "Alex")]).await;
    let (echo, mut lent) = told_echo().await;
    let echoes = share_udp(&sam, echo.port(), &[]);
    let (carried, at_echo) = open(&them[0], &sam, &echoes).await;
    let _echo = carried.expect("Alex's computer opens Sam's echo");
    let _hears = keeps_sending(program().await, at_echo, b"echo");
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(
        drained(&mut lent).iter().any(|p| p == b"echo"),
        "control: echo is reached"
    );
    let held = svrngs_node::lend::mutate(sam.dir.path(), &echoes.name, |lend| {
        lend.terms.approved.as_mut().expect("approved").held = true;
    });
    assert!(held.unwrap());
    tokio::time::sleep(Duration::from_secs(2)).await;
    drained(&mut lent);
    tokio::time::sleep(Duration::from_secs(1)).await;
    let reached = drained(&mut lent);
    assert!(
        reached.is_empty(),
        "echo still reached {} times",
        reached.len()
    );
}

/// A Python program on Sam's computer, as its first argument says: `echo`,
/// answering each packet with itself on 127.0.0.1, with `SO_REUSEPORT` so
/// another socket may bind its address beside it, and printing its port;
/// `orphan`, binding the port its second argument names beside it, sending
/// once, so it is that socket's last user, then leaving it to a child that
/// sends nothing and printing the child's pid as it exits. A Mac then reads
/// that socket's last user as gone, and can't judge it.
#[cfg(target_os = "macos")]
const BESIDE: &str = r#"
import os, socket, sys, time
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
s.bind(("127.0.0.1", int(sys.argv[2]) if sys.argv[1] == "orphan" else 0))
if sys.argv[1] == "orphan":
    s.sendto(b"x", ("127.0.0.1", 9))
    child = os.fork()
    if child == 0:
        time.sleep(60)
        os._exit(0)
    print(child, flush=True)
    os._exit(0)
print(s.getsockname()[1], flush=True)
while True:
    data, at = s.recvfrom(65535)
    s.sendto(data, at)
"#;

/// [`BESIDE`] run as `how` beside `port`, and the line it printed.
#[cfg(target_os = "macos")]
fn beside(how: &str, port: u16) -> (std::process::Child, u32) {
    use std::io::BufRead as _;
    let mut python = std::process::Command::new("python3")
        .args(["-c", BESIDE, how, &port.to_string()])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("python3 runs here");
    let mut said = String::new();
    let out = python.stdout.take().unwrap();
    std::io::BufReader::new(out).read_line(&mut said).unwrap();
    (python, said.trim().parse().expect("a number"))
}

/// **A flow opened while what answers on its port can't be judged opens,
/// carries nothing then, and carries once it can** (door/udp.rs `opened`):
/// on Sam's Mac a socket beside echo's on its port whose last user has
/// exited can't be judged, as a socket a server is still making for another
/// flow can't on Linux. Alex's program keeps sending: nothing comes back
/// while that socket is there, Alex is told why once at most, and once it
/// closes, echo's answers come back within seconds on the flow opened first.
/// Planted, watched failing: an unjudged opening refused and remembered for
/// a minute (no answer within the wait).
#[cfg(target_os = "macos")]
#[tokio::test(flavor = "multi_thread")]
async fn a_flow_opened_while_its_port_is_unjudged_carries_once_it_is() {
    let (sam, them) = house(&[(2, "Alex")]).await;
    let (mut echo, port) = beside("echo", 0);
    let echoes = share_udp(&sam, u16::try_from(port).unwrap(), &[]);
    let (mut orphan, child) = beside("orphan", u16::try_from(port).unwrap());
    orphan.wait().unwrap();
    let (carried, at_echo) = open(&them[0], &sam, &echoes).await;
    let (mut told, _echo) = carried.expect("Alex's computer opens Sam's echo");
    let mut heard = keeps_sending(program().await, at_echo, b"echo");
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert!(
        drained(&mut heard).is_empty(),
        "nothing carried while it can't be judged"
    );
    let gone = std::process::Command::new("kill")
        .args(["-9", &child.to_string()])
        .status();
    assert!(gone.unwrap().success(), "the orphan's child stops");
    let mut answered = false;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if drained(&mut heard).iter().any(|p| p == b"echo") {
            answered = true;
            break;
        }
    }
    let _ = (echo.kill(), echo.wait());
    let lines = std::iter::from_fn(|| told.try_recv().ok()).count();
    assert!(answered, "echo's answers come back once it can be judged");
    assert!(lines <= 1, "told why {lines} times");
}
