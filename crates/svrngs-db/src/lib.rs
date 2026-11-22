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
//! cr-sqlite's functions are missing is refused ([`Refused::Unbuilt`]).
//!
//! Each carried app in each group gets a SQLite file on this computer
//! ([`States`]), opened from the `schema.sql` beside its `index.html`, its
//! tables made CRDTs by cr-sqlite ([`Db::open`]). A page asks it at its own
//! origin ([`Db::run`]). What SQL may do has one decider, SQLite's authorizer
//! ([`page_rule`]), asked as the page's statement is prepared; a query runs
//! under a progress handler that stops it at [`QUERY_MOST`] and gives at most
//! [`ROWS_MOST`] rows, saying when there were more. rusqlite takes both by
//! `&mut`, so each is installed once, at open, and the statement at hand is
//! told them through a [`Gate`].
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

use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::types::{Value as Sql, ValueRef};
use rusqlite::{Connection, ErrorCode};
use serde_json::{Value, json};
use tokio::sync::watch;

/// Where a carried app's page asks its database, at its own origin.
pub const SQL_PATH: &str = "/svrngs/sql";
/// The app's schema, beside its `index.html`.
pub const SCHEMA: &str = "schema.sql";
/// The longest a page's statement runs before it is stopped.
pub const QUERY_MOST: Duration = Duration::from_millis(250);
/// The most rows a page is given at once.
pub const ROWS_MOST: usize = 1000;

/// Why a page's SQL was not run, said to the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// The authorizer refused it: what it tried.
    NotAllowed(String),
    /// It ran past [`QUERY_MOST`] and was stopped.
    TooLong,
    /// SQLite could not run it: its words.
    Malformed(String),
    /// This svrngs was built without cr-sqlite, so it opens no database.
    Unbuilt,
    /// This computer could not: its words.
    Fault(String),
}

