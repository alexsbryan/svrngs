// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/tests_sealing.rs@e1a43bcd2
//! The verifier seam, a seal held, and acting on behalf of a guest.
//!
//! Split out of `tests.rs`, which the ring-rail work pushed into the
//! 800-1200 approach band (ARCH §3.1). Same crate, same fixtures.

use crate::*;

use crate::tests_support::*;

// ── the verifier seam ────────────────────────────────────────

/// A verifier whose answer is fixed, so what admission DOES with an answer can
/// be pinned without a keypair standing in the way. One stub for both answers:
/// two would be two spellings of the same nothing.
struct Says(bool);

impl RingVerifier for Says {
    fn name(&self) -> &'static str {
        "test-fixed-answer"
    }
    fn verify(&self, _: &str, _: &OpId, _: &str) -> bool {
        self.0
    }
}

/// **The seam is real or it is decoration.** Ops the shipped verifier admits
/// must become gaps under a verifier that refuses them — which is false if
/// `admit` asks [`verify_ring_op`](crate::sig) directly and takes the
/// parameter for show. The first assertion is the negative control: without it
/// the second passes on a fixture that was never admissible.
#[test]
fn admission_asks_the_verifier_it_was_handed_and_not_a_hardcoded_one() {
    let ops = [
        signed(&key(1), 100, 0, record("a")),
        signed(&key(2), 101, 0, record("b")),
    ];
    assert_eq!(
        admitted(&ops).ops().len(),
        2,
        "control: both ops are admissible under the shipped verifier"
    );

    let f = admit(&crate::ActSet::of(&ops), &[], &ring(), NS, &Says(false));
    assert!(f.ops().is_empty(), "a refused signature is never an act");
    assert!(
        f.gaps()
            .iter()
            .all(|g| matches!(g, RailGap::BadSignature { .. })),
        "{:?}",
        f.gaps()
    );
}

/// **A refusal is a refusal, never an absence (ARCH §18.3).** An op the rail
/// cannot authenticate must be counted in `held`, reported as a gap, and make
/// the answer say it covers a subset. Dropping it quietly is how an app states
/// a wrong total with complete confidence.
#[test]
fn a_signature_the_verifier_refuses_is_a_gap_and_never_a_silent_drop() {
    let f = admit(
        &crate::ActSet::of(&[signed(&key(1), 100, 0, record("a"))]),
        &[],
        &ring(),
        NS,
        &Says(false),
    );
    assert!(!f.is_complete(), "an unverifiable op makes this a subset");
    assert_eq!(
        f.held(),
        1,
        "the op is held and accounted for, not forgotten"
    );
    assert!(matches!(f.gaps(), [RailGap::BadSignature { .. }]));
}

/// **A verifier is not a roster.** The seam decides whether a key signed these
/// bytes; the roster decides whether the ring claims that key. Swapping in a
/// verifier that accepts everything must still leave a stranger out, or the
/// seam has handed membership to whoever installs a verifier.
#[test]
fn a_permissive_verifier_still_cannot_admit_a_stranger() {
    let f = admit(
        &crate::ActSet::of(&[signed(&key(42), 100, 0, record("x"))]),
        &[],
        &ring(),
        NS,
        &Says(true),
    );
    assert!(f.ops().is_empty());
    assert!(matches!(f.gaps(), [RailGap::UnknownSigner { .. }]));
}

// ── a seal, held ─────────────────────────────────────────────

/// A `Seal` whose author signed it, at `seq`, plus one act above it: the
/// holding an older build left after it pruned what the seal retired.
fn sealed_then(seq: u64, what: &str) -> Vec<Op<SignedOp>> {
    let seal = signed(&key(1), 100 + seq as i64, seq, RailAct::Seal);
    let above = after(&key(1), &seal, 101 + seq as i64, record(what));
    vec![seal, above]
}

