#![allow(clippy::disallowed_methods, reason = "a test dials what it runs")]
use super::*;

fn at(addr: &str) -> SocketAddr {
    addr.parse().unwrap()
}

/// One `inet_diag_msg` as sock_diag answers it, `family` 2 or 10, its
/// local and remote ports and addresses, cookie, uid and inode, then a
/// `INET_DIAG_CGROUP_ID` attribute when `cgroup` is given, behind an
/// attribute svrngs passes over, as the kernel's padding is.
fn message(
    family: u8,
    (local, peer): (SocketAddr, SocketAddr),
    (cookie, uid, inode): (u64, u32, u32),
    cgroup: Option<u64>,
) -> Vec<u8> {
    let ip = |a: SocketAddr| match a.ip() {
        IpAddr::V4(v4) => [&v4.octets()[..], &[0; 12]].concat(),
        IpAddr::V6(v6) => v6.octets().to_vec(),
    };
    let mut m = vec![family, 10, 0, 0];
    m.extend(local.port().to_be_bytes());
    m.extend(peer.port().to_be_bytes());
    m.extend(ip(local));
    m.extend(ip(peer));
    m.extend([0; 4]);
    m.extend((cookie as u32).to_ne_bytes());
    m.extend(((cookie >> 32) as u32).to_ne_bytes());
    m.extend([0; 12]);
    m.extend(uid.to_ne_bytes());
    m.extend(inode.to_ne_bytes());
    m.extend([8, 0, 15, 0, 0, 0, 0, 0]);
    if let Some(cgroup) = cgroup {
        m.extend([12, 0, 21, 0]);
        m.extend(cgroup.to_ne_bytes());
    }
    m
}

/// `body` as one netlink message of `kind`, padded to 4 bytes.
fn netlink(kind: u16, body: &[u8]) -> Vec<u8> {
    let mut m = u32::try_from(16 + body.len())
        .unwrap()
        .to_ne_bytes()
        .to_vec();
    m.extend(kind.to_ne_bytes());
    m.extend([0; 10]);
    m.extend(body);
    m.resize((m.len() + 3) & !3, 0);
    m
}

/// **sock_diag's answer is read where it holds it** (SHARE_BINDING §5.1):
/// the dump asked is 84 bytes, its filter the local port, going past its
/// end when the port differs; a listener's cookie (both its words), uid,
/// inode and cgroup are read, an IPv6 one's address and `IPV6_V6ONLY` too;
/// a socket with no cgroup attribute reads none; `NLMSG_DONE` ends the
/// dump, and an error, or a message cut short, refuses it. Planted, each
/// watched failing: the cookie's high word dropped (`0x100e` for
/// `0x1_0000_100e`); the attributes read from 68, the inode's offset (no
/// cgroup read); `INET_DIAG_SKV6ONLY` not read (the IPv6-only one dual).
#[test]
fn sock_diag_s_answer_is_read_where_it_holds_it() {
    let asked = diag::request(10, Raw::Udp, u32::MAX, 34197);
    assert_eq!(asked.len(), 84);
    assert_eq!((asked[16], asked[17]), (10, 17), "IPv6, UDP");
    let filter = [12, 0, 1, 0, 11, 8, 12, 0, 0, 0, 0x95, 0x85];
    assert_eq!(
        &asked[72..],
        &filter[..],
        "S_EQ 34197, on 8 or past the end"
    );
    let listener = (at("0.0.0.0:8000"), at("0.0.0.0:0"));
    let v6 = (at("[::]:8000"), at("[::]:0"));
    let mut answer = netlink(
        20,
        &message(2, listener, (0x1_0000_100e, 1000, 5501), Some(4242)),
    );
    let mut only = message(10, v6, (0x700e, 193, 5502), None);
    only.extend([5, 0, 11, 0, 1, 0, 0, 0]);
    answer.extend(netlink(20, &only));
    let (mut found, done) = (Vec::new(), diag::read(&answer, &mut Vec::new()));
    assert_eq!(done, Ok(false), "no NLMSG_DONE yet");
    answer.extend(netlink(3, &0u32.to_ne_bytes()));
    assert_eq!(diag::read(&answer, &mut found), Ok(true));
    let read: Vec<_> = found
        .iter()
        .map(|d| (d.local, d.cookie, d.uid, d.inode, d.cgroup, d.v6only))
        .collect();
    assert_eq!(
        read,
        [
            (listener.0, 0x1_0000_100e, 1000, 5501, Some(4242), false),
            (v6.0, 0x700e, 193, 5502, None, true)
        ]
    );
    assert!(
        diag::read(&netlink(2, &(-22i32).to_ne_bytes()), &mut found).is_err(),
        "NLMSG_ERROR"
    );
    assert!(
        diag::read(&netlink(3, &(-22i32).to_ne_bytes()), &mut found).is_err(),
        "DONE with an error"
    );
    assert!(diag::read(&answer[..40], &mut found).is_err(), "cut short");
}

/// The `socket_fdinfo` `PROC_PIDFDSOCKETINFO` answered for a TCP socket that
/// had dialled 127.0.0.1:61125 from 127.0.0.1:61126, and for a UDP socket on
/// 127.0.0.1:63655 (2026-11-18, a C program), kernel pointers zeroed.
fn record(protocol: &str) -> Vec<u8> {
    let hex = match protocol {
        "tcp" => include_str!("fixtures/socket_fdinfo.tcp.hex"),
        _ => include_str!("fixtures/socket_fdinfo.udp.hex"),
    };
    hex::decode(hex.trim()).unwrap()
}

