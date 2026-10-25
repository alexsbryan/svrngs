// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/tests.rs@e1a43bcd2
//! Convergence correctness, settled offline.
//!
//! Each of these is one of the failure modes the design was reviewed against.
//! The names say which interleaving loses an act, because a test called
//! `test_admit_2` teaches the next reader nothing about why the rule exists.
//!
//! **Nothing here knows what an expense is.** These tests used to be about
//! money, and half of them were really about delivery wearing a money
//! costume. The money half now lives in the reference app, in JS, where an
//! app author can read it. What is left is the part every ring app on this
//! rail inherits whether it wants to or not.

use crate::*;

use crate::tests_support::*;

// ── the property the whole design exists to have ─────────────

/// Every permutation of the same ops must produce the identical admission —
/// the act order AND the gap report. Nineteen laptops gossip in nineteen
/// orders; if this fails, two housemates read different answers off the same
/// journal and the plan says ship nothing.
///
/// Exhaustive over 6 ops (720 orderings) rather than randomised, so a failure
/// is reproducible and the coverage is a fact rather than a sample.
///
/// This used to be asserted over balances, which made the strongest property
/// in the system contingent on one tenant's arithmetic. Asserting it over the
/// admission is both stronger and true for every app that will ever sit on
/// this rail.
#[test]
fn the_admission_is_a_function_of_the_op_set_not_of_arrival_order() {
    let groceries = signed(&key(1), 100, 0, record("groceries"));
    let beer = signed(&key(2), 101, 0, record("beer"));
    let paid_back = signed(&key(3), 102, 0, record("paid-back"));
    // A correction whose timestamp is EARLIER than the op it corrects — a
    // clock-skew case that a positional walk would get wrong.
    let fix = after(
        &key(1),
        &groceries,
        99,
        RailAct::Correct {
            corrects: groceries.id.clone(),
            replacement: Some(payload("groceries-corrected")),
        },
    );
    // A correction pointing at an op nobody in this set holds.
    let dangling = after(
        &key(2),
        &beer,
        103,
        RailAct::Correct {
            corrects: stray_id(0xde),
            replacement: None,
        },
    );
    // An op from a key the roster does not know.
    let stranger = signed(&key(9), 104, 0, record("stranger"));

    let ops = vec![groceries, beer, paid_back, fix, dangling, stranger];
    let expected = admitted(&ops);
    assert!(!expected.ops().is_empty());
    assert!(
        !expected.gaps().is_empty(),
        "this fixture must exercise gaps too"
    );

    let mut seen = 0usize;
    for order in permutations(ops.len()) {
        let shuffled: Vec<_> = order.iter().map(|i| ops[*i].clone()).collect();
        assert_eq!(admitted(&shuffled), expected, "order {order:?} disagreed");
        seen += 1;
    }
    assert_eq!(seen, 720, "6! orderings");
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    fn go(current: &mut Vec<usize>, rest: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if rest.is_empty() {
            out.push(current.clone());
            return;
        }
        for i in 0..rest.len() {
            let taken = rest.remove(i);
            current.push(taken);
            go(current, rest, out);
            current.pop();
            rest.insert(i, taken);
        }
    }
    let mut out = Vec::new();
    go(&mut Vec::new(), &mut (0..n).collect(), &mut out);
    out
}

/// Two byte-identical acts in the same second by the same actor must stay two
/// acts. [`Op::new`] hashes `(prefix, ts, actor, body)` and documents the
/// collision as by-design; `seq` is in [`act_id`]'s input, which is what
/// separates them.
#[test]
fn two_identical_acts_in_the_same_second_are_two_acts() {
    let a = signed(&key(1), 100, 0, record("coffee"));
    let b = after(&key(1), &a, 100, record("coffee"));
    assert_ne!(a.id, b.id, "seq must separate them");
    let f = admitted(&[a, b]);
    assert_eq!(applied(&f), vec!["coffee", "coffee"]);
    assert!(f.is_complete(), "{:?}", f.gaps());
}

// ── correction ───────────────────────────────────────────────

/// **The reason the void set is built from every correction at once.** A
/// correction that arrives before its target must still void it, or two nodes
/// disagree purely on delivery order.
#[test]
fn a_correction_that_arrives_first_still_voids_its_target() {
    let target = signed(&key(1), 100, 0, record("wrong"));
    let fix = after(
        &key(1),
        &target,
        101,
        RailAct::Correct {
            corrects: target.id.clone(),
            replacement: Some(payload("right")),
        },
    );
    let fix_first = admitted(&[fix.clone(), target.clone()]);
    let target_first = admitted(&[target, fix]);
    assert_eq!(fix_first, target_first);
    assert_eq!(applied(&fix_first), vec!["right"]);
}

