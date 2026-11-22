// SPDX-License-Identifier: AGPL-3.0-or-later
//! An app's database in a group: SQLite tables beside the group's record, a
//! copy on every member's computer, merged by cr-sqlite (svrn-docs
//! `rings/current/APP_STATE.md` §4).
//!
//! cr-sqlite is built from its source, the submodule `vendor/cr-sqlite`, by
//! `build.rs`, into one object linked here whose one global is a shim in C
//! (`shim.c`). SQLite calls the shim as it starts, so cr-sqlite is loaded on
//! every connection and no Rust in svrngs calls it: nothing here is `unsafe`.
//! `tests/crsqlite.rs` holds that, with its plant; a database opened where
//! cr-sqlite's functions are missing is refused ([`Refused::Unbuilt`]). A
//! database's connection closes as it drops, cr-sqlite's statements
//! finalized first ([`close`]).
//!
//! Each carried app in each group gets a SQLite file on this computer
//! ([`States`]), opened from the `schema.sql` beside its `index.html`, its
//! tables made CRDTs by cr-sqlite, all in one transaction or none of it
//! ([`Db::open`]). A page asks it at its own origin
//! ([`asked`]): SQL to read ([`Db::read`]), and a write in the write
//! form, by key ([`Db::write`], `form.rs`). What SQL may do has one decider,
//! SQLite's authorizer ([`page_rule`]), asked as the page's statement is
//! prepared, and a statement that writes is refused, its words naming the
//! form ([`Refused::NotARead`]); a query runs under a progress handler that
//! stops it at [`QUERY_MOST`] and gives at most [`ROWS_MOST`] rows, saying
//! when there were more. rusqlite takes both by `&mut`, so each is
//! installed once, at open, and the statement at hand is told them through
//! a [`Gate`].
//!
//! Moved from the app-state spike's `state.rs` (svrn-docs
//! `rings/current/app-state-spike/spike.patch`), the half that keeps a
//! database on one computer: change sets, their signing and their sync come
//! with the next slice (`MAGNETS_BUILD.md` §3, slice B).
//!
//! Its tests came first, in `tests/w1/`: the reference model every copy of a
//! database is checked against, and the seeded simulator of deliveries that
//! drives the copies. They come before the code they judge, because "copies
//! agree, on the right state" (APP_STATE §11, W1) is the riskiest promise the
//! database makes, and tests count against no budget
//! (`scripts/lean_check.py`, `code_lines`).

#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::time::{Duration, Instant};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::types::{Value as Sql, ValueRef};
use rusqlite::{Connection, ErrorCode};
use serde_json::{Value, json};
use tokio::sync::watch;

mod form;
pub use form::FORM;
use form::Table;

/// The app's schema, beside its `index.html`.
pub const SCHEMA: &str = "schema.sql";
/// The longest a page's statement runs before it is stopped.
pub const QUERY_MOST: Duration = Duration::from_millis(250);
/// The most rows a page is given at once.
pub const ROWS_MOST: usize = 1000;

/// Why a page's ask was not run, said to the page in its words: what
/// stopped it, why, and what to do.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refused {
    /// A statement sent as SQL that writes.
    #[error(
        "svrngs/sql takes SQL only to read, and this statement writes: a write names its rows by key, the same on every device. {FORM}"
    )]
    NotARead,
    /// A body or an op the form doesn't take: what to change.
    #[error("{0}")]
    Form(String),
    /// The authorizer refused it: what it tried, as SQLite says.
    #[error("An app's SQL reads only the app's own tables, so SQLite refused it: {0}.")]
    NotAllowed(String),
    /// It ran past [`QUERY_MOST`] and was stopped.
    #[error("The query ran past {} ms and was stopped. Ask for less, or for it in parts.", QUERY_MOST.as_millis())]
    TooLong,
    /// SQLite could not run it: its words.
    #[error("SQLite couldn't run it: {0}.")]
    Malformed(String),
    /// This svrngs was built without cr-sqlite, so it opens no database.
    #[error(
        "This computer's svrngs was built without cr-sqlite, so it keeps no app's database. Build it again from its checkout, as its README says."
    )]
    Unbuilt,
    /// This computer could not: its words.
    #[error("This computer couldn't: {0}.")]
    Fault(String),
}

