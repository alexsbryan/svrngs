// SPDX-License-Identifier: AGPL-3.0-or-later
//! What only a test asks of a [`Node`], behind `test-support` as
//! svrngs-core's fixtures are: moved here from node.rs in lean-3, since no
//! path of the product reaches them.

use super::*;
use crate::iroh::Endpoint;

impl Node {
    /// Hold a deliberately historical or synthetic journal fixture still
    /// while exercising its live routes. Normal startup and writes wake sync.
    pub fn stop_sync(&self) {
        self._round.abort();
    }

    /// [`Node::start_every`] with the product's round. Only tests start a
    /// node with no relays of its own to ask for. Its groups' folders are
    /// placed in `data_dir`'s `svrngs`, never the person's own `~/svrngs`.
    pub async fn start(
        data_dir: PathBuf,
        key: SigningKey,
        relay: &RelayConfig,
        seeds: SeedText,
        api: SocketAddr,
    ) -> Result<Self, NodeError> {
        let every = crate::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
        let places = Some(data_dir.join("svrngs"));
        Self::start_every((data_dir, places), key, relay, &[], seeds, api, every).await
    }

    /// This node's folders, a group's placed under `places` instead: two
    /// computers sharing one home, as two data dirs on one computer do.
    pub fn placing_in(&self, places: PathBuf) -> crate::folder::Folders {
        let mut folders = self.asking.door.folders.clone();
        folders.places = Some(places);
        folders
    }

    /// The routes served at [`Node::api_addr`], behind their loopback
    /// check: what a test hands a request whose caller it names.
    pub fn api_routes(&self) -> axum::Router {
        self.routes.clone()
    }

    /// The token a program on this computer shows the record, read from
    /// the file beside the `api` file as a program reads it
    /// ([`crate::api::TOKEN_FILE`]).
    pub fn token(&self) -> String {
        let file = self.asking.door.data_dir.join(crate::api::TOKEN_FILE);
        std::fs::read_to_string(file).expect("a started node keeps its token")
    }

    /// A read capability for protocol tests, independent of browser enrollment.
    /// The origin still checks the app's declared reads and the person's choice.
    pub fn read_capability(
        &self,
        app: &svrngs_core::pass::Label,
        read: &svrngs_core::pass::Label,
    ) -> String {
        self.asking.sessions.cap(app, read)
    }

    /// The dial-by-key transport the round dials through; a caller seeds a
    /// member's addresses here when no address lookup can find it. A node
    /// started over a test's own network has none.
    pub fn transport(&self) -> &IrohTransport {
        let net: &dyn std::any::Any = &*self.host.transport;
        net.downcast_ref().expect("a node started over iroh")
    }

    /// The endpoint this node dials and is dialled on: its transport's.
    pub fn endpoint(&self) -> &Endpoint {
        &self.transport().endpoint
    }

    /// What this computer's endpoint holds now, as [`crate::tiers`] spells
    /// it: the tier and the relays a test of a move reads, which iroh keeps
    /// to itself once the map changes under a home (2db4a71f).
    pub fn tier_held(&self) -> String {
        format!("{:?}", *self.host.held.borrow())
    }

    /// The relays this computer dials `actor` at, each way it dials: a
    /// folder's fetch and a lend's origin ([`Host::contact`]), then the
    /// round and the lend asks ([`Host::standing_peers`]) (#127).
    pub fn relays_dialled(&self, actor: &str) -> [Vec<String>; 2] {
        let groups = self.host.standings().unwrap_or_default();
        let said = |c: crate::iroh::Contact| c.relays.iter().map(ToString::to_string).collect();
        let round = (self.host.standing_peers(&groups).into_iter())
            .find_map(|(a, _, contact)| (a == actor).then_some(contact));
        [
            self.host.contact(actor).map_or(Vec::new(), said),
            round.map_or(Vec::new(), said),
        ]
    }

