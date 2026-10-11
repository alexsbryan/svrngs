//! The perf instrument (perf-1, `demo/runs/perf/2026-10-10/README.md` §6):
//! one member of a group as a process of its own, a real node on the path
//! asked, for the run's driver (`demo/runs/perf/2026-10-10/run.py`) to
//! start, share a directory on and stop. An instrument, not a check: both
//! tests are `#[ignore]`d, so `scripts/check.sh` runs neither.
//!
//! `perf_node` reads, from the environment:
//! - `PERF_DIR`, its data dir; `PERF_ME`, its index, and `PERF_MEMBERS`, the
//!   group's size: member `i` signs with `[i + 1; 32]`, member 0 founds the
//!   group (as `tests/folders.rs`) and admits every other at its start;
//! - `PERF_PATH`: `loopback` (no relay, no lookup, each told the others'
//!   loopback addresses), `relay` (the relay at `PERF_RELAY` alone, each
//!   told the others' keys there) or `n0` (iroh's defaults, as `svrngs`
//!   runs, nothing told).
//!
//! It writes `PERF_DIR/addr.json` (its key, loopback sockets and record
//! API) once up, then reads `PERF_DIR/peers.json` (every member's
//! `addr.json`, a list) each time the driver writes it anew, and runs until killed.
//! Its log is the node's traces on stderr, as `RUST_LOG` asks.
//!
//! `perf_relay` runs iroh's relay on 127.0.0.1, plain HTTP (the conformance
//! canary's, `tests/browser/harness.rs`), writing its URL to `PERF_RELAY_URL`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[path = "common/member.rs"]
mod member;
use member::env;

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};

#[test]
#[ignore = "an instrument the perf run's driver starts"]
fn perf_node() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(member::node());
}

#[test]
#[ignore = "an instrument the perf run's driver starts"]
fn perf_relay() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let mut config = ServerConfig::default();
        config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
        let relay: Server = Server::spawn(config)
            .await
            .expect("a relay starts on 127.0.0.1");
        let url = format!("http://{}/", relay.http_addr().unwrap());
        std::fs::write(env("PERF_RELAY_URL"), &url).unwrap();
        std::future::pending::<()>().await
    });
}
