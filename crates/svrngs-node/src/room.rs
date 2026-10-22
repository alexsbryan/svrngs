// SPDX-License-Identifier: AGPL-3.0-or-later
//! The room (u6-1; DESIGN §3, "The room", calls 2 and 3): the lends this
//! computer's person turns on for the room, each served to this computer's
//! own network at `http://<private address>:<port>/`, a port of its own, so
//! a device there, a TV's own app, reaches the lend's root as at its origin
//! on loopback. One body with those origins ([`crate::origin::reach`]):
//! reached as this computer's person, dialled under its key, so the
//! lender's door counts the room as one person and guesses its path as it
//! does now; a device in the room is this person's own, and no lend is
//! carried for anyone else.
//!
//! **The room is the running node's** ([`spawn`], its one owner), turned on
//! and off at [`ROUTE`] by the home page's button and by `svrngs room`
//! alike. Which lends are on is kept in [`ROOM_FILE`] by the lender's key
//! and the lend's name, so the room opens again as svrngs starts, before
//! any lender answers: a listener is bound by the lend's label alone
//! ([`port_of`]), and a request to a lend whose lender is away is answered
//! as [`crate::origin::reach`] answers one, within its dial's cap. A file
//! that cannot be read is said, never read as none: the room serves
//! nothing and says why until a turn writes it over (AGENTS.md 6).
//!
//! **Only on the network it was turned on at** ([`serves`]): each lend
//! keeps the network it was turned on at, told apart by its router's
//! hardware address ([`look`]), and is served only while this computer is
//! on that network, read again every [`LOOK_EVERY`]. On another network, or
//! one that cannot be read, it is not served, and the room says why.
//!
//! New, in the shape of upstream's LAN door and address picker, not their
//! code (PLAN §4, U6's inventory): "the door names the PORT, this machine
//! names the ADDRESS" (`sovereign-cli-mesh/src/mesh_guest_link.rs:10-12`
//! at `e1a43bcd2`) — here bound on the private address alone, so safe by
//! address and by network.

use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use n0_future::task::AbortOnDropHandle;
use serde::{Deserialize, Serialize};
use svrngs_core::pass::Label;
use tokio::sync::{mpsc, oneshot};

use crate::door::limit::{guess, Whereabouts};
use crate::door::LendRefusal;
use crate::lend::LendName;
use crate::node::Asking;
use crate::Person;

/// Where the room is read and turned on and off: the home page's button
/// and `svrngs room` (cli.rs) ask here.
pub const ROUTE: &str = "/v1/room";

/// The file in the data dir that keeps which lends are on for the room.
pub const ROOM_FILE: &str = "room.json";

/// How often the room reads which network this computer is on: the
/// longest a lend stays served after this computer moves to another
/// network that gives it the same address, which iroh's watch of its
/// addresses (`Endpoint::watch_addr`) never sees. A read is the interface
/// and routing tables (5-9 ms here, netdev's) and configd's state (5 ms a
/// `scutil`), and sends nothing; nothing is read while nothing is on.
const LOOK_EVERY: Duration = Duration::from_secs(5);

/// A network, by its router's hardware address: what tells home from a
/// café whose router gives this computer the same private address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network(String);

/// The network this computer is on, by its router's hardware address, and
/// its addresses there, for [`pick`]: read from its default route's
/// interface (netdev), the router's hardware address from where this
/// computer keeps it ([`routers`], [`hardware_in`]). No network when any of
/// it cannot be read, never a guess.
pub fn look() -> (Option<Network>, Vec<IpAddr>) {
    let iface = netdev::get_default_interface();
    let router = (iface.as_ref().ok()).and_then(|i| i.gateway.as_ref()?.ipv4.first().copied());
    let addrs = |i: &netdev::Interface| i.ipv4.iter().map(|n| IpAddr::V4(n.addr())).collect();
    let listed = iface.as_ref().map_or(Vec::new(), addrs);
    let network = router
        .and_then(|r| hardware_in(&routers()?, r))
        .map(Network);
    tracing::debug!(target: "room", error = ?iface.err(), ?router, ?network, ?listed, "room: the network read by its router's hardware address, none when any of it is unread");
    (network, listed)
}

