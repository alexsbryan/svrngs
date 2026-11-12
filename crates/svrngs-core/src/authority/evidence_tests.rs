// SPDX-License-Identifier: AGPL-3.0-or-later
//! The interpreter's queries only the laws ask, each traced at debug as
//! before. The product folds through [`super::interpreted`] alone; these
//! sat beside it in `evidence.rs`, counted as product, until the economizing
//! pass before release 1 moved them here unchanged.
use super::*;
use crate::authority::GroupPolicy;
use crate::{ActSet, RingVerifier};

/// Moved here from `evidence.rs` with adoption (release 2): only these
/// laws read lines this way now.
fn originals<'a>(
    namespace: &str,
    held: &'a ActSet,
    verifier: &dyn RingVerifier,
) -> (Lines<'a>, BTreeSet<OpId>) {
    let (mut originals, mut invalid) = (Lines::new(), BTreeSet::new());
    for op in held.iter() {
        let id = crate::derive_id(op, namespace);
        if !verifier.verify(&op.actor, &id, &op.kind.sig) {
            invalid.insert(id);
            continue;
        }
        // Version is unsigned envelope metadata. Preserve a supported encoding
        // of this signed body when a peer also sends an unsupported alias.
        let rank = |op: &Op<SignedOp>| {
            (
                !(op.kind.line_version()..=SignedOp::UNDERSTANDS).contains(&op.v),
                op.v,
            )
        };
        if originals.get(&id).is_none_or(|old| rank(op) < rank(old)) {
            originals.insert(id, op);
        }
    }
    (originals, invalid)
}

impl SourceRoots {
    pub fn from_seed(seed: &crate::seed::Seed) -> Judged<Self> {
        if seed.policy() != Some(GroupPolicy::SourcesV1) {
            return refused(SourceFault::Root);
        }
        Self::genesis(&seed.id(), &seed.roster(), &seed.founder_actor())
    }
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}

impl<'a> NativeAuthority<'a> {
    pub fn new(roots: &'a SourceRoots, held: &ActSet, verifier: &dyn RingVerifier) -> Self {
        let (lines, forged) = originals(&roots.namespace, held, verifier);
        Self::of(roots, &lines, forged)
    }
    pub fn identity(&self, key: &str, source: &AuthorityRef) -> Judged<SourceMember> {
        let result = self.graph().identity(key, source);
        tracing::debug!(target: "rail:source", key, ?source, ?result, "source: original binding");
        result
    }
    pub fn office(&self) -> SourceOffice {
        let office = self.graph().office();
        tracing::debug!(target: "rail:source", ?office, "source: office resolved");
        office
    }
    pub fn members(&self) -> BTreeMap<String, SourceMember> {
        self.graph().members()
    }
    pub fn assess(&self, id: &OpId) -> Judged<()> {
        let result = self.graph().assess(id);
        tracing::debug!(target: "rail:source", %id, ?result, "source: native statement assessment");
        result
    }
}
