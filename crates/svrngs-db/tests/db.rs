//! An app's database on one computer, asked as a page asks it: the spike's
//! tests of its local half (svrn-docs `rings/current/app-state-spike/`,
//! `tests/app_state.rs`), at the database itself, with no node or route.
//! Each names the plant it was watched failing on.

#![allow(clippy::disallowed_methods, reason = "a query's time, as the page waits it")]

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use svrngs_db::{Db, Refused, States};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, col TEXT);\n";

/// The board's database, `cards(id, title, col)`, in a directory of its own.
fn board() -> (tempfile::TempDir, Arc<Db>) {
    let dir = tempfile::tempdir().unwrap();
    let db = States::new(dir.path()).open("g", "board", SCHEMA);
    // Plant: a build without the shim, `LIBSQLITE3_FLAGS= cargo test -p
    // svrngs-db --test db`: every test here fails at this line, refused
    // `Unbuilt` in its words, and none is served a database.
    let db = db.unwrap_or_else(|r| panic!("refused {r:?}: {}", r.words()));
    (dir, db)
}

/// **A database opens only where cr-sqlite is loaded**: its tables are
/// cr-sqlite's, each write kept in their clocks.
#[test]
fn a_database_opens_with_its_tables_made_crdts() {
    let (dir, db) = board();
    db.run(true, "INSERT INTO cards (id, title) VALUES ('c1', 'milk')", &[])
        .unwrap();
    let plain = rusqlite::Connection::open(dir.path().join("state/g/board.db")).unwrap();
    let mut kept = plain
        .prepare("SELECT col_name FROM cards__crsql_clock ORDER BY col_name")
        .unwrap();
    let kept: Vec<String> = kept.query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
    // Every column of the insert, the one it left NULL too (cr-sqlite
    // `core/src/local_writes/mod.rs`), as no plain SQLite keeps.
    assert_eq!(kept, ["col", "title"], "the insert, in cr-sqlite's clock");
}

/// **Every query ends** (APP_STATE §7, 5): a recursive query that never
/// stops is stopped at the limit, and the page told so. Plant: no progress
/// handler (the one `Db::open` installs taken out): the query answers no
/// more within 5 s.
#[test]
fn a_query_that_never_ends_is_stopped() {
    let (_dir, db) = board();
    let forever = "WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x + 1 FROM c) SELECT count(*) FROM c";
    // The plant's run never answers, so the database is asked on a thread
    // of its own, and waited on.
    let (tx, rx) = std::sync::mpsc::channel();
    let began = Instant::now();
    std::thread::spawn(move || tx.send(db.run(false, forever, &[])).ok());
    let answered = rx.recv_timeout(Duration::from_secs(5));
    let took = began.elapsed();
    assert!(matches!(answered, Ok(Err(Refused::TooLong))), "{answered:?}");
    assert!(took < Duration::from_secs(2), "stopped near 250 ms, not {took:?}");
    println!("MEASURE never-ending query: stopped after {} ms", took.as_millis());
}

/// **An app reaches only its own tables** (APP_STATE §7, 4): the
/// authorizer refuses ATTACH, PRAGMA, CREATE TABLE, load_extension, and
/// cr-sqlite's and svrngs's own tables. Plant: an authorizer that allows
/// `ATTACH`: it runs, and its file is made.
#[test]
fn the_authorizer_refuses_all_but_the_app_s_own_tables() {
    let (dir, db) = board();
    let attached = dir.path().join("attached.db");
    let refused = [
        format!("ATTACH DATABASE '{}' AS other", attached.display()),
        "PRAGMA journal_mode = DELETE".into(),
        "PRAGMA table_info(cards)".into(),
        "CREATE TABLE more (id INTEGER PRIMARY KEY)".into(),
        "SELECT load_extension('/tmp/nothing')".into(),
        "SELECT * FROM svrngs_sets".into(),
        "SELECT * FROM crsql_changes".into(),
        "SELECT * FROM cards__crsql_clock".into(),
        "INSERT INTO crsql_changes (\"table\", pk, cid, val, col_version, db_version, site_id, cl, seq, ts) VALUES ('cards', x'01', 'title', 'x', 1, 1, x'00', 1, 0, '0')".into(),
        "DROP TABLE cards".into(),
        "SELECT crsql_site_id()".into(),
        "BEGIN".into(),
    ];
    let mut said = Vec::new();
    for q in &refused {
        let answer = db.run(true, q, &[]);
        let status = answer.as_ref().map_or_else(Refused::status, |_| 200);
        said.push(format!("{status} {q} -> {answer:?}"));
        // cr-sqlite's own statement is refused under the authorizer before
        // the page's is, and said as SQLite's error.
        assert!(
            status == 403 || q.contains("crsql_changes") && status == 400,
            "{q}: {answer:?}"
        );
    }
    let own = db.run(true, "INSERT INTO cards (id, title) VALUES ('x', 'y')", &[]);
    assert_eq!(own.unwrap()["changed"], 1, "its own table is written");
    assert!(!Path::new(&attached).exists(), "the ATTACH made no file");
    println!("MEASURE authorizer: {said:#?}");
}
