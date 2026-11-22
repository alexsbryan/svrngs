//! A copy of an app's database as the simulator reaches it (`Replica`), and
//! the one copy there is so far, `ModelCopy`: cr-sqlite's clock tables, its
//! local writes and its merge, as read from Fly's fork at the pin `change.rs`
//! names (paths under `core/rs/core/src/`). It merges a change at a time, in
//! the order sets arrive, as the engine does, so it is a second decider for
//! the reference to judge, not the reference again. svrngs-db goes behind the
//! same trait when it is built. It is built with a rule for a set above a
//! missing re-add (`rule.rs`), by default N, the one the fork's engine keeps.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::change::{Change, ChangeSet, RowKey, SENTINEL, SetId, Site, State, Val, pack};
use crate::rule::Rule;

/// Each device's mark: every set of the device up to this version has been
/// taken here, or by a source this copy pulled from in full.
pub type Marks = BTreeMap<Site, i64>;

/// A copy of one app's database on one device.
pub trait Replica {
    fn site(&self) -> Site;
    /// Runs one write, the write form's ops in one transaction (APP_STATE
    /// §4, "What a page can do"), and returns the set it sealed, or none when
    /// it changed nothing.
    fn write(&mut self, ops: &[Op]) -> Option<ChangeSet>;
    /// Takes a set a member delivered. A set taken twice changes nothing.
    fn take(&mut self, set: &ChangeSet);
    /// The sets this copy keeps, those with a value that still stands here
    /// (APP_STATE §4, "What is kept"): what it sends a member.
    fn kept(&self) -> Vec<ChangeSet>;
    /// The set each of this copy's clock rows names, by site and
    /// `db_version`. Its kept sets must cover them (APP_STATE §7, invariant 3).
    fn standing(&self) -> BTreeSet<SetId>;
    /// Every row of every table.
    fn state(&self) -> State;
    /// The record removed a device. Its computer reaches the group no more,
    /// which the simulator holds to, and a copy undoes nothing it merged
    /// (APP_STATE §4, "A removed device").
    fn removed(&mut self, site: Site);
    /// The sets taken and not applied, under a rule that holds them
    /// (`Rule::WaitForLife`, `Rule::Contiguous`). They are among `kept`.
    fn held(&self) -> BTreeSet<SetId> {
        BTreeSet::new()
    }
    /// This copy's marks, under a rule that applies by them
    /// (`Rule::Contiguous`).
    fn marks(&self) -> Marks {
        Marks::new()
    }
    /// Takes a source's marks once it has taken every set the source keeps;
    /// whether any of its own rose.
    fn adopt(&mut self, _marks: &Marks) -> bool {
        false
    }
}

/// A table of an app's schema: its name, its non-key columns in schema
/// order, the order cr-sqlite clocks an insert's columns in
/// (`local_writes/mod.rs`, `mark_locally_inserted`), and each column's
/// DEFAULT where it has one.
#[derive(Clone, Copy, Debug)]
pub struct Table {
    pub name: &'static str,
    pub cols: &'static [&'static str],
    pub defaults: &'static [(&'static str, Val)],
}

impl Table {
    /// The table named so among these.
    pub fn named<'a>(tables: &'a [Table], name: &str) -> &'a Table {
        tables
            .iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("no table {name}"))
    }

    /// A row as SQLite makes it from its key alone, or from some of its
    /// columns: each other column at its DEFAULT, NULL where it has none.
    pub fn fresh(&self) -> BTreeMap<String, Val> {
        let default = |col: &str| self.defaults.iter().find(|(c, _)| *c == col).map(|(_, v)| v.clone());
        self.cols
            .iter()
            .map(|c| (c.to_string(), default(c).unwrap_or(Val::Null)))
            .collect()
    }
}

