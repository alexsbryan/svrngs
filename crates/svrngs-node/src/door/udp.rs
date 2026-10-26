// SPDX-License-Identifier: AGPL-3.0-or-later
//! The door's UDP half (give-udp; svrn-docs rings/current/GIVE.md §6): a
//! lend of a UDP port ([`Raw::Udp`]), carried as QUIC datagrams on
//! [`Alpn::Port`], one connection for one member's use of one lend.
//!
//! ```text
//!   a program ─UDP─▶ the member's socket ═datagrams═▶ the lender ─UDP─▶ 127.0.0.1:<port>
//!   on 127.0.0.1     a flow per program's port          a socket per flow, on 127.0.0.1:0
//! ```
//!
//! **The opening.** The member opens one bi-stream and writes the lend's
//! name on a line. The lender answers `ok`, or its refusal in a person's
//! words, and keeps the stream to tell a flow it refuses: `<flow> <words>`,
//! a line each.
//!
//! **The frame** (its layout this row's, ralph/next/give/CHARTER.md
//! "Decide"): the flow, u32; the packet's number, u32; the fragment's index
//! and the packet's count of them, u8 each; then the fragment. A packet
//! larger than `max_datagram_size` less [`HEAD`] is cut, and the far side
//! passes it on once every fragment has come. A fragment lost loses its
//! packet alone, as IP fragmentation would, so a program sees the loss it
//! was built for.
//!
//! **A flow** is the lender's task for one program on the member's
//! computer: let in by its person's standing and the lend's limit, it holds
//! a socket to the lent port and its person's use of the lend, and closes
//! once idle past [`IDLE`] either way, traced. A flow refused is asked again
//! once as long has passed.

use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::num::NonZeroU16;
use std::time::{Duration, Instant};

use axum::body::Bytes;
use iroh::endpoint::{Connection, RecvStream, SendStream};
use n0_future::task::AbortOnDropHandle;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader, Lines};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use super::{busy_words, find, now, Door, InFlight, PersonKey, Verdict, LENDS_NOTHING};
use crate::iroh::{closed_unadmitted, key_actor, Alpn, Endpoint, EndpointAddr, Via};
use crate::lend::{Lend, LendName, Raw, Source};

/// The frame's bytes before a fragment: flow, number, index and count.
pub(crate) const HEAD: usize = 10;
/// How long a flow with no packet either way stays open: a program that
/// sends nothing for a minute has gone, or will open its flow again.
pub(crate) const IDLE: Duration = Duration::from_secs(60);
/// How long a packet's fragments wait for the rest: past it, one was lost.
pub(crate) const WHOLE_WITHIN: Duration = Duration::from_secs(2);
/// How often each side looks for what is past its time.
const SWEEP: Duration = Duration::from_secs(1);
/// The largest UDP packet, the bound of what a socket reads.
const LARGEST: usize = 65_535;

/// One datagram, its frame read: flow, number, index, count and fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Frame(pub u32, pub u32, pub u8, pub u8, pub Bytes);

impl Frame {
    /// The frame `d` holds; `None` for one too short, or whose index is
    /// past its count.
    pub(crate) fn parse(d: Bytes) -> Option<Self> {
        let word = |at: usize| Some(u32::from_be_bytes(d.get(at..at + 4)?.try_into().ok()?));
        let (flow, number, index, count) = (word(0)?, word(4)?, *d.get(8)?, *d.get(9)?);
        (index < count).then(|| Self(flow, number, index, count, d.slice(HEAD..)))
    }
}

/// `packet`, the `number`th of `flow`, cut into datagrams of at most `max`
/// bytes, each framed; `None` when that takes more than 255.
pub(crate) fn framed(flow: u32, number: u32, packet: &[u8], max: usize) -> Option<Vec<Bytes>> {
    let room = max.checked_sub(HEAD).filter(|r| *r > 0)?;
    let mut parts: Vec<&[u8]> = packet.chunks(room).collect();
    parts.resize(parts.len().max(1), &[]);
    let count = u8::try_from(parts.len()).ok()?;
    let head = [flow.to_be_bytes(), number.to_be_bytes()].concat();
    let frame = |(i, part): (u8, &&[u8])| [&head[..], &[i, count], part].concat().into();
    Some((0..count).zip(&parts).map(frame).collect())
}

