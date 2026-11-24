// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned rows (svrn-docs `rings/current/APP_STATE.md` §4, "Who may write
//! which row"; §7, invariant 13): a table whose key begins with
//! `person_key` keeps each row its person's ([`Table::owned`]). Who may
//! change one is authority, so Eudoxia decides it, a pure function
//! (`eudoxia_community::db::row`); svrngs states the record it judges by,
//! [`Binds`], the set's signer, and what each change does to its row. One
//! call at each place a set is judged: at the write, as this computer's
//! person, before it signs ([`mine`], `Db::seal`), and at apply on every
//! copy, by the signer's binding ([`kept`], `Db::take`), so the writer's
//! copy never takes what another refuses. A delete of a person's row by
//! anyone else is taken once the office has cut every key of theirs; until
//! this copy's record holds the cut, it is dropped and sent again, as a set
//! of a key not bound yet is.

use std::collections::BTreeMap;

use rusqlite::types::Value as Sql;

use eudoxia_community::db::{self, Change as Does, Key, Lacks, Type, Verdict};

use crate::form::Table;
use crate::parse::{Change, SENTINEL, Valid};
use crate::sync::{Binds, Device, Kept};
use crate::{Refused, fault};

/// The app's tables, by name.
type Tables = BTreeMap<String, Table>;

/// The first value of a change's key as the parse unpacked it
/// ([`Valid`]), its type and, for TEXT or a BLOB, its bytes.
fn first(key: &[Sql]) -> Key<'_> {
    let (kind, bytes) = match key.first() {
        Some(Sql::Integer(_)) => (Type::Integer, &[][..]),
        Some(Sql::Real(_)) => (Type::Real, &[][..]),
        Some(Sql::Text(t)) => (Type::Text, t.as_bytes()),
        Some(Sql::Blob(b)) => (Type::Blob, &b[..]),
        _ => (Type::Null, &[][..]),
    };
    Key { kind, bytes }
}

/// What a change does to its row, as Eudoxia reads it: a row's own change
/// at an even causal length deletes it; any other writes it.
fn does(c: &Change) -> Does {
    match c.cid == SENTINEL && c.cl % 2 == 0 {
        true => Does::Delete,
        false => Does::Write,
    }
}

/// Eudoxia's verdict on the first change of `valid` that `signer`'s set
/// may not make to an owned row of `tables`, by the record `binds`, a
/// refusal before a wait, and the table it is in; none when it may make
/// each.
fn stopped<'c>(valid: &'c Valid, tables: &Tables, binds: &Binds, signer: &Device) -> Option<(Verdict, &'c str)> {
    let owned = (valid.0.iter().zip(&valid.1)).filter(|(c, _)| tables.get(&c.table).is_some_and(Table::owned));
    let judged = owned.map(|(c, key)| (db::row(binds, signer.as_str(), first(key), does(c)), c.table.as_str()));
    let stopped = (judged.filter(|(v, _)| *v != Verdict::Taken)).min_by_key(|(v, _)| matches!(v, Verdict::Waits(_)));
    tracing::debug!(target: "state", %signer, ?stopped, "state: a set's changes to owned rows judged by Eudoxia's rule, by the record's binding of its signer: each taken, or the first refused or waiting");
    stopped
}

/// Whether this computer, `signer`, may write `made`, its write's set, by
/// the record `binds`: refused naming the table, in words for the page.
pub(crate) fn mine(made: &Valid, tables: &Tables, binds: &Binds, signer: &Device) -> Result<(), Refused> {
    match stopped(made, tables, binds, signer) {
        None => Ok(()),
        Some((Verdict::Refused(_), table)) => Err(Refused::NotYours(table.into())),
        Some((Verdict::Waits(Lacks::Removal), table)) => Err(Refused::NotYetGone(table.into())),
        Some(_) => Err(fault("this computer's key isn't in your group's record here yet")),
    }
}

/// `judged`, a set `signer` signed as the parse left it, as a copy keeps it
/// at apply by the record `binds`: taken, or refused for good; none while
/// it waits for the office's cut of a row's person, which the record's
/// exchange, first in every dial, brings before or with it.
pub(crate) fn kept(
    judged: Result<Valid, Kept>,
    tables: &Tables,
    binds: &Binds,
    signer: &Device,
) -> Option<Result<Valid, Kept>> {
    let Ok(valid) = judged else {
        return Some(judged);
    };
    match stopped(&valid, tables, binds, signer) {
        None => Some(Ok(valid)),
        Some((Verdict::Refused(_), _)) => Some(Err(Kept::Refused)),
        Some(_) => None,
    }
}
