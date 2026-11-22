//! An app's database on one computer, asked as a page asks it, at the
//! database itself, with no node or route: the spike's tests of its local
//! half (svrn-docs `rings/current/app-state-spike/`, `tests/app_state.rs`),
//! and the write form's (APP_STATE §4, "The write form"). Each names the
//! plant it was watched failing on.

#![allow(clippy::disallowed_methods, reason = "a query's time, as the page waits it")]

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use svrngs_db::{Db, Refused, States};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, col TEXT DEFAULT 'todo');\n\
    CREATE TABLE seats (row INTEGER NOT NULL, seat INTEGER NOT NULL, who TEXT, PRIMARY KEY (seat, row));\n";

/// The board's database, `cards(id, title, col)` and `seats`, a key of two
/// columns, in a directory of its own.
fn board() -> (tempfile::TempDir, Arc<Db>) {
    let dir = tempfile::tempdir().unwrap();
    let db = States::new(dir.path()).open("g", "board", SCHEMA);
    // Plant: a build without the shim, `LIBSQLITE3_FLAGS= cargo test -p
    // svrngs-db --test db`: every test here fails at this line, refused
    // `Unbuilt` in its words, and none is served a database.
    let db = db.unwrap_or_else(|r| panic!("refused {r:?}: {r}"));
    (dir, db)
}

/// `ops` written: the rows changed, or why not.
fn write(db: &Db, ops: Value) -> Result<Value, Refused> {
    db.write(&ops)
}

/// Every row of `cards`, by id.
fn cards(db: &Db) -> Value {
    db.read("SELECT id, title, col FROM cards ORDER BY id", &[]).unwrap()["rows"].clone()
}

/// **A database opens only where cr-sqlite is loaded**: its tables are
/// cr-sqlite's, each write kept in their clocks.
#[test]
fn a_database_opens_with_its_tables_made_crdts() {
    let (dir, db) = board();
    write(
        &db,
        json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]),
    )
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
    std::thread::spawn(move || tx.send(db.read(forever, &[])).ok());
    let answered = rx.recv_timeout(Duration::from_secs(5));
    let took = began.elapsed();
    assert!(matches!(answered, Ok(Err(Refused::TooLong))), "{answered:?}");
    assert!(took < Duration::from_secs(2), "stopped near 250 ms, not {took:?}");
    println!("MEASURE never-ending query: stopped after {} ms", took.as_millis());
}

/// **An app reaches only its own tables** (APP_STATE §7, 4): the
/// authorizer refuses ATTACH, PRAGMA, CREATE TABLE, load_extension, and
/// cr-sqlite's own tables. Plant: an authorizer that allows
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
        "SELECT * FROM crsql_master".into(),
        "SELECT * FROM crsql_changes".into(),
        "SELECT * FROM cards__crsql_clock".into(),
        "INSERT INTO crsql_changes (\"table\", pk, cid, val, col_version, db_version, site_id, cl, seq, ts) VALUES ('cards', x'01', 'title', 'x', 1, 1, x'00', 1, 0, '0')".into(),
        "DROP TABLE cards".into(),
        "SELECT crsql_site_id()".into(),
        "BEGIN".into(),
    ];
    let mut said = Vec::new();
    for q in &refused {
        let answer = db.read(q, &[]);
        let status = answer.as_ref().map_or_else(|r| r.said().0, |_| 200);
        said.push(format!(
            "{status} {q} -> {}",
            answer.map_or_else(|r| r.to_string(), |a| a.to_string())
        ));
        // cr-sqlite's own statement is refused under the authorizer before
        // the page's is, and said as SQLite's error.
        assert!(
            status == 403 || q.contains("crsql_changes") && status == 400,
            "{}",
            said.last().unwrap()
        );
    }
    let own = db.read("SELECT count(*) AS n FROM cards", &[]);
    assert_eq!(own.unwrap()["rows"], json!([{ "n": 0 }]), "its own table is read");
    assert!(!Path::new(&attached).exists(), "the ATTACH made no file");
    println!("MEASURE authorizer: {said:#?}");
}

/// **A write sent as SQL is refused, its words naming the write form**
/// (APP_STATE, "The pitch", 3), and nothing of it kept: SQLite says what a
/// statement does, so a write behind a WITH or before a RETURNING is
/// refused as one that begins INSERT is. Plant: the read's check of
/// `readonly` taken out: the first INSERT runs, and the row is kept.
#[test]
fn a_write_sent_as_sql_is_refused_naming_the_form() {
    let (_dir, db) = board();
    for sql in [
        "INSERT INTO cards (id, title) VALUES ('c1', 'milk')",
        "UPDATE cards SET title = 'eggs'",
        "INSERT INTO cards (id) VALUES ('c2') RETURNING id",
        "WITH new (id) AS (VALUES ('c3')) INSERT INTO cards (id) SELECT id FROM new",
        "DELETE FROM cards",
    ] {
        let refused = db.read(sql, &[]).expect_err(sql);
        assert_eq!(refused, Refused::NotARead, "{sql}");
        assert_eq!(refused.said(), (400, "not-a-read"));
        assert!(refused.to_string().contains("{write: [op, ...]}"), "{refused}");
    }
    assert_eq!(cards(&db), json!([]), "nothing of them kept");
}

