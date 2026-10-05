// SPDX-License-Identifier: AGPL-3.0-or-later
//! A lend's address, and the pass that opens it (DESIGN §3, "Lent apps",
//! calls 1 and 6; u5-2). New: upstream had no lend at an origin of its own
//! and nothing a person signs for another key (DESIGN §8, U5's inventory).
//!
//! **A lend's label** is one DNS label naming its lender's key and its
//! name, so a lend can be an origin of its own — `<label>.localhost` on a
//! computer, `<label>.<page host>` on a phone — under a wildcard
//! certificate, which covers one label. A label holds at most 63 bytes, and
//! a lend's name may be 64, so the label is the lender's key in full, which
//! is dialled, then ten letters of the name's hash, which only its lender's
//! door reads back, from what it declares: base32, lowercase, 62 letters.
//!
//! **A pass**: a person's key signs that a fresh key, the holder, may act
//! as them toward the lends it names until a time. The page signs one for
//! the lend it opens, and the lend's origin dials with the holder, so the
//! person's key never enters a lend's origin. The door checks it
//! ([`Pass::check`]) and then sets the person as it would for the person's
//! own key. Pure: the time is an argument.

use alloc::string::String;
use alloc::vec::Vec;

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::{actor_of, field, sign_message, verify_hex, SigningKey};

/// Domain separator for a lend name's hash in its label.
const LABEL_DOMAIN: &[u8] = b"svrngs-lend-label:";

/// Domain separator for a person's signature over a pass, so it is never
/// valid as an act's, a link's or a consent.
const PASS_DOMAIN: &[u8] = b"svrngs-pass:";

/// RFC 4648's base32 alphabet, lowercased: every letter a DNS label takes.
const BASE32: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

/// The lender's 32 bytes in base32, and the name's part.
const KEY_LETTERS: usize = 52;
const NAME_LETTERS: usize = 10;

/// `bytes` in [`BASE32`], unpadded.
fn base32(bytes: &[u8]) -> String {
    let (mut out, mut acc, mut bits) = (String::new(), 0u32, 0);
    for &b in bytes {
        acc = (acc << 8) | u32::from(b);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(BASE32[((acc >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(BASE32[((acc << (5 - bits)) & 31) as usize] as char);
    }
    out
}

/// The 32 bytes [`base32`] wrote as `letters`, or `None`.
fn key_of(letters: &str) -> Option<[u8; 32]> {
    let (mut out, mut acc, mut bits) = (Vec::with_capacity(33), 0u32, 0);
    for c in letters.bytes() {
        let v = BASE32.iter().position(|&a| a == c)? as u32;
        acc = (acc << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    // 52 letters carry 260 bits: the key's 256 and four zeros.
    (acc & ((1 << bits) - 1) == 0).then_some(())?;
    out.try_into().ok()
}

/// A lend's label (the module docs).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Label(String);

impl Label {
    /// The label of the lend `name` lent by the computer whose key is
    /// `lender`.
    pub fn of(lender: &[u8; 32], name: &str) -> Self {
        let mut hashed = LABEL_DOMAIN.to_vec();
        hashed.extend_from_slice(name.as_bytes());
        let mut label = base32(lender);
        label.push_str(&base32(blake3::hash(&hashed).as_bytes())[..NAME_LETTERS]);
        Self(label)
    }

    /// `raw` as a label: 62 letters of [`BASE32`] whose first 52 spell a
    /// key. `None` for anything else, in any case: a host is lowercased
    /// before it is read.
    pub fn parse(raw: &str) -> Option<Self> {
        (raw.len() == KEY_LETTERS + NAME_LETTERS && raw.bytes().all(|c| BASE32.contains(&c)))
            .then_some(())?;
        key_of(&raw[..KEY_LETTERS])?;
        Some(Self(raw.into()))
    }

    /// The lender's key, dialled to reach the lend.
    pub fn lender(&self) -> [u8; 32] {
        key_of(&self.0[..KEY_LETTERS]).expect("a label parsed holds a key")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Label {
    type Error = &'static str;
    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw).ok_or("not a lend's label")
    }
}

impl From<Label> for String {
    fn from(label: Label) -> Self {
        label.0
    }
}

impl core::fmt::Display for Label {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A pass (the module docs): keys as the record spells an actor, its
/// lapse in seconds since 1970.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pass {
    /// The person it lets act: their key.
    pub person: String,
    /// The key that may act as them.
    pub holder: String,
    /// The lends it opens.
    pub lends: Vec<Label>,
    /// When it stops opening them.
    pub until: i64,
    /// The person's signature over the four.
    pub sig: String,
}

/// Why a pass opens nothing here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PassFault {
    #[error("not signed by the person it names")]
    Unsigned,
    #[error("held by another key than the one presenting it")]
    NotTheHolder,
    #[error("lapsed")]
    Lapsed,
    #[error("it does not name this lend")]
    NotNamed,
}

impl Pass {
    /// `person` lets `holder`, actor hex, act as them toward `lends` until
    /// `until`.
    pub fn sign(person: &SigningKey, holder: &str, lends: Vec<Label>, until: i64) -> Self {
        let mut pass = Self {
            person: actor_of(person),
            holder: holder.into(),
            lends,
            until,
            sig: String::new(),
        };
        pass.sig = sign_message(person, &pass.message());
        pass
    }

    /// `DOMAIN || person || holder || until[8 BE] || each lend`, every
    /// variable field preceded by its `u32` big-endian length.
    fn message(&self) -> Vec<u8> {
        let mut msg = PASS_DOMAIN.to_vec();
        field(&mut msg, self.person.as_bytes());
        field(&mut msg, self.holder.as_bytes());
        msg.extend_from_slice(&self.until.to_be_bytes());
        for lend in &self.lends {
            field(&mut msg, lend.as_str().as_bytes());
        }
        msg
    }

    /// Whether this pass, presented by `holder` at `now`, opens `lend`.
    pub fn check(&self, holder: &str, lend: &Label, now: i64) -> Result<(), PassFault> {
        if !verify_hex(&self.person, &self.message(), &self.sig) {
            return Err(PassFault::Unsigned);
        }
        if self.holder != holder {
            return Err(PassFault::NotTheHolder);
        }
        if now >= self.until {
            return Err(PassFault::Lapsed);
        }
        if !self.lends.contains(lend) {
            return Err(PassFault::NotNamed);
        }
        Ok(())
    }

    /// The pass as a header carries it: its JSON in base64url.
    pub fn encode(&self) -> String {
        let json = serde_json::to_vec(self).expect("a pass serialises");
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
    }

    /// A pass [`Pass::encode`] wrote, or `None`.
    pub fn decode(raw: &str) -> Option<Self> {
        let json = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(raw.trim())
            .ok()?;
        serde_json::from_slice(&json).ok()
    }
}

#[cfg(test)]
#[path = "pass_tests.rs"]
mod tests;
