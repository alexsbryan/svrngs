// SPDX-License-Identifier: AGPL-3.0-or-later
//! The door's TCP half (give-tcp; svrn-docs rings/current/GIVE.md §6): a
//! lend of a TCP port ([`Raw::Tcp`]), opened as the UDP half's is
//! ([`super::udp::serve`]), each connection to it one QUIC bi-stream on the
//! one connection a member's computer keeps for the lend.
//!
//! ```text
//!   a program ─TCP─▶ the member's listener ═a stream each═▶ the lender ─TCP─▶ 127.0.0.1:<port>
//! ```
//!
//! A stream has flow control of its own, so one connection stalled holds
//! back no other, and the record's sync is a connection of its own
//! ([`crate::iroh::Alpn::Member`]). The member writes a newline first on
//! each stream, since its far side sees a stream only once it is written
//! to. The lender lets it in by [`admitted`] and answers `ok`, or its
//! refusal in a person's words, and the two are copied each way until both
//! have closed: a side's close is its stream's finish, so a half-close
//! crosses.

use std::net::Ipv4Addr;

use iroh::endpoint::{Connection, RecvStream, SendStream};
use n0_future::task::AbortOnDropHandle;
use tokio::io::{copy_bidirectional, join, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::{sync::mpsc, task::JoinSet};

use super::udp::{admitted, opening, Lending, Opened, Tell};
pub use super::udp::{Carried, Unopened};
use crate::iroh::{Endpoint, EndpointAddr};
use crate::lend::{LendName, Raw};

/// The lender's half, the lend opened: each stream the member opens, one
/// connection to the lent port, in a task of its own.
pub(super) async fn serve((door, actor, lend, port, conn): Lending<'_>) {
    while let Ok((send, recv)) = conn.accept_bi().await {
        let lent = (door.clone(), actor.to_owned(), lend.clone(), conn.clone());
        tokio::spawn(async move {
            let (door, actor, lend, conn) = &lent;
            connection((door, actor.as_str(), lend, port, conn), send, recv).await
        });
    }
    tracing::debug!(target: "door", actor, lend = %lend.name, "tcp: the member's connection closed, and its streams with it");
}

/// One program's connection, let in by [`admitted`]: a connection to the
/// lent port, copied each way with the stream until both have closed; or
/// why not, in a person's words on the stream.
async fn connection(lent: Lending<'_>, mut send: SendStream, mut recv: RecvStream) {
    let (_, actor, lend, port, _) = lent;
    let _ = recv.read_exact(&mut [0; 1]).await;
    let unreached = |e| format!("{} cannot be reached here: {e}", lend.name);
    let opened = match admitted(&lent, "tcp: a connection").await {
        Ok(held) => (TcpStream::connect((Ipv4Addr::LOCALHOST, port.get())).await)
            .map(|to| (to, held))
            .map_err(unreached),
        Err(words) => Err(words),
    };
    let (mut to, held) = match opened {
        Ok(opened) => opened,
        Err(words) => {
            tracing::debug!(target: "door", actor, lend = %lend.name, words, "tcp: a connection refused");
            let _ = send.write_all(format!("{words}\n").as_bytes()).await;
            return drop(send.finish());
        }
    };
    let _ = send.write_all(b"ok\n").await;
    tracing::debug!(target: "door", actor, lend = %lend.name, port, counted = held.is_some(), "tcp: a connection opened to the lent port");
    let copied = copy_bidirectional(&mut to, &mut join(recv, send)).await;
    tracing::debug!(target: "door", actor, lend = %lend.name, ?copied, "tcp: a connection closed, both its sides or by a failure");
}

/// Open the TCP lend `lend` of the computer at `to`, each connection a
/// program makes to `local` carried on a stream of its own: the member's
/// half of the door.
pub async fn carried(
    endpoint: &Endpoint,
    to: EndpointAddr,
    lend: &LendName,
    local: TcpListener,
) -> Result<Carried, Unopened> {
    let (conn, opened) = opening(endpoint, to, Raw::Tcp, lend).await?;
    let (tell, told) = mpsc::unbounded_channel();
    let task = tokio::spawn(member(conn, local, opened, tell));
    Ok((told, AbortOnDropHandle::new(task)))
}

/// The member's loop: each program's connection to `local`, carried until
/// the lender closes the lend, every one ending with it.
async fn member(conn: Connection, local: TcpListener, _opened: Opened, tell: Tell) {
    let mut each = JoinSet::new();
    loop {
        tokio::select! {
            Ok((program, _)) = local.accept() => drop(each.spawn(carry(conn.clone(), program, tell.clone()))),
            Some(_) = each.join_next() => {}
            _ = conn.closed() => break,
        }
    }
    tracing::debug!(target: "door", "tcp: the lender closed the lend");
}

/// A program's connection on a stream of its own: the lender's `ok`, then
/// its bytes each way until both sides have closed; or the lender's words,
/// told, and the program's connection closed.
async fn carry(conn: Connection, mut program: TcpStream, tell: Tell) {
    let opened = async {
        let (mut send, recv) = conn.open_bi().await?;
        send.write_all(b"\n").await?;
        let (mut recv, mut said) = (BufReader::new(recv), String::new());
        recv.read_line(&mut said).await?;
        Ok::<_, std::io::Error>((said, join(recv, send)))
    };
    let copied = match opened.await {
        Ok((said, mut stream)) if said == "ok\n" => {
            copy_bidirectional(&mut program, &mut stream).await
        }
        Ok((words, _)) => {
            tracing::debug!(target: "door", words, "tcp: the lender refused a connection");
            return drop(tell.send(words.trim().into()));
        }
        Err(e) => Err(e),
    };
    tracing::debug!(target: "door", ?copied, "tcp: a program's connection closed, both its sides or by a failure");
}
