//! The sync's variants (`sync.rs`) on the cases that hold its invariants, and
//! on the gate's seeds and the soak's, which the gate doesn't run:
//! - `carried`: a device's two sets carried through two computers, invariant
//!   11 (svrn-docs APP_STATE.md §7), whose plant is the spike's
//!   highest-taken mark (engine-and-sync C1);
//! - `refused`: a set every copy refuses for good, then two more, synced ten
//!   rounds, invariant 12, whose plant is the spike's refusal that stops its
//!   device and holds its mark (C4);
//! - `cut`: a removed device's set relayed after its removal, invariant 6's
//!   "every set of its that reaches any copy is taken", whose plant is the
//!   spike's refusal of a device that doesn't stand now (spike README, Q3);
//! - `outranked`: a member's marks adopted after a call, which vouch for the
//!   sets it dropped as outranked, whose plant is a refusal one copy alone
//!   makes, the same standing-now rule.
//!
//! Each case runs under the rule svrngs keeps, on any copy. The seeded runs
//! are judged by class (`sim::Class`): copies that split, and copies that
//! agree with each other off the reference, which would stop slice B; the
//! judge is held to telling the second on its own plant.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crate::cases::cat;
use crate::change::{ChangeSet, SetId, Site, State, Val};
use crate::live::{LiveCopy, Pair, named, watched};
use crate::reference;
use crate::replica::{ModelCopy, Op, Replica};
use crate::rule::Rule;
use crate::seeds::GATE;
use crate::sim::{self, Class, TABLES, Verdict};
use crate::sync::{Book, Call, Carry, Sync, Variant};

fn model(site: Site) -> ModelCopy {
    ModelCopy::new(site, TABLES)
}

fn sync(variant: Variant) -> Sync {
    variant.sync().expect("a variant with marks")
}

fn dog() -> Vec<Val> {
    vec![Val::text("dog")]
}

fn add(key: Vec<Val>, pos: i64) -> Op {
    Op::insert("magnets", key, vec![("pos", Val::Integer(pos))])
}

/// A case's copies, each with its marks, under one sync.
struct Net<R> {
    sync: Sync,
    copies: Vec<R>,
    books: Vec<Book>,
    /// The sets each copy took, and those it refused.
    took: Vec<BTreeSet<SetId>>,
    refused: Vec<BTreeSet<SetId>>,
    gone: BTreeSet<Site>,
}

impl<R: Replica> Net<R> {
    fn new(sync: Sync, n: u8, make: &dyn Fn(Site) -> R) -> Net<R> {
        Net {
            sync,
            copies: (1..=n).map(|i| make(Site::of(i))).collect(),
            books: (1..=n).map(|i| Book::of(Site::of(i))).collect(),
            took: vec![BTreeSet::new(); usize::from(n)],
            refused: vec![BTreeSet::new(); usize::from(n)],
            gone: BTreeSet::new(),
        }
    }

    fn write(&mut self, c: usize, ops: &[Op], what: &str) -> ChangeSet {
        let set = self.copies[c].write(ops).unwrap_or_else(|| panic!("{what}"));
        self.sync.wrote(&mut self.books[c], set.id());
        self.took[c].insert(set.id());
        set
    }

    /// A call taken by `to`; the sets it carried.
    fn take(&mut self, to: usize, call: &Call) -> Vec<SetId> {
        let got = self
            .sync
            .ingest(&mut self.copies[to], &mut self.books[to], call, &self.gone);
        self.took[to].extend(got.taken);
        self.refused[to].extend(got.refused);
        call.sets.iter().map(ChangeSet::id).collect()
    }

    /// One call from `from` to `to` by `to`'s marks; the sets it carried.
    fn call(&mut self, from: usize, to: usize) -> Vec<SetId> {
        let (f, t) = (
            (&self.copies[from], &self.books[from]),
            (&self.copies[to], &self.books[to]),
        );
        let call = self.sync.call_from(f, t);
        self.take(to, &call)
    }

    /// The copies meet by calls until no book changes.
    fn meet(&mut self, standing: &[usize]) {
        for round in 0.. {
            assert!(round < 20, "the meet settles");
            let before = self.books.clone();
            for &from in standing {
                for &to in standing.iter().filter(|&&t| t != from) {
                    self.call(from, to);
                }
            }
            if self.books == before {
                return;
            }
        }
    }

