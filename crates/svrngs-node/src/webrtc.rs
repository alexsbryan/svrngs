// SPDX-License-Identifier: AGPL-3.0-or-later
//! The WebRTC path (webrtc-node; RELAY_COMMONS design 4): iroh's custom
//! transport over a datagram wire, so a phone on the lender's Wi-Fi
//! reaches it by WebRTC while the relay stays the fallback, and nothing
//! above iroh changes.
//!
//! [`wire`] is the transport itself, ported from the WebRTC spikes
//! (svrn-docs `rings/current/webrtc-transport-spike/`, CHARTER "Reuse");
//! the page's half compiles the same file by `#[path]` (the port map,
//! `demo/runs/relay-commons/PORT_MAP.md`).
//!
//! The node's half is the str0m peer of the spike's `native/src/main.rs`
//! (`answer`, `run`, `lan_address`), reached by a page's offer over
//! [`Alpn::WebRtc`](crate::alpn::Alpn::WebRtc): one peer per offer, on a
//! UDP socket of this computer's own network, host candidates only
//! (CHARTER, "What the spikes learned"), keyed by the data endpoint's key
//! the offer names — the fresh endpoint whose datagrams the channel
//! carries. The channel's datagrams go to iroh as that key's, and iroh's
//! datagrams for it into the channel ([`Route`]).
//!
//! Paths are given up after [`PATH_IDLE`] of silence with a keep-alive
//! each [`KEEP_ALIVE`] ([`quick_paths`]), set on this endpoint: the spike
//! measured iroh's own 15 s stall down to 3.2 s this way, and `network_
//! change()` does nothing for it (the spike's README, "The
//! pre-requisites" 1).

pub mod wire;

#[cfg(test)]
pub(crate) mod tests;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use iroh::EndpointId;
use str0m::change::SdpOffer;
use str0m::net::{Protocol, Receive};
use str0m::{Candidate, Event, IceConnectionState, Input, Output, Rtc, RtcConfig};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use wire::{PeerKey, Wire};

/// How long a path with no answer is given up after (the spike's
/// `PATH_IDLE_MS` 3000; RELAY_COMMONS design 4): inside any player's
/// buffer, and slack for a lossy Wi-Fi.
pub const PATH_IDLE: Duration = Duration::from_secs(3);

/// The keep-alive every path sends, a third of [`PATH_IDLE`] (the spike's
/// `ms / 3`), so a live path is never idle that long.
pub const KEEP_ALIVE: Duration = Duration::from_secs(1);

/// The path timing this computer's endpoint is built with ([`crate::guest`]'s
/// `build_relayed_endpoint_with`): both ends carry it, since the computer
/// sends the film and must find a dead path too (the spike's README).
pub fn quick_paths() -> iroh::endpoint::QuicTransportConfig {
    iroh::endpoint::QuicTransportConfig::builder()
        .default_path_max_idle_timeout(PATH_IDLE)
        .default_path_keep_alive_interval(KEEP_ALIVE)
        .build()
}

/// Where the page offers a path: the route the offer and answer cross on
/// [`Alpn::WebRtc`](crate::alpn::Alpn::WebRtc), JSON in and out, as the
/// spike's `S` stream was (native/src/main.rs:135-148).
pub const OFFER_ROUTE: &str = "/offer";

/// The node's endpoint: [`crate::guest::build_relayed_endpoint`] with this
/// computer's wire added and [`quick_paths`] given to it, and the handle
/// its offers are answered through (webrtc-node; RELAY_COMMONS design 4).
pub async fn endpoint(
    secret: crate::iroh::SecretKey,
    alpns: Vec<Vec<u8>>,
    cfg: &crate::iroh::RelayConfig,
) -> Result<(iroh::Endpoint, Arc<WebRtc>), String> {
    let me = *secret.public().as_bytes();
    let (webrtc, wire) = WebRtc::new(&me);
    let endpoint = crate::guest::build_relayed_endpoint_with(
        secret,
        alpns,
        cfg,
        Some(wire),
        Some(quick_paths()),
    )
    .await?;
    Ok((endpoint, webrtc))
}

