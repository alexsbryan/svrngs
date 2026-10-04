// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/sync.rs@e1a43bcd2
use super::*;
use crate::tests_support::{key, marks, record, signed, NS};
use crate::{Ed25519Verifier, RailAct};

fn actor(seed: u8) -> String {
    crate::actor_of(&key(seed))
}

fn op(seed: u8, seq: u64, ts: i64) -> Op<SignedOp> {
    signed(&key(seed), ts, seq, record("x"))
}

/// The digest under the shipped verifier — what the bars assert on.
fn digest_of(ops: &[Op<SignedOp>]) -> Digest {
    digest(ops, NS, &Ed25519Verifier)
}

#[test]
fn a_contiguous_run_reports_its_last_seq() {
    let ops = vec![op(1, 0, 10), op(1, 1, 11), op(1, 2, 12)];
    assert_eq!(marks(&digest_of(&ops)), BTreeMap::from([(actor(1), 2)]));
}

/// **The reason the mark is contiguous.** A node holding 0 and 2 must not
/// claim 2, or the peer answers "nothing above 2" and seq 1 is lost
/// forever while both sides believe they are in sync.
#[test]
fn a_hole_lowers_the_mark_so_the_peer_re_sends_across_it() {
    let held = vec![op(1, 0, 10), op(1, 2, 12)];
    assert_eq!(marks(&digest_of(&held)), BTreeMap::from([(actor(1), 0)]));

    let peer_has = vec![op(1, 0, 10), op(1, 1, 11), op(1, 2, 12)];
    let sent = ops_missing_from(&peer_has, NS, &Ed25519Verifier, &digest_of(&held));
    assert_eq!(
        sent.iter().map(|o| o.kind.seq).collect::<Vec<_>>(),
        vec![1, 2],
        "the hole and everything above it come back"
    );
}

/// Missing seq 0 means we cannot make any contiguous claim, so we ask for
/// everything rather than quietly claiming a mark we do not have.
#[test]
fn an_actor_whose_first_op_is_missing_is_absent_from_the_digest() {
    let held = vec![op(1, 3, 13)];
    assert!(digest_of(&held).is_empty());
    let peer_has = vec![op(1, 0, 10), op(1, 1, 11), op(1, 3, 13)];
    assert_eq!(
        ops_missing_from(&peer_has, NS, &Ed25519Verifier, &digest_of(&held)).len(),
        3
    );
}

/// An actor we have never heard of is absent, and absence asks for
/// everything rather than defaulting to zero (which would ask for
/// everything ABOVE zero and silently skip their first op).
#[test]
fn an_unknown_actor_gets_their_whole_history() {
    let peer_has = vec![op(2, 0, 10), op(2, 1, 11)];
    assert_eq!(
        ops_missing_from(&peer_has, NS, &Ed25519Verifier, &Digest::new()).len(),
        2
    );
}

#[test]
fn a_peer_that_is_already_caught_up_is_sent_nothing() {
    let ops = vec![op(1, 0, 10), op(1, 1, 11)];
    assert!(ops_missing_from(&ops, NS, &Ed25519Verifier, &digest_of(&ops)).is_empty());
}

/// Replication is author-blind: a node republishes what it HOLDS, so a
/// housemate who leaves does not take their half of the journal with them.
#[test]
fn a_node_republishes_ops_it_did_not_author() {
    let departed = op(3, 0, 10);
    let held = vec![op(1, 0, 11), departed.clone()];
    let sent = ops_missing_from(&held, NS, &Ed25519Verifier, &digest_of(&[op(1, 0, 11)]));
    assert_eq!(sent, vec![departed], "someone else's op, still republished");
}

/// Two nodes with the same holdings must produce the same bytes, or a
/// captured payload diff means nothing.
// ── the sealed floor ─────────────────────────────────

fn seal(seed: u8, seq: u64, ts: i64) -> Op<SignedOp> {
    signed(&key(seed), ts, seq, RailAct::Seal)
}

