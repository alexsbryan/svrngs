use super::*;

/// The command `ran` makes, as [`held`] reads it.
fn command(ran: impl IntoIterator<Item = Vec<Vec<u8>>>) -> Command {
    held(Sandbox::No, ran, Vec::new()).command
}

fn at(addr: &str) -> SocketAddr {
    addr.parse().unwrap()
}

fn conn(local: &str, peer: &str) -> Conn {
    Conn {
        local: at(local),
        peer: at(peer),
    }
}

/// The one uid among `rows` holding the socket at `side` of `c`, as the
/// Linux reader answers it.
fn answer(rows: &[proc_net::Row], raw: Raw, c: Conn, side: Side) -> Owner {
    let (at, to) = ends(c, side);
    one(rows.iter().filter(|r| holds(raw, r.0, at, to)).map(|r| r.1))
}

/// `/proc/net/tcp` on a computer where uid 1000 dialled root's
/// 127.0.0.1:631 from 127.0.0.1:53696, its rows for that connection as read
/// (`python3`, 2026-11-18): the accepted socket's row is root's, the
/// dialler's 1000's.
const TCP: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
  30: 0100007F:0277 0100007F:D1C0 01 00000000:00000000 00:00000000 00000000     0        0 2556494 1 00000000ff346488 20 0 0 10 -1
  32: 0100007F:D1C0 0100007F:0277 01 00000000:00000000 00:00000000 00000000  1000        0 2550773 2 00000000cbbd1318 20 0 0 10 -1
";

/// The same computer: a listener of uid 1000's on every IPv6 address and IPv4's
/// too, dialled on 127.0.0.1:60559 from 127.0.0.1:60810. The socket it
/// accepted is listed in `tcp6`, its addresses IPv4 written as IPv6.
const TCP6: &str = "  sl  local_address                         remote_address                        st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   5: 0000000000000000FFFF00000100007F:EC8F 0000000000000000FFFF00000100007F:ED8A 01 00000000:00000000 00:00000000 00000000  1000        0 2550777 1 000000009cb9fc24 20 0 0 10 -1
";

/// The same computer's `/proc/net/udp`: uid 193's sockets on
/// 127.0.0.53:53 and 127.0.0.54:53, and uid 1000's connected to the first
/// from 127.0.0.1:34077.
const UDP: &str = "   sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode ref pointer drops
21944: 3600007F:0035 00000000:0000 07 00000000:00000000 00:00000000 00000000   193        0 10158 2 00000000b1eafe38 0
21944: 3500007F:0035 00000000:0000 07 00000000:00000000 00:00000000 00000000   193        0 10156 2 000000007a36108f 0
55968: 0100007F:851D 3500007F:0035 01 00000000:00000000 00:00000000 00000000  1000        0 2550778 2 00000000400f7600 0
";

/// **The uid column answers whose each end of a connection is**
/// (LEND_OWNER §5, 1): asked of the dialler's connection to root's 631, the
/// far end is root's and its own end 1000's; the accepted socket of a
/// dual-stack listener, listed in `tcp6` as an IPv4 address written as IPv6,
/// is found; a datagram to 127.0.0.53:53 reaches resolved's socket alone.
/// Planted, each watched failing: the two ends swapped in `ends` (the far
/// end read 1000); the IPv6 form of an IPv4 address compared as written
/// (the `tcp6` row missed, could not judge).
#[test]
fn the_uid_column_answers_whose_each_end_of_a_connection_is() {
    let dialled = conn("127.0.0.1:53696", "127.0.0.1:631");
    let rows = proc_net::rows(TCP).unwrap();
    assert_eq!(
        answer(&rows, Raw::Tcp, dialled, Side::Peer),
        Owner::Uid(0, None)
    );
    assert_eq!(
        answer(&rows, Raw::Tcp, dialled, Side::Local),
        Owner::Uid(1000, None)
    );
    let dual = conn("127.0.0.1:60810", "127.0.0.1:60559");
    let rows = proc_net::rows(TCP6).unwrap();
    assert_eq!(
        answer(&rows, Raw::Tcp, dual, Side::Peer),
        Owner::Uid(1000, None)
    );
    let flow = conn("127.0.0.1:34077", "127.0.0.53:53");
    let rows = proc_net::rows(UDP).unwrap();
    assert_eq!(
        answer(&rows, Raw::Udp, flow, Side::Peer),
        Owner::Uid(193, None)
    );
}

