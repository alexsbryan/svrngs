// SPDX-License-Identifier: AGPL-3.0-or-later
//! What only a test asks of a [`Node`], behind `test-support` as
//! svrngs-core's fixtures are: moved here from node.rs in lean-3, since no
//! path of the product reaches them.

use super::*;
use crate::door::bound::Unchecked;
use crate::iroh::Endpoint;
use crate::uid::{Conn, Owner};
use eudoxia_community::binding;

impl Node {
    /// This node's journals, signed with its key.
    pub fn rail(&self) -> &Arc<RingRail> {
        &self.host.rail
    }

    /// This computer's folders, where `new` and `join` place a group's.
    pub fn folders(&self) -> &crate::folder::Folders {
        &self.asking.door.folders
    }

    /// A group made as `svrngs new` asks the svrngs running here for one,
    /// at [`crate::group::NEW_ROUTE`] ([`Asking::made`]).
    pub async fn made(&self, asked: crate::group::NewAsk) -> Result<crate::group::NewMade, String> {
        self.asking.made(asked).await
    }

    /// A link made as `svrngs link` asks the svrngs running here for one,
    /// at [`crate::home::LINKS_ROUTE`] ([`Asking::linked`]).
    pub async fn linked(
        &self,
        asked: crate::home::Linking,
    ) -> Result<crate::home::LinkMade, crate::home::Unlinked> {
        self.asking.linked(asked).await
    }

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