/// **Truncation must not undo itself.** Before the floor existed,
/// `digest` early-returned `None` for any actor whose seq 0 it did not
/// hold — so a node that compacted a sealed prefix advertised NOTHING for
/// that actor, every peer read that as "I have none of theirs", and the
/// whole holding came back on the next sixty-second round. Compaction
/// amplified traffic and then reversed itself.
///
/// The first assertion is the negative control: the same holding with no
/// seal in it really is unadvertisable, so the second cannot pass on a
/// digest that was never empty.
#[test]
fn a_compacted_actor_is_advertised_from_its_seal_instead_of_not_at_all() {
    let no_seal = vec![op(1, 3, 103), op(1, 4, 104)];
    assert!(
        digest_of(&no_seal).is_empty(),
        "control: nothing sealed, nothing contiguous from 0, nothing to claim"
    );

    let compacted = vec![seal(1, 3, 103), op(1, 4, 104)];
    assert_eq!(
        marks(&digest_of(&compacted)),
        BTreeMap::from([(actor(1), 4)]),
        "the mark counts from the sealed floor, not from zero"
    );
}

/// **A peer must not re-send a sealed prefix, ever.** This is the same
/// defect seen from the other end: the responder holds the retired ops
/// and, reading a digest that could not name a floor, shipped all of them
/// every round.
#[test]
fn a_peer_never_re_sends_what_the_seal_retired() {
    // The prefix the seal retires, and what is left after compaction. The
    // peer is a node that never compacted, so it holds both.
    let retired: Vec<_> = (0..3).map(|s| op(1, s, 100 + s as i64)).collect();
    let compacted = vec![seal(1, 3, 103), op(1, 4, 104)];
    let peer_holds: Vec<_> = retired.iter().chain(&compacted).cloned().collect();
    assert!(
        ops_missing_from(&peer_holds, NS, &Ed25519Verifier, &digest_of(&compacted)).is_empty(),
        "the retired prefix is not wanted and must not be sent"
    );

    // And the seal still travels to a peer that has not got it yet: a
    // node cannot honour a floor it has never seen.
    assert_eq!(
        ops_missing_from(&peer_holds, NS, &Ed25519Verifier, &digest_of(&retired))
            .iter()
            .map(|o| o.kind.seq)
            .collect::<Vec<_>>(),
        vec![3, 4],
        "the seal itself is an op, and it is how the floor propagates"
    );
}

// ── authenticity: an unauthenticated line counts for nothing ──

/// A line whose signature is `signer`'s while its `actor` field claims
/// somebody else: the shape one hostile journal line takes before anyone
/// verifies it. `Op::new` is the same builder the honest fixtures use, so
/// the id is self-consistent and only the signature gives it away.
fn spoofed(signer: u8, claimed: u8, seq: u64, ts: i64, act: RailAct) -> Op<SignedOp> {
    let body = crate::body_json(&act, None, None);
    let sig = crate::sign_ring_op(&key(signer), crate::tests_support::NS, ts, seq, &body);
    Op::new(
        SignedOp {
            seq,
            sig,
            act,
            on_behalf_of: None,
            view: None,
        },
        ts,
        actor(claimed),
    )
}

/// **A forged seal must not retire a member's past.** The floor is a
/// `max` ratchet — one pushed line claiming `actor(1)` at seq 5 makes the
/// digest claim 5 and every peer stop sending seq 3..=5 forever. Before
/// this bar the digest trusted `op.actor` off the wire; the signature is
/// the only field a writer cannot forge for somebody else.
#[test]
fn a_forged_seal_does_not_raise_the_floor() {
    let held = vec![
        op(1, 0, 10),
        op(1, 1, 11),
        op(1, 2, 12),
        spoofed(9, 1, 5, 15, RailAct::Seal),
    ];
    assert_eq!(
        marks(&digest_of(&held)),
        BTreeMap::from([(actor(1), 2)]),
        "the forged seal retires nothing and the mark stops at what is real"
    );
    let peer_has: Vec<_> = (0..=5).map(|s| op(1, s, 10 + s as i64)).collect();
    assert_eq!(
        ops_missing_from(&peer_has, NS, &Ed25519Verifier, &digest_of(&held))
            .iter()
            .map(|o| o.kind.seq)
            .collect::<Vec<_>>(),
        vec![3, 4, 5],
        "the real ops the forgery tried to retire still come back"
    );
}

