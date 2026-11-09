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
//! `perf_partial` is step 0 of `docs/LESSONS_FROM_BITTORRENT.md` §6: a member
//! that stopped a get mid-file serves exactly what it holds, verified, and a
//! byte flipped in its data file is refused. It prints one JSON line.
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
        let relay = svrngs_node::guest::RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: true,
        };
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
fn perf_partial() {
    use iroh_blobs::api::proto::Bitfield;
    use iroh_blobs::api::remote::GetProgressItem;
    use iroh_blobs::protocol::{ChunkRanges, ChunkRangesExt, GetRequest, ObserveRequest};
    use iroh_blobs::store::fs::FsStore;
    use n0_future::StreamExt as _;
    let size: usize = env("PERF_SIZE").parse().unwrap();
    let dir = std::path::PathBuf::from(env("PERF_DIR"));
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let relay = svrngs_node::guest::RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: true,
        };
        let alpn = vec![iroh_blobs::ALPN.to_vec()];
        let key = |b| svrngs_node::iroh::SecretKey::from_bytes(&[b; 32]);
        let bind = |b| svrngs_node::guest::build_relayed_endpoint(key(b), alpn.clone(), &relay);
        let (a_ep, h_ep, m_ep, p_ep) = (
            bind(1).await.unwrap(),
            bind(2).await.unwrap(),
            bind(3).await.unwrap(),
            bind(4).await.unwrap(),
        );
        let (a, h, m, p) = (
            FsStore::load(dir.join("a")).await.unwrap(),
            FsStore::load(dir.join("h")).await.unwrap(),
            FsStore::load(dir.join("m")).await.unwrap(),
            FsStore::load(dir.join("p")).await.unwrap(),
        );
        let mut bytes = vec![0; size];
        blake3::Hasher::new().finalize_xof().fill(&mut bytes);
        let hash = a.add_bytes(bytes.clone()).await.unwrap().hash;
        let _a_router = iroh::protocol::Router::builder(a_ep.clone())
            .accept(iroh_blobs::ALPN, iroh_blobs::BlobsProtocol::new(&a, None))
            .spawn();
        let _h_router = iroh::protocol::Router::builder(h_ep.clone())
            .accept(iroh_blobs::ALPN, iroh_blobs::BlobsProtocol::new(&h, None))
            .spawn();
        // H keeps the partial from the sweep, then stops its get mid-file.
        let _held = h.tags().temp_tag(hash).await.unwrap();
        let conn = h_ep.connect(a_ep.addr(), iroh_blobs::ALPN).await.unwrap();
        {
            let mut got =
                std::pin::pin!(h.remote().execute_get(conn, GetRequest::blob(hash)).stream());
            while let Some(GetProgressItem::Progress(n)) = got.next().await {
                if n >= (size / 2) as u64 {
                    break;
                }
            }
        }
        h.sync_db().await.unwrap();
        let held: Bitfield = h.observe(hash).await.unwrap();
        // M asks H what it holds, then for exactly that.
        let conn = m_ep.connect(h_ep.addr(), iroh_blobs::ALPN).await.unwrap();
        let mut saw = std::pin::pin!(m.remote().observe(conn.clone(), ObserveRequest::new(hash)));
        let saw = saw.next().await.expect("a first bitfield").unwrap();
        let _mine = m.tags().temp_tag(hash).await.unwrap();
        let wanted = saw.ranges.clone() & ChunkRanges::bytes(0..size as u64);
        let mut get = std::pin::pin!(m.remote().execute_get(conn, GetRequest::blob_ranges(hash, wanted.clone())).stream());
        let done = loop {
            match get.next().await.expect("the get ends") {
                GetProgressItem::Progress(_) => {}
                GetProgressItem::Done(s) => break Ok(s.payload_bytes_read),
                GetProgressItem::Error(e) => break Err(format!("{e:?}")),
            }
        };
        let mine: Bitfield = m.observe(hash).await.unwrap();
        let exact = mine.ranges == wanted || mine.ranges == saw.ranges;
        let byte_ranges: Vec<_> = saw.ranges.iter().collect();
        // The last whole chunk of the held prefix, then compared as bytes.
        let upto = wanted.boundaries().last().map_or(0, |c| c.0) * 1024;
        let copied = m.export_ranges(hash, 0..upto.min(size as u64)).concatenate().await.unwrap();
        let bytes_match = copied == bytes[..upto.min(size as u64) as usize];
        // The plant: a byte of the held half flipped in H's data file.
        let data = h_dir_data(&dir.join("h"), hash).expect("the half-holder's data file");
        let mut file = std::fs::read(&data).unwrap();
        file[1 << 20] ^= 1;
        std::fs::write(&data, &file).unwrap();
        // H's store holds its partial open: stop and reopen it to read the flip.
        drop(_h_router);
        h.shutdown().await.ok();
        let h2 = FsStore::load(dir.join("h")).await.unwrap();
        let _h2_router = iroh::protocol::Router::builder(h_ep.clone())
            .accept(iroh_blobs::ALPN, iroh_blobs::BlobsProtocol::new(&h2, None))
            .spawn();
        let conn = p_ep.connect(h_ep.addr(), iroh_blobs::ALPN).await.unwrap();
        let _p_mine = p.tags().temp_tag(hash).await.unwrap();
        let mut get = std::pin::pin!(p.remote().execute_get(conn, GetRequest::blob_ranges(hash, ChunkRanges::bytes(0..4 << 20))).stream());
        let refused = loop {
            match get.next().await {
                None => break "the get ended with no result".to_string(),
                Some(GetProgressItem::Progress(_)) => {}
                Some(GetProgressItem::Done(_)) => break "accepted".to_string(),
                Some(GetProgressItem::Error(e)) => break format!("{e:?}"),
            }
        };
        // The control: an unflipped range of H's held half still arrives.
        let conn = p_ep.connect(h_ep.addr(), iroh_blobs::ALPN).await.unwrap();
        let mut get = std::pin::pin!(p.remote().execute_get(conn, GetRequest::blob_ranges(hash, ChunkRanges::bytes((4 << 20)..(8 << 20)))).stream());
        let control = loop {
            match get.next().await.expect("the control ends") {
                GetProgressItem::Progress(_) => {}
                GetProgressItem::Done(s) => break Ok(s.payload_bytes_read),
                GetProgressItem::Error(e) => break Err(format!("{e:?}")),
            }
        };
        let plant: Bitfield = p.observe(hash).await.unwrap();
        let flipped_chunk_held = !(plant.ranges.clone() & ChunkRanges::bytes((1 << 20)..(1 << 20) + 1)).is_empty();
        let said = serde_json::json!({
            "size": size, "held_bytes": held.total_bytes(), "ranges": byte_ranges.len(),
            "observed_equals_store": saw.ranges == held.ranges, "fetched": done,
            "arrived_exactly": exact, "bytes_match": bytes_match,
            "plant_flip_result": refused, "control_unflipped_range": control, "plant_flipped_chunk_held": flipped_chunk_held,
        });
        println!("PERF_PARTIAL {said}");
    });
}

/// `hash`'s data file under a store directory `dir`.
fn h_dir_data(dir: &std::path::Path, hash: iroh_blobs::Hash) -> Option<std::path::PathBuf> {
    let name = format!("{}.data", hash.to_hex());
    let mut walk = vec![dir.to_path_buf()];
    while let Some(at) = walk.pop() {
        for e in std::fs::read_dir(&at).ok()?.flatten() {
            if e.path().is_dir() {
                walk.push(e.path());
            } else if e.file_name().to_string_lossy() == name {
                return Some(e.path());
            }
        }
    }
    None
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
