// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/publish_cmd.rs@e1a43bcd2
//! `resolve_target`'s tests, publish_cmd.rs:559-591.
use super::*;

/// A bare port is the common form and must mean loopback, never the
/// wildcard — the difference between "the house can reach it" and "the
/// coffee shop wifi can".
#[test]
fn a_bare_port_resolves_to_loopback() {
    assert_eq!(
        resolve_target("5000").unwrap(),
        "127.0.0.1:5000".parse().unwrap()
    );
}

/// The failing inputs. A routable target is refused rather than accepted
/// and quietly published, because accepting it would put somebody's app
/// on the LAN with no identity on the caller.
#[test]
fn a_routable_or_nonsense_target_is_refused_by_name() {
    for bad in ["0.0.0.0:5000", "192.168.1.5:80", "not-a-port", "0"] {
        let e = resolve_target(bad).unwrap_err();
        assert!(!e.is_empty(), "{bad} must be refused with a reason");
    }
    assert!(resolve_target("0.0.0.0:5000")
        .unwrap_err()
        .contains("not loopback"));
}

#[test]
fn an_explicit_loopback_host_port_is_taken_as_given() {
    assert_eq!(
        resolve_target("127.0.0.1:8096").unwrap(),
        "127.0.0.1:8096".parse().unwrap()
    );
    assert!(resolve_target("[::1]:8096").is_ok());
}
