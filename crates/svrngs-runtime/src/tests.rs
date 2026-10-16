// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail/src/tests.rs@e1a43bcd2
//! The journal on disk, and the two-node drill.
//!
//! The half of the rail's suite that needs a filesystem. Its sibling —
//! convergence correctness over an op SET, which needs no disk — is
//! `commonwealth-rail-core`'s `tests.rs`, and the two were one file until the
//! crates split (2026-09-04).
//!
//! The fixtures come from `svrngs_core::tests_support` rather than
//! being restated here: the sync tests and these must be talking about the
//! same signed op or neither proves anything (ARCH §10.6).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use crate::*;

use svrngs_core::tests_support::*;

fn open(dir: &std::path::Path) -> RingJournal {
    RingJournal::open(dir, NS).unwrap()
}

/// One peer op, and the honest total a peer lacks: the tests' spellings of
/// the doors the round uses, `ingest_all` of one and
/// `ops_missing_from_within` with no budget.
pub(crate) trait Doors {
    fn ingest(&self, op: &Op<SignedOp>) -> Result<bool, RailError>;
    fn ops_missing_from(&self, theirs: &Digest) -> Result<Vec<Op<SignedOp>>, RailError>;
}

impl Doors for RingJournal {
    fn ingest(&self, op: &Op<SignedOp>) -> Result<bool, RailError> {
        Ok(self.ingest_all(std::slice::from_ref(op))? == 1)
    }
    fn ops_missing_from(&self, theirs: &Digest) -> Result<Vec<Op<SignedOp>>, RailError> {
        Ok(self.ops_missing_from_within(theirs, NO_BUDGET)?.0)
    }
}

#[test]
fn a_namespace_cannot_be_a_path() {
    let dir = tempfile::tempdir().unwrap();
    for bad in [
        "../../etc",
        "a/b",
        "",
        "Has-Caps",
        "with space",
        // A colon is refused DELIBERATELY and the charset is not widening to
        // admit one. It is legal on POSIX and APFS and it is not on NTFS,
        // where it separates a file from an alternate data stream — and the
        // desktop ships on Windows (`scripts/build-desktop-windows.sh`)
        // linking `sovereign-mesh` and through it this crate. cw-lift 4 makes
        // every replicating `MeshStore` app_id a namespace here, and the one
        // app_id in the workspace that carried a colon
        // (`wikipedia-newsworthy:tracked`) was RENAMED rather than this rule
        // relaxed for every namespace that comes after it.
        "has:colon",
        &"x".repeat(65),
    ] {
        assert!(
            matches!(RingJournal::open(dir.path(), bad), Err(RailError::BadNamespace(_))),
            "{bad:?} was accepted"
        );
    }
    assert!(RingJournal::open(dir.path(), "house-expenses_2").is_ok());
}

#[test]
fn appending_assigns_contiguous_sequence_numbers_per_actor() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    for i in 0..3 {
        let op = journal
            .append(record("x"), &key(1), &r, None, &Ed25519Verifier)
            .unwrap();
        assert_eq!(op.kind.seq, i);
    }
    // A second actor writing to the same journal keeps its OWN counter.
    let op = journal
        .append(record("y"), &key(2), &r, None, &Ed25519Verifier)
        .unwrap();
    assert_eq!(op.kind.seq, 0);

    let f = journal.admit(&r, &Ed25519Verifier).unwrap();
    assert!(f.is_complete(), "{:?}", f.gaps());
    assert_eq!(wire_ops(&f).len(), 4);
}

