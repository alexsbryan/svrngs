// SPDX-License-Identifier: AGPL-3.0-or-later
//! The node (PLAN §4, U1): one group member's computer — its journals, the
//! iroh endpoint it dials and is dialled on, the accept loop serving the sync
//! route, the round, and (U3) the loopback listener serving the record to
//! this computer's apps ([`crate::api`]). It replaces upstream's `RailsDaemon` and `RailsNode`
//! wiring (`commonwealth/crates/commonwealth-rails/src/lib.rs:387-485` and
//! `574-632` at `e1a43bcd2`), which is not forked: gossip, join, the mesh
//! store, the work plane, presence, the self-heal and
//! `rails.toml` stay behind (the inventory's `RailsDaemon` row).
//!
//! **One key signs and dials** (PLAN §4, U1's inventory): the endpoint is
//! bound with the key the rail signs with, as upstream's was
//! (`lib.rs:400-402`), so a member's actor is the key iroh verifies when it
//! dials, and a key standing in a group's register is a key the round can
//! dial.
//!
//! **A link is redeemed on a second protocol** (PLAN §4, U2's inventory,
//! the acceptor row): [`Alpn::Redeem`] is served to any dialer, with its
//! two routes, the ask and the redemption ([`crate::redeem`]), so a key standing nowhere reaches that and
//! nothing else; [`Alpn::Member`] stays closed to it.
//!
//! **What the people of its groups lend is asked, not stored** (DESIGN §3,
//! "Lends are asked, not stored"; u3-5): [`Node::lends`] asks every computer's key standing in a
//! group the node holds, now, what its computer lends ([`crate::door`]'s `GET /lends`),
//! and answers each lend with whose it is and its origin on this
//! computer's loopback ([`crate::origin`]), which reaches it on a request
//! on the stream to its lender. A computer that does not say is named, its lends absent
//! rather than stale, and by how ([`Unlisted`]): it did not answer, it
//! closed the connection as not admitted — it has not heard this computer
//! is in its group — or it answered with a refusal. A member asks the same of the computer it dials,
//! `GET /people/lends` ([`PEOPLE_LENDS_ROUTE`], u4-5): that computer's
//! lends and the others' of the groups the asker stands in, each with its
//! lender's key, so a phone dials the lender itself (DESIGN §3, call 4).
//! The same computer brings the shelf's apps for it (`POST /shelf`) and
//! writes its yes to a carried app's new version (`POST /runs`; canon-pin),
//! each as the asker.
//!
//! **The group decides who is in it** (DESIGN §6): the register —
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
use svrngs_core::pass::Label;
use svrngs_core::relay::Url;

use crate::door::LENDS_ROUTE;
use crate::iroh::{
    actor_key, build_relayed_endpoint, format_dial_string, key_actor, Admit, Alpn, Contact, Dialer,
    Endpoint, EndpointAddr, IrohAcceptor, IrohTransport, PublicKey, RelayConfig, SecretKey, Unsent,
};
use crate::lend::Offered;
use crate::rail::{visiting, LiveBuffer, RailApi};
use crate::redeem::SeedText;
use crate::register::{Member, Standing};
use crate::ring_routes::{self, err, RingInbound};
use crate::ring_sync::{
    exchange, spawn_ring_sync_loop, AskRound, ExchangeOutcome, RingSyncHandle, RoundMembers,
    Rounds, DEFAULT_RING_SYNC_INTERVAL,
};
use crate::{actor_of, Person, RingRail, SigningKey};

/// Groups held here, each by its namespace with who stands in it.
pub(crate) type Groups = Vec<(String, BTreeMap<String, Member>)>;

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

/// How long an ask of what the people of a group lend waits on one
/// computer before naming it unanswered (asleep-1-none-held): an awake
/// lender answered a phone's page in 90-324 ms and an asleep one's dial
/// gave up at about 10 s (`demo/runs/mnp/2026-10-09/README.md`, "What no
/// prediction named" 2), so a computer that answers is not held by one
/// that does not.
pub const ANSWER_WITHIN: Duration = Duration::from_secs(3);

