// SPDX-License-Identifier: AGPL-3.0-or-later
//! A TCP port is lent (give-tcp; svrn-docs rings/current/GIVE.md §6): Sam
//! runs `svrngs share <port> --tcp --kind echo`, and a program on Alex's
//! computer connects to a listener there and hears the lent port answer,
//! each connection one stream on the one connection Alex's computer keeps
//! to Sam's for the lend, between two nodes on 127.0.0.1, no relay. Two
//! connections at once are two streams: one stalled holds back neither the
//! other nor the record's sync. A half-close crosses, so a program that
//! answers at the end of what it is sent answers. What is refused, and why:
//! a key standing in no group Sam's computer holds is closed at the
//! connection, as on every protocol a member speaks, since a raw port
//! carries no pass; and a person past the lend's limit is told so, the
//! limit counting people, so one person's two connections are one; and a
//! person removed with a connection open is cut by Sam's computer within
//! seconds, since the member's own computer may not cut itself (give-cut).

use std::net::SocketAddr;
use std::time::Duration;

use svrngs::cli::{self, Said, Verb};
use svrngs::group::{self, Name, Seed};
use svrngs::share::{self, ShareRefusal};
use svrngs_node::door::tcp::{self, Carried, Unopened};
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, RelayConfig};
use svrngs_node::lend::{Lend, Raw, Source};
use svrngs_node::link::Link;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, SigningKey};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

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
async fn house(joiners: &[(u8, &str)]) -> (Computer, Vec<Computer>, Seed) {
    let sam = computer(1).await;
    let relay = no_relay();
    let made = cli::new_group(&sam.node, name("house"), name("Sam"), None, None, &relay);
    let (seed, printed) = made.await.expect("Sam makes house");
    let mut them = Vec::new();
    for (n, who) in joiners {
        let c = computer(*n).await;
        let book = MemoryLookup::new();
        book.add_endpoint_info(at(&sam.node));
        c.node.endpoint().address_lookup().unwrap().add(book);
        let link = Link::parse(&printed.client).unwrap();
        let joined = cli::join_group(&c.node, &c.key, &link, &name(who)).await;
        joined.unwrap_or_else(|e| panic!("{who} is let in: {e:?}"));
        them.push(c);
    }
    (sam, them, seed)
}

/// A program on Sam's computer that answers each connection with what it
/// is sent, as it comes, and closes its side once the other closes theirs.
async fn echo() -> u16 {
    serve(|mut c: TcpStream| async move {
        let (mut from, mut to) = c.split();
        let _ = tokio::io::copy(&mut from, &mut to).await;
        let _ = to.shutdown().await;
    })
    .await
}

/// A program on Sam's computer that answers only once the other side has
/// closed theirs: how many bytes it was sent.
async fn counts_at_the_end() -> u16 {
    serve(|mut c: TcpStream| async move {
        let mut all = Vec::new();
        let _ = c.read_to_end(&mut all).await;
        let _ = c.write_all(format!("{} bytes", all.len()).as_bytes()).await;
        let _ = c.shutdown().await;
    })
    .await
}

async fn serve<F, Fut>(each: F) -> u16
where
    F: Fn(TcpStream) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((c, from)) = listener.accept().await {
            assert!(
                from.ip().is_loopback(),
                "the door connects from 127.0.0.1: {from}"
            );
            tokio::spawn(each(c));
        }
    });
    port
}

/// `svrngs share <port> --tcp --kind echo [args]` on Sam's computer, as the
/// command parses and declares it.
fn share_tcp(sam: &Computer, port: u16, args: &[&str]) -> Lend {
    let argv = [
        "svrngs",
        "share",
        &port.to_string(),
        "--tcp",
        "--kind",
        "echo",
    ];
    let argv = argv.iter().chain(args).map(|s| s.to_string());
    let Ok(Verb::Share(asked)) = cli::parse(argv) else {
        panic!("share --tcp parses");
    };
    assert_eq!(asked.raw, Some(Raw::Tcp));
    // A TCP port answers a connection, so it is lent only when one does.
    let unanswered = share::lend_of(&asked, |_| false, None);
    assert!(
        matches!(unanswered, Err(ShareRefusal::NothingListens(_))),
        "a TCP port nothing listens on is refused: {unanswered:?}"
    );
    let lend = share::lend_of(&asked, |_| true, share::well_known_kind(asked.port))
        .expect("a TCP port and its kind");
    assert_eq!(lend.source, Source::Raw(Raw::Tcp, asked.port));
    cli::share_lend(sam.dir.path(), &lend, None).unwrap();
    lend
}

