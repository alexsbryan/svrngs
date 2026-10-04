// SPDX-License-Identifier: AGPL-3.0-or-later
//! A group's seed and its id (DESIGN §8, U2's inventory, the
//! `valid_namespace` row). A group's id was a name the caller typed, so two
//! groups called House on one computer were one directory. Now the id is
//! the hash of the seed — the founder's key, the name the founder goes by,
//! the group's name and 16 random bytes drawn at `new` — so two groups
//! called House are two ids, and a seed carried in a link proves the id it
//! names. The group's name is what a person reads; the id is the directory
//! its record lives in on every computer that holds it.
//!
//! The seed is written beside the group's record as `seed.json`, and the
//! register's seed, `roster.json`, is written from it by [`Seed::roster`]
//! alone: the founder's key bound to the founder's name (DESIGN §6). The
//! register reads `roster.json` as it did in U1; this module reads
//! `seed.json` to find a group by its name.
//!
//! Beside them, `origin` keeps the page this computer prints the group's
//! links on (u2-7): `new` writes the one it was given or the default,
//! `join` the one its link was printed on, and `link --origin` changes it.
//! It is this computer's setting, outside the seed and its id, so a group
//! that serves its page privately never gets a link on the default because
//! one `link` named none (ARCH 6).

use std::path::{Path, PathBuf};

use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use svrngs_node::{namespaces_in, Person, Roster};

use crate::link::Origin;

/// Domain separator for a group id's hash, so an id is never the hash of
/// anything else svrngs hashes (an act id is `svrngs/act/v1`).
const GROUP_ID_DOMAIN: &[u8] = b"svrngs/group/v1\0";

/// The one spelling of the seed's file name in a group's directory.
const SEED_FILE: &str = "seed.json";

/// The one spelling of the kept origin's file name in a group's directory.
const ORIGIN_FILE: &str = "origin";

/// A name a person reads: a group's, or a person's. Trimmed, 1..=64 bytes,
/// no control characters — it is printed in a terminal and carried in a
/// link.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    pub fn parse(raw: &str) -> Option<Self> {
        let name = raw.trim();
        if name.is_empty() || name.len() > 64 || name.chars().any(char::is_control) {
            tracing::debug!(target: "group", raw, "group: not a name — empty, over 64 bytes or with a control character");
            return None;
        }
        Some(Self(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// What a group is founded on. Every field is inside the id's hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seed {
    name: Name,
    founder: Name,
    key: VerifyingKey,
    salt: [u8; 16],
}

impl Seed {
    /// A new group's seed: `founder` names the person holding `key`, and
    /// 16 bytes are drawn so that a second group of the same name by the
    /// same person is a second group.
    pub fn draw(name: Name, founder: Name, key: VerifyingKey) -> Self {
        let mut salt = [0u8; 16];
        // As identity.rs: getrandom failing is a host with no entropy, and
        // no group can be made on it.
        getrandom::fill(&mut salt).expect("failed to draw a group's salt");
        Self::from_parts(name, founder, key, salt)
    }

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
        let mut h = blake3::Hasher::new();
        h.update(GROUP_ID_DOMAIN);
        for field in [
            self.name.as_str().as_bytes(),
            self.founder.as_str().as_bytes(),
        ] {
            h.update(&(field.len() as u32).to_be_bytes());
            h.update(field);
        }
        h.update(self.key.as_bytes());
        h.update(&self.salt);
        h.finalize().to_hex().to_string()
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
                Person::from(self.founder.as_str()),
                vec![self.founder_actor()],
            )]
            .into_iter()
            .collect(),
        )
    }

    /// The seed from its four fields as text — a link's or `seed.json`'s.
    pub fn parse(name: &str, founder: &str, key_hex: &str, salt_hex: &str) -> Option<Self> {
        let key: [u8; 32] = hex::decode(key_hex).ok()?.try_into().ok()?;
        let key = VerifyingKey::from_bytes(&key).ok()?;
        let salt: [u8; 16] = hex::decode(salt_hex).ok()?.try_into().ok()?;
        Some(Self::from_parts(
            Name::parse(name)?,
            Name::parse(founder)?,
            key,
            salt,
        ))
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

/// Why a group's seed or origin could not be written or read.
#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot read {path}: {why}")]
    Read { path: PathBuf, why: String },
}