/// A journal line is one flat JSON object — the envelope's fields and the
/// act's, side by side, the same shape the other three oplog tenants write.
#[test]
fn a_journal_line_is_one_flat_object_that_reads_back() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    let written = journal
        .append(record("milk"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();

    let raw = std::fs::read_to_string(journal.dir().join("ring_oplog.jsonl")).unwrap();
    let v: serde_json::Value = serde_json::from_str(raw.trim()).unwrap();
    for k in ["id", "v", "ts_unix", "actor", "seq", "sig", "op", "payload"] {
        assert!(v.get(k).is_some(), "line is missing {k}: {raw}");
    }
    assert_eq!(v["op"], "record");
    assert_eq!(v["payload"]["what"], "milk");

    let (back, skipped) = journal.read().unwrap();
    assert!(skipped.is_empty());
    assert_eq!(back, vec![written]);
}

/// **The one thing the door still refuses, and why.** Authoring under a key
/// the ring's own roster does not carry produces ops that every node — this
/// one included — reports as `UnknownSigner` forever. Refusing here turns a
/// permanent silent gap into one sentence naming the command that fixes it.
#[test]
fn the_door_refuses_to_author_under_a_key_the_ring_does_not_know() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let stranger = journal.append(record("x"), &key(42), &ring(), None, &Ed25519Verifier);
    let Err(e @ RailError::NotInRoster { .. }) = stranger else {
        panic!("the door authored an op nobody in the ring can read");
    };
    // TYPED, so the renderer that knows where this namespace's roster comes
    // from can say something else (`routes_rail::not_in_roster_refusal`).
    // (svrngs: the Display names the fix the register has, an `Admit`; the
    // hand-written ring's `svrn ring roster add` did not come over.)
    let why = e.to_string();
    assert!(why.contains("admit"), "the refusal must name the fix: {why}");
    assert_eq!(journal.read().unwrap().0.len(), 0, "nothing was written");

    // A member writes fine.
    assert!(journal
        .append(record("x"), &key(1), &ring(), None, &Ed25519Verifier)
        .is_ok());
}

/// The rail no longer judges what an act MEANS, and that is the trade the
/// opaque payload buys. An app's own refusals belong to the app, which owns
/// one validator its door and its reducer both call — the same shape this
/// module used to have, one layer up.
#[test]
fn the_door_has_no_opinion_about_what_an_act_says() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let nonsense = RailAct::Record {
        payload: Payload::new(serde_json::json!({
            "kind": "expense",
            "amount_cents": -1,
            "participants": [],
        }))
        .unwrap(),
    };
    assert!(journal
        .append(nonsense, &key(1), &ring(), None, &Ed25519Verifier)
        .is_ok());
}

