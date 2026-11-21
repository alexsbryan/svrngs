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
//! every [`crate::room::owner::LOOK_EVERY`]; one its lender's computer
//! closed, or did not answer, is opened again at the next look. Turned at
//! [`ROUTE`] by the home page's Play and Stop, its row's and "What acts as
//! you"'s alike.
//!
//! **What it lets in** (svrn-docs WHAT_CAN_ACT_AS_YOU §4; the trust floor's
//! Play lines): a raw port carries no token, so any program on this
//! computer reaches the lend as this person while Play is on, which the
//! home page says beside its Stop. Nothing listens while it is off.
//!
//! **A lend at an address** (an app on a port, a model among them) opens
//! nowhere new: while Play is on, its address on this computer,
//! `http://<label>.localhost:<port>/`, lets in a program of this account
//! with no key, as [`shown`] decides it for
//! `crate::origin`'s `lent`, the lender's terms and the door's rules as
//! for any asker. Off, the address is as it was: a browser's session or
//! the token.

use std::collections::BTreeMap;
use std::net::Ipv4Addr;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};
use n0_future::task::AbortOnDropHandle;
use serde::{Deserialize, Serialize};
use svrngs_core::pass::Label;
use tokio::net::{TcpListener, UdpSocket};

use crate::door::{tcp, udp, LendRefusal};
use crate::lend::{LendName, Raw};
use crate::node::Asking;
use crate::room::{kept, port_of};
use crate::session::{peer, Shown};
use crate::uid::Owner;
use crate::Person;

/// Where Play is read and turned on and off: the home page's Play and Stop.
pub const ROUTE: &str = "/v1/play";

/// The file in the data dir that keeps which lends have Play on.
pub const PLAY_FILE: &str = "play.json";

/// A lend with Play on, as [`PLAY_FILE`] keeps it: the lender's key, the
/// lend's name, and how its bytes travel, as its lender lent it: a raw
/// port's way, or none for a lend at an address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct On {
    key: String,
    name: LendName,
    raw: Option<Raw>,
}

/// What has Play on, as Play's owner last looked ([`play`]): where
/// [`shown`] reads whether Play is open at a lend's address, and only
/// Play's owner writes.
pub(crate) type Played = tokio::sync::watch::Sender<Vec<Playing>>;

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
    /// What the home page says of it, the one wording, a program connecting
    /// `at`.
    pub fn said(&self, at: &str) -> String {
        match self {
            Open::At(None) => format!("Connect to {at}."),
            Open::At(Some(w)) => format!("Connect to {at}. Its lender’s computer last said: {w}."),
            Open::Apart => "Closed: you and its lender are in no group together now.".into(),
            Open::Unread => "Closed for now: this computer cannot read which groups it is in.".into(),
            Open::Unopened(w) => format!("Closed for now: {w}. It opens again on its own once it can."),
            Open::Taken => format!("Closed for now: another program on this computer holds {at}. Stop that program, and it opens again."),
        }
    }
}

/// A lend with Play on, as [`ROUTE`] answers it: whose it is, `None` for a
/// key no group held here names now; the lender's key; its name; where a
/// program connects, `127.0.0.1:<port>` for a raw port and the lend's
/// address here for one at an address (`crate::origin::origin`); where it
/// is open or why not; and that in a person's words ([`Open::said`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Playing {
    pub person: Option<Person>,
    pub key: String,
    pub name: LendName,
    pub at: String,
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

/// A change to what has Play on, sent to its owner.
pub(crate) type Turn = kept::Turn<On>;

/// Each lend open, by its label: the member's half of the door, and its
/// lender's last words of a program refused.
type Opened = BTreeMap<Label, (udp::Carried, Option<String>)>;

/// Play's owner's handle ([`spawn`]), held by the node's routers.
pub(crate) type Plays = kept::Owner<Turn, Playing>;

