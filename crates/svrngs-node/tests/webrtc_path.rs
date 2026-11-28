//! A page's own half, natively (webrtc-node; RELAY_COMMONS design 4): the
//! row's check — a native peer over the wire carries a stream, then falls
//! back to the relay within 4 s of its channel closing. The plants: no
//! custom address, where it stays relayed; the relay stopped, where it
//! stalls.
//!
//! The peer is the spike's `page/src/lib.rs`@0753e2d in Rust: two
//! endpoints as a page has (the signalling one, relay only, and a fresh
//! data one whose wire carries the bytes), a str0m offerer whose channel
//! the wire's datagrams cross, and the two dials of the spike's Q1 — the
//! data endpoint dials WebRTC alone, then again with the relay beside it,
//! since iroh takes a custom path only on a connection whose first path is
//! not the relay.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use iroh::TransportAddr;
use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use str0m::change::SdpAnswer;
use str0m::channel::ChannelId;
use str0m::net::{Protocol, Receive};
use str0m::{Candidate, Event, IceConnectionState, Input, Output, Rtc, RtcConfig};
use svrngs_node::alpn::ALPN;
use svrngs_node::iroh::{EndpointAddr, RelayConfig, SecretKey};
use svrngs_node::node::Node;
use svrngs_node::webrtc::wire;
use svrngs_node::SigningKey;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

#[path = "common/unjudged.rs"]
mod unjudged;

/// The node homed at `url`, with group `house` founded here, and that
/// group's id. Direct connections are on for the network this host is on,
/// as the home page turns them: an offer is answered only on a network the
/// lender enabled (webrtc-network), and this test's lender is this host;
/// `None`, could-not-judge, on a host whose network cannot be read.
async fn node_at(url: &str) -> Option<(tempfile::TempDir, Node, String)> {
    let network = unjudged::network()?;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(svrngs_node::webrtc::direct::DIRECT_FILE),
        serde_json::to_vec(&vec![network]).unwrap(),
    )
    .unwrap();
    let key = SigningKey::from_bytes(&[3; 32]);
    let relay = RelayConfig {
        relay_urls: vec![url.into()],
        n0_services: false,
        loopback_only: false,
    };
    let node = Node::start(
        dir.path().to_path_buf(),
        key.clone(),
        &relay,
        |_| Ok(None),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let name = |n: &str| svrngs_node::group::Name::parse(n).unwrap();
    let seed = svrngs_node::group::draw(name("house"), name("Ada"), key.verifying_key());
    node.host().keep_group(&seed, None, &[]).unwrap();
    Some((dir, node, seed.id()))
}

/// `key` let into `group` as `person`, by a link this node mints (a test
/// admits a member by hand, PLAN §4, U1).
fn let_in(node: &Node, group: &str, key: &SigningKey, person: &str) {
    svrngs_core::tests_support::let_in_by(group, key, person, |act| {
        let act = svrngs_core::tests_support::Fixture { act };
        node.rail().commit(group, &act).unwrap().unwrap().lines()[0]
            .id
            .clone()
    });
}

/// An endpoint of a page's: relay only, `wire` added when given.
async fn bind(secret: SecretKey, url: &str, wire: Option<Arc<wire::Wire>>) -> iroh::Endpoint {
    let mut builder = iroh::endpoint::Builder::new(iroh::endpoint::presets::Minimal)
        .crypto_provider(svrngs_node::guest::ring_crypto_provider())
        .secret_key(secret)
        .relay_mode(iroh::RelayMode::custom([url.parse().unwrap()]))
        // A browser's endpoint has no IP transports at all (`ip` is
        // `cfg(not(wasm_browser))`): only the relay, and the wire.
        .clear_ip_transports();
    if let Some(wire) = wire {
        builder = builder.add_custom_transport(wire);
    }
    builder.bind().await.unwrap()
}

