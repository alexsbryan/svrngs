//! S1's recorded outcomes, rebuilt from the model (svrn-docs
//! `rings/current/magnets-spikes/`, its README §S1, its harness
//! `code/src/bin/s1.rs`, its run `data/s1-run-2000x80.txt`), under the rule
//! the operator kept, Fly's (the operator, 2026-11-17; APP_STATE §4, "How it
//! merges"). Only those rows are rebuilt: the others are merge-by-time
//! patches svrngs doesn't run.
//!
//! The run records per-seed outcomes only as totals over 2,000 seeds, so the
//! model can't be held to real cr-sqlite seed by seed. What can be rebuilt:
//! - **the scripted cases**, each copy's final value as printed;
//! - **the seeded run's totals** for the ten Fly rows whose runs S1's seed
//!   fixes alone (`run_matches_s1_s_recorded_totals`). Fly's rule never reads
//!   a stamp, so a clock changes those runs only by holding a set, which
//!   only Hold with the 2 s skews did (lines 98 and 102).
//!
//! The spike's `the_merge_rule` is here too (svrn-docs
//! `rings/current/app-state-spike/spike.patch:2139-2190`, its run
//! `run-2026-11-17.txt:28`).

use std::collections::{BTreeMap, BTreeSet};

use crate::change::{Change, ChangeSet, Rng, RowKey, SENTINEL, SetId, Site, State, Val, pack};
use crate::reference;
use crate::replica::{ModelCopy, Op, Plant, Replica, Table};
use crate::rule::Rule;

const T: &[Table] = &[Table {
    name: "t",
    cols: &["a", "b"],
    defaults: &[],
}];
const MAGNETS: &[Table] = &[Table {
    name: "magnets",
    cols: &["pos"],
    defaults: &[],
}];
const CARDS: &[Table] = &[Table {
    name: "cards",
    cols: &["title", "col"],
    defaults: &[],
}];
/// Every table S1's cases write, for the reference to read each by.
const S1: &[Table] = &[T[0], MAGNETS[0], CARDS[0]];

/// The engine S1 ran, Fly's fork before svrngs's patch: today's rule
/// (`rule.rs`). Under N its totals part from the recorded ones: in seeds 172
/// and 1554 of order Any, a column change cleared a value of the row's
/// earlier life that today's engine keeps, and a later local update set it
/// back to that value, a change under N and none under today's, so one more
/// set each, 59311 and not 59309.
const S1_RULE: Rule = Rule::Today;

/// A scripted case's devices, and every set written, in the order written.
struct Case {
    copies: Vec<ModelCopy>,
    /// Which sets each copy wrote or took.
    had: Vec<BTreeSet<SetId>>,
    log: Vec<ChangeSet>,
}

impl Case {
    fn new(tables: &'static [Table], n: u8) -> Case {
        Case {
            copies: (1..=n)
                .map(|i| ModelCopy::ruled(Site::of(i), tables, S1_RULE))
                .collect(),
            had: vec![BTreeSet::new(); n as usize],
            log: Vec::new(),
        }
    }

    fn write(&mut self, d: usize, op: Op) -> SetId {
        let set = self.copies[d]
            .write(&[op])
            .expect("each scripted write changes something");
        self.had[d].insert(set.id());
        self.log.push(set.clone());
        set.id()
    }

    /// `to` takes these sets, in this order.
    fn give(&mut self, to: usize, ids: &[SetId]) {
        for id in ids {
            let set = self.log.iter().find(|s| s.id() == *id).expect("a set written").clone();
            self.copies[to].take(&set);
            self.had[to].insert(*id);
        }
    }

    /// The sets `d` holds, in the order they were written.
    fn sets_of(&self, d: usize) -> Vec<&ChangeSet> {
        self.log.iter().filter(|s| self.had[d].contains(&s.id())).collect()
    }
}

fn key(v: Val) -> Vec<Val> {
    vec![v]
}

/// S1's `MOVE` (s1.rs:340): the cat magnet to `pos`, an upsert.
fn moved(pos: Option<i64>) -> Op {
    Op::insert(
        "magnets",
        key(Val::text("cat")),
        vec![("pos", pos.map_or(Val::Null, Val::Integer))],
    )
}

