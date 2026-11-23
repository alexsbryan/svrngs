// SPDX-License-Identifier: AGPL-3.0-or-later
// `journal_size_limit`: ported from superfly/corrosion crates/corro-types/src/sqlite.rs@e7295ff0 (`setup_conn`)
//! An app's database in a group: SQLite tables beside the group's record, a
//! copy on every member's computer, merged by cr-sqlite (svrn-docs
//! `rings/current/APP_STATE.md` §4).
//!
//! cr-sqlite is built from its source, the submodule `vendor/cr-sqlite`, by
//! `build.rs`, into one object linked here whose one global is a shim in C
//! (`shim.c`). SQLite calls the shim as it opens each connection, so cr-sqlite is loaded on
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
//! stops it at [`QUERY_MOST`], and under SQLite's own limit on a value,
//! [`ANSWER_MOST`] bytes, and gives at most [`ROWS_MOST`] rows, saying
//! when there were more. rusqlite takes both by `&mut`, so each is
//! installed once, at open, and the statement at hand is told them through
//! a [`Gate`].
//!
//! Moved from the app-state spike's `state.rs` (svrn-docs
//! `rings/current/app-state-spike/spike.patch`), the half that keeps a
//! database on one computer. Each write is sealed as a change set in its own
//! transaction, signed by this computer, and travels to the group's other
//! computers, which take it by the four outcomes (`sync.rs`). A database is
//! made with a nonce of its own, so its site, which names its writes, is
//! derived from its device's key and that nonce (`svrngs_core::set::site`).
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
use std::time::{Duration, Instant, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::types::{Value as Sql, ValueRef};
use rusqlite::{Connection, ErrorCode, TransactionBehavior::Immediate};
use serde_json::{Value, json};
use tokio::sync::watch;

mod form;
mod owned;
mod parse;
mod schema;
mod sync;
#[cfg(test)]
mod tests;
mod tidy;
mod wire;
pub use form::FORM;
use form::Table;
pub use parse::Unfit;
pub use schema::{Unmade, check};
pub use svrngs_core::SigningKey;
pub use sync::{Ask, Binds, Call, Device, Head, Held, Hex, Place, ROW_MOST, SIZE_MOST, Seq, Set, Sig, Took};

/// The app's schema, beside its `index.html`.
pub const SCHEMA: &str = "schema.sql";
/// The longest a page's statement runs before it is stopped.
pub const QUERY_MOST: Duration = Duration::from_millis(250);
/// The most rows a page is given at once.
pub const ROWS_MOST: usize = 1000;
/// The most bytes of text and blobs one read answers, past which it stops,
/// saying there were more: this number's one home. Provisional: the ask's
/// room (`svrngs-node` `peer_wire.rs`, `ASK_ROOM_BYTES`), [`ROWS_MOST`] rows
/// of 4 KiB, until Magnets' recorded answers set it.
pub const ANSWER_MOST: usize = 4 << 20;
/// The most columns a table, an index or a page's answer has, by SQLite's
/// own limit on every connection a page's statement or a schema runs on
/// ([`narrow`]): this number's one home. With [`VALUE_MOST`], every row a
/// copy holds fits one answer whole, and a sorted read of it.
pub const COLUMNS_MOST: usize = 32;
/// The most bytes of one value, or of a row's key, a copy keeps:
/// [`ANSWER_MOST`] over [`COLUMNS_MOST`], held where a value is taken, a
/// write, a phone's prepare and every copy's apply, by one parse
/// (`parse.rs`, `fit`), a fixed ceiling every copy shares.
pub const VALUE_MOST: usize = ANSWER_MOST / COLUMNS_MOST;

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
    /// This computer keeps no such database.
    #[error("This computer keeps no database of this app in this group.")]
    NotHere,
    /// A `schema.sql` that makes no database: what stopped it, why, and the
    /// fix.
    #[error("{0}")]
    Schema(schema::Unmade),
    /// A write that changes an owned row of the table named, not its
    /// person's (`owned.rs`).
    #[error(
        "This write changes a row of {0} whose person_key isn't yours, and only that person's own devices change it. To fix it, give the row your own you_key, from svrngs/acts, as its person_key."
    )]
    NotYours(String),
    /// A write that deletes another person's owned row of the table named
    /// before the office has cut every key of theirs (`owned.rs`).
    #[error(
        "This write deletes a row of {0} that is another person's, and only their own devices change it until your group's keeper removes them. To fix it, delete only your own rows: theirs can go once they're removed."
    )]
    NotYetGone(String),
    /// A write whose set no other computer would take: why, and what to do.
    #[error("This write isn't kept, since no other computer would take it: {0}.")]
    Unkept(Unfit),
    /// A write past this computer's share of the app's database, of the
    /// MiB named (`sync.rs`, `SIZE_MOST`).
    #[error(
        "This computer's writes fill its share of the {0} MB your group keeps of this app, so this write isn't kept. Delete rows you no longer need, then write again."
    )]
    Full(u64),
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
            Self::NotHere => (404, "not-here"),
            Self::Unkept(_) => (422, "unkept"),
            Self::Schema(_) => (422, "schema"),
            Self::NotYours(_) | Self::NotYetGone(_) => (403, "not-yours"),
            Self::Full(_) => (507, "full"),
        }
    }
}