/// The packets waiting for the rest of their fragments, by flow and
/// number, since their first came.
pub(crate) type Whole = BTreeMap<(u32, u32), (Instant, Vec<Option<Bytes>>)>;

/// Whether what came `at` is past `after` at `now`: a flow's last packet
/// past [`IDLE`], or a packet's first fragment past [`WHOLE_WITHIN`].
pub(crate) fn past(at: Instant, after: Duration, now: Instant) -> bool {
    now.saturating_duration_since(at) >= after
}

/// The flow and packet the datagram `d` completes in `whole`, at `now`;
/// each packet still missing a fragment past [`WHOLE_WITHIN`] dropped,
/// traced.
pub(crate) fn take(whole: &mut Whole, d: Bytes, now: Instant) -> Option<(u32, Vec<u8>)> {
    let lost = whole.extract_if(.., |_, (at, _)| past(*at, WHOLE_WITHIN, now));
    let lost: Vec<(u32, u32)> = lost.map(|(at, _)| at).collect();
    if !lost.is_empty() {
        tracing::debug!(target: "door", ?lost, "udp: packets dropped — a fragment of each was lost");
    }
    let Frame(flow, n, i, count, part) = Frame::parse(d)?;
    let (_, parts) = (whole.entry((flow, n))).or_insert((now, vec![None; count.into()]));
    *parts.get_mut(usize::from(i))? = Some(part);
    parts.iter().all(Option::is_some).then_some(())?;
    let (_, parts) = whole.remove(&(flow, n))?;
    Some((flow, parts.into_iter().flatten().flatten().collect()))
}

/// `packet`, the next of `flow` on `conn`: dropped, at debug, when the
/// connection carries no datagram or it needs more than 255.
fn carry(conn: &Connection, flow: u32, number: &mut u32, packet: &[u8]) {
    *number = number.wrapping_add(1);
    let max = conn.max_datagram_size();
    let frames = max.and_then(|max| framed(flow, *number, packet, max));
    let sent = (frames.iter().flatten()).try_for_each(|d| conn.send_datagram(d.clone()));
    if sent.is_err() || frames.is_none() {
        tracing::debug!(target: "door", flow, len = packet.len(), ?max, ?sent, "udp: a packet dropped — the connection carries no datagram that size, or did not send it");
    }
}

/// Each flow's task, by the packets it is sent, or when it was refused.
type Flows = BTreeMap<u32, Result<mpsc::Sender<Vec<u8>>, Instant>>;

/// The lender's half: `conn`, its dialer admitted by standing in a group
/// held here (node.rs `admit_dialer`), opening one raw lend, its line the
/// way it travels and its name, `UDP <name>` or `TCP <name>`; a TCP lend's
/// connections are [`super::tcp`]'s.
pub(crate) async fn serve(door: Door, conn: Connection) {
    let actor = key_actor(&conn.remote_id());
    let Ok((mut send, recv)) = conn.accept_bi().await else {
        return tracing::debug!(target: "door", actor, "raw: the member closed before naming a lend");
    };
    let mut named = String::new();
    let _ = BufReader::new(recv.take(256)).read_line(&mut named).await;
    let (said, name) = named.trim().split_once(' ').unwrap_or_default();
    let raw = [Raw::Udp, Raw::Tcp].into_iter().find(|r| r.said() == said);
    let found = find(&door, name).ok().flatten();
    tracing::debug!(target: "door", actor, named, source = ?found.as_ref().map(|l| &l.source), "raw: a lend asked for, opened only when declared as the port asked");
    let port = |l: &Lend| match l.source {
        Source::Raw(r, port) if Some(r) == raw => Some(port),
        _ => None,
    };
    let Some((port, lend)) = found.and_then(|l| Some((port(&l)?, l))) else {
        let said = raw.map_or("raw", Raw::said);
        let words = format!("{name} is not lent here as a {said} port\n");
        let _ = (send.write_all(words.as_bytes()).await, send.finish());
        return drop(tokio::time::timeout(Duration::from_secs(5), conn.closed()).await);
    };
    let _ = send.write_all(b"ok\n").await;
    let lent = (&door, actor.as_str(), &lend, port, &conn);
    if raw == Some(Raw::Tcp) {
        return super::tcp::serve(lent).await;
    }
    let mut flows = Flows::new();
    let mut whole = Whole::new();
    while let Ok(d) = conn.read_datagram().await {
        let Some((flow, packet)) = take(&mut whole, d, now()) else {
            continue;
        };
        let ask = match flows.get(&flow) {
            Some(Ok(packets)) => packets.is_closed(),
            Some(Err(at)) => past(*at, IDLE, now()),
            None => true,
        };
        if ask {
            let opened = opened(&lent, flow).await;
            let refused = opened.as_ref().err().map(|w| format!("{flow} {w}\n"));
            let _ = send.write_all(refused.unwrap_or_default().as_bytes()).await;
            flows.insert(flow, opened.map_err(|_| now()));
        }
        if let Some(Ok(packets)) = flows.get(&flow) {
            let _ = packets.try_send(packet);
        }
    }
    tracing::debug!(target: "door", actor, lend = %lend.name, "udp: the member's connection closed, and its flows with it");
}