/// What S1 printed for a cell: `no row`, `NULL`, a number, or a quoted text.
fn shown(state: &State, table: &str, k: Val, col: &str) -> String {
    match state.get(&(table.to_owned(), pack(&[k]))) {
        None => "no row".into(),
        Some(row) => match row.get(col) {
            None => "NULL".into(),
            Some(Val::Integer(i)) => i.to_string(),
            Some(Val::Text(t)) => format!("'{t}'"),
            Some(v) => format!("{v:?}"),
        },
    }
}

fn pos(state: &State) -> String {
    shown(state, "magnets", Val::text("cat"), "pos")
}

fn cell(state: &State) -> String {
    shown(state, "t", Val::Integer(1), "a")
}

/// S1's `magnets` (s1.rs:349-378): Ana taps the magnet while apart, Ben
/// moves it once a minute later, then they meet.
fn magnets(touched: bool, taps: &[Option<i64>], ben: i64) -> Case {
    let mut case = Case::new(MAGNETS, 2);
    if touched {
        let s = case.write(0, moved(None));
        case.give(1, &[s]);
    }
    let ana: Vec<SetId> = taps.iter().map(|p| case.write(0, moved(*p))).collect();
    let ben = case.write(1, moved(Some(ben)));
    case.give(1, &ana);
    case.give(0, &[ben]);
    case
}

/// S1's `magnets_tray_as_delete` (s1.rs:381-403).
fn tray_as_delete() -> Case {
    let mut case = Case::new(MAGNETS, 2);
    let s = case.write(0, moved(Some(1)));
    case.give(1, &[s]);
    let out = || Op::Delete {
        table: "magnets",
        key: key(Val::text("cat")),
    };
    let ana: Vec<SetId> = [out(), moved(Some(3)), out(), moved(Some(5)), out()]
        .into_iter()
        .map(|op| case.write(0, op))
        .collect();
    let ben = case.write(1, moved(Some(7)));
    case.give(1, &ana);
    case.give(0, &[ben]);
    case
}

fn set_a(v: &str) -> Op {
    Op::Update {
        table: "t",
        key: key(Val::Integer(1)),
        set: vec![("a", Val::text(v))],
    }
}

fn insert_a(v: &str) -> Op {
    Op::insert("t", key(Val::Integer(1)), vec![("a", Val::text(v))])
}

fn delete_1() -> Op {
    Op::Delete {
        table: "t",
        key: key(Val::Integer(1)),
    }
}

/// S1's `causal` (s1.rs:411-439): Ben edits after taking Ana's five. Under
/// Fly's rule its four clock settings are one case.
fn causal() -> Case {
    let mut case = Case::new(T, 2);
    let mut ana = vec![case.write(0, insert_a("ana0"))];
    ana.extend((1..5).map(|i| case.write(0, set_a(&format!("ana{i}")))));
    case.give(1, &ana);
    let ben = case.write(1, set_a("ben-after-seeing"));
    case.give(0, &[ben]);
    case
}

/// S1's d1 (s1.rs:458-470): a delete against a later edit that never saw it.
fn d1() -> Case {
    let mut case = Case::new(T, 2);
    let s = case.write(0, insert_a("x"));
    case.give(1, &[s]);
    let ana = case.write(0, delete_1());
    let ben = case.write(1, set_a("ben-later"));
    case.give(1, &[ana]);
    case.give(0, &[ben]);
    case
}

/// S1's d3 (s1.rs:471-489): deleted and re-added twice, against one later delete.
fn d3() -> Case {
    let mut case = Case::new(T, 2);
    let s = case.write(0, insert_a("x"));
    case.give(1, &[s]);
    let ana: Vec<SetId> = [delete_1(), insert_a("ana1"), delete_1(), insert_a("ana2")]
        .into_iter()
        .map(|op| case.write(0, op))
        .collect();
    let ben = case.write(1, delete_1());
    case.give(1, &ana);
    case.give(0, &[ben]);
    case
}

