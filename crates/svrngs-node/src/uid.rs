// SPDX-License-Identifier: AGPL-3.0-or-later
//! What svrngs reads of this computer's sockets, read now and never kept.
//!
//! **Whose a socket is** ([`owner`]; svrn-docs rings/current/LEND_OWNER.md
//! §2): the account holding the socket at one end of a TCP connection.
//! - Linux: sock_diag (`NETLINK_SOCK_DIAG`) lists each socket with its uid,
//!   any account's, read exactly.
//! - macOS: this account's processes alone, by libproc (this uid's pids,
//!   each one's files, each socket's addresses): found, it is this
//!   account's; not found, [`Owner::NotMine`]. A Mac shows a program signed
//!   as svrngs no other account's sockets.
//! - Elsewhere no reader: [`Owner::NoReader`].
//!
//! **A share's port** ([`port`], [`answering`]; svrn-docs SHARE_BINDING
//! §5.1, §5.2): the sockets on it, as [`eudoxia_community::binding`] decides over them.
//! - Linux: sock_diag's dump of the port, each socket's cookie and cgroup,
//!   a TCP one's listening or bound and not listening, and the server end of
//!   the door's connection by its two ports; for an approval, the processes
//!   holding each by their `/proc/<pid>/fd`.
//! - macOS UDP: ntstat (`com.apple.network.statistics`, [`record::stats`]),
//!   which lists each of this account's UDP sockets from the kernel's own
//!   record of it, made with the socket, and the process that last used it,
//!   by pid and unique id. A new client each read: about 8 µs.
//! - macOS TCP: one scan of this account's processes' files by libproc, each
//!   socket on the port by its generation count (`insi_gencnt`), which lasts
//!   for the socket's life, with the processes holding it, in this boot
//!   (`kern.bootsessionuuid`). Its socket tables (`net.inet.*.pcblist_n`) list a process only its
//!   own sockets unless Apple entitles it (`netstat` holds
//!   `com.apple.private.network.statistics`; seen 2026-11-19, macOS 27.2.1).
//!
//! What the kernel answers is parsed in safe Rust, every read
//! bounds-checked; [`kernel`]'s calls are the one `unsafe` in svrngs
//! (DESIGN §5). What a holder is shown as, its program and folder, is read
//! only to be shown, never compared.

use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, SocketAddr};

use crate::lend::{Answer, Raw};
use eudoxia_community::binding::{self, Peer, Process, Read, Took};

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
    /// The uid of the account that holds it.
    Uid(u32),
    /// A Mac's: no process of this account's holds it within
    /// [`ACCEPT_WITHIN`]; whose it is, a Mac doesn't show svrngs.
    NotMine,
    /// It was asked, and no one answer came: why.
    Unjudged(&'static str),
    /// This system has no reader, so nothing was asked.
    NoReader,
}

/// Why a share's port wasn't read: a closed set, as a lender's own list of
/// lends names it (`Offered::unread`), `{"why": "no_reader"}` or
/// `{"why": "failed", "detail": <why>}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "why", content = "detail", rename_all = "snake_case")]
pub enum Unread {
    /// This system has no reader.
    NoReader,
    /// It was asked, and didn't answer: why.
    Failed(&'static str),
}

/// A share's port read for an approval ([`answering`]): what
/// [`binding::approve`] reads, each process's parent among those read, the
/// start now of the process asked for, and what to show the person of each
/// socket's holders.
#[derive(Debug, Clone, Default)]
pub struct Answering {
    /// What was read, as [`binding::approve`] reads it.
    pub read: Read,
    pub parents: BTreeMap<u32, u32>,
    pub start: Option<u64>,
    /// What each socket is shown as, by its id: a socket shown nothing has
    /// no entry, and an approval of it is refused (`share::approve`).
    pub shown: Vec<(u64, Answer)>,
    /// On a Mac, a UDP socket there takes 127.0.0.1's datagrams on every
    /// address and lets others bind beside it (`SO_REUSEPORT`), which a
    /// program of another account may then do (`in_pcbbind`), where this
    /// account's read can't list it: an approval of it is refused
    /// (`share::approve`).
    pub beside: bool,
}

/// A share's port read to approve, as each system's reader gives it: the
/// sockets, each process's parent among those read, what to show, and
/// [`Answering::beside`].
type Held = (Read, BTreeMap<u32, u32>, Vec<(u64, Answer)>, bool);

/// How long a reader looks again for a TCP end it doesn't list yet,
/// waiting longer each time: a Mac's connection is in no process's files
/// until its listener accepts it, and Linux's server end is listed once its
/// handshake ends. A server that accepts at once was found by the first
/// scan 29 times in 30, and the 30th within 3.7 ms (2026-11-18, `python3 -m
/// http.server`); this is some 25 times that, short beside the 10 s a
/// request through the door may take.
pub const ACCEPT_WITHIN: std::time::Duration = std::time::Duration::from_millis(100);

/// The uid holding the socket at `side` of the TCP connection `conn`, read
/// now and never kept, or why it can't be told ([`Owner::Unjudged`]). It
/// may wait up to [`ACCEPT_WITHIN`], so an async caller reads it off its
/// runtime.
pub fn owner(conn: Conn, side: Side) -> Owner {
    let (at, to) = match side {
        Side::Local => (conn.local, conn.peer),
        Side::Peer => (conn.peer, conn.local),
    };
    let owner = read(at, to);
    tracing::debug!(target: "uid", %at, %to, ?owner, "uid: whose the socket at this end is");
    owner
}

/// The `raw` sockets on `port` now, and, given `conn`, the door's TCP
/// connection to it, what took it ([`binding::check`], SHARE_BINDING
/// §5.2): Linux's server end's cgroup, a Mac's acceptor's sockets; a
/// datagram's none. Blocking, and it may wait up to [`ACCEPT_WITHIN`].
pub fn port(raw: Raw, port: u16, conn: Option<Conn>) -> Result<(Read, Took), Unread> {
    let read = sockets(raw, port, conn.filter(|_| raw == Raw::Tcp));
    tracing::debug!(target: "uid", ?raw, port, ?conn, ?read, "uid: the sockets on a share's port, and what took the door's connection");
    read
}

/// The `raw` sockets on `port` now with the processes holding each, their
/// parents and what to show of each ([`binding::approve`], SHARE_BINDING
/// §5.1), from the same read, and the start now of `pid`, when one is
/// asked for. Blocking.
pub fn answering(raw: Raw, port: u16, pid: Option<u32>) -> Result<Answering, Unread> {
    let read = held(raw, port).map(|(read, parents, shown, beside)| Answering {
        read,
        parents,
        start: pid.and_then(start),
        shown,
        beside,
    });
    tracing::debug!(target: "uid", ?raw, port, ?pid, read = ?read.as_ref().map(|a| (&a.read, &a.shown)), "uid: what answers on a share's port, to approve");
    read
}

/// The start of process `pid` now, as `--pid <pid>@<start>` names it:
/// Linux's `/proc/<pid>/stat` field 22, a Mac's `proc_bsdinfo` start in
/// microseconds; none when no such process runs.
pub fn start(pid: u32) -> Option<u64> {
    #[cfg(target_os = "linux")]
    let start = linux::stat(pid).map(|(_, start)| start);
    #[cfg(target_os = "macos")]
    let start = mac::bsd(i32::try_from(pid).ok()?).map(|(_, start, _)| start);
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let start = None;
    tracing::debug!(target: "uid", pid, ?start, "uid: a process's start now");
    start
}

/// A socket's local address and the one it is connected to.
#[cfg(any(target_os = "macos", test))]
type Ends = (SocketAddr, SocketAddr);

/// Whether two addresses are one, an IPv4 one written as IPv6 the IPv4 one.
fn same(a: SocketAddr, b: SocketAddr) -> bool {
    a.port() == b.port() && a.ip().to_canonical() == b.ip().to_canonical()
}

/// `look`'s answer once it finds one, looked at again, waiting longer each
/// time, until [`ACCEPT_WITHIN`] has passed; none past it.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[expect(
    clippy::disallowed_methods,
    reason = "the reader's wait for an accept, at the edge"
)]
fn waited<T, E>(mut look: impl FnMut() -> Result<Option<T>, E>) -> Result<Option<T>, E> {
    let (began, mut wait) = (
        std::time::Instant::now(),
        std::time::Duration::from_millis(1),
    );
    loop {
        if let Some(found) = look()? {
            return Ok(Some(found));
        }
        let left = ACCEPT_WITHIN.saturating_sub(began.elapsed());
        tracing::debug!(target: "uid", ?left, "uid: the end asked for isn't listed now");
        if left.is_zero() {
            return Ok(None);
        }
        std::thread::sleep(wait.min(left));
        wait *= 2;
    }
}

