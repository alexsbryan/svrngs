// SPDX-License-Identifier: AGPL-3.0-or-later
//! Redeeming a link: the issuer's node admits the key that opened it.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::command::{Command, Reads};
use crate::{Admission, InviteFault, OpId, Person, RailAct, Redemption};

/// The issuer's node asked to admit `key` as `person` through its `Invite`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redeem {
    pub invite: OpId,
    pub person: Person,
    pub key: String,
    /// The new key's consent, as the redeeming `Admit` carries it.
    pub cosig: String,
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

#[cfg(test)]
#[path = "redeem_tests.rs"]
mod tests;