/// **What the uid column can't settle is never an answer**: a socket in
/// TIME_WAIT, which lists uid 0 whoever held it, is passed over beside the
/// connection that took its four numbers; a port nobody holds, and one two
/// accounts hold, could not be judged; a line that is not a row refuses
/// the table. Planted: TIME_WAIT kept (two accounts, could not judge).
#[test]
fn what_the_uid_column_can_t_settle_is_never_an_answer() {
    let waiting = TCP.replace(
        "  30: 0100007F:0277 0100007F:D1C0 01",
        "  29: 0100007F:0277 0100007F:D1C0 06 00000000:00000000 00:00000000 00000000     0        0 0 1 0\n  30: 0100007F:0277 0100007F:D1C0 01",
    );
    let rows = proc_net::rows(&waiting.replace(
        "00000000     0        0 2556494",
        "00000000   999        0 2556494",
    ))
    .unwrap();
    let dialled = conn("127.0.0.1:53696", "127.0.0.1:631");
    assert_eq!(
        answer(&rows, Raw::Tcp, dialled, Side::Peer),
        Owner::Uid(999, None)
    );
    let nobody = conn("127.0.0.1:53696", "127.0.0.1:632");
    assert!(matches!(
        answer(&rows, Raw::Tcp, nobody, Side::Peer),
        Owner::Unjudged(_)
    ));
    let shared = UDP
        .replace("3600007F:0035", "00000000:0035")
        .replace("   193        0 10158", "  1001        0 10158");
    let flow = conn("127.0.0.1:34077", "127.0.0.53:53");
    let rows = proc_net::rows(&shared).unwrap();
    assert_eq!(
        answer(&rows, Raw::Udp, flow, Side::Peer),
        Owner::Unjudged("sockets of more than one account at that end")
    );
    assert!(proc_net::rows(&format!("{TCP}  33: not a row\n")).is_err());
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

/// **libproc's records are read where they hold it** (LEND_OWNER §5, 2): a
/// captured TCP socket's local and foreign address; a UDP socket's, bound
/// and connected to none; a socket of neither, none; the `proc_bsdinfo` a
/// process read of itself, uid 502, and a record of another length, none.
/// Planted, each watched failing: the local and foreign addresses' offsets
/// swapped (the ends read backwards); the uid read at 24, `pbi_gid` (20).
#[test]
fn libproc_s_records_are_read_where_they_hold_it() {
    assert_eq!(
        record("tcp").len(),
        record::SOCKET_LEN,
        "the capture's length"
    );
    let tcp = record::socket(&record("tcp"));
    let dialled = (at("127.0.0.1:61126"), at("127.0.0.1:61125"));
    assert_eq!(tcp, Some((Raw::Tcp, dialled)));
    let udp = record::socket(&record("udp"));
    assert_eq!(
        udp,
        Some((Raw::Udp, (at("127.0.0.1:63655"), at("0.0.0.0:0"))))
    );
    let mut unix = record("tcp");
    unix[256..260].copy_from_slice(&0u32.to_ne_bytes());
    assert_eq!(record::socket(&unix), None);
    let bsd = hex::decode(include_str!("fixtures/proc_bsdinfo.hex").trim()).unwrap();
    assert_eq!(record::uid(&bsd), Some(502));
    assert_eq!(record::uid(&bsd[..128]), None);
}

/// **This computer answers for a connection and a flow made here**: a
/// listener and the socket that dialled it, each end ours, the uid that
/// owns the folder this test made, which [`mine`] says too; a UDP socket and
/// one connected to it, ours. On a Mac also: a connection its listener
/// accepts 30 ms late, in no process's files until then, found within
/// [`ACCEPT_WITHIN`]; one never accepted is not this account's; a port of no
/// socket of ours is not. Read as this computer reads them, so it passes on
/// each only with its reader right. Planted on the Mac, watched failing:
/// one scan, no look again (the late accept not this account's); errno
/// not cleared (a process with no files made the scan unjudged).
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn this_computer_answers_for_a_connection_made_here() {
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
    assert_eq!(
        owner(Raw::Tcp, made, Side::Peer, None),
        Owner::Uid(uid, None)
    );
    assert_eq!(
        owner(Raw::Tcp, made, Side::Local, None),
        Owner::Uid(uid, None)
    );
    let service = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let flow = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    flow.connect(service.local_addr().unwrap()).unwrap();
    let sent = Conn {
        local: flow.local_addr().unwrap(),
        peer: service.local_addr().unwrap(),
    };
    assert_eq!(
        owner(Raw::Udp, sent, Side::Peer, None),
        Owner::Uid(uid, None)
    );
    // This test's own sockets, asked with what is kept: its program, this
    // test, in no sandbox, read once.
    let seen = Seen::default();
    let tcp = owner(Raw::Tcp, made, Side::Peer, Some(&seen));
    let Owner::Uid(read, Some(holder)) = &tcp else {
        panic!("{tcp:?}")
    };
    assert_eq!((*read, &holder.sandbox), (uid, &Sandbox::No));
    assert!(holder.pids.contains(&std::process::id()), "{holder:?}");
    assert_eq!(owner(Raw::Udp, sent, Side::Peer, Some(&seen)), tcp);
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
        assert_eq!(
            owner(Raw::Tcp, queued, Side::Peer, None),
            Owner::Uid(uid, None),
            "accepted late"
        );
        let (listener, _kept) = late.join().unwrap().unwrap();
        let never = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let never = Conn {
            local: never.local_addr().unwrap(),
            peer: never.peer_addr().unwrap(),
        };
        assert_eq!(
            owner(Raw::Tcp, never, Side::Peer, None),
            Owner::NotMine,
            "never accepted"
        );
        let nobody = Conn {
            local: sent.local,
            peer: at("127.0.0.1:1"),
        };
        assert_eq!(owner(Raw::Udp, nobody, Side::Peer, None), Owner::NotMine);
        // A process of ours with no open files, which libproc answers 0
        // for, as it does a failure: the whole scan still answers.
        let mut bare = std::process::Command::new("/bin/sh")
            .args(["-c", "exec 0<&- 1>&- 2>&- /bin/sleep 5"])
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let scanned = owner(Raw::Udp, nobody, Side::Peer, None);
        bare.kill().unwrap();
        bare.wait().unwrap();
        assert_eq!(scanned, Owner::NotMine, "a process with no files");
    }
}