/// What a page's POST asks: one read, `{sql, params}`, or a write in the
/// form, `{write: [op, ...]}`; or what a phone's worker asks.
pub enum Asked<'a> {
    Read(&'a str, &'a [Value]),
    Write(&'a Value),
    /// A phone's write, `{prepare: [op, ...]}`, for its signer ([`Db::prepare`]).
    Prepare(&'a Value),
    /// A set a phone signed, `{carry}`, postcard's bytes in base64 ([`Db::carry`]).
    Carry(Set),
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
        (Some(["prepare"]), ..) => Ok(Asked::Prepare(&body["prepare"])),
        (Some(["carry"]), ..) => {
            let bytes = body["carry"].as_str().and_then(|b| STANDARD.decode(b).ok());
            let set = bytes.and_then(|b| postcard::from_bytes(&b).ok());
            set.map(Asked::Carry).ok_or_else(|| {
                Refused::Form("{carry} is a set, postcard's bytes in base64, as a phone's signer makes one.".into())
            })
        }
        (Some(["sql"] | ["params", "sql"]), Some(sql), Some(params)) => Ok(Asked::Read(sql, params)),
        _ => Err(Refused::Form(format!(
            "svrngs/sql takes {{sql, params}}, one read, its params a list of values, or {{write: [op, ...]}}, not {body}. {FORM}"
        ))),
    }
}

/// A database's name in its directory, its group's or its app's: one part
/// of a path under a folder, as the folder's own rule reads one
/// (`svrngs_core::SyncPath::parse`), and no `/`. So it is never empty, `.`
/// or `..`, and holds no `\`, NUL or drive, and a database is opened under
/// its group's directory and nowhere else, as `Path::join` would put one
/// named by an absolute path wherever it names. Parsed where a name comes
/// in, the wire's included.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct Name(String);

impl Name {
    /// `text` as a name, if it is a plain one.
    pub fn parse(text: &str) -> Option<Self> {
        let part = !text.contains('/') && svrngs_core::SyncPath::parse(text).is_ok();
        part.then(|| Self(text.into()))
    }
}

svrngs_core::text_newtype!(Name);

impl TryFrom<String> for Name {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        Self::parse(&text).ok_or_else(|| "not a plain name".into())
    }
}

/// This computer's app databases, by group and app, and the key that signs
/// their sets.
pub struct States {
    dir: PathBuf,
    key: SigningKey,
    #[expect(clippy::disallowed_types, reason = "each database's place, found once")]
    dbs: std::sync::Mutex<BTreeMap<(String, String), Opened>>,
}

/// A database's place among [`States`]': opened once, by the first to ask,
/// while any other who asks for it meanwhile waits for that one, and no
/// one else does; refused, the place is let go, so the next open tries it
/// again.
type Opened = Arc<std::sync::OnceLock<Result<Arc<Db>, Refused>>>;

impl States {
    pub fn new(data_dir: &Path, key: SigningKey) -> Self {
        Self {
            dir: data_dir.join("state"),
            key,
            dbs: Default::default(),
        }
    }

    /// `app`'s database in `group`, made with `schema` the first time, or,
    /// with none, only one already on this computer's disk; refused, a new
    /// one keeps none of it, and the next open tries it again. One made here
    /// is synced only by a synced copy's full call.
    pub fn open(&self, group: &str, app: &str, schema: Option<&str>) -> Result<Arc<Db>, Refused> {
        self.made(group, app, (schema, false))
    }

    /// `app`'s database in `group` made from `schema` synced from the start,
    /// as the computer that carries its schema first makes it, as it carries
    /// it: the root every other copy is synced from (APP_STATE §7, invariant
    /// 8). One already on this disk is opened as it is.
    pub fn carry(&self, group: &str, app: &str, schema: &str) -> Result<Arc<Db>, Refused> {
        self.made(group, app, (Some(schema), true))
    }

    /// [`Self::open`] and [`Self::carry`]: a new one made `root` or not, made
    /// again where it was copied, in its place ([`Opened`]), the map of
    /// places held only to find it.
    fn made(&self, group: &str, app: &str, (schema, root): (Option<&str>, bool)) -> Result<Arc<Db>, Refused> {
        let names = Name::parse(group).zip(Name::parse(app));
        tracing::debug!(target: "state", group, app, plain = names.is_some(), "state: an app's database asked by its group's and app's names, opened only by plain ones");
        let path = names
            .map(|(g, a)| self.dir.join(g.as_str()).join(format!("{a}.db")))
            .ok_or(Refused::NotHere)?;
        let named = (group.to_owned(), app.to_owned());
        let mut dbs = self.dbs.lock().expect("the databases' lock");
        let place = dbs.entry(named.clone()).or_default().clone();
        drop(dbs);
        let opened = place.get_or_init(|| {
            tracing::debug!(target: "state", group, app, made = schema.is_some(), on_disk = path.exists(), "state: an app's database asked, made from its schema if new, or only one on disk");
            if schema.is_none() && !path.exists() {
                return Err(Refused::NotHere);
            }
            std::fs::create_dir_all(path.parent().unwrap_or(&path)).map_err(fault)?;
            let db = Arc::new(Db::open(&path, (group, app), (schema, root), self.key.clone())?);
            db.copied().then(|| self.remake(&db)).transpose()?;
            tracing::debug!(target: "state", group, app, "state: an app's database opened, its schema applied and its tables made CRRs");
            Ok(db)
        });
        let mut dbs = self.dbs.lock().expect("the databases' lock");
        let gone = opened.is_err() && dbs.get(&named).is_some_and(|p| Arc::ptr_eq(p, &place));
        gone.then(|| dbs.remove(&named));
        opened.clone()
    }

