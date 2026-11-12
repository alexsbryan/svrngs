// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a joiner signs with a link's key (DESIGN §6, "A link admits each
//! person once"): the [`AskRequest`] that asks the issuer what the link is
//! for, the issuer's answer to it ([`Asked`], here since u4-5, so a page
//! reads it as the command does), and the [`RedeemRequest`] that presents
//! the redemption. Moved from
//! `svrngs-node`'s `redeem.rs` (u4-2; DESIGN §3, "Joining", call 5), so the command
//! and a browser's page sign them, and the issuer's node checks them, with
//! one body: keys and text in, bytes and hex out. The routes that answer
//! them, and every check that reads a journal, stay the node's.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    actor_of, field, sign_message, verify_hex, Device, OpId, Person, Redemption, SigningKey,
};

/// What a joiner holding only a link asks its issuer: the link's public
/// key and the joiner's, actor hex, signed by the link's secret key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskRequest {
    /// The link's public key.
    pub link: String,
    /// The key the joiner dials with.
    pub key: String,
    /// The link's secret key's signature over the two.
    pub sig: String,
}

impl AskRequest {
    /// `joining` asks about the link whose secret key is `link`.
    pub fn sign(link: &SigningKey, joining: &SigningKey) -> Self {
        let mut req = Self {
            link: actor_of(link),
            key: actor_of(joining),
            sig: String::new(),
        };
        req.sig = sign_message(link, &req.message());
        req
    }

    /// `DOMAIN || link || key`, each preceded by its `u32` big-endian length.
    fn message(&self) -> Vec<u8> {
        let mut msg = crate::sig::ASK_DOMAIN.to_vec();
        field(&mut msg, self.link.as_bytes());
        field(&mut msg, self.key.as_bytes());
        msg
    }

    /// Whether the link's key signed this ask. `false` on anything
    /// malformed.
    pub fn signed_by_link(&self) -> bool {
        verify_hex(&self.link, &self.message(), &self.sig)
    }
}

/// The issuer's answer to an ask. The joiner believes `seed` only if it
/// hashes to `group` ([`crate::seed::answered`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asked {
    /// The id of the group the `Invite` is in.
    pub group: String,
    /// That group's seed, as the issuer keeps it.
    pub seed: String,
    /// The id of the issuer's `Invite` naming the link's key.
    pub invite: OpId,
    /// A device link's person, by name (bash-devices): the joiner joins as
    /// them, asked no name. Skipped when absent, so any other answer reads
    /// as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person: Option<Person>,
}

/// What a joiner presents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedeemRequest {
    /// The group's id, as the seed hashes it.
    pub group: String,
    /// The id of the issuer's `Invite`, as the link carries it.
    pub invite: OpId,
    /// The name the joiner goes by in the group, as sent: the issuer parses
    /// it ([`crate::Name::kept`]) before anything is written.
    pub person: String,
    /// The new key, actor hex: the key the joiner dials with.
    pub key: String,
    /// The new key's consent, as a redeeming `Admit` carries it.
    pub cosig: String,
    /// What holds the new key: the `Admit` names it as sent.
    pub device: Device,
    /// The link's public key, actor hex.
    pub link: String,
    /// The link's secret key's signature over everything above.
    pub sig: String,
}

impl RedeemRequest {
    /// `joining`, held by `device`, asks to join `group` as `person`,
    /// through the link whose secret is `link` and whose `Invite` is
    /// `invite`.
    pub fn sign(
        link: &SigningKey,
        joining: &SigningKey,
        group: &str,
        invite: &OpId,
        person: &Person,
        device: Device,
    ) -> Self {
        let consent = Redemption::sign(joining, group, invite, person, device);
        let mut req = Self {
            group: group.to_string(),
            invite: invite.clone(),
            person: person.as_str().to_string(),
            key: actor_of(joining),
            cosig: consent.sig,
            device,
            link: actor_of(link),
            sig: String::new(),
        };
        req.sig = sign_message(link, &req.message());
        req
    }

    /// The bytes the link's key signs: `DOMAIN || group || invite[32] ||
    /// person || key || cosig || device || link`, every variable field
    /// preceded by its `u32` big-endian length. Public so a test can sign a
    /// request naming a person no `Person` holds, as a hand-built client
    /// could.
    pub fn message(&self) -> Vec<u8> {
        let mut msg = crate::sig::REQUEST_DOMAIN.to_vec();
        field(&mut msg, self.group.as_bytes());
        field(&mut msg, self.invite.as_bytes());
        field(&mut msg, self.person.as_bytes());
        field(&mut msg, self.key.as_bytes());
        field(&mut msg, self.cosig.as_bytes());
        field(&mut msg, self.device.as_str().as_bytes());
        field(&mut msg, self.link.as_bytes());
        msg
    }

    /// Whether the link's key signed this request. `false` on anything
    /// malformed.
    pub fn signed_by_link(&self) -> bool {
        verify_hex(&self.link, &self.message(), &self.sig)
    }
}