/// A correction naming an op this node does not hold is not fatal — the void
/// is recorded and applies the moment the target arrives — but it means we
/// are missing an op, and that gets said.
#[test]
fn a_correction_pointing_at_an_op_we_do_not_hold_is_reported() {
    let orphan = signed(
        &key(1),
        100,
        0,
        RailAct::Correct {
            corrects: stray_id(0xde),
            replacement: None,
        },
    );
    let f = admitted(&[orphan.clone()]);
    assert_eq!(
        f.gaps(),
        vec![RailGap::DanglingCorrection {
            by: orphan.id.clone(),
            missing: stray_id(0xde),
        }]
    );
    assert!(applied(&f).is_empty());

    // And when the target does arrive, the void applies with no repair step.
    let target = signed(&key(2), 99, 0, record("wrong"));
    let fix = signed(
        &key(1),
        100,
        0,
        RailAct::Correct {
            corrects: target.id.clone(),
            replacement: None,
        },
    );
    let healed = admitted(&[fix, target]);
    assert!(healed.is_complete(), "{:?}", healed.gaps());
    assert!(applied(&healed).is_empty(), "the target is voided");
}

/// The total order is `(ts_unix, actor, seq, id)`, but the void set does not
/// consult it at all — so a correction stamped in the past still corrects.
#[test]
fn a_correction_stamped_in_the_past_still_corrects() {
    let target = signed(&key(1), 500, 0, record("wrong"));
    let fix = after(
        &key(1),
        &target,
        1,
        RailAct::Correct {
            corrects: target.id.clone(),
            replacement: Some(payload("right")),
        },
    );
    let f = admitted(&[target, fix]);
    assert_eq!(applied(&f), vec!["right"]);
}

/// **One actor's own acts inside one second fold in the order it wrote them.**
///
/// `ts_unix` is second resolution, so a burst from one node is one timestamp;
/// before the `seq` term the tie fell to the content-derived id, which is
/// arbitrary with respect to causality. Failing input: three acts at one
/// timestamp whose ids do not sort in seq order — which is what happened on
/// the work plane on 2026-09-09 (`Submit`, `Lease`, `Complete` in one second,
/// folded lease-first, both later acts reported unreadable, and the unit ran
/// twice).
///
/// Ordering by seq costs nothing in determinism: it is on the wire, and
/// `SequenceFork` already refuses two ops sharing one.
#[test]
fn one_actors_acts_inside_one_second_fold_in_seq_order() {
    let mut ops = vec![signed(&key(1), 700, 0, record("first"))];
    for what in ["second", "third"] {
        let next = after(&key(1), ops.last().unwrap(), 700, record(what));
        ops.push(next);
    }
    // The precondition that makes this a gate rather than a coincidence: the
    // ids do NOT already sort in seq order, so the old comparator produced a
    // different answer.
    let by_id: Vec<String> = {
        let mut v: Vec<_> = ops.iter().collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v.iter()
            .map(|o| match &o.kind.act {
                RailAct::Record { payload } => serde_json::to_value(payload).expect("a payload")
                    ["what"]
                    .as_str()
                    .expect("what")
                    .to_string(),
                _ => unreachable!(),
            })
            .collect()
    };
    assert_ne!(
        by_id,
        vec!["first", "second", "third"],
        "these three payloads sort the same by id as by seq, so this test \
         cannot tell the two comparators apart — pick different ones"
    );

    assert_eq!(
        applied(&admitted(&ops)),
        vec!["first", "second", "third"],
        "one actor's acts must fold in the order it wrote them"
    );
}

/// **A correction never resurrects.** Correcting a correction cancels ITS
/// replacement and leaves the original voided; to bring an act back, write it
/// again. That is what keeps the void set one commutative scan instead of a
/// liveness walk whose answer depends on arrival order.
#[test]
fn correcting_a_correction_does_not_resurrect_the_original() {
    let original = signed(&key(1), 100, 0, record("original"));
    let first = after(
        &key(1),
        &original,
        101,
        RailAct::Correct {
            corrects: original.id.clone(),
            replacement: Some(payload("replacement")),
        },
    );
    let second = after(
        &key(1),
        &first,
        102,
        RailAct::Correct {
            corrects: first.id.clone(),
            replacement: None,
        },
    );
    let f = admitted(&[original, first, second]);
    assert!(
        applied(&f).is_empty(),
        "neither the original nor its replacement comes back: {:?}",
        applied(&f)
    );
    assert!(f.is_complete(), "{:?}", f.gaps());
}

/// A voided op stays in the list, marked, so an app can render what changed —
/// and `applied()` is the one definition of which ops a reducer sees.
#[test]
fn a_voided_op_is_still_visible_but_is_never_applied() {
    let target = signed(&key(1), 100, 0, record("wrong"));
    let fix = after(
        &key(1),
        &target,
        101,
        RailAct::Correct {
            corrects: target.id.clone(),
            replacement: Some(payload("right")),
        },
    );
    let f = admitted(&[target.clone(), fix.clone()]);
    assert_eq!(f.ops().len(), 2, "history is kept");
    let voided = f.ops().iter().find(|o| o.id == target.id).unwrap();
    assert!(voided.voided);
    assert!(!voided.applies());
    assert!(
        voided.payload.is_some(),
        "the app can still show what was corrected away"
    );
    let correction = f.ops().iter().find(|o| o.id == fix.id).unwrap();
    assert_eq!(correction.corrects.as_ref(), Some(&target.id));
    assert_eq!(applied(&f), vec!["right"]);
}

