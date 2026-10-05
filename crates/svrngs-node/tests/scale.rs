//! The scale instrument: how one group's cost grows with its members, its
//! journal and its asleep computers. An instrument, not a check: it is
//! `#[ignore]`d, so `scripts/check.sh` never runs it, and it asserts only
//! what makes a reading meaningful (the group converged, every member was
//! reached, an idle round moved nothing).
//!
//! It stands up real nodes — the record, the round, iroh — as `canaries.rs`
//! does: in one process, no relay, peers reached by seeded loopback
//! addresses. Every member writes an equal share of the journal, so each
//! act carries a view naming every member (`svrngs-core`
//! `SignedOp::view`), as it does in a group where everyone has written.
//! Each node is started with a round every day (`Node::start_every`), so no
//! node's own round runs inside one being timed.
//!
//! Run, one point at a time, in release:
//!
//! ```sh
//! SVRNGS_SCALE_MEMBERS=25 SVRNGS_SCALE_ACTS=10000 SVRNGS_SCALE_ASLEEP=0 \
//!   cargo test --release -p svrngs-node --test scale -- --ignored --nocapture
//! ```
//!
//! It prints one line, `SCALE {json}`:
//! - `idle_round_cpu_s` — the process's processor time for one computer's
//!   round with nothing to move. In one process it includes the answering
//!   side of every exchange, which is the per-computer average when every
//!   member rounds.
//! - `idle_round_wall_s` — how long that round took.
//! - `push_round_s` — the round after five appends: one write reaching
//!   everyone.
//! - `asleep_round` — with `SVRNGS_SCALE_ASLEEP` computers shut, how long
//!   one round took and how many it could not reach.
//! - `bytes_per_act`, `digest_pass_s`, `append_s`, and the journal's size.
//!
//! Its record: first run 2026-10-05 at `15a9de7` (svrn-docs
//! `rings/runs/size-2026-10-05/`). Fitted on points up to 100 members, its
//! cost line predicted the 150-member idle round at 188 CPU-seconds before
//! it ran; it took 166. One run is not a rate (AGENTS.md 7): repeat a point
//! before reading a difference smaller than about 5% (the size run's two
//! readings of one point).
//!
//! It needs the process's own processor time, which std does not give;
//! `getrusage` through `libc` is the one way the lock already holds, and the
//! one `unsafe` block below is that call.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::sync::Arc;
use std::time::{Duration, Instant};

use svrngs_core::tests_support::Batch;
use svrngs_node::iroh::{PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Digest, Payload, Person, RailAct, Roster, SigningKey};

const GROUP: &str = "scale";

/// No node's own round inside a measured one.
const A_DAY: Duration = Duration::from_secs(86_400);

fn setting(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from_bytes(&signing(seed).verifying_key().to_bytes()).unwrap()
}

/// This process's processor time so far, user and system, and its peak
/// resident memory in bytes as the platform reports it (bytes on macOS,
/// kilobytes on Linux).
fn usage() -> (f64, i64) {
    // SAFETY: `getrusage` writes one `rusage` it is given a valid pointer
    // to; a zeroed `rusage` is a valid value of that plain C struct.
    let ru = unsafe {
        let mut ru: libc::rusage = std::mem::zeroed();
        libc::getrusage(libc::RUSAGE_SELF, &mut ru);
        ru
    };
    let secs = |tv: libc::timeval| tv.tv_sec as f64 + tv.tv_usec as f64 / 1e6;
    (secs(ru.ru_utime) + secs(ru.ru_stime), ru.ru_maxrss as i64)
}

fn cpu_secs() -> f64 {
    usage().0
}

fn dir_bytes(p: &std::path::Path) -> u64 {
    let mut n = 0;
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let m = e.metadata().unwrap();
            n += if m.is_dir() {
                dir_bytes(&e.path())
            } else {
                m.len()
            };
        }
    }
    n
}

