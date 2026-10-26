// SPDX-License-Identifier: AGPL-3.0-or-later
//! The UDP door's frame, its reassembly and its flows (give-udp), the time
//! an argument.

#![allow(clippy::disallowed_methods)]

use std::time::{Duration, Instant};

use super::{framed, Flows, Frame, Whole, HEAD, IDLE};

/// What a path carries in one datagram, about QUIC's floor.
const MAX: usize = 1_200;

fn frames(flow: u32, number: u32, packet: &[u8]) -> Vec<Frame> {
    let framed = framed(flow, number, packet, MAX).expect("it fits in 255 datagrams");
    framed
        .into_iter()
        .map(|d| Frame::parse(d).expect("a frame"))
        .collect()
}

#[test]
fn a_packet_is_cut_to_fit_and_made_whole_again() {
    let packet: Vec<u8> = (0..5_000u32).map(|i| (i % 251) as u8).collect();
    let parts = frames(7, 1, &packet);
    assert_eq!(parts.len(), packet.len().div_ceil(MAX - HEAD));
    let (mut whole, now) = (Whole::default(), Instant::now());
    let mut got = None;
    // Unordered, as datagrams arrive.
    for f in parts.into_iter().rev() {
        assert!(got.is_none(), "whole before its last fragment");
        got = whole.take(f, now);
    }
    assert_eq!(got, Some((7, packet)));
    assert_eq!(frames(7, 2, b"").len(), 1, "an empty packet is one frame");
    assert_eq!(Frame::parse(vec![0; HEAD - 1].into()), None, "too short");
}

/// A fragment lost loses its packet, and only it: the next packet of the
/// same flow is whole, and the one missing a fragment is dropped once it
/// has waited.
#[test]
fn a_dropped_fragment_drops_that_packet_alone() {
    let (mut whole, now) = (Whole::default(), Instant::now());
    let first: Vec<u8> = vec![1; 3_000];
    let mut lost = frames(3, 1, &first);
    lost.remove(1);
    for f in lost.clone() {
        assert_eq!(
            whole.take(f, now),
            None,
            "a packet missing a fragment is never passed on"
        );
    }
    let second: Vec<u8> = vec![2; 3_000];
    let got = frames(3, 2, &second)
        .into_iter()
        .find_map(|f| whole.take(f, now));
    assert_eq!(got, Some((3, second)), "the next packet is whole");
    whole.sweep(now + Duration::from_secs(3));
    let late = lost.into_iter().last().map(|f| whole.take(f, now));
    assert_eq!(
        late,
        Some(None),
        "the dropped packet's fragments wait no more"
    );
}

/// A flow with no packet either way for [`IDLE`] closes; one that carried
/// a packet since stays.
#[test]
fn a_flow_idle_past_its_expiry_closes() {
    let (mut flows, t0) = (Flows::default(), Instant::now());
    flows.open(1, "a", t0);
    flows.open(2, "b", t0);
    flows.touch(2, t0 + IDLE / 2);
    assert!(flows.sweep(t0 + IDLE - Duration::from_secs(1)).is_empty());
    assert_eq!(flows.sweep(t0 + IDLE), vec!["a"]);
    assert_eq!((flows.get(1), flows.get(2)), (None, Some(&"b")));
    assert_eq!(flows.sweep(t0 + IDLE + IDLE / 2), vec!["b"]);
}
