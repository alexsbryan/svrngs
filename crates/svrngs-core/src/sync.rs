// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/sync.rs@e1a43bcd2
//! Replication — what a node tells a peer it has, and what the peer sends back.
//!
//! # Why this is not the mesh store's snapshot push
//!
//! The existing app-state replication ships a **full snapshot to every online
//! peer every ten seconds** (`gossip.rs` Step 4, 8,640 rounds/day). A household
//! writes on the order of 3,500 ops a year — call it 1.5 MB — so riding that
//! path would cost roughly **246 GB/day of egress per node**, and would tax
//! every other namespace on the same push forever. Bandwidth is the binding
//! constraint here and it binds on day one, not at scale.
//!
//! So the journal gets its own, slower push (sixty seconds is ample for money)
//! and syncs by **digest**, not by snapshot: about 600 bytes regardless of how
//! long the journal is.
//!
//! # The digest is a CONTIGUOUS high-water mark, and that word is load-bearing
//!
//! Per actor, the highest `n` such that this node needs nothing at or below
//! `n` — **not** the highest seq it holds. A maximum cannot express a hole: a
//! node holding seq 0 and 2 would advertise `2`, the peer would answer
//! "nothing above 2", and seq 1 would never arrive. It would sit in the
//! admission report as a permanent [`SequenceHole`](crate::RailGap::SequenceHole)
//! while both nodes believed they were in sync.
//!
//! An actor missing from the digest means "I hold nothing of theirs", so the
//! peer sends everything. Absence is a request, never a default.
//!
//! # The run starts at zero: a seal sets no floor
//!
//! Upstream counted the run from the seq of the actor's own highest
//! [`Seal`](crate::RailAct::Seal), so a copy could delete what the seal
//! retired and still make a claim. svrngs removed that (arch-seal; svrn-docs
//! `rings/current/A_RECORD_FOR_YEARS.md` §2, §6 q2): a seal retires nothing
//! here, so every run counts from zero, and a copy missing an actor's first
//! acts is absent from its digest for that actor and is sent them.
//!
//! An older build still claims from its floor, in [`View::from`], and
//! [`ops_missing_from_within`] keeps to the window a peer claims: an older
//! copy that pruned below its floor is not sent what it pruned every round,
//! to prune again.
//!
//! # Everyone republishes everything they hold
//!
//! [`ops_missing_from_within`] does not filter by author. A node that has been
//! offline gets its ops back from whoever holds them, and a housemate who
//! leaves the ring does not take their half of the journal with them. This is
//! also why there is no own-origin skip to get wrong: the mesh store's
//! `origin` field names the last republisher rather than the author, which is
//! a trap for anyone syncing through it — this path has no origin field at
//! all, because the op carries its author in a signature.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::op::{Op, OpId};

use crate::{Digest, RingVerifier, SignedOp, View};

/// The ops whose signature verifies under their claimed actor — the ONE
/// reading of "what this node honestly holds" (ARCH 8). [`digest`] folds a
/// head over it, and [`ops_missing_from_within`] must fold the peer's window
/// over the same set: a check that folded everything held would compare a
/// forged line against a head that never counted it, see a mismatch no
/// exchange can heal, and resend the actor's whole window every call.
///
/// Each beside the id its content derives in `namespace` (svrngs, U2): the
/// signature is over that id, so verifying derives it, and every reader of
/// the set — the view's heads — takes it from here rather than from the
/// line.
fn authentic<'a>(
    ops: &'a [Op<SignedOp>],
    namespace: &str,
    verifier: &dyn RingVerifier,
) -> Vec<(OpId, &'a Op<SignedOp>)> {
    ops.iter()
        .map(|op| (crate::derive_id(op, namespace), op))
        .filter(|(id, op)| verifier.verify(&op.actor, id, &op.kind.sig))
        .collect()
}

/// What this node can honestly claim to need nothing below, per actor.
///
/// An op that does not verify under its claimed actor counts for nothing —
/// not toward a floor, not toward a mark. Until verification the `actor`
/// field is only what the line claimed, and a digest that believed it would
/// advertise history its holder does not have: a forged line at the next
/// contiguous seq silently loses the real op there.
///
/// See the module docs: this is the **contiguous** mark, so a hole below the
/// maximum lowers it and the peer re-sends from there. Healing a hole is
/// therefore automatic rather than a separate repair path.
pub fn digest(ops: &[Op<SignedOp>], namespace: &str, verifier: &dyn RingVerifier) -> Digest {
    let mut fold = DigestFold::default();
    for (id, op) in authentic(ops, namespace, verifier) {
        fold.add(id, op);
    }
    fold.digest()
}

/// [`digest`]'s fold, kept: each authentic act's derived id by actor and
/// seq, folded in as acts arrive, so a journal that keeps one beside its
/// cursor answers its digest for the acts appended since (svrngs-runtime
/// journal.rs `Held`). An actor's view is folded again only when one of its
/// acts arrives. [`digest`] is this over a whole journal, so the two cannot
/// part.
#[derive(Debug, Default)]
pub struct DigestFold {
    runs: BTreeMap<String, BTreeMap<u64, BTreeSet<OpId>>>,
    stale: BTreeSet<String>,
    views: BTreeMap<String, View>,
}

