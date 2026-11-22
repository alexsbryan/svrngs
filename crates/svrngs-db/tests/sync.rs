//! Change sets between copies of an app's database, at the databases
//! themselves, with no node or route (svrn-docs `rings/current/APP_STATE.md`
//! §4, "Sealed and signed", "What travels" and the four outcomes). Each
//! names the plant it was watched failing on (svrn-docs
//! `app-state-diligence/b-sync-build-2026-11-22/`, `plants.txt`).

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use svrngs_core::set::{At, site};
use svrngs_db::{Ask, Call, Db, SigningKey, States, Took};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);";
const BUDGET: usize = 4 << 20;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

/// Computer `n`'s database of the board in `g`, under `dir`.
fn board(dir: &Path, n: u8) -> Arc<Db> {
    let states = States::new(&dir.join(format!("computer-{n}")), key(n));
    states.open("g", "board", Some(SCHEMA)).unwrap()
}

fn write(db: &Db, id: &str, title: &str) {
    let op = json!([{ "insert": "cards", "row": { "id": id, "title": title } }]);
    db.write(&op).unwrap();
}

fn cards(db: &Db) -> Value {
    db.read("SELECT id, title FROM cards ORDER BY id", &[]).unwrap()["rows"].clone()
}

/// Every set `from` keeps that `to` lacks, as one call.
fn call(from: &Db, to: &Db) -> Call {
    from.missing(&to.ask().unwrap(), BUDGET).unwrap()
}

/// How each set of a call ended.
fn took(to: &Db, call: &Call) -> Vec<Took> {
    let all = |_: &str| true;
    to.ingest(call, &all, None)
        .unwrap()
        .into_iter()
        .map(|(_, t)| t)
        .collect()
}

/// A device's mark for its one site, in `ask`.
fn marks(ask: &Ask) -> Vec<u64> {
    ask.marks.values().map(|m| m.get()).collect()
}

/// A change row as svrngs-db writes one, for a set a test makes.
#[derive(Serialize, Deserialize)]
struct Change {
    table: String,
    pk: Vec<u8>,
    cid: String,
    val: Val,
    col_version: i64,
    cl: i64,
    seq: i64,
}

#[derive(Serialize, Deserialize)]
enum Val {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

/// `set`'s changes made by `f`, signed again by `by`.
fn remade(set: &svrngs_db::Set, by: &SigningKey, f: impl Fn(&mut Vec<Change>)) -> svrngs_db::Set {
    let mut changes: Vec<Change> = postcard::from_bytes(&set.changes).unwrap();
    f(&mut changes);
    let mut set = set.clone();
    set.changes = postcard::to_allocvec(&changes).unwrap();
    set.head.changes = svrngs_core::set::changes(&set.changes);
    let at = At {
        group: "g",
        app: "board",
        device: set.head.device.as_str(),
        nonce: &set.head.nonce,
        seq: set.head.seq.get(),
        schema: &set.head.schema,
    };
    set.head.sig = svrngs_db::Sig::parse(&at.sign(by, &set.head.changes)).unwrap();
    set
}

/// **A write seals one set, signed by its computer, in its own
/// transaction** (invariant 2): its site is its device's with the nonce its
/// database was made with; it checks under the device's key alone; a write
/// that changes nothing seals none. Another computer takes it once and has
/// the card, and fetches it no more. Plants: the seal signing with another
/// key (B refuses it as not its device's); a write's set never kept (B is
/// sent nothing).
#[test]
fn a_write_s_set_is_its_computer_s_and_is_taken_once() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (board(dir.path(), 1), board(dir.path(), 2));
    write(&a, "c1", "milk");
    write(&a, "c1", "milk");
    let sent = call(&a, &b);
    assert_eq!(
        sent.sets.len(),
        1,
        "one set; the write that changed nothing sealed none"
    );
    let set = &sent.sets[0];
    let device = svrngs_core::actor_of(&key(1));
    assert_eq!((set.head.device.as_str(), set.head.seq.get()), (device.as_str(), 1));
    assert_eq!(set.head.schema, svrngs_core::set::schema(SCHEMA));
    let place = (site("g", "board", &device, &set.head.nonce), set.head.seq);
    assert_eq!(
        a.ask().unwrap().marks,
        [(place.0, set.head.seq)].into(),
        "counted in its own mark"
    );
    assert_eq!(b.ingest(&sent, &|_| true, None).unwrap(), [(place, Took::Taken)]);
    assert_eq!(cards(&b), json!([{ "id": "c1", "title": "milk" }]));
    assert!(call(&a, &b).sets.is_empty(), "fetched once");
    assert_eq!(took(&b, &sent), [Took::Held], "and passed by if sent again");
}

