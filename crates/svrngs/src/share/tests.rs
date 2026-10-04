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
/// the limit, else three. Nothing listening, or no kind to be had, is
/// refused by name.
#[test]
fn a_lend_is_decided_from_what_was_typed_and_what_listens() {
    let jf = Kind::parse("jellyfin").unwrap();
    let lend = lend_of(&share(8096), true, Some(jf.clone())).unwrap();
    assert_eq!(
        (lend.name.as_str(), lend.kind.as_str(), lend.limit.get()),
        ("jellyfin", "jellyfin", 3)
    );
    let mut typed = share(5000);
    typed.kind = Some(Kind::parse("notes").unwrap());
    typed.name = Some(LendName::parse("pad").unwrap());
    typed.limit = Some(port(7));
    let lend = lend_of(&typed, true, None).unwrap();
    assert_eq!(
        (lend.name.as_str(), lend.kind.as_str(), lend.limit.get()),
        ("pad", "notes", 7)
    );
    assert_eq!(
        lend_of(&share(5000), true, None),
        Err(ShareRefusal::NoKind(port(5000)))
    );
    assert_eq!(
        lend_of(&share(8096), false, Some(jf)),
        Err(ShareRefusal::NothingListens(port(8096)))
    );
}
