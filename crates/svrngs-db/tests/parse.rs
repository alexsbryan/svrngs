//! A set's changes are parsed before cr-sqlite sees them (svrn-docs
//! `rings/current/APP_STATE.md` §4, the four outcomes): each change names one
//! of the app's tables, a column of it or the row's sentinel, a key in
//! cr-sqlite's packed form with the table's key columns and no NULL, and a
//! `col_version` and causal length from 1 to 2^53, a column's causal length
//! odd. A set that doesn't parse
//! is refused for good, the same on every copy, and counted, so it is
//! fetched once; a set written under a schema this copy doesn't hold, naming
//! what it lacks, is dropped and sent again.
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

/// Every key these tests sign with, bound, in the room a request has
/// beside its sets (svrngs-node `peer_wire.rs`, `ASK_ROOM_BYTES`).
fn bound() -> svrngs_db::Binds {
    let keys = (0..=255).map(|n| svrngs_core::actor_of(&SigningKey::from_bytes(&[n; 32])));
    svrngs_db::Binds::new(keys.map(|k| (k.clone(), k, false)), 4 << 20)
}

/// Computer `n`'s database of `app` in `g`, made from `schema`.
fn db(dir: &Path, n: u8, (app, schema): (&str, &str)) -> Arc<Db> {
    let states = States::new(&dir.join(format!("computer-{n}")), key(n));
    states.open("g", app, Some(schema)).unwrap()
}

fn took(to: &Db, sets: &[Set]) -> Vec<Took> {
    let call = Call {
        sets: sets.to_vec(),
        ..Default::default()
    };
    to.ingest(&call, &bound(), None)
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

/// Computer 9's one set of `app`, its write `op`, computer 9 the app's
/// root, synced from the start, as the computer that carries it.
fn written(dir: &Path, app: (&str, &str), op: Value) -> Set {
    let h = States::new(&dir.join("computer-9"), key(9))
        .carry("g", app.0, app.1)
        .unwrap();
    h.write(&op, &bound()).unwrap();
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
                to.write(&json!([{ "insert": "t", "row": { "id": id, "v": "mine" } }]), &bound())
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
    assert_eq!(b.ingest(&sent, &bound(), None).unwrap()[0].1, Took::Refused);
    for round in 0..10 {
        let again = h.missing(&b.ask().unwrap(), BUDGET).unwrap();
        assert!(again.sets.is_empty(), "round {round}: fetched again");
        b.ingest(&again, &bound(), None).unwrap();
    }
    assert!(!b.syncing(), "refused for good is no fault");
}

/// **A set written under a schema this copy doesn't hold, naming what it
/// lacks, is dropped and sent again** (`Took::NotYet`), as a set of a key
/// the record doesn't bind yet is, and the copy is still syncing: a set a newer version
/// wrote, naming a column that version added. Nothing of it is kept or
/// counted, so this copy's mark for its site doesn't move, and a call by
/// marks sends it again once this copy takes the wider schema
/// (`States::widen`). An older version's set at the wider copy, naming only
/// what it holds, is taken. (A newer version's insert names every column of
/// its own, NULL where the page gave none, so it names what an older copy
/// lacks.) Plants: the lacking set refused for good, as the parse refuses
/// any other (counted, so never sent again); every set under another schema
/// dropped (the older version's card isn't taken).
#[test]
fn a_set_under_a_schema_this_copy_does_not_hold_is_dropped_and_sent_again() {
    const WIDER: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, note TEXT);";
    let dir = tempfile::tempdir().unwrap();
    let base = written(dir.path(), ("board", WIDER), card("c1", "milk"));
    assert_eq!(base.head.schema, svrngs_core::set::schema(WIDER));
    let noted = remade(&base, &key(9), "board", |c| {
        let mut note = c[c.len() - 1].clone();
        (note.cid, note.val, note.seq) = ("note".into(), Val::Text("buy two".into()), note.seq + 1);
        c.push(note);
    });
    let states = States::new(&dir.path().join("b"), key(2));
    let b = states.open("g", "board", Some(CARDS)).unwrap();
    let call = |set: &Set| Call {
        sets: vec![set.clone()],
        full: true,
        synced: true,
        ..Default::default()
    };
    assert_eq!(b.ingest(&call(&noted), &bound(), None).unwrap()[0].1, Took::NotYet);
    assert!(b.syncing(), "a full reply that deferred a set isn't a complete sync");
    assert_eq!(rows(&b, "SELECT * FROM cards"), json!([]));
    assert!(b.ask().unwrap().marks.is_empty(), "its site's mark moved");
    states.widen(&b, WIDER).unwrap();
    assert_eq!(b.ingest(&call(&noted), &bound(), None).unwrap()[0].1, Took::Taken);
    assert!(!b.syncing(), "the complete exchange now includes the deferred row");
    assert_eq!(rows(&b, "SELECT note FROM cards"), json!([{ "note": "buy two" }]));
    let older = written(&dir.path().join("older"), ("board", CARDS), card("c2", "eggs"));
    let w = db(&dir.path().join("w"), 3, ("board", WIDER));
    assert_eq!(w.ingest(&call(&older), &bound(), None).unwrap()[0].1, Took::Taken);
    let taken = json!([{ "id": "c2", "title": "eggs", "note": null }]);
    assert_eq!(rows(&w, "SELECT * FROM cards"), taken);
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
        h.write(&json!([op]), &bound()).unwrap();
    }
    let sent = h.missing(&to.ask().unwrap(), BUDGET).unwrap();
    let got = to.ingest(&sent, &bound(), None).unwrap();
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
        .write(&card("c1", "fixed"), &bound())
        .expect_err("past the bound, refused at the write");
    let words = refused.to_string();
    assert!(words.contains("title"), "the words name the column: {words}");
    assert_eq!(a.ask().unwrap().marks, marks, "nothing sealed");
    assert_eq!(rows(&a, "SELECT title FROM cards"), json!([{ "title": "milk" }]));
    a.write(&card("c2", "eggs"), &bound()).unwrap();
    let sent = a.missing(&b.ask().unwrap(), BUDGET).unwrap();
    assert_eq!(
        b.ingest(&sent, &bound(), None)
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
    let refused = h
        .write(&key(128), &bound())
        .expect_err("a BLOB key cr-sqlite reads back signed");
    assert!(refused.to_string().contains("BLOB key"), "{refused}");
    assert_eq!(refused.said(), (422, "unkept"));
    h.write(&key(127), &bound()).unwrap();
}

