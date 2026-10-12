// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/publish_cmd.rs@e1a43bcd2
//! `resolve_target`'s tests, publish_cmd.rs:559-591. (svrngs: a port
//! alone, so a `host:port` is refused with the rest; then the lend's
//! decision, `lend_of`, and the well-known table.)
use super::*;

fn port(p: u16) -> NonZeroU16 {
    NonZeroU16::new(p).unwrap()
}

/// A bare port is the common form and must mean loopback, never the
/// wildcard — the difference between "the house can reach it" and "the
/// coffee shop wifi can".
#[test]
fn a_bare_port_resolves_to_loopback() {
    assert_eq!(resolve_target("5000").unwrap(), port(5000));
}

/// The failing inputs. A routable target is refused rather than accepted
/// and quietly published, because accepting it would put somebody's app
/// on the LAN with no identity on the caller. (svrngs: and a loopback
/// `host:port` too, since a lend is a port on 127.0.0.1.)
#[test]
fn a_routable_or_nonsense_target_is_refused_by_name() {
    for bad in [
        "0.0.0.0:5000",
        "192.168.1.5:80",
        "not-a-port",
        "0",
        "65536",
        "127.0.0.1:8096",
        "[::1]:8096",
    ] {
        let e = resolve_target(bad).unwrap_err();
        assert!(e.contains(bad), "{bad} must be refused, named: {e}");
    }
}

fn share(p: u16) -> Share {
    Share {
        port: port(p),
        name: None,
        kind: None,
        limit: None,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        sign_in: None,
        asks: None,
    }
}

/// Every line of the table is a port and a kind, and the one listed is
/// found by its port.
#[test]
fn the_well_known_table_reads_whole() {
    let rows: Vec<&str> = WELL_KNOWN
        .lines()
        .map(|l| l.split('#').next().unwrap().trim())
        .filter(|l| !l.is_empty())
        .collect();
    assert!(!rows.is_empty());
    for row in rows {
        let (p, _) = row.split_once(' ').expect(row);
        let p = port(p.parse().expect(row));
        assert!(well_known_kind(p).is_some(), "{row}");
    }
    assert_eq!(well_known_kind(port(8096)).unwrap().as_str(), "jellyfin");
    assert_eq!(
        well_known_kind(port(8920)),
        None,
        "a TLS port is not shared by number"
    );
    assert_eq!(well_known_kind(port(5000)), None);
}

/// The kind is the one given, else the port's; the name, else the kind;
/// the limit, else none. Nothing listening, or no kind to be had, is
/// refused by name.
#[test]
fn a_lend_is_decided_from_what_was_typed_and_what_listens() {
    let jf = Kind::parse("jellyfin").unwrap();
    let lend = lend_of(&share(8096), |_| true, Some(jf.clone())).unwrap();
    assert_eq!(
        (lend.name.as_str(), lend.kind.as_str(), lend.limit),
        ("jellyfin", "jellyfin", None)
    );
    let mut typed = share(5000);
    typed.kind = Some(Kind::parse("notes").unwrap());
    typed.name = Some(LendName::parse("pad").unwrap());
    typed.limit = Some(port(7));
    let lend = lend_of(&typed, |_| true, None).unwrap();
    assert_eq!(
        (lend.name.as_str(), lend.kind.as_str(), lend.limit),
        ("pad", "notes", Some(port(7)))
    );
    assert_eq!(
        lend_of(&share(5000), |_| true, None),
        Err(ShareRefusal::NoKind(port(5000)))
    );
    assert_eq!(
        lend_of(&share(8096), |_| false, Some(jf.clone())),
        Err(ShareRefusal::NothingListens(port(8096)))
    );
    // A sign-in named is kept, and refused when nothing listens there.
    let mut signed = share(8096);
    signed.sign_in = Some(port(9000));
    assert_eq!(
        lend_of(&signed, |_| true, Some(jf.clone()))
            .unwrap()
            .sign_in,
        Some(port(9000))
    );
    assert_eq!(
        lend_of(&signed, |p| p != port(9000), Some(jf)),
        Err(ShareRefusal::NoSignIn(port(9000)))
    );
}

/// The table's third column, the kinds that keep their own accounts: one
/// for jellyfin, in DESIGN §1's words, and none for a kind without it or
/// with no row.
#[test]
fn a_kind_keeps_its_own_accounts_only_if_the_table_says_how() {
    let how = accounts_of(&Kind::parse("jellyfin").unwrap()).expect("jellyfin keeps its own");
    assert!(
        how.starts_with("your picture at the top right, then Dashboard"),
        "{how}"
    );
    for word in svrngs_node::link::KEPT_WORDS {
        assert!(!how.to_lowercase().contains(word), "{how} says {word}");
    }
    assert_eq!(accounts_of(&Kind::parse("pad").unwrap()), None);
    assert_eq!(well_known_kind(port(8096)).unwrap().as_str(), "jellyfin");
}

/// A lend shared with no limit lets in everyone in its groups at once —
/// a fourth person too — where it once let in three, a number nobody
/// chose (bash-limit (a); the operator, 2026-10-12). Planted: the limit
/// given, else three.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the time is the counts' argument"
)]
fn a_lend_shared_with_no_limit_lets_a_fourth_in_at_once() {
    use std::time::Instant;
    use svrngs_node::door::limit::{Counts, Verdict, Whereabouts};
    let jf = Kind::parse("jellyfin").unwrap();
    let lend = lend_of(&share(8096), |_| true, Some(jf)).unwrap();
    let (mut counts, now) = (Counts::default(), Instant::now());
    for who in ["ana", "bo", "cy", "dee"] {
        let verdict = counts.ask(&lend, who, Whereabouts::Away, now);
        assert!(matches!(verdict, Verdict::In { .. }), "{who}: {verdict:?}");
    }
}