    /// A one-time code the door's authorize path would mint, for `lend`'s
    /// client and the person, with its redirect, PKCE challenge and nonce:
    /// a test's way to the token route before the door's path exists. The
    /// challenge and method are read as the door reads them
    /// ([`crate::oidc::provider::S256Challenge::parse`]): for a pair the
    /// door refuses, such as a method other than S256, no code is minted,
    /// and what is answered is a value the token route never minted.
    #[allow(clippy::too_many_arguments)]
    pub fn oidc_code(
        &self,
        lend: &str,
        client: &str,
        sub: &str,
        name: &str,
        username: &str,
        group: svrngs_core::token::Group,
        redirect: &str,
        challenge: Option<&str>,
        method: Option<&str>,
        nonce: Option<&str>,
    ) -> String {
        let Ok(challenge) = crate::oidc::provider::S256Challenge::parse(challenge, method) else {
            return "no-code-the-door-refuses-this-pkce-method".into();
        };
        let who = svrngs_core::token::Subject {
            sub: sub.into(),
            name: name.into(),
            username: username.into(),
            group,
        };
        // Through the first group the lend is lent to, as the door lets in.
        let lend = crate::lend::LendName::parse(lend).expect("a test's lend name");
        let read = crate::lend::read(&self.asking.door.data_dir, &lend)
            .ok()
            .flatten();
        let through = read.and_then(|l| l.audience.ids()?.first().cloned());
        self.asking.door.oidc.mint(crate::oidc::Code {
            lend,
            client: client.into(),
            device: who.sub.clone(),
            who,
            group: through.unwrap_or_default(),
            redirect: redirect.into(),
            challenge,
            nonce: nonce.map(Into::into),
        })
    }

    /// Run one round now, beside the interval's, and say what it moved.
    pub async fn round(&self) -> crate::ring_sync::RoundOutcome {
        crate::ring_sync::run_one_round(self.host.as_ref(), &|| {}, &mut Default::default()).await
    }
}

/// A loopback port whose every connection is spliced to a new bi-stream to
/// `to` on `alpn`, dialled from `endpoint` — so as its key: how a test
/// drives raw HTTP, or a socket, at a door. The product has no such port
/// (arch-peer (d): through one, any program on the computer reached a
/// friend's door as its person, `tests/peer.rs`); a test's lives as long as
/// its runtime.
pub async fn splice(endpoint: Endpoint, to: EndpointAddr, alpn: &'static [u8]) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a loopback port");
    let at = listener.local_addr().expect("a bound port");
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            let (endpoint, to) = (endpoint.clone(), to.clone());
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                let Ok(conn) = endpoint.connect(to, alpn).await else {
                    return;
                };
                let Ok((mut send, mut recv)) = conn.open_bi().await else {
                    return;
                };
                let (mut from, mut to) = tcp.into_split();
                // Each way on its own, as the bridge's pump copied: a door
                // that answers 413 and stops reading still has its answer read.
                let up = async {
                    let _ = tokio::io::copy(&mut from, &mut send).await;
                    let _ = send.shutdown().await;
                };
                let down = async {
                    let _ = tokio::io::copy(&mut recv, &mut to).await;
                    let _ = to.shutdown().await;
                };
                tokio::join!(up, down);
            });
        }
    });
    at
}

impl IrohTransport {
    /// A [`splice`] to the member `peer` names on the members' protocol,
    /// from this transport's endpoint, as `http://<port>`; `None` with no
    /// path to it. What a test that drives raw HTTP at a member asks, where
    /// it asked the bridge's `base_url` before arch-peer (d).
    pub async fn spliced(&self, peer: &Contact) -> Option<String> {
        let to = self.endpoint_addr_for(peer)?;
        Some(format!(
            "http://{}",
            splice(self.endpoint.clone(), to, crate::iroh::ALPN).await
        ))
    }
}