/// S1's d4 (s1.rs:490-511): Yan deletes and re-adds; Xia, apart, edits later
/// and takes only the re-add. Xia is copy 0, Yan copy 1.
fn d4() -> Case {
    let mut case = Case::new(T, 2);
    let s = case.write(1, insert_a("first"));
    case.give(0, &[s]);
    let _delete = case.write(1, delete_1());
    let reinsert = case.write(1, insert_a("yan-new"));
    let old = case.write(0, set_a("xia-later"));
    case.give(0, &[reinsert]);
    case.give(1, &[old]);
    case
}

/// The spike's `the_merge_rule`: Ana updates one column five times, Ben once
/// after; then one update each of another.
fn the_merge_rule() -> Case {
    let mut case = Case::new(CARDS, 2);
    let card = |id: &str, title: &str| {
        Op::insert(
            "cards",
            key(Val::text(id)),
            vec![("title", Val::text(title)), ("col", Val::text("todo"))],
        )
    };
    let col = |id: &str, v: &str| Op::Update {
        table: "cards",
        key: key(Val::text(id)),
        set: vec![("col", Val::text(v))],
    };
    let first = [case.write(0, card("c1", "milk")), case.write(0, card("c2", "eggs"))];
    case.give(1, &first);
    let mut ana: Vec<SetId> = ["a1", "a2", "a3", "a4", "a5"]
        .iter()
        .map(|v| case.write(0, col("c1", v)))
        .collect();
    let mut ben = vec![case.write(1, col("c1", "zzz-ben-later"))];
    ana.push(case.write(0, col("c2", "ana-once")));
    ben.push(case.write(1, col("c2", "ben-once")));
    case.give(1, &ana);
    case.give(0, &ben);
    case
}

fn card_col(id: &'static str) -> impl Fn(&State) -> String {
    move |s| shown(s, "cards", Val::text(id), "col")
}

