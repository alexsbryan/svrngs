// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/membership.rs@e1a43bcd2
//! Membership — the seed plus the acts that move it, in one function (ARCH
//! §10.6).
//!
//! `RING_APPLICATIONS.md` "Amendment 2026-09-18" is the design; this is its
//! one membership function, the piece `admit` calls and "the piece most likely
//! to be replaced", with the permutation property as its contract. Nothing
//! else on the journal changes membership. `p2panda-auth` was read first.
//!
//! The rule (svrngs, U7; DESIGN §6 "Removal has one decider"):
//!
//! 1. **One walk in the rail's order** `(ts_unix, actor, seq, id)`: an act
//!    counts iff its signer holds standing at the act's position — the
//!    seed's keys to start, plus keys admitted by counting `Admit`s, minus
//!    keys the keeper cut — and, for a cut key, iff the cut covers it. The
//!    keeper is one key at every position: the seed's (the maker's; for a
//!    seed of many keys, upstream's shape, the first key of the person whose
//!    name sorts first), moved only by the keeper's own counting
//!    [`RailAct::HandOn`] to a standing key. A counting [`RailAct::Remove`]
//!    by the keeper of a standing key other than its own is the cut; by
//!    anyone else it is an ask, and cuts nobody. A cut key's own acts after
//!    it count nowhere, so it cannot remove, ask or hand on.
//! 2. **The cut names acts by hash.** It covers the acts `through` names and
//!    every act they descend from by `prev`; any other act of the cut key
//!    counts nowhere, wherever its timestamp puts it. Which cuts count turns
//!    on the walk, and the walk on which acts the cuts cover, so the walk
//!    runs again with the cuts it found until they are the cuts it was
//!    given (a cut key's unseen acts cannot move the keeper, who alone
//!    cuts, so this settles at the second walk unless the keeper forked;
//!    it is bounded either way, and a function of the set).
//! 3. **Then the void set**: the targets of counting [`RailAct::Correct`]s,
//!    except an act that moves membership, which nothing undoes. Voids
//!    never resurrect, and they cannot reach standing, so they come after
//!    the walk. A voided act is not [`Membership::counted`].
//!
//! The permutation contract: the walk sorts, so every permutation of the op
//! set produces a byte-identical [`Membership`], gaps included.
//! `ra-membership-is-order-free`'s legs pin exactly that.
//!
//! svrngs (U2, DESIGN §6 "A link admits each person once"): an `Admit` that
//! redeems an [`RailAct::Invite`] counts only when its signer signed that
//! `Invite`, the `Invite` is held and counted earlier in the walk, the
//! `Admit` is timed before the `Invite`'s `until`, and the admitted key's
//! co-signature verifies. Otherwise it is [`Membership::refused`] with its
//! [`InviteFault`], and binds nothing. An `Admit` naming no `Invite` keeps
//! the rule above.
//!
//! An issuer has one open link (u2-4b, DESIGN §8 "A new link closes the old
//! one"): their newest counting `Invite` at a place in the walk. A
//! redemption under an older one of theirs, after the newer, is refused
//! [`InviteFault::Closed`]; what the older link admitted before keeps its
//! standing, since closing is a place in the order.
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
    /// Who holds standing NOW: the seed plus counting `Admit`s, minus the
    /// keys the keeper cut.
    pub standing: BTreeSet<String>,
    /// Derived ids of the acts that count: the signer held standing at the
    /// act's position in the rail's order, a cut covers it if its signer was
    /// cut, and no counting `Correct` voids it. Everything else is held and
    /// reported, never counted.
    pub counted: BTreeSet<OpId>,
    /// The void set — the targets of counting corrections, never an act
    /// that moves membership.
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
    /// The group's keeper now (U7): the seed's, or whoever it was handed
    /// on to. `None` only for an empty seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keeper: Option<String>,
    /// Each key the keeper cut, with the cut (U7).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cut: BTreeMap<String, Cut>,
}

