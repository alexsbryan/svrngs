// SPDX-License-Identifier: AGPL-3.0-or-later
//! A group's seed and its id (PLAN §4, U2's inventory, the
//! `valid_namespace` row). A group's id was a name the caller typed, so two
//! groups called House on one computer were one directory. Now the id is
//! the hash of the seed — the founder's key, the name the founder goes by,
//! the group's name and 16 random bytes drawn at `new` — so two groups
//! called House are two ids, and a seed a link's issuer answers proves
//! the id it names ([`answered`]). The group's name is what a person reads; the id is the directory
//! its record lives in on every computer that holds it.
//!
//! Moved from the command's `group.rs` (u4-5; DESIGN §3, call 5), so a
//! browser's page believes the seed an issuer answers with the command's
//! body. Drawing a seed's salt and keeping the seed on disk stay the
//! command's: the core draws nothing and opens no file.

use alloc::string::{String, ToString};
use alloc::vec;

use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

use crate::{field, Name, Person, Roster};

/// What a group is founded on. Every field is inside the id's hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seed {
    name: Name,
    founder: Name,
    key: VerifyingKey,
    salt: [u8; 16],
}

impl Seed {
    pub fn from_parts(name: Name, founder: Name, key: VerifyingKey, salt: [u8; 16]) -> Self {
        Self {
            name,
            founder,
            key,
            salt,
        }
    }

    /// The group's id: the whole BLAKE3 hash, 64 lowercase hex, of
    /// `DOMAIN || name || founder || key || salt`, every variable field
    /// preceded by its `u32` big-endian length. It names the group's
    /// directory, as a namespace (`svrngs_node`'s `valid_namespace`).
    pub fn id(&self) -> String {
        let mut input = crate::sig::GROUP_ID_DOMAIN.to_vec();
        field(&mut input, self.name.as_str().as_bytes());
        field(&mut input, self.founder.as_str().as_bytes());
        input.extend_from_slice(self.key.as_bytes());
        input.extend_from_slice(&self.salt);
        blake3::hash(&input).to_hex().to_string()
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn founder(&self) -> &Name {
        &self.founder
    }

    /// The founder's key, as the record spells an actor.
    pub fn founder_actor(&self) -> String {
        hex::encode(self.key.as_bytes())
    }

    pub fn salt_hex(&self) -> String {
        hex::encode(self.salt)
    }

    /// The register's seed: the founder's key, bound to the founder's name
    /// (DESIGN §6). The one writer of a group's `roster.json`.
    pub fn roster(&self) -> Roster {
        Roster::new(
            [(
                Person::from(self.founder.clone()),
                vec![self.founder_actor()],
            )]
            .into_iter()
            .collect(),
        )
    }

    /// The seed from its four fields as text, as `seed.json` holds them.
    pub fn parse(name: &str, founder: &str, key_hex: &str, salt_hex: &str) -> Option<Self> {
        let key: [u8; 32] = hex::decode(key_hex).ok()?.try_into().ok()?;
        let key = VerifyingKey::from_bytes(&key).ok()?;
        let salt: [u8; 16] = hex::decode(salt_hex).ok()?.try_into().ok()?;
        Some(Self::from_parts(
            Name::kept(name.to_string()).ok()?,
            Name::kept(founder.to_string()).ok()?,
            key,
            salt,
        ))
    }

    /// The seed as `seed.json` holds it.
    pub fn to_text(&self) -> String {
        let file = SeedFile {
            name: self.name.as_str().to_string(),
            founder: self.founder.as_str().to_string(),
            key: self.founder_actor(),
            salt: self.salt_hex(),
        };
        serde_json::to_string_pretty(&file).expect("a seed serializes")
    }

    /// A seed from `seed.json`'s text.
    pub fn from_text(raw: &str) -> Result<Self, String> {
        let file: SeedFile = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        Self::parse(&file.name, &file.founder, &file.key, &file.salt)
            .ok_or_else(|| "a field is not what a seed holds".into())
    }
}

/// `seed.json`'s shape.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedFile {
    name: String,
    founder: String,
    key: String,
    salt: String,
}

/// The seed an issuer answered for `group`, believed only when it hashes
/// to `group`: an issuer answering another group's seed is not believed.
pub fn answered(group: &str, text: &str) -> Option<Seed> {
    let seed = match Seed::from_text(text) {
        Ok(seed) => seed,
        Err(why) => {
            tracing::debug!(target: "group", group, why, "group: refused — the seed answered is not a seed");
            return None;
        }
    };
    if seed.id() != group {
        tracing::debug!(target: "group", group, hashes_to = %seed.id(), "group: refused — the seed answered is another group's");
        return None;
    }
    Some(seed)
}
