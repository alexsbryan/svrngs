// SPDX-License-Identifier: AGPL-3.0-or-later
//! The WebRTC half's unit checks: what an offer that is not one is told,
//! and the routes' one owner.

use super::*;

/// **An offer that is not an offer is refused**, and said so: the page is
/// told why, never answered with a peer (the spike's "not an offer",
/// native/src/main.rs:197). Planted, watched red: the parse failure
/// answered as a peer (`Ok(..)`), and this test's line was not there.
#[tokio::test]
async fn an_offer_that_is_not_one_is_refused() {
    let (webrtc, _wire) = WebRtc::new(&[1; 32]);
    let said = webrtc.answer(b"not an offer", [2; 32]).unwrap_err();
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
