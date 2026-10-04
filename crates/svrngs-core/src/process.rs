// SPDX-License-Identifier: AGPL-3.0-or-later
//! Effects. A process manager decides from the acts in the record alone what
//! this computer must do outside it. [`Admission::react`] is the one way to a
//! [`Due`], and it makes one only when every act the manager names as a cause
//! stands in the record, so the log is written ahead of the effect.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::{derive_id, Admission, Floors, Op, OpId, RailGap, SignedOp};

mod sealed {
    #[diagnostic::on_unimplemented(
        message = "`{Self}` decides an effect outside svrngs-core, where the compiler cannot hold it to the acts",
        note = "move the decision into svrngs-core and `impl ProcessManager for {Self}` there"
    )]
    pub trait Sealed {}
}

/// Decides, from a group's admitted acts, an effect outside the record.
/// Implemented only in this crate, which is `no_std`: a decision here cannot
/// reach a clock, a file or the network.
///
/// ```
/// fn due(a: &svrngs_core::Admission) -> bool {
///     matches!(a.react(&svrngs_core::Compact), Ok(Some(_)))
/// }
/// ```
/// ```compile_fail
/// struct Mine;
/// impl svrngs_core::ProcessManager for Mine {
///     type Effect = ();
///     fn react(&self, _: &svrngs_core::Admission) -> Option<((), Vec<svrngs_core::OpId>)> {
///         None
///     }
/// }
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a ProcessManager, so no effect can be due from it",
    label = "react takes a ProcessManager",
    note = "add `impl ProcessManager for {Self}` in svrngs-core: `react` over the Admission, naming the acts that cause the effect"
)]
pub trait ProcessManager: sealed::Sealed {
    /// What this computer does outside the record.
    type Effect;

    /// The effect the record asks for, and the admitted acts that ask for it.
    fn react(&self, record: &Admission) -> Option<(Self::Effect, Vec<OpId>)>;
}

/// An effect, with the acts in the record that cause it. Only
/// [`Admission::react`] makes one, so an effect that takes a `Due` cannot run
/// ahead of its acts.
///
/// ```
/// fn causes(due: &svrngs_core::Due<svrngs_core::Retire>) -> usize { due.causes().len() }
/// ```
/// ```compile_fail
/// let due = svrngs_core::Due { effect: (), causes: Vec::new() };
/// ```
#[derive(Debug)]
pub struct Due<E> {
    effect: E,
    causes: Vec<OpId>,
}

impl<E> Due<E> {
    pub fn effect(&self) -> &E {
        &self.effect
    }

    pub fn causes(&self) -> &[OpId] {
        &self.causes
    }
}

/// A manager's effect that named no act standing in the record: refused,
/// since no act means no effect. Carries the causes it named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uncaused(pub Vec<OpId>);

impl Admission {
    /// What `manager` decides these acts ask of this computer. `Ok(None)` is
    /// nothing due; `Err` is an effect whose causes are not all acts that
    /// stand here — none, one not admitted, or one a correction voided.
    pub fn react<P: ProcessManager>(
        &self,
        manager: &P,
    ) -> Result<Option<Due<P::Effect>>, Uncaused> {
        let Some((effect, causes)) = manager.react(self) else {
            tracing::debug!(target: "rail:effect", "react: nothing is due");
            return Ok(None);
        };
        let stands = |id: &OpId| self.ops().iter().any(|o| &o.id == id && !o.voided);
        if causes.is_empty() || !causes.iter().all(stands) {
            tracing::warn!(target: "rail:effect", ?causes,
                "react refused — an effect is due only from acts that stand in the record");
            return Err(Uncaused(causes));
        }
        tracing::debug!(target: "rail:effect", ?causes, "react: an effect is due");
        Ok(Some(Due { effect, causes }))
    }
}

/// A seal's process manager: each actor's lines below the floor its seal set
/// retire, except a line a correction names.
pub struct Compact;

/// The lines a group's seals retire, decided over the record before.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retire {
    floors: Floors,
    referenced: BTreeSet<OpId>,
    before: Vec<RailGap>,
}

impl Retire {
    /// The authenticated floors it retires below: [`admit`](crate::admit)'s
    /// own map, carried through rather than re-derived.
    pub fn floors(&self) -> &Floors {
        &self.floors
    }

    /// Whether `line` stays. By derived id, because that is what a correction
    /// resolves against; a line whose on-disk id was rewritten does not
    /// match, and [`Self::raises`] stops that becoming a deletion a
    /// correction would dangle over.
    pub fn keeps(&self, line: &Op<SignedOp>, namespace: &str) -> bool {
        let floor = self.floors.get(&line.actor).copied().unwrap_or(0);
        line.kind.seq >= floor || self.referenced.contains(&derive_id(line, namespace))
    }

    /// The gaps `after` holds that the record before did not. A deletion that
    /// raises one is refused.
    pub fn raises<'a>(&self, after: &'a Admission) -> Vec<&'a RailGap> {
        after
            .gaps()
            .iter()
            .filter(|g| !self.before.contains(g))
            .collect()
    }

    /// The gaps the record before held that `after` does not.
    pub fn clears(&self, after: &Admission) -> usize {
        self.before
            .iter()
            .filter(|g| !after.gaps().contains(g))
            .count()
    }
}

impl sealed::Sealed for Compact {}

impl ProcessManager for Compact {
    type Effect = Retire;

    fn react(&self, record: &Admission) -> Option<(Retire, Vec<OpId>)> {
        if record.floors().is_empty() {
            return None;
        }
        let referenced = record
            .ops()
            .iter()
            .filter_map(|o| o.corrects.clone())
            .collect();
        let retire = Retire {
            floors: record.floors().clone(),
            referenced,
            before: record.gaps().to_vec(),
        };
        Some((retire, record.seals().to_vec()))
    }
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
