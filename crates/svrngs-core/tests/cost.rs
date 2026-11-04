// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a fold costs (capability-signed's check: "verification's cost per
//! fold measured before and after"; `demo/runs/authorship/2026-10-24/
//! DESIGN.md` §15). An instrument, not a check: `#[ignore]`d, so
//! `scripts/check.sh` runs it never. One group of `PEOPLE` people, each
//! writing `EACH` records, folded `ROUNDS` times; each prints one JSON line,
//! the lines folded and the median fold's milliseconds. Before: each
//! person's computer signs its own. After: each person's phone's holder
//! signs, its pass on every line, verified once per distinct pass; and
//! the computers' records past the keeper's cutover, whose cover is walked
//! by ancestry on every fold.
//!
//! Run: `cargo test --release -p svrngs-core --test cost -- --ignored --nocapture`.

// An instrument times a fold: the clock is what it reads.
#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::time::Instant;

use svrngs_core::pass::{Label, Pass};
use svrngs_core::{
    actor_of, sign_act, ActSet, Ed25519Verifier, Op, OpId, Person, RailAct, Roster, SignedOp,
    SigningKey,
};

const NS: &str = "cost";
const PEOPLE: u8 = 8;
const EACH: u64 = 250;
const ROUNDS: usize = 15;

fn keys(from: u8) -> Vec<SigningKey> {
    (from..from + PEOPLE)
        .map(|n| SigningKey::from_bytes(&[n; 32]))
        .collect()
}

/// Each person by their computer's key; the keeper is p0's, whose name
/// sorts first.
fn roster(computers: &[SigningKey]) -> Roster {
    Roster::new(BTreeMap::from_iter(computers.iter().enumerate().map(
        |(i, k)| (Person::parse(&format!("p{i}")).unwrap(), vec![actor_of(k)]),
    )))
}

/// `EACH` records signed by `k`, chained, each under `grant` if any.
fn records(k: &SigningKey, grant: Option<&Pass>) -> Vec<Op<SignedOp>> {
    let mut prev: Option<OpId> = None;
    (0..EACH)
        .map(|seq| {
            let body = serde_json::json!({ "op": "record", "payload": { "kind": "money/expense", "n": seq } });
            let act = RailAct::from_json(body).unwrap();
            let ts = 1_760_000_000 + seq as i64;
            let op = sign_act(k, NS, ts, seq, prev.take(), act, None, None, grant.cloned());
            prev = Some(op.id.clone());
            op
        })
        .collect()
}

/// The median and fastest of `ROUNDS` folds of `ops`, each complete with
/// no gap, as one JSON line.
fn timed(what: &str, ops: &[Op<SignedOp>], roster: &Roster) {
    let set = ActSet::of(ops);
    let mut ms: Vec<f64> = (0..ROUNDS)
        .map(|_| {
            let t = Instant::now();
            let a = svrngs_core::admit(&set, &[], roster, NS, &Ed25519Verifier);
            assert!(a.is_complete() && a.gaps().is_empty(), "{:?}", a.gaps());
            t.elapsed().as_secs_f64() * 1e3
        })
        .collect();
    ms.sort_by(f64::total_cmp);
    println!(
        "{}",
        serde_json::json!({ "fold": what, "lines": ops.len(), "median_ms": ms[ROUNDS / 2], "min_ms": ms[0] })
    );
}

/// Before: each computer signs its own records.
#[test]
#[ignore = "an instrument: run with --ignored --nocapture"]
fn a_fold_of_computers_records() {
    let computers = keys(1);
    let ops: Vec<_> = computers.iter().flat_map(|k| records(k, None)).collect();
    timed("computers", &ops, &roster(&computers));
}

/// After: each person's phone's holder signs, one pass of Money's on every
/// line, so `PEOPLE` distinct passes.
#[test]
#[ignore = "an instrument: run with --ignored --nocapture"]
fn a_fold_of_holders_records() {
    let (computers, holders) = (keys(1), keys(101));
    let ops: Vec<_> = (computers.iter().zip(&holders))
        .flat_map(|(k, h)| {
            let app = Label::carried(&k.verifying_key().to_bytes(), "money", NS);
            let pass = Pass::sign(k, &actor_of(h), (app, vec![]), 1_800_000_000);
            records(h, Some(&pass))
        })
        .collect();
    timed("holders", &ops, &roster(&computers));
}

/// After: the computers' records, then the keeper's cutover naming each
/// key's newest, which every fold walks back through each chain.
#[test]
#[ignore = "an instrument: run with --ignored --nocapture"]
fn a_fold_past_the_keepers_cutover() {
    let computers = keys(1);
    let mut ops: Vec<_> = computers.iter().flat_map(|k| records(k, None)).collect();
    let through = (ops.iter().filter(|o| o.kind.seq == EACH - 1))
        .map(|o| o.id.clone())
        .collect();
    let keeper = ops.iter().rfind(|o| o.actor == actor_of(&computers[0]));
    let prev = keeper.map(|o| o.id.clone());
    let act = RailAct::Signers { through };
    let ts = 1_760_000_000 + EACH as i64;
    ops.push(sign_act(
        &computers[0],
        NS,
        ts,
        EACH,
        prev,
        act,
        None,
        None,
        None,
    ));
    let set = ActSet::of(&ops);
    let a = svrngs_core::admit(&set, &[], &roster(&computers), NS, &Ed25519Verifier);
    assert!(
        a.membership().signers.is_some(),
        "the keeper's cutover counts"
    );
    timed("cutover", &ops, &roster(&computers));
}
