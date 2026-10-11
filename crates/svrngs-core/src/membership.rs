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
//!
//! svrngs (U2, DESIGN §6 "A link admits each person once"): an `Admit` that
//! redeems an [`RailAct::Invite`] counts only when its signer signed that
//! `Invite`, the `Invite` is held, no counting `Correct` voids it and it
//! counted earlier in the walk, the `Admit` is timed before the `Invite`'s
//! `until`, and the admitted key's co-signature verifies. Otherwise it is
//! [`Membership::refused`] with its [`InviteFault`], and binds nothing in
//! either layer — a refused redemption's key lands no voids. An `Admit`
//! naming no `Invite` keeps the rule above.
//!
//! An issuer has one open link (u2-4b, DESIGN §8 "A new link closes the old
//! one"): their newest counting `Invite` at a place in the walk. A
//! redemption under an older one of theirs, after the newer, is refused
//! [`InviteFault::Closed`]; what the older link admitted before keeps its
//! standing, since closing is a place in the order and voids are not.
//! Known limit, as for a `Remove` above: the void layer cannot see an
//! order, so a closed redemption's key still binds there and can land
//! voids. Only the issuer signs a redemption, so one exists only if the
//! issuer's own node redeemed a link it had closed.
//!
//! svrngs (u4-1, DESIGN §3 "Joining", call 2): each standing key is held by
//! a [`Device`] — a browser's when the counting `Admit` that last bound it
//! redeemed a link naming a browser, a computer's otherwise: a seed key, an
//! `Admit` naming no `Invite`, and every redemption before line version 4.
//! [`Membership::device`] answers it.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::op::Op;

use crate::{Device, OpId, Person, RailAct, Redemption, Roster, SignedOp};

/// Why a redeeming `Admit` does not count — one per condition of the rule
/// in the module docs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InviteFault {
    /// A counting `Correct` voids the `Invite`, and with it everything it
    /// admitted: voids are order-free, so this is not how a link closes.
    Voided,
    /// No `Invite` with that id is held here.
    Unheld,
    /// The `Admit`'s signer is not the `Invite`'s.
    NotIssuer,
    /// The `Admit` is timed at or after the `Invite`'s `until`.
    Lapsed,
    /// The admitted key's signature over the redemption does not verify.
    BadCosignature,
    /// The `Invite` did not count before the `Admit` in the rail's order.
    Uncounted,
    /// The issuer's newer `Invite`, `by`, counted before the `Admit` in the
    /// rail's order and closed this one (u2-4b).
    Closed { by: OpId },
}

impl core::fmt::Display for InviteFault {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Voided => "its link was taken back",
            Self::Unheld => "this node does not hold its link",
            Self::NotIssuer => "its link was made by someone else",
            Self::Lapsed => "its link had lapsed",
            Self::BadCosignature => "the joining key did not sign it",
            Self::Uncounted => "its link did not count before it",
            Self::Closed { .. } => "its link was closed by a newer one",
        })
    }
}

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
    /// Redeeming `Admit`s by a standing signer that do not count, each with
    /// its fault (svrngs, U2). Skipped when empty, so a fold with none
    /// serialises as before.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub refused: BTreeMap<OpId, InviteFault>,
    /// Every `Invite` held, by derived id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub invites: BTreeMap<OpId, Invited>,
    /// The keys whose counting `Admit` that last bound them names a
    /// browser (u4-1). Cumulative as [`Self::bindings`] is; ask
    /// [`Self::device`], which answers only for a standing key. Skipped
    /// when empty, so a fold with none serialises as before.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub browsers: BTreeSet<String>,
}

/// An `Invite` as the register holds it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Invited {
    pub issuer: String,
    pub key: String,
    pub until: i64,
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

    /// What holds `key`, if it stands now; `None` if it does not.
    pub fn device(&self, key: &str) -> Option<Device> {
        self.standing.contains(key).then(|| {
            if self.browsers.contains(key) {
                Device::Browser
            } else {
                Device::Computer
            }
        })
    }
}

