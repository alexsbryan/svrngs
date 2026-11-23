//! Where a computer meets its groups, in tiers (relay fallbacks; svrn-docs
//! `rings/current/GROUP_RELAYS.md` §7): iroh dials a member at a relay this
//! computer's endpoint does not hold, which every dial across tiers rests
//! on; and a computer whose group names a relay of its own meets there,
//! takes the build's relay when its own stops answering, and gives it back
//! once its own answers again (`svrngs_node::tiers`). The tiers'
//! decisions are asked on one wire (common/on_wire.rs), each relay's answer
//! and registration the wire's (`Wire::relay`), the clock paused. Over real
//! relays, each its seam's own: a dial at a relay this endpoint does not
//! hold, the relay that carried an action as iroh's path names it, and a
//! relay with a tiny rate throttling a lender (share-refusal); each
//! relay plain HTTP on 127.0.0.1, as `svrngs relay --at` serves one.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[path = "common/on_wire.rs"]
mod on_wire;

use std::future::Future as _;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use iroh_relay::server::{ClientRateLimit, RelayConfig as RelayServer, Server, ServerConfig};
use svrngs_core::pass::Label;
use svrngs_core::relay::Url;
use svrngs_node::group::Seed;
use svrngs_node::iroh::{build_relayed_endpoint, EndpointAddr, RelayConfig, RelayUrl, SecretKey};
use svrngs_node::node::{Node, Relaying};
use svrngs_node::tiers::Away;
use svrngs_node::SigningKey;

/// A plain-HTTP relay on 127.0.0.1 as `relay` configures it (its port 0
/// for any), and its URL.
async fn relay(relay: RelayServer) -> (Server, String) {
    let mut config = ServerConfig::default();
    config.relay = Some(relay);
    let server = Server::spawn(config).await.expect("a relay on 127.0.0.1");
    let url = format!("http://{}/", server.http_addr().unwrap());
    (server, url)
}

fn at(url: &str) -> RelayConfig {
    RelayConfig {
        relay_urls: vec![url.into()],
        n0_services: false,
        loopback_only: false,
    }
}

/// **A dial reaches a member at a relay this computer does not hold**:
/// Bo's endpoint holds R2 alone, and dials Ada at R1, her home, through
/// R1, in a second. Control: Bo dialling Ada at R2, where she is not, is
/// not answered.
#[tokio::test(flavor = "multi_thread")]
async fn a_dial_reaches_a_member_at_a_relay_this_computer_does_not_hold() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_r1, u1), (_r2, u2)) = (
        relay(RelayServer::new(any)).await,
        relay(RelayServer::new(any)).await,
    );
    let alpn = b"tiers/0".to_vec();
    let bind = |seed: u8, url: String| {
        let alpn = vec![alpn.clone()];
        async move { build_relayed_endpoint(SecretKey::from_bytes(&[seed; 32]), alpn, &at(&url)).await }
    };
    let (ada, bo) = (
        bind(1, u1.clone()).await.unwrap(),
        bind(2, u2.clone()).await.unwrap(),
    );
    for e in [&ada, &bo] {
        tokio::time::timeout(Duration::from_secs(10), e.online())
            .await
            .expect("online");
    }
    let taken = tokio::spawn({
        let ada = ada.clone();
        async move { ada.accept().await.unwrap().await.unwrap() }
    });
    let dial =
        |url: &str| EndpointAddr::new(ada.id()).with_relay_url(url.parse::<RelayUrl>().unwrap());
    let missed = tokio::time::timeout(Duration::from_secs(3), bo.connect(dial(&u2), &alpn)).await;
    assert!(!matches!(missed, Ok(Ok(_))), "control: Ada is not at R2");
    let reached = tokio::time::timeout(Duration::from_secs(10), bo.connect(dial(&u1), &alpn)).await;
    assert!(
        matches!(reached, Ok(Ok(_))),
        "Bo reaches Ada at R1: {reached:?}"
    );
    taken.await.unwrap();
}

/// The asks that leave the fallback and two more, and iroh's slowest
/// re-check.
fn homing_by() -> Duration {
    svrngs_node::tiers::ASKED_EVERY * (svrngs_node::tiers::LEFT_AFTER + 2) + Duration::from_secs(26)
}

