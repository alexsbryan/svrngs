// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a sources-v1 group costs on a computer's journal (EUDOXIA_JUDGE §6,
//! P8), as the node pays it: one commit of an app's record, and the kept
//! fold read again after a journal change ([`RingRail::register`]), at
//! `HELD` lines on disk, beside a legacy group of the same people and
//! records. An instrument, not a check: `#[ignore]`d. Each prints one JSON
//! line, medians of `ROUNDS`.
//!
//! Run: `cargo test --release -p svrngs-runtime --test sources_cost -- --ignored --nocapture`.

// An instrument times a commit: the clock is what it reads.
#![allow(clippy::disallowed_methods)]

use std::sync::Arc;
use std::time::Instant;

use svrngs_runtime::seed::Seed;
use svrngs_runtime::{
    actor_of, sign_act, Append, Device, MintInvite, Name, Op, Person, RailAct, Redeem, Redemption, RingRail, Roster,
    SignedOp, SigningKey,
};

const ROUNDS: usize = 5;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn record(n: usize) -> RailAct {
    let body = serde_json::json!({ "op": "record", "payload": { "kind": "money/expense", "n": n } });
    RailAct::from_json(body).unwrap()
}

/// `held` records by keys 1–4 in turn, each chained on its signer's newest
/// line, after `before`.
fn records(ns: &str, before: &[Op<SignedOp>], held: usize) -> Vec<Op<SignedOp>> {
    let mut tips: Vec<Option<Op<SignedOp>>> = (1..=4u8)
        .map(|k| {
            (before.iter().filter(|op| op.actor == actor_of(&key(k))))
                .max_by_key(|op| op.kind.seq)
                .cloned()
        })
        .collect();
    (0..held)
        .map(|n| {
            let i = n % 4;
            let (seq, prev) = tips[i]
                .as_ref()
                .map_or((0, None), |t| (t.kind.seq + 1, Some(t.id.clone())));
            let op = sign_act(
                &key(1 + i as u8),
                ns,
                2_000_000_000 + n as i64,
                seq,
                prev,
                record(n),
                None,
                None,
                None,
            );
            tips[i] = Some(op.clone());
            op
        })
        .collect()
}

/// Ana's sources-v1 group on her computer, Bo, Cy and Di let in by link.
fn sources(dir: &std::path::Path) -> (RingRail, String) {
    let name = |n: &str| Name::parse(n).unwrap();
    let seed = Seed::from_parts(name("House"), name("Ana"), key(1).verifying_key(), [5; 16]).with_source_policy();
    let (rail, ns) = (RingRail::new(dir, Arc::new(key(1))), seed.id());
    rail.set_seed(&ns, &seed.roster()).unwrap();
    for (k, person) in [(2u8, "Bo"), (3, "Cy"), (4, "Di")] {
        let link = key(20 + k);
        let mint = MintInvite {
            key: actor_of(&link),
            until: i64::MAX,
            device_of: None,
        };
        let invite = rail.commit(&ns, &mint).unwrap().unwrap().lines()[0].id.clone();
        let person = Person::parse(person).unwrap();
        let policy = seed.policy();
        let consent = Redemption::sign(&key(k), &ns, &invite, &person, Device::Computer, policy);
        let redeem = Redeem {
            invite,
            person,
            key: actor_of(&key(k)),
            cosig: consent.sig,
            device: Device::Computer,
            link: actor_of(&link),
            issuer: actor_of(&key(1)),
        };
        assert!(rail.commit(&ns, &redeem).unwrap().is_ok());
    }
    (rail, ns)
}

/// The same four people as a legacy seed.
fn legacy(dir: &std::path::Path) -> (RingRail, String) {
    let people = (1..=4u8).map(|k| (Person::parse(&format!("p{k}")).unwrap(), vec![actor_of(&key(k))]));
    let (rail, ns) = (RingRail::new(dir, Arc::new(key(1))), String::from("legacy-cost"));
    rail.set_seed(&ns, &Roster::new(people.collect())).unwrap();
    (rail, ns)
}

fn median(mut ms: Vec<f64>) -> f64 {
    ms.sort_by(f64::total_cmp);
    ms[ms.len() / 2]
}

fn ms(at: Instant) -> f64 {
    at.elapsed().as_secs_f64() * 1e3
}

fn measure(kind: &str, (rail, ns): (RingRail, String), held: usize) {
    let (before, _) = rail
        .ops_missing_from_within(&ns, &Default::default(), usize::MAX)
        .unwrap();
    rail.ingest_all(&ns, &records(&ns, &before, held)).unwrap();
    assert_eq!(rail.register(&ns).unwrap().members.len(), 4);
    let (mut commit, mut refold) = (Vec::new(), Vec::new());
    for n in 0..ROUNDS {
        let at = Instant::now();
        assert!(rail.commit(&ns, &Append { act: record(n) }).unwrap().is_ok());
        commit.push(ms(at));
        let at = Instant::now();
        rail.register(&ns).unwrap();
        refold.push(ms(at));
    }
    // A peer's line, synced in: the kept fold is made again, once.
    let mut synced = Vec::new();
    for n in 0..ROUNDS {
        let (all, _) = rail
            .ops_missing_from_within(&ns, &Default::default(), usize::MAX)
            .unwrap();
        rail.ingest_all(&ns, &records(&ns, &all, 2)[1..]).unwrap();
        let at = Instant::now();
        assert_eq!(rail.register(&ns).unwrap().members.len(), 4, "round {n}");
        synced.push(ms(at));
    }
    let line = serde_json::json!({ "group": kind, "lines": before.len() + held, "commit_ms": median(commit), "refold_after_commit_ms": median(refold), "refold_after_sync_ms": median(synced) });
    println!("{line}");
}

#[test]
#[ignore = "an instrument: run with --ignored --nocapture"]
fn a_sources_group_written_and_refolded_at_each_size() {
    for held in [100, 1_000, 10_000] {
        let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        measure("sources-v1", sources(a.path()), held);
        measure("legacy", legacy(b.path()), held);
    }
}
