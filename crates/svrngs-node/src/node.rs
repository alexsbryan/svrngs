// SPDX-License-Identifier: AGPL-3.0-or-later
//! The node (DESIGN §8, U1): one group member's computer — its journals, the
//! iroh endpoint it dials and is dialled on, the accept loop serving the sync
//! route, the round, and (U3) the loopback listener serving the record to
//! this computer's apps ([`crate::api`]). It replaces upstream's `RailsDaemon` and `RailsNode`
//! wiring (`commonwealth/crates/commonwealth-rails/src/lib.rs:387-485` and
//! `574-632` at `e1a43bcd2`), which is not forked: gossip, join, the mesh
//! store, the work plane, presence, the self-heal and
//! `rails.toml` stay behind (the inventory's `RailsDaemon` row).
//!
//! **One key signs and dials** (DESIGN §8, "Four facts"): the endpoint is
//! bound with the key the rail signs with, as upstream's was
//! (`lib.rs:400-402`), so a member's actor is the key iroh verifies when it
//! dials, and a key standing in a group's register is a key the round can
//! dial.
//!
//! **A link is redeemed on a second protocol** (DESIGN §8, U2's inventory,
//! the acceptor row): [`Alpn::Redeem`] is served to any dialer, with its
//! two routes, the ask and the redemption ([`crate::redeem`]), so a key standing nowhere reaches that and
//! nothing else; [`Alpn::Member`] stays closed to it.
//!
//! **The group decides who is in it** (DESIGN §6, U1): the register —
//! [`RingRail::standing`], over each group's seed and its acts — answers
//! all three questions the node asks. The round dials every key standing in
//! a group the node holds and offers each group to its own; the accept loop
//! admits a dialer standing in any group the node holds and closes the
//! rest; and the sync route serves a dialer standing in the group its
//! request names.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Notify;

use crate::iroh::{
    build_relayed_endpoint, format_dial_string, key_actor, Admit, Alpn, Contact, Endpoint,
    EndpointAddr, IrohAcceptor, IrohTransport, PublicKey, RelayConfig, SecretKey,
};
use crate::rail::RailApi;
use crate::redeem::SeedText;
use crate::register::Standing;
use crate::ring_routes::{self, RingInbound};
use crate::ring_sync::{
    exchange, peer_client, run_one_round, spawn_ring_sync_loop, ExchangeOutcome, ExchangeStop,
    RingSyncHandle, RoundMembers, RoundOutcome, DEFAULT_RING_SYNC_INTERVAL,
};
use crate::{actor_of, Ed25519Verifier, RingRail, SigningKey};

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
    #[error(transparent)]
    Api(#[from] crate::api::Refusal),
}