/// **libproc's records are read where they hold it** (LEND_OWNER §5, 2;
/// SHARE_BINDING §5.1): a captured TCP socket's local and foreign address
/// and its handle; a UDP socket's, bound and connected to none, and the same
/// with both `insi_vflag` flags on every address, read as IPv6's and dual;
/// a socket of neither, none; the `proc_bsdinfo` a process read of itself,
/// uid 502, its parent and its start in 2026, and a record of another
/// length, none; a `proc_vnodepathinfo`'s folder. Planted, each watched
/// failing: the local and foreign addresses' offsets swapped (the ends read
/// backwards); the uid read at 24, `pbi_gid` (20); the generation count
/// read at 160, `soi_so`, the handle libproc names a socket by in one read
/// (another number);
/// both flags read as IPv4 (the dual socket read as 0.0.0.0, not dual).
#[test]
fn libproc_s_records_are_read_where_they_hold_it() {
    assert_eq!(
        record("tcp").len(),
        record::SOCKET_LEN,
        "the capture's length"
    );
    let mut tcp = record("tcp");
    tcp[272..280].copy_from_slice(&0x602d_d91e_a0a6_f331u64.to_ne_bytes());
    let dialled = (at("127.0.0.1:61126"), at("127.0.0.1:61125"));
    assert_eq!(
        record::socket(&tcp),
        Some((Raw::Tcp, 0x602d_d91e_a0a6_f331, dialled, false))
    );
    // Its generation count as captured.
    let udp = record::socket(&record("udp"));
    let bound = (at("127.0.0.1:63655"), at("0.0.0.0:0"));
    assert_eq!(udp, Some((Raw::Udp, 71_345_560, bound, false)));
    // The same socket on every IPv6 address, taking IPv4's too: both flags.
    let mut dual = record("udp");
    dual[288] = 3;
    dual[312..328].fill(0);
    let every = (at("[::]:63655"), at("[::]:0"));
    assert_eq!(
        record::socket(&dual),
        Some((Raw::Udp, 71_345_560, every, true))
    );
    let mut unix = record("tcp");
    unix[256..260].copy_from_slice(&0u32.to_ne_bytes());
    assert_eq!(record::socket(&unix), None);
    // Its `soi_options`: the capture's none, then `SO_REUSEPORT` with
    // `SO_ACCEPTCONN` beside it, as a listener on this Mac read (0x202).
    let mut shared = record("udp");
    assert!(!record::reuses(&shared), "the capture's options are none");
    shared[188..190].copy_from_slice(&0x202u16.to_ne_bytes());
    assert!(record::reuses(&shared), "SO_REUSEPORT read");
    // The capture's `proc_bsdinfo`, then a `proc_uniqidentifierinfo`: its
    // UUID, then its unique id.
    let mut bsd = hex::decode(include_str!("fixtures/proc_bsdinfo.hex").trim()).unwrap();
    bsd.extend([[0; 16].as_slice(), &2_853_096u64.to_ne_bytes(), &[0; 32]].concat());
    assert_eq!(record::unique(&bsd), Some(2_853_096));
    assert_eq!(record::unique(&bsd[..136]), None);
    assert_eq!(record::uid(&bsd), Some(502));
    assert_eq!(record::uid(&bsd[..128]), None);
    assert_eq!(record::ppid(&bsd), Some(75476));
    let year = 1_767_225_600..1_798_761_600;
    let start = record::start(&bsd).map(|us| us / 1_000_000);
    assert!(start.is_some_and(|s| year.contains(&s)), "{start:?}");
    assert_eq!(record::start(&bsd[..128]), None);
    let mut vnode = vec![0u8; record::VNODE_LEN];
    vnode[152..166].copy_from_slice(b"/Users/a/dev/g");
    assert_eq!(record::cwd(&vnode), Some("/Users/a/dev/g".into()));
    assert_eq!(record::cwd(&vnode[..100]), None);
}

/// **This computer answers whose each end of a connection made here is**:
/// a listener and the socket that dialled it, each end ours, the uid that
/// owns the folder this test made, which [`mine`] says too. On a Mac also:
/// a connection its listener accepts 30 ms late, in no process's files
/// until then, found within [`ACCEPT_WITHIN`]; one never accepted is not
/// this account's. Read as this computer reads them, so it passes on each
/// only with its reader right. Planted on the Mac, watched failing: one
/// scan, no look again (the late accept not this account's).
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn this_computer_answers_whose_each_end_of_a_connection_made_here_is() {
    use std::os::unix::fs::MetadataExt as _;
    let dir = tempfile::tempdir().unwrap();
    let uid = std::fs::metadata(dir.path()).unwrap().uid();
    assert_eq!(mine(), Ok(uid));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let dialled = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let _accepted = listener.accept().unwrap();
    let made = Conn {
        local: dialled.local_addr().unwrap(),
        peer: dialled.peer_addr().unwrap(),
    };
    assert_eq!(owner(made, Side::Peer), Owner::Uid(uid));
    assert_eq!(owner(made, Side::Local), Owner::Uid(uid));
    #[cfg(target_os = "macos")]
    {
        let late = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(30));
            listener.accept().map(|(kept, _)| (listener, kept))
        });
        let queued = std::net::TcpStream::connect(made.peer).unwrap();
        let queued = Conn {
            local: queued.local_addr().unwrap(),
            peer: queued.peer_addr().unwrap(),
        };
        assert_eq!(owner(queued, Side::Peer), Owner::Uid(uid), "accepted late");
        let (listener, _kept) = late.join().unwrap().unwrap();
        let never = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let never = Conn {
            local: never.local_addr().unwrap(),
            peer: never.peer_addr().unwrap(),
        };
        assert_eq!(owner(never, Side::Peer), Owner::NotMine, "never accepted");
    }
}

/// A Python program, as its first argument says: `tcp` a listener on
/// 127.0.0.1, `two` the same with a child holding it too, `handoff` the
/// same with a child that doesn't, to which it hands each connection it
/// accepts, as Node cluster's primary does, the child writing `!` on it
/// once the parent has let go; `udp` a UDP socket bound to every address
/// alone, `reused` one with `SO_REUSEADDR`, `shared` one with
/// `SO_REUSEPORT`, `beside` one on 127.0.0.1 with
/// `SO_REUSEADDR` and `SO_REUSEPORT`, as F5 binds beside it on a Mac;
/// `dual` a UDP socket on every IPv6 address taking IPv4's too, `v6only`
/// one taking IPv6's alone; `reuseport` a TCP listener with `SO_REUSEPORT`,
/// and `joins` one that, the first connection it accepts, closes its
/// listener and prints `took`; `relisten` a listener that, sent `!` on a
/// connection it accepts, closes its listener and listens again on the
/// same port; `prefork` a listener a child accepts on, as a prefork server's
/// workers do; on the port its second argument names, else
/// any. It prints its port and its child's pid (0 for none), and keeps each
/// connection it accepts.
#[cfg(any(target_os = "linux", target_os = "macos"))]
const SERVE: &str = r#"
import os, socket, sys, time
how, port = sys.argv[1], int(sys.argv[2] or 0)
udp = how in ("udp", "beside", "reused", "shared", "inflight", "dual", "v6only")
six = how in ("dual", "v6only")
s = socket.socket(socket.AF_INET6 if six else socket.AF_INET, socket.SOCK_DGRAM if udp else socket.SOCK_STREAM)
if six:
    s.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, how == "v6only")
if how in ("tcp", "two", "beside", "reused", "inflight", "relisten"):
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
if how in ("beside", "reuseport", "joins", "inflight", "shared"):
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
if how == "deferred":
    s.setsockopt(socket.IPPROTO_TCP, socket.TCP_DEFER_ACCEPT, 5)
s.bind(("::" if six else "0.0.0.0" if how in ("udp", "reused", "shared") else "127.0.0.1", port))
bound = s.getsockname()[1]
if not udp:
    s.listen(8)
if how == "inflight":
    kept, carrier = socket.socketpair()
    socket.send_fds(kept, [b"x"], [s.fileno()])
    s.close()
child = 0
if how == "two":
    s.set_inheritable(True)
    ran, done = os.pipe()
    child = os.fork()
    if child == 0:
        os.close(ran)
        os.write(done, b"1")
        time.sleep(60)
        os._exit(0)
    os.close(done)
    os.read(ran, 1)
if how == "handoff":
    near, far = socket.socketpair()
    child = os.fork()
    if child == 0:
        s.close()
        while True:
            _, fds, _, _ = socket.recv_fds(far, 1, 1)
            far.recv(1)
            os.write(fds[0], b"!")
