// SPDX-License-Identifier: AGPL-3.0-or-later
//! The room door (u6-1; DESIGN §3, "The room", calls 2 and 3): this
//! computer serves the lends its person names to its own network while
//! the command runs, each at `http://<private address>:<port>/`, a port of
//! its own, so the lend's root is its own as at its origin on loopback. One
//! body with those origins ([`crate::origin::reach`]): reached as this
//! computer's person, dialled under its key, so the lender's door counts
//! the room as one person and guesses its path as it does now; a device in
//! the room is this person's own, and no lend is carried for anyone else.
//!
//! New, in the shape of upstream's LAN door and address picker, not their
//! code (DESIGN §8, U6's inventory): "the door names the PORT, this machine
//! names the ADDRESS" (`sovereign-cli-mesh/src/mesh_guest_link.rs:10-12`
//! at `e1a43bcd2`), and a listener that lives only while it is wanted
//! (`sovereign-daemon/src/guest_door.rs:429-431`) — here bound on the
//! private address alone, so safe by address and by lifetime.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::Router;
use svrngs_core::pass::Label;

use crate::api::ApiError;
use crate::door::limit::{guess, Whereabouts};
use crate::lend::LendName;
use crate::node::{label, Asking, Lent, Node};

/// The address a room's devices reach this computer at, from the
/// addresses its endpoint lists (`Endpoint::addr().ip_addrs()`) and the
/// one its outward route leaves from ([`route`]): that one when it is a
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
/// device is typed the address once (DESIGN §8, U6's inventory), and below
/// macOS's ephemeral range. One that is taken is refused by name.
pub fn port_of(label: &Label) -> u16 {
    let h = label
        .as_str()
        .bytes()
        .fold(0u32, |h, b| h.wrapping_mul(31) ^ u32::from(b));
    40_000 + (h % 9_000) as u16
}

/// The lends served to the room. Dropping it stops every listener, so
/// nothing is served once the command stops.
pub struct Room {
    /// Each lend served, and where the room reaches it.
    pub served: Vec<(Lent, SocketAddr)>,
    tasks: Vec<tokio::task::JoinHandle<()>>,
    asking: Asking,
}

impl Drop for Room {
    fn drop(&mut self) {
        self.tasks.iter().for_each(|t| t.abort());
        let room = &self.asking.host.room;
        room.send_modify(|m| m.retain(|_, at| !self.served.iter().any(|(_, a)| a == at)));
        tracing::debug!(target: "room", lends = self.served.len(), "room: stopped serving the room");
    }
}

/// Why the room door did not open.
#[derive(Debug, thiserror::Error)]
pub enum RoomError {
    #[error("nobody in your groups lends {0} to this computer now")]
    NotLent(LendName),
    #[error(transparent)]
    Listen(#[from] ApiError),
}

/// Serve every lend of the people of this computer's groups that `names`
/// names — each other computer's, asked now ([`Node::lends`]) — on `at`,
/// this computer's private address ([`pick`]), at its [`port_of`], to
/// callers on this network ([`from_this_network`]), until the [`Room`] is
/// dropped. A name nobody lends opens nothing.
pub async fn open(node: &Node, at: Ipv4Addr, names: &[LendName]) -> Result<Room, RoomError> {
    let lent = node.lends().await.lent;
    if let Some(none) = names
        .iter()
        .find(|n| !lent.iter().any(|l| &l.offered.name == *n))
    {
        tracing::debug!(target: "room", lend = %none, "room: refused — a name nobody lends now");
        return Err(RoomError::NotLent(none.clone()));
    }
    let lent = lent.into_iter().filter(|l| names.contains(&l.offered.name));
    let asking = node.asking.clone();
    let (served, tasks) = (Vec::new(), Vec::new());
    let mut room = Room {
        served,
        tasks,
        asking: asking.clone(),
    };
    for lent in lent {
        let Some(label) = label(&lent.key, lent.offered.name.as_str()) else {
            continue;
        };
        let addr = SocketAddr::new(at.into(), port_of(&label));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| ApiError::Listen(addr, e))?;
        let routes = Router::new()
            .fallback(in_room)
            .with_state((asking.clone(), label.clone()))
            .layer(axum::middleware::from_fn(from_this_network))
            .into_make_service_with_connect_info::<SocketAddr>();
        room.tasks.push(tokio::spawn(async move {
            let _ = axum::serve(listener, routes).await;
        }));
        tracing::info!(target: "room", %addr, person = %lent.person, lend = %lent.offered.name, "room: serving a lend to this computer's network");
        asking
            .host
            .room
            .send_if_modified(|m| m.insert(label, addr).is_none());
        room.served.push((lent, addr));
    }
    Ok(room)
}

/// A request at the room door, carried to its lend as at its origin.
async fn in_room(State((at, label)): State<(Asking, Label)>, req: Request) -> Response {
    crate::origin::reach(&at, label, req).await
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
