//! The seeded simulator of deliveries (APP_STATE §11, W1), grown from S1's
//! harness (`s1.rs`): N copies write in the write form, all three ops, on a
//! shared table and on one shaped as an owned one, while sets are sent,
//! relayed by any copy that keeps them, reordered in flight, duplicated,
//! dropped and held by partitions, and a device may be removed while members
//! still relay its sets. Then the copies still standing meet in full, and
//! `judge` holds each to W1, under a rule for a set above a missing re-add
//! (`rule.rs`). A seed names a run under one rule: where a rule changes what
//! a copy keeps, the draws part from another rule's run of the same seed.
//!
//! Not yet, and each lands with what it needs: phones (slice C), a copy
//! crashed mid-apply and two schema versions adopted at different times
//! (they need svrngs-db's own storage and schema).

use std::collections::BTreeSet;

use crate::change::{ChangeSet, Rng, SetId, Site, State, Val};
use crate::reference;
use crate::replica::{Op, Replica, Table};
use crate::rule::Rule;

/// The simulator's schema, as SQL. `votes` is owned (APP_STATE §4, "Who may
/// write which row"), and each device writes it only under its own person's
/// key, as an honest one does. `x` has no type, so SQLite converts no value
/// put in it (its affinity is BLOB), and it takes every kind of value; every
/// other column takes only its own type, its text never a number's.
pub const SCHEMA: &str = "
    CREATE TABLE magnets (id TEXT PRIMARY KEY NOT NULL, pos INTEGER, word TEXT, x);
    CREATE TABLE votes (person_key TEXT NOT NULL, item INTEGER NOT NULL,
      n INTEGER, note TEXT, PRIMARY KEY (person_key, item)) STRICT;";

/// `SCHEMA`'s tables as the model copy reads them, each with its non-key
/// columns in schema order. The live copy holds these to the columns the
/// engine reads from `SCHEMA` (`live.rs`, `LiveCopy::new`).
pub const TABLES: &[Table] = &[
    Table {
        name: "magnets",
        cols: &["pos", "word", "x"],
    },
    Table {
        name: "votes",
        cols: &["n", "note"],
    },
];

/// How a run is shaped: its copies, and its steps before the meet.
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub copies: usize,
    pub steps: usize,
}

impl Shape {
    /// A seed's shape: 2 to 6 copies, 60 to 359 steps.
    pub fn of(seed: u64) -> Shape {
        Shape {
            copies: 2 + (seed % 5) as usize,
            steps: 60 + (seed.wrapping_mul(37) % 300) as usize,
        }
    }
}

/// What a run did, said with a failure.
#[derive(Debug, Default)]
struct Counts {
    written: usize,
    sent: usize,
    duplicated: usize,
    taken: usize,
    dropped: usize,
    held: usize,
    partitions: usize,
    removed: usize,
}

/// A run so far, as its later draws see it (`Sim::fingerprint`): the
/// generator's state, each copy's kept sets, and the sets in flight.
pub type Fingerprint = (u64, Vec<Vec<SetId>>, Vec<(usize, usize, SetId)>);

pub struct Sim<R> {
    rng: Rng,
    copies: Vec<R>,
    /// Every set any device sealed.
    written: Vec<ChangeSet>,
    /// The sets each copy has written or taken.
    seen: Vec<BTreeSet<SetId>>,
    /// Sets in flight: from, to, set.
    wire: Vec<(usize, usize, ChangeSet)>,
    /// Each copy's side while a partition lasts.
    sides: Option<Vec<bool>>,
    removed: BTreeSet<usize>,
    counts: Counts,
}

fn site(i: usize) -> Site {
    Site::of(u8::try_from(i + 1).expect("under 255 copies"))
}

fn value(rng: &mut Rng, col: &str) -> Val {
    if rng.chance(15) {
        return Val::Null;
    }
    let words = ["ice", "milk", "eggs", "jam"];
    match col {
        "pos" | "n" => Val::Integer(rng.below(5) as i64),
        "word" | "note" => Val::text(words[rng.pick(4)]),
        _ => match rng.below(4) {
            0 => Val::Integer(rng.below(3) as i64),
            1 => Val::Real(0.5 + rng.below(3) as f64),
            2 => Val::text(words[rng.pick(4)]),
            _ => Val::Blob(vec![rng.below(3) as u8]),
        },
    }
}

