// SPDX-License-Identifier: AGPL-3.0-or-later
//! The record's commands for what apps write, and the seal.

use alloc::vec;
use alloc::vec::Vec;
use core::convert::Infallible;

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
    /// Admitting, removing, inviting and sealing are the register's and the
    /// seal's commands, never an app's.
    NotAnAppAct,
    /// The person named holds no standing in the group.
    NotInGroup(Person),
}

impl Command for Append {
    type Refusal = AppendRefusal;
    const READS: Reads = Reads::Membership;

    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, AppendRefusal> {
        if !matches!(self.act, RailAct::Record { .. } | RailAct::Correct { .. }) {
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

/// This computer retires everything it wrote before the seal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seal;

impl Command for Seal {
    type Refusal = Infallible;
    const READS: Reads = Reads::Membership;

    fn decide(&self, _before: &Admission) -> Result<Vec<RailAct>, Infallible> {
        Ok(vec![RailAct::Seal])
    }
}

#[cfg(test)]
#[path = "record_tests.rs"]
mod tests;