impl Refused {
    /// Its status, and its short name, as the route answers `{refused, words}`.
    pub fn said(&self) -> (u16, &'static str) {
        match self {
            Self::NotARead => (400, "not-a-read"),
            Self::Form(_) => (400, "form"),
            Self::NotAllowed(_) => (403, "not-allowed"),
            Self::TooLong => (422, "too-long"),
            Self::Malformed(_) => (400, "sql"),
            Self::Unbuilt => (500, "unbuilt"),
            Self::Fault(_) => (500, "fault"),
        }
    }
}

/// What a page's POST asks: one read, `{sql, params}`, or a write in the
/// form, `{write: [op, ...]}`.
pub enum Asked<'a> {
    Read(&'a str, &'a [Value]),
    Write(&'a Value),
}

/// The POST `body` as what it asks; refused naming both, for anything else.
pub fn asked(body: &Value) -> Result<Asked<'_>, Refused> {
    let fields = body
        .as_object()
        .map(|o| o.keys().map(String::as_str).collect::<Vec<_>>());
    let params = body
        .get("params")
        .map_or(Some(&[][..]), |p| p.as_array().map(Vec::as_slice));
    match (fields.as_deref(), body["sql"].as_str(), params) {
        (Some(["write"]), ..) => Ok(Asked::Write(&body["write"])),
        (Some(["sql"] | ["params", "sql"]), Some(sql), Some(params)) => Ok(Asked::Read(sql, params)),
        _ => Err(Refused::Form(format!(
            "svrngs/sql takes {{sql, params}}, one read, its params a list of values, or {{write: [op, ...]}}, not {body}. {FORM}"
        ))),
    }
}

/// This computer's app databases, by group and app.
#[derive(Default)]
pub struct States {
    dir: PathBuf,
    #[expect(clippy::disallowed_types, reason = "the databases opened, each once")]
    dbs: std::sync::Mutex<BTreeMap<(String, String), Arc<Db>>>,
}

impl States {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            dir: data_dir.join("state"),
            dbs: Default::default(),
        }
    }

    /// `app`'s database in `group`, opened with `schema` the first time;
    /// refused, it keeps none of it, and the next open tries it again.
    pub fn open(&self, group: &str, app: &str, schema: &str) -> Result<Arc<Db>, Refused> {
        let mut dbs = self.dbs.lock().expect("the databases' lock");
        if let Some(db) = dbs.get(&(group.into(), app.into())) {
            return Ok(db.clone());
        }
        let dir = self.dir.join(group);
        std::fs::create_dir_all(&dir).map_err(|e| Refused::Fault(e.to_string()))?;
        let db = Arc::new(Db::open(&dir.join(format!("{app}.db")), (group, app), schema)?);
        tracing::debug!(target: "state", group, app, tables = ?db.tables, "state: an app's database opened, its schema applied and its tables made CRRs");
        dbs.insert((group.into(), app.into()), db.clone());
        Ok(db)
    }
}

/// One app's database in one group on this computer.
pub struct Db {
    pub group: String,
    pub app: String,
    #[expect(clippy::disallowed_types, reason = "one connection, one statement at a time")]
    conn: std::sync::Mutex<Open>,
    tables: BTreeMap<String, Table>,
    gate: Arc<Gate>,
    changed: watch::Sender<u64>,
}

/// A connection cr-sqlite is loaded on, closed as it drops ([`close`]),
/// whether its database opened or was refused.
struct Open(Option<Connection>);

