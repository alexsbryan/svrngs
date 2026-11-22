// SPDX-License-Identifier: AGPL-3.0-or-later
//! A share's port, written to only through its check (svrn-docs
//! SHARE_BINDING §5.2, §5.7). The transport opens an [`Unchecked`]
//! connection, which nothing but [`bound`] opens, at the address its
//! approval names: it reads the port's sockets and what took the
//! connection, and [`eudoxia_community::binding::check`] decides, before any byte is
//! sent. A [`Bound`] is what the door's HTTP, its WebSocket upgrade, its raw
//! TCP and the lender's credentials (share.rs `tried`, `sign_in_readers`) are
//! written on; a UDP share's datagrams go only through a [`Flow`], checked
//! before and after each, and each answer before it is carried. A program
//! not approved seen where the door reaches a UDP share holds it until its
//! lender approves again ([`decided`]); a TCP connection is judged alone. A
//! path that dials unchecked doesn't build: each dial is a disallowed
//! method, allowed only where door/tests.rs lists it.

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use tokio::net::UdpSocket;

use crate::door::Using;
use crate::iroh::{Duplex, Net};
use crate::lend::{Hold, Lend, LendName, Raw};
use crate::uid::{Conn, Unread};
use eudoxia_community::binding::{self, Held, Read, Took};

/// How long a share's port has to take a connection; once connected, a
/// body takes as long as it takes, as upstream's `forward_stream` bounded
/// only its connect.
pub(crate) const CONNECT_WITHIN: Duration = Duration::from_secs(10);

/// A connection to a share's port that nothing has checked: what
/// [`Net::service`] answers, opened by [`bound`] alone.
pub struct Unchecked(Box<dyn Duplex>, Conn);

impl Unchecked {
    /// `stream`, connected at `conn`'s two ends, unchecked.
    pub(crate) fn new(stream: Box<dyn Duplex>, conn: Conn) -> Self {
        Self(stream, conn)
    }
}

/// A connection to a share's port that its check carried: the one thing a
/// request, a WebSocket, a raw TCP connection or a credential is written on.
pub struct Bound(Box<dyn Duplex>);

impl Bound {
    /// The connection, to write on.
    pub(crate) fn stream(self) -> Box<dyn Duplex> {
        self.0
    }
}

/// Why nothing was written to a share's port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unsent {
    /// Nothing took the connection, or nothing is on the port.
    Away,
    /// Something answers that the share doesn't carry to.
    Held(Hold),
}

/// What a share's port is judged with: the transport that reads it, the
/// folder its lend is kept in, where a hold is kept, and the lend.
pub(crate) type Judged<'a> = (&'a dyn Net, &'a Path, &'a Lend);

/// `lend`'s TCP port, connected within [`CONNECT_WITHIN`] at the address
/// its approval names and checked: a [`Bound`] when what is on it and what
/// took the connection are what its lender approved, else why not, at warn.
pub(crate) async fn bound((net, dir, lend): Judged<'_>) -> Result<Bound, Unsent> {
    let at = reached(lend)?;
    let taken = tokio::time::timeout(CONNECT_WITHIN, net.service(at)).await;
    // Past it, `Elapsed` is read as `std::io::ErrorKind::TimedOut` (tokio's
    // `From<Elapsed> for std::io::Error`).
    let taken = taken.unwrap_or_else(|e| Err(e.into()));
    let Unchecked(stream, conn) = taken.map_err(away(lend))?;
    let read = net.read_port(Raw::Tcp, at.port(), Some(conn)).await;
    decided(dir, lend, read).map(|()| Bound(stream))
}

/// [`bound`] with this computer's own readers, `lend` kept in `dir`: for a
/// command with no node's transport, as `svrngs share --readers` is.
pub async fn here(dir: &Path, lend: &Lend) -> Result<Bound, Unsent> {
    let at = reached(lend)?;
    let Unchecked(stream, conn) = dial(at).await.map_err(away(lend))?;
    let port = at.port();
    let read = tokio::task::spawn_blocking(move || crate::uid::port(Raw::Tcp, port, Some(conn)));
    let stopped = Err(Unread::Failed("the read stopped"));
    let read = read.await.unwrap_or(stopped);
    decided(dir, lend, read).map(|()| Bound(stream))
}