if how == "prefork":
    child = os.fork()
    while child == 0:
        taken = s.accept()
print(bound, child, flush=True)
kept = []
while True:
    if udp or how == "prefork":
        time.sleep(60)
    kept.append(s.accept())
    if how == "relisten":
        kept[-1][0].settimeout(1)
        try:
            again = kept[-1][0].recv(1) == b"!"
        except OSError:
            again = False
    if how == "relisten" and again:
        s.close()
        s = socket.socket()
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("127.0.0.1", bound))
        s.listen(8)
    if how == "handoff":
        taken = kept.pop()[0]
        socket.send_fds(near, [b"x"], [taken.fileno()])
        taken.close()
        near.send(b"!")
    if how == "joins":
        s.close()
        print("took", flush=True)
        time.sleep(60)
"#;

/// A running [`SERVE`], its port and its child's pid; its group killed
/// when dropped, its child with it.
#[cfg(any(target_os = "linux", target_os = "macos"))]
struct Serving(std::process::Child, u16, u32);

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl Serving {
    fn new(how: &str, port: Option<u16>) -> Self {
        use std::io::BufRead as _;
        use std::os::unix::process::CommandExt as _;
        let port = port.map(|p| p.to_string()).unwrap_or_default();
        let mut python = std::process::Command::new("python3")
            .args(["-c", SERVE, how, &port])
            .stdout(std::process::Stdio::piped())
            .process_group(0)
            .spawn()
            .expect("python3 runs here");
        let mut said = String::new();
        let out = python.stdout.take().unwrap();
        std::io::BufReader::new(out).read_line(&mut said).unwrap();
        let (port, child) = said.trim().split_once(' ').expect("a port and a pid");
        Self(python, port.parse().unwrap(), child.parse().unwrap())
    }

    /// The process it runs as, by its pid and start now.
    fn pid(&self) -> (u32, u64) {
        (self.0.id(), start(self.0.id()).expect("its start reads"))
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl Drop for Serving {
    fn drop(&mut self) {
        let group = format!("-{}", self.0.id());
        let _ = std::process::Command::new("kill")
            .args(["-9", "--", &group])
            .status();
        let _ = self.0.wait();
    }
}

/// What `answering` reads of `at`'s port approved, with `pid` when given.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn approved(
    raw: Raw,
    at: &Serving,
    pid: Option<(u32, u64)>,
) -> Result<binding::Approved, binding::Refused> {
    let read = answering(raw, at.1, pid.map(|p| p.0)).expect("the port reads");
    let chosen = pid.map(|(pid, start)| binding::Chosen {
        pid,
        start,
        now: read.start,
        parents: read.parents.clone(),
    });
    binding::approve(&read.read, chosen.as_ref())
}

/// `at`'s TCP port dialled and read as the door reads it, then checked
/// against `approved`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn carried(approved: &binding::Approved, at: &Serving) -> Result<(), binding::Held> {
    carried_at(approved, at.1)
}

/// [`carried`], to `port`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn carried_at(approved: &binding::Approved, port: u16) -> Result<(), binding::Held> {
    let dialled = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    let conn = Conn {
        local: dialled.local_addr().unwrap(),
        peer: dialled.peer_addr().unwrap(),
    };
    let (read, took) = super::port(Raw::Tcp, port, Some(conn)).expect("the port reads");
    binding::check(Some(approved), &read, &took)
}

/// **A share's port is approved as it answers, and held once it changes**
/// (SHARE_BINDING §5.1-§5.3, §9), as this computer reads it: a Python
/// listener is shown as `python3` in this test's folder, approved with its
/// own pid and start, and a connection to it carried; started again on the
/// same port, the same command, it is held; with a stale start refused as
/// stopped; with a sibling's pid, which holds none of its sockets, refused
/// as another's. A listener its child holds too is approved by its parent's
/// pid, and a connection the child accepts carried. A UDP socket is
/// approved and its datagrams carried; one beside it on the same address
/// (F5) holds them. Planted, each watched failing: the holder check
/// skipped in `binding::approve` (the sibling's pid approved); on the Fedora
/// box, the cgroup check skipped in `binding::check` (an accept in another
/// cgroup carried, uid_tests.rs `a_listener_joined_by_reuseport...`).
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn a_share_s_port_is_approved_as_it_answers_and_held_once_it_changes() {
    let one = Serving::new("tcp", None);
    let read = answering(Raw::Tcp, one.1, None).unwrap();
    let shown = format!("{:?}", read.shown);
    assert!(
        shown.contains("python3") || shown.contains("Python"),
        "{shown}"
    );
    let approved = approved(Raw::Tcp, &one, Some(one.pid())).unwrap();
    assert_eq!(carried(&approved, &one), Ok(()));
    // A sibling of it, which holds none of its sockets, nor an ancestor of
    // them (this test is one: its child's sockets are its own to lend).
    let mut sleep = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .unwrap();
    let sibling = (sleep.id(), start(sleep.id()).unwrap());
    let refused = super::tests::approved(Raw::Tcp, &one, Some(sibling));
    let _ = (sleep.kill(), sleep.wait());
    assert_eq!(refused, Err(binding::Refused::Another));
    let stale = (one.pid().0, one.pid().1 + 1);
    assert_eq!(
        super::tests::approved(Raw::Tcp, &one, Some(stale)),
        Err(binding::Refused::Stopped)
    );
    let port = one.1;
    drop(one);
    let again = Serving::new("tcp", Some(port));
    assert_eq!(
        carried(&approved, &again),
        Err(binding::Held::Another),
        "started again"
    );
    drop(again);
    let two = Serving::new("two", None);
    assert_ne!(two.2, 0, "a child");
    let parent = super::tests::approved(Raw::Tcp, &two, Some(two.pid())).unwrap();
    assert_eq!(carried(&parent, &two), Ok(()));
    drop(two);
    let udp = Serving::new("udp", None);
    let datagrams = super::tests::approved(Raw::Udp, &udp, Some(udp.pid())).unwrap();
    let (read, took) = super::port(Raw::Udp, udp.1, None).unwrap();
    assert_eq!(binding::check(Some(&datagrams), &read, &took), Ok(()));
    // A Mac lets a program of this account bind a more specific address
    // beside a wildcard UDP socket and take its datagrams (F5): it is read,
    // and holds them.
    #[cfg(target_os = "macos")]
    {
        let beside = Serving::new("beside", Some(udp.1));
        let (read, took) = super::port(Raw::Udp, udp.1, None).unwrap();
        assert_eq!(read.sockets.len(), 2, "both read: {read:?}");
        let held = binding::check(Some(&datagrams), &read, &took);
        assert_eq!(held, Err(binding::Held::Another), "a socket beside");
        drop(beside);
    }
    drop(udp);
}

