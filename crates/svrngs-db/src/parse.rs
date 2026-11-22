// SPDX-License-Identifier: AGPL-3.0-or-later
//! A set's changes as they travel and are kept: each change row, and the
//! bound on its versions that every copy reads alike from the set
//! (svrn-docs `rings/current/APP_STATE.md` §4, the four outcomes).

use rusqlite::types::Value as Sql;
use serde::{Deserialize, Serialize};

/// A `col_version` or a causal length past this is refused for good: one at
/// `i64::MAX` freezes its cell (svrn-docs
/// `app-state-diligence/engine-and-sync.md`, C3), and every copy reads the
/// bound alike from the set.
pub(crate) const BOUND: i64 = 1 << 53;

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

/// rusqlite's value, as postcard writes it.
#[derive(Serialize, Deserialize)]
#[serde(remote = "Sql")]
enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}