/// The end at `at`, connected to `to`, as sock_diag lists it: the one uid
/// among the sockets there, TIME_WAIT's passed over, which lists uid 0
/// whoever held it while a new connection may take its four numbers. None,
/// or two accounts, could not be judged.
#[cfg(target_os = "linux")]
fn read(at: SocketAddr, to: SocketAddr) -> Owner {
    let states = !(1 << 6);
    let listed = match diag::dump(Raw::Tcp, states, at.port()) {
        Ok(listed) => listed,
        Err(why) => return Owner::Unjudged(why),
    };
    let mut uids = listed
        .iter()
        .filter(|d| same(d.local, at) && same(d.peer, to))
        .map(|d| d.uid);
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

/// Linux's share port ([`linux::read`]), and what took the connection: the
/// cgroup its server end carries, looked for until its handshake ends; or,
/// for a server end that waits on the door's first bytes (`TCP_DEFER_ACCEPT`,
/// a request with no cgroup: tcp_diag.c `tcp_req_diag_fill`), the listener
/// its handshake reached ([`Took::Waiting`]), which makes it a socket when
/// it listens (tcp_ipv4.c `tcp_v4_rcv`, its `TCP_NEW_SYN_RECV` arm). A
/// listener that stops listening stays bound and may listen again (tcp.c
/// `tcp_set_state`, af_inet.c `__inet_listen_sk`), so the read lists bound
/// sockets too, which a kernel before 6.8 doesn't (tcp_diag.c,
/// `TCPF_BOUND_INACTIVE`). Linux at torvalds/linux 65e4fd94, read 2026-11-19.
#[cfg(target_os = "linux")]
fn sockets(raw: Raw, port: u16, conn: Option<Conn>) -> Result<(Read, Took), Unread> {
    let (read, ..) = linux::read(raw, port, false)?;
    let Some(conn) = conn else {
        return Ok((read, Took::Datagram));
    };
    let end = || diag::dump(Raw::Tcp, diag::ESTABLISHED | diag::SYN_RECV, port);
    let found = waited(|| Ok(end()?.into_iter().find(|d| same(d.peer, conn.local))));
    let found = found.map_err(Unread::Failed)?;
    let found = found.ok_or(Unread::Failed("the connection's server end isn't listed"))?;
    let file = |at: &str| std::fs::read_to_string(at).unwrap_or_default();
    let took = match found.cgroup {
        _ if !cgroups_v2(&file("/proc/self/cgroup")) => Err("this computer doesn't keep its programs in cgroup v2, so which one took the connection can't be told"),
        Some(cgroup) => Ok(Took::Cgroup(cgroup)),
        None if found.state == 3 && lists_bound(&file("/proc/sys/kernel/osrelease")) => {
            Ok(Took::Waiting)
        }
        None if found.state == 3 => Err("the program waits for data before it takes a connection, and this kernel, before 6.8, can't list every socket that could take it"),
        None => Err("the connection's cgroup can't be read"),
    };
    Ok((read, took.map_err(Unread::Failed)?))
}

/// Whether a computer whose `/proc/self/cgroup` reads `cgroups` keeps its
/// programs in cgroup v2, where a socket's cgroup is its program's own: a
/// `0::` line. Under v1 alone each socket's is the root's, which tells no
/// program from another (SHARE_BINDING_REVIEWS §6 #11b).
#[cfg(any(target_os = "linux", test))]
fn cgroups_v2(cgroups: &str) -> bool {
    cgroups.lines().any(|l| l.starts_with("0::"))
}

/// What the socket `d` is connected to, as a check may take it to name a
/// server's socket for one of svrngs's flows (`Socket::peer`), read for a
/// UDP socket of this account (`me`) where svrngs runs in a cgroup v2 of its
/// own, not the root one (`/proc/self/cgroup` reading `cgroups`), so its
/// cgroup names its program: the one address it is connected to, or nothing
/// (`idiag_dport` 0, `idiag_dst` unspecified). Elsewhere unread, so the
/// socket is judged as one no flow names.
#[cfg(any(target_os = "linux", test))]
fn peer(d: &diag::Diag, raw: Raw, cgroups: &str, me: Option<u32>) -> Peer {
    let apart = cgroups
        .lines()
        .any(|l| l.starts_with("0::") && l.trim() != "0::/");
    match d.peer {
        _ if raw != Raw::Udp || !apart || me != Some(d.uid) => Peer::Unread,
        at if at.port() != 0 => Peer::To(SocketAddr::new(at.ip().to_canonical(), at.port())),
        at if at.ip().is_unspecified() => Peer::Unconnected,
        _ => Peer::Unread,
    }
}

/// Whether a kernel whose `/proc/sys/kernel/osrelease` reads `release`
/// lists a TCP socket bound to a port and not listening: 6.8 and later.
#[cfg(any(target_os = "linux", test))]
fn lists_bound(release: &str) -> bool {
    let mut v = release
        .split(['.', '-'])
        .map(|n| n.trim().parse::<u32>().unwrap_or(0));
    (v.next().unwrap_or(0), v.next().unwrap_or(0)) >= (6, 8)
}

/// Linux's share port to approve: [`linux::read`] with each socket's
/// holders and their parents; shown, each holder, or for another account's
/// socket none of whose holders can be read, that account's uid. One of this
/// account's whose holders can't be read (a program made undumpable, so its
/// `/proc/<pid>/fd` is root's) is shown nothing.
#[cfg(target_os = "linux")]
fn held(raw: Raw, port: u16) -> Result<Held, Unread> {
    let (read, procs, listed) = linux::read(raw, port, true)?;
    let mut shown: Vec<_> = holding(&read)
        .map(|(id, h)| (id, linux::shown(&h)))
        .collect();
    let others = (listed.iter().zip(&read.sockets))
        .filter(|(d, s)| s.holders.is_empty() && mine() != Ok(d.uid));
    shown.extend(others.map(|(d, s)| {
        let account = d.uid.to_string();
        (s.id, Answer::Account { account })
    }));
    let parents = procs.iter().map(|(pid, p)| (*pid, p.0)).collect();
    Ok((read, parents, shown, false))
}

/// This Linux computer's sockets on a port, its processes and its boot.
#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    /// Each process by its id: its parent, its start, and the inodes it
    /// holds a socket of, of those asked.
    pub(super) type Procs = BTreeMap<u32, (u32, u64, BTreeSet<u64>)>;

    /// `raw`'s sockets on `port`, as sock_diag lists them in this boot: a
    /// TCP socket listening or bound and not listening, a UDP socket bound in
    /// any state; with the processes holding each, and those processes, when
    /// `held`.
    pub(super) fn read(
        raw: Raw,
        port: u16,
        held: bool,
    ) -> Result<(Read, Procs, Vec<diag::Diag>), Unread> {
        let boot = std::fs::read_to_string("/proc/sys/kernel/random/boot_id").unwrap_or_default();
        let boot =
            super::boot(&boot).ok_or(Unread::Failed("this computer's boot can't be read"))?;
        let on = if raw == Raw::Tcp {
            diag::LISTEN | diag::BOUND
        } else {
            u32::MAX
        };
        let listed = diag::dump(raw, on, port).map_err(Unread::Failed)?;
        let procs = if held {
            procs(&listed.iter().map(|d| d.inode).collect())
        } else {
            Procs::new()
        };
        let holders = |d: &diag::Diag| -> BTreeSet<Process> {
            let holding = procs.iter().filter(|(_, p)| p.2.contains(&d.inode));
            holding.map(|(pid, p)| (*pid, p.1)).collect()
        };
        let (cgroups, me) = (std::fs::read_to_string("/proc/self/cgroup"), mine().ok());
        let cgroups = cgroups.unwrap_or_default();
        let socket = |d: &diag::Diag| binding::Socket {
            id: d.cookie,
            cgroup: d.cgroup,
            holders: holders(d),
            at: d.local.ip(),
            dual: d.local.is_ipv6() && !d.v6only,
            peer: peer(d, raw, &cgroups, me),
        };
        let sockets = listed.iter().map(socket).collect();
        let read = Read {
            boot: Some(boot),
            lasting: true,
            sockets,
            unlisted: false,
        };
        Ok((read, procs, listed))
    }

    /// Process `pid`'s parent and start: its `/proc/<pid>/stat`'s fields 4
    /// and 22, counted past the `)` that ends its name, which may hold any.
    pub(super) fn stat(pid: u32) -> Option<(u32, u64)> {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let fields: Vec<&str> = stat.rsplit_once(')')?.1.split_whitespace().collect();
        Some((fields.get(1)?.parse().ok()?, fields.get(19)?.parse().ok()?))
    }

    /// Every process whose stat reads, with the sockets of `inodes` among
    /// its files, those it can read: another account's are that account's.
    fn procs(inodes: &BTreeSet<u64>) -> Procs {
        let listed = |dir: String| std::fs::read_dir(dir).into_iter().flatten().flatten();
        let socket = |l: std::path::PathBuf| {
            let inode = l
                .to_str()?
                .strip_prefix("socket:[")?
                .strip_suffix(']')?
                .parse()
                .ok();
            inode.filter(|i| inodes.contains(i))
        };
        let holds = |pid: u32| {
            listed(format!("/proc/{pid}/fd"))
                .filter_map(|f| socket(std::fs::read_link(f.path()).ok()?))
        };
        let pids =
            listed("/proc".into()).filter_map(|d| d.file_name().to_str()?.parse::<u32>().ok());
        let read = |pid| {
            Some((
                pid,
                stat(pid).map(|(parent, start)| (parent, start, holds(pid).collect()))?,
            ))
        };
        pids.filter_map(read).collect()
    }

    /// What a process is shown as: its executable's file name and its
    /// folder, each read only to be shown.
    pub(super) fn shown(&(pid, start): &Process) -> Answer {
        let at = |f: &str| std::fs::read_link(format!("/proc/{pid}/{f}")).ok();
        let exe = at("exe")
            .map(|e| e.display().to_string())
            .unwrap_or_default();
        let program = exe
            .trim_end_matches(" (deleted)")
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .into();
        Answer::Program {
            pid,
            start,
            program,
            folder: at("cwd").map(|c| folder(&c)),
        }
    }
}

