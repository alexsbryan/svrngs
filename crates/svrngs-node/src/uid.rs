// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whose a socket on this computer is (svrn-docs rings/current/LEND_OWNER.md
//! §2): the account holding the socket at one end of a TCP connection, or
//! the socket a UDP datagram to a port reaches. The account, never the
//! program: a program's path and pid change at each restart where its
//! account doesn't.
//!
//! - Linux: `/proc/net/tcp`, `tcp6`, `udp` and `udp6`, each row's uid: any
//!   account's, read exactly.
//! - macOS: this account's processes alone, by libproc (this uid's pids,
//!   each one's files, each socket's addresses): found, it is this
//!   account's; not found, [`Owner::NotMine`]. A Mac shows a program signed
//!   as svrngs is no other account's sockets: neither its socket table nor
//!   a `netstat` it runs lists them (svrn-docs LEND_OWNER.md, header).
//! - Elsewhere no reader: [`Owner::NoReader`].
//!
//! An answer comes only from sockets whose addresses and ports are the
//! end's; none, two accounts, or anything not read is [`Owner::Unjudged`].
//! What the kernel answers is parsed in safe Rust, every read
//! bounds-checked; [`kernel`]'s calls are the one `unsafe` in svrngs
//! (DESIGN §5).

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

/// Whose a socket is, as this computer says now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// The uid of the account that holds it.
    Uid(u32),
    /// A Mac's: no process of this account's holds it, a TCP end within
    /// [`ACCEPT_WITHIN`]; whose it is, a Mac doesn't show svrngs.
    NotMine,
    /// It was asked, and no one answer came: why.
    Unjudged(&'static str),
    /// This system has no reader, so nothing was asked.
    NoReader,
}

/// How long a Mac's reader looks again for a TCP end this account's
/// processes don't hold yet, waiting longer each time: a connection is in no
/// process's files until its listener accepts it. A server that accepts at
/// once was found by the first scan 29 times in 30, and the 30th within
/// 3.7 ms (2026-11-18, `python3 -m http.server`); this is some 25 times that,
/// short beside the 10 s a request through the door may take.
pub const ACCEPT_WITHIN: std::time::Duration = std::time::Duration::from_millis(100);

/// The uid holding the socket at `side` of `conn`, a TCP connection or a UDP
/// flow as `raw` says, read now and never kept. A Mac's may wait up to
/// [`ACCEPT_WITHIN`], so an async caller reads it off its runtime.
pub fn owner(raw: Raw, conn: Conn, side: Side) -> Owner {
    let (at, to) = ends(conn, side);
    let owner = read(raw, at, to);
    tracing::debug!(target: "uid", ?raw, %at, %to, ?owner, "uid: whose the socket at this end is");
    owner
}

/// The address of the socket at `side` of `conn`, and the one it is
/// connected to.
fn ends(conn: Conn, side: Side) -> (SocketAddr, SocketAddr) {
    match side {
        Side::Local => (conn.local, conn.peer),
        Side::Peer => (conn.peer, conn.local),
    }
}

/// Whether a socket at `local`, connected to `foreign`, holds the end at `at`
/// connected to `to`: for TCP, its two addresses those exactly; for UDP also
/// one bound to every address or connected to none, since a datagram
/// reaches it too. An IPv4 address written as IPv6 is the IPv4 one.
fn holds(raw: Raw, (local, foreign): Ends, at: SocketAddr, to: SocketAddr) -> bool {
    let same = |a: SocketAddr, b: SocketAddr| {
        a.port() == b.port() && a.ip().to_canonical() == b.ip().to_canonical()
    };
    let any = |a: SocketAddr| a.ip().is_unspecified();
    match raw {
        Raw::Tcp => same(local, at) && same(foreign, to),
        Raw::Udp => {
            let bound = same(local, at) || (any(local) && local.port() == at.port());
            bound && (same(foreign, to) || (any(foreign) && foreign.port() == 0))
        }
    }
}

/// A socket's local address and the one it is connected to.
type Ends = (SocketAddr, SocketAddr);

/// The end at `at`, connected to `to`, as `/proc/net` lists it.
#[cfg(target_os = "linux")]
fn read(raw: Raw, at: SocketAddr, to: SocketAddr) -> Owner {
    let rows = match proc_net::table(raw) {
        Ok(rows) => rows,
        Err(why) => return Owner::Unjudged(why),
    };
    one(rows
        .into_iter()
        .filter(|r| holds(raw, r.0, at, to))
        .map(|r| r.1))
}

