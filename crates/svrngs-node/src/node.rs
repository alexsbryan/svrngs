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
//! **What the people of its groups lend is asked, not stored** (DESIGN §8,
//! U3; u3-5): [`Node::lends`] asks every computer's key standing in a
//! group the node holds, now, what its computer lends ([`crate::door`]'s `GET /lends`),
//! and answers each lend with whose it is and the address on this
//! computer's loopback that reaches it, through the dialler's bridge to its
//! lender. A computer that does not answer is named, its lends absent
//! rather than stale.
//!
//! **The group decides who is in it** (DESIGN §6, U1): the register —
//! [`RingRail::standing`], over each group's seed and its acts — answers
//! all three questions the node asks. The round dials every computer's key
//! standing in a group the node holds and offers each group to its own; the
//! accept loop admits a dialer standing in any group the node holds, a
//! browser's key as a computer's, and closes the rest; and the sync route
//! serves a dialer standing in the group its request names.
//!
//! **A browser's key dials and is never dialled** (DESIGN §3 "Joining",
//! call 2; u4-1): the round and [`Node::lends`] dial only the keys the
//! register says a computer holds ([`crate::Device::dialled`]), since a
//! browser's endpoint lives only while its page runs and every dial of it
//! would wait out a timeout.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Notify;

use crate::door::{LENDS_ROUTE, LEND_ROUTE};
use crate::iroh::{
    actor_key, build_relayed_endpoint, format_dial_string, key_actor, Admit, Alpn, Contact,
    Endpoint, EndpointAddr, IrohAcceptor, IrohTransport, PublicKey, RelayConfig, SecretKey,
};
use crate::lend::Offered;
use crate::rail::{LiveBuffer, RailApi};
use crate::redeem::SeedText;
use crate::register::{Member, Standing};
use crate::ring_routes::{self, RingInbound};
use crate::ring_sync::{
    exchange, peer_client, run_one_round, spawn_ring_sync_loop, ExchangeOutcome, ExchangeStop,
    RingSyncHandle, RoundMembers, RoundOutcome, DEFAULT_RING_SYNC_INTERVAL,
};
use crate::{actor_of, Ed25519Verifier, Person, RingRail, SigningKey};

/// A lend another computer of this one's groups lends, as it answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lent {
    /// Whose it is: the person the first group naming their key names.
    pub person: Person,
    pub offered: Offered,
    /// Where this computer's browser reaches it: the loopback bridge to its
    /// lender's computer (`IrohTransport::base_url`), then `/lend/<name>/`.
    pub address: String,
}

/// What the people of this computer's groups lend, asked now.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lends {
    pub lent: Vec<Lent>,
    /// The people whose computer did not answer, so what they lend is not
    /// known: absent, which is not lending nothing.
    pub unanswered: Vec<Person>,
}

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
    Api(#[from] crate::api::ApiError),
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
    transport: Arc<IrohTransport>,
    actor: String,
}