/// Each socket of `read`, by its id, with each process holding it.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn holding(read: &Read) -> impl Iterator<Item = (u64, Process)> + Clone + '_ {
    (read.sockets.iter()).flat_map(|s| s.holders.iter().map(|h| (s.id, *h)))
}

/// A boot by its id, a UUID as text, read as its first 64 bits: Linux's
/// `/proc/sys/kernel/random/boot_id`, a Mac's `kern.bootsessionuuid`, each
/// made once for a boot. Not a Mac's `kern.boottime`, which moves when the
/// wall clock is stepped (osfmk/kern/clock.c `clock_set_calendar_microtime`).
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn boot(id: &str) -> Option<u64> {
    let hex = id.trim().replace('-', "");
    u64::from_str_radix(hex.get(..16)?, 16).ok()
}

/// A folder as a person reads it: under their home, from `~`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn folder(path: &std::path::Path) -> String {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let under = home.as_deref().and_then(|h| path.strip_prefix(h).ok());
    under.map_or(path.display().to_string(), |p| format!("~/{}", p.display()))
}

/// sock_diag (`NETLINK_SOCK_DIAG`): the sockets of a protocol on a port,
/// asked of the kernel in one dump each for IPv4 and IPv6, and its answer
/// read in safe Rust (`linux/inet_diag.h`, `linux/netlink.h`). Compiled
/// where it is asked, and for the tests that read a captured answer.
#[cfg(any(target_os = "linux", test))]
mod diag {
    use super::*;

