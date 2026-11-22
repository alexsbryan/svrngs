//! The live copy, `LiveCopy`: a real cr-sqlite database behind `Replica`,
//! Fly's fork at the pin `change.rs` names, built from the submodule and
//! loaded on every connection SQLite opens (`tests/crsqlite.rs`). The model
//! copy emulates the engine branch by branch (`replica.rs`); here the engine
//! itself is held to W1, and the `Pair` runs a model copy and a live copy in
//! lockstep and says where they first part (svrn-docs APP_STATE.md §8,
//! question 9: "The live engine is checked once slice A's build lands").
//! cr-sqlite is cited under `vendor/cr-sqlite/core/`.

// Linked for its object, which holds cr-sqlite and the shim SQLite calls.
extern crate svrngs_db;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

use rusqlite::types::{Value, ValueRef};
use rusqlite::{Connection, OpenFlags, Row, params, params_from_iter};

use crate::cases::Case;
use crate::change::{Change, ChangeSet, SetId, Site, State, Val};
use crate::replica::{ModelCopy, Op, Plant, Replica};
use crate::rule::Rule;
use crate::seeds::GATE;
use crate::sim::{self, SCHEMA, TABLES};

/// A table as the engine reads it from `SCHEMA`: its key columns in key
/// order, the order `crsql_pack_columns` packs them in, and its non-key
/// columns in schema order.
struct Shape {
    name: &'static str,
    keys: Vec<String>,
    cols: &'static [&'static str],
}

/// A copy whose engine is cr-sqlite itself, on an in-memory database of its
/// own.
pub struct LiveCopy {
    site: Site,
    conn: Connection,
    shapes: Vec<Shape>,
    kept: BTreeMap<SetId, ChangeSet>,
    plant: Plant,
}

/// Names each live copy's database apart from every other in this process.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// What crsql_changes is read by: its columns but `ts`, in `Change`'s order.
const CHANGES: &str = "SELECT \"table\", pk, cid, val, col_version, db_version, site_id, cl, seq FROM crsql_changes";

fn ok<T>(r: rusqlite::Result<T>, what: &str) -> T {
    r.unwrap_or_else(|e| panic!("the live copy {what}: {e}"))
}

fn val(v: ValueRef) -> Val {
    match v {
        ValueRef::Null => Val::Null,
        ValueRef::Integer(i) => Val::Integer(i),
        ValueRef::Real(r) => Val::Real(r),
        ValueRef::Text(t) => Val::Text(std::str::from_utf8(t).expect("text is UTF-8").to_owned()),
        ValueRef::Blob(b) => Val::Blob(b.to_vec()),
    }
}

fn value(v: &Val) -> Value {
    match v {
        Val::Null => Value::Null,
        Val::Integer(i) => Value::Integer(*i),
        Val::Real(r) => Value::Real(*r),
        Val::Text(s) => Value::Text(s.clone()),
        Val::Blob(b) => Value::Blob(b.clone()),
    }
}

fn site_of(bytes: Vec<u8>) -> Site {
    Site(
        bytes
            .try_into()
            .expect("a site id is 16 bytes (consts.rs, SITE_ID_LEN)"),
    )
}

/// One crsql_changes row, read by `CHANGES`.
fn change(r: &Row) -> rusqlite::Result<Change> {
    Ok(Change {
        table: r.get(0)?,
        pk: r.get(1)?,
        cid: r.get(2)?,
        val: val(r.get_ref(3)?),
        col_version: r.get(4)?,
        db_version: r.get(5)?,
        site: site_of(r.get(6)?),
        cl: r.get(7)?,
        seq: r.get(8)?,
    })
}

fn quoted<S: AsRef<str>>(names: &[S]) -> String {
    let quoted: Vec<String> = names.iter().map(|n| format!("\"{}\"", n.as_ref())).collect();
    quoted.join(", ")
}

