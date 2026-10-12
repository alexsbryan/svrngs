// SPDX-License-Identifier: AGPL-3.0-or-later
//! The register (DESIGN §6, U1): who stands in a group, read from the group
//! itself — its seed and its `Admit` and `Remove` acts, as
//! [`membership`](crate::membership::membership) folds them over the acts
//! admission authenticates. One answer, asked by all four callers: the round
//! offers a group to its standing keys, the acceptor admits a dialer standing
//! in any group the node holds, the sync route serves a dialer standing in
//! the group its request names, and the door signs only for a standing key.
//! Upstream asked the mesh for the first three and the seed for the last
//! (DESIGN §8, U1's inventory, "the mesh" row).
//!
//! The seed is the group's `roster.json`: the founder's key alone, bound to
//! the founder's name, written by `svrngs new` from the group's seed (DESIGN
//! §6).

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{
    admit, is_local_only, ring_dir, valid_namespace, Device, Membership, Op, Person, RailAct, RailError, RingJournal,
    RingRail, RingVerifier, Roster, SignedOp,
};

/// Who stands in one group, on this node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// This node holds no such group, or holds it local-only and offers it
    /// to nobody: nobody stands in it here.
    NotHeld,
    /// The group is held with no seed, or a seed naming no key: nobody can
    /// be shown to stand, so everyone is refused — never an empty roster
    /// read as an answer (ARCH 6).
    NoSeed,
    /// The keys standing now, each with the person it is bound to and
    /// what holds it.
    Members(BTreeMap<String, Member>),
}

/// One fold of a group's register (U7): who stands, each key the group's
/// keeper cut with the person it was bound to, the keeper's key, and every
/// removal and ask to remove, shown to everyone in the group.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Register {
    pub members: BTreeMap<String, Member>,
    pub removed: BTreeMap<String, Person>,
    pub keeper: Option<String>,
    pub removals: Vec<Removing>,
}

/// One counting `Remove`, as the group is shown it (v1-keeper-office (b)):
/// who signed it, whom it names, when it is timed, and whether it was the
/// keeper's cut or an ask, as `membership` judged it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Removing {
    pub by: Person,
    pub of: Person,
    pub at: i64,
    pub said: RemovalSaid,
}

/// What a counting `Remove` was (DESIGN §6, "Removal has one decider").
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemovalSaid {
    /// The keeper's: the cut.
    Removed,
    /// Anyone else's, or one that cut nobody: an ask.
    Asked,
}

/// One key standing in a group: the person it is bound to, and what holds
/// it ([`Device`], u4-1), as `membership` answers both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub person: Person,
    pub device: Device,
}

impl Standing {
    /// Whether `actor` — a key's lowercase hex, as the record spells a
    /// signer — stands in the group.
    pub fn names(&self, actor: &str) -> bool {
        matches!(self, Standing::Members(m) if m.contains_key(actor))
    }
}

/// The keys standing in `namespace` after `ops`, from `seed`: [`register_in`]'s
/// first answer, which the tests hold the kept fold to.
#[cfg(test)]
pub(crate) fn standing_in(
    ops: &[Op<SignedOp>],
    seed: &Roster,
    namespace: &str,
    verifier: &dyn RingVerifier,
) -> Result<BTreeMap<String, Member>, RailError> {
    Ok(register_in(ops, seed, namespace, verifier)?.members)
}

/// The keys standing in `namespace` after `ops`, from `seed`: membership
/// over the acts admission authenticates, each with its person and device;
/// and beside them each key the group's keeper cut, with the person it was
/// bound to, and the keeper (U7). The one decider: [`RingRail::standing`]
/// and [`RingRail::register`] answer with it, through the journal's kept
/// fold (`RingJournal::standing`), which asks it under its write lock.
pub(crate) fn register_in(
    ops: &[Op<SignedOp>],
    seed: &Roster,
    namespace: &str,
    verifier: &dyn RingVerifier,
) -> Result<Register, RailError> {
    let m = membership_in(ops, seed, namespace, verifier)?;
    if m.unsettled {
        tracing::warn!(target: "register", namespace, "register: the group's removals do not come to one answer, so nobody is read as standing");
        return Err(RailError::Unsettled(namespace.to_string()));
    }
    let members = m
        .standing
        .iter()
        .filter_map(|key| {
            let person = m.bindings.get(key)?.clone();
            let device = m.device(key)?;
            Some((key.clone(), Member { person, device }))
        })
        .collect();
    let removed = m
        .cut
        .keys()
        .filter_map(|key| Some((key.clone(), m.bindings.get(key)?.clone())))
        .collect();
    let removals = removals_in(ops, &m, namespace);
    Ok(Register {
        members,
        removed,
        keeper: m.keeper,
        removals,
    })
}

