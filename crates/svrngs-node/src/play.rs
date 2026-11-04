// SPDX-License-Identifier: AGPL-3.0-or-later
//! Play (give-play; svrn-docs rings/current/GIVE.md §3 scene 2, §6): a port
//! another computer of the group lends ([`Raw`]), opened on this computer
//! at `127.0.0.1:<port>` while its person has Play on, so a program here
//! connects to it as to the lent port: the member's half of the door
//! ([`udp::carried`], [`tcp::carried`]). The port is the room's for the
//! lend's label ([`port_of`]), the same each time, so a program's saved
//! address keeps working.
//!
//! **One owner** ([`spawn`]), on the room's model ([`crate::room::owner`]):
//! which Plays are on is kept in [`PLAY_FILE`] by the room's store, so they
//! open again as svrngs starts. Each is open only while this computer's
//! person and its lender stand in a group held here together, read again
//! every [`LOOK_EVERY`]; one its lender's computer closed, or did not
//! answer, is opened again at the next look. Turned at [`ROUTE`] by the
//! home page's Play and Stop, its row's and "What acts as you"'s alike.
//!
//! **What it lets in** (svrn-docs WHAT_CAN_ACT_AS_YOU §4; the trust floor's
//! Play lines): a raw port carries no token, so any program on this
//! computer reaches the lend as this person while Play is on, which the
//! home page says beside its Stop. Nothing listens while it is off.

use std::collections::BTreeMap;
use std::net::Ipv4Addr;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};
use n0_future::task::AbortOnDropHandle;
use serde::{Deserialize, Serialize};
use svrngs_core::pass::Label;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{mpsc, oneshot};

use crate::door::{tcp, udp, LendRefusal};
use crate::lend::{LendName, Raw};
use crate::node::Asking;
use crate::room::{kept, owner::LOOK_EVERY, port_of};
use crate::Person;

/// Where Play is read and turned on and off: the home page's Play and Stop.
pub const ROUTE: &str = "/v1/play";

/// The file in the data dir that keeps which lends have Play on.
pub const PLAY_FILE: &str = "play.json";

/// A lend with Play on, as [`PLAY_FILE`] keeps it: the lender's key, the
/// lend's name, and how its bytes travel, as its lender lent it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct On {
    key: String,
    name: LendName,
    raw: Raw,
}

/// Where a lend with Play on is now: a closed set (AGENTS 9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "is", content = "words", rename_all = "snake_case")]
pub enum Open {
    /// Listening at its port; the lender's last words of a program it
    /// refused, if any (a limit reached).
    At(Option<String>),
    /// This computer's person and its lender stand in no group held here
    /// together now.
    Apart,
    /// Which groups this computer is in cannot be read.
    Unread,
    /// Its lender's computer did not open it, in its words.
    Unopened(String),
    /// Another program on this computer holds its port.
    Taken,
}

impl Open {
    /// What the home page says of it, the one wording, its port `port`.
    pub fn said(&self, port: u16) -> String {
        match self {
            Open::At(None) => format!("Connect to 127.0.0.1:{port}."),
            Open::At(Some(w)) => format!("Connect to 127.0.0.1:{port}. Its lender’s computer last said: {w}."),
            Open::Apart => "Closed: you and its lender are in no group together now.".into(),
            Open::Unread => "Closed for now: this computer cannot read which groups it is in.".into(),
            Open::Unopened(w) => format!("Closed for now: {w}. It opens again on its own once it can."),
            Open::Taken => format!("Closed for now: another program on this computer holds 127.0.0.1:{port}. Stop that program, and it opens again."),
        }
    }
}

/// A lend with Play on, as [`ROUTE`] answers it: whose it is, `None` for a
/// key no group held here names now; the lender's key; its name; its port;
/// where it is open or why not; and that in a person's words
/// ([`Open::said`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Playing {
    pub person: Option<Person>,
    pub key: String,
    pub name: LendName,
    pub port: u16,
    pub open: Open,
    pub said: String,
}

/// What Play is asked at [`ROUTE`]: the lend `name` of the computer whose
/// key is `key`, turned on or off.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayAsk {
    On { key: String, name: LendName },
    Off { key: String, name: LendName },
}