/// One counting cut: the keeper's `Remove`, and the acts it names in
/// `through` that this copy does not hold, so cannot yet cover.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cut {
    pub by: OpId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unheld: Vec<OpId>,
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

    /// The keys standing now as `person`.
    pub fn keys_of<'a, 'p>(
        &'a self,
        person: &'p Person,
    ) -> impl Iterator<Item = &'a String> + use<'a, 'p> {
        self.standing
            .iter()
            .filter(move |k| self.bindings.get(*k) == Some(person))
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
    let mut ops: Vec<(&OpId, &Op<SignedOp>)> = ops.into_iter().collect();
    ops.sort_by_key(|(id, op)| (op.ts_unix, op.actor.clone(), op.kind.seq, *id));

    // ── the link: each redemption judged once on what it alone says ───
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

    // ── what follows a hand-on in its author's chain keeps nothing ───
    //
    // An act descending by `prev` from a `HandOn` its own author signed
    // carries no keeper's authority, wherever its timestamp puts it: an
    // ex-keeper's backdated second hand-on moves nothing.
    let mut by_chain = ops.clone();
    by_chain.sort_by_key(|(id, op)| (op.actor.clone(), op.kind.seq, *id));
    // id → whether the act is a hand-on or follows one, along its chain.
    let mut handed: BTreeMap<&OpId, (bool, bool)> = BTreeMap::new();
    for (id, op) in &by_chain {
        let after = op
            .kind
            .prev
            .as_ref()
            .and_then(|p| handed.get(p))
            .is_some_and(|(is, after)| *is || *after);
        handed.insert(*id, (matches!(op.kind.act, RailAct::HandOn { .. }), after));
    }
    let kept: BTreeSet<&OpId> = handed
        .into_iter()
        .filter(|(_, (_, after))| !after)
        .map(|(id, _)| id)
        .collect();

    // ── the walk, again until the cuts it finds are the cuts it was given ──
    let mut covers: BTreeMap<String, BTreeSet<OpId>> = BTreeMap::new();
    let mut m = walk(&ops, seed, &redeems, &covers, &kept);
    for round in 0.. {
        let found = covered(&ops, &mut m.cut);
        let settled = found == covers;
        tracing::debug!(target: "rail:membership", round, cuts = found.len(), settled,
            "membership: the walk's cuts against the cuts it was given");
        if settled || round > ops.len() {
            break;
        }
        covers = found;
        m = walk(&ops, seed, &redeems, &covers, &kept);
    }

    // ── the void set, from the corrections that counted ──────────────
    let moves: BTreeSet<&OpId> = ops
        .iter()
        .filter(|(_, op)| op.kind.act.moves_membership())
        .map(|(id, _)| *id)
        .collect();
    for (id, op) in &ops {
        if let RailAct::Correct { corrects, .. } = &op.kind.act {
            if !m.counted.contains(*id) {
                continue;
            }
            if moves.contains(corrects) {
                tracing::debug!(target: "rail:membership", id = %id, target = %corrects,
                    "membership: a Correct of an act that moves membership voids nothing");
                continue;
            }
            m.voided.insert(corrects.clone());
        }
    }
    let voided = m.voided.clone();
    m.counted.retain(|id| !voided.contains(id));
    m.invites = ops
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
    m
}

/// For each cut, the acts it covers: the acts of the cut key its `through`
/// names and every act they descend from by `prev`, among `ops`. A name
/// this copy does not hold goes in the cut's `unheld`.
fn covered(
    ops: &[(&OpId, &Op<SignedOp>)],
    cuts: &mut BTreeMap<String, Cut>,
) -> BTreeMap<String, BTreeSet<OpId>> {
    let mut out = BTreeMap::new();
    for (key, cut) in cuts.iter_mut() {
        let chain: BTreeMap<&OpId, Option<&OpId>> = ops
            .iter()
            .filter(|(_, op)| &op.actor == key)
            .map(|(id, op)| (*id, op.kind.prev.as_ref()))
            .collect();
        let Some(RailAct::Remove { through, .. }) = ops
            .iter()
            .find(|(id, _)| **id == cut.by)
            .map(|(_, op)| &op.kind.act)
        else {
            continue;
        };
        let mut seen = BTreeSet::new();
        cut.unheld.clear();
        for tip in through {
            if !chain.contains_key(tip) {
                cut.unheld.push(tip.clone());
            }
            let mut at = Some(tip);
            while let Some(id) =
                at.filter(|id| chain.contains_key(id) && seen.insert((*id).clone()))
            {
                at = chain[id];
            }
        }
        out.insert(key.clone(), seen);
    }
    out
}