/// The computers of one test met in this process, no endpoint bound, no
/// socket (docs/CHECKS.md, harness 3): each key's acceptor decision, as
/// [`Net::accept`] registers it. A request reaches the router its peer's
/// decision names for the asker's key, by tower `oneshot`, the asker's
/// `Dialer` inserted as the acceptor inserts it; a Blobs get is served by
/// the peer's own provider over an in-memory pipe. It decides nothing:
/// every admission and route is the node's own.
#[derive(Clone, Default)]
pub struct Wire(
    #[expect(
        clippy::disallowed_types,
        reason = "a test's in-process network, shared by its nodes"
    )]
    Arc<std::sync::Mutex<Met>>,
);

/// Who is on a [`Wire`]: each key's acceptor decision, each pair of keys
/// the first has no path from to the second ([`Wire::part`]), each key
/// whose computer answers no dial ([`Wire::asleep`]), and each service a
/// computer's door reaches at a port ([`Wire::listen`]).
#[derive(Default)]
struct Met {
    admits: std::collections::BTreeMap<[u8; 32], crate::iroh::Admitting>,
    parted: std::collections::BTreeSet<([u8; 32], [u8; 32])>,
    asleep: std::collections::BTreeSet<[u8; 32]>,
    services: std::collections::BTreeMap<([u8; 32], u16), Listening>,
    holders: std::collections::BTreeMap<[u8; 32], Holding>,
    relays: std::collections::BTreeMap<crate::iroh::RelayUrl, Relaying>,
    /// Each computer's relays held, as its endpoint was given them
    /// ([`Net::relays`]).
    held: std::collections::BTreeMap<[u8; 32], std::collections::BTreeSet<crate::iroh::RelayUrl>>,
    /// Told of every change to a relay or to what a computer holds, so a
    /// computer's homes are read again.
    changed: Changed,
}

/// [`Met`]'s change, which a computer's homes are read again on.
struct Changed(tokio::sync::watch::Sender<u64>);

impl Default for Changed {
    fn default() -> Self {
        Self(tokio::sync::watch::Sender::new(0))
    }
}

/// What a relay on a [`Wire`] does now ([`Wire::relay`]), as a relay
/// process, a way to one, or a server that is none would.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relaying {
    /// Answers svrngs's ask and registers every endpoint that holds it.
    Up,
    /// Answers the ask and registers nobody, as a captive portal does.
    Pings,
    /// The way to it cut: the ask waits out [`crate::tiers::ANSWERS_WITHIN`]
    /// unanswered, and nobody is registered.
    Cut,
    /// Nothing there: the ask is refused at once, nobody registered.
    Down,
}

/// Where [`Wire::listen`]'s connections arrive, one pipe's end each.
type Listening = tokio::sync::mpsc::UnboundedSender<tokio::io::DuplexStream>;

/// Where [`Wire::holder`]'s gets arrive: the ask to read, the answer to
/// write, each one end of a pipe.
type Holding =
    tokio::sync::mpsc::UnboundedSender<(tokio::io::DuplexStream, tokio::io::DuplexStream)>;

impl Wire {
    /// No path from `from`'s computer to `to`'s, as when `from` knows no
    /// address of `to` and no relay or lookup finds one: each ask is
    /// [`Unsent::NoPath`]. `to` still reaches `from`.
    pub fn part(&self, from: PublicKey, to: PublicKey) {
        let mut met = self.0.lock().unwrap();
        met.parted.insert((*from.as_bytes(), *to.as_bytes()));
    }

    /// `key`'s computer answers no dial, as an address nothing answers at:
    /// each ask waits the dial's cap ([`crate::dial::DIAL_WITHIN`]), then is
    /// [`Unsent::Asleep`], as [`IrohTransport`]'s dial is.
    pub fn asleep(&self, key: PublicKey) {
        self.0.lock().unwrap().asleep.insert(*key.as_bytes());
    }

    /// `url` does as `now` says from here on, each computer's homes read
    /// again: a relay a test starts, cuts the way to, stops or brings back.
    pub fn relay(&self, url: &crate::iroh::RelayUrl, now: Relaying) {
        let mut met = self.0.lock().unwrap();
        met.relays.insert(url.clone(), now);
        met.changed.0.send_modify(|n| *n += 1);
    }

