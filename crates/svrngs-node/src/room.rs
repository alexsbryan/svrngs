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
//! **The room is the running node's** ([`owner::spawn`], its one owner), turned on
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
//! hardware address ([`crate::network`]), and is served only while this computer is
//! on that network, read again every [`owner::LOOK_EVERY`]. On another network, or
//! one that cannot be read, it is not served, and the room says why.
//!
//! New, in the shape of upstream's LAN door and address picker, not their
//! code (PLAN §4, U6's inventory): "the door names the PORT, this machine
//! names the ADDRESS" (`sovereign-cli-mesh/src/mesh_guest_link.rs:10-12`
//! at `e1a43bcd2`) — here bound on the private address alone, so safe by
//! address and by network.
//!
//! Its parts: this module, the room as a person turns it, pure (each lend
//! on with its network, and [`serves`], whether it is served and the words
//! for it); and its edges, each a module of its own: [`kept`], room.json;
//! [`owner`], the task that owns the room and its listeners; [`route`],
//! `/v1/room`, which translates, deciding nothing.

use std::net::{Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};
use svrngs_core::pass::Label;

use crate::lend::LendName;
use crate::network::{Network, Unread};
use crate::Person;

pub(crate) mod kept;
pub(crate) mod owner;
pub(crate) mod route;

/// Where the room is read and turned on and off: the home page's button
/// and `svrngs room` (cli.rs) ask here.
pub const ROUTE: &str = "/v1/room";

/// The file in the data dir that keeps which lends are on for the room.
pub const ROOM_FILE: &str = "room.json";

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
            Served::At(at) => format!("Open to this WiFi: type http://{at}/ into an app or a browser on a device here, such as your TV, a phone or another computer. Anyone on this WiFi can open it there as you."),
            Served::Elsewhere => "Closed for now: this computer is on another WiFi than the one where you turned it on. It opens again when you are there again.".into(),
            Served::Unread => "Closed for now: this computer cannot tell which WiFi it is on, so it opens it to nobody.".into(),
            Served::NoAddress => "Closed for now: this computer is on no WiFi or home network. Join one, or plug it in.".into(),
            Served::Taken => "Closed for now: another program on this computer holds the port it needs. Stop that program, and it opens again.".into(),
        }
    }
}

/// Whether a lend turned on at `kept` is served at `port`, with this
/// computer on `now` at `at` ([`pick`]): only on the network it was turned
/// on at, never on one that cannot be read. Pure, so each answer is judged
/// with values (`room_tests.rs`).
pub fn serves(
    kept: &Network,
    now: Result<&Network, &Unread>,
    at: Option<Ipv4Addr>,
    port: u16,
) -> Served {
    let served = match (now, at) {
        (Err(_), _) => Served::Unread,
        (Ok(now), _) if now != kept => Served::Elsewhere,
        (Ok(_), None) => Served::NoAddress,
        (Ok(_), Some(at)) => Served::At(SocketAddr::new(at.into(), port)),
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

/// A change to the room, sent to its owner: these on, this one off, or
/// none, to read it.
pub(crate) enum Turn {
    On(Vec<On>),
    Off(String, LendName),
    Read,
}

impl Turn {
    /// The lends `on` once turned, each lend on once; `None` for a read,
    /// which changes nothing.
    fn made(self, on: &[On]) -> Option<Vec<On>> {
        let mut on = on.to_vec();
        match self {
            Turn::On(lends) => lends.into_iter().for_each(|l| {
                on.retain(|o| (&o.key, &o.name) != (&l.key, &l.name));
                on.push(l);
            }),
            Turn::Off(key, name) => on.retain(|o| o.key != key || o.name != name),
            Turn::Read => return None,
        }
        Some(on)
    }
}

#[cfg(test)]
#[path = "room_tests.rs"]
mod tests;
