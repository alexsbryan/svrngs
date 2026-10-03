// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/payload.rs@3e6afe1aa
//! [`Payload`] — JSON in the one spelling every copy hashes.
//!
//! An act's id covers `H(body)`, and a `serde_json::Value` has no bytes of its
//! own: which bytes it serializes to depends on serde_json's `preserve_order`
//! feature, which any dependency can switch on. So a `Payload` is canonical by
//! construction — object keys sorted, recursively, on the way in, including
//! on deserialization — and refuses fractions, which have many spellings
//! (`1e2`, `100.0`, `100`). Pick a unit and use a whole number.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

/// The largest canonical payload an act carries. An act is "Ama paid for the
/// pizza", not a file; files go beside the record (DESIGN D0a `group.files`).
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

/// One JSON object, canonical. Construct with [`Payload::new`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload(Value);

/// Why a value cannot be carried. Each names its fix, because an app author
/// reads it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PayloadError {
    #[error("a payload must be a JSON object (try `{{\"kind\": \"...\"}}`)")]
    NotAnObject,
    #[error(
        "a payload may not contain the fractional number {0}: every copy must \
         derive the same bytes from it. Pick a unit and use a whole number \
         (cents, grams, milliseconds)"
    )]
    Fractional(String),
    #[error("this payload is {0} bytes and an act carries at most 65536")]
    TooLarge(usize),
}

impl Payload {
    /// Canonicalize and check one value.
    pub fn new(value: Value) -> Result<Self, PayloadError> {
        if !value.is_object() {
            return Err(PayloadError::NotAnObject);
        }
        let canonical = canonicalize(value)?;
        let bytes = serde_json::to_vec(&canonical).map_or(0, |b| b.len());
        if bytes > MAX_PAYLOAD_BYTES {
            return Err(PayloadError::TooLarge(bytes));
        }
        Ok(Self(canonical))
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }

    /// The canonical bytes: what `H(body)` is taken over.
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap_or_default()
    }
}

/// Objects keyed in sorted order, recursively; array order is data and stays.
fn canonicalize(value: Value) -> Result<Value, PayloadError> {
    Ok(match value {
        Value::Object(mut map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            // Inserted in sorted order so a `Map` backed by an IndexMap
            // (`preserve_order`) iterates as a BTreeMap-backed one does.
            let mut out = Map::new();
            for k in keys {
                let v = map.remove(&k).unwrap_or(Value::Null);
                out.insert(k, canonicalize(v)?);
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonicalize).collect::<Result<Vec<_>, _>>()?),
        Value::Number(n) if n.is_f64() => return Err(PayloadError::Fractional(n.to_string())),
        other => other,
    })
}

impl Serialize for Payload {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

/// The same rules as [`Payload::new`], so a line read back canonicalizes to
/// the bytes its id was derived from.
impl<'de> Deserialize<'de> for Payload {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Payload::new(Value::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
