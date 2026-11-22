//! W1, "copies agree, on the right state" (svrn-docs
//! `rings/current/APP_STATE.md` §11): the reference model every copy of an
//! app's database is checked against, and what checks it, written before the
//! database they judge (MAGNETS_BUILD.md §3, "W, beside A").
//!
//! - `change`: a change row as cr-sqlite's `crsql_changes` carries it, and a
//!   change set.
//! - `reference`: the reference model, the join of every admissible set.
//! - `replica`: the trait a copy is reached through, and the model copy,
//!   cr-sqlite's merge as read from Fly's fork.
//! - `sim`: the seeded simulator of deliveries, and its judge of a full meet.
//! - `live`: the live copy, cr-sqlite itself behind the same trait, and the
//!   pair, a model copy and a live copy in lockstep.
//! - `rule`: what a copy does with a set above a missing re-add (APP_STATE
//!   §8, question 9): N, the rule svrngs keeps and the fork's engine has,
//!   and today's rule and the two other candidates, kept as variants.
//! - `cases`: the cases driven by hand, on any copy under any rule: the
//!   minimal case of the split, two removals, and column defaults.
//! - `sync`: the sync's bookkeeping above the engine, each copy's marks and
//!   what it takes by them: APP_STATE §4's, and the spike's rules each in
//!   its place, kept as variants of the simulator's own.
//! - `db`: svrngs-db's own database behind the same trait, its own sync held
//!   at every call to the bookkeeping under §4's marks, on the gate's seeds,
//!   W's sync cases and the review's case between computers.
//!
//! The tests, each beside the plant it was watched failing on:
//! - `laws`: the join's laws over seeded histories;
//! - `s1`: S1's recorded outcomes, rebuilt from the model;
//! - `review`: the review's case;
//! - `seeds`: the gate's fixed seeds, and the soak the gate doesn't run;
//! - `live`: the live engine on the gate's seeds, the pair on them, the
//!   copies agreeing on the fixed cases, and the live soak, which the gate
//!   doesn't run;
//! - `rules`: question 9's rules on the fixed cases, and on the gate's seeds
//!   and the soak's, which the gate doesn't run;
//! - `syncs`: invariants 11 and 12, and a removed device's set relayed, each
//!   on every copy beside the spike's rule that fails it, and each variant of
//!   the sync on the gate's seeds and the soak's, which the gate doesn't run.

mod cases;
mod change;
mod db;
mod laws;
mod live;
mod reference;
mod replica;
mod review;
mod rule;
mod rules;
mod s1;
mod seeds;
mod sim;
mod sync;
mod syncs;