// ── lines this build cannot read ─────────────────────────────

/// **A dangling correction is named beside a complete answer.** One typo'd
/// `Correct` citing an op nobody holds must not flip every node's
/// completeness bit forever (ROOT_CAUSE_FIXES A2): nothing is hidden from
/// the fold — the record cites what nobody here can resolve, and that is a
/// contradiction to name, not an absence to carry. It heals when the cited
/// op arrives; until then the answer is complete and contested.
#[test]
fn a_dangling_correction_is_named_beside_a_complete_answer() {
    let target = stray_id(0x5e);
    let fine = signed(&key(1), 100, 0, record("fine"));
    let f = admitted(&[
        fine.clone(),
        after(
            &key(1),
            &fine,
            101,
            RailAct::Correct {
                corrects: target,
                replacement: None,
            },
        ),
    ]);
    assert!(
        f.is_complete(),
        "a citation nobody can resolve hides no data: {:?}",
        f.gaps()
    );
    assert!(
        f.gaps()
            .iter()
            .any(|g| matches!(g, RailGap::DanglingCorrection { .. })),
        "and the contradiction is named: {:?}",
        f.gaps()
    );
}

/// **Every gap kind has a named verdict.** One classification table (ARCH
/// §10.6), split by coverage: can this gap hide acts that belong in the
/// fold? A refused act CAN — it may be real data the fold turned away (the
/// pinned "an unverifiable op makes this a subset"). A tampered id cannot
/// ("this changes no outcome" — the act folds under its derived id), a
/// dangling citation cannot (it is the ask, not a hole), and since U7 a fork
/// cannot: both branches fold, so nothing at the seq is lost. The A2
/// pre-registration's parenthetical grouped fork and tamper with dangling,
/// which is where fork now sits.
#[test]
fn every_gap_class_is_classified() {
    let actor = actor_of(&key(1));
    let id = stray_id(0);
    let absences = [
        RailGap::MalformedLine {
            line: 1,
            error: "torn".into(),
        },
        RailGap::NewerVersionLine { line: 2, v: 2 },
        RailGap::BadSignature {
            id: id.clone(),
            actor: actor.clone(),
        },
        RailGap::UnknownSigner {
            id: id.clone(),
            actor: actor.clone(),
        },
        RailGap::NotAMember {
            id: id.clone(),
            actor: actor.clone(),
        },
        RailGap::WrongPrevious {
            id: id.clone(),
            actor: actor.clone(),
            seq: 1,
        },
        RailGap::SequenceHole {
            actor: actor.clone(),
            from: 4,
            to: 4,
        },
        RailGap::Ungranted {
            id: id.clone(),
            actor: actor.clone(),
            why: crate::grant::Why::Unheld,
        },
    ];
    for gap in &absences {
        assert_eq!(
            gap.class(),
            GapClass::Absence,
            "{gap:?} can hide acts that belong in the fold"
        );
    }
    let contradictions = [
        RailGap::TamperedId {
            claimed: id.clone(),
            derived: id.clone(),
        },
        RailGap::DanglingCorrection {
            by: id.clone(),
            missing: id.clone(),
        },
        RailGap::SequenceFork {
            actor: actor.clone(),
            seq: 3,
            ids: vec![id.clone()],
        },
        RailGap::Ungranted {
            id: id.clone(),
            actor: actor.clone(),
            why: crate::grant::Why::OtherKind,
        },
    ];
    for gap in &contradictions {
        assert_eq!(
            gap.class(),
            GapClass::Contradiction,
            "{gap:?} hides nothing — a claim the record makes"
        );
    }
}

