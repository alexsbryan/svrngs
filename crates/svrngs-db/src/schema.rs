// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a `schema.sql` may make (svrn-docs `rings/current/APP_STATE.md`, the
//! pitch's first and fourth places, and §4, "Its schema"; MAGNETS_BUILD S5):
//! one decider, for `svrngs check` ([`check`]) and for a database made from
//! it (`Db::open`), so the two never disagree ([`made`]). SQLite runs the
//! file under the schema's authorizer, which lets it only declare: make
//! tables and indexes, write no row and call no function (`schema_rule`,
//! which SQLite asks of a CHECK's, an index's, a WHERE's and a DEFAULT's
//! functions as each is made; `tests/schema.rs` holds it); cr-sqlite's
//! `crsql_as_crr` decides each table's shape, and its six refusals are said
//! by their text, which `tests/schema.rs` pins, so a fork that rewords one
//! fails there; svrngs's own limits are read from SQLite's pragmas and its
//! program for a row's insert ([`limits`]): a table with a rowid, of plain
//! columns, a type, NOT NULL, the key, STRICT and a DEFAULT of one value,
//! with no CHECK; an index of columns, with no expression and no WHERE. So
//! no expression of a schema runs on any copy, for a row it writes or
//! takes, and what a copy holds is what SQLite's pragmas say of it, which a
//! later version is compared by ([`follows`]). No statement is parsed here:
//! a DEFAULT's words, as SQLite keeps them, are read as one value or not
//! ([`literal`]).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc;

use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::{Connection, OptionalExtension as _};

use crate::form::Table;
use crate::sync::rows;
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
        "schema.sql drops or changes {0}, which the version it follows has, and a schema only grows: every computer of your group keeps what it holds of {0}. To fix it, keep {0} as it was, in its place, and add a new column after the table's last one for what changed."
    )]
    Changed(String),
    #[error(
        "schema.sql adds {0} as NOT NULL, with a DEFAULT, or in the key, and a page on the version it follows still writes rows without it. To fix it, add {0} as a plain column that can be NULL, with no DEFAULT, outside the key."
    )]
    Added(String),
    #[error(
        "{0}, and every computer of your group would work it out for each row it writes or takes, so schema.sql only declares. To fix it, {1}."
    )]
    Runs(String, &'static str),
    #[error(
        "In {0}, {1}, and svrngs keeps only tables with a rowid, of columns a write fills, so every later version of your app reads each row alike. To fix it, take out WITHOUT ROWID and GENERATED: the primary key still finds each row, and your page can work out a value as it writes."
    )]
    Unplain(String, String),
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
            AuthAction::Function { function_name } => Unmade::Runs(
                format!("schema.sql calls {function_name}"),
                "write a plain value, and work out anything else in your page",
            ),
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
/// STRICT table, so a column of that name never promises a check it isn't;
/// and a table with a rowid, none of its columns GENERATED, as a copy made
/// wider reads the rows it holds by their place in the table (`widen`);
/// and no expression of it that a copy works out for a row: no CHECK,
/// which SQLite's program for a row's insert halts on as
/// `SQLITE_CONSTRAINT_CHECK`, a CHECK of no function included; no index
/// of an expression (`pragma_index_xinfo`'s cid below 0), whose entry
/// can be many times its row, or with a WHERE; and each DEFAULT one value
/// ([`literal`]) no longer than a value a copy keeps (`parse.rs`, `fit`),
/// as cr-sqlite writes a DEFAULT's words into its own statements
/// (`vendor/cr-sqlite/core/rs/core/src/backfill.rs:226-228`) and a merge
/// fills it into the row it makes again (`changes_vtab_write.rs`,
/// `clear_earlier_life`).
fn limits(conn: &Connection, t: &str) -> Result<(), Refused> {
    let plain = "SELECT CASE WHEN l.wr THEN 'the table is WITHOUT ROWID' ELSE \
        (SELECT c.name || ' is GENERATED' FROM pragma_table_xinfo(?1) c WHERE c.hidden > 1) END FROM pragma_table_list(?1) l";
    let unplain: Option<String> = conn.query_row(plain, [t], |r| r.get(0)).map_err(fault)?;
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
    let insert = format!("EXPLAIN INSERT INTO \"{}\" DEFAULT VALUES", t.replace('"', "\"\""));
    let halts = rows(conn, &insert, [], |r| Ok((r.get::<_, String>(1)?, r.get::<_, i64>(2)?)));
    let check = ("Halt".into(), i64::from(rusqlite::ffi::SQLITE_CONSTRAINT_CHECK));
    let checked = halts.map_err(fault)?.contains(&check);
    let checked = checked.then(|| format!("The table {t} has a CHECK"));
    let index = "SELECT 'The index ' || l.name || iif(l.partial, ' has a WHERE', ' orders an expression') \
        FROM pragma_index_list(?1) l \
        WHERE l.partial OR EXISTS (SELECT 1 FROM pragma_index_xinfo(l.name) x WHERE x.key AND x.cid < 0)";
    let index: Option<String> = conn.query_row(index, [t], |r| r.get(0)).optional().map_err(fault)?;
    let defaults = "SELECT name, dflt_value FROM pragma_table_xinfo(?1) WHERE dflt_value NOT NULL";
    let defaults: Vec<(String, String)> = rows(conn, defaults, [t], |r| Ok((r.get(0)?, r.get(1)?))).map_err(fault)?;
    let unvalued = (defaults.into_iter()).find(|(_, d)| !literal(d) || d.len() > crate::ANSWER_MOST);
    let unvalued = unvalued.map(|(c, _)| format!("In {t}, the DEFAULT of {c} isn't one short value"));
    tracing::debug!(target: "state", table = t, ?unplain, ?folded, off, ?checked, ?index, ?unvalued, "state: a table held to svrngs's limits, a rowid table of plain columns with no CHECK and each DEFAULT one value, its key compared as BINARY, its person_key TEXT first in the key of a STRICT table, and each index of columns with no WHERE");
    let runs = [
        (checked, "take the CHECK out, and check the value in your page"),
        (index, "index columns, keeping a worked-out value in its own column"),
        (unvalued, "write a number, a 'string', x'…' bytes or NULL"),
    ];
    let runs = (runs.into_iter()).find_map(|(what, fix)| what.map(|w| Unmade::Runs(w, fix)));
    let unmade = (unplain.map(|u| Unmade::Unplain(t.into(), u)))
        .or(folded.map(|c| Unmade::Folded(t.into(), c)))
        .or(off.map(|o| Unmade::PersonKey(t.into(), o)))
        .or(runs);
    unmade.map_or(Ok(()), |u| Err(Refused::Schema(u)))
}