/// Where this computer keeps its router's hardware address: on macOS, the
/// primary network service's state in configd (`scutil`), as macOS
/// resolved it for the network's own signature, since the ARP table is
/// hidden from a program not let find devices on the network (`arp -n`
/// answered "no entry" to the tests' process here, and its full answer to a
/// terminal); on Linux, the ARP table, `/proc/net/arp`.
#[cfg(target_os = "macos")]
fn routers() -> Option<String> {
    use std::process::{Command, Stdio};
    let show = |key: &str| {
        let mut scutil = Command::new("/usr/sbin/scutil");
        let mut scutil = (scutil.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()).ok()?;
        std::io::Write::write_all(
            &mut scutil.stdin.take()?,
            format!("show {key}\n").as_bytes(),
        )
        .ok()?;
        String::from_utf8(scutil.wait_with_output().ok()?.stdout).ok()
    };
    let global = show("State:/Network/Global/IPv4")?;
    let service = global
        .lines()
        .find_map(|l| l.trim().strip_prefix("PrimaryService : "))?;
    show(&format!("State:/Network/Service/{service}/IPv4"))
}
#[cfg(not(target_os = "macos"))]
fn routers() -> Option<String> {
    std::fs::read_to_string("/proc/net/arp").ok()
}

/// The hardware address of `router` in `text`, as [`routers`] reads it:
/// configd's `ARPResolvedHardwareAddress` line (macOS), or the router's row
/// of the ARP table (Linux); as six octets in two hex digits each, `None`
/// when absent or incomplete (all zeros).
pub fn hardware_in(text: &str, router: Ipv4Addr) -> Option<String> {
    let router = router.to_string();
    let word = text
        .lines()
        .find_map(|l| match l.split_whitespace().collect::<Vec<_>>()[..] {
            ["ARPResolvedHardwareAddress", ":", mac] => Some(mac),
            [ip, _, _, mac, ..] if ip == router => Some(mac),
            _ => None,
        })?;
    let octets = word.split(':').map(|o| u8::from_str_radix(o, 16).ok());
    let octets: Vec<u8> = octets.collect::<Option<_>>()?;
    let hex: Vec<String> = octets.iter().map(|o| format!("{o:02x}")).collect();
    (octets.len() == 6 && octets.iter().any(|o| *o != 0)).then(|| hex.join(":"))
}

/// The address a room's devices reach this computer at, from the
/// addresses its default route's interface lists ([`look`]) and the one
/// its outward route leaves from ([`route`]): that one when it is a
/// private IPv4 address listed; else the one private IPv4 address listed;
/// else `None`, absence, said so by the caller. Never loopback, a wildcard,
/// link-local, tailscale's 100.64/10 — a device on the room's WiFi cannot
/// reach it, so "the LAN before tailscale" upstream measured is "instead
/// of" here — or a public address. A container's bridge is private too,
/// and is told from the LAN only by the route, which does not leave by it.
pub fn pick(listed: &[IpAddr], route: Option<IpAddr>) -> Option<Ipv4Addr> {
    let v4 = |a: &IpAddr| match a.to_canonical() {
        IpAddr::V4(v4) if v4.is_private() => Some(v4),
        _ => None,
    };
    let private: Vec<Ipv4Addr> = listed.iter().filter_map(v4).collect();
    let routed = route.as_ref().and_then(v4).filter(|r| private.contains(r));
    let picked = routed.or(match private[..] {
        [one] => Some(one),
        _ => None,
    });
    tracing::debug!(target: "room", ?listed, ?route, ?private, ?picked, "room: the address picked — the route's if private and listed, else the one private, else none");
    picked
}

/// The address this computer's outward route leaves from: a UDP socket
/// connected to a public address, which sends nothing (upstream's
/// fallback, `commonwealth-discovery/src/mesh_discovery.rs:290-298`).
pub fn route() -> Option<IpAddr> {
    let probe = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    probe.connect("1.1.1.1:80").ok()?;
    probe.local_addr().ok().map(|a| a.ip())
}

/// The port the room serves `label` at: the same on every run, so a
/// device is typed the address once (PLAN §4, U6's inventory), and below
/// macOS's ephemeral range. One that is taken is said so, [`Served::Taken`].
pub fn port_of(label: &Label) -> u16 {
    let h = label
        .as_str()
        .bytes()
        .fold(0u32, |h, b| h.wrapping_mul(31) ^ u32::from(b));
    40_000 + (h % 9_000) as u16
}

/// Where a lend on for the room is now: a closed set (AGENTS 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Served {
    /// Served at this address, to this computer's network.
    At(SocketAddr),
    /// This computer is on another network than the one it was turned on at.
    Elsewhere,
    /// Which network this computer is on cannot be read.
    Unread,
    /// This computer has no private address on its network ([`pick`]).
    NoAddress,
    /// Another program on this computer holds its port.
    Taken,
}

