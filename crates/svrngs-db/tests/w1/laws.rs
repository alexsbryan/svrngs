//! The join's laws. A copy's state is the join of every set it has taken
//! (APP_STATE §4, "A removed device", "Why"), so the reference model must be
//! one: the same sets merge to one state whatever their order, however
//! often one comes, and however they are grouped before they meet. Each law
//! is checked over the sets of seeded histories, written by model copies
//! under the simulator.

use std::collections::BTreeMap;

use crate::change::{ChangeSet, Rng, RowKey, SENTINEL, State, Val};
use crate::reference::Join;
use crate::replica::ModelCopy;
use crate::sim::{Shape, Sim, TABLES};

/// What the laws need of a join.
trait Lattice: Clone {
    fn empty() -> Self;
    fn of(set: &ChangeSet) -> Self;
    fn join(self, other: &Self) -> Self;
    fn state(&self) -> State;
}

impl Lattice for Join {
    fn empty() -> Self {
        Join::default()
    }
    fn of(set: &ChangeSet) -> Self {
        Join::of([set])
    }
    fn join(self, other: &Self) -> Self {
        Join::join(self, other)
    }
    fn state(&self) -> State {
        Join::state(self)
    }
}

/// The laws' plant: a join that keeps the first set it sees. A row one side
/// already holds keeps that side's causal length and values.
#[derive(Clone, Default)]
struct FirstWins(BTreeMap<RowKey, (i64, BTreeMap<String, Val>)>);

impl Lattice for FirstWins {
    fn empty() -> Self {
        FirstWins::default()
    }
    fn of(set: &ChangeSet) -> Self {
        let mut rows = BTreeMap::new();
        for c in set.changes() {
            let line: &mut (i64, BTreeMap<String, Val>) = rows
                .entry((c.table.clone(), c.pk.clone()))
                .or_insert((c.cl, BTreeMap::new()));
            if c.cid != SENTINEL && c.cl % 2 == 1 {
                line.1.insert(c.cid.clone(), c.val.clone());
            }
        }
        FirstWins(rows)
    }
    fn join(mut self, other: &Self) -> Self {
        for (key, line) in &other.0 {
            self.0.entry(key.clone()).or_insert_with(|| line.clone());
        }
        self
    }
    fn state(&self) -> State {
        let rows = self.0.iter().filter(|(_, (cl, _))| cl % 2 == 1);
        rows.map(|(k, (_, cells))| {
            (
                k.clone(),
                cells
                    .iter()
                    .filter(|(_, v)| **v != Val::Null)
                    .map(|(c, v)| (c.clone(), v.clone()))
                    .collect(),
            )
        })
        .collect()
    }
}

fn fold<L: Lattice>(sets: &[&ChangeSet]) -> L {
    sets.iter().fold(L::empty(), |acc, s| acc.join(&L::of(s)))
}

/// Each law on one history's sets, the first broken named.
fn laws<L: Lattice>(sets: &[ChangeSet], rng: &mut Rng) -> Result<(), String> {
    let sets: Vec<&ChangeSet> = sets.iter().collect();
    if sets.is_empty() {
        return Ok(());
    }
    let want = fold::<L>(&sets).state();

    let mut shuffled = sets.clone();
    rng.shuffle(&mut shuffled);
    if fold::<L>(&shuffled).state() != want {
        return Err("order: the same sets in another order merge to another state".into());
    }

    // Duplicates, each after the set it repeats, so order stays as it was.
    let mut doubled = sets.clone();
    for _ in 0..=sets.len() / 3 {
        let i = rng.pick(doubled.len());
        let at = i + 1 + rng.pick(doubled.len() - i);
        doubled.insert(at, doubled[i]);
    }
    if fold::<L>(&doubled).state() != want {
        return Err("duplicates: a set taken again changes the state".into());
    }

    // Grouping: runs of sets, in their order, each joined, then neighbours
    // joined in a random tree.
    let mut groups: Vec<L> = Vec::new();
    let mut rest = &sets[..];
    while !rest.is_empty() {
        let n = 1 + rng.pick(rest.len().min(6));
        groups.push(fold(&rest[..n]));
        rest = &rest[n..];
    }
    while groups.len() > 1 {
        let i = rng.pick(groups.len() - 1);
        let right = groups.remove(i + 1);
        let left = groups.remove(i);
        groups.insert(i, left.join(&right));
    }
    if groups.pop().map_or_else(State::new, |g| g.state()) != want {
        return Err("grouping: joining groups' joins gives another state than joining the sets".into());
    }
    Ok(())
}

/// One seeded history's sets: model copies writing under the simulator.
fn history(seed: u64) -> Vec<ChangeSet> {
    let shape = Shape::of(seed);
    let mut sim = Sim::new(seed, shape, &|site| ModelCopy::new(site, TABLES));
    sim.run(shape.steps);
    sim.written().to_vec()
}

const SEEDS: std::ops::Range<u64> = 0..200;

#[test]
fn the_join_s_laws_hold_over_seeded_histories() {
    for seed in SEEDS {
        let sets = history(seed);
        if let Err(e) = laws::<Join>(&sets, &mut Rng(seed)) {
            panic!("seed {seed}, {} sets: {e}", sets.len());
        }
    }
}

#[test]
fn a_join_that_keeps_the_first_set_it_sees_breaks_the_order_law() {
    let broken = SEEDS
        .into_iter()
        .find_map(|seed| laws::<FirstWins>(&history(seed), &mut Rng(seed)).err());
    let broken = broken.expect("the plant breaks a law on some seed");
    assert!(broken.starts_with("order:"), "{broken}");
}
