// SPDX-License-Identifier: AGPL-3.0-or-later
//! The limit and the share, on values: given who asked when and from
//! where, when one more asks, then what each is told. The time is an
//! argument, so the window is crossed without waiting it out.
#![allow(clippy::disallowed_methods)]

use std::net::IpAddr;
use std::num::{NonZeroU16, NonZeroU32};
use std::time::{Duration, Instant};

use super::*;
use crate::lend::{Kind, Lend, LendName};

const AWAY: Whereabouts = Whereabouts::Away;

/// A lend of 30 Mbit/s declared, to three people at once.
fn shelf(upload: Option<u64>) -> Lend {
    Lend {
        name: LendName::parse("shelf").unwrap(),
        port: NonZeroU16::new(8096).unwrap(),
        kind: Kind::parse("k").unwrap(),
        limit: NonZeroU16::new(3).unwrap(),
        refused: Vec::new(),
        // Read as a declaration's `upload_bps` is.
        upload: upload.map(|b| serde_json::from_value(serde_json::json!(b)).unwrap()),
        reads: Vec::new(),
        sign_in: None,
        asks: None,
    }
}

fn let_in(people: u32, share: u64) -> Verdict {
    Verdict::In {
        people: NonZeroU32::new(people).unwrap(),
        share: Some(share),
    }
}

/// Three people streaming: ada, bo and cy each have a request in flight.
fn three_streaming(lend: &Lend, t0: Instant) -> Counts {
    let mut counts = Counts::default();
    assert_eq!(counts.ask(lend, "ada", AWAY, t0), let_in(1, 30_000_000));
    assert_eq!(counts.ask(lend, "bo", AWAY, t0), let_in(2, 15_000_000));
    assert_eq!(counts.ask(lend, "cy", AWAY, t0), let_in(3, 10_000_000));
    counts
}

/// Three people streaming are each told a third of the declared upload —
/// ada too, whose second request (a seek) is in flight beside her first:
/// four requests, three people. Planted: the share divided by requests
/// (ada told a quarter).
#[test]
fn three_people_streaming_are_each_told_a_third() {
    let (lend, t0) = (shelf(Some(30_000_000)), Instant::now());
    let mut counts = three_streaming(&lend, t0);
    let t1 = t0 + Duration::from_secs(5);
    for who in ["ada", "bo", "cy"] {
        assert_eq!(
            counts.ask(&lend, who, AWAY, t1),
            let_in(3, 10_000_000),
            "{who} is one of three, however many requests they have in flight"
        );
    }
    assert_eq!(counts.people(&lend.name, t1), 3);
}

/// A fourth is refused, with how many are using it: three.
#[test]
fn a_fourth_is_refused_naming_three() {
    let (lend, t0) = (shelf(Some(30_000_000)), Instant::now());
    let mut counts = three_streaming(&lend, t0);
    assert_eq!(
        counts.ask(&lend, "dee", AWAY, t0),
        Verdict::Full { using: 3 }
    );
    assert_eq!(counts.people(&lend.name, t0), 3, "a refusal counts no one");
}

/// One stops, and is counted until the window passes: just inside it the
/// fourth is still refused; at it, the fourth is let in, and each is told
/// a third.
#[test]
fn one_stops_and_after_the_window_the_fourth_is_let_in() {
    let (lend, t0) = (shelf(Some(30_000_000)), Instant::now());
    let mut counts = three_streaming(&lend, t0);
    let stopped = t0 + Duration::from_secs(10);
    counts.end(&lend.name, "cy", stopped);
    let inside = stopped + WINDOW - Duration::from_millis(1);
    assert_eq!(
        counts.ask(&lend, "dee", AWAY, inside),
        Verdict::Full { using: 3 },
        "cy stopped within the window, so is still counted"
    );
    let after = stopped + WINDOW;
    assert_eq!(counts.people(&lend.name, after), 2);
    assert_eq!(counts.ask(&lend, "dee", AWAY, after), let_in(3, 10_000_000));
    for who in ["ada", "bo"] {
        assert_eq!(counts.ask(&lend, who, AWAY, after), let_in(3, 10_000_000));
    }
    assert_eq!(
        counts.ask(&lend, "cy", AWAY, after),
        Verdict::Full { using: 3 },
        "cy, gone past the window, is a newcomer again"
    );
}

/// A viewer whose path is a private, link-local or unique-local address is
/// guessed home: let in past the limit, uncounted, and the three are still
/// told a third. Public, loopback and relayed paths are away. Planted: the
/// local guess skipped (every viewer away).
#[test]
fn a_viewer_on_a_private_address_is_not_counted() {
    let home: [&str; 7] = [
        "10.0.0.2",
        "172.16.5.1",
        "192.168.1.20",
        "169.254.3.3",
        "fd12:3456::1",
        "fe80::1",
        "::ffff:192.168.1.20",
    ];
    for addr in home {
        let ip: IpAddr = addr.parse().unwrap();
        assert_eq!(guess(Some(ip)), Whereabouts::Home, "{addr}");
    }
    for addr in ["8.8.8.8", "172.32.0.1", "127.0.0.1", "::1", "2001:db8::1"] {
        let ip: IpAddr = addr.parse().unwrap();
        assert_eq!(guess(Some(ip)), Whereabouts::Away, "{addr}");
    }
    assert_eq!(guess(None), Whereabouts::Away, "a relayed path is away");

    let (lend, t0) = (shelf(Some(30_000_000)), Instant::now());
    let mut counts = three_streaming(&lend, t0);
    let sam_s_tv = guess(Some("192.168.1.20".parse().unwrap()));
    assert_eq!(counts.ask(&lend, "eve", sam_s_tv, t0), Verdict::Home);
    assert_eq!(counts.people(&lend.name, t0), 3);
    assert_eq!(counts.ask(&lend, "ada", AWAY, t0), let_in(3, 10_000_000));
}

/// A lend declaring no upload lets people in and counts them, and tells
/// none a share: no figure nobody stated.
#[test]
fn no_upload_declared_tells_no_share() {
    let (lend, t0) = (shelf(None), Instant::now());
    let mut counts = Counts::default();
    assert_eq!(
        counts.ask(&lend, "ada", AWAY, t0),
        Verdict::In {
            people: NonZeroU32::new(1).unwrap(),
            share: None
        }
    );
}

/// A request that was never let in ends as no one's: it frees no place.
#[test]
fn an_end_of_no_one_s_request_frees_nothing() {
    let (lend, t0) = (shelf(Some(30_000_000)), Instant::now());
    let mut counts = three_streaming(&lend, t0);
    counts.end(&lend.name, "dee", t0);
    assert_eq!(counts.people(&lend.name, t0 + WINDOW), 3);
}