    /// Each of these sets that a copy's mark counts and the copy neither took
    /// nor refused, whose values would change what it took (`sets` holds all
    /// it took).
    fn passed(&self, c: usize, sets: &[&ChangeSet]) -> Vec<String> {
        let took: Vec<&ChangeSet> = sets
            .iter()
            .copied()
            .filter(|s| self.took[c].contains(&s.id()))
            .collect();
        let here = reference::state(TABLES, took.iter().copied());
        sets.iter()
            .filter(|s| {
                let id = s.id();
                self.books[c].holds(id) && !self.took[c].contains(&id) && !self.refused[c].contains(&id)
            })
            .filter(|s| reference::state(TABLES, took.iter().copied().chain([**s])) != here)
            .map(|s| named(s.id()))
            .collect()
    }
}

/// A board as a person sees it: each magnet and its pos.
fn board(state: &State) -> String {
    let each: Vec<String> = state
        .iter()
        .filter(|(key, _)| key.0 == "magnets")
        .map(|(key, row)| {
            let name = ["cat", "dog"]
                .into_iter()
                .find(|n| crate::change::pack(&[Val::text(n)]) == key.1)
                .unwrap_or("?");
            match row.get("pos") {
                Some(Val::Integer(p)) => format!("{name} {p}"),
                _ => name.to_owned(),
            }
        })
        .collect();
    format!("{{{}}}", each.join(", "))
}

fn marks(book: &Book) -> String {
    let each: Vec<String> = book.marks().iter().map(|(s, m)| format!("{} at {m}", s.0[0])).collect();
    format!("[{}]", each.join(", "))
}

/// A case's verdict: what is wrong, if anything.
fn verdict(wrong: Vec<String>) -> Result<(), String> {
    if wrong.is_empty() {
        Ok(())
    } else {
        Err(wrong.join("; "))
    }
}

/// A device's two sets carried through two computers (engine-and-sync C1):
/// W (site 1) adds the cat (1/1), then the dog (1/2). Each is carried on its
/// own, as a phone's sets are, 1/1 through X (site 2) and 1/2 through Y (site
/// 3), before X and Y meet. W holds no copy in the meet, as a phone holds
/// none. Then X and Y meet by calls. Each must end on the reference over
/// both, and neither's mark for W may pass a set it lacks (invariant 11).
pub fn carried<R: Replica>(sync: Sync, make: &dyn Fn(Site) -> R) -> (String, Result<(), String>) {
    const W: usize = 0;
    const X: usize = 1;
    const Y: usize = 2;
    let mut net = Net::new(sync, 3, make);
    let w1 = net.write(W, &[add(cat(), 1)], "W adds the cat");
    let w2 = net.write(W, &[add(dog(), 2)], "W adds the dog");
    net.take(X, &Call::carried(&w1));
    net.take(Y, &Call::carried(&w2));
    let mut said = vec![format!(
        "  after the carries: X's marks {}, Y's marks {}",
        marks(&net.books[X]),
        marks(&net.books[Y])
    )];
    net.meet(&[X, Y]);
    let want = reference::state(TABLES, [&w1, &w2]);
    said.push(format!("  the reference over 1/1 and 1/2: {}", board(&want)));
    let mut wrong = Vec::new();
    for (c, name) in [(X, "X"), (Y, "Y")] {
        let state = net.copies[c].state();
        let passed = net.passed(c, &[&w1, &w2]);
        said.push(format!(
            "  {name}: {}, its marks {}, its mark passing {passed:?}",
            board(&state),
            marks(&net.books[c])
        ));
        if !passed.is_empty() {
            wrong.push(format!("{name}'s mark for W passes {passed:?}, which it lacks"));
        }
        if state != want {
            wrong.push(format!("{name} is not the reference's"));
        }
    }
    if net.copies[X].state() != net.copies[Y].state() {
        wrong.insert(0, "X and Y split".into());
    }
    (said.join("\n"), verdict(wrong))
}