/// One walk in the rail's order, a cut key's acts counting only when
/// `covers` holds them, and the keeper's acting as the keeper only when
/// `kept` holds them.
fn walk(
    order: &[(&OpId, &Op<SignedOp>)],
    seed: &Roster,
    redeems: &BTreeMap<&OpId, Result<&OpId, InviteFault>>,
    covers: &BTreeMap<String, BTreeSet<OpId>>,
    kept: &BTreeSet<&OpId>,
) -> Membership {
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
    let mut keeper: Option<String> = seed.members.values().flatten().next().cloned();
    let mut cut: BTreeMap<String, Cut> = BTreeMap::new();
    let mut counted: BTreeSet<OpId> = BTreeSet::new();
    let mut refused: BTreeMap<OpId, InviteFault> = BTreeMap::new();
    let mut browsers: BTreeSet<String> = BTreeSet::new();
    // Each issuer's newest counting `Invite` so far: their one open link.
    let mut open: BTreeMap<&str, &OpId> = BTreeMap::new();
    for (id, op) in order.iter().copied() {
        if !live.contains_key(&op.actor) {
            continue;
        }
        if covers.get(&op.actor).is_some_and(|c| !c.contains(id)) {
            tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor,
                "membership: an act of a cut key its cut does not cover counts nowhere");
            continue;
        }
        let fault = match redeems.get(id) {
            None => None,
            Some(Err(fault)) => Some(fault.clone()),
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
        let keeps = keeper.as_deref() == Some(op.actor.as_str()) && kept.contains(id);
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
                let cuts = keeps && *key != op.actor && live.contains_key(key);
                tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor, key = %key,
                    keeper = ?keeper, cuts,
                    "membership: a Remove — the keeper's cuts; anyone else's is an ask and cuts nobody");
                if cuts {
                    live.remove(key);
                    cut.insert(
                        key.clone(),
                        Cut {
                            by: id.clone(),
                            unheld: Vec::new(),
                        },
                    );
                }
            }
            RailAct::HandOn { key } => {
                let hands = keeps && *key != op.actor && live.contains_key(key);
                tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor, key = %key,
                    keeper = ?keeper, hands,
                    "membership: a HandOn — only the keeper's, to a standing key, moves the keeper");
                if hands {
                    keeper = Some(key.clone());
                }
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
    Membership {
        bindings,
        standing: live.into_keys().collect(),
        counted,
        voided: BTreeSet::new(),
        refused,
        invites: BTreeMap::new(),
        browsers,
        keeper,
        cut,
    }
}

impl RailAct {
    /// The key a `Remove` names, asked or cut; `None` for any other act.
    pub fn names(&self) -> Option<&str> {
        match self {
            RailAct::Remove { key, .. } => Some(key),
            _ => None,
        }
    }

    /// Whether this act moves membership: it binds (`Admit`), cuts or asks
    /// (`Remove`), hands the keeper on (`HandOn`) or opens a link
    /// (`Invite`). A `Correct` does not, since U7: it voids nothing that
    /// moves membership. [`standing_reads`] keeps these, and the acts
    /// of every key a `Remove` names, so the list has one home beside the
    /// fold that reads it, and the match has no wildcard: a new kind of act
    /// does not compile until it is placed here.
    pub const fn moves_membership(&self) -> bool {
        match self {
            RailAct::Admit { .. }
            | RailAct::Remove { .. }
            | RailAct::HandOn { .. }
            | RailAct::Invite { .. } => true,
            RailAct::Record { .. }
            | RailAct::Correct { .. }
            | RailAct::Seal
            | RailAct::Synced { .. } => false,
        }
    }
}

/// The acts a fold of standing reads (U7): every act that moves membership,
/// and every act of a key some `Remove` names, since a cut covers that key's
/// acts along their chain by `prev`. A caller that folds standing from
/// fewer acts than it holds (svrngs-runtime's register) keeps these, and
/// [`RailAct::names`] is how it tells a key named as it goes.
pub fn standing_reads(ops: &[Op<SignedOp>]) -> impl Iterator<Item = &Op<SignedOp>> {
    let named: BTreeSet<&str> = ops.iter().filter_map(|op| op.kind.act.names()).collect();
    ops.iter()
        .filter(move |op| op.kind.act.moves_membership() || named.contains(op.actor.as_str()))
}

/// What a redeeming `Admit` says alone: the `Invite` it names, if that is
/// held, was signed by the `Admit`'s own signer, had not lapsed when the
/// `Admit` was timed, and the admitted key consented. Whether the `Invite`
/// counted is the walk's.
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