impl Drop for Open {
    fn drop(&mut self) {
        let conn = self.0.take().expect("a connection is open until it drops");
        let path = conn.path().unwrap_or_default().to_owned();
        let closed = close(conn);
        tracing::debug!(target: "state", path, closed = closed.is_ok(), "state: a database's connection closed as it dropped, cr-sqlite's statements finalized first, or SQLite refused");
        if let Err(e) = closed {
            tracing::error!(target: "state", path, %e, "state: a database's connection did not close, so it and its file stay open until svrngs exits");
        }
    }
}

/// `conn`, a connection cr-sqlite is loaded on, closed, or SQLite's words
/// for why not. cr-sqlite's statements are finalized first, so SQLite can
/// close it: only libSQL's build of cr-sqlite finalizes them itself
/// (`vendor/cr-sqlite/core/src/crsqlite.c:31-36`, `:80-83`, under `#ifdef
/// LIBSQL`). A statement left open makes `sqlite3_close` refuse, and
/// rusqlite drops a connection without a word when it does (rusqlite 0.41
/// `inner_connection.rs:401-406`), so the connection and its file stay
/// open for the life of the process. `Connection::close` flushes rusqlite's
/// own cache of statements (rusqlite 0.41 `lib.rs:821-826`).
pub fn close(conn: Connection) -> Result<(), rusqlite::Error> {
    conn.query_row("SELECT crsql_finalize()", [], |_| Ok(()))?;
    conn.close().map_err(|(_, e)| e)
}

/// What the authorizer and the progress handler, each installed once at
/// open, read of the statement at hand: whether the page's rule decides
/// it, and when it must stop, in µs past `since` (`u64::MAX`: never).
struct Gate {
    page: AtomicBool,
    until: AtomicU64,
    since: Instant,
}

impl Gate {
    /// µs past `since`, now.
    #[expect(clippy::disallowed_methods, reason = "a page's query is timed where SQLite runs it")]
    fn now(&self) -> u64 {
        u64::try_from(Instant::now().duration_since(self.since).as_micros()).unwrap_or(u64::MAX)
    }
}

fn fault(e: impl std::fmt::Display) -> Refused {
    Refused::Fault(e.to_string())
}

/// The internal tables, which the authorizer never lets a page name.
fn internal(table: &str) -> bool {
    let t = table.to_ascii_lowercase();
    t.starts_with("crsql")
        || t.starts_with("__crsql")
        || t.starts_with("sqlite_")
        || t.starts_with("svrngs_")
        || t.contains("__crsql_")
}

/// What `schema.sql` may do: make tables and indexes in `main`, and what
/// SQLite reads and writes of its own schema to do it.
fn schema_rule(ctx: AuthContext<'_>) -> Authorization {
    let main = ctx.database_name.is_none_or(|d| d == "main");
    let allowed = main
        && match ctx.action {
            AuthAction::CreateTable { table_name } | AuthAction::CreateIndex { table_name, .. } => {
                !internal(table_name)
            }
            AuthAction::Insert { table_name }
            | AuthAction::Update { table_name, .. }
            | AuthAction::Read { table_name, .. } => {
                table_name == "sqlite_master" || table_name == "sqlite_schema" || !internal(table_name)
            }
            AuthAction::Function { function_name } => {
                !function_name.eq_ignore_ascii_case("load_extension") && !function_name.starts_with("crsql")
            }
            _ => false,
        };
    tracing::debug!(target: "state", action = ?ctx.action, allowed, "state: schema.sql's statement, allowed or refused by the authorizer");
    if allowed {
        Authorization::Allow
    } else {
        Authorization::Deny
    }
}