impl Node {
    /// Start a node over `data_dir` (its journals under `rings/`), signing and
    /// dialling with `key`, reaching the network as `relay` says,
    /// answering a link's ask with the seed `seeds` reads, and serving the
    /// record to this computer's apps on `api`, which must be loopback
    /// ([`crate::api`]; port `0` takes any free one, [`Node::api_addr`]).
    /// The round runs once now, then every [`DEFAULT_RING_SYNC_INTERVAL`],
    /// and at once after an app's append.
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
        // One nudge, the round's, which the append door fires so a write
        // goes now rather than at the next interval: upstream built one
        // (`lib.rs:421`) and handed it to the round and the door; until u3-2
        // the node built one for the round alone.
        let nudge = Arc::new(Notify::new());
        // One live buffer: the live route fills it, the loopback drain empties it.
        let live = Arc::new(LiveBuffer::default());
        let api = crate::api::bind(api).await?;
        let endpoint = build_relayed_endpoint(
            secret,
            Alpn::ALL.iter().map(|a| a.bytes().to_vec()).collect(),
            relay,
        )
        .await
        .map_err(NodeError::Endpoint)?;
        // One transport: the round dials through it, and so does the live
        // lane's fan-out, which the record API runs (u3-3).
        let transport = Arc::new(IrohTransport::new(endpoint.clone()));
        let api = crate::api::serve(
            RailApi {
                rail: rail.clone(),
                nudge: nudge.clone(),
                live: live.clone(),
                transport: transport.clone(),
            },
            api,
        );
        tracing::info!(
            target: "node",
            actor,
            data_dir = %data_dir.display(),
            dial = ?format_dial_string(&endpoint.addr()),
            api = %api.addr(),
            "node: endpoint bound with the signing key"
        );
        let admitting = rail.clone();
        // The members' protocol serves sync and the live lane, and the
        // door to what this computer lends (u3-4).
        let members = ring_routes::router(RingInbound {
            rail: rail.clone(),
            live,
        })
        .merge(crate::door::router(crate::door::Door {
            rail: rail.clone(),
            data_dir: data_dir.clone(),
            using: crate::door::Using::spawn(),
        }));
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
            transport,
            rail,
            endpoint,
            actor,
        });
        let round = spawn_ring_sync_loop(host.clone(), DEFAULT_RING_SYNC_INTERVAL, nudge);
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

    /// What every other computer's key standing in a group this node holds
    /// lends, each computer asked now and all at once, and where this computer reaches
    /// each lend. Nothing is kept between asks.
    pub async fn lends(&self) -> Lends {
        let peers = self.host.standing_peers().unwrap_or_default();
        let mut asks = Vec::with_capacity(peers.len());
        for (actor, person, key) in peers {
            let transport = self.host.transport.clone();
            asks.push(tokio::spawn(async move {
                let answer = ask_lends(&transport, &actor, key).await;
                (person, answer)
            }));
        }
        let mut lends = Lends::default();
        for ask in asks {
            match ask.await {
                Ok((person, Some(answered))) => {
                    lends
                        .lent
                        .extend(answered.into_iter().map(|(offered, address)| Lent {
                            person: person.clone(),
                            offered,
                            address,
                        }))
                }
                Ok((person, None)) => lends.unanswered.push(person),
                Err(e) => {
                    tracing::warn!(target: "node", error = %e, "node: lends — an ask ended without an answer")
                }
            }
        }
        lends.lent.sort_by(|a, b| {
            (a.person.as_str(), &a.offered.name).cmp(&(b.person.as_str(), &b.offered.name))
        });
        tracing::debug!(target: "node", lent = lends.lent.len(), unanswered = lends.unanswered.len(),
            "node: lends — asked every computer of this one's groups");
        lends
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
    /// with: every computer's key standing in a group this node holds, but
    /// this node's. A group whose register is unreadable adds nobody, as the
    /// round offers it to nobody.
    pub(crate) async fn members(&self) -> Option<RoundMembers> {
        let peers = self.standing_peers()?;
        Some(RoundMembers::of(
            peers
                .into_iter()
                .map(|(actor, person, key)| (format!("{person}/{}", &actor[..8]), key))
                .collect(),
        ))
    }

    /// Every key standing in a group this node holds that is dialled — a
    /// computer's, never a browser's (u4-1) — but this node's, once each, by
    /// its actor, with the person the first group naming it names; or
    /// `None` when the groups held cannot be listed. A group whose register
    /// is unreadable adds nobody.
    fn standing_peers(&self) -> Option<Vec<(String, Person, PublicKey)>> {
        let namespaces = match self.rail.namespaces() {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(target: "node", error = %e, "node: cannot list the groups held, so the round has no members");
                return None;
            }
        };
        let mut keys: BTreeMap<String, (Person, PublicKey)> = BTreeMap::new();
        for namespace in &namespaces {
            let standing = match self.rail.standing(namespace, &Ed25519Verifier) {
                Ok(Standing::Members(m)) => m,
                Ok(_) => continue,
                Err(e) => {
                    tracing::warn!(target: "node", namespace, error = %e, "node: a group's register is unreadable, so it adds no member");
                    continue;
                }
            };
            for (actor, person, key) in dialled(&self.actor, namespace, &standing) {
                if keys.contains_key(actor) {
                    continue;
                }
                let name = format!("{person}/{}", &actor[..8]);
                tracing::debug!(target: "node", namespace, member = %name, "node: a member stands in the group, to dial");
                keys.insert(actor.clone(), (person.clone(), key));
            }
        }
        Some(
            keys.into_iter()
                .map(|(actor, (person, key))| (actor, person, key))
                .collect(),
        )
    }
}

