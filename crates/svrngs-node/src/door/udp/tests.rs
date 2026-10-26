// SPDX-License-Identifier: AGPL-3.0-or-later
//! The UDP door's frame, its reassembly and its flows (give-udp), the time
//! an argument.

#![allow(clippy::disallowed_methods)]

use std::time::{Duration, Instant};

use axum::body::Bytes;

use super::{framed, past, take, Frame, Whole, HEAD, IDLE, WHOLE_WITHIN};

/// What a path carries in one datagram, about QUIC's floor.
const MAX: usize = 1_200;

fn frames(flow: u32, number: u32, packet: &[u8]) -> Vec<Bytes> {
    let framed = framed(flow, number, packet, MAX).expect("it fits in 255 datagrams");
    assert!(
        framed.iter().all(|d| Frame::parse(d.clone()).is_some()),
        "each a frame"
    );
    framed
}

#[test]
fn a_packet_is_cut_to_fit_and_made_whole_again() {
    let packet: Vec<u8> = (0..5_000u32).map(|i| (i % 251) as u8).collect();
    let parts = frames(7, 1, &packet);
    assert_eq!(parts.len(), packet.len().div_ceil(MAX - HEAD));
    let (mut whole, now) = (Whole::new(), Instant::now());
    let mut got = None;
    // Unordered, as datagrams arrive.
    for f in parts.into_iter().rev() {
        assert!(got.is_none(), "whole before its last fragment");
        got = take(&mut whole, f, now);
    }
    assert_eq!(got, Some((7, packet)));
    assert_eq!(frames(7, 2, b"").len(), 1, "an empty packet is one frame");
    assert_eq!(Frame::parse(vec![0; HEAD - 1].into()), None, "too short");
}

/// A fragment lost loses its packet, and only it: the next packet of the
/// same flow is whole, and the one missing a fragment is dropped once it
/// has waited [`WHOLE_WITHIN`].
#[test]
fn a_dropped_fragment_drops_that_packet_alone() {
    let (mut whole, now) = (Whole::new(), Instant::now());
    let mut lost = frames(3, 1, &[1; 3_000]);
    let late = lost.remove(1);
    for f in lost {
        let got = take(&mut whole, f, now);
        assert_eq!(got, None, "a packet missing a fragment is never passed on");
    }
    let second = frames(3, 2, &[2; 3_000]);
    let got = second.into_iter().find_map(|f| take(&mut whole, f, now));
    assert_eq!(got, Some((3, vec![2; 3_000])), "the next packet is whole");
    let after = now + WHOLE_WITHIN;
    assert_eq!(
        take(&mut whole, late, after),
        None,
        "the dropped packet waits no more"
    );
    assert_eq!(
        whole.len(),
        1,
        "only its late fragment waits, as a packet of its own"
    );
}

/// A flow with no packet either way for [`IDLE`] closes; one that carried
/// a packet since stays.
#[test]
fn a_flow_idle_past_its_expiry_closes() {
    let t0 = Instant::now();
    assert!(!past(t0, IDLE, t0 + IDLE - Duration::from_secs(1)));
    assert!(past(t0, IDLE, t0 + IDLE));
    assert!(
        !past(t0 + IDLE / 2, IDLE, t0 + IDLE),
        "a packet since keeps it open"
    );
}