impl Served {
    /// What the home page and `svrngs room` say of it, the one wording.
    pub fn said(&self) -> String {
        match self {
            Served::At(at) => format!("On for your TV: type http://{at}/ into its app. Anyone on this WiFi can open it there as you."),
            Served::Elsewhere => "Off for now: this computer is on another WiFi than the one where you turned it on. It comes back on when you are there again.".into(),
            Served::Unread => "Off for now: this computer cannot tell which WiFi it is on, so it shows it to nobody.".into(),
            Served::NoAddress => "Off for now: this computer is on no WiFi or home network. Join one, or plug it in.".into(),
            Served::Taken => "Off for now: another program on this computer holds the port it needs. Stop that program, and it comes back on.".into(),
        }
    }
}

/// Whether a lend turned on at `kept` is served at `port`, with this
/// computer on `now` at `at` ([`pick`]): only on the network it was turned
/// on at, never on one that cannot be read. Pure, so each answer is judged
/// with values (`room_tests.rs`).
pub fn serves(kept: &Network, now: Option<&Network>, at: Option<Ipv4Addr>, port: u16) -> Served {
    let served = match (now, at) {
        (None, _) => Served::Unread,
        (Some(now), _) if now != kept => Served::Elsewhere,
        (Some(_), None) => Served::NoAddress,
        (Some(_), Some(at)) => Served::At(SocketAddr::new(at.into(), port)),
    };
    tracing::debug!(target: "room", ?kept, ?now, ?at, ?served, "room: served only on the network it was turned on at, and only one read");
    served
}

/// A lend on for the room, as [`ROOM_FILE`] keeps it: the lender's key, the
/// lend's name, and the network it was turned on at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct On {
    key: String,
    name: LendName,
    network: Network,
}

/// A lend on for the room, as [`ROUTE`] answers it: whose it is, `None` for
/// a key no group held here names now; the lender's key; its name; where
/// it is served or why not; and that in a person's words ([`Served::said`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InRoom {
    pub person: Option<Person>,
    pub key: String,
    pub name: LendName,
    pub served: Served,
    pub said: String,
}

/// What the room is asked at [`ROUTE`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoomAsk {
    /// Every lend of these names, lent to this computer now or on already:
    /// `svrngs room <name>...`.
    Names(Vec<LendName>),
    /// The lend `name` of the computer whose key is `key`, turned on: the
    /// home page's button.
    On { key: String, name: LendName },
    /// That lend, turned off.
    Off { key: String, name: LendName },
}

/// A change to the room, sent to its owner: these on, none to read it
/// alone, or this one off.
pub(crate) enum Turn {
    On(Vec<On>),
    Off(String, LendName),
}

/// A turn, and where its owner answers the room as it stands after it, or
/// why what is on cannot be read.
type Asked = (Turn, oneshot::Sender<Result<Vec<InRoom>, String>>);

/// What the room's owner reads its asks from.
type Asks = mpsc::UnboundedReceiver<Asked>;

/// Each lend served, by its label: where, and its listener.
type Listening = BTreeMap<Label, (SocketAddr, AbortOnDropHandle<()>)>;

/// The room's owner's handle ([`spawn`]), held by the node's routers.
#[derive(Clone)]
pub(crate) struct Room(mpsc::UnboundedSender<Asked>);

impl Room {
    /// A room, and what its owner takes.
    pub(crate) fn new() -> (Self, Asks) {
        let (room, asked) = mpsc::unbounded_channel();
        (Room(room), asked)
    }

    /// `turn` made by the room's owner, and the room as it stands then, or
    /// why it cannot be read; `None` once it has stopped.
    async fn turn(&self, turn: Turn) -> Option<Result<Vec<InRoom>, String>> {
        let (done, made) = oneshot::channel();
        self.0.send((turn, done)).ok()?;
        made.await.ok()
    }
}

