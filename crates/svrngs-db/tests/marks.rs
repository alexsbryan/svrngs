//! What a copy counts, and on whose word (svrn-docs
//! `rings/current/APP_STATE.md` §4, "What travels" and the four outcomes; §7,
//! invariants 8, 11 and 12): what the wire says is parsed before anything in
//! it is used, and a set's signature is checked first; a second set at a
//! place this copy refused is judged; a mark moves only past a set its
//! device signed; and a copy is synced only by a copy that is.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use svrngs_core::set::At;
use svrngs_db::{Ask, Call, Db, Device, Seq, Set, Sig, SigningKey, States, Took};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);";
const BUDGET: usize = 4 << 20;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn board(dir: &Path, n: u8) -> Arc<Db> {
    let states = States::new(&dir.join(format!("computer-{n}")), key(n));
    states.open("g", "board", Some(SCHEMA)).unwrap()
}

/// Computer `n`'s board made as the root, as the computer that carries the
/// app makes it, synced from the start.
fn root(dir: &Path, n: u8) -> Arc<Db> {
    let states = States::new(&dir.join(format!("computer-{n}")), key(n));
    states.carry("g", "board", SCHEMA).unwrap()
}

fn write(db: &Db, id: &str, title: &str) {
    db.write(&json!([{ "insert": "cards", "row": { "id": id, "title": title } }]))
        .unwrap();
}

fn cards(db: &Db) -> Value {
    db.read("SELECT id, title FROM cards ORDER BY id", &[]).unwrap()["rows"].clone()
}

fn call(from: &Db, to: &Db) -> Call {
    from.missing(&to.ask().unwrap(), BUDGET).unwrap()
}

fn took(to: &Db, call: &Call) -> Vec<Took> {
    to.ingest(call, &|_| true, None)
        .unwrap()
        .into_iter()
        .map(|(_, t)| t)
        .collect()
}

fn alone(sets: Vec<Set>) -> Call {
    Call {
        sets,
        ..Default::default()
    }
}

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

fn remade(set: &Set, by: &SigningKey, f: impl Fn(&mut Vec<Change>)) -> Set {
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
    set.head.sig = Sig::parse(&at.sign(by, &set.head.changes)).unwrap();
    set
}

/// **A set's signature is checked before anything in it is used**: H's
/// second set is refused for good at Ben, so Ben keeps none of it, and Cy,
/// pulling from Ben in full, counts that place on Ben's word without ever
/// holding a set there. A set at that place signed by nobody is then
/// dropped at Cy as not its device's, where it was passed by unread.
/// Plant: the signature checked after the place is read (passed by, `Held`).
#[test]
fn a_set_is_judged_by_its_signature_first() {
    let dir = tempfile::tempdir().unwrap();
    let (h, b, c) = (root(dir.path(), 9), board(dir.path(), 2), board(dir.path(), 3));
    for (id, title) in [("c1", "milk"), ("c2", "eggs"), ("c3", "jam")] {
        write(&h, id, title);
    }
    let mut sets = h.missing(&Ask::default(), BUDGET).unwrap().sets;
    sets[1] = remade(&sets[1], &key(9), |c| c[0].col_version = (1 << 53) + 1);
    let mut handed = alone(sets.clone());
    (handed.full, handed.synced) = (true, true);
    assert_eq!(took(&b, &handed), [Took::Taken, Took::Refused, Took::Taken]);
    took(&c, &call(&b, &c));
    let mut forged = sets[1].clone();
    forged.head.sig = Sig::parse(&"00".repeat(64)).unwrap();
    assert_eq!(took(&c, &alone(vec![forged])), [Took::NotItsDevice]);
}