/// **A seal held retires nothing, and claims nothing complete** (arch-seal
/// (c); svrn-docs A_RECORD_FOR_YEARS §6 q2). A copy holding alex's seal at
/// seq 3 and an act above it, and nothing below, is missing seqs 0..=2: three
/// holes, and alex absent from its digest, so a peer sends the lot. The seal
/// is held as an act that applies nothing. The failing input: the seal sets
/// alex's floor, as at the row's start, and the copy reads as whole.
#[test]
fn a_seal_held_retires_nothing_and_claims_nothing_complete() {
    let ops = sealed_then(3, "after");
    let f = admitted(&ops);
    let holes = vec![RailGap::SequenceHole {
        actor: actor_of(&key(1)),
        from: 0,
        to: 2,
    }];
    assert_eq!(f.gaps(), holes, "seqs 0..=2 are still missing");
    assert_eq!(wire_ops(&f).len(), 2, "the seal is held as an act");
    assert_eq!(applied(&f), vec!["after"]);
    assert!(
        marks(&crate::digest(&ops, NS, &Ed25519Verifier)).is_empty(),
        "nothing from alex's first act on is held, so the digest claims none of theirs"
    );
}

// ── whose words an act was ───────────────────────────────────

/// The field is LAST and skipped when absent, so an act that names nobody
/// signs the bytes it signed before the field existed. Stated here rather
/// than trusted to serde's declaration order, because every op on every
/// replica rests on it.
#[test]
fn an_act_naming_nobody_signs_the_bytes_it_always_did() {
    let act = record("milk");
    assert_eq!(
        body_json(&act, None, None),
        serde_json::to_string(&act).unwrap(),
        "an absent name must not reach the signed bytes"
    );
    assert!(
        body_json(&act, Some("dee"), None).ends_with(r#","on_behalf_of":"dee"}"#),
        "a stated name goes last: {}",
        body_json(&act, Some("dee"), None)
    );
}

// ── the view an act commits to ──────────────────────────────

fn claimed_view(head_fill: char) -> crate::Digest {
    crate::Digest::from(std::collections::BTreeMap::from([(
        String::from("c0ffee"),
        crate::View {
            from: 0,
            mark: 3,
            head: head_fill.to_string().repeat(64),
        },
    )]))
}

/// **The signed body carries the view, last.** The two with-view vectors are
/// the wire contract (ROOT_CAUSE_FIXES A3): frozen by hand from the layout
/// rule — act, then `on_behalf_of`, then `view`, each skipped when absent —
/// and watched failing against the old bytes before the field reached them.
#[test]
fn the_signed_body_golden_vectors() {
    let act = record("milk");
    let view = claimed_view('a');
    let view_json = format!(
        r#"{{"v":2,"entries":{{"c0ffee":{{"from":0,"mark":3,"head":"{}"}}}}}}"#,
        "a".repeat(64)
    );

    // The bytes every pre-view op on every replica signed.
    assert_eq!(
        body_json(&act, None, None),
        serde_json::to_string(&act).unwrap()
    );
    // The view goes last and is skipped when absent.
    assert_eq!(
        body_json(&act, None, Some(&view)),
        format!(
            r#"{{"op":"record","payload":{{"kind":"thing","what":"milk"}},"view":{view_json}}}"#
        ),
        "an act with a view signs it, last"
    );
    assert_eq!(
        body_json(&act, Some("dee"), Some(&view)),
        format!(
            r#"{{"op":"record","payload":{{"kind":"thing","what":"milk"}},"on_behalf_of":"dee","view":{view_json}}}"#
        ),
        "name before view, both after the act"
    );
}

/// **The view is inside the signature.** A stamp a peer could rewrite in
/// flight would make every act claim whichever history suited the carrier
/// (ARCH §18.1 — the `on_behalf_of` rule one field later).
#[test]
fn a_view_rewritten_after_signing_is_refused() {
    let act = record("milk");
    let view = claimed_view('a');
    let op = sign_act(&key(1), NS, 100, 0, None, act, None, Some(view));
    assert!(
        admitted(std::slice::from_ref(&op)).gaps().is_empty(),
        "the honest op verifies"
    );

    let mut rewritten = op;
    rewritten.kind.view = Some(claimed_view('b'));
    let f = admitted(std::slice::from_ref(&rewritten));
    assert!(
        f.gaps()
            .iter()
            .any(|g| matches!(g, RailGap::BadSignature { .. })),
        "a rewritten view must break the signature: {:?}",
        f.gaps()
    );
}

/// **An equivocation pair is its own evidence pack.** Two acts at one seq,
/// each line carrying the view its author claimed when writing it: each act
/// alone proves its own claimed history (it is signed over), and the pair
/// shows the divergence with nothing but the two lines. Admission's half —
/// both branches excluded — is already pinned where forks are audited.
#[test]
fn an_equivocation_carries_two_claimed_views() {
    let (view_a, view_b) = (claimed_view('a'), claimed_view('b'));
    let build = |act: RailAct, view: &crate::Digest| {
        let prev = Some(absent_parent());
        sign_act(&key(1), NS, 102, 2, prev, act, None, Some(view.clone()))
    };
    let left = build(record("left"), &view_a);
    let right = build(record("right"), &view_b);

    // The evidence rides the wire form — parse it back and the claimed views
    // are still there, still signed.
    for (op, expected) in [(&left, &view_a), (&right, &view_b)] {
        let wire = serde_json::to_string(op).unwrap();
        let back: Op<SignedOp> = serde_json::from_str(&wire).unwrap();
        assert_eq!(
            back.kind.view.as_ref(),
            Some(expected),
            "the claim survives the wire"
        );
    }
    assert_ne!(left.kind.view, right.kind.view, "two claimed histories");

    let f = admitted(&[left, right]);
    assert!(
        f.gaps()
            .iter()
            .any(|g| matches!(g, RailGap::SequenceFork { .. })),
        "both branches are excluded by name: {:?}",
        f.gaps()
    );
}

/// Leg 5's stamp half: the membership acts ship at the bumped line version
/// and the original three keep the bytes they always had — so an un-upgraded
/// node names exactly the acts it cannot read, and nothing else changes
/// version.
#[test]
fn the_membership_acts_ship_at_a_bumped_line_version() {
    let wrote = signed(&key(1), 100, 0, record("x"));
    let sealed = signed(&key(1), 101, 1, RailAct::Seal);
    let corrected = signed(
        &key(1),
        102,
        2,
        RailAct::Correct {
            corrects: wrote.id.clone(),
            replacement: None,
        },
    );
    let admitted_someone = signed(
        &key(1),
        103,
        3,
        RailAct::Admit {
            person: Person::parse("bo").unwrap(),
            key: crate::actor_of(&key(2)),
            invite: None,
        },
    );
    let cut = signed(
        &key(1),
        104,
        4,
        RailAct::Remove {
            key: crate::actor_of(&key(2)),
            through: Vec::new(),
        },
    );
    assert_eq!(
        (wrote.v, sealed.v, corrected.v),
        (1, 1, 1),
        "Record, Correct and Seal keep the version they always had"
    );
    assert_eq!(
        (admitted_someone.v, cut.v),
        (MEMBERSHIP_LINE_VERSION, crate::REMOVAL_LINE_VERSION),
        "the membership acts ship at the bump, a removal at U7's"
    );
}

/// The back-compat stop condition of the membership amendment, pinned
/// byte-for-byte: a ring with no `Admit` in its journal must admit exactly
/// as it did before membership existed. Frozen from this fixture's first
/// post-landing fold (RFC style — generated once, then pinned); every field
/// of the Admission is in the string, so any drift in the no-act path is a
/// diff here rather than an invisible behaviour change.
#[test]
fn a_ring_with_no_membership_acts_admits_exactly_as_before() {
    let ops: Vec<Op<SignedOp>> = include_str!("fixtures/rail_ops_before_on_behalf_of.jsonl")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("the fixture is a journal line"))
        .collect();
    let f = admitted(&ops);
    assert_eq!(
        serde_json::to_string(&(&f.ops(), &f.gaps())).unwrap(),
        r#"[[{"id":"6efe8a46f81d82df03b43ed537f3d50b854c24f8810a4b4aac58c61629d2c840","actor":"8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c","person":"alex","seq":0,"ts_unix":1700000000,"voided":true,"payload":{"kind":"thing","what":"milk"}},{"id":"e754694b85150ea490f4c305099f4d26142f95c8a031134e172500abc1983fc4","actor":"8139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b394","person":"bo","seq":0,"ts_unix":1700000001,"corrects":"6efe8a46f81d82df03b43ed537f3d50b854c24f8810a4b4aac58c61629d2c840","voided":false},{"id":"0ce63175592c0b3b80fb1cff79983bbd42610fede6f73e3bb2610808f171a711","actor":"ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d1","person":"cy","seq":0,"ts_unix":1700000002,"voided":false}],[]]"#,
        "a ring with no membership act folds byte-identically to the build before"
    );
}

