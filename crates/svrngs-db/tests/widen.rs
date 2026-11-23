//! A database made wider for a newer version of its app (svrn-docs
//! `rings/current/APP_STATE.md` §4, "Its schema"; §7, invariant 14; W5; the
//! operator's review of `41d94397`): its file restored whole, never
//! replayed from the sets it keeps, the wider schema made on the copy, and
//! the copy put in its place only once it checks. So every row, clock,
//! mark, kept set and handed place stays, a copy alone among offline ones
//! included, and a copy made wider is the copy made from the wider schema:
//! the same text, columns, rows and clocks. At the database itself, with no
//! node or route. Each test names the plant it was watched failing on.

use std::path::Path;
use std::sync::Arc;

use rusqlite::Connection;
use serde_json::{Value, json};
use svrngs_db::{Binds, Db, Refused, SigningKey, States, Unmade};

const V1: &str =
    "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);\nCREATE INDEX cards_title ON cards (title);\n";
/// A note a page sorts and compares without case, and holds short, with an
/// index of its own; and a table of lists, whose index's name sorts before
/// its table's.
const V2: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, note TEXT COLLATE NOCASE CHECK (length(note) < 9));\n\
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
    let ana = ana_states.widen(ana, V2).unwrap();
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
    let bo = bo_states.widen(bo, V2).unwrap();
    call(&ana, &bo);
    let both = rows(&ana, "SELECT * FROM cards ORDER BY id");
    assert_eq!(rows(&bo, "SELECT * FROM cards ORDER BY id"), both);
    assert_eq!(both[1]["note"], "Two");
}

/// **A copy made wider is the copy made from the wider schema**: Ana's copy
/// on version 1, written and pruned, made wider by version 2's note, which
/// compares without case, checks its length and has an index, against a
/// copy made from version 2 that took every set Ana's keeps. Each has the
/// same text for every table and index, the index version 1 had among
/// them, the same columns, the same cards and the same clocks. Each then sorts notes alike, case set aside, and
/// refuses the same long note. Plants, watched failing: the table's text
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
    let ana = ana_states.widen(ana, V2).unwrap();
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
        let long = json!([{ "update": "cards", "key": { "id": "c1" }, "set": { "note": "ninechars" } }]);
        assert!(db.write(&long, &bound()).is_err(), "the long note refused");
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
    let refused = ana_states.widen(ana.clone(), moved).err();
    assert_eq!(refused, Some(Refused::Schema(Unmade::Changed("cards.title".into()))));
    let ana = ana_states.open("g", "board", None).unwrap();
    write(&ana, json!({ "id": "c2", "title": "eggs" })).unwrap();
    assert_eq!(rows(&ana, "SELECT * FROM cards").as_array().unwrap().len(), 2);
    let wide = ana_states.widen(ana, V2).unwrap();
    let same = ana_states.widen(wide.clone(), V1).unwrap();
    assert!(Arc::ptr_eq(&wide, &same), "an older version adds nothing");
}

/// **A copy made wider waits for its pages' subscriptions**: while a page
/// holds Ana's database, it is given back as it is, marked to retire, and
/// its subscriptions told it changed, so each ends; once none holds it, the
/// next ask makes it wider. Plant, watched failing: the database not marked
/// (its subscriptions never told to end).
#[test]
fn a_copy_made_wider_waits_for_its_pages_subscriptions() {
    let dir = tempfile::tempdir().unwrap();
    let ana_states = states(dir.path(), 1);
    let ana = ana_states.carry("g", "board", V1).unwrap();
    let (page, told) = (ana.clone(), ana.watch());
    let held = ana_states.widen(ana, V2).unwrap();
    assert!(held.retiring() && told.has_changed().unwrap(), "marked, and told");
    assert!(Arc::ptr_eq(&held, &page), "given back as it is");
    drop(page);
    let ana = ana_states.widen(held, V2).unwrap();
    assert!(!ana.retiring());
    write(&ana, json!({ "id": "c1", "title": "milk", "note": "one" })).unwrap();
}
