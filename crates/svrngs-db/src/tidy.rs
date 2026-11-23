// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from superfly/corrosion crates/corro-agent/src/agent/handlers.rs@e7295ff0
//! A database's file kept near the size of what it keeps (svrn-docs
//! `rings/current/APP_STATE.md` §4, "What is kept": "The file shrinks only
//! with `auto_vacuum=INCREMENTAL`, a `journal_size_limit`, and Corrosion's
//! WAL truncation and incremental vacuum, forked"). A new database is made
//! with `auto_vacuum=INCREMENTAL` and every connection keeps Corrosion's
//! `journal_size_limit` (`lib.rs`, `Db::open`); [`Db::tidy`] gives back the
//! pages that pruned sets free and truncates a WAL grown past its bound.
//!
//! Forked from Corrosion's `vacuum_db` (handlers.rs:541-595), `wal_checkpoint`
//! (:514-535) and `wal_checkpoint_over_threshold` (:651-676), trimmed. A
//! database here has one connection, so there is no pool and no priority
//! connection; it is asked between a round's exchanges, so no sleep between
//! vacuum steps; its cache isn't raised for the vacuum; no metrics; and a
//! database made before `auto_vacuum` was set here is left as it is, said at
//! debug, where Corrosion refuses one. A WAL busy past its timeout is said at
//! warn, as Corrosion says it. Corrosion's numbers are kept, each at its source,
//! until slice E sets svrngs's own with the size cap, from a recorded run
//! (MAGNETS_BUILD §3, "Slice E").

use rusqlite::Connection;

use crate::{Db, Refused, fault};

/// From this many free pages they are given back: Corrosion's
/// `MAX_DB_FREE_PAGES` (handlers.rs:622).
const FREE_MOST: u64 = 10_000;

/// Pages each step of the vacuum gives back: Corrosion's
/// `incremental_vacuum(1000)` (handlers.rs:575).
const STEP: &str = "PRAGMA incremental_vacuum(1000)";

/// A WAL longer than this is truncated: Corrosion's default
/// `wal_threshold_mb`, 5 GiB (corro-types `config.rs:45-48`).
const WAL_MOST: u64 = 5 << 30;

/// How long a truncation waits for the database, in ms: Corrosion's base
/// busy timeout (handlers.rs `calc_busy_timeout`, :678-700), which only a
/// WAL gigabytes past its bound raises, and that is the size cap's to stop.
const BUSY_MS: u64 = 30_000;

impl Db {
    /// This database's free pages given back, a step at a time, while
    /// [`FREE_MOST`] or more are free, where it was made to give them back;
    /// and its WAL truncated, once past [`WAL_MOST`]. The pages given back,
    /// and whether the WAL was truncated.
    pub fn tidy(&self) -> Result<(u64, bool), Refused> {
        self.tidied(&self.conn()).map_err(fault)
    }

    /// [`Self::tidy`] on this database's connection: Corrosion's `vacuum_db`,
    /// then its `wal_checkpoint_over_threshold`.
    fn tidied(&self, conn: &Connection) -> rusqlite::Result<(u64, bool)> {
        let count = |p: &str| conn.pragma_query_value(None, p, |r| r.get::<_, u64>(0));
        let (incremental, before) = (count("auto_vacuum")? == 2, count("freelist_count")?);
        let mut free = before;
        while incremental && free >= FREE_MOST {
            conn.prepare(STEP)?.query_map([], |_| Ok(()))?.count();
            free = count("freelist_count")?;
        }
        let wal = conn.path().and_then(|p| std::fs::metadata(format!("{p}-wal")).ok());
        let wal = wal.map_or(0, |m| m.len());
        tracing::debug!(target: "state", group = self.group, app = self.app, incremental, before, free, wal, "state: a database's free pages given back past its bound, where it was made to, and its WAL truncated past its own");
        Ok((before - free, wal > WAL_MOST && self.truncate(conn)?))
    }

    /// Corrosion's `wal_checkpoint`: the WAL folded in and truncated, under
    /// [`BUSY_MS`]'s timeout, then the connection's own put back; whether it
    /// was, or the database was busy past the timeout.
    fn truncate(&self, conn: &Connection) -> rusqlite::Result<bool> {
        let busy: u64 = conn.pragma_query_value(None, "busy_timeout", |r| r.get(0))?;
        conn.pragma_update(None, "busy_timeout", BUSY_MS)?;
        let held: bool = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| r.get(0))?;
        conn.pragma_update(None, "busy_timeout", busy)?;
        if held {
            tracing::warn!(target: "state", group = self.group, app = self.app, "state: a database's WAL couldn't be truncated, as it was busy past the timeout");
        }
        Ok(!held)
    }
}