/// **A refused set is fetched once and stalls nothing after it**
/// (invariant 12, its case in W's `syncs.rs`, `refused`): H, a hostile
/// computer, keeps four sets, the second of a `col_version` past 2^53. B
/// takes the first, third and fourth, refuses the second for good, counted
/// in H's mark and never applied, and ten calls more send nothing. Owned
/// rows are refused at the write and at apply in slice D. Plant: the
/// spike's refusal, which holds its device's mark (the refused set fetched
/// again in each of the ten calls).
#[test]
fn a_refused_set_is_fetched_once_and_stalls_nothing_after_it() {
    let dir = tempfile::tempdir().unwrap();
    let (h, b) = (board(dir.path(), 1), board(dir.path(), 2));
    for (id, title) in [("c1", "milk"), ("c2", "eggs"), ("c3", "jam"), ("c1", "salt")] {
        write(&h, id, title);
    }
    let all = call(&h, &b);
    let bad = remade(&all.sets[1], &key(1), |c| c[0].col_version = (1 << 53) + 1);
    let held = rusqlite::Connection::open(dir.path().join("computer-1/state/g/board.db")).unwrap();
    let bytes = postcard::to_allocvec(&bad).unwrap();
    held.execute(
        "UPDATE svrngs_sets SET sig = ?1, bytes = ?2 WHERE seq = 2",
        (&bad.head.sig, bytes),
    )
    .unwrap();
    svrngs_db::close(held).unwrap();
    let sent = call(&h, &b);
    assert_eq!(sent.sets[1], bad);
    assert_eq!(took(&b, &sent), [Took::Taken, Took::Refused, Took::Taken, Took::Taken]);
    assert_eq!(
        cards(&b),
        json!([{ "id": "c1", "title": "salt" }, { "id": "c3", "title": "jam" }])
    );
    assert_eq!(marks(&b.ask().unwrap()), [4], "the refusal counted in the mark");
    for _ in 0..10 {
        assert!(call(&h, &b).sets.is_empty(), "nothing fetched again");
    }
}

/// **A set whose signature isn't its device's is dropped as if it never
/// came**: its mark doesn't move, so a member can't hide the real set at
/// its seq behind a forged one; nothing after it stops; the real one is
/// taken when it comes. Plant: the forged set counted in the mark (the real
/// one passed by as held).
#[test]
fn a_set_not_its_device_s_is_dropped_and_its_mark_holds() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (board(dir.path(), 1), board(dir.path(), 2));
    write(&a, "c1", "milk");
    write(&a, "c2", "eggs");
    let real = call(&a, &b);
    let mut forged = real.clone();
    forged.sets[0] = remade(&real.sets[0], &key(9), |c| c[0].val = Val::Text("salt".into()));
    forged.full = false;
    assert_eq!(took(&b, &forged), [Took::NotItsDevice, Took::Taken]);
    assert_eq!(cards(&b), json!([{ "id": "c2", "title": "eggs" }]));
    assert_eq!(
        b.ask().unwrap().haves.len(),
        1,
        "the second counted above an unmoved mark"
    );
    assert_eq!(took(&b, &call(&a, &b)), [Took::Taken], "the real first set, fetched");
    assert_eq!(marks(&b.ask().unwrap()), [2]);
}

