use super::*;

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
fn answer(rows: &[(Ends, u32)], raw: Raw, c: Conn, side: Side) -> Owner {
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
    assert_eq!(answer(&rows, Raw::Tcp, dialled, Side::Peer), Owner::Uid(0));
    assert_eq!(
        answer(&rows, Raw::Tcp, dialled, Side::Local),
        Owner::Uid(1000)
    );
    let dual = conn("127.0.0.1:60810", "127.0.0.1:60559");
    let rows = proc_net::rows(TCP6).unwrap();
    assert_eq!(answer(&rows, Raw::Tcp, dual, Side::Peer), Owner::Uid(1000));
    let flow = conn("127.0.0.1:34077", "127.0.0.53:53");
    let rows = proc_net::rows(UDP).unwrap();
    assert_eq!(answer(&rows, Raw::Udp, flow, Side::Peer), Owner::Uid(193));
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
        Owner::Uid(999)
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
    assert_eq!(owner(Raw::Tcp, made, Side::Peer), Owner::Uid(uid));
    assert_eq!(owner(Raw::Tcp, made, Side::Local), Owner::Uid(uid));
    let service = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let flow = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    flow.connect(service.local_addr().unwrap()).unwrap();
    let sent = Conn {
        local: flow.local_addr().unwrap(),
        peer: service.local_addr().unwrap(),
    };
    assert_eq!(owner(Raw::Udp, sent, Side::Peer), Owner::Uid(uid));
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
            owner(Raw::Tcp, queued, Side::Peer),
            Owner::Uid(uid),
            "accepted late"
        );
        let (listener, _kept) = late.join().unwrap().unwrap();
        let never = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let never = Conn {
            local: never.local_addr().unwrap(),
            peer: never.peer_addr().unwrap(),
        };
        assert_eq!(
            owner(Raw::Tcp, never, Side::Peer),
            Owner::NotMine,
            "never accepted"
        );
        let nobody = Conn {
            local: sent.local,
            peer: at("127.0.0.1:1"),
        };
        assert_eq!(owner(Raw::Udp, nobody, Side::Peer), Owner::NotMine);
        // A process of ours with no open files, which libproc answers 0
        // for, as it does a failure: the whole scan still answers.
        let mut bare = std::process::Command::new("/bin/sh")
            .args(["-c", "exec 0<&- 1>&- 2>&- /bin/sleep 5"])
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let scanned = owner(Raw::Udp, nobody, Side::Peer);
        bare.kill().unwrap();
        bare.wait().unwrap();
        assert_eq!(scanned, Owner::NotMine, "a process with no files");
    }
}
