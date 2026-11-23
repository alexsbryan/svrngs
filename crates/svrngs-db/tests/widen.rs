//! A database made wider for a newer version of its app (svrn-docs
//! `rings/current/APP_STATE.md` §4, "Its schema"; §7, invariant 14; W5; the
//! operator's review of `41d94397`): its file restored whole, never
//! replayed from the sets it keeps, the wider schema made on the copy, and
//! the copy put in its place only once it checks. So every row, clock,
//! mark and kept set stays, a copy alone among offline ones
//! included, and a copy made wider is the copy made from the wider schema:
//! the same text, columns, rows and clocks. At the database itself, with no
//! node or route. Each test names the plant it was watched failing on.

use std::path::Path;

use rusqlite::Connection;
use serde_json::{Value, json};
use svrngs_db::{Binds, Db, Refused, SigningKey, States, Unmade};

const V1: &str =
    "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);\nCREATE INDEX cards_title ON cards (title);\n";
/// A note a page sorts and compares without case, with an index of its
/// own; and a table of lists, whose index's name sorts before its table's.
const V2: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, note TEXT COLLATE NOCASE);\n\
    CREATE INDEX cards_title ON cards (title);\nCREATE INDEX cards_note ON cards (note);\n\
    CREATE TABLE zlists (id TEXT PRIMARY KEY NOT NULL, name TEXT);\nCREATE INDEX a_zlists ON zlists (name);\n";

fn bound() -> Binds {
    let keys = (0..=255).map(|n| svrngs_core::actor_of(&SigningKey::from_bytes(&[n; 32])));
    Binds::new(keys.map(|k| (k.clone(), k, false)), 4 << 20)
}

fn states(dir: &Path, key: u8) -> States {
    States::new(dir, SigningKey::from_bytes(&[key; 32]))
}

fn write(db: &Db, row: Value) -> Result<Value, Refused> {
    db.write(&json!([{ "insert": "cards", "row": row }]), &bound())
        .map(|(answer, _)| answer)
}

/// `to` takes `from`'s full call.
fn call(from: &Db, to: &Db) {
    to.ingest(&from.missing(&to.ask().unwrap(), 4 << 20).unwrap(), &bound(), None)
        .unwrap();
}

fn rows(db: &Db, sql: &str) -> Value {
    db.read(sql, &[]).unwrap()["rows"].clone()
}

