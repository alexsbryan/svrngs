// SPDX-License-Identifier: AGPL-3.0-or-later
//! The record's command for what apps write, and a person's yes to a
//! version of an app the group carries ([`RailAct::Runs`]; canon-pin).

use alloc::vec;
use alloc::vec::Vec;

use crate::command::{Command, Reads};
use crate::{Admission, Person, RailAct};

/// An app's act, written by this computer, perhaps on behalf of a person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Append {
    pub act: RailAct,
    pub on_behalf_of: Option<Person>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppendRefusal {
    /// Admitting, removing and inviting are the register's commands, never
    /// an app's, and a seal is no one's (arch-seal).
    NotAnAppAct,
    /// The person named holds no standing in the group.
    NotInGroup(Person),
}

impl Command for Append {
    type Refusal = AppendRefusal;
    const READS: Reads = Reads::Membership;

    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, AppendRefusal> {
        let app_act = matches!(self.act, RailAct::Record { .. } | RailAct::Correct { .. });
        if !app_act && !matches!(self.act, RailAct::Runs { .. }) {
            return Err(AppendRefusal::NotAnAppAct);
        }
        if let Some(person) = &self.on_behalf_of {
            if before.membership().keys_of(person).next().is_none() {
                return Err(AppendRefusal::NotInGroup(person.clone()));
            }
        }
        Ok(vec![self.act.clone()])
    }

    fn on_behalf_of(&self) -> Option<&str> {
        self.on_behalf_of.as_ref().map(Person::as_str)
    }
}

#[cfg(test)]
#[path = "record_tests.rs"]
mod tests;