/// A refused set, fetched once and stalling nothing (engine-and-sync C4): H
/// (site 1), a hostile computer, adds the cat (1/1), then a vote in B's
/// person's row (1/2), which every copy refuses for good (APP_STATE §4, "Who
/// may write which row"; `sync::breach`), then adds the dog (1/3) and moves
/// the cat (1/4). B (site 2) syncs from H ten rounds, a call a round by its
/// marks. 1/2 must be sent once, no set sent twice, and B end on the
/// reference over the other three (invariant 12).
pub fn refused<R: Replica>(sync: Sync, make: &dyn Fn(Site) -> R) -> (String, Result<(), String>) {
    const H: usize = 0;
    const B: usize = 1;
    let mut net = Net::new(sync, 2, make);
    let h1 = net.write(H, &[add(cat(), 1)], "H adds the cat");
    let vote = Op::insert(
        "votes",
        vec![Val::text("person-1"), Val::Integer(0)],
        vec![("n", Val::Integer(1))],
    );
    let h2 = net.write(H, &[vote], "H writes B's person's vote");
    let h3 = net.write(H, &[add(dog(), 2)], "H adds the dog");
    let to_3 = Op::Update {
        table: "magnets",
        key: cat(),
        set: vec![("pos", Val::Integer(3))],
    };
    let h4 = net.write(H, &[to_3], "H moves the cat");
    assert!(crate::sync::breach(&h2) && ![&h1, &h3, &h4].iter().any(|s| crate::sync::breach(s)));
    let mut sent: BTreeMap<SetId, usize> = BTreeMap::new();
    let mut said = Vec::new();
    for round in 1..=10 {
        let carried = net.call(H, B);
        for id in &carried {
            *sent.entry(*id).or_default() += 1;
        }
        let names: Vec<String> = carried.iter().map(|id| named(*id)).collect();
        said.push(format!(
            "  round {round}: H sends {names:?}; B's marks {}",
            marks(&net.books[B])
        ));
    }
    let resent: usize = sent.values().map(|n| n - 1).sum();
    let want = reference::state(TABLES, [&h1, &h3, &h4]);
    let got = net.copies[B].state();
    said.push(format!(
        "  the reference over 1/1, 1/3 and 1/4: {}; B: {}; 1/2 sent {} times; sets sent again across the ten rounds: {resent}",
        board(&want),
        board(&got),
        sent.get(&h2.id()).copied().unwrap_or(0)
    ));
    let mut wrong = Vec::new();
    if sent.get(&h2.id()) != Some(&1) {
        wrong.push(format!(
            "1/2 is fetched {} times",
            sent.get(&h2.id()).copied().unwrap_or(0)
        ));
    }
    if resent > 0 {
        wrong.push(format!("{resent} sets are sent again"));
    }
    let stalled: Vec<String> = [&h3, &h4]
        .iter()
        .filter(|s| !net.took[B].contains(&s.id()))
        .map(|s| named(s.id()))
        .collect();
    if !stalled.is_empty() {
        wrong.push(format!("B never takes {stalled:?}"));
    }
    if got != want {
        wrong.push("B is not the reference's".into());
    }
    (said.join("\n"), verdict(wrong))
}

/// A removed device's set relayed after its removal (spike README, Q3): W
/// (site 1) adds the cat (1/1), and X (site 2) takes it. W is removed. Then X
/// and Y (site 3) meet by calls. Every set of a removed device that reaches
/// a copy is taken (APP_STATE §4, "A removed device"), so X and Y must
/// agree, on the reference over 1/1.
pub fn cut<R: Replica>(sync: Sync, make: &dyn Fn(Site) -> R) -> (String, Result<(), String>) {
    const W: usize = 0;
    const X: usize = 1;
    const Y: usize = 2;
    let mut net = Net::new(sync, 3, make);
    let w1 = net.write(W, &[add(cat(), 1)], "W adds the cat");
    net.take(X, &Call::carried(&w1));
    net.gone.insert(net.copies[W].site());
    net.meet(&[X, Y]);
    let want = reference::state(TABLES, [&w1]);
    let (x, y) = (net.copies[X].state(), net.copies[Y].state());
    let said = format!(
        "  the reference over 1/1: {}; X: {}, its marks {}; Y: {}, its marks {}",
        board(&want),
        board(&x),
        marks(&net.books[X]),
        board(&y),
        marks(&net.books[Y])
    );
    let mut wrong = Vec::new();
    if x != y {
        wrong.push("X and Y split".to_owned());
    }
    for (state, name) in [(&x, "X"), (&y, "Y")] {
        if *state != want {
            wrong.push(format!("{name} is not the reference's"));
        }
    }
    (said, verdict(wrong))
}