/// The offer POSTed on the WebRTC protocol to the node at `url`, its
/// answer's JSON body read: what the phone's endpoint does, with the pass
/// `pass` it presents (none, where the plant asks without one).
async fn posted(
    signal: &iroh::Endpoint,
    node: iroh::EndpointId,
    url: &str,
    body: &serde_json::Value,
    pass: Option<&str>,
) -> String {
    let at = EndpointAddr::from_parts(node, [TransportAddr::Relay(url.parse().unwrap())]);
    let conn = signal
        .connect(at, svrngs_node::alpn::Alpn::WebRtc.bytes())
        .await
        .unwrap();
    let (mut send, mut recv) = conn.open_bi().await.unwrap();
    let body = serde_json::to_vec(body).unwrap();
    let held = pass.map_or(String::new(), |pass| {
        format!("{}: {pass}\r\n", svrngs_node::door::PASS_HEADER)
    });
    let head = format!(
        "POST {} HTTP/1.1\r\nHost: svrngs\r\ncontent-type: application/json\r\ncontent-length: {}\r\n{held}Connection: close\r\n\r\n",
        svrngs_node::webrtc::OFFER_ROUTE,
        body.len()
    );
    send.write_all(head.as_bytes()).await.unwrap();
    send.write_all(&body).await.unwrap();
    let got = String::from_utf8_lossy(&recv.read_to_end(1 << 20).await.unwrap()).into_owned();
    got.split_once("\r\n\r\n")
        .unwrap_or_else(|| panic!("no body: {got}"))
        .1
        .to_string()
}

/// One stream over the connection: a member asks what this computer
/// lends, which the door answers to a key standing in a group it holds.
async fn asked(conn: &iroh::endpoint::Connection) -> String {
    let (mut send, mut recv) = conn.open_bi().await.unwrap();
    let head = format!(
        "GET {} HTTP/1.1\r\nHost: x\r\n{}: {}\r\nConnection: close\r\n\r\n",
        svrngs_node::door::LENDS_ROUTE,
        svrngs_node::version::SCOPE_HEADER,
        svrngs_node::version::VERSION
    );
    send.write_all(head.as_bytes()).await.unwrap();
    String::from_utf8_lossy(&recv.read_to_end(1 << 20).await.unwrap()).into_owned()
}