/// What a second connection reads of the board under `dir`: its tables'
/// and indexes' text, the cards' columns, and every clock as
/// `crsql_changes` gives it, but for the rows' timestamps. SQLite's own
/// tables, the statistics `ANALYZE` keeps among them, are left out.
fn file(dir: &Path) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
    let side = Connection::open(dir.join("state/g/board.db")).unwrap();
    let all = |sql: &str| -> Vec<Vec<String>> {
        let mut stmt = side.prepare(sql).unwrap();
        let n = stmt.column_count();
        let each = stmt.query_map([], |r| {
            (0..n)
                .map(|i| {
                    r.get_ref(i).map(|v| match v {
                        rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                        other => format!("{other:?}"),
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        });
        each.unwrap().map(Result::unwrap).collect()
    };
    let schema = all(
        "SELECT name, sql FROM sqlite_master WHERE name NOT LIKE '%crsql%' AND name NOT LIKE 'svrngs%' AND name NOT LIKE 'sqlite%' AND sql NOT NULL ORDER BY name",
    );
    let columns = all("SELECT * FROM pragma_table_xinfo('cards')");
    let clocks = all(
        "SELECT \"table\", pk, cid, val, col_version, db_version, site_id, cl, seq FROM crsql_changes ORDER BY 1, 2, 3",
    );
    svrngs_db::close(side).unwrap();
    ([schema, columns].concat(), clocks)
}

/// **A saved edit outlasts an upgrade and a restart, with every other
/// computer offline, then syncs when they come back** (the release's test;
/// the operator's review of `41d94397`, (a)): Ana and Bo keep a board on
/// version 1. With Bo's computer off, Ana writes a card, and version 2,
/// which adds a note, makes her copy wider. Her card, her site's mark and
/// the set she keeps stand, and still do after she restarts. She writes a
/// card with a note. Bo comes back still on version 1: he takes her first
/// card, and her noted one waits, dropped and his mark for her site
/// unmoved. Once his copy is made wider too, the next call brings it. Plant,
/// watched failing: a copy made wider by replaying the sets it keeps
/// (Ana's mark lost, as `41d94397`'s remake lost it).
#[test]
fn a_saved_edit_outlasts_an_upgrade_and_a_restart_then_syncs() {
    let (ana_dir, bo_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana_states, bo_states) = (states(ana_dir.path(), 1), states(bo_dir.path(), 2));
    let ana = ana_states.carry("g", "board", V1).unwrap();
    let bo = bo_states.open("g", "board", Some(V1)).unwrap();
    call(&ana, &bo);
    write(&ana, json!({ "id": "c1", "title": "milk" })).unwrap();
    write(&ana, json!({ "id": "c1", "title": "tea" })).unwrap();
    let asked = ana.ask().unwrap();
    ana_states.widen(&ana, V2).unwrap();
    let tea = json!([{ "id": "c1", "title": "tea", "note": null }]);
    assert_eq!(rows(&ana, "SELECT * FROM cards"), tea, "the saved edit, made wider");
    assert_eq!(ana.ask().unwrap(), asked, "her mark and what she keeps");
    drop((ana, ana_states));
    let ana_states = states(ana_dir.path(), 1);
    let ana = ana_states.open("g", "board", None).unwrap();
    assert_eq!(rows(&ana, "SELECT * FROM cards"), tea, "and after a restart");
    assert_eq!(ana.ask().unwrap(), asked);
    write(&ana, json!({ "id": "c2", "title": "jam", "note": "Two" })).unwrap();

    call(&ana, &bo);
    let older = json!([{ "id": "c1", "title": "tea" }]);
    assert_eq!(rows(&bo, "SELECT * FROM cards"), older, "the noted card waits");
    let site = ana.ask().unwrap().marks;
    let theirs = bo.ask().unwrap().marks;
    assert!(
        site.iter().all(|(s, m)| theirs.get(s).is_none_or(|t| t < m)),
        "Bo's mark for her site unmoved past it"
    );
    bo_states.widen(&bo, V2).unwrap();
    call(&ana, &bo);
    let both = rows(&ana, "SELECT * FROM cards ORDER BY id");
    assert_eq!(rows(&bo, "SELECT * FROM cards ORDER BY id"), both);
    assert_eq!(both[1]["note"], "Two");
}

/// **A copy made wider is the copy made from the wider schema**: Ana's copy
/// on version 1, written and pruned, made wider by version 2's note, which
/// compares without case and has an index, against a copy made from
/// version 2 that took every set Ana's keeps. Each has the same text for
/// every table and index, the index version 1 had among them, the same
/// columns, the same cards and the same clocks. Each then sorts notes alike,
/// case set aside. Plants, watched failing: the table's text
/// left as `ALTER TABLE` writes it (the texts differ, and Ana's copy sorts
/// "B" before "a"); the new index made before its new table (the copy
/// refused).
#[test]
fn a_copy_made_wider_is_the_copy_made_from_the_wider_schema() {
    let (ana_dir, fresh_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ana_states = states(ana_dir.path(), 1);
    let ana = ana_states.carry("g", "board", V1).unwrap();
    for (id, title) in [("c1", "milk"), ("c2", "eggs"), ("c1", "tea")] {
        write(&ana, json!({ "id": id, "title": title })).unwrap();
    }
    ana_states.widen(&ana, V2).unwrap();
    let fresh_states = states(fresh_dir.path(), 2);
    let fresh = fresh_states.open("g", "board", Some(V2)).unwrap();
    call(&ana, &fresh);
    assert_eq!(file(ana_dir.path()), file(fresh_dir.path()));
    for (id, note) in [("c1", "B"), ("c2", "a")] {
        let op = json!([{ "update": "cards", "key": { "id": id }, "set": { "note": note } }]);
        ana.write(&op, &bound()).unwrap();
    }
    call(&ana, &fresh);
    let sorted = json!([{ "id": "c2" }, { "id": "c1" }]);
    for db in [&ana, &fresh] {
        assert_eq!(rows(db, "SELECT id FROM cards ORDER BY note"), sorted);
    }
}

/// **A version that moves a column makes no copy wider**: version 2 puts
/// its note before the title, where a copy made wider adds it after the
/// last, so the two would differ. It is refused, naming the column it
/// moves, and Ana's copy stays as it was, on version 1, writing. A version
/// that adds nothing, an older one, leaves the copy as it is, the same
/// database. Plant, watched failing: a column's place left out of what a
/// version keeps (the moved note taken).
#[test]
fn a_version_that_moves_a_column_makes_no_copy_wider() {
    let dir = tempfile::tempdir().unwrap();
    let ana_states = states(dir.path(), 1);
    let ana = ana_states.carry("g", "board", V1).unwrap();
    write(&ana, json!({ "id": "c1", "title": "milk" })).unwrap();
    let moved = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, note TEXT, title TEXT);";
    let refused = ana_states.widen(&ana, moved).err();
    assert_eq!(refused, Some(Refused::Schema(Unmade::Changed("cards.title".into()))));
    write(&ana, json!({ "id": "c2", "title": "eggs" })).unwrap();
    assert_eq!(rows(&ana, "SELECT * FROM cards").as_array().unwrap().len(), 2);
    ana_states.widen(&ana, V2).unwrap();
    ana_states.widen(&ana, V1).unwrap();
    let noted = rows(&ana, "SELECT * FROM cards ORDER BY id");
    assert_eq!(
        noted[0]["note"],
        Value::Null,
        "an older version adds nothing, and takes nothing away"
    );
}

/// **A copy is made wider while its page holds it** (svrn-docs
/// `app-state-diligence/invariants-2026-11-22/` §2.2): Ana's page holds her
/// database and listens to it. Made wider then, under the database's own
/// lock, the page's own handle has the wider file, its card kept, its note
/// written, and the page is told it changed: no subscription ends, and none
/// is waited on. Plant, watched failing: the wait for no other holder (the
/// page's handle still on version 1, its note refused).
#[test]
fn a_copy_is_made_wider_while_its_page_holds_it() {
    let dir = tempfile::tempdir().unwrap();
    let ana_states = states(dir.path(), 1);
    let page = ana_states.carry("g", "board", V1).unwrap();
    write(&page, json!({ "id": "c1", "title": "milk" })).unwrap();
    // Its subscription's own hold, beside the page's and this computer's.
    let (subscription, told) = (page.clone(), page.watch());
    ana_states.widen(&page, V2).unwrap();
    assert!(told.has_changed().unwrap(), "the page told");
    write(&page, json!({ "id": "c2", "title": "tea", "note": "one" })).unwrap();
    let both = json!([{ "id": "c1", "title": "milk", "note": null }, { "id": "c2", "title": "tea", "note": "one" }]);
    assert_eq!(rows(&subscription, "SELECT * FROM cards ORDER BY id"), both);
}

/// **A version that declares a kept column otherwise makes no copy wider**
/// (the review's P5): Ana's copy holds a card of `cards (id, a, b)`. A
/// version that makes `a` GENERATED, or the table WITHOUT ROWID, or adds a
/// CHECK to `b`, as it adds `c`, is refused in words, and one that folds
/// `a` where an index orders it is refused naming the index; each time her
/// card reads and writes as it did. One that folds `a` where no index
/// orders it makes her copy the copy made from it: the same text, columns,
/// cards and clocks as a fresh one. Plants, watched failing: `limits`
/// without its rowid and GENERATED arm (the GENERATED version taken);
/// `texts` without an index's collations (the folded index taken).
#[test]
fn a_version_that_declares_a_kept_column_otherwise_makes_no_copy_wider() {
    let v1 = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, a TEXT, b TEXT);\n";
    let indexed = "CREATE INDEX cards_a ON cards (a);\n";
    let card = json!([{ "id": "r1", "a": "A1", "b": "B1" }]);
    let refusals = [
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, a TEXT GENERATED ALWAYS AS ('pwned') VIRTUAL, b TEXT, c TEXT);\n",
            "",
            "In cards, a is GENERATED",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, a TEXT, b TEXT, c TEXT) WITHOUT ROWID;\n",
            "",
            "In cards, the table is WITHOUT ROWID",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, a TEXT, b TEXT CHECK (b IS NULL OR b > 'm'), c TEXT);\n",
            "",
            "The table cards has a CHECK",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, a TEXT COLLATE NOCASE, b TEXT, c TEXT);\n",
            indexed,
            "schema.sql drops or changes cards_a",
        ),
    ];
    for (v2, index, said) in refusals {
        let dir = tempfile::tempdir().unwrap();
        let ana_states = states(dir.path(), 1);
        let ana = ana_states.carry("g", "board", &format!("{v1}{index}")).unwrap();
        write(&ana, card[0].clone()).unwrap();
        let words = ana_states.widen(&ana, &format!("{v2}{index}")).unwrap_err().to_string();
        println!("MEASURE {v2}\n  {words}");
        assert!(words.starts_with(said) && words.contains("To fix it"), "{v2}: {words}");
        assert_eq!(rows(&ana, "SELECT id, a, b FROM cards"), card, "{v2}");
        write(&ana, json!({ "id": "r2", "b": "B2" })).unwrap();
    }
    let v2 = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, a TEXT COLLATE NOCASE, b TEXT, c TEXT);\n";
    let (ana_dir, fresh_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ana_states = states(ana_dir.path(), 1);
    let ana = ana_states.carry("g", "board", v1).unwrap();
    write(&ana, card[0].clone()).unwrap();
    ana_states.widen(&ana, v2).unwrap();
    let fresh_states = states(fresh_dir.path(), 2);
    let fresh = fresh_states.open("g", "board", Some(v2)).unwrap();
    call(&ana, &fresh);
    assert_eq!(file(ana_dir.path()), file(fresh_dir.path()), "{v2}");
}

