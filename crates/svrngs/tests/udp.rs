// SPDX-License-Identifier: AGPL-3.0-or-later
//! A UDP port is lent (give-udp; svrn-docs rings/current/GIVE.md §6): Sam
//! runs `svrngs share <port> --udp --kind echo`, and a program on Alex's
//! computer sends to a socket there and hears the lent port answer, its
//! packets crossing as QUIC datagrams between two nodes on 127.0.0.1, no
//! relay. A packet larger than one datagram arrives whole. What is refused,
//! and why: a key standing in no group Sam's computer holds is closed at
//! the connection, as on every protocol a member speaks, since a raw port
//! carries no pass; and a person past the lend's limit is told so, the
//! limit counting people, so one person's two programs are one.

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
use svrngs_node::SigningKey;
use tokio::net::UdpSocket;

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
    let relay = no_relay();
    let made = cli::new_group(&sam.node, name("house"), name("Sam"), None, None, &relay);
    let (_, printed) = made.await.expect("Sam makes house");
    let mut them = Vec::new();
    for (seed, who) in joiners {
        let c = computer(*seed).await;
        let book = MemoryLookup::new();
        book.add_endpoint_info(at(&sam.node));
        c.node.endpoint().address_lookup().unwrap().add(book);
        let link = Link::parse(&printed.client).unwrap();
        let joined = cli::join_group(&c.node, &c.key, &link, &name(who)).await;
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
    let Ok(Verb::Share(asked)) = cli::parse(argv) else {
        panic!("share --udp parses");
    };
    assert_eq!(asked.raw, Some(Raw::Udp));
    // A UDP port answers no connection, so it is lent unprobed.
    let lend = share::lend_of(&asked, |_| false, share::well_known_kind(asked.port))
        .expect("a UDP port and its kind");
    assert_eq!(lend.source, Source::Raw(Raw::Udp, asked.port));
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
    let carried = udp::carried(who.node.endpoint(), at(&sam.node), &lend.name, local).await;
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
        lend: lend.clone(),
        secret: false,
        accounts: None,
        running: true,
        client: None,
        issuer: None,
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
    let mut dee_s = carried.expect("Dee's computer opens it; a flow is what is counted");
    let theirs = program().await;
    assert_eq!(
        asks(&theirs, local, b"dee").await,
        None,
        "Dee's packet is carried past the limit"
    );
    let told = tokio::time::timeout(Duration::from_secs(5), dee_s.told()).await;
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