/// A payload with no canonical form never becomes a journal line, and if one
/// arrives from a peer it reads back as a malformed line rather than as a
/// mysterious bad signature.
#[test]
fn a_payload_with_no_canonical_form_is_a_malformed_line() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    journal
        .append(record("whole"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();
    let path = journal.dir().join("ring_oplog.jsonl");
    let mut raw = std::fs::read_to_string(&path).unwrap();
    raw.push_str(
        r#"{"id":"ring-abc","v":1,"ts_unix":1,"actor":"aa","seq":0,"sig":"x","op":"record","payload":{"amount":3.5}}"#,
    );
    raw.push('\n');
    std::fs::write(&path, raw).unwrap();

    let f = journal.admit(&r, &Ed25519Verifier).unwrap();
    assert_eq!(applied(&f), vec!["whole"]);
    assert!(
        f.gaps()
            .iter()
            .any(|g| matches!(g, RailGap::MalformedLine { line: 2, .. })),
        "{:?}",
        f.gaps()
    );
}

/// A line `k` signs at seq 0 whose `Admit` gives `joining` the name
/// `person` as written — as a hand or another build could write it, since
/// no `Person` this build makes can hold a name that does not parse. Its id
/// and signature are over its bytes, so only the name is wrong with it.
fn admit_line(k: &SigningKey, joining: &SigningKey, person: &str) -> String {
    let held = signed(
        k,
        1,
        0,
        RailAct::Admit {
            person: p("PLACEHOLDER"),
            key: actor_of(joining),
            invite: None,
        },
    );
    let named = serde_json::to_string(person).unwrap();
    let body = body_json(&held.kind.act, None, None).replace("\"PLACEHOLDER\"", &named);
    let id = act_id(NS, &held.actor, 0, None, 1, &body);
    serde_json::to_string(&held)
        .unwrap()
        .replace("\"PLACEHOLDER\"", &named)
        .replace(&held.id.to_string(), &id.to_string())
        .replace(&held.kind.sig, &sign_ring_op(k, &id))
}

/// `line` as the whole of `journal`'s file.
fn write_lines(journal: &RingJournal, line: &str) {
    std::fs::create_dir_all(journal.dir()).unwrap();
    std::fs::write(journal.dir().join("ring_oplog.jsonl"), format!("{line}\n")).unwrap();
}

/// **A person's name parses, or the line is a gap** (svrngs, u2-9;
/// REVIEW_FINDINGS #23). An `Admit` naming an empty person, 10 KB or a
/// terminal's escape codes, signed by a member, is a `MalformedLine` on
/// every copy that holds it, saying why, and gives its key no standing; a
/// name with a space and an accent is admitted. The line never travels:
/// a copy's read skips it, so sync never sends it on.
#[test]
fn an_admit_naming_no_person_is_a_gap_on_every_copy() {
    let joining = key(5);
    let r = ring();
    for (bad, why) in [
        ("", "cannot be empty"),
        (&*"x".repeat(10 * 1024), "at most 64 bytes"),
        ("\x1b[2J", "control characters"),
        (" bo", "start or end with a space"),
    ] {
        for copy in ["sam", "alex"] {
            let dir = tempfile::tempdir().unwrap();
            let journal = open(dir.path());
            write_lines(&journal, &admit_line(&key(1), &joining, bad));
            let f = journal.admit(&r, &Ed25519Verifier).unwrap();
            assert!(
                f.gaps().iter().any(|g| matches!(g,
                    RailGap::MalformedLine { line: 1, error } if error.contains(why))),
                "{copy}'s copy, {bad:?}: {:?}",
                f.gaps()
            );
            assert!(
                !f.membership().standing.contains(&actor_of(&joining)),
                "{copy}'s copy, {bad:?}: the key stands"
            );
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    write_lines(&journal, &admit_line(&key(1), &joining, "Zoë Ann"));
    let f = journal.admit(&r, &Ed25519Verifier).unwrap();
    assert!(f.gaps().is_empty(), "{:?}", f.gaps());
    let m = f.membership();
    assert!(m.standing.contains(&actor_of(&joining)));
    assert_eq!(m.person_of(&actor_of(&joining)).map(|(_, n)| n), Some(&p("Zoë Ann")));
}

/// **Held means the act, not the id a line claims** (svrngs, U2; DESIGN §6).
/// A line whose `id` field names an act the journal holds, with other
/// content, is another act and lands; and a line that claims the id of an
/// act not yet here does not keep that act out when it arrives. Upstream's
/// held set was keyed by the line's field, so whoever sent the claim first
/// decided which of two acts a copy kept. Both doors: one op, and a batch.
#[test]
fn a_line_claiming_a_held_id_is_not_held_and_the_act_it_named_still_lands() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let held = signed(&key(1), 100, 0, record("held"));
    assert!(journal.ingest(&held).unwrap());

    let mut claims_held = signed(&key(2), 101, 0, record("other"));
    claims_held.id = held.id.clone();
    assert!(
        journal.ingest(&claims_held).unwrap(),
        "another act, though its line names one this journal holds"
    );

    let named = signed(&key(3), 102, 0, record("named"));
    let mut claims_named = after(&key(1), &held, 103, record("decoy"));
    claims_named.id = named.id.clone();
    assert_eq!(journal.ingest_all(&[claims_named]).unwrap(), 1);
    assert_eq!(
        journal.ingest_all(&[named]).unwrap(),
        1,
        "the act a line claimed lands when it arrives"
    );

    let f = journal.admit(&ring(), &Ed25519Verifier).unwrap();
    let mut whats = applied(&f);
    whats.sort();
    assert_eq!(whats, vec!["decoy", "held", "named", "other"]);
    assert!(f.is_complete(), "{:?}", f.gaps());
    assert_eq!(
        f.gaps()
            .iter()
            .filter(|g| matches!(g, RailGap::TamperedId { .. }))
            .count(),
        2,
        "each claim is named: {:?}",
        f.gaps()
    );
}

/// A peer's op is written exactly as signed — not re-signed, not re-numbered —
/// and arriving twice is not an error.
#[test]
fn ingesting_a_peers_op_preserves_it_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let peer = signed(&key(2), 100, 0, record("x"));
    assert!(journal.ingest(&peer).unwrap());
    assert!(!journal.ingest(&peer).unwrap(), "second delivery is a no-op");
    let (back, _) = journal.read().unwrap();
    assert_eq!(back, vec![peer]);
    assert_eq!(wire_ops(&journal.admit(&ring(), &Ed25519Verifier).unwrap()).len(), 1);
}

#[test]
fn a_missing_roster_is_an_empty_ring_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    assert_eq!(journal.roster_file().unwrap(), Roster::default());
    journal.set_roster(&ring()).unwrap();
    assert_eq!(journal.roster_file().unwrap(), ring());
}