/// A set outranked by one a copy refused (seed 772 of the gate's, under the
/// spike's standing-now rule, cut to its moves): E (site 2) adds the cat at
/// pos 1 (2/1), which clocks each of its columns. D (site 1) takes it and
/// sets each of them, the cat to 2 with word milk and x 1 (1/1), outranking
/// it. X (site 3) takes both, so it keeps 1/1 and drops 2/1, nothing of which
/// stands there. D is removed. Then Y (site 4) calls X first, then X, Y and E
/// meet by calls. A member's marks adopted vouch for what it dropped as
/// outranked by what it sends, so Y must take what X sends: every copy ends on
/// the reference over both, D's cat, and no copy's marks pass a set whose
/// values it lacks.
pub fn outranked<R: Replica>(sync: Sync, make: &dyn Fn(Site) -> R) -> (String, Result<(), String>) {
    const D: usize = 0;
    const E: usize = 1;
    const X: usize = 2;
    const Y: usize = 3;
    let mut net = Net::new(sync, 4, make);
    let e1 = net.write(E, &[add(cat(), 1)], "E adds the cat");
    net.take(D, &Call::carried(&e1));
    let every = Op::Update {
        table: "magnets",
        key: cat(),
        set: vec![
            ("pos", Val::Integer(2)),
            ("word", Val::text("milk")),
            ("x", Val::Integer(1)),
        ],
    };
    let d1 = net.write(D, &[every], "D sets each of the cat's columns");
    net.take(X, &Call::carried(&e1));
    net.take(X, &Call::carried(&d1));
    let kept: Vec<String> = net.copies[X].kept().iter().map(|s| named(s.id())).collect();
    net.gone.insert(net.copies[D].site());
    net.call(X, Y);
    let mut said = vec![format!(
        "  X keeps {kept:?}; after Y's call to X, Y's marks {}",
        marks(&net.books[Y])
    )];
    net.meet(&[X, Y, E]);
    let want = reference::state(TABLES, [&e1, &d1]);
    said.push(format!("  the reference over 2/1 and 1/1: {}", board(&want)));
    let mut wrong = Vec::new();
    for (c, name) in [(X, "X"), (Y, "Y"), (E, "E")] {
        let state = net.copies[c].state();
        let passed = net.passed(c, &[&e1, &d1]);
        said.push(format!(
            "  {name}: {}, its marks {}, its mark passing {passed:?}",
            board(&state),
            marks(&net.books[c])
        ));
        if !passed.is_empty() {
            wrong.push(format!("{name}'s marks pass {passed:?}, which it lacks"));
        }
        if state != want {
            wrong.push(format!("{name} is not the reference's"));
        }
    }
    (said.join("\n"), verdict(wrong))
}

type CaseFn<R> = fn(Sync, &dyn Fn(Site) -> R) -> (String, Result<(), String>);

/// A case by its name.
fn case<R: Replica>(name: &str) -> CaseFn<R> {
    let cases: [(&str, CaseFn<R>); 4] = [
        ("two sets carried", carried::<R>),
        ("a refused set", refused::<R>),
        ("a removed device's set", cut::<R>),
        ("a set outranked by one refused", outranked::<R>),
    ];
    cases
        .into_iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("no case {name}"))
        .1
}

/// One case under a variant on model copies, live copies and pairs, each
/// said; the verdicts by who ran it.
fn on_every_copy(name: &str, variant: Variant) -> Vec<(String, Result<(), String>)> {
    let s = sync(variant);
    let mut out = Vec::new();
    let (said, v) = case::<ModelCopy>(name)(s, &model);
    println!("{name}, {}, model copies:\n{said}", variant.name());
    out.push(("model copies".to_owned(), v));
    let (said, v) = case::<LiveCopy>(name)(s, &LiveCopy::new);
    println!("{name}, {}, live copies:\n{said}", variant.name());
    out.push(("live copies".to_owned(), v));
    let pair = case::<Pair>(name);
    let ((said, v), parted) = watched(format!("{name}, {}", variant.name()), || pair(s, &Pair::new));
    println!("{name}, {}, pairs:\n{said}", variant.name());
    out.push(("pairs".to_owned(), v));
    out.push((
        "live copies against the model's".to_owned(),
        if parted.is_empty() {
            Ok(())
        } else {
            Err(parted.join("; "))
        },
    ));
    for (who, v) in &out {
        println!(
            "  {name}, {}, {who}: {}",
            variant.name(),
            v.as_ref().map_or_else(|e| format!("FAILED {e}"), |_| "passed".into())
        );
    }
    out
}

fn failed(verdicts: &[(String, Result<(), String>)]) -> Vec<String> {
    verdicts
        .iter()
        .filter(|(_, v)| v.is_err())
        .map(|(who, v)| format!("{who}: {}", v.as_ref().unwrap_err()))
        .collect()
}

/// Invariant 11 under §4's marks: neither copy's mark passes a set it lacks,
/// and both end on the reference, on model copies, live copies and pairs.
/// Its plant is the spike's highest-taken mark (below).
#[test]
fn a_mark_never_passes_a_set_this_copy_lacks() {
    let f = failed(&on_every_copy("two sets carried", Variant::Marks));
    assert!(f.is_empty(), "{f:?}");
}

