// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::disallowed_methods, reason = "a test dials what it runs")]
use super::*;

/// **svrngs starts without ::1 only where this computer has no IPv6 at
/// all** (V1.13.3's second practice review, finding 5): the family refused
/// (`EAFNOSUPPORT`), and no program here can listen at ::1 either. Every
/// other refusal of ::1 at svrngs's port stops it starting, ::1 off among
/// them (`EADDRNOTAVAIL`), since another account could take ::1 there once
/// it is on and answer every `<label>.localhost` first. Planted, watched
/// failing: the candidate's rule, any refusal but "in use" carried on
/// alone (`EADDRNOTAVAIL` was let start).
#[test]
fn svrngs_starts_without_ipv6_s_loopback_only_where_there_is_no_ipv6() {
    // errno.h: EAFNOSUPPORT, EADDRNOTAVAIL, EACCES, EPERM, EINVAL.
    let (none, off) = if cfg!(target_os = "macos") {
        (47, 49)
    } else {
        (97, 99)
    };
    assert!(alone(&std::io::Error::from_raw_os_error(none)), "no IPv6");
    for refused in [off, 13, 1, 22] {
        let e = std::io::Error::from_raw_os_error(refused);
        assert!(!alone(&e), "{e} carried on without ::1");
    }
}

/// **Linux holds ::1 at svrngs's port while it is off, so nothing else
/// takes it once on**: in a network namespace whose ::1 is off, svrngs
/// binds 127.0.0.1 and ::1 (`IPV6_FREEBIND`); once ::1 is on, another
/// program's bind there is refused as in use, and a connection to ::1
/// reaches svrngs's listener. By hand, as root of a namespace of its own:
/// `unshare --map-root-user --net sh -c 'ip link set lo up && echo 1 >
/// /proc/sys/net/ipv6/conf/lo/disable_ipv6 && exec <this test binary>
/// --ignored --exact api::tests::linux_holds_ipv6_s_loopback_while_it_is_off'`.
#[cfg(target_os = "linux")]
#[tokio::test]
#[ignore = "by hand on Linux, in a network namespace whose ::1 is off"]
async fn linux_holds_ipv6_s_loopback_while_it_is_off() {
    let off = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap_err();
    assert_eq!(
        off.kind(),
        std::io::ErrorKind::AddrNotAvailable,
        "::1 is off here"
    );
    let bound = bind("127.0.0.1:0".parse().unwrap())
        .await
        .expect("svrngs starts");
    assert_eq!(bound.listeners.len(), 2, "on 127.0.0.1 and ::1");
    let six = SocketAddr::from((Ipv6Addr::LOCALHOST, bound.addr.port()));
    std::fs::write("/proc/sys/net/ipv6/conf/lo/disable_ipv6", "0").unwrap();
    let taken = std::net::TcpListener::bind(six).unwrap_err();
    assert_eq!(
        taken.kind(),
        std::io::ErrorKind::AddrInUse,
        "::1 is svrngs's once on"
    );
    let dialled = tokio::net::TcpStream::connect(six)
        .await
        .expect("::1 answers once on");
    let (_, from) = bound.listeners[1].accept().await.unwrap();
    assert_eq!(from, dialled.local_addr().unwrap());
    eprintln!("RESULT: ::1 held while off, svrngs's once on, at {six}");
}
