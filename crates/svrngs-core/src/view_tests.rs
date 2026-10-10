// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/view.rs@e1a43bcd2
use super::*;
use crate::tests_support::{after, key, record, run, with_ids};

/// `n` acts by `seed`'s key from seq 0, each naming the one before.
fn ops_of(seed: u8, n: usize) -> Vec<Op<SignedOp>> {
    run(&key(seed), 100, (0..n).map(|_| record("x")))
}

/// **Golden vectors — frozen 2026-09-23 from this implementation**, and
/// regenerated 2026-10-03 when an act's id became the whole hash over its
/// previous act too (svrngs, U2): the fold is unchanged, the ids it folds
/// are not.
/// RFC style: generated once, then pinned. Their job is cross-
/// implementation conformance and refactor-proofing; a reimplementation
/// of the fold must reproduce these byte-for-byte, which is also what
/// makes the A3 signing-bytes vectors meaningful.
#[test]
fn view_golden_vectors() {
    let actor = crate::actor_of(&key(1));
    // (from, mark, ops, expected head)
    let cases: &[(u64, u64, Vec<Op<SignedOp>>, &str)] = &[
        // An empty window folds to the bare domain hash.
        (
            0,
            0,
            vec![],
            "6b0ee48a13235b72410a661e043021f58e3c27dcd38c1bd39b80e90c3247e056",
        ),
        // One op at seq 0.
        (
            0,
            0,
            ops_of(1, 1),
            "39c2dfec1ea98e955950c62aa3f7a25edc20d75d1571839dc82ac2e2c72b24c2",
        ),
        // The same op at a different mark is a different window.
        (
            0,
            1,
            ops_of(1, 2),
            "3ba4f8041a4c8b7f7664d47bebc1c9c6fe4dc5e78f406093702d74968c84a791",
        ),
        // The same window from a different floor is a different claim.
        (
            1,
            1,
            ops_of(1, 2),
            "1fe28d302b0b042b812a4de8f84c61d81491c4fb5bb06e8f9cb98e72cc467048",
        ),
    ];
    for (from, mark, ops, expected) in cases {
        let view = View::of(&actor, *from, *mark, with_ids(ops.iter()));
        assert_eq!(
            view.head, *expected,
            "golden vector (from={from}, mark={mark}) — update ONLY with a domain bump \
             or a change to what an id is"
        );
    }

    // The chain reacts to content, not to arrival: two different ops at
    // one seq fold differently, and the same set folds identically
    // however it is presented.
    let prefix = ops_of(1, 2);
    let left = after(&key(1), &prefix[1], 200, record("left"));
    let right = after(&key(1), &prefix[1], 200, record("right"));
    let mut a = prefix.clone();
    a.push(left.clone());
    let mut b = prefix.clone();
    b.push(right.clone());
    let mut both = a.clone();
    both.push(right.clone());
    let mut both_reordered = vec![right, left];
    both_reordered.extend(prefix.iter().cloned());
    assert_ne!(
        View::of(&actor, 0, 2, with_ids(a.iter())).head,
        View::of(&actor, 0, 2, with_ids(b.iter())).head,
        "a fork at one seq must not fold equal"
    );
    assert_eq!(
        View::of(&actor, 0, 2, with_ids(both.iter())).head,
        View::of(&actor, 0, 2, with_ids(both_reordered.iter())).head,
        "the fold is over the set, not the arrival order"
    );
}

/// The envelope's refusals are named. Silence here is how a commitment
/// becomes a suggestion (ARCH §18.3).
#[test]
fn an_unknown_digest_version_is_a_named_refusal() {
    let v1 =
        serde_json::json!({"8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c": 5});
    let err = serde_json::from_value::<Digest>(v1).unwrap_err();
    assert!(err.to_string().contains("v1"), "{err}");
    assert!(err.to_string().contains("marks-only"), "{err}");

    let v3 = serde_json::json!({"v": 3, "entries": {}});
    let err = serde_json::from_value::<Digest>(v3).unwrap_err();
    assert!(err.to_string().contains("v3"), "{err}");

    let garbage = serde_json::json!({"who": "knows"});
    let err = serde_json::from_value::<Digest>(garbage).unwrap_err();
    assert!(err.to_string().contains("unrecognised"), "{err}");

    // And v2 round-trips.
    let d = Digest::from(BTreeMap::from([(
        "a".to_string(),
        View {
            from: 0,
            mark: 0,
            head: "h".into(),
        },
    )]));
    let wire = serde_json::to_value(&d).unwrap();
    assert_eq!(wire["v"], 2);
    assert_eq!(serde_json::from_value::<Digest>(wire).unwrap(), d);
}

/// The fixture is the SHAREABLE artifact — the spec-with-vectors a
/// second implementation checks against without reading a line of Rust
/// (the conformance proof the golden vectors exist for;
/// `scripts/view_conformance.py` is the second implementation, written
/// from the formula above). This test pins the fixture to the reference
/// so the two cannot drift; regenerate with
/// `cargo test --lib emit_view_fixture -- --ignored --nocapture`.
#[test]
fn the_view_fixture_matches_the_reference() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/view_golden.json"))
            .expect("the fixture is JSON");
    for case in fixture["cases"].as_array().expect("cases") {
        let actor = case["actor"].as_str().unwrap().to_string();
        let from = case["from"].as_u64().unwrap();
        let mark = case["mark"].as_u64().unwrap();
        let pairs: Vec<(u64, String)> = case["ops"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_u64().unwrap(), p[1].as_str().unwrap().to_string()))
            .collect();
        let head = View::fold_pairs(&actor, from, mark, &pairs).head;
        assert_eq!(
            head,
            case["head"].as_str().unwrap(),
            "case {} drifted from the fixture",
            case["name"]
        );
    }
}

/// The fixture generator. Prints the JSON to paste into
/// `fixtures/view_golden.json`; kept so the freeze is reproducible
/// (RFC style: generated once, then pinned).
#[test]
#[ignore = "fixture generator — run with --ignored --nocapture"]
fn emit_view_fixture() {
    let actor = crate::actor_of(&key(1));
    let ops = ops_of(1, 2);
    let (one, two) = (&ops[0], &ops[1]);
    let left = after(&key(1), two, 102, record("left"));
    let right = after(&key(1), two, 102, record("right"));
    let mut cases = Vec::new();
    for (name, from, mark, ops) in [
        ("empty window", 0u64, 0u64, vec![]),
        ("one op", 0, 0, vec![one]),
        ("two ops", 0, 1, vec![one, two]),
        ("offset window", 1, 1, vec![one, two]),
        (
            "a fork at one seq — ids sorted and deduped",
            0,
            2,
            vec![one, &left, &right, two, &left],
        ),
    ] {
        let pairs: Vec<(u64, String)> =
            ops.iter().map(|o| (o.kind.seq, o.id.to_string())).collect();
        let head = View::fold_pairs(&actor, from, mark, &pairs).head;
        cases.push(serde_json::json!({
            "name": name, "actor": actor, "from": from, "mark": mark,
            "ops": pairs, "head": head,
        }));
    }
    let doc = serde_json::json!({
        "domain": "cwth/view/1",
        "generated": "2026-09-24 from the reference implementation — RFC style: generated once, then pinned; regenerated 2026-10-03 when an act's id became the whole hash (svrngs U2)",
        "formula": "h0 = BLAKE3(domain || 0x00 || actor); hs = BLAKE3(h{s-1} || 0x00 || be64(s) || 0x00 || id), ids at s sorted then deduped, s in [from, mark]",
        "cases": cases,
    });
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}
