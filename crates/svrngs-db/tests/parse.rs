//! A set's changes are parsed before cr-sqlite sees them (svrn-docs
//! `rings/current/APP_STATE.md` §4, the four outcomes): each change names one
//! of the app's tables, a column of it or the row's sentinel, a key in
//! cr-sqlite's packed form with the table's key columns and no NULL, and a
//! `col_version` and causal length from 1 to 2^53. A set that doesn't parse
//! is refused for good, the same on every copy, and counted, so it is
//! fetched once; a set written under a schema this copy doesn't hold waits.
//! The write's own seal parses its set with the same parser, so the writer's
//! copy refuses what every other copy would.
//!
//! Its own binary: before the parse, the first case here stopped the whole
//! test process inside cr-sqlite, which is built to abort on a panic.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use svrngs_core::set::At;
use svrngs_db::{Ask, Call, Db, Set, Sig, SigningKey, States, Took};

const CARDS: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);";
const INTS: &str = "CREATE TABLE t (id INTEGER PRIMARY KEY NOT NULL, v TEXT);";
const PAIRS: &str = "CREATE TABLE p (a TEXT NOT NULL, b INTEGER NOT NULL, v TEXT, PRIMARY KEY (a, b));";
const KEPT: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT NOT NULL DEFAULT '');";
const BUDGET: usize = 4 << 20;
const BOUND: i64 = 1 << 53;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

/// Computer `n`'s database of `app` in `g`, made from `schema`.
fn db(dir: &Path, n: u8, (app, schema): (&str, &str)) -> Arc<Db> {
    let states = States::new(&dir.join(format!("computer-{n}")), key(n));
    states.open("g", app, Some(schema)).unwrap()
}

fn all(_: &str) -> bool {
    true
}

fn took(to: &Db, sets: &[Set]) -> Vec<Took> {
    let call = Call {
        sets: sets.to_vec(),
        ..Default::default()
    };
    to.ingest(&call, &all, None)
        .unwrap()
        .into_iter()
        .map(|(_, t)| t)
        .collect()
}

fn rows(db: &Db, sql: &str) -> Value {
    db.read(sql, &[]).unwrap()["rows"].clone()
}

/// What makes a change one its table can't hold.
type Unfit = fn(&mut Change);

/// A change row as svrngs-db writes one, for a set a test makes.
#[derive(Clone, Serialize, Deserialize)]
struct Change {
    table: String,
    pk: Vec<u8>,
    cid: String,
    val: Val,
    col_version: i64,
    cl: i64,
    seq: i64,
}