/// What a page's SQL may do (APP_STATE §4, "What SQL may do"): read,
/// insert, update and delete the app's own tables, select, recurse and call
/// functions but `load_extension` and cr-sqlite's own; and whatever
/// cr-sqlite's own triggers on those tables do. Nothing else: no schema
/// change, ATTACH, PRAGMA, transaction or extension. It decides only while
/// the page's statement is prepared ([`Gate`]); svrngs's own run as asked.
fn page_rule(
    tables: BTreeSet<String>,
    gate: Arc<Gate>,
) -> impl for<'r> FnMut(AuthContext<'r>) -> Authorization + Send + 'static {
    move |ctx: AuthContext<'_>| {
        if !gate.page.load(Relaxed) {
            return Authorization::Allow;
        }
        let own = |t: &str| tables.contains(t);
        // A trigger cr-sqlite made on one of the app's tables.
        let by_crsql = (ctx.accessor).is_some_and(|a| tables.iter().any(|t| a.starts_with(&format!("{t}__crsql_"))));
        let main = ctx.database_name.is_none_or(|d| d == "main");
        let allowed = by_crsql
            || main
                && match ctx.action {
                    AuthAction::Select | AuthAction::Recursive => true,
                    // A read with no database names a WITH's table, never a stored one.
                    AuthAction::Read { table_name, .. } => ctx.database_name.is_none() || own(table_name),
                    AuthAction::Insert { table_name }
                    | AuthAction::Delete { table_name }
                    | AuthAction::Update { table_name, .. } => own(table_name),
                    AuthAction::Function { function_name } => {
                        let f = function_name.to_ascii_lowercase();
                        f != "load_extension" && !f.starts_with("crsql") && f != "fts3_tokenizer"
                    }
                    _ => false,
                };
        if allowed {
            tracing::trace!(target: "state", action = ?ctx.action, accessor = ?ctx.accessor, "state: a page's SQL, allowed by the authorizer");
            Authorization::Allow
        } else {
            tracing::debug!(target: "state", action = ?ctx.action, accessor = ?ctx.accessor, db = ?ctx.database_name, "state: a page's SQL, refused by the authorizer");
            Authorization::Deny
        }
    }
}

impl Db {
    fn open(path: &Path, (group, app): (&str, &str), schema: &str) -> Result<Self, Refused> {
        let conn = Connection::open(path).map_err(fault)?;
        // cr-sqlite, loaded on each connection by SQLite itself (shim.c): a
        // build without the shim holds none of its functions, and a database
        // opened there would merge nothing, so none is.
        let loaded = conn.query_row("SELECT crsql_site_id()", [], |_| Ok(()));
        tracing::debug!(target: "state", group, app, loaded = loaded.is_ok(), "state: a database opened only where cr-sqlite is loaded");
        if let Err(e) = loaded {
            tracing::error!(target: "state", group, app, %e, "state: this svrngs was built without cr-sqlite, so no database opens");
            return Err(Refused::Unbuilt);
        }
        // Closed as it drops from here, refused below or not: cr-sqlite
        // prepared statements on it as it loaded
        // (`vendor/cr-sqlite/core/src/ext-data.c:24-29`), which only
        // [`close`] finalizes. Without cr-sqlite, above, it has none.
        let mut open = Open(Some(conn));
        let conn = open.0.as_mut().expect("just opened");
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(fault)?;
        // A database with no table of the app's is new: its schema applied,
        // then its tables made cr-sqlite's, in one transaction, begun and
        // ended with no authorizer, as `schema_rule` refuses one in
        // `schema.sql`. Refused at any step, none of it is kept, so the next
        // open finds the database new again, and refuses it in the same words.
        let mut tables = tables_of(conn)?;
        tracing::debug!(target: "state", group, app, fresh = tables.is_empty(), "state: a database opened; its schema applied only when new");
        if tables.is_empty() {
            conn.execute_batch("BEGIN IMMEDIATE").map_err(fault)?;
            let made = new_tables(conn, schema);
            // `crsql_as_crr` refusing a table rolls back the whole transaction
            // itself (`vendor/cr-sqlite/core/rs/core/src/lib.rs:783`), so
            // only one still open is rolled back here.
            let still_open = !conn.is_autocommit();
            let ended = match &made {
                Ok(_) => conn.execute_batch("COMMIT"),
                Err(_) if still_open => conn.execute_batch("ROLLBACK"),
                Err(_) => Ok(()),
            };
            tracing::debug!(target: "state", group, app, refused = ?made.as_ref().err(), still_open, ?ended, "state: a new database's schema applied and its tables made CRRs in one transaction, committed; or refused and rolled back whole, here if still open, else by cr-sqlite as it refused");
            // A COMMIT or ROLLBACK that fails is this computer's fault.
            ended.map_err(fault)?;
            tables = made?;
        }
        let (page, until) = (AtomicBool::new(false), AtomicU64::new(u64::MAX));
        #[expect(clippy::disallowed_methods, reason = "a page's query is timed where SQLite runs it")]
        let gate = Arc::new(Gate {
            page,
            until,
            since: Instant::now(),
        });
        conn.authorizer(Some(page_rule(tables.keys().cloned().collect(), gate.clone())))
            .map_err(fault)?;
        let timed = gate.clone();
        conn.progress_handler(1000, Some(move || timed.now() > timed.until.load(Relaxed)))
            .map_err(fault)?;
        Ok(Self {
            group: group.into(),
            app: app.into(),
            conn: open.into(),
            tables,
            gate,
            changed: watch::Sender::new(0),
        })
    }