/// One recorded outcome: where it was printed, the case, the cell read, and
/// what each copy printed.
struct Recorded {
    at: &'static str,
    case: fn() -> Case,
    read: Box<dyn Fn(&State) -> String>,
    want: [&'static str; 2],
}

fn recorded() -> Vec<Recorded> {
    let r = |at, case, read: Box<dyn Fn(&State) -> String>, want| Recorded { at, case, read, want };
    vec![
        r(
            "s1-run-2000x80.txt:2",
            || magnets(true, &[Some(1), None, Some(2), None, Some(3)], 7),
            Box::new(pos),
            ["3", "3"],
        ),
        r(
            "s1-run-2000x80.txt:3",
            || magnets(false, &[Some(1), None, Some(2), None, Some(3)], 7),
            Box::new(pos),
            ["3", "3"],
        ),
        r(
            "s1-run-2000x80.txt:4",
            || magnets(true, &[Some(1), None, Some(2), None], 7),
            Box::new(pos),
            ["NULL", "NULL"],
        ),
        r(
            "s1-run-2000x80.txt:5-6",
            || magnets(true, &[Some(9)], 7),
            Box::new(pos),
            ["9", "9"],
        ),
        r(
            "s1-run-2000x80.txt:7",
            tray_as_delete,
            Box::new(pos),
            ["no row", "no row"],
        ),
        r(
            "s1-run-2000x80.txt:26-29",
            causal,
            Box::new(cell),
            ["'ben-after-seeing'", "'ben-after-seeing'"],
        ),
        r("s1-run-2000x80.txt:46", d1, Box::new(cell), ["no row", "no row"]),
        r("s1-run-2000x80.txt:47", d3, Box::new(cell), ["'ana2'", "'ana2'"]),
        r("s1-run-2000x80.txt:48", d4, Box::new(cell), ["'yan-new'", "'yan-new'"]),
        r(
            "run-2026-11-17.txt:28, c1",
            the_merge_rule,
            Box::new(card_col("c1")),
            ["'a5'", "'a5'"],
        ),
        r(
            "run-2026-11-17.txt:28, c2",
            the_merge_rule,
            Box::new(card_col("c2")),
            ["'ben-once'", "'ben-once'"],
        ),
    ]
}

/// Every recorded outcome the copies or a model miss: the copies' own
/// merge, and `model` over the sets each copy holds, in the order written.
fn misses(model: impl Fn(&[&ChangeSet]) -> State) -> Vec<String> {
    let mut out = Vec::new();
    for r in recorded() {
        let case = (r.case)();
        for (d, want) in r.want.iter().enumerate() {
            let copy = (r.read)(&case.copies[d].state());
            let modelled = (r.read)(&model(&case.sets_of(d)));
            if copy != *want {
                out.push(format!("{}: copy {d}'s merge gives {copy}, S1 printed {want}", r.at));
            }
            if modelled != *want {
                out.push(format!(
                    "{}: the model over copy {d}'s sets gives {modelled}, S1 printed {want}",
                    r.at
                ));
            }
        }
    }
    out
}

fn the_reference(sets: &[&ChangeSet]) -> State {
    reference::state(S1, sets.iter().copied())
}

/// The plant MAGNETS_BUILD §3 names for W: a model that lets the later edit
/// win. Sets come in the order written, so a change at the row's causal
/// length or longer takes its cell from any before it.
fn later_wins(sets: &[&ChangeSet]) -> State {
    // Each row's causal length, and its values at it.
    let mut rows: BTreeMap<RowKey, (i64, BTreeMap<String, Val>)> = BTreeMap::new();
    for c in sets.iter().flat_map(|s| s.changes()) {
        let line = rows
            .entry((c.table.clone(), c.pk.clone()))
            .or_insert((c.cl, BTreeMap::new()));
        if c.cl < line.0 {
            continue;
        }
        if c.cl > line.0 {
            *line = (c.cl, BTreeMap::new());
        }
        if c.cid != SENTINEL && c.cl % 2 == 1 {
            line.1.insert(c.cid.clone(), c.val.clone());
        }
    }
    rows.into_iter()
        .filter(|(_, (cl, _))| cl % 2 == 1)
        .map(|(k, (_, cells))| (k, cells.into_iter().filter(|(_, v)| *v != Val::Null).collect()))
        .collect()
}

#[test]
fn the_model_gives_s1_s_recorded_outcomes() {
    let misses = misses(the_reference);
    assert!(misses.is_empty(), "{misses:#?}");
}

#[test]
fn a_model_that_lets_the_later_edit_win_misses_them() {
    let misses = misses(later_wins);
    for at in [
        "s1-run-2000x80.txt:2:",
        "s1-run-2000x80.txt:3:",
        "run-2026-11-17.txt:28, c1:",
    ] {
        assert!(
            misses.iter().any(|m| m.starts_with(at)),
            "the plant should miss {at}; it missed {misses:#?}"
        );
    }
}

/// S1's `causal_unstamped` (s1.rs:443-454): 200 fresh pairs, Ben's edit made
/// after taking Ana's; S1's site ids were random, so here each pair's are
/// two different numbers, either way round.
#[test]
fn the_edit_made_after_seeing_wins_in_200_fresh_pairs() {
    let mut lost = 0;
    for i in 0..200u8 {
        let (ana, ben) = if i % 2 == 0 {
            (i / 2 + 1, 255 - i / 2)
        } else {
            (255 - i / 2, i / 2 + 1)
        };
        let mut a = ModelCopy::ruled(Site::of(ana), T, S1_RULE);
        let mut b = ModelCopy::ruled(Site::of(ben), T, S1_RULE);
        let first = a.write(&[insert_a("ana")]).unwrap();
        b.take(&first);
        let after = b
            .write(&[Op::Update {
                table: "t",
                key: key(Val::Integer(1)),
                set: vec![("a", Val::text("ben-after-seeing"))],
            }])
            .unwrap();
        a.take(&after);
        let want = "'ben-after-seeing'";
        let model = cell(&reference::state(T, [&first, &after]));
        lost += (cell(&a.state()) != want || cell(&b.state()) != want || model != want) as u32;
    }
    // s1-run-2000x80.txt:42: "lost in 0 of 200 pairs".
    assert_eq!(lost, 0);
}

/// S1's `Order` (s1.rs:97-103).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Order {
    /// Any set in any order, a device's own out of order too.
    Any,
    /// A device's sets in its own order, devices interleaved.
    PerOrigin,
}

