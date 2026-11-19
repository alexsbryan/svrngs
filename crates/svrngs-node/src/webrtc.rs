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

pub mod wire;
