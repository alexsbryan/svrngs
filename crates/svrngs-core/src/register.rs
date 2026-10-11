// SPDX-License-Identifier: AGPL-3.0-or-later
//! The register's commands: minting an invite, and redeeming one.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::command::{Command, Reads};
use core::convert::Infallible;

use crate::{Admission, Device, InviteFault, OpId, Person, RailAct, Redemption};

/// The issuer's node asked to admit `key` as `person` through its `Invite`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redeem {
    pub invite: OpId,
    pub person: Person,
    pub key: String,
    /// The new key's consent, as the redeeming `Admit` carries it.
    pub cosig: String,
    /// What holds the new key, as the request named it (u4-1).
    pub device: Device,
    /// The link's public key, which the `Invite` must name.
    pub link: String,
    /// This node's key: only an `Invite`'s issuer redeems it.
    pub issuer: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedeemRefusal {
    NoInvite,
    NotOwn,
    OtherLink,
    /// The register would not count the `Admit`.
    Fault(InviteFault),
    /// The register neither counted the `Admit` nor said why.
    Uncounted,
}

impl Command for Redeem {
    type Refusal = RedeemRefusal;
    const READS: Reads = Reads::Membership;

    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, RedeemRefusal> {
        let register = before.membership();
        let invite = register
            .invites
            .get(&self.invite)
            .ok_or(RedeemRefusal::NoInvite)?;
        if invite.issuer != self.issuer {
            return Err(RedeemRefusal::NotOwn);
        }
        if invite.key != self.link {
            return Err(RedeemRefusal::OtherLink);
        }
        if register.standing.contains(&self.key) {
            tracing::debug!(target: "redeem", key = %self.key,
                "redeem: the key stands already, so nothing is written");
            return Ok(Vec::new());
        }
        Ok(vec![RailAct::Admit {
            person: self.person.clone(),
            key: self.key.clone(),
            invite: Some(Redemption {
                id: self.invite.clone(),
                sig: self.cosig.clone(),
                device: self.device,
            }),
        }])
    }

    fn confirm(&self, after: &Admission, written: &[OpId]) -> Result<(), RedeemRefusal> {
        let register = after.membership();
        let admit = &written[0];
        if let Some(fault) = register.refused.get(admit) {
            return Err(RedeemRefusal::Fault(fault.clone()));
        }
        if !register.counts(admit) || !register.standing.contains(&self.key) {
            return Err(RedeemRefusal::Uncounted);
        }
        Ok(())
    }
}

/// A member mints a link: an `Invite` naming the link's key until `until`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintInvite {
    pub key: String,
    pub until: i64,
}

impl Command for MintInvite {
    type Refusal = Infallible;
    const READS: Reads = Reads::Membership;

    fn decide(&self, _before: &Admission) -> Result<Vec<RailAct>, Infallible> {
        Ok(vec![RailAct::Invite {
            key: self.key.clone(),
            until: self.until,
        }])
    }
}

/// A member asks that `person` leave the group (U7; DESIGN §6 "Removal has
/// one decider"): a `Remove` of each key standing as them, naming each key's
/// newest acts this node holds. From the keeper it is the cut; from anyone
/// else an ask, which the keeper sequences by asking the same.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Removal {
    pub person: Person,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemovalRefusal {
    /// No key stands as that person.
    NotInGroup,
    /// The person keeps the group, and the keeper cannot be removed: they
    /// hand it on first.
    Keeps,
}

impl Command for Removal {
    type Refusal = RemovalRefusal;
    const READS: Reads = Reads::All;

    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, RemovalRefusal> {
        let m = before.membership();
        let keys: Vec<&String> = m.keys_of(&self.person).collect();
        let verdict = match keys.iter().any(|k| m.keeper.as_ref() == Some(*k)) {
            _ if keys.is_empty() => Err(RemovalRefusal::NotInGroup),
            true => Err(RemovalRefusal::Keeps),
            false => Ok(keys
                .iter()
                .map(|k| RailAct::Remove {
                    key: (*k).clone(),
                    through: before.tips(k).to_vec(),
                })
                .collect()),
        };
        tracing::debug!(target: "register", person = %self.person, keys = keys.len(),
            keeper = ?m.keeper, ok = verdict.is_ok(), "removal: decided");
        verdict
    }
}

/// The keeper hands the group's keeping to `person`'s computer (U7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandOnTo {
    pub person: Person,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandOnRefusal {
    /// No computer's key stands as that person.
    NotInGroup,
    /// The signer does not keep the group, so the hand-on would move
    /// nothing.
    NotKeeper,
}

impl HandOnTo {
    fn key<'a>(&self, m: &'a crate::Membership) -> Option<&'a String> {
        m.keys_of(&self.person)
            .find(|k| m.device(k) == Some(Device::Computer))
    }
}

impl Command for HandOnTo {
    type Refusal = HandOnRefusal;
    const READS: Reads = Reads::Membership;

    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, HandOnRefusal> {
        let key = self.key(before.membership());
        tracing::debug!(target: "register", person = %self.person, ?key, "hand-on: the computer it goes to");
        let key = key.ok_or(HandOnRefusal::NotInGroup)?;
        Ok(vec![RailAct::HandOn { key: key.clone() }])
    }

    fn confirm(&self, after: &Admission, _written: &[OpId]) -> Result<(), HandOnRefusal> {
        let m = after.membership();
        let moved = self.key(m).is_some_and(|k| m.keeper.as_ref() == Some(k));
        tracing::debug!(target: "register", person = %self.person, moved, "hand-on: the fold's verdict");
        moved.then_some(()).ok_or(HandOnRefusal::NotKeeper)
    }
}

#[cfg(test)]
#[path = "register_tests.rs"]
mod tests;