/// Whether `words`, a DEFAULT as SQLite keeps it (`pragma_table_xinfo`'s
/// `dflt_value`, the text between its parentheses where it had them), is
/// one value as written: a number, signed or not, a 'string', its quotes
/// doubled inside it, x'…' bytes, or NULL. Anything else is an expression.
fn literal(words: &str) -> bool {
    let quoted = |w: &str| {
        let inside = w.strip_prefix('\'').and_then(|w| w.strip_suffix('\''));
        inside.is_some_and(|i| !i.replace("''", "").contains('\''))
    };
    let number = words.strip_prefix(['-', '+']).unwrap_or(words);
    let number = number.starts_with(|c: char| c.is_ascii_digit() || c == '.') && number.parse::<f64>().is_ok();
    number || quoted(words) || words.strip_prefix(['x', 'X']).is_some_and(quoted) || words.eq_ignore_ascii_case("null")
}

/// Each column of the app's tables, `table.column`, as `pragma_table_xinfo`
/// and `pragma_table_list` give it: what an earlier version's must keep, its
/// declared type, NOT NULL, place in the key and place in the table, as a
/// copy made wider adds a column after the last (`widen`), its DEFAULT,
/// whether it is GENERATED, and whether its table is STRICT or WITHOUT
/// ROWID, a copy made before those were refused included; and whether one
/// added must not be there, as NOT NULL, with a DEFAULT or in the key. And
/// each table's key as SQLite orders it, `the primary key of <table>`: the
/// index SQLite makes for the key or a UNIQUE (`pragma_index_list`'s origin
/// other than `c`), which has no text, each key column's name, order and
/// collation, and none where an INTEGER key is the rowid. So a key declared
/// DESC, or an INTEGER key made the rowid or no longer it, which leaves the
/// rows a copy holds read in another order or place, is a change, either way.
type Columns = BTreeMap<String, (String, bool)>;

fn columns(conn: &Connection) -> Result<Columns, Refused> {
    let each = "SELECT t.name, t.name || '.' || c.name, \
        json_array(lower(c.type), c.\"notnull\", c.pk, c.cid, c.dflt_value, c.hidden, l.strict, l.wr), \
        c.\"notnull\" OR c.dflt_value IS NOT NULL OR c.pk > 0 \
        FROM sqlite_master AS t, pragma_table_xinfo(t.name) AS c, pragma_table_list(t.name) AS l WHERE t.type = 'table' \
        UNION ALL SELECT t.name, 'the primary key of ' || t.name, (SELECT json_group_array(json_array(l.origin, x.name, x.desc, x.coll)) \
        FROM pragma_index_list(t.name) AS l, pragma_index_xinfo(l.name) AS x WHERE l.origin <> 'c' AND x.key), 0 \
        FROM sqlite_master AS t WHERE t.type = 'table'";
    let read = rows(conn, each, [], |r| {
        Ok((r.get::<_, String>(0)?, (r.get(1)?, (r.get(2)?, r.get(3)?))))
    });
    Ok(read
        .map_err(fault)?
        .into_iter()
        .filter(|(t, _)| !internal(t))
        .map(|(_, c)| c)
        .collect())
}