/// One op of a write, its row named by key (APP_STATE §4, "The write form").
#[derive(Clone, Debug)]
pub enum Op {
    /// Makes the row with `row`'s columns and the rest at their DEFAULT, NULL
    /// where there is none, as plain SQL does; on a key that has a row, sets
    /// `on_conflict`'s columns of it to `row`'s values. svrngs's write form
    /// binds NULL in every column an insert doesn't name (`src/form.rs`), so
    /// every insert here of a table with a DEFAULT names each column. The
    /// write form sets every column it names (`Op::insert`); S1's upsert set
    /// only `a` (magnets-spikes `code/src/bin/s1.rs:137`).
    Insert {
        table: &'static str,
        key: Vec<Val>,
        row: Vec<(&'static str, Val)>,
        on_conflict: Vec<&'static str>,
    },
    /// Sets these columns of the row with this key, if it has one.
    Update {
        table: &'static str,
        key: Vec<Val>,
        set: Vec<(&'static str, Val)>,
    },
    /// Deletes the row with this key, if it has one.
    Delete { table: &'static str, key: Vec<Val> },
}

impl Op {
    /// The write form's insert, which sets every column it names.
    pub fn insert(table: &'static str, key: Vec<Val>, row: Vec<(&'static str, Val)>) -> Op {
        let on_conflict = row.iter().map(|(c, _)| *c).collect();
        Op::Insert {
            table,
            key,
            row,
            on_conflict,
        }
    }
}

/// A fault planted in the model copy, so a check is watched failing on it
/// (AGENTS.md 5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Plant {
    #[default]
    None,
    /// Keeps a set only while every one of its values stands, so a set
    /// half overwritten is dropped though half of it still stands.
    EagerPrune,
    /// A resurrect leaves the columns' clocks as they were, where cr-sqlite
    /// zeroes them (`tableinfo.rs:440-454`).
    KeepClocksOnResurrect,
    /// The withdrawn fence: a removal drops the device's kept sets, rebuilds
    /// from the rest and refuses its sets after (APP_STATE §4, "A removed
    /// device", the review's case).
    UndoRemoved,
}

/// One clock row (`<table>__crsql_clock`): which change won a column, or
/// the row's sentinel.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Clock {
    cv: i64,
    dbv: i64,
    site: Site,
    seq: i64,
}

impl Clock {
    fn of(c: &Change) -> Clock {
        Clock {
            cv: c.col_version,
            dbv: c.db_version,
            site: c.site,
            seq: c.seq,
        }
    }
}

/// A copy whose engine is this model of cr-sqlite.
pub struct ModelCopy {
    site: Site,
    tables: &'static [Table],
    /// Each row that exists, with every non-key column, NULLs included.
    rows: BTreeMap<RowKey, BTreeMap<String, Val>>,
    /// Each row's clock rows, by column id, the sentinel under `SENTINEL`.
    clocks: BTreeMap<RowKey, BTreeMap<String, Clock>>,
    /// This site's last `db_version`. Fly's fork counts one per site
    /// (`db_version.rs:51-81`), so a set taken never moves it.
    db_version: i64,
    kept: BTreeMap<SetId, ChangeSet>,
    plant: Plant,
    /// The sites `Plant::UndoRemoved` refuses.
    fenced: BTreeSet<Site>,
    rule: Rule,
    /// The sets taken and not applied, under H and P.
    held: BTreeMap<SetId, ChangeSet>,
    /// Under P, the marks it applies by. A copy's own writes count in its own.
    marks: Marks,
    /// How often a column change brought back a row standing here at a
    /// shorter causal length while a value of its earlier life stood in
    /// another column: question 9's moment, where today's rule keeps that
    /// value and N clears it.
    earlier: usize,
}

impl ModelCopy {
    pub fn new(site: Site, tables: &'static [Table]) -> ModelCopy {
        ModelCopy::planted(site, tables, Plant::None)
    }

    pub fn planted(site: Site, tables: &'static [Table], plant: Plant) -> ModelCopy {
        ModelCopy::made(site, tables, plant, Rule::default())
    }

    /// A copy under one of question 9's rules.
    pub fn ruled(site: Site, tables: &'static [Table], rule: Rule) -> ModelCopy {
        ModelCopy::made(site, tables, Plant::None, rule)
    }

