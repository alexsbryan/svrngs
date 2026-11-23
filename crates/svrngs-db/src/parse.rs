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

use crate::form::Table;

#[path = "change.rs"]
mod change;
pub(crate) use change::{BOUND, Change, cell, unpack};

/// The `cid` of a row's own change, its insert or delete, as cr-sqlite
/// names it (`vendor/cr-sqlite/core/rs/core/src/c.rs:10-11`).
pub(crate) const SENTINEL: &str = "-1";

/// A set's changes, each one its table can hold, and each one's key as
/// [`change::unpack`] read it, in their order: made only by [`fit`].
pub(crate) struct Valid(pub(crate) Vec<Change>, pub(crate) Vec<Vec<Sql>>);

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
/// cr-sqlite's packed form ([`unpack`]) with the table's key columns; a
/// `col_version` and causal length from 1 to [`BOUND`]; and no NULL for a
/// NOT NULL column, which SQLite would refuse on every copy alike.
pub(crate) fn fit(changes: Vec<Change>, tables: &BTreeMap<String, Table>) -> Result<Valid, Unfit> {
    let mut keys = Vec::with_capacity(changes.len());
    for c in &changes {
        let t = tables.get(&c.table).ok_or_else(|| Unfit::NoTable(c.table.clone()))?;
        let (named, versions) = (|| format!("{}.{}", c.table, c.cid), 1..=BOUND);
        let column = c.cid == SENTINEL || t.columns.contains(&c.cid) && !t.key.contains(&c.cid);
        let key = unpack(&c.pk).filter(|k| k.len() == t.key.len());
        let why = match key {
            _ if !column => Unfit::NoColumn(named()),
            None => Unfit::Key(c.table.clone()),
            _ if !versions.contains(&c.col_version) || !versions.contains(&c.cl) => Unfit::Versions(named()),
            _ if c.val == Sql::Null && t.not_null.contains(&c.cid) => Unfit::Null(named()),
            Some(key) => {
                keys.push(key);
                continue;
            }
        };
        tracing::debug!(target: "state", table = c.table, cid = c.cid, col_version = c.col_version, cl = c.cl, %why, "state: a change no copy takes, so its set is refused for good");
        return Err(why);
    }
    Ok(Valid(changes, keys))
}