/// Until this process is asked to stop: Ctrl-C, or SIGTERM, which a
/// service manager and `kill` send (syp-6 b). Its caller then closes the
/// node's endpoint, telling every peer each connection is closed. A
/// process gone on SIGTERM with it open left each peer's iroh sending to
/// its dead address alone, the path in use, until that idled out at 15 s
/// (iroh-1.0.2 `remote_state.rs:799`, `socket.rs:117`), so its next
/// process was not reached: syp-5's S3, Ana dialling Ben's last process
/// (`demo/runs/syp/2026-10-10/rerun/README.md` §6, item 3). Closed, a
/// peer's last connection to it closes and iroh forgets the path
/// (`remote_state.rs:486-488`). A process killed outright still leaves
/// it, until it idles out.
pub async fn asked_to_stop() {
    #[cfg(unix)]
    let term = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut term) => drop(term.recv().await),
            Err(e) => {
                tracing::warn!(target: "node", error = %e, "node: SIGTERM cannot be waited on, so it stops this process with its endpoint open");
                std::future::pending::<()>().await
            }
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! {
        said = tokio::signal::ctrl_c() => match said {
            Ok(()) => tracing::debug!(target: "node", "node: asked to stop by Ctrl-C"),
            Err(e) => tracing::warn!(target: "node", error = %e, "node: Ctrl-C cannot be waited on, so stopping now"),
        },
        () = term => tracing::debug!(target: "node", "node: asked to stop by SIGTERM"),
    }
}

/// What the people of this computer's groups lend, asked now.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lends {
    pub lent: Vec<Lent>,
    pub unlisted: Unlisted,
}

/// The people whose computer did not say what they lend, so it is not
/// known: absent, which is not lending nothing — each by how it was not
/// said, so a computer that answered is never said not to have answered
/// (bash-first-minute (c); Demo C's dry run, finding 6).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Unlisted {
    /// Whose computer did not answer.
    pub unanswered: Vec<Person>,
    /// Whose computer answered that it has not heard this one is in its
    /// group: it closed the connection as not admitted.
    pub unheard: Vec<Person>,
    /// Whose computer answered with a refusal, or with what is not a list
    /// of lends.
    pub refused: Vec<Person>,
    /// Whose computer runs an older svrngs ([`crate::version`]): what it
    /// lends is not read.
    pub older: Vec<Person>,
}