    /// `path` of this node's API at its socket, `body` posted there as JSON
    /// or `path` got where there is none, asked as you as svrngs's command
    /// asks it ([`crate::api::asked`]): its status and its body. A node on
    /// a wire, whose API is asked in this process, is asked by [`as_you`].
    pub async fn ask_as_you(&self, path: &str, body: Option<serde_json::Value>) -> (u16, String) {
        let (at, token) = (format!("http://{}", self.api_addr()), self.token());
        let asked = crate::api::asked(&at, path, &token, body.map(|b| b.to_string()));
        let (status, said) = asked.await.expect("svrngs proves itself and answers");
        (status, String::from_utf8_lossy(&said).into_owned())
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

    /// The relays this computer's endpoint is given now, as [`crate::tiers`]
    /// holds them, the meeting's three tiers apart from it: what a test of
    /// the emergency reads, n0's absent while ours answers.
    pub fn relays_held(&self) -> Vec<String> {
        self.host
            .held
            .borrow()
            .relays
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    /// Why this computer meets `group` the backup way, as the home page is
    /// told ([`crate::node::Host::at_fallback`]): what a test of the
    /// emergency reads, `None` when it meets the group at its own.
    pub fn fallback_of(&self, group: &str) -> Option<crate::tiers::Away> {
        self.host.at_fallback(group)
    }

    /// The relays this computer dials `actor` at, each way it dials: a
    /// folder's fetch and a lend's origin ([`Host::contact`]), then the
    /// round and the lend asks ([`Host::standing_peers`]) (#127).
    pub fn relays_dialled(&self, actor: &str) -> [Vec<String>; 2] {
        let groups = self.host.rail.standings().unwrap_or_default();
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
        let through = read.and_then(|l| l.terms.audience.ids()?.first().cloned());
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
        crate::ring_sync::run_one_round(&self.asking.door.folders, &|| {}, &mut Default::default())
            .await
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
/// whose computer answers no dial ([`Wire::asleep`]), each key whose fetch
/// panics on the asker's side ([`Wire::panics`]), and each service a
/// computer's door reaches at a port ([`Wire::listen`]).
#[derive(Default)]
struct Met {
    admits: std::collections::BTreeMap<[u8; 32], crate::iroh::Admitting>,
    parted: std::collections::BTreeSet<([u8; 32], [u8; 32])>,
    asleep: std::collections::BTreeSet<[u8; 32]>,
    panics: std::collections::BTreeSet<[u8; 32]>,
    services: std::collections::BTreeMap<([u8; 32], u16), Listening>,
    /// Whose each computer's socket at a port is, where a test says
    /// ([`Wire::owns`]); else this account's.
    owners: std::collections::BTreeMap<([u8; 32], u16), crate::uid::Owner>,
    /// What each computer reads on a share's port, where a test says
    /// ([`Wire::reads`], [`Wire::answers`]); else [`approval`]'s socket.
    reads: std::collections::BTreeMap<([u8; 32], u16), crate::iroh::PortRead>,
    /// How long each computer's read of a port takes, where a test says
    /// ([`Wire::slow`]); else none.
    slow: std::collections::BTreeMap<([u8; 32], u16), std::time::Duration>,
    answers: std::collections::BTreeMap<([u8; 32], u16), crate::iroh::PortAnswer>,
    holders: std::collections::BTreeMap<[u8; 32], Holding>,
    relays: std::collections::BTreeMap<crate::iroh::RelayUrl, Relaying>,
    /// Each computer's relays held, as its endpoint was given them
    /// ([`Net::relays`]).
    held: std::collections::BTreeMap<[u8; 32], std::collections::BTreeSet<crate::iroh::RelayUrl>>,
    /// Each computer's count of relay connections its relay reported rate
    /// limited, as its share reads it ([`Net::ratelimited`]).
    ratelimited: std::collections::BTreeMap<[u8; 32], u64>,
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

    /// A fault in the asker, not in `key`'s computer: each [`Fetch`] of
    /// `key` panics in the task that asks it, as a bug in this computer's
    /// own ask would, before anything is sent. The fault no transport here
    /// has otherwise, so a fan-out's join is watched taking a task that
    /// never returned.
    pub fn panics(&self, key: PublicKey) {
        self.0.lock().unwrap().panics.insert(*key.as_bytes());
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

    /// The socket at `port` on `key`'s computer is `owner`'s from here on, as
    /// that computer reads it ([`Net::owner`]): a browser of another account
    /// opening svrngs. Unsaid, every socket is this account's, as this
    /// test's process is.
    pub fn owns(&self, key: PublicKey, port: u16, owner: crate::uid::Owner) {
        let mut met = self.0.lock().unwrap();
        met.owners.insert((*key.as_bytes(), port), owner);
    }

    /// `key`'s computer's transport on this wire, for a test that asks it
    /// alone, as the door's flows do.
    pub fn net(&self, key: PublicKey) -> OnWire {
        OnWire {
            key,
            wire: self.clone(),
        }
    }

    /// `key`'s computer reads `read` on the share's `port` from here on, as
    /// the door checks it ([`Net::read_port`]): another socket beside the
    /// approved one, a restart, an accept elsewhere, or a read that failed.
    /// Unsaid, [`approval`]'s socket, which took the connection.
    pub fn reads(&self, key: PublicKey, port: u16, read: crate::iroh::PortRead) {
        let mut met = self.0.lock().unwrap();
        met.reads.insert((*key.as_bytes(), port), read);
    }

    /// `key`'s computer's reads of the share's `port` take `long` from here
    /// on, as a Mac's slowest do: a check a test drops while it reads.
    pub fn slow(&self, key: PublicKey, port: u16, long: std::time::Duration) {
        let mut met = self.0.lock().unwrap();
        met.slow.insert((*key.as_bytes(), port), long);
    }

    /// `key`'s computer reads `read` on the share's `port` from here on, as
    /// an approval reads it ([`Net::answering`]). Unsaid, [`answering`]'s.
    pub fn answers(&self, key: PublicKey, port: u16, read: crate::iroh::PortAnswer) {
        let mut met = self.0.lock().unwrap();
        met.answers.insert((*key.as_bytes(), port), read);
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

    /// The relay reports `key`'s computer's connection rate limited: one
    /// more connection its share's count names ([`Net::ratelimited`];
    /// share-refusal).
    pub fn ratelimited(&self, key: PublicKey) {
        *self
            .0
            .lock()
            .unwrap()
            .ratelimited
            .entry(*key.as_bytes())
            .or_default() += 1;
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
            // Read and let go before the panic, so the wire's lock is not
            // poisoned for every other computer on it.
            let panics = self
                .wire
                .0
                .lock()
                .unwrap()
                .panics
                .contains(peer.key.as_bytes());
            if panics {
                panic!(
                    "a fault planted in the fetch of {} (Wire::panics)",
                    peer.key.fmt_short()
                );
            }
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
    /// [`Wire::listen`]'s service at `at`'s port on this computer, a pipe's
    /// end. Its two ends as a loopback connection's, this side's port none.
    fn service(&self, at: SocketAddr) -> crate::iroh::Later<'_, std::io::Result<Unchecked>> {
        let port = at.port();
        let (ours, theirs) = tokio::io::duplex(64 * 1024);
        let met = self.wire.0.lock().unwrap();
        let to = met.services.get(&(*self.key.as_bytes(), port));
        let taken = to.is_some_and(|to| to.send(theirs).is_ok());
        drop(met);
        let at = |port| SocketAddr::from(([127, 0, 0, 1], port));
        let conn = Conn {
            local: at(0),
            peer: at(port),
        };
        Box::pin(async move {
            match taken {
                true => Ok(Unchecked::new(Box::new(ours), conn)),
                false => Err(std::io::ErrorKind::ConnectionRefused.into()),
            }
        })
    }
    /// What [`Wire::owns`] says of the far end's port, else this account's.
    fn owner(&self, conn: Conn) -> crate::iroh::Later<'_, Owner> {
        let port = conn.peer.port();
        let met = self.wire.0.lock().unwrap();
        let said = met.owners.get(&(*self.key.as_bytes(), port)).cloned();
        let owner =
            said.unwrap_or_else(|| crate::uid::mine().map_or_else(Owner::Unjudged, Owner::Uid));
        Box::pin(async move { owner })
    }
    /// What [`Wire::reads`] says of the port, else a real socket there as
    /// this computer reads it ([`real`]), else [`approval`]'s socket alone
    /// on it, and for a connection the one that took it.
    fn read_port(
        &self,
        raw: crate::lend::Raw,
        port: u16,
        conn: Option<Conn>,
    ) -> crate::iroh::Later<'_, crate::iroh::PortRead> {
        let met = self.wire.0.lock().unwrap();
        let said = met.reads.get(&(*self.key.as_bytes(), port)).cloned();
        let took = match conn {
            Some(_) => binding::Took::Waiting,
            None => binding::Took::Datagram,
        };
        let read = said.unwrap_or_else(|| Ok(real(raw, port).unwrap_or((read(), took))));
        let slow = met.slow.get(&(*self.key.as_bytes(), port)).copied();
        Box::pin(async move {
            tokio::time::sleep(slow.unwrap_or_default()).await;
            read
        })
    }
    /// What [`Wire::answers`] says of the port, else [`answering`].
    fn answering(
        &self,
        _: crate::lend::Raw,
        port: u16,
        _: Option<u32>,
    ) -> crate::iroh::Later<'_, crate::iroh::PortAnswer> {
        let met = self.wire.0.lock().unwrap();
        let said = met.answers.get(&(*self.key.as_bytes(), port)).cloned();
        let read = said.unwrap_or_else(|| Ok(answering()));
        Box::pin(async move { read })
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
    /// Once a relay it holds is up on the wire ([`Self::homes`]), as an
    /// endpoint is online once it has a home: a computer holding none that
    /// answers waits, as one whose relays never answer does.
    fn online(&self) -> crate::iroh::Later<'_, ()> {
        use n0_future::StreamExt as _;
        let mut homes = self.homes();
        Box::pin(async move {
            while let Some(now) = homes.next().await {
                if !now.is_empty() {
                    return;
                }
            }
            std::future::pending().await
        })
    }
    /// As [`Wire::ratelimited`] says, none until a test says one.
    fn ratelimited(&self) -> u64 {
        *self
            .wire
            .0
            .lock()
            .unwrap()
            .ratelimited
            .get(self.key.as_bytes())
            .unwrap_or(&0)
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
                _tasks: Vec::new(),
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
                _tasks: Vec::new(),
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
        let meeting = crate::tiers::everywhere(
            &rail,
            &groups,
            &relay.build(),
            &crate::tiers::behind(),
            &asked,
        );
        let (held, relays) = crate::tiers::Held::built(meeting, asked);
        net.relays(relays.into_iter().map(Into::into).collect(), Vec::new())
            .await;
        let places = Some(data_dir.join("svrngs"));
        let seeds: SeedText = |_| Ok(None);
        // A test's network has no wire; the handle is what the acceptor's
        // WebRTC route answers through, and it binds nothing until an offer
        // comes (webrtc-node).
        let direct = data_dir.join(crate::webrtc::direct::DIRECT_FILE);
        let (webrtc, _wire) = crate::webrtc::WebRtc::new(net.id().as_bytes(), direct);
        Self::start_on(
            net,
            (data_dir, places),
            rail,
            (relay, held),
            seeds,
            (token, port, unbound),
            every,
            webrtc,
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
        let (held, _) = crate::tiers::Held::built(
            svrngs_core::relay::Meeting::of([], &[], &[]),
            Default::default(),
        );
        let relay = RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: true,
        };
        let places = Some(data_dir.join("svrngs"));
        let every = crate::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
        let direct = data_dir.join(crate::webrtc::direct::DIRECT_FILE);
        let (webrtc, _wire) = crate::webrtc::WebRtc::new(net.id().as_bytes(), direct);
        Self::start_on(
            net,
            (data_dir, places),
            rail,
            (&relay, held),
            seeds,
            (token, port, serve),
            every,
            webrtc,
        )
        .await
    }
}

/// The socket a test's share's port holds unless a test says otherwise
/// ([`Wire::reads`]): socket 1, held by process 1 started at 1, the
/// program `test` in `~/test`.
fn read() -> binding::Read {
    let socket = binding::Socket {
        id: 1,
        cgroup: None,
        holders: [(1, 1)].into(),
        at: std::net::Ipv4Addr::LOCALHOST.into(),
        dual: false,
        peer: binding::Peer::Unread,
    };
    binding::Read {
        boot: None,
        lasting: true,
        sockets: vec![socket],
        unlisted: false,
    }
}

/// [`read`] as an approval reads it, process 1 shown, and asked for, at 1.
pub fn answering() -> crate::uid::Answering {
    let shown = crate::lend::Answer::Program {
        pid: 1,
        start: 1,
        program: "test".into(),
        folder: Some("~/test".into()),
    };
    crate::uid::Answering {
        read: read(),
        parents: Default::default(),
        start: Some(1),
        shown: vec![(1, shown)],
        beside: false,
    }
}

/// What a test's share approves: [`read`]'s socket.
pub fn approval() -> binding::Approved {
    binding::approve(&read(), None).expect("a socket is approved")
}

/// [`approval`] as a lend's file keeps it, for a test that writes one: its
/// socket by its id, and, as it carries no cgroup, its holder beside it.
pub fn approved() -> serde_json::Value {
    serde_json::json!({ "sockets": [1], "processes": [[1, 1]], "at": "127.0.0.1" })
}

/// `lend`, a lend's file as a test writes it, with what answers on its port
/// approved when it names a port and no approval, as [`real`] reads it:
/// what a test's lender approved.
pub fn approving(lend: &serde_json::Value) -> String {
    let mut lend = lend.clone();
    let port = lend["port"].as_u64().and_then(|p| u16::try_from(p).ok());
    let raw = match lend["raw"].as_str() {
        Some("udp") => crate::lend::Raw::Udp,
        _ => crate::lend::Raw::Tcp,
    };
    if let Some(port) = port.filter(|_| lend.get("approved").is_none()) {
        let read = real(raw, port).map(|(read, _)| crate::uid::Answering {
            read,
            ..answering()
        });
        let read = read.unwrap_or_else(answering);
        let approved = binding::approve(&read.read, None).expect("a socket is approved");
        lend["approved"] = serde_json::to_value(approved).expect("an approval serializes");
    }
    lend.to_string()
}

/// `lend` approved as `svrngs share` approves it here: what this computer's
/// own reader reads on its port now ([`real`]), else [`read`]'s socket.
pub fn approve(lend: &mut crate::lend::Lend) {
    let port = lend.port().expect("a port's lend");
    let read = real(lend.raw(), port.get()).map_or_else(read, |(read, _)| read);
    lend.terms.approved = binding::approve(&read, None).ok();
}

/// What this computer's own reader reads on `port`, when a real socket a
/// test runs is on it: then the wire reads it too, so a test's stand-in on
/// a real listener is approved and checked as it is; none for a port only
/// the wire answers, which reads [`read`]'s socket.
fn real(raw: crate::lend::Raw, port: u16) -> Option<(binding::Read, binding::Took)> {
    let read = crate::uid::port(raw, port, None).ok()?;
    (!read.0.sockets.is_empty()).then_some(read)
}

/// The line a test writes where a program asks svrngs as its person ("Never
/// send the key", V1.13.3): a stand-in for the code its connection's proof
/// makes, which a test's transport puts in its place once it has proved
/// svrngs on the connection it sends on ([`as_you`] in this process,
/// [`as_you_on`] on a socket). It is never a code: svrngs refuses it as
/// one made for another connection.
pub const AS_YOU: &str = "Authorization: Svrngs as-you\r\n";

/// [`AS_YOU`]'s value, as a header holds it.
pub const AS_YOU_VALUE: &str = "Svrngs as-you";

/// `request` asked of `routes` in this process as a program on this
/// computer asks svrngs as its person (apps/client/svrngs.mjs, `svrngs
/// ask`): on a connection of its own from `caller` to `server`
/// ([`crate::api::Line`]), on which svrngs is first asked its proof
/// (`POST /v1/proof`), which is checked, then `request` with the code made
/// for the nonce it gave ([`crate::api::ask`]), never the token. The one
/// way a test asks svrngs as you in this process; a socket's is
/// [`as_you_on`].
pub async fn as_you(
    routes: &axum::Router,
    token: &str,
    mut request: axum::extract::Request,
    (caller, server): (SocketAddr, SocketAddr),
) -> axum::response::Response {
    use tower::ServiceExt as _;
    let line = crate::api::Line::new(Some(server));
    let challenge = "5a".repeat(32);
    let proving = axum::http::Request::post("/v1/proof")
        .header("host", server.to_string())
        .header("content-type", "application/json")
        .extension(axum::extract::ConnectInfo(caller))
        .extension(line.clone());
    let proving = proving.body(axum::body::Body::from(format!(
        r#"{{"challenge":"{challenge}"}}"#
    )));
    let proved = routes.clone().oneshot(proving.unwrap()).await;
    let Ok(proved) = proved;
    let said = axum::body::to_bytes(proved.into_body(), 4096)
        .await
        .unwrap();
    let (proof, code) = codes(token, &challenge, &said, (caller, server));
    assert_eq!(proof, "proved", "svrngs's proof on a connection of its own");
    let headers = request.headers_mut();
    headers.insert("authorization", code.parse().unwrap());
    request
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(caller));
    request.extensions_mut().insert(line);
    let Ok(answer) = routes.clone().oneshot(request).await;
    answer
}

/// The `Authorization` line for `stream`, a socket connected to svrngs's
/// API, once svrngs has proved on it that it holds `token`: `POST
/// /v1/proof` asked on it as raw HTTP/1, its answer read whole, the proof
/// checked against the socket's own two ends and the code made for the
/// nonce it gave. What a test's socket sends in [`AS_YOU`]'s place, its
/// next request on the same socket.
pub async fn as_you_on(stream: &mut tokio::net::TcpStream, token: &str) -> String {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    let ends = (stream.local_addr().unwrap(), stream.peer_addr().unwrap());
    let challenge = "5a".repeat(32);
    let body = format!(r#"{{"challenge":"{challenge}"}}"#);
    let asked = format!("POST /v1/proof HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", ends.1, body.len());
    stream.write_all(asked.as_bytes()).await.unwrap();
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        head.push(stream.read_u8().await.expect("svrngs answers its proof"));
    }
    let head = String::from_utf8(head).unwrap().to_ascii_lowercase();
    let length = (head.lines().find_map(|l| l.strip_prefix("content-length:")))
        .and_then(|n| n.trim().parse().ok())
        .expect("a proof's answer says its length");
    let mut said = vec![0; length];
    stream.read_exact(&mut said).await.unwrap();
    let (proof, code) = codes(token, &challenge, &said, ends);
    assert_eq!(proof, "proved", "svrngs's proof on this socket: {head}");
    format!("Authorization: {code}\r\n")
}

/// `request` asked of svrngs's API at `api` on a connection of its own, as
/// you, once svrngs has proved itself on it ([`as_you_on`]): its answer,
/// its body read whole, as a program asking a lend's address, or a page's
/// host, as you reads it, its own `Host` and headers sent as they are.
#[expect(
    clippy::disallowed_methods,
    reason = "a test's dial of svrngs's own API"
)]
pub async fn as_you_at(
    api: SocketAddr,
    token: &str,
    mut request: axum::http::Request<String>,
) -> axum::http::Response<axum::body::Bytes> {
    let mut stream = tokio::net::TcpStream::connect(api).await.unwrap();
    let line = as_you_on(&mut stream, token).await;
    let code = line.trim_end().trim_start_matches("Authorization: ");
    request
        .headers_mut()
        .insert("authorization", code.parse().unwrap());
    let mut on = crate::share::http_on(stream).await.unwrap();
    let (head, body) = on.send_request(request).await.unwrap().into_parts();
    let body = http_body_util::BodyExt::collect(body)
        .await
        .unwrap()
        .to_bytes();
    axum::http::Response::from_parts(head, body)
}