/// A change to what has Play on, sent to its owner: this one on, this one
/// off, or none, to read it.
pub(crate) enum Turn {
    On(On),
    Off(String, LendName),
    Read,
}

impl Turn {
    /// The lends `on` once turned, each once; `None` for a read.
    fn made(self, on: &[On]) -> Option<Vec<On>> {
        let mut on = on.to_vec();
        match self {
            Turn::On(o) => {
                on.retain(|x| (&x.key, &x.name) != (&o.key, &o.name));
                on.push(o);
            }
            Turn::Off(key, name) => on.retain(|o| o.key != key || o.name != name),
            Turn::Read => return None,
        }
        Some(on)
    }
}

/// A turn, and where its owner answers what has Play on after it, or why
/// that cannot be read.
type TurnAsk = (Turn, oneshot::Sender<Result<Vec<Playing>, String>>);

/// What Play's owner reads its asks from.
type Asks = mpsc::UnboundedReceiver<TurnAsk>;

/// Each lend open, by its label: the member's half of the door, and its
/// lender's last words of a program refused.
type Opened = BTreeMap<Label, (udp::Carried, Option<String>)>;

/// Play's owner's handle ([`spawn`]), held by the node's routers.
#[derive(Clone)]
pub(crate) struct Plays(mpsc::UnboundedSender<TurnAsk>);

impl Plays {
    /// A handle, and what its owner takes.
    pub(crate) fn new() -> (Self, Asks) {
        let (plays, asked) = mpsc::unbounded_channel();
        (Plays(plays), asked)
    }

    /// `turn` made by Play's owner, and what has Play on then, or why it
    /// cannot be read; `None` once it has stopped.
    async fn turn(&self, turn: Turn) -> Option<Result<Vec<Playing>, String>> {
        let (done, made) = oneshot::channel();
        self.0.send((turn, done)).ok()?;
        made.await.ok()
    }
}

/// Play's one owner, for as long as the node runs: the lends with Play on,
/// read from [`PLAY_FILE`] as it starts and written there at each turn,
/// each opened as [`play`] decides every [`LOOK_EVERY`] and at each ask.
pub(crate) fn spawn(at: Asking, mut asked: Asks) -> AbortOnDropHandle<()> {
    let file = at.door.data_dir.join(PLAY_FILE);
    let (mut on, mut open) = (kept::read::<On>(&file), Opened::new());
    AbortOnDropHandle::new(tokio::spawn(async move {
        let mut every = tokio::time::interval(LOOK_EVERY);
        loop {
            let done = tokio::select! {
                _ = every.tick() => None,
                turned = asked.recv() => {
                    let Some((turn, done)) = turned else { return };
                    if let Some(made) = turn.made(on.as_deref().unwrap_or_default()) {
                        kept::write(&file, &made);
                        on = Ok(made);
                    }
                    Some(done)
                }
            };
            let plays = play(&at, on.as_deref().unwrap_or_default(), &mut open).await;
            done.map(|d| d.send(on.as_ref().map(|_| plays).map_err(String::clone)));
        }
    }))
}

/// Each lend `on` open or not: open while this computer's person and its
/// lender stand in a group held here together, its carrying kept while it
/// runs and opened anew once it has ended; every other in `open` dropped,
/// which closes its port; and what has Play on, as it stands.
async fn play(at: &Asking, on: &[On], open: &mut Opened) -> Vec<Playing> {
    let (mut was, mut plays) = (std::mem::take(open), Vec::new());
    for o in on {
        let Some(label) = Label::of_hex(&o.key, o.name.as_str()) else {
            continue;
        };
        let port = port_of(&label);
        let kept = (was.remove(&label)).filter(|((_, task), _)| !task.is_finished());
        let now = match (at.host.shared_with(&o.key), kept) {
            (Err(_), _) => Open::Unread,
            (Ok((_, None)), _) => Open::Apart,
            (Ok(_), Some(((mut told, task), mut last))) => {
                while let Ok(words) = told.try_recv() {
                    last = Some(words);
                }
                open.insert(label, ((told, task), last.clone()));
                Open::At(last)
            }
            (Ok(_), None) => opened(at, o, (label, port), open).await,
        };
        tracing::debug!(target: "play", key = o.key, lend = %o.name, port, ?now, "play: open only while its person and its lender stand in a group together, at its label's port");
        let person = (at.host.rail.standing_anywhere(&o.key)).map(|(_, m)| m.person);
        let (key, name, said) = (o.key.clone(), o.name.clone(), now.said(port));
        plays.push(Playing {
            person,
            key,
            name,
            port,
            open: now,
            said,
        });
    }
    plays
}