/// **A write names its rows by key** (APP_STATE §4, "The write form"): an
/// insert makes its row, NULL in each column it doesn't name, a DEFAULT
/// too, and on a key that has a row sets only the columns it names; an
/// update sets one or more columns of a row named by every key column; a
/// delete takes a row so named; a BLOB goes as base64 both ways; and a
/// list is one transaction, the rows changed counted. Plant: an insert of
/// the named columns alone, as plain SQL writes one: `col` takes its
/// DEFAULT, 'todo'.
#[test]
fn a_write_names_its_rows_by_key() {
    let (_dir, db) = board();
    let made = write(
        &db,
        json!([
            { "insert": "cards", "row": { "id": "c1", "title": "milk" } },
            { "insert": "cards", "row": { "id": "c2", "title": "eggs", "col": "doing" } },
            { "insert": "seats", "row": { "row": 1, "seat": 2, "who": { "base64": "AP8=" } } },
        ]),
    );
    assert_eq!(made, Ok(json!({ "count": 3 })));
    assert_eq!(
        cards(&db),
        json!([{ "id": "c1", "title": "milk", "col": null }, { "id": "c2", "title": "eggs", "col": "doing" }]),
        "each column the insert names, and NULL in the rest"
    );
    let seat = db.read(
        "SELECT who FROM seats WHERE row = ? AND seat = ?",
        &[json!(1), json!(2)],
    );
    assert_eq!(
        seat.unwrap()["rows"],
        json!([{ "who": { "base64": "AP8=" } }]),
        "the BLOB, as it went"
    );
    let changed = write(
        &db,
        json!([
            { "insert": "cards", "row": { "id": "c2", "title": "bread" } },
            { "update": "cards", "key": { "id": "c1" }, "set": { "col": "done" } },
            { "delete": "seats", "key": { "seat": 2, "row": 1 } },
            { "update": "cards", "key": { "id": "c9" }, "set": { "col": "done" } },
        ]),
    );
    assert_eq!(
        changed,
        Ok(json!({ "count": 3 })),
        "the row no key names changes nothing"
    );
    assert_eq!(
        cards(&db),
        json!([{ "id": "c1", "title": "milk", "col": "done" }, { "id": "c2", "title": "bread", "col": "doing" }]),
        "an insert on a key with a row sets only the columns it names"
    );
    let seats = db.read("SELECT count(*) AS n FROM seats", &[]).unwrap();
    assert_eq!(
        seats["rows"],
        json!([{ "n": 0 }]),
        "the seat, deleted by its two key columns"
    );
}

/// **What the form doesn't take is refused in words that say what to
/// change**, and nothing of the list kept: a name the table lacks never
/// reaches SQLite (app-fit, N2's plant). Plants: the columns named taken
/// as the page spells them, unchecked: the unknown column is SQLite's
/// error, `sql`, not the form's; a key checked by its count alone: the key
/// naming `title` updates a row by it.
#[test]
fn what_the_form_doesn_t_take_is_refused_saying_what_to_change() {
    let (_dir, db) = board();
    let good = json!({ "insert": "cards", "row": { "id": "c1", "title": "milk" } });
    for (op, said) in [
        (
            json!({ "insert": "card", "row": { "id": "c1" } }),
            "names the table \"card\", but there is no such table; the tables are cards, seats.",
        ),
        (
            json!({ "insert": "cards", "row": { "id": "c1", "colour": "red" } }),
            "names \"colour\", but cards has no such column; its columns are id, title, col.",
        ),
        (
            json!({ "insert": "cards", "row": { "title": "milk" } }),
            "row leaves out id: an insert names every key column of cards, id.",
        ),
        (
            json!({ "insert": "seats", "row": { "row": 1, "who": "ana" } }),
            "row leaves out seat: an insert names every key column of seats, seat, row.",
        ),
        (
            json!({ "update": "cards", "key": { "title": "milk" }, "set": { "col": "done" } }),
            "key names title: a key of cards names every key column, id, and no other.",
        ),
        (
            json!({ "delete": "seats", "key": { "row": 1 } }),
            "key names row: a key of seats names every key column, seat, row, and no other.",
        ),
        (
            json!({ "update": "cards", "key": { "id": "c1" }, "set": { "id": "c2" } }),
            "set changes id, a key column of cards: to give a row another key, delete it and insert it.",
        ),
        (
            json!({ "update": "cards", "key": { "id": "c1" }, "set": {} }),
            "set names no column: an update sets one or more of the other columns of cards.",
        ),
        (
            json!({ "insert": "cards", "row": { "id": "c1", "title": [1] } }),
            "row's title is [1]: a value is null, a number, a string, or {base64: \"...\"} for a BLOB.",
        ),
        (
            json!({ "insert": "cards", "row": { "id": "c1" }, "set": {} }),
            "is {\"insert\":\"cards\",\"row\":{\"id\":\"c1\"},\"set\":{}}. A write goes as {write: [op, ...]}",
        ),
        (
            json!({ "upsert": "cards" }),
            "is {\"upsert\":\"cards\"}. A write goes as {write: [op, ...]}",
        ),
    ] {
        let refused = write(&db, json!([good, op])).expect_err(&op.to_string());
        assert_eq!(refused.said(), (400, "form"), "{op}: {refused}");
        assert!(refused.to_string().starts_with("write[1]"), "{refused}");
        assert!(refused.to_string().contains(said), "{op}: {refused}");
    }
    for empty in [json!([]), json!({ "insert": "cards" })] {
        let refused = write(&db, empty).unwrap_err();
        assert!(
            refused.to_string().starts_with("write takes a list of one or more ops"),
            "{refused}"
        );
    }
    assert_eq!(
        cards(&db),
        json!([]),
        "nothing of a refused list kept, its good op neither"
    );
    // SQLite's own refusal of a list ends it whole, too.
    let unnamed = json!({ "insert": "seats", "row": { "row": null, "seat": 1 } });
    let refused = write(&db, json!([good, unnamed])).unwrap_err();
    assert_eq!(refused.said(), (400, "sql"), "{refused}");
    assert_eq!(cards(&db), json!([]), "nothing of a list SQLite refused kept");
}

