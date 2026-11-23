// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a `schema.sql` may make (svrn-docs `rings/current/APP_STATE.md`, the
//! pitch's first and fourth places, and §4, "Its schema"; MAGNETS_BUILD S5):
//! one decider, for `svrngs check` ([`check`]) and for a database made from
//! it (`Db::open`), so the two never disagree ([`made`]). SQLite runs the
//! file under the schema's authorizer, which lets it make only tables and
//! indexes (`schema_rule`); cr-sqlite's `crsql_as_crr` decides each table's
//! shape, and its six refusals are said by their text, which
//! `tests/schema.rs` pins, so a fork that rewords one fails there; svrngs's
//! own limits are read from SQLite's pragmas ([`limits`]). No SQL is parsed
//! here. A CHECK can't be told from the rest without parsing it, so it is
//! taken, and copies apply each other's sets with CHECKs off (`sync.rs`,
//! `keep`): the guide says so beside its first `CREATE TABLE`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc;

use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::{Connection, OptionalExtension as _};

use crate::form::Table;
use crate::{Open, Refused, fault, internal, schema_rule, tables_of};

/// Why a `schema.sql` makes no database, a closed set (AGENTS 9): what
/// stopped it, why, and the fix (svrn-docs `magnets-spikes/README.md`, S5,
/// the words drafted there).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Unmade {
    #[error(
        "The table {0} has no primary key, or its key can be NULL, and each computer in your group finds a row by its key. To fix it, give {0} a key that can't be NULL, such as `id TEXT PRIMARY KEY NOT NULL`: `INTEGER PRIMARY KEY` needs NOT NULL too."
    )]
    NoKey(String),
    #[error(
        "Part of the primary key of {0} can be NULL, and each computer in your group finds a row by its whole key. To fix it, make each column of the key NOT NULL."
    )]
    PartKey(String),
    #[error(
        "The table {0} has a UNIQUE column or index besides its primary key, and two computers writing apart can't keep a value unique between them. To fix it, drop the UNIQUE, or make that column the key."
    )]
    Unique(String),
    #[error(
        "The table {0} has a foreign key (REFERENCES), and a row can reach a computer before the row it points to. To fix it, keep the column but drop its REFERENCES, and have your page check that the row is there as it reads."
    )]
    Foreign(String),
    #[error(
        "The table {0} has a NOT NULL column with no DEFAULT, and a computer on an earlier version of your app makes rows without that column. To fix it, give the column a DEFAULT, or let it be NULL."
    )]
    NoDefault(String),
    #[error(
        "The table {0} has the word autoincrement in it, in a name or a comment, and cr-sqlite refuses a table with that word. To fix it, rename the column, or take the word out of the comment."
    )]
    Word(String),
    #[error(
        "The table {0} can't be kept in your group's database, and svrngs has no words for why yet. cr-sqlite says: {1}"
    )]
    Shape(String, String),
    #[error(
        "schema.sql says AUTOINCREMENT, and two computers would give two rows the same number. To fix it, key a row by what it is, or by an ID your page makes, such as `crypto.randomUUID()`."
    )]
    Autoincrement,
    #[error(
        "schema.sql asks SQLite for {0}, and schema.sql may only make tables and indexes, in names that don't begin with sqlite_, crsql or svrngs_. To fix it, take that statement out, and do its work in your page."
    )]
    Statement(String),
    #[error(
        "The primary key of {0} compares text as {1}, so 'Cat' written on one computer and 'cat' on another land in one row on each, with different values, and the computers never agree. To fix it, drop COLLATE from the key, and keep the text in the case you compare it in, such as lower case."
    )]
    Folded(String, String),
    #[error(
        "In {0}, person_key {1}. A table whose key begins with person_key keeps each row its own person's, and svrngs reads whose a row is by that column's exact text. To fix it, make it `person_key TEXT NOT NULL`, first in the PRIMARY KEY of a STRICT table."
    )]
    PersonKey(String, &'static str),
    #[error(
        "schema.sql drops or changes {0}, which the version it follows has, and a schema only grows: every computer of your group keeps what it holds of {0}. To fix it, keep {0} as it was, and add a new column for what changed."
    )]
    Changed(String),
    #[error(
        "schema.sql adds {0} as NOT NULL, with a DEFAULT, or in the key, and a page on the version it follows still writes rows without it. To fix it, add {0} as a column that can be NULL, with no DEFAULT, outside the key."
    )]
    Added(String),
    #[error("The schema.sql it follows makes no database either: {0}")]
    Follows(String),
}

