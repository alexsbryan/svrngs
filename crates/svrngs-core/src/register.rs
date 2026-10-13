// SPDX-License-Identifier: AGPL-3.0-or-later
//! The register's commands: minting an invite, and redeeming one; removing
//! a person, and handing the group's keeping on to one, each named as the
//! fold names them, by name and, of two people of one name, by when they
//! joined ([`crate::Membership::one_named`]).

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::command::{Command, Reads};
use core::convert::Infallible;

use crate::{Admission, Device, InviteFault, Joined, OpId, Person, RailAct, Redemption};

/// The issuer's node asked to admit `key` as `person` through its `Invite`.
/// Decided at the issuer's write, where a link holder's name arrives, and
/// not in the fold, where a rule of "first in the rail's order binds" would
/// let any member's computer evict a person by a backdated `Admit` under
/// their name (ralph/next/bash/CHARTER.md, "The name").
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
    /// A standing person goes by the name, in some case: a name binds once
    /// where a stranger gives one (bash-person), so a link lets no one in
    /// as someone already in. Who holds it, as they are named.
    NameTaken(Person),
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
        if let Some(of) = &invite.device_of {
            tracing::debug!(target: "redeem", key = %self.key, device_of = %of,
                "redeem: a device link, so the key joins as its person and no name is weighed");
        }
        let folded = self.person.as_str().to_lowercase();
        let holder = (register.standing.iter())
            .filter(|_| invite.device_of.is_none())
            .filter_map(|k| Some(register.person_of(k)?.1))
            .find(|p| p.as_str().to_lowercase() == folded);
        if let Some(holder) = holder {
            tracing::debug!(target: "redeem", key = %self.key, person = %self.person, %holder,
                "redeem: refused — a standing person goes by the name, so nothing is written");
            return Err(RedeemRefusal::NameTaken(holder.clone()));
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

/// A member mints a link: an `Invite` naming the link's key until `until`;
/// a device link when it names the person, by their first key, whose device
/// it adds — the minting key's own person, or the fold counts it nowhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintInvite {
    pub key: String,
    pub until: i64,
    pub device_of: Option<String>,
}

impl Command for MintInvite {
    type Refusal = Infallible;
    const READS: Reads = Reads::Membership;

    fn decide(&self, _before: &Admission) -> Result<Vec<RailAct>, Infallible> {
        Ok(vec![RailAct::Invite {
            key: self.key.clone(),
            until: self.until,
            device_of: self.device_of.clone(),
        }])
    }
}

/// A member asks that `person` leave the group (U7; DESIGN §6 "Removal has
/// one decider"): a `Remove` of each key standing as the one person of that
/// name who joined `joined`, when it is asked, every device of theirs,
/// naming each key's newest acts this node holds. From the keeper it is the
/// cut; from anyone else an ask, which the keeper sequences by asking the
/// same.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Removal {
    pub person: Person,
    /// Which of two people of the name (bash-same-name); `None` names
    /// whoever stands under it, if one person does.
    pub joined: Option<Joined>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemovalRefusal {
    /// No key stands as that person.
    NotInGroup,
    /// The ask names no one person: more than one stands under the name
    /// (bash-devices), or none of them joined when it says
    /// (bash-same-name). When each of them joined, which names one.
    Which(Vec<Joined>),
    /// The person keeps the group, and the keeper cannot be removed: they
    /// hand it on first.
    Keeps,
}

impl Command for Removal {
    type Refusal = RemovalRefusal;
    const READS: Reads = Reads::All;

    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, RemovalRefusal> {
        let m = before.membership();
        let keys = (m.one_named(&self.person, self.joined))
            .map(|first| m.devices_of(first).collect::<Vec<&String>>());
        let verdict = match keys {
            Err(each) if each.is_empty() => Err(RemovalRefusal::NotInGroup),
            Err(each) => Err(RemovalRefusal::Which(each)),
            Ok(keys) if keys.iter().any(|k| m.keeper.as_ref() == Some(*k)) => {
                Err(RemovalRefusal::Keeps)
            }
            Ok(keys) => Ok(keys
                .iter()
                .map(|k| RailAct::Remove {
                    key: (*k).clone(),
                    through: before.tips(k).to_vec(),
                })
                .collect()),
        };
        tracing::debug!(target: "register", person = %self.person, joined = ?self.joined,
            keeper = ?m.keeper, ?verdict, "removal: decided");
        verdict
    }
}

/// The keeper hands the group's keeping to `person`'s computer (U7): of
/// the one person of that name who joined `joined`, when it is asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandOnTo {
    pub person: Person,
    /// Which of two people of the name (bash-same-name), as [`Removal`]'s.
    pub joined: Option<Joined>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandOnRefusal {
    /// No computer's key stands as that person.
    NotInGroup,
    /// The ask names no one person, as [`RemovalRefusal::Which`]: when
    /// each person of the name joined.
    Which(Vec<Joined>),
    /// The signer does not keep the group, so the hand-on would move
    /// nothing.
    NotKeeper,
}

impl HandOnTo {
    /// A computer of the one person the ask names.
    fn key<'a>(&self, m: &'a crate::Membership) -> Result<&'a String, HandOnRefusal> {
        match m.one_named(&self.person, self.joined) {
            Ok(first) => (m
                .devices_of(first)
                .find(|k| m.device(k) == Some(Device::Computer)))
            .ok_or(HandOnRefusal::NotInGroup),
            Err(each) if each.is_empty() => Err(HandOnRefusal::NotInGroup),
            Err(each) => Err(HandOnRefusal::Which(each)),
        }
    }
}

impl Command for HandOnTo {
    type Refusal = HandOnRefusal;
    const READS: Reads = Reads::Membership;

    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, HandOnRefusal> {
        let key = self.key(before.membership());
        tracing::debug!(target: "register", person = %self.person, ?key, "hand-on: the computer it goes to");
        Ok(vec![RailAct::HandOn { key: key?.clone() }])
    }

    fn confirm(&self, after: &Admission, _written: &[OpId]) -> Result<(), HandOnRefusal> {
        let m = after.membership();
        let moved = self.key(m).is_ok_and(|k| m.keeper.as_ref() == Some(k));
        tracing::debug!(target: "register", person = %self.person, moved, "hand-on: the fold's verdict");
        moved.then_some(()).ok_or(HandOnRefusal::NotKeeper)
    }
}

/// This computer, `key`, is reached through `url`, or through no relay
/// (relay-in-record): written only when the record says otherwise, so a
/// round that asks each time writes once per move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayAt {
    pub key: String,
    pub url: Option<crate::relay::Url>,
}

impl Command for RelayAt {
    type Refusal = Infallible;
    const READS: Reads = Reads::Membership;

    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, Infallible> {
        let said = before.membership().relay(&self.key);
        let writes = said != self.url;
        tracing::debug!(target: "register", key = %self.key, ?said, url = ?self.url, writes,
            "relay: the record's relay for this computer against the one it is reached through");
        let url = self.url.as_ref().map(ToString::to_string);
        Ok(writes
            .then_some(RailAct::Relay { url })
            .into_iter()
            .collect())
    }
}

#[cfg(test)]
#[path = "register_tests.rs"]
mod tests;
