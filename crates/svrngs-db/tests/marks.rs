//! What a copy counts, and on whose word (svrn-docs
//! `rings/current/APP_STATE.md` §4, "What travels" and the four outcomes; §7,
//! invariants 8, 11 and 12): what the wire says is parsed before anything in
//! it is used, and a set's signature is checked first; and a second set at a
//! place this copy refused is judged.

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
    to.ingest(call, &|_| true)
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
    let at = At {
        group: "g",
        app: "board",
        device: set.device.as_str(),
        nonce: &set.nonce,
        seq: set.seq.get(),
        schema: &set.schema,
    };
    set.sig = Sig::parse(&at.sign(by, &set.changes)).unwrap();
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
    let (h, b, c) = (board(dir.path(), 9), board(dir.path(), 2), board(dir.path(), 3));
    for (id, title) in [("c1", "milk"), ("c2", "eggs"), ("c3", "jam")] {
        write(&h, id, title);
    }
    let mut sets = h.missing(&Ask::default(), BUDGET).unwrap().sets;
    sets[1] = remade(&sets[1], &key(9), |c| c[0].col_version = (1 << 53) + 1);
    assert_eq!(
        took(&b, &alone(sets.clone())),
        [Took::Taken, Took::Refused, Took::Taken]
    );
    took(&c, &call(&b, &c));
    let mut forged = sets[1].clone();
    forged.sig = Sig::parse(&"00".repeat(64)).unwrap();
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
    let device = set.device.as_str();
    for spelled in [
        device.to_uppercase(),
        device[..62].to_owned(),
        format!("{device}00"),
        format!("{}g", &device[..63]),
    ] {
        assert!(hex(&spelled).is_err(), "{spelled} decodes as a device");
    }
    assert_eq!(hex(device).unwrap(), set.device);
    let sig = postcard::to_allocvec(&set.sig.as_str().to_uppercase()).unwrap();
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
