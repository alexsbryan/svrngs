//! The `svrngs` command with n0 out of its path, for relay-n0-off's run
//! (`demo/runs/n0-off/2026-10-12/run.py`). `the_command` is what
//! `src/main.rs` runs — `cli::parse`, `cli::data_dir`, `cli::run`, its
//! exit codes and its tracing — save its relay: `RelayConfig { relay_urls:
//! [$N0OFF_RELAY], n0_services: false }`, the command's own posture since
//! relay-default (`cli::reach`), its relay this run's on 127.0.0.1.
//! `the_relay` is iroh's relay on 127.0.0.1, plain HTTP, as the browser
//! tests' harness starts it. Ignored: `scripts/check.sh` never runs them;
//! run.py does, each a process of its own, by name:
//!
//! ```text
//! N0OFF_RELAY_AT=<file> <test binary> --ignored --exact the_relay
//! N0OFF_RELAY=<url> N0OFF_ARGS=<the args, a line each> HOME=<home> \
//!   <test binary> --ignored --exact the_command --nocapture
//! ```

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs::cli;
use svrngs_node::iroh::RelayConfig;

fn var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is set by run.py"))
}

/// Each line's decisions at the level `RUST_LOG` names, as main.rs does.
fn traced() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error")),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
}

#[test]
#[ignore = "relay-n0-off's run, run by demo/runs/n0-off/2026-10-12/run.py"]
fn the_command() {
    traced();
    let args = std::iter::once("svrngs".to_string())
        .chain(var("N0OFF_ARGS").lines().map(str::to_string))
        .collect::<Vec<_>>();
    let relay = RelayConfig {
        relay_urls: vec![var("N0OFF_RELAY")],
        n0_services: false,
        loopback_only: false,
    };
    let code = match cli::parse(args) {
        Err(why) => {
            eprintln!("svrngs: {why}");
            1
        }
        Ok(command) => {
            let root = cli::data_dir().expect("a data dir");
            let runtime = tokio::runtime::Runtime::new().expect("a runtime");
            match runtime.block_on(cli::run(command, root, &relay)) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("svrngs: {e}");
                    i32::from(e.exit_code())
                }
            }
        }
    };
    std::process::exit(code);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "relay-n0-off's run, run by demo/runs/n0-off/2026-10-12/run.py"]
async fn the_relay() {
    traced();
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    let relay = Server::spawn(config)
        .await
        .expect("a relay starts on 127.0.0.1");
    let url = format!("http://{}/", relay.http_addr().expect("it serves HTTP"));
    std::fs::write(var("N0OFF_RELAY_AT"), &url).unwrap();
    eprintln!("relay at {url}");
    svrngs_node::node::asked_to_stop().await;
}