/// The plant of the test above, kept: under the spike's highest-taken mark,
/// Y's mark for W goes to 2 with 1/2, so 1/1 is never sent it, and X and Y
/// split, on every copy.
#[test]
fn the_spike_s_highest_taken_mark_passes_a_carried_set() {
    let verdicts = on_every_copy("two sets carried", Variant::Highest);
    for (who, v) in &verdicts[..3] {
        let e = v.as_ref().expect_err(who);
        assert!(
            e.contains("X and Y split") && e.contains("Y's mark for W passes [\"1/1\"]"),
            "{who}: {e}"
        );
    }
}

/// Invariant 12 under §4's marks: the refused set is sent once, nothing is
/// sent twice, and the two sets after it are taken, on model copies, live
/// copies and pairs. Its plant is the spike's refusal that stops its device
/// and holds its mark (below).
#[test]
fn a_refused_set_is_fetched_once_and_stalls_nothing_after_it() {
    let f = failed(&on_every_copy("a refused set", Variant::Marks));
    assert!(f.is_empty(), "{f:?}");
}

/// The plant of the test above, kept: under the spike's refusal, B's mark for
/// H holds at 1, so every round H sends 1/2, 1/3 and 1/4 again, 27 sets sent
/// again across ten rounds, and B never takes 1/3 or 1/4.
#[test]
fn the_spike_s_refusal_refetches_its_set_and_stalls_its_device() {
    let verdicts = on_every_copy("a refused set", Variant::Stops);
    for (who, v) in &verdicts[..3] {
        let e = v.as_ref().expect_err(who);
        assert!(
            e.contains("1/2 is fetched 10 times")
                && e.contains("27 sets are sent again")
                && e.contains("B never takes [\"1/3\", \"1/4\"]"),
            "{who}: {e}"
        );
    }
}

/// A removed device's set that reaches a copy is taken there under §4's
/// marks, so the copies agree, on model copies, live copies and pairs. Its
/// plant is the spike's refusal of a device that doesn't stand now (below).
#[test]
fn a_removed_device_s_set_relayed_is_taken() {
    let f = failed(&on_every_copy("a removed device's set", Variant::Marks));
    assert!(f.is_empty(), "{f:?}");
}

/// The plant of the test above, kept: Y refuses the set X took before the
/// removal, and the copies split for good.
#[test]
fn the_spike_s_standing_now_splits_on_a_removed_device_s_set() {
    let verdicts = on_every_copy("a removed device's set", Variant::Standing);
    for (who, v) in &verdicts[..3] {
        let e = v.as_ref().expect_err(who);
        assert!(e.contains("X and Y split"), "{who}: {e}");
    }
}

/// A member's marks adopted vouch only for what this copy holds or holds
/// outranked, under §4's marks, where every copy refuses alike: on model
/// copies, live copies and pairs. Its plant is the spike's standing-now rule,
/// which Y alone applies (below).
#[test]
fn a_member_s_marks_adopted_pass_no_set_whose_values_this_copy_lacks() {
    let f = failed(&on_every_copy("a set outranked by one refused", Variant::Marks));
    assert!(f.is_empty(), "{f:?}");
}

/// The plant of the test above, kept: Y refuses D's move, adopts X's mark for
/// E, and so never takes E's add, which X dropped as outranked by the move. E
/// stands, and its write is lost at Y.
#[test]
fn a_refusal_one_copy_alone_makes_lets_adopted_marks_pass_a_standing_device_s_set() {
    let verdicts = on_every_copy("a set outranked by one refused", Variant::Standing);
    for (who, v) in &verdicts[..3] {
        let e = v.as_ref().expect_err(who);
        assert!(e.contains("Y's marks pass [\"2/1\"], which it lacks"), "{who}: {e}");
    }
}

/// What one variant's runs over these seeds did, said: failures by class,
/// and what its sync did.
#[derive(Default)]
struct Tally {
    runs: usize,
    failed: BTreeMap<Class, Vec<u64>>,
    /// Failures by what the failing copy lacked and why (`why`).
    by_why: BTreeMap<&'static str, usize>,
    said: Vec<String>,
    passed: (usize, usize),
    refetched: (usize, usize, usize),
    resent: (usize, usize, usize),
    resent_gone: usize,
    late: usize,
}

