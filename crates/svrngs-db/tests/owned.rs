//! Owned rows (svrn-docs APP_STATE §4, "Who may write which row"; §7,
//! invariant 13), at the databases themselves: a row of a table whose key
//! begins with `person_key` changes only by its person's devices, judged by
//! Eudoxia's rule at the write and again at apply (`src/owned.rs`). Each
//! names the plant it was watched failing on.

use std::path::Path;
use std::sync::Arc;

use serde_json::{Value, json};
use svrngs_db::{Binds, Call, Db, Refused, SigningKey, States, Took};

const SCHEMA: &str = "CREATE TABLE votes (person_key TEXT NOT NULL, film TEXT NOT NULL, yes INTEGER, \
    PRIMARY KEY (person_key, film)) STRICT;";

/// Priya, Sam and a third person, each one computer.
const PRIYA: u8 = 1;
const SAM: u8 = 2;
const THIRD: u8 = 3;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

/// Computer `n`'s key, as the record spells it, which is its person's first.
fn actor(n: u8) -> String {
    svrngs_core::actor_of(&key(n))
}

/// The record as it stands: each computer bound to its own person, and
/// `cut` the people the office has cut.
fn record(cut: &[u8]) -> Binds {
    Binds::new((1..=3).map(|n| (actor(n), actor(n), cut.contains(&n))), 4 << 20)
}

fn board(dir: &Path, n: u8) -> Arc<Db> {
    let states = States::new(&dir.join(format!("computer-{n}")), key(n));
    states.open("g", "films", Some(SCHEMA)).unwrap()
}

/// A vote by `person` for `film`, in the write form.
fn vote(person: u8, film: &str, yes: i64) -> Value {
    json!([{ "insert": "votes", "row": { "person_key": actor(person), "film": film, "yes": yes } }])
}

fn unvote(person: u8, film: &str) -> Value {
    json!([{ "delete": "votes", "key": { "person_key": actor(person), "film": film } }])
}

fn votes(db: &Db) -> Value {
    let read = db.read("SELECT person_key, film, yes FROM votes ORDER BY person_key, film", &[]);
    read.unwrap()["rows"].clone()
}

fn call(from: &Db, to: &Db) -> Call {
    from.missing(&to.ask().unwrap(), 4 << 20).unwrap()
}

fn took(to: &Db, call: &Call, binds: &Binds) -> Vec<Took> {
    to.ingest(call, binds, None)
        .unwrap()
        .into_iter()
        .map(|(_, t)| t)
        .collect()
}

/// **A page writes only its own person's rows** (invariant 13's second
/// plant): Priya's computer writes her vote, and is refused one written as
/// Sam, a change to Sam's row, and a delete of it while Sam stands, each in
/// words that say whose and what to do; once the office has cut Sam's key,
/// her delete of his row is taken. Plant, watched failing: `Db::seal`
/// without `owned::mine`, and the vote as Sam is taken.
#[test]
fn a_page_writes_only_its_own_person_s_rows() {
    let dir = tempfile::tempdir().unwrap();
    let (priya, sam) = (board(dir.path(), PRIYA), board(dir.path(), SAM));
    priya.write(&vote(PRIYA, "f1", 1), &record(&[])).unwrap();
    sam.write(&vote(SAM, "f1", 1), &record(&[])).unwrap();
    assert_eq!(took(&priya, &call(&sam, &priya), &record(&[])), [Took::Taken]);
    let as_sam = priya.write(&vote(SAM, "f2", 1), &record(&[])).unwrap_err();
    assert_eq!(as_sam, Refused::NotYours("votes".into()));
    assert!(as_sam.to_string().contains("give the row your own you_key"), "{as_sam}");
    let change = json!([{ "update": "votes", "key": { "person_key": actor(SAM), "film": "f1" }, "set": { "yes": 0 } }]);
    assert_eq!(
        priya.write(&change, &record(&[])).unwrap_err(),
        Refused::NotYours("votes".into())
    );
    let gone = priya.write(&unvote(SAM, "f1"), &record(&[])).unwrap_err();
    assert_eq!(
        (gone.said(), &gone),
        ((403, "not-yours"), &Refused::NotYetGone("votes".into()))
    );
    assert_eq!(
        votes(&priya).as_array().map(Vec::len),
        Some(2),
        "nothing of the three kept"
    );
    priya.write(&unvote(SAM, "f1"), &record(&[SAM])).unwrap();
    assert_eq!(
        votes(&priya),
        json!([{ "person_key": actor(PRIYA), "film": "f1", "yes": 1 }])
    );
}