/// A refusal of the table it is given.
type Said = fn(String) -> Unmade;

/// cr-sqlite's six refusals of a table, by the words its text holds
/// (`vendor/cr-sqlite/core/rs/core/src/tableinfo.rs:1044-1106`). A real
/// AUTOINCREMENT never reaches it: the authorizer refuses the
/// `sqlite_sequence` SQLite makes for it first.
const SHAPES: [(&str, Said); 6] = [
    ("no primary key or primary key is nullable", Unmade::NoKey),
    ("composite primary key part of which is nullable", Unmade::PartKey),
    ("unique indices besides", Unmade::Unique),
    ("checked foreign key", Unmade::Foreign),
    ("NOT NULL column without a DEFAULT", Unmade::NoDefault),
    ("auto-increment", Unmade::Word),
];

/// `schema` made on `conn`, which cr-sqlite is loaded on and holds none of
/// the app's tables yet: applied under the schema's authorizer, the first
/// statement it refuses said; each table it made a CRR; then held to
/// svrngs's limits. Its tables, or the first refusal. `crsql_as_crr` answers
/// a refusal with a bare ROLLBACK (`vendor/cr-sqlite/core/rs/core/src/lib.rs:783`),
/// so a caller's transaction may have ended as it returns.
pub(crate) fn made(conn: &mut Connection, schema: &str) -> Result<BTreeMap<String, Table>, Refused> {
    let (said, heard) = mpsc::channel();
    let ruled = move |ctx: AuthContext<'_>| {
        let ruled = schema_rule(ctx);
        let unmade = match ctx.action {
            AuthAction::CreateTable {
                table_name: "sqlite_sequence",
            } => Unmade::Autoincrement,
            other => Unmade::Statement(format!("{other:?}")),
        };
        let _ = (ruled == Authorization::Deny).then(|| said.send(unmade));
        ruled
    };
    conn.authorizer(Some(ruled)).map_err(fault)?;
    let applied = conn.execute_batch(schema);
    conn.authorizer(None::<fn(AuthContext<'_>) -> Authorization>)
        .map_err(fault)?;
    let unrun = |e| Refused::Malformed(format!("schema.sql: {e}"));
    applied.map_err(|e| heard.try_iter().next().map_or_else(|| unrun(e), Refused::Schema))?;
    let tables = tables_of(conn)?;
    for t in tables.keys() {
        let Err(e) = conn.query_row("SELECT crsql_as_crr(?1)", [t], |_| Ok(())) else {
            continue;
        };
        let (text, t) = (e.to_string(), t.clone());
        let shape = SHAPES.iter().find(|(said, _)| text.contains(said));
        tracing::debug!(target: "state", table = t, text, known = shape.is_some(), "state: a table of schema.sql refused by cr-sqlite, in words of svrngs's where it knows the text");
        return Err(Refused::Schema(
            shape.map_or(Unmade::Shape(t.clone(), text), |(_, s)| s(t)),
        ));
    }
    tables.keys().try_for_each(|t| limits(conn, t))?;
    Ok(tables)
}

/// svrngs's own limits on the table `t`, which cr-sqlite takes (MAGNETS_BUILD
/// S5; APP_STATE §4, "Who may write which row"): its key compared as BINARY,
/// since cr-sqlite keys a row by its bytes and a NOCASE key folds two of them
/// into one row on each copy, so copies never agree (`pragma_index_xinfo`
/// names a key's collation); and `person_key`, by which Eudoxia's rule reads
/// whose a row is, comparing its type and bytes, TEXT first in the key of a
/// STRICT table, so a column of that name never promises a check it isn't.
fn limits(conn: &Connection, t: &str) -> Result<(), Refused> {
    let folded = "SELECT x.coll FROM pragma_index_list(?1) l, pragma_index_xinfo(l.name) x \
        WHERE l.origin = 'pk' AND x.key AND x.coll NOT LIKE 'binary'";
    let folded: Option<String> = conn.query_row(folded, [t], |r| r.get(0)).optional().map_err(fault)?;
    let keyed = "SELECT c.pk, c.type LIKE 'text', l.strict FROM pragma_table_xinfo(?1) c, pragma_table_list(?1) l \
        WHERE lower(c.name) = 'person_key'";
    let keyed = conn.query_row(keyed, [t], |r| Ok((r.get::<_, i64>(0)?, r.get(1)?, r.get(2)?)));
    let off = match keyed.optional().map_err(fault)? {
        Some((pk, ..)) if pk != 1 => Some("isn't first in its primary key"),
        Some((_, false, _)) => Some("isn't TEXT"),
        Some((.., false)) => Some("is in a table that isn't STRICT"),
        _ => None,
    };
    tracing::debug!(target: "state", table = t, ?folded, off, "state: a table held to svrngs's limits, its key compared as BINARY and its person_key TEXT first in the key of a STRICT table");
    let unmade = folded
        .map(|c| Unmade::Folded(t.into(), c))
        .or(off.map(|o| Unmade::PersonKey(t.into(), o)));
    unmade.map_or(Ok(()), |u| Err(Refused::Schema(u)))
}

/// Each column of the app's tables, `table.column`, as `pragma_table_xinfo`
/// gives it: what an earlier version's must keep, its declared type, NOT
/// NULL and place in the key; and whether one added must not be there, as
/// NOT NULL, with a DEFAULT or in the key.
type Columns = BTreeMap<String, (String, bool)>;

fn columns(conn: &Connection) -> Result<Columns, Refused> {
    let each = "SELECT t.name, t.name || '.' || c.name, lower(c.type) || ' ' || c.\"notnull\" || ' ' || c.pk, \
        c.\"notnull\" OR c.dflt_value IS NOT NULL OR c.pk > 0 FROM sqlite_master AS t, pragma_table_xinfo(t.name) AS c \
        WHERE t.type = 'table'";
    let mut stmt = conn.prepare(each).map_err(fault)?;
    let read = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, (r.get(1)?, (r.get(2)?, r.get(3)?))))
    });
    let read: Vec<_> = read.and_then(Iterator::collect).map_err(fault)?;
    Ok(read.into_iter().filter(|(t, _)| !internal(t)).map(|(_, c)| c).collect())
}

