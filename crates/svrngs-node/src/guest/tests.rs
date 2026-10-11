// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-reach/src/guest/tests.rs@e1a43bcd2
//! Unit tests of the guest dialer's private helpers, moved with them from
//! commonwealth-transport iroh.rs, and one bridge. (pb-reach-guest's proof,
//! a non-member's dial, went with upstream's `GuestTunnel`, lean-3.)

use super::*;
use crate::alpn::ALPN;

#[test]
fn parse_relay_mode_empty_is_default() {
    // Empty config = leave the caller on the preset (n0) relays.
    assert!(parse_relay_mode(&[]).is_none());
}

#[test]
fn parse_relay_mode_valid_urls_build_custom() {
    let mode = parse_relay_mode(&[
        "https://relay.corp.example:443".to_string(),
        "https://relay2.corp.example".to_string(),
    ]);
    assert!(
        matches!(mode, Some(iroh::RelayMode::Custom(_))),
        "valid relay URLs must build a custom relay map"
    );
}

#[test]
fn parse_relay_mode_all_invalid_falls_back_to_default() {
    // A fat-fingered relay URL must not abort — fall back to the
    // default relays rather than take the node offline.
    assert!(parse_relay_mode(&["not a url".to_string()]).is_none());
}

#[test]
fn parse_relay_mode_skips_bad_keeps_good() {
    let mode = parse_relay_mode(&[
        "://broken".to_string(),
        "https://relay.corp.example:443".to_string(),
    ]);
    assert!(
        matches!(mode, Some(iroh::RelayMode::Custom(_))),
        "one valid URL among bad ones still yields a custom map"
    );
}

#[test]
fn redact_userinfo_hides_credentials() {
    assert_eq!(
        redact_userinfo("https://user:secret@proxy.corp:443"),
        "https://***@proxy.corp:443"
    );
    // No userinfo → unchanged.
    assert_eq!(
        redact_userinfo("https://proxy.corp:443"),
        "https://proxy.corp:443"
    );
    // Non-URL → unchanged (don't mangle).
    assert_eq!(redact_userinfo("proxy.corp:443"), "proxy.corp:443");
}

#[tokio::test]
async fn build_relayed_endpoint_with_custom_relay_binds() {
    // A configured self-hosted relay must not break endpoint
    // construction: bind is a local operation, the relay connection
    // is a background task (so this succeeds offline). Proves the
    // relay_urls → custom RelayMode path threads through to a valid
    // endpoint. proxy_from_env is a no-op here (env unset).
    let ep = build_relayed_endpoint(
        SecretKey::from_bytes(&[77u8; 32]),
        vec![ALPN.to_vec()],
        &RelayConfig {
            relay_urls: vec!["https://relay.corp.example:443".to_string()],
            n0_services: true,
            loopback_only: false,
        },
    )
    .await
    .expect("custom-relay endpoint must bind");
    assert!(
        !ep.bound_sockets().is_empty(),
        "endpoint must bind a socket"
    );
}

/// A loopback-only endpoint binds 127.0.0.1 and nothing else, so it has
/// no other address of this computer to offer a peer (#33). Planted: the
/// posture ignored, and the endpoint binds the wildcard.
#[tokio::test]
async fn a_loopback_only_endpoint_binds_loopback_alone() {
    let ep = build_relayed_endpoint(
        SecretKey::from_bytes(&[79u8; 32]),
        vec![ALPN.to_vec()],
        &RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: false,
        }
        .on_loopback_only(),
    )
    .await
    .expect("a loopback-only endpoint binds");
    let bound = ep.bound_sockets();
    assert!(!bound.is_empty(), "endpoint must bind a socket");
    assert!(
        bound.iter().all(|a| a.ip().is_loopback()),
        "bound beyond loopback: {bound:?}"
    );
}

/// There is ONE iroh HTTP bridge: `HttpBridge`'s accept loop is the only
/// place in this leaf that opens a stream to a peer, and `pump` is defined
/// once, so a dial rides the bridge instead of keeping a loop of its
/// own. The needles are split so this file does not match itself.
#[test]
fn there_is_one_bridge_loop_and_one_pump() {
    fn rs_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rs_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    rs_files(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    let sites = |needle: &str| -> Vec<String> {
        files
            .iter()
            .flat_map(|f| {
                let text = std::fs::read_to_string(f).unwrap();
                let name = f.display().to_string();
                text.lines()
                    .enumerate()
                    .filter(|(_, l)| l.contains(needle))
                    .map(|(i, _)| format!("{name}:{}", i + 1))
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    let bridges = sites(concat!("open_bi", "()"));
    assert_eq!(bridges.len(), 1, "one bridge loop, found {bridges:?}");
    assert!(bridges[0].contains("guest.rs"), "{bridges:?}");
    let pumps = sites(concat!("fn ", "pump("));
    assert_eq!(pumps.len(), 1, "one pump, found {pumps:?}");
}