/// **What the wire carries is parsed as it is read** (RUST_ARCH, "parse,
/// don't validate"): a seq, and so a set's place or a mark, from 1 to
/// `i64::MAX`, what SQLite holds; a device and a signature in lower-case hex
/// of their length, as the record spells keys. Anything else doesn't decode,
/// so no call naming it does, and nothing of it is judged, counted or
/// adopted. Plants: `Seq`'s bound dropped (a seq past `i64::MAX` decodes);
/// `Hex`'s case check dropped (an upper-case device decodes).
#[test]
fn what_the_wire_carries_is_parsed_as_it_is_read() {
    let dir = tempfile::tempdir().unwrap();
    let a = board(dir.path(), 1);
    write(&a, "c1", "milk");
    let set = a.missing(&Ask::default(), BUDGET).unwrap().sets.remove(0);
    let seq = |n: u64| postcard::from_bytes::<Seq>(&postcard::to_allocvec(&n).unwrap());
    for n in [0, i64::MAX as u64 + 1, u64::MAX] {
        assert!(seq(n).is_err(), "{n} decodes as a seq");
    }
    for n in [1, 2, i64::MAX as u64] {
        assert_eq!(seq(n).unwrap().get(), n);
    }
    let hex = |text: &str| postcard::from_bytes::<Device>(&postcard::to_allocvec(text).unwrap());
    let device = set.head.device.as_str();
    for spelled in [
        device.to_uppercase(),
        device[..62].to_owned(),
        format!("{device}00"),
        format!("{}g", &device[..63]),
    ] {
        assert!(hex(&spelled).is_err(), "{spelled} decodes as a device");
    }
    assert_eq!(hex(device).unwrap(), set.head.device);
    let sig = postcard::to_allocvec(&set.head.sig.as_str().to_uppercase()).unwrap();
    assert!(
        postcard::from_bytes::<Sig>(&sig).is_err(),
        "an upper-case signature decodes"
    );
    let whole = postcard::to_allocvec(&alone(vec![set.clone()])).unwrap();
    assert_eq!(
        postcard::from_bytes::<Call>(&whole).unwrap().sets,
        [set],
        "a call that fits decodes"
    );
}

/// **A second set at a place this copy refused is judged, and taken as a
/// twin**: two sets signed at one seq, one past the 2^53 bound. Ben, handed
/// the refused one first, takes the other as a twin; Cy, handed them the
/// other way, takes the same one; both name the place in their asks, and
/// they agree. Plant: a refused set not kept (Ben passes the other by,
/// `Held`, and shows no card).
#[test]
fn a_second_set_at_a_place_refused_here_is_taken_as_a_twin() {
    let dir = tempfile::tempdir().unwrap();
    let h = board(dir.path(), 9);
    write(&h, "c1", "milk");
    let y = h.missing(&Ask::default(), BUDGET).unwrap().sets.remove(0);
    let x = remade(&y, &key(9), |c| {
        c.iter_mut()
            .filter(|c| c.cid == "title")
            .for_each(|c| c.col_version = (1 << 53) + 1)
    });
    let (b, c) = (board(dir.path(), 2), board(dir.path(), 3));
    assert_eq!(took(&b, &alone(vec![x.clone()])), [Took::Refused]);
    assert_eq!(took(&b, &alone(vec![y.clone()])), [Took::Twin], "judged, not passed by");
    assert_eq!(took(&c, &alone(vec![y])), [Took::Taken]);
    assert_eq!(took(&c, &alone(vec![x])), [Took::Refused]);
    assert_eq!(cards(&b), cards(&c), "the copies agree");
    assert_eq!(cards(&b), json!([{ "id": "c1", "title": "milk" }]));
}

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

/// The one site `db` has written at, its own.
fn own(db: &Db) -> svrngs_core::set::Site {
    *db.ask().unwrap().marks.keys().next().unwrap()
}