impl Unlisted {
    /// Whether `person`'s computer did not say what they lend, by any of
    /// the four.
    pub(crate) fn names(&self, person: &Person) -> bool {
        let lists = [&self.unanswered, &self.unheard, &self.refused, &self.older];
        lists.iter().any(|l| l.contains(person))
    }
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
    #[error(transparent)]
    Oidc(#[from] crate::oidc::KeyFailure),
}

/// A running node. Dropping it stops its round and its accept loop.
pub struct Node {
    host: Arc<Host>,
    /// The state its routers ask, kept for what only a test asks.
    pub(crate) asking: Asking,
    api: crate::api::Serving,
    /// What [`Self::api`] serves, kept so a test can ask it as a caller
    /// no socket can be (one off this computer, `Node::api_routes`).
    #[cfg(any(test, feature = "test-support"))]
    routes: axum::Router,
    _acceptor: IrohAcceptor,
    _round: RingSyncHandle,
    /// The room's one owner ([`crate::room::owner::spawn`]), stopped with the node.
    _room: n0_future::task::AbortOnDropHandle<()>,
}

/// What the round reads: the journals, the endpoint's transport, and the key
/// this node signs and dials with. Upstream's round read these through its
/// `RingSyncHost` trait, which `RailsDaemon` implemented; with one host the
/// trait went, and the round calls this (RUST_ARCH 7).
pub struct Host {
    pub(crate) rail: Arc<RingRail>,
    pub(crate) endpoint: Endpoint,
    /// The transport a member's contact is resolved through.
    pub(crate) transport: Arc<IrohTransport>,
    /// The key this node signs and dials with, as the record spells an actor.
    pub(crate) actor: String,
    /// Whether this computer reaches the network through a relay at all
    /// ([`RelayConfig`]): with none, it says so in the record.
    relayed: bool,
    /// How it reaches the network: the relay a link made on the home page
    /// carries when none is reached yet ([`crate::invite::carried`]).
    pub(crate) relay: RelayConfig,
    /// The record API's port, where each lend has its origin (u5-2).
    pub(crate) api_port: u16,
    /// Held while a link is minted, and across `new`'s keeping of a group
    /// and its first link ([`Host::keep_linked`]): an issuer's newer link
    /// closes their last, so the home page's link, made once the group
    /// shows, is never minted between the two and closed by `new`'s
    /// (capability-scenes-found (e)).
    #[expect(clippy::disallowed_types, reason = "the links' sequencer")]
    pub(crate) minting: std::sync::Mutex<()>,
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
            relayed: relay.n0_services || !relay.relay_urls.is_empty(),
            relay: relay.clone(),
            api_port: api.addr().port(),
            minting: Default::default(),
        });
        let (folders, blobs) =
            crate::folder::Folders::start(&data_dir, host.clone(), nudge.clone())
                .await
                .map_err(NodeError::Endpoint)?;
        let rounds = Rounds::new(Default::default());
        // One provider, its key and its code store, shared by the door that
        // mints a code and the API that redeems it.
        let oidc = crate::oidc::Provider::new(data_dir.clone(), host.api_port)?;
        let door = crate::door::Door {
            ring: AskRound::new(nudge.clone(), &rounds),
            rail: rail.clone(),
            key: endpoint.id().as_bytes().to_owned(),
            data_dir: data_dir.clone(),
            using: crate::door::Using::spawn(),
            choices: crate::door::choices::Choices::spawn(),
            folders: folders.clone(),
            oidc: oidc.clone(),
        };
        let (room, owner) = crate::room::owner::Room::new();
        let asking = Asking {
            host: host.clone(),
            door,
            folders: folders.clone(),
            seeds,
            oidc,
            room,
        };
        let room = crate::room::owner::spawn(asking.clone(), owner);
        // The record, and beside it what an app asks of this computer (u5-1).
        let rail_api = RailApi {
            rail: rail.clone(),
            nudge: nudge.clone(),
            live: live.clone(),
            transport,
        };
        let routes = crate::api::router(
            rail_api.clone(),
            crate::surface::router(asking.clone())
                .merge(asking.oidc.clone().router())
                .merge(crate::folder::router(folders.clone()))
                .merge(crate::removal::router(asking.clone()))
                .merge(crate::room::route::router(asking.clone())),
            crate::home::router(asking.clone(), rail_api),
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
        let kept = asking.clone();
        let inbound = RingInbound {
            rail: rail.clone(),
            live,
            wake: nudge.clone(),
        };
        let members = ring_routes::router(inbound.clone())
            .merge(crate::door::router(asking.door.clone()))
            .merge(people_router(asking))
            .merge(crate::folder::members_router(folders));
        let redeem = crate::redeem::router(kept.clone());
        // A dialer the keeper cut, on the members' protocol, is told so in
        // words on every route (U7), and served nothing else but the lines
        // that cut it, where it asks to sync (#179; ring_routes.rs `told_cut`).
        let gone = ring_routes::router(inbound).fallback(|| async { crate::door::gone() });
        let acceptor = IrohAcceptor::spawn(
            endpoint.clone(),
            move |alpn: &[u8], dialer: &PublicKey| match admit_dialer(&admitting, alpn, dialer) {
                Some(Alpn::Member) => Admit::Serve(members.clone()),
                Some(Alpn::Redeem) => Admit::Serve(redeem.clone()),
                Some(Alpn::Lend) => Admit::Serve(holders.clone()),
                Some(Alpn::Blobs) => Admit::Blobs(blobs.clone()),
                None if Alpn::of(alpn) == Some(Alpn::Member)
                    && admitting.removed_anywhere(&key_actor(dialer)).is_some() =>
                {
                    tracing::debug!(target: "node", "node: the dialer was cut here, so it is told its cut");
                    Admit::Serve(gone.clone())
                }
                None => Admit::Refuse,
            },
        );
        let round = spawn_ring_sync_loop(host.clone(), every, nudge, rounds);
        Ok(Self {
            host,
            asking: kept,
            api,
            #[cfg(any(test, feature = "test-support"))]
            routes,
            _acceptor: acceptor,
            _round: round,
            _room: room,
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

    /// This computer's folders, where `new` and `join` place a group's.
    pub fn folders(&self) -> &crate::folder::Folders {
        &self.asking.folders
    }

    /// What the round reads, and what makes a link ([`crate::invite`]).
    pub fn host(&self) -> &Host {
        &self.host
    }

    /// What every other computer's key standing in a group this node holds
    /// lends, each computer asked now and all at once, and where this computer reaches
    /// each lend. Nothing is kept between asks.
    pub async fn lends(&self) -> Lends {
        let groups = self.host.standings().unwrap_or_default();
        self.host.lends_in(&groups, None).await
    }

    /// Whether this computer reaches the network through a relay at all.
    pub fn relayed(&self) -> bool {
        self.host.relayed
    }
}

impl Host {
    /// One exchange of `namespace` with the node at `peer`, whatever this
    /// node's register says of it: a joiner syncs once from the issuer that
    /// admitted it, whom its own register may not know yet (PLAN §4,
    /// U2). The peer's addresses are seeded on the transport, so the round
    /// reaches it too once the register names it.
    pub async fn sync_with(&self, namespace: &str, peer: &EndpointAddr) -> ExchangeOutcome {
        let addrs: Vec<_> = peer.ip_addrs().copied().collect();
        self.transport.add_known_peer(peer.id, addrs.clone());
        let contact = Contact {
            key: peer.id,
            relay_url: peer.relay_urls().next().map(|r| r.to_string()),
            iroh_direct_addrs: addrs,
        };
        let to = (&*self.transport, &contact);
        let mut out = exchange(to, &self.rail, namespace).await;
        tracing::debug!(target: "node", namespace, pulled = out.pulled, pushed = out.pushed, stop = ?out.stop,
            "node: sync_with — one exchange with the peer");
        // A joiner says its relay once the exchange brought its admission,
        // and takes it to the peer in a second: until one member holds it,
        // no member can dial this computer (relay-in-record).
        if out.stop.is_none() && self.say_relay() {
            out.pushed += exchange(to, &self.rail, namespace).await.pushed;
        }
        out
    }

    /// Say in each group held where this computer is reached — its home
    /// relay, or none when it reaches the network through none — by
    /// svrngs-core's `RelayAt`, which writes only when the record says
    /// otherwise (relay-in-record). With relays and none reached yet it
    /// says nothing, so a computer starting never withdraws its relay.
    /// Whether anything was written.
    pub fn say_relay(&self) -> bool {
        let home = self.endpoint.addr().relay_urls().next().cloned();
        if home.is_none() && self.relayed {
            tracing::debug!(target: "node", "node: no relay reached yet, so the record keeps what it says of this computer");
            return false;
        }
        let at = svrngs_core::RelayAt {
            key: self.actor.clone(),
            url: home.map(Into::into),
        };
        let mut wrote = false;
        let held = self.rail.namespaces().inspect_err(|e| {
            tracing::warn!(target: "node", error = %e, "node: cannot list the groups held, so this computer's relay is said in none");
        });
        for namespace in held.unwrap_or_default() {
            // The kept register first: a commit reads the whole journal under
            // its lock, which every idle round paid (scale.rs's ceiling, 0.59 s
            // at 42e36ca against 0.02 s before). The core still decides a move.
            if let Ok(Standing::Members(m)) = self.rail.standing(&namespace) {
                if m.get(&self.actor).is_some_and(|me| me.relay == at.url) {
                    tracing::debug!(target: "node", namespace, url = ?at.url,
                        "node: the group already names this computer's relay, so nothing is committed");
                    continue;
                }
            }
            match self.rail.commit(&namespace, &at) {
                Ok(Ok(said)) => wrote |= !said.is_empty(),
                Ok(Err(why)) => {
                    tracing::debug!(target: "node", namespace, ?why, "node: this computer's relay not said in the group")
                }
                Err(e) => {
                    tracing::warn!(target: "node", namespace, error = %e, "node: this computer's relay could not be said in the group")
                }
            }
        }
        wrote
    }

    /// The contact the computer `actor` is dialled by: its key and the
    /// relay the groups held name for it ([`relay_of`]); `None` when it
    /// stands in none, or its key is no iroh key.
    pub(crate) fn contact(&self, actor: &str) -> Option<Contact> {
        self.rail.standing_anywhere(actor)?;
        let groups = self.standings()?;
        Some(Contact::named(actor_key(actor)?, relay_of(&groups, actor)))
    }

    /// Where this computer's browser opens the lend `name` of the computer
    /// whose key is `lender`, actor hex: the lend's origin, or `None` for a
    /// key that is not one.
    pub(crate) fn origin(&self, lender: &str, name: &str) -> Option<String> {
        let label = Label::of_hex(lender, name)?;
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
                .map(|(actor, person, contact)| (format!("{person}/{}", &actor[..8]), contact))
                .collect(),
        ))
    }

    /// Every group this node holds, with who stands in it; or `None` when
    /// the groups held cannot be listed. A group whose register is
    /// unreadable is left out.
    pub(crate) fn standings(&self) -> Option<Groups> {
        let namespaces = match self.rail.namespaces() {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(target: "node", error = %e, "node: cannot list the groups held, so the round has no members");
                return None;
            }
        };
        let mut out = Vec::new();
        for namespace in namespaces {
            match self.rail.standing(&namespace) {
                Ok(Standing::Members(m)) => out.push((namespace, m)),
                Ok(_) => continue,
                Err(e) => {
                    tracing::warn!(target: "node", namespace, error = %e, "node: a group's register is unreadable, so it adds no member");
                }
            }
        }
        Some(out)
    }

    /// The groups held here that `actor` stands in, as [`Self::standings`]
    /// gives them, and this computer's person in the first of them it
    /// stands in too: whom a group's lends are answered to, and as whom.
    /// Groups that cannot be listed are svrngs's refusal, never no group
    /// (ralph/REVIEW_FINDINGS.md #103).
    pub(crate) fn shared_with(&self, actor: &str) -> Result<(Groups, Option<Person>), Response> {
        let Some(mut groups) = self.standings() else {
            return Err(crate::door::LendRefusal::Fault.say(
                StatusCode::INTERNAL_SERVER_ERROR,
                "the groups this computer is in cannot be read on it",
            ));
        };
        groups.retain(|(_, standing)| standing.contains_key(actor));
        let own = groups
            .iter()
            .find_map(|(_, standing)| standing.get(&self.actor))
            .map(|m| m.person.clone());
        Ok((groups, own))
    }

    /// Every key standing in `groups` that is dialled — a computer's,
    /// never a browser's (u4-1) — but this node's, once each, by its
    /// actor, with the person the first group naming it names and the
    /// relay `groups` name for it ([`relay_of`]).
    fn standing_peers(
        &self,
        groups: &[(String, BTreeMap<String, Member>)],
    ) -> Vec<(String, Person, Contact)> {
        let mut keys: BTreeMap<String, (Person, Contact)> = BTreeMap::new();
        let relay = |a: &str| relay_of(groups, a);
        for (namespace, standing) in groups {
            for (actor, person, key) in dialled(&self.actor, namespace, standing, relay) {
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
    /// once, each for at most [`ANSWER_WITHIN`], and where this computer
    /// reaches each lend. Nothing is kept between asks.
    pub(crate) async fn lends_in(
        &self,
        groups: &[(String, BTreeMap<String, Member>)],
        asker: Option<&str>,
    ) -> Lends {
        let mut peers = self.standing_peers(groups);
        peers.retain(|(actor, _, _)| Some(actor.as_str()) != asker);
        let mut asks = Vec::with_capacity(peers.len());
        for (actor, person, contact) in peers {
            let transport = self.transport.clone();
            asks.push(tokio::spawn(async move {
                let asked = tokio::time::timeout(ANSWER_WITHIN, ask_lends(&transport, &actor, contact));
                let answer = asked.await.unwrap_or_else(|_| {
                    tracing::debug!(target: "node", actor, within_ms = ANSWER_WITHIN.as_millis() as u64,
                        "node: lends — the computer did not answer in time, so it is named unanswered");
                    Answered::Unanswered
                });
                (person, actor, answer)
            }));
        }
        let mut lends = Lends::default();
        for ask in asks {
            match ask.await {
                Ok((person, key, Answered::Lends(answered))) => {
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
                Ok((person, _, Answered::Unanswered)) => lends.unlisted.unanswered.push(person),
                Ok((person, _, Answered::Unheard)) => lends.unlisted.unheard.push(person),
                Ok((person, _, Answered::Refused)) => lends.unlisted.refused.push(person),
                Ok((person, _, Answered::Older)) => lends.unlisted.older.push(person),
                Err(e) => {
                    tracing::warn!(target: "node", error = %e, "node: lends — an ask ended without an answer")
                }
            }
        }
        lends.lent.sort_by(|a, b| {
            (a.person.as_str(), &a.offered.name).cmp(&(b.person.as_str(), &b.offered.name))
        });
        tracing::debug!(target: "node", lent = lends.lent.len(), unlisted = ?lends.unlisted,
            "node: lends — asked every computer of the groups");
        lends
    }
}

/// The relay `key` is dialled through: the first of `groups`, in the order
/// they are listed, that names one for it. The one rule the round, the lend
/// asks, a folder's fetch, a lend's origin and the people answer ask
/// (ralph/REVIEW_FINDINGS.md #127), so a group that has not yet heard where
/// `key` is reached never stands for one that has. Two that name different
/// relays are not told apart by age; `say_relay` writes in every group, so
/// they agree once each has synced.
pub(crate) fn relay_of<'a>(
    groups: &'a [(String, BTreeMap<String, Member>)],
    key: &str,
) -> Option<&'a Url> {
    let named = (groups.iter()).find_map(|(ns, m)| Some((ns, m.get(key)?.relay.as_ref()?)));
    tracing::debug!(target: "node", key, group = named.map(|(ns, _)| ns.as_str()), "node: the relay a key is dialled through, from the first group naming one");
    named.map(|(_, url)| url)
}

/// The keys standing in `namespace` that this node, `this`, dials: each a
/// computer's ([`crate::Device::dialled`], u4-1) and an iroh key, with its
/// person and the relay `relay` gives it — [`relay_of`] the groups held for
/// the round and [`Node::lends`], the group's own for the live fan-out. The
/// one decider the round, [`Node::lends`] and the live fan-out
/// (`rail.rs`'s `live_peers`) ask, so a browser's key is skipped by all
/// three or by none.
pub(crate) fn dialled<'a, 'r>(
    this: &str,
    namespace: &str,
    standing: &'a BTreeMap<String, Member>,
    relay: impl Fn(&str) -> Option<&'r Url>,
) -> Vec<(&'a String, &'a Person, Contact)> {
    let mut out = Vec::new();
    for (actor, m) in standing {
        let (person, device) = (&m.person, m.device);
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
        out.push((actor, person, Contact::named(key, relay(actor))));
    }
    out
}

/// How a computer asked what it lends answered: a closed set (AGENTS 9),
/// one for each of [`Unlisted`]'s lists and the list itself.
enum Answered {
    Lends(Vec<Offered>),
    Unanswered,
    Unheard,
    Refused,
    Older,
}

/// Ask the computer `key` what it lends. One that has not heard of this
/// one closes the connection as not admitted, read from that same dial
/// (`demo/runs/first-minute/2026-10-12/`; arch-peer (c), where a second
/// dial asked it once a request through the bridge had failed).
async fn ask_lends(transport: &IrohTransport, actor: &str, contact: Contact) -> Answered {
    let asked = axum::http::Request::get(LENDS_ROUTE).body(Default::default());
    let asked = asked.expect("a fixed route");
    match transport.fetch(&contact, asked).await {
        Ok(r) if r.status().is_success() => answered(actor, r.body()),
        Ok(r) => {
            tracing::debug!(target: "node", actor, status = r.status().as_u16(), "node: lends — the computer refused the ask");
            Answered::Refused
        }
        Err(Unsent::NotAdmitted) => {
            tracing::debug!(target: "node", actor, "node: lends — the computer has not heard this one is in its group");
            Answered::Unheard
        }
        Err(e) => {
            tracing::debug!(target: "node", actor, error = %e, "node: lends — the computer did not answer");
            Answered::Unanswered
        }
    }
}

/// Computer `actor`'s answer `body` of what it lends: read only when it
/// carries this build's version or a later one ([`crate::version`]), so one
/// running an older svrngs is named so, never read field by field.
fn answered(actor: &str, body: &[u8]) -> Answered {
    #[derive(serde::Deserialize)]
    struct Answer {
        version: Option<u32>,
        lends: serde_json::Value,
    }
    let read = serde_json::from_slice::<Answer>(body);
    let version = read.as_ref().ok().map(|a| a.version);
    match read.and_then(|a| serde_json::from_value::<Vec<Offered>>(a.lends)) {
        _ if version.is_some_and(crate::version::older) => {
            tracing::debug!(target: "node", actor, version = ?version.flatten(), "node: lends — the computer runs an older svrngs, so what it lends is not read");
            Answered::Older
        }
        Ok(lends) => {
            tracing::debug!(target: "node", actor, lends = lends.len(), "node: lends — the computer answered");
            Answered::Lends(lends)
        }
        Err(e) => {
            tracing::debug!(target: "node", actor, error = %e, "node: lends — the answer is not a list of lends, so it is refused");
            Answered::Refused
        }
    }
}

/// Where a member asks what the people of its groups lend (u4-5).
pub const PEOPLE_LENDS_ROUTE: &str = "/people/lends";

/// A lend as a member is told it when asking what the people of its
/// groups lend: whose it is, the key their computer is dialled by and the
/// relay its group names for it, `null` when it names none
/// (relay-in-record), and the lend (DESIGN §3, call 4: the asker dials the
/// lender itself).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lending {
    pub person: Person,
    pub key: String,
    pub relay: Option<String>,
    #[serde(flatten)]
    pub offered: Offered,
}

