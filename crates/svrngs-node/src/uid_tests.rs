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
/// Planted, each watched failing: the two ends swapped in `answer` (the far
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

/// The table `net.inet.tcp.pcblist_n` answered on a computer where
/// uid 502 dialled root's 127.0.0.1:49153 from 127.0.0.1:58408 (2026-11-18,
/// `python3` with ctypes): its header, the two sockets' records, each
/// socket's six, and its trailer, kernel pointers zeroed.
fn captured() -> Vec<u8> {
    hex::decode(include_str!("fixtures/pcblist_n.tcp.hex").trim()).unwrap()
}

/// **A captured `pcblist_n` answers whose each end is** (LEND_OWNER §5, 2):
/// the accepted socket root's, the dialler's 502's. Planted: `so_uid` read
/// at 68, `so_last_pid` (1077, root's listener's pid, not 0).
#[test]
fn a_captured_pcblist_answers_whose_each_end_is() {
    let rows = pcblist::rows(&captured()).unwrap();
    let dialled = conn("127.0.0.1:58408", "127.0.0.1:49153");
    assert_eq!(answer(&rows, Raw::Tcp, dialled, Side::Peer), Owner::Uid(0));
    assert_eq!(
        answer(&rows, Raw::Tcp, dialled, Side::Local),
        Owner::Uid(502)
    );
}

/// **A `pcblist_n` laid out otherwise is refused, never read**: a socket's
/// record of a length this reader wasn't built for; a table cut off inside
/// a record; a hand-made one whose addresses' record is too short for them;
/// and a socket's record with no addresses before it. Planted: the length
/// unchecked (the longer record read as before, an answer).
#[test]
fn a_pcblist_laid_out_otherwise_is_refused() {
    let table = captured();
    let mut longer = table.clone();
    // The first socket's `xsocket_n`, after the header (24) and its
    // `xinpcb_n` (104): its length, 104, said 112.
    assert_eq!(pcblist::word(&longer, 128), Some(104));
    longer[128..132].copy_from_slice(&112u32.to_ne_bytes());
    assert!(pcblist::rows(&longer).is_err());
    assert!(pcblist::rows(&table[..400]).is_err());
    let mut made = 24u32.to_ne_bytes().to_vec();
    made.extend([0; 20]);
    made.extend(
        40u32
            .to_ne_bytes()
            .iter()
            .chain(&pcblist::XSO_INPCB.to_ne_bytes()),
    );
    made.extend([0; 32]);
    made.extend(24u32.to_ne_bytes().iter().chain(&[0; 20]));
    assert!(pcblist::rows(&made).is_err());
    let mut socket_first = table[..24].to_vec();
    socket_first.extend(&table[128..232]);
    socket_first.extend(&table[table.len() - 24..]);
    assert!(pcblist::rows(&socket_first).is_err());
}

/// **This computer's own table answers for a connection and a flow made
/// here**: a listener and the socket that dialled it, each end ours, the
/// uid that owns the folder this test made; a UDP socket and one connected
/// to it, ours. Read from `/proc/net` or `pcblist_n` as this computer has
/// it, so it passes on both only with both readers right.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn this_computer_s_own_table_answers_for_a_connection_made_here() {
    use std::os::unix::fs::MetadataExt as _;
    let dir = tempfile::tempdir().unwrap();
    let ours = Owner::Uid(std::fs::metadata(dir.path()).unwrap().uid());
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let dialled = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let _accepted = listener.accept().unwrap();
    let made = Conn {
        local: dialled.local_addr().unwrap(),
        peer: dialled.peer_addr().unwrap(),
    };
    assert_eq!(owner(Raw::Tcp, made, Side::Peer), ours);
    assert_eq!(owner(Raw::Tcp, made, Side::Local), ours);
    let service = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let flow = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    flow.connect(service.local_addr().unwrap()).unwrap();
    let made = Conn {
        local: flow.local_addr().unwrap(),
        peer: service.local_addr().unwrap(),
    };
    assert_eq!(owner(Raw::Udp, made, Side::Peer), ours);
}
