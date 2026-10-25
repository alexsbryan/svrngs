// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a fold costs (capability-signed's check: "verification's cost per
//! fold measured before and after"; `demo/runs/authorship/2026-10-24/
//! DESIGN.md` §15). An instrument, not a check: `#[ignore]`d, so
//! `scripts/check.sh` runs it never. One group of `PEOPLE` computers, each
//! writing `EACH` records, folded `ROUNDS` times; it prints one JSON line,
//! the lines folded and the median fold's milliseconds.
//!
//! Run: `cargo test --release -p svrngs-core --test cost -- --ignored --nocapture`.

use std::collections::BTreeMap;
use std::time::Instant;

use svrngs_core::{actor_of, sign_act, ActSet, Ed25519Verifier, Person, RailAct, Roster, SigningKey};

const NS: &str = "cost";
const PEOPLE: u8 = 8;
const EACH: u64 = 250;
const ROUNDS: usize = 15;

#[test]
#[ignore = "an instrument: run with --ignored --nocapture"]
fn a_fold_of_computers_records() {
    let keys: Vec<SigningKey> = (1..=PEOPLE).map(|n| SigningKey::from_bytes(&[n; 32])).collect();
    let roster = Roster::new(BTreeMap::from_iter(keys.iter().enumerate().map(|(i, k)| {
        (Person::parse(&format!("p{i}")).unwrap(), vec![actor_of(k)])
    })));
    let mut ops = Vec::new();
    for k in &keys {
        let mut prev = None;
        for seq in 0..EACH {
            let body = serde_json::json!({ "op": "record", "payload": { "kind": "money/expense", "n": seq } });
            let act = RailAct::from_json(body).unwrap();
            let op = sign_act(k, NS, 1_760_000_000 + seq as i64, seq, prev, act, None, None);
            prev = Some(op.id.clone());
            ops.push(op);
        }
    }
    let set = ActSet::of(&ops);
    let mut ms: Vec<f64> = (0..ROUNDS)
        .map(|_| {
            let t = Instant::now();
            let a = svrngs_core::admit(&set, &[], &roster, NS, &Ed25519Verifier);
            assert!(a.is_complete());
            t.elapsed().as_secs_f64() * 1e3
        })
        .collect();
    ms.sort_by(f64::total_cmp);
    println!("{}", serde_json::json!({ "lines": ops.len(), "median_ms": ms[ROUNDS / 2], "min_ms": ms[0] }));
}