/// **Leg 4 — the lost laptop.** A replacement key admitted under the same
/// person, with the old key removed through its last seq, leaves every past
/// act admitted — the red-team's retroactivity hole answered by record-bound
/// resolution: the binding an act resolves through is IN the record, so key
/// churn cannot re-flip the past. The companion teeth keep it off the
/// vacuous pass: the new key counts under the old person's name, and the old
/// key's post-cut write does not count. Watched red first against the fold
/// where Admit/Remove are inert (re-resolution then falls back to the file).
#[test]
fn a_replacement_key_leaves_every_past_act_admitted() {
    let old = signed(&key(4), 100, 0, record("old-key-writes"));
    let past = signed(&key(4), 101, 1, record("more-past"));
    let admit_new = signed(
        &key(1),
        102,
        0,
        RailAct::Admit {
            person: Person::parse("dee").unwrap(),
            key: actor_of(&key(5)),
            invite: None,
        },
    );
    let cut_old = signed(
        &key(1),
        103,
        1,
        RailAct::Remove {
            key: actor_of(&key(4)),
            through: vec![old.id.clone(), past.id.clone()],
        },
    );
    let new_writes = signed(&key(5), 104, 0, record("new-key-writes"));
    let old_after_cut = signed(&key(4), 105, 2, record("after-the-cut"));
    let after_cut_id = old_after_cut.id.clone();

    // The seed names dee with the OLD key — the replacement changes the key,
    // not the person.
    let mut members = std::collections::BTreeMap::new();
    members.insert(Person::parse("alex").unwrap(), vec![actor_of(&key(1))]);
    members.insert(Person::parse("dee").unwrap(), vec![actor_of(&key(4))]);
    let seed = Roster::new(members);

    let f = admit(
        &crate::ActSet::of(&[
            old.clone(),
            past,
            admit_new,
            cut_old,
            new_writes.clone(),
            old_after_cut,
        ]),
        &[],
        &seed,
        NS,
        &Ed25519Verifier,
    );
    assert!(
        f.ops().iter().any(|o| o.id == old.id),
        "every past act of the old key stays admitted"
    );
    let kept = f.ops().iter().find(|o| o.id == old.id).unwrap();
    assert_eq!(
        kept.person,
        Person::parse("dee").unwrap(),
        "under the person it always had"
    );
    assert!(
        f.ops()
            .iter()
            .any(|o| o.id == new_writes.id && o.person == Person::parse("dee").unwrap()),
        "the replacement key counts under the same person"
    );
    assert!(
        !f.ops().iter().any(|o| o.id == after_cut_id),
        "and the old key's post-cut write does not count: {:?}",
        f.gaps()
    );
}

/// **One renderer, and it must survive a name.** Gap sentences shorten ids
/// and actors for humans; a byte slice panics on a char boundary and takes
/// the log route down with it (ROOT_CAUSE_FIXES B1). The char-safe renderers
/// in `lib.rs` are the only spelling (ARCH §10.6) — multi-byte input is the
/// bar because ASCII passes either way.
#[test]
fn a_gap_renders_multi_byte_names_without_panicking() {
    let wide_actor = "日本語のキー";
    let wide_id = stray_id(0xaa);
    let gaps = [
        RailGap::BadSignature {
            id: wide_id.clone(),
            actor: wide_actor.to_string(),
        },
        RailGap::UnknownSigner {
            id: wide_id.clone(),
            actor: wide_actor.to_string(),
        },
        RailGap::NotAMember {
            id: wide_id.clone(),
            actor: wide_actor.to_string(),
        },
        RailGap::WrongPrevious {
            id: wide_id.clone(),
            actor: wide_actor.to_string(),
            seq: 3,
        },
        RailGap::SequenceHole {
            actor: wide_actor.to_string(),
            from: 4,
            to: 4,
        },
        RailGap::SequenceFork {
            actor: wide_actor.to_string(),
            seq: 5,
            ids: vec![wide_id.clone()],
        },
        RailGap::TamperedId {
            claimed: wide_id.clone(),
            derived: wide_id.clone(),
        },
        RailGap::DanglingCorrection {
            by: wide_id.clone(),
            missing: wide_id,
        },
    ];
    for gap in &gaps {
        assert!(!gap.to_string().is_empty(), "{gap:?} renders");
    }
}

/// **An un-upgraded node must say its answer covers a subset.** A
/// newer-format line it cannot read becomes a gap; admitting the rest and
/// reporting it bare is the §18.3 failure.
#[test]
fn a_line_from_a_newer_build_is_a_gap_not_an_invisible_omission() {
    let ops = vec![signed(&key(1), 100, 0, record("readable"))];
    let skipped = vec![
        SkippedLine::NewerVersion { line: 4, v: 2 },
        SkippedLine::Malformed {
            line: 7,
            error: "expected value".into(),
        },
    ];
    let f = admit(
        &crate::ActSet::of(&ops),
        &skipped,
        &ring(),
        NS,
        &Ed25519Verifier,
    );
    assert_eq!(
        applied(&f),
        vec!["readable"],
        "what it CAN read is still right"
    );
    assert!(f
        .gaps()
        .contains(&RailGap::NewerVersionLine { line: 4, v: 2 }));
    assert!(f
        .gaps()
        .iter()
        .any(|g| matches!(g, RailGap::MalformedLine { line: 7, .. })));
    assert!(!f.is_complete());
}

// ── admission ────────────────────────────────────────────────