/// **A version is judged once against a file** (the review's H3): Ana's
/// copy is made wider while the place its file is set aside in during the
/// swap is taken, by a directory, so the widening faults. The place is
/// then free again, and the version is answered as it was, never made
/// again at the next ask; once svrngs starts again it is judged afresh,
/// and taken. Plant, watched failing: `States::widen` keeping nothing it
/// judged (the version taken at the second ask).
#[test]
fn a_version_is_judged_once_against_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let ana_states = states(dir.path(), 1);
    let ana = ana_states.carry("g", "board", V1).unwrap();
    write(&ana, json!({ "id": "c1", "title": "milk" })).unwrap();
    let aside = dir.path().join("state/g/board.db.old");
    std::fs::create_dir(&aside).unwrap();
    let faulted = ana_states.widen(&ana, V2);
    assert!(matches!(faulted, Err(Refused::Fault(_))), "{faulted:?}");
    std::fs::remove_dir(&aside).unwrap();
    assert_eq!(ana_states.widen(&ana, V2), faulted, "answered as it was");
    drop((ana, ana_states));
    let ana_states = states(dir.path(), 1);
    let ana = ana_states.open("g", "board", None).unwrap();
    ana_states.widen(&ana, V2).unwrap();
}

/// **A restore that can't swap leaves the database open as it was** (the
/// review's L2): with the place the live file is kept in during the swap
/// taken, by a directory, Ana's copy made wider faults, and her database
/// still reads and writes, then after a restart holds both her cards on the
/// version it had. Plant, watched failing: the old connection closed before
/// the copy opens in its place (the next write panics under the database's
/// lock).
#[test]
fn a_restore_that_can_t_swap_leaves_the_database_open_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let ana_states = states(dir.path(), 1);
    let ana = ana_states.carry("g", "board", V1).unwrap();
    write(&ana, json!({ "id": "c1", "title": "milk" })).unwrap();
    std::fs::create_dir(dir.path().join("state/g/board.db.old")).unwrap();
    let faulted = ana_states.widen(&ana, V2);
    assert!(matches!(faulted, Err(Refused::Fault(_))), "{faulted:?}");
    write(&ana, json!({ "id": "c2", "title": "tea" })).unwrap();
    let both = json!([{ "id": "c1", "title": "milk" }, { "id": "c2", "title": "tea" }]);
    assert_eq!(rows(&ana, "SELECT * FROM cards ORDER BY id"), both);
    drop((ana, ana_states));
    let ana_states = states(dir.path(), 1);
    let ana = ana_states.open("g", "board", None).unwrap();
    assert_eq!(rows(&ana, "SELECT * FROM cards ORDER BY id"), both);
}