/// Ops written before `on_behalf_of` existed, captured from this crate at
/// f51b66112 and committed verbatim. Every replica holds lines like these;
/// if adding the field moved a byte, they would all become `BadSignature`
/// the day a node upgraded, and no test that constructs its ops with the
/// NEW code could ever notice.
///
/// svrngs (U2): regenerated by [`emit_journal_fixture`] when an act's id
/// became the whole hash and the signature moved onto it — no journal
/// outside tests existed, so there was no older form to keep reading. It
/// still pins what it always did: lines that name nobody verify unchanged.
#[test]
fn rail_ops_written_before_on_behalf_of_still_verify() {
    let ops: Vec<Op<SignedOp>> = include_str!("fixtures/rail_ops_before_on_behalf_of.jsonl")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("the fixture is a journal line"))
        .collect();
    assert_eq!(ops.len(), 3, "record, correct-without-replacement, seal");
    assert!(
        ops.iter().all(|o| o.kind.on_behalf_of.is_none()),
        "the fixture predates the field"
    );

    let f = admitted(&ops);
    assert!(
        f.gaps().is_empty(),
        "an op written before the field must still verify: {:?}",
        f.gaps()
    );
    assert!(
        f.ops().iter().all(|o| o.on_behalf_of.is_none()),
        "nothing invents a name"
    );
}