/// The one membership function. `ops` must already be authenticated (the
/// caller's verifier judged the signatures), each beside the id its content
/// derives ([`crate::derive_id`]; svrngs, U2: the id needs the namespace, so
/// the caller that has it derives once); everything else is decided here,
/// and decided the same way for every arrival order. `namespace` is the one
/// a redemption's co-signature binds.
pub fn membership<'a>(
    ops: impl IntoIterator<Item = (&'a OpId, &'a Op<SignedOp>)>,
    seed: &Roster,
    namespace: &str,
) -> Membership {
    let ops: Vec<(&OpId, &Op<SignedOp>)> = ops.into_iter().collect();

    // ── the link: each redemption judged once on what it alone says ───
    //
    // Issuer, lapse and co-signature turn on the `Admit` and its `Invite`
    // alone, so they are judged here, before either layer; the void and
    // whether the `Invite` counted are each layer's own.
    let invites: BTreeMap<&OpId, (&str, i64)> = ops
        .iter()
        .filter_map(|(id, op)| match &op.kind.act {
            RailAct::Invite { until, .. } => Some((*id, (op.actor.as_str(), *until))),
            _ => None,
        })
        .collect();
    let redeems: BTreeMap<&OpId, Result<&OpId, InviteFault>> = ops
        .iter()
        .filter_map(|(id, op)| match &op.kind.act {
            RailAct::Admit {
                person,
                key,
                invite: Some(r),
            } => Some((*id, redemption(op, person, key, r, &invites, namespace))),
            _ => None,
        })
        .collect();

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
                let redeemed = match redeems.get(*id) {
                    None => true,
                    Some(Ok(invite)) => !voided.contains(*invite),
                    Some(Err(_)) => false,
                };
                if redeemed && !voided.contains(*id) {
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
    let mut refused: BTreeMap<OpId, InviteFault> = BTreeMap::new();
    let mut browsers: BTreeSet<String> = BTreeSet::new();
    // Each issuer's newest counting `Invite` so far: their one open link.
    let mut open: BTreeMap<&str, &OpId> = BTreeMap::new();
    for (id, op) in order {
        if !live.contains_key(&op.actor) {
            continue;
        }
        let fault = match redeems.get(id) {
            None => None,
            Some(Err(fault)) => Some(fault.clone()),
            Some(Ok(invite)) if voided.contains(*invite) => Some(InviteFault::Voided),
            Some(Ok(invite)) if !counted.contains(*invite) => Some(InviteFault::Uncounted),
            // Counted, so it opened: the issuer (the signer, past NotIssuer)
            // has an open link, and it is this one or a newer one.
            Some(Ok(invite)) => open
                .get(op.actor.as_str())
                .filter(|newest| **newest != *invite)
                .map(|newest| InviteFault::Closed {
                    by: (*newest).clone(),
                }),
        };
        if let Some(fault) = fault {
            tracing::debug!(
                target: "rail:membership",
                id = %id,
                actor = %op.actor,
                ?fault,
                "membership: a redeeming Admit does not count"
            );
            refused.insert(id.clone(), fault);
            continue;
        }
        counted.insert(id.clone());
        match &op.kind.act {
            RailAct::Admit {
                person,
                key,
                invite,
            } => {
                // An `Admit` naming no `Invite` admits a computer: before a
                // redemption could name a browser, only a computer joined.
                let device = invite.as_ref().map_or(Device::Computer, |r| r.device);
                tracing::debug!(
                    target: "rail:membership",
                    key = %key,
                    person = %person,
                    ?device,
                    "membership: standing gained"
                );
                match device {
                    Device::Browser => browsers.insert(key.clone()),
                    Device::Computer => browsers.remove(key),
                };
                bindings.insert(key.clone(), person.clone());
                live.insert(key.clone(), person.clone());
            }
            RailAct::Remove { key, .. } => {
                tracing::debug!(target: "rail:membership", key = %key, "membership: standing lost");
                live.remove(key);
            }
            RailAct::Invite { .. } => {
                let older = open.insert(op.actor.as_str(), id);
                tracing::debug!(
                    target: "rail:membership",
                    id = %id,
                    actor = %op.actor,
                    closes = ?older,
                    "membership: a link opens, closing its issuer's older one"
                );
            }
            _ => {}
        }
    }

    let invites = ops
        .iter()
        .filter_map(|(id, op)| match &op.kind.act {
            RailAct::Invite { key, until } => Some((
                (*id).clone(),
                Invited {
                    issuer: op.actor.clone(),
                    key: key.clone(),
                    until: *until,
                },
            )),
            _ => None,
        })
        .collect();
    Membership {
        bindings,
        standing: live.into_keys().collect(),
        counted,
        voided,
        refused,
        invites,
        browsers,
    }
}

impl RailAct {
    /// Whether [`membership`] reads this act: it binds (`Admit`), cuts
    /// (`Remove`), voids (`Correct`) or opens a link (`Invite`). A caller that
    /// folds standing from fewer acts than the journal holds (svrngs-node's
    /// `register::standing_in`) keeps these and only these, so the list has
    /// one home beside the fold that reads it, and the match has no wildcard:
    /// a new kind of act does not compile until it is placed here.
    pub const fn moves_membership(&self) -> bool {
        match self {
            RailAct::Admit { .. }
            | RailAct::Remove { .. }
            | RailAct::Correct { .. }
            | RailAct::Invite { .. } => true,
            RailAct::Record { .. } | RailAct::Seal | RailAct::Synced { .. } => false,
        }
    }
}

/// What a redeeming `Admit` says alone: the `Invite` it names, if that is
/// held, was signed by the `Admit`'s own signer, had not lapsed when the
/// `Admit` was timed, and the admitted key consented. The void and whether
/// the `Invite` counted are the walk's.
fn redemption<'a>(
    op: &Op<SignedOp>,
    person: &Person,
    key: &str,
    r: &'a Redemption,
    invites: &BTreeMap<&OpId, (&str, i64)>,
    namespace: &str,
) -> Result<&'a OpId, InviteFault> {
    let verdict = match invites.get(&r.id) {
        None => Err(InviteFault::Unheld),
        Some((issuer, _)) if *issuer != op.actor => Err(InviteFault::NotIssuer),
        Some((_, until)) if op.ts_unix >= *until => Err(InviteFault::Lapsed),
        Some(_) if !r.verifies(namespace, person, key) => Err(InviteFault::BadCosignature),
        Some(_) => Ok(&r.id),
    };
    tracing::debug!(
        target: "rail:membership",
        invite = %r.id,
        actor = %op.actor,
        key,
        ?verdict,
        "membership: a redemption judged on what it says"
    );
    verdict
}

#[cfg(test)]
#[path = "membership_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "membership_invite_tests.rs"]
mod invite_tests;

#[cfg(test)]
#[path = "removal_tests.rs"]
mod removal_tests;
