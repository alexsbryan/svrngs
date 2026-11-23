// SPDX-License-Identifier: AGPL-3.0-or-later
//! One change row of a set as it travels and is kept, as postcard writes
//! it: the one home of its form, and of a value as the write form names one
//! ([`cell`]). svrngs-db reads them (`parse.rs`, `form.rs`), and a phone's
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