/// Play's one owner, for as long as the node runs: the lends with Play on,
/// read from [`PLAY_FILE`] as it starts and written there at each turn,
/// each opened as [`play`] decides every
/// [`crate::room::owner::LOOK_EVERY`] and at each ask.
pub(crate) fn spawn(at: Asking, asked: kept::Asks<Turn, Playing>) -> AbortOnDropHandle<()> {
    let (file, mut open) = (at.door.data_dir.join(PLAY_FILE), Opened::new());
    let played = async move |on: &[On]| play(&at, on, &mut open).await;
    let id: fn(&On) -> (&String, &LendName) = |o| (&o.key, &o.name);
    AbortOnDropHandle::new(tokio::spawn(kept::run(file, asked, id, played)))
}

/// Each lend `on` open or not: open while this computer's person and its
/// lender stand in a group held here together, its carrying kept while it
/// runs and opened anew once it has ended; every other in `open` dropped,
/// which closes its port; and what has Play on, as it stands, told to
/// [`Played`] too.
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
                open.insert(label.clone(), ((told, task), last.clone()));
                Open::At(last)
            }
            (Ok(_), None) => opened(at, o, (label.clone(), port), open).await,
        };
        let to = (o.raw.map(|_| format!("127.0.0.1:{port}")))
            .unwrap_or_else(|| crate::origin::origin(&label, at.host.api_port));
        tracing::debug!(target: "play", key = o.key, lend = %o.name, to, ?now, "play: open only while its person and its lender stand in a group together, a port at its label's port, a lend at an address there alone");
        plays.push(Playing {
            person: (at.host.rail.standing_anywhere(&o.key)).map(|(_, m)| m.person),
            key: o.key.clone(),
            name: o.name.clone(),
            said: now.said(&to),
            at: to,
            open: now,
        });
    }
    at.played.send_replace(plays.clone());
    plays
}

