//! A database file copied beside its original, and made again (svrn-docs
//! `rings/current/app-state-diligence/sync-simplify-2026-11-22/`, change 5;
//! APP_STATE §8, question 10): found by a set signed at its own site above
//! every version it sealed, made again with a fresh nonce once nothing else
//! holds it, and its overlap healed by twins. At the database itself, with
//! no node or route. Each test names the plant it was watched failing on.

use std::path::Path;
use std::sync::Arc;

use serde_json::{Value, json};
use svrngs_db::{Binds, Db, Refused, SigningKey, States};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);\n";

/// The keys every copy here binds: every test key's.
fn bound() -> Binds {
    let keys = (0..=255).map(|n| svrngs_core::actor_of(&SigningKey::from_bytes(&[n; 32])));
    Binds::new(keys.map(|k| (k.clone(), k, false)), 4 << 20)
}

/// The computer of `[key; 32]` keeping its data under `dir`.
fn states(dir: &Path, key: u8) -> States {
    States::new(dir, SigningKey::from_bytes(&[key; 32]))
}

/// The board on `states`, made from its schema if new.
fn board(states: &States) -> Arc<Db> {
    states.open("g", "board", Some(SCHEMA)).unwrap()
}

/// `ops` written on `db`.
fn write(db: &Db, ops: Value) -> Result<Value, Refused> {
    db.write(&ops, &bound()).map(|(answer, _)| answer)
}

/// `title` put on the card `id`, by key.
fn put(id: &str, title: &str) -> Value {
    json!([{ "insert": "cards", "row": { "id": id, "title": title } }])
}

/// `to` takes `from`'s full call.
fn call(from: &Db, to: &Db) {
    to.ingest(&from.missing(&to.ask().unwrap(), 4 << 20).unwrap(), &bound(), None)
        .unwrap();
}

/// Every card on `db`, by id.
fn cards(db: &Db) -> Value {
    db.read("SELECT id, title FROM cards ORDER BY id", &[]).unwrap()["rows"].clone()
}