/// The room's one owner, for as long as the node runs: the lends on, read
/// from [`ROOM_FILE`] as it starts and written there at each turn, each
/// served as [`serve`] decides every [`LOOK_EVERY`] and at each ask.
pub(crate) fn spawn(at: Asking, mut asked: Asks) -> AbortOnDropHandle<()> {
    let file = at.door.data_dir.join(ROOM_FILE);
    let on = match std::fs::read(&file) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        read => (read.map_err(|e| e.to_string()))
            .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string())),
    };
    tracing::debug!(target: "room", file = %file.display(), error = ?on.as_ref().err(), "room: the lends on, as kept: none before the first, and one unread said, serving nothing");
    let (mut on, mut listening): (Result<Vec<On>, String>, Listening) = (on, BTreeMap::new());
    AbortOnDropHandle::new(tokio::spawn(async move {
        let mut every = tokio::time::interval(LOOK_EVERY);
        loop {
            let done = tokio::select! {
                _ = every.tick() => None,
                turned = asked.recv() => {
                    let Some((turn, done)) = turned else { return };
                    // A turn, not a read, writes the room over: one that
                    // could not be read too.
                    if !matches!(&turn, Turn::On(lends) if lends.is_empty()) {
                        let mut list = on.as_mut().map_or(Vec::new(), std::mem::take);
                        match turn {
                            Turn::On(lends) => lends.into_iter().for_each(|l| {
                                list.retain(|o| (&o.key, &o.name) != (&l.key, &l.name));
                                list.push(l);
                            }),
                            Turn::Off(key, name) => list.retain(|o| o.key != key || o.name != name),
                        }
                        let bytes = serde_json::to_vec(&list).expect("the room is JSON");
                        let kept = crate::declared::write_whole(&file, &bytes);
                        tracing::debug!(target: "room", on = list.len(), ?kept, "room: turned, and kept so it opens again as svrngs starts");
                        on = Ok(list);
                    }
                    Some(done)
                }
            };
            let room = serve(&at, on.as_deref().unwrap_or_default(), &mut listening).await;
            done.map(|d| d.send(on.as_ref().map(|_| room).map_err(String::clone)));
        }
    }))
}

/// Each lend `on` served or not, as [`serves`] decides at the network read
/// now: its listener kept where it is served still, bound where it is
/// served anew, and dropped, with `listening`'s rest, where it is not; and
/// the room as it stands.
async fn serve(at: &Asking, on: &[On], listening: &mut Listening) -> Vec<InRoom> {
    let (network, address) = match on.is_empty() {
        true => (None, None),
        false => {
            let (network, listed) = tokio::task::spawn_blocking(look).await.unwrap_or_default();
            (network, pick(&listed, route()))
        }
    };
    let (mut was, rail) = (std::mem::take(listening), &at.host.rail);
    let mut room = Vec::new();
    for o in on {
        let Some(label) = Label::of_hex(&o.key, o.name.as_str()) else {
            continue;
        };
        let served = match serves(&o.network, network.as_ref(), address, port_of(&label)) {
            Served::At(addr) => match was.remove(&label).filter(|(a, _)| *a == addr) {
                Some(kept) => {
                    listening.insert(label, kept);
                    Served::At(addr)
                }
                None => listen(at, label, addr, listening).await,
            },
            closed => closed,
        };
        let person = rail.standing_anywhere(&o.key).map(|(_, m)| m.person);
        let (key, name, said) = (o.key.clone(), o.name.clone(), served.said());
        room.push(InRoom {
            person,
            key,
            name,
            served,
            said,
        });
    }
    room
}

/// `label` served at `addr`, to callers on this network
/// ([`from_this_network`]): [`Served::At`], or [`Served::Taken`] when
/// another program holds the address.
async fn listen(at: &Asking, label: Label, addr: SocketAddr, listening: &mut Listening) -> Served {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .inspect_err(|e| tracing::debug!(target: "room", %addr, %label, error = %e, "room: not served — another program holds its address"));
    let Ok(listener) = listener else {
        return Served::Taken;
    };
    let routes = Router::new()
        .fallback(in_room)
        .with_state((at.clone(), label.clone()))
        .layer(axum::middleware::from_fn(from_this_network))
        .into_make_service_with_connect_info::<SocketAddr>();
    let serving = tokio::spawn(async move {
        let _ = axum::serve(listener, routes).await;
    });
    tracing::info!(target: "room", %addr, %label, "room: serving a lend to this computer's network");
    listening.insert(label, (addr, AbortOnDropHandle::new(serving)));
    Served::At(addr)
}

/// The routes, for [`crate::api::router`] and the home page to serve.
pub(crate) fn router(asking: Asking) -> Router {
    Router::new()
        .route(ROUTE, get(listed).post(turn))
        .with_state(asking)
}

/// `GET /v1/room`: each lend on for the room, where it is served or why not.
async fn listed(State(at): State<Asking>) -> Response {
    turned(&at, Turn::On(Vec::new())).await
}

