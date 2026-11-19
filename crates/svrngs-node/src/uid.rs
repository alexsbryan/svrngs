// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whose a socket on this computer is (svrn-docs rings/current/LEND_OWNER.md
//! §2): the uid the kernel records for the socket at one end of a TCP
//! connection, or for the socket a UDP datagram to a port reaches, read from
//! the table of sockets any account may read. The account, never the
//! program: another account's program can't be read, and a program's path
//! and pid change at each restart where its uid doesn't.
//!
//! - `/proc/net/tcp`, `tcp6`, `udp` and `udp6`: each row's uid column.
//! - The `net.inet.tcp.pcblist_n` and `net.inet.udp.pcblist_n` sysctls, which
//!   `netstat` reads: each socket's records, walked by their own length and
//!   kind, its addresses in its `xinpcb_n` and its uid in its `xsocket_n`.
//!   The kernel lists every process's sockets there only to a program whose
//!   code signature names a dotted identifier, `sh.svrnme.svrngs`'s shape;
//!   to one signed as the linker signs it, `<name>-<hash>`, its own sockets
//!   alone, so another program's is not found and could not be judged
//!   (measured 2026-11-18: 48 bytes, a header and its trailer, where a
//!   dotted identifier read 92,448).
//! - Elsewhere no reader: [`Owner::NoReader`].
//!
//! An answer is taken only from sockets whose addresses and ports are the
//! connection's; none, or sockets of two accounts, could not be judged. The
//! table is parsed in safe Rust, every read bounds-checked; [`sysctl`]'s call
//! is the one `unsafe` in svrngs (DESIGN §5).

use std::net::{IpAddr, SocketAddr};

use crate::lend::Raw;

/// A connection on this computer, by its two ends as one of its sockets
/// sees them: its own address, and the address it is connected to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conn {
    pub local: SocketAddr,
    pub peer: SocketAddr,
}

/// Which end of a [`Conn`] is asked about: the socket at its `local`
/// address, or the one at its `peer`'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Local,
    Peer,
}

/// Whose a socket is, as this computer's table says now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// The uid of the account that holds it.
    Uid(u32),
    /// The table was asked and gave no one answer: why.
    Unjudged(&'static str),
    /// This system has no reader, so nothing was asked.
    NoReader,
}

/// Whether this build reads whose a socket is: `false` where every read
/// answers [`Owner::NoReader`].
pub const READS: bool = cfg!(any(target_os = "linux", target_os = "macos"));

/// The uid holding the socket at `side` of `conn`, a TCP connection or a UDP
/// flow as `raw` says, read from the table now and never kept.
pub fn owner(raw: Raw, conn: Conn, side: Side) -> Owner {
    let owner = match table(raw) {
        Ok(Some(rows)) => answer(&rows, raw, conn, side),
        Ok(None) => Owner::NoReader,
        Err(why) => Owner::Unjudged(why),
    };
    tracing::debug!(target: "uid", ?raw, ?conn, ?side, ?owner, "uid: whose the socket at this end is, read from this computer's table");
    owner
}

/// One socket as a table lists it: its address, the address it is connected
/// to (unspecified, port 0, for none), and its uid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Row {
    local: SocketAddr,
    foreign: SocketAddr,
    uid: u32,
}

/// The one uid among `rows` holding the socket at `side` of `conn`: at its
/// address, connected to the other end's. For TCP the socket whose two
/// addresses are those; for UDP also a socket bound to every address or
/// connected to none, since a datagram reaches it too, so every one that
/// could take it is counted. None, or two accounts among them, could not be
/// judged.
fn answer(rows: &[Row], raw: Raw, conn: Conn, side: Side) -> Owner {
    let (at, to) = match side {
        Side::Local => (conn.local, conn.peer),
        Side::Peer => (conn.peer, conn.local),
    };
    let same = |a: SocketAddr, b: SocketAddr| {
        a.port() == b.port() && a.ip().to_canonical() == b.ip().to_canonical()
    };
    let any = |a: SocketAddr| a.ip().is_unspecified();
    let holds = |r: &&Row| match raw {
        Raw::Tcp => same(r.local, at) && same(r.foreign, to),
        Raw::Udp => {
            let bound = same(r.local, at) || (any(r.local) && r.local.port() == at.port());
            bound && (same(r.foreign, to) || (any(r.foreign) && r.foreign.port() == 0))
        }
    };
    let mut uids = rows.iter().filter(holds).map(|r| r.uid);
    match uids.next() {
        None => Owner::Unjudged("no socket at that end in this computer's table"),
        Some(uid) if uids.all(|u| u == uid) => Owner::Uid(uid),
        Some(_) => Owner::Unjudged("sockets of more than one account at that end"),
    }
}