/// An expense-sized act: about two hundred bytes of payload.
fn act(i: usize) -> RailAct {
    RailAct::Record {
        payload: Payload::new(serde_json::json!({
            "kind": "scale",
            "i": i,
            "amount_cents": 4720,
            "note": "propane for the house, split four ways, with a sentence of context"
        }))
        .unwrap(),
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// Every act any node holds, ingested by every node: a converged group
/// without paying a round per member to reach it. The rounds measured
/// afterwards are the steady state, not the catch-up.
async fn converge(nodes: &[Node]) {
    let mut all = Vec::new();
    for n in nodes {
        let (ops, more) = n
            .rail()
            .ops_missing_from_within(GROUP, &Digest::new(), usize::MAX)
            .unwrap();
        assert!(!more, "an unbounded budget leaves nothing behind");
        all.extend(ops);
    }
    let all = Arc::new(all);
    let mut ingests = Vec::new();
    for n in nodes {
        let rail = n.rail().clone();
        let all = all.clone();
        ingests.push(tokio::task::spawn_blocking(move || {
            rail.ingest_all(GROUP, &all).unwrap();
        }));
    }
    for i in ingests {
        i.await.unwrap();
    }
}

async fn timed_round(node: &Node) -> (f64, f64, svrngs_node::ring_sync::RoundOutcome) {
    let (c, t) = (cpu_secs(), Instant::now());
    let out = tokio::time::timeout(Duration::from_secs(3600), node.round())
        .await
        .expect("a round finishes within the hour");
    (t.elapsed().as_secs_f64(), cpu_secs() - c, out)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 12)]
#[ignore = "an instrument, not a check: run by hand, in release (see the module docs)"]
async fn scale() {
    let members = setting("SVRNGS_SCALE_MEMBERS", 5);
    let acts = setting("SVRNGS_SCALE_ACTS", 1000);
    let asleep = setting("SVRNGS_SCALE_ASLEEP", 0).min(members - 1);
    assert!(
        (2..=250).contains(&members),
        "a key per member is seeded from one byte"
    );
    let started = Instant::now();

    let dirs: Vec<_> = (0..members).map(|_| tempfile::tempdir().unwrap()).collect();
    let no_relay = RelayConfig::from_parts(Vec::new(), Some("none")).unwrap();
    let mut nodes = Vec::new();
    for (i, d) in dirs.iter().enumerate() {
        nodes.push(
            Node::start_every(
                d.path().to_path_buf(),
                signing(i as u8 + 1),
                &no_relay,
                |_| Ok(None),
                "127.0.0.1:0".parse().unwrap(),
                A_DAY,
            )
            .await
            .expect("a node starts"),
        );
    }
    let founded = Roster::new(
        [(Person::parse("p1").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    for node in &nodes {
        node.rail().set_seed(GROUP, &founded).unwrap();
    }
    let admits: Vec<RailAct> = (2..=members)
        .map(|i| RailAct::Admit {
            person: Person::parse(&format!("p{i}")).unwrap(),
            key: actor_of(&signing(i as u8)),
            invite: None,
        })
        .collect();
    nodes[0]
        .rail()
        .commit(GROUP, &Batch(admits))
        .unwrap()
        .unwrap();
    for (i, a) in nodes.iter().enumerate() {
        for (k, b) in nodes.iter().enumerate() {
            if i != k {
                a.transport()
                    .add_known_peer(key(k as u8 + 1), loopback_sockets(b.endpoint()));
            }
        }
    }
    converge(&nodes).await;
    // One act each, converged, so every later act's view names everyone.
    for (i, node) in nodes.iter().enumerate() {
        node.rail()
            .commit(GROUP, &Batch(vec![act(i)]))
            .unwrap()
            .unwrap();
    }
    converge(&nodes).await;
    let share = acts / members;
    let mut writes = Vec::new();
    for node in &nodes {
        let rail = node.rail().clone();
        writes.push(tokio::task::spawn_blocking(move || {
            let mut left = share;
            while left > 0 {
                // One commit per five hundred: a batch shares one stamp, so
                // writing fast does not run the clock ahead (DESIGN §6).
                let k = left.min(500);
                rail.commit(GROUP, &Batch((0..k).map(act).collect()))
                    .unwrap()
                    .unwrap();
                left -= k;
            }
        }));
    }
    for w in writes {
        w.await.unwrap();
    }
    converge(&nodes).await;
    let digest = nodes[0].rail().digest(GROUP).unwrap();
    for node in &nodes[1..] {
        assert_eq!(
            node.rail().digest(GROUP).unwrap(),
            digest,
            "every copy holds the same journal before anything is measured"
        );
    }
    let setup_s = started.elapsed().as_secs_f64();

    let (held, _) = nodes[0]
        .rail()
        .ops_missing_from_within(GROUP, &Digest::new(), usize::MAX)
        .unwrap();
    let journal_acts = held.len();
    let journal_bytes = dir_bytes(&nodes[0].rail().group_dir(GROUP).unwrap());
    let digest_bytes = serde_json::to_vec(&digest).unwrap().len();
    let digest_pass_s = median(
        (0..5)
            .map(|_| {
                let t = Instant::now();
                nodes[0].rail().digest(GROUP).unwrap();
                t.elapsed().as_secs_f64()
            })
            .collect(),
    );
    let append_s = median(
        (0..5)
            .map(|i| {
                let t = Instant::now();
                nodes[0]
                    .rail()
                    .commit(GROUP, &Batch(vec![act(i)]))
                    .unwrap()
                    .unwrap();
                t.elapsed().as_secs_f64()
            })
            .collect(),
    );
    // The five appends put node 0 ahead: its next round is one write
    // reaching everyone.
    let (push_round_s, _, push) = timed_round(&nodes[0]).await;

    let (mut idle_wall, mut idle_cpu) = (Vec::new(), Vec::new());
    for _ in 0..3 {
        let (wall, cpu, out) = timed_round(&nodes[0]).await;
        assert_eq!(
            out.peers_reached,
            members - 1,
            "every member reached: {out:?}"
        );
        assert_eq!(
            out.ops_pulled + out.ops_pushed,
            0,
            "an idle round moves nothing: {out:?}"
        );
        idle_wall.push(wall);
        idle_cpu.push(cpu);
    }

    let mut asleep_round = None;
    if asleep > 0 {
        for node in &nodes[members - asleep..] {
            node.endpoint().close().await;
        }
        let (wall, _, out) = timed_round(&nodes[0]).await;
        asleep_round = Some((wall, out.peers_unreachable));
    }

    println!(
        "SCALE {}",
        serde_json::json!({
            "members": members, "acts": acts, "asleep": asleep,
            "setup_s": setup_s,
            "journal_acts": journal_acts,
            "journal_bytes": journal_bytes,
            "bytes_per_act": journal_bytes as f64 / journal_acts as f64,
            "digest_bytes": digest_bytes,
            "digest_pass_s": digest_pass_s,
            "append_s": append_s,
            "push_round_s": push_round_s,
            "push_pushed": push.ops_pushed,
            "idle_round_wall_s": median(idle_wall),
            "idle_round_cpu_s": median(idle_cpu),
            "asleep_round": asleep_round,
            "max_rss": usage().1,
        })
    );
    for node in &nodes[..members - asleep] {
        node.endpoint().close().await;
    }
}
