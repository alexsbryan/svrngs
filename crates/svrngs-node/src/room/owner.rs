// SPDX-License-Identifier: AGPL-3.0-or-later
//! The room's owner: one task for as long as the node runs ([`spawn`]),
//! told by messages ([`Room`]), owning the lends on and each one's
//! listener on this computer's network, bound by the lend's label alone
//! and dropped as [`super::serves`] decides; and the listener's two
//! layers, the caller on this network ([`from_this_network`]) and the
//! lend's body ([`in_room`], [`crate::origin::reach`]).

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::Router;
use n0_future::task::AbortOnDropHandle;
use svrngs_core::pass::Label;
use tokio::sync::{mpsc, oneshot};

use super::{kept, port_of, serves, InRoom, On, Served, Turn, ROOM_FILE};
use crate::network::{guess, Unread, Whereabouts};
use crate::node::Asking;

/// How often the room reads which network this computer is on: the
/// longest a lend stays served after this computer moves to another
/// network that gives it the same address, which iroh's watch of its
/// addresses (`Endpoint::watch_addr`) never sees. A read is the interface
/// and routing tables (5-9 ms here, netdev's) and configd's state (5 ms a
/// `scutil`), and sends nothing; nothing is read while nothing is on.
pub(super) const LOOK_EVERY: Duration = Duration::from_secs(5);

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
    pub(super) async fn turn(&self, turn: Turn) -> Option<Result<Vec<InRoom>, String>> {
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
    let (mut on, mut listening) = (kept::read(&file), Listening::new());
    AbortOnDropHandle::new(tokio::spawn(async move {
        let mut every = tokio::time::interval(LOOK_EVERY);
        loop {
            let done = tokio::select! {
                _ = every.tick() => None,
                turned = asked.recv() => {
                    let Some((turn, done)) = turned else { return };
                    // A turn, not a read, writes the room over: one that
                    // could not be read too.
                    if let Some(made) = turn.made(on.as_deref().unwrap_or_default()) {
                        kept::write(&file, &made);
                        on = Ok(made);
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
        true => (Err(Unread::NoRouter), None),
        false => crate::network::now().await,
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
pub(super) async fn from_this_network(request: Request, next: Next) -> Response {
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