/// `k1 = ? AND k2 = ?`.
fn by_key(keys: &[String]) -> String {
    let each: Vec<String> = keys.iter().map(|k| format!("\"{k}\" = ?")).collect();
    each.join(" AND ")
}

impl LiveCopy {
    pub fn new(site: Site) -> LiveCopy {
        LiveCopy::planted(site, Plant::None)
    }

    /// A live copy with a fault planted in what it keeps. The engine's merge
    /// can't be planted, so only `Plant::EagerPrune` is.
    pub fn planted(site: Site, plant: Plant) -> LiveCopy {
        assert!(
            matches!(plant, Plant::None | Plant::EagerPrune),
            "a live copy plants only what it keeps, not {plant:?}"
        );
        // cr-sqlite reads a connection's site at open, the crsql_site_id row
        // at ordinal 0 (rs/core/src/bootstrap.rs:101-120), and that table's
        // triggers fire only off ordinal 0 (:51-70). So the site is set on
        // one connection, and a second one on the same database opens with
        // it. Shared cache keeps an in-memory database while one connection
        // to it stays open.
        let name = format!(
            "file:w1-live-{}-{}?mode=memory&cache=shared",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let first = ok(Connection::open_with_flags(&name, flags), "opens");
        let set = first.execute("UPDATE crsql_site_id SET site_id = ? WHERE ordinal = 0", [&site.0[..]]);
        assert_eq!(ok(set, "sets its site"), 1, "one site row at ordinal 0");
        let conn = ok(Connection::open_with_flags(&name, flags), "opens again");
        finalize(&first);
        drop(first);
        let read = conn.query_row("SELECT crsql_site_id()", [], |r| r.get::<_, Vec<u8>>(0));
        assert_eq!(
            site_of(ok(read, "reads its site")),
            site,
            "the engine opens with the site set"
        );
        ok(conn.execute_batch(SCHEMA), "makes its tables");
        let mut shapes = Vec::new();
        for t in TABLES {
            ok(
                conn.query_row("SELECT crsql_as_crr(?)", [t.name], |_| Ok(())),
                "makes a CRR",
            );
            let names = |sql: &str| -> Vec<String> {
                let mut stmt = ok(conn.prepare(sql), "reads its schema");
                let rows = ok(stmt.query_map([t.name], |r| r.get(0)), "reads its schema");
                rows.map(|r| ok(r, "reads its schema")).collect()
            };
            let keys = names("SELECT name FROM pragma_table_info(?) WHERE pk > 0 ORDER BY pk");
            let cols = names("SELECT name FROM pragma_table_info(?) WHERE pk = 0 ORDER BY cid");
            assert_eq!(
                cols, t.cols,
                "sim::TABLES names {}'s non-key columns as SCHEMA has them",
                t.name
            );
            let defaults = names(
                "SELECT name || ' ' || dflt_value FROM pragma_table_info(?) WHERE pk = 0 AND dflt_value IS NOT NULL ORDER BY cid",
            );
            let literal = |v: &Val| match v {
                Val::Integer(i) => i.to_string(),
                other => panic!("sim::TABLES gives a default SCHEMA can't: {other:?}"),
            };
            let want: Vec<String> = t.defaults.iter().map(|(c, v)| format!("{c} {}", literal(v))).collect();
            assert_eq!(
                defaults, want,
                "sim::TABLES gives {}'s defaults as SCHEMA has them",
                t.name
            );
            shapes.push(Shape {
                name: t.name,
                keys,
                cols: t.cols,
            });
        }
        // Equal values tie on site, as svrngs sets it (APP_STATE.md §4, "How
        // it merges"; rs/core/src/changes_vtab_write.rs:95-106, :527-553)
        // and as the model copy and the reference read the merge.
        let config = conn.query_row("SELECT crsql_config_set('merge-equal-values', 1)", [], |_| Ok(()));
        ok(config, "sets merge-equal-values");
        LiveCopy {
            site,
            conn,
            shapes,
            kept: BTreeMap::new(),
            plant,
        }
    }

    fn shape(&self, table: &str) -> &Shape {
        self.shapes
            .iter()
            .find(|s| s.name == table)
            .unwrap_or_else(|| panic!("no table {table}"))
    }

    /// One op of the write form as SQL, and its values; none for an update
    /// that sets nothing.
    fn sql(&self, op: &Op) -> Option<(String, Vec<Value>)> {
        match op {
            Op::Insert {
                table,
                key,
                row,
                on_conflict,
            } => {
                let shape = self.shape(table);
                let mut names: Vec<&str> = shape.keys.iter().map(String::as_str).collect();
                names.extend(row.iter().map(|(c, _)| *c));
                let marks = vec!["?"; names.len()].join(", ");
                let set: Vec<String> = row
                    .iter()
                    .filter(|(c, _)| on_conflict.contains(c))
                    .map(|(c, _)| format!("\"{c}\" = excluded.\"{c}\""))
                    .collect();
                let then = match set.is_empty() {
                    true => "NOTHING".to_owned(),
                    false => format!("UPDATE SET {}", set.join(", ")),
                };
                let sql = format!(
                    "INSERT INTO \"{table}\" ({}) VALUES ({marks}) ON CONFLICT ({}) DO {then}",
                    quoted(&names),
                    quoted(&shape.keys)
                );
                Some((sql, key.iter().chain(row.iter().map(|(_, v)| v)).map(value).collect()))
            }
            Op::Update { table, key, set } => {
                if set.is_empty() {
                    return None;
                }
                let cols: Vec<String> = set.iter().map(|(c, _)| format!("\"{c}\" = ?")).collect();
                let sql = format!(
                    "UPDATE \"{table}\" SET {} WHERE {}",
                    cols.join(", "),
                    by_key(&self.shape(table).keys)
                );
                Some((sql, set.iter().map(|(_, v)| v).chain(key).map(value).collect()))
            }
            Op::Delete { table, key } => {
                let sql = format!("DELETE FROM \"{table}\" WHERE {}", by_key(&self.shape(table).keys));
                Some((sql, key.iter().map(value).collect()))
            }
        }
    }

    /// Every row of crsql_changes, one per clock row, as the engine reads
    /// them: what this copy would send if it sent its clocks.
    pub fn rows(&self) -> Vec<Change> {
        let mut stmt = ok(self.conn.prepare_cached(CHANGES), "reads its changes");
        let rows = ok(stmt.query_map([], change), "reads its changes");
        rows.map(|r| ok(r, "reads a change")).collect()
    }

    /// Drops the sets nothing of which stands here, as the model copy does;
    /// planted, keeps a set only while every one of its changes stands.
    fn prune(&mut self) {
        if self.plant == Plant::EagerPrune {
            let rows = self.rows();
            let stands = |c: &Change| {
                rows.iter().any(|r| {
                    (&r.table, &r.pk, &r.cid, r.col_version, r.db_version, r.site, r.seq)
                        == (&c.table, &c.pk, &c.cid, c.col_version, c.db_version, c.site, c.seq)
                })
            };
            self.kept.retain(|_, set| set.changes().iter().all(stands));
            return;
        }
        let standing = self.standing();
        self.kept.retain(|id, _| standing.contains(id));
    }
}

/// cr-sqlite's statements, finalized so SQLite can close the connection:
/// only libSQL's build closes them itself (src/crsqlite.c:31-36, :80-83,
/// under `#ifdef LIBSQL`). Without it, a dropped copy's tables were still
/// there to the next connection opened on its name.
fn finalize(conn: &Connection) {
    ok(conn.query_row("SELECT crsql_finalize()", [], |_| Ok(())), "finalizes");
}

impl Drop for LiveCopy {
    fn drop(&mut self) {
        self.conn.flush_prepared_statement_cache();
        finalize(&self.conn);
    }
}

impl Replica for LiveCopy {
    fn site(&self) -> Site {
        self.site
    }

