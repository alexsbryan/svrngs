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
//! Its sync, as above, is the base: every set that arrives is taken, and the
//! meet sends each copy what it hasn't taken. Its variants (`sync.rs`) keep
//! marks, send by them, and take by them, as APP_STATE §4 says and as the
//! spike did; under one, the meet is calls by marks, and after it the
//! standing copies sync ten rounds more, each set sent again counted. A seed
//! names a run under one sync as under one rule.
//!
//! Not yet, and each lands with what it needs: phones (slice C), a copy
//! crashed mid-apply and two schema versions adopted at different times
//! (they need svrngs-db's own storage and schema).

use std::collections::BTreeSet;

use crate::change::{ChangeSet, Rng, SetId, Site, State, Val};
use crate::reference;
use crate::replica::{Op, Replica, Table};
use crate::rule::Rule;
use crate::sync::{Book, Call, Carry, Ingested, Sync, Variant};

/// The simulator's schema, as SQL. `votes` is owned (APP_STATE §4, "Who may
/// write which row"), and each device writes it only under its own person's
/// key, as an honest one does. `x` has no type, so SQLite converts no value
/// put in it (its affinity is BLOB), and it takes every kind of value; every
/// other column takes only its own type, its text never a number's. `tiles`
/// has a NOT NULL column with a DEFAULT, which cr-sqlite allows only so
/// (`tableinfo.rs:1117-1130`). The simulator writes only `magnets` and
/// `votes`, so a seed's run is as it was before `tiles`; the case of column
/// defaults writes `tiles` (`cases.rs`).
pub const SCHEMA: &str = "
    CREATE TABLE magnets (id TEXT PRIMARY KEY NOT NULL, pos INTEGER, word TEXT, x);
    CREATE TABLE votes (person_key TEXT NOT NULL, item INTEGER NOT NULL,
      n INTEGER, note TEXT, PRIMARY KEY (person_key, item)) STRICT;
    CREATE TABLE tiles (id TEXT PRIMARY KEY NOT NULL, pos INTEGER,
      size INTEGER NOT NULL DEFAULT 3) STRICT;";

/// `SCHEMA`'s tables as the model copy reads them, each with its non-key
/// columns in schema order and their defaults. The live copy holds these to
/// what the engine reads from `SCHEMA` (`live.rs`, `LiveCopy::new`).
pub const TABLES: &[Table] = &[
    Table {
        name: "magnets",
        cols: &["pos", "word", "x"],
        defaults: &[],
    },
    Table {
        name: "votes",
        cols: &["n", "note"],
        defaults: &[],
    },
    Table {
        name: "tiles",
        cols: &["pos", "size"],
        defaults: &[("size", Val::Integer(3))],
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
    /// Under a variant: calls sent, sets passed by as counted already, sets
    /// refused, and sets stopped behind a refusal.
    calls: usize,
    marked: usize,
    refused: usize,
    stopped: usize,
}

/// W1's failure, by what the standing copies show (MAGNETS_BUILD §3, slice B,
/// "Stops if").
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// Two standing copies hold different states.
    Split,
    /// Every standing copy holds the same state, and it isn't the reference's.
    Agree,
    /// The copies are the reference's, and a newcomer synced from one isn't
    /// equal to it.
    Newcomer,
    /// A copy's clock rows name a set it doesn't keep (invariant 3).
    Uncovered,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Split => "copies split",
            Class::Agree => "copies agree, off the reference",
            Class::Newcomer => "a newcomer differs",
            Class::Uncovered => "clock rows uncovered",
        }
    }
}

