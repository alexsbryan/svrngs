// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::disallowed_methods)]
//! The WebRTC half's unit checks: what an offer that is not one is told,
//! the routes' one owner, and direct connections — the decider, the kept
//! setting, the cap, and the peer's socket closing on a network change.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::network::{HardwareAddress, Here, Network, Unread};

use super::direct::{binds_at, kept, setting, turned};
use super::*;

/// A network for the tests: its router's hardware address.
fn network(text: &str) -> Network {
    Network(HardwareAddress::parse(text).unwrap())
}

/// A setting file with `enabled` kept, as the home page's route and each
/// offer read one.
fn file(enabled: Vec<Network>) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(direct::DIRECT_FILE);
    std::fs::write(&file, serde_json::to_vec(&enabled).unwrap()).unwrap();
    (dir, file)
}

/// An offer this computer can accept: one channel, a host candidate on
/// loopback (the test's own).
fn an_offer() -> Vec<u8> {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut rtc = RtcConfig::new().build(Instant::now());
    rtc.add_local_candidate(Candidate::host(socket.local_addr().unwrap(), "udp").unwrap());
    let mut change = rtc.sdp_api();
    change.add_channel("a test".into());
    let (offer, _pending) = change.apply().expect("an offer");
    serde_json::to_vec(&offer).unwrap()
}

/// **An offer that is not an offer is refused**, and said so: the page is
/// told why, never answered with a peer (the spike's "not an offer",
/// native/src/main.rs:197). Planted, watched red: the parse failure
/// answered as a peer (`Ok(..)`), and this test's line was not there.
#[tokio::test]
async fn an_offer_that_is_not_one_is_refused() {
    let (_dir, file) = file(vec![]);
    let (webrtc, _wire) = WebRtc::new(&[1; 32], file);
    let here = Here {
        network: Err(Unread::NoRouter),
        address: None,
        tunnel: None,
    };
    let said = webrtc.answer(b"not an offer", [2; 32], &here).unwrap_err();
    assert!(said.contains("not an offer"), "{said}");
}

/// **The routes' one owner**: iroh's datagrams for a key reach its open
/// channel; one after the peer ended, or for a key with none, is dropped
/// (RUST_ARCH 3). Planted, watched red: `Route::Ended` leaving the entry
/// in place, and the datagram after the end arrived.
#[tokio::test(flavor = "multi_thread")]
async fn a_datagram_for_a_key_with_no_channel_is_dropped() {
    let (routes, told) = unbounded_channel();
    tokio::spawn(routed(told));
    let (out, got) = std::sync::mpsc::channel();
    routes.send(Route::Opened([3; 32], out)).unwrap();
    routes.send(Route::Send([3; 32], b"one".to_vec())).unwrap();
    routes
        .send(Route::Send([4; 32], b"nowhere".to_vec()))
        .unwrap();
    assert_eq!(
        got.recv_timeout(Duration::from_secs(1)).unwrap(),
        b"one".to_vec()
    );
    routes.send(Route::Ended([3; 32])).unwrap();
    routes.send(Route::Send([3; 32], b"two".to_vec())).unwrap();
    assert!(
        got.recv_timeout(Duration::from_millis(100)).is_err(),
        "a datagram arrived after the channel ended"
    );
}

/// **An offer is answered only on a network the lender enabled**, never on
/// one that hands out the same private address: the decider reads the
/// network's own name, its router's hardware address, not the address
/// (`binds_at`). Planted, watched red: the decider matching the address
/// (the enabled kept as addresses), and the café's offer was answered.
#[test]
fn an_offer_is_answered_only_on_a_network_the_lender_enabled() {
    let (home, cafe) = (network("e4:f4:c6:0a:5e:a1"), network("02:00:5e:00:53:01"));
    let address = Ipv4Addr::new(192, 168, 1, 42);
    let at = |network| Here {
        network,
        address: Some(address),
        tunnel: None,
    };
    let (_dir, file) = file(vec![home]);
    assert_eq!(
        binds_at(&file, &at(Ok(home))),
        Ok((home, address)),
        "an enabled network answers at the address picked there"
    );
    let refused = binds_at(&file, &at(Ok(cafe))).unwrap_err();
    assert!(refused.contains("not on for this WiFi"), "{refused}");
}