    /// The ops in one transaction, and before it commits, the set it seals:
    /// its rows at the version it writes, which cr-sqlite fixes at its first
    /// change (rs/core/src/db_version.rs:51-81).
    fn write(&mut self, ops: &[Op]) -> Option<ChangeSet> {
        let tx = ok(self.conn.unchecked_transaction(), "begins a write");
        for op in ops {
            if let Some((sql, values)) = self.sql(op) {
                ok(tx.execute(&sql, params_from_iter(values)), &format!("runs {sql}"));
            }
        }
        let v: i64 = ok(
            tx.query_row("SELECT crsql_peek_next_db_version()", [], |r| r.get(0)),
            "reads its version",
        );
        let rows: Vec<Change> = {
            let mut stmt = ok(
                tx.prepare_cached(&format!("{CHANGES} WHERE db_version = ? AND site_id = ? ORDER BY seq")),
                "reads its set",
            );
            let rows = ok(stmt.query_map(params![v, &self.site.0[..]], change), "reads its set");
            rows.map(|r| ok(r, "reads its set")).collect()
        };
        ok(tx.commit(), "commits a write");
        let set = ChangeSet::new(rows)?;
        self.kept.insert(set.id(), set.clone());
        self.prune();
        Some(set)
    }

    /// The set's rows into crsql_changes, in one transaction, `ts` as text
    /// (rs/core/src/changes_vtab_write.rs:471): svrngs merges by no clock.
    fn take(&mut self, set: &ChangeSet) {
        let tx = ok(self.conn.unchecked_transaction(), "begins a take");
        {
            let mut stmt = ok(
                tx.prepare_cached(
                    "INSERT INTO crsql_changes (\"table\", pk, cid, val, col_version, db_version, site_id, cl, seq, ts)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, '0')",
                ),
                "prepares a take",
            );
            for c in set.changes() {
                let merged = stmt.execute(params![
                    c.table,
                    c.pk,
                    c.cid,
                    value(&c.val),
                    c.col_version,
                    c.db_version,
                    &c.site.0[..],
                    c.cl,
                    c.seq
                ]);
                ok(merged, &format!("merges {c:?}"));
            }
        }
        ok(tx.commit(), "commits a take");
        self.kept.insert(set.id(), set.clone());
        self.prune();
    }