/// **Only a row's person starts it anew, and only a row's own change
/// deletes it** (svrn-docs
/// `app-state-diligence/security-review-2026-11-23/`, R): a column change
/// at a causal length past its row's brings the row into its next life,
/// every other column at its default, on every copy that takes it. On a
/// row a person owns, a set signed by any other person's device is refused
/// for good, so no member starts another's row anew. A column change at an
/// even causal length, which cr-sqlite merges as the row's delete and no
/// write makes, is refused for good on every copy. Plants, each watched
/// failing: owned rows judged by their own change alone (the other's change
/// taken, the note wiped); the parse taking any causal length (the card's
/// title at 2 taken, the card deleted).
#[test]
fn only_a_row_s_person_starts_it_anew_and_only_its_own_change_deletes_it() {
    const VOTES: &str = "CREATE TABLE votes (person_key TEXT NOT NULL, film TEXT NOT NULL, yes INTEGER, \
        note TEXT, PRIMARY KEY (person_key, film)) STRICT;";
    let dir = tempfile::tempdir().unwrap();
    let nine = svrngs_core::actor_of(&key(9));
    let row = json!({ "person_key": nine, "film": "f1", "yes": 1, "note": "loved it" });
    let mine = written(
        &dir.path().join("votes"),
        ("votes", VOTES),
        json!([{ "insert": "votes", "row": row }]),
    );
    let mut theirs = mine.clone();
    theirs.head.device = svrngs_db::Device::parse(&svrngs_core::actor_of(&key(2))).unwrap();
    let anew = remade(&theirs, &key(2), "votes", |c| {
        c.retain(|c| c.cid == "yes");
        (c[0].cl, c[0].val) = (3, Val::Integer(0));
    });
    let to = db(dir.path(), 3, ("votes", VOTES));
    assert_eq!(took(&to, &[mine]), [Took::Taken]);
    assert_eq!(took(&to, &[anew]), [Took::Refused], "another's row started anew");
    assert_eq!(
        rows(&to, "SELECT yes, note FROM votes"),
        json!([{ "yes": 1, "note": "loved it" }])
    );

    let base = written(dir.path(), ("board", CARDS), card("c1", "milk"));
    let deletes = remade(&base, &key(9), "board", |c| {
        c.iter_mut().filter(|c| c.cid == "title").for_each(|c| c.cl = 2)
    });
    refused_on_two_copies(
        dir.path(),
        ("board", CARDS),
        &deletes,
        "a column at an even causal length",
    );
}