/// **A connection its approved program hands to a process it started is
/// carried** (SHARE_BINDING_REVIEWS §7 #1; Node cluster's round-robin): a
/// listener on 127.0.0.1, approved by its pid, whose child holds no
/// listener; it hands the door's connection to the child, which says so on
/// it once its parent has let go, and the check made then finds the child
/// alone holding it, and carries it. Planted on this Mac, watched failing:
/// the acceptor alone compared, not its parents (held).
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn a_connection_handed_to_a_process_its_program_started_is_carried() {
    use std::io::Read as _;
    let handoff = Serving::new("handoff", None);
    assert_ne!(handoff.2, 0, "a child");
    let approved = approved(Raw::Tcp, &handoff, Some(handoff.pid())).unwrap();
    let mut dialled = std::net::TcpStream::connect(("127.0.0.1", handoff.1)).unwrap();
    let conn = Conn {
        local: dialled.local_addr().unwrap(),
        peer: dialled.peer_addr().unwrap(),
    };
    dialled.read_exact(&mut [0; 1]).expect("the child has it");
    let (read, took) = port(Raw::Tcp, handoff.1, Some(conn)).expect("the port reads");
    let checked = binding::check(Some(&approved), &read, &took);
    assert_eq!(checked, Ok(()), "{took:?}");
}

/// **A listener the approved process makes again on the port is held, and
/// a prefork worker's accept is carried** (SHARE_BINDING_REVIEWS §9 #1): a
/// TCP socket is approved by an id that lasts its life, Linux's cookie and
/// a Mac's generation count, so a program that closes its listener and
/// listens again on the same port, as Podman's gvproxy does for each new
/// container, is held once it does; the listener a prefork server's child
/// accepts on, approved by its parent's pid, carries what the child
/// accepts. Planted, watched failing on this Mac: a Mac's TCP read not
/// lasting (the new listener carried, as the same process's).
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn a_listener_the_approved_process_makes_again_is_held() {
    let one = Serving::new("relisten", None);
    let approved = approved(Raw::Tcp, &one, Some(one.pid())).unwrap();
    let mut asked = std::net::TcpStream::connect(("127.0.0.1", one.1)).unwrap();
    std::io::Write::write_all(&mut asked, b"!").unwrap();
    let made = |r: &binding::Read| r.sockets.iter().any(|s| !approved.sockets.contains(&s.id));
    let again = (0..200).find(|_| {
        std::thread::sleep(std::time::Duration::from_millis(10));
        port(Raw::Tcp, one.1, None).is_ok_and(|(read, _)| made(&read))
    });
    assert!(again.is_some(), "it listens again");
    assert_eq!(
        carried(&approved, &one),
        Err(binding::Held::Another),
        "the same process's new listener"
    );
    let prefork = Serving::new("prefork", None);
    assert_ne!(prefork.2, 0, "a worker");
    let parent = super::tests::approved(Raw::Tcp, &prefork, Some(prefork.pid())).unwrap();
    assert_eq!(carried(&parent, &prefork), Ok(()), "a worker's accept");
}

/// **A Mac's TCP sockets are read in this boot** (SHARE_BINDING_REVIEWS §9
/// #1): its generation counts start again when the Mac does, so an approval
/// names its boot, `kern.bootsessionuuid`'s first 64 bits, as Linux's names
/// its boot id's; one made in a boot read as another is held. Planted,
/// watched failing: a boot reader answering one value for every boot (an
/// approval from another boot carried).
#[cfg(target_os = "macos")]
#[test]
fn a_mac_s_tcp_sockets_are_read_in_this_boot() {
    let one = Serving::new("tcp", None);
    let mut approved = approved(Raw::Tcp, &one, Some(one.pid())).unwrap();
    assert_eq!(carried(&approved, &one), Ok(()), "this boot's");
    let this = approved.boot;
    approved.boot = Some(0);
    assert_eq!(
        carried(&approved, &one),
        Err(binding::Held::Another),
        "another boot's"
    );
    let said = std::process::Command::new("sysctl")
        .args(["-n", "kern.bootsessionuuid"])
        .output()
        .unwrap();
    assert_eq!(this, boot(&String::from_utf8_lossy(&said.stdout)));
}

/// **A new container behind Podman's forwarder is held** (SHARE_BINDING_REVIEWS
/// §9 #1): on a Mac what answers on a container's published port is Podman's
/// gvproxy, one process for every container; a container lent on
/// `127.0.0.1:P:80` is approved by the listener gvproxy holds for it, so
/// another container run on P after it is held. Needs Podman's machine and
/// busybox; the containers it runs are removed.
#[cfg(target_os = "macos")]
#[test]
#[ignore = "runs Podman containers: run on a Mac by hand"]
fn a_new_container_behind_podman_s_forwarder_is_held() {
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = free.local_addr().unwrap().port();
    drop(free);
    let published = format!("127.0.0.1:{port}:80");
    let run = |name: &str, does: &[&str]| {
        let _ = std::process::Command::new("podman")
            .args(["rm", "-f", name])
            .output();
        let ran = std::process::Command::new("podman")
            .args([
                "run",
                "-d",
                "--name",
                name,
                "-p",
                &published,
                "docker.io/library/busybox",
            ])
            .args(does)
            .output()
            .unwrap();
        assert!(ran.status.success(), "{ran:?}");
        let up = (0..100).find(|_| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            answering(Raw::Tcp, port, None).is_ok_and(|a| !a.read.sockets.is_empty())
        });
        assert!(up.is_some(), "{name} answers on {port}");
    };
    let gone = |name: &str| {
        let _ = std::process::Command::new("podman")
            .args(["rm", "-f", name])
            .output();
    };
    run("svrngs-uid-test-a", &["httpd", "-f", "-p", "80"]);
    let read = answering(Raw::Tcp, port, None).unwrap();
    eprintln!("first: {:?} shown {:?}", read.read, read.shown);
    let first = binding::approve(&read.read, None).unwrap();
    let carried_first = carried_at(&first, port);
    gone("svrngs-uid-test-a");
    run(
        "svrngs-uid-test-b",
        &["sh", "-c", "while true; do echo other | nc -l -p 80; done"],
    );
    let read = answering(Raw::Tcp, port, None).unwrap();
    eprintln!("second: {:?} shown {:?}", read.read, read.shown);
    let carried_second = carried_at(&first, port);
    gone("svrngs-uid-test-b");
    assert_eq!(carried_first, Ok(()), "the container approved");
    assert_eq!(
        carried_second,
        Err(binding::Held::Another),
        "the container after it"
    );
}

/// **A connected UDP socket is read with the one address it is connected
/// to** (eudoxia `Socket::peer`), on Linux by sock_diag's `idiag_dst` and
/// `idiag_dport`, so a server's socket for one of svrngs's flows is known by
/// it, and one connected to nothing is read so; a Mac's read can't tell.
/// Planted, watched failing on the Fedora box: `peer` left unread on Linux.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn a_connected_udp_socket_is_read_with_its_peer() {
    let far = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let near = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    near.connect(far.local_addr().unwrap()).unwrap();
    let peer = |s: &std::net::UdpSocket| {
        let (read, _) = port(Raw::Udp, s.local_addr().unwrap().port(), None).unwrap();
        read.sockets.first().map(|s| s.peer)
    };
    let linux = cfg!(target_os = "linux");
    let (connected, unconnected) = match linux {
        true => (
            binding::Peer::To(far.local_addr().unwrap()),
            binding::Peer::Unconnected,
        ),
        false => (binding::Peer::Unread, binding::Peer::Unread),
    };
    assert_eq!(peer(&near), Some(connected), "connected");
    assert_eq!(peer(&far), Some(unconnected), "not connected");
}

