// SPDX-License-Identifier: AGPL-3.0-or-later
//! One change row of a set as it travels and is kept, as postcard writes
//! it: the one home of its form, of a value as the write form names one
//! ([`cell`]), and of a key as cr-sqlite packs one ([`pack`], [`unpack`]).
//! svrngs-db reads them (`parse.rs`, `form.rs`, `owned.rs`), and a phone's
//! signer compiles this file too (`web/src/set.rs`, by `#[path]`), so what a
//! phone checks before it signs is read as every copy reads it. Pure, of
//! serde, serde_json and base64 alone, so it builds for the page as well:
//! the module that declares it names `Sql`, the value a cell holds there,
//! rusqlite's here.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use super::Sql;

/// One change row of a set, as `crsql_changes` carries it, less the site
/// and version its set names once.
#[derive(Serialize, Deserialize)]
pub(crate) struct Change {
    pub(crate) table: String,
    pub(crate) pk: Vec<u8>,
    pub(crate) cid: String,
    #[serde(with = "Value")]
    pub(crate) val: Sql,
    pub(crate) col_version: i64,
    pub(crate) cl: i64,
    pub(crate) seq: i64,
}

/// A `col_version` or a causal length past this is refused for good: one at
/// `i64::MAX` freezes its cell (svrn-docs
/// `app-state-diligence/engine-and-sync.md`, C3), and every copy reads the
/// bound alike from the set; a phone signs none past its half
/// (`web/src/set.rs`).
pub(crate) const BOUND: i64 = 1 << 53;

/// A cell's value, as postcard writes rusqlite's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Sql")]
pub(crate) enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

/// `v`, a value of the write form (svrn-docs `rings/current/APP_STATE.md`
/// §4, "The write form"), as a cell holds it: null, a number, a string, or
/// `{base64}`, its one key, for a BLOB; none for anything else.
pub(crate) fn cell(v: &serde_json::Value) -> Option<Sql> {
    use serde_json::Value as Json;
    Some(match v {
        Json::Null => Sql::Null,
        Json::String(s) => Sql::Text(s.clone()),
        Json::Number(n) if n.is_i64() => Sql::Integer(n.as_i64()?),
        Json::Number(n) => Sql::Real(n.as_f64().unwrap_or_default()),
        Json::Object(o) if o.len() == 1 => Sql::Blob(STANDARD.decode(o.get("base64")?.as_str()?).ok()?),
        _ => return None,
    })
}

/// cr-sqlite's numbers for a packed column's type
/// (`vendor/cr-sqlite/core/rs/sqlite-rs-embedded/sqlite_nostd/src/nostd.rs:221-227`);
/// 5, NULL, is no key's.
const INTEGER: u8 = 1;
const FLOAT: u8 = 2;
const TEXT: u8 = 3;
const BLOB: u8 = 4;

/// `values`, a key, as cr-sqlite's `pack_columns` packs one
/// (`vendor/cr-sqlite/core/rs/core/src/pack_columns.rs:31-113`): the count;
/// then each value's type, with its integer or length in the fewest bytes,
/// as `num_bytes_needed_i64` and `_i32` count them, a length no longer than
/// an `i32` holds; a float's eight bytes; a text's or blob's bytes. None
/// for NULL, which no key holds, or past 255 values.
pub(crate) fn pack(values: &[Sql]) -> Option<Vec<u8>> {
    let mut out = vec![u8::try_from(values.len()).ok()?];
    for v in values {
        let (kind, n, bytes) = match v {
            Sql::Integer(i) => (INTEGER, *i as u64, &[][..]),
            Sql::Real(r) => (FLOAT, 0, &r.to_be_bytes()[..]),
            Sql::Text(t) => (TEXT, t.len() as u64, t.as_bytes()),
            Sql::Blob(b) => (BLOB, b.len() as u64, &b[..]),
            Sql::Null => return None,
        };
        (kind == INTEGER || n <= i32::MAX as u64).then_some(())?;
        let width = 8 - n.leading_zeros() as usize / 8;
        out.push(kind | (width as u8) << 3);
        out.extend_from_slice(&n.to_be_bytes()[8 - width..]);
        out.extend_from_slice(bytes);
    }
    Some(out)
}

/// `key`'s values, as cr-sqlite's `unpack_columns` reads them back
/// (`pack_columns.rs:125-185`), where it is a key [`pack`] writes, byte for
/// byte, so nothing but its values can differ between two copies' reading
/// of it; none where it isn't, or where `unpack_columns` would read it
/// otherwise: a blob's length it reads signed (`get_int`, `:143`), so one
/// whose top bit is set, and a text that isn't UTF-8, which it assumes
/// unchecked (`:176`).
pub(crate) fn unpack(key: &[u8]) -> Option<Vec<Sql>> {
    let (&count, mut rest) = key.split_first()?;
    let mut values = Vec::new();
    for _ in 0..count {
        let (&tag, after) = rest.split_first()?;
        let (kind, width) = (tag & 7, usize::from(tag >> 3));
        let (head, after) = after.split_at_checked(if kind == FLOAT { 8 } else { width })?;
        let n = head.iter().fold(0u64, |n, b| n << 8 | u64::from(*b));
        let len = if kind == TEXT || kind == BLOB { n } else { 0 };
        let (bytes, after) = after.split_at_checked(usize::try_from(len).ok()?)?;
        values.push(match kind {
            INTEGER => Sql::Integer(n as i64),
            FLOAT => Sql::Real(f64::from_be_bytes(head.try_into().ok()?)),
            TEXT => Sql::Text(String::from_utf8(bytes.to_vec()).ok()?),
            BLOB if head.first().is_none_or(|b| *b < 0x80) => Sql::Blob(bytes.to_vec()),
            _ => return None,
        });
        rest = after;
    }
    (pack(&values)? == key).then_some(values)
}