/// A running node. Dropping it stops its round and its accept loop.
pub struct Node {
    host: Arc<Host>,
    api: crate::api::Serving,
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
    /// dialling with `key`, reaching the network as `relay` says,
    /// answering a link's ask with the seed `seeds` reads, and serving the
    /// record to this computer's apps on `api`, which must be loopback
    /// ([`crate::api`]; port `0` takes any free one, [`Node::api_addr`]).
    /// The round runs once now and then every [`DEFAULT_RING_SYNC_INTERVAL`];
    /// nothing nudges it yet.
    pub async fn start(
        data_dir: PathBuf,
        key: SigningKey,
        relay: &RelayConfig,
        seeds: SeedText,
        api: SocketAddr,
    ) -> Result<Self, NodeError> {
        std::fs::create_dir_all(&data_dir).map_err(|source| NodeError::DataDir {
            path: data_dir.clone(),
            source,
        })?;
        let actor = actor_of(&key);
        let secret = SecretKey::from_bytes(&key.to_bytes());
        let rail = Arc::new(RingRail::new(&data_dir, Arc::new(key)));
        // Bound before the endpoint, so a refused address leaves no endpoint
        // dropped open.
        let api = crate::api::serve(RailApi { rail: rail.clone() }, api).await?;
        let endpoint = build_relayed_endpoint(
            secret,
            Alpn::ALL.iter().map(|a| a.bytes().to_vec()).collect(),
            relay,
        )
        .await
        .map_err(NodeError::Endpoint)?;
        tracing::info!(
            target: "node",
            actor,
            data_dir = %data_dir.display(),
            dial = ?format_dial_string(&endpoint.addr()),
            api = %api.addr(),
            "node: endpoint bound with the signing key"
        );
        let admitting = rail.clone();
        let members = ring_routes::router(RingInbound { rail: rail.clone() });
        let redeem = crate::redeem::router(rail.clone(), seeds);
        let acceptor =
            IrohAcceptor::spawn(endpoint.clone(), move |alpn: &[u8], dialer: &PublicKey| {
                match admit_dialer(&admitting, alpn, dialer) {
                    Some(Alpn::Member) => Admit::Serve(members.clone()),
                    Some(Alpn::Redeem) => Admit::Serve(redeem.clone()),
                    None => Admit::Refuse,
                }
            });
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
            api,
            _acceptor: acceptor,
            _round: round,
        })
    }

    /// This node's journals, signed with its key.
    pub fn rail(&self) -> &Arc<RingRail> {
        &self.host.rail
    }

    /// Where an app on this computer reaches the record: a loopback
    /// address ([`crate::api`]).
    pub fn api_addr(&self) -> SocketAddr {
        self.api.addr()
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

    /// One exchange of `namespace` with the node at `peer`, whatever this
    /// node's register says of it: a joiner syncs once from the issuer that
    /// admitted it, whom its own register may not know yet (DESIGN §8,
    /// U2). The peer's addresses are seeded on the transport, so the round
    /// reaches it too once the register names it.
    pub async fn sync_with(&self, namespace: &str, peer: &EndpointAddr) -> ExchangeOutcome {
        let addrs: Vec<_> = peer.ip_addrs().copied().collect();
        self.host.transport.add_known_peer(peer.id, addrs.clone());
        let contact = Contact {
            key: peer.id,
            relay_url: peer.relay_urls().next().map(|r| r.to_string()),
            iroh_direct_addrs: addrs,
        };
        let stopped = |why: String| ExchangeOutcome {
            stop: Some(ExchangeStop::Failed(why)),
            ..Default::default()
        };
        let Some(base) = self.host.transport.base_url(&contact).await else {
            tracing::debug!(target: "node", namespace, "node: sync_with — the peer has no dialable path");
            return stopped("no dialable path".into());
        };
        let http = match peer_client() {
            Ok(h) => h,
            Err(e) => return stopped(e.to_string()),
        };
        let url = format!("{base}/internal/ring/sync");
        let out = exchange(http, &url, &self.host.rail, namespace).await;
        tracing::debug!(target: "node", namespace, pulled = out.pulled, pushed = out.pushed, stop = ?out.stop,
            "node: sync_with — one exchange with the peer");
        out
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
    /// with: every key standing in a group this node holds, but this
    /// node's. A group whose register is unreadable adds nobody, as the
    /// round offers it to nobody.
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
            let standing = match self.rail.standing(namespace, &Ed25519Verifier) {
                Ok(Standing::Members(m)) => m,
                Ok(_) => continue,
                Err(e) => {
                    tracing::warn!(target: "node", namespace, error = %e, "node: a group's register is unreadable, so it adds no member");
                    continue;
                }
            };
            for (actor, person) in &standing {
                if *actor == self.actor || keys.contains_key(actor) {
                    continue;
                }
                let Some(key) = key_of(actor) else {
                    tracing::warn!(target: "node", namespace, actor, "node: a standing key is not an iroh key, so it is not dialled");
                    continue;
                };
                let name = format!("{person}/{}", &actor[..8]);
                tracing::debug!(target: "node", namespace, member = %name, "node: a member stands in the group, to dial");
                keys.insert(actor.clone(), (name, key));
            }
        }
        Some(RoundMembers::of(keys.into_values().collect()))
    }
}

/// The accept loop's decision: the protocol whose routes a dialer is
/// served, or `None` to close it. Any dialer may redeem a link; the member
/// protocol serves a dialer standing in any group this node holds and closes
/// the rest (DESIGN §8, U1's inventory, "the mesh" row — upstream asked the
/// mesh, `acceptor.rs:47-61`). Asked once per connection, so a member
/// admitted or removed since the last dial is answered as it stands now.
fn admit_dialer(rail: &RingRail, alpn: &[u8], dialer: &PublicKey) -> Option<Alpn> {
    let actor = key_actor(dialer);
    match Alpn::of(alpn) {
        Some(Alpn::Member) => {}
        Some(Alpn::Redeem) => {
            tracing::debug!(target: "node", actor, "node: admitted to redeem a link — any dialer may");
            return Some(Alpn::Redeem);
        }
        None => {
            tracing::debug!(target: "node", actor, alpn = %String::from_utf8_lossy(alpn), "node: refused — a protocol this node does not speak");
            return None;
        }
    }
    let namespaces = match rail.namespaces() {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!(target: "node", actor, error = %e, "node: refused — cannot list the groups held, so nobody can be shown to stand in one");
            return None;
        }
    };
    for namespace in &namespaces {
        match rail.standing(namespace, &Ed25519Verifier) {
            Ok(standing) if standing.names(&actor) => {
                tracing::debug!(target: "node", actor, namespace, "node: admitted — the dialer stands in a group held here");
                return Some(Alpn::Member);
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(target: "node", actor, namespace, error = %e, "node: a group's register is unreadable, so it admits nobody");
            }
        }
    }
    tracing::debug!(target: "node", actor, groups = namespaces.len(), "node: refused — the dialer stands in no group held here");
    None
}

/// The iroh key an actor's lowercase hex spells, or `None`.
fn key_of(actor: &str) -> Option<PublicKey> {
    let bytes: [u8; 32] = hex::decode(actor).ok()?.try_into().ok()?;
    PublicKey::from_bytes(&bytes).ok()
}
