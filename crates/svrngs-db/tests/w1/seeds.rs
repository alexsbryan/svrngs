//! W1's seeds: fixed ones the gate runs, each run met in full and judged
//! (`sim::one`), with the plants they catch; and the soak, which the gate
//! doesn't run.

use crate::change::Site;
use crate::replica::{ModelCopy, Plant};
use crate::sim::{self, TABLES};

/// The gate's seeds, which run in under ten seconds: 1,000 runs took 2.9 s
/// in a debug build on an M-series Mac.
const GATE: std::ops::Range<u64> = 0..1000;

fn model(site: Site) -> ModelCopy {
    ModelCopy::new(site, TABLES)
}

/// The first of the gate's seeds a planted copy fails, and why.
fn caught(plant: Plant) -> String {
    let make = |site| ModelCopy::planted(site, TABLES, plant);
    GATE.into_iter()
        .find_map(|seed| sim::one(seed, &make).err())
        .unwrap_or_else(|| panic!("no seed catches {plant:?}"))
}

#[test]
fn every_copy_ends_on_the_reference_on_the_gate_s_seeds() {
    for seed in GATE {
        if let Err(e) = sim::one(seed, &model) {
            panic!("{e}");
        }
    }
}

#[test]
fn the_gate_s_seeds_catch_a_prune_that_drops_a_set_still_standing() {
    let e = caught(Plant::EagerPrune);
    assert!(e.contains("sets it doesn't keep"), "{e}");
}

#[test]
fn the_gate_s_seeds_catch_clocks_kept_on_resurrect() {
    let e = caught(Plant::KeepClocksOnResurrect);
    assert!(e.contains("is not the reference's"), "{e}");
}

/// The soak: `W1_SEEDS` as `<first>..<end>`, else the 100,000 runs APP_STATE
/// §11 names for the Fedora box, recorded in `demo/runs/`. Run
/// `cargo test --release -p svrngs-db --test w1 -- --ignored soak --nocapture`.
#[test]
#[ignore = "the soak: 100,000 runs, not the gate's"]
fn soak() {
    let seeds = std::env::var("W1_SEEDS").unwrap_or_else(|_| "0..100000".into());
    let (first, end) = seeds.split_once("..").expect("W1_SEEDS is <first>..<end>");
    let (first, end): (u64, u64) = (first.parse().expect("a first seed"), end.parse().expect("an end seed"));
    let failed: Vec<String> = (first..end).filter_map(|seed| sim::one(seed, &model).err()).collect();
    for e in &failed {
        println!("FAILED {e}");
    }
    println!(
        "soak: seeds {first}..{end}, {} runs, {} failed",
        end - first,
        failed.len()
    );
    assert!(failed.is_empty());
}
