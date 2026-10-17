// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai kernel-types/src/hash.rs@e1a43bcd2
//! [`ContentHash`] — identity from essence (ARCH §7.5).
//!
//! # Why this type exists
//!
//! Measured 2026-08-20 across the workspace: content hashes were carried as a
//! bare `String` in FOUR mutually incompatible encodings — full 64-hex BLAKE3,
//! 16-hex-prefix BLAKE3, 16-hex-prefix SHA-256, and `sha256:`-prefixed 64-hex
//! — produced by TWO algorithms. A field named `content_hash` in one crate
//! could not be safely compared against one from another, and nothing in the
//! type system said so. Alongside that: `short_hash` was implemented verbatim
//! three times (`enrichment/atlas/atoms.rs`, `enrichment/governance.rs`,
//! `enrichment/code_intel/mod.rs`), each with a comment naming the others.
//!
//! This is the one decider (ARCH §10.6). One algorithm, one encoding, one
//! implementation of the truncated form.
//!
//! # The algorithm is not a parameter
//!
//! `ContentHash` is **always BLAKE3-256**. There is deliberately no algorithm
//! field and no `Sha256` variant: a hash whose algorithm is data is a hash two
//! values of which may be equal-as-bytes and different-as-content, which is
//! precisely the defect above wearing a type. Content hashed under SHA-256
//! elsewhere in the tree is a different value space and must not be converted
//! into this type — rehash the content instead.

use alloc::format;
use alloc::string::String;
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The BLAKE3-256 digest of some content — its identity, derived from what it
/// IS rather than from where it sits or when it arrived (ARCH §7.5: identity
/// from essence, never a counter or an address).
///
/// # Wire form
///
/// Serializes as a 64-character lowercase hex **string**, not as a byte array.
/// That is deliberate and load-bearing: the tree already persists this value
/// as `content_hash TEXT` in SQLite and as `content_hash: String` in serde
/// rows, so a stored row round-trips into this type without a migration.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Hash content. **The** implementation — every other content-hash site in
    /// the workspace should reach this one rather than calling `blake3`
    /// directly, so the encoding cannot diverge again.
    pub fn of(content: &[u8]) -> Self {
        ContentHash(*blake3::hash(content).as_bytes())
    }

    /// Hash text. Convenience for the overwhelmingly common case; identical to
    /// `ContentHash::of(s.as_bytes())`.
    pub fn of_str(s: &str) -> Self {
        ContentHash::of(s.as_bytes())
    }

    /// Adopt a digest computed elsewhere. Named `from_bytes` rather than
    /// `new` so a reader sees that no hashing happened here and asks where it
    /// did.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        ContentHash(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Full 64-character lowercase hex — the canonical wire and storage form.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse the canonical form. `None` on anything that is not exactly 64 hex
    /// characters — a truncated `short()` value is REFUSED rather than
    /// zero-extended, because silently accepting it would re-create the
    /// compare-across-encodings defect this type exists to remove
    /// (principle 6: absence is reported, never defaulted).
    pub fn from_hex(s: &str) -> Option<Self> {
        let arr: [u8; 32] = hex::decode(s.trim()).ok()?.try_into().ok()?;
        Some(ContentHash(arr))
    }

    /// The 16-character prefix — 64-bit truncation, safe for <10M items per
    /// corpus by the birthday bound. This is the ONE implementation of a
    /// convention that existed verbatim in three modules.
    ///
    /// A `short()` value is a display and bucketing key, never an identity:
    /// it does not parse back via [`from_hex`](Self::from_hex), on purpose.
    pub fn short(&self) -> String {
        hex::encode(&self.0[..8])
    }
}

impl fmt::Debug for ContentHash {
    /// Truncated for humans; [`to_hex`](ContentHash::to_hex) is the full value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", self.short())
    }
}

impl Serialize for ContentHash {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        ContentHash::from_hex(&s).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "expected 64 hex characters (BLAKE3-256), got {:?}",
                s
            ))
        })
    }
}

#[cfg(test)]
#[path = "hash_tests.rs"]
mod tests;
