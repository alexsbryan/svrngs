//! What a copy counts, and on whose word (svrn-docs
//! `rings/current/APP_STATE.md` §4, "What travels" and the four outcomes; §7,
//! invariants 8, 11 and 12): what the wire says is parsed before anything in
//! it is used, and a set's signature is checked first; a second set at a
//! place this copy refused is judged; a mark moves only up to what this
//! copy has counted, on a word it trusts; and a copy is synced only by a
//! copy that is.

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

/// Every key these tests sign with, bound, in the room a request has
/// beside its sets (svrngs-node `peer_wire.rs`, `ASK_ROOM_BYTES`).
fn bound() -> svrngs_db::Binds {
    let keys = (0..=255).map(|n| svrngs_core::actor_of(&SigningKey::from_bytes(&[n; 32])));
    svrngs_db::Binds::new(keys, 4 << 20)
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
    to.ingest(call, &bound(), None)
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

/// **A full call's marks pass nothing this copy hasn't counted, on any
/// word**: Ben holds Ana's one set. Her own computer's full call, which
/// vouches for her site, claims her mark at 1,000,000,000, and so does Ben's
/// call into Dee's new copy, which trusts its source. Ben's mark stays at 1,
/// and so does Dee's, so Ana's next set, carried to Cy alone, reaches both
/// through Cy, and the copies agree. Plant: the cap dropped (Ben's mark at
/// 1,000,000,000, her next set passed by).
#[test]
fn a_full_call_s_marks_pass_nothing_this_copy_has_not_counted() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, c) = (root(dir.path(), 1), board(dir.path(), 2), board(dir.path(), 3));
    write(&a, "c1", "milk");
    took(&b, &call(&a, &b));
    let a_site = own(&a);
    let ana = Device::parse(&svrngs_core::actor_of(&key(1))).unwrap();
    let mut claim = call(&a, &b);
    claim.ask.marks.insert(a_site, seq(1_000_000_000));
    b.ingest(&claim, &bound(), Some(&ana)).unwrap();
    assert_eq!(
        b.ask().unwrap().marks[&a_site],
        seq(1),
        "not on her own computer's word"
    );
    let d = board(dir.path(), 4);
    let mut claim = call(&b, &d);
    claim.ask.marks.insert(a_site, seq(1_000_000_000));
    took(&d, &claim);
    assert_eq!(d.ask().unwrap().marks[&a_site], seq(1), "nor on a newcomer's source's");
    write(&a, "c2", "eggs");
    took(&c, &call(&a, &c));
    for to in [&b, &d] {
        assert_eq!(took(to, &call(&c, to)), [Took::Taken], "her next set, through Cy");
        assert_eq!(cards(to), cards(&c));
    }
}

/// **A newcomer adopts its source's marks up to what it has counted, its
/// gaps filled**: Ana's first set, its one value outranked by her third, is
/// pruned where it was written, so Ben is sent two of her three, and so is
/// Cy's new copy, pulled from Ben in full, his call claiming her mark far
/// past them. Cy's mark is 3, the highest he counted, and his ask names no
/// gap. Plants: a newcomer's source not trusted (her two sets counted above
/// a gap, and no mark); the cap dropped (Cy's mark at 1,000,000,000).
#[test]
fn a_newcomer_adopts_its_source_s_marks_up_to_what_it_has_counted() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, c) = (root(dir.path(), 1), board(dir.path(), 2), board(dir.path(), 3));
    for (id, title) in [("c1", "milk"), ("c2", "eggs"), ("c1", "jam")] {
        write(&a, id, title);
    }
    assert_eq!(took(&b, &call(&a, &b)), [Took::Taken; 2]);
    let a_site = own(&a);
    let mut sent = call(&b, &c);
    sent.ask.marks.insert(a_site, seq(1_000_000_000));
    assert_eq!(took(&c, &sent), [Took::Taken; 2]);
    let ask = c.ask().unwrap();
    assert_eq!(ask.marks.get(&a_site), Some(&seq(3)), "up to what Cy counted");
    assert!(ask.haves.is_empty(), "no gap left");
    assert_eq!(cards(&c), cards(&a));
}

/// **An established copy adopts a mark only on its site's own computer's
/// word**: Cy has synced, and has Ana's first set; her second, its one
/// value outranked by her third, is pruned where it was written. Ben's full
/// call sends Cy her third and claims her mark at 3, and Cy counts the third
/// above a gap his ask names, as Ben's word moves no established copy's
/// mark. Her own computer's call closes the gap: it vouches for its own
/// sets. Plant: any caller trusted (Cy's mark at 3 on Ben's word).
#[test]
fn an_established_copy_adopts_a_mark_only_on_its_site_s_own_computer_s_word() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, c) = (root(dir.path(), 1), board(dir.path(), 2), board(dir.path(), 3));
    write(&a, "c1", "milk");
    took(&c, &call(&a, &c));
    assert!(!c.syncing(), "Cy has synced");
    write(&a, "c2", "eggs");
    write(&a, "c2", "jam");
    assert_eq!(took(&b, &call(&a, &b)), [Took::Taken; 2]);
    let a_site = own(&a);
    let devices = |n: u8| Some(Device::parse(&svrngs_core::actor_of(&key(n))).unwrap());
    let word = c.ingest(&call(&b, &c), &bound(), devices(2).as_ref()).unwrap();
    assert_eq!(word.iter().map(|(_, t)| *t).collect::<Vec<_>>(), [Took::Taken]);
    let ask = c.ask().unwrap();
    assert_eq!(
        (ask.marks[&a_site], ask.haves.len()),
        (seq(1), 1),
        "Ben's word moves no established copy's mark"
    );
    c.ingest(&call(&a, &c), &bound(), devices(1).as_ref()).unwrap();
    let ask = c.ask().unwrap();
    assert_eq!(
        (ask.marks[&a_site], ask.haves.len()),
        (seq(3), 0),
        "her own computer's word does"
    );
    assert_eq!(cards(&c), cards(&a));
}

/// **A copy never adopts a mark for its own site**, though the word is its
/// own key's: Ana's database copied beside its original writes a third and
/// a fourth set. Her copy, sent the fourth past the 2^53 bound, refuses it
/// for good and counts it above a gap. The copy's full call, claiming her
/// site's mark at 4, moves her mark not at all. Plant: this copy's own site
/// not left out (her mark at 4).
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
    write(&copy, "c4", "salt");
    let mut word = call(&copy, &a);
    word.sets.clear();
    let past = remade(&call(&copy, &a).sets[1], &key(1), |c| {
        c.iter_mut().for_each(|c| c.col_version = (1 << 53) + 1)
    });
    assert_eq!(took(&a, &alone(vec![past])), [Took::Refused]);
    let ana = Device::parse(&svrngs_core::actor_of(&key(1))).unwrap();
    a.ingest(&word, &bound(), Some(&ana)).unwrap();
    assert_eq!(a.ask().unwrap().marks[&own(&a)], seq(2), "her own site's mark, unmoved");
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