/// **A full call's marks pass no set their device hasn't signed, and never
/// this copy's own site**: a member's call that claims a mark far past
/// every set Ana's device has signed, with no head, moves no copy's mark
/// for her, her own copy's included, so her next set reaches Ben, and Cy
/// through Ben. Plant: the claimed mark adopted with no head (Ben's mark at
/// 1,000,000,000).
#[test]
fn a_full_call_s_marks_pass_no_set_their_device_has_not_signed() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, c) = (board(dir.path(), 1), board(dir.path(), 2), board(dir.path(), 3));
    write(&a, "c1", "milk");
    assert_eq!(took(&b, &call(&a, &b)), [Took::Taken]);
    let a_site = own(&a);
    let mut claim = Call {
        full: true,
        ..Default::default()
    };
    claim.ask.marks.insert(a_site, seq(1_000_000_000));
    b.ingest(&claim, &|_| true, None).unwrap();
    assert_eq!(
        b.ask().unwrap().marks[&a_site],
        seq(1),
        "no mark past a set its device signed"
    );
    a.ingest(&claim, &|_| true, None).unwrap();
    assert_eq!(
        a.ask().unwrap().marks[&a_site],
        seq(1),
        "nor another's word for this copy's own site"
    );
    write(&a, "c2", "eggs");
    assert_eq!(took(&b, &call(&a, &b)), [Took::Taken], "Ana's next set reaches Ben");
    took(&c, &call(&b, &c));
    assert_eq!(cards(&c), cards(&a), "and Cy, through Ben");
}

/// **A mark is adopted up to the head its device signed, and no further**:
/// Ben holds Ana's two sets; a new copy pulled from him in full, his call
/// claiming her mark far past them, adopts it to 2, her head's seq. A head
/// whose signature isn't hers, or one of another site, adopts nothing.
/// Plant: the call's head taken unchecked (the forged head's seq adopted).
#[test]
fn a_mark_is_adopted_up_to_the_head_its_device_signed() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (root(dir.path(), 1), board(dir.path(), 2));
    write(&a, "c1", "milk");
    write(&a, "c2", "eggs");
    took(&b, &call(&a, &b));
    let a_site = own(&a);
    let c = board(dir.path(), 3);
    let mut sent = call(&b, &c);
    assert_eq!(
        sent.heads.len(),
        1,
        "a head for the one site whose mark Ben has past Cy's"
    );
    sent.ask.marks.insert(a_site, seq(1_000_000_000));
    took(&c, &sent);
    assert_eq!(c.ask().unwrap().marks[&a_site], seq(2), "up to her head");

    let head = sent.heads[0].clone();
    let claimed = |heads: Vec<svrngs_db::Head>| {
        let mut claim = Call {
            full: true,
            synced: true,
            heads,
            ..Default::default()
        };
        claim.ask.marks.insert(a_site, seq(2));
        claim
    };
    let mut forged = head.clone();
    forged.sig = Sig::parse(&"00".repeat(64)).unwrap();
    let mut elsewhere = head.clone();
    elsewhere.nonce = [9; 16];
    for (what, h) in [("a head not hers", forged), ("a head of another site", elsewhere)] {
        let d = board(&dir.path().join(what), 4);
        d.ingest(&claimed(vec![h]), &|_| true, None).unwrap();
        assert!(d.ask().unwrap().marks.is_empty(), "{what} adopted");
    }
    let d = board(&dir.path().join("hers"), 4);
    d.ingest(&claimed(vec![head]), &|_| true, None).unwrap();
    assert_eq!(
        d.ask().unwrap().marks[&a_site],
        seq(2),
        "her own head, adopted by a new copy"
    );
}

/// **An established copy adopts a mark only on its site's own computer's
/// word**: Ana writes three sets, and Ben has all three; Cy has synced, and
/// has the first. Ben's full call, claiming her mark at 3 with her own head
/// at 3, moves Cy's mark not at all, so Cy fetches the two he lacks. Her
/// own computer's call does: it vouches for its own sets. A new copy
/// adopts Ben's word, as a newcomer trusts its source (APP_STATE §4).
/// Plant: any caller trusted (Cy's mark at 3, sets 2 and 3 never fetched).
#[test]
fn an_established_copy_adopts_a_mark_only_on_its_site_s_own_computer_s_word() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, c) = (root(dir.path(), 1), board(dir.path(), 2), board(dir.path(), 3));
    write(&a, "c1", "milk");
    took(&c, &call(&a, &c));
    assert!(!c.syncing(), "Cy has synced");
    write(&a, "c2", "eggs");
    write(&a, "c3", "jam");
    took(&b, &call(&a, &b));
    let a_site = own(&a);
    let devices = |n: u8| Some(Device::parse(&svrngs_core::actor_of(&key(n))).unwrap());
    let mut word = call(&b, &c);
    word.sets.clear();
    assert_eq!((word.full, word.heads.len()), (true, 1));
    c.ingest(&word, &|_| true, devices(2).as_ref()).unwrap();
    assert_eq!(
        c.ask().unwrap().marks[&a_site],
        seq(1),
        "Ben's word moves no established copy's mark"
    );
    assert_eq!(call(&a, &c).sets.len(), 2, "so Cy fetches what he lacks");
    c.ingest(&word, &|_| true, devices(1).as_ref()).unwrap();
    assert_eq!(c.ask().unwrap().marks[&a_site], seq(3), "her own computer's word does");
}