/// The WebRTC half's handle: the wire's send side, and what an offer is
/// answered through. One [`Route`] owner is told of each peer's channel
/// (RUST_ARCH 3: one owner and messages; the spike's map was behind a
/// Mutex).
pub struct WebRtc {
    /// The datagrams the channel brought, each as the key it came from:
    /// the wire's receive side (its `Bound`).
    arrived: UnboundedSender<(PeerKey, Vec<u8>)>,
    /// What the routes' owner is told: a peer's channel, its end, and
    /// iroh's datagrams to send through it.
    routes: UnboundedSender<Route>,
}

/// What the routes' one owner is told.
enum Route {
    /// The channel for `key`, its peer running now.
    Opened(PeerKey, std::sync::mpsc::Sender<Vec<u8>>),
    /// `key`'s peer ended: iroh's datagrams for it are dropped again.
    Ended(PeerKey),
    /// One datagram to `key` from iroh's sender; dropped when no channel
    /// is open, as a lossy path drops.
    Send(PeerKey, Vec<u8>),
}

impl WebRtc {
    /// This computer's `me`, its wire for the endpoint's builder, and the
    /// handle its offers are answered through.
    pub fn new(me: &PeerKey) -> (Arc<Self>, Arc<Wire>) {
        // The provider str0m's Rtc is built with, installed once for the
        // process (str0m 0.25 `crypto::from_feature_flags`; the spike's
        // `native/src/main.rs:41`).
        str0m::crypto::from_feature_flags().install_process_default();
        let (arrived, arrivals) = unbounded_channel();
        let (routes, told) = unbounded_channel();
        tokio::spawn(routed(told));
        let send_routes = routes.clone();
        let send: wire::Send = Arc::new(move |to: &PeerKey, datagram: &[u8]| {
            let _ = send_routes.send(Route::Send(*to, datagram.to_vec()));
        });
        (
            Arc::new(Self { arrived, routes }),
            Wire::new(me, arrivals, send),
        )
    }

    /// The offer `offer` answered for the data endpoint `data` names, its
    /// SDP answer as JSON (the spike's `answer`): a str0m peer on a UDP
    /// socket of this computer's own network, host candidates only, its
    /// channel's datagrams fed to iroh as `data`'s and iroh's for `data`
    /// written to it, until it ends.
    #[expect(
        clippy::disallowed_methods,
        reason = "str0m's own clock: the peer's loop times itself, as the spike's"
    )]
    pub fn answer(&self, offer: &[u8], data: PeerKey) -> Result<String, String> {
        let offer: SdpOffer =
            serde_json::from_slice(offer).map_err(|e| format!("not an offer: {e}"))?;
        let mut rtc = RtcConfig::new().build(Instant::now());
        let socket =
            UdpSocket::bind(SocketAddr::new(lan_address(), 0)).map_err(|e| e.to_string())?;
        let local = socket.local_addr().map_err(|e| e.to_string())?;
        rtc.add_local_candidate(Candidate::host(local, "udp").map_err(|e| e.to_string())?);
        let answer = rtc
            .sdp_api()
            .accept_offer(offer)
            .map_err(|e| e.to_string())?;
        let (out, outgoing) = std::sync::mpsc::channel();
        let _ = self.routes.send(Route::Opened(data, out));
        let (arrived, routes) = (self.arrived.clone(), self.routes.clone());
        std::thread::spawn(move || {
            let why = run(rtc, socket, data, arrived, outgoing);
            let _ = routes.send(Route::Ended(data));
            tracing::debug!(target: "webrtc", peer = %hex::encode(data), ?why, "webrtc: a peer ended");
        });
        tracing::debug!(target: "webrtc", peer = %hex::encode(data), at = %local, "webrtc: an offer answered, its peer on a socket of this computer's own network");
        serde_json::to_string(&answer).map_err(|e| e.to_string())
    }
}