/// Alex's (or Dee's) computer opens Sam's lend, a listener on 127.0.0.1 its
/// programs connect to.
async fn open(
    who: &Computer,
    sam: &Computer,
    lend: &Lend,
) -> (Result<Carried, Unopened>, SocketAddr) {
    let local = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = local.local_addr().unwrap();
    let carried = tcp::carried(who.node.endpoint(), at(&sam.node), &lend.name, local).await;
    (carried, addr)
}

/// A program's connection writing `said` and reading as many bytes back,
/// within five seconds.
async fn asks(program: &mut TcpStream, said: &[u8]) -> Option<Vec<u8>> {
    program.write_all(said).await.ok()?;
    let mut heard = vec![0; said.len()];
    let read = timeout(Duration::from_secs(5), program.read_exact(&mut heard));
    read.await.ok()?.ok()?;
    Some(heard)
}

#[tokio::test(flavor = "multi_thread")]
async fn share_tcp_lends_a_port_and_a_member_s_bytes_come_back() {
    let (sam, them, _) = house(&[(2, "Alex")]).await;
    let port = echo().await;
    let lend = share_tcp(&sam, port, &[]);
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
        said.contains(&format!("You lend echo (TCP port {port})")),
        "{said}"
    );

    let (carried, local) = open(&them[0], &sam, &lend).await;
    let _carried = carried.expect("Alex's computer opens Sam's echo");
    let mut program = TcpStream::connect(local).await.unwrap();
    let heard = asks(&mut program, b"hello from Alex").await;
    assert_eq!(heard.as_deref(), Some(&b"hello from Alex"[..]));
}