/// The keys standing in `namespace` that this node, `this`, dials: each a
/// computer's ([`crate::Device::dialled`], u4-1) and an iroh key, with its
/// person. The one decider the round, [`Node::lends`] and the live fan-out
/// (`rail.rs`'s `live_peers`) ask, so a browser's key is skipped by all
/// three or by none.
pub(crate) fn dialled<'a>(
    this: &str,
    namespace: &str,
    standing: &'a BTreeMap<String, Member>,
) -> Vec<(&'a String, &'a Person, PublicKey)> {
    let mut out = Vec::new();
    for (actor, Member { person, device }) in standing {
        if actor == this {
            continue;
        }
        if !device.dialled() {
            tracing::debug!(target: "node", namespace, actor, ?device,
                "node: a member stands in the group from a browser, which dials and is never dialled");
            continue;
        }
        let Some(key) = actor_key(actor) else {
            tracing::warn!(target: "node", namespace, actor, "node: a standing key is not an iroh key, so it is not dialled");
            continue;
        };
        out.push((actor, person, key));
    }
    out
}

/// Ask the computer `key` what it lends: each lend with the address that
/// reaches it from here, or `None` when it did not answer, or answered
/// with something that is not a list of lends.
async fn ask_lends(
    transport: &IrohTransport,
    actor: &str,
    key: PublicKey,
) -> Option<Vec<(Offered, String)>> {
    #[derive(serde::Deserialize)]
    struct Answer {
        lends: Vec<Offered>,
    }
    let contact = Contact {
        key,
        relay_url: None,
        iroh_direct_addrs: Vec::new(),
    };
    let Some(base) = transport.base_url(&contact).await else {
        tracing::debug!(target: "node", actor, "node: lends — no dialable path to the computer");
        return None;
    };
    let http = peer_client().ok()?;
    let answer = match http.get(format!("{base}{LENDS_ROUTE}")).send().await {
        Ok(r) if r.status().is_success() => r.json::<Answer>().await,
        Ok(r) => {
            tracing::debug!(target: "node", actor, status = r.status().as_u16(), "node: lends — the computer refused the ask");
            return None;
        }
        Err(e) => {
            tracing::debug!(target: "node", actor, error = %e, "node: lends — the computer did not answer");
            return None;
        }
    };
    match answer {
        Ok(a) => {
            tracing::debug!(target: "node", actor, lends = a.lends.len(), "node: lends — the computer answered");
            Some(
                a.lends
                    .into_iter()
                    .map(|o| {
                        let address = format!("{base}{LEND_ROUTE}{}/", o.name);
                        (o, address)
                    })
                    .collect(),
            )
        }
        Err(e) => {
            tracing::debug!(target: "node", actor, error = %e, "node: lends — the answer is not a list of lends");
            None
        }
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
    if let Some((namespace, _)) = rail.standing_anywhere(&actor, &Ed25519Verifier) {
        tracing::debug!(target: "node", actor, namespace, "node: admitted — the dialer stands in a group held here");
        return Some(Alpn::Member);
    }
    tracing::debug!(target: "node", actor, "node: refused — the dialer stands in no group held here");
    None
}