impl DigestFold {
    /// Fold in one act already verified under its claimed actor, beside the
    /// id its content derives ([`authentic`]'s pairs).
    pub fn add(&mut self, id: OpId, op: &Op<SignedOp>) {
        let run = self.runs.entry(op.actor.clone()).or_default();
        run.entry(op.kind.seq).or_default().insert(id);
        self.stale.insert(op.actor.clone());
    }

    /// The digest of everything added: each stale actor's view folded again.
    pub fn digest(&mut self) -> Digest {
        for actor in core::mem::take(&mut self.stale) {
            let seqs = &self.runs[&actor];
            // Absent from the digest is the honest answer when the actor's
            // first act is missing: "I have some of their ops but not from
            // where their history starts" is "send me theirs" for the
            // purpose of asking.
            let Some(mark) = (0..).take_while(|n| seqs.contains_key(n)).last() else {
                self.views.remove(&actor);
                continue;
            };
            let ids = seqs
                .range(0..=mark)
                .flat_map(|(n, ids)| ids.iter().map(move |id| (*n, id.to_string())));
            let view = View::fold_pairs(&actor, 0, mark, &ids.collect::<Vec<_>>());
            self.views.insert(actor, view);
        }
        self.views.clone().into()
    }

    /// Whether [`ops_missing_from_within`] would send `theirs` nothing,
    /// judged without the journal: `newest` is each actor's highest seq
    /// among every line held, verified or not.
    ///
    /// Equal digests are not the whole test (svrn-docs
    /// `rings/current/THE_CEILING.md`, fix 1): with `theirs` ours no window
    /// mismatches, since both heads fold the same authentic set over the
    /// same run, so what is still sent is every line of an actor absent from
    /// the digest and every line above its actor's mark — a line past a
    /// hole, a forged one. Those are the slow path's to answer, so this says
    /// nothing is missing only when no line sits there.
    pub fn missing_nothing<'a>(
        &mut self,
        theirs: &Digest,
        newest: impl Iterator<Item = (&'a str, u64)>,
    ) -> bool {
        let ours = self.digest();
        ours == *theirs && { newest }
            .all(|(actor, seq)| ours.get(actor).is_some_and(|v| seq <= v.mark))
    }
}

/// The budget that never binds: with it, [`ops_missing_from_within`] is the
/// honest TOTAL, what a caller wants when it is measuring, not sending.
///
/// Named rather than spelled `usize::MAX` at the call site because it is also
/// the value the pricing fast path tests for: costing every op is a second
/// whole serialisation of the journal, and a caller that cannot be cut short
/// must not pay for it.
pub const NO_BUDGET: usize = usize::MAX;