/// **A copied file is found by its own site's set, made again, and its
/// overlap healed by twins**: Ana's computer keeps a board with one card,
/// and its file is copied to another computer of her key, run beside it.
/// Each writes on at the same site: Ana's copy a new card, the copy a card
/// twice. Ben's copy takes the copy's sets and drops the first, outranked.
/// Ana's copy takes the copy's sets: the one at her second version as a
/// twin, and the one above every version she sealed found as the copy's,
/// set aside, her file flagged and her writes refused, saying why. Made
/// again once nothing else holds her file, never under a live connection,
/// her copy has a site of its own, hands a phone its place
/// at a site the copy's file never hands, writes there, and names the sets
/// of the site it retired as twins, so Ben's copy, which pruned the copy's
/// set at that place, takes hers there too. Every copy then holds every
/// card. Plants, each watched failing: the own-site set above Ana's
/// versions not looked for (her copy never flagged); a set at a place the
/// call names a twin passed by where this copy pruned its pair (Ben's copy
/// never has Ana's jam); the retired site's sets not named as twins (the
/// same); a phone's first epoch fixed at 0 (the remade file hands the copy's
/// epoch, so the original's places, again).
#[test]
fn a_copied_file_is_found_made_again_and_healed_by_twins() {
    let (ana_dir, copy_dir, ben_dir) = (
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
    );
    let first = states(ana_dir.path(), 1);
    write(
        &states(ana_dir.path(), 1).carry("g", "board", SCHEMA).unwrap(),
        put("c1", "milk"),
    )
    .unwrap();
    drop(first);
    let at = |dir: &Path| dir.join("state/g/board.db");
    std::fs::create_dir_all(at(copy_dir.path()).parent().unwrap()).unwrap();
    std::fs::copy(at(ana_dir.path()), at(copy_dir.path())).unwrap();
    let (ana_states, copy_states, ben_states) = (
        states(ana_dir.path(), 1),
        states(copy_dir.path(), 1),
        states(ben_dir.path(), 2),
    );
    let (ana, copy, ben) = (board(&ana_states), board(&copy_states), board(&ben_states));
    write(&ana, put("c2", "jam")).unwrap();
    write(&copy, put("c1", "tea")).unwrap();
    write(&copy, put("c1", "ham")).unwrap();
    call(&copy, &ben);
    assert_eq!(cards(&ben), json!([{ "id": "c1", "title": "ham" }]));

    let phone = svrngs_core::actor_of(&SigningKey::from_bytes(&[7; 32]));
    let handed = |db: &Db| {
        let answer = db.prepare(&put("c8", "fig"), (&phone, &bound())).unwrap();
        let text: Value = serde_json::from_str(answer["prepared"].as_str().unwrap()).unwrap();
        (text["epoch"].as_u64().unwrap(), text["seq"].as_u64().unwrap())
    };
    let first = handed(&ana);
    assert_eq!(first.1, 1, "a phone's first place, handed before");
    call(&copy, &ana);
    assert!(
        ana.copied(),
        "Ana's file found copied by the copy's set above her versions"
    );
    let site = ana.ask().unwrap();
    ana_states.remake(&ana).unwrap();
    assert!(!ana.copied(), "made again");
    let after = handed(&ana);
    assert_ne!(
        after.0, first.0,
        "a fresh epoch, from the fresh nonce, so no place the copy hands: {after:?}"
    );
    call(&copy, &ana);
    write(&ana, put("c3", "pie")).unwrap();
    let marks = ana.ask().unwrap().marks;
    assert!(
        marks.keys().any(|s| !site.marks.contains_key(s)),
        "a site of its own: {marks:?}"
    );

    call(&ana, &ben);
    let all = json!([{ "id": "c1", "title": "ham" }, { "id": "c2", "title": "jam" }, { "id": "c3", "title": "pie" }]);
    assert_eq!(
        cards(&ben),
        all,
        "Ben's copy has Ana's jam, the twin at a place it pruned"
    );
    call(&ana, &copy);
    for db in [&ana, &copy, &ben] {
        assert_eq!(cards(db), all);
    }
}

/// **A database made again keeps all it held, its marks among it** (the
/// operator's review of `41d94397`, (b)): Ana's copy, alone, every other
/// computer offline, writes a card twice, so her first set is pruned,
/// outranked, and her site's mark counts both. Made again, her copy holds
/// the card, the one set it kept, her retired site's mark at 2 with nothing
/// kept above it, and its synced flag, and still does once restarted. Her
/// next write is at a site of its own. Plant, watched failing: the copy made
/// again by replaying the sets it keeps (the retired site's mark at 0, seq
/// 2 kept above a gap that prune made).
#[test]
fn a_database_made_again_keeps_its_rows_marks_and_sets() {
    let dir = tempfile::tempdir().unwrap();
    let ana_states = states(dir.path(), 1);
    let ana = ana_states.carry("g", "board", SCHEMA).unwrap();
    write(&ana, put("c1", "milk")).unwrap();
    write(&ana, put("c1", "tea")).unwrap();
    let (asked, kept) = (
        ana.ask().unwrap(),
        ana.missing(&Default::default(), 4 << 20).unwrap().sets,
    );
    let marks: Vec<u64> = asked.marks.values().map(|m| m.get()).collect();
    assert_eq!((marks, kept.len()), (vec![2], 1), "the first set pruned, both counted");
    let tea = json!([{ "id": "c1", "title": "tea" }]);
    let same = |ana: &Db| {
        assert_eq!(cards(ana), tea);
        assert_eq!(ana.ask().unwrap().marks, asked.marks, "the retired site's mark");
        assert!(ana.ask().unwrap().haves.is_empty(), "nothing kept above a gap");
        assert_eq!(ana.missing(&Default::default(), 4 << 20).unwrap().sets, kept);
        assert!(!ana.syncing(), "its synced flag");
    };
    ana_states.remake(&ana).unwrap();
    same(&ana);
    drop((ana, ana_states));
    let ana = board(&states(dir.path(), 1));
    same(&ana);
    write(&ana, put("c2", "jam")).unwrap();
    let marks = ana.ask().unwrap().marks;
    assert_eq!(marks.len(), 2, "a site of its own beside the retired one: {marks:?}");
}