/// What a flow is opened from: the door, the dialer, the lend and its port,
/// and the member's connection.
pub(super) type Lending<'a> = (&'a Door, &'a str, &'a Lend, NonZeroU16, &'a Connection);

/// A flow's first packet, let in by [`admitted`]: a socket on 127.0.0.1:0
/// connected to the lent port alone, in a task of its own; or why not, in
/// a person's words.
async fn opened(lent: &Lending<'_>, flow: u32) -> Result<mpsc::Sender<Vec<u8>>, String> {
    let (_, actor, lend, port, conn) = *lent;
    let held = admitted(lent, &format!("udp: flow {flow}")).await?;
    let unreached = |e: std::io::Error| format!("{} cannot be reached here: {e}", lend.name);
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await;
    let socket = socket.map_err(unreached)?;
    let to = (Ipv4Addr::LOCALHOST, port.get());
    socket.connect(to).await.map_err(unreached)?;
    tracing::debug!(target: "door", actor, lend = %lend.name, flow, port, counted = held.is_some(), "udp: a flow opened, its socket connected to the lent port");
    let (packets, sent) = mpsc::channel(256);
    tokio::spawn(flowing(socket, (conn.clone(), flow), sent, held));
    Ok(packets)
}

/// One use of a raw lend, `what` in the traces, let in by the dialer's
/// standing now and the lend's limit, which counts people, not flows or
/// connections: that person's use, held (none for one not counted); or why
/// not, in a person's words.
pub(super) async fn admitted(lent: &Lending<'_>, what: &str) -> Result<Option<InFlight>, String> {
    let (door, actor, lend, _, conn) = *lent;
    let Some((_, _, whose)) = PersonKey::resolved(door, actor) else {
        tracing::debug!(target: "door", actor, what, "raw: a use refused — the dialer stands in no group held here now");
        return Err(LENDS_NOTHING.into());
    };
    let at = crate::network::guess(Via(conn.clone()).observe().and_then(|o| o.addr));
    match door.using.ask(lend, whose.as_str(), at).await {
        Some((Verdict::Full { using, limit }, _)) => {
            tracing::info!(target: "door", person = whose.as_str(), lend = %lend.name, what, using, limit,
                "raw: a use refused — as many people as the lend's limit are using it");
            Err(busy_words(lend, using, limit))
        }
        Some((_, held)) => Ok(held),
        None => Err(super::UNCOUNTED.into()),
    }
}

/// One flow: its packets to the lent port, and the port's answers to the
/// member, until idle past [`IDLE`] or its connection closes; `held`, its
/// person's use of the lend, ends with it.
async fn flowing(
    socket: UdpSocket,
    (conn, flow): (Connection, u32),
    mut packets: mpsc::Receiver<Vec<u8>>,
    held: Option<InFlight>,
) {
    let (mut buf, mut number, mut last) = (vec![0; LARGEST], 0, now());
    let mut every = tokio::time::interval(SWEEP);
    loop {
        let sent = tokio::select! {
            got = packets.recv() => match got {
                Some(packet) => socket.send(&packet).await.map(drop),
                None => break,
            },
            got = socket.recv(&mut buf) => got.map(|n| carry(&conn, flow, &mut number, &buf[..n])),
            _ = every.tick() => if past(last, IDLE, now()) { break } else { continue },
        };
        last = now();
        if let Err(e) = sent {
            tracing::debug!(target: "door", flow, error = %e, "udp: a packet not carried — the lent port refused it");
        }
    }
    tracing::debug!(target: "door", flow, idle = past(last, IDLE, now()), counted = held.is_some(), "udp: a flow closed, idle past its expiry or its connection gone");
}

