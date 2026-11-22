// SPDX-License-Identifier: AGPL-3.0-or-later
//! A set's changes, parsed against this copy's tables before cr-sqlite sees
//! any of them (svrn-docs `rings/current/APP_STATE.md` §4, the four
//! outcomes). Nothing a set carries reaches `crsql_changes`, or cr-sqlite's
//! `unpack_columns`, unparsed: only a [`Valid`] is merged (`sync.rs`,
//! `keep`). A set that doesn't parse is refused for good, and every copy
//! that holds its schema refuses it alike, since the parse reads only the
//! set and the tables that schema made. The write's own seal parses its set
//! with the same parser ([`fit`]), so the writer's copy refuses what every
//! other copy would, and says why ([`Unfit`]).
//!
//! Pure: the bytes and the tables in, the changes or why not out.

use std::collections::BTreeMap;

use rusqlite::types::Value as Sql;
use serde::{Deserialize, Serialize};

use crate::form::Table;

/// A `col_version` or a causal length past this is refused for good: one at
/// `i64::MAX` freezes its cell (svrn-docs
/// `app-state-diligence/engine-and-sync.md`, C3), and every copy reads the
/// bound alike from the set.
const BOUND: i64 = 1 << 53;

/// The `cid` of a row's own change, its insert or delete, as cr-sqlite
/// names it (`vendor/cr-sqlite/core/rs/core/src/c.rs:10-11`).
const SENTINEL: &str = "-1";

/// cr-sqlite's numbers for a packed column's type
/// (`vendor/cr-sqlite/core/rs/sqlite-rs-embedded/sqlite_nostd/src/nostd.rs:221-227`);
/// 5, NULL, is no key's.
const INTEGER: u8 = 1;
const FLOAT: u8 = 2;
const TEXT: u8 = 3;
const BLOB: u8 = 4;

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

/// A set's changes, each one its table can hold: made only by [`fit`].
pub(crate) struct Valid(pub(crate) Vec<Change>);

/// Why a set's changes aren't ones every copy takes, in the words a page is
/// told when its own write would make one.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Unfit {
    #[error("its changes aren't a set's")]
    Unread,
    #[error("it names the table {0}, which the app doesn't have")]
    NoTable(String),
    #[error("it names {0}, which isn't one of that table's columns")]
    NoColumn(String),
    #[error(
        "a key of {0} isn't one cr-sqlite reads back on another computer, as a BLOB key 128 to 255 bytes long isn't: key it by text, or by a BLOB of another length"
    )]
    Key(String),
    #[error(
        "{0} has been changed as many times as one value can be (2^53), so no other computer would take another change to it: write it in a new row instead"
    )]
    Versions(String),
    #[error("{0} is NOT NULL, and this gives it NULL")]
    Null(String),
}

/// `bytes`, a set's changes, as ones `tables` hold, or why not: postcard's
/// whole, with nothing after.
pub(crate) fn parse(bytes: &[u8], tables: &BTreeMap<String, Table>) -> Result<Valid, Unfit> {
    let read = postcard::take_from_bytes::<Vec<Change>>(bytes).ok();
    let changes = read.filter(|(_, rest)| rest.is_empty()).ok_or(Unfit::Unread)?.0;
    fit(changes, tables)
}

/// `changes` as ones `tables` hold, or the first that isn't: a table of the
/// app's; a column of it that isn't a key column, or the sentinel; a key in
/// cr-sqlite's packed form with the table's key columns ([`packed`]); a
/// `col_version` and causal length from 1 to [`BOUND`]; and no NULL for a
/// NOT NULL column, which SQLite would refuse on every copy alike.
pub(crate) fn fit(changes: Vec<Change>, tables: &BTreeMap<String, Table>) -> Result<Valid, Unfit> {
    for c in &changes {
        let t = tables.get(&c.table).ok_or_else(|| Unfit::NoTable(c.table.clone()))?;
        let (named, versions) = (|| format!("{}.{}", c.table, c.cid), 1..=BOUND);
        let column = c.cid == SENTINEL || t.columns.contains(&c.cid) && !t.key.contains(&c.cid);
        let why = match () {
            _ if !column => Unfit::NoColumn(named()),
            _ if packed(&c.pk, t.key.len()).is_none() => Unfit::Key(c.table.clone()),
            _ if !versions.contains(&c.col_version) || !versions.contains(&c.cl) => Unfit::Versions(named()),
            _ if c.val == Sql::Null && t.not_null.contains(&c.cid) => Unfit::Null(named()),
            _ => continue,
        };
        tracing::debug!(target: "state", table = c.table, cid = c.cid, col_version = c.col_version, cl = c.cl, %why, "state: a change no copy takes, so its set is refused for good");
        return Err(why);
    }
    Ok(Valid(changes))
}

/// Whether `key` is a key of `columns` columns exactly as cr-sqlite's
/// `pack_columns` writes one and its `unpack_columns` reads it back
/// (`vendor/cr-sqlite/core/rs/core/src/pack_columns.rs:31-113`, `:125-185`):
/// the count; then each column's type, an INTEGER, FLOAT, TEXT or BLOB and
/// never NULL, with its integer or length in the fewest bytes, as
/// `num_bytes_needed_i64` and `_i32` count them, at most eight; a float's
/// eight bytes; a text's UTF-8, which `unpack_columns` assumes unchecked
/// (`:176`); a blob's bytes, whose length `unpack_columns` reads signed
/// (`get_int`, `:143`), so its top bit is clear; and nothing after.
fn packed(key: &[u8], columns: usize) -> Option<()> {
    let (&count, mut rest) = key.split_first()?;
    (usize::from(count) == columns).then_some(())?;
    for _ in 0..columns {
        let (&tag, after) = rest.split_first()?;
        let (kind, width) = (tag & 7, usize::from(tag >> 3));
        let (head, after) = after.split_at_checked(if kind == FLOAT { 8 } else { width })?;
        let n = head.iter().fold(0u64, |n, b| n << 8 | u64::from(*b));
        let fewest = width <= 8 && width == 8 - n.leading_zeros() as usize / 8;
        let unsigned = kind == TEXT || head.first().is_none_or(|b| *b < 0x80);
        let fits = match kind {
            INTEGER => fewest,
            FLOAT => width == 0,
            TEXT | BLOB => fewest && unsigned && n <= i32::MAX as u64,
            _ => false,
        };
        fits.then_some(())?;
        rest = after;
        if kind == TEXT || kind == BLOB {
            let (bytes, after) = rest.split_at_checked(usize::try_from(n).ok()?)?;
            (kind == BLOB || std::str::from_utf8(bytes).is_ok()).then_some(())?;
            rest = after;
        }
    }
    rest.is_empty().then_some(())
}