/// The op admission checks is the one nobody else could have written. Flip a
/// byte of the body after signing and the signature is what catches it — not
/// `actor`, which the writer supplies.
#[test]
fn a_tampered_body_fails_its_signature_and_is_not_admitted() {
    let mut op = signed(&key(1), 100, 0, record("small"));
    op.kind.act = record("enormous");
    let f = admitted(&[op]);
    assert!(f.ops().is_empty(), "nothing admitted");
    assert!(
        f.gaps()
            .iter()
            .any(|g| matches!(g, RailGap::BadSignature { .. })),
        "{:?}",
        f.gaps()
    );
    // Rewriting the body also breaks the content-derived id, so both facts
    // are reported. Neither is enough on its own: the id mismatch says the
    // line was edited, the signature says it was not edited by its author.
    assert!(
        f.gaps()
            .iter()
            .any(|g| matches!(g, RailGap::TamperedId { .. })),
        "{:?}",
        f.gaps()
    );
}

/// Signing under your own fresh key proves only that you hold it. The roster
/// is what makes a signature mean membership.
#[test]
fn a_perfectly_signed_op_from_a_stranger_is_refused() {
    let op = signed(&key(42), 100, 0, record("x"));
    let f = admitted(&[op]);
    assert!(f.ops().is_empty());
    assert!(matches!(f.gaps(), [RailGap::UnknownSigner { .. }]));
}

/// A signed op lifted out of one namespace and replayed into another must
/// fail. The namespace is in the id the signature covers (svrngs, U2), so
/// here the act derives another id — named beside the refusal — and the
/// signature over the id its author derived does not verify over it.
#[test]
fn an_op_signed_for_another_namespace_cannot_be_replayed_here() {
    let op = signed_in("tool-lending", &key(1), 100, 0, record("x"));
    let f = admitted(&[op]);
    assert!(f.ops().is_empty());
    assert!(
        matches!(
            f.gaps(),
            [RailGap::BadSignature { .. }, RailGap::TamperedId { .. }]
        ),
        "{:?}",
        f.gaps()
    );
}

/// The id on the line is advisory; admission uses the one the content
/// derives. A rewritten id therefore cannot make an op impersonate another
/// op's correction target — and it is still reported, because a peer doing it
/// is worth seeing.
#[test]
fn a_rewritten_id_changes_no_outcome_and_is_still_reported() {
    let mut op = signed(&key(1), 100, 0, record("x"));
    let real = op.id.clone();
    op.id = stray_id(0xff);
    let f = admitted(&[op]);
    assert_eq!(applied(&f), vec!["x"], "the op still counts");
    assert_eq!(f.ops()[0].id, real, "under the id its content derives");
    assert!(f.gaps().contains(&RailGap::TamperedId {
        claimed: stray_id(0xff),
        derived: real,
    }));
}

/// **Each act names its author's act at seq − 1** (svrngs, U2; DESIGN §6).
/// An act whose `prev` is not the id of the act this copy holds there —
/// another branch's, none above seq 0, any at seq 0 — is a gap and is not
/// admitted. A parent not held is not judged: its absence is the hole.
#[test]
fn an_act_whose_previous_act_is_not_its_authors_at_seq_minus_one_is_a_gap() {
    let alex = actor_of(&key(1));
    let a = signed(&key(1), 100, 0, record("a"));
    let b = after(&key(1), &a, 101, record("b"));
    let honest = admitted(&[a.clone(), b]);
    assert!(honest.gaps().is_empty(), "{:?}", honest.gaps());
    assert_eq!(applied(&honest), vec!["a", "b"]);

    let wrong = |op: &Op<SignedOp>, seq| RailGap::WrongPrevious {
        id: op.id.clone(),
        actor: op.actor.clone(),
        seq,
    };
    // Another branch's child: it names an act at seq 0 this copy does not
    // hold, and this copy holds `a` there.
    let elsewhere = signed(&key(1), 100, 0, record("a-elsewhere"));
    let follows_elsewhere = after(&key(1), &elsewhere, 101, record("b"));
    // None above seq 0.
    let names_none = sign_act(&key(1), NS, 102, 1, None, record("c"), None, None, None);
    // One at seq 0.
    let names_one_at_zero = sign_act(
        &key(2),
        NS,
        100,
        0,
        Some(a.id.clone()),
        record("x"),
        None,
        None,
        None,
    );
    for (child, gap) in [
        (&follows_elsewhere, wrong(&follows_elsewhere, 1)),
        (&names_none, wrong(&names_none, 1)),
        (&names_one_at_zero, wrong(&names_one_at_zero, 0)),
    ] {
        let f = admitted(&[a.clone(), child.clone()]);
        assert!(f.gaps().contains(&gap), "{:?}", f.gaps());
        assert_eq!(applied(&f), vec!["a"], "the act is not admitted");
        assert!(!f.is_complete());
    }

    // A parent not held is not judged — the hole is the report.
    let f = admitted(std::slice::from_ref(&follows_elsewhere));
    assert_eq!(
        f.gaps(),
        vec![RailGap::SequenceHole {
            actor: alex,
            from: 0,
            to: 0,
        }]
    );
    assert_eq!(applied(&f), vec!["b"]);
}