    /// The relays `key`'s computer holds that register it now, in order:
    /// its homes, the first the one its address names.
    fn homes_of(&self, key: &PublicKey) -> Vec<crate::iroh::RelayUrl> {
        let met = self.0.lock().unwrap();
        let held = met.held.get(key.as_bytes()).into_iter().flatten();
        let up = |r: &&crate::iroh::RelayUrl| met.relays.get(*r) == Some(&Relaying::Up);
        held.filter(up).cloned().collect()
    }

    /// A service on `key`'s computer at `port`, answered in this process
    /// (docs/CHECKS.md, harness 2): each connection its door opens there
    /// ([`Net::service`]) is one end of a pipe, the other end received
    /// here, as a listener accepts one. Nothing listening, or the receiver
    /// dropped, is a refused connection, as a closed port is.
    pub fn listen(
        &self,
        key: PublicKey,
        port: u16,
    ) -> tokio::sync::mpsc::UnboundedReceiver<tokio::io::DuplexStream> {
        let (to, at) = tokio::sync::mpsc::unbounded_channel();
        let mut met = self.0.lock().unwrap();
        met.services.insert((*key.as_bytes(), port), to);
        at
    }

    /// A Blobs holder at `key` that is a test's script, past any store and
    /// any node: each get asked of it arrives here as its ask, to be read
    /// to its end, and its answer, to be written, the raw bytes a QUIC
    /// stream carries, so a test answers silent, slow or with a byte
    /// flipped. The asker's fetch reads the answer as it reads any
    /// holder's, verified by its own store; the script decides nothing it
    /// does. Its receiver dropped, a get of it fails as an unanswered dial.
    pub fn holder(
        &self,
        key: PublicKey,
    ) -> tokio::sync::mpsc::UnboundedReceiver<(tokio::io::DuplexStream, tokio::io::DuplexStream)>
    {
        let (to, at) = tokio::sync::mpsc::unbounded_channel();
        self.0.lock().unwrap().holders.insert(*key.as_bytes(), to);
        at
    }
}

/// One computer's place on a [`Wire`]: its key.
pub struct OnWire {
    key: PublicKey,
    wire: Wire,
}

impl OnWire {
    /// Now, then each change on its wire ([`Changed`]).
    fn changes(&self) -> n0_future::boxed::BoxStream<()> {
        let mut changed = self.wire.0.lock().unwrap().changed.0.subscribe();
        changed.mark_changed();
        Box::pin(n0_future::stream::unfold(changed, |mut c| async move {
            c.changed().await.ok()?;
            Some(((), c))
        }))
    }

    /// What `peer`'s acceptor decides for this computer on `alpn`, or why
    /// nothing answers: no computer on the wire, or none on a path from
    /// here, one asleep, or one that refuses.
    async fn admitted(&self, peer: &PublicKey, alpn: Alpn) -> Result<Admit, Unsent> {
        let asleep = self.wire.0.lock().unwrap().asleep.contains(peer.as_bytes());
        if asleep {
            tokio::time::sleep(crate::dial::DIAL_WITHIN).await;
            return Err(Unsent::Asleep(crate::dial::DIAL_WITHIN));
        }
        let met = self.wire.0.lock().unwrap();
        let parted = met
            .parted
            .contains(&(*self.key.as_bytes(), *peer.as_bytes()));
        let admit = (!parted).then(|| met.admits.get(peer.as_bytes()).cloned());
        drop(met);
        match admit.flatten().ok_or(Unsent::NoPath)?(alpn.bytes(), &self.key) {
            Admit::Refuse => Err(Unsent::NotAdmitted),
            admitted => Ok(admitted),
        }
    }