/// **A Flatpak is read by the app its root names** (LEND_SANDBOX): a
/// process's `/proc/<pid>` whose root holds the `.flatpak-info` read on the
/// Fedora box (2026-11-19, sabnzbd) is that app, whichever instance; one
/// whose root holds none is in no sandbox; one whose root is gone can't be
/// judged, never none; one naming no app can't be judged. Planted, watched
/// failing: `instance-id=` read for the app (the second run another app).
#[test]
fn a_flatpak_is_read_by_the_app_its_root_names() {
    let info = |instance: &str| {
        format!("[Application]\nname=org.sabnzbd.sabnzbd\nruntime=runtime/org.freedesktop.Platform/x86_64/26.08\n\n[Instance]\ninstance-id={instance}\napp-path=/home/a/.local/share/flatpak/app/org.sabnzbd.sabnzbd/x86_64/stable/c241/files\n")
    };
    let proc = tempfile::tempdir().unwrap();
    std::fs::create_dir(proc.path().join("root")).unwrap();
    let sabnzbd = Ok(Sandbox::App("org.sabnzbd.sabnzbd".into()));
    for instance in ["1244589967", "839627432"] {
        std::fs::write(proc.path().join("root/.flatpak-info"), info(instance)).unwrap();
        assert_eq!(proc_net::flatpak(proc.path()), sabnzbd, "run {instance}");
    }
    std::fs::write(proc.path().join("root/.flatpak-info"), "[Application]\n").unwrap();
    assert!(proc_net::flatpak(proc.path()).is_err(), "no app named");
    std::fs::remove_file(proc.path().join("root/.flatpak-info")).unwrap();
    assert_eq!(proc_net::flatpak(proc.path()), Ok(Sandbox::No));
    std::fs::remove_dir(proc.path().join("root")).unwrap();
    assert!(proc_net::flatpak(proc.path()).is_err(), "a process gone");
}

