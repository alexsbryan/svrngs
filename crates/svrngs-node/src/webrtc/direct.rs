// SPDX-License-Identifier: AGPL-3.0-or-later
//! Direct connections, on a network the lender enabled (webrtc-network;
//! RELAY_COMMONS design 4; the operator's UDP decision, 2026-11-19): the
//! home page turns them on or off for the network this computer is on now
//! ([`ROUTE`]), kept whole in [`DIRECT_FILE`] by the network's own name —
//! its router's hardware address ([`Network`]), as the room keeps the
//! network a lend was turned on at — and an offer is answered only on one
//! of them, at the private address [`crate::network::pick`] gives
//! ([`binds_at`]). Elsewhere nothing is bound and the phone stays on the
//! relay.
//!
//! The guard is recognition, not a firewall (the operator's decision): it
//! keeps a socket from being opened by accident on a café's router that
//! hands out the same private address, and nothing here says it keeps the
//! port "on this Wi-Fi".
//!
//! Its parts: [`binds_at`], the offer's pure decision; [`setting`], what
//! the home page is told, in a person's words; and [`router`], the home
//! page's one route, mounted beside the room's.

use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::network::{Here, Network};

/// The file in the data dir that keeps which networks direct connections
/// are on for, written whole ([`crate::declared::write_whole`]).
pub const DIRECT_FILE: &str = "direct.json";

/// Where the home page turns direct connections on or off for this
/// network: the one route ([`router`]).
pub const ROUTE: &str = "/v1/direct";

/// What the home page says while direct connections are on for the network
/// this computer is on.
const ON: &str = "Direct connections are on for this WiFi: a phone on it may connect to this computer directly for what it lends.";

/// What it says while they are off.
const OFF: &str =
    "Direct connections are off for this WiFi. Lends reach this computer the way they usually do.";

/// What it says, and what an offer is refused with, while the network this
/// computer is on cannot be read.
const UNREAD: &str = "This computer cannot tell which WiFi it is on, so direct connections cannot be turned on here.";

/// The networks direct connections are on for in `file`, or why it cannot
/// be read: none before the first, and one unread said, never read as none
/// (each offer is refused until a turn writes it over).
pub(crate) fn kept(file: &Path) -> Result<Vec<Network>, String> {
    crate::declared::read_json(file).map(Option::unwrap_or_default)
}

/// `network` turned `on` or off in `file`, written whole, and the networks
/// as they are now; a file that cannot be read is written over.
pub(crate) fn turned(file: &Path, network: Network, on: bool) -> Vec<Network> {
    let mut kept = kept(file).unwrap_or_default();
    kept.retain(|n| *n != network);
    if on {
        kept.push(network);
    }
    let bytes = serde_json::to_vec(&kept).expect("the enabled networks are JSON");
    let written = crate::declared::write_whole(file, &bytes);
    tracing::debug!(target: "webrtc", ?network, on, ?written, "webrtc: direct connections turned, and kept whole so they open again as svrngs starts");
    kept
}

/// Where an offer is answered, and on which network: the private address
/// [`crate::network::pick`] gives, on a network `file` holds enabled; else
/// why nothing is bound. Pure but for `file`, judged with values
/// (`webrtc/tests.rs`).
pub fn binds_at(file: &Path, here: &Here) -> Result<(Network, Ipv4Addr), String> {
    let enabled =
        kept(file).map_err(|why| format!("cannot read the direct connections setting ({why})"))?;
    let network = here.network.as_ref().map_err(|_| UNREAD.to_string())?;
    let decided = match (enabled.contains(network), here.address) {
        (false, _) => Err("direct connections are not on for this WiFi".to_string()),
        (true, Some(address)) => Ok((*network, address)),
        (true, None) => Err("this computer has no private address on this WiFi".to_string()),
    };
    tracing::debug!(target: "webrtc", ?enabled, network = ?here.network, address = ?here.address, ?decided, "webrtc: an offer is answered only on a network the lender enabled, at the address picked there, or why not");
    decided
}

/// What the home page is told at [`ROUTE`] and after a turn: whether
/// direct connections are on for the network this computer is on now, and
/// that in a person's words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    pub on: bool,
    pub said: String,
}

/// [`Setting`], from the network read now and what `file` holds. Pure but
/// for `file`, judged with values (`webrtc/tests.rs`).
pub fn setting(file: &Path, here: &Here) -> Setting {
    let (on, said) = match (&here.network, kept(file)) {
        (Err(_), _) => (false, UNREAD.to_string()),
        (Ok(_), Err(why)) => (
            false,
            format!("This computer cannot read the direct connections setting ({why}), so they are on for none."),
        ),
        (Ok(network), Ok(enabled)) => {
            let on = enabled.contains(network);
            (on, String::from(if on { ON } else { OFF }))
        }
    };
    let setting = Setting { on, said };
    tracing::debug!(target: "webrtc", ?setting, network = ?here.network, "webrtc: what the home page is told of direct connections");
    setting
}

/// The home page's routes, for [`crate::home::router`] to serve.
pub fn router(file: PathBuf) -> Router {
    Router::new()
        .route(ROUTE, get(listed).post(turn))
        .with_state(file)
}

/// `GET /v1/direct`: what direct connections are on for now.
async fn listed(State(file): State<PathBuf>) -> Response {
    Json(setting(&file, &crate::network::now().await)).into_response()
}

/// What a press asks: on for the network this computer is on, or off.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Asking {
    on: bool,
}

/// `POST /v1/direct`: the setting turned for the network read now, and the
/// setting as it stands then ([`listed`]). A network that cannot be read
/// turns nothing on, and is refused.
async fn turn(State(file): State<PathBuf>, Json(ask): Json<Asking>) -> Response {
    let here = crate::network::now().await;
    let Ok(network) = here.network else {
        tracing::debug!(target: "webrtc", "webrtc: direct connections not turned — this computer cannot tell which network it is on");
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": UNREAD })),
        )
            .into_response();
    };
    turned(&file, network, ask.on);
    Json(setting(&file, &here)).into_response()
}