#[tokio::test(flavor = "multi_thread")]
async fn one_stalled_connection_holds_back_neither_the_other_nor_the_record() {
    let (sam, them, seed) = house(&[(2, "Alex")]).await;
    let lend = share_tcp(&sam, echo().await, &[]);
    let alex = &them[0];
    let (carried, local) = open(alex, &sam, &lend).await;
    let _carried = carried.expect("Alex's computer opens Sam's echo");

    // One program writes far more than any window holds and reads none of
    // what comes back, so its connection stalls end to end.
    let mut stalled = TcpStream::connect(local).await.unwrap();
    let flood = tokio::spawn(async move {
        let chunk = vec![7u8; 64 * 1024];
        for _ in 0..512 {
            if stalled.write_all(&chunk).await.is_err() {
                break;
            }
        }
        stalled
    });
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(
        !flood.is_finished(),
        "the flood stalls: it is read by nobody"
    );

    // A second program's connection, at the same time, is answered.
    let mut other = TcpStream::connect(local).await.unwrap();
    let heard = asks(&mut other, b"still here").await;
    assert_eq!(
        heard.as_deref(),
        Some(&b"still here"[..]),
        "a second connection is its own stream, not held back by the first"
    );
    // And the record still syncs between the two computers.
    let (group, sam_at) = (seed.id(), at(&sam.node));
    let synced = timeout(
        Duration::from_secs(10),
        alex.node.host().sync_with(&group, &sam_at),
    );
    let synced = synced.await.expect("the record syncs within 10s");
    assert!(synced.stop.is_none(), "the record syncs: {:?}", synced.stop);
    assert!(!flood.is_finished(), "the flood is still stalled");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_half_close_is_carried() {
    let (sam, them, _) = house(&[(2, "Alex")]).await;
    let lend = share_tcp(&sam, counts_at_the_end().await, &[]);
    let (carried, local) = open(&them[0], &sam, &lend).await;
    let _carried = carried.expect("Alex's computer opens Sam's counter");
    let mut program = TcpStream::connect(local).await.unwrap();
    program.write_all(&[1; 100_000]).await.unwrap();
    // The program closes its side and still reads: the lent port answers
    // only once that close reaches it, and closes its own after.
    program.shutdown().await.unwrap();
    let mut heard = Vec::new();
    let read = timeout(Duration::from_secs(10), program.read_to_end(&mut heard));
    read.await
        .expect("the answer and its close come within 10s")
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&heard), "100000 bytes");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_key_standing_in_no_group_here_is_refused() {
    let (sam, _, _) = house(&[]).await;
    let lend = share_tcp(&sam, echo().await, &[]);
    let stranger = computer(9).await;
    let (carried, _) = open(&stranger, &sam, &lend).await;
    assert!(
        matches!(carried, Err(Unopened::NotAdmitted)),
        "a key standing in no group Sam's computer holds is refused: {:?}",
        carried.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_limit_counts_people_across_connections() {
    let (sam, them, _) = house(&[(2, "Alex"), (3, "Dee")]).await;
    let lend = share_tcp(&sam, echo().await, &["--limit", "1"]);
    let (alex, dee) = (&them[0], &them[1]);

    // Two programs on Alex's computer are two connections and one person.
    let (carried, alex_s) = open(alex, &sam, &lend).await;
    let _alex = carried.expect("Alex's computer opens Sam's echo");
    let mut one = TcpStream::connect(alex_s).await.unwrap();
    let mut two = TcpStream::connect(alex_s).await.unwrap();
    assert_eq!(asks(&mut one, b"one").await.as_deref(), Some(&b"one"[..]));
    assert_eq!(asks(&mut two, b"two").await.as_deref(), Some(&b"two"[..]));

    // Dee is a second person, past a limit of one: told so, and closed.
    let (carried, local) = open(dee, &sam, &lend).await;
    let (mut told, _dee) =
        carried.expect("Dee's computer opens it; a connection is what is counted");
    let mut theirs = TcpStream::connect(local).await.unwrap();
    let _ = theirs.write_all(b"dee").await;
    let mut heard = Vec::new();
    let read = timeout(Duration::from_secs(5), theirs.read_to_end(&mut heard));
    let _ = read.await.expect("Dee's connection is closed within 5s");
    assert!(heard.is_empty(), "Dee's bytes are carried past the limit");
    let told = timeout(Duration::from_secs(5), told.recv()).await;
    let told = told
        .expect("Dee is told within 5s")
        .expect("the lender's words");
    assert!(
        told.starts_with("echo is lent to 1 person at once, and 1 is using it now"),
        "{told}"
    );
    // Alex's connections still cross.
    assert_eq!(
        asks(&mut one, b"again").await.as_deref(),
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

/// **A member no longer standing is cut** (give-cut). Alex and Dee each
/// have a connection open on Sam's echo. Sam removes Alex, and within a few
/// seconds Sam's computer closes Alex's: Alex's program reads its end.
/// Dee, still standing, keeps theirs through the same window. Alex's side
/// is the door's own member half, not Play, so it is Sam's computer that
/// cuts: a removal says nothing new reaches the person, and an open
/// connection would otherwise run until it closes. Planted: the lender's
/// sweep that asks standing again, skipped.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_no_longer_standing_is_cut() {
    let (sam, them, _) = house(&[(2, "Alex"), (3, "Dee")]).await;
    let lend = share_tcp(&sam, echo().await, &[]);
    let (alex, dee) = (&them[0], &them[1]);
    let (carried, alex_s) = open(alex, &sam, &lend).await;
    let _alex = carried.expect("Alex's computer opens Sam's echo");
    let (carried, dee_s) = open(dee, &sam, &lend).await;
    let _dee = carried.expect("Dee's computer opens Sam's echo");
    let mut alexs = TcpStream::connect(alex_s).await.unwrap();
    let mut dees = TcpStream::connect(dee_s).await.unwrap();
    assert_eq!(
        asks(&mut alexs, b"alex").await.as_deref(),
        Some(&b"alex"[..])
    );
    assert_eq!(asks(&mut dees, b"dee").await.as_deref(), Some(&b"dee"[..]));

    remove(&sam, alex);
    let mut rest = Vec::new();
    let ended = timeout(Duration::from_secs(5), alexs.read_to_end(&mut rest)).await;
    assert!(
        ended.is_ok(),
        "Alex, removed, still has a connection open 5s later"
    );
    assert!(
        rest.is_empty(),
        "Alex hears {} bytes after the cut",
        rest.len()
    );
    assert_eq!(
        asks(&mut dees, b"still").await.as_deref(),
        Some(&b"still"[..]),
        "Dee, still standing, keeps their connection through the same window"
    );
}