/// One op by `device` on a key from a small set, so writes meet.
fn op(rng: &mut Rng, device: usize) -> Op {
    let (table, key, cols): (&'static str, Vec<Val>, &'static [&'static str]) = if rng.chance(35) {
        (
            "votes",
            vec![Val::Text(format!("person-{device}")), Val::Integer(rng.below(3) as i64)],
            TABLES[1].cols,
        )
    } else {
        (
            "magnets",
            vec![Val::text(["cat", "dog", "owl", "fox"][rng.pick(4)])],
            TABLES[0].cols,
        )
    };
    let mut row: Vec<(&'static str, Val)> = Vec::new();
    for &col in cols {
        if rng.chance(50) {
            row.push((col, value(rng, col)));
        }
    }
    match rng.below(100) {
        0..=44 => {
            if row.is_empty() {
                row.push((cols[0], value(rng, cols[0])));
            }
            Op::Update { table, key, set: row }
        }
        45..=74 => Op::insert(table, key, row),
        _ => Op::Delete { table, key },
    }
}

impl<R: Replica> Sim<R> {
    pub fn new(seed: u64, shape: Shape, make: &dyn Fn(Site) -> R) -> Sim<R> {
        Sim {
            rng: Rng(seed),
            copies: (0..shape.copies).map(|i| make(site(i))).collect(),
            written: Vec::new(),
            seen: vec![BTreeSet::new(); shape.copies],
            wire: Vec::new(),
            sides: None,
            removed: BTreeSet::new(),
            counts: Counts::default(),
        }
    }

    /// Every set any device sealed.
    pub fn written(&self) -> &[ChangeSet] {
        &self.written
    }

    pub fn copies(&self) -> &[R] {
        &self.copies
    }

    /// What the run has done so far, as its later draws can see it. Two runs
    /// of one seed that agree on it make the same draws from here on.
    pub fn fingerprint(&self) -> Fingerprint {
        let kept = self
            .copies
            .iter()
            .map(|c| c.kept().iter().map(ChangeSet::id).collect())
            .collect();
        let wire = self.wire.iter().map(|(f, t, s)| (*f, *t, s.id())).collect();
        (self.rng.0, kept, wire)
    }

    fn standing(&self) -> Vec<usize> {
        (0..self.copies.len()).filter(|c| !self.removed.contains(c)).collect()
    }

    pub fn run(&mut self, steps: usize) {
        for _ in 0..steps {
            match self.rng.below(100) {
                0..=34 => self.write(),
                35..=59 => self.send(),
                60..=94 => self.deliver(),
                95..=97 => self.partition(),
                _ => self.remove(),
            }
        }
    }

    /// A device writes one to three ops in one transaction. A removed
    /// device writes too, and its sets never leave it.
    fn write(&mut self) {
        let d = self.rng.pick(self.copies.len());
        let n = if self.rng.chance(70) { 1 } else { 2 + self.rng.pick(2) };
        let ops: Vec<Op> = (0..n).map(|_| op(&mut self.rng, d)).collect();
        if let Some(set) = self.copies[d].write(&ops) {
            self.seen[d].insert(set.id());
            self.written.push(set);
            self.counts.written += 1;
        }
    }

    /// A copy sends about half the sets it keeps, its own and others', to
    /// another, some twice. A removed device's computer neither sends nor is
    /// sent to: the members' sync refuses it.
    fn send(&mut self) {
        let n = self.copies.len();
        let (from, to) = (self.rng.pick(n), self.rng.pick(n));
        if from == to || self.removed.contains(&from) || self.removed.contains(&to) {
            return;
        }
        for set in self.copies[from].kept() {
            if self.rng.chance(50) {
                if self.rng.chance(10) {
                    self.wire.push((from, to, set.clone()));
                    self.counts.duplicated += 1;
                }
                self.wire.push((from, to, set));
                self.counts.sent += 1;
            }
        }
    }

    /// Any set in flight arrives, so they arrive in any order; one across a
    /// partition waits, and one in ten is lost.
    fn deliver(&mut self) {
        if self.wire.is_empty() {
            return;
        }
        let i = self.rng.pick(self.wire.len());
        let (from, to) = (self.wire[i].0, self.wire[i].1);
        if self.sides.as_ref().is_some_and(|s| s[from] != s[to]) {
            self.counts.held += 1;
            return;
        }
        let (_, to, set) = self.wire.swap_remove(i);
        if self.rng.chance(10) {
            self.counts.dropped += 1;
            return;
        }
        self.copies[to].take(&set);
        self.seen[to].insert(set.id());
        self.counts.taken += 1;
    }

    /// A partition heals, or the copies split in two.
    fn partition(&mut self) {
        if self.sides.take().is_some() || self.copies.len() < 2 {
            return;
        }
        let mut sides: Vec<bool> = (0..self.copies.len()).map(|_| self.rng.chance(50)).collect();
        if sides.iter().all(|s| *s == sides[0]) {
            sides[0] = !sides[0];
        }
        self.sides = Some(sides);
        self.counts.partitions += 1;
    }

    /// At most once a run, while three or more stand, the record removes a
    /// device, and every copy is told. Its computer is cut off from then on;
    /// what members keep of its sets they still relay.
    fn remove(&mut self) {
        let standing = self.standing();
        if !self.removed.is_empty() || standing.len() < 3 || !self.rng.chance(25) {
            return;
        }
        let d = standing[self.rng.pick(standing.len())];
        self.removed.insert(d);
        let gone = self.copies[d].site();
        for copy in &mut self.copies {
            copy.removed(gone);
        }
        self.wire.retain(|(f, t, _)| *f != d && *t != d);
        self.counts.removed += 1;
    }

    /// The full meet: the partition heals, what is in flight between
    /// standing copies arrives in any order, then each sends each the sets it
    /// keeps that the other hasn't taken, some twice, until none is left.
    /// Having taken every set one keeps, the other adopts its marks, where a
    /// rule counts them; the meet goes on while a mark rises.
    pub fn meet(&mut self) {
        self.sides = None;
        let mut wire = std::mem::take(&mut self.wire);
        self.rng.shuffle(&mut wire);
        for (_, to, set) in wire {
            self.copies[to].take(&set);
            self.seen[to].insert(set.id());
        }
        loop {
            let standing = self.standing();
            let mut pairs: Vec<(usize, usize)> = standing
                .iter()
                .flat_map(|&f| standing.iter().filter(move |&&t| t != f).map(move |&t| (f, t)))
                .collect();
            self.rng.shuffle(&mut pairs);
            let mut moved = false;
            for (from, to) in pairs {
                let mut sets: Vec<ChangeSet> = self.copies[from]
                    .kept()
                    .into_iter()
                    .filter(|s| !self.seen[to].contains(&s.id()))
                    .collect();
                self.rng.shuffle(&mut sets);
                for set in &sets {
                    self.copies[to].take(set);
                    if self.rng.chance(5) {
                        self.copies[to].take(set);
                    }
                    self.seen[to].insert(set.id());
                    moved = true;
                }
                let marks = self.copies[from].marks();
                moved |= self.copies[to].adopt(&marks);
            }
            if !moved {
                return;
            }
        }
    }

    /// W1 after a full meet: each standing copy's data is the reference's
    /// under `rule` over every admissible set, those some standing copy wrote
    /// or took; its kept sets cover its own clock rows (invariant 3); and a
    /// newcomer synced from it, its marks adopted, its sets in any order and
    /// one twice, ends equal to it.
    pub fn judge(&mut self, rule: Rule, make: &dyn Fn(Site) -> R) -> Result<(), String> {
        let standing = self.standing();
        let admissible: BTreeSet<SetId> = standing.iter().flat_map(|&c| self.seen[c].iter().copied()).collect();
        let want = reference::under(rule, self.written.iter().filter(|s| admissible.contains(&s.id())));
        for &c in &standing {
            let got = self.copies[c].state();
            if got != want {
                return Err(format!("copy {c} is not the reference's: {}", diff(&got, &want)));
            }
        }
        for &c in &standing {
            let kept: BTreeSet<SetId> = self.copies[c].kept().iter().map(ChangeSet::id).collect();
            let missing = self.copies[c].standing().difference(&kept).count();
            if missing > 0 {
                return Err(format!("copy {c}'s clock rows name {missing} sets it doesn't keep"));
            }
        }
        for &c in &standing {
            let mut sets = self.copies[c].kept();
            self.rng.shuffle(&mut sets);
            let mut newcomer = make(Site::of(250));
            newcomer.adopt(&self.copies[c].marks());
            for set in &sets {
                newcomer.take(set);
            }
            if let Some(set) = sets.first() {
                newcomer.take(set);
            }
            let (got, source) = (newcomer.state(), self.copies[c].state());
            if got != source {
                return Err(format!(
                    "a newcomer synced from copy {c} is not equal to it: {}",
                    diff(&got, &source)
                ));
            }
        }
        Ok(())
    }
}

/// The first row two states differ in.
pub fn diff(got: &State, want: &State) -> String {
    let keys: BTreeSet<_> = got.keys().chain(want.keys()).collect();
    for key in keys {
        let (g, w) = (got.get(key), want.get(key));
        if g != w {
            return format!("{} key {:02x?}: {g:?}, where the other has {w:?}", key.0, key.1);
        }
    }
    "no row differs".into()
}

/// One seeded run, met and judged under today's rule; a failure names its
/// seed and what the run did.
pub fn one<R: Replica>(seed: u64, make: &dyn Fn(Site) -> R) -> Result<(), String> {
    one_under(seed, Rule::Today, make).map(|_| ())
}

/// One seeded run of copies `make` builds under `rule`, met and judged by
/// it; how many sets the standing copies hold at the end, or the failure.
pub fn one_under<R: Replica>(seed: u64, rule: Rule, make: &dyn Fn(Site) -> R) -> Result<usize, String> {
    let shape = Shape::of(seed);
    let mut sim = Sim::new(seed, shape, make);
    sim.run(shape.steps);
    sim.meet();
    sim.judge(rule, make)
        .map_err(|e| format!("seed {seed}, {shape:?}, {:?}: {e}", sim.counts))?;
    let held: BTreeSet<SetId> = sim.standing().iter().flat_map(|&c| sim.copies[c].held()).collect();
    Ok(held.len())
}
