// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/payload.rs@e1a43bcd2
//! [`Payload`] — the app's own bytes, in the one spelling every node signs.
//!
//! # Why the rail needs a type here at all
//!
//! The rail does not read inside a payload. It carries it, signs it, orders
//! it and hands it back. So the obvious move is `serde_json::Value` and no
//! type — and that move has a defect that would not surface for months.
//!
//! A signature covers *bytes*. `Value` does not have bytes; it has a
//! serializer, and which bytes that serializer emits for a given value
//! depends on a Cargo feature — `serde_json/preserve_order`, which is ON in
//! this workspace (something in the dependency graph enables it) and which
//! any crate added later can turn off. With it, an object round-trips in
//! insertion order; without it, in sorted order. Flip that feature and every
//! signature in every ring on the mesh stops verifying at once, and the
//! symptom is not "the feature changed", it is a journal that has become all
//! [`BadSignature`](crate::RailGap::BadSignature).
//!
//! A typed body did not have this problem: serde writes struct fields in
//! declaration order regardless. Making the body opaque is what introduces
//! it, so the fix belongs here, in the type that makes the body opaque.
//!
//! **So a `Payload` is canonical by construction.** Objects are rebuilt with
//! their keys in sorted order, recursively, on the way in — including on
//! deserialization, so a line read off the journal and a body read off the
//! wire are canonical too, not just the ones this node authored. The
//! serializer's choice stops mattering, because for a canonical value both
//! choices emit the same bytes.
//!
//! # No floating-point numbers, and that is a real restriction
//!
//! Sorting keys fixes objects. Numbers have the same problem one level
//! deeper: `1e2`, `100.0` and `100` are one value with three spellings, and
//! which one comes back out is `serde_json`'s formatting choice rather than
//! anything the data says. Integers have exactly one spelling. Floats do not.
//!
//! On a rail whose entire purpose is that nineteen laptops derive identical
//! bytes from identical facts — for years, across library upgrades — that is
//! not a risk worth carrying for the convenience of writing `3.5`. So a
//! payload may not contain one, and the refusal says what to write instead:
//! pick a unit and use an integer (cents, grams, micro-degrees, milliseconds).
//! The reference expense app is denominated in cents for exactly this reason.
//!
//! # What is NOT checked here
//!
//! Anything about what the payload *means*. A payload that is well-formed and
//! canonical but says something absurd — a negative amount, a borrower who is
//! not in the house — is the app's to judge, in the app's vocabulary. The
//! rail has no opinion and cannot have one; see [`crate::admit`].

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

/// The largest canonical payload the rail will carry.
///
/// A journal line is an *act* — "bo paid for beer", "alex borrowed the
/// drill". At 64 KiB it has stopped being an act and become a file, and the
/// rail is the wrong place to put a file: every op on this journal is
/// replicated to every peer in the ring, forever, and re-read on every fold.
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

/// One app-authored act, canonical and opaque.
///
/// Construct with [`Payload::new`], which refuses what cannot have a stable
/// canonical form. Deserialization applies the same rules, so a `Payload` in
/// hand — from a journal line, from a peer, from this node's own door — is
/// always one whose bytes every node agrees on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload(Value);

/// Why a payload cannot go on the rail. Every variant renders as a sentence
/// naming the fix, because this text reaches an app author as a 422 body.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PayloadError {
    #[error(
        "a rail payload must be a JSON object — the rail carries acts, and an \
         act needs a name for what it is (try `{{\"kind\": \"...\"}}`)"
    )]
    NotAnObject,
    #[error(
        "a rail payload may not contain the fractional number {0} — two nodes \
         must derive identical bytes from it and JSON does not promise that \
         for fractions. Pick a unit and use a whole number (cents, grams, \
         milliseconds)"
    )]
    Fractional(String),
    #[error(
        "this payload is {0} bytes and the rail carries at most 65536 — a \
         journal line is an act, not a file, and every peer in the ring keeps \
         a copy of it forever"
    )]
    TooLarge(usize),
}

impl Payload {
    /// Canonicalize and check one app-authored value.
    pub fn new(value: Value) -> Result<Self, PayloadError> {
        if !value.is_object() {
            return Err(PayloadError::NotAnObject);
        }
        let canonical = canonicalize(value)?;
        let bytes = serde_json::to_string(&canonical).map_or(0, |s| s.len());
        if bytes > MAX_PAYLOAD_BYTES {
            return Err(PayloadError::TooLarge(bytes));
        }
        Ok(Self(canonical))
    }

    /// The app's value. Read it; never rebuild a `Payload` around a mutated
    /// copy without going back through [`Payload::new`].
    pub fn as_value(&self) -> &Value {
        &self.0
    }
}

/// Rebuild `value` in its one canonical spelling: objects keyed in sorted
/// order, recursively. Array order is data and is left alone.
fn canonicalize(value: Value) -> Result<Value, PayloadError> {
    Ok(match value {
        Value::Object(mut map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            // Inserted in sorted order so that a `Map` backed by an IndexMap
            // (`preserve_order`, which is on here) iterates in the same order
            // as one backed by a BTreeMap. That equivalence is the whole
            // point — see the module docs.
            let mut out = Map::new();
            for k in keys {
                let v = map.remove(&k).unwrap_or(Value::Null);
                out.insert(k, canonicalize(v)?);
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(canonicalize)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Number(n) => {
            if n.is_f64() {
                return Err(PayloadError::Fractional(n.to_string()));
            }
            Value::Number(n)
        }
        other => other,
    })
}

impl Serialize for Payload {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

/// Deserialization runs the same rules as [`Payload::new`], on purpose.
///
/// A journal line whose payload is not canonical would re-serialize to bytes
/// other than the ones its signature covers, and would be reported as a bad
/// signature rather than as what it is. Canonicalizing here means a line
/// written by a correct peer always verifies, and a line that cannot be
/// canonicalized at all is a [`MalformedLine`](crate::RailGap::MalformedLine)
/// — which is exactly what it is.
impl<'de> Deserialize<'de> for Payload {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        Payload::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
#[path = "payload_tests.rs"]
mod tests;