// ── the two-node drill ───────────────────────────────────────

/// **Partition, write on both sides, heal — and agree.**
///
/// The whole design's reason for existing, exercised against two real
/// journals on disk rather than against `admit` in isolation.
#[test]
fn two_partitioned_nodes_converge_on_an_identical_admission() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (open(dir_a.path()), open(dir_b.path()));
    let r = ring();

    // Partitioned: neither node can see the other's write.
    a.append(record("groceries"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();
    b.append(record("beer"), &key(2), &r, None, &Ed25519Verifier).unwrap();
    assert_ne!(
        a.admit(&r, &Ed25519Verifier).unwrap(),
        b.admit(&r, &Ed25519Verifier).unwrap(),
        "the fixture must actually be partitioned"
    );

    // Heal, both directions, one exchange each way.
    let for_a = b.ops_missing_from(&a.digest().unwrap()).unwrap();
    let for_b = a.ops_missing_from(&b.digest().unwrap()).unwrap();
    assert_eq!(a.ingest_all(&for_a).unwrap(), 1);
    assert_eq!(b.ingest_all(&for_b).unwrap(), 1);

    let (fa, fb) = (
        a.admit(&r, &Ed25519Verifier).unwrap(),
        b.admit(&r, &Ed25519Verifier).unwrap(),
    );
    assert_eq!(fa, fb, "two nodes, one answer");
    assert!(fa.is_complete(), "{:?}", fa.gaps());
    // Both acts survive, and the ORDER is content-derived rather than the
    // order either node happened to write in: these two land in the same
    // second, so the tie is broken by actor key and neither node's local
    // history wins. Sorted here for that reason — asserting the literal
    // sequence would be asserting a property of two fixture keypairs.
    let mut acts = applied(&fa);
    acts.sort();
    assert_eq!(acts, vec!["beer", "groceries"]);

    // And the exchange is idempotent: running it again moves nothing.
    let again = b.ops_missing_from(&a.digest().unwrap()).unwrap();
    assert!(again.is_empty());
    assert_eq!(a.ingest_all(&for_a).unwrap(), 0);
    assert_eq!(a.admit(&r, &Ed25519Verifier).unwrap(), fa);
}

/// **A fork must meet.** One actor signs two different acts at one seq — the
/// equivocation — and each node holds only its own branch. Marks-only
/// digests agree at the same mark and exchange NOTHING, so each node keeps a
/// private history forever while both claim completeness. The content
/// commitment is what makes the branches meet; admit then names the fork
/// and, since U7, counts both branches (forks void nothing, DESIGN §6).
#[test]
fn two_forked_nodes_exchange_until_both_hold_the_fork() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (open(dir_a.path()), open(dir_b.path()));
    let r = ring();

    let prefix = run(&key(1), 100, [record("shared-0"), record("shared-1")]);
    let left = after(&key(1), &prefix[1], 102, record("left"));
    let right = after(&key(1), &prefix[1], 102, record("right"));
    let mut a_holds = prefix.clone();
    a_holds.push(left);
    let mut b_holds = prefix.clone();
    b_holds.push(right);
    a.ingest_all(&a_holds).unwrap();
    b.ingest_all(&b_holds).unwrap();

    for round in 0..5 {
        let for_a = b.ops_missing_from(&a.digest().unwrap()).unwrap();
        let for_b = a.ops_missing_from(&b.digest().unwrap()).unwrap();
        if for_a.is_empty() && for_b.is_empty() {
            break;
        }
        assert!(round < 4, "the exchange is not converging");
        a.ingest_all(&for_a).unwrap();
        b.ingest_all(&for_b).unwrap();
    }
    assert_eq!(a.read().unwrap().0.len(), 4, "A holds both branches");
    assert_eq!(b.read().unwrap().0.len(), 4, "B holds both branches");

    let (fa, fb) = (
        a.admit(&r, &Ed25519Verifier).unwrap(),
        b.admit(&r, &Ed25519Verifier).unwrap(),
    );
    assert!(
        fa.gaps().iter().any(|g| matches!(g, RailGap::SequenceFork { .. })),
        "the fork is named, not silently merged: {:?}",
        fa.gaps()
    );
    assert_eq!(fa, fb, "two nodes, one answer");
    assert!(fa.is_complete(), "both branches fold: {:?}", fa.gaps());
    assert_eq!(applied(&fa), vec!["shared-0", "shared-1", "left", "right"]);
}