    /// One socket as sock_diag lists it.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(super) struct Diag {
        /// Its TCP state: 3 `TCP_SYN_RECV`, as a request is listed.
        pub state: u8,
        pub local: SocketAddr,
        pub peer: SocketAddr,
        pub uid: u32,
        pub inode: u64,
        pub cookie: u64,
        pub cgroup: Option<u64>,
        /// An IPv6 socket that takes IPv6 alone, as said of one listening or
        /// not connected (inet_diag.c:122-124, v6.18); one not said of, a
        /// TCP one bound and not listening, is taken to take IPv4's too.
        pub v6only: bool,
    }

    /// TCP's states asked by bit (include/net/tcp_states.h): `TCP_LISTEN`
    /// (10), `TCP_BOUND_INACTIVE` (13), `TCP_ESTABLISHED` (1), `TCP_SYN_RECV`
    /// (3), which asks requests too (tcp_diag.c `tcp_diag_dump`).
    #[cfg(target_os = "linux")]
    pub(super) const LISTEN: u32 = 1 << 10;
    #[cfg(target_os = "linux")]
    pub(super) const BOUND: u32 = 1 << 13;
    #[cfg(target_os = "linux")]
    pub(super) const ESTABLISHED: u32 = 1 << 1;
    #[cfg(target_os = "linux")]
    pub(super) const SYN_RECV: u32 = 1 << 3;

    /// The sockets of `raw` in `states` whose local port is `port`.
    #[cfg(target_os = "linux")]
    pub(super) fn dump(raw: Raw, states: u32, port: u16) -> Result<Vec<Diag>, &'static str> {
        use netlink_sys::{protocols::NETLINK_SOCK_DIAG, Socket, SocketAddr as Kernel};
        let failed = |e: std::io::Error| {
            tracing::debug!(target: "uid", error = %e, "uid: sock_diag failed");
            "this computer's sockets can't be listed"
        };
        let socket = Socket::new(NETLINK_SOCK_DIAG).map_err(failed)?;
        let mut found = Vec::new();
        for family in [2, 10] {
            let asked = request(family, raw, states, port);
            socket
                .send_to(&asked, &Kernel::new(0, 0), 0)
                .map_err(failed)?;
            let mut done = false;
            while !done {
                let mut answer = Vec::with_capacity(64 * 1024);
                socket.recv(&mut answer, 0).map_err(failed)?;
                done = read(&answer, &mut found)?;
            }
        }
        Ok(found)
    }

    /// The dump asked: a netlink head of 84 bytes, `SOCK_DIAG_BY_FAMILY`
    /// (20) with `NLM_F_REQUEST | NLM_F_DUMP`; an `inet_diag_req_v2` of
    /// `family` and `raw`'s protocol in `states`; and its filter
    /// (`INET_DIAG_REQ_BYTECODE`), the local port equal to `port`
    /// (`INET_DIAG_BC_S_EQ`, 11): two `inet_diag_bc_op`s, the second's last
    /// field the port, going on 8 bytes, its end, when it holds, and 12,
    /// past its end, which refuses the socket, when it doesn't.
    pub(super) fn request(family: u8, raw: Raw, states: u32, port: u16) -> Vec<u8> {
        let mut asked = [0; 84];
        let mut put = |at: usize, bytes: &[u8]| asked[at..at + bytes.len()].copy_from_slice(bytes);
        put(0, &84u32.to_ne_bytes());
        put(4, &[20u16.to_ne_bytes(), 0x301u16.to_ne_bytes()].concat());
        put(16, &[family, if raw == Raw::Tcp { 6 } else { 17 }]);
        put(20, &states.to_ne_bytes());
        put(72, &[12u16.to_ne_bytes(), 1u16.to_ne_bytes()].concat());
        put(76, &[11, 8]);
        put(78, &12u16.to_ne_bytes());
        put(82, &port.to_ne_bytes());
        asked.to_vec()
    }

    /// Each socket in `answer`, one receive's messages, put in `found`;
    /// whether the dump is done (`NLMSG_DONE`); why not, for an error
    /// (`NLMSG_ERROR`, or `NLMSG_DONE` with one) or a message not read.
    pub(super) fn read(answer: &[u8], found: &mut Vec<Diag>) -> Result<bool, &'static str> {
        let unread = "this computer's list of sockets can't be read";
        let mut at = 0;
        while at < answer.len() {
            let head = answer.get(at..at + 16).ok_or(unread)?;
            let len = word(head, 0).ok_or(unread)? as usize;
            let body = answer.get(at + 16..at + len).ok_or(unread)?;
            match u16::from_ne_bytes([head[4], head[5]]) {
                3 if word(body, 0).is_some_and(|e| e != 0) => return Err(unread),
                3 => return Ok(true),
                2 => {
                    tracing::debug!(target: "uid", errno = ?word(body, 0), "uid: sock_diag refused the dump");
                    return Err("this computer refused to list its sockets");
                }
                20 => found.push(socket(body).ok_or(unread)?),
                _ => {}
            }
            at += (len + 3) & !3;
        }
        Ok(false)
    }

    /// An `inet_diag_msg`: its family and state, then its ports and
    /// addresses (each 16 bytes, an IPv4 one in the first 4), its cookie (two
    /// words, low first), its uid and inode; then its attributes,
    /// `INET_DIAG_SKV6ONLY` (11) whether it takes IPv6 alone, and
    /// `INET_DIAG_CGROUP_ID` (21) the cgroup it was made in.
    pub(super) fn socket(m: &[u8]) -> Option<Diag> {
        let port = |at: usize| Some(u16::from_be_bytes(m.get(at..at + 2)?.try_into().ok()?));
        let ip = |at: usize| match m.first()? {
            2 => Some(IpAddr::from(<[u8; 4]>::try_from(m.get(at..at + 4)?).ok()?)),
            10 => Some(IpAddr::from(
                <[u8; 16]>::try_from(m.get(at..at + 16)?).ok()?,
            )),
            _ => None,
        };
        let (mut cgroup, mut v6only, mut at) = (None, false, 72);
        while let Some(attr) = m.get(at..at + 4) {
            match u16::from_ne_bytes([attr[2], attr[3]]) {
                11 => v6only = m.get(at + 4) == Some(&1),
                21 => cgroup = Some(u64::from_ne_bytes(m.get(at + 4..at + 12)?.try_into().ok()?)),
                _ => {}
            }
            at += (usize::from(u16::from_ne_bytes([attr[0], attr[1]])).max(4) + 3) & !3;
        }
        let cookie = u64::from(word(m, 44)?) | u64::from(word(m, 48)?) << 32;
        let (local, peer) = (
            SocketAddr::new(ip(8)?, port(4)?),
            SocketAddr::new(ip(24)?, port(6)?),
        );
        Some(Diag {
            state: *m.get(1)?,
            local,
            peer,
            uid: word(m, 64)?,
            inode: word(m, 68)?.into(),
            cookie,
            cgroup,
            v6only,
        })
    }

    fn word(bytes: &[u8], at: usize) -> Option<u32> {
        Some(u32::from_ne_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    }
}