/// **A set whose device the record doesn't bind yet waits**: kept, held,
/// counted, never applied, and no member's mark adopted past its site; once
/// the record binds the device, it is applied (`rejudge`). Plant: `rejudge`
/// never applying a held set (the card never shows).
#[test]
fn a_set_waits_for_its_device_and_applies_once_the_record_binds_it() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (board(dir.path(), 1), board(dir.path(), 2));
    write(&a, "c1", "milk");
    let unbound = |_: &str| false;
    let got = b.ingest(&call(&a, &b), &unbound, None).unwrap();
    assert_eq!(got[0].1, Took::Waits);
    assert_eq!(cards(&b), json!([]), "held, not applied");
    assert!(call(&a, &b).sets.is_empty(), "counted, so fetched no more");
    assert_eq!(b.rejudge(&unbound).unwrap(), 0, "still unbound, still held");
    assert_eq!(b.rejudge(&|_| true).unwrap(), 1);
    assert_eq!(cards(&b), json!([{ "id": "c1", "title": "milk" }]));
}

/// **A fault holds its mark, and the copy says it is still syncing**: a set
/// this copy can't apply, here one whose title the table's CHECK refuses as
/// SQLite applies it, leaves the device's mark where it was and blocks
/// adopting the member's mark for it; a call taken whole after clears it.
/// The parse leaves a CHECK to SQLite, since one can read the row's other
/// columns as they stand on each copy. Plant: a fault counted in the mark
/// (the set never fetched again).
#[test]
fn a_fault_holds_its_mark_and_is_said_syncing() {
    const CHECKED: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT CHECK (title <> 'gone'));";
    let dir = tempfile::tempdir().unwrap();
    let open = |n: u8| States::new(&dir.path().join(format!("computer-{n}")), key(n)).open("g", "board", Some(CHECKED));
    let (a, b) = (open(1).unwrap(), open(2).unwrap());
    write(&a, "c1", "milk");
    assert!(b.syncing(), "a new copy is syncing until its first full sync");
    let mut sent = call(&a, &b);
    let good = sent.sets[0].clone();
    let gone = |c: &mut Change| c.val = Val::Text("gone".into());
    sent.sets[0] = remade(&good, &key(1), |c| {
        c.iter_mut().filter(|c| c.cid == "title").for_each(gone)
    });
    assert_eq!(took(&b, &sent), [Took::Fault]);
    assert!(b.syncing(), "said syncing while a fault stands");
    assert!(
        b.ask().unwrap().marks.is_empty(),
        "its mark held, the member's not adopted"
    );
    assert_eq!(call(&a, &b).sets, [good], "fetched again");
    assert_eq!(took(&b, &call(&a, &b)), [Took::Taken]);
    assert!(!b.syncing(), "a full call taken whole");
}

/// **A copy's first full sync is kept**: opened again, it isn't said
/// syncing. Plant: the flag kept in memory alone.
#[test]
fn a_copy_s_first_full_sync_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (board(dir.path(), 1), board(dir.path(), 2));
    took(&b, &call(&a, &b));
    assert!(!b.syncing());
    drop(b);
    let again = States::new(&dir.path().join("computer-2"), key(2));
    assert!(!again.open("g", "board", None).unwrap().syncing());
}

/// **A newcomer adopts its source's marks after a full pull, and only
/// then**: a call cut short by the budget adopts none, and a full one does.
/// Plant: marks adopted from a call cut short (the newcomer's mark passes
/// the sets the cut left out).
#[test]
fn a_newcomer_adopts_marks_after_a_full_pull_alone() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, c) = (board(dir.path(), 1), board(dir.path(), 2), board(dir.path(), 3));
    write(&a, "c1", "milk");
    write(&a, "c2", "eggs");
    took(&b, &call(&a, &b));
    let cut = b.missing(&c.ask().unwrap(), 1).unwrap();
    assert_eq!((cut.sets.len(), cut.full), (1, false));
    took(&c, &cut);
    assert_eq!(marks(&c.ask().unwrap()), [1], "nothing adopted past what came");
    took(&c, &call(&b, &c));
    assert_eq!(marks(&c.ask().unwrap()), [2]);
    assert_eq!(cards(&c), cards(&a));
}