    /// `req` answered by the router `peer`'s acceptor serves this computer
    /// on `alpn`, its `Dialer` inserted as the acceptor inserts it.
    async fn ask(
        &self,
        peer: &PublicKey,
        alpn: Alpn,
        mut req: crate::iroh::Request,
    ) -> Result<crate::iroh::Streamed, Unsent> {
        use tower::ServiceExt;
        let Admit::Serve(router) = self.admitted(peer, alpn).await? else {
            return Err(Unsent::Failed(format!("not served on {alpn:?}")));
        };
        req.extensions_mut().insert(Dialer(self.key));
        Ok(router
            .oneshot(req)
            .await
            .unwrap_or_else(|never| match never {}))
    }
}

impl Wire {
    /// `req` asked of `peer` on `alpn` by a key on no computer of the wire,
    /// as a test dials as one (tests/common bridge_as): a phone's holder,
    /// or a member's key apart from its node. `peer` decides it as it
    /// decides any dialer's.
    pub async fn ask_as(
        &self,
        key: &SigningKey,
        peer: &PublicKey,
        alpn: Alpn,
        req: crate::iroh::Request,
    ) -> Result<crate::iroh::Streamed, Unsent> {
        let asker = OnWire {
            key: SecretKey::from_bytes(&key.to_bytes()).public(),
            wire: self.clone(),
        };
        asker.ask(peer, alpn, req).await
    }
}

impl Fetch for OnWire {
    fn fetch<'a>(
        &'a self,
        peer: &'a Contact,
        req: crate::iroh::WholeRequest,
    ) -> crate::iroh::Fetched<'a> {
        Box::pin(async move {
            let (head, body) = self.send(peer, req.map(Into::into)).await?.into_parts();
            let body = axum::body::to_bytes(body, usize::MAX).await;
            Ok(crate::iroh::Whole::from_parts(
                head,
                body.map_err(|e| Unsent::Failed(e.to_string()))?,
            ))
        })
    }
}

/// One end of a pipe, as iroh-blobs reads or writes a QUIC stream's half.
struct Half<T>(T);

impl iroh_blobs::util::AsyncReadRecvStreamExtra for Half<tokio::io::DuplexStream> {
    fn inner(&mut self) -> &mut (impl tokio::io::AsyncRead + Unpin + Send) {
        &mut self.0
    }
    fn stop(&mut self, _: iroh::endpoint::VarInt) -> std::io::Result<()> {
        Ok(())
    }
    fn id(&self) -> u64 {
        0
    }
}

impl iroh_blobs::util::AsyncWriteSendStreamExtra for Half<tokio::io::DuplexStream> {
    fn inner(&mut self) -> &mut (impl tokio::io::AsyncWrite + Unpin + Send) {
        &mut self.0
    }
    fn reset(&mut self, _: iroh::endpoint::VarInt) -> std::io::Result<()> {
        Ok(())
    }
    async fn stopped(&mut self) -> std::io::Result<Option<iroh::endpoint::VarInt>> {
        std::future::pending().await
    }
    fn id(&self) -> u64 {
        0
    }
}

