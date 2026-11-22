//! Question 9's rules (`rule.rs`; svrn-docs APP_STATE.md §8 q9) on model
//! copies, each built under a rule and judged by the reference under it: on
//! the fixed cases slice B opens with (MAGNETS_BUILD.md §3, "Slice B"), the
//! minimal case, two removals and seeds 2706 and 5691; on the gate's seeds;
//! and on the soak's.
//!
//! A seed names a run under one rule, and a rule that changes what a copy
//! keeps changes the simulator's later draws. So each fixed seed is said
//! with the step its run parts from today's, and the step of today's first
//! moment where a column change brought a standing row into a new life with
//! an earlier value in it (`ModelCopy::earlier`). A rule whose run is still
//! today's at that step met the moment today's copies split on.

use std::collections::BTreeSet;
use std::time::Instant;

use crate::cases::Case;
use crate::change::Site;
use crate::replica::ModelCopy;
use crate::rule::Rule;
use crate::seeds::GATE;
use crate::sim::{self, Shape, Sim, TABLES};

/// The seeds W's soak found today's split on.
const SPLIT: [u64; 2] = [2706, 5691];

fn ruled(rule: Rule) -> impl Fn(Site) -> ModelCopy {
    move |site| ModelCopy::ruled(site, TABLES, rule)
}

/// Question 9's moments the copies of a run met so far.
fn moments(sim: &Sim<ModelCopy>) -> usize {
    sim.copies().iter().map(ModelCopy::earlier).sum()
}

/// A fixed seed's run under `rule` against today's, said: the step its
/// draws part from today's, today's first moment, and the moments each run
/// met.
fn traced(seed: u64, rule: Rule) -> String {
    let shape = Shape::of(seed);
    let (today, theirs) = (ruled(Rule::Today), ruled(rule));
    let mut a = Sim::new(seed, shape, &today);
    let mut b = Sim::new(seed, shape, &theirs);
    let (mut parted, mut first) = (None, None);
    for step in 1..=shape.steps {
        a.run(1);
        b.run(1);
        if first.is_none() && moments(&a) > 0 {
            first = Some(step);
        }
        if parted.is_none() && a.fingerprint() != b.fingerprint() {
            parted = Some(step);
        }
    }
    let (in_today, in_theirs) = (moments(&a), moments(&b));
    a.meet();
    b.meet();
    let parted = match parted {
        Some(step) => format!("parts from today's at step {step} of {}", shape.steps),
        None if a.fingerprint() != b.fingerprint() => "parts from today's at the meet".to_owned(),
        None => "is today's throughout".to_owned(),
    };
    let first = first.map_or("none".to_owned(), |s| format!("step {s}"));
    format!(
        "its run {parted}; today's first moment: {first}; moments before the meet, today's {in_today}, this rule's {in_theirs}; after it, today's {}, this rule's {}",
        moments(&a),
        moments(&b)
    )
}

/// Each fixed case under `rule`, said as it runs: its name and W1's verdict.
fn fixed(rule: Rule) -> Vec<(String, Result<(), String>)> {
    let mut verdicts = Vec::new();
    let mut said = |what: String, v: Result<(), String>| {
        let line = v.as_ref().map_or_else(|e| format!("FAILED {e}"), |_| "passed".into());
        println!("  {}, {what}: {line}", rule.name());
        verdicts.push((what, v));
    };
    for case in Case::ALL {
        println!("{}, {}:", rule.name(), case.name());
        let (_, lines, v) = case.run(rule, &ruled(rule));
        println!("{lines}");
        said(case.name().to_owned(), v);
    }
    for seed in SPLIT {
        println!("{}, seed {seed}, {:?}:", rule.name(), Shape::of(seed));
        println!("  {}", traced(seed, rule));
        let v = sim::one_under(seed, rule, &ruled(rule));
        if let Ok(held) = v {
            println!("  sets held at the end: {held}");
        }
        said(format!("seed {seed}"), v.map(|_| ()));
    }
    verdicts
}

/// The fixed cases' names a rule fails.
fn failing(rule: Rule) -> BTreeSet<String> {
    fixed(rule)
        .into_iter()
        .filter(|(_, v)| v.is_err())
        .map(|(what, _)| what)
        .collect()
}

/// N and H each keep every fixed case on the reference. Its plant is
/// today's rule, which fails each (below).
#[test]
fn n_and_h_keep_each_fixed_case_on_the_reference() {
    for rule in [Rule::NewLifeEmpty, Rule::WaitForLife] {
        let failed = failing(rule);
        assert!(failed.is_empty(), "{} fails {failed:?}", rule.name());
    }
}

/// The plant of the test above, and P's split: today's rule fails every
/// fixed case, and P fails two removals, where the missing re-add is another
/// device's, and nothing else.
#[test]
fn today_s_rule_fails_each_fixed_case_and_p_fails_two_removals() {
    let all: BTreeSet<String> = ["the minimal case", "two removals", "seed 2706", "seed 5691"]
        .map(String::from)
        .into();
    assert_eq!(failing(Rule::Today), all);
    assert_eq!(failing(Rule::Contiguous), BTreeSet::from(["two removals".to_owned()]));
}