/// Every `Remove` among `ops` that counts in `m`, by time: the cut when it
/// is the one `m` cut its key by, an ask otherwise. One naming a key bound
/// to nobody names no one, said at debug and left out.
fn removals_in(ops: &[Op<SignedOp>], m: &Membership, namespace: &str) -> Vec<Removing> {
    let mut out: Vec<Removing> = ops
        .iter()
        .filter_map(|op| {
            let RailAct::Remove { key, .. } = &op.kind.act else {
                return None;
            };
            let id = crate::derive_id(op, namespace);
            if !m.counts(&id) {
                return None;
            }
            let (Some(by), Some(of)) = (m.bindings.get(&op.actor), m.bindings.get(key)) else {
                tracing::debug!(target: "register", namespace, id = %id, key, "register: a removal names a key bound to nobody, so it is not shown");
                return None;
            };
            let said = if m.cut.get(key).is_some_and(|c| c.by == id) { RemovalSaid::Removed } else { RemovalSaid::Asked };
            tracing::debug!(target: "register", namespace, id = %id, by = %by, of = %of, ?said, "register: a removal shown to the group");
            Some(Removing { by: by.clone(), of: of.clone(), at: op.ts_unix, said })
        })
        .collect();
    out.sort_by_key(|r| r.at);
    out
}

/// `membership` over the acts admission authenticates among `ops`, from
/// `seed`: what [`register_in`] reads, and what a redemption asks of the
/// journal with its `Admit` in it (`crate::redeem`).
pub(crate) fn membership_in(
    ops: &[Op<SignedOp>],
    seed: &Roster,
    namespace: &str,
    verifier: &dyn RingVerifier,
) -> Result<Membership, RailError> {
    // Standing turns only on the acts `membership` reads
    // (`svrngs_core::standing_reads`, beside it in svrngs-core's
    // membership.rs), so the rest are left out before admission verifies
    // each signature — the sync route asks this per request, of journals of
    // tens of thousands of acts (ring_sync's ceiling tests timed out
    // verifying them all).
    let acts: Vec<Op<SignedOp>> = svrngs_core::standing_reads(ops).cloned().collect();
    Ok(admit(&crate::ActSet::of(&acts), &[], seed, namespace, verifier).project::<Membership>())
}

impl RingRail {
    /// Who stands in `namespace` on this node — the ONE accessor the round,
    /// the acceptor and the sync route ask (ARCH 8). Its signatures are
    /// judged by [`crate::Ed25519Verifier`], the scheme every caller passed.
    ///
    /// A group this node does not hold is answered before anything is
    /// opened, so asking about it leaves no trace in the rail. Every branch
    /// traces at debug; a held group with no seed at warn, since it refuses
    /// everyone.
    pub fn standing(&self, namespace: &str) -> Result<Standing, RailError> {
        let (journal, seed) = match self.seeded(namespace)? {
            Ok(held) => held,
            Err(nobody) => return Ok(nobody),
        };
        let (members, kept) = journal.standing(&seed)?;
        let seeded = seed.members.values().flatten().count();
        if kept {
            tracing::debug!(target: "register", namespace, seeded, standing = members.len(),
                "register: standing answered from the kept fold, no act that moves it since");
        } else {
            tracing::debug!(target: "register", namespace, seeded, standing = members.len(),
                "register: standing read from the seed and the group's acts");
        }
        Ok(Standing::Members(members))
    }

