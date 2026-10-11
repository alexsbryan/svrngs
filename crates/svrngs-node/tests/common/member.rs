//! One member of a group as a process of its own (perf-1's instrument,
//! moved here from tests/perf.rs at syp-4 (d), unchanged, for the test of
//! a store across a stop to start too); its environment as perf.rs's
//! module docs say.

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::common::loopback_sockets;
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::seed::Seed;
use svrngs_node::{actor_of, Name, Person, RailAct, SigningKey};

pub fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set"))
}

pub fn signing(i: usize) -> SigningKey {
    SigningKey::from_bytes(&[i as u8 + 1; 32])
}

pub fn seed_file(dir: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(dir.join("seed.json")) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Addr {
    pub me: usize,
    pub id: String,
    pub sockets: Vec<std::net::SocketAddr>,
    pub api: std::net::SocketAddr,
}

pub async fn node() {
    let (dir, me) = (
        PathBuf::from(env("PERF_DIR")),
        env("PERF_ME").parse().unwrap(),
    );
    let members: usize = env("PERF_MEMBERS").parse().unwrap();
    let path = env("PERF_PATH");
    let relay = match path.as_str() {
        "loopback" => RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: true,
        },
        "relay" => RelayConfig {
            relay_urls: vec![env("PERF_RELAY")],
            n0_services: false,
            loopback_only: true,
        },
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
    // The group's id, for the driver's `share --folder` ask (syp-4 a).
    std::fs::write(dir.join("group"), &id).unwrap();
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
