// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_sync/tests.rs@e1a43bcd2
//! The ring round and its peer route (pb-mesh-exit-transport; director
//! phase-b-81 (5)). Every exchange with a member is asked in this process,
//! `in_process.rs` (docs/CHECKS.md, harness 4); what follows here needs no
//! member at all.
//!
//! The successors of the daemon's ring tests (`ring_sync_loop_tests`,
//! `rail_e2e/ceiling`, `ring_sync_by_roster`, `ring_append_nudges_sync`,
//! `ring_return_syncs`), which drove this same loop when it ran in the
//! daemon: the round and the route are cw-rails' since pb-rails-parity, and
//! the daemon's copies retired with the flip. (svrngs: their responders were
//! served by the acceptor on a hermetic endpoint until 2026-11-13; the
//! route is asked by `oneshot` now, with the dialer the acceptor inserts.)

use std::sync::Arc;

use crate::peer_wire::{
    RingSyncRequest, RingSyncResponse, MAX_REQUEST_BODY_BYTES, RING_SYNC_OPS_BUDGET_BYTES,
};
use crate::{
    actor_of, body_json, sign_act, Admission, Digest, Ed25519Verifier, Op, OpId, Payload, Person,
    RailAct, RailError, RingRail, RingVerifier, Roster, SignedOp, SigningKey, NO_BUDGET,
};
use axum::response::IntoResponse;

use super::*;
use crate::ring_routes::{router, RingInbound};

pub(super) const NS: &str = "house-expenses";

/// The fixture the ceiling table is quoted against: a 594-byte serialised
/// body, ~990 B/op on the wire (svrngs, U2: ~873 upstream, before the id
/// became 64 hex and the line named its previous act).
const FIXTURE_BODY_BYTES: usize = 594;

pub(super) fn body_of_size(target: usize) -> Payload {
    let mut filler = target.saturating_sub(40);
    loop {
        let p = Payload::new(serde_json::json!({ "b": "x".repeat(filler) })).unwrap();
        let n = body_json(&RailAct::Record { payload: p.clone() }, None, None, None).len();
        if n >= target {
            return p;
        }
        filler += target - n;
    }
}

/// One op signed for its own `(namespace, ts, seq)`, so its id is distinct,
/// naming `prev` (svrngs, U2). A signature binds the namespace, so an op
/// signed for one ring is a gap on any other.
fn signed_after(
    ns: &str,
    key: &SigningKey,
    seq: u64,
    prev: Option<OpId>,
    act: RailAct,
) -> Op<SignedOp> {
    let ts = 1_700_000_000i64 + seq as i64;
    sign_act(key, ns, ts, seq, prev, act, None, None, None)
}

/// The act `key` signs after `prev`: one seq on, naming it.
fn after(key: &SigningKey, prev: &Op<SignedOp>, act: RailAct) -> Op<SignedOp> {
    signed_after(NS, key, prev.kind.seq + 1, Some(prev.id.clone()), act)
}

/// Group [`NS`] as the runtime offers it to an adapter, spelled as the journal
/// these tests were written against: every call is the runtime's public port,
/// and `read` is the journal file itself.
struct Journal(Arc<RingRail>);

impl Journal {
    fn read(&self) -> Result<(Vec<Op<SignedOp>>, ()), RailError> {
        let file = self.0.group_dir(NS)?.join("ring_oplog.jsonl");
        let lines = std::fs::read_to_string(file)
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        Ok((lines, ()))
    }
    fn digest(&self, _: &dyn RingVerifier) -> Result<Digest, RailError> {
        self.0.digest(NS)
    }
    fn admit(&self, _: &Roster, _: &dyn RingVerifier) -> Result<Admission, RailError> {
        self.0.admission(NS)
    }
    fn ingest(&self, op: &Op<SignedOp>) -> Result<bool, RailError> {
        Ok(self.0.ingest_all(NS, std::slice::from_ref(op))? == 1)
    }
    fn ingest_all(&self, ops: &[Op<SignedOp>]) -> Result<usize, RailError> {
        self.0.ingest_all(NS, ops)
    }
    fn ops_missing_from_within(
        &self,
        _: &dyn RingVerifier,
        theirs: &Digest,
        budget: usize,
    ) -> Result<(Vec<Op<SignedOp>>, bool), RailError> {
        self.0.ops_missing_from_within(NS, theirs, budget)
    }
    fn ops_missing_from(
        &self,
        _: &dyn RingVerifier,
        theirs: &Digest,
    ) -> Result<Vec<Op<SignedOp>>, RailError> {
        Ok(self.0.ops_missing_from_within(NS, theirs, NO_BUDGET)?.0)
    }
}

/// The act `key` signs next after the newest it has in `journal`.
fn next_in(journal: &Journal, key: &SigningKey, act: RailAct) -> Op<SignedOp> {
    let held = journal.read().unwrap().0;
    let newest = held
        .iter()
        .filter(|op| op.actor == actor_of(key))
        .max_by_key(|op| op.kind.seq)
        .expect("the journal holds an act by this key");
    after(key, newest, act)
}

pub(super) fn ops_in(ns: &str, key: &SigningKey, n: usize) -> Vec<Op<SignedOp>> {
    let payload = body_of_size(FIXTURE_BODY_BYTES);
    let mut ops: Vec<Op<SignedOp>> = Vec::with_capacity(n);
    for seq in 0..n as u64 {
        let act = RailAct::Record {
            payload: payload.clone(),
        };
        let prev = ops.last().map(|op| op.id.clone());
        ops.push(signed_after(ns, key, seq, prev, act));
    }
    ops
}