/// **A connected socket names a flow only where its cgroup names its
/// program, and only of this account** (SHARE_BINDING_REVIEWS §10 A): its
/// peer is read where svrngs runs in a cgroup v2 of its own and the socket
/// is this account's, else unread, so a check of a flow holds it; one
/// connected to nothing there is read so, and a check of a flow beside it is
/// unjudged. Planted, each watched failing: the cgroup asked of nothing (one
/// of this account's on a cgroup v1 computer carried); the account asked of
/// nothing (another account's in the approved cgroup carried).
#[test]
fn a_connected_socket_names_a_flow_only_as_this_account_s_in_its_own_cgroup() {
    let connected = |uid| diag::Diag {
        state: 7,
        local: at("127.0.0.1:3000"),
        peer: at("127.0.0.1:40000"),
        uid,
        inode: 9,
        cookie: 2,
        cgroup: Some(4),
        v6only: false,
    };
    let v2 = "0::/user.slice/user-1000.slice/session-2.scope\n";
    let flow = binding::Took::Flow {
        named: [at("127.0.0.1:40000")].into(),
        new: [at("127.0.0.1:40000")].into(),
    };
    let socket = |id, peer| binding::Socket {
        id,
        cgroup: Some(4),
        holders: Default::default(),
        at: "127.0.0.1".parse().unwrap(),
        dual: false,
        peer,
    };
    let read = |peer| binding::Read {
        boot: None,
        lasting: true,
        sockets: vec![socket(1, binding::Peer::Unread), socket(2, peer)],
        unlisted: false,
    };
    let approved = binding::approve(&read(binding::Peer::Unread), None).unwrap();
    let approved = binding::Approved {
        sockets: [1].into(),
        ..approved
    };
    let carried = |d: &diag::Diag, cgroups| {
        let peer = peer(d, Raw::Udp, cgroups, Some(1000));
        binding::check(Some(&approved), &read(peer), &flow)
    };
    assert_eq!(
        carried(&connected(1000), v2),
        Ok(()),
        "this account's, in its cgroup"
    );
    let v1 = "12:pids:/user.slice\n1:name=systemd:/user.slice/session-2.scope\n";
    for (case, d, cgroups) in [
        ("cgroup v1 alone", connected(1000), v1),
        (
            "every program in the root cgroup",
            connected(1000),
            "0::/\n",
        ),
        ("another account's", connected(1001), v2),
    ] {
        assert_eq!(carried(&d, cgroups), Err(binding::Held::Another), "{case}");
    }
    assert_eq!(
        peer(&connected(1000), Raw::Tcp, v2, Some(1000)),
        binding::Peer::Unread,
        "TCP"
    );
    // Connected to nothing, between a server's bind and its connect: read
    // so only as this account's in its cgroup, and judged unjudged there.
    let unconnected = |uid| diag::Diag {
        peer: at("0.0.0.0:0"),
        ..connected(uid)
    };
    let window = |d: &diag::Diag, cgroups| {
        let peer = peer(d, Raw::Udp, cgroups, Some(1000));
        (peer, binding::check(Some(&approved), &read(peer), &flow))
    };
    let unjudged = (binding::Peer::Unconnected, Err(binding::Held::Unjudged));
    assert_eq!(
        window(&unconnected(1000), v2),
        unjudged,
        "this account's, connecting"
    );
    let unread = (binding::Peer::Unread, Err(binding::Held::Another));
    assert_eq!(window(&unconnected(1001), v2), unread, "another account's");
    assert_eq!(window(&unconnected(1000), v1), unread, "cgroup v1 alone");
}

/// **A UDP socket on every IPv6 address is reached at 127.0.0.1 when it
/// takes IPv4's too, and at ::1 when it doesn't** (SHARE_BINDING_REVIEWS §7
/// #2), as this computer reads it: sock_diag's `INET_DIAG_SKV6ONLY` on
/// Linux, libproc's `insi_vflag` on a Mac; the dual one's datagrams are
/// carried. Planted, each watched failing: on this Mac, the approval's
/// libproc read skipped (the IPv6-only one reached at 127.0.0.1); on the
/// Fedora box, `INET_DIAG_SKV6ONLY` not read (the same).
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn a_udp_socket_on_every_ipv6_address_is_reached_as_it_takes_ipv4_or_not() {
    let dual = Serving::new("dual", None);
    let approved = approved(Raw::Udp, &dual, Some(dual.pid())).unwrap();
    assert_eq!(approved.at.to_string(), "127.0.0.1", "dual");
    let (read, took) = port(Raw::Udp, dual.1, None).unwrap();
    assert_eq!(binding::check(Some(&approved), &read, &took), Ok(()));
    let alone = Serving::new("v6only", None);
    let approved = super::tests::approved(Raw::Udp, &alone, None).unwrap();
    assert_eq!(approved.at.to_string(), "::1", "IPv6 alone");
}

/// **On a Mac, a UDP socket on every address that lets others bind beside
/// it isn't approved** (SHARE_BINDING_REVIEWS §9 #2): another account may
/// bind beside it there (in_pcb.c:1016-1031), and the Mac's read lists only
/// this account's sockets, so `share` refuses it, saying to listen on
/// 127.0.0.1 for svrngs. One on every address alone, and one on 127.0.0.1
/// with `SO_REUSEPORT`, beside which no other account can bind, are
/// approved. Planted, each watched failing: the options read at 186, not
/// 188 (the shared one approved); the address not asked (the one on
/// 127.0.0.1 refused); `share::approve` not asking (the shared one
/// approved).
#[cfg(target_os = "macos")]
#[test]
fn a_udp_socket_others_may_bind_beside_isn_t_approved_on_a_mac() {
    let said = |how| {
        let at = Serving::new(how, None);
        let read = answering(Raw::Udp, at.1, None).expect("the port reads");
        let port = std::num::NonZeroU16::new(at.1).unwrap();
        let approved = crate::share::approve((port, Raw::Udp), Ok(read), None, None);
        approved
            .map(|_| ())
            .map_err(|e| e.to_string().replace(&at.1.to_string(), "P"))
    };
    let refused = "svrngs can't share UDP port P: the program there listens on every address and lets other programs listen on the port beside it, so on a Mac svrngs can't keep the share to it alone. Have it listen on 127.0.0.1 for svrngs, then share it again.";
    assert_eq!(
        said("shared"),
        Err(refused.into()),
        "0.0.0.0 with SO_REUSEPORT"
    );
    assert_eq!(said("udp"), Ok(()), "0.0.0.0 alone");
    assert_eq!(said("beside"), Ok(()), "127.0.0.1 with SO_REUSEPORT");
}

