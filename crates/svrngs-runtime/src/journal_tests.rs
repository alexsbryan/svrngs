// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail/src/journal.rs@e1a43bcd2
use super::*;

impl RingJournal {
    /// A test's fixture, written through [`commit`](svrngs_core::commit) like
    /// every other act: never compiled outside tests.
    pub fn append(
        &self,
        act: RailAct,
        signer: &SigningKey,
        roster: &Roster,
        on_behalf_of: Option<&str>,
        verifier: &dyn RingVerifier,
    ) -> Result<Op<SignedOp>, RailError> {
        let fixture = svrngs_core::tests_support::Fixture {
            act,
            on_behalf_of: on_behalf_of.map(str::to_string),
        };
        let author = svrngs_core::Author {
            signer,
            roster,
            namespace: &self.namespace,
            verifier,
            now: now_unix(&self.namespace)?,
        };
        match svrngs_core::commit(self, &fixture, &author)? {
            Ok(recorded) => Ok(recorded.lines()[0].clone()),
            Err(_) => Err(RailError::NotInRoster {
                actor: actor_of(signer),
                namespace: self.namespace.clone(),
            }),
        }
    }
}