/// `raw`'s table as `/proc/net` lists it, IPv4's and IPv6's.
#[cfg(target_os = "linux")]
fn table(raw: Raw) -> Result<Option<Vec<Row>>, &'static str> {
    let files = match raw {
        Raw::Tcp => ["/proc/net/tcp", "/proc/net/tcp6"],
        Raw::Udp => ["/proc/net/udp", "/proc/net/udp6"],
    };
    let mut rows = Vec::new();
    for file in files {
        match std::fs::read_to_string(file) {
            Ok(text) => rows.extend(proc_net::rows(&text)?),
            // A computer with IPv6 turned off lists no `tcp6`.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && file.ends_with('6') => {
                tracing::debug!(target: "uid", file, "uid: no IPv6 table, so IPv4's alone");
            }
            Err(e) => {
                tracing::debug!(target: "uid", file, error = %e, "uid: the table cannot be read");
                return Err("this computer's table of sockets can't be read");
            }
        }
    }
    Ok(Some(rows))
}

/// `raw`'s table as its `pcblist_n` sysctl gives it.
#[cfg(target_os = "macos")]
fn table(raw: Raw) -> Result<Option<Vec<Row>>, &'static str> {
    let name = match raw {
        Raw::Tcp => c"net.inet.tcp.pcblist_n",
        Raw::Udp => c"net.inet.udp.pcblist_n",
    };
    let table = sysctl(name).map_err(|e| {
        tracing::debug!(target: "uid", ?name, error = %e, "uid: the table cannot be read");
        "this computer's table of sockets can't be read"
    })?;
    pcblist::rows(&table).map(Some)
}

/// No reader here.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn table(_: Raw) -> Result<Option<Vec<Row>>, &'static str> {
    Ok(None)
}

/// A `/proc/net` table read: compiled where it is read, and for the tests
/// that read a captured one anywhere.
#[cfg(any(target_os = "linux", test))]
mod proc_net {
    use super::*;

    /// The rows of a `/proc/net` table past its header: each line's local and
    /// remote address and its uid, the eighth field. A TCP socket in TIME_WAIT
    /// (state `06`) is passed over: it lists uid 0 whoever held it, and a new
    /// connection on loopback may take its four numbers. A line not read so
    /// refuses the table.
    pub(super) fn rows(text: &str) -> Result<Vec<Row>, &'static str> {
        let row = |line: &str| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let (local, foreign) = (addr(fields.get(1)?)?, addr(fields.get(2)?)?);
            let waiting = *fields.get(3)? == "06";
            let uid = fields.get(7)?.parse().ok()?;
            Some((!waiting).then_some(Row {
                local,
                foreign,
                uid,
            }))
        };
        let mut rows = Vec::new();
        for line in text.lines().skip(1).filter(|l| !l.trim().is_empty()) {
            rows.extend(
                row(line).ok_or("a line of this computer's table of sockets can't be read")?,
            );
        }
        Ok(rows)
    }

    /// `0100007F:0277`: the address as 32-bit words, each written as this
    /// computer holds the number its bytes make, then the port, all hex.
    fn addr(field: &str) -> Option<SocketAddr> {
        let (words, port) = field.split_once(':')?;
        let word = |w: &[u8]| u32::from_str_radix(std::str::from_utf8(w).ok()?, 16).ok();
        let words: Vec<u32> = words
            .as_bytes()
            .chunks(8)
            .map(word)
            .collect::<Option<_>>()?;
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_ne_bytes()).collect();
        let ip = match <[u8; 4]>::try_from(&bytes[..]) {
            Ok(v4) => IpAddr::from(v4),
            Err(_) => IpAddr::from(<[u8; 16]>::try_from(&bytes[..]).ok()?),
        };
        Some(SocketAddr::new(ip, u16::from_str_radix(port, 16).ok()?))
    }
}

/// A `pcblist_n` table read: compiled where it is read, and for the tests
/// that read a captured one anywhere.
#[cfg(any(target_os = "macos", test))]
mod pcblist {
    use super::*;