/// S1's `Set` (s1.rs:105-111).
struct Set {
    origin: usize,
    n: usize,
    set: ChangeSet,
    /// The sets its writer had taken when it wrote this one.
    seen: Vec<bool>,
}

/// What S1's run counts (s1.rs:113-126), less what reads a stamp.
#[derive(Debug, Default, PartialEq)]
struct Totals {
    data_equal: u64,
    clock_rows_equal: u64,
    column_clocks_equal: u64,
    sets: u64,
    violations: u64,
    unmapped: u64,
    /// Not S1's: runs where a copy's data is not the reference's over every set.
    off_the_reference: u64,
}

/// S1's `op` (s1.rs:128-142), draw for draw.
fn op(rng: &mut Rng) -> Op {
    let k = key(Val::Integer(rng.below(4) as i64));
    let roll = rng.below(100);
    let val = Val::Text(format!("v{}", rng.below(12)));
    let val2 = Val::Text(format!("v{}", rng.below(12)));
    match roll {
        0..=29 => Op::Update {
            table: "t",
            key: k,
            set: vec![("a", val)],
        },
        30..=44 => Op::Update {
            table: "t",
            key: k,
            set: vec![("b", val)],
        },
        45..=74 => Op::Insert {
            table: "t",
            key: k,
            row: vec![("a", val), ("b", val2)],
            on_conflict: vec!["a"],
        },
        _ => Op::Delete { table: "t", key: k },
    }
}

/// S1's `run` (s1.rs:148-288) under Fly's rule, draw for draw; its clock
/// reads only stamps, so it is left out. Adds its outcome to `totals`.
fn run(order: Order, seed: u64, steps: usize, plant: Plant, totals: &mut Totals) {
    let mut rng = Rng(seed);
    let mut copies: Vec<ModelCopy> = (1..=3)
        .map(|i| ModelCopy::made(Site::of(i), T, plant, S1_RULE))
        .collect();
    let mut sets: Vec<Set> = Vec::new();
    let mut known: Vec<Vec<bool>> = vec![Vec::new(); 3];

    let deliver = |rng: &mut Rng,
                   sets: &[Set],
                   known: &mut [Vec<bool>],
                   copies: &mut [ModelCopy],
                   from: usize,
                   to: usize,
                   all: bool| {
        let mut picked: Vec<usize> = (0..sets.len())
            .filter(|&i| known[from][i] && !known[to][i] && (all || rng.chance(50)))
            .collect();
        rng.shuffle(&mut picked);
        if order == Order::PerOrigin {
            let mut queues: Vec<Vec<usize>> = vec![Vec::new(); 3];
            let mut keyed = picked.clone();
            keyed.sort_by_key(|&i| (sets[i].origin, std::cmp::Reverse(sets[i].n)));
            for i in keyed {
                queues[sets[i].origin].push(i);
            }
            picked.clear();
            while queues.iter().any(|q| !q.is_empty()) {
                let live: Vec<usize> = (0..3).filter(|&o| !queues[o].is_empty()).collect();
                let o = live[rng.below(live.len() as u64) as usize];
                picked.push(queues[o].pop().unwrap());
            }
        }
        for i in picked {
            if order == Order::PerOrigin {
                let o = sets[i].origin;
                if (0..sets.len()).any(|j| sets[j].origin == o && sets[j].n < sets[i].n && !known[to][j]) {
                    continue;
                }
            }
            copies[to].take(&sets[i].set);
            known[to][i] = true;
        }
    };

    for _ in 0..steps {
        let _now = rng.below(50);
        if rng.chance(65) {
            let c = rng.below(3) as usize;
            let op = op(&mut rng);
            let Some(set) = copies[c].write(&[op]) else { continue };
            let n = sets.iter().filter(|s| s.origin == c).count();
            let seen = known[c].clone();
            sets.push(Set {
                origin: c,
                n,
                set,
                seen,
            });
            for (k, row) in known.iter_mut().enumerate() {
                row.push(k == c);
            }
        } else {
            let to = rng.below(3) as usize;
            let from = (to + 1 + rng.below(2) as usize) % 3;
            deliver(&mut rng, &sets, &mut known, &mut copies, from, to, false);
        }
    }
    for _ in 0..60 {
        let mut pairs: Vec<(usize, usize)> = (0..3)
            .flat_map(|f| (0..3).filter(move |&t| t != f).map(move |t| (f, t)))
            .collect();
        rng.shuffle(&mut pairs);
        for (f, t) in pairs {
            deliver(&mut rng, &sets, &mut known, &mut copies, f, t, true);
        }
        if known.iter().all(|k| k.iter().all(|&b| b)) {
            break;
        }
    }
    assert!(known.iter().all(|k| k.iter().all(|&b| b)), "every set delivered");

    let data: Vec<State> = copies.iter().map(|c| c.state()).collect();
    let clocks: Vec<Vec<Change>> = copies.iter().map(|c| c.clock_rows()).collect();
    let columns: Vec<Vec<&Change>> = clocks
        .iter()
        .map(|m| m.iter().filter(|c| c.cid != SENTINEL).collect())
        .collect();
    totals.data_equal += data.windows(2).all(|w| w[0] == w[1]) as u64;
    totals.clock_rows_equal += clocks.windows(2).all(|w| w[0] == w[1]) as u64;
    totals.column_clocks_equal += columns.windows(2).all(|w| w[0] == w[1]) as u64;
    totals.sets += sets.len() as u64;
    let want = reference::state(T, sets.iter().map(|s| &s.set));
    totals.off_the_reference += data.iter().any(|d| *d != want) as u64;

    // S1's causality count (s1.rs:266-286): a set written after taking a
    // cell's final winner, writing that cell at the same causal length, lost.
    for clock in &clocks {
        for ch in clock.iter().filter(|c| c.cid != SENTINEL) {
            let w = sets.iter().position(|s| {
                s.set.id().0 == ch.site
                    && s.set
                        .changes()
                        .iter()
                        .any(|r| r.db_version == ch.db_version && r.pk == ch.pk && r.cid == ch.cid)
            });
            let Some(w) = w else {
                totals.unmapped += 1;
                continue;
            };
            for (i, s) in sets.iter().enumerate() {
                if i != w
                    && s.seen.get(w).copied().unwrap_or(false)
                    && s.set
                        .changes()
                        .iter()
                        .any(|r| r.pk == ch.pk && r.cid == ch.cid && r.cl == ch.cl)
                {
                    totals.violations += 1;
                }
            }
        }
    }
}