/// **The termination pin for the window rule.** A fork at the LAST seq of a
/// window too large for one budgeted chunk must still converge: a truncated
/// window resend would ship the same duplicate prefix forever with the
/// difference past the cut, so the window travels as one atomic unit (sent
/// whole when it alone exceeds the budget, as a single oversized op is).
#[test]
fn a_forked_window_with_a_tiny_budget_still_converges() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (open(dir_a.path()), open(dir_b.path()));
    let r = ring();

    let mut a_holds = run(&key(1), 100, (0..20).map(|_| record("shared")));
    let mut b_holds = a_holds.clone();
    a_holds.push(after(&key(1), &b_holds[19], 120, record("left")));
    b_holds.push(after(&key(1), &a_holds[19], 120, record("right")));
    a.ingest_all(&a_holds).unwrap();
    b.ingest_all(&b_holds).unwrap();

    // Room for roughly two ops: a split window would never reach seq 20.
    let budget = 400;
    for round in 0..40 {
        let (for_a, more_a) = b.ops_missing_from_within(&a.digest().unwrap(), budget).unwrap();
        let (for_b, more_b) = a.ops_missing_from_within(&b.digest().unwrap(), budget).unwrap();
        if for_a.is_empty() && for_b.is_empty() && !more_a && !more_b {
            break;
        }
        assert!(round < 39, "the exchange is not converging");
        a.ingest_all(&for_a).unwrap();
        b.ingest_all(&for_b).unwrap();
    }
    let (fa, fb) = (
        a.admit(&r, &Ed25519Verifier).unwrap(),
        b.admit(&r, &Ed25519Verifier).unwrap(),
    );
    assert_eq!(a.read().unwrap().0.len(), 22, "A holds both branches");
    assert_eq!(b.read().unwrap().0.len(), 22, "B holds both branches");
    assert!(
        fa.gaps().iter().any(|g| matches!(g, RailGap::SequenceFork { .. })),
        "the fork is named: {:?}",
        fa.gaps()
    );
    assert_eq!(fa, fb);
}

/// **The mutual-removal resolver is deterministic across nodes.** The
/// amendment's "two members removing each other… the rail's order picks one"
/// — with both cuts arriving at BOTH nodes in OPPOSITE orders. The rail's
/// order is content-derived: both nodes fold the same survivor. Since U7
/// only the keeper cuts (alex, the seed's first person), so bo is out and
/// bo's counter-cut, outside what alex's cut covers, is `Removed`.
/// The arrival-order defect's WATCH is the battery in
/// `membership.rs::every_interleaving_of_a_membership_set_folds_identically`
/// — measured, not assumed: this end-to-end form stays green under that
/// plant because `admit` hands the fold an OpId-ordered set, canonicalizing
/// arrival before the walk. This test pins the property across a wire and
/// the survivor by name; the battery pins the fold.
#[test]
fn two_nodes_that_receive_the_cuts_in_opposite_orders_agree() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (open(dir_a.path()), open(dir_b.path()));
    let r = ring();

    let alex_cut = signed(
        &key(1),
        100,
        0,
        RailAct::Remove {
            key: actor_of(&key(2)),
            through: Vec::new(),
        },
    );
    let bo_cut = signed(
        &key(2),
        101,
        0,
        RailAct::Remove {
            key: actor_of(&key(1)),
            through: Vec::new(),
        },
    );
    // Same SET, opposite ARRIVAL.
    a.ingest_all(&[alex_cut.clone(), bo_cut.clone()]).unwrap();
    b.ingest_all(&[bo_cut.clone(), alex_cut.clone()]).unwrap();

    let (fa, fb) = (
        a.admit(&r, &Ed25519Verifier).unwrap(),
        b.admit(&r, &Ed25519Verifier).unwrap(),
    );
    assert_eq!(fa, fb, "the rail's order picks one — the same one on both nodes");
    let removed = fa
        .gaps()
        .iter()
        .filter(|g| matches!(g, RailGap::Removed { .. }))
        .count();
    assert_eq!(removed, 1, "exactly bo's counter-cut is unstanding: {:?}", fa.gaps());
    assert!(
        wire_ops(&fa).iter().any(|o| o.id == alex_cut.id),
        "alex's cut counted — it sorted first, so bo was already out when theirs arrived"
    );
}