impl Net for OnWire {
    /// [`Wire::listen`]'s service at `port` on this computer, a pipe's end.
    fn service(
        &self,
        port: u16,
    ) -> crate::iroh::Later<'_, std::io::Result<Box<dyn crate::iroh::Duplex>>> {
        let (ours, theirs) = tokio::io::duplex(64 * 1024);
        let met = self.wire.0.lock().unwrap();
        let to = met.services.get(&(*self.key.as_bytes(), port));
        let taken = to.is_some_and(|to| to.send(theirs).is_ok());
        drop(met);
        Box::pin(async move {
            match taken {
                true => Ok(Box::new(ours) as Box<dyn crate::iroh::Duplex>),
                false => Err(std::io::ErrorKind::ConnectionRefused.into()),
            }
        })
    }
    fn id(&self) -> PublicKey {
        self.key
    }
    /// Its key, at its first home on the wire when it has one.
    fn addr(&self) -> EndpointAddr {
        let home = self.wire.homes_of(&self.key).into_iter().next();
        let addr = EndpointAddr::new(self.key);
        home.map_or(addr.clone(), |h| addr.with_relay_url(h))
    }
    fn addrs(&self) -> n0_future::boxed::BoxStream<EndpointAddr> {
        use n0_future::StreamExt as _;
        let (wire, key) = (self.wire.clone(), self.key);
        let at = move |_| {
            OnWire {
                key,
                wire: wire.clone(),
            }
            .addr()
        };
        Box::pin(self.changes().map(at))
    }
    /// The relays it holds that are up on the wire, at each change.
    fn homes(&self) -> n0_future::boxed::BoxStream<Vec<crate::iroh::RelayUrl>> {
        use n0_future::StreamExt as _;
        let (wire, key) = (self.wire.clone(), self.key);
        Box::pin(self.changes().map(move |_| wire.homes_of(&key)))
    }
    /// As [`Wire::relay`] says `relay` does: an unknown one is down.
    fn answers<'a>(&'a self, relay: &'a svrngs_core::relay::Url) -> crate::iroh::Later<'a, bool> {
        let url = crate::iroh::RelayUrl::from(relay.clone());
        let now = self.wire.0.lock().unwrap().relays.get(&url).copied();
        Box::pin(async move {
            match now.unwrap_or(Relaying::Down) {
                Relaying::Up | Relaying::Pings => true,
                Relaying::Cut => {
                    tokio::time::sleep(crate::tiers::ANSWERS_WITHIN).await;
                    false
                }
                Relaying::Down => false,
            }
        })
    }
    fn online(&self) -> crate::iroh::Later<'_, ()> {
        Box::pin(async {})
    }
    /// Held on the wire, its homes read again.
    fn relays(
        &self,
        add: Vec<crate::iroh::RelayUrl>,
        drop: Vec<crate::iroh::RelayUrl>,
    ) -> crate::iroh::Later<'_, ()> {
        let mut met = self.wire.0.lock().unwrap();
        let held = met.held.entry(*self.key.as_bytes()).or_default();
        held.extend(add);
        held.retain(|r| !drop.contains(r));
        met.changed.0.send_modify(|n| *n += 1);
        Box::pin(async {})
    }
    fn send<'a>(
        &'a self,
        peer: &'a Contact,
        req: crate::iroh::Request,
    ) -> crate::iroh::Sent<'a, crate::iroh::Streamed> {
        Box::pin(self.ask(&peer.key, Alpn::Member, req))
    }
    fn dial(
        &self,
        _: EndpointAddr,
        alpn: Alpn,
        _: Option<Duration>,
    ) -> crate::iroh::Sent<'_, iroh::endpoint::Connection> {
        Box::pin(async move {
            Err(Unsent::Failed(format!(
                "no QUIC connection on a wire, {alpn:?}"
            )))
        })
    }
    /// Asked of `to`'s router for this computer on `alpn`, as a fresh
    /// connection's one stream is: a link's issuer, a join's redeem.
    fn ask_once(
        &self,
        to: EndpointAddr,
        alpn: Alpn,
        req: crate::iroh::Request,
    ) -> crate::iroh::Sent<'_, crate::iroh::Streamed> {
        Box::pin(async move { self.ask(&to.id, alpn, req).await })
    }
    fn get<'a>(
        &'a self,
        peer: &'a Contact,
        store: &'a iroh_blobs::api::Store,
        request: iroh_blobs::protocol::GetRequest,
    ) -> crate::iroh::Sent<'a, iroh_blobs::api::remote::GetProgress> {
        Box::pin(async move {
            use iroh_blobs::util::{AsyncReadRecvStream, AsyncWriteSendStream};
            let scripted = (self.wire.0.lock().unwrap().holders)
                .get(peer.key.as_bytes())
                .cloned();
            if let Some(holding) = scripted {
                // A test's holder ([`Wire::holder`]): the same two pipes,
                // their far ends handed to its script.
                let (asks, their_read) = tokio::io::duplex(64 * 1024);
                let (their_write, answers) = tokio::io::duplex(64 * 1024);
                let gone = |_| Unsent::Failed("the test's holder is gone".into());
                holding.send((their_read, their_write)).map_err(gone)?;
                let asked = iroh_blobs::get::StreamPair::new(
                    0,
                    AsyncReadRecvStream::new(Half(answers)),
                    AsyncWriteSendStream::new(Half(asks)),
                );
                return Ok(store.remote().execute_get(asked, request));
            }
            let Admit::Blobs(theirs, events) = self.admitted(&peer.key, Alpn::Blobs).await? else {
                return Err(Unsent::Failed("not served on the Blobs protocol".into()));
            };
            // Told to the holder's events as a QUIC connection is, by the
            // asker's key, which decide whether it is served.
            static CONNECTIONS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
            let id = CONNECTIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let connected = || iroh_blobs::provider::events::ClientConnected {
                connection_id: id,
                endpoint_id: Some(self.key),
            };
            if events.client_connected(connected).await.is_err() {
                return Err(Unsent::NotAdmitted);
            }
            // One pipe each way, as a QUIC stream's halves: the provider
            // reads the ask to its end, which the asker's dropped writer
            // gives, and the asker reads the answer.
            let (asks, their_read) = tokio::io::duplex(64 * 1024);
            let (their_write, answers) = tokio::io::duplex(64 * 1024);
            let served = iroh_blobs::provider::StreamPair::new(
                id,
                AsyncReadRecvStream::new(Half(their_read)),
                AsyncWriteSendStream::new(Half(their_write)),
                events,
            );
            tokio::spawn(iroh_blobs::provider::handle_stream(served, theirs));
            let asked = iroh_blobs::get::StreamPair::new(
                id,
                AsyncReadRecvStream::new(Half(answers)),
                AsyncWriteSendStream::new(Half(asks)),
            );
            Ok(store.remote().execute_get(asked, request))
        })
    }
    fn let_go(&self, _: &PublicKey, _: Alpn) {}
    fn add_known_peer(&self, _: PublicKey, _: Vec<std::net::SocketAddr>) {}
    fn endpoint_addr_for(&self, peer: &Contact) -> Option<EndpointAddr> {
        Some(EndpointAddr::new(peer.key))
    }
    fn accept(&self, admit: crate::iroh::Admitting) -> crate::iroh::IrohAcceptor {
        let mut met = self.wire.0.lock().unwrap();
        met.admits.insert(*self.key.as_bytes(), admit);
        n0_future::task::AbortOnDropHandle::new(tokio::spawn(async {}))
    }
    fn close(&self) -> crate::iroh::Later<'_, ()> {
        self.wire
            .0
            .lock()
            .unwrap()
            .admits
            .remove(self.key.as_bytes());
        Box::pin(async {})
    }
}

