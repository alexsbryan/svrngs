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
//! (`answer`, `run`), reached by a page's offer over
//! [`Alpn::WebRtc`](crate::alpn::Alpn::WebRtc): one peer per offer, on a
//! UDP socket of a network the lender enabled ([`direct`]; the operator's
//! UDP decision, 2026-11-19), at the private address [`crate::network::
//! pick`] gives there, host candidates only (CHARTER, "What the spikes
//! learned"), keyed by the data endpoint's key the offer names — the fresh
//! endpoint whose datagrams the channel carries. The channel's datagrams
//! go to iroh as that key's, and iroh's datagrams for it into the channel
//! ([`Route`]). At most [`MAX_ATTEMPTS`] peers are set up at once, and a
//! peer's socket closes on setup timeout, on the session's end, after
//! [`IDLE`] with no datagram either way, or when this computer is no
//! longer read on the network it was answered on.
//!
//! Paths are given up after [`PATH_IDLE`] of silence with a keep-alive
//! each [`KEEP_ALIVE`] ([`quick_paths`]), set on this endpoint: the spike
//! measured iroh's own 15 s stall down to 3.2 s this way, and `network_
//! change()` does nothing for it (the spike's README, "The
//! pre-requisites" 1).

pub mod direct;
pub mod wire;

#[cfg(test)]
pub(crate) mod tests;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
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

use crate::network::{Here, Network, Unread};
use wire::{PeerKey, Wire};

/// How long a path with no answer is given up after (the spike's
/// `PATH_IDLE_MS` 3000; RELAY_COMMONS design 4): inside any player's
/// buffer, and slack for a lossy Wi-Fi.
pub const PATH_IDLE: Duration = Duration::from_secs(3);

/// The keep-alive every path sends, a third of [`PATH_IDLE`] (the spike's
/// `ms / 3`), so a live path is never idle that long.
pub const KEEP_ALIVE: Duration = Duration::from_secs(1);

/// The most peers one computer sets up at once: a few phones around one
/// lender; past it, an offer waits for the public way (the operator,
/// 2026-11-19: "cap outstanding attempts").
const MAX_ATTEMPTS: usize = 4;

/// How long an answered offer's peer is given to open its channel: host
/// candidates answer in well under a second when the phone is on this
/// WiFi, so ten is slack, and a phone that never arrives closes its
/// socket rather than holding a peer forever.
const SETUP: Duration = Duration::from_secs(10);

/// How long a peer with no datagram either way is kept: iroh's keep-alive
/// ([`KEEP_ALIVE`]) keeps a live path's channel busy, so this closes a
/// socket nothing has used for half a minute.
const IDLE: Duration = Duration::from_secs(30);

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
    /// The file the networks the lender enabled direct connections on are
    /// kept in ([`direct::DIRECT_FILE`]): the home page turns them, and an
    /// offer is answered only on one.
    direct: PathBuf,
    /// How many peers are being set up or running now: [`MAX_ATTEMPTS`],
    /// past which an offer waits for the public way. One thread starts
    /// and one ends per peer, so an atomic is the one count.
    open: Arc<AtomicUsize>,
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
    /// This computer's `me`, the file its direct connections setting is
    /// kept in, its wire for the endpoint's builder, and the handle its
    /// offers are answered through.
    pub fn new(me: &PeerKey, direct: PathBuf) -> (Arc<Self>, Arc<Wire>) {
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
        let open = Arc::new(AtomicUsize::new(0));
        (
            Arc::new(Self {
                arrived,
                routes,
                direct,
                open,
            }),
            Wire::new(me, arrivals, send),
        )
    }

    /// The offer `offer` answered for the data endpoint `data` names, its
    /// SDP answer as JSON (the spike's `answer`): a str0m peer on a UDP
    /// socket of `here`'s network when the lender enabled direct
    /// connections there ([`direct::binds_at`]), at the private address
    /// picked there and never the outward route's, host candidates only,
    /// its channel's datagrams fed to iroh as `data`'s and iroh's for
    /// `data` written to it, until it ends. Else nothing is bound and why
    /// is said. Past [`MAX_ATTEMPTS`] peers at once, the offer waits.
    #[expect(
        clippy::disallowed_methods,
        reason = "str0m's own clock: the peer's loop times itself, as the spike's"
    )]
    pub fn answer(&self, offer: &[u8], data: PeerKey, here: &Here) -> Result<String, String> {
        let offer: SdpOffer =
            serde_json::from_slice(offer).map_err(|e| format!("not an offer: {e}"))?;
        if self.open.load(Ordering::Relaxed) >= MAX_ATTEMPTS {
            return Err(format!("{MAX_ATTEMPTS} direct connections are already being set up here; this one waits for the public way"));
        }
        let (network, address) = direct::binds_at(&self.direct, here)?;
        let mut rtc = RtcConfig::new().build(Instant::now());
        let socket =
            UdpSocket::bind(SocketAddr::new(address.into(), 0)).map_err(|e| e.to_string())?;
        let local = socket.local_addr().map_err(|e| e.to_string())?;
        rtc.add_local_candidate(Candidate::host(local, "udp").map_err(|e| e.to_string())?);
        let answer = rtc
            .sdp_api()
            .accept_offer(offer)
            .map_err(|e| e.to_string())?;
        let (out, outgoing) = std::sync::mpsc::channel();
        let _ = self.routes.send(Route::Opened(data, out));
        let (arrived, routes, open) =
            (self.arrived.clone(), self.routes.clone(), self.open.clone());
        open.fetch_add(1, Ordering::Relaxed);
        std::thread::spawn(move || {
            let why = run(
                rtc,
                socket,
                data,
                arrived,
                outgoing,
                network,
                || crate::network::here().network,
                crate::network::LOOK_EVERY,
            );
            let _ = routes.send(Route::Ended(data));
            open.fetch_sub(1, Ordering::Relaxed);
            tracing::debug!(target: "webrtc", peer = %hex::encode(data), ?why, "webrtc: a peer ended");
        });
        tracing::debug!(target: "webrtc", peer = %hex::encode(data), at = %local, "webrtc: an offer answered, its peer on a network the lender enabled");
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
        return refused("the offer names no data key".into());
    };
    let here = crate::network::now().await;
    let offer = serde_json::to_vec(&asked.offer).unwrap_or_default();
    match webrtc.answer(&offer, data, &here) {
        Ok(answer) => ([(header::CONTENT_TYPE, "application/json")], answer).into_response(),
        Err(why) => refused(why),
    }
}