/// The end at `at`, connected to `to`, among this account's processes,
/// looked for again until [`ACCEPT_WITHIN`] has passed.
#[cfg(target_os = "macos")]
fn read(at: SocketAddr, to: SocketAddr) -> Owner {
    let me = match mine() {
        Ok(me) => me,
        Err(why) => return Owner::Unjudged(why),
    };
    match mac::scan(me, Raw::Tcp, None, Some((at, to))) {
        Ok(scan) if scan.took.is_some() => Owner::Uid(me),
        Ok(_) => Owner::NotMine,
        Err(e) => {
            tracing::debug!(target: "uid", %at, error = %e, "uid: this account's processes can't be read");
            Owner::Unjudged("this account's programs can't be read")
        }
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

/// A Mac's share port: this account's `raw` sockets on it by their
/// generation counts in this boot, each with the processes holding it, and
/// for a TCP connection the process holding its server end and its parents
/// ([`mac::scan`], [`mac::up`]); a UDP port as [`datagrams`] reads it.
#[cfg(target_os = "macos")]
fn sockets(raw: Raw, port: u16, conn: Option<Conn>) -> Result<(Read, Took), Unread> {
    let me = mine().map_err(Unread::Failed)?;
    if raw == Raw::Udp {
        return Ok((datagrams(me, port)?, Took::Datagram));
    }
    let boot = mac::boot().ok_or(Unread::Failed("this Mac's boot can't be read"))?;
    let scan = scanned(me, raw, port, conn.map(|c| (c.peer, c.local)))?;
    let named = |pid: &i32| Some((pid.unsigned_abs(), mac::bsd(*pid)?.1));
    let socket = |(&id, (at, dual, _, pids)): (&u64, &mac::On)| binding::Socket {
        id,
        cgroup: None,
        holders: pids.iter().filter_map(named).collect(),
        at: *at,
        dual: *dual,
        peer: Peer::Unread,
    };
    let read = Read {
        boot: Some(boot),
        lasting: true,
        sockets: scan.on.iter().map(socket).collect(),
        unlisted: false,
    };
    let up = scan
        .took
        .map(|pid| mac::up(pid.unsigned_abs()))
        .unwrap_or_default();
    let lineage: Vec<Process> = up.into_iter().map(|(pid, start, _)| (pid, start)).collect();
    match (conn, lineage) {
        (None, _) => Ok((read, Took::Datagram)),
        (Some(_), took) if !took.is_empty() => Ok((read, Took::Lineage(took))),
        (Some(_), _) => Err(Unread::Failed(
            "no program of this account that took the connection can be read",
        )),
    }
}

/// [`mac::scan`]'s answer, or why the port can't be judged.
#[cfg(target_os = "macos")]
fn scanned(me: u32, raw: Raw, port: u16, end: Option<Ends>) -> Result<mac::Scan, Unread> {
    mac::scan(me, raw, Some(port), end).map_err(|e| {
        tracing::debug!(target: "uid", port, error = %e, "uid: this account's processes can't be read");
        Unread::Failed("this account's programs can't be read")
    })
}

/// A Mac's UDP port, as ntstat lists this account's sockets
/// ([`record::stats`]): each on it, held by the process that last used it
/// while the process at that pid is still that one ([`last_user`]), else by
/// none, so a check holds unjudged. ntstat gives a socket on every IPv6
/// address as IPv6's, whether it takes IPv4's too or not (ntstat.c:2835-2840,
/// xnu main f6217f89), so a check takes each as dual, judged for 127.0.0.1.
#[cfg(target_os = "macos")]
fn datagrams(me: u32, port: u16) -> Result<Read, Unread> {
    let pid = std::process::id();
    let own =
        mac::bsd(pid as i32).ok_or(Unread::Failed("this process's own record can't be read"))?;
    let answer = mac::kernel(mac::Ask::Stats).map_err(|e| {
        tracing::debug!(target: "uid", port, error = %e, "uid: ntstat didn't answer");
        Unread::Failed("this Mac's UDP sockets can't be listed")
    })?;
    let listed = record::stats(&answer, (pid, me, own.2)).map_err(Unread::Failed)?;
    let socket = |s: &record::Stat| binding::Socket {
        id: s.srcref,
        cgroup: None,
        holders: last_user(s, mac::bsd(s.pid as i32)).into_iter().collect(),
        at: s.local.ip(),
        dual: true,
        peer: Peer::Unread,
    };
    let sockets = listed
        .iter()
        .filter(|s| s.local.port() == port)
        .map(socket)
        .collect();
    tracing::debug!(target: "uid", port, listed = listed.len(), ?sockets, "uid: a Mac's UDP sockets on a port, by ntstat");
    Ok(Read {
        boot: None,
        lasting: false,
        sockets,
        unlisted: false,
    })
}

/// The process that last used `stat`'s socket, by its pid and start, when
/// the process at that pid `now` (parent, start, unique id) is the same one:
/// a pid is used again, a unique id isn't while this computer runs.
#[cfg(any(target_os = "macos", test))]
fn last_user(stat: &record::Stat, now: Option<(u32, u64, u64)>) -> Option<Process> {
    now.filter(|(_, _, unique)| *unique == stat.upid)
        .map(|(_, start, _)| (stat.pid, start))
}

/// A Mac's share port to approve: [`sockets`], with each holder's parents
/// up to the first and what it is shown as. Only this account's sockets
/// are listed, so a TCP port that answers with none listed is another
/// account's ([`Read::unlisted`]). A UDP socket on every IPv6 address is
/// dual only where libproc reads a dual one on the port held by a process
/// holding it, so 127.0.0.1 is chosen only where an approved socket takes
/// IPv4's. `in_pcbbind` lets another uid bind beside a socket its lookup
/// finds where both are on every address and the one found has
/// `SO_REUSEPORT` (in_pcb.c:1016-1031, xnu main), so such a UDP socket is
/// said at approval ([`Answering::beside`]).
#[cfg(target_os = "macos")]
#[expect(clippy::disallowed_methods, reason = "dial: unwritten probe")]
fn held(raw: Raw, port: u16) -> Result<Held, Unread> {
    let (mut read, _) = sockets(raw, port, None)?;
    let mut beside = false;
    if raw == Raw::Udp {
        let scan = scanned(mine().map_err(Unread::Failed)?, raw, port, None)?;
        let open = |(at, dual, reuses, _): &mac::On| {
            *reuses && at.is_unspecified() && (at.is_ipv4() || *dual)
        };
        beside = scan.on.values().any(open);
        let duals = scan.on.values().filter(|(_, dual, ..)| *dual);
        let duals: BTreeSet<i32> = duals.flat_map(|(.., pids)| pids.iter().copied()).collect();
        for s in &mut read.sockets {
            s.dual = s
                .holders
                .iter()
                .any(|(pid, _)| duals.contains(&(*pid as i32)));
        }
    }
    let to = SocketAddr::from(([127, 0, 0, 1], port));
    let within = std::time::Duration::from_millis(500);
    let answers = raw == Raw::Tcp && std::net::TcpStream::connect_timeout(&to, within).is_ok();
    let read = Read {
        unlisted: read.sockets.is_empty() && answers,
        ..read
    };
    let holders = holding(&read);
    let up = holders.clone().flat_map(|(_, (pid, _))| mac::up(pid));
    let parents = up.map(|(pid, _, parent)| (pid, parent)).collect();
    let shown = holders.map(|(id, p)| (id, mac::shown(p))).collect();
    Ok((read, parents, shown, beside))
}

/// No reader here.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read(_: SocketAddr, _: SocketAddr) -> Owner {
    Owner::NoReader
}

/// No reader here.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn sockets(_: Raw, _: u16, _: Option<Conn>) -> Result<(Read, Took), Unread> {
    Err(Unread::NoReader)
}

/// No reader here.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn held(_: Raw, _: u16) -> Result<Held, Unread> {
    Err(Unread::NoReader)
}

/// No reader here.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn mine() -> Result<u32, &'static str> {
    Err("this system's accounts can't be read")
}

/// What a `socket_fdinfo`, a `proc_bsdinfo` and a `proc_vnodepathinfo`
/// hold, as svrngs reads them, their lengths and offsets those of the
/// captures in `fixtures/`: compiled where they are read, and for the tests.
#[cfg(any(target_os = "macos", test))]
mod record {
    use super::*;

    /// A `socket_fdinfo`'s length; its `soi_protocol`; its `soi_kind`; its
    /// `in_sockinfo`'s foreign and local ports, its `insi_gencnt`, IPv4/IPv6
    /// flag, and foreign and local addresses, an IPv4 one in the last 4 bytes
    /// of its 16. `insi_gencnt` is the generation count XNU's `in_pcballoc`
    /// gives the socket's protocol control block as it is made, from its
    /// protocol's count (`ipi_gencnt`): one socket's for its life, every
    /// process holding it reading the same, and another for a socket made
    /// after it, even by the same process on the same port. On this Mac
    /// (2026-11-20), a listener read the same bound, listening, after an
    /// accept and in a forked child, and Podman's gvproxy, one process, held
    /// a new one on its port for each new container.
    pub(super) const SOCKET_LEN: usize = 792;
    const GENCNT: usize = 272;
    const PROTOCOL: usize = 180;
    const KIND: usize = 256;
    const FPORT: usize = 264;
    const LPORT: usize = 268;
    const VFLAG: usize = 288;
    const FADDR: usize = 296;
    const LADDR: usize = 312;
    /// A `proc_bsdinfowithuniqid`'s length: a `proc_bsdinfo`, its
    /// `pbi_ppid` and `pbi_uid`, then a `proc_uniqidentifierinfo`, its
    /// `p_uniqueid` past its UUID.
    pub(super) const BSD_LEN: usize = 192;
    const PPID: usize = 16;
    const UID: usize = 20;
    const UNIQUE: usize = 152;
    /// A `proc_vnodepathinfo`'s length, and its current folder's path, past
    /// that folder's `vnode_info`.
    pub(super) const VNODE_LEN: usize = 2352;
    const CWD: usize = 152;

