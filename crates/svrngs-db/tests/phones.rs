//! A phone's write to an app's database (svrn-docs
//! `rings/current/APP_STATE.md` §4, "Phones"; MAGNETS_BUILD S2): prepared
//! by a computer, which keeps none of it, signed by the phone, applied as
//! a carried set. At the database itself, with no node or route. Each test
//! names the plant it was watched failing on.

use std::path::Path;
use std::sync::Arc;

use serde_json::{Value, json};
use svrngs_db::{Db, SigningKey, States};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, col TEXT DEFAULT 'todo');\n\
    CREATE TABLE seats (row INTEGER NOT NULL, seat INTEGER NOT NULL, who TEXT, PRIMARY KEY (seat, row));\n";

/// `key`'s computer's databases under `dir`.
fn states(dir: &Path, key: u8) -> States {
    States::new(dir, SigningKey::from_bytes(&[key; 32]))
}

/// The board's database on the computer of `key`, in a directory of its own.
fn board(key: u8) -> (tempfile::TempDir, Arc<Db>) {
    let dir = tempfile::tempdir().unwrap();
    let db = states(dir.path(), key).open("g", "board", Some(SCHEMA)).unwrap();
    (dir, db)
}

/// Every row of `cards`, by id.
fn cards(db: &Db) -> Value {
    db.read("SELECT id, title FROM cards ORDER BY id", &[]).unwrap()["rows"].clone()
}

/// **A computer prepares a phone's write and keeps none of it** (§4,
/// "Phones", step 1): Ana's computer prepares an update of her card and a
/// new one, and answers them signed as hers, with the ops, each table's
/// key in order, the rows changed and each op's row as it would stand.
/// Her board is as it was, and her next write takes the version the
/// prepare didn't spend. A no-op is answered its count alone, as nothing
/// is to sign. Plant, watched failing: the prepare committing its
/// transaction (Ana's board has the eggs).
#[test]
fn a_computer_prepares_a_phone_s_write_and_keeps_none_of_it() {
    let (_dir, db) = board(1);
    let milk = json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]);
    assert_eq!(db.write(&milk).unwrap().1.map(|s| s.get()), Some(1));
    let ops = json!([
        { "update": "cards", "key": { "id": "c1" }, "set": { "title": "eggs" } },
        { "insert": "cards", "row": { "id": "c2", "title": "jam" } },
    ]);
    let answer = db.prepare(&ops).unwrap();
    let (text, by, sig) = (answer["prepared"].as_str().unwrap(), &answer["by"], &answer["sig"]);
    assert_eq!(by, &json!(svrngs_core::actor_of(&SigningKey::from_bytes(&[1; 32]))));
    let message = svrngs_core::set::prepared("g", "board", text);
    assert!(svrngs_core::verify_hex(
        by.as_str().unwrap(),
        &message,
        sig.as_str().unwrap()
    ));
    let prepared: Value = serde_json::from_str(text).unwrap();
    assert_eq!(prepared["ops"], ops);
    assert_eq!(prepared["keys"], json!({ "cards": ["id"], "seats": ["seat", "row"] }));
    assert_eq!(prepared["count"], 2);
    let stands = json!([{ "id": "c1", "title": "eggs", "col": null }, { "id": "c2", "title": "jam", "col": null }]);
    assert_eq!(prepared["stands"], stands);
    assert_eq!(prepared["schema"].as_str().unwrap().len(), 64);
    assert!(prepared["changes"].as_str().is_some_and(|c| !c.is_empty()));
    assert_eq!(
        cards(&db),
        json!([{ "id": "c1", "title": "milk" }]),
        "nothing of it kept"
    );
    let tea = json!([{ "insert": "cards", "row": { "id": "c3", "title": "tea" } }]);
    assert_eq!(db.write(&tea).unwrap().1.map(|s| s.get()), Some(2), "no version spent");

    let same = json!([{ "update": "cards", "key": { "id": "c1" }, "set": { "title": "milk" } }]);
    assert_eq!(
        db.prepare(&same).unwrap(),
        json!({ "count": 1 }),
        "a no-op: nothing to sign"
    );
}