/// The node's handles, one of each, and the one state of every router
/// that asks more than one of them (AUDIT §2.8, where `Surface` and the
/// redeem routes' `Issuer` each held a copy): its host, whose rail's
/// signer is the key a computer's origin signs its own passes with
/// ([`crate::origin`], sup-1); its door; its folders, where a group's is
/// placed ([`crate::folder::Folders::place`]); and how a group's seed is
/// read, for a link's ask and `/v1/groups`. [`RailApi`] and
/// [`RingInbound`] stay states of their own: tests build each without a
/// node (`tests/record_api.rs`, `ring_routes/tests.rs`).
#[derive(Clone)]
pub(crate) struct Asking {
    pub(crate) host: Arc<Host>,
    pub(crate) door: crate::door::Door,
    pub(crate) folders: crate::folder::Folders,
    pub(crate) seeds: SeedText,
    pub(crate) oidc: crate::oidc::Provider,
    /// The room's handle: its turns, and what it serves now ([`crate::room`]).
    pub(crate) room: crate::room::owner::Room,
}

fn people_router(asking: Asking) -> axum::Router {
    axum::Router::new()
        .route(PEOPLE_LENDS_ROUTE, get(people_lends))
        .route("/shelf", axum::routing::post(crate::folder::bring))
        .route("/runs", axum::routing::post(crate::folder::yes))
        .route("/carry", axum::routing::post(crate::rail::carry))
        .with_state(asking)
}