impl Refused {
    pub fn status(&self) -> u16 {
        match self {
            Self::NotAllowed(_) => 403,
            Self::TooLong => 422,
            Self::Malformed(_) => 400,
            Self::Unbuilt | Self::Fault(_) => 500,
        }
    }
    pub fn words(&self) -> String {
        match self {
            Self::NotAllowed(what) => {
                format!("an app's SQL may only read and write its own tables, so {what} is refused")
            }
            Self::TooLong => format!(
                "the query ran past {} ms and was stopped; ask for less",
                QUERY_MOST.as_millis()
            ),
            Self::Malformed(why) => format!("SQLite could not run it: {why}"),
            Self::Unbuilt => "this computer's svrngs was built without cr-sqlite, so it keeps no app's \
                database; build it again from its checkout, as its README says"
                .into(),
            Self::Fault(why) => format!("this computer could not: {why}"),
        }
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

    /// `app`'s database in `group`, opened with `schema` the first time.
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
    pub path: PathBuf,
    #[expect(clippy::disallowed_types, reason = "one connection, one statement at a time")]
    conn: std::sync::Mutex<Connection>,
    tables: BTreeSet<String>,
    gate: Arc<Gate>,
    changed: watch::Sender<u64>,
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
    #[expect(clippy::disallowed_methods, reason = "a page's query is timed where SQLite runs it")]
    fn new() -> Self {
        let (page, until) = (AtomicBool::new(false), AtomicU64::new(u64::MAX));
        Self {
            page,
            until,
            since: Instant::now(),
        }
    }

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
        let mut conn = Connection::open(path).map_err(fault)?;
        // cr-sqlite, loaded on each connection by SQLite itself (shim.c): a
        // build without the shim holds none of its functions, and a database
        // opened there would merge nothing, so none is.
        let loaded = conn.query_row("SELECT crsql_site_id()", [], |_| Ok(()));
        tracing::debug!(target: "state", group, app, loaded = loaded.is_ok(), "state: a database opened only where cr-sqlite is loaded");
        if let Err(e) = loaded {
            tracing::error!(target: "state", group, app, %e, "state: this svrngs was built without cr-sqlite, so no database opens");
            return Err(Refused::Unbuilt);
        }
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(fault)?;
        let fresh: bool = conn
            .query_row(
                "SELECT count(*) = 0 FROM sqlite_master WHERE name = 'svrngs_sets'",
                [],
                |r| r.get(0),
            )
            .map_err(fault)?;
        tracing::debug!(target: "state", group, app, fresh, "state: a database opened; its schema applied only when new");
        if fresh {
            conn.authorizer(Some(schema_rule)).map_err(fault)?;
            let applied = conn.execute_batch(schema);
            conn.authorizer(None::<fn(AuthContext<'_>) -> Authorization>)
                .map_err(fault)?;
            applied.map_err(|e| Refused::Malformed(format!("schema.sql: {e}")))?;
        }
        let mut tables = BTreeSet::new();
        {
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
                .map_err(fault)?;
            let names = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(fault)?;
            for name in names {
                let name = name.map_err(fault)?;
                if !internal(&name) {
                    tables.insert(name);
                }
            }
        }
        if fresh {
            for t in &tables {
                conn.query_row("SELECT crsql_as_crr(?1)", [t], |_| Ok(()))
                    .map_err(|e| Refused::Malformed(format!("{t}: {e}")))?;
            }
            conn.execute_batch(
                "CREATE TABLE svrngs_sets (device TEXT NOT NULL, seq INTEGER NOT NULL, site BLOB NOT NULL, db_version INTEGER NOT NULL, body TEXT NOT NULL, sig TEXT NOT NULL, PRIMARY KEY (device, seq));
                 CREATE TABLE svrngs_marks (device TEXT PRIMARY KEY, mark INTEGER NOT NULL);
                 CREATE TABLE svrngs_sites (device TEXT PRIMARY KEY, site BLOB NOT NULL UNIQUE);",
            )
            .map_err(fault)?;
        }
        let gate = Arc::new(Gate::new());
        conn.authorizer(Some(page_rule(tables.clone(), gate.clone())))
            .map_err(fault)?;
        let timed = gate.clone();
        conn.progress_handler(1000, Some(move || timed.now() > timed.until.load(Relaxed)))
            .map_err(fault)?;
        Ok(Self {
            group: group.into(),
            app: app.into(),
            path: path.into(),
            conn: conn.into(),
            tables,
            gate,
            changed: watch::Sender::new(0),
        })
    }

    /// Told each time this database changes, by a write here.
    pub fn watch(&self) -> watch::Receiver<u64> {
        self.changed.subscribe()
    }

    /// The page's `sql` with `params`: a read's rows, or, when `writes`, a
    /// write's, one transaction of its own; with no `writes`, a write is
    /// refused (a read's own route, and a subscription). `{rows, more}`, and
    /// a write's `changed`.
    pub fn run(&self, writes: bool, sql: &str, params: &[Value]) -> Result<Value, Refused> {
        let conn = self.conn.lock().expect("the database's lock");
        let params: Vec<Sql> = params.iter().map(param).collect();
        self.gate.page.store(true, Relaxed);
        let prepared = conn.prepare(sql);
        self.gate.page.store(false, Relaxed);
        let mut stmt = prepared.map_err(said)?;
        let readonly = stmt.readonly();
        tracing::debug!(target: "state", group = self.group, app = self.app, readonly, writes, "state: a page's statement prepared under the authorizer; a read runs alone, a write in a transaction of its own, refused where no write is asked");
        if !readonly && !writes {
            return Err(Refused::NotAllowed("a write here, where only reads are asked".into()));
        }
        if !readonly {
            conn.execute_batch("BEGIN IMMEDIATE").map_err(fault)?;
        }
        let until = self.gate.now().saturating_add(QUERY_MOST.as_micros() as u64);
        self.gate.until.store(until, Relaxed);
        let ran = rows(&mut stmt, &params);
        self.gate.until.store(u64::MAX, Relaxed);
        drop(stmt);
        let (rows, more) = match ran {
            Ok(r) => r,
            Err(e) => {
                if !readonly {
                    conn.execute_batch("ROLLBACK").ok();
                }
                tracing::debug!(target: "state", group = self.group, app = self.app, refused = ?e, "state: a page's statement stopped, rolled back");
                return Err(e);
            }
        };
        if readonly {
            return Ok(json!({ "rows": rows, "more": more }));
        }
        let changed = conn.changes();
        conn.execute_batch("COMMIT").map_err(fault)?;
        drop(conn);
        tracing::debug!(target: "state", group = self.group, app = self.app, changed, "state: a page's write committed");
        if changed > 0 {
            self.changed.send_modify(|n| *n += 1);
        }
        Ok(json!({ "rows": rows, "more": more, "changed": changed }))
    }
}

/// A page's parameter as SQLite takes it.
fn param(v: &Value) -> Sql {
    match v {
        Value::Null => Sql::Null,
        Value::Bool(b) => Sql::Integer(i64::from(*b)),
        Value::Number(n) => n
            .as_i64()
            .map_or_else(|| Sql::Real(n.as_f64().unwrap_or_default()), Sql::Integer),
        Value::String(s) => Sql::Text(s.clone()),
        other => Sql::Text(other.to_string()),
    }
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
                ValueRef::Blob(b) => json!(hex::encode(b)),
            };
            cells.insert(name.clone(), cell);
        }
        out.push(Value::Object(cells));
    }
    Ok((out, false))
}