/// **The sockets a connection reaches are told apart by inode, and agree
/// or say nothing**: each `/proc/net` row keeps its inode; the holders of
/// the sockets there, one program, are it; two sandboxes or programs, or
/// none read, can't be judged. Planted, watched failing: the inode read
/// from the ninth field (the timeout, 0); `alike` taking the first said.
#[test]
fn the_sockets_a_connection_reaches_agree_or_say_nothing() {
    use crate::node::holder;
    let rows = proc_net::rows(TCP).unwrap();
    assert_eq!((rows[0].2, rows[1].2), (2556494, 2550773));
    let said = |all: Vec<Read>| proc_net::alike(all.into_iter(), "none");
    let app = || holder(Sandbox::App("org.sabnzbd.sabnzbd".into()), "sabnzbd");
    let none = |name| holder(Sandbox::No, name);
    assert_eq!(said(vec![Ok(app()), Ok(app())]), Ok(app()));
    assert_eq!(said(vec![Ok(none("serve"))]), Ok(none("serve")));
    assert!(
        said(vec![Ok(none("sabnzbd")), Ok(app())]).is_err(),
        "two sandboxes"
    );
    assert!(
        said(vec![Ok(none("serve")), Ok(none("other"))]).is_err(),
        "two programs"
    );
    assert_eq!(said(vec![]), Err("none"));
    assert!(said(vec![Ok(app()), Err("unread")]).is_err());
}

/// **A sandbox is read once for what a taker must make anew** (LEND_SANDBOX):
/// asked again at the same key, `Seen` answers what it kept without
/// reading; at another key it reads; a read that failed is not kept; full,
/// it empties. And a Mac's key: the `proc_bsdinfo` capture's start, in
/// 2026, none for a record of another length. Planted, watched failing:
/// nothing kept (read twice); never emptied (4097 kept); the start read at
/// 112 (not in 2026).
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn a_sandbox_is_read_once_for_what_a_taker_must_make_anew() {
    let seen = Seen::default();
    let reads = std::cell::Cell::new(0);
    let read = |holder: Read| {
        reads.set(reads.get() + 1);
        holder
    };
    let no = || crate::node::holder(Sandbox::No, "serve");
    assert_eq!(seen.get_or((7, 0), || read(Ok(no()))), Ok(no()));
    assert_eq!(seen.get_or((7, 0), || read(Err("again"))), Ok(no()));
    assert_eq!(reads.get(), 1, "kept, not read again");
    assert!(seen.get_or((8, 0), || read(Err("unread"))).is_err());
    assert_eq!(seen.get_or((8, 0), || read(Ok(no()))), Ok(no()));
    assert_eq!(reads.get(), 3, "a failed read not kept");
    for pid in 0..KEPT as u64 + 1 {
        let _ = seen.get_or((pid, 1), || Ok(no()));
    }
    assert!(seen.0.lock().unwrap().len() <= KEPT, "emptied when full");
    let bsd = hex::decode(include_str!("fixtures/proc_bsdinfo.hex").trim()).unwrap();
    let year = 1_767_225_600..1_798_761_600;
    let start = record::start(&bsd).map(|us| us / 1_000_000);
    assert!(start.is_some_and(|s| year.contains(&s)), "{start:?}");
    assert_eq!(record::start(&bsd[..128]), None);
}