/// W1 on the fixed cases under the rule svrngs keeps, `Rule::default()`. It
/// joins the gate in the commit that makes it pass (MAGNETS_BUILD §3, "Slice
/// B"). Run `cargo test -p svrngs-db --test w1 -- --ignored rule_svrngs_keeps
/// --nocapture`.
#[test]
#[ignore = "fails on today's rule, APP_STATE §8 q9; it joins the gate with the rule slice B keeps"]
fn the_fixed_cases_hold_under_the_rule_svrngs_keeps() {
    let failed = failing(Rule::default());
    assert!(failed.is_empty(), "{} fails {failed:?}", Rule::default().name());
}

/// Each rule's runs over these seeds, shared among the computer's threads:
/// each failure said, and the sets the standing copies hold at the end.
#[expect(clippy::disallowed_methods, reason = "each rule's time over its seeds, said")]
fn runs(rule: Rule, seeds: std::ops::Range<u64>) -> Vec<String> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    let start = Instant::now();
    let mut outcomes: Vec<(u64, Result<usize, String>)> = std::thread::scope(|s| {
        let each: Vec<_> = (0..threads)
            .map(|t| {
                let seeds = seeds.clone();
                s.spawn(move || {
                    let make = ruled(rule);
                    seeds
                        .filter(|seed| seed % threads == t)
                        .map(|seed| (seed, sim::one_under(seed, rule, &make)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        each.into_iter()
            .flat_map(|t| t.join().expect("a thread of runs ran"))
            .collect()
    });
    outcomes.sort_by_key(|(seed, _)| *seed);
    let failed: Vec<String> = outcomes
        .iter()
        .filter_map(|(_, v)| v.as_ref().err())
        .map(|e| format!("FAILED, {}: {e}", rule.name()))
        .collect();
    for e in &failed {
        println!("{e}");
    }
    let held: Vec<(u64, usize)> = outcomes
        .iter()
        .filter_map(|(seed, v)| v.as_ref().ok().filter(|h| **h > 0).map(|h| (*seed, *h)))
        .collect();
    let seeds_failed: Vec<u64> = outcomes.iter().filter(|(_, v)| v.is_err()).map(|(s, _)| *s).collect();
    println!(
        "{}: seeds {}..{}, {} runs, {} failed {seeds_failed:?}; sets held at the end: {} in all, in {} runs, at most {}; {:.1} s on {threads} threads",
        rule.name(),
        seeds.start,
        seeds.end,
        outcomes.len(),
        failed.len(),
        held.iter().map(|(_, h)| h).sum::<usize>(),
        held.len(),
        held.iter().map(|(_, h)| *h).max().unwrap_or(0),
        start.elapsed().as_secs_f64()
    );
    failed
}

/// The rules `W1_RULES` names, as `N,H,P`, else every one.
fn rules() -> Vec<Rule> {
    match std::env::var("W1_RULES") {
        Ok(names) => names.split(',').map(Rule::named).collect(),
        Err(_) => Rule::ALL.to_vec(),
    }
}

/// Each rule on the gate's seeds. It fails where any rule run fails, today's
/// included, so it waits out of the gate for slice B's choice, whose seeds
/// then replace today's there. Run `cargo test -p svrngs-db --test w1 --
/// --ignored each_rule_on_the_gate --nocapture`, `W1_RULES` to name rules.
#[test]
#[ignore = "today's rule and P fail some, APP_STATE §8 q9; the rule slice B keeps takes today's place in the gate"]
fn each_rule_on_the_gate_s_seeds() {
    let failed: Vec<String> = rules().into_iter().flat_map(|r| runs(r, GATE)).collect();
    assert!(failed.is_empty(), "{} runs failed", failed.len());
}

/// The soak under each rule: `W1_SEEDS` as `<first>..<end>`, else the
/// 100,000 runs APP_STATE §11 names, model copies only. Run `cargo test
/// --release -p svrngs-db --test w1 -- --ignored each_rule_s_soak
/// --nocapture`, `W1_RULES` to name rules.
#[test]
#[ignore = "the soak: 100,000 runs under each rule, not the gate's"]
fn each_rule_s_soak() {
    let seeds = std::env::var("W1_SEEDS").unwrap_or_else(|_| "0..100000".into());
    let (first, end) = seeds.split_once("..").expect("W1_SEEDS is <first>..<end>");
    let (first, end): (u64, u64) = (first.parse().expect("a first seed"), end.parse().expect("an end seed"));
    let failed: Vec<String> = rules().into_iter().flat_map(|r| runs(r, first..end)).collect();
    assert!(failed.is_empty(), "{} runs failed", failed.len());
}