/// The lend `o` opened at `port` on 127.0.0.1, UDP or TCP as it is lent,
/// carried to its lender's computer: [`Open::At`], kept in `open`; or why
/// not.
async fn opened(at: &Asking, o: &On, (label, port): (Label, u16), open: &mut Opened) -> Open {
    let local = (Ipv4Addr::LOCALHOST, port);
    let to = (at.host.contact(&o.key)).and_then(|c| at.host.transport.endpoint_addr_for(&c));
    let (Some(to), endpoint, name) = (to, &at.host.endpoint, &o.name) else {
        return Open::Apart;
    };
    let taken = |e: std::io::Error| {
        tracing::debug!(target: "play", lend = %name, port, error = %e, "play: not open — another program holds its port");
        Open::Taken
    };
    let carried = match o.raw {
        Raw::Udp => match UdpSocket::bind(local).await {
            Ok(socket) => udp::carried(endpoint, to, name, socket).await,
            Err(e) => return taken(e),
        },
        Raw::Tcp => match TcpListener::bind(local).await {
            Ok(listener) => tcp::carried(endpoint, to, name, listener).await,
            Err(e) => return taken(e),
        },
    };
    tracing::info!(target: "play", lend = %name, port, refused = ?carried.as_ref().err(), "play: opened on 127.0.0.1 and carried to its lender, or not");
    match carried {
        Ok(carried) => drop(open.insert(label, (carried, None))),
        Err(e) => return Open::Unopened(e.to_string()),
    }
    Open::At(None)
}

/// The route, for [`crate::api::router`] and the home page to serve.
pub(crate) fn router(asking: Asking) -> Router {
    (Router::new().route(ROUTE, get(listed).post(turn))).with_state(asking)
}

/// `GET /v1/play`: each lend with Play on, where it is open or why not.
async fn listed(State(at): State<Asking>) -> Response {
    turned(&at, Turn::Read).await
}

/// `POST /v1/play`: `ask` turned, and what has Play on then ([`listed`]).
/// A lend is turned on only as its lender's computer lends it to this one
/// now, a port: how its bytes travel is asked of that computer, never
/// taken from the page.
async fn turn(State(at): State<Asking>, Json(ask): Json<PlayAsk>) -> Response {
    let (key, name) = match ask {
        PlayAsk::Off { key, name } => return turned(&at, Turn::Off(key, name)).await,
        PlayAsk::On { key, name } => (key, name),
    };
    let lent = match at.host.shared_with(&key) {
        Ok((groups, Some(_))) => at.host.lends_in(&groups, None).await.lent,
        _ => Vec::new(),
    };
    let asked = |l: &crate::node::Lent| l.key == key && l.offered.name == name;
    let raw = (lent.iter().find(|l| asked(l))).and_then(|l| l.offered.raw);
    tracing::debug!(target: "play", key, lend = %name, ?raw, "play: turned on only as its lender lends it now, a port");
    let Some(raw) = raw else {
        let words = format!("{name} is not lent to this computer as a port now");
        return LendRefusal::Unlent.say(StatusCode::CONFLICT, words);
    };
    turned(&at, Turn::On(On { key, name, raw })).await
}

/// `turn` made by Play's owner, and what has Play on then; why that cannot
/// be read, which opens nothing until a turn writes it over; or refused
/// when its owner has stopped.
async fn turned(at: &Asking, turn: Turn) -> Response {
    let plays = at.plays.turn(turn).await;
    tracing::debug!(target: "play", made = plays.is_some(), unread = ?plays.as_ref().and_then(|p| p.as_ref().err()), "play: asked of its owner: what has Play on, why it cannot be read, or refused as it has stopped");
    match plays {
        Some(Ok(plays)) => Json(plays).into_response(),
        Some(Err(why)) => LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, format!("This computer cannot read what you have Play on ({why}), so nothing is open. Press Play again, and it starts over from that one.")),
        None => LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, "Play cannot be changed on this computer now"),
    }
}