    /// A copy under one of question 9's rules, with a fault planted.
    pub fn made(site: Site, tables: &'static [Table], plant: Plant, rule: Rule) -> ModelCopy {
        assert!(
            tables.iter().all(|t| !t.cols.is_empty()),
            "a table of keys alone takes cr-sqlite's other insert path (after_insert.rs:50-54), which this model leaves out"
        );
        ModelCopy {
            site,
            tables,
            rows: BTreeMap::new(),
            clocks: BTreeMap::new(),
            db_version: 0,
            kept: BTreeMap::new(),
            plant,
            fenced: BTreeSet::new(),
            rule,
            held: BTreeMap::new(),
            marks: Marks::new(),
            earlier: 0,
        }
    }

    /// How often question 9's moment came here (`ModelCopy::earlier`).
    pub fn earlier(&self) -> usize {
        self.earlier
    }

    fn cols(&self, table: &str) -> &'static [&'static str] {
        Table::named(self.tables, table).cols
    }

    /// The key's row as SQLite makes it (`Table::fresh`): by a local insert
    /// of some of its columns, by the merge's insert of one
    /// (`tableinfo.rs:802-822`), or from its key alone (`:391-406`).
    fn fresh(&self, table: &str) -> BTreeMap<String, Val> {
        Table::named(self.tables, table).fresh()
    }

    /// A row's causal length here: its sentinel's version, else 1 if it has
    /// any clock row, else 0 (`tableinfo.rs:341-359`, `get_local_cl_stmt`).
    fn local_cl(&self, key: &RowKey) -> i64 {
        match self.clocks.get(key) {
            Some(clocks) => clocks.get(SENTINEL).map_or(1, |s| s.cv),
            None => 0,
        }
    }

    /// Every clock row as `crsql_changes` reads it (`changes_vtab_read.rs:26-47`):
    /// its value the column's now, its `cl` the row's sentinel's version or 1.
    pub fn clock_rows(&self) -> Vec<Change> {
        let mut out = Vec::new();
        for (key, clocks) in &self.clocks {
            let cl = self.local_cl(key);
            for (cid, clock) in clocks {
                let val = match self.rows.get(key) {
                    Some(row) if cid != SENTINEL => row[cid].clone(),
                    _ => Val::Null,
                };
                out.push(Change {
                    table: key.0.clone(),
                    pk: key.1.clone(),
                    cid: cid.clone(),
                    val,
                    col_version: clock.cv,
                    db_version: clock.dbv,
                    site: clock.site,
                    cl,
                    seq: clock.seq,
                });
            }
        }
        out
    }

    /// One local op inside the write at `dbv`; whether it changed anything.
    fn local(&mut self, op: &Op, dbv: i64, seq: &mut i64) -> bool {
        let mut next = |s: Site| {
            *seq += 1;
            Clock {
                cv: 0,
                dbv,
                site: s,
                seq: *seq - 1,
            }
        };
        let me = self.site;
        match op {
            Op::Insert {
                table,
                key,
                row,
                on_conflict,
            } => {
                let key = (table.to_string(), pack(key));
                if self.rows.contains_key(&key) {
                    let set: Vec<_> = row.iter().filter(|(c, _)| on_conflict.contains(c)).cloned().collect();
                    return self.update(&key, &set, &mut next);
                }
                // after_insert.rs:37-72. A key with a sentinel was deleted, so
                // its sentinel moves to the next odd length
                // (tableinfo.rs:655-679); a fresh key gets no sentinel.
                if let Some(s) = self.clocks.get_mut(&key).and_then(|c| c.get_mut(SENTINEL)) {
                    let cv = s.cv + if s.cv % 2 == 0 { 1 } else { 2 };
                    *s = Clock { cv, ..next(me) };
                }
                let mut values = self.fresh(table);
                for (c, v) in row {
                    values.insert(c.to_string(), v.clone());
                }
                self.rows.insert(key.clone(), values);
                // mark_locally_inserted: every non-key column clocked, one
                // past its clock or 1 (tableinfo.rs:576-653).
                let cols = self.cols(table);
                let clocks = self.clocks.entry(key).or_default();
                for col in cols {
                    let cv = clocks.get(*col).map_or(1, |c| c.cv + 1);
                    clocks.insert(col.to_string(), Clock { cv, ..next(me) });
                }
                true
            }
            Op::Update { table, key, set } => {
                let key = (table.to_string(), pack(key));
                self.rows.contains_key(&key) && self.update(&key, set, &mut next)
            }
            Op::Delete { table, key } => {
                let key = (table.to_string(), pack(key));
                if self.rows.remove(&key).is_none() {
                    return false;
                }
                // after_delete.rs: the sentinel one past, or 2 for a row that
                // had none (tableinfo.rs:456-489), then the columns' clocks
                // dropped (:424-438).
                let clocks = self.clocks.entry(key).or_default();
                let cv = clocks.get(SENTINEL).map_or(2, |s| s.cv + 1);
                clocks.retain(|cid, _| cid == SENTINEL);
                clocks.insert(SENTINEL.to_string(), Clock { cv, ..next(me) });
                true
            }
        }
    }

    /// after_update.rs:117-138: each column whose value changes, in schema
    /// order, clocked one past or at 1 (`mark_locally_updated`).
    fn update(&mut self, key: &RowKey, set: &[(&'static str, Val)], next: &mut impl FnMut(Site) -> Clock) -> bool {
        let mut changed = false;
        for col in self.cols(&key.0) {
            let Some((_, v)) = set.iter().find(|(c, _)| c == col) else {
                continue;
            };
            let row = self.rows.get_mut(key).expect("an update reaches a row that exists");
            if v.cmp_crsql(&row[*col]) == Ordering::Equal {
                continue;
            }
            row.insert(col.to_string(), v.clone());
            let clocks = self.clocks.entry(key.clone()).or_default();
            let cv = clocks.get(*col).map_or(1, |c| c.cv + 1);
            clocks.insert(col.to_string(), Clock { cv, ..next(self.site) });
            changed = true;
        }
        changed
    }

    /// The rows of this site's write at `dbv`, in `seq` order, as S1 and the
    /// spike read a set inside its transaction (`crsql_changes` where
    /// `db_version` is the write's and the site is this one).
    fn own(&self, dbv: i64) -> Vec<Change> {
        let mut rows: Vec<Change> = self
            .clock_rows()
            .into_iter()
            .filter(|c| c.db_version == dbv && c.site == self.site)
            .collect();
        rows.sort_by_key(|c| c.seq);
        rows
    }

    /// One change merged, `crsql_merge_insert` (changes_vtab_write.rs:424-742).
    fn merge(&mut self, c: &Change) {
        let key = (c.table.clone(), c.pk.clone());
        let local_cl = self.local_cl(&key);
        // :511-513: an older causal length wins nothing.
        if c.cl < local_cl {
            return;
        }
        // :515, :524-578: an even length is a delete.
        if c.cl % 2 == 0 {
            if c.cl == local_cl {
                // :527-552: the same delete; with merge-equal-values on, the
                // larger site names the sentinel.
                let held = self
                    .clocks
                    .get_mut(&key)
                    .and_then(|m| m.get_mut(&c.cid))
                    .expect("a row at an even length has its sentinel");
                if c.site > held.site {
                    *held = Clock::of(c);
                }
                return;
            }
            // :556-577, merge_delete (:304-366).
            self.rows.remove(&key);
            let clocks = self.clocks.entry(key).or_default();
            clocks.retain(|cid, _| cid == SENTINEL);
            clocks.insert(SENTINEL.to_string(), Clock::of(c));
            return;
        }
        // :588-612: a sentinel at the length already held changes nothing.
        if c.cid == SENTINEL {
            if c.cl > local_cl {
                self.resurrect(&key, Clock::of(c));
            }
            return;
        }
        let exists = local_cl != 0;
        let longer = c.cl > local_cl;
        // :620-645: a column change at a longer length brings the row back
        // first, its sentinel at that length. Over a row standing here, at an
        // odd length, today's engine keeps its earlier life's values
        // (resurrect, below), and N clears them first.
        if longer && (exists || c.cl > 1) {
            if local_cl % 2 == 1 {
                let carried = self
                    .rows
                    .get(&key)
                    .is_some_and(|row| row.iter().any(|(cid, v)| *cid != c.cid && *v != Val::Null));
                self.earlier += usize::from(carried);
                if self.rule == Rule::NewLifeEmpty {
                    // N, the fork's patch (:621-625, clear_earlier_life at
                    // :744-803): the row as a copy that never held it makes
                    // it, from its key alone. Its base row is deleted and its
                    // columns' clocks dropped, as merge_delete leaves a
                    // deleted row (:316-339, :354-363), and the sentinel is
                    // left for the resurrect to set.
                    self.rows.remove(&key);
                    if let Some(clocks) = self.clocks.get_mut(&key) {
                        clocks.retain(|cid, _| cid == SENTINEL);
                    }
                }
            }
            self.resurrect(
                &key,
                Clock {
                    cv: c.cl,
                    ..Clock::of(c)
                },
            );
        }
        // :650-670: a longer length, or a row this copy never had, wins
        // outright; otherwise the column's own clock decides.
        if !(longer || !exists || self.wins(&key, c)) {
            return;
        }
        let fresh = self.fresh(&c.table);
        self.rows
            .entry(key.clone())
            .or_insert(fresh)
            .insert(c.cid.clone(), c.val.clone());
        self.clocks.entry(key).or_default().insert(c.cid.clone(), Clock::of(c));
    }

    /// merge_sentinel_only_insert (:229-289): the key's row made if it has
    /// none, each column at its default (tableinfo.rs:391-406), its columns'
    /// clocks zeroed with their values kept (tableinfo.rs:440-454), and the
    /// sentinel set.
    fn resurrect(&mut self, key: &RowKey, sentinel: Clock) {
        let fresh = self.fresh(&key.0);
        self.rows.entry(key.clone()).or_insert(fresh);
        let clocks = self.clocks.entry(key.clone()).or_default();
        if self.plant != Plant::KeepClocksOnResurrect {
            for (cid, clock) in clocks.iter_mut() {
                if cid != SENTINEL {
                    clock.cv = 0;
                }
            }
        }
        clocks.insert(SENTINEL.to_string(), sentinel);
    }

    /// did_cid_win (:29-122) at one causal length: no clock here, the most
    /// edits, the larger value, then with merge-equal-values the larger site
    /// (did_site_id_win, :124-167).
    fn wins(&self, key: &RowKey, c: &Change) -> bool {
        let Some(held) = self.clocks.get(key).and_then(|m| m.get(&c.cid)) else {
            return true;
        };
        match c.col_version.cmp(&held.cv) {
            Ordering::Greater => true,
            Ordering::Less => false,
            Ordering::Equal => match c.val.cmp_crsql(&self.rows[key][&c.cid]) {
                Ordering::Equal => c.site > held.site,
                by_value => by_value == Ordering::Greater,
            },
        }
    }

    /// Drops the sets nothing of which stands here (APP_STATE §4, "What is
    /// kept"; spike `prune`).
    fn prune(&mut self) {
        if self.plant == Plant::EagerPrune {
            let clocks = &self.clocks;
            self.kept.retain(|_, set| {
                set.changes().iter().all(|c| {
                    clocks.get(&(c.table.clone(), c.pk.clone())).and_then(|m| m.get(&c.cid)) == Some(&Clock::of(c))
                })
            });
            return;
        }
        let standing = self.standing();
        self.kept.retain(|id, _| standing.contains(id));
    }

    /// A set merged, kept while it stands, and counted in its device's mark.
    fn apply(&mut self, set: &ChangeSet) {
        for c in set.changes() {
            self.merge(c);
        }
        let (site, version) = set.id();
        let mark = self.marks.entry(site).or_default();
        *mark = (*mark).max(version);
        self.kept.insert(set.id(), set.clone());
        self.prune();
    }

    /// Whether the rule lets this set apply here now.
    fn admits(&self, set: &ChangeSet) -> bool {
        match self.rule {
            Rule::Today | Rule::NewLifeEmpty => true,
            // H: each column change past length 1 finds its row here at that
            // length or a longer one, which outranks it, or finds its row's
            // row-level change at that length in the set.
            Rule::WaitForLife => set.changes().iter().all(|c| {
                let here = self.local_cl(&(c.table.clone(), c.pk.clone()));
                let own = set
                    .changes()
                    .iter()
                    .any(|o| o.cid == SENTINEL && o.table == c.table && o.pk == c.pk && o.cl == c.cl);
                c.cid == SENTINEL || c.cl <= 1 || here >= c.cl || own
            }),
            // P: every earlier set of its device is counted in the mark.
            Rule::Contiguous => {
                let (site, version) = set.id();
                version <= self.marks.get(&site).copied().unwrap_or(0) + 1
            }
        }
    }

    /// Applies the held sets the rule now admits, the lowest named first,
    /// until it admits none.
    fn release(&mut self) {
        while let Some(id) = self.held.iter().find(|(_, s)| self.admits(s)).map(|(id, _)| *id) {
            let set = self.held.remove(&id).expect("a held set found is held");
            self.apply(&set);
        }
    }
}

impl Replica for ModelCopy {
    fn site(&self) -> Site {
        self.site
    }

    fn write(&mut self, ops: &[Op]) -> Option<ChangeSet> {
        // A write's version is fixed for its transaction and used only if it
        // changes something (db_version.rs:51-81; after_update.rs:140-143).
        let dbv = self.db_version + 1;
        let mut seq = 0;
        let mut changed = false;
        for op in ops {
            changed |= self.local(op, dbv, &mut seq);
        }
        if !changed {
            return None;
        }
        self.db_version = dbv;
        let set = ChangeSet::new(self.own(dbv)).expect("a write that changed something has rows");
        self.marks.insert(self.site, dbv);
        self.kept.insert(set.id(), set.clone());
        self.prune();
        Some(set)
    }

    fn take(&mut self, set: &ChangeSet) {
        if self.fenced.contains(&set.id().0) {
            return;
        }
        match self.rule {
            Rule::Today | Rule::NewLifeEmpty => self.apply(set),
            Rule::WaitForLife | Rule::Contiguous => {
                self.held.insert(set.id(), set.clone());
                self.release();
            }
        }
    }

    /// The sets standing here and those held, in the order of their names,
    /// as today's copy lists its own.
    fn kept(&self) -> Vec<ChangeSet> {
        if self.held.is_empty() {
            return self.kept.values().cloned().collect();
        }
        let mut all = self.kept.clone();
        all.extend(self.held.iter().map(|(id, s)| (*id, s.clone())));
        all.into_values().collect()
    }

    fn standing(&self) -> BTreeSet<SetId> {
        self.clocks
            .values()
            .flat_map(|m| m.values().map(|c| (c.site, c.dbv)))
            .collect()
    }

    fn state(&self) -> State {
        self.rows
            .iter()
            .map(|(key, row)| {
                let values = row
                    .iter()
                    .filter(|(_, v)| **v != Val::Null)
                    .map(|(c, v)| (c.clone(), v.clone()));
                (key.clone(), values.collect())
            })
            .collect()
    }

    fn held(&self) -> BTreeSet<SetId> {
        self.held.keys().copied().collect()
    }

    fn marks(&self) -> Marks {
        match self.rule {
            Rule::Contiguous => self.marks.clone(),
            _ => Marks::new(),
        }
    }

    fn adopt(&mut self, marks: &Marks) -> bool {
        if self.rule != Rule::Contiguous {
            return false;
        }
        let mut rose = false;
        for (site, theirs) in marks {
            let mine = self.marks.entry(*site).or_default();
            rose |= *theirs > *mine;
            *mine = (*mine).max(*theirs);
        }
        self.release();
        rose
    }

    fn removed(&mut self, site: Site) {
        if self.plant != Plant::UndoRemoved {
            return;
        }
        let rest: Vec<ChangeSet> = self.kept.values().filter(|s| s.id().0 != site).cloned().collect();
        let mut rebuilt = ModelCopy::made(self.site, self.tables, self.plant, self.rule);
        rebuilt.db_version = self.db_version;
        rebuilt.fenced = self.fenced.clone();
        rebuilt.fenced.insert(site);
        for set in &rest {
            rebuilt.take(set);
        }
        *self = rebuilt;
    }
}