/// An offer this computer does not answer: `400` and why, as the spike's
/// `{"error": ...}` (native/src/main.rs:146).
fn refused(why: String) -> Response {
    tracing::debug!(target: "webrtc", %why, "webrtc: an offer refused");
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": why })),
    )
        .into_response()
}

/// The str0m loop of the spike's `native/src/main.rs:233-291`, its first
/// tick given at once (spike 1, C): the channel's datagrams to the wire as
/// `peer`'s, and the wire's for `peer` written to the channel while it
/// takes them, dropped otherwise. The socket closes with the loop: no
/// channel within [`SETUP`] of the offer, no datagram either way for
/// [`IDLE`], and, checked each `every`, a `read` that is not the
/// `network` the socket was bound on — the phone moved off the Wi-Fi, or
/// the network cannot be read now, and a socket nothing can be said of is
/// not kept.
#[expect(
    clippy::disallowed_methods,
    reason = "str0m's own clock: the peer's loop times itself, as the spike's"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "one call per peer: what it runs on, what it carries, and the network it was answered on"
)]
fn run(
    mut rtc: Rtc,
    socket: UdpSocket,
    peer: PeerKey,
    arrived: UnboundedSender<(PeerKey, Vec<u8>)>,
    outgoing: std::sync::mpsc::Receiver<Vec<u8>>,
    network: Network,
    read: impl Fn() -> Result<Network, Unread>,
    every: Duration,
) -> Result<String, String> {
    let e = |e: &dyn std::fmt::Display| e.to_string();
    let mut buf = vec![0u8; 2000];
    let mut channel = None;
    let (answered, mut busy, mut looked) = (Instant::now(), Instant::now(), Instant::now());
    rtc.handle_input(Input::Timeout(Instant::now()))
        .map_err(|x| e(&x))?;
    loop {
        if channel.is_none() && answered.elapsed() > SETUP {
            return Ok("setup timed out".into());
        }
        if busy.elapsed() > IDLE {
            return Ok("idle".into());
        }
        if looked.elapsed() > every {
            looked = Instant::now();
            if read() != Ok(network) {
                return Ok("network changed".into());
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
                )) => {
                    return Ok("ice disconnected".into());
                }
                Output::Event(Event::ChannelOpen(id, label)) => {
                    tracing::debug!(target: "webrtc", peer = %hex::encode(peer), %label, "webrtc: a data channel opened");
                    channel = Some(id);
                }
                Output::Event(Event::ChannelData(d)) => {
                    busy = Instant::now();
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
            busy = Instant::now();
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