/// **A POST asks one read or one write**, in the two shapes the route
/// takes, and anything else is refused naming both.
#[test]
fn a_post_asks_one_read_or_one_write() {
    let read = json!({ "sql": "SELECT 1", "params": [1] });
    assert!(matches!(
        svrngs_db::asked(&read),
        Ok(svrngs_db::Asked::Read("SELECT 1", [_]))
    ));
    let alone = json!({ "sql": "SELECT 1" });
    assert!(matches!(svrngs_db::asked(&alone), Ok(svrngs_db::Asked::Read(_, []))));
    let write = json!({ "write": [] });
    assert!(matches!(svrngs_db::asked(&write), Ok(svrngs_db::Asked::Write(_))));
    for other in [
        json!({ "sql": "SELECT 1", "write": [] }),
        json!({ "sql": 1 }),
        json!({ "sql": "x", "params": 1 }),
        json!([]),
    ] {
        let refused = svrngs_db::asked(&other)
            .err()
            .unwrap_or_else(|| panic!("{other} taken"));
        assert!(
            refused
                .to_string()
                .starts_with("svrngs/sql takes {sql, params}, one read"),
            "{refused}"
        );
    }
}

/// **No app's trigger sees a merge**: `schema.sql` makes tables and indexes
/// and nothing else (APP_STATE §4, "Its schema"; `schema_rule`), so a
/// trigger is refused and the database not opened. cr-sqlite's merge
/// deletes and inserts an app's rows under its sync bit, which only its own
/// triggers heed: since the fork's patch, a column change that brings back
/// a row standing in an earlier life deletes it and makes it again
/// (`vendor/cr-sqlite/core/rs/core/src/changes_vtab_write.rs:621-625`,
/// `:744-803`). Plant: `schema_rule` allowing `CreateTrigger`: the schema is
/// applied, its tables made cr-sqlite's, and the database opened.
#[test]
fn a_schema_s_trigger_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let schema = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);\n\
        CREATE TABLE seen (n INTEGER PRIMARY KEY NOT NULL);\n\
        CREATE TRIGGER gone AFTER DELETE ON cards BEGIN INSERT INTO seen VALUES (1); END;\n";
    let opened = States::new(dir.path()).open("g", "board", schema);
    let refused = opened.err().expect("a schema with a trigger is refused");
    println!("MEASURE a schema's trigger: {refused}");
    assert!(matches!(refused, Refused::Malformed(_)), "{refused:?}");
    assert!(refused.to_string().contains("not authorized"), "{refused}");
}

/// **A database closes as it drops**, and so does one refused at open:
/// cr-sqlite's statements finalized, SQLite closes the connection and its
/// file, and the last connection to a database in WAL mode checkpoints it
/// and removes its log as it closes (libsqlite3-sys 0.39.1
/// `sqlite3/sqlite3.c:69474-69500`, `sqlite3WalClose`). A table with no key
/// is refused by `crsql_as_crr`, after `schema.sql` made it. Plant: `close`
/// without `crsql_finalize`: SQLite refuses to close each connection, and
/// each log stays, as does its database's file handle.
#[test]
fn a_database_closes_as_it_drops() {
    let (dir, db) = board();
    write(
        &db,
        json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]),
    )
    .unwrap();
    let log = dir.path().join("state/g/board.db-wal");
    assert!(log.exists(), "open, the database keeps its log");
    drop(db);
    assert!(!log.exists(), "dropped, the database closed and left no log");
    let keyless = States::new(dir.path()).open("g", "notes", "CREATE TABLE notes (body TEXT);\n");
    let refused = keyless.err().expect("a table with no key is refused");
    println!("MEASURE a table with no key: {refused}");
    assert!(matches!(refused, Refused::Malformed(_)), "{refused:?}");
    let log = dir.path().join("state/g/notes.db-wal");
    assert!(!log.exists(), "refused at open, the database closed and left no log");
}
