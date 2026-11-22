//! What a copy's ask may carry, and who fills it (svrn-docs
//! `rings/current/APP_STATE.md` §4, "What travels"; §7, invariant 9): a set
//! that waits is never sent on, and is judged again only when the record
//! changes; every key the record binds has an equal share of the room a
//! call's ask may fill beside its sets, and the keys it doesn't one share
//! together; a device past its share is muted at that copy, named in its
//! ask, so no member sends it more, and its sets before stand.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::json;
use svrngs_core::set::At;
use svrngs_db::{Ask, Binds, Call, Db, Device, Head, ROW_MOST, Seq, Set, Sig, SigningKey, States, Took};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);";
const BUDGET: usize = 4 << 20;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn actor(n: u8) -> String {
    svrngs_core::actor_of(&key(n))
}

fn board(dir: &Path, n: u8) -> Arc<Db> {
    let states = States::new(&dir.join(format!("computer-{n}")), key(n));
    states.open("g", "board", Some(SCHEMA)).unwrap()
}

fn write(db: &Db, id: &str, title: &str) {
    db.write(&json!([{ "insert": "cards", "row": { "id": id, "title": title } }]))
        .unwrap();
}

fn call(from: &Db, to: &Db) -> Call {
    from.missing(&to.ask().unwrap(), BUDGET).unwrap()
}

fn alone(sets: Vec<Set>) -> Call {
    Call {
        sets,
        ..Default::default()
    }
}

/// `keys` bound, sharing a room of `rows` shares a key, as `Binds::share`
/// counts them.
fn binds(keys: &[u8], share: usize) -> Binds {
    Binds::new(keys.iter().map(|n| actor(*n)), share * ROW_MOST * (keys.len() + 1))
}

fn took(to: &Db, call: &Call, binds: &Binds) -> Vec<Took> {
    to.ingest(call, binds, None)
        .unwrap()
        .into_iter()
        .map(|(_, t)| t)
        .collect()
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

/// `set`, its changes made by `f`, signed by `by` as `device`'s.
fn signed(set: &Set, by: &SigningKey, f: impl Fn(&mut Vec<Change>)) -> Set {
    let mut changes: Vec<Change> = postcard::from_bytes(&set.changes).unwrap();
    f(&mut changes);
    let mut set = set.clone();
    set.head.device = Device::parse(&svrngs_core::actor_of(by)).unwrap();
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

/// **`ROW_MOST` is the most one row of an ask costs, with its head**:
/// postcard's encoding of the largest site's mark beside its head, and of
/// the largest have, twin and muted device. Plant: one byte fewer.
#[test]
fn row_most_is_the_most_a_row_of_an_ask_costs() {
    let most = Seq::new(i64::MAX as u64).unwrap();
    let head = Head {
        device: Device::parse(&"f".repeat(64)).unwrap(),
        nonce: [0xff; 16],
        seq: most,
        schema: [0xff; 32],
        changes: [0xff; 32],
        sig: Sig::parse(&"f".repeat(128)).unwrap(),
    };
    let site =
        postcard::to_allocvec(&([0xffu8; 16], most)).unwrap().len() + postcard::to_allocvec(&head).unwrap().len();
    assert_eq!(site, ROW_MOST, "a site's mark beside its head");
    let twin = postcard::to_allocvec(&(([0xffu8; 16], most), head.sig.clone()))
        .unwrap()
        .len();
    let muted = postcard::to_allocvec(&head.device).unwrap().len();
    for (what, n) in [("a twin", twin), ("a muted device", muted)] {
        assert!(n <= ROW_MOST, "{what}: {n}");
    }
}

/// **A set that waits is never sent on**: held here while the record
/// doesn't bind its device, or under a schema this copy doesn't hold, it
/// goes to no member that asks. Its own device's copy, or one that took
/// it, sends it. Plant: `missing` sending every set with bytes (the one
/// waiting for its schema sent on; one waiting for its key has no head,
/// which `missing` joins).
#[test]
fn a_waiting_set_is_never_sent_on() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, c) = (board(dir.path(), 1), board(dir.path(), 2), board(dir.path(), 3));
    write(&a, "c1", "milk");
    assert_eq!(took(&b, &call(&a, &b), &binds(&[], 10)), [Took::Waits]);
    assert!(call(&b, &c).sets.is_empty(), "not sent on as it waits");
    assert_eq!(
        took(&c, &call(&a, &c), &binds(&[1], 10)),
        [Took::Taken],
        "its device's copy sends it"
    );

    // One under a schema Dee's copy doesn't hold, its device bound.
    const WIDER: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, note TEXT);";
    let wider = States::new(&dir.path().join("computer-5"), key(5));
    let wider = wider.open("g", "board", Some(WIDER)).unwrap();
    write(&wider, "c2", "eggs");
    let d = board(dir.path(), 4);
    assert_eq!(took(&d, &call(&wider, &d), &binds(&[4, 5], 10)), [Took::Waits]);
    assert!(call(&d, &c).sets.is_empty(), "not sent on as it waits for its schema");
}

