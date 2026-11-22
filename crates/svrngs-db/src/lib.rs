// SPDX-License-Identifier: AGPL-3.0-or-later
//! An app's database in a group: SQLite tables beside the group's record, a
//! copy on every member's computer, merged by cr-sqlite (svrn-docs
//! `rings/current/APP_STATE.md` §4).
//!
//! None of it is built yet. The crate holds its tests first, in `tests/w1/`:
//! the reference model every copy of a database is checked against, and the
//! seeded simulator of deliveries that drives the copies. They come before the
//! code they judge, because "copies agree, on the right state" (APP_STATE §11,
//! W1) is the riskiest promise the database makes, and tests count against no
//! budget (`scripts/lean_check.py`, `code_lines`). The database lands here
//! next, and the simulator's copies then include it.

#![deny(unsafe_code)]
