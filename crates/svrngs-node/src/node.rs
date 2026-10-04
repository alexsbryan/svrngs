// SPDX-License-Identifier: AGPL-3.0-or-later
//! The node (DESIGN §8, U1): one group member's computer — its journals, the
//! iroh endpoint it dials and is dialled on, the accept loop serving the sync
//! route, and the round. It replaces upstream's `RailsDaemon` and `RailsNode`
//! wiring (`commonwealth/crates/commonwealth-rails/src/lib.rs:387-485` and
//! `574-632` at `e1a43bcd2`), which is not forked: gossip, join, the mesh
//! store, the work plane, presence, the self-heal, the loopback listener and
//! `rails.toml` stay behind (the inventory's `RailsDaemon` row).
//!
//! **One key signs and dials** (DESIGN §8, "Four facts"): the endpoint is
//! bound with the key the rail signs with, as upstream's was
//! (`lib.rs:400-402`), so a member's actor is the key iroh verifies when it
//! dials, and a key the seed names is a key the round can dial.
//!
//! Until U1's register lands (u1-5) the seed — each group's `roster.json` —
//! answers who is in a group: the round dials the keys the seeds name, the
//! sync route serves the keys its group's seed names, and the accept loop
//! admits any dialer, leaving the refusal to the route.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Notify;

use crate::iroh::{
    build_relayed_endpoint, format_dial_string, Admit, Endpoint, IrohAcceptor, IrohTransport,
    PublicKey, RelayConfig, SecretKey, ALPN,
};
use crate::ring_routes::{self, RingInbound};
use crate::ring_sync::{
    peer_client, roster_of, run_one_round, spawn_ring_sync_loop, RingSyncHandle, RoundMembers,
    RoundOutcome, DEFAULT_RING_SYNC_INTERVAL,
};
use crate::{actor_of, RingRail, SigningKey};

/// Why a node did not start.
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("the data dir {path} cannot be made: {source}")]
    DataDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{0}")]
    Endpoint(String),
}

/// A running node. Dropping it stops its round and its accept loop.
pub struct Node {
    host: Arc<Host>,
    _acceptor: IrohAcceptor,
    _round: RingSyncHandle,
}

/// What the round reads: the journals, the endpoint's transport, and the key
/// this node signs and dials with. Upstream's round read these through its
/// `RingSyncHost` trait, which `RailsDaemon` implemented; with one host the
/// trait went, and the round calls this (RUST_ARCH 7).
pub struct Host {
    rail: Arc<RingRail>,
    endpoint: Endpoint,
    transport: IrohTransport,
    actor: String,
}

impl Node {
    /// Start a node over `data_dir` (its journals under `rings/`), signing and
    /// dialling with `key`, reaching the network as `relay` says. The round
    /// runs once now and then every [`DEFAULT_RING_SYNC_INTERVAL`]; nothing
    /// nudges it until U3's record API.
    pub async fn start(
        data_dir: PathBuf,
        key: SigningKey,
        relay: &RelayConfig,
    ) -> Result<Self, NodeError> {
        std::fs::create_dir_all(&data_dir).map_err(|source| NodeError::DataDir {
            path: data_dir.clone(),
            source,
        })?;
        let endpoint = build_relayed_endpoint(
            SecretKey::from_bytes(&key.to_bytes()),
            vec![ALPN.to_vec()],
            relay,
        )
        .await
        .map_err(NodeError::Endpoint)?;
        let actor = actor_of(&key);
        tracing::info!(
            target: "node",
            actor,
            data_dir = %data_dir.display(),
            dial = ?format_dial_string(&endpoint.addr()),
            "node: endpoint bound with the signing key"
        );
        let rail = Arc::new(RingRail::new(&data_dir, Arc::new(key)));
        // Until u1-5 any dialer is admitted, and the sync route refuses (403)
        // one its group's seed does not name.
        let acceptor = IrohAcceptor::spawn(
            endpoint.clone(),
            |_: &[u8], _: &PublicKey| Admit::Serve,
            ring_routes::router(RingInbound { rail: rail.clone() }),
        );
        let host = Arc::new(Host {
            transport: IrohTransport::new(endpoint.clone()),
            rail,
            endpoint,
            actor,
        });
        let round = spawn_ring_sync_loop(
            host.clone(),
            DEFAULT_RING_SYNC_INTERVAL,
            Arc::new(Notify::new()),
        );
        Ok(Self {
            host,
            _acceptor: acceptor,
            _round: round,
        })
    }

    /// This node's journals, signed with its key.
    pub fn rail(&self) -> &Arc<RingRail> {
        &self.host.rail
    }

    /// The endpoint this node dials and is dialled on.
    pub fn endpoint(&self) -> &Endpoint {
        &self.host.endpoint
    }

    /// The dial-by-key transport the round dials through; a caller seeds a
    /// member's addresses here when no address lookup can find it.
    pub fn transport(&self) -> &IrohTransport {
        &self.host.transport
    }

    /// Run one round now, beside the interval's, and say what it moved.
    pub async fn round(&self) -> RoundOutcome {
        run_one_round(self.host.as_ref()).await
    }
}

impl Host {
    /// This node's journals.
    pub(crate) fn rail(&self) -> &RingRail {
        &self.rail
    }

    /// The one client every peer call rides — one pool, one timeout policy.
    pub(crate) fn http(&self) -> Result<&'static reqwest::Client, &'static str> {
        peer_client()
    }

    /// The transport a member's contact is resolved through.
    pub(crate) fn transport(&self) -> &IrohTransport {
        &self.transport
    }

    /// This round's members, or `None` when the node has none to round
    /// with: every key a seed of a group this node holds names, but this node's —
    /// the round's members until the register answers (u1-5). A group whose
    /// seed is unreadable adds nobody, as the round offers it to nobody.
    pub(crate) async fn members(&self) -> Option<RoundMembers> {
        let namespaces = match self.rail.namespaces() {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(target: "node", error = %e, "node: cannot list the groups held, so the round has no members");
                return None;
            }
        };
        let mut keys: BTreeMap<String, (String, PublicKey)> = BTreeMap::new();
        for namespace in &namespaces {
            let seed = match roster_of(&self.rail, namespace).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!(target: "node", namespace, error = %e, "node: a group's seed is unreadable, so it adds no member");
                    continue;
                }
            };
            for (person, actors) in &seed.members {
                for actor in actors {
                    if *actor == self.actor || keys.contains_key(actor) {
                        continue;
                    }
                    let Some(key) = key_of(actor) else {
                        tracing::warn!(target: "node", namespace, actor, "node: a seed names a key that is not one, so it is not dialled");
                        continue;
                    };
                    let name = format!("{person}/{}", &actor[..8]);
                    tracing::debug!(target: "node", namespace, member = %name, "node: the seed names a member to dial");
                    keys.insert(actor.clone(), (name, key));
                }
            }
        }
        Some(RoundMembers::of(keys.into_values().collect()))
    }
}

/// The iroh key an actor's lowercase hex spells, or `None`.
fn key_of(actor: &str) -> Option<PublicKey> {
    let bytes: [u8; 32] = hex::decode(actor).ok()?.try_into().ok()?;
    PublicKey::from_bytes(&bytes).ok()
}
