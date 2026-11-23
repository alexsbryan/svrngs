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
    #[error(
        "{0} takes only a value of its declared type, which every computer keeps as written: write it as that type, or declare the column TEXT, INTEGER, REAL or BLOB to fit what it holds"
    )]
    Kind(String),
}

/// What a column keeps a value as it is given: its declared type's
/// affinity, or a STRICT table's type (sqlite.org `datatype3.html` §3.1,
/// `stricttables.html`), read from the schema every copy holds. A value of
/// another type is converted where it is kept, or refused in a STRICT
/// table: a key kept so names one row by two keys, as cr-sqlite keys its
/// clocks by the value the set carries, and a value compared with the one
/// kept differs by the order sets come in. So a value only of its column's
/// kind is taken ([`fit`]), and copies holding the same sets end alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Integer,
    Real,
    Text,
    Blob,
    Any,
}

impl Kind {
    /// The kind of a column declared `declared`, in a STRICT table where
    /// `strict`: by SQLite's rules in their order, and NUMERIC's, as
    /// `DATE` or `BOOLEAN` are, as an integer, the one type it keeps as
    /// given whatever its value.
    pub(crate) fn of(declared: &str, strict: bool) -> Self {
        let t = declared.to_ascii_uppercase();
        let has = |words: &[&str]| words.iter().any(|w| t.contains(w));
        match () {
            _ if strict && t == "ANY" => Self::Any,
            _ if has(&["INT"]) => Self::Integer,
            _ if has(&["CHAR", "CLOB", "TEXT"]) => Self::Text,
            _ if strict && t == "BLOB" => Self::Blob,
            _ if t.is_empty() || has(&["BLOB"]) => Self::Any,
            _ if has(&["REAL", "FLOA", "DOUB"]) => Self::Real,
            _ => Self::Integer,
        }
    }

    /// Whether it keeps `v` as given, a key's value where `key`, in a table
    /// STRICT where `strict`: NULL, and any value of its own type; a column
    /// of no type any value, and one not STRICT a BLOB, which no affinity
    /// converts; and an integer a REAL column, but for a key, as the real it
    /// equals, which compares equal to it. A float that is no number SQLite
    /// keeps as NULL.
    fn keeps(self, v: &Sql, (key, strict): (bool, bool)) -> bool {
        match (self, v) {
            (_, Sql::Real(r)) if r.is_nan() => false,
            (_, Sql::Null) | (Self::Any, _) => true,
            (Self::Integer, Sql::Integer(_)) | (Self::Real, Sql::Real(_)) => true,
            (Self::Text, Sql::Text(_)) | (Self::Blob, Sql::Blob(_)) => true,
            (_, Sql::Blob(_)) => !strict,
            (Self::Real, Sql::Integer(i)) => !key && i.unsigned_abs() <= 1 << 53,
            _ => false,
        }
    }
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
/// cr-sqlite's packed form ([`unpack`]) with the table's key columns; each
/// value, of the key and of a column, one its column keeps ([`Kind`]); a
/// `col_version` and causal length from 1 to [`BOUND`], a column's causal
/// length odd, since cr-sqlite merges a change at an even one as its row's
/// delete and no write makes one, so only a row's own change deletes it;
/// and no NULL for a NOT NULL column, which SQLite would refuse on every
/// copy alike.
pub(crate) fn fit(changes: Vec<Change>, tables: &BTreeMap<String, Table>) -> Result<Valid, Unfit> {
    let mut keys = Vec::with_capacity(changes.len());
    for c in &changes {
        let t = tables.get(&c.table).ok_or_else(|| Unfit::NoTable(c.table.clone()))?;
        let (named, versions) = (|| format!("{}.{}", c.table, c.cid), 1..=BOUND);
        let column = c.cid == SENTINEL || t.columns.contains(&c.cid) && !t.key.contains(&c.cid);
        let key = unpack(&c.pk).filter(|k| k.len() == t.key.len());
        let kept = |col: &String, v: &Sql, key: bool| t.kinds.get(col).is_some_and(|k| k.keeps(v, (key, t.strict)));
        let keyed = key
            .iter()
            .flat_map(|k| t.key.iter().zip(k))
            .map(|(col, v)| (col, v, true));
        let value = (c.cid != SENTINEL).then_some((&c.cid, &c.val, false));
        let wrong = keyed.chain(value).find(|(col, v, key)| !kept(col, v, *key));
        let why = match key {
            _ if !column => Unfit::NoColumn(named()),
            None => Unfit::Key(c.table.clone()),
            _ if let Some((col, ..)) = wrong => Unfit::Kind(format!("{}.{col}", c.table)),
            _ if !versions.contains(&c.col_version) || !versions.contains(&c.cl) => Unfit::Versions(named()),
            _ if c.cid != SENTINEL && c.cl % 2 == 0 => Unfit::Unread,
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
