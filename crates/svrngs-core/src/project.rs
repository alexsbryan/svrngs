// SPDX-License-Identifier: AGPL-3.0-or-later
//! The read side. A group's state leaves this crate only as a [`Projection`]
//! folded here, where `no_std` refuses a clock, a socket and `HashMap`'s
//! per-process order, or as [`Wire`], the admitted acts for an app to fold
//! itself.

use alloc::vec::Vec;

use crate::{Admission, AdmittedOp, Invited, Membership, OpId};

mod sealed {
    #[diagnostic::on_unimplemented(
        message = "`{Self}` is folded outside svrngs-core, where the compiler cannot hold it to the acts",
        note = "move the fold into svrngs-core and `impl Projection for {Self}` there"
    )]
    pub trait Sealed {}
}

/// State that is a pure function of a group's admitted acts. Implemented only
/// in this crate, the one place the acts can be read.
///
/// ```
/// fn standing(a: &svrngs_core::Admission) -> usize {
///     a.project::<svrngs_core::Membership>().standing.len()
/// }
/// ```
/// ```compile_fail
/// struct Mine;
/// impl svrngs_core::Projection for Mine {
///     fn fold(_: &svrngs_core::Admission) -> Self {
///         Mine
///     }
/// }
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a Projection, so an Admission cannot hand it out",
    label = "project takes a Projection",
    note = "add `impl Projection for {Self}` in svrngs-core, folding the Admission's acts there"
)]
pub trait Projection: sealed::Sealed + Sized {
    fn fold(admission: &Admission) -> Self;
}

/// The admitted acts in their one order, voided ones marked, for the record
/// API, where an app folds them itself. It only serializes: an adapter
/// carries the acts and answers nothing from them.
///
/// ```
/// fn log(a: &svrngs_core::Admission) -> String {
///     serde_json::to_string(&a.wire()).unwrap()
/// }
/// ```
/// ```compile_fail
/// fn count(a: &svrngs_core::Admission) -> usize {
///     a.ops().len()
/// }
/// ```
#[derive(serde::Serialize)]
#[serde(transparent)]
pub struct Wire<'a>(&'a [AdmittedOp]);

impl Admission {
    /// The state `P` folds from these acts.
    pub fn project<P: Projection>(&self) -> P {
        P::fold(self)
    }

    pub fn wire(&self) -> Wire<'_> {
        Wire(self.ops())
    }
}

impl sealed::Sealed for Membership {}

/// The walk admission made to decide who is in.
impl Projection for Membership {
    fn fold(admission: &Admission) -> Self {
        admission.membership().clone()
    }
}

/// Every `Invite` held, in the record's order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invites(Vec<(OpId, Invited)>);

impl Invites {
    /// The newest `Invite` `issuer` wrote naming `key`.
    pub fn newest(&self, issuer: &str, key: &str) -> Option<&OpId> {
        self.0
            .iter()
            .rev()
            .find(|(_, i)| i.issuer == issuer && i.key == key)
            .map(|(id, _)| id)
    }
}

impl sealed::Sealed for Invites {}

impl Projection for Invites {
    fn fold(admission: &Admission) -> Self {
        let held = &admission.membership().invites;
        Self(
            admission
                .ops()
                .iter()
                .filter_map(|op| Some((op.id.clone(), held.get(&op.id)?.clone())))
                .collect(),
        )
    }
}

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;