    /// `db`, a file found copied ([`Db::copied`]), made again with a fresh
    /// nonce, so a site of its own (svrn-docs
    /// `app-state-diligence/sync-simplify-2026-11-22/`, change 5): restored
    /// whole ([`Self::restored`]), then on the copy its own site made one it
    /// retired, another's to its clocks, kept as it was with its sets and
    /// mark, and a new nonce's site in its place.
    pub fn remake(&self, db: &Db) -> Result<(), Refused> {
        tracing::warn!(target: "state", group = db.group, app = db.app, "state: a database found copied from another computer's is made again with a fresh nonce");
        let (old, nonce) = (db.conn().file.site, drawn()?);
        let new = svrngs_core::set::site(&db.group, &db.app, db.device.as_str(), &nonce);
        self.restored(db, |conn| {
            let tables = tables_of(conn)?;
            let tx = conn.unchecked_transaction().map_err(fault)?;
            let next = "SELECT max(ordinal) + 1 FROM crsql_site_id";
            let next: i64 = tx.query_row(next, [], |r| r.get(0)).map_err(fault)?;
            let run = |sql: &str, p: &[&dyn rusqlite::ToSql]| tx.execute(sql, p).map(drop).map_err(fault);
            run("UPDATE crsql_site_id SET site_id = ?1 WHERE ordinal = 0", &[&new])?;
            run(
                "INSERT INTO crsql_site_id (site_id, ordinal) VALUES (?1, ?2)",
                &[&old, &next],
            )?;
            for t in tables.keys().map(|t| t.replace('"', "\"\"")) {
                run(
                    &format!("UPDATE \"{t}__crsql_clock\" SET site_id = ?1 WHERE site_id = 0"),
                    &[&next],
                )?;
            }
            run(
                "UPDATE svrngs_db SET nonce = ?1, copied = 0, retired = ?2",
                &[&nonce, &old],
            )?;
            tx.commit().map_err(fault)
        })
    }

    /// `db` made wider by `schema`, a version a standing member runs, where
    /// it adds to what `db` holds: its file restored whole
    /// ([`Self::restored`]) and `schema` made on the copy, a wider version's
    /// tables, columns and indexes, each as a copy made from `schema` has
    /// it (`schema.rs`, `widen`). Nothing where `schema` adds nothing, and
    /// refused where it doesn't only add. Judged once against each file
    /// `db` holds: a version that adds nothing, is refused or whose widening
    /// faulted is answered as it was, until the file is made wider or
    /// svrngs starts again, so no member's version is made again at every
    /// round and every page's ask.
    pub fn widen(&self, db: &Db, schema: &str) -> Result<(), Refused> {
        let hash = svrngs_core::set::schema(schema);
        let conn = db.conn();
        if let Some(judged) = conn.judged.get(&hash) {
            return judged.clone();
        }
        let adds = schema::adds(&conn, schema);
        drop(conn);
        tracing::debug!(target: "state", group = db.group, app = db.app, ?adds, "state: a version's schema judged against a database's file for the first time, made wider where it adds to it");
        let judged = match adds {
            Ok(true) => self.restored(db, |conn| schema::widen(conn, schema)),
            adds => adds.map(drop),
        };
        db.conn().judged.insert(hash, judged.clone());
        judged
    }

    /// `db`'s file restored whole, `change` made on the restored copy, and
    /// the copy put in its place once it opens as this computer's (svrn-docs
    /// `app-state-diligence/e-build-2026-11-22/`, from the operator's review
    /// of `41d94397`): every row, clock, mark and kept set carried as the
    /// file holds them, never replayed from the sets it
    /// keeps, which prune has thinned. All under `db`'s one lock, so no
    /// statement runs on the file between the copy and the swap, and every
    /// holder of `db`, a page's subscription among them, has the new file at
    /// its next ask, told it changed. On any failure before the copy opens
    /// in the file's place, the old file stays, open as it was, and the copy
    /// goes: `db` never holds no connection.
    fn restored(&self, db: &Db, change: impl FnOnce(&Connection) -> Result<(), Refused>) -> Result<(), Refused> {
        let mut live = db.conn();
        let (names, path) = (
            (db.group.as_str(), db.app.as_str()),
            self.dir.join(&db.group).join(format!("{}.db", db.app)),
        );
        let copy = path.with_extension("db.new");
        let _ = std::fs::remove_file(&copy);
        let made = (|| {
            live.execute("VACUUM INTO ?1", [copy.to_string_lossy()])
                .map_err(fault)?;
            let restored = Open(Some(Connection::open(&copy).map_err(fault)?));
            narrow(&restored)?;
            change(&restored)?;
            // The copy's own identity, taken as it is first opened (`Db::open`).
            let own = restored.execute_batch("UPDATE svrngs_db SET file = NULL");
            own.map_err(fault)?;
            drop(restored);
            drop(Db::open(&copy, names, (None, false), self.key.clone())?);
            let closed = !copy.with_extension("new-wal").exists();
            closed
                .then_some(())
                .ok_or_else(|| fault("the restored copy's WAL stayed open"))
        })();
        // Then swapped in only once it opens in the live file's place: the
        // live file's WAL folded into it and gone, its connection kept open
        // on it in rollback mode, so the copy opens with WAL files of its
        // own; the live file kept under a second name meanwhile ([`moves`]),
        // so the app's name always holds a whole database, and put back
        // where the copy doesn't open, its connection still on it. So `live`
        // always holds a connection that works, the old one or the copy's.
        let old = path.with_extension("db.old");
        let moved = made.and_then(|()| {
            live.execute_batch("PRAGMA journal_mode=DELETE").map_err(fault)?;
            let _ = std::fs::remove_file(&old);
            let mut moves = moves(&path, &copy, &old).into_iter();
            moves.try_for_each(|(step, from, to)| step(from, to)).map_err(fault)
        });
        let swapped = moved
            .clone()
            .and_then(|()| Db::open(&path, names, (None, false), self.key.clone()));
        tracing::debug!(target: "state", group = db.group, app = db.app, refused = ?swapped.as_ref().err(), "state: a database restored whole into a copy, changed there and opened as this computer's in its place, then swapped in under its lock; or left as it was, open, the copy refused");
        let new = swapped.inspect_err(|_| {
            if moved.is_ok() {
                // What the copy's failed open left of its WAL, then the old file back.
                let _ = ["db-wal", "db-shm"].map(|g| std::fs::remove_file(path.with_extension(g)));
                let _ = std::fs::rename(&old, &path);
            }
            let _ = std::fs::remove_file(&copy);
        })?;
        let _ = std::fs::remove_file(&old);
        (db.synced).store(new.synced.load(Relaxed), Relaxed);
        db.copied.store(new.copied(), Relaxed);
        *live = new.live.into_inner().expect("a lock just made");
        drop(live);
        db.changed.send_modify(|n| *n += 1);
        Ok(())
    }

