// SPDX-License-Identifier: AGPL-3.0-or-later
//! An app's database in a group: SQLite tables beside the group's record, a
//! copy on every member's computer, merged by cr-sqlite (svrn-docs
//! `rings/current/APP_STATE.md` §4).
//!
//! cr-sqlite is built from its source, the submodule `vendor/cr-sqlite`, by
//! `build.rs`, into one object linked here whose one global is a shim in C
//! (`shim.c`). SQLite calls the shim as it starts, so cr-sqlite is loaded on
//! every connection and no Rust in svrngs calls it: nothing here is `unsafe`.
//! `tests/crsqlite.rs` holds that, with its plant.
//!
//! The database's code lands here next. Its tests came first, in
//! `tests/w1/`: the reference model every copy of a database is checked
//! against, and the seeded simulator of deliveries that drives the copies.
//! They come before the code they judge, because "copies agree, on the right
//! state" (APP_STATE §11, W1) is the riskiest promise the database makes,
//! and tests count against no budget (`scripts/lean_check.py`, `code_lines`).

#![deny(unsafe_code)]