/// Where `lend`'s TCP service is dialled: its port, at the address its
/// approval names. One of another kind, or approved for nothing, is
/// refused, and nothing dialled.
fn reached(lend: &Lend) -> Result<SocketAddr, Unsent> {
    let port = lend.port().filter(|_| lend.raw() == Raw::Tcp);
    let approved = lend.terms.approved.as_ref();
    let at = match (port, approved) {
        (None, _) => Err(Unsent::Away),
        (_, None) => Err(Unsent::Held(Hold::Unapproved)),
        (Some(port), Some(a)) => Ok(SocketAddr::new(a.at, port.get())),
    };
    tracing::debug!(target: "door", lend = %lend.name, ?at, "door: where a lend's TCP port is dialled, or why it isn't");
    at
}

/// What a connection to `lend`'s port that nothing took is, traced.
fn away(lend: &Lend) -> impl FnOnce(std::io::Error) -> Unsent + '_ {
    move |e| {
        tracing::warn!(target: "door", lend = %lend.name, error = %e, within = ?CONNECT_WITHIN, "door: the lend's service took no connection, refused or not in time");
        Unsent::Away
    }
}

/// A share's port as `read`, judged against `lend`'s approval ([`binding::check`]),
/// each branch traced; an unread port, or one with a socket whose holder
/// can't be read, held [`Hold::Unjudged`]. A program not approved seen
/// there holds a UDP share in `dir` too, until its lender approves again
/// ([`hold`]); a TCP connection is judged alone, so it holds only itself.
pub(crate) fn decided(
    dir: &Path,
    lend: &Lend,
    read: Result<(Read, Took), Unread>,
) -> Result<(), Unsent> {
    let approved = lend.terms.approved.as_ref();
    let checked = match &read {
        Ok((now, took)) => binding::check(approved, now, took).map_err(|held| match held {
            Held::Unapproved => Unsent::Held(Hold::Unapproved),
            Held::Another => Unsent::Held(Hold::Another),
            Held::Unjudged => Unsent::Held(Hold::Unjudged),
            Held::Empty => Unsent::Away,
        }),
        Err(_) => Err(Unsent::Held(Hold::Unjudged)),
    };
    tracing::debug!(target: "door", lend = %lend.name, ?approved, ?read, ?checked, "door: what is on the lend's port, carried when it is what its lender approved, else held");
    let seen = checked == Err(Unsent::Held(Hold::Another)) && lend.raw() == Raw::Udp;
    if seen && approved.is_some_and(|a| !a.held) {
        hold(dir, lend);
    }
    checked
}

/// `lend` held in its file until its lender approves again, so neither the
/// program going nor svrngs starting again opens it (SHARE_BINDING_REVIEWS
/// §6 #3): only while its approval is still the one judged, so an approval
/// made since is never held for what it didn't see ([`mutated`]); past
/// that wait, the check that saw it has refused its datagram all the same.
fn hold(dir: &Path, lend: &Lend) {
    let kept = mutated(dir, &lend.name, |now| {
        let judged = now.terms.approved == lend.terms.approved;
        if let Some(approved) = now.terms.approved.as_mut().filter(|_| judged) {
            approved.held = true;
        }
    });
    tracing::warn!(target: "door", lend = %lend.name, ?kept, "door: held until its lender approves again — a program not approved was seen where the door reaches it");
}

/// `edit` made to the lend `name`'s file ([`crate::lend::mutate`]): another
/// change holding its lock is waited on a moment, up to 20 ms.
fn mutated(
    dir: &Path,
    name: &LendName,
    mut edit: impl FnMut(&mut Lend),
) -> Result<bool, crate::lend::LendError> {
    let mut kept = crate::lend::mutate(dir, name, &mut edit);
    for _ in 0..20 {
        if !matches!(kept, Err(crate::lend::LendError::Busy { .. })) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
        kept = crate::lend::mutate(dir, name, &mut edit);
    }
    kept
}

/// One UDP flow's socket on 127.0.0.1, connected to a share's port alone
/// at the address its approval names: written to only by [`Flow::send`],
/// checked before and after, and its answers given only by [`Flow::recv`].
pub(crate) struct Flow {
    /// Its own socket among its lend's open flows, which each check names
    /// ([`Took::Flow`]): a server's socket for one is answering it. First,
    /// as a struct's fields drop in the order they are declared: so it
    /// leaves the flows before `socket` closes, and a flow that takes the
    /// socket's port after it is never taken out with it.
    open: crate::door::Flowing,
    socket: UdpSocket,
    port: u16,
    /// An answer read and not yet given, its length in `buf`: kept while it
    /// is checked, so a check a `select!` drops loses none
    /// (SHARE_BINDING_REVIEWS §6 #4).
    answer: Option<usize>,
    buf: Vec<u8>,
}

