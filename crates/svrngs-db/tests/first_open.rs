//! cr-sqlite on every connection from the process's first open on, however
//! many threads open at once (the W1 live tests' failure at
//! `tests/w1/live.rs:55`, after a rebuild: svrn-docs
//! `app-state-diligence/e-build-2026-11-22/`). SQLite ran the shim named by
//! `SQLITE_EXTRA_INIT` after it marked itself started and left its mutexes
//! (bundled SQLite 3.54.1, `sqlite3Initialize`), so a second thread's first
//! open could find SQLite started and cr-sqlite not yet registered, and open
//! a connection without it. `SQLITE_EXTRA_INIT_MUTEXED`, inside the init
//! mutex, deadlocks, as the shim's `sqlite3_auto_extension` starts SQLite
//! again. Named by `SQLITE_EXTRA_AUTOEXT`, the shim is a built-in extension
//! SQLite runs as it opens each connection: no step at start to race.
//!
//! Each run is a process of its own, where SQLite has not started: this test
//! runs its own binary again as children, each opening from 32 threads at a
//! barrier, and every connection of every child must have cr-sqlite's
//! functions. Plant, watched failing: the shim registered at start, by
//! `SQLITE_EXTRA_INIT`, as before (3 of 100 processes had connections
//! without it).

// Linked for its object, which holds cr-sqlite and the shim SQLite calls.
extern crate svrngs_db;

use std::sync::{Arc, Barrier};

use rusqlite::Connection;

const CHILD: &str = "SVRNGS_DB_FIRST_OPEN_CHILD";

#[test]
fn every_connection_has_cr_sqlite_from_the_first_open_on() {
    if std::env::var_os(CHILD).is_some() {
        let start = Arc::new(Barrier::new(32));
        let threads = (0..32).map(|_| {
            let start = start.clone();
            std::thread::spawn(move || {
                start.wait();
                let conn = Connection::open_in_memory().unwrap();
                let site = conn.query_row("SELECT crsql_site_id()", [], |r| r.get::<_, Vec<u8>>(0));
                site.map(|s| s.len()).map_err(|e| e.to_string())
            })
        });
        let opened: Vec<_> = threads
            .collect::<Vec<_>>()
            .into_iter()
            .map(|t| t.join().unwrap())
            .collect();
        let without = opened.iter().filter(|o| o.is_err()).count();
        assert_eq!(without, 0, "connections without cr-sqlite: {opened:?}");
        return;
    }
    let me = std::env::current_exe().unwrap();
    let mut failed = Vec::new();
    for run in 0..100 {
        let child = std::process::Command::new(&me)
            .args([
                "--exact",
                "every_connection_has_cr_sqlite_from_the_first_open_on",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        if !child.status.success() {
            failed.push((
                run,
                String::from_utf8_lossy(&child.stderr)
                    .lines()
                    .find(|l| l.contains("without"))
                    .map(str::to_owned),
            ));
        }
    }
    assert!(
        failed.is_empty(),
        "{} of 100 processes opened a connection without cr-sqlite: {:?}",
        failed.len(),
        failed.first()
    );
}