/// **An act far past its signer's chain costs what is held, not the seq it
/// claims** (#187). A parent not held is not judged (above), so a bound
/// signer's act at a seq a million past its seq 0 is admitted, and the
/// missing seqs between are one hole: one gap, from 1 to the seq below it.
/// And so at `u64::MAX`, where a gap a missing seq never returned. Red at
/// 71be570f (#187): 999,999 gaps for the first, in 0.17 s.
#[test]
fn an_act_far_past_its_chain_is_one_hole() {
    let first = signed(&key(1), 100, 0, record("first"));
    for seq in [1_000_000, u64::MAX] {
        let far = signed(&key(1), 101, seq, record("far"));
        let f = admitted(&[first.clone(), far]);
        let hole = RailGap::SequenceHole {
            actor: actor_of(&key(1)),
            from: 1,
            to: seq - 1,
        };
        assert_eq!(f.gaps(), [hole]);
        assert_eq!(applied(&f), vec!["first", "far"]);
    }
}

/// **The one thing that distinguishes "nothing happened" from "it never
/// reached me."** Alex wrote three ops and we hold two.
#[test]
fn a_hole_in_an_actors_sequence_is_reported() {
    let ops = vec![
        signed(&key(1), 100, 0, record("a")),
        signed(&key(1), 102, 2, record("c")),
    ];
    let f = admitted(&ops);
    assert_eq!(
        f.gaps(),
        vec![RailGap::SequenceHole {
            actor: actor_of(&key(1)),
            from: 1,
            to: 1,
        }]
    );
    assert_eq!(
        applied(&f),
        vec!["a", "c"],
        "what we hold is still admitted"
    );
}

/// One actor, one sequence number, two different ops: either equivocation or
/// a counter lost across a restart. Choosing one would be inventing an
/// answer, and since U7 neither is excluded: forks void nothing (DESIGN §6),
/// so both count and the fork is named beside a complete answer. The name
/// is the rule before U7, kept so u7-removal's check follows every test by
/// name.
#[test]
fn one_sequence_number_used_twice_excludes_both_ops() {
    let a = signed(&key(1), 100, 0, record("a"));
    let b = signed(&key(1), 101, 0, record("b"));
    let f = admitted(&[a.clone(), b.clone()]);
    assert_eq!(applied(&f), vec!["a", "b"]);
    assert!(f.is_complete());
    let mut ids = vec![a.id, b.id];
    ids.sort();
    assert_eq!(
        f.gaps(),
        vec![RailGap::SequenceFork {
            actor: actor_of(&key(1)),
            seq: 0,
            ids
        }]
    );
}

/// Gossip delivers the same op many times. That is the normal case, not an
/// anomaly, and it must not double-count.
#[test]
fn the_same_op_delivered_three_times_is_admitted_once() {
    let op = signed(&key(1), 100, 0, record("x"));
    let f = admitted(&[op.clone(), op.clone(), op]);
    assert_eq!(applied(&f), vec!["x"]);
    assert_eq!(f.held(), 1, "a line delivered three times is held once");
    assert!(f.is_complete(), "{:?}", f.gaps());
}

#[test]
fn admission_is_a_function_of_the_set_of_lines() {
    let a = signed(&key(1), 100, 0, record("a"));
    let correction = RailAct::Correct {
        corrects: a.id.clone(),
        replacement: Some(payload("a2")),
    };
    let lines = vec![
        a.clone(),
        after(&key(1), &a, 101, correction),
        signed(&key(2), 100, 0, record("c")),
        signed(&key(2), 102, 0, record("d")),
        forged(&key(3), &key(1), 103, 2, None, record("e")),
        Op {
            id: stray_id(9),
            ..a.clone()
        },
    ];
    let once = admitted(&lines);
    let mut order: Vec<usize> = (0..lines.len()).collect();
    let mut seen = 0;
    permute(&mut order, 0, &mut |order| {
        let shuffled: Vec<_> = order.iter().map(|&i| lines[i].clone()).collect();
        assert_eq!(admitted(&shuffled), once, "order {order:?}");
        seen += 1;
    });
    assert_eq!(seen, 720);
    for (i, line) in lines.iter().enumerate() {
        let twice = [lines.clone(), vec![line.clone()]].concat();
        assert_eq!(admitted(&twice), once, "line {i} delivered twice");
    }
}

pub(crate) fn permute(v: &mut Vec<usize>, k: usize, f: &mut impl FnMut(&[usize])) {
    if k == v.len() {
        f(v);
        return;
    }
    for i in k..v.len() {
        v.swap(k, i);
        permute(v, k + 1, f);
        v.swap(k, i);
    }
}

// ── what the rail refuses to know ────────────────────────────