/// Every op this node holds that `theirs` says the peer is missing, stopped
/// at `budget_bytes` of serialised ops. Ordered by `(actor, seq)` so the wire
/// payload is deterministic — two nodes with the same holdings produce
/// byte-identical bodies, which makes a diff of two captures mean something.
///
/// Returns `(ops, more)`. **`more` is the whole reason this is not a
/// `Vec`**: "here is everything you lack" and "here is as much of it as fits"
/// are different facts, and a truncation the caller cannot see is a silent
/// substitution (ARCH §18.3). The receiver's body limit answers an oversized
/// exchange at the extractor, so nothing downstream of a quiet truncation
/// would ever have said otherwise.
///
/// # Why repeating this terminates (the sync loop's whole safety argument)
///
/// Per call — one exchange carries both directions — every non-empty
/// selection delivers at least one op at least one side did not hold, or the
/// call is empty and the sides fold equal.
///
/// **`seq > mark` keeps the original property**: a mark of `n` means the run
/// is contiguous THROUGH `n`, so `n + 1` is not held; the selection orders by
/// `(actor, seq)`, so the first such op is exactly one the peer lacks.
///
/// **A mismatching window travels as one atomic per-actor unit**, and that is
/// the rest of the argument. Both folds run over AUTHENTICATED ops — the
/// peer's head by [`digest`], ours here by the same [`authentic`] filter —
/// because holdings the two folds count differently can never agree: a forged
/// line in the window once read as a permanent fork and resent the whole unit
/// every call. A fold mismatch at the same `(from, mark)` then means the
/// authenticated sets differ in the window, so the two directions' units cannot
/// both be subsets of the other side's holdings — the all-duplicates
/// direction is paid for by the reverse direction's progress in the same
/// call. Atomicity is what makes that unconditional under a budget: a split
/// window could ship the same duplicate prefix forever with the difference
/// past the cut, so a unit that does not fit is deferred whole (`more`)
/// unless it is alone in the chunk — then it goes whole, exactly as a single
/// oversized op does below, and the receiver's body limit refuses it by name
/// rather than the sender quietly stopping. Holdings only grow (ingest only
/// adds) and are finite, so the caller's loop
/// (`sovereign-mesh/src/ring_sync.rs::exchange`) stops.
///
/// # The budget is in WIRE bytes, and one op is never split
///
/// Cost is `serde_json` length plus the array's separating comma — the same
/// serialisation the receiver's limit compares against, because any other
/// number would be a second implementation of "how big is this" (ARCH §10.6).
/// A window unit's cost is the sum over its ops.
///
/// An op that alone exceeds the budget is still sent. A chunk of none would
/// be the spin this exists to prevent, and the honest failure for an op too
/// large for any exchange is the receiver's refusal — not a sender that
/// quietly stops.
pub fn ops_missing_from_within(
    ops: &[Op<SignedOp>],
    namespace: &str,
    verifier: &dyn RingVerifier,
    theirs: &Digest,
    budget_bytes: usize,
) -> (Vec<Op<SignedOp>>, bool) {
    let cost_of = |op: &Op<SignedOp>, budget_bytes: usize| match serde_json::to_vec(op) {
        // `+ 1` for the comma the enclosing array puts between elements.
        Ok(bytes) => bytes.len() + 1,
        // Unreachable for an op that came off a wire or out of a journal, but
        // a price that cannot be computed must not read as free: charge the
        // whole budget so such an op is only ever sent alone, and let the
        // receiver — not a silent drop — judge it.
        Err(_) => budget_bytes,
    };
    // (svrngs, lean-4: the two walks below were written twice upstream.)
    // Each op onto `out` while it fits the budget — one always fits an empty
    // `out` — and `false` at the first that does not.
    let fill = |ops: Vec<&Op<SignedOp>>, out: &mut Vec<Op<SignedOp>>, spent: &mut usize| {
        for op in ops {
            let cost = cost_of(op, budget_bytes);
            if budget_bytes != NO_BUDGET
                && !out.is_empty()
                && spent.saturating_add(cost) > budget_bytes
            {
                return false;
            }
            *spent = spent.saturating_add(cost);
            out.push(op.clone());
        }
        true
    };

    let trusted = authentic(ops, namespace, verifier);

    // Actors in order, and ops within an actor by `(seq, id)` so the wire
    // payload is deterministic — two nodes with the same holdings produce
    // byte-identical bodies, which makes a diff of two captures mean
    // something (and orders a fork's two branches the same on every node).
    // By the DERIVED id (svrngs, U2), never the one a line carries.
    let mut by_actor: BTreeMap<&str, Vec<(OpId, &Op<SignedOp>)>> = BTreeMap::new();
    for op in ops {
        by_actor
            .entry(op.actor.as_str())
            .or_default()
            .push((crate::derive_id(op, namespace), op));
    }
    for held in by_actor.values_mut() {
        held.sort_by(|(a_id, a), (b_id, b)| (a.kind.seq, a_id).cmp(&(b.kind.seq, b_id)));
    }
    let by_actor: BTreeMap<&str, Vec<&Op<SignedOp>>> = by_actor
        .into_iter()
        .map(|(actor, held)| (actor, held.into_iter().map(|(_, op)| op).collect()))
        .collect();

    let mut out: Vec<Op<SignedOp>> = Vec::new();
    let mut spent: usize = 0;
    let mut more = false;
    for (actor, held) in &by_actor {
        let Some(view) = theirs.get(*actor) else {
            // Absent actor: absence asks for everything (module docs) — the
            // first non-fitting op stops the walk and names `more`.
            if !fill(held.clone(), &mut out, &mut spent) {
                return (out, true);
            }
            continue;
        };
        let window: Vec<&Op<SignedOp>> = held
            .iter()
            .copied()
            .filter(|op| op.kind.seq >= view.from && op.kind.seq <= view.mark)
            .collect();
        let over: Vec<&Op<SignedOp>> = held
            .iter()
            .copied()
            .filter(|op| op.kind.seq > view.mark)
            .collect();
        // Folded over what is AUTHENTIC in the window, exactly as the peer's
        // head was ([`authentic`]); the unit below still carries everything
        // held — author-blind, store it and gossip it.
        let mismatch = View::of(
            actor,
            view.from,
            view.mark,
            trusted.iter().map(|(id, op)| (id, *op)),
        )
        .head
            != view.head;
        if !mismatch {
            if !fill(over, &mut out, &mut spent) {
                return (out, true);
            }
            continue;
        }
        // The mismatching window is ONE per-actor unit (the termination
        // argument above): what is in the claimed run first, then what is
        // above the mark — all of it in this call, or none of it.
        let unit: Vec<&Op<SignedOp>> = window.into_iter().chain(over).collect();
        tracing::debug!(
            actor = %crate::actor_prefix(actor),
            from = view.from,
            mark = view.mark,
            ops = unit.len(),
            "rail sync: window_mismatch — the claimed run folds differently here, sending it as one unit"
        );
        if unit.is_empty() {
            continue;
        }
        let cost: usize = unit.iter().map(|op| cost_of(op, budget_bytes)).sum();
        if budget_bytes != NO_BUDGET && !out.is_empty() && spent.saturating_add(cost) > budget_bytes
        {
            more = true;
            continue;
        }
        spent = spent.saturating_add(cost);
        out.extend(unit.into_iter().map(|op| (*op).clone()));
    }
    (out, more)
}

#[cfg(test)]
#[path = "sync_tests.rs"]
mod tests;