/// The same hole one step down the ratchet: a forged record at the next
/// contiguous seq extends the mark, and the real op at that seq is never
/// requested — one silent lost op per pushed line.
#[test]
fn a_forged_record_does_not_extend_the_mark() {
    let held = vec![
        op(1, 0, 10),
        op(1, 1, 11),
        op(1, 2, 12),
        spoofed(9, 1, 3, 13, record("evil")),
    ];
    assert_eq!(marks(&digest_of(&held)), BTreeMap::from([(actor(1), 2)]));
}

/// **A forged line inside a window must not read as a fork.** The peer's
/// head folds only authenticated ops ([`digest`]); a mismatch check that
/// folded everything held saw a disagreement no exchange could heal, and
/// resent the actor's whole window every call, forever — two nodes with
/// IDENTICAL holdings, one pushed line between them.
#[test]
fn identical_holdings_with_a_forged_line_in_the_window_send_nothing() {
    let held = vec![
        op(1, 0, 10),
        spoofed(9, 1, 1, 11, record("evil")),
        op(1, 1, 11),
        op(1, 2, 12),
    ];
    assert!(
        ops_missing_from(&held, NS, &Ed25519Verifier, &digest_of(&held)).is_empty(),
        "same holdings, nothing to send"
    );
    let forked = vec![op(1, 0, 10), op(1, 1, 11), op(1, 1, 99), op(1, 2, 12)];
    assert_eq!(
        ops_missing_from(&forked, NS, &Ed25519Verifier, &digest_of(&held)).len(),
        4,
        "the control: a REAL fork in the same window still travels as the unit"
    );
}

/// **A voided seal retires nothing.** `Correct` voids without erasure and
/// the void set is commutative — but the floor was computed before the
/// void set saw the light, so a seal the log itself calls wrong kept
/// suppressing its author's holes. Seq 1 is missing here and must come
/// back precisely because the seal that claimed to retire it is voided.
#[test]
fn a_voided_seal_retires_nothing() {
    let seal = seal(1, 2, 102);
    let held = vec![
        op(1, 0, 10),
        seal.clone(),
        signed(
            &key(1),
            103,
            3,
            RailAct::Correct {
                corrects: seal.id.clone(),
                replacement: None,
            },
        ),
    ];
    assert_eq!(
        marks(&digest_of(&held)),
        BTreeMap::from([(actor(1), 0)]),
        "no floor, and the hole at seq 1 lowers the mark"
    );
}

#[test]
fn the_payload_order_is_content_derived_not_arrival_order() {
    let a = vec![op(1, 0, 10), op(2, 0, 11), op(1, 1, 12)];
    let b = vec![op(1, 1, 12), op(1, 0, 10), op(2, 0, 11)];
    assert_eq!(
        ops_missing_from(&a, NS, &Ed25519Verifier, &Digest::new()),
        ops_missing_from(&b, NS, &Ed25519Verifier, &Digest::new())
    );
}

// ── the byte budget ──────────────────────────────────

fn wire_bytes(ops: &[Op<SignedOp>]) -> usize {
    serde_json::to_vec(ops).unwrap().len()
}

/// The unbudgeted form is the budgeted one with a budget that cannot
/// bind — one selection rule, not two (ARCH §10.6).
#[test]
fn the_unbudgeted_form_is_the_budgeted_one_with_no_budget() {
    let held: Vec<_> = (0..20).map(|s| op(1, s, 100 + s as i64)).collect();
    let (all, more) =
        ops_missing_from_within(&held, NS, &Ed25519Verifier, &Digest::new(), NO_BUDGET);
    assert_eq!(
        all,
        ops_missing_from(&held, NS, &Ed25519Verifier, &Digest::new())
    );
    assert!(!more, "a budget that cannot bind never truncates");
}