/// Why a member's computer could not open a raw lend.
#[derive(Debug, thiserror::Error)]
pub enum Unopened {
    #[error("the computer that lends it does not know this one stands in its group")]
    NotAdmitted,
    #[error("the computer that lends it was not reached: {0}")]
    Unreached(String),
    /// The lender's own words.
    #[error("{0}")]
    Refused(String),
}

/// A member's use of a raw lend, held until dropped: the lender's words
/// for each flow or connection it refused, and the task carrying every
/// program's packets or connections.
pub type Carried = (mpsc::UnboundedReceiver<String>, AbortOnDropHandle<()>);

/// The opening's stream, kept open: the lender's words come on it.
pub(super) type Opened = (SendStream, Lines<BufReader<RecvStream>>);
pub(super) type Tell = mpsc::UnboundedSender<String>;

/// Open the UDP lend `lend` of the computer at `to`, its flows those of
/// the programs on 127.0.0.1 sending to `local`, each by its port: the
/// member's half of the door.
pub async fn carried(
    endpoint: &Endpoint,
    to: EndpointAddr,
    lend: &LendName,
    local: UdpSocket,
) -> Result<Carried, Unopened> {
    let (conn, opened) = opening(endpoint, to, Raw::Udp, lend).await?;
    let (tell, told) = mpsc::unbounded_channel();
    let task = tokio::spawn(member(conn, local, opened, tell));
    Ok((told, AbortOnDropHandle::new(task)))
}

/// The opening of `lend` at `to`, as `raw`: the member's one connection
/// for it, and the stream the lender's words come on.
pub(super) async fn opening(
    endpoint: &Endpoint,
    to: EndpointAddr,
    raw: Raw,
    lend: &LendName,
) -> Result<(Connection, Opened), Unopened> {
    let within = Some(crate::dial::DIAL_WITHIN);
    let dialled = crate::iroh::dialled(endpoint, to, Alpn::Port.bytes(), within).await;
    let conn = dialled.map_err(|e| Unopened::Unreached(e.to_string()))?;
    let opened = async {
        let (mut send, recv) = conn.open_bi().await?;
        send.write_all(format!("{} {lend}\n", raw.said()).as_bytes())
            .await?;
        let mut lines = BufReader::new(recv).lines();
        Ok::<_, std::io::Error>((lines.next_line().await?, (send, lines)))
    };
    let opened = match opened.await {
        Ok((Some(ok), opened)) if ok == "ok" => Ok(opened),
        Ok((Some(words), _)) => Err(Unopened::Refused(words)),
        _ if closed_unadmitted(&conn).await => Err(Unopened::NotAdmitted),
        got => Err(Unopened::Unreached(format!("{:?}", got.err()))),
    };
    tracing::debug!(target: "door", lend = %lend, refused = ?opened.as_ref().err(), "raw: the lend opened, or refused");
    Ok((conn, opened?))
}

/// The member's loop: `local`'s packets to the lender, each flow's answers
/// to its port on 127.0.0.1, and the lender's words of a flow refused.
async fn member(conn: Connection, local: UdpSocket, (_send, mut lines): Opened, tell: Tell) {
    let (mut whole, mut number, mut buf) = (Whole::new(), 0, vec![0; LARGEST]);
    loop {
        tokio::select! {
            Ok((n, from)) = local.recv_from(&mut buf) => {
                carry(&conn, u32::from(from.port()), &mut number, &buf[..n]);
            }
            got = conn.read_datagram() => {
                let Ok(d) = got else { break };
                let Some((flow, packet)) = take(&mut whole, d, now()) else { continue };
                let to = (Ipv4Addr::LOCALHOST, u16::try_from(flow).unwrap_or_default());
                if let Err(e) = local.send_to(&packet, to).await {
                    tracing::debug!(target: "door", flow, error = %e, "udp: an answer not passed to its program");
                }
            }
            line = lines.next_line() => {
                let Ok(Some(line)) = line else { break };
                tracing::debug!(target: "door", line, "udp: the lender refused a flow");
                let _ = tell.send(line.split_once(' ').map_or(line.clone(), |(_, words)| words.into()));
            }
        }
    }
    tracing::debug!(target: "door", "udp: the lender closed the lend");
}

#[cfg(test)]
mod tests;
