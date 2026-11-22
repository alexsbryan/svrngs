// SPDX-License-Identifier: AGPL-3.0-or-later
//! The room's owner: one task for as long as the node runs ([`spawn`]),
//! told by messages ([`Room`]), owning the lends on and each one's
//! listener on this computer's network, bound by the lend's label alone
//! and dropped as [`super::serves`] decides; and the listener's two
//! layers, the caller on this network ([`from_this_network`]) and the
//! lend's body ([`in_room`], [`crate::origin::reach`]). And a lend's
//! withdrawal ([`withdraw`]), which turns it off here first.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::Path;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::Router;
use n0_future::task::AbortOnDropHandle;
use svrngs_core::pass::Label;

use super::{kept, port_of, serves, InRoom, On, Served, Turn, ROOM_FILE};
use crate::folder::carried::Handled;
use crate::lend::{LendError, LendName};
use crate::network::{guess, Whereabouts};
use crate::node::Asking;
use crate::origin::{Appending, Unconsented};

/// How often the room reads which network this computer is on: the
/// longest a lend stays served after this computer moves to another
/// network that gives it the same address, which iroh's watch of its
/// addresses (`Endpoint::watch_addr`) never sees ([`crate::network::
/// LOOK_EVERY`], the cadence's one home).
pub(crate) use crate::network::LOOK_EVERY;

/// Each lend served, by its label: where, and its listener.
type Listening = BTreeMap<Label, (SocketAddr, AbortOnDropHandle<()>)>;

/// The room's owner's handle ([`spawn`]), held by the node's routers.
pub(crate) type Room = kept::Owner<Turn, InRoom>;

/// How the room's store tells its lends apart: by the lender's key and the
/// lend's name.
const ID: fn(&On) -> (&String, &LendName) = |o| (&o.key, &o.name);

/// The room's one owner, for as long as the node runs: the lends on, read
/// from [`ROOM_FILE`] as it starts and written there at each turn, each
/// served as [`serve`] decides every [`LOOK_EVERY`] and at each ask.
pub(crate) fn spawn(at: Asking, asked: kept::Asks<Turn, InRoom>) -> AbortOnDropHandle<()> {
    let (file, mut listening) = (at.door.data_dir.join(ROOM_FILE), Listening::new());
    let served = async move |on: &[On]| serve(&at, on, &mut listening).await;
    AbortOnDropHandle::new(tokio::spawn(kept::run(file, asked, ID, served)))
}

/// This computer's lend `name`, `key` its key, turned off for the room by
/// `off`, which tells the room's owner, then withdrawn
/// ([`crate::lend::withdraw`]): so a lend shared again under that name
/// starts off this computer's network, and its person opens it there again
/// if they want it. The one way svrngs takes a lend back, by every path:
/// the home page's Stop and its API's (home.rs), its owner told here
/// ([`off`]); `svrngs unshare` (cli.rs), svrngs running here told at
/// [`super::ROUTE`] as the home page tells it ([`super::RoomAsk::Off`]),
/// else [`withdraw_stopped`]. Nothing is withdrawn while the room's owner
/// can't be told, `off`'s words saying why.
pub async fn withdraw(
    dir: &Path,
    key: &str,
    name: &LendName,
    off: impl AsyncFnOnce(String, LendName) -> Result<(), String>,
) -> Result<bool, LendError> {
    let told = off(key.to_string(), name.clone()).await;
    tracing::debug!(target: "room", lend = %name, ?told, "room: a lend withdrawn is turned off for the room first, so one lent again under its name starts off this network; refused, nothing withdrawn, while the room's owner can't be told");
    told.map_err(|why| LendError::Unclosed(name.clone(), why))?;
    crate::lend::withdraw(dir, name)
}

/// `room`, the room's owner, told to turn the lend `name` of the computer
/// whose key is `key` off ([`Turn::Off`]); or why not: it has stopped.
pub(crate) async fn off(room: &Room, key: String, name: LendName) -> Result<(), String> {
    let made = room.turn(Turn::Off(key, name)).await;
    made.map(drop)
        .ok_or_else(|| "the room's owner has stopped".into())
}

/// [`withdraw`] with svrngs stopped here, while `svrngs unshare` holds the
/// data dir `dir`: the room's store told by its own owner ([`kept::run`]),
/// run for that one turn and serving nothing, as nothing listens while
/// svrngs is stopped. So the store is changed as it always is, by its
/// owner's turn, never written behind it, and svrngs started again opens
/// none of it.
pub async fn withdraw_stopped(dir: &Path, key: &str, name: &LendName) -> Result<bool, LendError> {
    let (room, asked) = Room::new();
    // Its turn is answered after it is written, so it is let go then.
    let owner = kept::run(dir.join(ROOM_FILE), asked, ID, async |_: &[On]| Vec::new());
    let _owner = AbortOnDropHandle::new(tokio::spawn(owner));
    withdraw(dir, key, name, async |k, n| off(&room, k, n).await).await
}

/// Each lend `on` served or not, as [`serves`] decides at the network read
/// now: its listener kept where it is served still, bound where it is
/// served anew, and dropped, with `listening`'s rest, where it is not; and
/// the room as it stands.
async fn serve(at: &Asking, on: &[On], listening: &mut Listening) -> Vec<InRoom> {
    if on.is_empty() {
        tracing::debug!(target: "room", "room: no lend on, so no network read and no listener kept");
        listening.clear();
        return Vec::new();
    }
    let here = crate::network::now().await;
    let (network, address, tunnel) = (here.network, here.address, here.tunnel);
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
        room.push(InRoom {
            person: rail.standing_anywhere(&o.key).map(|(_, m)| m.person),
            key: o.key.clone(),
            name: o.name.clone(),
            said: served.said_beside(tunnel.as_deref()),
            served,
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
    let serving = tokio::spawn(async move { drop(axum::serve(listener, routes).await) });
    tracing::info!(target: "room", %addr, %label, "room: serving a lend to this computer's network");
    listening.insert(label, (addr, AbortOnDropHandle::new(serving)));
    Served::At(addr)
}

/// What a caller at the room door proves of the entries a lend's answer
/// names to write as this computer's person: nothing. The door's one
/// check, [`from_this_network`], is of the caller's address, which proves
/// no person, so each such answer is refused.
pub(super) const IN_ROOM: Appending = Appending::Denied(Unconsented::NetworkOnly);

/// A request at the room door, carried to its lend as at its origin, for
/// the lend's own page: the room serves one lend at each port.
async fn in_room(State((at, label)): State<(Asking, Label)>, req: Request) -> Handled {
    crate::origin::reach(&at, label.clone(), label, IN_ROOM, req).await
}

/// Serve a request only from a caller on this computer's network: one the
/// door's own guess calls home ([`guess`]: private, link-local or
/// unique-local), so "this network" has one decider. Any other, or one
/// whose address is unknown, is refused 403 before any lend is reached.
pub(super) async fn from_this_network(request: Request, next: Next) -> Response {
    let caller = crate::session::peer(&request).map(|c| c.ip());
    if guess(caller) == Whereabouts::Home {
        tracing::debug!(target: "room", ?caller, path = %request.uri().path(), "room: a caller on this network");
        return next.run(request).await;
    }
    tracing::debug!(target: "room", ?caller, path = %request.uri().path(), "room: refused — the caller is not on this computer's network");
    let why = "this is opened only on the network of the computer that serves it";
    crate::door::LendRefusal::closed(why)
}
