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
//! It writes `PERF_DIR/group` (the group's id) and `PERF_DIR/addr.json`
//! (its key, loopback sockets and record API) once up, then reads `PERF_DIR/peers.json` (every member's
//! `addr.json`, a list) each time the driver writes it anew, and runs until killed.
//! Its log is the node's traces on stderr, as `RUST_LOG` asks.
//!
//! `perf_relay` runs iroh's relay on 127.0.0.1, plain HTTP (the conformance
//! canary's, `tests/browser/harness.rs`), writing its URL to `PERF_RELAY_URL`.
//!
//! `perf_blobs` is the measurement perf-1's candidate 1 names (its README
//! §10): one get of `PERF_SIZE` bytes between two iroh-blobs stores on
//! loopback, past every svrngs path but the endpoint's builder, so what of
//! T3 remains is said iroh's or svrngs's. It prints one JSON line.

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
fn perf_blobs() {
    use iroh_blobs::api::remote::GetProgressItem;
    use iroh_blobs::store::fs::FsStore;
    use n0_future::StreamExt as _;
    use std::time::Instant;
    let size: usize = env("PERF_SIZE").parse().unwrap();
    let dir = std::path::PathBuf::from(env("PERF_DIR"));
    let ms = |t: Instant| t.elapsed().as_micros() as f64 / 1000.0;
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let relay = svrngs_node::guest::RelayConfig::from_parts(Vec::new(), Some("none"))
            .unwrap()
            .on_loopback_only();
        let alpn = vec![iroh_blobs::ALPN.to_vec()];
        let key = |b| svrngs_node::iroh::SecretKey::from_bytes(&[b; 32]);
        let bind = |b| svrngs_node::guest::build_relayed_endpoint(key(b), alpn.clone(), &relay);
        let (a, b) = (bind(1).await.unwrap(), bind(2).await.unwrap());
        let (held, asks) = (
            FsStore::load(dir.join("a")).await.unwrap(),
            FsStore::load(dir.join("b")).await.unwrap(),
        );
        let mut bytes = vec![0; size];
        blake3::Hasher::new().finalize_xof().fill(&mut bytes);
        let t = Instant::now();
        let hash = held.add_bytes(bytes).await.unwrap().hash;
        let import_ms = ms(t);
        let blobs = iroh_blobs::BlobsProtocol::new(&held, None);
        let _router = iroh::protocol::Router::builder(a.clone())
            .accept(iroh_blobs::ALPN, blobs)
            .spawn();
        let t = Instant::now();
        let conn = b.connect(a.addr(), iroh_blobs::ALPN).await.unwrap();
        let dial_ms = ms(t);
        let relayed = conn
            .paths()
            .iter()
            .find(|p| p.is_selected())
            .map(|p| p.is_relay());
        let request = iroh_blobs::protocol::GetRequest::blob(hash);
        let mut got = std::pin::pin!(asks.remote().execute_get(conn, request).stream());
        let (t, mut first_ms) = (Instant::now(), None);
        let stats = loop {
            match got.next().await.expect("the get ends with a result") {
                GetProgressItem::Progress(_) => first_ms = first_ms.or(Some(ms(t))),
                GetProgressItem::Done(stats) => break stats,
                GetProgressItem::Error(e) => panic!("the get: {e:?}"),
            }
        };
        let last_ms = ms(t);
        let mb_s = size as f64 / (last_ms / 1000.0) / 1e6;
        let said = serde_json::json!({"size": size, "import_ms": import_ms, "dial_ms": dial_ms,
            "first_ms": first_ms, "last_ms": last_ms, "mb_s": mb_s, "relayed": relayed,
            "payload": stats.payload_bytes_read});
        println!("PERF_BLOBS {said}");
    });
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
