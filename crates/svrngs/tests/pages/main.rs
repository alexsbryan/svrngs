//! The pages' own tests (ralph/next/page/CHARTER.md, "What is settled" 6):
//! what the pages do that the page queue made them do, each read again,
//! answered beside its button, or refused by name, in scripts/check.sh's
//! "pages" step. One module a feature. Each was watched red on the pages
//! before the queue, then green.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
#[path = "../../../svrngs-node/tests/browser/drive.rs"]
#[allow(dead_code)]
mod drive;

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
