// SPDX-License-Identifier: AGPL-3.0-or-later
//! The read side. A group's state leaves this crate only as a fold made
//! here, [`Admission::membership`] or [`Admission::invites`], where `no_std`
//! refuses a clock, a socket and `HashMap`'s per-process order, or as
//! [`Wire`], the admitted acts for an app to fold itself. The folds were a
//! sealed `Projection` trait until arch-core; the acts' privacy below already
//! keeps a fold from being made outside this crate, so the seal guarded
//! nothing more.

use alloc::vec::Vec;

use crate::{Admission, AdmittedOp, Invited, OpId};

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
    /// Every `Invite` held, in the record's order.
    pub fn invites(&self) -> Invites {
        let held = &self.membership().invites;
        Invites(
            self.ops()
                .iter()
                .filter_map(|op| Some((op.id.clone(), held.get(&op.id)?.clone())))
                .collect(),
        )
    }

    pub fn wire(&self) -> Wire<'_> {
        Wire(self.ops())
    }
}

/// Every `Invite` held, in the record's order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invites(Vec<(OpId, Invited)>);

impl Invites {
    /// The newest `Invite` `issuer` wrote naming `key`.
    pub fn newest(&self, issuer: &str, key: &str) -> Option<&OpId> {
        let (id, _) = (self.0.iter().rev()).find(|(_, i)| i.issuer == issuer && i.key == key)?;
        Some(id)
    }
}

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;