/// The one uid among the sockets holding an end: none, or two accounts,
/// could not be judged.
#[cfg(any(target_os = "linux", test))]
fn one(mut uids: impl Iterator<Item = u32>) -> Owner {
    match uids.next() {
        None => Owner::Unjudged("no socket at that end on this computer"),
        Some(uid) if uids.all(|u| u == uid) => Owner::Uid(uid),
        Some(_) => Owner::Unjudged("sockets of more than one account at that end"),
    }
}

/// The uid this process runs as: `/proc/self`'s.
#[cfg(target_os = "linux")]
pub fn mine() -> Result<u32, &'static str> {
    use std::os::unix::fs::MetadataExt as _;
    let me = std::fs::metadata("/proc/self").map(|m| m.uid());
    me.map_err(|_| "this process's own account can't be read")
}

/// The end at `at`, connected to `to`, among this account's processes: a
/// TCP one looked for again until [`ACCEPT_WITHIN`] has passed.
#[cfg(target_os = "macos")]
#[expect(
    clippy::disallowed_methods,
    reason = "the Mac reader's wait for an accept, at the edge"
)]
fn read(raw: Raw, at: SocketAddr, to: SocketAddr) -> Owner {
    let me = match mine() {
        Ok(me) => me,
        Err(why) => return Owner::Unjudged(why),
    };
    let (began, mut wait) = (
        std::time::Instant::now(),
        std::time::Duration::from_millis(1),
    );
    loop {
        match mac::ours(me, raw, at, to) {
            Ok(true) => return Owner::Uid(me),
            Err(e) => {
                tracing::debug!(target: "uid", %at, error = %e, "uid: this account's processes can't be read");
                return Owner::Unjudged("this account's programs can't be read");
            }
            Ok(false) => {}
        }
        let left = ACCEPT_WITHIN.saturating_sub(began.elapsed());
        tracing::debug!(target: "uid", %at, ?raw, ?left, "uid: not among this account's sockets now");
        if raw == Raw::Udp || left.is_zero() {
            return Owner::NotMine;
        }
        std::thread::sleep(wait.min(left));
        wait *= 2;
    }
}

/// The uid this process runs as, by libproc.
#[cfg(target_os = "macos")]
pub fn mine() -> Result<u32, &'static str> {
    let read = i32::try_from(std::process::id()).map_err(std::io::Error::other);
    let bsd = read.and_then(|pid| mac::kernel(mac::Ask::Bsd(pid)));
    let bsd = bsd.inspect_err(|e| tracing::debug!(target: "uid", error = %e, "uid: this process's own record can't be read"));
    (bsd.ok().and_then(|b| record::uid(&b))).ok_or("this process's own account can't be read")
}

/// No reader here.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read(_: Raw, _: SocketAddr, _: SocketAddr) -> Owner {
    Owner::NoReader
}

/// No reader here.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn mine() -> Result<u32, &'static str> {
    Err("this system's accounts can't be read")
}

/// A `/proc/net` table read: compiled where it is read, and for the tests
/// that read a captured one anywhere.
#[cfg(any(target_os = "linux", test))]
mod proc_net {
    use super::*;