impl Node {
    /// A node over `data_dir` on `wire`, signing as `key`, its seeds read by
    /// `seeds`: no endpoint, no relay and no API listener; its record API
    /// asked by [`Node::api_routes`]. Its groups' folders are placed in
    /// `data_dir`'s `svrngs`, as [`Node::start`] places them.
    pub async fn on_wire(
        wire: &Wire,
        data_dir: PathBuf,
        key: SigningKey,
        seeds: SeedText,
    ) -> Result<Self, NodeError> {
        // A port no socket holds, one per key, so a test tells its nodes'
        // APIs apart by address as their programs would: two bytes of it,
        // since one gave keys 1 and 6 one port (common/on_wire.rs `served`
        // refuses two keys at one).
        let public = SecretKey::from_bytes(&key.to_bytes()).public();
        let [hi, lo, ..] = *public.as_bytes();
        let port = 1024 + u16::from_be_bytes([hi, lo]) % 64_000;
        let unbound: Serve = Box::new(move |_| {
            Ok(crate::api::Serving {
                addr: SocketAddr::from(([127, 0, 0, 1], port)),
                _task: n0_future::task::AbortOnDropHandle::new(tokio::spawn(async {})),
            })
        });
        Self::on_wire_serving(wire, (data_dir, key, seeds), (port, unbound)).await
    }

