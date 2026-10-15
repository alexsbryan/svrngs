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

/// Every line [`the_egress_posture_is_one_info_line_naming_no_proxy`] traces.
#[expect(clippy::disallowed_types, reason = "a test's trace sink")]
static TRACES: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(Vec::new());

struct Kept;

impl std::io::Write for Kept {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        TRACES.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// **The egress posture, one info line** (arch-peer (d), upstream's proxy
/// and egress logging cut): building an endpoint says at info, once, its n0
/// posture, whether it listens on loopback alone, and its relays — and
/// nothing of a proxy, which nobody running svrngs sets. Planted:
/// upstream's `log_egress_posture`, which read four environment variables
/// for a proxy to say (`proxy="none"`).
#[test]
fn the_egress_posture_is_one_info_line_naming_no_proxy() {
    let kept = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("transport=info"))
        .with_writer(|| Kept)
        .with_ansi(false)
        .finish();
    let cfg = RelayConfig {
        relay_urls: vec![],
        n0_services: false,
        loopback_only: true,
    };
    let key = SecretKey::from_bytes(&[5; 32]);
    let built = tracing::subscriber::with_default(kept, || {
        relayed_endpoint_builder(key, vec![], &cfg).is_ok()
    });
    let traced = String::from_utf8_lossy(&TRACES.lock().unwrap()).into_owned();
    let said: Vec<&str> = traced
        .lines()
        .filter(|l| l.contains("egress posture"))
        .collect();
    assert!(built, "{traced}");
    assert_eq!(said.len(), 1, "{traced}");
    for field in ["n0_services=false", "loopback_only=true", "relays=[]"] {
        assert!(said[0].contains(field), "{field}: {traced}");
    }
    assert!(!said[0].contains("proxy="), "{traced}");
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
            loopback_only: true,
        },
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
/// (svrngs, arch-peer (b): beside it, the request on the stream that
/// replaces it, `iroh.rs` `on_stream`, until (d) takes the bridge out; and
/// the tests' splice, `node/tests_support.rs`, behind `test-support`.)
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
    assert_eq!(
        bridges.len(),
        3,
        "one bridge loop, one request on the stream, one tests' splice: {bridges:?}"
    );
    assert!(
        bridges.iter().any(|b| b.contains("tests_support.rs")),
        "{bridges:?}"
    );
    assert!(
        bridges.iter().any(|b| b.contains("guest.rs")),
        "{bridges:?}"
    );
    assert!(bridges.iter().any(|b| b.contains("iroh.rs")), "{bridges:?}");
    let pumps = sites(concat!("fn ", "pump("));
    assert_eq!(pumps.len(), 1, "one pump, found {pumps:?}");
}