    /// `raw`'s table, IPv4's and IPv6's.
    #[cfg(target_os = "linux")]
    pub(super) fn table(raw: Raw) -> Result<Vec<(Ends, u32)>, &'static str> {
        let files = match raw {
            Raw::Tcp => ["/proc/net/tcp", "/proc/net/tcp6"],
            Raw::Udp => ["/proc/net/udp", "/proc/net/udp6"],
        };
        let mut rows = Vec::new();
        for file in files {
            match std::fs::read_to_string(file) {
                Ok(text) => rows.extend(self::rows(&text)?),
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
        Ok(rows)
    }

    /// The rows of a `/proc/net` table past its header: each line's local and
    /// remote address and its uid, the eighth field. A TCP socket in
    /// TIME_WAIT (state `06`) is passed over: it lists uid 0 whoever held it,
    /// and a new connection on loopback may take its four numbers. A line not
    /// read so refuses the table.
    pub(super) fn rows(text: &str) -> Result<Vec<(Ends, u32)>, &'static str> {
        let row = |line: &str| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let ends = (addr(fields.get(1)?)?, addr(fields.get(2)?)?);
            let waiting = *fields.get(3)? == "06";
            Some((!waiting).then_some((ends, fields.get(7)?.parse().ok()?)))
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

/// What a `socket_fdinfo` and a `proc_bsdinfo` hold, as svrngs reads them,
/// their lengths and offsets those of the captures in `fixtures/`: compiled
/// where they are read, and for the tests.
#[cfg(any(target_os = "macos", test))]
mod record {
    use super::*;

    /// A `socket_fdinfo`'s length; its `soi_protocol`; its `soi_kind`; its
    /// `in_sockinfo`'s foreign and local ports, IPv4/IPv6 flag, and foreign
    /// and local addresses, an IPv4 one in the last 4 bytes of its 16.
    pub(super) const SOCKET_LEN: usize = 792;
    const PROTOCOL: usize = 180;
    const KIND: usize = 256;
    const FPORT: usize = 264;
    const LPORT: usize = 268;
    const VFLAG: usize = 288;
    const FADDR: usize = 296;
    const LADDR: usize = 312;
    /// A `proc_bsdinfo`'s length, and its `pbi_uid`.
    pub(super) const BSD_LEN: usize = 136;
    const UID: usize = 20;

    /// An internet socket's protocol and its local and foreign address:
    /// TCP's (`SOCKINFO_TCP`) or UDP's (`SOCKINFO_IN`, protocol 17); `None`
    /// for another socket, or one of neither family.
    pub(super) fn socket(socket: &[u8]) -> Option<(Raw, Ends)> {
        let port = |at: usize| Some(u16::from_be_bytes(socket.get(at..at + 2)?.try_into().ok()?));
        let ip = |at: usize, v4: bool| -> Option<IpAddr> {
            Some(match v4 {
                true => IpAddr::from(<[u8; 4]>::try_from(socket.get(at + 12..at + 16)?).ok()?),
                false => IpAddr::from(<[u8; 16]>::try_from(socket.get(at..at + 16)?).ok()?),
            })
        };
        let raw = match (word(socket, KIND)?, word(socket, PROTOCOL)?) {
            (2, _) => Raw::Tcp,
            (1, 17) => Raw::Udp,
            _ => return None,
        };
        let v4 = match *socket.get(VFLAG)? {
            f if f & 0x1 != 0 => true,
            f if f & 0x2 != 0 => false,
            _ => return None,
        };
        let local = SocketAddr::new(ip(LADDR, v4)?, port(LPORT)?);
        Some((raw, (local, SocketAddr::new(ip(FADDR, v4)?, port(FPORT)?))))
    }

    /// A `proc_bsdinfo`'s uid: none for a record of another length.
    pub(super) fn uid(bsd: &[u8]) -> Option<u32> {
        (bsd.len() == BSD_LEN).then(|| word(bsd, UID)).flatten()
    }

    /// The 32-bit word at `at` in `bytes`, in this computer's byte order.
    pub(super) fn word(bytes: &[u8], at: usize) -> Option<u32> {
        Some(u32::from_ne_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    }
}

/// This Mac's kernel, asked by libproc.
#[cfg(target_os = "macos")]
mod mac {
    use super::*;

    /// Whether one of `me`'s processes holds the `raw` end at `at` connected
    /// to `to`; an error when libproc fails, but for a process that exited
    /// or a file it closed while asked.
    pub(super) fn ours(me: u32, raw: Raw, at: SocketAddr, to: SocketAddr) -> std::io::Result<bool> {
        // ESRCH: the process exited; EBADF: the file was closed.
        let gone = |e: &std::io::Error| matches!(e.raw_os_error(), Some(3 | 9));
        let int = |b: &[u8]| Some(i32::from_ne_bytes(b.get(..4)?.try_into().ok()?));
        let pids = kernel(Ask::Pids(me))?;
        for pid in pids.chunks_exact(4).filter_map(int) {
            let files = match kernel(Ask::Files(pid)) {
                Err(e) if gone(&e) => continue,
                files => files?,
            };
            // Each a `proc_fdinfo`: the fd, then its kind, 2 a socket.
            let sockets = files
                .chunks_exact(8)
                .filter(|f| record::word(f, 4) == Some(2));
            for fd in sockets.filter_map(int) {
                let socket = match kernel(Ask::Socket(pid, fd)) {
                    Err(e) if gone(&e) => continue,
                    socket => socket?,
                };
                if socket.len() != record::SOCKET_LEN {
                    let len = socket.len();
                    return Err(std::io::Error::other(format!(
                        "a socket's record of {len} bytes"
                    )));
                }
                let read = record::socket(&socket);
                if read.is_some_and(|(r, ends)| r == raw && holds(raw, ends, at, to)) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// What this Mac's kernel is asked, one libproc call each ([`kernel`]).
    #[derive(Debug, Clone, Copy)]
    pub(super) enum Ask {
        /// The pids of a uid's processes: `proc_listpids`.
        Pids(u32),
        /// A process's open files, each a `proc_fdinfo`: `PROC_PIDLISTFDS`.
        Files(i32),
        /// One file's `socket_fdinfo`: `PROC_PIDFDSOCKETINFO`.
        Socket(i32, i32),
        /// A process's `proc_bsdinfo`: `PROC_PIDTBSDINFO`.
        Bsd(i32),
    }

    /// `PROC_UID_ONLY`, `PROC_PIDTBSDINFO` and `PROC_PIDFDSOCKETINFO`, which
    /// `libc` lacks (`libproc.h`, `sys/proc_info.h`).
    const PROC_UID_ONLY: u32 = 4;
    const PROC_PIDTBSDINFO: i32 = 3;
    const PROC_PIDFDSOCKETINFO: i32 = 3;
    /// Room for what began between asking a list's size and filling it.
    const SLACK: usize = 4096;

    /// `ask`'s answer, the bytes the kernel wrote. The one `unsafe` in svrngs:
    /// every product crate denies `unsafe_code`, and `scripts/lean_check.py`
    /// refuses an exemption anywhere but here.
    #[allow(unsafe_code)]
    pub(super) fn kernel(ask: Ask) -> std::io::Result<Vec<u8>> {
        use libc::{c_int, c_void};
        let none: *mut c_void = std::ptr::null_mut();
        let len = |b: &Vec<u8>| c_int::try_from(b.len()).unwrap_or(c_int::MAX);
        let sized = |n: c_int| vec![0u8; usize::try_from(n).unwrap_or(0) + SLACK];
        // libproc answers 0 for a failure, setting errno, and for a list
        // with nothing in it, leaving errno as it was: a process with no
        // open files answers 0 with errno 0 (seen 2026-11-18, `sleep` with
        // its three closed). So errno is cleared before each answer read.
        // SAFETY: `__error` is this thread's own errno, always valid.
        let clear = || unsafe { *libc::__error() = 0 };
        let mut buf;
        let wrote = match ask {
            Ask::Pids(uid) => {
                // SAFETY: with no buffer and a length of 0 it writes nothing,
                // and answers the length it would fill.
                buf = sized(unsafe { libc::proc_listpids(PROC_UID_ONLY, uid, none, 0) });
                clear();
                // SAFETY: the length passed is `buf`'s, so it writes nothing
                // past its end.
                unsafe {
                    libc::proc_listpids(PROC_UID_ONLY, uid, buf.as_mut_ptr().cast(), len(&buf))
                }
            }
            Ask::Files(pid) => {
                let list = libc::PROC_PIDLISTFDS;
                // SAFETY: with no buffer and a length of 0 it writes nothing,
                // and answers the length it would fill.
                buf = sized(unsafe { libc::proc_pidinfo(pid, list, 0, none, 0) });
                clear();
                // SAFETY: the length passed is `buf`'s.
                unsafe { libc::proc_pidinfo(pid, list, 0, buf.as_mut_ptr().cast(), len(&buf)) }
            }
            Ask::Bsd(pid) => {
                buf = vec![0u8; record::BSD_LEN];
                // SAFETY: the length passed is `buf`'s, and the kernel refuses
                // a buffer shorter than its record rather than overrun it.
                unsafe {
                    libc::proc_pidinfo(pid, PROC_PIDTBSDINFO, 0, buf.as_mut_ptr().cast(), len(&buf))
                }
            }
            Ask::Socket(pid, fd) => {
                buf = vec![0u8; record::SOCKET_LEN];
                let info = PROC_PIDFDSOCKETINFO;
                // SAFETY: the length passed is `buf`'s, and the kernel refuses
                // a buffer shorter than its record rather than overrun it.
                unsafe { libc::proc_pidfdinfo(pid, fd, info, buf.as_mut_ptr().cast(), len(&buf)) }
            }
        };
        let why = std::io::Error::last_os_error();
        let wrote = usize::try_from(wrote).unwrap_or(0);
        let list = matches!(ask, Ask::Pids(_) | Ask::Files(_));
        if wrote == 0 && !(list && why.raw_os_error() == Some(0)) {
            return Err(why);
        }
        buf.truncate(wrote);
        Ok(buf)
    }
}

#[cfg(test)]
#[path = "uid_tests.rs"]
mod tests;