fn totals(order: Order, plant: Plant) -> Totals {
    let mut t = Totals::default();
    for seed in 0..2000 {
        run(order, seed, 80, plant, &mut t);
    }
    t
}

/// s1-run-2000x80.txt:58, :66, :74, :82, :90 (order Any) and :62, :70, :78,
/// :86, :94 (PerOrigin): "data equal 2000/2000", "clock rows equal
/// 1046/2000" and "1501/2000", "column clocks equal 2000/2000", "sets 59309"
/// and "59268", "causality violations 0", "unmapped 0".
fn s1_s(order: Order) -> Totals {
    let (clock_rows_equal, sets) = match order {
        Order::Any => (1046, 59309),
        Order::PerOrigin => (1501, 59268),
    };
    Totals {
        data_equal: 2000,
        clock_rows_equal,
        column_clocks_equal: 2000,
        sets,
        violations: 0,
        unmapped: 0,
        off_the_reference: 0,
    }
}

#[test]
fn run_matches_s1_s_recorded_totals() {
    for order in [Order::Any, Order::PerOrigin] {
        assert_eq!(totals(order, Plant::None), s1_s(order), "order {order:?}");
    }
}

/// The totals' plant: cr-sqlite's resurrect without its zeroing. Its copies
/// split, and leave the reference, as S1's patch without its guard did
/// (README §S1, "The guard is needed").
#[test]
fn clocks_kept_on_resurrect_miss_s1_s_totals() {
    let t = totals(Order::Any, Plant::KeepClocksOnResurrect);
    assert!(t.data_equal < 2000 && t.off_the_reference > 0, "{t:?}");
}