/// **A3's cost, measured once** (ROOT_CAUSE_FIXES, cross-cutting bar): every
/// append re-checks the journal's signatures to build its view. This records
/// the shape — time per append against journal size, and the act's own size —
/// and asserts only a generous ceiling: the NUMBERS are the bar, and a tight
/// ceiling would be a flake. The committed figures live in the commit body.
#[test]
fn the_views_cost_is_measured_and_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();

    let mut sizes = Vec::new();
    for &n in &[200usize, 1000, 5000] {
        // Seed without paying the view cost (ingest does not mint views).
        let start = journal.read().unwrap().0.len();
        let ops: Vec<_> = (start..n)
            .map(|seq| signed(&key(1), 1_700_000_000 + seq as i64, seq as u64, record("seed")))
            .collect();
        journal.ingest_all(&ops).unwrap();

        let t = std::time::Instant::now();
        let written = journal
            .append(record("measured"), &key(2), &r, None, &Ed25519Verifier)
            .unwrap();
        let per_append = t.elapsed();
        let act_bytes = serde_json::to_vec(&written).unwrap().len();
        sizes.push((n, per_append, act_bytes));
        eprintln!(
            "A3 cost: journal {n} ops — one append (with its view) took \
             {per_append:?}; the act is {act_bytes} bytes"
        );
        assert!(
            per_append < std::time::Duration::from_secs(5),
            "append at {n} ops took {per_append:?} — the view re-check is unbounded"
        );
        assert!(act_bytes < 16 * 1024, "the act itself is {act_bytes} bytes");
    }
    // The shape is the point: cost grows with the journal (one verify per
    // held op per append). Recorded, not asserted — see the commit body.
    assert_eq!(sizes.len(), 3);
}

/// **A peer that dies mid-sync leaves a hole, and the hole is named.**
///
/// Half of B's ops reach A. A must not report a clean answer over what it
/// got: the acts are real and they are a subset, and only the gap says so.
#[test]
fn a_half_delivered_peer_is_a_named_hole_not_a_clean_answer() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (open(dir_a.path()), open(dir_b.path()));
    let r = ring();
    b.append(record("first"), &key(2), &r, None, &Ed25519Verifier).unwrap();
    b.append(record("second"), &key(2), &r, None, &Ed25519Verifier).unwrap();

    // Only the SECOND op lands — the connection dropped after one of them.
    let all = b.read().unwrap().0;
    let second = all.iter().find(|o| o.kind.seq == 1).unwrap().clone();
    a.ingest_all(&[second]).unwrap();

    let fa = a.admit(&r, &Ed25519Verifier).unwrap();
    assert!(!fa.is_complete(), "A must not claim a complete answer");
    assert_eq!(
        fa.gaps(),
        vec![RailGap::SequenceHole {
            actor: actor_of(&key(2)),
            missing: 0
        }]
    );

    // And the digest asks for the hole rather than claiming the high mark.
    assert!(
        a.digest().unwrap().get(&actor_of(&key(2))).is_none(),
        "A holds nothing contiguous from B, so it must claim nothing"
    );
    let repair = b.ops_missing_from(&a.digest().unwrap()).unwrap();
    assert_eq!(repair.len(), 2, "the hole and everything above it come back");
    a.ingest_all(&repair).unwrap();
    let healed = a.admit(&r, &Ed25519Verifier).unwrap();
    assert!(healed.is_complete(), "{:?}", healed.gaps());
    assert_eq!(healed, b.admit(&r, &Ed25519Verifier).unwrap());
}