    /// Each database of `group` this computer has open closed, as it drops,
    /// and its file left on disk: a group this computer left, or was removed
    /// from, keeps its other files held and readable here, none of it live
    /// (svrngs-node removal.rs `leave`). One a page's subscription still
    /// holds closes as that ends. How many it let go.
    pub fn close(&self, group: &str) -> usize {
        let mut dbs = self.dbs.lock().expect("the databases' lock");
        let open = dbs.len();
        dbs.retain(|(g, _), _| g != group);
        let closed = open - dbs.len();
        tracing::debug!(target: "state", group, closed, "state: a group's databases closed, as this computer stands in it no more, their files kept");
        closed
    }

    /// Every database this computer keeps in `group`, as it syncs them:
    /// each one on its disk, opened if it isn't yet. One that won't open
    /// is left out, said at warn.
    pub fn held(&self, group: &str) -> Vec<Arc<Db>> {
        let dir = Name::parse(group).map(|g| self.dir.join(g.as_str()));
        let files = dir
            .and_then(|d| std::fs::read_dir(d).ok())
            .into_iter()
            .flatten()
            .flatten();
        let apps = files.filter_map(|f| Some(f.file_name().to_str()?.strip_suffix(".db")?.to_owned()));
        let opened = apps.map(|app| (self.open(group, &app, None), app));
        let said = |(db, app): (Result<_, _>, String)| {
            db.inspect_err(|e| tracing::warn!(target: "state", group, app, %e, "state: a database on this disk won't open, so it isn't synced")).ok()
        };
        opened.filter_map(said).collect()
    }
}

/// One step of a restore on disk, from one name to another.
type Move = fn(&Path, &Path) -> std::io::Result<()>;

/// A restore's moves on disk, in order (`States::restored`): the live file
/// at `path` given a second name, `old`, then the restored `copy` renamed
/// onto `path`, which is atomic. So after each, as after a crash between
/// them, `path` names a whole database, the live one and then the copy, and
/// a later restore drops what `old` still names.
fn moves<'a>(path: &'a Path, copy: &'a Path, old: &'a Path) -> [(Move, &'a Path, &'a Path); 2] {
    [
        (|p, o| std::fs::hard_link(p, o), path, old),
        (|c, p| std::fs::rename(c, p), copy, path),
    ]
}

/// One app's database in one group on this computer: its device, the key
/// that signs its sets, the nonce it was made with and its site, and the
/// hash of the schema it was made from.
pub struct Db {
    pub group: String,
    pub app: String,
    device: Device,
    key: SigningKey,
    /// Its file and what the file names, under the one lock every statement
    /// takes, so a file made again or wider is swapped in whole, under it,
    /// and every holder of this `Db` has the new one at its next ask (svrn-docs
    /// `app-state-diligence/invariants-2026-11-22/` §2.2).
    #[expect(clippy::disallowed_types, reason = "one connection, one statement at a time")]
    live: std::sync::Mutex<Live>,
    changed: watch::Sender<u64>,
    /// Whether it has synced in full once, and whether a set faulted since.
    synced: AtomicBool,
    faulted: AtomicBool,
    /// Whether a set sealed by another file of its nonce and key reached it:
    /// it was copied, and is to be made again (`States::remake`).
    copied: AtomicBool,
    /// Each bound key's share of the ask's rows, and of the database, as
    /// the record gave them at this copy's last call (`sync.rs`, `room`).
    shares: [AtomicU64; 2],
}

/// A connection cr-sqlite is loaded on, closed as it drops ([`close`]),
/// whether its database opened or was refused.
struct Open(Option<Connection>);

/// A `Db`'s file as it holds it: its connection, the query gate the
/// connection's handlers read, what the file names, and how each version's
/// schema judged against it went (`States::widen`).
pub(crate) struct Live {
    open: Open,
    gate: Arc<Gate>,
    pub(crate) file: File,
    judged: BTreeMap<[u8; 32], Result<(), Refused>>,
}