/// The lend `o` opened at `port` on 127.0.0.1, UDP or TCP as it is lent,
/// carried to its lender's computer: [`Open::At`], kept in `open`; or why
/// not. A lend at an address opens nothing here: its address lets a
/// program in ([`shown`]).
async fn opened(at: &Asking, o: &On, (label, port): (Label, u16), open: &mut Opened) -> Open {
    let local = (Ipv4Addr::LOCALHOST, port);
    let to = (at.host.contact(&o.key)).and_then(|c| at.host.transport.endpoint_addr_for(&c));
    let (Some(to), net, name) = (to, &*at.host.transport, &o.name) else {
        return Open::Apart;
    };
    let taken = |e: std::io::Error| {
        tracing::debug!(target: "play", lend = %name, port, error = %e, "play: not open — another program holds its port");
        Open::Taken
    };
    let carried = match o.raw {
        None => return Open::At(None),
        Some(Raw::Udp) => match UdpSocket::bind(local).await {
            Ok(socket) => udp::carried(net, to, name, socket).await,
            Err(e) => return taken(e),
        },
        Some(Raw::Tcp) => match TcpListener::bind(local).await {
            Ok(listener) => tcp::carried(net, to, name, listener).await,
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
/// now, a port or at an address: how its bytes travel is asked of that
/// computer, never taken from the page. A carried app, the group's page
/// and no program's, is not Play's.
async fn turn(State(at): State<Asking>, Json(ask): Json<PlayAsk>) -> Response {
    let (key, name) = match ask {
        PlayAsk::Off { key, name } => return turned(&at, Turn::Off(key, name)).await,
        PlayAsk::On { key, name } => (key, name),
    };
    let lent = match at.host.shared_with(&key) {
        Ok((groups, Some(_))) => at.host.lends_in(&groups, None).await.lent,
        _ => Vec::new(),
    };
    let asked = |l: &&crate::node::Lent| l.key == key && l.offered.name == name;
    let raw =
        (lent.iter().find(asked)).map(|l| (l.offered.group.is_none()).then_some(l.offered.raw));
    tracing::debug!(target: "play", key, lend = %name, ?raw, "play: turned on only as its lender lends it now, a port or at an address, and never a carried app");
    let Some(Some(raw)) = raw else {
        let words = format!("{name} is not lent to this computer now");
        return LendRefusal::Unlent.say(StatusCode::CONFLICT, words);
    };
    turned(&at, Turn::On(vec![On { key, name, raw }])).await
}

/// Why Play lets in no program at a lend's address it opened, in the words
/// its refusal says: a closed set (AGENTS 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum Unplayed {
    #[error("Play lets in programs of your own account on this computer, and this came from another account’s")]
    Theirs,
    /// Whose program it is can't be read: why.
    #[error("Play lets in programs of your own account on this computer, and which account this came from can’t be read here: {0}")]
    Unread(&'static str),
    /// It asked what svrngs answers at the lend's address in the lend's
    /// place (`crate::surface::AppRoute`), or by a read's address: the
    /// lends its page reads, whose read addresses outlive Play, its stop,
    /// its opener, its reads through svrngs.
    #[error("Play lets a program on this computer use the lend itself, and this asked svrngs’s own address there, which answers only your browser, or a program with your token")]
    Svrngs,
}

/// The headers a program's client sends a key in, as each one's own client
/// sends it (read 2026-11-21): `Authorization`, OpenAI's and Anthropic's
/// and most; `x-api-key`, as Anthropic's pages spell a request;
/// `api-key`, Azure OpenAI's; `x-goog-api-key`, Google's. Taken off a
/// request Play lets in, unread, in any case, as a header's name is one. A
/// key in its address or a cookie is not one svrngs can tell from the rest.
const KEYS: [&str; 4] = ["authorization", "x-api-key", "api-key", "x-goog-api-key"];

/// Whether the socket a program asked from, as `owner`, is of this
/// process's account, `me`: Play's rule of whose program it lets in.
fn ours(owner: Owner, me: Option<u32>) -> Result<(), Unplayed> {
    match (owner, me) {
        (Owner::Uid(uid), Some(me)) if uid == me => Ok(()),
        (Owner::Uid(_) | Owner::NotMine, Some(_)) => Err(Unplayed::Theirs),
        (Owner::Unjudged(why), _) => Err(Unplayed::Unread(why)),
        (_, None) => Err(Unplayed::Unread("this program's own account can't be read")),
        (Owner::NoReader, _) => Err(Unplayed::Unread("this system has no reader of sockets")),
    }
}

/// What `req` at the lend `l`'s address shows by Play: [`Shown::Played`],
/// each header of [`KEYS`] taken off unread, since a program that insists
/// on a key sends any text, where Play opened it and `req` is a program's
/// of this account ([`ours`]), its socket's owner read as an open's is
/// (`crate::session::Sessions::whose`); it reaches the lend itself alone
/// (`crate::origin`'s `lent`, [`Unplayed::Svrngs`]). [`Shown::Neither`], as
/// before Play, where Play is off, or where `req` is a page's, as the
/// record API tells one (`crate::api::page_of`: an `Origin` or a
/// `Sec-Fetch-Site`, which a program's own HTTP client doesn't send). Or
/// refused, saying why.
pub(crate) async fn shown(at: &Asking, l: &Label, req: &mut Request) -> Result<Shown, Response> {
    let address = crate::origin::origin(l, at.host.api_port);
    let on = (at.played.borrow().iter()).any(|p| p.at == address && matches!(p.open, Open::At(_)));
    if !on || crate::api::page_of(req.headers()).is_some() {
        tracing::debug!(target: "play", label = %l, on, "play: a lend's address lets in by Play no page, and nothing while Play is off");
        return Ok(Shown::Neither);
    }
    let owner = at.sessions.whose(peer(req), &*at.host.transport).await;
    let judged = ours(owner.clone(), crate::uid::mine().ok());
    tracing::debug!(target: "play", label = %l, ?owner, ?judged, "play: on at this lend's address, so it lets in a program of this account, whatever key it sends");
    judged.map_err(|why| LendRefusal::closed(why.to_string()))?;
    let headers = req.headers_mut();
    KEYS.iter().for_each(|key| drop(headers.remove(*key)));
    Ok(Shown::Played)
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
