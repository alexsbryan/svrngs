// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/crates/sovereign-daemon/src/routes_rail_live.rs@e1a43bcd2
//! The ring rail's **live lane** — delivery, not record.
//!
//! `POST /v1/rail/live` fans one small payload out to every online peer
//! right now, each reached through cw-rails' reach door to that peer's
//! cw-rails `/internal/ring/live`; `GET /v1/rail/live` drains what arrived,
//! from the buffer the local cw-rails holds (pb-mesh-exit-transport). Nothing
//! here reaches a store, a journal or a disk, and that is the whole point:
//! presence is the case the durable lane provably cannot serve. y-protocols'
//! awareness `outdatedTimeout` is 30 s and the ring round's cadence is 60 s,
//! so a cursor carried by the journal would have faded before it arrived
//! (`quality/campaigns/ring-apps-shelf.md` §gossip).
//!
//! **The payload is opaque TEXT and this module never looks inside it.** The
//! page already base64s its Yjs bytes for a rail act (`sovereign/apps/ring-doc/
//! adapter.js`), so the live lane takes the same spelling rather than minting
//! a second one — and a daemon that decoded it would be claiming to know
//! what an app's presence means. Text also means the drain can hand the
//! bytes back verbatim in JSON with no encoder anywhere in this crate.
//!
//! (svrngs: the fan-out alone came over, `push_ephemeral` and `offer_peer`,
//! :76-196; the daemon's `live_push` and `live_drain` did not, since the
//! node's own doors are rail.rs's, [`crate::rail`]. The walk of the mesh's
//! Online members that chose the peers went with the mesh, and the caller
//! names them, [`LivePeer`]: the push door, [`crate::rail`], names the keys
//! standing in the group but this node's. DESIGN §8, U3's inventory.)

use crate::iroh::{Contact, IrohTransport};

// svrngs: `LIVE_PAYLOAD_MAX_BYTES` is rail.rs's, the same number, so the
// lane has one cap (ARCH §10.6); the push door that reads it is there.

/// How long one peer has to take a live payload.
///
/// Short on purpose: a cursor that arrives late is worth nothing, and this
/// fan-out runs inside a client request the page makes every 100 ms.
const LIVE_PUSH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// Moved to `sovereign_peer_wire` (phase-b pb-rails-parity): cw-rails'
/// `/internal/ring/live` reads the same body. (svrngs: forked whole, crate::peer_wire.)
use crate::peer_wire::LiveEnvelope;

/// One peer to offer a payload to: its key's hex, its name, and the
/// contact the transport dials. (svrngs: upstream read each from a mesh
/// `MemberRecord`, which did not come over.)
#[derive(Debug, Clone)]
pub struct LivePeer {
    pub node: String,
    pub name: Option<String>,
    pub contact: Contact,
}

/// What happened when one peer was offered a payload.
#[derive(Debug, serde::Serialize)]
pub struct PeerDelivery {
    pub node: String,
    pub name: Option<String>,
    pub delivered: bool,
    /// Why not, in the words of whatever refused. Present iff `delivered` is
    /// false — the page shows a live lane that is half up rather than
    /// believing a 200 meant everyone saw it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// POST `bytes` to every online peer's `/internal/ring/live`, concurrently.
///
/// **The fan-out is `forward_to_peers`'s, one file over**
/// (`routes_internal/pipeline_pause.rs:302`): read `inner.mesh`, drop self
/// and anything not `Online`, resolve each peer's endpoints through the
/// `PeerTransport` seam, and try them in order until one answers. Direct
/// fan-out rather than iroh-gossip because at ring scale every pair already
/// holds an authenticated connection (`ring-apps-shelf.md` §gossip, "default
/// impl"). (svrngs, arch-peer (c): a request on the stream to each, by its
/// key, [`IrohTransport::fetch`], where upstream tried each address a
/// loopback bridge answered with a reqwest client of its own.)
pub async fn push_ephemeral(
    transport: &IrohTransport,
    peers: Vec<LivePeer>,
    namespace: &str,
    payload: &str,
) -> Vec<PeerDelivery> {
    let envelope = serde_json::to_string(&LiveEnvelope {
        namespace: namespace.to_string(),
        payload: payload.to_string(),
    })
    .expect("a struct of two Strings always serializes");
    let offers = peers
        .into_iter()
        .map(|peer| offer_peer(transport, &envelope, peer));
    n0_future::join_all(offers).await
}

/// Offer `envelope` to one peer, which has [`LIVE_PUSH_TIMEOUT`] to take it.
async fn offer_peer(transport: &IrohTransport, envelope: &str, peer: LivePeer) -> PeerDelivery {
    let offer = axum::http::Request::post("/internal/ring/live")
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(envelope.to_string()))
        .expect("a fixed route and header");
    let taken =
        tokio::time::timeout(LIVE_PUSH_TIMEOUT, transport.fetch(&peer.contact, offer)).await;
    let error = match taken {
        Ok(Ok(a)) if a.status().is_success() => None,
        Ok(Ok(a)) => Some(a.status().to_string()),
        Ok(Err(e)) => Some(e.to_string()),
        Err(_) => Some(format!("not taken within {LIVE_PUSH_TIMEOUT:?}")),
    };
    if let Some(error) = &error {
        tracing::debug!(node = peer.node, error, "rail live: peer not reached");
    }
    PeerDelivery {
        node: peer.node,
        name: peer.name,
        delivered: error.is_none(),
        error,
    }
}