    fn kept(&self) -> Vec<ChangeSet> {
        self.kept.values().cloned().collect()
    }

    fn standing(&self) -> BTreeSet<SetId> {
        let mut stmt = ok(
            self.conn
                .prepare_cached("SELECT DISTINCT site_id, db_version FROM crsql_changes"),
            "reads what stands",
        );
        let rows = ok(
            stmt.query_map([], |r| Ok((site_of(r.get(0)?), r.get(1)?))),
            "reads what stands",
        );
        rows.map(|r| ok(r, "reads what stands")).collect()
    }

    fn state(&self) -> State {
        let mut state = State::new();
        for shape in &self.shapes {
            let sql = format!(
                "SELECT crsql_pack_columns({}), {} FROM \"{}\"",
                quoted(&shape.keys),
                quoted(shape.cols),
                shape.name
            );
            let mut stmt = ok(self.conn.prepare_cached(&sql), "reads its rows");
            let mut rows = ok(stmt.query([]), "reads its rows");
            while let Some(r) = ok(rows.next(), "reads a row") {
                let mut values = BTreeMap::new();
                for (i, col) in shape.cols.iter().enumerate() {
                    let v = val(ok(r.get_ref(i + 1), "reads a value"));
                    if v != Val::Null {
                        values.insert(col.to_string(), v);
                    }
                }
                state.insert((shape.name.to_owned(), ok(r.get(0), "reads a key")), values);
            }
        }
        state
    }