/// **An offer is refused while the network is unread**, and while the
/// setting cannot be read: nothing is bound on a guess. Planted, watched
/// red: an unread network answered (`Err(_) => Ok(..)`), and the offer
/// was answered.
#[test]
fn an_offer_is_refused_while_the_network_is_unread() {
    let home = network("e4:f4:c6:0a:5e:a1");
    let (_dir, file) = file(vec![home]);
    let unread = Here {
        network: Err(Unread::NoRouter),
        address: Some(Ipv4Addr::new(192, 168, 1, 42)),
        tunnel: None,
    };
    let refused = binds_at(&file, &unread).unwrap_err();
    assert!(refused.contains("cannot tell which WiFi"), "{refused}");
    let here = Here {
        network: Ok(home),
        address: unread.address,
        tunnel: None,
    };
    std::fs::write(&file, b"not json").unwrap();
    let refused = binds_at(&file, &here).unwrap_err();
    assert!(
        refused.contains("cannot read the direct connections"),
        "{refused}"
    );
}

/// **The setting is kept whole and read back**, and one that cannot be
/// read is said, never read as none (`kept`, `turned`, `setting`). Planted,
/// watched red: `turned` keeping nothing (`Ok(vec![])`), and the read-back
/// said off.
#[test]
fn the_setting_is_kept_whole_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(direct::DIRECT_FILE);
    let (home, cafe) = (network("e4:f4:c6:0a:5e:a1"), network("02:00:5e:00:53:01"));
    let at = |network| Here {
        network: Ok(network),
        address: Some(Ipv4Addr::new(192, 168, 1, 42)),
        tunnel: None,
    };
    assert_eq!(kept(&file), Ok(vec![]));
    assert_eq!(turned(&file, home, true), vec![home]);
    assert_eq!(turned(&file, cafe, true), vec![home, cafe]);
    assert_eq!(kept(&file), Ok(vec![home, cafe]));
    assert!(setting(&file, &at(home)).on);
    assert_eq!(turned(&file, home, false), vec![cafe]);
    assert!(!setting(&file, &at(home)).on);
    std::fs::write(&file, b"not json").unwrap();
    assert!(kept(&file).is_err());
    assert!(!setting(&file, &at(home)).on);
}

/// **Attempts past the cap are refused** (the operator's UDP decision:
/// "cap outstanding attempts"): four peers being set up fill it, and the
/// fifth offer waits for the public way. Planted, watched red: no cap,
/// and the fifth was answered.
#[tokio::test(flavor = "multi_thread")]
async fn attempts_past_the_cap_are_refused() {
    let home = network("e4:f4:c6:0a:5e:a1");
    let (_dir, file) = file(vec![home]);
    let (webrtc, _wire) = WebRtc::new(&[1; 32], file);
    let here = Here {
        network: Ok(home),
        address: Some(Ipv4Addr::LOCALHOST),
        tunnel: None,
    };
    let offer = an_offer();
    for _ in 0..MAX_ATTEMPTS {
        webrtc.answer(&offer, [7; 32], &here).expect("answered");
    }
    let refused = webrtc.answer(&offer, [8; 32], &here).unwrap_err();
    assert!(refused.contains("already being set up"), "{refused}");
}

/// **The peer's socket closes when this computer is no longer read on the
/// network it was answered on** (the operator's UDP decision): the read
/// flips to another network and the peer's loop ends, closing its socket.
/// Planted, watched red: no watch, and the loop never ended.
#[test]
fn the_peer_s_socket_closes_when_the_network_changes() {
    let (home, cafe) = (network("e4:f4:c6:0a:5e:a1"), network("02:00:5e:00:53:01"));
    let moved = Arc::new(AtomicBool::new(false));
    let told = moved.clone();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let rtc = RtcConfig::new().build(Instant::now());
    let (arrived, _arrivals) = unbounded_channel();
    let (_out, outgoing) = std::sync::mpsc::channel();
    let ended = std::thread::spawn(move || {
        run(
            rtc,
            socket,
            [1; 32],
            arrived,
            outgoing,
            home,
            || {
                Ok(if told.load(Ordering::Relaxed) {
                    cafe
                } else {
                    home
                })
            },
            Duration::from_millis(20),
        )
    });
    std::thread::sleep(Duration::from_millis(100));
    moved.store(true, Ordering::Relaxed);
    assert_eq!(ended.join().unwrap().unwrap(), "network changed");
}