    /// A `pcblist_n` record's kind: a socket's addresses (`XSO_INPCB`), and the
    /// socket itself (`XSO_SOCKET`).
    pub(super) const XSO_INPCB: u32 = 0x10;
    const XSO_SOCKET: u32 = 0x01;
    /// The length of each record read, as this reader was built for it (the
    /// capture in `fixtures/pcblist_n.tcp.hex`): a record of either kind of
    /// another length is a layout it does not know, refused.
    const INPCB_LEN: usize = 104;
    const SOCKET_LEN: usize = 104;
    /// Within an `xinpcb_n`: the foreign and local ports, the IPv4/IPv6 flag,
    /// and the foreign and local addresses, an IPv4 one in each's last 4 bytes.
    const INP_FPORT: usize = 16;
    const INP_LPORT: usize = 18;
    const INP_VFLAG: usize = 44;
    const INP_FADDR: usize = 48;
    const INP_LADDR: usize = 64;
    /// Within an `xsocket_n`: `so_uid`.
    const SO_UID: usize = 64;

    /// The sockets in a `pcblist_n` table: a header, then each socket's records,
    /// each led by its length and kind and padded to 8 bytes, until one no
    /// longer than the header, its trailer, ends them. A socket's `xinpcb_n`
    /// comes before its `xsocket_n`. A record cut off, shorter than what is read
    /// of it, or of a length this reader wasn't built for refuses the table.
    pub(super) fn rows(table: &[u8]) -> Result<Vec<Row>, &'static str> {
        const CUT: &str = "this computer's table of sockets is laid out as svrngs doesn't know";
        let head = word(table, 0).ok_or(CUT)? as usize;
        let (mut at, mut rows, mut addresses) = (head.next_multiple_of(8), Vec::new(), None);
        loop {
            let len = word(table, at).ok_or(CUT)? as usize;
            let kind = word(table, at + 4).ok_or(CUT)?;
            if len <= head {
                return Ok(rows);
            }
            let record = table.get(at..at + len).ok_or(CUT)?;
            match kind {
                XSO_INPCB if len == INPCB_LEN => addresses = Some(inpcb(record).ok_or(CUT)?),
                XSO_SOCKET if len == SOCKET_LEN => {
                    let (local, foreign) = addresses.take().ok_or(CUT)?;
                    let uid = word(record, SO_UID).ok_or(CUT)?;
                    rows.push(Row {
                        local,
                        foreign,
                        uid,
                    });
                }
                XSO_INPCB | XSO_SOCKET => return Err(CUT),
                _ => {}
            }
            at += len.next_multiple_of(8);
        }
    }

    /// The 32-bit word at `at` in `bytes`, in this computer's byte order.
    pub(super) fn word(bytes: &[u8], at: usize) -> Option<u32> {
        Some(u32::from_ne_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    }

    /// An `xinpcb_n`'s local and foreign address: IPv4 where its flag says
    /// (`INP_IPV4`, as `netstat` reads it first), else IPv6 (`INP_IPV6`).
    fn inpcb(record: &[u8]) -> Option<(SocketAddr, SocketAddr)> {
        let port = |at: usize| Some(u16::from_be_bytes(record.get(at..at + 2)?.try_into().ok()?));
        let ip = |at: usize, v4: bool| -> Option<IpAddr> {
            Some(match v4 {
                true => IpAddr::from(<[u8; 4]>::try_from(record.get(at + 12..at + 16)?).ok()?),
                false => IpAddr::from(<[u8; 16]>::try_from(record.get(at..at + 16)?).ok()?),
            })
        };
        let v4 = match *record.get(INP_VFLAG)? {
            f if f & 0x1 != 0 => true,
            f if f & 0x2 != 0 => false,
            _ => return None,
        };
        let local = SocketAddr::new(ip(INP_LADDR, v4)?, port(INP_LPORT)?);
        Some((local, SocketAddr::new(ip(INP_FADDR, v4)?, port(INP_FPORT)?)))
    }
}

/// The sysctl `name` read whole: its size asked, then a buffer that long
/// filled, cut to what the kernel wrote. The one `unsafe` in svrngs: every
/// product crate denies `unsafe_code`, and `scripts/lean_check.py` refuses
/// an exemption anywhere but here.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn sysctl(name: &std::ffi::CStr) -> std::io::Result<Vec<u8>> {
    let (mut len, none) = (0, std::ptr::null_mut());
    // SAFETY: `name` ends in a zero byte, being a `CStr`; and with no buffer
    // the kernel writes `len` alone, the length it would fill.
    if unsafe { libc::sysctlbyname(name.as_ptr(), none, &mut len, none, 0) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut table = vec![0u8; len];
    // SAFETY: `name` ends in a zero byte, and the length passed, `len`, is
    // `table`'s, so the kernel writes nothing past its end.
    let buffer = table.as_mut_ptr().cast();
    if unsafe { libc::sysctlbyname(name.as_ptr(), buffer, &mut len, none, 0) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    table.truncate(len);
    Ok(table)
}

#[cfg(test)]
#[path = "uid_tests.rs"]
mod tests;
