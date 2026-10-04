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
//! The seed is the group's `roster.json`: until U2's link carries it, the
//! founder's key alone (DESIGN §6).

use std::collections::BTreeMap;

use crate::{
    admit, is_local_only, ring_dir, valid_namespace, Op, Person, RailAct, RailError, RingRail,
    RingVerifier, Roster, SignedOp,
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
    /// The keys standing now, each with the person it is bound to.
    Members(BTreeMap<String, Person>),
}

impl Standing {
    /// Whether `actor` — a key's lowercase hex, as the record spells a
    /// signer — stands in the group.
    pub fn names(&self, actor: &str) -> bool {
        matches!(self, Standing::Members(m) if m.contains_key(actor))
    }
}

/// The keys standing in `namespace` after `ops`, from `seed`: membership
/// over the acts admission authenticates, each with its person. The one
/// decider: [`RingRail::standing`] answers with it, and the door
/// ([`crate::RingJournal::append_all`]) asks it under its write lock.
pub(crate) fn standing_in(
    ops: &[Op<SignedOp>],
    seed: &Roster,
    namespace: &str,
    verifier: &dyn RingVerifier,
) -> Result<BTreeMap<String, Person>, RailError> {
    // Standing turns only on the acts that bind, cut, void or open a link:
    // `membership` moves standing on `Admit` and `Remove` alone, voids by
    // `Correct` alone, and counts a redeeming `Admit` only under an `Invite`
    // it holds (crates/svrngs-core/src/membership.rs, `membership` and
    // `redemption`), so the rest are left out before admission verifies each signature — the
    // sync route asks this per request, of journals of tens of thousands of
    // acts (ring_sync's ceiling tests timed out verifying them all).
    let acts: Vec<Op<SignedOp>> = ops
        .iter()
        .filter(|op| {
            matches!(
                op.kind.act,
                RailAct::Admit { .. }
                    | RailAct::Remove { .. }
                    | RailAct::Correct { .. }
                    | RailAct::Invite { .. }
            )
        })
        .cloned()
        .collect();
    // `membership` is `None` only on an admission rebuilt from an older wire
    // shape; this one was folded here, so its absence is reported, never
    // read as nobody standing (ARCH 6).
    let m = admit(&acts, &[], seed, namespace, verifier)
        .membership
        .ok_or_else(|| RailError::Io(format!("{namespace}: admission carried no membership")))?;
    Ok(m.standing
        .into_iter()
        .filter_map(|key| {
            let person = m.bindings.get(&key)?.clone();
            Some((key, person))
        })
        .collect())
}

impl RingRail {
    /// Who stands in `namespace` on this node — the ONE accessor the round,
    /// the acceptor and the sync route ask (ARCH 8).
    ///
    /// A group this node does not hold is answered before anything is
    /// opened, so asking about it leaves no trace in the rail. Every branch
    /// traces at debug; a held group with no seed at warn, since it refuses
    /// everyone.
    pub fn standing(
        &self,
        namespace: &str,
        verifier: &dyn RingVerifier,
    ) -> Result<Standing, RailError> {
        if !valid_namespace(namespace)
            || is_local_only(namespace)
            || !ring_dir(&self.root, namespace).is_dir()
        {
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
        let members = standing_in(&ops, &seed, namespace, verifier)?;
        tracing::debug!(
            target: "register",
            namespace,
            seeded = seed.members.values().flatten().count(),
            standing = members.len(),
            "register: standing read from the seed and the group's acts"
        );
        Ok(Standing::Members(members))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{actor_of, Ed25519Verifier, OpId, Redemption};
    use svrngs_core::tests_support::{after, key, record, signed, NS};

    fn admit_act(person: &str, joining: u8) -> RailAct {
        RailAct::Admit {
            person: Person::from(person),
            key: actor_of(&key(joining)),
            invite: None,
        }
    }

    fn void(target: &OpId) -> RailAct {
        RailAct::Correct {
            corrects: target.clone(),
            replacement: None,
        }
    }

    /// Leaving out every act but `Admit`, `Remove`, `Correct` and `Invite`
    /// changes nobody's standing: records between the membership acts, an
    /// `Admit` voided, a `Remove`, a record voided, and an `Admit` redeeming
    /// an `Invite` fold to the same standing keys and persons from those acts
    /// alone as from the whole journal. The failing inputs are a filter that
    /// leaves out `Correct`, and one that leaves out `Invite`.
    #[test]
    fn standing_from_the_membership_acts_alone_is_the_journals() {
        let seed = Roster::new(
            [(Person::from("alex"), vec![actor_of(&key(1))])]
                .into_iter()
                .collect(),
        );
        let admit_bo = signed(&key(1), 100, 0, admit_act("bo", 2));
        let bo_writes = signed(&key(2), 101, 0, record("a"));
        let admit_dee = after(&key(1), &admit_bo, 103, admit_act("dee", 4));
        let void_dee = after(&key(1), &admit_dee, 104, void(&admit_dee.id));
        let remove_cy = after(
            &key(1),
            &void_dee,
            106,
            RailAct::Remove {
                key: actor_of(&key(3)),
                through_seq: 0,
            },
        );
        let ops = vec![
            admit_bo,
            bo_writes.clone(),
            after(&key(2), &bo_writes, 102, admit_act("cy", 3)),
            admit_dee.clone(),
            void_dee,
            signed(&key(3), 105, 0, record("b")),
            remove_cy.clone(),
            after(&key(1), &remove_cy, 107, void(&bo_writes.id)),
        ];
        let inv = after(
            &key(1),
            ops.last().unwrap(),
            108,
            RailAct::Invite {
                key: actor_of(&key(9)),
                until: 200,
            },
        );
        let eve = Person::from("eve");
        let redeem_eve = after(
            &key(1),
            &inv,
            109,
            RailAct::Admit {
                invite: Some(Redemption::sign(&key(5), NS, &inv.id, &eve)),
                person: eve,
                key: actor_of(&key(5)),
            },
        );
        let ops = [ops, vec![inv, redeem_eve]].concat();
        let whole = admit(&ops, &[], &seed, NS, &Ed25519Verifier)
            .membership
            .unwrap();
        let expected: BTreeMap<String, Person> = whole
            .standing
            .iter()
            .map(|k| (k.clone(), whole.bindings[k].clone()))
            .collect();
        assert_eq!(
            expected.keys().cloned().collect::<Vec<_>>(),
            {
                let mut v = vec![actor_of(&key(1)), actor_of(&key(2)), actor_of(&key(5))];
                v.sort();
                v
            },
            "control: alex, bo and eve stand; cy was removed and dee's Admit voided"
        );
        assert_eq!(
            standing_in(&ops, &seed, NS, &Ed25519Verifier).unwrap(),
            expected
        );
    }
}
