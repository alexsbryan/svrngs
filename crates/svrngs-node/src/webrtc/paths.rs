// SPDX-License-Identifier: AGPL-3.0-or-later
//! What both ends of a WebRTC path spell alike (webrtc-node; webrtc-page;
//! RELAY_COMMONS design 4): the node ([`crate::webrtc`]) and the page
//! (web/src/lib.rs, which compiles this file by `#[path]`; the port map,
//! demo/runs/relay-commons/PORT_MAP.md) carry the same path timing — the
//! computer sends the film and must find a dead path too (the spike's
//! README, "The pre-requisites" 1) — and the same route the offer and
//! answer cross.

/// How long a path with no answer is given up after (the spike's
/// `PATH_IDLE_MS` 3000; RELAY_COMMONS design 4): inside any player's
/// buffer, and slack for a lossy Wi-Fi.
pub const PATH_IDLE: std::time::Duration = std::time::Duration::from_secs(3);

/// The keep-alive every path sends, a third of [`PATH_IDLE`] (the spike's
/// `ms / 3`), so a live path is never idle that long.
pub const KEEP_ALIVE: std::time::Duration = std::time::Duration::from_secs(1);

/// The path timing an endpoint of a WebRTC path is built with.
pub fn quick_paths() -> iroh::endpoint::QuicTransportConfig {
    iroh::endpoint::QuicTransportConfig::builder()
        .default_path_max_idle_timeout(PATH_IDLE)
        .default_path_keep_alive_interval(KEEP_ALIVE)
        .build()
}

/// Where the page offers a path: the route the offer and answer cross on
/// `Alpn::WebRtc` (webrtc-node; webrtc-page), JSON in and out, as the
/// spike's `S` stream was (native/src/main.rs:135-148).
pub const OFFER_ROUTE: &str = "/offer";