/// **A restore that fails leaves the file as it was**: Ana's copy made
/// again where its restored copy can't be written (a directory stands at
/// the copy's path) is refused, and her board, its rows and its mark, is
/// still hers, open, and after a restart. Plant, watched failing: the old
/// file taken away before the copy is checked (her card gone).
#[test]
fn a_restore_that_fails_leaves_the_file_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let ana_states = states(dir.path(), 1);
    let ana = ana_states.carry("g", "board", SCHEMA).unwrap();
    write(&ana, put("c1", "milk")).unwrap();
    let asked = ana.ask().unwrap();
    std::fs::create_dir(dir.path().join("state/g/board.db.new")).unwrap();
    assert!(ana_states.remake(&ana).is_err(), "the restore refused");
    let milk = json!([{ "id": "c1", "title": "milk" }]);
    let ana = board(&ana_states);
    assert_eq!((cards(&ana), ana.ask().unwrap()), (milk.clone(), asked.clone()));
    drop((ana, ana_states));
    let ana = board(&states(dir.path(), 1));
    assert_eq!((cards(&ana), ana.ask().unwrap()), (milk, asked));
}

/// **A copied file is made again while its person's page holds it**
/// (svrn-docs `app-state-diligence/invariants-2026-11-22/` §2.2, finding
/// (c)): Ana's file is copied to another computer of her key, which writes
/// above her versions, and her copy finds itself copied, while her page
/// holds her database and listens to it. Made again then, her page's own
/// handle has the new file: not copied, her card on it, its next write
/// taken at a site of its own, and the page told it changed; the copy's next
/// call brings its card, at the site she retired. Plant, watched
/// failing: the wait for no other holder (her page's write refused `Copied`
/// for as long as it holds the database).
#[test]
fn a_copied_file_is_made_again_while_its_person_s_page_holds_it() {
    let (ana_dir, copy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ana_states = states(ana_dir.path(), 1);
    let ana = ana_states.carry("g", "board", SCHEMA).unwrap();
    write(&ana, put("c1", "milk")).unwrap();
    drop(ana);
    ana_states.close("g");
    let at = |dir: &Path| dir.join("state/g/board.db");
    std::fs::create_dir_all(at(copy_dir.path()).parent().unwrap()).unwrap();
    std::fs::copy(at(ana_dir.path()), at(copy_dir.path())).unwrap();
    let copy_states = states(copy_dir.path(), 1);
    let copy = board(&copy_states);
    write(&copy, put("c2", "tea")).unwrap();
    let page = board(&ana_states);
    // Its subscription's own hold, beside the page's and this computer's.
    let (subscription, told) = (page.clone(), page.watch());
    call(&copy, &page);
    assert!(page.copied(), "found copied");
    let site = page.ask().unwrap().marks;
    ana_states.remake(&page).unwrap();
    assert!(!page.copied(), "the page's own handle has the new file");
    assert!(told.has_changed().unwrap(), "and the page is told");
    write(&page, put("c3", "pie")).unwrap();
    call(&copy, &page);
    let marks = page.ask().unwrap().marks;
    assert!(
        marks.keys().any(|s| !site.contains_key(s)),
        "a site of its own: {marks:?}"
    );
    let all = json!([{ "id": "c1", "title": "milk" }, { "id": "c2", "title": "tea" }, { "id": "c3", "title": "pie" }]);
    assert_eq!(cards(&page), all);
    assert_eq!(cards(&subscription), all, "the subscription's hold, the same file");
}
