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

use crate::{
    admit, is_local_only, ring_dir, valid_namespace, Device, Ed25519Verifier, Membership, Op, Person, RailError,
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

/// The keys standing in `namespace` after `ops`, from `seed`: membership
/// over the acts admission authenticates, each with its person and device. The one
/// decider: [`RingRail::standing`] answers with it, and the door
/// ([`crate::RingJournal::append_all`]) asks it under its write lock.
pub(crate) fn standing_in(
    ops: &[Op<SignedOp>],
    seed: &Roster,
    namespace: &str,
    verifier: &dyn RingVerifier,
) -> Result<BTreeMap<String, Member>, RailError> {
    let m = membership_in(ops, seed, namespace, verifier)?;
    Ok(m.standing
        .iter()
        .filter_map(|key| {
            let person = m.bindings.get(key)?.clone();
            let device = m.device(key)?;
            Some((key.clone(), Member { person, device }))
        })
        .collect())
}

/// `membership` over the acts admission authenticates among `ops`, from
/// `seed`: what [`standing_in`] reads, and what a redemption asks of the
/// journal with its `Admit` in it (`crate::redeem`).
pub(crate) fn membership_in(
    ops: &[Op<SignedOp>],
    seed: &Roster,
    namespace: &str,
    verifier: &dyn RingVerifier,
) -> Result<Membership, RailError> {
    // Standing turns only on the acts `membership` reads
    // (`RailAct::moves_membership`, beside it in svrngs-core's
    // membership.rs), so the rest are left out before admission verifies
    // each signature — the sync route asks this per request, of journals of
    // tens of thousands of acts (ring_sync's ceiling tests timed out
    // verifying them all).
    let acts: Vec<Op<SignedOp>> = ops
        .iter()
        .filter(|op| op.kind.act.moves_membership())
        .cloned()
        .collect();
    Ok(admit(&crate::ActSet::of(&acts), &[], seed, namespace, verifier).project::<Membership>())
}

impl RingRail {
    /// Who stands in `namespace` on this node — the ONE accessor the round,
    /// the acceptor and the sync route ask (ARCH 8). Its signatures are
    /// judged by [`Ed25519Verifier`], the scheme every caller passed.
    ///
    /// A group this node does not hold is answered before anything is
    /// opened, so asking about it leaves no trace in the rail. Every branch
    /// traces at debug; a held group with no seed at warn, since it refuses
    /// everyone.
    pub fn standing(&self, namespace: &str) -> Result<Standing, RailError> {
        if !valid_namespace(namespace) || is_local_only(namespace) || !ring_dir(&self.root, namespace).is_dir() {
            tracing::debug!(
                target: "register",
                namespace,
                "register: this node holds no such group to offer, so nobody stands in it here"
            );
            return Ok(Standing::NotHeld);
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
            return Ok(Standing::NoSeed);
        };
        let (ops, _) = journal.read()?;
        let members = standing_in(&ops, &seed, namespace, &Ed25519Verifier)?;
        tracing::debug!(
            target: "register",
            namespace,
            seeded = seed.members.values().flatten().count(),
            standing = members.len(),
            "register: standing read from the seed and the group's acts"
        );
        Ok(Standing::Members(members))
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
}

#[cfg(test)]
#[path = "register_tests.rs"]
mod tests;