impl Flow {
    /// `socket`, on 127.0.0.1:0, connected to `at` alone, open among the
    /// flows `using` keeps of the lend `name` while it is.
    #[expect(clippy::disallowed_methods, reason = "dial: a checked flow")]
    pub(crate) async fn open(
        socket: UdpSocket,
        (at, using, name): (SocketAddr, &crate::door::Using, &crate::lend::LendName),
    ) -> std::io::Result<Self> {
        socket.connect(at).await?;
        let open = using.flow(name, socket.local_addr()?);
        let (port, buf) = (at.port(), vec![0; 65_535]);
        Ok(Self {
            socket,
            port,
            answer: None,
            buf,
            open,
        })
    }

    /// What the share's port answers this flow, given once its sockets are
    /// checked as for a datagram sent (SHARE_BINDING §5.2): a check failing
    /// drops the answer and holds the flow; one that can't be read is away.
    /// Cancel-safe: an answer read while a check was dropped is checked and
    /// given at the next call.
    pub(crate) async fn recv(&mut self, judged: Judged<'_>) -> Result<&[u8], Unsent> {
        if self.answer.is_none() {
            let got = self.socket.recv(&mut self.buf).await.map_err(|e| {
                tracing::debug!(target: "door", port = self.port, error = %e, "udp: an answer not read from the lent port");
                Unsent::Away
            })?;
            self.answer = Some(got);
        }
        let checked = self.check(judged).await;
        let got = self.answer.take().unwrap_or_default();
        checked.map(|()| &self.buf[..got])
    }

    /// `first`, and each datagram queued behind it in `more`, sent to the
    /// share's port, its sockets read and checked against `lend`'s
    /// approval just before the first and just after each batch
    /// (SHARE_BINDING §5.2): a batch queued while one went out is sent on
    /// the check after it, so each datagram has a check just before it and
    /// one just after. A check failing sends nothing more, and a datagram
    /// the port refuses is traced.
    pub(crate) async fn send(
        &self,
        judged: Judged<'_>,
        first: Vec<u8>,
        more: &mut tokio::sync::mpsc::Receiver<Vec<u8>>,
    ) -> Result<(), Unsent> {
        self.check(judged).await?;
        let mut batch = vec![first];
        while !batch.is_empty() {
            for datagram in batch.drain(..) {
                if let Err(e) = self.socket.send(&datagram).await {
                    tracing::debug!(target: "door", port = self.port, error = %e, "udp: a packet not carried — the lent port refused it");
                }
            }
            self.check(judged).await?;
            batch.extend(std::iter::from_fn(|| more.try_recv().ok()));
        }
        Ok(())
    }

    /// The share's port read and checked against `lend`'s approval now.
    pub(crate) async fn check(&self, (net, dir, lend): Judged<'_>) -> Result<(), Unsent> {
        decided(dir, lend, udp_port(net, self.port, self.open.of()).await)
    }
}

/// A UDP share's `port` read as each of its checks judges it, a flow's and
/// its lender's list's alike: with the flows `using` names for `lend` by
/// this read, each by its own socket's address, so a server's socket
/// answering one is known ([`Took::Flow`]).
pub(crate) async fn udp_port(
    net: &dyn Net,
    port: u16,
    of: (&Using, &LendName),
) -> Result<(Read, Took), Unread> {
    let read = net.read_port(Raw::Udp, port, None).await;
    let now = read.as_ref().ok().map(|(now, _)| &now.sockets);
    let to = |s: &binding::Socket| match s.peer {
        binding::Peer::To(at) => Some(at),
        binding::Peer::Unread | binding::Peer::Unconnected => None,
    };
    let peers = now.map(|sockets| sockets.iter().filter_map(to).collect());
    let (named, new) = of.0.flows(of.1, peers).await;
    read.map(|(now, _)| (now, Took::Flow { named, new }))
}

/// A TCP connection to this computer's `at`, unchecked: the product's
/// [`Net::service`].
#[expect(clippy::disallowed_methods, reason = "dial: checked by bound")]
pub(crate) async fn dial(at: SocketAddr) -> std::io::Result<Unchecked> {
    let stream = tokio::net::TcpStream::connect(at).await?;
    let conn = crate::iroh::ends(&stream)?;
    Ok(Unchecked::new(Box::new(stream), conn))
}