/// A torn last line — the daemon died between the write and the sync — is a
/// reported gap, not an invisible subtraction.
#[test]
fn a_torn_last_line_is_reported_rather_than_quietly_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    journal
        .append(record("whole"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();
    let path = journal.dir().join("ring_oplog.jsonl");
    let mut raw = std::fs::read_to_string(&path).unwrap();
    raw.push_str("{\"id\":\"ring-abc\",\"v\":1,\"ts_un");
    std::fs::write(&path, raw).unwrap();

    let f = journal.admit(&r, &Ed25519Verifier).unwrap();
    assert_eq!(applied(&f), vec!["whole"], "the whole line still counts");
    assert!(
        f.gaps()
            .iter()
            .any(|g| matches!(g, RailGap::MalformedLine { line: 2, .. })),
        "{:?}",
        f.gaps()
    );
}

// ── a journal pruned by hand ─────────────────────────────────

/// The lines a naive truncation would keep: everything at or above `from`.
fn truncate_by_hand(journal: &RingJournal, from: u64) {
    let path = journal.dir().join("ring_oplog.jsonl");
    let kept: String = std::fs::read_to_string(&path)
        .unwrap()
        .lines()
        .filter(|l| {
            serde_json::from_str::<Op<SignedOp>>(l)
                .map(|o| o.kind.seq >= from)
                .unwrap_or(true)
        })
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&path, kept).unwrap();
}

/// **A journal missing a prefix says so.** Deleting a prefix on a local rule
/// — a truncation setting, a size cap, an age policy — reports one
/// `SequenceHole` per deleted op. (svrngs: upstream's seal and compaction
/// deleted one without a hole; they went in arch-seal, and so this is what a
/// copy an older build pruned reads as here.)
#[test]
fn deleting_a_prefix_makes_holes() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    for what in ["one", "two", "three", "after"] {
        journal
            .append(record(what), &key(1), &r, None, &Ed25519Verifier)
            .unwrap();
    }
    truncate_by_hand(&journal, 3);

    let f = journal.admit(&r, &Ed25519Verifier).unwrap();
    assert!(!f.is_complete());
    assert_eq!(
        f.gaps().len(),
        3,
        "one hole per deleted op, and they never close: {:?}",
        f.gaps()
    );
}

/// Cy's signature over a line that claims to be alex's act at `seq`,
/// following `prev`: refused by admission, held on disk.
fn forged_as_alex(seq: u64, prev: &Op<SignedOp>) -> Op<SignedOp> {
    let ts = 100 + seq as i64;
    forged(&key(3), &key(1), ts, seq, Some(prev.id.clone()), record("forged"))
}