/// **A set waiting for its key is judged again only when the record
/// changes**: judged once by the keys bound now, not again while they
/// stay the same, and applied once the record binds its key. Plant: the
/// record's digest unread (judged again by the same keys).
#[test]
fn a_waiting_set_is_judged_again_only_when_the_record_changes() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (board(dir.path(), 1), board(dir.path(), 2));
    write(&a, "c1", "milk");
    let unbound = binds(&[2], 10);
    assert_eq!(took(&b, &call(&a, &b), &unbound), [Took::Waits]);
    assert_eq!(b.rejudge(&unbound).unwrap(), Some(0), "judged by the keys bound now");
    assert_eq!(b.rejudge(&unbound).unwrap(), None, "not again while they stay the same");
    assert_eq!(
        b.rejudge(&binds(&[1, 2], 10)).unwrap(),
        Some(1),
        "applied once its key is bound"
    );
}

/// **The keys the record doesn't bind share one share of the room**: sets
/// signed by keys no record binds wait, each in that one share, and past it
/// are dropped as if they never came, counted nowhere, naming no device.
/// Plant: that share unchecked (every one waits).
#[test]
fn keys_the_record_doesn_t_bind_share_one_share() {
    let dir = tempfile::tempdir().unwrap();
    let (h, b) = (board(dir.path(), 9), board(dir.path(), 2));
    write(&h, "c1", "milk");
    let base = h.missing(&Ask::default(), BUDGET).unwrap().sets.remove(0);
    let throwaway = |i: u8| SigningKey::from_bytes(&[100 + i; 32]);
    let sets: Vec<Set> = (0..10).map(|i| signed(&base, &throwaway(i), |_| {})).collect();
    let (share, room) = (6, binds(&[2, 9], 6));
    let got = took(&b, &alone(sets), &room);
    assert_eq!(got[..share], [Took::Waits; 6]);
    assert_eq!(got[share..], [Took::Over; 4]);
    let ask = b.ask().unwrap();
    assert!(ask.muted.is_empty(), "no device named");
    assert_eq!(ask.marks.len(), share, "the dropped ones counted nowhere");
}

/// **A device past its share is muted at that copy, and its sets before
/// stand**: one device's sets at a new site each, and then at one place,
/// each a row of Ben's ask, are taken until its share is full; the next is
/// dropped, the device named muted in Ben's ask, so a member keeping more of
/// its sets sends him none, and his ask stays within the room. Plant: the
/// share unchecked (every set taken, the ask past its room).
#[test]
fn a_device_past_its_share_is_muted_and_its_sets_before_stand() {
    let dir = tempfile::tempdir().unwrap();
    let b = board(dir.path(), 2);
    let room = binds(&[2, 9], 6);
    // Computer 9's databases, each made again under its key: a site each.
    let sites: Vec<Set> = (0..7)
        .map(|i| {
            let h = States::new(&dir.path().join(format!("h{i}")), key(9))
                .open("g", "board", Some(SCHEMA))
                .unwrap();
            write(&h, &format!("c{i}"), "milk");
            h.missing(&Ask::default(), BUDGET).unwrap().sets.remove(0)
        })
        .collect();
    let got: Vec<Took> = sites
        .iter()
        .flat_map(|s| took(&b, &alone(vec![s.clone()]), &room))
        .collect();
    assert_eq!(got, [[Took::Taken; 5].as_slice(), &[Took::Over; 2]].concat());
    let ask = b.ask().unwrap();
    let nine = Device::parse(&actor(9)).unwrap();
    assert!(ask.muted.contains(&nine), "named muted in the ask");
    let rows = b.read("SELECT count(*) AS n FROM cards", &[]).unwrap();
    assert_eq!(rows["rows"][0]["n"], 5, "its sets before stand");
    let c = board(dir.path(), 3);
    for s in &sites {
        took(&c, &alone(vec![s.clone()]), &binds(&[3, 9], 20));
    }
    assert!(
        call(&c, &b).sets.is_empty(),
        "a member keeping more of them sends Ben none"
    );
    assert!(
        postcard::to_allocvec(&ask).unwrap().len() <= 6 * ROW_MOST * 3,
        "the ask within its room"
    );

    // Twins at one place: a row each, and two for the first.
    let d = board(dir.path(), 4);
    let base = sites[0].clone();
    let twins: Vec<Set> = (0..5)
        .map(|i| {
            signed(&base, &key(9), |c| {
                c.iter_mut()
                    .filter(|c| c.cid == "title")
                    .for_each(|c| c.val = Val::Text(format!("t{i}")))
            })
        })
        .collect();
    let got = took(&d, &alone(twins), &binds(&[4, 9], 6));
    assert_eq!(got, [Took::Taken, Took::Twin, Took::Twin, Took::Twin, Took::Over]);
}
