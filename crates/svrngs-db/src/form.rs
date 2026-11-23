// SPDX-License-Identifier: AGPL-3.0-or-later
//! The write form (svrn-docs `rings/current/APP_STATE.md` §4, "The write
//! form"): a page's write is a list of ops, each naming its row by key, and
//! never SQL. Each op is made one statement from the names
//! `pragma_table_info` gives its table, with every value bound, so a name
//! the table lacks never reaches SQLite, and no write runs a condition or
//! computes a value that would hold only on the copy that ran it (svrn-docs
//! `app-state-diligence/app-fit.md`, N2). Pure: the tables in, the
//! statements out, or why not in words that name the form.

use std::collections::BTreeMap;
use std::convert::identity;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rusqlite::types::Value as Sql;
use serde_json::Value;

use crate::Refused;

/// The form, as every refusal that names it ends.
pub const FORM: &str = "A write goes as {write: [op, ...]}, each op one of \
    {insert: <table>, row: {<column>: <value>, ...}}, \
    {update: <table>, key: {<every key column>: <value>}, set: {<column>: <value>, ...}} \
    or {delete: <table>, key: {<every key column>: <value>}}, \
    each value null, a number, a string or {base64: \"...\"}.";

/// A table as `pragma_table_info` names it: its columns, its key's in the
/// key's order, and those that are NOT NULL.
#[derive(Debug, Default)]
pub struct Table {
    pub columns: Vec<String>,
    pub key: Vec<String>,
    pub not_null: Vec<String>,
}

impl Table {
    /// Whether its rows are each their person's: its key's first column is
    /// `person_key` (svrn-docs APP_STATE §4, "Who may write which row").
    pub fn owned(&self) -> bool {
        self.key.first().is_some_and(|k| k.eq_ignore_ascii_case("person_key"))
    }
}

/// One op as one statement, and the values it binds.
pub type Statement = (String, Vec<Sql>);

/// Columns named, each by its table's own name for it, to bound values.
type Named<'t> = Vec<(&'t String, Sql)>;

/// `v` as SQLite binds it: null, a number, a string, or `{base64}` for a
/// BLOB; refused naming `at` for anything else.
pub fn value(v: &Value, at: &str) -> Result<Sql, Refused> {
    let blob = v
        .as_object()
        .filter(|o| o.len() == 1)
        .and_then(|o| o.get("base64")?.as_str());
    match (v, blob.map(|b| STANDARD.decode(b))) {
        (Value::Null, _) => Ok(Sql::Null),
        (Value::String(s), _) => Ok(Sql::Text(s.clone())),
        (Value::Number(n), _) => Ok(n
            .as_i64()
            .map_or_else(|| Sql::Real(n.as_f64().unwrap_or_default()), Sql::Integer)),
        (_, Some(Ok(bytes))) => Ok(Sql::Blob(bytes)),
        _ => Err(Refused::Form(format!(
            "{at} is {v}: a value is null, a number, a string, or {{base64: \"...\"}} for a BLOB."
        ))),
    }
}

/// `named`, columns of the table `name` to values; refused for a column
/// the table lacks.
fn columns<'t>(named: &Value, (name, table): (&str, &'t Table), at: &str) -> Result<Named<'t>, Refused> {
    let named = named.as_object();
    let named = named.ok_or_else(|| Refused::Form(format!("{at} is not an object of columns to values.")))?;
    let mut out = Vec::new();
    for (c, v) in named {
        let Some(c) = table.columns.iter().find(|t| *t == c) else {
            let all = table.columns.join(", ");
            return Err(Refused::Form(format!(
                "{at} names {c:?}, but {name} has no such column; its columns are {all}."
            )));
        };
        out.push((c, value(v, &format!("{at}'s {c}"))?));
    }
    Ok(out)
}

/// `name` as SQL quotes it.
fn quoted(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Each of `names` quoted and spelled by `each`, joined by `by`.
fn listed<'n>(names: impl IntoIterator<Item = &'n String>, each: impl Fn(String) -> String, by: &str) -> String {
    names.into_iter().map(|n| each(quoted(n))).collect::<Vec<_>>().join(by)
}