    /// An internet socket's protocol, its generation count, its local and foreign
    /// address, and whether it is dual: TCP's (`SOCKINFO_TCP`) or UDP's
    /// (`SOCKINFO_IN`, protocol 17); `None` for another socket, or one of
    /// neither family. Its `insi_vflag` (socket_info.c:140) is `INP_IPV4`,
    /// `INP_IPV6`, or both for an IPv6 socket that takes IPv4's too, on every
    /// address or not yet bound (udp6_usrreq.c:836-838, :868-878,
    /// tcp_usrreq.c:334-338, xnu main f6217f89), which is read as IPv6's.
    pub(super) fn socket(socket: &[u8]) -> Option<(Raw, u64, Ends, bool)> {
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
        let (v4, dual) = match *socket.get(VFLAG)? & 0x3 {
            1 => (true, false),
            2 => (false, false),
            3 => (false, true),
            _ => return None,
        };
        let gencnt = u64::from_ne_bytes(socket.get(GENCNT..GENCNT + 8)?.try_into().ok()?);
        let local = SocketAddr::new(ip(LADDR, v4)?, port(LPORT)?);
        let foreign = SocketAddr::new(ip(FADDR, v4)?, port(FPORT)?);
        Some((raw, gencnt, (local, foreign), dual))
    }

    /// Whether a `socket_fdinfo`'s socket lets others bind beside it: its
    /// `soi_options` holds `SO_REUSEPORT` (0x200, sys/socket.h).
    pub(super) fn reuses(socket: &[u8]) -> bool {
        let options = socket.get(OPTIONS..OPTIONS + 2);
        matches!(options, Some(&[a, b]) if u16::from_ne_bytes([a, b]) & 0x200 != 0)
    }
    const OPTIONS: usize = 188;

    /// A `proc_bsdinfo`'s uid: none for a record of another length.
    pub(super) fn uid(bsd: &[u8]) -> Option<u32> {
        (bsd.len() == BSD_LEN).then(|| word(bsd, UID)).flatten()
    }

    /// A `proc_bsdinfo`'s parent: none for a record of another length.
    pub(super) fn ppid(bsd: &[u8]) -> Option<u32> {
        (bsd.len() == BSD_LEN).then(|| word(bsd, PPID)).flatten()
    }

    /// The process's unique id: none for a record of another length.
    pub(super) fn unique(bsd: &[u8]) -> Option<u64> {
        (bsd.len() == BSD_LEN).then(|| long(bsd, UNIQUE)).flatten()
    }

    /// A `proc_bsdinfo`'s start, its `pbi_start_tvsec` and `pbi_start_tvusec`
    /// as microseconds: none for a record of another length.
    pub(super) fn start(bsd: &[u8]) -> Option<u64> {
        let at = |at: usize| Some(u64::from_ne_bytes(bsd.get(at..at + 8)?.try_into().ok()?));
        let micros = |secs: u64, us: u64| secs.wrapping_mul(1_000_000).wrapping_add(us);
        (bsd.len() == BSD_LEN).then(|| Some(micros(at(START)?, at(START + 8)?)))?
    }
    const START: usize = 120;

    /// A `proc_vnodepathinfo`'s current folder, its path up to its NUL.
    pub(super) fn cwd(vnode: &[u8]) -> Option<std::path::PathBuf> {
        let path = vnode.get(CWD..).filter(|_| vnode.len() == VNODE_LEN)?;
        let path = &path[..path.iter().position(|b| *b == 0)?];
        Some(std::path::PathBuf::from(
            String::from_utf8_lossy(path).into_owned(),
        ))
    }

    /// The 32-bit word at `at` in `bytes`, in this computer's byte order.
    pub(super) fn word(bytes: &[u8], at: usize) -> Option<u32> {
        Some(u32::from_ne_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    }

    /// The 64-bit word at `at` in `bytes`, in this computer's byte order.
    fn long(bytes: &[u8], at: usize) -> Option<u64> {
        Some(u64::from_ne_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
    }

    /// ntstat's control, and the room its answer is given: 4 MiB, some
    /// 9,800 UDP sockets of the account (SHARE_BINDING_UDP §4.6).
    #[cfg(target_os = "macos")]
    pub(super) const STATS: &[u8] = b"com.apple.network.statistics";
    #[cfg(target_os = "macos")]
    pub(super) const STATS_ROOM: i32 = 4 << 20;
    /// A `nstat_udp_descriptor`'s length (ntstat.h:518-555), after the 40
    /// bytes of its `nstat_msg_src_description`, and its `upid`, `local`,
    /// `pid` and `uid`.
    const UDP_DESC: usize = 280;
    const UPID: usize = 0;
    const LOCAL: usize = 56;
    const PID: usize = 128;
    const OWNER: usize = 268;

    /// One UDP socket as ntstat describes it: its local address, and the
    /// last process to use it, by pid and unique id, and its account.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(super) struct Stat {
        pub srcref: u64,
        pub local: SocketAddr,
        pub pid: u32,
        pub upid: u64,
        pub uid: u32,
    }

    /// The two asks of a read, each with its context: `ADD_ALL_SRCS` (1002)
    /// of `UDP_KERNEL` (4), `SUPPRESS_SRC_ADDED` its filter, then
    /// `GET_SRC_DESC` (1005) of every source (ntstat.h:977-982, :1020-1044).
    #[cfg(target_os = "macos")]
    pub(super) fn stats_asks() -> [(u64, Vec<u8>); 2] {
        let head = |ctx: u64, kind: u32, len: u16| {
            [
                &ctx.to_ne_bytes()[..],
                &kind.to_ne_bytes(),
                &len.to_ne_bytes(),
                &[0, 0],
            ]
            .concat()
        };
        let filter = 0x0010_0000u64.to_ne_bytes();
        let all = [
            head(1, 1002, 56),
            filter.to_vec(),
            vec![0; 8],
            4u32.to_ne_bytes().to_vec(),
            vec![0; 20],
        ];
        [
            (1, all.concat()),
            (
                2,
                [head(2, 1005, 24), u64::MAX.to_ne_bytes().to_vec()].concat(),
            ),
        ]
    }

    /// Each message in `record`, one received, by its context, type and
    /// bytes, walked by their lengths; an `Err` for one past its end.
    fn messages(record: &[u8]) -> impl Iterator<Item = Result<(u64, u32, &[u8]), &'static str>> {
        let mut at = 0;
        std::iter::from_fn(move || {
            let head = record.get(at..).filter(|h| !h.is_empty())?;
            let len = head
                .get(12..14)
                .map(|l| usize::from(u16::from_ne_bytes([l[0], l[1]])));
            let msg = len.filter(|l| *l >= 16).and_then(|l| head.get(..l));
            let Some(msg) = msg else {
                at = record.len();
                return Some(Err("a message's length runs past its record"));
            };
            at += msg.len();
            Some(Ok((long(msg, 0)?, word(msg, 8)?, msg)))
        })
    }

    /// Whether `record` ends the ask of `context`: its `SUCCESS` (0) or
    /// `ERROR` (1).
    #[cfg(target_os = "macos")]
    pub(super) fn ended(record: &[u8], context: u64) -> bool {
        messages(record)
            .flatten()
            .any(|(ctx, kind, _)| ctx == context && kind <= 1)
    }