    /// [`Node::on_wire`] with its record API served on a listener at `api`,
    /// which must be loopback, and named in `data_dir`'s `api` file, as
    /// [`Node::start`] serves it: for a program outside this process, such
    /// as an app's canary under Node, which asks the record over HTTP.
    pub async fn on_wire_listening(
        wire: &Wire,
        data_dir: PathBuf,
        key: SigningKey,
        seeds: SeedText,
        api: SocketAddr,
    ) -> Result<Self, NodeError> {
        let bound = crate::api::bind(api).await?;
        let (port, dir) = (bound.addr.port(), data_dir.clone());
        let serve: Serve = Box::new(move |routes| {
            let api = crate::api::serve(routes, bound);
            crate::api::write_address(&dir, api.addr).map(|()| api)
        });
        Self::on_wire_serving(wire, (data_dir, key, seeds), (port, serve)).await
    }

    /// [`Node::on_wire`] met at `relay`'s build relays and the relays
    /// `asked`, its round every `every`: its tiers decided as a started
    /// node's are ([`crate::tiers`]), each relay's answer and registration
    /// the wire's ([`Wire::relay`]).
    pub async fn on_wire_relayed(
        wire: &Wire,
        (data_dir, key): (PathBuf, SigningKey),
        relay: &RelayConfig,
        asked: &[Url],
        every: Duration,
    ) -> Result<Self, NodeError> {
        let public = SecretKey::from_bytes(&key.to_bytes()).public();
        let [hi, lo, ..] = *public.as_bytes();
        let port = 1024 + u16::from_be_bytes([hi, lo]) % 64_000;
        let unbound: Serve = Box::new(move |_| {
            Ok(crate::api::Serving {
                addr: SocketAddr::from(([127, 0, 0, 1], port)),
                _task: n0_future::task::AbortOnDropHandle::new(tokio::spawn(async {})),
            })
        });
        let net = Arc::new(OnWire {
            key: public,
            wire: wire.clone(),
        });
        let rail = Arc::new(RingRail::new(&data_dir, Arc::new(key)));
        let token = crate::api::kept_token(&data_dir)?;
        let asked: std::collections::BTreeSet<Url> = asked.iter().cloned().collect();
        let groups = rail.namespaces().unwrap_or_default();
        let meeting = crate::tiers::everywhere(&rail, &groups, &relay.build(), &asked);
        let (held, relays) = crate::tiers::Held::built(meeting, asked);
        net.relays(relays.into_iter().map(Into::into).collect(), Vec::new())
            .await;
        let places = Some(data_dir.join("svrngs"));
        let seeds: SeedText = |_| Ok(None);
        Self::start_on(
            net,
            (data_dir, places),
            rail,
            (relay, held),
            seeds,
            (token, port, unbound),
            every,
        )
        .await
    }

    /// A node on `wire` whose record API, at `port`, `serve` serves.
    async fn on_wire_serving(
        wire: &Wire,
        (data_dir, key, seeds): (PathBuf, SigningKey, SeedText),
        (port, serve): (u16, Serve),
    ) -> Result<Self, NodeError> {
        crate::declared::secret_dir(&data_dir).map_err(|source| NodeError::DataDir {
            path: data_dir.clone(),
            source,
        })?;
        let net = Arc::new(OnWire {
            key: SecretKey::from_bytes(&key.to_bytes()).public(),
            wire: wire.clone(),
        });
        let rail = Arc::new(RingRail::new(&data_dir, Arc::new(key)));
        let token = crate::api::kept_token(&data_dir)?;
        let (held, _) =
            crate::tiers::Held::built(svrngs_core::relay::Meeting::of([], &[]), Default::default());
        let relay = RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: true,
        };
        let places = Some(data_dir.join("svrngs"));
        let every = crate::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
        Self::start_on(
            net,
            (data_dir, places),
            rail,
            (&relay, held),
            seeds,
            (token, port, serve),
            every,
        )
        .await
    }
}