/// `op`, the `at`th of a write, as one statement on `tables`; refused in
/// words that say what to change.
pub fn statement(op: &Value, tables: &BTreeMap<String, Table>, at: usize) -> Result<Statement, Refused> {
    let (at, o) = (format!("write[{at}]"), op.as_object().cloned().unwrap_or_default());
    let kind = ["insert", "update", "delete"].into_iter().find(|k| o.contains_key(*k));
    let fields: &[&str] = match kind {
        Some("insert") => &["insert", "row"],
        Some("update") => &["update", "key", "set"],
        _ => &["delete", "key"],
    };
    if kind.is_none() || o.len() != fields.len() || !fields.iter().all(|f| o.contains_key(*f)) {
        return Err(Refused::Form(format!("{at} is {op}. {FORM}")));
    }
    let name = o[fields[0]].as_str().unwrap_or_default();
    let Some(table) = tables.get(name) else {
        let all = tables.keys().cloned().collect::<Vec<_>>().join(", ");
        let words = format!(
            "{at} names the table {}, but there is no such table; the tables are {all}.",
            o[fields[0]]
        );
        return Err(Refused::Form(words));
    };
    let (it, keyed) = ((name, table), |c: &String| table.key.contains(c));
    let into = quoted(name);
    if kind == Some("insert") {
        let row = columns(&o["row"], it, &format!("{at}'s row"))?;
        let given = |c: &String| row.iter().find(|(n, _)| *n == c).map(|(_, v)| v.clone());
        if let Some(k) = table.key.iter().find(|k| given(k).is_none()) {
            let key = table.key.join(", ");
            return Err(Refused::Form(format!(
                "{at}'s row leaves out {k}: an insert names every key column of {name}, {key}."
            )));
        }
        // Every column, NULL where the op names none, so a DEFAULT never
        // applies; on a key that has a row, only those it names are set.
        let set = listed(
            row.iter().map(|(c, _)| *c).filter(|c| !keyed(c)),
            |c| format!("{c} = excluded.{c}"),
            ", ",
        );
        let upsert = if set.is_empty() {
            "NOTHING".into()
        } else {
            format!("UPDATE SET {set}")
        };
        let (all, holes, key) = (
            listed(&table.columns, identity, ", "),
            vec!["?"; table.columns.len()],
            listed(&table.key, identity, ", "),
        );
        let sql = format!(
            "INSERT INTO {into} ({all}) VALUES ({}) ON CONFLICT ({key}) DO {upsert}",
            holes.join(", ")
        );
        return Ok((
            sql,
            table.columns.iter().map(|c| given(c).unwrap_or(Sql::Null)).collect(),
        ));
    }
    let key = columns(&o["key"], it, &format!("{at}'s key"))?;
    if key.len() != table.key.len() || !key.iter().all(|(c, _)| keyed(c)) {
        let (named, all) = (
            key.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>().join(", "),
            table.key.join(", "),
        );
        return Err(Refused::Form(format!(
            "{at}'s key names {named}: a key of {name} names every key column, {all}, and no other."
        )));
    }
    let matched = listed(key.iter().map(|(c, _)| *c), |c| format!("{c} = ?"), " AND ");
    if kind == Some("delete") {
        return Ok((
            format!("DELETE FROM {into} WHERE {matched}"),
            key.into_iter().map(|(_, v)| v).collect(),
        ));
    }
    let set = columns(&o["set"], it, &format!("{at}'s set"))?;
    if let Some((c, _)) = set.iter().find(|(c, _)| keyed(c)) {
        let words = format!(
            "{at}'s set changes {c}, a key column of {name}: to give a row another key, delete it and insert it."
        );
        return Err(Refused::Form(words));
    }
    if set.is_empty() {
        return Err(Refused::Form(format!(
            "{at}'s set names no column: an update sets one or more of the other columns of {name}."
        )));
    }
    let sql = format!(
        "UPDATE {into} SET {} WHERE {matched}",
        listed(set.iter().map(|(c, _)| *c), |c| format!("{c} = ?"), ", ")
    );
    Ok((sql, set.into_iter().chain(key).map(|(_, v)| v).collect()))
}