/// `schema`, a `schema.sql`, as `svrngs check` reads it: each table it
/// makes and whether it is owned ([`Table::owned`]), or the first refusal,
/// made on a scratch database as a database is made from it ([`made`]);
/// and, where it follows `was`, an earlier version's, refused unless it only
/// adds to it ([`follows`]).
pub fn check(schema: &str, was: Option<&str>) -> Result<Vec<(String, bool)>, Refused> {
    let (_, now, tables) = scratch(schema)?;
    let earlier = |r: Refused| Refused::Schema(Unmade::Follows(r.to_string()));
    was.map(|was| follows(&scratch(was).map_err(earlier)?.1, &now))
        .transpose()?;
    Ok(tables.iter().map(|(t, table)| (t.clone(), table.owned())).collect())
}

/// Whether `now` only adds to `was` (APP_STATE §4, "It only adds"), each a
/// schema's columns and texts: the one decider for `svrngs check`, a version
/// held against a copy and a copy made wider. Each column of `was` kept as
/// [`Columns`] reads it, and each table's key as SQLite orders it, both
/// ways; each index of `was` that `now` has kept with its text and its
/// columns' collations, as an index a copy holds is ordered by them, which
/// with the key's is every index a copy holds, as cr-sqlite takes no other
/// UNIQUE ([`made`]); and each column `now` adds to a table of `was` one that can be
/// NULL, with no DEFAULT and outside the key, so a page on the earlier
/// version still writes its rows, and no copy rebuilds a table's clocks as
/// its own, as a changed key does (cr-sqlite `alter.rs:42-71`,
/// `backfill.rs:100-113`). So a wider version's table text, which a copy made
/// wider takes (`widen`), declares each column the copy holds as it holds
/// it, and the closed set ([`limits`]) leaves nothing that stores a row
/// otherwise. Corrosion's rules are the reference
/// (`corro-types/src/schema.rs:433-558`).
fn follows((was, held): &Shape, (now, texts): &Shape) -> Result<(), Refused> {
    let tables: BTreeSet<_> = was.keys().filter_map(|c| c.split_once('.').map(|(t, _)| t)).collect();
    let in_was = |c: &String| c.split_once('.').is_some_and(|(t, _)| tables.contains(t));
    let changed = was
        .iter()
        .find(|(c, (kept, _))| now.get(*c).is_none_or(|(n, _)| n != kept));
    let index = (held.iter()).find(|(n, t)| t.0 == "index" && texts.get(*n).is_some_and(|m| m != *t));
    let added = now
        .iter()
        .find(|(c, (_, held))| *held && in_was(c) && !was.contains_key(*c));
    tracing::debug!(target: "state", changed = ?changed.map(|c| c.0), index = ?index.map(|i| i.0), added = ?added.map(|c| c.0), "state: a schema.sql held to only adding to the version it follows");
    let unmade = (added.map(|(c, _)| Unmade::Added(c.clone())))
        .or((changed.map(|(c, _)| c).or(index.map(|(n, _)| n))).map(|c| Unmade::Changed(c.clone())));
    unmade.map_or(Ok(()), |u| Err(Refused::Schema(u)))
}

/// The text of each of the app's tables and indexes, by name, as
/// `sqlite_master` keeps it, with its kind and, for an index, its key
/// columns' collations.
type Texts = BTreeMap<String, (String, String, Option<String>)>;

/// What [`follows`] compares of a schema.
type Shape = (Columns, Texts);

fn texts(conn: &Connection) -> Result<Texts, Refused> {
    let each = "SELECT m.name, m.type, m.tbl_name, m.sql, (SELECT group_concat(x.coll) FROM pragma_index_xinfo(m.name) x WHERE x.key) \
        FROM sqlite_master m WHERE m.type IN ('table', 'index') AND m.sql NOT NULL";
    let read = rows(conn, each, [], |r| {
        Ok((r.get(0)?, r.get(2)?, (r.get(1)?, r.get(3)?, r.get(4)?)))
    });
    let read: Vec<(String, String, _)> = read.map_err(fault)?;
    let app = read.into_iter().filter(|(n, t, _)| !internal(n) && !internal(t));
    Ok(app.map(|(n, _, text)| (n, text)).collect())
}