/// **A copy never adopts a mark for its own site**, though the word is its
/// own key's and the head is signed: Ana's database copied beside its
/// original writes a third set, and its call, claiming her site's mark at 3
/// with that set's head, moves her copy's mark not at all, so the set
/// stays one she is sent. Plant: this copy's own site not left out (her
/// mark at 3).
#[test]
fn a_copy_never_adopts_a_mark_for_its_own_site() {
    let dir = tempfile::tempdir().unwrap();
    let a = board(dir.path(), 1);
    write(&a, "c1", "milk");
    write(&a, "c2", "eggs");
    drop(a);
    let (from, to) = (dir.path().join("computer-1"), dir.path().join("copy"));
    std::fs::create_dir_all(to.join("state/g")).unwrap();
    std::fs::copy(from.join("state/g/board.db"), to.join("state/g/board.db")).unwrap();
    let a = board(dir.path(), 1);
    let copy = States::new(&to, key(1)).open("g", "board", None).unwrap();
    write(&copy, "c3", "jam");
    let mut word = call(&copy, &a);
    word.sets.clear();
    assert_eq!(word.heads.len(), 1, "the copy's head at 3");
    let ana = Device::parse(&svrngs_core::actor_of(&key(1))).unwrap();
    a.ingest(&word, &|_| true, Some(&ana)).unwrap();
    assert_eq!(a.ask().unwrap().marks[&own(&a)], seq(2), "her own site's mark, unmoved");
    assert_eq!(call(&copy, &a).sets.len(), 1, "the set at 3, still sent her");
}

/// **A copy is synced only by a copy that is** (invariant 8): Ana holds the
/// only card, and her computer is away. Ben's and Cy's new copies each
/// answer the other in full with nothing, and both are still said syncing.
/// Plant: the caller's synced flag unread (both said synced, an empty board
/// shown as the group's).
#[test]
fn a_full_call_from_a_copy_never_synced_makes_no_copy_synced() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, c) = (root(dir.path(), 1), board(dir.path(), 2), board(dir.path(), 3));
    write(&a, "c1", "milk");
    let answer = c.missing(&b.ask().unwrap(), BUDGET).unwrap();
    assert!(answer.full && answer.sets.is_empty() && !answer.synced);
    took(&b, &answer);
    took(&c, &b.missing(&c.ask().unwrap(), BUDGET).unwrap());
    assert!(b.syncing() && c.syncing(), "neither has synced with a copy that has");
}

/// **The copy made as its app is carried is synced from the start; one
/// made again on that computer is not**: Ana's root says nothing syncing,
/// and Ben's copy syncs from it. Her database deleted and made again, as a
/// page opens it, is syncing until a synced copy, Ben's, answers it in
/// full, and then it holds her card again. Plant: a root made unsynced (Ana
/// said syncing, and so Ben, for good).
#[test]
fn the_root_is_synced_from_the_start_and_a_copy_made_again_is_not() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (root(dir.path(), 1), board(dir.path(), 2));
    assert!(!a.syncing(), "the root is synced from the start");
    write(&a, "c1", "milk");
    took(&b, &call(&a, &b));
    assert!(!b.syncing(), "a full call from the root syncs Ben");
    drop(a);
    std::fs::remove_dir_all(dir.path().join("computer-1/state")).unwrap();
    let again = board(dir.path(), 1);
    assert!(again.syncing(), "made again, it is no root");
    took(&again, &call(&b, &again));
    assert!(!again.syncing(), "synced by Ben's full call");
    assert_eq!(cards(&again), json!([{ "id": "c1", "title": "milk" }]));
}