/// What a database's file names: the nonce it was made with, its site, its
/// schema's hash, and its tables.
#[derive(Clone)]
pub(crate) struct File {
    pub(crate) nonce: svrngs_core::set::Nonce,
    pub(crate) site: svrngs_core::set::Site,
    pub(crate) schema: [u8; 32],
    pub(crate) tables: BTreeMap<String, Table>,
}

impl std::ops::Deref for Live {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        &self.open
    }
}

impl std::ops::DerefMut for Live {
    fn deref_mut(&mut self) -> &mut Connection {
        &mut self.open
    }
}

impl std::ops::Deref for Open {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        self.0.as_ref().expect("a connection is open until it drops")
    }
}

/// Mutably, so a transaction or savepoint borrows it as a guard that rolls
/// back as it drops, on every path that doesn't commit it.
impl std::ops::DerefMut for Open {
    fn deref_mut(&mut self) -> &mut Connection {
        self.0.as_mut().expect("a connection is open until it drops")
    }
}

impl Drop for Open {
    fn drop(&mut self) {
        let Some(conn) = self.0.take() else { return };
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

/// `conn` held to [`COLUMNS_MOST`] columns in a table, an index and a
/// statement's answer, by SQLite's own limit, which refuses one of more as
/// it is prepared, so no read makes a row of more values at once.
fn narrow(conn: &Connection) -> Result<(), Refused> {
    let columns = rusqlite::limits::Limit::SQLITE_LIMIT_COLUMN;
    conn.set_limit(columns, COLUMNS_MOST as i32).map(drop).map_err(fault)
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

/// What `schema.sql` may do: declare. Make tables and indexes in `main`,
/// with what SQLite reads of them and writes of its own schema to do it,
/// the `REINDEX` it asks for as it makes an index included (svrn-docs
/// `magnets-spikes/README.md`, S5), write no row of any other table, and
/// call no function, so no expression of it makes data on any copy
/// (`schema.rs`, `limits`, holds the rest). What it refuses is said by the
/// check (`schema.rs`, [`schema::Unmade`]).
fn schema_rule(ctx: AuthContext<'_>) -> Authorization {
    let main = ctx.database_name.is_none_or(|d| d == "main");
    let own = |t: &str| t == "sqlite_master" || t == "sqlite_schema";
    let allowed = main
        && match ctx.action {
            AuthAction::CreateTable { table_name } | AuthAction::CreateIndex { table_name, .. } => {
                !internal(table_name)
            }
            AuthAction::Reindex { index_name } => !internal(index_name),
            AuthAction::Insert { table_name } | AuthAction::Update { table_name, .. } => own(table_name),
            AuthAction::Read { table_name, .. } => own(table_name) || !internal(table_name),
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
                        // json(b)_group_array and _object grow with no length check, so only the deadline
                        // would stop one; a page builds that JSON from the rows it reads.
                        f != "load_extension"
                            && !f.starts_with("crsql")
                            && f != "fts3_tokenizer"
                            && !f.contains("_group_")
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
    fn open(
        path: &Path,
        names: (&str, &str),
        (schema, root): (Option<&str>, bool),
        key: SigningKey,
    ) -> Result<Self, Refused> {
        let (group, app) = names;
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
        narrow(&open)?;
        let conn = open.0.as_mut().expect("just opened");
        // Corrosion's: a WAL folded in is cut back to 1 GiB (`tidy.rs`). And
        // the fixed ceiling no file passes, 1 GiB in SQLite's 4 KiB pages,
        // far above every share of `SIZE_MOST`, which may rise to it: a copy
        // full to it faults a set, its mark held, and refuses none.
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA journal_size_limit=1073741824; PRAGMA max_page_count=262144;")
            .map_err(fault)?;
        // A database without svrngs's own tables is new: its schema applied,
        // its tables made cr-sqlite's, and svrngs's made, its nonce and site
        // with them, in one transaction, begun and ended with no authorizer,
        // as `schema_rule` refuses one in `schema.sql`. Refused at any step,
        // none of it is kept, so the next open finds the database new again,
        // and refuses it in the same words.
        let device = svrngs_core::actor_of(&key);
        let new = "SELECT count(*) = 0 FROM sqlite_master WHERE name = 'svrngs_db'";
        let fresh: bool = conn.query_row(new, [], |r| r.get(0)).map_err(fault)?;
        tracing::debug!(target: "state", group, app, fresh, "state: a database opened; its schema applied only when new");
        if fresh {
            let schema = schema.ok_or(Refused::NotHere)?;
            // Its pages given back as sets are pruned (`tidy.rs`): set before
            // the first table, as Corrosion sets it (corro-agent
            // `setup.rs:89-91`), but cr-sqlite made its own as the connection
            // opened, so the file is made again under it.
            conn.execute_batch("PRAGMA auto_vacuum=INCREMENTAL; VACUUM;")
                .map_err(fault)?;
            conn.execute_batch("BEGIN IMMEDIATE").map_err(fault)?;
            let made = new_tables(conn, (schema, root), (group, app, &device));
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
            made?;
            // cr-sqlite reads its site as a connection opens
            // (`vendor/cr-sqlite/core/rs/core/src/bootstrap.rs:101-120`), so
            // the new database is opened again, with its own.
            drop(open);
            return Self::open(path, names, (None, false), key);
        }
        let tables = tables_of(conn)?;
        let own = "SELECT nonce, schema, synced, crsql_site_id(), copied FROM svrngs_db";
        let own = conn.query_row(own, [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, Vec<u8>>(3)?, r.get(4)?))
        });
        let (nonce, hash, synced, site, copied): ([u8; 16], [u8; 32], bool, _, bool) = own.map_err(fault)?;
        // The file's own identity: one that isn't the one recorded as it was
        // made, or as svrngs last swapped one in (`States::restored`), was
        // copied, restored or moved, and is made again before it serves
        // anything (`States::made`; svrn-docs
        // `app-state-diligence/invariants-2026-11-22/` §2.4). One with none
        // recorded takes its own.
        let same = "UPDATE svrngs_db SET file = coalesce(file, ?1) RETURNING file = ?1";
        let same: bool = conn.query_row(same, [identity(path)?], |r| r.get(0)).map_err(fault)?;
        let copied = copied || !same;
        let derived = svrngs_core::set::site(group, app, &device, &nonce);
        tracing::debug!(target: "state", group, app, own = site == derived, "state: a database's site is its own, derived from this computer's key and the nonce it was made with, or it was made under another key");
        if site != derived {
            return Err(Refused::Fault(
                "this app's database here was made under another key".into(),
            ));
        }
        // Equal values tie on site (APP_STATE §4, "How it merges").
        conn.query_row("SELECT crsql_config_set('merge-equal-values', 1)", [], |_| Ok(()))
            .map_err(fault)?;
        conn.execute_batch(&sync::recorded(&tables)).map_err(fault)?;
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
            device: Device::parse(&device).expect("a key is written in lower-case hex"),
            key,
            live: Live {
                open,
                gate,
                file: File {
                    nonce,
                    site: derived,
                    schema: hash,
                    tables,
                },
                // The schema it was made from, judged as it was made.
                judged: BTreeMap::from([(hash, Ok(()))]),
            }
            .into(),
            changed: watch::Sender::new(0),
            synced: synced.into(),
            faulted: false.into(),
            copied: copied.into(),
            shares: [AtomicU64::new(u64::MAX), AtomicU64::new(u64::MAX)],
        })
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, Live> {
        self.live.lock().expect("the database's lock")
    }

    /// Whether this file was found copied, and is to be made again.
    pub fn copied(&self) -> bool {
        self.copied.load(Relaxed)
    }

    /// Whether this copy is still to be said syncing: it has never synced in
    /// full with a member, or a set faulted here since its last full call.
    pub fn syncing(&self) -> bool {
        !self.synced.load(Relaxed) || self.faulted.load(Relaxed)
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
        let conn = self.conn();
        let gate = conn.gate.clone();
        gate.page.store(true, Relaxed);
        let prepared = conn.prepare(sql);
        gate.page.store(false, Relaxed);
        let mut stmt = prepared.map_err(said)?;
        tracing::debug!(target: "state", group = self.group, app = self.app, readonly = stmt.readonly(), "state: a page's statement prepared under the authorizer; it runs only if it only reads");
        if !stmt.readonly() {
            return Err(Refused::NotARead);
        }
        // Each value it makes at most `ANSWER_MOST`, refused by SQLite
        // (`SQLITE_TOOBIG`) before it is made, as no copy keeps a larger one
        // (`parse.rs`); svrngs's own statements, a set applied among them,
        // run under SQLite's, so a merged row is never refused on one copy.
        // The limit is set before the deadline, and both are put back with
        // no return between, so no path out of a read leaves either.
        let length = rusqlite::limits::Limit::SQLITE_LIMIT_LENGTH;
        let was = conn.set_limit(length, ANSWER_MOST as i32).map_err(fault)?;
        gate.until
            .store(gate.now().saturating_add(QUERY_MOST.as_micros() as u64), Relaxed);
        let ran = rows(&mut stmt, &params);
        gate.until.store(u64::MAX, Relaxed);
        let _ = conn.set_limit(length, was);
        tracing::debug!(target: "state", group = self.group, app = self.app, refused = ?ran.as_ref().err(), "state: a page's read answered, or stopped");
        let (rows, more) = ran?;
        Ok(json!({ "rows": rows, "more": more }))
    }

    /// The page's write, `ops` in the write form, as one transaction:
    /// `{count}`, the rows changed in all, and the seq of the set it sealed,
    /// none where it changed nothing; or refused, nothing of it kept, as
    /// where it changes an owned row the record `binds` doesn't give this
    /// computer's person (`owned.rs`).
    pub fn write(&self, ops: &Value, binds: &Binds) -> Result<(Value, Option<Seq>), Refused> {
        let (sealed, count) = self.run(ops, false, |tx, file, count, _| {
            // Its set sealed in its own transaction (APP_STATE §4): refused,
            // the write is rolled back with it.
            let sealed = self.seal((&tx, file), binds)?;
            tx.commit().map_err(fault)?;
            Ok((sealed, count))
        })?;
        if count > 0 {
            self.changed.send_modify(|n| *n += 1);
        }
        Ok((json!({ "count": count }), sealed))
    }

    /// A phone's write, `ops`, prepared for its signer, as a phone holds no
    /// copy (svrn-docs APP_STATE §4, "Phones"; MAGNETS_BUILD S2): run as
    /// [`Self::write`] runs one, in a whole transaction rolled back, never a
    /// savepoint in one that commits, which spends a version, so clocks and
    /// `db_version` stay as they were. `{prepared, by, sig}`, whether or not
    /// it changes anything: this computer's signature over `prepared`, the
    /// text it is ([`svrngs_core::set::prepared`]), so the signer takes it
    /// from a computer and never from the app's page, and checks every
    /// prepare, one with no change rows too. Its
    /// text holds the ops, the set's changes as the parse every copy runs
    /// takes them, the schema's hash, each table's key columns in order, the
    /// rows changed, each op's row as it stood before them, null where there
    /// was none, so an op with no change row is a no-op only where its row
    /// already stood as the op would leave it (§4, "A no-op is told from a
    /// missing effect"). No place: the phone's signer picks the place it
    /// signs the set at, as this computer, its home, keeps no count of the
    /// phone's (svrn-docs `app-state-diligence/invariants-2026-11-22/`,
    /// change 5). Refused, as a write is, where it changes an owned row the
    /// record `binds` doesn't give the phone, `phone`, its person.
    pub fn prepare(&self, ops: &Value, (phone, binds): (&str, &Binds)) -> Result<Value, Refused> {
        let (made, stands, count, file) = self.run(ops, true, |tx, file, count, stands| {
            let seq = tx.query_row("SELECT crsql_peek_next_db_version()", [], |r| r.get(0));
            let made = self.changes(&tx, seq.map_err(fault)?).map_err(fault)?;
            Ok((made, stands, count, file.clone()))
        })?;
        tracing::debug!(target: "state", group = self.group, app = self.app, rows = made.len(), count, "state: a phone's write prepared and rolled back whole, its rows for its signer, none where it changes nothing");
        let made = parse::fit(made, &file.tables).map_err(Refused::Unkept)?;
        // Its owned rows judged as every copy judges the phone's set again.
        let device = Device::parse(phone).ok_or_else(|| fault("a phone's key is lower-case hex"))?;
        owned::mine(&made, &file.tables, binds, &device)?;
        let keys: BTreeMap<_, _> = file.tables.iter().map(|(t, table)| (t, &table.key)).collect();
        let changes = STANDARD.encode(postcard::to_allocvec(&made.0).expect("changes are written in memory"));
        let schema = STANDARD.encode(file.schema);
        let text =
            json!({ "ops": ops, "changes": changes, "schema": schema, "keys": keys, "count": count, "stands": stands });
        let text = text.to_string();
        let sig = svrngs_core::sign_message(&self.key, &svrngs_core::set::prepared(&self.group, &self.app, &text));
        Ok(json!({ "prepared": text, "by": self.device, "sig": sig }))
    }

    /// `ops`, in the write form, run whole in one transaction, which `then`
    /// is handed with the rows they changed and, where it `stands`, each
    /// op's row as it stood before, to commit or drop; rolled back as it
    /// drops on any refusal, or a COMMIT that fails (rusqlite 0.41
    /// `transaction.rs:220-229`), so no write leaves its transaction open.
    fn run<T>(
        &self,
        ops: &Value,
        stands: bool,
        then: impl FnOnce(rusqlite::Transaction<'_>, &File, usize, Vec<Option<Value>>) -> Result<T, Refused>,
    ) -> Result<T, Refused> {
        let listed = ops.as_array().filter(|o| !o.is_empty());
        let listed =
            listed.ok_or_else(|| Refused::Form(format!("write takes a list of one or more ops, not {ops}. {FORM}")))?;
        let mut conn = self.conn();
        let file = conn.file.clone();
        let made = |(i, op)| form::statement(op, &file.tables, i);
        let statements = listed.iter().enumerate().map(made).collect::<Result<Vec<_>, _>>();
        tracing::debug!(target: "state", group = self.group, app = self.app, ops = listed.len(), refused = ?statements.as_ref().err(), "state: a write's ops made statements by key, or refused");
        let statements = statements?;
        let tx = conn.transaction_with_behavior(Immediate).map_err(fault)?;
        let stood = statements.iter().filter(|_| stands).map(|(_, (sql, key))| {
            let mut row = tx.prepare(sql).map_err(said)?;
            Ok(rows(&mut row, key)?.0.into_iter().next())
        });
        let stood = stood.collect::<Result<Vec<_>, Refused>>()?;
        let ran = (statements.iter()).map(|((sql, values), _)| tx.execute(sql, rusqlite::params_from_iter(values)));
        let ran = ran.sum::<Result<usize, _>>();
        tracing::debug!(target: "state", group = self.group, app = self.app, ?ran, "state: a write run whole, or rolled back whole as SQLite refused it");
        let count = ran.map_err(said)?;
        then(tx, &file, count, stood)
    }
}

/// svrngs's own tables in a database, made with it: the nonce it was made
/// with, its schema's hash, whether it has synced in full once, whether it
/// was found copied, and the site it retired when it was made again
/// (`States::remake`); the sets
/// it keeps, as postcard's bytes (APP_STATE §4, "Sealed and signed"), named
/// by site, seq and signature so two at one seq are both kept, and how each
/// is kept (`sync.rs`, `Kept`), a refused one with no bytes; each site's
/// mark, and the sets counted above a mark; each site's device and its home
/// (`sync.rs`, `keep`). No count of a phone's places: its signer keeps its
/// own (svrn-docs `app-state-diligence/invariants-2026-11-22/`, change 5). No
/// device is stored muted: one over its share is worked out from what it
/// holds now (`sync.rs`, `Db::held`).
const OWN: &str = "CREATE TABLE svrngs_db (nonce BLOB NOT NULL, schema BLOB NOT NULL, synced INTEGER NOT NULL,
      copied INTEGER NOT NULL DEFAULT 0, retired BLOB, file TEXT);
    CREATE TABLE svrngs_sets (site BLOB NOT NULL, seq INTEGER NOT NULL, sig TEXT NOT NULL, kept INTEGER NOT NULL,
      bytes BLOB, PRIMARY KEY (site, seq, sig));
    CREATE TABLE svrngs_marks (site BLOB PRIMARY KEY, mark INTEGER NOT NULL);
    CREATE TABLE svrngs_above (site BLOB NOT NULL, seq INTEGER NOT NULL, PRIMARY KEY (site, seq));
    CREATE TABLE svrngs_sites (site BLOB PRIMARY KEY, device TEXT NOT NULL, home TEXT NOT NULL);";

/// A new database's tables: `schema` made as `svrngs check` makes it
/// (`schema.rs`, `made`), and svrngs's own made ([`OWN`]), with a
/// nonce drawn for it and its site, `device`'s with that nonce, and synced
/// from the start where it is the `root`; or the first refusal.
fn new_tables(conn: &mut Connection, (schema, root): (&str, bool), names: (&str, &str, &str)) -> Result<(), Refused> {
    let (group, app, device) = names;
    schema::made(conn, schema)?;
    let nonce = drawn()?;
    let site = svrngs_core::set::site(group, app, device, &nonce);
    conn.execute_batch(OWN).map_err(fault)?;
    let own = "INSERT INTO svrngs_db (nonce, schema, synced) VALUES (?1, ?2, ?3)";
    conn.execute(own, rusqlite::params![nonce, svrngs_core::set::schema(schema), root])
        .map_err(fault)?;
    conn.execute("UPDATE crsql_site_id SET site_id = ?1 WHERE ordinal = 0", [site])
        .map_err(fault)
        .map(drop)
}

/// The identity of the file at `path`: its inode, and its birth time where
/// the filesystem keeps one, so a copy, a restore that makes a new file or a
/// move is a file of its own. A file overwritten in place, a disk's clone or
/// a rolled-back snapshot keeps both, and is found by what it writes
/// (`sync.rs`, `take`).
fn identity(path: &Path) -> Result<String, Refused> {
    use std::os::unix::fs::MetadataExt as _;
    let meta = std::fs::metadata(path).map_err(fault)?;
    let born = meta.created().map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default());
    Ok(format!("{}:{}", meta.ino(), born.unwrap_or_default().as_nanos()))
}