/// The rail carries an act it cannot read. An app payload that is nonsense to
/// its own app is not the rail's business — the whole point of the cut is
/// that this file has no opinion about it.
#[test]
fn an_act_the_rail_cannot_interpret_is_still_admitted() {
    let weird = Payload::new(serde_json::json!({
        "kind": "something-this-build-has-never-heard-of",
        "nested": { "z": 1, "a": [1, 2, 3] },
    }))
    .unwrap();
    let op = signed(
        &key(1),
        100,
        0,
        RailAct::Record {
            payload: weird.clone(),
        },
    );
    let f = admitted(&[op]);
    assert!(f.is_complete(), "{:?}", f.gaps());
    assert_eq!(f.ops()[0].payload.as_ref(), Some(&weird));
    assert_eq!(f.ops()[0].person, p("alex"), "and it says who wrote it");
}

/// **The canonical-payload property, end to end.** Two spellings of one act
/// sign the same bytes and derive the same id, so a client library that
/// serialises its keys in a different order does not fork the ring.
#[test]
fn payload_key_order_does_not_change_the_id_or_the_signature() {
    let one: Payload = serde_json::from_str(r#"{"b":1,"a":2}"#).unwrap();
    let other: Payload = serde_json::from_str(r#"{"a":2,"b":1}"#).unwrap();
    let a = signed(&key(1), 100, 0, RailAct::Record { payload: one });
    let b = signed(&key(1), 100, 0, RailAct::Record { payload: other });
    assert_eq!(a.id, b.id);
    assert_eq!(a.kind.sig, b.kind.sig);
    assert!(admitted(&[a]).is_complete());
}

/// **A refusal reaching a housemate must be a sentence.** The append door
/// returned a raw serde dump at a person before the gap renderer existed, and
/// making the payload opaque puts a second serializer between the app and the
/// refusal — so the property gets a test rather than a promise.
#[test]
fn a_refused_act_comes_back_as_a_sentence_not_a_serde_dump() {
    let float = serde_json::json!({ "op": "record", "payload": { "amount": 3.5 } });
    let Err(RailError::Rejected(why)) = RailAct::from_json(float) else {
        panic!("a fractional payload was accepted");
    };
    assert!(why.contains("whole number"), "{why}");
    assert!(!why.contains('{'), "rendered as a dump: {why}");
    assert!(!why.contains("line 0"), "carries a phantom position: {why}");

    let scalar = serde_json::json!({ "op": "record", "payload": 42 });
    let Err(RailError::Rejected(why)) = RailAct::from_json(scalar) else {
        panic!("a bare scalar payload was accepted");
    };
    assert!(why.contains("JSON object"), "{why}");

    // And a well-formed act reads back through the same door.
    let good = serde_json::json!({ "op": "record", "payload": { "kind": "thing" } });
    assert!(RailAct::from_json(good).is_ok());
}

/// Every gap the rail can emit renders as a sentence. An unhandled variant
/// reaching a housemate as a serde dump is what this renderer exists to stop —
/// and the append door returned exactly that until it was written.
#[test]
fn every_gap_renders_as_a_sentence() {
    let id = stray_id(0x01);
    let actor = actor_of(&key(1));
    let all = vec![
        RailGap::MalformedLine {
            line: 3,
            error: "x".into(),
        },
        RailGap::NewerVersionLine { line: 4, v: 2 },
        RailGap::BadSignature {
            id: id.clone(),
            actor: actor.clone(),
        },
        RailGap::UnknownSigner {
            id: id.clone(),
            actor: actor.clone(),
        },
        RailGap::TamperedId {
            claimed: id.clone(),
            derived: id.clone(),
        },
        RailGap::SequenceHole {
            actor: actor.clone(),
            from: 2,
            to: 2,
        },
        RailGap::WrongPrevious {
            id: id.clone(),
            actor: actor.clone(),
            seq: 2,
        },
        RailGap::SequenceFork {
            actor,
            seq: 1,
            ids: vec![id.clone()],
        },
        RailGap::DanglingCorrection {
            by: id.clone(),
            missing: id,
        },
    ];
    for gap in &all {
        let sentence = gap.to_string();
        assert!(
            !sentence.contains('{'),
            "{gap:?} rendered as a dump: {sentence}"
        );
        assert!(
            sentence.len() > 15,
            "{gap:?} rendered as a fragment: {sentence}"
        );
        // Round-trips, because the CLI reads these back off the wire and
        // renders them through this same impl.
        let json = serde_json::to_value(gap).unwrap();
        assert_eq!(&serde_json::from_value::<RailGap>(json).unwrap(), gap);
    }
}

/// **A person's name has one rule** (svrngs, u2-9; REVIEW_FINDINGS #23):
/// trimmed, 1..=64 bytes, no control characters. A typed name is trimmed; a
/// kept one, deserialised, must already be trimmed, since trimming it would
/// change bytes its signature covers.
#[test]
fn a_person_is_made_only_from_a_name() {
    assert_eq!(Person::parse("  Zoë Ann ").unwrap().as_str(), "Zoë Ann");
    for (raw, why) in [
        ("", NotAName::Empty),
        ("   ", NotAName::Empty),
        (&*"x".repeat(65), NotAName::TooLong),
        ("\x1b[2J", NotAName::Control),
        ("bo\nby", NotAName::Control),
    ] {
        assert_eq!(Person::parse(raw), Err(why), "{raw:?}");
    }
    assert!(Person::parse(&"é".repeat(32)).is_ok(), "64 bytes is a name");
    let zoe: Person = serde_json::from_str("\"Zoë Ann\"").unwrap();
    assert_eq!(serde_json::to_string(&zoe).unwrap(), "\"Zoë Ann\"");
    for raw in ["\"\"", "\" bo\"", "\"\\u001b[2J\""] {
        assert!(serde_json::from_str::<Person>(raw).is_err(), "{raw}");
    }
}

// ── the roster's bytes ───────────────────────────────────────

/// **A roster reads as it always did, with or without upstream's `vouches`.**
/// svrngs never wrote that field (it was `skip_serializing_if` empty) and it
/// went with `introduce.rs` in core-0; a file that carried it must still
/// parse to the same members, and a roster serializes to exactly the bytes a
/// deployed `roster.json` holds. Watched failing first with
/// `#[serde(deny_unknown_fields)]` on [`Roster`]: the second parse refused.
/// (Moved from upstream's `tests_introduce.rs`, its warrant half deleted with
/// the warrant.)
#[test]
fn a_roster_written_with_or_without_vouches_reads_as_its_members() {
    let before = r#"{
  "members": {
    "alex": [
      "aaaa",
      "bbbb"
    ],
    "bo": [
      "cccc"
    ]
  }
}"#;
    let with_vouches = r#"{
  "members": { "alex": ["aaaa", "bbbb"], "bo": ["cccc"] },
  "vouches": { "bbbb": { "op": "ring-0123456789abcdef", "by": "aaaa", "at": 200 } }
}"#;
    let roster: Roster = serde_json::from_str(before).expect("a deployed roster.json must parse");
    assert_eq!(roster.members.len(), 2);
    assert_eq!(roster.members[&p("alex")], vec!["aaaa", "bbbb"]);
    let vouched: Roster =
        serde_json::from_str(with_vouches).expect("a roster with upstream's vouches must parse");
    assert_eq!(vouched, roster, "and to the same members");
    assert_eq!(serde_json::to_string_pretty(&roster).unwrap(), before);
    assert_eq!(serde_json::to_string_pretty(&vouched).unwrap(), before);
}

