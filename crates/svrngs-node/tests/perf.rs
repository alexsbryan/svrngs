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
use common::loopback_sockets;

use std::path::{Path, PathBuf};
use std::time::Duration;

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::seed::Seed;
use svrngs_node::{actor_of, Name, Person, RailAct, SigningKey};

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set"))
}

fn signing(i: usize) -> SigningKey {
    SigningKey::from_bytes(&[i as u8 + 1; 32])
}

fn seed_file(dir: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(dir.join("seed.json")) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Addr {
    me: usize,
    id: String,
    sockets: Vec<std::net::SocketAddr>,
    api: std::net::SocketAddr,
}

#[test]
#[ignore = "an instrument the perf run's driver starts"]
fn perf_node() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(node());
}

async fn node() {
    let (dir, me) = (
        PathBuf::from(env("PERF_DIR")),
        env("PERF_ME").parse().unwrap(),
    );
    let members: usize = env("PERF_MEMBERS").parse().unwrap();
    let path = env("PERF_PATH");
    let relay = match path.as_str() {
        "loopback" => RelayConfig::from_parts(Vec::new(), Some("none"))
            .unwrap()
            .on_loopback_only(),
        "relay" => RelayConfig::from_parts(vec![env("PERF_RELAY")], Some("none"))
            .unwrap()
            .on_loopback_only(),
        "n0" => RelayConfig::default(),
        other => panic!("PERF_PATH {other}: loopback, relay or n0"),
    };
    let node = Node::start(
        dir.clone(),
        signing(me),
        &relay,
        seed_file,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    let seed = Seed::from_parts(
        Name::parse("perf").unwrap(),
        Name::parse("p0").unwrap(),
        signing(0).verifying_key(),
        [9; 16],
    );
    let id = seed.id();
    node.rail().set_seed(&id, &seed.roster()).unwrap();
    let group = node.rail().group_dir(&id).unwrap();
    std::fs::write(group.join("seed.json"), seed.to_text()).unwrap();
    let admitted = group.join("perf-admitted");
    if me == 0 && !admitted.exists() {
        for i in 1..members {
            let admit = svrngs_core::tests_support::Fixture {
                act: RailAct::Admit {
                    person: Person::parse(&format!("p{i}")).unwrap(),
                    key: actor_of(&signing(i)),
                    invite: None,
                },
                on_behalf_of: None,
            };
            node.rail().commit(&id, &admit).unwrap().unwrap();
        }
        std::fs::write(&admitted, b"").unwrap();
    }
    if path == "relay" {
        tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
            .await
            .ok();
    }
    let addr = Addr {
        me,
        id: node.endpoint().id().to_string(),
        sockets: loopback_sockets(node.endpoint()),
        api: node.api_addr(),
    };
    let at = dir.join("addr.json");
    std::fs::write(at.with_extension("tmp"), serde_json::to_vec(&addr).unwrap()).unwrap();
    std::fs::rename(at.with_extension("tmp"), &at).unwrap();
    let peers = dir.join("peers.json");
    let mut told = Vec::new();
    loop {
        if let Ok(text) = std::fs::read(&peers) {
            if text != told {
                let all: Vec<Addr> = serde_json::from_slice(&text).unwrap();
                let book = MemoryLookup::new();
                for other in all.iter().filter(|a| a.me != me) {
                    let key: PublicKey = other.id.parse().unwrap();
                    let mut ea = EndpointAddr::new(key);
                    match path.as_str() {
                        "loopback" => {
                            for s in &other.sockets {
                                ea = ea.with_ip_addr(*s);
                            }
                            node.transport().add_known_peer(key, other.sockets.clone());
                        }
                        "relay" => ea = ea.with_relay_url(env("PERF_RELAY").parse().unwrap()),
                        _ => {}
                    }
                    book.add_endpoint_info(ea);
                }
                if path != "n0" {
                    node.endpoint().address_lookup().unwrap().add(book);
                }
                told = text;
                eprintln!("PERF told of {} members", all.len() - 1);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
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