/// `schema` made on a scratch database, with its shape and tables; closed
/// as it drops, cr-sqlite's statements finalized first.
fn scratch(schema: &str) -> Result<(Open, Shape, BTreeMap<String, Table>), Refused> {
    let mut open = Open(Some(Connection::open_in_memory().map_err(fault)?));
    let tables = made(&mut open, schema)?;
    let shape = (columns(&open)?, texts(&open)?);
    Ok((open, shape, tables))
}

/// Whether `schema` adds to what `conn`, a database, holds: false for one
/// that adds nothing, as an older version's; refused where it doesn't only
/// add to it ([`follows`]), as two versions that each add to a third, which
/// no copy made from either matches.
pub(crate) fn adds(conn: &Connection, schema: &str) -> Result<bool, Refused> {
    let (wider, had) = (scratch(schema)?.1, (columns(conn)?, texts(conn)?));
    let more = (wider.0.keys()).any(|c| !had.0.contains_key(c)) || wider.1.keys().any(|n| !had.1.contains_key(n));
    tracing::debug!(target: "state", more, "state: a version's schema held against a database's, adding to it or not");
    more.then(|| follows(&had, &wider)).transpose().map(|g| g.is_some())
}

/// `schema`, a wider version's, made on `conn`, a restored copy of a
/// database (`States::widen`), where it only adds to it ([`follows`]): each
/// column it adds to a table here added after the last, then the table's
/// text made `schema`'s, so it compares, sorts and checks as on a copy made
/// from `schema`, within cr-sqlite's bracket, so its triggers take the
/// column; each table it adds made, and a CRR, then each index. Held after
/// to `schema`'s text, table for table and index for index, and its
/// columns, then `ANALYZE`d, `schema`'s hash the copy's. The text is
/// written under `writable_schema`, as no statement of SQLite's makes a
/// table's text a fresh copy's: `ALTER TABLE … ADD COLUMN` writes `, ` and
/// the column's own text after the last, so a table written over lines
/// ends unlike its fresh copy; what [`follows`] compared first is what lets
/// it be written.
pub(crate) fn widen(conn: &Connection, schema: &str) -> Result<(), Refused> {
    let (scratch, wider, _) = scratch(schema)?;
    let ours = (columns(conn)?, texts(conn)?);
    follows(&ours, &wider)?;
    let ((had, held), (now, texts)) = (&ours, &wider);
    let run = |sql: &str| conn.execute_batch(sql).map_err(fault);
    let crsql = |f: &str, t: &str| conn.query_row(&format!("SELECT crsql_{f}"), [t], |_| Ok(()));
    let quoted = |n: &str| format!("\"{}\"", n.replace('"', "\"\""));
    let tables = texts
        .iter()
        .filter(|(n, (kind, ..))| kind == "table" && held.contains_key(*n));
    for (name, (_, sql, _)) in tables {
        let each = "SELECT name FROM pragma_table_xinfo(?1) ORDER BY cid";
        let each = rows(&scratch, each, [name], |r| r.get::<_, String>(0)).map_err(fault)?;
        let added: Vec<_> = each
            .into_iter()
            .filter(|c| !had.contains_key(&format!("{name}.{c}")))
            .collect();
        if added.is_empty() {
            continue;
        }
        crsql("begin_alter(?1)", name).map_err(fault)?;
        for c in added {
            run(&format!("ALTER TABLE {} ADD COLUMN {}", quoted(name), quoted(&c)))?;
        }
        run("PRAGMA writable_schema = ON")?;
        let text = "UPDATE sqlite_master SET sql = ?1 WHERE type = 'table' AND name = ?2";
        conn.execute(text, [sql, name]).map_err(fault)?;
        run("PRAGMA writable_schema = RESET")?;
        crsql("commit_alter('main', ?1, 1)", name).map_err(fault)?;
    }
    let mut made: Vec<_> = texts.iter().filter(|(n, _)| !held.contains_key(*n)).collect();
    made.sort_by_key(|(_, (kind, ..))| kind != "table");
    for (name, (kind, sql, _)) in made {
        run(sql)?;
        (kind == "table")
            .then(|| crsql("as_crr(?1)", name))
            .transpose()
            .map_err(fault)?;
    }
    let (held, cols) = (self::texts(conn)?, columns(conn)?);
    let same = texts.iter().all(|(n, t)| held.get(n) == Some(t)) && now.iter().all(|(c, d)| cols.get(c) == Some(d));
    tracing::debug!(target: "state", same, "state: a restored copy made wider, held to the wider schema's text and columns");
    same.then_some(())
        .ok_or_else(|| fault("a copy made wider isn't its schema's"))?;
    conn.execute("UPDATE svrngs_db SET schema = ?1", [svrngs_core::set::schema(schema)])
        .map_err(fault)?;
    run("ANALYZE")
}