/// `set`, computer 9's, as device `n`'s, its changes made by `f` and signed
/// by `n`'s key: a member's own set, as any member can sign one.
fn as_device(set: &Set, n: u8, app: &str, f: impl Fn(&mut Vec<Change>)) -> Set {
    let mut theirs = set.clone();
    theirs.head.device = svrngs_db::Device::parse(&svrngs_core::actor_of(&key(n))).unwrap();
    remade(&theirs, &key(n), app, f)
}

/// **A key is kept as its column's type, so copies holding the same sets
/// end alike** (svrn-docs `app-state-diligence/security-review-2026-11-23/`,
/// M2): an INTEGER key for a TEXT key column, which SQLite keeps as TEXT
/// while cr-sqlite keys its clocks by the INTEGER, is refused for good on
/// every copy, and two copies taking it and an honest card of that key in
/// either order end on the honest card. A STRICT table holds no key to its
/// type either, as it keeps an INTEGER in a TEXT column as TEXT. Plant,
/// watched failing: the parse reading no key's type (the review's P8: the
/// copies end on "bad" and "honest").
#[test]
fn a_key_is_kept_as_its_column_s_type_so_copies_end_alike() {
    const STRICT: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT) STRICT;";
    for (n, schema) in [CARDS, STRICT].into_iter().enumerate() {
        let dir = tempfile::tempdir().unwrap();
        let honest = written(dir.path(), ("board", schema), card("5", "honest"));
        let bad = as_device(&honest, 2, "board", |c| {
            for c in c.iter_mut() {
                c.pk = vec![1, (1 << 3) | 1, 5];
                if c.cid == "title" {
                    c.val = Val::Text("bad".into());
                }
            }
        });
        let (a, b) = (
            db(dir.path(), 3, ("board", schema)),
            db(dir.path(), 4, ("board", schema)),
        );
        let took_a = [
            took(&a, std::slice::from_ref(&honest)),
            took(&a, std::slice::from_ref(&bad)),
        ];
        let took_b = [
            took(&b, std::slice::from_ref(&bad)),
            took(&b, std::slice::from_ref(&honest)),
        ];
        let (all, card) = ("SELECT id, title FROM cards", json!([{ "id": "5", "title": "honest" }]));
        assert_eq!((rows(&a, all), rows(&b, all)), (card.clone(), card), "schema {n}");
        let refused = [vec![Took::Refused], vec![Took::Taken]];
        assert_eq!((took_a, took_b), ([vec![Took::Taken], vec![Took::Refused]], refused));
    }
}

/// **A value is kept as its column's type, so a set every copy would fail
/// alike is refused for good** (svrn-docs
/// `app-state-diligence/security-review-2026-11-23/`, L1): a BLOB for a
/// STRICT table's TEXT column, which SQLite refuses on every copy where the
/// change wins, is refused at the parse, counted and never fetched again,
/// not a fault fetched every round with the copy said syncing. And a value
/// SQLite would convert where it is kept, an INTEGER for a TEXT column,
/// which a copy then compares as the TEXT it kept against a tie's, where
/// cr-sqlite ranks an INTEGER above any TEXT, so two copies taking it and a
/// tie of another device in either order part, is refused for good on
/// both, and they end alike. Plants, each watched failing: the parse
/// holding no value to its column's type (the BLOB a fault, the copy
/// syncing; the copies on "6" and "5").
#[test]
fn a_value_is_kept_as_its_column_s_type_so_a_set_every_copy_fails_is_refused() {
    const STRICT: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT) STRICT;";
    let dir = tempfile::tempdir().unwrap();
    let title = |n: u8, val: Val, base: &Set| {
        as_device(base, n, "board", move |c| {
            c.retain(|c| c.cid == "title");
            (c[0].col_version, c[0].val) = (2, val.clone());
        })
    };
    let strict = written(&dir.path().join("strict"), ("board", STRICT), card("c1", "milk"));
    let blob = title(2, Val::Blob(vec![1]), &strict);
    let synced = States::new(&dir.path().join("synced"), key(3));
    let to = synced.carry("g", "board", STRICT).unwrap();
    assert_eq!(took(&to, &[strict]), [Took::Taken]);
    assert_eq!(took(&to, &[blob]), [Took::Refused], "a BLOB for TEXT");
    assert!(!to.syncing(), "refused for good is no fault");

    let base = written(dir.path(), ("board", CARDS), card("c1", "milk"));
    let (five, six) = (title(2, Val::Integer(5), &base), title(3, Val::Text("6".into()), &base));
    let (a, b) = (db(dir.path(), 4, ("board", CARDS)), db(dir.path(), 5, ("board", CARDS)));
    for (to, order) in [(&a, [&five, &six]), (&b, [&six, &five])] {
        for set in [&base, order[0], order[1]] {
            took(to, std::slice::from_ref(set));
        }
    }
    let (all, six) = ("SELECT title FROM cards", json!([{ "title": "6" }]));
    assert_eq!((rows(&a, all), rows(&b, all)), (six.clone(), six));
    assert_eq!(took(&a, &[five]), [Took::Held], "refused for good, counted");
}

