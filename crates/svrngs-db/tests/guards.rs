//! What a database leaves behind when a step fails, and where it opens one:
//! a call that fails leaves no transaction open, a set is kept and counted
//! together or not at all, and a database is opened only under its group's
//! directory, by a plain name.

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

/// Every key these tests sign with, bound, in the room a request has
/// beside its sets (svrngs-node `peer_wire.rs`, `ASK_ROOM_BYTES`).
fn bound() -> svrngs_db::Binds {
    let keys = (0..=255).map(|n| svrngs_core::actor_of(&SigningKey::from_bytes(&[n; 32])));
    svrngs_db::Binds::new(keys.map(|k| (k.clone(), k, false)), 4 << 20)
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
        b.ingest(&Call::default(), &bound(), None).is_err(),
        "the plant fails the call"
    );
    let free = side.execute_batch("BEGIN IMMEDIATE; ROLLBACK");
    assert!(free.is_ok(), "the call's transaction is still open: {free:?}");
    side.execute_batch("ALTER TABLE svrngs_sets RENAME COLUMN aside TO kept")
        .unwrap();
    svrngs_db::close(side).unwrap();
}

/// **A set that waits is kept and counted together, or neither**: one
/// written under a schema this copy doesn't hold, its count failing (a
/// trigger from a second connection refusing every insert into svrngs's
/// table of sets counted above a mark, the plant), is a fault, and nothing
/// of it is kept; once the trigger is gone, the next call keeps it as
/// waiting, counted.
#[test]
fn a_waiting_set_is_kept_and_counted_together_or_not_at_all() {
    const WIDER: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, note TEXT);";
    let dir = tempfile::tempdir().unwrap();
    let a = States::new(&dir.path().join("computer-1"), key(1));
    let (a, b) = (a.open("g", "board", Some(WIDER)).unwrap(), board(dir.path(), 2));
    a.write(
        &json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]),
        &bound(),
    )
    .unwrap();
    let sent = a.missing(&b.ask().unwrap(), BUDGET).unwrap();
    let side = side(dir.path(), 2);
    let plant = "CREATE TRIGGER aside BEFORE INSERT ON svrngs_above BEGIN SELECT RAISE(ABORT, 'planted'); END";
    side.execute_batch(plant).unwrap();
    let got = b.ingest(&sent, &bound(), None).unwrap();
    assert_eq!(got[0].1, Took::Fault);
    side.execute_batch("DROP TRIGGER aside").unwrap();
    let kept: i64 = side
        .query_row("SELECT count(*) FROM svrngs_sets", [], |r| r.get(0))
        .unwrap();
    assert_eq!(kept, 0, "a set kept, uncounted");
    assert_eq!(b.ingest(&sent, &bound(), None).unwrap()[0].1, Took::Waits);
    svrngs_db::close(side).unwrap();
}

/// **A database is opened only by a plain name, under its group's
/// directory**: a name that is a path, absolute or climbing out, or holds a
/// separator, opens nothing, and a SQLite file elsewhere is left as it was,
/// byte for byte, with no WAL beside it: its tables, and its journal mode.
/// The file is made by SQLite as this crate links it, so the test needs no
/// `sqlite3` program, which a Linux host may lack (svrn-docs
/// `app-state-diligence/e-opening-2026-11-22/`). Plant: a name with a `/`
/// taken (the other file gains svrngs's tables and a WAL).
#[test]
fn a_database_is_opened_only_by_a_plain_name() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = dir.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let other = elsewhere.join("other.db");
    let made = Connection::open(&other).unwrap();
    made.execute_batch("CREATE TABLE notes (x);").unwrap();
    svrngs_db::close(made).unwrap();
    let before = std::fs::read(&other).unwrap();
    let states = States::new(&dir.path().join("computer-1"), key(1));
    let absolute = elsewhere.join("other");
    let names = [
        ("g", absolute.to_str().unwrap()),
        ("g", "../../elsewhere/other"),
        ("g", "sub/other"),
        ("g", "sub\\other"),
        ("g", ".."),
        ("g", ""),
        ("../elsewhere", "other"),
    ];
    let mut opened = Vec::new();
    for (group, app) in names {
        for schema in [None, Some(SCHEMA)] {
            if states.open(group, app, schema).is_ok() {
                opened.push(format!("{group}/{app}, made: {}", schema.is_some()));
            }
        }
    }
    assert!(std::fs::read(&other).unwrap() == before, "the other file, untouched");
    assert!(!elsewhere.join("other.db-wal").exists(), "and no WAL beside it");
    assert!(opened.is_empty(), "opened: {opened:?}");
    assert!(states.open("g", "board", Some(SCHEMA)).is_ok(), "a plain name opens");
}

/// **A version pruned is never sealed again** (APP_STATE §4, "Sealed and
/// signed": the seal fails loudly on a version this device's sets hold, the
/// reuse Fly's fork PR #29 names, engine-and-sync C6): Ana writes a card and
/// writes it again, so her first set, outranked, is pruned. cr-sqlite's
/// count of her versions is put back to none from a second connection, as
/// that reuse would, so her next write is given version 1 again: it is
/// refused, in words, and nothing of it is kept. Plant, watched failing:
/// the guard reading the kept sets alone, not the site's own mark (the write
/// sealed at 1 a second time).
#[test]
fn a_version_pruned_is_never_sealed_again() {
    let dir = tempfile::tempdir().unwrap();
    let a = board(dir.path(), 1);
    let card = |id: &str, title: &str| json!([{ "insert": "cards", "row": { "id": id, "title": title } }]);
    a.write(&card("c1", "milk"), &bound()).unwrap();
    a.write(&card("c1", "jam"), &bound()).unwrap();
    let side = side(dir.path(), 1);
    side.execute("UPDATE crsql_db_versions SET db_version = 0", []).unwrap();
    let refused = a.write(&card("c2", "eggs"), &bound()).err().map(|r| r.to_string());
    assert_eq!(
        refused.as_deref(),
        Some("This computer couldn't: this database gave a write the version 1 again."),
    );
    let rows = a.read("SELECT id, title FROM cards ORDER BY id", &[]).unwrap();
    assert_eq!(
        rows["rows"],
        json!([{ "id": "c1", "title": "jam" }]),
        "nothing of it kept"
    );
    svrngs_db::close(side).unwrap();
}
