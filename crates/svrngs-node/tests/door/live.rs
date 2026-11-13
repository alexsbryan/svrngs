// SPDX-License-Identifier: AGPL-3.0-or-later
//! What stays open through the door: a WebSocket's upgrade carried both
//! ways, and every answer still streaming, or socket still open, cut once
//! the decision that let its asker in no longer would (LENDING_SCOPE
//! §"Existing streams and app sessions"). The decisions are judged on
//! their own (`src/door/tests.rs`: `guest_of`, `guest_admitted`,
//! `member_for`; eudoxia's `rules::recheck`); the cut, `withdrawn` asked
//! every `udp::SWEEP` on a real clock, only a crossing shows, so its bars
//! (2 s, and 1.5 s past a pass's lapse) stay here. Each step names the test
//! it came from, retired at its move (docs/seams.tsv).

use super::*;
use svrngs_core::alpn::Alpn;

/// A request through the bridge at `at` whose answer's head has come: the
/// socket, its body still to come.
async fn opened(at: SocketAddr, head: &str) -> tokio::net::TcpStream {
    let mut sock = tokio::net::TcpStream::connect(at).await.unwrap();
    sock.write_all(format!("{head}Host: local\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let (mut seen, mut byte) = (Vec::new(), [0u8; 1]);
    while !seen.ends_with(b"\r\n\r\n") {
        let n = tokio::time::timeout(Duration::from_secs(20), sock.read(&mut byte))
            .await
            .expect("the door answers")
            .unwrap();
        assert_eq!(n, 1, "{:?}", String::from_utf8_lossy(&seen));
        seen.push(byte[0]);
    }
    let seen = String::from_utf8_lossy(&seen).into_owned();
    assert!(
        seen.starts_with("HTTP/1.1 20") || seen.starts_with("HTTP/1.1 101"),
        "{seen}"
    );
    sock
}

/// How long `sock` carries on before it ends, read as it comes, up to `within`.
async fn ends_within(sock: &mut tokio::net::TcpStream, within: Duration) -> Option<Duration> {
    let began = std::time::Instant::now();
    let mut buf = [0u8; 4096];
    let ended = async { while sock.read(&mut buf).await.is_ok_and(|n| n > 0) {} };
    tokio::time::timeout(within, ended)
        .await
        .ok()
        .map(|()| began.elapsed())
}

/// Planted, each watched failing when its test was its own: the door
/// stripping `Upgrade` as hop-by-hop (door.rs:128 before u5-2), which the
/// service answers 426; the answer's body and the upgraded copy carried on
/// to their end, unwatched; the lapse unchecked once a request is let in
/// (authority spike, "withdraw-expiry-inflight": ran 2.9 s past it); no
/// watch on a guest's use.
#[tokio::test]
async fn a_live_answer_is_carried_and_cut_once_its_asker_is_not_let_in() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let two = Duration::from_secs(2);
    let world = World::new(|_| {}).await;
    let (stream, _) = origins::streaming_origin().await;
    let (live, live_heads) = origins::upgrade_origin().await;
    for (name, port) in [
        ("shelf", stream.port()),
        ("radio", stream.port()),
        ("live", live.port()),
    ] {
        world.lend(
            name,
            serde_json::json!({ "port": port, "kind": "k", "groups": [NS] }),
        );
    }
    let bo = world.as_key(2, Alpn::Member).await;

    // a_websocket_upgrade_is_carried_both_ways (main.rs:1060-1092).
    let mut socket = tokio::net::TcpStream::connect(bo).await.unwrap();
    socket
        .write_all(
            b"GET /lend/live/socket HTTP/1.1\r\nHost: local\r\nConnection: Upgrade\r\n\
              Upgrade: websocket\r\nSec-WebSocket-Version: 13\r\n\r\n",
        )
        .await
        .unwrap();
    let (mut head, mut byte) = (Vec::new(), [0u8; 1]);
    while !head.ends_with(b"\r\n\r\n") {
        let n = tokio::time::timeout(Duration::from_secs(20), socket.read(&mut byte))
            .await
            .expect("the door answers the upgrade")
            .unwrap();
        assert!(
            n == 1,
            "the door closed before its answer ended: {:?}",
            String::from_utf8_lossy(&head)
        );
        head.push(byte[0]);
    }
    let head = String::from_utf8_lossy(&head).into_owned();
    assert!(
        head.starts_with("HTTP/1.1 101"),
        "the upgrade must be answered 101: {head}"
    );
    assert_eq!(header_values(&head, "upgrade"), vec!["websocket"], "{head}");
    for word in ["ping", "and again"] {
        socket.write_all(word.as_bytes()).await.unwrap();
        let mut echo = vec![0u8; word.len()];
        tokio::time::timeout(Duration::from_secs(20), socket.read_exact(&mut echo))
            .await
            .expect("the service's echo comes back")
            .unwrap();
        assert_eq!(echo, word.as_bytes());
    }
    let asked = last(&live_heads);
    assert_eq!(
        header_values(&asked, "x-svrngs-person"),
        vec!["bo"],
        "{asked}"
    );

    // a_stream_ends_within_two_seconds_of_its_group_taken_away_and_no_other
    // (scope.rs:224-252): the shelf's answer and the socket cut; the radio's not.
    let mut shelf = opened(bo, "GET /lend/shelf/stream HTTP/1.1\r\n").await;
    let mut radio = opened(bo, "GET /lend/radio/stream HTTP/1.1\r\n").await;
    for name in ["shelf", "live"] {
        let name = LendName::parse(name).unwrap();
        assert!(lend::set_audience(world.dir.path(), &name, vec!["yard".into()]).unwrap());
    }
    let cut = ends_within(&mut shelf, two).await;
    let closed = ends_within(&mut socket, two).await;
    eprintln!("the shelf's answer ended after {cut:?}, the socket after {closed:?}");
    assert!(
        cut.is_some(),
        "the shelf's answer still carries on after 2 s"
    );
    assert!(closed.is_some(), "the socket is still open after 2 s");
    let mut more = [0u8; 64];
    let carried = tokio::time::timeout(two, radio.read(&mut more)).await;
    assert!(
        matches!(carried, Ok(Ok(n)) if n > 0),
        "the radio stopped: {carried:?}"
    );
    assert_eq!(ask(bo, "GET", "/lend/radio/now", "").await.status, 200);

    // a_stream_on_a_pass_ends_within_a_sweep_of_its_pass_or_link_lapsing
    // (scope.rs:283-330).
    let radio_label = world.label("radio");
    let (holder, endpoint) = (
        world.as_key(19, Alpn::Lend).await,
        world.as_key(10, Alpn::Lend).await,
    );
    let lapse = unix_now() + 2;
    let bos = |until| {
        svrngs_core::pass::Pass::sign(
            &signing(2),
            &actor_of(&signing(19)),
            (radio_label.clone(), vec![]),
            until,
        )
    };
    let (short, long) = (bos(lapse), bos(lapse + 3600));
    let mut link = long.link(&actor_of(&signing(10)));
    link.until = lapse;
    link.sig = svrngs_core::sign_message(&signing(19), &link.message());
    let head = |pass: &svrngs_core::pass::Pass, link: Option<&svrngs_core::pass::Pass>| {
        let link = link.map_or(String::new(), |l| {
            format!("x-svrngs-link: {}\r\n", l.encode())
        });
        format!(
            "GET /lend/{radio_label}/stream HTTP/1.1\r\nx-svrngs-pass: {}\r\n{link}",
            pass.encode()
        )
    };
    let mut lapsing = opened(holder, &head(&short, None)).await;
    let mut linked = opened(endpoint, &head(&long, Some(&link))).await;
    let mut lasting = opened(holder, &head(&long, None)).await;
    let lapses_at = UNIX_EPOCH + Duration::from_secs(lapse as u64);
    let within = lapses_at
        .duration_since(SystemTime::now())
        .unwrap_or_default()
        + Duration::from_millis(1500);
    let ended = async |sock: &mut tokio::net::TcpStream| {
        let ended = ends_within(sock, within).await;
        ended.map(|_| SystemTime::now().duration_since(lapses_at))
    };
    let (cut, cut_linked) = tokio::join!(ended(&mut lapsing), ended(&mut linked));
    eprintln!("after the lapse, the pass's answer ended {cut:?}, the link's {cut_linked:?}");
    for (what, cut) in [("the pass's", cut), ("the link's", cut_linked)] {
        let cut =
            cut.unwrap_or_else(|| panic!("{what} answer still carries on 1.5 s past its lapse"));
        assert!(cut.is_ok(), "{what} answer ended before its lapse: {cut:?}");
    }
    let carried = tokio::time::timeout(two, lasting.read(&mut more)).await;
    assert!(
        matches!(carried, Ok(Ok(n)) if n > 0),
        "the stream on the lasting pass stopped: {carried:?}"
    );

    // a_guests_stream_ends_when_its_group_or_visit_no_longer_lets_them_in
    // (guests.rs:395-433): Sam streams the radio, cut once it is lent to no
    // group; lent back, cut once ada stops the visit.
    let visit = world.visit(&["radio"]);
    let sams_holder = world.as_key(11, Alpn::Lend).await;
    let sams = format!(
        "GET /lend/{radio_label}/stream HTTP/1.1\r\n{}",
        pass(SAM, 11, &radio_label, &[], unix_now() + 3600)
    );
    let radio_name = LendName::parse("radio").unwrap();
    let mut sam = opened(sams_holder, &sams).await;
    assert!(lend::set_audience(world.dir.path(), &radio_name, Vec::new()).unwrap());
    assert!(
        ends_within(&mut sam, two).await.is_some(),
        "the guest's answer carries on, its group taken away"
    );
    assert!(lend::set_audience(world.dir.path(), &radio_name, vec![NS.into()]).unwrap());
    let mut sam = opened(sams_holder, &sams).await;
    let stop = svrngs_core::tests_support::Fixture {
        act: RailAct::Correct {
            corrects: visit,
            replacement: None,
        },
    };
    world.a.rail().commit(NS, &stop).unwrap().unwrap();
    assert!(
        ends_within(&mut sam, two).await.is_some(),
        "the guest's answer carries on, the visit stopped"
    );
    world.a.endpoint().close().await;
}