/// `schema`, a `schema.sql`, as `svrngs check` reads it: each table it
/// makes and whether it is owned ([`Table::owned`]), or the first refusal,
/// made on a scratch database as a database is made from it ([`made`]);
/// and, where it follows `was`, an earlier version's, refused unless it only
/// adds to it ([`grows`]).
pub fn check(schema: &str, was: Option<&str>) -> Result<Vec<(String, bool)>, Refused> {
    let scratch = |s: &str| {
        // Closed as it drops, cr-sqlite's statements finalized first.
        let mut open = Open(Some(Connection::open_in_memory().map_err(fault)?));
        let tables = made(&mut open, s)?;
        Ok::<_, Refused>((columns(&open)?, tables))
    };
    let (now, tables) = scratch(schema)?;
    let follows = |r: Refused| Refused::Schema(Unmade::Follows(r.to_string()));
    was.map(|was| grows(&scratch(was).map_err(follows)?.0, &now))
        .transpose()?;
    Ok(tables.iter().map(|(t, table)| (t.clone(), table.owned())).collect())
}

/// Whether `now` only adds to `was` (APP_STATE §4, "It only adds"): each
/// column of `was` kept with its declared type, NOT NULL and place in the
/// key; and each column `now` adds to a table of `was` one that can be NULL,
/// with no DEFAULT and outside the key, so a page on the earlier version
/// still writes its rows, and no copy rebuilds a table's clocks as its own,
/// as a changed key does (cr-sqlite `alter.rs:42-71`, `backfill.rs:100-113`).
/// Corrosion's rules are the reference (`corro-types/src/schema.rs:433-558`).
fn grows(was: &Columns, now: &Columns) -> Result<(), Refused> {
    let tables: BTreeSet<_> = was.keys().filter_map(|c| c.split_once('.').map(|(t, _)| t)).collect();
    let in_was = |c: &String| c.split_once('.').is_some_and(|(t, _)| tables.contains(t));
    let changed = was
        .iter()
        .find(|(c, (kept, _))| now.get(*c).is_none_or(|(n, _)| n != kept));
    let added = now
        .iter()
        .find(|(c, (_, held))| *held && in_was(c) && !was.contains_key(*c));
    tracing::debug!(target: "state", changed = ?changed.map(|c| c.0), added = ?added.map(|c| c.0), "state: a schema.sql held to only adding to the version it follows");
    let unmade = changed
        .map(|(c, _)| Unmade::Changed(c.clone()))
        .or(added.map(|(c, _)| Unmade::Added(c.clone())));
    unmade.map_or(Ok(()), |u| Err(Refused::Schema(u)))
}