/// The relay `node` is homed at, once it is `want`, waited for up to
/// [`homing_by`]; the wait in ms.
async fn homed(node: &Node, want: &str) -> u128 {
    let (t, by) = (Instant::now(), homing_by());
    while t.elapsed() < by {
        let home = node
            .endpoint()
            .addr()
            .relay_urls()
            .next()
            .map(ToString::to_string);
        if home.as_deref() == Some(want) {
            return t.elapsed().as_millis();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("not homed at {want} within {by:?}");
}

/// Ada's computer, its build's relay `build`, keeping house met at `own`;
/// its data directory kept alive by the caller.
async fn ada(build: &str, own: &str) -> (tempfile::TempDir, Node) {
    let (dir, key) = (
        tempfile::tempdir().unwrap(),
        SigningKey::from_bytes(&[3; 32]),
    );
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let node = Node::start(
        dir.path().to_path_buf(),
        key.clone(),
        &at(build),
        |_| Ok(None),
        any,
    )
    .await
    .unwrap();
    let name = |n: &str| svrngs_node::group::Name::parse(n).unwrap();
    let seed = svrngs_node::group::draw(name("house"), name("Ada"), key.verifying_key());
    let at_own = [own.parse::<Url>().unwrap()];
    node.host()
        .keep_group(&seed, None, &at_own)
        .expect("Ada keeps house, met at her own relay");
    (dir, node)
}

/// **A group meets at the build's fallback while it answers, and at its
/// own relay when it does not** (relay fallbacks; the relay-fallback
/// charter: a browser may reach this computer only through the fallback):
/// Ada's group names A; her build's relay is B, reached through a way
/// that can be cut. She is homed at B; the way cut for 15 s, still at B
/// (review F2: planted `LEFT_AFTER` 1, and she was at A 15.08 s after the
/// cut); B stopped, at A; B back at its address, at B again. Planted: the
/// own tier preferred while the build's answers, and once the fallback
/// stops she never reaches her own relay (red: not homed at A within 56 s).
#[tokio::test(start_paused = true)]
async fn a_group_meets_at_the_builds_fallback_while_it_answers() {
    on_wire::relay(A, Relaying::Up);
    on_wire::relay(B, Relaying::Up);
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let (_dir, node, _seed) = ada_on_wire(B, A, every).await;
    eprintln!(
        "TIERS at the build's fallback while it answers: {} ms",
        homed_on(&node, B).await
    );
    // Reached: the first ask, at the node's start, found her registered
    // at B, as iroh's registration had by the time a real relay homed her.
    tokio::time::sleep(Duration::from_secs(1)).await;
    let away = tokio::time::Instant::now();
    on_wire::relay(B, Relaying::Cut);
    let mut back = false;
    while away.elapsed() < Duration::from_secs(45) {
        if away.elapsed() > Duration::from_secs(15) && !back {
            on_wire::relay(B, Relaying::Up);
            back = true;
        }
        assert_ne!(
            home(&node).as_deref(),
            Some(A),
            "at A {:?} after the cut",
            away.elapsed()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    on_wire::relay(B, Relaying::Down);
    eprintln!(
        "TIERS the fallback stopped, at its own: {} ms",
        homed_on(&node, A).await
    );
    on_wire::relay(B, Relaying::Up);
    eprintln!(
        "TIERS the fallback back, at the fallback: {} ms",
        homed_on(&node, B).await
    );
}

/// **A group's own HTTPS relay comes before the build's** (self-host;
/// DESIGN §3, "Where a computer meets its groups"): Ada's group names an
/// HTTPS relay of its own while her build's answers; she is held at her
/// own and not at the build's, and her page says she meets there, not at
/// the fallback. Planted: the own HTTPS relay left behind, and she was
/// held at the build's.
#[tokio::test(start_paused = true)]
async fn a_group_meets_at_its_own_https_relay_while_the_build_s_answers() {
    let own = "https://relay.example.org/";
    on_wire::relay(B, Relaying::Up);
    on_wire::relay(own, Relaying::Up);
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let (_dir, node, seed) = ada_on_wire(B, own, every).await;
    // The ask runs each ASKED_EVERY; wait for the decision it applies.
    let (t, by) = (tokio::time::Instant::now(), homing_by());
    while t.elapsed() < by && node.relays_held() != vec![own.to_string()] {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        node.relays_held(),
        vec![own.to_string()],
        "held at its own HTTPS relay"
    );
    assert_eq!(
        node.fallback_of(&seed.id()),
        None,
        "the page says she meets at her own"
    );
    assert!(
        node.tier_held().contains("tier: Own"),
        "{}",
        node.tier_held()
    );
}

/// **The own HTTPS relay's silence is said as its own, and the press says
/// which way it went** (self-host; KEEP_GOING_AT_HOME 7): Ada's group names
/// an HTTPS relay of its own while her build's answers. With the own
/// answering, the button says "stayed-own" and her page says she meets at
/// her own; the own stopped, she is homed at the build's and her page says
/// why in the own line's words, and the button says she moved to the public
/// way. Planted, watched red: `at_fallback` answering the plain-HTTP
/// `fallback` for the own's silence (the page said the wrong why), and
/// `check_now` answering "stayed"/"moved" whatever tier came first.
#[tokio::test(start_paused = true)]
async fn the_own_https_relay_s_silence_is_said_as_its_own_and_the_press_says_which_way() {
    let own = "https://relay.example.org/";
    on_wire::relay(B, Relaying::Up);
    on_wire::relay(own, Relaying::Up);
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let (_dir, node, seed) = ada_on_wire(B, own, every).await;
    let (t, by) = (tokio::time::Instant::now(), homing_by());
    while t.elapsed() < by && node.relays_held() != vec![own.to_string()] {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        node.fallback_of(&seed.id()),
        None,
        "the page says she meets at her own"
    );
    assert_eq!(
        pressed_on(&node).await,
        "/?view=computer&stayed-own",
        "the button says the own answered"
    );

    on_wire::relay(own, Relaying::Cut);
    homed_on(&node, B).await;
    // The ask runs each ASKED_EVERY; wait for the decision it applies.
    let (t, by) = (tokio::time::Instant::now(), homing_by());
    while t.elapsed() < by && node.fallback_of(&seed.id()) != Some(Away::Unanswered) {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        node.fallback_of(&seed.id()),
        Some(Away::Unanswered),
        "the page says the own is not answering, not that the own is plain-HTTP"
    );
    assert_eq!(
        pressed_on(&node).await,
        "/?view=computer&moved-public",
        "the button says she moved to the public way"
    );
}

/// **A keeper who takes away a group's last relay is met nowhere near it
/// again** (KEEP_GOING_AT_HOME 5): Ada made house at A, asked as `new
/// --relay` asks, and sits at A, her build's way cut. The keeper's act names
/// no relay at all. A new link does not carry A; the way back, she is homed
/// at the build's; the way cut again past `LEFT_AFTER` and iroh's slowest
/// re-check, she does not go back to A. Planted, watched red: `keep_group`
/// leaving A in `Held::asked`, and she went back. While no other relay
/// answers iroh keeps the home it has, map or no map (iroh-1.0.2
/// socket.rs:1967), so this waits for the way back rather than assert her
/// gone from A at once. No command writes the empty set yet (`Host::meet`
/// reads on empty): the act goes in through the rail, and a one-second
/// round reads it.
#[tokio::test(start_paused = true)]
async fn a_keeper_taking_away_the_last_relay_is_met_nowhere_near_it_again() {
    on_wire::relay(A, Relaying::Up);
    on_wire::relay(B, Relaying::Cut);
    let (dir, key) = (
        tempfile::tempdir().unwrap(),
        SigningKey::from_bytes(&[4; 32]),
    );
    let node = on_wire::relayed(key.clone(), dir.path(), B, &[], Duration::from_secs(1)).await;
    let name = |n: &str| svrngs_node::group::Name::parse(n).unwrap();
    let seed = svrngs_node::group::draw(name("house"), name("Ada"), key.verifying_key());
    let at_own = [A.parse::<Url>().unwrap()];
    node.host().meet_at(&at_own).await;
    node.host().keep_group(&seed, None, &at_own).unwrap();
    homed_on(&node, A).await;

    node.rail()
        .commit(&seed.id(), &svrngs_core::MeetsAt(Default::default()))
        .expect("the journal takes it")
        .expect("the keeper's act counts");
    let link = node.host().link(&seed.id(), false, None, None).unwrap();
    assert!(
        !link.token.relays.contains(&at_own[0]),
        "a new link still carries A: {:?}",
        link.token.relays
    );
    on_wire::relay(B, Relaying::Up);
    homed_on(&node, B).await;
    on_wire::relay(B, Relaying::Cut);
    let (away, by) = (tokio::time::Instant::now(), homing_by());
    while away.elapsed() < by {
        assert_ne!(
            home(&node).as_deref(),
            Some(A),
            "back at A {:?} after the cut",
            away.elapsed()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// **A person who cannot wait presses and this computer moves now**
/// (KEEP_GOING_AT_HOME 7): the fallback cut, the button asks it once, it
/// does not answer, and Ada is homed at her group's own relay inside the
/// `LEFT_AFTER` asks the automatic move waits out, and stays, an ask already
/// under way undoing nothing; the round fired says so in the group's record
/// at once, where the next would be a minute off.
#[tokio::test(start_paused = true)]
async fn pressed_with_the_fallback_cut_this_computer_moves_now() {
    on_wire::relay(A, Relaying::Up);
    on_wire::relay(B, Relaying::Up);
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let (_dir, node, _seed) = ada_on_wire(B, A, every).await;
    homed_on(&node, B).await;
    on_wire::relay(B, Relaying::Cut);
    let t = tokio::time::Instant::now();
    assert_eq!(pressed_on(&node).await, "/?view=computer&moved");
    homed_on(&node, A).await;
    let took = t.elapsed();
    eprintln!("TIERS pressed, at its own: {took:?}");
    let automatic = svrngs_node::tiers::ASKED_EVERY * svrngs_node::tiers::LEFT_AFTER;
    assert!(
        took < automatic,
        "{took:?}, no sooner than the automatic move"
    );
    let group = node.rail().namespaces().unwrap().remove(0);
    let journal = node
        .rail()
        .group_dir(&group)
        .unwrap()
        .join("ring_oplog.jsonl");
    let says = |l: &str| l.contains(A) && !l.contains("\"relays\"");
    let round = tokio::time::timeout(Duration::from_secs(5), async {
        while !(std::fs::read_to_string(&journal).unwrap().lines()).any(says) {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    round.await.expect("the round fired says where she is now");
    while t.elapsed() < took + svrngs_node::tiers::ASKED_EVERY + Duration::from_secs(6) {
        assert_eq!(
            home(&node).as_deref(),
            Some(A),
            "not kept at A {:?} after the press",
            t.elapsed()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// **An ask under way when a person pressed does not undo the press**
/// (KEEP_GOING_AT_HOME 7; `tiers.rs` `pressed`): Ada is registered at her
/// build's fallback, and the way to it is cut 1.5 s before an ask is due, so
/// the ask waits on a registration while the press, its one question taking
/// `ANSWERS_WITHIN`, leaves the fallback. That ask ends a miss short of
/// `LEFT_AFTER`, the kind a reached fallback is kept through; the tier stays
/// the group's own for as long as it takes to end and be decided on. Read
/// from the tier held, since iroh keeps the home it has when the map changes
/// under it. Planted, watched failing: `pressed` always false, and the tier
/// went back to the fallback.
#[tokio::test(start_paused = true)]
async fn an_ask_under_way_does_not_undo_a_press() {
    on_wire::relay(A, Relaying::Up);
    on_wire::relay(B, Relaying::Up);
    let every = svrngs_node::tiers::ASKED_EVERY;
    // The asks fall every ASKED_EVERY from the node's start.
    let started = tokio::time::Instant::now();
    let round = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let (_dir, node, _seed) = ada_on_wire(B, A, round).await;
    homed_on(&node, B).await;
    let lead = Duration::from_millis(1500);
    let asks = started.elapsed().as_millis() / every.as_millis() + 2;
    let due = started + every * asks as u32 - lead;
    tokio::time::sleep_until(due).await;
    on_wire::relay(B, Relaying::Cut);
    assert_eq!(pressed_on(&node).await, "/?view=computer&moved");
    let until = due + lead + svrngs_node::tiers::REGISTERED_WITHIN + Duration::from_secs(4);
    while tokio::time::Instant::now() < until {
        let held = node.tier_held();
        assert!(held.contains("tier: Own"), "{held}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// **A press with the fallback answering moves nobody**, and says so.
#[tokio::test(start_paused = true)]
async fn pressed_with_the_fallback_answering_nothing_moves() {
    on_wire::relay(A, Relaying::Up);
    on_wire::relay(B, Relaying::Up);
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let (_dir, node, _seed) = ada_on_wire(B, A, every).await;
    homed_on(&node, B).await;
    assert_eq!(pressed_on(&node).await, "/?view=computer&stayed");
    tokio::time::sleep(Duration::from_secs(5)).await;
    homed_on(&node, B).await;
}

/// **The log says which relay carried an action** (KEEP_GOING_AT_HOME 20):
/// Sam, a member whose endpoint has no way but the relay, opens what Ada
/// lends, and the door's line for it names that relay. Why she is there is
/// the tier's: `tiers` traces each move at debug.
#[tokio::test]
async fn the_door_says_which_relay_carried_an_action() {
    let (said, _heard) = common::heard("door=info");
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let (_relay, url) = relay(RelayServer::new(any)).await;
    let (_dir, node, dialer, conn, label) = sam_reaches_ada_at(&url, 0).await;
    let reply = ask(&conn, &label, "").await;
    assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
    let lines = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let line = lines.lines().find(|l| l.contains("door: let in"));
    let line = line.unwrap_or_else(|| panic!("no line: {lines}"));
    assert!(line.contains(&url), "{line}");
    // apps/jellyfin/check/trio.py reads the fields before it, in this order.
    assert!(line.contains(" relayed=Some(true) rtt_ms="), "{line}");
    dialer.close().await;
    node.endpoint().close().await;
}

/// **A relay with a tiny rate throttles a lender for real** (share-refusal;
/// RELAY_COMMONS design 3, "The signal" and "The refusal"): an open relay
/// held to a megabyte a second with a 64 KiB burst, a member pulling a
/// range through it, so the lender's own endpoint is the connection the
/// relay reports rate limited (iroh's `relay_conns_ratelimited`, read by
/// `Host::watch_share`); the door then answers the next range over that
/// relay `429`, `x-svrngs-refused: share`, in the design's words, while a
/// request under the door's line passes. Planted, watched red: `Net::ratelimited`
/// answering zero, where the range after the pull was served again and the
/// poll timed out.
#[tokio::test]
async fn a_relay_with_a_tiny_rate_throttles_a_lender_and_the_door_refuses_a_range() {
    let (said, _heard) = common::heard("share=debug");
    let mut held = RelayServer::new("127.0.0.1:0".parse::<SocketAddr>().unwrap());
    let mut share = ClientRateLimit::new(NonZeroU32::new(1_000_000).unwrap());
    share.max_burst_bytes = NonZeroU32::new(64 * 1024);
    held.limits.client_rx = Some(share);
    let (_relay, url) = relay(held).await;
    let (_dir, node, dialer, conn, label) = sam_reaches_ada_at(&url, 256 * 1024).await;
    // The pull: its bytes go up through the relay until the bucket is spent.
    let pulled = ask(&conn, &label, "Range: bytes=0-\r\n").await;
    assert!(pulled.starts_with("HTTP/1.1 206"), "{pulled}");
    assert!(pulled.len() > 256 * 1024, "whole: {}", pulled.len());
    // The next range is refused once the watch, each second, has read the report.
    let refused = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let answered = ask(&conn, &label, "Range: bytes=0-\r\n").await;
            if answered.starts_with("HTTP/1.1 429") {
                return answered;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
    .await
    .expect("the range over the spent relay was refused");
    assert!(refused.contains("x-svrngs-refused: share"), "{refused}");
    assert!(refused.contains("has used its share of the public way for now"));
    let small = ask(&conn, &label, "").await;
    assert!(
        small.starts_with("HTTP/1.1 200") && small.ends_with("ok"),
        "{small}"
    );
    // The signal was the report of the build's relay this computer held.
    let lines = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let seen = lines
        .lines()
        .any(|l| l.contains("count=1") && l.contains("at_build=true"));
    assert!(seen, "the relay's report was seen: {lines}");
    dialer.close().await;
    node.endpoint().close().await;
}

/// Ada's computer homed at `url`, her build's relay, lending `telly` (a
/// [`serving`] of `big`) to her group, and Sam, let in, connected to her
/// through `url` alone: a member's endpoint with no way but the relay.
async fn sam_reaches_ada_at(
    url: &str,
    big: usize,
) -> (
    tempfile::TempDir,
    Node,
    iroh::Endpoint,
    iroh::endpoint::Connection,
    Label,
) {
    let (dir, node) = ada(url, url).await;
    homed(&node, url).await;
    let label = lent_to_sam(&node, serving(big).await);
    let dialer = sam_dialer(url, Default::default()).await;
    let at = EndpointAddr::new(node.endpoint().id()).with_relay_url(url.parse().unwrap());
    let conn = dialer.connect(at, svrngs_node::alpn::ALPN).await.unwrap();
    (dir, node, dialer, conn, label)
}

/// `GET /lend/<label>/` over `conn`, `extra` a header line (or none), the
/// answer read whole: a member's request through the door, on the stream.
async fn ask(conn: &iroh::endpoint::Connection, label: &Label, extra: &str) -> String {
    let (mut send, mut recv) = conn.open_bi().await.unwrap();
    let head =
        format!("GET /lend/{label}/ HTTP/1.1\r\nHost: x\r\n{extra}Connection: close\r\n\r\n");
    send.write_all(head.as_bytes()).await.unwrap();
    String::from_utf8_lossy(&recv.read_to_end(1 << 23).await.unwrap()).into_owned()
}

/// **A computer whose groups name no relay meets at n0's while ours does
/// not answer** (n0-emergency; RELAY_COMMONS 1: "n0's public relays come in
/// only while ours does not answer"): Ada's group names none of its own,
/// her build's relay is B, and n0's is on the wire. While B answers she is
/// homed at B and holds no n0's; B stopped, she is homed at n0's; B back at
/// its address, at B again, n0's out of the endpoint. Planted, watched red:
/// `tiers::behind` answering none, and she was not homed at n0's within
/// 76 s; the emergency held at the build's tier, and the first assertion
/// failed.
#[tokio::test(start_paused = true)]
async fn a_group_with_none_of_its_own_meets_at_n0_s_while_ours_does_not_answer() {
    let (n0, _dir, node, seed) = ada_beside_n0().await;
    node.host().keep_group(&seed, None, &[]).unwrap();
    homed_on(&node, B).await;
    let held = node.relays_held();
    assert!(
        !held.contains(&n0),
        "n0's held while ours answers: {held:?}"
    );
    assert_eq!(
        node.fallback_of(&seed.id()),
        None,
        "met at ours says nothing"
    );
    on_wire::relay(B, Relaying::Down);
    homed_on(&node, &n0).await;
    assert_eq!(
        node.fallback_of(&seed.id()),
        Some(Away::Emergency),
        "the home page says the group is met at n0's"
    );
    // A person who presses while ours is down is told "backup", not
    // "nowhere": this computer moved to n0's (KEEP_GOING_AT_HOME 7).
    assert_eq!(pressed_on(&node).await, "/?view=computer&backup");
    on_wire::relay(B, Relaying::Up);
    homed_on(&node, B).await;
    let held = node.relays_held();
    assert!(
        !held.contains(&n0),
        "n0's held once ours answers again: {held:?}"
    );
    assert_eq!(node.fallback_of(&seed.id()), None);
}

/// **A group whose own relay is down along with ours meets at n0's**
/// (the audit, director 2026-11-19; RELAY_COMMONS 2: n0's is "below ours
/// and below a group's own", in the endpoint "only while none of those
/// answers"): Ada's group names an HTTPS relay of its own and her build's
/// relay is B, n0's on the wire. Both answering, she is homed at her own
/// and holds no n0's; both stopped, she is homed at n0's and her page says
/// the group is met there; her own back, at her own again, n0's out of the
/// endpoint. Planted, watched red: the decision as n0-emergency left it,
/// n0's only for a computer whose groups name none of their own, and she
/// was never homed at n0's.
#[tokio::test(start_paused = true)]
async fn a_group_whose_own_relay_is_down_with_ours_meets_at_n0_s() {
    let own = "https://relay.example.org/";
    let (n0, _dir, node, seed) = ada_beside_n0().await;
    on_wire::relay(own, Relaying::Up);
    let at_own = [own.parse::<Url>().unwrap()];
    node.host().keep_group(&seed, None, &at_own).unwrap();
    homed_on(&node, own).await;
    assert!(
        !node.relays_held().contains(&n0),
        "{:?}",
        node.relays_held()
    );
    on_wire::relay(own, Relaying::Down);
    on_wire::relay(B, Relaying::Down);
    homed_on(&node, &n0).await;
    let held = node.relays_held();
    assert!(
        held.iter().all(|r| r.ends_with(".relay.n0.iroh.link/")),
        "n0's alone, none of ours beside them: {held:?}"
    );
    assert_eq!(
        node.fallback_of(&seed.id()),
        Some(Away::Emergency),
        "the home page says the group is met at n0's"
    );
    on_wire::relay(own, Relaying::Up);
    homed_on(&node, own).await;
    assert!(
        !node.relays_held().contains(&n0),
        "{:?}",
        node.relays_held()
    );
    assert_eq!(node.fallback_of(&seed.id()), None);
}

/// **A link minted before the operator's switch is not kept at n0's** (the
/// #134 gap, rescoped by n0-emergency to "n0 only in the emergency tier"):
/// a joiner's link carries one of n0's relays and one of ours; the group is
/// kept with the build's relays and n0's left out of the file it keeps
/// (`Host::keep_group`), so it meets at ours while ours answers and never
/// at n0's as its own. Planted, watched red: `keep_group` filtering the
/// build's relays alone (the code before this unit), and n0's was filed.
#[tokio::test(start_paused = true)]
async fn a_link_minted_at_n0_s_is_not_kept_as_the_group_s_own() {
    let (n0, _dir, node, seed) = ada_beside_n0().await;
    let link: Vec<Url> = [&n0, B].iter().map(|r| r.parse().unwrap()).collect();
    node.host().keep_group(&seed, None, &link).unwrap();
    let file = node.rail().group_dir(&seed.id()).unwrap().join("relay");
    assert!(!file.exists(), "n0's filed as the group's own: {file:?}");
    homed_on(&node, B).await;
    let held = node.relays_held();
    assert!(!held.iter().any(|r| r.contains("iroh.link")), "{held:?}");
}

/// Ada's computer on this test's wire, her build's relay B, one of n0's
/// relays up beside it, and a group named house founded by another computer
/// (a joiner keeps it by a link): what the two n0-emergency tests start
/// from, and that relay's URL.
async fn ada_beside_n0() -> (String, tempfile::TempDir, Node, svrngs_node::group::Seed) {
    let n0 = svrngs_node::relay::n0_relays()
        .urls::<Vec<RelayUrl>>()
        .remove(0)
        .to_string();
    on_wire::relay(&n0, Relaying::Up);
    on_wire::relay(B, Relaying::Up);
    let (dir, key) = (
        tempfile::tempdir().unwrap(),
        SigningKey::from_bytes(&[6; 32]),
    );
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let node = on_wire::relayed(key.clone(), dir.path(), B, &[], every).await;
    let name = |n: &str| svrngs_node::group::Name::parse(n).unwrap();
    let founder = SigningKey::from_bytes(&[7; 32]);
    let seed = svrngs_node::group::draw(name("house"), name("Ada"), founder.verifying_key());
    (n0, dir, node, seed)
}

/// A server on 127.0.0.1, and its port, that answers a request with
/// `Range:` a 206 of `big` bytes and any other 200 `ok`: what a lend
/// serves in the door's scenes over a real relay.
async fn serving(big: usize) -> u16 {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut asked, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = asked.read(&mut buf).await.unwrap_or(0);
            let ranged = String::from_utf8_lossy(&buf[..n])
                .to_ascii_lowercase()
                .contains("range:");
            let (status, body) = if ranged {
                ("206 Partial Content", vec![b'r'; big])
            } else {
                ("200 OK", b"ok".to_vec())
            };
            let head = format!(
                "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            let _ = asked.write_all(head.as_bytes()).await;
            let _ = asked.write_all(&body).await;
        }
    });
    port
}

/// **A fallback this computer cannot register at is left for its own**
/// (review F1): Ada's group names A; her build's fallback B answers the
/// ask and is no relay, so her endpoint is never connected there. She is
/// homed at A, and stays there for most of the next 90 s, the moves back
/// held off. Planted, watched red at 7a35fe1a: the ask's answer alone
/// held the fallback, and she was not homed at A within 56 s; and the
/// hold-off at 0, at A 30% of the time.
#[tokio::test(start_paused = true)]
async fn a_fallback_this_computer_cannot_register_at_is_left() {
    on_wire::relay(A, Relaying::Up);
    on_wire::relay(B, Relaying::Pings);
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let (_dir, node, _seed) = ada_on_wire(B, A, every).await;
    eprintln!(
        "TIERS unregistered at the fallback, at its own: {} ms",
        homed_on(&node, A).await
    );
    let (t, mut at_a) = (tokio::time::Instant::now(), 0);
    while t.elapsed() < Duration::from_secs(90) {
        tokio::time::sleep(Duration::from_millis(500)).await;
        at_a += usize::from(home(&node).as_deref() == Some(A));
    }
    eprintln!("TIERS at its own {at_a} of 180 half-seconds");
    assert!(at_a >= 108, "at A only {at_a} of 180 half-seconds");
}

/// The own relay and the build's, on the wire: names, never dialled.
const A: &str = "http://127.0.0.1:1/";
const B: &str = "http://127.0.0.1:2/";

/// [`ada`] on this test's wire, her round every `every`, her relays the
/// wire's, with house's seed, whose `id()` names it.
async fn ada_on_wire(build: &str, own: &str, every: Duration) -> (tempfile::TempDir, Node, Seed) {
    let (dir, key) = (
        tempfile::tempdir().unwrap(),
        SigningKey::from_bytes(&[3; 32]),
    );
    let node = on_wire::relayed(key.clone(), dir.path(), build, &[], every).await;
    let name = |n: &str| svrngs_node::group::Name::parse(n).unwrap();
    let seed = svrngs_node::group::draw(name("house"), name("Ada"), key.verifying_key());
    let at_own = [own.parse::<Url>().unwrap()];
    node.host()
        .keep_group(&seed, None, &at_own)
        .expect("Ada keeps house, met at her own relay");
    (dir, node, seed)
}

/// The relay `node`'s address names now.
fn home(node: &Node) -> Option<String> {
    node.net()
        .addr()
        .relay_urls()
        .next()
        .map(ToString::to_string)
}

/// The relay `node` is homed at on the wire, once it is `want`, waited for
/// up to [`homing_by`] on the test's clock: the wait in ms.
async fn homed_on(node: &Node, want: &str) -> u128 {
    let (t, by) = (tokio::time::Instant::now(), homing_by());
    while t.elapsed() < by {
        if home(node).as_deref() == Some(want) {
            return t.elapsed().as_millis();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("not homed at {want} within {by:?}");
}

/// "Check the connection now" pressed on `node`'s home page, its routes
/// asked in this process: where the page is sent, `/?` and what happened.
async fn pressed_on(node: &Node) -> String {
    let (browser, home) = (
        on_wire::Browser::of(node),
        format!("svrngs.localhost:{}", node.api_addr().port()),
    );
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let path = browser.path(&home, "/connection", &own);
    let proof = browser.proof("POST", &home, &path, &own, "").await;
    let asked = format!("POST {path} HTTP/1.1\r\nHost: {home}\r\n{own}{proof}Content-Length: 0\r\nConnection: close\r\n\r\n");
    let (reply, _) = on_wire::raw(browser.api(), &asked).await;
    let to = reply.lines().find_map(|l| {
        l.to_ascii_lowercase()
            .strip_prefix("location: ")
            .map(str::to_string)
    });
    to.unwrap_or_else(|| panic!("no page to go to: {reply}"))
}

/// **A seek over the relay's share waits behind no more than the page lets
/// a lender queue** (demo/runs/phone-home/2026-11-20, "What it found" 2):
/// a relay holding Ada's computer to the share, 125,000 bytes a second
/// (`crates/svrngs/src/relay.rs` `SHARE_RATE`; a 64 KiB burst in place of
/// its 8 MiB), and Sam's dial through it, built as the page builds
/// `paths`. First witness the exact connection credit with a controlled
/// QUIC sender. Then open two HTTP ranges through Ada's real door; the
/// seek must return before either lookahead is allowed to finish.
/// Ada's group meets at its own plain-HTTP relay, her build's answering
/// nowhere, so the share watch, which reads only the build's relay,
/// refuses nothing. Elapsed time is diagnostic only.
async fn a_seek_over_the_share(way: &str, paths: iroh::endpoint::QuicTransportConfig) {
    let part = svrngs_node::dial::QUEUED as usize / 2;
    let mut held = RelayServer::new("127.0.0.1:0".parse::<SocketAddr>().unwrap());
    let mut share = ClientRateLimit::new(NonZeroU32::new(125_000).unwrap());
    share.max_burst_bytes = NonZeroU32::new(64 * 1024);
    held.limits.client_rx = Some(share);
    let (_relay, url) = relay(held).await;
    let (_dir, node) = ada("http://127.0.0.1:9/", &url).await;
    homed(&node, &url).await;
    let (finish, held) = tokio::sync::watch::channel(false);
    let label = lent_to_sam(&node, serving_each(part, held).await);
    let dialer = sam_dialer(&url, paths).await;
    tokio::time::timeout(Duration::from_secs(45), connection_credit(&url, &dialer))
        .await
        .expect("the credit probe must make progress over the relay");
    let at = EndpointAddr::new(node.endpoint().id()).with_relay_url(url.parse().unwrap());
    let conn = dialer.connect(at, svrngs_node::alpn::ALPN).await.unwrap();
    let head = |range: &str| {
        format!(
            "GET /lend/{label}/ HTTP/1.1\r\nHost: x\r\nRange: {range}\r\nConnection: close\r\n\r\n"
        )
    };
    let ask = async |range: &str| {
        let (mut send, recv) = conn.open_bi().await.unwrap();
        send.write_all(head(range).as_bytes()).await.unwrap();
        (send, recv)
    };
    tokio::time::timeout(Duration::from_secs(45), async {
        let (_first, mut first) = ask("bytes=0-").await;
        let (_second, mut second) = ask(&format!("bytes={part}-")).await;
        async fn started(stream: &mut iroh::endpoint::RecvStream) {
            let mut head = Vec::new();
            while !head.ends_with(b"\r\n\r\n") && head.len() < 4096 {
                let mut byte = [0];
                stream.read_exact(&mut byte).await.unwrap();
                head.push(byte[0]);
            }
            assert!(head.starts_with(b"HTTP/1.1 206") && head.ends_with(b"\r\n\r\n"), "{head:?}");
            let mut first_byte = [0];
            stream.read_exact(&mut first_byte).await.unwrap();
            assert_eq!(&first_byte, b"r");
        }
        tokio::join!(started(&mut first), started(&mut second));
        let (_seek, mut seek) = ask("bytes=0-1").await;
        let asked = Instant::now();
        // Drain just the finite prefixes already offered, to return credit
        // even if they occupy the whole window. The tails stay gated.
        let mut prefix_a = vec![0; part - 1];
        let mut prefix_b = vec![0; part - 1];
        let (a, b, reply) = tokio::join!(
            first.read_exact(&mut prefix_a),
            second.read_exact(&mut prefix_b),
            seek.read_to_end(4096),
        );
        a.unwrap();
        b.unwrap();
        assert_eq!(prefix_a, vec![b'r'; part - 1]);
        assert_eq!(prefix_b, prefix_a);
        let reply = reply.unwrap();
        let response = String::from_utf8(reply).unwrap();
        assert!(response.starts_with("HTTP/1.1 206"), "{response:?}");
        assert_eq!(response.split_once("\r\n\r\n").unwrap().1, "ok");
        eprintln!("the seek over {way}: both lookaheads started; seek arrived in {} ms before releasing their tails", asked.elapsed().as_millis());
        finish.send(true).unwrap();
        let (a, b) = tokio::join!(first.read_to_end(1), second.read_to_end(1));
        assert_eq!(a.unwrap(), b"!");
        assert_eq!(b.unwrap(), b"!");
    })
    .await
    .expect("the seek and lookahead prefixes must arrive before the lookahead tails are released");
    dialer.close().await;
    node.endpoint().close().await;
}

/// Fill exactly `QUEUED` on two streams without returning any credit. A
/// third stream must be unwritable until the reader drains the lookahead.
async fn connection_credit(url: &str, dialer: &iroh::Endpoint) {
    const ALPN: &[u8] = b"seek-credit/0";
    let part = svrngs_node::dial::QUEUED as usize / 2;
    let lender = build_relayed_endpoint(
        SecretKey::from_bytes(&[11; 32]),
        vec![ALPN.to_vec()],
        &at(url),
    )
    .await
    .unwrap();
    lender.online().await;
    let addr = EndpointAddr::new(lender.id()).with_relay_url(url.parse().unwrap());
    let (blocked, at_limit) = tokio::sync::oneshot::channel();
    let (_server, _client) = tokio::join!(
        async {
            let conn = lender.accept().await.unwrap().await.unwrap();
            let (mut a, _) = conn.accept_bi().await.unwrap();
            let (mut b, _) = conn.accept_bi().await.unwrap();
            let (mut seek, _) = conn.accept_bi().await.unwrap();
            let bytes = vec![b'r'; part];
            a.write_all(&bytes).await.unwrap();
            b.write_all(&bytes).await.unwrap();
            let extra = {
                let mut write = std::pin::pin!(seek.write(b"s"));
                std::future::poll_fn(|cx| std::task::Poll::Ready(write.as_mut().poll(cx))).await
            };
            assert!(extra.is_pending(), "seek write exceeded QUEUED: {extra:?}");
            blocked.send(()).unwrap();
            seek.write_all(b"s").await.unwrap();
            seek.finish().unwrap();
            // The seek can be acknowledged with lookahead bytes still in flight.
            conn
        },
        async {
            let conn = dialer.connect(addr, ALPN).await.unwrap();
            let open = async || {
                let (mut send, recv) = conn.open_bi().await.unwrap();
                send.write_all(b"?").await.unwrap();
                send.finish().unwrap();
                recv
            };
            let (mut a, mut b, mut seek) = (open().await, open().await, open().await);
            at_limit.await.unwrap();
            let mut bytes = vec![0; part];
            a.read_exact(&mut bytes).await.unwrap();
            assert_eq!(bytes, vec![b'r'; part]);
            b.read_exact(&mut bytes).await.unwrap();
            assert_eq!(bytes, vec![b'r'; part]);
            assert_eq!(seek.read_to_end(1).await.unwrap(), b"s");
            eprintln!(
                "the seek over QUIC: {} B filled the connection; the next stream waited for credit",
                2 * part
            );
            conn
        }
    );
    lender.close().await;
}

/// [`a_seek_over_the_share`] over a page's endpoint's dial to a lender
/// (`dial::held`, web/src/lib.rs `reach`).
#[tokio::test(flavor = "multi_thread")]
async fn a_seek_over_the_share_on_a_pages_dial_waits_behind_what_it_lets_be_queued() {
    let queued = Some(svrngs_node::dial::QUEUED);
    a_seek_over_the_share("a page's dial", svrngs_node::dial::held(None, queued)).await;
}

/// [`a_seek_over_the_share`] over a page's data endpoint, whose relay is
/// its direct path's backup (`quick_paths`, web/src/lib.rs `direct_start`).
#[tokio::test(flavor = "multi_thread")]
async fn a_seek_over_the_share_on_a_data_endpoint_waits_behind_what_it_lets_be_queued() {
    let queued = Some(svrngs_node::dial::QUEUED);
    a_seek_over_the_share("a data endpoint", svrngs_node::webrtc::quick_paths(queued)).await;
}

/// Sam let in to Ada's group, and the lend `telly`, the service at `port`,
/// lent to it: its label.
fn lent_to_sam(node: &Node, port: u16) -> Label {
    let group = node.rail().namespaces().unwrap().remove(0);
    let (sam, rail) = (SigningKey::from_bytes(&[9; 32]), node.rail());
    let ada = svrngs_node::actor_of(rail.signer());
    svrngs_core::tests_support::let_in(
        (&group, &ada),
        (&sam, "sam", svrngs_node::Device::Computer),
        |mint| {
            rail.commit(&group, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(rail.commit(&group, redeem).unwrap().unwrap()),
    );
    let lends = svrngs_node::lend::lends_dir(node.rail().root());
    std::fs::create_dir_all(&lends).unwrap();
    let lent = serde_json::json!({ "port": port, "kind": "app", "limit": 3, "groups": [group] });
    std::fs::write(
        lends.join("telly.json"),
        svrngs_node::node::approving(&lent),
    )
    .unwrap();
    Label::of(node.endpoint().id().as_bytes(), "telly")
}

/// Sam's endpoint, its only way `url`, its connections built with `paths`.
async fn sam_dialer(url: &str, paths: iroh::endpoint::QuicTransportConfig) -> iroh::Endpoint {
    let map = iroh::RelayMap::from(url.parse::<RelayUrl>().unwrap());
    iroh::endpoint::Builder::new(iroh::endpoint::presets::Minimal)
        .crypto_provider(svrngs_node::guest::ring_crypto_provider())
        .secret_key(SecretKey::from_bytes(&[9; 32]))
        .relay_mode(iroh::RelayMode::Custom(map))
        .clear_ip_transports()
        .transport_config(paths)
        .bind()
        .await
        .unwrap()
}

/// Each lookahead serves its prefix, then waits for the test to release
/// its last byte. The seek has no such dependency.
async fn serving_each(prefix: usize, held: tokio::sync::watch::Receiver<bool>) -> u16 {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut asked, _)) = listener.accept().await {
            let mut held = held.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                let mut n = 0;
                while !buf[..n].ends_with(b"\r\n\r\n") && n < buf.len() {
                    let read = asked.read(&mut buf[n..]).await.unwrap_or(0);
                    if read == 0 {
                        return;
                    }
                    n += read;
                }
                let head = String::from_utf8_lossy(&buf[..n]).to_ascii_lowercase();
                let ranged = head.contains("range:");
                let lookahead = ranged && !head.contains("range: bytes=0-1\r\n");
                let body = if lookahead {
                    vec![b'r'; prefix]
                } else {
                    b"ok".to_vec()
                };
                let status = if ranged {
                    "206 Partial Content"
                } else {
                    "200 OK"
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    body.len() + usize::from(lookahead)
                );
                let _ = asked.write_all(head.as_bytes()).await;
                let _ = asked.write_all(&body).await;
                if lookahead && held.wait_for(|released| *released).await.is_ok() {
                    let _ = asked.write_all(b"!").await;
                }
            });
        }
    });
    port
}