/// **A value past `VALUE_MOST` is refused for good on every copy, and at
/// the write in words** (`fit`), so every row a copy holds fits a page's
/// read (`Db::read`): computer 9's card, its title made 128 KiB and a byte, is
/// refused by two copies whose record binds three keys, so the set is
/// within its device's share and reaches the parse; none of it is applied;
/// and the write of such a card is refused at the writer. Plant, watched
/// failing: `fit` without its length arm (both copies take it).
#[test]
fn a_value_past_value_most_is_refused_for_good_on_every_copy() {
    let dir = tempfile::tempdir().unwrap();
    let base = written(dir.path(), ("board", CARDS), card("c1", "milk"));
    let long = "x".repeat(svrngs_db::VALUE_MOST + 1);
    let set = remade(&base, &key(9), "board", |c| {
        let titles = c.iter_mut().filter(|c| c.cid == "title");
        titles.for_each(|c| c.val = Val::Text(long.clone()))
    });
    let three = [2, 3, 9].map(|n| svrngs_core::actor_of(&key(n)));
    let three = svrngs_db::Binds::new(three.map(|k| (k.clone(), k, false)), BUDGET);
    for n in [2, 3] {
        let to = db(&dir.path().join("long"), n, ("board", CARDS));
        let call = Call {
            sets: vec![set.clone()],
            ..Default::default()
        };
        let took: Vec<Took> = (to.ingest(&call, &three, None).unwrap().into_iter())
            .map(|(_, t)| t)
            .collect();
        assert_eq!(took, [Took::Refused], "on copy {n}");
        assert_eq!(rows(&to, "SELECT * FROM cards"), json!([]), "none of it applied");
    }
    let refused = db(dir.path(), 4, ("board", CARDS))
        .write(&card("c2", &long), &bound())
        .expect_err("a value past VALUE_MOST kept");
    assert_eq!(refused.said(), (422, "unkept"), "{refused}");
}

/// **A file column holds one spelling of a file's name, on every copy**
/// (svrn-docs `rings/current/APP_FILES.md` §4, "A file column"; §7,
/// invariant 2): a column named `file`, or ending in `_file`, holds 64
/// lowercase hex, or null. Computer 9's card names its photo so, and is
/// taken, as is one naming none; the same card, its name in upper case or
/// in base32, as iroh-blobs also reads one, is refused for good by two
/// copies, none of it applied; and a write of either is refused at the
/// writer, in words naming the column. Plant, watched failing: the code
/// before this commit (the upper-case name taken at the write and at
/// apply).
#[test]
fn a_file_column_holds_one_spelling_of_a_file_s_name() {
    const PHOTOS: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, photo_file TEXT);";
    let dir = tempfile::tempdir().unwrap();
    let name = "0123456789abcdef".repeat(4);
    let photo = |n: Value| json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk", "photo_file": n } }]);
    let good = written(dir.path(), ("board", PHOTOS), photo(json!(name)));
    let none = written(&dir.path().join("none"), ("board", PHOTOS), photo(Value::Null));
    for (n, set) in [(5, &good), (6, &none)] {
        let to = db(&dir.path().join("taken"), n, ("board", PHOTOS));
        assert_eq!(took(&to, std::slice::from_ref(set)), [Took::Taken], "on copy {n}");
    }
    let base32 = "ciqdkzy5xb7acbbacnv3tgjslc5hmrpkxfyq2ukjvsy6dhnvi4kwsnq".to_string();
    for (what, bad) in [("upper case", name.to_uppercase()), ("base32", base32)] {
        let set = remade(&good, &key(9), "board", |c| {
            let named = c.iter_mut().filter(|c| c.cid == "photo_file");
            named.for_each(|c| c.val = Val::Text(bad.clone()))
        });
        refused_on_two_copies(dir.path(), ("board", PHOTOS), &set, what);
        let refused = db(&dir.path().join(what), 4, ("board", PHOTOS)).write(&photo(json!(bad)), &bound());
        let refused = refused.expect_err("a name in another spelling kept");
        assert_eq!(refused.said(), (422, "unkept"), "{refused}");
        assert!(refused.to_string().contains("cards.photo_file"), "{refused}");
    }
}