/// **Two sets at one site and seq are both taken, said and named**: a
/// database copied beside its original, each writing its own card at the
/// same version. A copy handed both, the second carried on its own as a
/// phone's or a relayed set travels, takes it as a twin and names both in
/// its ask; each original, holding one, is then sent the other though its
/// mark counts the seq, and all three agree. Between copies that each hold
/// one and sync only by full calls on marks, neither is ever sent the
/// other's: the pair is found only where both meet (APP_STATE §4). Plant: a
/// twin passed by as held (the third copy shows one card of the two).
#[test]
fn two_sets_at_one_seq_are_both_taken_and_named() {
    let dir = tempfile::tempdir().unwrap();
    let a = board(dir.path(), 1);
    write(&a, "c1", "milk");
    drop(a);
    let (from, to) = (dir.path().join("computer-1"), dir.path().join("copy"));
    std::fs::create_dir_all(to.join("state/g")).unwrap();
    std::fs::copy(from.join("state/g/board.db"), to.join("state/g/board.db")).unwrap();
    let a = board(dir.path(), 1);
    let copy = States::new(&to, key(1)).open("g", "board", None).unwrap();
    write(&a, "c2", "eggs");
    write(&copy, "c3", "jam");
    let c = board(dir.path(), 3);
    took(&c, &call(&a, &c));
    assert!(
        call(&copy, &c).sets.is_empty(),
        "by marks alone, the copy's is never sent"
    );
    let mut carried = copy.missing(&Ask::default(), BUDGET).unwrap();
    carried.sets.retain(|s| s.head.seq.get() == 2);
    carried.full = false;
    assert_eq!(took(&c, &carried), [Took::Twin]);
    assert_eq!(c.ask().unwrap().twins.len(), 2, "both named in its ask");
    for one in [&a, &copy] {
        let got = took(one, &call(&c, one));
        assert!(got.contains(&Took::Twin), "the other sent, though counted: {got:?}");
        assert_eq!(cards(one), cards(&c));
    }
    assert_eq!(cards(&c).as_array().unwrap().len(), 3);
}

/// **A database made again under its device's key is a new site** (the
/// seat's case, R2): Ana's computer writes three sets, Ben takes them, and
/// her database is deleted and made again. Her new database takes her old
/// sets back as another site's, and her new writes, numbered from 1 at its
/// own site, reach Ben. Plant: the site of the device alone, as
/// `svrngs_core::set::site` without the nonce (her old sets fault, cr-sqlite
/// refusing its own site's rows above its own version, and her new write is
/// passed by at Ben as counted).
#[test]
fn a_database_made_again_under_its_key_reaches_the_group() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (board(dir.path(), 1), board(dir.path(), 2));
    for (id, title) in [("c1", "milk"), ("c2", "eggs"), ("c1", "jam")] {
        write(&a, id, title);
    }
    assert_eq!(took(&b, &call(&a, &b)), [Took::Taken; 3]);
    drop(a);
    std::fs::remove_dir_all(dir.path().join("computer-1/state")).unwrap();
    let again = board(dir.path(), 1);
    assert_eq!(took(&again, &call(&b, &again)), [Took::Taken; 3], "her old sets, back");
    write(&again, "c3", "salt");
    let new = call(&again, &b);
    assert_eq!(new.sets.iter().map(|s| s.head.seq.get()).collect::<Vec<_>>(), [1]);
    assert_eq!(took(&b, &new), [Took::Taken], "her new write reaches Ben");
    assert_eq!(cards(&b), cards(&again));
    assert_eq!(cards(&b).as_array().unwrap().len(), 3);
}