/// Write `seed` into `dir`, the group's directory, with the register's
/// seed beside it.
pub fn write_seed(dir: &Path, seed: &Seed) -> Result<(), SeedError> {
    let path = dir.join(SEED_FILE);
    let file = SeedFile {
        name: seed.name.0.clone(),
        founder: seed.founder.0.clone(),
        key: seed.founder_actor(),
        salt: seed.salt_hex(),
    };
    let raw = serde_json::to_string_pretty(&file).expect("a seed serializes");
    std::fs::create_dir_all(dir)
        .and_then(|()| std::fs::write(&path, raw))
        .map_err(|source| SeedError::Write { path, source })
}

/// Read the seed in `dir`, or `None` when the directory holds none.
pub fn read_seed(dir: &Path) -> Result<Option<Seed>, SeedError> {
    let path = dir.join(SEED_FILE);
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(SeedError::Read {
                path,
                why: e.to_string(),
            })
        }
    };
    let file: SeedFile = serde_json::from_str(&raw).map_err(|e| SeedError::Read {
        path: path.clone(),
        why: e.to_string(),
    })?;
    Seed::parse(&file.name, &file.founder, &file.key, &file.salt)
        .map(Some)
        .ok_or_else(|| SeedError::Read {
            path,
            why: "a field is not what a seed holds".into(),
        })
}

/// Keep `origin` in `dir`, the group's directory, as the page its links
/// print on from here.
pub fn write_origin(dir: &Path, origin: &Origin) -> Result<(), SeedError> {
    let path = dir.join(ORIGIN_FILE);
    tracing::debug!(target: "group", path = %path.display(), origin = origin.as_str(), "group: origin kept");
    std::fs::create_dir_all(dir)
        .and_then(|()| std::fs::write(&path, origin.as_str()))
        .map_err(|source| SeedError::Write { path, source })
}

/// The origin kept in `dir`, or `None` when the directory keeps none.
pub fn read_origin(dir: &Path) -> Result<Option<Origin>, SeedError> {
    let path = dir.join(ORIGIN_FILE);
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(target: "group", path = %path.display(), "group: no origin kept");
            return Ok(None);
        }
        Err(e) => {
            return Err(SeedError::Read {
                path,
                why: e.to_string(),
            })
        }
    };
    Origin::parse(&raw)
        .map(Some)
        .ok_or_else(|| SeedError::Read {
            path,
            why: "not an http(s) address without a '#'".into(),
        })
}

/// Every group this computer holds a seed for under `root`, the data dir.
/// A directory whose seed does not hash to its name is left out, at warn:
/// its record is not the group its seed names.
pub fn groups(root: &Path) -> Vec<Seed> {
    let namespaces = match namespaces_in(root) {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!(target: "group", error = %e, "group: cannot list the groups held");
            return Vec::new();
        }
    };
    let mut out = Vec::new();
    for ns in namespaces {
        // `namespaces_in` lists only names a journal opens, and opening
        // creates nothing: this is the rail's spelling of the directory.
        let dir = match svrngs_node::RingJournal::open(root, &ns) {
            Ok(journal) => journal.dir().to_path_buf(),
            Err(e) => {
                tracing::warn!(target: "group", namespace = %ns, error = %e, "group: a directory the rail will not open, left out");
                continue;
            }
        };
        match read_seed(&dir) {
            Ok(Some(seed)) if seed.id() == ns => out.push(seed),
            Ok(Some(seed)) => {
                tracing::warn!(target: "group", namespace = %ns, id = %seed.id(), "group: a seed that does not hash to its directory, left out")
            }
            Ok(None) => {
                tracing::debug!(target: "group", namespace = %ns, "group: a directory with no seed, left out")
            }
            Err(e) => {
                tracing::warn!(target: "group", namespace = %ns, error = %e, "group: an unreadable seed, left out")
            }
        }
    }
    out
}

/// Which groups `reference` names: a group's name, case-insensitively,
/// or a prefix of at least 8 of its id's hex — upstream's rule for naming
/// a mesh (`commonwealth-discovery/src/membership.rs:62-68` at
/// `e1a43bcd2`), so two groups of one name can be told apart.
pub fn named<'a>(groups: &'a [Seed], reference: &str) -> Vec<&'a Seed> {
    let needle = reference.trim().to_lowercase();
    groups
        .iter()
        .filter(|s| {
            !needle.is_empty()
                && (s.name.as_str().to_lowercase() == needle
                    || (needle.len() >= 8 && s.id().starts_with(&needle)))
        })
        .collect()
}