/// A budget larger than the whole selection changes nothing and says so.
#[test]
fn a_budget_nothing_reaches_sends_everything_and_reports_no_more() {
    let held: Vec<_> = (0..20).map(|s| op(1, s, 100 + s as i64)).collect();
    let whole = wire_bytes(&held);
    let (sent, more) =
        ops_missing_from_within(&held, NS, &Ed25519Verifier, &Digest::new(), whole * 2);
    assert_eq!(sent.len(), 20);
    assert!(!more);
}

/// **The truncation is visible, and it is the lowest ops that go.** A
/// chunk that quietly dropped the tail would leave the peer with a hole
/// it could never name.
#[test]
fn a_budget_that_binds_sends_the_lowest_ops_and_reports_more() {
    let held: Vec<_> = (0..20).map(|s| op(1, s, 100 + s as i64)).collect();
    let five = wire_bytes(&held[..5]);
    let (sent, more) = ops_missing_from_within(&held, NS, &Ed25519Verifier, &Digest::new(), five);
    assert!(more, "the budget cut it short and must say so");
    assert!(
        (1..20).contains(&sent.len()),
        "a budget of five ops' bytes sent {} of 20",
        sent.len()
    );
    assert_eq!(
        sent.iter().map(|o| o.kind.seq).collect::<Vec<_>>(),
        (0..sent.len() as u64).collect::<Vec<_>>(),
        "the chunk is the contiguous LOW end, so the peer's mark advances"
    );
    assert!(
        wire_bytes(&sent) <= five,
        "the chunk must fit the budget it was given"
    );
}

/// **A chunk of none would be the spin.** One op larger than the whole
/// budget still goes out alone; the receiver's refusal is the honest
/// failure, and a sender that returned nothing forever is not.
#[test]
fn an_op_bigger_than_the_whole_budget_is_still_sent_alone() {
    let held: Vec<_> = (0..3).map(|s| op(1, s, 100 + s as i64)).collect();
    let (sent, more) = ops_missing_from_within(&held, NS, &Ed25519Verifier, &Digest::new(), 1);
    assert_eq!(sent.len(), 1, "exactly one, never zero");
    assert_eq!(sent[0].kind.seq, 0);
    assert!(more);
}

/// **K9, in the small.** Repeating the chunked selection against a peer
/// that ingests what it is sent converges, and the bound on the number of
/// rounds is the count of ops rather than anything larger: every non-empty
/// chunk starts at an op the peer provably lacks, so every round moves the
/// peer's mark.
#[test]
fn repeating_a_budgeted_chunk_converges_and_every_round_moves_the_mark() {
    let held: Vec<_> = (0..40).map(|s| op(1, s, 100 + s as i64)).collect();
    let budget = wire_bytes(&held[..3]);
    let mut peer: Vec<Op<SignedOp>> = Vec::new();
    let mut rounds = 0;
    loop {
        let (chunk, more) =
            ops_missing_from_within(&held, NS, &Ed25519Verifier, &digest_of(&peer), budget);
        if chunk.is_empty() {
            assert!(!more, "nothing to send cannot also mean more remains");
            break;
        }
        let before = digest_of(&peer);
        peer.extend(chunk);
        assert_ne!(
            digest_of(&peer),
            before,
            "round {rounds} sent a chunk that moved nothing — this is the spin"
        );
        rounds += 1;
        assert!(rounds <= 40, "a 40-op journal must not need 40+ rounds");
    }
    assert_eq!(peer.len(), 40, "converged, in {rounds} rounds");
    assert_eq!(digest_of(&peer), digest_of(&held), "two nodes, one claim");
    assert!(rounds > 1, "the control: this budget really did chunk");
}