/// The one rendering every surface that shows an op id to a person uses.
/// (Upstream's `tests_introduce.rs` held it with `short_stamp`'s, which went
/// with the warrant, core-0.)
#[test]
fn an_op_id_reads_short_enough_to_say() {
    assert_eq!(short_id("ring-0123456789abcdef"), "ring-0123456…");
    assert_eq!(short_id("short"), "short");
}

/// **Who wrote an act is told by key, beside the name** (capability-keys;
/// svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md` §7.4): alex lets two
/// people in as dee, each by hand, and each writes a record; bo writes one
/// from his phone, a second key of his seed row. Each act on the wire names
/// its writer's first key, `person_key`: the two dees' differ though both
/// say dee, and bo's phone's is his computer's, so an app keeps two
/// people's entries apart and one person's devices together. Red at HEAD:
/// an act named only the name.
#[test]
fn each_act_names_its_writer_s_first_key_beside_their_name() {
    let dee = |k: u8| RailAct::Admit {
        person: p("dee"),
        key: actor_of(&key(k)),
        invite: None,
    };
    let mut ops = run(&key(1), 100, [dee(6), dee(8)]);
    ops.extend([6, 8, 4].map(|k| signed(&key(k), 110, 0, record(&format!("by {k}")))));
    let mut seed = ring().members;
    seed.entry(p("bo")).or_default().push(actor_of(&key(4)));
    let a = admit(
        &ActSet::of(&ops),
        &[],
        &Roster::new(seed),
        NS,
        &Ed25519Verifier,
    );
    let wire = serde_json::to_value(a.wire()).unwrap();
    let whose = |what: &str| {
        let of = wire.as_array().unwrap().iter();
        let op = of.clone().find(|o| o["payload"]["what"] == what);
        op.map(|o| (o["person"].clone(), o["person_key"].clone()))
    };
    let said = |name: &str, k: u8| {
        Some((
            serde_json::json!(name),
            serde_json::json!(actor_of(&key(k))),
        ))
    };
    assert_eq!(
        ["by 6", "by 8", "by 4"].map(whose),
        [said("dee", 6), said("dee", 8), said("bo", 2)],
        "(each dee by her own key, bo's phone by his first)"
    );
}