#[derive(Clone, Serialize, Deserialize)]
enum Val {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

/// `set` signed again by `by`, its own device's key, for `app`.
fn signed(mut set: Set, by: &SigningKey, app: &str) -> Set {
    set.head.changes = svrngs_core::set::changes(&set.changes);
    let at = At {
        group: "g",
        app,
        device: set.head.device.as_str(),
        nonce: &set.head.nonce,
        seq: set.head.seq.get(),
        schema: &set.head.schema,
    };
    set.head.sig = Sig::parse(&at.sign(by, &set.head.changes)).unwrap();
    set
}

/// `set` with its changes made by `f`, signed again by `by`.
fn remade(set: &Set, by: &SigningKey, app: &str, f: impl Fn(&mut Vec<Change>)) -> Set {
    let mut changes: Vec<Change> = postcard::from_bytes(&set.changes).unwrap();
    f(&mut changes);
    let mut set = set.clone();
    set.changes = postcard::to_allocvec(&changes).unwrap();
    signed(set, by, app)
}

/// Computer 9's one set of `app`, its write `op`.
fn written(dir: &Path, app: (&str, &str), op: Value) -> Set {
    let h = db(dir, 9, app);
    h.write(&op).unwrap();
    h.missing(&Ask::default(), BUDGET).unwrap().sets.remove(0)
}

fn card(id: &str, title: &str) -> Value {
    json!([{ "insert": "cards", "row": { "id": id, "title": title } }])
}

/// `bad`, a set computer 9 signed, offered to two new copies: each refuses it
/// for good, applies none of it, and counts it in its device's mark, so no
/// member sends it again.
fn refused_on_two_copies(dir: &Path, app: (&str, &str), bad: &Set, what: &str) {
    for n in [2, 3] {
        let to = db(&dir.join(what), n, app);
        assert_eq!(
            took(&to, std::slice::from_ref(bad)),
            [Took::Refused],
            "{what}, on copy {n}"
        );
        let table = if app.0 == "board" { "cards" } else { "t" };
        assert_eq!(
            rows(&to, &format!("SELECT * FROM {table}")),
            json!([]),
            "{what}: none of it applied"
        );
        let marks: Vec<u64> = to.ask().unwrap().marks.values().map(|m| m.get()).collect();
        assert_eq!(marks, [1], "{what}: counted in its mark");
    }
}

/// **A key cr-sqlite couldn't unpack is refused for good, on every copy,
/// and nothing of it reaches cr-sqlite**: an empty key; a count with no
/// column; an integer, or a text's length, said to take nine bytes; a text
/// shorter than its length says; a type cr-sqlite has no number for; a text
/// that isn't UTF-8. Each is signed by its own device's key, as a member
/// can always sign its own.
#[test]
fn a_key_cr_sqlite_cannot_unpack_is_refused_for_good_on_every_copy() {
    let dir = tempfile::tempdir().unwrap();
    let base = written(dir.path(), ("board", CARDS), card("c1", "milk"));
    let keys: [(&str, Vec<u8>); 8] = [
        ("empty", vec![]),
        ("a count and no column", vec![1]),
        ("a nine-byte integer", vec![1, (9 << 3) | 1, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        (
            "a nine-byte length",
            vec![1, (9 << 3) | 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, b'a'],
        ),
        ("a text short of its length", vec![1, (1 << 3) | 3, 5, b'a']),
        ("type 0", vec![1, 0]),
        ("type 7", vec![1, 7]),
        ("text not UTF-8", vec![1, (1 << 3) | 3, 1, 0xff]),
    ];
    for (what, pk) in keys {
        let bad = remade(&base, &key(9), "board", |c| {
            c.iter_mut().for_each(|c| c.pk = pk.clone())
        });
        refused_on_two_copies(dir.path(), ("board", CARDS), &bad, what);
    }
}

/// **A set that waits for its device is parsed when it is judged again**:
/// held while the record doesn't bind its key, then refused for good once it
/// does, and nothing of it reaches cr-sqlite.
#[test]
fn a_waiting_set_whose_key_cr_sqlite_cannot_unpack_is_refused_once_its_device_is_bound() {
    let dir = tempfile::tempdir().unwrap();
    let base = written(dir.path(), ("board", CARDS), card("c1", "milk"));
    let bad = remade(&base, &key(9), "board", |c| c.iter_mut().for_each(|c| c.pk = vec![]));
    let b = db(dir.path(), 2, ("board", CARDS));
    let call = Call {
        sets: vec![bad],
        ..Default::default()
    };
    let got = b.ingest(&call, &|_| false, None).unwrap();
    assert_eq!(got[0].1, Took::Waits);
    assert_eq!(b.rejudge(&all).unwrap(), 0, "judged again, and refused");
    assert_eq!(rows(&b, "SELECT * FROM cards"), json!([]));
    assert_eq!(b.rejudge(&all).unwrap(), 0, "and held no more");
}

/// **A key of the wrong shape for its table is refused for good, on every
/// copy**: a NULL for an `INTEGER PRIMARY KEY`, which cr-sqlite would give
/// each copy's own next rowid; two columns for a table keyed by one; a
/// byte after the key; an integer in more bytes than cr-sqlite packs it in.
#[test]
fn a_key_not_of_its_table_s_shape_is_refused_for_good_on_every_copy() {
    let dir = tempfile::tempdir().unwrap();
    let ints = written(
        &dir.path().join("ints"),
        ("ints", INTS),
        json!([{ "insert": "t", "row": { "id": 1, "v": "h" } }]),
    );
    let pk = |pk: Vec<u8>| move |c: &mut Vec<Change>| c.iter_mut().for_each(|c| c.pk = pk.clone());
    let null = remade(&ints, &key(9), "ints", pk(vec![1, 5]));
    // One copy holds rows of its own, so its next rowid differs from a new one's.
    for n in [2, 3] {
        let to = db(&dir.path().join("null"), n, ("ints", INTS));
        if n == 2 {
            for id in [10, 11, 12] {
                to.write(&json!([{ "insert": "t", "row": { "id": id, "v": "mine" } }]))
                    .unwrap();
            }
        }
        assert_eq!(
            took(&to, std::slice::from_ref(&null)),
            [Took::Refused],
            "a NULL key, copy {n}"
        );
        assert_eq!(
            rows(&to, "SELECT id FROM t WHERE v = 'h'"),
            json!([]),
            "a NULL key, copy {n}"
        );
    }
    let padded = remade(&ints, &key(9), "ints", pk(vec![1, (2 << 3) | 1, 0, 1]));
    refused_on_two_copies(dir.path(), ("ints", INTS), &padded, "an integer in two bytes");

    let cards = written(&dir.path().join("cards"), ("board", CARDS), card("c1", "milk"));
    let two = vec![2, (2 << 3) | 3, 2, b'c', b'1', (1 << 3) | 3, 1, b'x'];
    let shapes: [(&str, Vec<u8>); 2] = [
        ("two columns for one", two),
        ("a byte after the key", vec![1, (1 << 3) | 3, 2, b'c', b'1', 0]),
    ];
    for (what, bytes) in shapes {
        let bad = remade(&cards, &key(9), "board", pk(bytes));
        refused_on_two_copies(dir.path(), ("board", CARDS), &bad, what);
    }

    let pairs = written(
        &dir.path().join("pairs"),
        ("pairs", PAIRS),
        json!([{ "insert": "p", "row": { "a": "x", "b": 1, "v": "h" } }]),
    );
    let one = remade(&pairs, &key(9), "pairs", pk(vec![1, (1 << 3) | 3, 1, b'x']));
    let to = db(&dir.path().join("one"), 2, ("pairs", PAIRS));
    assert_eq!(took(&to, &[one]), [Took::Refused], "one column for a key of two");
    assert_eq!(rows(&to, "SELECT * FROM p"), json!([]));
}

/// **A change its table can't hold is refused for good, not retried**: a
/// column the table lacks, a table the app lacks, a key column named as a
/// column, a `col_version` or causal length of 0, a NULL for a NOT NULL
/// column. Each was a fault before, fetched again in every call while the
/// copy said it was syncing. Refused, it is counted, fetched once, and the
/// copy is no longer said syncing after a full call.
#[test]
fn a_change_its_table_cannot_hold_is_refused_for_good_and_fetched_once() {
    let dir = tempfile::tempdir().unwrap();
    let kept = written(&dir.path().join("kept"), ("kept", KEPT), card("c1", "milk"));
    let null = remade(&kept, &key(9), "kept", |c| {
        c.iter_mut()
            .filter(|c| c.cid == "title")
            .for_each(|c| c.val = Val::Null)
    });
    let to = db(&dir.path().join("null"), 2, ("kept", KEPT));
    assert_eq!(took(&to, &[null]), [Took::Refused], "a NULL for a NOT NULL column");

    let base = written(dir.path(), ("board", CARDS), card("c1", "milk"));
    let unfit: [(&str, Unfit); 5] = [
        ("a column the table lacks", |c| c.cid = "nope".into()),
        ("a table the app lacks", |c| c.table = "gone".into()),
        ("a key column as a column", |c| c.cid = "id".into()),
        ("col_version 0", |c| c.col_version = 0),
        ("causal length 0", |c| c.cl = 0),
    ];
    for (what, f) in unfit {
        let bad = remade(&base, &key(9), "board", |c| c.iter_mut().for_each(f));
        refused_on_two_copies(dir.path(), ("board", CARDS), &bad, what);
    }

    // Kept by computer 9 in place of its own set, and fetched by a member:
    // once, then never again, the member said synced.
    let bad = remade(&base, &key(9), "board", |c| {
        c.iter_mut().for_each(|c| c.cid = "nope".into())
    });
    let held = rusqlite::Connection::open(dir.path().join("computer-9/state/g/board.db")).unwrap();
    let sql = "UPDATE svrngs_sets SET sig = ?1, bytes = ?2";
    held.execute(sql, (bad.head.sig.as_str(), postcard::to_allocvec(&bad).unwrap()))
        .unwrap();
    svrngs_db::close(held).unwrap();
    let h = States::new(&dir.path().join("computer-9"), key(9))
        .open("g", "board", None)
        .unwrap();
    let b = db(&dir.path().join("fetched"), 2, ("board", CARDS));
    let sent = h.missing(&b.ask().unwrap(), BUDGET).unwrap();
    assert_eq!(b.ingest(&sent, &all, None).unwrap()[0].1, Took::Refused);
    for round in 0..10 {
        let again = h.missing(&b.ask().unwrap(), BUDGET).unwrap();
        assert!(again.sets.is_empty(), "round {round}: fetched again");
        b.ingest(&again, &all, None).unwrap();
    }
    assert!(!b.syncing(), "refused for good is no fault");
}

/// **A set written under a schema this copy doesn't hold waits**, as APP_STATE
/// §4 says, and is no fault: a set a newer version wrote, naming a column
/// that version added. It is held, and the copy isn't said syncing for it.
#[test]
fn a_set_under_a_schema_this_copy_does_not_hold_waits() {
    const WIDER: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, note TEXT);";
    let dir = tempfile::tempdir().unwrap();
    let base = written(dir.path(), ("board", WIDER), card("c1", "milk"));
    assert_eq!(base.head.schema, svrngs_core::set::schema(WIDER));
    let noted = remade(&base, &key(9), "board", |c| {
        let mut note = c[c.len() - 1].clone();
        (note.cid, note.val, note.seq) = ("note".into(), Val::Text("buy two".into()), note.seq + 1);
        c.push(note);
    });
    let b = db(&dir.path().join("b"), 2, ("board", CARDS));
    let call = Call {
        sets: vec![noted],
        full: true,
        ..Default::default()
    };
    assert_eq!(b.ingest(&call, &all, None).unwrap()[0].1, Took::Waits);
    assert!(!b.syncing(), "a set that waits is no fault");
    assert_eq!(rows(&b, "SELECT * FROM cards"), json!([]));
}

/// **Every key cr-sqlite packs from a write is one the parse takes**: integer
/// keys at each width's edges and negative ones, text keys of each length's
/// width, short blob keys and a float key, each written by its device and
/// taken on another copy, which ends equal. So the parse refuses no honest
/// set (the seal's own parse, and W1's seeds, hold the rest).
#[test]
fn every_key_cr_sqlite_packs_from_a_write_is_taken() {
    const KEYS: &str = "CREATE TABLE i (id INTEGER PRIMARY KEY NOT NULL, v TEXT); \
        CREATE TABLE s (id TEXT PRIMARY KEY NOT NULL, v TEXT); \
        CREATE TABLE b (id BLOB PRIMARY KEY NOT NULL, v TEXT); \
        CREATE TABLE f (id REAL PRIMARY KEY NOT NULL, v TEXT);";
    let dir = tempfile::tempdir().unwrap();
    let (h, to) = (db(dir.path(), 9, ("keys", KEYS)), db(dir.path(), 2, ("keys", KEYS)));
    let ints = [
        0,
        1,
        127,
        128,
        255,
        256,
        65_535,
        65_536,
        (1 << 31) - 1,
        1 << 31,
        (1 << 32) - 1,
        1 << 32,
        1 << 56,
        -1,
        i64::MIN,
        i64::MAX,
    ];
    let mut ops: Vec<Value> = ints
        .iter()
        .map(|n| json!({ "insert": "i", "row": { "id": n, "v": "x" } }))
        .collect();
    for n in [0, 1, 255, 256, 65_536] {
        ops.push(json!({ "insert": "s", "row": { "id": "é".repeat(n / 2) + &"x".repeat(n % 2), "v": "x" } }));
    }
    for n in [0usize, 1, 127] {
        let blob = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, vec![7u8; n]);
        ops.push(json!({ "insert": "b", "row": { "id": { "base64": blob }, "v": "x" } }));
    }
    ops.push(json!({ "insert": "f", "row": { "id": 1.5, "v": "x" } }));
    for op in ops {
        h.write(&json!([op])).unwrap();
    }
    let sent = h.missing(&to.ask().unwrap(), BUDGET).unwrap();
    let got = to.ingest(&sent, &all, None).unwrap();
    assert!(got.iter().all(|(_, t)| *t == Took::Taken), "{got:?}");
    for t in ["i", "s", "b", "f"] {
        let all = format!("SELECT * FROM {t} ORDER BY id");
        assert_eq!(rows(&h, &all), rows(&to, &all), "table {t}");
    }
}

/// **The write's own seal refuses what every other copy would refuse**, and
/// tells the page why: a cell whose `col_version` stands at 2^53, the most
/// any copy takes, can't take another write, so a write to it is refused
/// whole, with words, and nothing is sealed. The copies stay together, and
/// a write to another row is kept. A BLOB key cr-sqlite couldn't read back
/// on another computer is refused at the write too, in words. Plant: the
/// seal not parsing its set (the write answers `{count: 1}`).
#[test]
fn a_write_past_the_bound_is_refused_at_the_write_in_words() {
    let dir = tempfile::tempdir().unwrap();
    let base = written(dir.path(), ("board", CARDS), card("c1", "milk"));
    let at = remade(&base, &key(9), "board", |c| {
        c.iter_mut()
            .filter(|c| c.cid == "title")
            .for_each(|c| c.col_version = BOUND)
    });
    let (a, b) = (db(dir.path(), 1, ("board", CARDS)), db(dir.path(), 2, ("board", CARDS)));
    for to in [&a, &b] {
        assert_eq!(took(to, std::slice::from_ref(&at)), [Took::Taken]);
    }
    let marks = a.ask().unwrap().marks;
    let refused = a
        .write(&card("c1", "fixed"))
        .expect_err("past the bound, refused at the write");
    let words = refused.to_string();
    assert!(words.contains("title"), "the words name the column: {words}");
    assert_eq!(a.ask().unwrap().marks, marks, "nothing sealed");
    assert_eq!(rows(&a, "SELECT title FROM cards"), json!([{ "title": "milk" }]));
    a.write(&card("c2", "eggs")).unwrap();
    let sent = a.missing(&b.ask().unwrap(), BUDGET).unwrap();
    assert_eq!(
        b.ingest(&sent, &all, None)
            .unwrap()
            .into_iter()
            .map(|(_, t)| t)
            .collect::<Vec<_>>(),
        [Took::Taken]
    );
    let both = "SELECT id, title FROM cards ORDER BY id";
    assert_eq!(rows(&a, both), rows(&b, both), "the copies agree");

    const BLOBS: &str = "CREATE TABLE b (id BLOB PRIMARY KEY NOT NULL, v TEXT);";
    let h = db(dir.path(), 9, ("blobs", BLOBS));
    let blob = |n: usize| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, vec![7u8; n]);
    let key = |n| json!([{ "insert": "b", "row": { "id": { "base64": blob(n) }, "v": "x" } }]);
    let refused = h.write(&key(128)).expect_err("a BLOB key cr-sqlite reads back signed");
    assert!(refused.to_string().contains("BLOB key"), "{refused}");
    assert_eq!(refused.said(), (422, "unkept"));
    h.write(&key(127)).unwrap();
}
