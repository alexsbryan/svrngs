//! The v2-only tests (ralph/next/page/CHARTER.md, "What is settled" 6):
//! what v2 does that v1 does not, so they drive v2 whatever `SVRNGS_UI`
//! says, in scripts/check.sh's "page v2" step, with the command's binary
//! built to serve it (`--features svrngs/test-support`). One module a
//! feature row, which that row alone fills. Red first: a row flips [`UI`]
//! to `Ui::V1`, watches its test fail, names that run in its commit, and
//! restores it. They go with v1, at page-switchover.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;

/// The page these tests drive: v2, never the switch's ([`harness::ui`]):
/// `harness::site_of(UI)`, a node's `.ui(UI)`, and `harness::home_is`.
#[allow(dead_code)]
const UI: harness::Ui = harness::Ui::V2;

// page-open: the phone page opens into its group; the home reads its groups.
mod open;
// page-shelf: what the group's shelf brings, read again after.
mod shelf;
// page-index: the group's index, what runs and what is lent.
mod index;
// page-members: the door, visits, leaving and the screen.
mod members;
// page-join: joining, the group's links, making one, opening one.
mod join;
// page-acts: the record's acts, its changes and uses, and the ask.
mod acts;
// page-home-rooms: the home's Wi-Fi and Play.
mod rooms;