/// **A program is its processes' commands, whatever their titles**
/// (LEND_PROGRAM §2, §6): a process's command is its executable's path and
/// its arguments past the first, a title's NUL padding passed over, so an
/// nginx master and worker, their titles in their first argument, are one
/// program, and two holders of one command count once; a holder running
/// another command is another program; an executable an update replaced
/// (` (deleted)`) is read at its path; a Mac's `KERN_PROCARGS2` answer is
/// read past its path's padding, up to its count, its environment never.
/// Planted, each watched failing: ` (deleted)` not stripped (two programs);
/// the count not kept to (the environment read as arguments).
#[test]
fn a_program_is_its_processes_commands_whatever_their_titles() {
    let proc = |exe: &str, cmdline: &[u8]| {
        let proc = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(exe, proc.path().join("exe")).unwrap();
        std::fs::write(proc.path().join("cmdline"), cmdline).unwrap();
        proc_net::ran(proc.path()).unwrap()
    };
    let nginx = "/usr/bin/nginx";
    let master = proc(nginx, b"nginx: master process /usr/bin/nginx\0\0\0\0\0\0");
    let worker = proc(
        nginx,
        b"nginx: worker process\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    );
    assert_eq!(
        (&master, &worker),
        (&vec![nginx.as_bytes().to_vec()], &master)
    );
    assert_eq!(command([master.clone(), worker]), command([master.clone()]));
    let served = b"python3\0-m\0http.server\08000\0";
    let python = proc("/usr/bin/python3.13", served);
    assert_eq!(
        python,
        proc("/usr/bin/python3.13 (deleted)", served),
        "updated"
    );
    let both = command([python.clone(), master]);
    assert_ne!(both, command([python.clone()]), "every holder counted");
    assert_eq!(both.name, "nginx, python3.13");
    let other = proc("/usr/bin/python3.13", b"python3\0-m\0http.server\08001\0");
    assert_ne!(command([other]).digest, command([python]).digest);
    let mut procargs = 3u32.to_ne_bytes().to_vec();
    procargs.extend(b"/usr/bin/python3\0\0\0\0python3\0-m\0http.server\0PATH=/bin\0\0");
    let args = |a: &[&str]| a.iter().map(|a| a.as_bytes().to_vec()).collect::<Vec<_>>();
    assert_eq!(record::args(&procargs), Some(args(&["-m", "http.server"])));
    assert_eq!(record::args(&procargs[..2]), None);
}

/// A listener on 127.0.0.1 held by this Python and, as its first argument
/// says, by a child once it runs: `two` the same command under another
/// title, `sleep` the `sleep` program. It prints its port and its child's
/// pid, 0 for none, and keeps each connection it accepts.
#[cfg(any(target_os = "linux", target_os = "macos"))]
const SERVE: &str = r#"
import os, socket, sys, time
if os.environ.pop("HELD", None):
    time.sleep(60)
s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", 0))
s.listen(8)
s.set_inheritable(True)
child = 0
if sys.argv[1] in ("two", "sleep"):
    ran, done = os.pipe()
    child = os.fork()
    if child == 0:
        os.environ["HELD"] = "1"
        if sys.argv[1] == "two":
            os.execv(sys.executable, ["another-title"] + sys.orig_argv[1:])
        os.execv("/bin/sleep", ["sleep", "60"])
    os.close(done)
    os.read(ran, 1)
print(s.getsockname()[1], child, flush=True)
kept = []
while True:
    kept.append(s.accept())
"#;

/// **A program is read by its command, and a restart keeps it**
/// (LEND_PROGRAM §2, §4, §6), as this computer reads it: a Python server
/// read with what is kept is in no sandbox, its pid among its holders;
/// started again with the same command, the same program; with other
/// arguments, another. A listener a child holds too, under another title:
/// on Linux both its holders, one program; that child is among them, as
/// `among` asks, on a Mac by the listener it holds; this test's process is
/// not. On Linux a child running `sleep` makes another program than the
/// Python alone. Planted on Linux, watched failing: `holders` hashing only
/// the first holder (the `sleep` child not counted).
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn a_program_is_read_by_its_command_and_a_restart_keeps_it() {
    use std::io::BufRead as _;
    use std::os::unix::process::CommandExt as _;
    /// A running [`SERVE`], its port and its child's pid; its group
    /// killed when dropped, its child with it.
    struct Serving(std::process::Child, u16, u32);
    impl Drop for Serving {
        fn drop(&mut self) {
            let group = format!("-{}", self.0.id());
            let _ = std::process::Command::new("kill")
                .args(["-9", "--", &group])
                .status();
            let _ = self.0.wait();
        }
    }
    let serve = |how: &str| {
        let mut python = std::process::Command::new("python3")
            .args(["-c", SERVE, how])
            .stdout(std::process::Stdio::piped())
            .process_group(0)
            .spawn()
            .expect("python3 runs here");
        let mut said = String::new();
        let out = python.stdout.take().unwrap();
        std::io::BufReader::new(out).read_line(&mut said).unwrap();
        let (port, child) = said.trim().split_once(' ').expect("a port and a pid");
        Serving(python, port.parse().unwrap(), child.parse().unwrap())
    };
    let read = |at: &Serving| {
        let dialled = std::net::TcpStream::connect(("127.0.0.1", at.1)).unwrap();
        let made = Conn {
            local: dialled.local_addr().unwrap(),
            peer: dialled.peer_addr().unwrap(),
        };
        match owner(Raw::Tcp, made, Side::Peer, Some(&Seen::default())) {
            Owner::Uid(_, Some(holder)) => holder,
            other => panic!("{other:?}"),
        }
    };
    let one = serve("one");
    let first = read(&one);
    assert_eq!(first.sandbox, Sandbox::No);
    assert!(first.pids.contains(&one.0.id()), "{first:?}");
    drop(one);
    let again = serve("one");
    assert_eq!(read(&again).command, first.command, "started again");
    drop(again);
    let other = serve("other");
    assert_ne!(read(&other).command.digest, first.command.digest);
    drop(other);
    let two = serve("two");
    let titled = read(&two);
    assert!(among(&titled, two.2, Raw::Tcp, two.1), "{titled:?}");
    assert!(!among(&titled, std::process::id(), Raw::Tcp, two.1));
    #[cfg(target_os = "linux")]
    {
        let exe = std::fs::read_link(format!("/proc/{}/exe", two.0.id())).unwrap();
        let ran = ["-c", SERVE, "two"].map(|a| a.as_bytes().to_vec());
        let ran = std::iter::once(exe.into_os_string().into_encoded_bytes()).chain(ran);
        assert_eq!(titled.pids.len(), 2, "{titled:?}");
        assert_eq!(titled.command, command([ran.collect()]), "one program");
        drop(two);
        let slept = serve("sleep");
        let both = read(&slept);
        let python = proc_net::ran(std::path::Path::new(&format!("/proc/{}", slept.0.id())));
        assert_eq!(both.pids.len(), 2, "{both:?}");
        assert_ne!(
            both.command,
            command([python.unwrap()]),
            "the child counted"
        );
        assert!(both.command.name.contains("sleep"), "{both:?}");
    }
}