/// **A set changing another person's row is refused on every copy**
/// (invariant 13's first plant): Sam's computer, its record lying that Sam
/// is Priya, writes Priya's vote; a third copy refuses it for good, counted
/// and never fetched again, whether Sam was bound when it first came or it
/// was dropped until he was; Priya's vote stands there as she wrote it.
/// Plant, watched failing: `Db::take` without `owned::kept` (the third copy
/// takes it, both times).
#[test]
fn a_set_changing_another_person_s_row_is_refused_on_every_copy() {
    let dir = tempfile::tempdir().unwrap();
    let (priya, sam, third) = (
        board(dir.path(), PRIYA),
        board(dir.path(), SAM),
        board(dir.path(), THIRD),
    );
    priya.write(&vote(PRIYA, "f1", 1), &record(&[])).unwrap();
    assert_eq!(took(&third, &call(&priya, &third), &record(&[])), [Took::Taken]);
    let lying = Binds::new([(actor(SAM), actor(PRIYA), false)], 4 << 20);
    sam.write(&vote(PRIYA, "f1", 0), &lying).unwrap();
    let forged = call(&sam, &third);
    assert_eq!(took(&third, &forged, &record(&[])), [Took::Refused]);
    assert!(call(&sam, &third).sets.is_empty(), "refused for good, counted");
    assert_eq!(
        votes(&third),
        json!([{ "person_key": actor(PRIYA), "film": "f1", "yes": 1 }])
    );

    let later = board(&dir.path().join("later"), THIRD);
    let unbound = Binds::new([(actor(PRIYA), actor(PRIYA), false)], 4 << 20);
    assert_eq!(took(&later, &call(&priya, &later), &unbound), [Took::Taken]);
    assert_eq!(took(&later, &forged, &unbound), [Took::NotYet], "Sam not bound yet");
    let again = call(&sam, &later);
    assert_eq!(
        took(&later, &again, &record(&[])),
        [Took::Refused],
        "sent again once bound: refused"
    );
    assert_eq!(
        votes(&later),
        json!([{ "person_key": actor(PRIYA), "film": "f1", "yes": 1 }])
    );
}

/// **A delete of a person's row by another waits for the office's cut**
/// (Eudoxia `db.rs`, "A removed person's rows"): Sam's computer, its record
/// holding Priya's cut, deletes her vote; a copy whose record doesn't hold
/// the cut yet drops the set, applying and counting nothing, and takes it
/// when it is sent again once its record does. Plant, watched failing:
/// `owned::kept` keeping a wait as a refusal (counted, so never sent again,
/// and the vote stands for good).
#[test]
fn a_delete_of_a_person_s_row_by_another_waits_for_the_office_s_cut() {
    let dir = tempfile::tempdir().unwrap();
    let (priya, sam, third) = (
        board(dir.path(), PRIYA),
        board(dir.path(), SAM),
        board(dir.path(), THIRD),
    );
    priya.write(&vote(PRIYA, "f1", 1), &record(&[])).unwrap();
    assert_eq!(took(&sam, &call(&priya, &sam), &record(&[])), [Took::Taken]);
    assert_eq!(took(&third, &call(&priya, &third), &record(&[])), [Took::Taken]);
    sam.write(&unvote(PRIYA, "f1"), &record(&[PRIYA])).unwrap();
    assert_eq!(took(&third, &call(&sam, &third), &record(&[])), [Took::NotYet]);
    assert_eq!(
        votes(&third).as_array().map(Vec::len),
        Some(1),
        "dropped, nothing applied"
    );
    assert_eq!(
        took(&third, &call(&sam, &third), &record(&[PRIYA])),
        [Took::Taken],
        "sent again by marks"
    );
    assert_eq!(votes(&third), json!([]));
}