/// [`as_you_on`] on a blocking socket, for a test whose svrngs answers on
/// another thread or in another process.
pub fn as_you_on_std(stream: &mut std::net::TcpStream, token: &str) -> String {
    use std::io::{Read as _, Write as _};
    let ends = (stream.local_addr().unwrap(), stream.peer_addr().unwrap());
    let challenge = "5a".repeat(32);
    let body = format!(r#"{{"challenge":"{challenge}"}}"#);
    let asked = format!("POST /v1/proof HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", ends.1, body.len());
    stream.write_all(asked.as_bytes()).unwrap();
    let (mut head, mut byte) = (Vec::new(), [0u8]);
    while !head.ends_with(b"\r\n\r\n") {
        stream
            .read_exact(&mut byte)
            .expect("svrngs answers its proof");
        head.push(byte[0]);
    }
    let head = String::from_utf8(head).unwrap().to_ascii_lowercase();
    let length = (head.lines().find_map(|l| l.strip_prefix("content-length:")))
        .and_then(|n| n.trim().parse().ok())
        .expect("a proof's answer says its length");
    let mut said = vec![0; length];
    stream.read_exact(&mut said).unwrap();
    let (proof, code) = codes(token, &challenge, &said, ends);
    assert_eq!(proof, "proved", "svrngs's proof on this socket: {head}");
    format!("Authorization: {code}\r\n")
}

/// What `said`, svrngs's answer to `challenge` on the connection from
/// `client` to `server`, proves, and the `Authorization` value its nonce
/// makes for that connection.
fn codes(
    token: &str,
    challenge: &str,
    said: &[u8],
    (client, server): (SocketAddr, SocketAddr),
) -> (&'static str, String) {
    let said: serde_json::Value = serde_json::from_slice(said).unwrap_or_default();
    let challenge = crate::api::Challenge::try_from(challenge.to_string()).unwrap();
    let wanted = crate::api::proof(token, &challenge, client, server);
    let proof = if said["proof"] == wanted.as_str() {
        "proved"
    } else {
        "unproven"
    };
    let nonce = said["nonce"].as_str().unwrap_or_default().to_string();
    let nonce = crate::api::Challenge::try_from(nonce).expect("a nonce for this connection");
    (
        proof,
        format!("Svrngs {}", crate::api::ask(token, &nonce, client, server)),
    )
}
