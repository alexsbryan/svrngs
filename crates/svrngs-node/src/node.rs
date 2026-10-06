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
//! rather than stale. A member asks the same of the computer it dials,
//! `GET /people/lends` ([`PEOPLE_LENDS_ROUTE`], u4-5): that computer's
//! lends and the others' of the groups the asker stands in, each with its
//! lender's key, so a phone dials the lender itself (DESIGN §3, call 4).
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
use std::time::Duration;

use tokio::sync::Notify;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use crate::door::LENDS_ROUTE;
use crate::iroh::{
    actor_key, build_relayed_endpoint, format_dial_string, key_actor, Admit, Alpn, Contact, Dialer,
    Endpoint, EndpointAddr, IrohAcceptor, IrohTransport, PublicKey, RelayConfig, SecretKey,
};
use crate::lend::Offered;
use crate::rail::{LiveBuffer, RailApi};
use crate::redeem::SeedText;
use crate::register::{Member, Standing};
use crate::ring_routes::{self, err, RingInbound};
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
    /// The key their computer is dialled by, as the record spells an actor.
    pub key: String,
    pub offered: Offered,
    /// Where this computer's browser opens it: its origin on this
    /// computer's loopback, `http://<label>.localhost:<api port>/`
    /// ([`crate::origin`], u5-2), where until u5-2 it was the bridge to its
    /// lender's computer, then `/lend/<name>/`.
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
    /// What [`Self::api`] serves, kept so a test can ask it as a caller
    /// no socket can be (one off this computer, [`Node::api_routes`]).
    routes: axum::Router,
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
    /// The record API's port, where each lend has its origin (u5-2).
    api_port: u16,
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
        Self::start_every(data_dir, key, relay, seeds, api, DEFAULT_RING_SYNC_INTERVAL).await
    }

    /// [`Node::start`], with the round every `every` rather than every
    /// [`DEFAULT_RING_SYNC_INTERVAL`]. The scale instrument
    /// (`tests/scale.rs`) passes a day, so that no node's own round runs
    /// inside one it is timing.
    pub async fn start_every(
        data_dir: PathBuf,
        key: SigningKey,
        relay: &RelayConfig,
        seeds: SeedText,
        api: SocketAddr,
        every: Duration,
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
        let admitting = rail.clone();
        let host = Arc::new(Host {
            transport: transport.clone(),
            rail: rail.clone(),
            endpoint: endpoint.clone(),
            actor: actor.clone(),
            api_port: api.addr().port(),
        });
        let door = crate::door::Door {
            rail: rail.clone(),
            key: endpoint.id().as_bytes().to_owned(),
            data_dir: data_dir.clone(),
            using: crate::door::Using::spawn(),
        };
        let asking = Asking {
            host: host.clone(),
            door,
        };
        // The record, and beside it what an app asks of this computer (u5-1).
        let routes = crate::api::router(
            RailApi {
                rail: rail.clone(),
                nudge: nudge.clone(),
                live: live.clone(),
                transport,
            },
            crate::surface::router(asking.clone(), seeds),
            crate::origin::router(asking.clone()),
        );
        let api = crate::api::serve(routes.clone(), api);
        if let Err(e) = crate::api::write_address(&data_dir, api.addr()) {
            endpoint.close().await;
            return Err(e.into());
        }
        tracing::info!(
            target: "node",
            actor,
            data_dir = %data_dir.display(),
            dial = ?format_dial_string(&endpoint.addr()),
            api = %api.addr(),
            "node: endpoint bound with the signing key"
        );
        // The members' protocol serves sync and the live lane, the door to
        // what this computer lends (u3-4), and what the people of a
        // member's groups lend (u4-5).
        let holders = crate::door::holders_router(asking.door.clone());
        let members = ring_routes::router(RingInbound {
            rail: rail.clone(),
            live,
        })
        .merge(crate::door::router(asking.door.clone()))
        .merge(people_router(asking));
        let redeem = crate::redeem::router(rail, seeds);
        let acceptor =
            IrohAcceptor::spawn(endpoint.clone(), move |alpn: &[u8], dialer: &PublicKey| {
                match admit_dialer(&admitting, alpn, dialer) {
                    Some(Alpn::Member) => Admit::Serve(members.clone()),
                    Some(Alpn::Redeem) => Admit::Serve(redeem.clone()),
                    Some(Alpn::Lend) => Admit::Serve(holders.clone()),
                    None => Admit::Refuse,
                }
            });
        let round = spawn_ring_sync_loop(host.clone(), every, nudge);
        Ok(Self {
            host,
            api,
            routes,
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

    /// The routes served at [`Node::api_addr`], behind their loopback
    /// check: what a test hands a request whose caller it names.
    pub fn api_routes(&self) -> axum::Router {
        self.routes.clone()
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
        let groups = self.host.standings().unwrap_or_default();
        self.host.lends_in(&groups, None).await
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

    /// The key this node signs and dials with, as the record spells an actor.
    pub(crate) fn actor(&self) -> &str {
        &self.actor
    }

    /// The transport a member's contact is resolved through.
    pub(crate) fn transport(&self) -> &IrohTransport {
        &self.transport
    }

    /// Where this computer's browser opens the lend `name` of the computer
    /// whose key is `lender`, actor hex: the lend's origin, or `None` for a
    /// key that is not one.
    pub(crate) fn origin(&self, lender: &str, name: &str) -> Option<String> {
        let key: [u8; 32] = hex::decode(lender).ok()?.try_into().ok()?;
        let label = svrngs_core::pass::Label::of(&key, name);
        Some(crate::origin::origin(&label, self.api_port))
    }

    /// This round's members, or `None` when the node has none to round
    /// with: every computer's key standing in a group this node holds, but
    /// this node's. A group whose register is unreadable adds nobody, as the
    /// round offers it to nobody.
    pub(crate) async fn members(&self) -> Option<RoundMembers> {
        let peers = self.standing_peers(&self.standings()?);
        Some(RoundMembers::of(
            peers
                .into_iter()
                .map(|(actor, person, key)| (format!("{person}/{}", &actor[..8]), key))
                .collect(),
        ))
    }

    /// Every group this node holds, with who stands in it; or `None` when
    /// the groups held cannot be listed. A group whose register is
    /// unreadable is left out.
    fn standings(&self) -> Option<Vec<(String, BTreeMap<String, Member>)>> {
        let namespaces = match self.rail.namespaces() {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(target: "node", error = %e, "node: cannot list the groups held, so the round has no members");
                return None;
            }
        };
        let mut out = Vec::new();
        for namespace in namespaces {
            match self.rail.standing(&namespace, &Ed25519Verifier) {
                Ok(Standing::Members(m)) => out.push((namespace, m)),
                Ok(_) => continue,
                Err(e) => {
                    tracing::warn!(target: "node", namespace, error = %e, "node: a group's register is unreadable, so it adds no member");
                }
            }
        }
        Some(out)
    }

    /// Every key standing in `groups` that is dialled — a computer's,
    /// never a browser's (u4-1) — but this node's, once each, by its
    /// actor, with the person the first group naming it names.
    fn standing_peers(
        &self,
        groups: &[(String, BTreeMap<String, Member>)],
    ) -> Vec<(String, Person, PublicKey)> {
        let mut keys: BTreeMap<String, (Person, PublicKey)> = BTreeMap::new();
        for (namespace, standing) in groups {
            for (actor, person, key) in dialled(&self.actor, namespace, standing) {
                if keys.contains_key(actor) {
                    continue;
                }
                let name = format!("{person}/{}", &actor[..8]);
                tracing::debug!(target: "node", namespace, member = %name, "node: a member stands in the group, to dial");
                keys.insert(actor.clone(), (person.clone(), key));
            }
        }
        keys.into_iter()
            .map(|(actor, (person, key))| (actor, person, key))
            .collect()
    }

    /// What the other computers' keys standing in `groups` lend, but
    /// `asker`'s, which knows its own, each computer asked now and all at
    /// once, and where this computer reaches each lend. Nothing is kept
    /// between asks.
    async fn lends_in(
        &self,
        groups: &[(String, BTreeMap<String, Member>)],
        asker: Option<&str>,
    ) -> Lends {
        let mut peers = self.standing_peers(groups);
        peers.retain(|(actor, _, _)| Some(actor.as_str()) != asker);
        let mut asks = Vec::with_capacity(peers.len());
        for (actor, person, key) in peers {
            let transport = self.transport.clone();
            asks.push(tokio::spawn(async move {
                let answer = ask_lends(&transport, &actor, key).await;
                (person, actor, answer)
            }));
        }
        let mut lends = Lends::default();
        for ask in asks {
            match ask.await {
                Ok((person, key, Some(answered))) => {
                    for offered in answered {
                        let Some(address) = self.origin(&key, offered.name.as_str()) else {
                            continue;
                        };
                        lends.lent.push(Lent {
                            person: person.clone(),
                            key: key.clone(),
                            offered,
                            address,
                        });
                    }
                }
                Ok((person, _, None)) => lends.unanswered.push(person),
                Err(e) => {
                    tracing::warn!(target: "node", error = %e, "node: lends — an ask ended without an answer")
                }
            }
        }
        lends.lent.sort_by(|a, b| {
            (a.person.as_str(), &a.offered.name).cmp(&(b.person.as_str(), &b.offered.name))
        });
        tracing::debug!(target: "node", lent = lends.lent.len(), unanswered = lends.unanswered.len(),
            "node: lends — asked every computer of the groups");
        lends
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

/// Ask the computer `key` what it lends, or `None` when it did not
/// answer, or answered with something that is not a list of lends.
async fn ask_lends(transport: &IrohTransport, actor: &str, key: PublicKey) -> Option<Vec<Offered>> {
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
            Some(a.lends)
        }
        Err(e) => {
            tracing::debug!(target: "node", actor, error = %e, "node: lends — the answer is not a list of lends");
            None
        }
    }
}

/// Where a member asks what the people of its groups lend (u4-5).
pub const PEOPLE_LENDS_ROUTE: &str = "/people/lends";

/// A lend as a member is told it when asking what the people of its
/// groups lend: whose it is, the key their computer is dialled by, and
/// the lend (DESIGN §3, call 4: the asker dials the lender itself).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lending {
    pub person: Person,
    pub key: String,
    #[serde(flatten)]
    pub offered: Offered,
}

/// What the people route reads: the node's groups and transport, and its
/// own door.
#[derive(Clone)]
pub(crate) struct Asking {
    pub(crate) host: Arc<Host>,
    door: crate::door::Door,
}

impl Asking {
    /// This computer's door.
    pub(crate) fn door(&self) -> &crate::door::Door {
        &self.door
    }
}

fn people_router(asking: Asking) -> axum::Router {
    axum::Router::new()
        .route(PEOPLE_LENDS_ROUTE, get(people_lends))
        .with_state(asking)
}

/// `GET /people/lends`: what the people of the groups the dialer stands
/// in lend — this computer's lends under its own person and key, and each
/// other computer's but the asker's, asked now as [`Node::lends`] asks,
/// with its key — and whose computer did not answer. A browser asks it of the computer it
/// dials, which answers what it can see and proxies no lend (DESIGN §3,
/// call 4). A group the dialer does not stand in adds nothing.
async fn people_lends(State(at): State<Asking>, dialer: Option<Extension<Dialer>>) -> Response {
    let Some(Extension(dialer)) = dialer else {
        tracing::warn!(target: "node", "node: people's lends refused — no verified dialer, so the request did not come on an iroh stream");
        return err(
            StatusCode::FORBIDDEN,
            "what a group's people lend is asked only from within it",
        );
    };
    let asker = dialer.actor();
    let mut groups = at.host.standings().unwrap_or_default();
    groups.retain(|(_, standing)| standing.contains_key(&asker));
    let own = groups
        .iter()
        .find_map(|(_, standing)| standing.get(&at.host.actor))
        .map(|m| m.person.clone());
    let Some(own) = own else {
        tracing::debug!(target: "node", asker, "node: people's lends refused — the dialer stands in no group this computer stands in");
        return err(StatusCode::FORBIDDEN, "this computer lends nothing to you");
    };
    let (lent, unanswered) = match at.lent(&groups, &own, Some(&asker)).await {
        Ok(answer) => answer,
        Err(refused) => return refused,
    };
    tracing::debug!(target: "node", asker, groups = groups.len(), lent = lent.len(), unanswered = unanswered.len(),
        "node: people's lends — answered what the people of the asker's groups lend, each with its lender's key");
    Json(serde_json::json!({ "lent": lent, "unanswered": unanswered })).into_response()
}

impl Asking {
    /// What the people of `groups` lend: this computer's own lends under
    /// `own`, its person, and each other computer's but `asker`'s, asked now
    /// as [`Node::lends`] asks, each with its lender's key, sorted by
    /// person then name; and whose computer did not answer. The refusal is
    /// [`crate::door::offered`]'s, when this computer's declarations cannot
    /// be read.
    pub(crate) async fn lent(
        &self,
        groups: &[(String, BTreeMap<String, Member>)],
        own: &Person,
        asker: Option<&str>,
    ) -> Result<(Vec<Lending>, Vec<Person>), Response> {
        let offered = crate::door::offered(&self.door).await?;
        let others = self.host.lends_in(groups, asker).await;
        let mut lent: Vec<Lending> = offered
            .into_iter()
            .map(|offered| Lending {
                person: own.clone(),
                key: self.host.actor.clone(),
                offered,
            })
            .chain(others.lent.into_iter().map(|l| Lending {
                person: l.person,
                key: l.key,
                offered: l.offered,
            }))
            .collect();
        lent.sort_by(|a, b| {
            (a.person.as_str(), &a.offered.name).cmp(&(b.person.as_str(), &b.offered.name))
        });
        Ok((lent, others.unanswered))
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
        Some(Alpn::Lend) => {
            tracing::debug!(target: "node", actor, "node: admitted to present a pass for a lend — any dialer may, and the door asks the pass");
            return Some(Alpn::Lend);
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