impl Tally {
    fn add(&mut self, seed: u64, v: Verdict) {
        self.runs += 1;
        if let Some((class, e)) = v.failure {
            self.failed.entry(class).or_default().push(seed);
            *self.by_why.entry(why(&e)).or_default() += 1;
            self.said.push(format!("{}: {e}", class.name()));
        }
        let each = |t: &mut (usize, usize, usize), n: usize| {
            t.0 += n;
            t.1 += usize::from(n > 0);
            t.2 = t.2.max(n);
        };
        self.passed.0 += v.passed;
        self.passed.1 += usize::from(v.passed > 0);
        each(&mut self.refetched, v.refetched);
        each(&mut self.resent, v.resent);
        self.resent_gone += v.resent_gone;
        self.late += v.late;
    }

    fn line(&self, name: &str, seeds: &std::ops::Range<u64>, secs: f64, threads: u64) -> String {
        let mut classes = Vec::new();
        for (class, seeds) in &self.failed {
            let shown: Vec<String> = seeds.iter().take(40).map(u64::to_string).collect();
            let more = if seeds.len() > 40 {
                format!(" and {} more", seeds.len() - 40)
            } else {
                String::new()
            };
            classes.push(format!(
                "{} {}: [{}{more}]",
                seeds.len(),
                class.name(),
                shown.join(", ")
            ));
        }
        let failed: usize = self.failed.values().map(Vec::len).sum();
        let whys: Vec<String> = self.by_why.iter().map(|(w, n)| format!("{n} {w}")).collect();
        let whys = match whys.is_empty() {
            true => String::new(),
            false => format!(", by why: {}", whys.join(", ")),
        };
        format!(
            "{name}: seeds {}..{}, {} runs, {failed} failed{}{whys}; marks passing a set lacked: {} in {} runs; refused sets fetched again in ten rounds: {} in {} runs, at most {}; sets sent again: {} in {} runs, at most {}, {} of them a removed device's; taken late: {}; {secs:.1} s on {threads} threads",
            seeds.start,
            seeds.end,
            self.runs,
            if classes.is_empty() {
                String::new()
            } else {
                format!(" ({})", classes.join("; "))
            },
            self.passed.0,
            self.passed.1,
            self.refetched.0,
            self.refetched.1,
            self.refetched.2,
            self.resent.0,
            self.resent.1,
            self.resent.2,
            self.resent_gone,
            self.late,
        )
    }
}

/// Why a run failed, read from what its judge said: a copy lacking a set its
/// mark passes (the highest-taken mark's), one it refused because its device
/// was removed (the standing-now rule's), a newcomer that refused its
/// source's sets (the same rule's), or none of those.
fn why(e: &str) -> &'static str {
    if e.contains("its mark passes it") {
        "a copy's mark passes a set it lacks"
    } else if e.contains("(refused here, its device removed)") {
        "a copy refused a removed device's set"
    } else if e.contains("the newcomer refused") {
        "a newcomer refused its source's sets"
    } else if e.contains("(never came") {
        "a copy lacks a set that never came to it"
    } else {
        "unexplained"
    }
}

