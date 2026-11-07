//! Where a computer meets its groups, in tiers (relay fallbacks; svrn-docs
//! `rings/current/GROUP_RELAYS.md` §7): iroh dials a member at a relay this
//! computer's endpoint does not hold, which every dial across tiers rests
//! on; and a computer whose group names a relay of its own meets there,
//! takes the build's relay when its own stops answering, and gives it back
//! once its own answers again (`svrngs_node::tiers`). Each relay plain
//! HTTP on 127.0.0.1, as `svrngs relay --at` serves one, the build's a
//! second such relay standing in for n0's so nothing leaves this computer.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[path = "common/cuttable.rs"]
mod cuttable;
use cuttable::cuttable;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs_core::relay::Url;
use svrngs_node::iroh::{build_relayed_endpoint, EndpointAddr, RelayConfig, RelayUrl, SecretKey};
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

/// A plain-HTTP relay on 127.0.0.1 at `at` (port 0 for any), and its URL.
async fn relay(at: SocketAddr) -> (Server, String) {
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(at));
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
    let ((_r1, u1), (_r2, u2)) = (relay(any).await, relay(any).await);
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
#[tokio::test(flavor = "multi_thread")]
async fn a_group_meets_at_the_builds_fallback_while_it_answers() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_own, a), (build, _)) = (relay(any).await, relay(any).await);
    let b_at = build.http_addr().unwrap();
    let (way, cut) = cuttable(b_at).await;
    let (_dir, node) = ada(&way, &a).await;
    eprintln!(
        "TIERS at the build's fallback while it answers: {} ms",
        homed(&node, &way).await
    );
    // A loss of fewer asks than LEFT_AFTER is no move (review F2): the way
    // to B cut for 15 s, so one or two asks find Ada unregistered there.
    let away = Instant::now();
    cut.send_replace(true);
    while away.elapsed() < Duration::from_secs(45) {
        if away.elapsed() > Duration::from_secs(15) {
            cut.send_if_modified(std::mem::take);
        }
        let home = node
            .endpoint()
            .addr()
            .relay_urls()
            .next()
            .map(ToString::to_string);
        assert_ne!(
            home,
            Some(a.clone()),
            "at A {:?} after the cut",
            away.elapsed()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    build.shutdown().await.unwrap();
    eprintln!(
        "TIERS the fallback stopped, at its own: {} ms",
        homed(&node, &a).await
    );
    let _back = relay(b_at).await;
    eprintln!(
        "TIERS the fallback back, at the fallback: {} ms",
        homed(&node, &way).await
    );
    node.endpoint().close().await;
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
#[tokio::test(flavor = "multi_thread")]
async fn a_keeper_taking_away_the_last_relay_is_met_nowhere_near_it_again() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_own, a), (build, _)) = (relay(any).await, relay(any).await);
    let (way, cut) = cuttable(build.http_addr().unwrap()).await;
    cut.send_replace(true);
    let (dir, key) = (
        tempfile::tempdir().unwrap(),
        SigningKey::from_bytes(&[4; 32]),
    );
    let every = Duration::from_secs(1);
    let node = Node::start_every(
        dir.path().into(),
        key.clone(),
        &at(&way),
        &[],
        |_| Ok(None),
        any,
        every,
    )
    .await
    .unwrap();
    let name = |n: &str| svrngs_node::group::Name::parse(n).unwrap();
    let seed = svrngs_node::group::draw(name("house"), name("Ada"), key.verifying_key());
    let at_own = [a.parse::<Url>().unwrap()];
    node.host().meet_at(&at_own).await;
    node.host().keep_group(&seed, None, &at_own).unwrap();
    homed(&node, &a).await;

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
    cut.send_replace(false);
    homed(&node, &way).await;
    cut.send_replace(true);
    let (away, by) = (Instant::now(), homing_by());
    while away.elapsed() < by {
        let home = node.endpoint().addr();
        assert_ne!(
            home.relay_urls().next().map(ToString::to_string),
            Some(a.clone()),
            "back at A {:?} after the cut",
            away.elapsed()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    node.endpoint().close().await;
}

/// "Check the connection now" pressed on `node`'s home page: where the
/// page is sent, `/?` and what happened.
async fn pressed(node: &Node) -> String {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    let (browser, home) = (
        common::Browser::of(node),
        format!("svrngs.localhost:{}", node.api_addr().port()),
    );
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let path = browser.path(&home, "/connection", &own);
    let proof = browser.proof("POST", &home, &path, &own, "").await;
    let mut sock = tokio::net::TcpStream::connect(browser.api()).await.unwrap();
    let asked = format!("POST {path} HTTP/1.1\r\nHost: {home}\r\n{own}{proof}Content-Length: 0\r\nConnection: close\r\n\r\n");
    sock.write_all(asked.as_bytes()).await.unwrap();
    let mut reply = String::new();
    sock.read_to_string(&mut reply).await.unwrap();
    let to = reply.lines().find_map(|l| {
        l.to_ascii_lowercase()
            .strip_prefix("location: ")
            .map(str::to_string)
    });
    to.unwrap_or_else(|| panic!("no page to go to: {reply}"))
}

/// **A person who cannot wait presses and this computer moves now**
/// (KEEP_GOING_AT_HOME 7): the fallback cut, the button asks it once, it
/// does not answer, and Ada is homed at her group's own relay inside the
/// `LEFT_AFTER` asks the automatic move waits out, and stays, an ask already
/// under way undoing nothing; the round fired says so in the group's record
/// at once, where the next would be a minute off.
#[tokio::test(flavor = "multi_thread")]
async fn pressed_with_the_fallback_cut_this_computer_moves_now() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_own, a), (build, _)) = (relay(any).await, relay(any).await);
    let (way, cut) = cuttable(build.http_addr().unwrap()).await;
    let (_dir, node) = ada(&way, &a).await;
    homed(&node, &way).await;
    cut.send_replace(true);
    let t = Instant::now();
    assert_eq!(pressed(&node).await, "/?view=computer&moved");
    homed(&node, &a).await;
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
    let says = |l: &str| l.contains(&a) && !l.contains("\"relays\"");
    let round = tokio::time::timeout(Duration::from_secs(5), async {
        while !(std::fs::read_to_string(&journal).unwrap().lines()).any(says) {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    round.await.expect("the round fired says where she is now");
    while t.elapsed() < took + svrngs_node::tiers::ASKED_EVERY + Duration::from_secs(6) {
        let home = node.endpoint().addr();
        let at_a = home.relay_urls().any(|r| r.to_string() == a);
        assert!(at_a, "not kept at A {:?} after the press", t.elapsed());
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    node.endpoint().close().await;
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
#[tokio::test(flavor = "multi_thread")]
async fn an_ask_under_way_does_not_undo_a_press() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_own, a), (build, _)) = (relay(any).await, relay(any).await);
    let (way, cut) = cuttable(build.http_addr().unwrap()).await;
    let every = svrngs_node::tiers::ASKED_EVERY;
    // The asks fall every ASKED_EVERY from the node's start.
    let started = Instant::now();
    let (_dir, node) = ada(&way, &a).await;
    homed(&node, &way).await;
    let lead = Duration::from_millis(1500);
    let asks = started.elapsed().as_millis() / every.as_millis() + 2;
    let due = started + every * asks as u32 - lead;
    tokio::time::sleep_until(due.into()).await;
    cut.send_replace(true);
    assert_eq!(pressed(&node).await, "/?view=computer&moved");
    let until = due + lead + svrngs_node::tiers::REGISTERED_WITHIN + Duration::from_secs(4);
    while Instant::now() < until {
        let held = node.tier_held();
        assert!(held.contains("tier: Own"), "{held}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    node.endpoint().close().await;
}

/// **A press with the fallback answering moves nobody**, and says so.
#[tokio::test(flavor = "multi_thread")]
async fn pressed_with_the_fallback_answering_nothing_moves() {
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let ((_own, a), (_build, b)) = (relay(any).await, relay(any).await);
    let (_dir, node) = ada(&b, &a).await;
    homed(&node, &b).await;
    assert_eq!(pressed(&node).await, "/?view=computer&stayed");
    tokio::time::sleep(Duration::from_secs(5)).await;
    homed(&node, &b).await;
    node.endpoint().close().await;
}

/// **The log says which relay carried an action** (KEEP_GOING_AT_HOME 20):
/// Sam, a member whose endpoint has no way but the relay, opens what Ada
/// lends, and the door's line for it names that relay. Why she is there is
/// the tier's: `tiers` traces each move at debug.
#[tokio::test]
async fn the_door_says_which_relay_carried_an_action() {
    let (said, _heard) = common::heard("door=info");
    let any: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let (_relay, url) = relay(any).await;
    let (_dir, node) = ada(&url, &url).await;
    homed(&node, &url).await;
    let group = node.rail().namespaces().unwrap().remove(0);
    let sam = SigningKey::from_bytes(&[9; 32]);
    let admit = svrngs_core::tests_support::Fixture {
        act: svrngs_node::RailAct::Admit {
            person: svrngs_node::Person::parse("sam").unwrap(),
            key: svrngs_node::actor_of(&sam),
            invite: None,
        },
    };
    node.rail().commit(&group, &admit).unwrap().unwrap();
    let port = pings()
        .await
        .rsplit(':')
        .next()
        .unwrap()
        .trim_end_matches('/')
        .parse::<u16>();
    let lends = svrngs_node::lend::lends_dir(node.rail().root());
    std::fs::create_dir_all(&lends).unwrap();
    let lent =
        serde_json::json!({ "port": port.unwrap(), "kind": "app", "limit": 3, "groups": [group] });
    std::fs::write(lends.join("telly.json"), lent.to_string()).unwrap();

    let map = iroh::RelayMap::from(url.parse::<RelayUrl>().unwrap());
    let dialer = iroh::endpoint::Builder::new(iroh::endpoint::presets::Minimal)
        .crypto_provider(svrngs_node::guest::ring_crypto_provider())
        .secret_key(SecretKey::from_bytes(&sam.to_bytes()))
        .relay_mode(iroh::RelayMode::Custom(map))
        .clear_ip_transports()
        .bind()
        .await
        .unwrap();
    let at = EndpointAddr::new(node.endpoint().id()).with_relay_url(url.parse().unwrap());
    let conn = dialer.connect(at, svrngs_node::alpn::ALPN).await.unwrap();
    let (mut send, mut recv) = conn.open_bi().await.unwrap();
    let label = svrngs_core::pass::Label::of(node.endpoint().id().as_bytes(), "telly");
    let ask = format!("GET /lend/{label}/ HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n");
    send.write_all(ask.as_bytes()).await.unwrap();
    let reply = recv.read_to_end(1 << 16).await.unwrap();
    let reply = String::from_utf8_lossy(&reply);
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

/// A server on 127.0.0.1 that answers every request 200 and is no relay:
/// svrngs's ask of it passes (`tiers::answers`, `GET /ping`), and an
/// endpoint never registers there, as at a captive portal taking TCP 443.
async fn pings() -> String {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((mut asked, _)) = listener.accept().await {
            let _ = asked.read(&mut [0; 1024]).await;
            let _ = asked
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n")
                .await;
        }
    });
    url
}

/// **A fallback this computer cannot register at is left for its own**
/// (review F1): Ada's group names A; her build's fallback B answers the
/// ask and is no relay, so her endpoint is never connected there. She is
/// homed at A, and stays there for most of the next 90 s, the moves back
/// held off. Planted, watched red at 7a35fe1a: the ask's answer alone
/// held the fallback, and she was not homed at A within 56 s; and the
/// hold-off at 0, at A 30% of the time.
#[tokio::test(flavor = "multi_thread")]
async fn a_fallback_this_computer_cannot_register_at_is_left() {
    let ((_own, a), b) = (relay("127.0.0.1:0".parse().unwrap()).await, pings().await);
    let (_dir, node) = ada(&b, &a).await;
    eprintln!(
        "TIERS unregistered at the fallback, at its own: {} ms",
        homed(&node, &a).await
    );
    // Held off doubling, she is at A for most of 90 s, not 30% (F1 residual).
    let (t, mut at_a) = (Instant::now(), 0);
    while t.elapsed() < Duration::from_secs(90) {
        tokio::time::sleep(Duration::from_millis(500)).await;
        at_a += usize::from(
            node.endpoint()
                .addr()
                .relay_urls()
                .any(|r| r.to_string() == a),
        );
    }
    eprintln!("TIERS at its own {at_a} of 180 half-seconds");
    assert!(at_a >= 108, "at A only {at_a} of 180 half-seconds");
    node.endpoint().close().await;
}