/// **The door follows its own act, never a line that claims its key**
/// (svrngs, U2). A peer can push a line naming this node's key at its newest
/// seq; were the door to name it as the next act's previous act, every copy
/// holding the real one would refuse that act as `WrongPrevious`.
#[test]
fn the_door_names_its_own_act_and_not_a_forgery_beside_it() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    let one = journal
        .append(record("one"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();
    let two = journal
        .append(record("two"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();
    journal.ingest(&forged_as_alex(1, &one)).unwrap();
    let three = journal
        .append(record("three"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();
    assert_eq!(three.kind.seq, 2);
    assert_eq!(three.kind.prev.as_ref(), Some(&two.id), "not the forgery");
    let f = journal.admit(&r, &Ed25519Verifier).unwrap();
    assert!(matches!(f.gaps(), [RailGap::BadSignature { .. }]), "{:?}", f.gaps());
    assert_eq!(applied(&f), vec!["one", "two", "three"]);
}

/// **A forged line above a member's newest act never moves their next seq**
/// (DESIGN §6, "U7 also closes two holes"; ralph/REVIEW_FINDINGS.md #16).
/// Cy pushes a line claiming alex's key at seq 5, above alex's seq 0. Alex's
/// next act is seq 1, following alex's own, and the one gap is the forgery.
/// At u7-removal's base `Held::catch_up` folded every line's seq, verified
/// or not, so the door signed seq 6 and left holes 1 to 5 that never fill.
#[test]
fn a_forged_line_above_a_members_newest_never_moves_their_next_seq() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    let one = journal
        .append(record("one"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();
    journal.ingest(&forged_as_alex(5, &one)).unwrap();
    let two = journal
        .append(record("two"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();
    assert_eq!(two.kind.seq, 1, "the forgery moved nothing");
    assert_eq!(two.kind.prev.as_ref(), Some(&one.id));
    let f = journal.admit(&r, &Ed25519Verifier).unwrap();
    assert!(matches!(f.gaps(), [RailGap::BadSignature { .. }]), "{:?}", f.gaps());
    assert_eq!(applied(&f), vec!["one", "two"]);
}

// ── an op never changes a roster ─────────────────────────────

/// **A peer's act naming a stranger's key arrives, is readable, and moves
/// nothing.**
///
/// The load-bearing negative leg of `ra-roster-stays-independent`, and the
/// refusal `sovereign-mesh/src/ring_roster.rs` states in prose: "there is no
/// roster route, so a deployed app cannot add a key to the ring — including
/// its own." It is a property of the route set rather than a check somebody
/// remembered, which is exactly why it needs a test that would notice a route
/// being added.
///
/// **Asserted on the FILE BYTES**, not on an in-memory `Roster` this test
/// built: an in-memory comparison would pass on a node whose `roster.json`
/// had been rewritten and re-read identically, and the thing being prevented
/// is a write. Watched failing first, with `ingest_all` rewriting the file.
///
/// The other half is from the bar's own goodhart line: zero is the expected
/// good result and proves nothing alone. So the same run shows the act
/// ARRIVING and being READABLE. (svrngs, core-0: upstream's act was an
/// `Introduce`, which went from the core with the roster warrant; this one
/// is a payload of its own, naming the same person and key.)
#[test]
fn a_peers_act_naming_a_key_arrives_readable_and_changes_no_roster_row() {
    let (mine, theirs) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (open(mine.path()), open(theirs.path()));
    let r = ring();
    // My roster, on disk, as `svrn ring roster add` would have left it.
    a.set_roster(&r).unwrap();
    let before = std::fs::read(a.roster_path()).unwrap();

    // bo — a member I already claim — writes an act naming a key nobody claims.
    let stranger = actor_of(&key(9));
    let says = Payload::new(serde_json::json!({
        "kind": "thing",
        "person": "dee",
        "key": stranger,
    }))
    .unwrap();
    let act = RailAct::Record { payload: says.clone() };
    let written = b.append(act, &key(2), &r, None, &Ed25519Verifier).unwrap();

    // Ring-sync, the way a peer delivers: digest out, missing ops back in.
    let for_me = b.ops_missing_from(&a.digest().unwrap()).unwrap();
    assert_eq!(a.ingest_all(&for_me).unwrap(), 1, "the op must actually arrive");

    // ── nothing moved ──
    assert_eq!(
        std::fs::read(a.roster_path()).unwrap(),
        before,
        "an arriving op rewrote {}",
        a.roster_path().display()
    );
    let on_disk = a.roster_file().unwrap();
    assert_eq!(on_disk, r, "and the roster it parses to is the one I wrote");
    assert!(
        !on_disk.members.contains_key(&Person::parse("dee").unwrap()),
        "dee is named, not admitted"
    );
    assert!(on_disk.members.values().flatten().all(|k| *k != stranger));

    // ── and the act is there to read ──
    let admission = a.admit(&r, &Ed25519Verifier).unwrap();
    assert!(admission.is_complete(), "{:?}", admission.gaps());
    let ops = wire_ops(&admission);
    let op = ops
        .iter()
        .find(|o| o.id == written.id)
        .expect("the act is admitted — it was signed by a member");
    assert_eq!(op.person, Person::parse("bo").unwrap(), "and it says who wrote it");
    assert_eq!(op.payload.as_ref(), Some(&says), "readable, as it was written");
}
