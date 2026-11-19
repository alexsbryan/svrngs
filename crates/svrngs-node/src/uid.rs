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
//!
//! A lend's program, [`program`], is read with its sandbox when it runs as
//! svrngs's account (svrn-docs LEND_SANDBOX): a sandboxed app of that
//! account can't read what svrngs keeps, yet shares its network.
//! - Linux: the Flatpak app id of the processes holding the sockets that
//!   listen there, by each one's inode among this account's `/proc/<pid>/fd`.
//! - macOS: whether the process holding the socket is an App Sandbox app,
//!   by its code's entitlement as Security.framework says, and its path.

use std::net::{IpAddr, SocketAddr};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::sync::PoisonError;

use crate::lend::{Raw, Sandbox};

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner {
    /// The uid of the account that holds it, and the sandbox of the program
    /// answering there when [`program`] asks and that account is svrngs's:
    /// another account's programs are that account's to read, so its
    /// account alone decides.
    Uid(u32, Option<Sandbox>),
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
/// flow as `raw` says, read now and never kept; asked with `seen` and of
/// svrngs's own account, with the sandbox of the program whose socket took
/// it, kept there, or why it can't be told ([`Owner::Unjudged`]). A Mac's
/// may wait up to [`ACCEPT_WITHIN`], so an async caller reads it off its
/// runtime.
pub fn owner(raw: Raw, conn: Conn, side: Side, seen: Option<&Seen>) -> Owner {
    let (at, to) = ends(conn, side);
    let owner = read(raw, at, to, seen);
    tracing::debug!(target: "uid", ?raw, %at, %to, ?owner, "uid: whose the socket at this end is, and in what sandbox");
    owner
}

/// The sandboxes [`owner`] read, each kept by what a program taking a
/// lend's port must make anew: on Linux the inode of a socket listening
/// there, since a new socket has a new one; on a Mac the pid and start of
/// the process holding it, since a process's App Sandbox is fixed at its
/// launch. So a holder is read once, not at each connection (Linux's scan
/// took 5–18 ms, 2026-11-19), and no other program can answer within what
/// is kept: the window LEND_OWNER §3 keeps a cache out for. A Mac program
/// updated on disk while it runs, whose signature Security then won't
/// judge, keeps what was read of it.
#[derive(Debug, Default)]
pub struct Seen(
    #[expect(clippy::disallowed_types, reason = "the uid reader's cache")]
    std::sync::Mutex<std::collections::HashMap<(u64, u64), Sandbox>>,
);

/// A sandbox read, or why it can't be.
type Read = Result<Sandbox, &'static str>;

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl Seen {
    /// The sandbox kept at `key`, else `read`'s, kept when it is one. The
    /// lock is held while it reads, so one listener is read once.
    fn get_or(&self, key: (u64, u64), read: impl FnOnce() -> Read) -> Read {
        let mut seen = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(kept) = seen.get(&key) {
            return Ok(kept.clone());
        }
        let read = read();
        tracing::debug!(target: "uid", ?key, ?read, "uid: the sandbox of the program holding a socket, read and kept");
        if let Ok(sandbox) = &read {
            // Emptied when full, so a server forking a process for each
            // connection can't grow it without end; what was kept is read
            // again, as after a restart.
            if seen.len() >= KEPT {
                seen.clear();
            }
            seen.insert(key, sandbox.clone());
        }
        read
    }
}

/// How many sandboxes [`Seen`] keeps before it empties.
#[cfg(any(target_os = "linux", target_os = "macos"))]
const KEPT: usize = 4096;

/// `uid`'s, asked with `seen` with its program's sandbox as `read` reads
/// it there, or why that can't be told.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn with(uid: u32, seen: Option<&Seen>, read: impl FnOnce(&Seen) -> Read) -> Owner {
    let read = |seen| read(seen).map_or_else(Owner::Unjudged, |s| Owner::Uid(uid, Some(s)));
    seen.map_or(Owner::Uid(uid, None), read)
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

/// The end at `at`, connected to `to`, as `/proc/net` lists it, and, asked
/// with `seen` and of this account's, the sandbox of what listens there.
#[cfg(target_os = "linux")]
fn read(raw: Raw, at: SocketAddr, to: SocketAddr, seen: Option<&Seen>) -> Owner {
    let rows = match proc_net::table(raw) {
        Ok(rows) => rows,
        Err(why) => return Owner::Unjudged(why),
    };
    let uids = rows.iter().filter(|r| holds(raw, r.0, at, to));
    match one(uids.map(|r| r.1)) {
        Owner::Uid(uid, _) if mine() == Ok(uid) => with(uid, seen, |seen| {
            // What a connection there reaches: for TCP a socket bound there
            // or to every address and connected to none, as one listening
            // is; for UDP one bound so, as `holds` says for a datagram.
            let unconnected = |r: &&proc_net::Row| raw == Raw::Udp || r.0 .1.port() == 0;
            let reached = rows.iter().filter(unconnected);
            let reached = reached.filter(|r| holds(Raw::Udp, r.0, at, to));
            let read = reached.map(|r| seen.get_or((r.2, 0), || proc_net::holders(r.2)));
            proc_net::alike(read, "no socket listens there")
        }),
        owner => owner,
    }
}

/// The one uid among the sockets holding an end: none, or two accounts,
/// could not be judged.
#[cfg(any(target_os = "linux", test))]
fn one(mut uids: impl Iterator<Item = u32>) -> Owner {
    match uids.next() {
        None => Owner::Unjudged("no socket at that end on this computer"),
        Some(uid) if uids.all(|u| u == uid) => Owner::Uid(uid, None),
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
fn read(raw: Raw, at: SocketAddr, to: SocketAddr, seen: Option<&Seen>) -> Owner {
    let me = match mine() {
        Ok(me) => me,
        Err(why) => return Owner::Unjudged(why),
    };
    let (began, mut wait) = (
        std::time::Instant::now(),
        std::time::Duration::from_millis(1),
    );
    loop {
        match mac::ours(me, raw, (at, to)) {
            Ok(Some(pid)) => return with(me, seen, |seen| mac::sandbox(pid, seen)),
            Err(e) => {
                tracing::debug!(target: "uid", %at, error = %e, "uid: this account's processes can't be read");
                return Owner::Unjudged("this account's programs can't be read");
            }
            Ok(None) => {}
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
fn read(_: Raw, _: SocketAddr, _: SocketAddr, _: Option<&Seen>) -> Owner {
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

    /// A socket as a `/proc/net` table lists it: its local address and the
    /// one it is connected to, its account and its inode.
    pub(super) type Row = (Ends, u32, u64);

    /// The sandbox of the processes holding socket `inode`, by their files.
    /// Yama's `ptrace_scope` 1 refuses none of these reads: they are
    /// ptrace's read mode, not attach (seen on Omarchy 4.5, 2026-11-19).
    /// Held by none whose files can be read, it is judged by the scopes of
    /// those whose files can't be (svrn-docs LEND_SANDBOX).
    #[cfg(target_os = "linux")]
    pub(super) fn holders(inode: u64) -> Read {
        use std::fs::{read_dir, read_link, read_to_string, DirEntry};
        let held = std::path::PathBuf::from(format!("socket:[{inode}]"));
        let procs = read_dir("/proc").map_err(|_| "this computer's programs can't be listed")?;
        let procs: Vec<DirEntry> = procs.flatten().collect();
        let link = |f: DirEntry| read_link(f.path()).is_ok_and(|l| l == held);
        let files = |p: &&DirEntry| read_dir(p.path().join("fd")).map(|fs| fs.flatten().any(link));
        let holding = procs.iter().filter(|p| files(p).is_ok_and(|held| held));
        let read: Vec<Read> = holding.map(|p| flatpak(&p.path())).collect();
        if !read.is_empty() {
            return alike(read.into_iter(), "no program holds it");
        }
        let scope = format!("user@{}.service/app.slice/app-flatpak-", mine()?);
        let theirs = |p: &DirEntry| read_to_string(p.path().join("cgroup"));
        let boxed = |p: &DirEntry| theirs(p).is_ok_and(|c| c.contains(&scope));
        match procs.iter().filter(|p| files(p).is_err()).any(boxed) {
            true => Err("a Flatpak whose files can't be read may hold the socket"),
            false => Ok(Sandbox::No),
        }
    }

    /// The Flatpak app the process at `proc` (its `/proc/<pid>`) runs in, by
    /// its root's `.flatpak-info`, which Flatpak mounts read-only; none
    /// where its root has none, as xdg-desktop-portal reads it. Its root is
    /// read again, so a process that exited is no answer, never none.
    pub(super) fn flatpak(proc: &std::path::Path) -> Read {
        match std::fs::read_to_string(proc.join("root/.flatpak-info")) {
            Ok(info) => (info.lines().find_map(|l| l.strip_prefix("name=")))
                .map(|app| Sandbox::App(app.to_owned()))
                .ok_or("a Flatpak that names no app holds the socket"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && proc.join("root").exists() => {
                Ok(Sandbox::No)
            }
            Err(_) => Err("the sandbox of the program holding the socket can't be read"),
        }
    }

    /// The one sandbox all of `read` say, or why not: none said, `none`.
    pub(super) fn alike(read: impl Iterator<Item = Read>, none: &'static str) -> Read {
        let mut said: Vec<Sandbox> = read.collect::<Result<_, _>>()?;
        said.dedup();
        match &said[..] {
            [one] => Ok(one.clone()),
            [] => Err(none),
            _ => Err("programs in more than one sandbox hold the sockets there"),
        }
    }

    /// `raw`'s table, IPv4's and IPv6's.
    #[cfg(target_os = "linux")]
    pub(super) fn table(raw: Raw) -> Result<Vec<Row>, &'static str> {
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
    /// remote address, its uid, the eighth field, and its inode, the tenth. A TCP socket in TIME_WAIT (state
    /// `06`) is passed over: it lists uid 0 whoever held it, and a new
    /// connection on loopback may take its four numbers. A line not read so
    /// refuses the table.
    pub(super) fn rows(text: &str) -> Result<Vec<Row>, &'static str> {
        let row = |line: &str| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let ends = (addr(fields.get(1)?)?, addr(fields.get(2)?)?);
            let (uid, inode) = (fields.get(7)?.parse().ok()?, fields.get(9)?.parse().ok()?);
            Some((*fields.get(3)? != "06").then_some((ends, uid, inode)))
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

    /// A `proc_bsdinfo`'s start, its `pbi_start_tvsec` and `pbi_start_tvusec`
    /// as microseconds: none for a record of another length.
    pub(super) fn start(bsd: &[u8]) -> Option<u64> {
        let at = |at: usize| Some(u64::from_ne_bytes(bsd.get(at..at + 8)?.try_into().ok()?));
        let micros = |secs: u64, us: u64| secs.wrapping_mul(1_000_000).wrapping_add(us);
        (bsd.len() == BSD_LEN).then(|| Some(micros(at(START)?, at(START + 8)?)))?
    }
    const START: usize = 120;

    /// The 32-bit word at `at` in `bytes`, in this computer's byte order.
    pub(super) fn word(bytes: &[u8], at: usize) -> Option<u32> {
        Some(u32::from_ne_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    }
}

/// This Mac's kernel, asked by libproc.
#[cfg(target_os = "macos")]
mod mac {
    use super::*;

    /// Whether process `pid` is an App Sandbox app, by its code's
    /// entitlement as Security.framework reads it, and if so its code's
    /// path; kept by its pid and start ([`Seen`]). Signed without the
    /// entitlement (`errSecCSReqFailed`) or unsigned (`errSecCSUnsigned`)
    /// is none; any other answer can't be judged.
    pub(super) fn sandbox(pid: i32, seen: &Seen) -> Read {
        use security_framework::os::macos::code_signing::{Flags, GuestAttributes, SecCode};
        let bsd = kernel(Ask::Bsd(pid)).unwrap_or_default();
        let start = record::start(&bsd).ok_or("the program's start can't be read")?;
        seen.get_or((u64::from(pid.unsigned_abs()), start), || {
            let mut asked = GuestAttributes::new();
            asked.set_pid(pid);
            let app = r#"entitlement["com.apple.security.app-sandbox"] exists"#.parse();
            let code = SecCode::copy_guest_with_attribues(None, &asked, Flags::NONE);
            match code.and_then(|c| c.check_validity(Flags::NONE, &app?).map(|()| c)) {
                Ok(code) => (code.path(Flags::NONE).ok().and_then(|u| u.to_path()))
                    .map(|path| Sandbox::App(path.display().to_string()))
                    .ok_or("the path of the app holding the socket can't be read"),
                Err(e) if matches!(e.code(), -67050 | -67062) => Ok(Sandbox::No),
                Err(e) => {
                    tracing::debug!(target: "uid", pid, error = %e, "uid: Security can't judge the signature of the process holding the socket");
                    Err("the signature of the program holding the socket can't be read")
                }
            }
        })
    }

    /// The pid of one of `me`'s processes holding the `raw` end at `at`
    /// connected to `to`; an error when libproc fails, but for a process
    /// that exited or a file it closed while asked.
    pub(super) fn ours(me: u32, raw: Raw, (at, to): Ends) -> std::io::Result<Option<i32>> {
        // Gone since it was listed: ESRCH, the process exited; EBADF, the
        // file was closed; ENOTSOCK, the file was closed and its number
        // reused for one that isn't a socket (seen 2026-11-18, the night
        // scene's Jellyfin, while it streamed).
        let gone = |e: &std::io::Error| matches!(e.raw_os_error(), Some(3 | 9 | 38));
        let int = |b: &[u8]| Some(i32::from_ne_bytes(b.get(..4)?.try_into().ok()?));
        let pids = kernel(Ask::Pids(me))?;
        for pid in pids.chunks_exact(4).filter_map(int) {
            let failed = |ask: Ask, e: std::io::Error| {
                tracing::debug!(target: "uid", ?ask, error = %e, "uid: libproc failed for one of this account's processes");
                e
            };
            let files = match kernel(Ask::Files(pid)) {
                Err(e) if gone(&e) => continue,
                files => files.map_err(|e| failed(Ask::Files(pid), e))?,
            };
            // Each a `proc_fdinfo`: the fd, then its kind, 2 a socket.
            let sockets = files
                .chunks_exact(8)
                .filter(|f| record::word(f, 4) == Some(2));
            for fd in sockets.filter_map(int) {
                let socket = match kernel(Ask::Socket(pid, fd)) {
                    Err(e) if gone(&e) => continue,
                    socket => socket.map_err(|e| failed(Ask::Socket(pid, fd), e))?,
                };
                if socket.len() != record::SOCKET_LEN {
                    let len = socket.len();
                    return Err(std::io::Error::other(format!(
                        "a socket's record of {len} bytes"
                    )));
                }
                let read = record::socket(&socket);
                if read.is_some_and(|(r, ends)| r == raw && holds(raw, ends, at, to)) {
                    return Ok(Some(pid));
                }
            }
        }
        Ok(None)
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
