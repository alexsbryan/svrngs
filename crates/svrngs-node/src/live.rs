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
//! names them, [`LivePeer`]; nothing calls the fan-out yet. DESIGN §8, U3's
//! inventory.)

use axum::http::StatusCode;

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
    node: String,
    name: Option<String>,
    delivered: bool,
    /// Why not, in the words of whatever refused. Present iff `delivered` is
    /// false — the page shows a live lane that is half up rather than
    /// believing a 200 meant everyone saw it.
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// POST `bytes` to every online peer's `/internal/ring/live`, concurrently.
///
/// **The fan-out is `forward_to_peers`'s, one file over**
/// (`routes_internal/pipeline_pause.rs:302`): read `inner.mesh`, drop self
/// and anything not `Online`, resolve each peer's endpoints through the
/// `PeerTransport` seam, and try them in order until one answers. Direct
/// fan-out rather than iroh-gossip because at ring scale every pair already
/// holds an authenticated connection (`ring-apps-shelf.md` §gossip, "default
/// impl").
pub async fn push_ephemeral(
    transport: &IrohTransport,
    peers: Vec<LivePeer>,
    namespace: &str,
    payload: &str,
) -> Vec<PeerDelivery> {
    if peers.is_empty() {
        return Vec::new();
    }

    let client = match reqwest::Client::builder()
        .timeout(LIVE_PUSH_TIMEOUT)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "rail live: reqwest build failed");
            return peers
                .into_iter()
                .map(|m| PeerDelivery {
                    node: m.node.clone(),
                    name: m.name.clone(),
                    delivered: false,
                    error: Some(format!("local reqwest build failed: {e}")),
                })
                .collect();
        }
    };

    let envelope = serde_json::to_string(&LiveEnvelope {
        namespace: namespace.to_string(),
        payload: payload.to_string(),
    })
    .expect("a struct of two Strings always serializes");
    let mut handles = Vec::with_capacity(peers.len());
    for peer in peers {
        let client = client.clone();
        let envelope = envelope.clone();
        let node = peer.node.clone();
        let name = peer.name.clone();
        // svrngs: `base_url` answers at most one, as ring_sync's round reads it.
        let endpoints: Vec<String> = transport
            .base_url(&peer.contact)
            .await
            .into_iter()
            .collect();
        handles.push(tokio::spawn(async move {
            offer_peer(&client, &envelope, node, name, &endpoints).await
        }));
    }

    let mut out = Vec::with_capacity(handles.len());
    for h in handles {
        match h.await {
            Ok(result) => out.push(result),
            Err(join_err) => {
                tracing::warn!(error = %join_err, "rail live: peer task panicked");
            }
        }
    }
    out
}

/// Try each transport-resolved endpoint for one peer; the first that answers
/// wins. Candidate ordering is the `PeerTransport` seam's, so live traffic
/// routes the way every other peer call on this daemon does.
async fn offer_peer(
    client: &reqwest::Client,
    envelope: &str,
    node: String,
    name: Option<String>,
    endpoints: &[String],
) -> PeerDelivery {
    let mut last_error = "no addresses advertised by peer".to_string();
    for ep in endpoints {
        let url = format!("{ep}/internal/ring/live");
        let request = client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(envelope.to_string());
        match request.send().await {
            Ok(resp) if resp.status().is_success() => {
                return PeerDelivery {
                    node,
                    name,
                    delivered: true,
                    error: None,
                };
            }
            Ok(resp) if resp.status() == StatusCode::NOT_FOUND => {
                last_error = format!(
                    "{url}: 404 — peer is running an older daemon without \
                     /internal/ring/live; rebuild + restart it to see cursors"
                );
                // Terminal for this peer: no other address routes differently.
                break;
            }
            Ok(resp) => {
                last_error = format!("{url}: {}", resp.status());
                continue;
            }
            Err(e) => {
                last_error = format!("{url}: {e}");
                continue;
            }
        }
    }
    tracing::debug!(
        node,
        endpoints = endpoints.len(),
        error = %last_error,
        "rail live: peer not reached"
    );
    PeerDelivery {
        node,
        name,
        delivered: false,
        error: Some(last_error),
    }
}
