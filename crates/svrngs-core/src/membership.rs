// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/membership.rs@e1a43bcd2
//! Membership — the seed plus two act kinds, in one function (ARCH §10.6).
//!
//! `RING_APPLICATIONS.md` "Amendment 2026-09-18" is the design; this is its
//! one membership function, the piece `admit` calls and "the piece most likely
//! to be replaced", with the permutation property as its contract. Nothing
//! else on the journal changes membership. `p2panda-auth` was read first (its
//! strong-removal rule 4 — invalidation is transitive over dependents — is
//! what the walk below gets for free; its absence of a `Remove{key,
//! through_seq}` cut is what makes ours novel).
//!
//! The rule, in two layers:
//!
//! 1. **The void set, computed first and commutatively** (today's rule,
//!    carried forward): the targets of [`RailAct::Correct`]s whose signer is
//!    seed-bound or named by a non-voided [`RailAct::Admit`] — a monotone
//!    fixpoint, so voiding an `Admit` unbinds its key and that key's own
//!    corrections stop voiding. Known limit of the replaceable rule, stated
//!    rather than discovered: a key whose `Admit` is voided in the SAME batch
//!    as its corrections may still land those voids (one pass of the fixpoint
//!    cannot see the other's removal). The same layer binds structurally, so
//!    a key that a `Remove` cut (rather than an un-admit) can still land
//!    voids — a candidate future leg.
//! 2. **One walk in the rail's order** `(ts_unix, actor, seq, id)` over the
//!    survivors: an act counts iff its signer holds standing at the act's
//!    position — the seed's keys to start, plus keys admitted by counting
//!    `Admit`s, minus keys cut by counting `Remove`s. This one sentence is
//!    the cascade (void one `Admit` and everything that key admitted falls —
//!    their admissions were signed by a key that no longer holds), the
//!    no-cuts-back rule (a removed key's later `Remove` does not count), and
//!    the mutual-removal resolver (the rail's order picks one — the
//!    replaceable place).
//!
//! The permutation contract: the walk sorts, so every permutation of the op
//! set produces a byte-identical [`Membership`], gaps included.
//! `ra-membership-is-order-free`'s five legs pin exactly that.

use std::collections::{BTreeMap, BTreeSet};

use crate::op::Op;

use crate::{OpId, Person, RailAct, Roster, SignedOp};

/// What one fold of the membership acts concludes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Membership {
    /// key → person, from the seed and every counting `Admit`. CUMULATIVE: a
    /// cut key keeps its binding — names stay stable and every past act keeps
    /// the person it always had (leg 4) — it only leaves [`Self::standing`].
    pub bindings: BTreeMap<String, Person>,
    /// Who holds standing NOW: the seed plus counting `Admit`s, minus
    /// counting `Remove`s.
    pub standing: BTreeSet<String>,
    /// Derived ids of the acts that count: the signer held standing at the
    /// act's position in the rail's order. Everything else is held and
    /// reported, never counted.
    pub counted: BTreeSet<OpId>,
    /// The void set the walk applied — the targets of counting corrections.
    pub voided: BTreeSet<OpId>,
}

impl Membership {
    /// Whether the act with this derived id counts.
    pub fn counts(&self, id: &OpId) -> bool {
        self.counted.contains(id)
    }

    /// The person an act's actor resolves to, through the record.
    pub fn person_for(&self, actor: &str) -> Option<&Person> {
        self.bindings.get(actor)
    }
}

/// The one membership function. `ops` must already be authenticated (the
/// caller's verifier judged the signatures), each beside the id its content
/// derives ([`crate::derive_id`]; svrngs, U2: the id needs the namespace, so
/// the caller that has it derives once); everything else is decided here,
/// and decided the same way for every arrival order.
pub fn membership<'a>(
    ops: impl IntoIterator<Item = (&'a OpId, &'a Op<SignedOp>)>,
    seed: &Roster,
) -> Membership {
    let ops: Vec<(&OpId, &Op<SignedOp>)> = ops.into_iter().collect();

    // ── layer 1: the void set (commutative; today's rule, carried) ────
    //
    // Targets of corrections whose signer is bound by the seed or by a
    // non-voided Admit. Monotone: voids only grow, so the loop terminates,
    // and "undoing an undo is a new act" keeps the original voided.
    let mut voided: BTreeSet<OpId> = BTreeSet::new();
    loop {
        let mut bound: BTreeSet<&str> = seed
            .members
            .values()
            .flatten()
            .map(String::as_str)
            .collect();
        for (id, op) in &ops {
            if let RailAct::Admit { key, .. } = &op.kind.act {
                if !voided.contains(*id) {
                    bound.insert(key);
                }
            }
        }
        let mut grew = false;
        for (_, op) in &ops {
            if let RailAct::Correct { corrects, .. } = &op.kind.act {
                if bound.contains(op.actor.as_str()) && voided.insert(corrects.clone()) {
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }

    // ── layer 2: one walk in the rail's order over the survivors ──────
    let mut order: Vec<(&OpId, &Op<SignedOp>)> = ops
        .iter()
        .copied()
        .filter(|(id, _)| !voided.contains(*id))
        .collect();
    order.sort_by_key(|(id, op)| (op.ts_unix, op.actor.clone(), op.kind.seq, *id));

    let mut bindings: BTreeMap<String, Person> = seed
        .members
        .iter()
        .flat_map(|(person, keys)| {
            keys.iter()
                .map(move |k| (k.clone(), person.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    let mut live: BTreeMap<String, Person> = bindings.clone();
    let mut counted: BTreeSet<OpId> = BTreeSet::new();
    for (id, op) in order {
        if !live.contains_key(&op.actor) {
            continue;
        }
        counted.insert(id.clone());
        match &op.kind.act {
            RailAct::Admit { person, key } => {
                tracing::debug!(
                    target: "rail:membership",
                    key = %key,
                    person = %person,
                    "membership: standing gained"
                );
                bindings.insert(key.clone(), person.clone());
                live.insert(key.clone(), person.clone());
            }
            RailAct::Remove { key, .. } => {
                tracing::debug!(target: "rail:membership", key = %key, "membership: standing lost");
                live.remove(key);
            }
            _ => {}
        }
    }

    Membership {
        bindings,
        standing: live.into_keys().collect(),
        counted,
        voided,
    }
}

#[cfg(test)]
#[path = "membership_tests.rs"]
mod tests;