/// `how` [`SERVE`] on `port`, run in a systemd user scope of its own, so in
/// a cgroup of its own, and its lines.
#[cfg(target_os = "linux")]
fn scoped(how: &str, port: u16) -> (std::process::Child, std::sync::mpsc::Receiver<String>) {
    use std::io::BufRead as _;
    let mut child = std::process::Command::new("systemd-run")
        .args([
            "--user",
            "--scope",
            "-q",
            "python3",
            "-c",
            SERVE,
            how,
            &port.to_string(),
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("systemd-run runs here");
    let (tell, told) = std::sync::mpsc::channel();
    let out = std::io::BufReader::new(child.stdout.take().unwrap());
    std::thread::spawn(move || {
        out.lines()
            .map_while(Result::ok)
            .for_each(|l| drop(tell.send(l)))
    });
    told.recv_timeout(std::time::Duration::from_secs(10))
        .expect("it listens");
    (child, told)
}

/// **On Linux a listener that joins with `SO_REUSEPORT` is held, and so is a
/// connection it took before it closed** (SHARE_BINDING §5.2 (a) and (b),
/// §9): A listens with `SO_REUSEPORT` in a scope of its own and is approved;
/// B joins in another scope: its listener is read, and a connection is
/// held by (a). B, having taken a connection, closes its listener: the
/// door's check of that connection, made after it, finds only A listening,
/// and holds it by (b), the cgroup B's. The pause between connect and check
/// is this test's own: it checks once B says it took the connection.
/// Planted, watched failing on the Fedora box: the cgroup check skipped in
/// `binding::check` (B's connection carried).
#[cfg(target_os = "linux")]
#[test]
#[ignore = "drives systemd-run --user scopes: run on Linux by hand"]
fn a_listener_joining_with_reuseport_is_held_and_so_is_its_accept() {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let (mut a, _) = scoped("reuseport", port);
    let approved = binding::approve(&answering(Raw::Tcp, port, None).unwrap().read, None).unwrap();
    let (mut b, took) = scoped("joins", port);
    let dial = || {
        let dialled = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        let conn = Conn {
            local: dialled.local_addr().unwrap(),
            peer: dialled.peer_addr().unwrap(),
        };
        (dialled, conn)
    };
    // Read before any connection, which B would take and then close on.
    let (read, by) = super::port(Raw::Tcp, port, None).unwrap();
    assert_eq!(read.sockets.len(), 2, "both listeners read");
    let joined = binding::check(Some(&approved), &read, &by);
    assert_eq!(joined, Err(binding::Held::Another), "(a)");
    let wait = std::time::Duration::from_millis(300);
    let mut held = None;
    for _ in 0..64 {
        let (dialled, conn) = dial();
        if took.recv_timeout(wait).is_ok() {
            let (read, by) = super::port(Raw::Tcp, port, Some(conn)).unwrap();
            assert_eq!(read.sockets.len(), 1, "B's listener closed: {read:?}");
            held = Some((binding::check(Some(&approved), &read, &by), dialled));
            break;
        }
    }
    let _ = (a.kill(), b.kill(), a.wait(), b.wait());
    let (held, _dialled) = held.expect("B took a connection within 64");
    assert_eq!(
        held,
        Err(binding::Held::Another),
        "(b): accepted in B's cgroup"
    );
}

/// **On Linux a server that waits on the first bytes is carried**
/// (SHARE_BINDING_REVIEWS §6 #5): a listener with `TCP_DEFER_ACCEPT` keeps
/// the door's connection a request until data comes (tcp_minisocks.c
/// `tcp_check_req`), listed in SYN-RECV with no cgroup; approved alone on
/// its port, it is read as the listener's to take ([`Took::Waiting`]) and
/// carried, and the bytes reach it. Planted, watched failing on the Fedora
/// box: a request taken for an unread cgroup (held unjudged, as before).
#[cfg(target_os = "linux")]
#[test]
fn a_server_waiting_on_the_first_bytes_is_carried_on_linux() {
    use std::io::Write as _;
    let deferred = Serving::new("deferred", None);
    let read = answering(Raw::Tcp, deferred.1, None).unwrap().read;
    let approved = binding::approve(&read, None).unwrap();
    let mut dialled = std::net::TcpStream::connect(("127.0.0.1", deferred.1)).unwrap();
    let conn = Conn {
        local: dialled.local_addr().unwrap(),
        peer: dialled.peer_addr().unwrap(),
    };
    let (now, took) = super::port(Raw::Tcp, deferred.1, Some(conn)).unwrap();
    assert_eq!(took, Took::Waiting);
    assert_eq!(binding::check(Some(&approved), &now, &took), Ok(()));
    dialled.write_all(b"GET").unwrap();
}

/// A listener on `port` with `SO_REUSEPORT` and `TCP_DEFER_ACCEPT`, run in a
/// systemd user scope of its own, as its stdin says: `listen`, `shut`
/// (`shutdown`, which stops it listening and keeps it bound), and `take`
/// (accept, and say what came first). Each said back once done.
#[cfg(target_os = "linux")]
const CYCLE: &str = r#"
import socket, sys
s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
s.setsockopt(socket.IPPROTO_TCP, socket.TCP_DEFER_ACCEPT, 5)
s.bind(("127.0.0.1", int(sys.argv[1])))
print("bound", flush=True)
for line in sys.stdin:
    said = line.strip()
    if said == "listen":
        s.listen(8)
    elif said == "shut":
        s.shutdown(socket.SHUT_RD)
    elif said == "take":
        c, _ = s.accept()
        c.settimeout(5)
        said = "took " + c.recv(64).decode()
    print(said, flush=True)
"#;

/// [`CYCLE`] on `port`: a function that tells it a word and waits for it
/// said back, and its process, to kill.
#[cfg(target_os = "linux")]
fn cycling(port: u16) -> (impl FnMut(&str) -> String, std::process::Child) {
    use std::io::{BufRead as _, Write as _};
    let mut child = std::process::Command::new("systemd-run")
        .args([
            "--user",
            "--scope",
            "-q",
            "python3",
            "-c",
            CYCLE,
            &port.to_string(),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("systemd-run runs here");
    let mut told = std::io::BufReader::new(child.stdout.take().unwrap()).lines();
    let mut stdin = child.stdin.take().unwrap();
    assert_eq!(told.next().unwrap().unwrap(), "bound");
    let tell = move |word: &str| {
        writeln!(stdin, "{word}").unwrap();
        told.next().unwrap().unwrap()
    };
    (tell, child)
}

/// **On Linux a listener that stopped listening is read, since it may take
/// the door's waiting connection when it listens again**
/// (SHARE_BINDING_REVIEWS §6 #5): A and B, `SO_REUSEPORT` and
/// `TCP_DEFER_ACCEPT`, each in a scope of its own; A approved. A stops
/// listening, B listens alone, and the door connects: its handshake reaches
/// B, and waits. B stops listening and A listens again: the check, made
/// now, reads A listening and B bound, and holds it. Had it carried, B
/// listening again takes the door's bytes: a request is made a socket by
/// the listener its handshake reached once that one listens (tcp_ipv4.c
/// `tcp_v4_rcv`), which this test shows. Planted, watched failing on the
/// Fedora box: bound sockets left out of the read (the check carried).
#[cfg(target_os = "linux")]
#[test]
#[ignore = "drives systemd-run --user scopes: run on Linux by hand"]
fn a_listener_that_stopped_listening_is_read_since_it_may_listen_again() {
    use std::io::Write as _;
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = free.local_addr().unwrap().port();
    drop(free);
    let (mut a, mut a_runs) = cycling(port);
    a("listen");
    let approved = binding::approve(&super::port(Raw::Tcp, port, None).unwrap().0, None);
    let approved = approved.unwrap();
    let (mut b, mut b_runs) = cycling(port);
    a("shut");
    b("listen");
    let mut dialled = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    let conn = Conn {
        local: dialled.local_addr().unwrap(),
        peer: dialled.peer_addr().unwrap(),
    };
    b("shut");
    a("listen");
    let (read, took) = super::port(Raw::Tcp, port, Some(conn)).unwrap();
    let checked = binding::check(Some(&approved), &read, &took);
    b("listen");
    dialled.write_all(b"GET").unwrap();
    let taken = b("take");
    let _ = (a_runs.kill(), b_runs.kill(), a_runs.wait(), b_runs.wait());
    assert_eq!(took, Took::Waiting);
    assert_eq!(read.sockets.len(), 2, "A listening, B bound: {read:?}");
    assert_eq!(checked, Err(binding::Held::Another));
    assert_eq!(
        taken, "took GET",
        "B, listening again, took the door's bytes"
    );
}

/// **A computer keeps its programs in cgroup v2 when `/proc/self/cgroup`
/// has a `0::` line, and its kernel lists bound sockets from 6.8**
/// (SHARE_BINDING_REVIEWS §6 #5, #11b): v2 alone and v1 beside v2 do, v1
/// alone doesn't; 6.8 and later list them, 6.7 and what can't be read
/// don't. Planted, each watched failing: any line taken for v2 (v1 alone
/// judged by its root cgroup); 6.7 taken for one that lists them.
#[test]
fn a_computer_s_cgroups_and_kernel_are_read_from_their_files() {
    assert!(super::cgroups_v2(
        "0::/user.slice/user-1000.slice/session-2.scope\n"
    ));
    assert!(super::cgroups_v2(
        "12:pids:/user.slice\n1:name=systemd:/user.slice\n0::/user.slice\n"
    ));
    assert!(!super::cgroups_v2(
        "12:pids:/user.slice\n1:name=systemd:/user.slice\n"
    ));
    assert!(!super::cgroups_v2(""));
    assert!(super::lists_bound("7.3.3-200.fc43.x86_64\n"));
    assert!(super::lists_bound("6.8.0-31-generic\n"));
    assert!(!super::lists_bound("6.7.12-200.fc39.x86_64\n"));
    assert!(!super::lists_bound("5.14.0-427.el9.x86_64\n"));
    assert!(!super::lists_bound(""));
}

/// **On Linux another account's listener is approved by its cookie** (§9,
/// A3): systemd-resolved's on 5355, where it listens: read with its uid,
/// shown as its account, approved by cookie and cgroup, and a connection
/// to it carried; approved with another cookie, held. Planted, watched
/// failing on the Fedora box: the cookie compare skipped in
/// `binding::check` (the other cookie carried).
#[cfg(target_os = "linux")]
#[test]
#[ignore = "reads systemd-resolved's port, where one listens: run on Linux by hand"]
fn another_account_s_listener_is_approved_by_its_cookie() {
    let read = answering(Raw::Tcp, 5355, None).expect("5355 reads");
    assert!(
        !read.read.sockets.is_empty(),
        "systemd-resolved listens on 5355 here"
    );
    assert!(
        format!("{:?}", read.shown).contains("Account"),
        "{:?}",
        read.shown
    );
    let mut approved = binding::approve(&read.read, None).unwrap();
    let dialled = std::net::TcpStream::connect(("127.0.0.1", 5355)).unwrap();
    let conn = Conn {
        local: dialled.local_addr().unwrap(),
        peer: dialled.peer_addr().unwrap(),
    };
    let (now, took) = super::port(Raw::Tcp, 5355, Some(conn)).unwrap();
    assert_eq!(binding::check(Some(&approved), &now, &took), Ok(()));
    approved.sockets = approved.sockets.iter().map(|c| c + 1).collect();
    assert_eq!(
        binding::check(Some(&approved), &now, &took),
        Err(binding::Held::Another)
    );
}

/// **What the checks of a datagram cost** (SHARE_BINDING §5.2, §10): a UDP
/// socket alone on a port, read and checked twice per datagram, as the door
/// does before and after each, at 60 and 200 datagrams a second for five
/// seconds each: the time each read takes, and the share of a processor
/// they use. Printed, not asserted: run with `--nocapture`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
#[ignore = "measures for seconds: run by hand with --nocapture"]
#[expect(clippy::disallowed_methods, reason = "a measurement's own clock")]
fn what_the_checks_of_a_datagram_cost() {
    let service = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = service.local_addr().unwrap().port();
    let approved = binding::approve(&super::port(Raw::Udp, port, None).unwrap().0, None).unwrap();
    for rate in [60u32, 200] {
        let (mut took, began) = (Vec::new(), std::time::Instant::now());
        let every = std::time::Duration::from_secs(1) / rate;
        while began.elapsed() < std::time::Duration::from_secs(5) {
            let at = std::time::Instant::now();
            for _ in 0..2 {
                let (read, by) = super::port(Raw::Udp, port, None).unwrap();
                assert_eq!(binding::check(Some(&approved), &read, &by), Ok(()));
            }
            took.push(at.elapsed());
            std::thread::sleep(every.saturating_sub(at.elapsed()));
        }
        took.sort();
        let (n, spent) = (took.len(), took.iter().sum::<std::time::Duration>());
        let (median, p99) = (took[n / 2], took[n * 99 / 100]);
        let busy = spent.as_secs_f64() / began.elapsed().as_secs_f64() * 100.0;
        println!("{rate}/s: {n} datagrams, both checks median {median:?}, p99 {p99:?}, {busy:.1}% of a processor");
    }
}

/// An ntstat message: its context, its type, and what follows its head.
fn stat_message(context: u64, kind: u32, body: &[u8]) -> Vec<u8> {
    let len = u16::try_from(16 + body.len()).unwrap();
    [
        &context.to_ne_bytes()[..],
        &kind.to_ne_bytes(),
        &len.to_ne_bytes(),
        &[0, 0],
        body,
    ]
    .concat()
}

/// A UDP socket's `SRC_DESC` (10003), `srcref` its reference: a
/// descriptor of `len` bytes, its local address `at` (a `sockaddr_in`'s or
/// `sockaddr_in6`'s bytes), the last to use it `pid` with unique id `upid`,
/// of account `uid`.
fn described(srcref: u64, at: &[u8], (pid, uid, upid): (u32, u32, u64), len: usize) -> Vec<u8> {
    let mut d = vec![0u8; len];
    d[..8].copy_from_slice(&upid.to_ne_bytes());
    d[56..56 + at.len()].copy_from_slice(at);
    d[128..132].copy_from_slice(&pid.to_ne_bytes());
    if len >= 272 {
        d[268..272].copy_from_slice(&uid.to_ne_bytes());
    }
    let head = [
        &srcref.to_ne_bytes()[..],
        &[0; 8],
        &4u32.to_ne_bytes(),
        &[0; 4],
    ]
    .concat();
    stat_message(0, 10003, &[head, d].concat())
}

/// A `sockaddr_in` of `ip` and `port`.
fn v4(ip: [u8; 4], port: u16) -> Vec<u8> {
    [&[16, 2][..], &port.to_be_bytes(), &ip, &[0; 8]].concat()
}

/// What `Ask::Stats` reads: the room given, then each record, its length first.
fn stats_answer(room: u32, records: &[Vec<u8>]) -> Vec<u8> {
    let framed = records
        .iter()
        .map(|r| [&u32::try_from(r.len()).unwrap().to_ne_bytes()[..], r].concat());
    [room.to_ne_bytes().to_vec()]
        .into_iter()
        .chain(framed)
        .collect::<Vec<_>>()
        .concat()
}

/// **ntstat's answer is read where it holds it, and nothing else is taken
/// for one** (SHARE_BINDING_UDP §4.6; ntstat.h:518-555, :977-1068): each
/// UDP socket's local address, last process by pid and unique id, and
/// account; and why it can't be told, so the share holds: no end, an error
/// answered, a descriptor of another length, a message past its record, an
/// address of no family, more read than ¾ of the room less 4 KiB, and no
/// canary, svrngs's own unbound socket, found with its pid, uid and unique
/// id. Planted, each watched failing: the canary skipped; the length check
/// skipped; the ¾ bound skipped.
#[test]
fn ntstat_s_answer_is_read_where_it_holds_it() {
    let me = (4242, 502, 77);
    let canary = described(1, &v4([0; 4], 0), me, 280);
    let game = described(2, &v4([0; 4], 34197), (900, 502, 88), 280);
    let six = [
        &[28, 30][..],
        &34197u16.to_be_bytes(),
        &[0; 4],
        &[0; 15],
        &[1],
        &[0; 4],
    ]
    .concat();
    let rogue = described(3, &six, (901, 502, 89), 280);
    let added = stat_message(1, 0, &[]);
    let (ended, room) = (stat_message(2, 0, &[]), 4 << 20);
    let read = |records: &[Vec<u8>]| record::stats(&stats_answer(room, records), me);
    let all = [canary.clone(), game.clone(), rogue].concat();
    let got = read(&[added.clone(), all, ended.clone()]).unwrap();
    assert_eq!(got.len(), 3);
    let game_read = record::Stat {
        srcref: 2,
        local: at("0.0.0.0:34197"),
        pid: 900,
        upid: 88,
        uid: 502,
    };
    assert_eq!(got[1], game_read);
    assert_eq!(got[2].local, at("[::1]:34197"));
    let both = [canary.clone(), game.clone()].concat();
    let none = Err("this Mac's list of UDP sockets didn't end");
    assert_eq!(read(&[added.clone(), both.clone()]), none);
    let refused = stat_message(2, 1, &[0; 8]);
    let said = Err("this Mac refused to list its UDP sockets");
    assert_eq!(read(&[added.clone(), refused]), said);
    let other = Err("this Mac describes a UDP socket in another form");
    let longer = described(2, &v4([0; 4], 34197), (900, 502, 88), 296);
    assert_eq!(
        read(&[[canary.clone(), longer].concat(), ended.clone()]),
        other
    );
    let cut = both[..both.len() - 8].to_vec();
    let past = Err("a message's length runs past its record");
    assert_eq!(read(&[cut, ended.clone()]), past);
    let nowhere = described(2, &[0; 28], (900, 502, 88), 280);
    let unaddressed = Err("this Mac describes a UDP socket's address in another form");
    assert_eq!(
        read(&[[canary.clone(), nowhere].concat(), ended.clone()]),
        unaddressed
    );
    let near = record::stats(&stats_answer(6000, &[both.clone(), ended.clone()]), me);
    assert_eq!(
        near,
        Err("this Mac's list of UDP sockets came near the end of its room")
    );
    let theirs = Err("this Mac's list of UDP sockets doesn't hold svrngs's own");
    assert_eq!(read(&[game.clone(), ended.clone()]), theirs);
    let moved = described(1, &v4([0; 4], 0), (4242, 502, 78), 280);
    assert_eq!(
        read(&[moved, game, ended]),
        theirs,
        "a canary read off by its unique id"
    );
}

/// **A UDP socket is held by the process that last used it while its pid
/// is still that process** (SHARE_BINDING_UDP §4.6 (4)): the unique id of
/// the process at that pid now matched to the socket's; another, or none
/// there, holds nothing, so the share holds. Planted, watched failing: the
/// unique id not compared (a pid used again taken as the socket's user).
#[test]
fn a_udp_socket_is_held_by_its_last_user_while_its_pid_is_that_process() {
    let stat = record::Stat {
        srcref: 2,
        local: at("0.0.0.0:34197"),
        pid: 900,
        upid: 88,
        uid: 502,
    };
    assert_eq!(super::last_user(&stat, Some((1, 55, 88))), Some((900, 55)));
    assert_eq!(
        super::last_user(&stat, Some((1, 55, 89))),
        None,
        "the pid used again"
    );
    assert_eq!(super::last_user(&stat, None), None, "no process there");
}

/// **On a Mac a UDP port is read by ntstat, as it is now, whatever keeps its
/// sockets** (SHARE_BINDING_UDP §4.6): a socket this test binds is read on
/// its port, at its address, held by this process by its pid and start; a
/// program's socket bound beside the approved one, kept only in a message
/// it sent itself and hasn't read, is read too, and holds the share.
#[cfg(target_os = "macos")]
#[test]
fn a_mac_udp_port_is_read_by_ntstat_as_it_is_now() {
    let ours = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = ours.local_addr().unwrap().port();
    let (read, took) = super::port(Raw::Udp, port, None).unwrap();
    assert_eq!(took, Took::Datagram);
    let me = (std::process::id(), start(std::process::id()).unwrap());
    let socket = &read.sockets[..];
    assert_eq!(socket.len(), 1, "{read:?}");
    assert_eq!(
        (socket[0].holders.clone(), socket[0].at),
        ([me].into(), [127, 0, 0, 1].into())
    );
    let game = Serving::new("udp", None);
    let approved = approved(Raw::Udp, &game, Some(game.pid())).unwrap();
    let (alone, took) = super::port(Raw::Udp, game.1, None).unwrap();
    assert_eq!(binding::check(Some(&approved), &alone, &took), Ok(()));
    let kept = Serving::new("inflight", Some(game.1));
    let (now, took) = super::port(Raw::Udp, game.1, None).unwrap();
    assert_eq!(now.sockets.len(), 2, "{now:?}");
    let held = binding::check(Some(&approved), &now, &took);
    assert_eq!(held, Err(binding::Held::Another));
    drop(kept);
}