/// The name is inside the signature, so rewriting it in flight — the only
/// way a peer could reattribute somebody's words — is a refusal and not a
/// silent correction. The id is re-derived from the tampered body, so the
/// ONLY thing wrong with this op is the signature.
#[test]
fn a_name_rewritten_after_signing_is_refused() {
    let honest = signed_for(NS, &key(1), 100, 0, record("milk"), Some("dee"));
    let f = admitted(&[honest.clone()]);
    assert!(f.gaps().is_empty(), "{:?}", f.gaps());
    assert_eq!(
        f.ops()[0].on_behalf_of.as_deref(),
        Some("dee"),
        "admission carries the name through"
    );

    let mut reattributed = honest.clone();
    reattributed.kind.on_behalf_of = Some("eve".to_string());
    reattributed.id = derive_id(&reattributed, NS);
    let f = admitted(&[reattributed]);
    assert!(
        matches!(f.gaps(), [RailGap::BadSignature { .. }]),
        "rewriting the name must not verify: {:?}",
        f.gaps()
    );
    assert!(f.ops().is_empty(), "nothing tampered reaches an app");
}

/// Every act kind carries it — including a correction that states no
/// replacement, which has no payload a name could have ridden in.
#[test]
fn a_correction_with_no_replacement_still_names_the_guest() {
    let first = signed_for(NS, &key(1), 100, 0, record("milk"), Some("dee"));
    let retraction = sign_act(
        &key(1),
        NS,
        101,
        1,
        Some(first.id.clone()),
        RailAct::Correct {
            corrects: first.id.clone(),
            replacement: None,
        },
        Some("dee"),
        None,
    );
    let f = admitted(&[first, retraction]);
    assert!(f.gaps().is_empty(), "{:?}", f.gaps());
    assert!(
        f.ops()
            .iter()
            .all(|o| o.on_behalf_of.as_deref() == Some("dee")),
        "a payload-less act names its guest too"
    );
    assert!(applied(&f).is_empty(), "the retraction voided the record");
}

/// The generator for `fixtures/rail_ops_before_on_behalf_of.jsonl`: alex's
/// record, bo's correction of it, cy's seal, as the lines a journal holds.
/// Kept so the regeneration is reproducible.
#[test]
#[ignore = "fixture generator — run with --ignored --nocapture"]
fn emit_journal_fixture() {
    let milk = signed(&key(1), 1_700_000_000, 0, record("milk"));
    let void = signed(
        &key(2),
        1_700_000_001,
        0,
        RailAct::Correct {
            corrects: milk.id.clone(),
            replacement: None,
        },
    );
    let seal = signed(&key(3), 1_700_000_002, 0, RailAct::Seal);
    for op in [milk, void, seal] {
        println!("{}", serde_json::to_string(&op).unwrap());
    }
}