/// A run judged: W1's failure, if any, by its class; and what the sync did.
#[derive(Debug, Default)]
pub struct Verdict {
    pub failure: Option<(Class, String)>,
    /// The sets the standing copies hold under the rule, H or P.
    pub held: usize,
    /// Under a variant, invariant 11: each standing copy's admissible sets it
    /// lacks and didn't refuse, that its mark counts, and that would change
    /// it.
    pub passed: usize,
    /// Under a variant, in the ten rounds after the meet: refused sets fetched
    /// again, and the sets stopped behind them (invariant 12).
    pub refetched: usize,
    /// Sets sent again in those rounds that the copy counts already, and of
    /// those, the ones a removed device wrote.
    pub resent: usize,
    pub resent_gone: usize,
    /// Sets taken in those rounds, which the meet should have left none of.
    pub late: usize,
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
    /// The variant's sync, and how sets travel under it; none for the base.
    synced: Option<(Sync, Carry)>,
    /// Each copy's marks, under a variant.
    books: Vec<Book>,
    /// The sets each copy refused, under a variant.
    refused: Vec<BTreeSet<SetId>>,
    /// Calls in flight under `Carry::Calls`: from, to, call.
    calls: Vec<(usize, usize, Call)>,
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
            synced: None,
            books: (0..shape.copies).map(|c| Book::of(site(c))).collect(),
            refused: vec![BTreeSet::new(); shape.copies],
            calls: Vec::new(),
        }
    }

    /// A run under a variant of the sync, its sets carried as `carry` says
    /// until the meet; the base's is `Sim::new`'s.
    pub fn synced(seed: u64, shape: Shape, make: &dyn Fn(Site) -> R, variant: Variant, carry: Carry) -> Sim<R> {
        let mut sim = Sim::new(seed, shape, make);
        sim.synced = variant.sync().map(|s| (s, carry));
        sim
    }

    /// The sites of the devices removed.
    fn gone(&self) -> BTreeSet<Site> {
        self.removed.iter().map(|&c| self.copies[c].site()).collect()
    }

    /// A call taken by copy `to` under the variant's sync, and what it took
    /// and refused noted.
    fn ingest(&mut self, to: usize, call: &Call) -> Ingested {
        let (sync, _) = self.synced.expect("a call is taken under a variant's sync");
        let gone = self.gone();
        let got = sync.ingest(&mut self.copies[to], &mut self.books[to], call, &gone);
        self.seen[to].extend(got.taken.iter().copied());
        self.refused[to].extend(got.refused.iter().copied());
        self.counts.taken += got.taken.len();
        self.counts.marked += got.held.len();
        self.counts.refused += got.refused.len();
        self.counts.stopped += got.stopped.len();
        got
    }

    /// A set delivered to copy `to`: taken, under the base; under a variant,
    /// taken by its sync as one set carried.
    fn offer(&mut self, to: usize, set: &ChangeSet) {
        if self.synced.is_some() {
            self.ingest(to, &Call::carried(set));
            return;
        }
        self.copies[to].take(set);
        self.seen[to].insert(set.id());
        self.counts.taken += 1;
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
            if let Some((sync, _)) = self.synced {
                sync.wrote(&mut self.books[d], set.id());
            }
            self.seen[d].insert(set.id());
            self.written.push(set);
            self.counts.written += 1;
        }
    }

    /// A copy sends about half the sets it keeps, its own and others', to
    /// another, some twice; or under `Carry::Calls`, one call, sometimes
    /// twice. A removed device's computer neither sends nor is sent to: the
    /// members' sync refuses it.
    fn send(&mut self) {
        let n = self.copies.len();
        let (from, to) = (self.rng.pick(n), self.rng.pick(n));
        if from == to || self.removed.contains(&from) || self.removed.contains(&to) {
            return;
        }
        if let Some((sync, Carry::Calls)) = self.synced {
            let call = sync.call_from(
                (&self.copies[from], &self.books[from]),
                (&self.copies[to], &self.books[to]),
            );
            self.counts.sent += call.sets.len();
            self.counts.calls += 1;
            if self.rng.chance(10) {
                self.calls.push((from, to, call.clone()));
                self.counts.duplicated += 1;
            }
            self.calls.push((from, to, call));
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
    /// partition waits, and one in ten is lost. Under `Carry::Calls`, so does
    /// a call, whole.
    fn deliver(&mut self) {
        if let Some((_, Carry::Calls)) = self.synced {
            if self.calls.is_empty() {
                return;
            }
            let i = self.rng.pick(self.calls.len());
            let (from, to) = (self.calls[i].0, self.calls[i].1);
            if self.sides.as_ref().is_some_and(|s| s[from] != s[to]) {
                self.counts.held += 1;
                return;
            }
            let (_, to, call) = self.calls.swap_remove(i);
            if self.rng.chance(10) {
                self.counts.dropped += call.sets.len();
                return;
            }
            self.ingest(to, &call);
            return;
        }
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
        self.offer(to, &set);
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
        self.calls.retain(|(f, t, _)| *f != d && *t != d);
        self.counts.removed += 1;
    }

    /// The full meet: the partition heals, what is in flight between
    /// standing copies arrives in any order, then each sends each the sets it
    /// keeps that the other hasn't taken, some twice, until none is left.
    /// Having taken every set one keeps, the other adopts its marks, where a
    /// rule counts them; the meet goes on while a mark rises. Under a variant,
    /// what is in flight arrives, then each calls each by its marks, until no
    /// book changes.
    pub fn meet(&mut self) {
        self.sides = None;
        if let Some((sync, _)) = self.synced {
            return self.meet_by(sync);
        }
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

    /// The full meet under a variant's sync.
    fn meet_by(&mut self, sync: Sync) {
        let mut wire = std::mem::take(&mut self.wire);
        self.rng.shuffle(&mut wire);
        for (_, to, set) in wire {
            self.ingest(to, &Call::carried(&set));
        }
        let mut calls = std::mem::take(&mut self.calls);
        self.rng.shuffle(&mut calls);
        for (_, to, call) in calls {
            self.ingest(to, &call);
        }
        for round in 0.. {
            assert!(round < 10_000, "the meet settles");
            let standing = self.standing();
            let mut pairs: Vec<(usize, usize)> = standing
                .iter()
                .flat_map(|&f| standing.iter().filter(move |&&t| t != f).map(move |&t| (f, t)))
                .collect();
            self.rng.shuffle(&mut pairs);
            let mut moved = false;
            for (from, to) in pairs {
                let before = self.books[to].clone();
                let call = sync.call_from(
                    (&self.copies[from], &self.books[from]),
                    (&self.copies[to], &self.books[to]),
                );
                self.ingest(to, &call);
                if self.rng.chance(5) {
                    self.ingest(to, &call);
                }
                let marks = self.copies[from].marks();
                moved |= self.copies[to].adopt(&marks);
                moved |= self.books[to] != before;
            }
            if !moved {
                return;
            }
        }
    }

    /// The admissible sets copy `c` lacks whose values would change it, each
    /// said with why: refused here, its mark passing it, or never come.
    fn lacking(&self, c: usize, rule: Rule, admitted: &[&ChangeSet]) -> Vec<(SetId, &'static str)> {
        let taken: Vec<&ChangeSet> = self.written.iter().filter(|s| self.seen[c].contains(&s.id())).collect();
        let here = reference::under(rule, TABLES, taken.iter().copied());
        let mut out = Vec::new();
        for s in admitted.iter().filter(|s| !self.seen[c].contains(&s.id())) {
            let mut with = taken.clone();
            with.push(s);
            if reference::under(rule, TABLES, with) == here {
                continue;
            }
            let id = s.id();
            let why = if self.refused[c].contains(&id) {
                "refused here"
            } else if self.synced.is_some() && self.books[c].holds(id) {
                "its mark passes it"
            } else {
                "never came"
            };
            out.push((id, why));
        }
        out
    }

    /// What a failing copy lacks, said.
    fn said_lacking(&self, c: usize, rule: Rule, admitted: &[&ChangeSet]) -> String {
        let lacks = self.lacking(c, rule, admitted);
        if lacks.is_empty() {
            return String::new();
        }
        let each: Vec<String> = lacks
            .iter()
            .map(|(id, why)| {
                let gone = if self.removed.iter().any(|&r| self.copies[r].site() == id.0) {
                    ", its device removed"
                } else {
                    ""
                };
                format!("{}/{} ({why}{gone})", id.0.0[0], id.1)
            })
            .collect();
        format!("; copy {c} lacks {}", each.join(", "))
    }

    /// W1 after a full meet, by its class, and under a variant what its sync
    /// did (`Verdict`): `judge`'s checks in `judge`'s order, then invariant
    /// 11's count and ten rounds more between the standing copies.
    pub fn verdict(&mut self, rule: Rule, make: &dyn Fn(Site) -> R) -> Verdict {
        let mut v = Verdict::default();
        let standing = self.standing();
        let admissible: BTreeSet<SetId> = standing.iter().flat_map(|&c| self.seen[c].iter().copied()).collect();
        let admitted: Vec<&ChangeSet> = self.written.iter().filter(|s| admissible.contains(&s.id())).collect();
        let want = reference::under(rule, TABLES, admitted.iter().copied());
        let states: Vec<State> = standing.iter().map(|&c| self.copies[c].state()).collect();
        let agree = states.windows(2).all(|w| w[0] == w[1]);
        for (&c, got) in standing.iter().zip(&states) {
            if *got != want {
                let class = if agree { Class::Agree } else { Class::Split };
                let lacks = self.said_lacking(c, rule, &admitted);
                let e = format!("copy {c} is not the reference's: {}{lacks}", diff(got, &want));
                v.failure = Some((class, e));
                break;
            }
        }
        if v.failure.is_none() {
            for &c in &standing {
                let kept: BTreeSet<SetId> = self.copies[c].kept().iter().map(ChangeSet::id).collect();
                let missing = self.copies[c].standing().difference(&kept).count();
                if missing > 0 {
                    let e = format!("copy {c}'s clock rows name {missing} sets it doesn't keep");
                    v.failure = Some((Class::Uncovered, e));
                    break;
                }
            }
        }
        if v.failure.is_none() {
            let gone = self.gone();
            for &c in &standing {
                let mut newcomer = make(Site::of(250));
                newcomer.adopt(&self.copies[c].marks());
                let mut refused = 0;
                match self.synced {
                    None => {
                        let mut sets = self.copies[c].kept();
                        self.rng.shuffle(&mut sets);
                        for set in &sets {
                            newcomer.take(set);
                        }
                        if let Some(set) = sets.first() {
                            newcomer.take(set);
                        }
                    }
                    Some((sync, _)) => {
                        let mut book = Book::of(Site::of(250));
                        let call = sync.call_from((&self.copies[c], &self.books[c]), (&newcomer, &book));
                        refused += sync.ingest(&mut newcomer, &mut book, &call, &gone).refused.len();
                        if let Some(set) = call.sets.first() {
                            sync.ingest(&mut newcomer, &mut book, &Call::carried(set), &gone);
                        }
                    }
                }
                let (got, source) = (newcomer.state(), self.copies[c].state());
                if got != source {
                    let refusals = match refused {
                        0 => String::new(),
                        n => format!("; the newcomer refused {n} of its sets"),
                    };
                    let e = format!(
                        "a newcomer synced from copy {c} is not equal to it: {}{refusals}",
                        diff(&got, &source)
                    );
                    v.failure = Some((Class::Newcomer, e));
                    break;
                }
            }
        }
        v.held = standing
            .iter()
            .flat_map(|&c| self.copies[c].held())
            .collect::<BTreeSet<SetId>>()
            .len();
        let Some((sync, _)) = self.synced else {
            return v;
        };
        v.passed = standing
            .iter()
            .map(|&c| {
                let lacks = self.lacking(c, rule, &admitted);
                lacks.iter().filter(|(_, why)| *why == "its mark passes it").count()
            })
            .sum();
        let gone = self.gone();
        for _ in 0..10 {
            for &from in &standing {
                for &to in standing.iter().filter(|&&t| t != from) {
                    let call = sync.call_from(
                        (&self.copies[from], &self.books[from]),
                        (&self.copies[to], &self.books[to]),
                    );
                    let got = sync.ingest(&mut self.copies[to], &mut self.books[to], &call, &gone);
                    v.refetched += got.refused.len() + got.stopped.len();
                    v.resent += got.held.len();
                    v.resent_gone += got.held.iter().filter(|id| gone.contains(&id.0)).count();
                    v.late += got.taken.len();
                }
            }
        }
        v
    }

    /// What the run did, said with a failure.
    pub fn said(&self) -> String {
        format!("{:?}", self.counts)
    }

    /// W1 after a full meet: each standing copy's data is the reference's
    /// under `rule` over every admissible set, those some standing copy wrote
    /// or took; its kept sets cover its own clock rows (invariant 3); and a
    /// newcomer synced from it, its marks adopted, its sets in any order and
    /// one twice, ends equal to it.
    pub fn judge(&mut self, rule: Rule, make: &dyn Fn(Site) -> R) -> Result<(), String> {
        match self.verdict(rule, make).failure {
            Some((class, e)) => Err(format!("{}: {e}", class.name())),
            None => Ok(()),
        }
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

/// One seeded run, met and judged under the rule svrngs keeps; a failure
/// names its seed and what the run did.
pub fn one<R: Replica>(seed: u64, make: &dyn Fn(Site) -> R) -> Result<(), String> {
    one_under(seed, Rule::default(), make).map(|_| ())
}

/// One seeded run of copies `make` builds under `rule`, met and judged by
/// it; how many sets the standing copies hold at the end, or the failure.
pub fn one_under<R: Replica>(seed: u64, rule: Rule, make: &dyn Fn(Site) -> R) -> Result<usize, String> {
    let v = one_synced(seed, rule, Variant::Base, Carry::Sets, make);
    match v.failure {
        Some((class, e)) => Err(format!("{}: {e}", class.name())),
        None => Ok(v.held),
    }
}

/// One seeded run under `rule` and a variant of the sync, met and judged; a
/// failure names its seed and what the run did.
pub fn one_synced<R: Replica>(
    seed: u64,
    rule: Rule,
    variant: Variant,
    carry: Carry,
    make: &dyn Fn(Site) -> R,
) -> Verdict {
    let shape = Shape::of(seed);
    let mut sim = Sim::synced(seed, shape, make, variant, carry);
    sim.run(shape.steps);
    sim.meet();
    let mut v = sim.verdict(rule, make);
    if let Some((_, e)) = &mut v.failure {
        *e = format!("seed {seed}, {shape:?}, {}: {e}", sim.said());
    }
    v
}