/// The routes' one owner: iroh's datagrams for a key go to its channel,
/// and one with no channel is dropped.
async fn routed(mut told: UnboundedReceiver<Route>) {
    let mut open = HashMap::<PeerKey, std::sync::mpsc::Sender<Vec<u8>>>::new();
    while let Some(route) = told.recv().await {
        match route {
            Route::Opened(key, out) => {
                tracing::debug!(target: "webrtc", peer = %hex::encode(key), "webrtc: a channel opened for the key");
                open.insert(key, out);
            }
            Route::Ended(key) => {
                tracing::debug!(target: "webrtc", peer = %hex::encode(key), "webrtc: a channel ended");
                open.remove(&key);
            }
            Route::Send(key, datagram) => {
                if let Some(out) = open.get(&key) {
                    let _ = out.send(datagram);
                }
            }
        }
    }
}

/// The WebRTC protocol's router: one offer answered per request, for a
/// dialer the acceptor admitted.
pub fn router(webrtc: Arc<WebRtc>) -> axum::Router {
    axum::Router::new()
        .route(OFFER_ROUTE, axum::routing::post(offer))
        .with_state(webrtc)
}

/// What an offer names: the data endpoint whose datagrams the channel
/// carries, and the offer itself (the spike's `S`, native/src/main.rs:135-148).
#[derive(serde::Deserialize)]
struct Offered {
    key: String,
    offer: serde_json::Value,
}

async fn offer(State(webrtc): State<Arc<WebRtc>>, Json(asked): Json<Offered>) -> Response {
    let Some(data) = asked.key.parse::<EndpointId>().ok().map(|k| *k.as_bytes()) else {
        return refused("the offer names no data key");
    };
    let offer = serde_json::to_vec(&asked.offer).unwrap_or_default();
    match webrtc.answer(&offer, data) {
        Ok(answer) => ([(header::CONTENT_TYPE, "application/json")], answer).into_response(),
        Err(why) => refused(&why),
    }
}

/// An offer this computer does not answer: `400` and why, as the spike's
/// `{"error": ...}` (native/src/main.rs:146).
fn refused(why: &str) -> Response {
    tracing::debug!(target: "webrtc", why, "webrtc: an offer refused");
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": why })),
    )
        .into_response()
}

/// This computer's address on its own network, where a phone on the same
/// Wi-Fi reaches it: [`crate::network::route`], the one read of the
/// outward route's address, else its own loopback, where a browser on the
/// same computer reaches it. Host candidates only, never STUN or TURN
/// (CHARTER, "What the spikes learned"; the trust floor's "this Wi-Fi").
fn lan_address() -> IpAddr {
    crate::network::route().unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
}

/// The str0m loop of the spike's `native/src/main.rs:233-291`, its first
/// tick given at once (spike 1, C): the channel's datagrams to the wire as
/// `peer`'s, and the wire's for `peer` written to the channel while it
/// takes them, dropped otherwise.
#[expect(
    clippy::disallowed_methods,
    reason = "str0m's own clock: the peer's loop times itself, as the spike's"
)]
fn run(
    mut rtc: Rtc,
    socket: UdpSocket,
    peer: PeerKey,
    arrived: UnboundedSender<(PeerKey, Vec<u8>)>,
    outgoing: std::sync::mpsc::Receiver<Vec<u8>>,
) -> Result<String, String> {
    let e = |e: &dyn std::fmt::Display| e.to_string();
    let mut buf = vec![0u8; 2000];
    let mut channel = None;
    rtc.handle_input(Input::Timeout(Instant::now()))
        .map_err(|x| e(&x))?;
    loop {
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
                )) => {
                    return Ok("ice disconnected".into());
                }
                Output::Event(Event::ChannelOpen(id, label)) => {
                    tracing::debug!(target: "webrtc", peer = %hex::encode(peer), %label, "webrtc: a data channel opened");
                    channel = Some(id);
                }
                Output::Event(Event::ChannelData(d)) => {
                    let _ = arrived.send((peer, d.data));
                }
                Output::Event(Event::ChannelClose(_)) => return Ok("channel closed".into()),
                Output::Event(_) => {}
            }
        };
        // What iroh gave for this peer, written while the channel takes it;
        // dropped otherwise, as a lossy path drops.
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