/// The connection's selected path, waited for until it is `kind`:
/// "webrtc" or "relay".
async fn selected(conn: &iroh::endpoint::Connection, kind: &str, by: Duration) -> Duration {
    let t = Instant::now();
    while t.elapsed() < by {
        let paths = conn.paths();
        if let Some(p) = paths.iter().find(|p| p.is_selected()) {
            let now = match p.remote_addr() {
                TransportAddr::Custom(_) => "webrtc",
                TransportAddr::Relay(_) => "relay",
                _ => "other",
            };
            if now == kind {
                return t.elapsed();
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let now: Vec<String> = conn
        .paths()
        .iter()
        .map(|p| format!("{:?}", p.remote_addr()))
        .collect();
    panic!("the selected path is not {kind} within {by:?}: {now:?}");
}

/// The offerer's loop, the node's `run` mirrored: the channel's datagrams
/// to the wire as the node's, and the wire's for the node written to the
/// channel while it takes them. `close` closes the channel, as a phone
/// leaving the Wi-Fi loses it.
fn pump(
    mut rtc: Rtc,
    socket: UdpSocket,
    node: wire::PeerKey,
    arrived: UnboundedSender<(wire::PeerKey, Vec<u8>)>,
    outgoing: std::sync::mpsc::Receiver<Vec<u8>>,
    close: Arc<AtomicBool>,
) -> Result<String, String> {
    let e = |e: &dyn std::fmt::Display| e.to_string();
    let mut buf = vec![0u8; 2000];
    let mut channel: Option<ChannelId> = None;
    rtc.handle_input(Input::Timeout(Instant::now()))
        .map_err(|x| e(&x))?;
    loop {
        if close.load(Ordering::Relaxed) {
            if let Some(id) = channel {
                rtc.direct_api().close_data_channel(id);
            }
        }
        let timeout = loop {
            match rtc.poll_output().map_err(|x| e(&x))? {
                Output::Timeout(t) => break t,
                Output::Transmit(t) => {
                    socket
                        .send_to(&t.contents, t.destination)
                        .map_err(|x| e(&x))?;
                }
                Output::Event(Event::IceConnectionStateChange(
                    IceConnectionState::Disconnected,
                )) => return Ok("ice disconnected".into()),
                Output::Event(Event::ChannelOpen(id, _)) => channel = Some(id),
                Output::Event(Event::ChannelData(d)) => {
                    let _ = arrived.send((node, d.data));
                }
                Output::Event(Event::ChannelClose(_)) => return Ok("channel closed".into()),
                Output::Event(_) => {}
            }
        };
        let mut wrote = false;
        while let Ok(datagram) = outgoing.try_recv() {
            if let Some(mut ch) = channel.and_then(|id| rtc.channel(id)) {
                let _ = ch.write(true, &datagram);
                wrote = true;
            }
        }
        if wrote {
            continue;
        }
        let wait = timeout
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(2));
        if wait.is_zero() {
            rtc.handle_input(Input::Timeout(Instant::now()))
                .map_err(|x| e(&x))?;
            continue;
        }
        socket.set_read_timeout(Some(wait)).map_err(|x| e(&x))?;
        buf.resize(2000, 0);
        let input = match socket.recv_from(&mut buf) {
            Ok((n, source)) => {
                buf.truncate(n);
                Input::Receive(
                    Instant::now(),
                    Receive {
                        proto: Protocol::Udp,
                        source,
                        destination: socket.local_addr().map_err(|x| e(&x))?,
                        contents: buf.as_slice().try_into().map_err(|x| format!("{x:?}"))?,
                    },
                )
            }
            Err(x) if matches!(x.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                Input::Timeout(Instant::now())
            }
            Err(x) => return Err(x.to_string()),
        };
        rtc.handle_input(input).map_err(|x| e(&x))?;
    }
}

/// **A page's own wire carries a stream, and iroh falls back to the relay
/// within 4 s of the channel closing** (webrtc-node; RELAY_COMMONS design
/// 4): the node homed at a real relay, the page's key and the worker's let
/// into its group, the offerer's channel open, the fresh data endpoint's
/// dials as the spike's Q1, a member's ask over the connection — on
/// `webrtc*` — then the channel closed, and the ask again, on `relay*`.
/// Planted, watched red: the dial with no custom address, where the
/// connection stays relayed; and the relay stopped before the close, where
/// the fallback never comes.
#[tokio::test(flavor = "multi_thread")]
async fn a_native_peer_carries_a_stream_over_the_wire_and_falls_back_in_four_seconds() {
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(
        "127.0.0.1:0".parse::<std::net::SocketAddr>().unwrap(),
    ));
    let relay = Server::spawn(config).await.expect("a relay on 127.0.0.1");
    let url = format!("http://{}/", relay.http_addr().unwrap());
    let Some((dir, node, group)) = node_at(&url).await else {
        return;
    };
    let by = Instant::now() + Duration::from_secs(15);
    while node
        .net()
        .addr()
        .relay_urls()
        .next()
        .map(ToString::to_string)
        .as_deref()
        != Some(url.as_str())
    {
        assert!(Instant::now() < by, "not homed at {url}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let page = SigningKey::from_bytes(&[9; 32]);
    let worker = SigningKey::from_bytes(&[10; 32]);
    let_in(&node, &group, &page, "page");
    let_in(&node, &group, &worker, "worker");
    // A lend the phone plays, and the pass the page signs for it: an offer
    // is answered only to a dialer holding one (webrtc-page).
    let lend = svrngs_node::lend::Lend {
        name: svrngs_node::lend::LendName::parse("page").unwrap(),
        source: svrngs_node::lend::Source::Port(std::num::NonZeroU16::new(8096).unwrap()),
        kind: svrngs_node::lend::Program::parse("page").unwrap(),
        terms: svrngs_node::lend::Terms {
            since: None,
            for_programs: false,
            audience: svrngs_node::lend::Audience::Selected(vec![group.clone()]),
            limit: std::num::NonZeroU16::new(4),
            refused: Vec::new(),
            upload: None,
            reads: Vec::new(),
            sign_in_with_svrngs: None,
            asks: None,
            sends: Vec::new(),
            approved: None,
            keep: None,
            revision: None,
        },
    };
    svrngs_node::lend::declare(dir.path(), &lend, None).unwrap();
    let until = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 600;
    let pass = svrngs_core::pass::Pass::sign(
        &page,
        &svrngs_node::actor_of(&page),
        (lend.label(node.endpoint().id().as_bytes()), Vec::new()),
        until,
    )
    .encode();

    // The page's two endpoints: the signalling one, and the fresh data one
    // whose wire carries the app's bytes.
    let signal = bind(SecretKey::from_bytes(&page.to_bytes()), &url, None).await;
    let me = worker.verifying_key().to_bytes();
    let (arrived, arrivals) = unbounded_channel();
    let (out, outgoing) = std::sync::mpsc::channel();
    let send: wire::Send = Arc::new(move |_to: &wire::PeerKey, datagram: &[u8]| {
        let _ = out.send(datagram.to_vec());
    });
    let data = bind(
        SecretKey::from_bytes(&worker.to_bytes()),
        &url,
        Some(wire::Wire::new(&me, arrivals, send)),
    )
    .await;

    // The offerer's channel, offered; the node's answer taken; the loop on
    // a thread of its own, as a page's is.
    let node_key = *node.endpoint().id().as_bytes();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut rtc = RtcConfig::new().build(Instant::now());
    rtc.add_local_candidate(Candidate::host(socket.local_addr().unwrap(), "udp").unwrap());
    let mut change = rtc.sdp_api();
    change.add_channel("the app's bytes".into());
    let (offer, pending) = change.apply().expect("an offer");
    let asked_offer = serde_json::json!({ "key": data.id().to_string(), "offer": offer });
    // Planted, watched red: the offer with no pass, refused; then the pass
    // its holder holds, answered (webrtc-page; the offer's one gate).
    let bare = posted(&signal, node.endpoint().id(), &url, &asked_offer, None).await;
    assert!(
        bare.contains("open this again from your group"),
        "an offer with no pass must be refused: {bare}"
    );
    let answered = posted(
        &signal,
        node.endpoint().id(),
        &url,
        &asked_offer,
        Some(&pass),
    )
    .await;
    let answer: SdpAnswer = serde_json::from_str(&answered).expect("an answer");
    rtc.sdp_api().accept_answer(pending, answer).unwrap();
    let close = Arc::new(AtomicBool::new(false));
    let told_close = close.clone();
    std::thread::spawn(move || {
        let why = pump(rtc, socket, node_key, arrived, outgoing, told_close);
        eprintln!("page: the offerer ended: {why:?}");
    });

    // The fresh data endpoint dials WebRTC alone, then again with the
    // relay beside it (the spike's Q1); the first carries nothing.
    let node_id = node.endpoint().id();
    let custom = TransportAddr::Custom(wire::addr_of(&node_key));
    let first = data
        .connect(EndpointAddr::from_parts(node_id, [custom.clone()]), ALPN)
        .await
        .unwrap();
    let conn = data
        .connect(
            EndpointAddr::from_parts(
                node_id,
                [custom, TransportAddr::Relay(url.parse().unwrap())],
            ),
            ALPN,
        )
        .await
        .unwrap();
    first.close(0u32.into(), b"the second carries on");

    let on = selected(&conn, "webrtc", Duration::from_secs(10)).await;
    eprintln!("PAGE the wire was selected in {on:?}");
    let answer = asked(&conn).await;
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(answer.contains("\"lends\""), "{answer}");

    // The channel closed: within 4 s iroh is on the relay, and a stream
    // still crosses.
    close.store(true, Ordering::Relaxed);
    let fell = selected(&conn, "relay", Duration::from_secs(4)).await;
    eprintln!("PAGE the fallback took {fell:?}");
    let answer = asked(&conn).await;
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
}