    /// Told each time this database changes, by a write here.
    pub fn watch(&self) -> watch::Receiver<u64> {
        self.changed.subscribe()
    }

    /// The page's `sql` with `params`, one statement that only reads:
    /// `{rows, more}`. A statement that writes is refused, before it runs,
    /// in words that name the write form.
    pub fn read(&self, sql: &str, params: &[Value]) -> Result<Value, Refused> {
        let params = (params.iter().enumerate()).map(|(i, v)| form::value(v, &format!("params[{i}]")));
        let params = params.collect::<Result<Vec<_>, _>>()?;
        let conn = self.conn.lock().expect("the database's lock");
        let conn = conn.0.as_ref().expect("a connection is open until it drops");
        self.gate.page.store(true, Relaxed);
        let prepared = conn.prepare(sql);
        self.gate.page.store(false, Relaxed);
        let mut stmt = prepared.map_err(said)?;
        tracing::debug!(target: "state", group = self.group, app = self.app, readonly = stmt.readonly(), "state: a page's statement prepared under the authorizer; it runs only if it only reads");
        if !stmt.readonly() {
            return Err(Refused::NotARead);
        }
        let until = self.gate.now().saturating_add(QUERY_MOST.as_micros() as u64);
        self.gate.until.store(until, Relaxed);
        let ran = rows(&mut stmt, &params);
        self.gate.until.store(u64::MAX, Relaxed);
        tracing::debug!(target: "state", group = self.group, app = self.app, refused = ?ran.as_ref().err(), "state: a page's read answered, or stopped");
        let (rows, more) = ran?;
        Ok(json!({ "rows": rows, "more": more }))
    }

    /// The page's write, `ops` in the write form, as one transaction:
    /// `{count}`, the rows changed in all; or refused, nothing of it kept.
    pub fn write(&self, ops: &Value) -> Result<Value, Refused> {
        let listed = ops.as_array().filter(|o| !o.is_empty());
        let listed =
            listed.ok_or_else(|| Refused::Form(format!("write takes a list of one or more ops, not {ops}. {FORM}")))?;
        let made = |(i, op)| form::statement(op, &self.tables, i);
        let statements = listed.iter().enumerate().map(made).collect::<Result<Vec<_>, _>>();
        tracing::debug!(target: "state", group = self.group, app = self.app, ops = listed.len(), refused = ?statements.as_ref().err(), "state: a write's ops made statements by key, or refused");
        let statements = statements?;
        let conn = self.conn.lock().expect("the database's lock");
        let conn = conn.0.as_ref().expect("a connection is open until it drops");
        conn.execute_batch("BEGIN IMMEDIATE").map_err(fault)?;
        let ran = (statements.iter()).map(|(sql, values)| conn.execute(sql, rusqlite::params_from_iter(values)));
        let ran = ran.sum::<Result<usize, _>>();
        tracing::debug!(target: "state", group = self.group, app = self.app, ?ran, "state: a write run whole and committed, or rolled back whole as SQLite refused it");
        // Rolled back, SQLite's refusal of the list is said; a rollback that
        // fails is this computer's fault, said as one.
        let count = ran.map_err(|e| conn.execute_batch("ROLLBACK").map_or_else(fault, |()| said(e)))?;
        conn.execute_batch("COMMIT").map_err(fault)?;
        if count > 0 {
            self.changed.send_modify(|n| *n += 1);
        }
        Ok(json!({ "count": count }))
    }
}