    /// The register of `namespace` (U7; DESIGN §6): who stands, each key
    /// its keeper cut with its person, and the keeper; from the journal's
    /// kept fold, as [`Self::standing`] is. A group not held here,
    /// local-only or with no seed is the empty register, said as
    /// [`Self::standing`] says it.
    pub fn register(&self, namespace: &str) -> Result<Register, RailError> {
        let Ok((journal, seed)) = self.seeded(namespace)? else {
            return Ok(Register::default());
        };
        let register = journal.register(&seed)?;
        tracing::debug!(target: "register", namespace, standing = register.members.len(),
            removed = register.removed.len(), keeper = ?register.keeper, "register: the group's register");
        Ok(register)
    }

    /// The journal of `namespace` and its seed, or why nobody stands in it
    /// here: the one check [`Self::standing`] and [`Self::register`] make.
    /// A group not held is said at debug; a held group with no seed at
    /// warn, since it refuses everyone.
    fn seeded(&self, namespace: &str) -> Result<Result<(Arc<RingJournal>, Roster), Standing>, RailError> {
        if !valid_namespace(namespace) || is_local_only(namespace) || !ring_dir(&self.root, namespace).is_dir() {
            tracing::debug!(
                target: "register",
                namespace,
                "register: this node holds no such group to offer, so nobody stands in it here"
            );
            return Ok(Err(Standing::NotHeld));
        }
        let journal = self.journal(namespace)?;
        let Some(seed) = journal.seed()?.filter(|s| !s.members.is_empty()) else {
            tracing::warn!(
                target: "register",
                namespace,
                seed = %journal.roster_path().display(),
                "register: this group has no seed, so nobody can be shown to stand in it and \
                 everyone is refused"
            );
            return Ok(Err(Standing::NoSeed));
        };
        Ok(Ok((journal, seed)))
    }

    /// The first group this node holds, in [`RingRail::namespaces`]' order,
    /// that `actor` stands in, and the person they stand as there; `None`
    /// when they stand in none. The one answer the acceptor
    /// (`node::admit_dialer`) and the member door (`door::through`) ask. A
    /// group whose register is unreadable vouches for nobody, and groups
    /// that cannot be listed vouch for nobody at all.
    pub fn standing_anywhere(&self, actor: &str) -> Option<(String, Person)> {
        let namespaces = match self.namespaces() {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(target: "register", actor, error = %e,
                    "register: cannot list the groups held, so nobody can be shown to stand in one");
                return None;
            }
        };
        for namespace in &namespaces {
            match self.standing(namespace) {
                Ok(Standing::Members(m)) => {
                    if let Some(Member { person, device }) = m.get(actor) {
                        tracing::debug!(target: "register", actor, namespace, %person, ?device,
                            "register: the key stands in a group held here");
                        return Some((namespace.clone(), person.clone()));
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(target: "register", actor, namespace, error = %e,
                        "register: a group's register is unreadable, so it vouches for nobody");
                }
            }
        }
        tracing::debug!(target: "register", actor, groups = namespaces.len(),
            "register: the key stands in no group held here");
        None
    }

    /// The first group held here whose keeper cut `actor`, or `None` (U7):
    /// whom the members' protocol and the door tell they were removed.
    ///
    /// Groups that cannot be listed, and a group whose register is
    /// unreadable, show nobody removed: said at warn, as
    /// [`Self::standing_anywhere`] says them.
    pub fn removed_anywhere(&self, actor: &str) -> Option<String> {
        let namespaces = match self.namespaces() {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(target: "register", actor, error = %e,
                    "register: cannot list the groups held, so nobody can be shown to have been removed from one");
                return None;
            }
        };
        let found = namespaces.into_iter().find(|ns| match self.register(ns) {
            Ok(r) => r.removed.contains_key(actor),
            Err(e) => {
                tracing::warn!(target: "register", actor, namespace = ns, error = %e,
                    "register: a group's register is unreadable, so it shows nobody removed");
                false
            }
        });
        tracing::debug!(target: "register", actor, group = ?found, "register: whether a group held here removed the key");
        found
    }
}

#[cfg(test)]
#[path = "register_tests.rs"]
mod tests;