/// A nonce for a database made, or made again, drawn where it is made.
fn drawn() -> Result<svrngs_core::set::Nonce, Refused> {
    let mut nonce = [0; 16];
    #[expect(clippy::disallowed_methods, reason = "a database's nonce is drawn where it is made")]
    let drawn = getrandom::fill(&mut nonce);
    drawn.map(|()| nonce).map_err(fault)
}

/// Each of the app's tables in `conn`, with its columns and key, as the
/// write form names them, and its NOT NULL columns and each column's kind,
/// as the parse reads them.
fn tables_of(conn: &Connection) -> Result<BTreeMap<String, Table>, Refused> {
    let each = "SELECT t.name, c.name, c.pk, c.\"notnull\", c.type, l.strict FROM sqlite_master AS t, \
        pragma_table_info(t.name) AS c, pragma_table_list(t.name) AS l \
        WHERE t.type = 'table' AND l.schema = 'main' ORDER BY t.name, c.pk = 0, c.pk, c.cid";
    let mut stmt = conn.prepare(each).map_err(fault)?;
    let columns = stmt.query_map([], |r| {
        Ok(((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?), (r.get(4)?, r.get(5)?)))
    });
    let mut tables = BTreeMap::<String, Table>::new();
    for column in columns.map_err(fault)? {
        let ((name, column, pk, not_null), (declared, strict)): ((String, String, i64, bool), (String, bool)) =
            column.map_err(fault)?;
        let table = tables.entry(name).or_default();
        (pk > 0).then(|| table.key.push(column.clone()));
        not_null.then(|| table.not_null.push(column.clone()));
        table.kinds.insert(column.clone(), parse::Kind::of(&declared, strict));
        table.strict = strict;
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

/// The statement's rows, at most [`ROWS_MOST`] and [`ANSWER_MOST`] bytes of
/// their text and blobs, and whether there were more: counted cell by cell,
/// so the row that would pass it is left out whole, and no row is read
/// after it; but the first row answered whole, so a page always gets past
/// it. Every row a copy holds fits [`ANSWER_MOST`] ([`VALUE_MOST`]); a
/// row a statement works out holds at most [`COLUMNS_MOST`] values of it.
fn rows(stmt: &mut rusqlite::Statement<'_>, params: &[Sql]) -> Result<(Vec<Value>, bool), Refused> {
    let names: Vec<String> = stmt.column_names().map(String::from).collect();
    let mut found = stmt.query(rusqlite::params_from_iter(params)).map_err(said)?;
    let (mut out, mut size) = (Vec::new(), 0);
    while let Some(row) = found.next().map_err(said)? {
        let mut cells = serde_json::Map::new();
        for (i, name) in names.iter().enumerate() {
            let cell = row.get_ref(i).map_err(said)?;
            size += cell.as_bytes().map_or(8, <[u8]>::len);
            if out.len() == ROWS_MOST || size > ANSWER_MOST && !out.is_empty() {
                return Ok((out, true));
            }
            let cell = match cell {
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
