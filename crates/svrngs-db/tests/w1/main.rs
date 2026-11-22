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
//!
//! The tests, each beside the plant it was watched failing on:
//! - `laws`: the join's laws over seeded histories;
//! - `s1`: S1's recorded outcomes, rebuilt from the model;
//! - `review`: the review's case;
//! - `seeds`: the gate's fixed seeds, and the soak the gate doesn't run.

mod change;
mod laws;
mod reference;
mod replica;
mod review;
mod s1;
mod seeds;
mod sim;