/// A new database's tables: `schema` applied under [`schema_rule`], then
/// each table it made made a CRR; or the first refusal.
fn new_tables(conn: &mut Connection, schema: &str) -> Result<BTreeMap<String, Table>, Refused> {
    conn.authorizer(Some(schema_rule)).map_err(fault)?;
    let applied = conn.execute_batch(schema);
    conn.authorizer(None::<fn(AuthContext<'_>) -> Authorization>)
        .map_err(fault)?;
    applied.map_err(|e| Refused::Malformed(format!("schema.sql: {e}")))?;
    let tables = tables_of(conn)?;
    for t in tables.keys() {
        conn.query_row("SELECT crsql_as_crr(?1)", [t], |_| Ok(()))
            .map_err(|e| Refused::Malformed(format!("{t}: {e}")))?;
    }
    Ok(tables)
}

/// Each of the app's tables in `conn`, with its columns and key, as the
/// write form names them.
fn tables_of(conn: &Connection) -> Result<BTreeMap<String, Table>, Refused> {
    let each = "SELECT t.name, c.name, c.pk FROM sqlite_master AS t, pragma_table_info(t.name) AS c \
        WHERE t.type = 'table' ORDER BY t.name, c.pk = 0, c.pk, c.cid";
    let mut stmt = conn.prepare(each).map_err(fault)?;
    let columns = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))
    });
    let mut tables = BTreeMap::<String, Table>::new();
    for column in columns.map_err(fault)? {
        let (name, column, pk) = column.map_err(fault)?;
        let table = tables.entry(name).or_default();
        (pk > 0).then(|| table.key.push(column.clone()));
        table.columns.push(column);
    }
    tables.retain(|name, _| !internal(name));
    Ok(tables)
}

/// A SQLite error as the page is told it.
fn said(e: rusqlite::Error) -> Refused {
    match e.sqlite_error_code() {
        Some(ErrorCode::AuthorizationForStatementDenied) => Refused::NotAllowed(e.to_string()),
        // A function the authorizer refused is said as a plain error.
        _ if e.to_string().contains("not authorized") => Refused::NotAllowed(e.to_string()),
        Some(ErrorCode::OperationInterrupted) => Refused::TooLong,
        _ => Refused::Malformed(e.to_string()),
    }
}

/// The statement's rows, at most [`ROWS_MOST`], and whether there were more.
fn rows(stmt: &mut rusqlite::Statement<'_>, params: &[Sql]) -> Result<(Vec<Value>, bool), Refused> {
    let names: Vec<String> = stmt.column_names().map(String::from).collect();
    let mut found = stmt.query(rusqlite::params_from_iter(params)).map_err(said)?;
    let mut out = Vec::new();
    while let Some(row) = found.next().map_err(said)? {
        if out.len() == ROWS_MOST {
            return Ok((out, true));
        }
        let mut cells = serde_json::Map::new();
        for (i, name) in names.iter().enumerate() {
            let cell = match row.get_ref(i).map_err(said)? {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(i) => json!(i),
                ValueRef::Real(r) => json!(r),
                ValueRef::Text(t) => json!(String::from_utf8_lossy(t)),
                ValueRef::Blob(b) => json!({ "base64": STANDARD.encode(b) }),
            };
            cells.insert(name.clone(), cell);
        }
        out.push(Value::Object(cells));
    }
    Ok((out, false))
}