/// What `PRAGMA integrity_check` says of the board under `dir`, from a
/// second connection.
fn sound(dir: &Path) -> Vec<String> {
    let side = Connection::open(dir.join("state/g/board.db")).unwrap();
    let mut stmt = side.prepare("PRAGMA integrity_check").unwrap();
    let said = stmt.query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
    drop(stmt);
    svrngs_db::close(side).unwrap();
    said
}

/// **A version that orders or keeps the key otherwise makes no copy
/// wider** (the delta review's D1): every index SQLite holds for a kept
/// table is compared, the key's own among them, which has no text. A TEXT
/// key declared DESC where the copy's is ascending, and an INTEGER key made
/// DESC or no longer DESC, which SQLite then keeps apart from the rowid or
/// as it, are each refused, by `svrngs check` and at the copy, naming the
/// key, and the copy still finds each card by its key, sound. A version
/// that keeps the key as it is and adds a column is taken, a DESC key's
/// included, and finds each card by its key after. Plant, watched failing:
/// the code before this commit (each version taken; the TEXT key's copy
/// then found no card by its key, its key's index missing every row).
#[test]
fn a_version_that_orders_the_key_otherwise_makes_no_copy_wider() {
    let text = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);\n";
    let desc = "CREATE TABLE cards (id TEXT PRIMARY KEY DESC NOT NULL, title TEXT, note TEXT);\n";
    let int = "CREATE TABLE cards (id INTEGER PRIMARY KEY NOT NULL, title TEXT);\n";
    let int_desc = "CREATE TABLE cards (id INTEGER PRIMARY KEY DESC NOT NULL, title TEXT);\n";
    let noted = |v: &str| v.replace("title TEXT)", "title TEXT, note TEXT)");
    let ids = |v: &str| -> Vec<Value> {
        match v.contains("INTEGER") {
            true => (1..=26).map(|n| json!(n)).collect(),
            false => (b'a'..=b'z').map(|c| json!((c as char).to_string())).collect(),
        }
    };
    for (v1, v2) in [(text, desc.to_string()), (int, noted(int_desc)), (int_desc, noted(int))] {
        let refused = svrngs_db::check(&v2, Some(v1)).map(drop).map_err(|r| r.to_string());
        println!("MEASURE {v1} then {v2}\n  {refused:?}");
        assert!(
            refused.as_ref().is_err_and(|w| w.contains("the primary key of cards")),
            "{v2}: {refused:?}"
        );
        let dir = tempfile::tempdir().unwrap();
        let ana_states = states(dir.path(), 1);
        let ana = ana_states.carry("g", "board", v1).unwrap();
        for id in ids(v1) {
            write(&ana, json!({ "id": id, "title": "t" })).unwrap();
        }
        let held = ana_states.widen(&ana, &v2).map_err(|r| r.to_string());
        assert_eq!(held, refused, "refused alike at the copy");
        let third = ana
            .read("SELECT id FROM cards WHERE id = ?1", &[ids(v1)[2].clone()])
            .unwrap();
        assert_eq!(third["rows"], json!([{ "id": ids(v1)[2] }]), "{v2}");
        assert_eq!(sound(dir.path()), ["ok"], "{v2}");
    }
    for v1 in [text, int_desc] {
        let dir = tempfile::tempdir().unwrap();
        let ana_states = states(dir.path(), 1);
        let ana = ana_states.carry("g", "board", v1).unwrap();
        for id in ids(v1) {
            write(&ana, json!({ "id": id, "title": "t" })).unwrap();
        }
        ana_states.widen(&ana, &noted(v1)).unwrap();
        let third = ana
            .read("SELECT id FROM cards WHERE id = ?1", &[ids(v1)[2].clone()])
            .unwrap();
        assert_eq!(third["rows"], json!([{ "id": ids(v1)[2] }]), "{v1}");
        assert_eq!(sound(dir.path()), ["ok"], "{v1}");
    }
}
