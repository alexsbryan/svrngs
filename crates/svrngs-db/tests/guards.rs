//! What a database leaves behind when a step fails: a call that fails
//! leaves no transaction open, and a set is kept and counted together or
//! not at all.

use std::path::Path;
use std::sync::Arc;

use rusqlite::Connection;
use serde_json::json;
use svrngs_db::{Call, Db, SigningKey, States, Took};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);";
const BUDGET: usize = 4 << 20;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn board(dir: &Path, n: u8) -> Arc<Db> {
    let states = States::new(&dir.join(format!("computer-{n}")), key(n));
    states.open("g", "board", Some(SCHEMA)).unwrap()
}

/// A second connection to computer `n`'s board, which waits for no lock.
fn side(dir: &Path, n: u8) -> Connection {
    let side = Connection::open(dir.join(format!("computer-{n}/state/g/board.db"))).unwrap();
    side.busy_timeout(std::time::Duration::ZERO).unwrap();
    side
}

/// **A call that fails partway leaves no transaction open**: one of the
/// call's own reads fails after its transaction began (a column of
/// svrngs's table renamed from a second connection, the plant), and the
/// call is refused; the database's write lock is free after it, so another
/// connection can begin a write at once.
#[test]
fn a_call_that_fails_leaves_no_transaction_open() {
    let dir = tempfile::tempdir().unwrap();
    let b = board(dir.path(), 2);
    let side = side(dir.path(), 2);
    side.execute_batch("ALTER TABLE svrngs_sets RENAME COLUMN kept TO aside")
        .unwrap();
    assert!(
        b.ingest(&Call::default(), &|_| true, None).is_err(),
        "the plant fails the call"
    );
    let free = side.execute_batch("BEGIN IMMEDIATE; ROLLBACK");
    assert!(free.is_ok(), "the call's transaction is still open: {free:?}");
    side.execute_batch("ALTER TABLE svrngs_sets RENAME COLUMN aside TO kept")
        .unwrap();
    svrngs_db::close(side).unwrap();
}

/// **A set that waits is kept and counted together, or neither**: its count
/// fails (a trigger from a second connection refusing every insert into
/// svrngs's table of sets counted above a mark, the plant), so the set is a
/// fault, and nothing of it is kept; once the trigger is gone, the next call
/// keeps it as waiting, counted.
#[test]
fn a_waiting_set_is_kept_and_counted_together_or_not_at_all() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (board(dir.path(), 1), board(dir.path(), 2));
    a.write(&json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]))
        .unwrap();
    let sent = a.missing(&b.ask().unwrap(), BUDGET).unwrap();
    let side = side(dir.path(), 2);
    let plant = "CREATE TRIGGER aside BEFORE INSERT ON svrngs_above BEGIN SELECT RAISE(ABORT, 'planted'); END";
    side.execute_batch(plant).unwrap();
    let got = b.ingest(&sent, &|_| false, None).unwrap();
    assert_eq!(got[0].1, Took::Fault);
    side.execute_batch("DROP TRIGGER aside").unwrap();
    let kept: i64 = side
        .query_row("SELECT count(*) FROM svrngs_sets", [], |r| r.get(0))
        .unwrap();
    assert_eq!(kept, 0, "a set kept, uncounted");
    assert_eq!(b.ingest(&sent, &|_| false, None).unwrap()[0].1, Took::Waits);
    svrngs_db::close(side).unwrap();
}