/// `GET /people/lends`: what the people of the groups the dialer stands
/// in lend — this computer's lends under its own person and key, and each
/// other computer's but the asker's, asked now as [`Node::lends`] asks,
/// with its key — and whose computer did not say, and how. A browser asks it of the computer it
/// dials, which answers what it can see and proxies no lend (DESIGN §3,
/// call 4). A group the dialer does not stand in adds nothing. Beside them,
/// every removal and ask to remove in those groups, from each register
/// (v1-keeper-office (b)), and who is visiting them now (guests-pages;
/// [`crate::rail::visiting`]): `null` when one cannot be read here; and the
/// people standing in them, as `/v1/rail/people` answers them, for the
/// door (v1ui-door (d)); each carried app whose files changed since the
/// version that runs as the asker ([`crate::folder::Folders::changed`];
/// canon-pin (c)); and this build's version, which the page checks
/// once ([`crate::version`]).
async fn people_lends(
    State(at): State<Asking>,
    dialer: Option<Extension<Dialer>>,
) -> Result<Response, Response> {
    let Some(Extension(dialer)) = dialer else {
        tracing::warn!(target: "node", "node: people's lends refused — no verified dialer, so the request did not come on an iroh stream");
        return Err(err(
            StatusCode::FORBIDDEN,
            "what a group's people lend is asked only from within it",
        ));
    };
    let asker = dialer.actor();
    let (groups, own) = at.host.shared_with(&asker)?;
    let Some(own) = own else {
        tracing::debug!(target: "node", asker, "node: people's lends refused — the dialer stands in no group this computer stands in");
        return Err(err(StatusCode::FORBIDDEN, crate::door::LENDS_NOTHING));
    };
    let (lent, unlisted) = at.lent(&groups, &own, Some(&asker)).await?;
    let registers = groups
        .iter()
        .map(|(namespace, _)| {
            at.host.rail.register(namespace).inspect_err(|e| {
                tracing::warn!(target: "node", asker, namespace, error = %e, "node: people's lends — a group's register is unreadable, so its removals and who is visiting are not known");
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .ok();
    let (me, rs) = (at.host.actor.as_str(), registers.as_deref());
    let removals: Option<Vec<_>> =
        rs.map(|rs| rs.iter().flat_map(|r| r.removals.clone()).collect());
    let on: Option<Vec<_>> = rs.and_then(|rs| rs.iter().map(|r| visiting(r, me)).collect());
    tracing::debug!(target: "node", asker, groups = groups.len(), lent = lent.len(), ?unlisted,
        removals = ?removals.as_ref().map(Vec::len),
        "node: people's lends — answered what the people of the asker's groups lend, each with its lender's key");
    let mut body = serde_json::json!(unlisted);
    body["lent"] = serde_json::json!(lent);
    body["removals"] = serde_json::json!(removals);
    body["visiting"] = serde_json::json!(on.map(|each| each.concat()));
    let people: BTreeMap<_, _> = groups.iter().flat_map(|(_, s)| s.clone()).collect();
    body["people"] = crate::rail::standing_json(&people);
    body["shelf"] = crate::folder::shelf();
    body["changed"] = at.folders.changed(&groups, &asker);
    body["version"] = crate::version::VERSION.into();
    Ok(Json(body).into_response())
}

impl Asking {
    /// What the people of `groups` lend: this computer's own lends under
    /// `own`, its person, and each other computer's but `asker`'s, asked now
    /// as [`Node::lends`] asks, each with its lender's key, sorted by
    /// person then name; and whose computer did not say. The refusal is
    /// [`crate::door::offered`]'s, when this computer's declarations cannot
    /// be read.
    pub(crate) async fn lent(
        &self,
        groups: &[(String, BTreeMap<String, Member>)],
        own: &Person,
        asker: Option<&str>,
    ) -> Result<(Vec<Lending>, Unlisted), Response> {
        let offered = crate::door::offered(&self.door).await?;
        let others = self.host.lends_in(groups, asker).await;
        let mine = (offered.into_iter()).map(|o| (own.clone(), self.host.actor.clone(), o));
        let theirs = (others.lent.into_iter()).map(|l| (l.person, l.key, l.offered));
        let mut lent: Vec<Lending> = (mine.chain(theirs))
            .map(|(person, key, offered)| Lending {
                relay: relay_of(groups, &key).map(ToString::to_string),
                person,
                key,
                offered,
            })
            .collect();
        // A carried app only in its own group (COMMUNITY_APPS §7, found
        // 3), each computer's copy listed, so a phone knows every computer
        // to reach the group through; the pages show one (question.js `once`).
        let carries = |g: &String| groups.iter().any(|(ns, _)| ns == g);
        lent.retain(|l| l.offered.listed(carries));
        lent.sort_by(|a, b| {
            (a.person.as_str(), &a.offered.name).cmp(&(b.person.as_str(), &b.offered.name))
        });
        // This computer's own copy of a carried app first, so the one copy
        // a page opens is the one whose acts this computer signs as its own
        // person (capability-signed).
        lent.sort_by_key(|l| l.offered.group.is_some() && l.key != self.host.actor);
        Ok((lent, others.unlisted))
    }
}

/// The accept loop's decision: the protocol whose routes a dialer is
/// served, or `None` to close it. Any dialer may redeem a link; the member
/// protocol serves a dialer standing in any group this node holds and closes
/// the rest (PLAN §4, U1's inventory, "the mesh" row — upstream asked the
/// mesh, `acceptor.rs:47-61`). Asked once per connection, so a member
/// admitted or removed since the last dial is answered as it stands now.
fn admit_dialer(rail: &RingRail, alpn: &[u8], dialer: &PublicKey) -> Option<Alpn> {
    let actor = key_actor(dialer);
    match Alpn::of(alpn) {
        Some(Alpn::Member) | Some(Alpn::Blobs) => {}
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
    if let Some((namespace, _)) = rail.standing_anywhere(&actor) {
        tracing::debug!(target: "node", actor, namespace, "node: admitted — the dialer stands in a group held here");
        return Alpn::of(alpn);
    }
    tracing::debug!(target: "node", actor, "node: refused — the dialer stands in no group held here");
    None
}

#[cfg(any(test, feature = "test-support"))]
mod tests_support;
#[cfg(any(test, feature = "test-support"))]
pub use tests_support::splice;

#[cfg(test)]
mod tests;