    /// A copy undoes nothing it merged (APP_STATE §4, "A removed device"), as
    /// the model's unplanted copy.
    fn removed(&mut self, _site: Site) {}
}

thread_local! {
    /// The run this thread's pairs are in, named in what they say.
    static RUN: RefCell<String> = const { RefCell::new(String::new()) };
    /// Each pair's first disagreement on this thread, in the order found.
    static PARTED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// A model copy and a live copy in lockstep. It answers as the model copy,
/// so a run of pairs is the model's run, and after each write and take it
/// holds the live copy to the model's: the set sealed, the state, the sets
/// standing and those kept. Each pair says only its first disagreement.
pub struct Pair {
    model: ModelCopy,
    live: LiveCopy,
    parted: Cell<bool>,
}

/// A set by its device's number and its `db_version`.
pub fn named(id: SetId) -> String {
    format!("{}/{}", id.0.0[0], id.1)
}

fn sealed(m: &Option<ChangeSet>, l: &Option<ChangeSet>) -> String {
    if let (Some(m), Some(l)) = (m, l) {
        let (m, l) = (m.changes(), l.changes());
        let i = (0..m.len().max(l.len())).find(|&i| m.get(i) != l.get(i));
        let i = i.expect("two sets that differ differ at a change");
        return format!(
            "the sets part at change {i}: the model's {:?}, the live's {:?}",
            m.get(i),
            l.get(i)
        );
    }
    let id = |s: &Option<ChangeSet>| s.as_ref().map(|s| named(s.id()));
    format!("the model sealed {:?}, the live {:?}", id(m), id(l))
}

fn parts(what: &str, m: &BTreeSet<SetId>, l: &BTreeSet<SetId>) -> Option<String> {
    let only =
        |a: &BTreeSet<SetId>, b: &BTreeSet<SetId>| -> Vec<String> { a.difference(b).map(|id| named(*id)).collect() };
    (m != l).then(|| {
        format!(
            "{what}: the model's alone {:?}, the live's alone {:?}",
            only(m, l),
            only(l, m)
        )
    })
}

pub fn kept_ids(r: &dyn Replica) -> BTreeSet<SetId> {
    r.kept().iter().map(ChangeSet::id).collect()
}

impl Pair {
    pub fn new(site: Site) -> Pair {
        Pair::planted(site, Plant::None)
    }

    /// A pair whose model copy is planted.
    pub fn planted(site: Site, plant: Plant) -> Pair {
        Pair::of(ModelCopy::planted(site, TABLES, plant))
    }

    /// A pair whose model copy keeps another rule than the engine's.
    pub fn ruled(site: Site, rule: Rule) -> Pair {
        Pair::of(ModelCopy::ruled(site, TABLES, rule))
    }

    fn of(model: ModelCopy) -> Pair {
        Pair {
            live: LiveCopy::new(model.site()),
            model,
            parted: Cell::new(false),
        }
    }

    fn check(&self, after: &dyn Fn() -> String, seals: Option<(&Option<ChangeSet>, &Option<ChangeSet>)>) {
        if self.parted.get() {
            return;
        }
        let mut found = Vec::new();
        if let Some((m, l)) = seals
            && m != l
        {
            found.push(sealed(m, l));
        }
        let (m, l) = (self.model.state(), self.live.state());
        if m != l {
            found.push(format!(
                "the state, the model's against the live's: {}",
                sim::diff(&m, &l)
            ));
        }
        found.extend(parts("standing", &self.model.standing(), &self.live.standing()));
        found.extend(parts("kept", &kept_ids(&self.model), &kept_ids(&self.live)));
        if found.is_empty() {
            return;
        }
        self.parted.set(true);
        let copy = self.model.site().0[0];
        let run = RUN.with_borrow(String::clone);
        let said = format!("{run}, copy {copy}, after {}: {}", after(), found.join("; "));
        PARTED.with_borrow_mut(|p| p.push(said));
    }
}

impl Replica for Pair {
    fn site(&self) -> Site {
        self.model.site()
    }

    fn write(&mut self, ops: &[Op]) -> Option<ChangeSet> {
        let (m, l) = (self.model.write(ops), self.live.write(ops));
        self.check(&|| format!("writing {ops:?}"), Some((&m, &l)));
        m
    }

    fn take(&mut self, set: &ChangeSet) {
        self.model.take(set);
        self.live.take(set);
        self.check(&|| format!("taking {}", named(set.id())), None);
    }

    fn kept(&self) -> Vec<ChangeSet> {
        self.model.kept()
    }

    fn standing(&self) -> BTreeSet<SetId> {
        self.model.standing()
    }

    fn state(&self) -> State {
        self.model.state()
    }

    fn removed(&mut self, site: Site) {
        self.model.removed(site);
        self.live.removed(site);
    }
}

/// What `f` returns, run as `run`, and the pairs' first disagreements in it.
fn watched<T>(run: String, f: impl FnOnce() -> T) -> (T, Vec<String>) {
    RUN.set(run);
    PARTED.take();
    let out = f();
    (out, PARTED.take())
}

fn model(site: Site) -> ModelCopy {
    ModelCopy::new(site, TABLES)
}

/// The live engine on the gate's seeds, each run met in full and judged:
/// 30 s in a debug build on an M-series Mac, the runs one after another.
/// Shared among threads they took 86 s, the threads waiting on SQLite's one
/// lock around every allocation, which it counted until /.cargo/config.toml
/// set SQLITE_DEFAULT_MEMSTATUS=0 (libsqlite3-sys 0.39.1's build.rs:168-188
/// leaves it at 1). Out of the gate: the pairs' test below runs every live
/// copy of these seeds at every step, and fails where this would.
/// Its plant, still in the gate: `the_gate_s_seeds_catch_a_live_prune_that_drops_a_set_still_standing`.
#[test]
#[ignore = "the pairs' test on the same seeds holds it, at every step"]
fn every_live_copy_ends_on_the_reference_on_the_gate_s_seeds() {
    let failed: Vec<String> = GATE.filter_map(|seed| sim::one(seed, &LiveCopy::new).err()).collect();
    assert!(
        failed.is_empty(),
        "{} seeds failed:\n{}",
        failed.len(),
        failed.join("\n")
    );
}

#[test]
fn the_gate_s_seeds_catch_a_live_prune_that_drops_a_set_still_standing() {
    let make = |site| LiveCopy::planted(site, Plant::EagerPrune);
    let e = GATE.into_iter().find_map(|seed| sim::one(seed, &make).err());
    let e = e.expect("a seed catches the plant");
    println!("caught: {e}");
    assert!(e.contains("sets it doesn't keep"), "{e}");
}

/// The model copy against the engine it emulates, at every write and take
/// of the gate's seeds. Where they never part, the live copies' runs are the
/// model's, so this also holds the one above. The seeds are shared among
/// the computer's threads, which SQLite's allocation count kept waiting on
/// one lock until it was turned off: 50 s on one thread in a debug build.
/// Its plant: `the_pair_catches_a_model_copy_that_keeps_clocks_on_resurrect`.
#[test]
fn the_live_copy_and_the_model_copy_agree_at_every_step_on_the_gate_s_seeds() {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    let parted: Vec<String> = std::thread::scope(|s| {
        let runs: Vec<_> = (0..threads)
            .map(|t| {
                s.spawn(move || {
                    let mut parted = Vec::new();
                    for seed in GATE.filter(|seed| seed % threads == t) {
                        let (verdict, p) = watched(format!("seed {seed}"), || sim::one(seed, &Pair::new));
                        if let Err(e) = verdict {
                            parted.push(format!("the pairs' run failed W1: {e}"));
                        }
                        parted.extend(p);
                    }
                    parted
                })
            })
            .collect();
        runs.into_iter()
            .flat_map(|run| run.join().expect("a thread of pairs ran"))
            .collect()
    });
    assert!(
        parted.is_empty(),
        "{} disagreements:\n{}",
        parted.len(),
        parted.join("\n")
    );
}

#[test]
fn the_pair_catches_a_model_copy_that_keeps_clocks_on_resurrect() {
    let make = |site| Pair::planted(site, Plant::KeepClocksOnResurrect);
    let first = GATE.into_iter().find_map(|seed| {
        let (_, parted) = watched(format!("seed {seed}"), || sim::one(seed, &make));
        parted.into_iter().next()
    });
    let first = first.expect("a seed parts the pair");
    println!("caught: {first}");
    assert!(first.contains("the state"), "{first}");
}

/// One thread's part of the live soak: each failure and disagreement by its
/// seed, the runs that met question 9's moment, and how often it came.
#[derive(Default)]
struct Soaked {
    said: Vec<(u64, String)>,
    met: usize,
    moments: usize,
}

/// The live engine's soak: seeds 0..100,000, or `W1_SEEDS` as
/// `<first>..<end>`, each on model copies alone, live copies alone and
/// pairs, shared among the computer's threads, every failure and
/// disagreement said, and how many runs met question 9's moment
/// (`ModelCopy::earlier`). Where no pair parts, the live copies' runs are
/// the model's, so they met the same moments. Run it where there are many
/// cores, the Fedora box: `cargo test --release -p svrngs-db --test w1 --
/// --ignored live_engine_s_soak --nocapture`.
#[test]
#[ignore = "100,000 runs each of model copies, live copies and pairs, not the gate's"]
#[expect(clippy::disallowed_methods, reason = "the soak's time, said")]
fn the_live_engine_s_soak() {
    let seeds = std::env::var("W1_SEEDS").unwrap_or_else(|_| "0..100000".into());
    let (first, end) = seeds.split_once("..").expect("W1_SEEDS is <first>..<end>");
    let (first, end): (u64, u64) = (first.parse().expect("a first seed"), end.parse().expect("an end seed"));
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    let start = std::time::Instant::now();
    let each: Vec<Soaked> = std::thread::scope(|s| {
        let runs: Vec<_> = (0..threads)
            .map(|t| {
                s.spawn(move || {
                    let mut soaked = Soaked::default();
                    for seed in (first..end).filter(|seed| seed % threads == t) {
                        let shape = sim::Shape::of(seed);
                        let mut alone = sim::Sim::new(seed, shape, &model);
                        alone.run(shape.steps);
                        alone.meet();
                        let here: usize = alone.copies().iter().map(ModelCopy::earlier).sum();
                        soaked.met += usize::from(here > 0);
                        soaked.moments += here;
                        let m = alone
                            .judge(Rule::default(), &model)
                            .map_err(|e| format!("seed {seed}: {e}"));
                        let l = sim::one(seed, &LiveCopy::new);
                        let (p, parted) = watched(format!("seed {seed}"), || sim::one(seed, &Pair::new));
                        for (who, v) in [("model copies", m), ("live copies", l), ("pairs", p)] {
                            soaked
                                .said
                                .extend(v.err().map(|e| (seed, format!("FAILED, {who}: {e}"))));
                        }
                        soaked
                            .said
                            .extend(parted.into_iter().map(|p| (seed, format!("PARTED {p}"))));
                    }
                    soaked
                })
            })
            .collect();
        runs.into_iter()
            .map(|run| run.join().expect("a thread of runs ran"))
            .collect()
    });
    let mut said: Vec<&(u64, String)> = each.iter().flat_map(|s| &s.said).collect();
    said.sort_by_key(|(seed, _)| *seed);
    for (_, s) in &said {
        println!("{s}");
    }
    let met: usize = each.iter().map(|s| s.met).sum();
    let moments: usize = each.iter().map(|s| s.moments).sum();
    println!(
        "seeds {first}..{end}: {} failures and disagreements; {met} runs met question 9's moment, {moments} times; {:.1} s on {threads} threads",
        said.len(),
        start.elapsed().as_secs_f64()
    );
    assert!(said.is_empty());
}

/// The copies agree on the live engine, the fork's, which keeps N (`rule.rs`;
/// svrn-docs APP_STATE.md §8, question 9): the fixed cases (`cases.rs`), then
/// seeds 2706 and 5691, where W's soak found today's split, each on model
/// copies alone, live copies alone and pairs, under the rule svrngs keeps.
/// Its plants, each watched failing: the engine before its patch, the
/// submodule at 6a59c477, where the live copies split as today's do; and the
/// pairs' model copy under today's rule, kept in the gate below. Run `cargo
/// test -p svrngs-db --test w1 -- copies_agree --nocapture`.
#[test]
fn the_copies_agree_on_the_live_engine() {
    let mut failed = Vec::new();
    let mut verdict = |what: String, v: &Result<(), String>| {
        let said = v.as_ref().map_or_else(|e| format!("FAILED {e}"), |_| "passed".into());
        println!("  {what}: {said}");
        if v.is_err() {
            failed.push(what);
        }
    };
    let agreed = |parted: &[String]| match parted.len() {
        0 => Ok(()),
        n => Err(format!("{n} pairs parted")),
    };
    let rule = Rule::default();
    for case in Case::ALL {
        let name = case.name();
        println!("{name}, model copies alone:");
        let (_, said, v) = case.run(rule, &model);
        println!("{said}");
        verdict(format!("{name}, model copies"), &v);
        println!("{name}, live copies alone:");
        let (live, said, v) = case.run(rule, &LiveCopy::new);
        println!("{said}");
        for &(c, copy) in case.standing() {
            println!("    {copy}'s crsql_changes rows:");
            for row in live[c].rows() {
                println!("      {row:?}");
            }
        }
        verdict(format!("{name}, live copies"), &v);
        println!("{name}, pairs:");
        let ((_, _, v), parted) = watched(name.into(), || case.run(rule, &Pair::new));
        for p in &parted {
            println!("    parted: {p}");
        }
        verdict(format!("{name}, pairs"), &v);
        verdict(format!("{name}, live copies against the model's"), &agreed(&parted));
    }
    for seed in [2706, 5691] {
        println!("seed {seed}, {:?}:", sim::Shape::of(seed));
        verdict(format!("seed {seed}, model copies"), &sim::one(seed, &model));
        verdict(format!("seed {seed}, live copies"), &sim::one(seed, &LiveCopy::new));
        let (v, parted) = watched(format!("seed {seed}"), || sim::one(seed, &Pair::new));
        for p in &parted {
            println!("    parted: {p}");
        }
        verdict(format!("seed {seed}, pairs"), &v);
        verdict(
            format!("seed {seed}, live copies against the model's"),
            &agreed(&parted),
        );
    }
    assert!(failed.is_empty(), "failed: {failed:?}");
}

/// The plant of the test above, kept in the gate: pairs whose model copy
/// keeps today's rule part from the engine on each fixed case, at the take
/// that brings a row back where its re-add is missing. So the engine is not
/// today's, and a pair would say so if a bump of the fork dropped its patch.
#[test]
fn the_pair_catches_a_model_copy_under_today_s_rule() {
    let make = |site| Pair::ruled(site, Rule::Today);
    for case in Case::ALL {
        let (_, parted) = watched(case.name().into(), || case.run(Rule::Today, &make));
        let first = parted
            .first()
            .unwrap_or_else(|| panic!("{}: no pair parted", case.name()));
        println!("caught: {first}");
        assert!(first.contains("the state"), "{first}");
    }
}