/// Each variant's runs over these seeds on model copies, shared among the
/// computer's threads: each failure said, then the variant's line.
#[expect(clippy::disallowed_methods, reason = "each variant's time over its seeds, said")]
fn runs(variant: Variant, carry: Carry, seeds: std::ops::Range<u64>) -> Tally {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    let start = Instant::now();
    let mut each: Vec<(u64, Verdict)> = std::thread::scope(|s| {
        let all: Vec<_> = (0..threads)
            .map(|t| {
                let seeds = seeds.clone();
                s.spawn(move || {
                    seeds
                        .filter(|seed| seed % threads == t)
                        .map(|seed| (seed, sim::one_synced(seed, Rule::default(), variant, carry, &model)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        all.into_iter()
            .flat_map(|t| t.join().expect("a thread of runs ran"))
            .collect()
    });
    each.sort_by_key(|(seed, _)| *seed);
    let mut tally = Tally::default();
    for (seed, v) in each {
        tally.add(seed, v);
    }
    let name = match variant {
        Variant::Base => "base".to_owned(),
        _ => format!("{}, {}", variant.name(), carry.name()),
    };
    for e in &tally.said {
        println!("FAILED, {name}: {e}");
    }
    println!("{}", tally.line(&name, &seeds, start.elapsed().as_secs_f64(), threads));
    tally
}

/// The variants `W1_SYNCS` names, as `marks,highest`, else every one; and
/// the carries `W1_CARRIES` names, else both.
fn chosen() -> Vec<(Variant, Carry)> {
    let variants: Vec<Variant> = match std::env::var("W1_SYNCS") {
        Ok(names) => names.split(',').map(Variant::named).collect(),
        Err(_) => Variant::ALL.to_vec(),
    };
    let carries: Vec<Carry> = match std::env::var("W1_CARRIES") {
        Ok(names) => names
            .split(',')
            .map(|n| {
                Carry::ALL
                    .into_iter()
                    .find(|c| c.name() == n)
                    .unwrap_or_else(|| panic!("no carry {n}"))
            })
            .collect(),
        Err(_) => Carry::ALL.to_vec(),
    };
    let mut out = Vec::new();
    for v in variants {
        if v == Variant::Base {
            out.push((v, Carry::Sets));
            continue;
        }
        out.extend(carries.iter().map(|c| (v, *c)));
    }
    out
}

pub fn range(var: &str, default: &str) -> std::ops::Range<u64> {
    let seeds = std::env::var(var).unwrap_or_else(|_| default.into());
    let (first, end) = seeds.split_once("..").expect("<first>..<end>");
    first.parse().expect("a first seed")..end.parse().expect("an end seed")
}

/// Each variant on the gate's seeds, model copies, under the rule svrngs
/// keeps. It fails where the base or §4's marks fails, or where any copies
/// agree off the reference; the spike's rules may fail. Run `cargo test -p
/// svrngs-db --test w1 -- --ignored each_sync_on_the_gate --nocapture`,
/// `W1_SYNCS` and `W1_CARRIES` to name them.
#[test]
#[ignore = "the spike's rules fail seeds; the gate runs the base on these seeds"]
fn each_sync_on_the_gate_s_seeds() {
    held(chosen().into_iter().map(|(v, c)| (v, runs(v, c, GATE))).collect());
}

/// The soak under each variant: `W1_SEEDS` as `<first>..<end>`, else the
/// 100,000 runs APP_STATE §11 names, model copies only. Run `cargo test
/// --release -p svrngs-db --test w1 -- --ignored each_sync_s_soak
/// --nocapture`.
#[test]
#[ignore = "the soak: 100,000 runs under each variant, not the gate's"]
fn each_sync_s_soak() {
    let seeds = range("W1_SEEDS", "0..100000");
    held(
        chosen()
            .into_iter()
            .map(|(v, c)| (v, runs(v, c, seeds.clone())))
            .collect(),
    );
}

/// What the runs must hold: the base and §4's marks on the reference, and no
/// copies anywhere that agree off it, which would stop slice B (MAGNETS_BUILD
/// §3).
fn held(tallies: Vec<(Variant, Tally)>) {
    for (variant, tally) in &tallies {
        assert!(
            !tally.failed.contains_key(&Class::Agree),
            "{}: copies agree off the reference",
            variant.name()
        );
        if matches!(
            variant,
            Variant::Base | Variant::Marks | Variant::Haves | Variant::Vouched
        ) {
            assert!(tally.failed.is_empty(), "{} fails", variant.name());
        }
    }
}

/// The seeds a variant parts on model copies, run again on live copies alone
/// and on pairs, so each failure is the rule's and not the model's: over
/// `W1_SEEDS` as `<first>..<end>`, else the gate's, each variant and carry
/// `W1_SYNCS` and `W1_CARRIES` name, and with `W1_LIVE_ALL` set, every seed.
/// Each seed where the live copies' class isn't the model's, or a pair parts,
/// is said; then each variant's line.
/// Run `cargo test -p svrngs-db --test w1 -- --ignored
/// the_seeds_that_part_on_live --nocapture`.
#[test]
#[ignore = "live copies on the seeds each variant parts, not the gate's"]
#[expect(clippy::disallowed_methods, reason = "each variant's time over its seeds, said")]
fn the_seeds_that_part_on_live_copies() {
    let seeds = range("W1_SEEDS", "0..1000");
    let all_seeds = std::env::var("W1_LIVE_ALL").is_ok();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    for (variant, carry) in chosen() {
        let start = Instant::now();
        let each: Vec<(u64, Option<Class>, Option<String>)> = std::thread::scope(|s| {
            let all: Vec<_> = (0..threads)
                .map(|t| {
                    let seeds = seeds.clone();
                    s.spawn(move || {
                        let class = |v: &Verdict| v.failure.as_ref().map(|f| f.0);
                        let mut out = Vec::new();
                        for seed in seeds.filter(|seed| seed % threads == t) {
                            let m = sim::one_synced(seed, Rule::default(), variant, carry, &model);
                            let want = class(&m);
                            if want.is_none() && !all_seeds {
                                continue;
                            }
                            let l = sim::one_synced(seed, Rule::default(), variant, carry, &LiveCopy::new);
                            let (p, parted) = watched(format!("seed {seed}"), || {
                                sim::one_synced(seed, Rule::default(), variant, carry, &Pair::new)
                            });
                            let odd = (class(&l) != want || class(&p) != want || !parted.is_empty()).then(|| {
                                let said = |v: &Verdict| {
                                    v.failure
                                        .as_ref()
                                        .map_or("passed".to_owned(), |(c, e)| format!("{}: {e}", c.name()))
                                };
                                format!(
                                    "seed {seed}: model {}\n  live {}\n  pairs {}\n  parted: {parted:?}",
                                    said(&m),
                                    said(&l),
                                    said(&p)
                                )
                            });
                            out.push((seed, want, odd));
                        }
                        out
                    })
                })
                .collect();
            all.into_iter()
                .flat_map(|t| t.join().expect("a thread of seeds ran"))
                .collect()
        });
        let name = format!("{}, {}", variant.name(), carry.name());
        let mut odd: Vec<&(u64, Option<Class>, Option<String>)> = each.iter().filter(|e| e.2.is_some()).collect();
        odd.sort_by_key(|e| e.0);
        for (_, _, said) in &odd {
            println!("ODD, {name}, {}", said.as_deref().unwrap_or_default());
        }
        let mut by: BTreeMap<Class, usize> = BTreeMap::new();
        for class in each.iter().filter_map(|e| e.1) {
            *by.entry(class).or_default() += 1;
        }
        let parted: usize = by.values().sum();
        let by: Vec<String> = by.iter().map(|(c, n)| format!("{n} {}", c.name())).collect();
        println!(
            "{name}: seeds {}..{}, {parted} parted on model copies ({}); {} run on live copies and pairs, {} not alike or a pair parted; {:.1} s on {threads} threads",
            seeds.start,
            seeds.end,
            by.join(", "),
            each.len(),
            odd.len(),
            start.elapsed().as_secs_f64()
        );
    }
}

/// The shortest run of each seed `W1_SYNC_SEEDS` lists that still fails
/// under the variant and carry named: its steps cut to the fewest after which
/// the meet and the judge still fail, said with what failed. A seed's run
/// cut short is the same run up to the cut, so this is its smallest prefix.
#[test]
#[ignore = "a probe of the seeds a variant fails, named by W1_SYNC_SEEDS"]
fn the_shortest_run_that_still_parts() {
    let seeds: Vec<u64> = std::env::var("W1_SYNC_SEEDS")
        .expect("W1_SYNC_SEEDS")
        .split(',')
        .map(|s| s.trim().parse().expect("a seed"))
        .collect();
    for (variant, carry) in chosen() {
        for &seed in &seeds {
            let shape = sim::Shape::of(seed);
            let fails = |steps: usize| {
                let mut sim = sim::Sim::synced(seed, shape, &model, variant, carry);
                sim.run(steps);
                sim.meet();
                let v = sim.verdict(Rule::default(), &model);
                v.failure.map(|(c, e)| format!("{}: {e}; {}", c.name(), sim.said()))
            };
            let shortest = (1..=shape.steps).find_map(|n| fails(n).map(|e| (n, e)));
            match shortest {
                Some((n, e)) => println!(
                    "{} {}, seed {seed}, {shape:?}: fails from {n} steps of {}: {e}",
                    variant.name(),
                    carry.name(),
                    shape.steps
                ),
                None => println!(
                    "{} {}, seed {seed}, {shape:?}: no cut fails",
                    variant.name(),
                    carry.name()
                ),
            }
        }
    }
}

/// The judge tells copies that agree off the reference from copies that
/// split, the class that would stop slice B (MAGNETS_BUILD §3, "Stops if"):
/// its plant is the withdrawn fence (`Plant::UndoRemoved`), where every copy
/// rebuilds without a removed device's sets and they all agree, wrongly, as
/// the review's case does (APP_STATE §4, "A removed device").
#[test]
fn the_judge_tells_copies_that_agree_off_the_reference() {
    let make = |site| ModelCopy::planted(site, TABLES, crate::replica::Plant::UndoRemoved);
    let first = GATE.into_iter().find_map(|seed| {
        sim::one_synced(seed, Rule::default(), Variant::Base, Carry::Sets, &make)
            .failure
            .map(|(class, e)| (class, format!("{}: {e}", class.name())))
    });
    let (class, said) = first.expect("a seed catches the plant");
    println!("caught: {said}");
    assert_eq!(class, Class::Agree, "{said}");
}