/// `POST /v1/room`: `ask` turned on or off, and the room as it stands then
/// ([`listed`]). A lend is turned on at the network read now, so one that
/// cannot be read turns nothing on; a name lent to this computer by nobody
/// now, and not on already, turns nothing on either.
async fn turn(State(at): State<Asking>, Json(ask): Json<RoomAsk>) -> Response {
    let lends = match ask {
        RoomAsk::Off { key, name } => return turned(&at, Turn::Off(key, name)).await,
        RoomAsk::On { key, name } => (Label::of_hex(&key, name.as_str())
            .map(|_| vec![(key, name)]))
        .ok_or_else(|| "That is not something lent in your groups.".to_string()),
        RoomAsk::Names(names) => named(&at, &names).await,
    };
    let (network, _) = tokio::task::spawn_blocking(look).await.unwrap_or_default();
    tracing::debug!(target: "room", ?lends, ?network, "room: turned on at the network read now, or refused");
    let refused = |words: &str| LendRefusal::Closed.say(StatusCode::CONFLICT, words);
    match (lends, network) {
        (Err(words), _) => refused(&words),
        (_, None) => refused("This computer cannot tell which WiFi it is on, so nothing was turned on for your TV: try again once it is on one."),
        (Ok(lends), Some(network)) => {
            let on = |(key, name)| On { key, name, network: network.clone() };
            turned(&at, Turn::On(lends.into_iter().map(on).collect())).await
        }
    }
}

/// `turn` made by the room's owner, and the room as it stands then; why
/// what is on cannot be read, which serves nothing until a turn writes it
/// over; or refused when its owner has stopped.
async fn turned(at: &Asking, turn: Turn) -> Response {
    let room = at.room.turn(turn).await;
    tracing::debug!(target: "room", made = room.is_some(), unread = ?room.as_ref().and_then(|r| r.as_ref().err()), "room: asked of its owner: the room, why it cannot be read, or refused as it has stopped");
    match room {
        Some(Ok(room)) => Json(room).into_response(),
        Some(Err(why)) => LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, format!("This computer cannot read what you turned on for your TV ({why}), so it shows nothing on your TV now. Turn a lend on for your TV again, and it starts over from that one.")),
        None => LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, "the room cannot be changed on this computer now"),
    }
}

/// The lends `names` names, by the lender's key and the lend's name: each
/// lent to this computer now, its own and the other computers' of its
/// groups, each asked now, or on already, so one whose lender is away
/// stays on; or the words of a name none is.
async fn named(at: &Asking, names: &[LendName]) -> Result<Vec<(String, LendName)>, String> {
    let groups = at.host.standings().unwrap_or_default();
    let own = crate::door::offered(&at.door).await.unwrap_or_default();
    let own = own.into_iter().map(|o| (at.host.actor.clone(), o.name));
    let others = at.host.lends_in(&groups, None).await.lent;
    let others = others.into_iter().map(|l| (l.key, l.offered.name));
    let on = at
        .room
        .turn(Turn::On(Vec::new()))
        .await
        .and_then(Result::ok);
    let on = on.unwrap_or_default();
    let on = on.into_iter().map(|r| (r.key, r.name));
    let found: Vec<_> = own
        .chain(others)
        .chain(on)
        .filter(|(_, n)| names.contains(n))
        .collect();
    let none = names.iter().find(|n| !found.iter().any(|(_, f)| f == *n));
    tracing::debug!(target: "room", ?names, found = found.len(), ?none, "room: the names turned on, each lent now or on already");
    match none {
        Some(none) => Err(format!(
            "nobody in your groups lends {none} to this computer now"
        )),
        None => Ok(found),
    }
}

/// A request at the room door, carried to its lend as at its origin, for
/// the lend's own page: the room serves one lend at each port.
async fn in_room(
    State((at, label)): State<(Asking, Label)>,
    req: Request,
) -> Result<Response, Response> {
    crate::origin::reach(&at, label.clone(), label, req).await
}

/// Serve a request only from a caller on this computer's network: one the
/// door's own guess calls home ([`guess`]: private, link-local or
/// unique-local), so "this network" has one decider. Any other, or one
/// whose address is unknown, is refused 403 before any lend is reached.
pub(crate) async fn from_this_network(request: Request, next: Next) -> Response {
    let caller = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip());
    if guess(caller) == Whereabouts::Home {
        tracing::debug!(target: "room", ?caller, path = %request.uri().path(), "room: a caller on this network");
        return next.run(request).await;
    }
    tracing::debug!(target: "room", ?caller, path = %request.uri().path(), "room: refused — the caller is not on this computer's network");
    crate::door::LendRefusal::Closed.say(
        StatusCode::FORBIDDEN,
        "this is opened only on the network of the computer that serves it",
    )
}

#[cfg(test)]
#[path = "room_tests.rs"]
mod tests;