pub(super) fn solo_roster(key: &SigningKey) -> Roster {
    let mut m = std::collections::BTreeMap::new();
    m.insert(Person::parse("alex").unwrap(), vec![actor_of(key)]);
    Roster::new(m)
}

/// A rail signing as `key` holding `n` signed ops on [`NS`], rostered to
/// `key`. `n = 0` is a node that has never seen this ring's ops — the
/// bootstrap case, which can only ever be TOLD.
fn node(dir: &std::path::Path, key: &SigningKey, n: usize) -> (Arc<RingRail>, Journal) {
    let rail = Arc::new(RingRail::new(dir, Arc::new(key.clone())));
    rail.set_seed(NS, &solo_roster(key)).unwrap();
    let journal = Journal(rail.clone());
    if n > 0 {
        assert_eq!(journal.ingest_all(&ops_in(NS, key, n)).unwrap(), n);
    }
    (rail, journal)
}

/// cw-rails' ring routes over `rail`, as the node serves them on the stream.
pub(super) fn ring_router(rail: Arc<RingRail>) -> axum::Router {
    router(RingInbound {
        rail,
        live: Default::default(),
        wake: Default::default(),
    })
}

/// The budget binds: a 10,000-op journal does not fit one chunk, so the
/// convergence test above is evidence about the LOOP. Also pins the
/// per-op cost the ceiling table assumes, and that a chunk fits the limit it
/// was budgeted against.
#[tokio::test]
async fn ten_thousand_ops_do_not_fit_one_chunk() {
    let key = SigningKey::from_bytes(&[1u8; 32]);
    let dir = tempfile::tempdir().unwrap();
    let (_rail, journal) = node(dir.path(), &key, 10_000);
    let (chunk, more) = journal
        .ops_missing_from_within(
            &crate::Ed25519Verifier,
            &Digest::default(),
            RING_SYNC_OPS_BUDGET_BYTES,
        )
        .unwrap();
    assert!(more, "the budget must cut a 10,000-op journal short");
    assert!(
        chunk.len() < 10_000,
        "one chunk carried all {}",
        chunk.len()
    );
    assert!(
        serde_json::to_vec(&chunk).unwrap().len() <= MAX_REQUEST_BODY_BYTES,
        "a chunk must fit the limit it was budgeted against"
    );
    let all = journal
        .ops_missing_from(&crate::Ed25519Verifier, &Digest::default())
        .unwrap();
    let per_op = serde_json::to_vec(&all).unwrap().len() / all.len();
    assert!(
        (975..=1005).contains(&per_op),
        "the {FIXTURE_BODY_BYTES}-byte fixture costs {per_op} B/op, not the ~990 the table assumes"
    );
}

/// **The answer from the kept digest is the whole log's, byte for byte**
/// (ceiling-2). This node answers, through `answer` as the route does, a
/// peer two acts ahead and then a peer in step, and each response, with its
/// `more`, is set beside the one built from the journal file read whole —
/// what `answer` sent when the digest and the offer were folded from the
/// whole log (svrngs-core `digest`, `ops_missing_from_within`). Two of the
/// four are answered unread, the peer's digest ours.
#[test]
fn the_answer_from_the_kept_digest_is_the_whole_logs_byte_for_byte() {
    let key = SigningKey::from_bytes(&[1u8; 32]);
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let (us, journal) = node(dirs[0].path(), &key, 40);
    let (ahead, _) = node(dirs[1].path(), &key, 40);
    let (in_step, _) = node(dirs[2].path(), &key, 40);
    let two = ops_in(NS, &key, 42).split_off(40);
    assert_eq!(ahead.ingest_all(NS, &two).unwrap(), 2);
    us.digest(NS).unwrap(); // caught up, as the last round left it

    let mut answers = Vec::new();
    let mut ask = |peer: &RingRail, ops: Vec<Op<SignedOp>>| {
        let req = RingSyncRequest {
            namespace: NS.to_string(),
            digest: peer.digest(NS).unwrap(),
            ops,
        };
        let (kept, more) = answer(&us, &req).unwrap();
        let held = journal.read().unwrap().0;
        let (sent, whole_more) = svrngs_core::ops_missing_from_within(
            &held,
            NS,
            &Ed25519Verifier,
            &req.digest,
            RING_SYNC_OPS_BUDGET_BYTES,
        );
        let whole = RingSyncResponse {
            namespace: NS.to_string(),
            digest: svrngs_core::digest(&held, NS, &Ed25519Verifier),
            ops: sent,
            ingested: kept.ingested,
        };
        let bytes = |r: &RingSyncResponse| serde_json::to_string(r).unwrap();
        assert_eq!((bytes(&kept), more), (bytes(&whole), whole_more));
        answers.push((kept.digest == req.digest, kept.ops.len(), kept.ingested));
        kept
    };
    ask(&ahead, Vec::new());
    ask(&ahead, two.clone());
    let behind = ask(&in_step, Vec::new());
    in_step.ingest_all(NS, &behind.ops).unwrap();
    ask(&in_step, Vec::new());
    assert_eq!(
        answers,
        [(false, 40, 0), (true, 0, 2), (false, 2, 0), (true, 0, 0)],
        "(digests equal, ops sent, ops ingested): ahead, whose window [0, 41] \
         folds differently here and so travels whole (svrngs-core sync.rs); \
         ahead with its two; behind once we hold them; and in step"
    );
}

mod in_process;
