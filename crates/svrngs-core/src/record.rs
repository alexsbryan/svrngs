// SPDX-License-Identifier: AGPL-3.0-or-later
//! The record's command for what apps write, and a person's yes to a
//! version of an app the group carries ([`RailAct::Runs`]; canon-pin).

use alloc::vec;
use alloc::vec::Vec;

use crate::command::{Command, Reads};
use crate::{Admission, RailAct};

/// An app's act, written by this computer as its own person: another
/// device's is signed there and carried (capability-signed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Append {
    pub act: RailAct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppendRefusal {
    /// Admitting, removing and inviting are the register's commands, never
    /// an app's, and a seal is no one's (arch-seal).
    NotAnAppAct,
}

impl Command for Append {
    type Refusal = AppendRefusal;
    const READS: Reads = Reads::Membership;

    fn decide(&self, _before: &Admission) -> Result<Vec<RailAct>, AppendRefusal> {
        let app_act = matches!(self.act, RailAct::Record { .. } | RailAct::Correct { .. });
        if !app_act && !matches!(self.act, RailAct::Runs { .. }) {
            return Err(AppendRefusal::NotAnAppAct);
        }
        Ok(vec![self.act.clone()])
    }
}

#[cfg(test)]
#[path = "record_tests.rs"]
mod tests;