    /// Each UDP socket `answer` lists, what [`super::mac::Ask::Stats`]
    /// read: the room it was given, then each record, its length first. Or
    /// why it can't be told, so the share holds:
    /// - a message past its record's end, an error answered, or no end;
    /// - a descriptor of another length than 280, or of another provider;
    /// - more read than ¾ of the room less 4 KiB, past which the kernel may
    ///   drop a part of the answer and still end it (kern_control.c:925-940);
    /// - none that is `me`'s canary, by its pid, uid and unique id, unbound
    ///   on IPv4 (its address `16, 2, 0, 0`), so a field read from the wrong
    ///   place reads as no canary.
    pub(super) fn stats(answer: &[u8], me: (u32, u32, u64)) -> Result<Vec<Stat>, &'static str> {
        let unread = "this Mac's list of UDP sockets can't be read";
        let room = word(answer, 0).ok_or(unread)? as usize;
        let (mut at, mut read, mut ended, mut found) = (4, 0, false, Vec::new());
        while let Some(len) = word(answer, at) {
            let record = answer.get(at + 4..at + 4 + len as usize).ok_or(unread)?;
            (at, read) = (at + 4 + record.len(), read + record.len());
            for msg in messages(record) {
                match msg? {
                    (1 | 2, 1, _) => return Err("this Mac refused to list its UDP sockets"),
                    (2, 0, _) => ended = true,
                    (0, 10003, desc) => found.push(described(desc)?),
                    _ => {}
                }
            }
        }
        let canary =
            |s: &Stat| (s.pid, s.uid, s.upid) == me && s.local == SocketAddr::from(([0; 4], 0));
        match () {
            () if !ended => Err("this Mac's list of UDP sockets didn't end"),
            () if read > (room - room / 4).saturating_sub(4096) => {
                Err("this Mac's list of UDP sockets came near the end of its room")
            }
            () if !found.iter().any(canary) => {
                Err("this Mac's list of UDP sockets doesn't hold svrngs's own")
            }
            () => Ok(found),
        }
    }

    /// A `SRC_DESC` (10003) of a UDP socket, its descriptor past the 40
    /// bytes of its head, `UDP_KERNEL` (4) its provider.
    fn described(desc: &[u8]) -> Result<Stat, &'static str> {
        let d = (desc.get(40..))
            .filter(|d| d.len() == UDP_DESC && word(desc, 32) == Some(4))
            .ok_or("this Mac describes a UDP socket in another form")?;
        let at = |b: &[u8]| -> Option<SocketAddr> {
            let port = u16::from_be_bytes([b[2], b[3]]);
            match (b[0], b[1]) {
                (16, 2) => Some(SocketAddr::from((
                    <[u8; 4]>::try_from(&b[4..8]).ok()?,
                    port,
                ))),
                (28, 30) => Some(SocketAddr::from((
                    <[u8; 16]>::try_from(&b[8..24]).ok()?,
                    port,
                ))),
                _ => None,
            }
        };
        let local = at(&d[LOCAL..LOCAL + 28])
            .ok_or("this Mac describes a UDP socket's address in another form")?;
        let stat = || {
            Some(Stat {
                srcref: long(desc, 16)?,
                local,
                pid: word(d, PID)?,
                upid: long(d, UPID)?,
                uid: word(d, OWNER)?,
            })
        };
        stat().ok_or("this Mac describes a UDP socket in another form")
    }
}

/// This Mac's kernel, asked by libproc.
#[cfg(target_os = "macos")]
mod mac {
    use super::*;

    /// A socket on the port: its address, whether it is dual, whether it
    /// lets others bind beside it ([`record::reuses`]), and the pids holding
    /// it.
    pub(super) type On = (IpAddr, bool, bool, BTreeSet<i32>);

    /// One scan of this account's processes ([`scan`]).
    pub(super) struct Scan {
        /// Each socket on the port, by its generation count.
        pub on: BTreeMap<u64, On>,
        /// The process holding the end asked for, if found.
        pub took: Option<i32>,
    }

    /// The `raw` sockets of `me`'s processes on `port`, TCP's connected to
    /// none and UDP's any; and, for `end`, the TCP end at its first address
    /// connected to its second, the process holding it, looked for until
    /// [`ACCEPT_WITHIN`] has passed, since a connection is in no process's
    /// files until it is accepted. An error when libproc fails, but for a
    /// process that exited or a file it closed while asked.
    pub(super) fn scan(
        me: u32,
        raw: Raw,
        port: Option<u16>,
        end: Option<Ends>,
    ) -> std::io::Result<Scan> {
        if end.is_none() {
            return once(me, raw, port, None);
        }
        let found = waited(|| once(me, raw, port, end).map(|s| s.took.is_some().then_some(s)))?;
        Ok(found.unwrap_or(Scan {
            on: BTreeMap::new(),
            took: None,
        }))
    }

    /// [`scan`]'s one look.
    fn once(me: u32, raw: Raw, port: Option<u16>, end: Option<Ends>) -> std::io::Result<Scan> {
        // Gone since it was listed: ESRCH, the process exited; EBADF, the
        // file was closed; ENOTSOCK, the file was closed and its number
        // reused for one that isn't a socket (seen 2026-11-18, the night
        // scene's Jellyfin, while it streamed).
        let gone = |e: &std::io::Error| matches!(e.raw_os_error(), Some(3 | 9 | 38));
        let int = |b: &[u8]| Some(i32::from_ne_bytes(b.get(..4)?.try_into().ok()?));
        let (mut on, mut took) = (BTreeMap::<u64, On>::new(), None);
        for pid in kernel(Ask::Pids(me))?.chunks_exact(4).filter_map(int) {
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
            let mut holds = false;
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
                let Some((_, gencnt, (local, foreign), dual)) =
                    record::socket(&socket).filter(|s| s.0 == raw)
                else {
                    continue;
                };
                holds |= end.is_some_and(|(at, to)| same(local, at) && same(foreign, to));
                if Some(local.port()) == port && (raw == Raw::Udp || foreign.port() == 0) {
                    let at = on.entry(gencnt);
                    let made = || (local.ip(), dual, record::reuses(&socket), BTreeSet::new());
                    at.or_insert_with(made).3.insert(pid);
                }
            }
            took = took.or(holds.then_some(pid));
        }
        Ok(Scan { on, took })
    }

    /// Process `pid`, then its parent and theirs, each by its id with its
    /// start and parent, as read now: at most 64, past any process tree, so
    /// parents read while they change end there.
    pub(super) fn up(pid: u32) -> Vec<(u32, u64, u32)> {
        let read = |pid: u32| bsd(pid as i32).map(|(parent, start, _)| (pid, start, parent));
        let next = |&(pid, _, parent): &(u32, u64, u32)| read(parent).filter(|_| parent != pid);
        std::iter::successors(read(pid), next).take(64).collect()
    }

    /// This boot's id ([`super::boot`]): `kern.bootsessionuuid`, none when it
    /// can't be read.
    pub(super) fn boot() -> Option<u64> {
        super::boot(&String::from_utf8_lossy(&kernel(Ask::Boot).ok()?))
    }

    /// Process `pid`'s parent, start and unique id, which no other process
    /// has while this computer runs, by its `proc_bsdinfowithuniqid`.
    pub(super) fn bsd(pid: i32) -> Option<(u32, u64, u64)> {
        let bsd = kernel(Ask::Bsd(pid)).ok()?;
        Some((
            record::ppid(&bsd)?,
            record::start(&bsd)?,
            record::unique(&bsd)?,
        ))
    }

    /// What process `pid` is shown as: its executable's file name, by
    /// `proc_pidpath`, and its folder, by `PROC_PIDVNODEPATHINFO`, each read
    /// only to be shown.
    pub(super) fn shown((pid, start): Process) -> Answer {
        let at = |ask: fn(i32) -> Ask| kernel(ask(pid as i32)).ok();
        let path = String::from_utf8_lossy(&at(Ask::Path).unwrap_or_default()).into_owned();
        let program: String = path.rsplit('/').next().unwrap_or_default().into();
        let folder = at(Ask::Cwd).and_then(|v| record::cwd(&v));
        let folder = folder.map(|c| super::folder(&c));
        Answer::Program {
            pid,
            start,
            program,
            folder,
        }
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
        /// A process's `proc_bsdinfowithuniqid`: `PROC_PIDT_BSDINFOWITHUNIQID`.
        Bsd(i32),
        /// A process's executable's path: `proc_pidpath`.
        Path(i32),
        /// This boot's id, a UUID as text: the `kern.bootsessionuuid` sysctl.
        Boot,
        /// A process's `proc_vnodepathinfo`, its folder's path:
        /// `PROC_PIDVNODEPATHINFO`.
        Cwd(i32),
        /// This account's UDP sockets as ntstat lists them, with a canary of
        /// this process's own made for the read ([`record::stats`]): a new
        /// client of `com.apple.network.statistics`, `ADD_ALL_SRCS` then
        /// `GET_SRC_DESC`, each read to its end, then closed.
        Stats,
    }

    /// `PROC_UID_ONLY`, `PROC_PIDT_BSDINFOWITHUNIQID`, `PROC_PIDFDSOCKETINFO`
    /// and `PROC_PIDVNODEPATHINFO`, which `libc` lacks (`libproc.h`,
    /// `sys/proc_info.h`, `sys/proc_info_private.h`).
    const PROC_UID_ONLY: u32 = 4;
    const PROC_PIDT_BSDINFOWITHUNIQID: i32 = 18;
    const PROC_PIDFDSOCKETINFO: i32 = 3;
    const PROC_PIDVNODEPATHINFO: i32 = 9;
    /// Room for what began between asking a list's size and filling it.
    const SLACK: usize = 4096;
    /// `PROC_PIDPATHINFO_MAXSIZE`, the longest path `proc_pidpath` answers.
    const PATH_LEN: usize = 4096;

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
            Ask::Bsd(pid) | Ask::Cwd(pid) => {
                let (flavor, size) = match ask {
                    Ask::Bsd(_) => (PROC_PIDT_BSDINFOWITHUNIQID, record::BSD_LEN),
                    _ => (PROC_PIDVNODEPATHINFO, record::VNODE_LEN),
                };
                buf = vec![0u8; size];
                // SAFETY: the length passed is `buf`'s, and the kernel refuses
                // a buffer shorter than its record rather than overrun it.
                unsafe { libc::proc_pidinfo(pid, flavor, 0, buf.as_mut_ptr().cast(), len(&buf)) }
            }
            Ask::Socket(pid, fd) => {
                buf = vec![0u8; record::SOCKET_LEN];
                let info = PROC_PIDFDSOCKETINFO;
                // SAFETY: the length passed is `buf`'s, and the kernel refuses
                // a buffer shorter than its record rather than overrun it.
                unsafe { libc::proc_pidfdinfo(pid, fd, info, buf.as_mut_ptr().cast(), len(&buf)) }
            }
            Ask::Stats => {
                // SAFETY: `socket` takes no pointer. The first is the canary,
                // never bound, connected or sent on; the second, ntstat's.
                let canary = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
                // SAFETY: as above.
                let at = unsafe {
                    libc::socket(libc::PF_SYSTEM, libc::SOCK_DGRAM, libc::SYSPROTO_CONTROL)
                };
                let failed = || Err(std::io::Error::last_os_error());
                let read = (|| {
                    if canary < 0 || at < 0 {
                        return failed();
                    }
                    let mut info = libc::ctl_info::default();
                    for (to, from) in info.ctl_name.iter_mut().zip(record::STATS) {
                        *to = *from as libc::c_char;
                    }
                    // SAFETY: `info` is the `ctl_info` CTLIOCGINFO reads and fills.
                    if unsafe { libc::ioctl(at, libc::CTLIOCGINFO, &mut info) } < 0 {
                        return failed();
                    }
                    let to = libc::sockaddr_ctl {
                        sc_len: size_of::<libc::sockaddr_ctl>() as u8,
                        sc_family: libc::AF_SYSTEM as u8,
                        ss_sysaddr: libc::AF_SYS_CONTROL as u16,
                        sc_id: info.ctl_id,
                        sc_unit: 0,
                        sc_reserved: [0; 5],
                    };
                    let len = size_of_val(&to) as libc::socklen_t;
                    // SAFETY: `to` is a `sockaddr_ctl` of `len` bytes.
                    if unsafe { libc::connect(at, (&to as *const libc::sockaddr_ctl).cast(), len) }
                        < 0
                    {
                        return failed();
                    }
                    let opt = |name: c_int, value: *const c_void, len: usize| {
                        // SAFETY: `value` points at the `len` bytes of the option's value.
                        unsafe {
                            libc::setsockopt(
                                at,
                                libc::SOL_SOCKET,
                                name,
                                value,
                                len as libc::socklen_t,
                            ) == 0
                        }
                    };
                    let (room, wait) = (
                        record::STATS_ROOM,
                        libc::timeval {
                            tv_sec: 0,
                            tv_usec: 200_000,
                        },
                    );
                    let room_set = opt(
                        libc::SO_RCVBUF,
                        (&room as *const c_int).cast(),
                        size_of_val(&room),
                    );
                    let waits = opt(
                        libc::SO_RCVTIMEO,
                        (&wait as *const libc::timeval).cast(),
                        16,
                    );
                    let mut given: c_int = 0;
                    let mut given_len = size_of_val(&given) as libc::socklen_t;
                    // SAFETY: it writes at most `given_len` bytes, an int's, into `given`.
                    let read = unsafe {
                        let into = (&mut given as *mut c_int).cast();
                        libc::getsockopt(
                            at,
                            libc::SOL_SOCKET,
                            libc::SO_RCVBUF,
                            into,
                            &mut given_len,
                        )
                    };
                    if !(room_set && waits && read == 0) {
                        return failed();
                    }
                    let mut out = given.unsigned_abs().to_ne_bytes().to_vec();
                    for (context, ask) in record::stats_asks() {
                        // SAFETY: the bytes sent are `ask`'s, of its length.
                        if unsafe { libc::send(at, ask.as_ptr().cast(), ask.len(), 0) }
                            != ask.len() as isize
                        {
                            return failed();
                        }
                        loop {
                            let mut got = vec![0u8; 65_536];
                            // SAFETY: the length passed is `got`'s.
                            let n =
                                unsafe { libc::recv(at, got.as_mut_ptr().cast(), got.len(), 0) };
                            let Ok(n @ 1..) = usize::try_from(n) else {
                                return failed();
                            };
                            got.truncate(n);
                            out.extend((n as u32).to_ne_bytes());
                            out.extend(&got);
                            if record::ended(&got, context) {
                                break;
                            }
                        }
                    }
                    Ok(out)
                })();
                // SAFETY: each is a descriptor this arm opened, closed once.
                unsafe {
                    libc::close(at);
                    libc::close(canary);
                }
                return read;
            }
            Ask::Boot => {
                buf = vec![0u8; 64];
                let mut len = buf.len();
                let (into, none) = (buf.as_mut_ptr().cast(), std::ptr::null_mut());
                // SAFETY: the name ends in a NUL, and `len` is `buf`'s, past
                // which the kernel writes nothing.
                let read = unsafe {
                    libc::sysctlbyname(c"kern.bootsessionuuid".as_ptr(), into, &mut len, none, 0)
                };
                c_int::try_from(len).ok().filter(|_| read == 0).unwrap_or(0)
            }
            Ask::Path(pid) => {
                buf = vec![0u8; PATH_LEN];
                let into = buf.as_mut_ptr().cast();
                // SAFETY: the length passed is `buf`'s.
                unsafe { libc::proc_pidpath(pid, into, len(&buf).unsigned_abs()) }
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
