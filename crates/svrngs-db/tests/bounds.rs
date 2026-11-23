//! What a copy's ask may carry, and who fills it (svrn-docs
//! `rings/current/APP_STATE.md` §4, "What travels"; §7, invariant 9): a set
//! that waits for its schema is never sent on; a key the record doesn't bind
//! stores nothing and adds no row; every key the record binds has an equal
//! share of the room a call's ask may fill beside its sets; a device past
//! its share is muted at that copy, named in its ask, so no member sends it
//! more, and its sets before stand.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::json;
use svrngs_core::set::At;
use svrngs_db::{Ask, Binds, Call, Db, Device, ROW_MOST, SIZE_MOST, Seq, Set, Sig, SigningKey, States, Took};

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
    db.write(
        &json!([{ "insert": "cards", "row": { "id": id, "title": title } }]),
        &binds(&[], 0),
    )
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
    Binds::new(
        keys.iter().map(|n| (actor(*n), actor(*n), false)),
        share * ROW_MOST * keys.len(),
    )
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

/// **`ROW_MOST` is the most one row of an ask costs**: postcard's encoding
/// of the largest twin, a place and its signature; and of the largest
/// site's mark or have, and muted device. Plant: one byte fewer.
#[test]
fn row_most_is_the_most_a_row_of_an_ask_costs() {
    let most = Seq::new(i64::MAX as u64).unwrap();
    let sig = Sig::parse(&"f".repeat(128)).unwrap();
    let twin = postcard::to_allocvec(&(([0xffu8; 16], most), sig)).unwrap().len();
    assert_eq!(twin, ROW_MOST, "a twin's place and signature");
    let mark = postcard::to_allocvec(&([0xffu8; 16], most)).unwrap().len();
    let muted = postcard::to_allocvec(&Device::parse(&"f".repeat(64)).unwrap())
        .unwrap()
        .len();
    for (what, n) in [("a site's mark or a have", mark), ("a muted device", muted)] {
        assert!(n <= ROW_MOST, "{what}: {n}");
    }
}

/// **A set waiting for its schema is never sent on**: one naming a column
/// this copy lacks is dropped here, kept nowhere, so it goes to no member
/// that asks; its own device's copy, or one that took it, sends it. Plant:
/// the set refused for good (counted, its place and signature kept).
#[test]
fn a_set_waiting_for_its_schema_is_never_sent_on() {
    const WIDER: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, note TEXT);";
    let dir = tempfile::tempdir().unwrap();
    let wider = States::new(&dir.path().join("computer-5"), key(5));
    let wider = wider.open("g", "board", Some(WIDER)).unwrap();
    let noted = json!([{ "insert": "cards", "row": { "id": "c2", "title": "eggs", "note": "two" } }]);
    wider.write(&noted, &binds(&[], 0)).unwrap();
    let (c, d) = (board(dir.path(), 3), board(dir.path(), 4));
    assert_eq!(took(&d, &call(&wider, &d), &binds(&[3, 4, 5], 10)), [Took::NotYet]);
    assert!(d.ask().unwrap().marks.is_empty(), "counted as it waits");
    assert!(call(&d, &c).sets.is_empty(), "not sent on as it waits for its schema");
}

/// **A key the record doesn't bind stores nothing and adds no row to the
/// ask**: sets signed by ten keys no record binds are each dropped as not
/// bound yet, kept nowhere, counted nowhere and naming no device, however
/// many come. Plant: such a set counted (a site's mark for each in Ben's
/// ask).
#[test]
fn a_key_the_record_doesn_t_bind_stores_nothing_and_adds_no_row_to_the_ask() {
    let dir = tempfile::tempdir().unwrap();
    let (h, b) = (board(dir.path(), 9), board(dir.path(), 2));
    write(&h, "c1", "milk");
    let base = h.missing(&Ask::default(), BUDGET).unwrap().sets.remove(0);
    let throwaway = |i: u8| SigningKey::from_bytes(&[100 + i; 32]);
    let sets: Vec<Set> = (0..10).map(|i| signed(&base, &throwaway(i), |_| {})).collect();
    assert_eq!(took(&b, &alone(sets), &binds(&[2, 9], 6)), [Took::NotYet; 10]);
    assert_eq!(b.ask().unwrap(), Ask::default(), "no row of the ask");
    let kept = b.missing(&Ask::default(), BUDGET).unwrap().sets;
    assert!(kept.is_empty(), "nothing kept");
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
        postcard::to_allocvec(&ask).unwrap().len() <= 6 * ROW_MOST * 2,
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

/// **A file made again counts its retired site's sets in its device's
/// share** (the operator's review of `41d94397`): a copy made again names
/// every set kept at the site it retired in its ask, as twins
/// (`sync.rs`, `ask`), so those rows count against its device's share of
/// the room as any other. Ana's copy keeps three sets at the site it
/// retires; a set of her key at another site, with a share of three rows,
/// is then past her share. Plant, watched failing: the retired site's sets
/// left out of the count (the set taken, the ask past its room).
#[test]
fn a_file_made_again_counts_its_retired_sites_sets_in_its_share() {
    let dir = tempfile::tempdir().unwrap();
    let states = States::new(&dir.path().join("computer-1"), key(1));
    let ana = states.open("g", "board", Some(SCHEMA)).unwrap();
    for (id, title) in [("c1", "milk"), ("c2", "eggs"), ("c3", "jam")] {
        write(&ana, id, title);
    }
    states.remake(&ana).unwrap();
    assert_eq!(ana.ask().unwrap().twins.len(), 3, "the retired site's sets, named");
    let other = States::new(&dir.path().join("other"), key(1));
    let other = other.open("g", "board", Some(SCHEMA)).unwrap();
    write(&other, "c4", "tea");
    assert_eq!(took(&ana, &call(&other, &ana), &binds(&[1], 3)), [Took::Over]);
}

/// Every test key bound, 256 of them, so each one's share of an app's
/// database is `SIZE_MOST / 256`.
fn all() -> Binds {
    Binds::new((0..=255).map(|n| (actor(n), actor(n), false)), 4 << 20)
}

/// **One device fills no more than its share of an app's database**
/// (APP_STATE §7, invariant 9; the seat's decision 3 for slice E): with 256
/// keys bound, each one's share is `SIZE_MOST / 256`. Bo's copy, binding his
/// key alone, keeps five cards of about 60 KiB each. Ana's copy takes his
/// first four; his fifth is past his share, dropped and his device muted,
/// his sets before standing, and Cy's card is still taken. Ana's own fifth
/// card is refused at the write, saying so, as every copy would drop it.
/// Plant, watched failing: the size left out of a set's room (Bo's fifth
/// taken, Ana's fifth written).
#[test]
fn one_device_fills_no_more_than_its_share_of_the_database() {
    let dir = tempfile::tempdir().unwrap();
    let (ana, bo, cy) = (board(dir.path(), 1), board(dir.path(), 2), board(dir.path(), 3));
    let big = |n: usize| "x".repeat(60 << 10) + &n.to_string();
    const { assert!(5 * (60 << 10) > SIZE_MOST / 256 && 4 * (61 << 10) < SIZE_MOST / 256) };
    for n in 0..5 {
        write(&bo, &format!("b{n}"), &big(n));
    }
    write(&cy, "c1", "tea");
    let took = took(&ana, &call(&bo, &ana), &all());
    assert_eq!(took, [Took::Taken, Took::Taken, Took::Taken, Took::Taken, Took::Over]);
    assert_eq!(taken_from(&ana, &cy), [Took::Taken], "Cy's card, still taken");
    for n in 0..4 {
        let card = json!([{ "insert": "cards", "row": { "id": format!("a{n}"), "title": big(n) } }]);
        ana.write(&card, &all()).unwrap();
    }
    let fifth = json!([{ "insert": "cards", "row": { "id": "a4", "title": big(4) } }]);
    let refused = ana.write(&fifth, &all()).unwrap_err();
    assert_eq!(refused, svrngs_db::Refused::Full(SIZE_MOST >> 20));
}

fn taken_from(to: &Db, from: &Db) -> Vec<Took> {
    took(to, &call(from, to), &all())
}

/// **A device past its share at a copy comes back once what it holds there
/// falls** (svrn-docs `app-state-diligence/invariants-2026-11-22/` §2.1,
/// finding (b)): Wes writes a card, which Eve's copy takes, then a card
/// twice, so its first set is pruned at his own copy, then twenty more. Ted's
/// copy, with a roomier share, takes them all. Eve's, established, takes them through Ted's, every
/// one above the gap that prune made, until Wes is past his share of her
/// ask's rows: the rest are dropped, and her ask names him over. Wes's own
/// full call then sends none of his sets, as her ask names him, and vouches
/// for his mark, so the gap goes and his rows there fall. His next call
/// brings the rest, and his next card is taken: Eve's board is his. Plant,
/// watched failing: today's latch, the mute written once and never lifted,
/// and no adoption for a muted device (his next card never sent to her,
/// nor taken).
#[test]
fn a_device_past_its_share_at_a_copy_comes_back_once_what_it_holds_falls() {
    let dir = tempfile::tempdir().unwrap();
    let (wes, ted, eve) = (board(dir.path(), 5), board(dir.path(), 6), board(dir.path(), 7));
    let (room, roomy) = (binds(&[5, 6, 7], 10), binds(&[5, 6, 7], 100));
    let by = |a: &Db, b: &Db, (who, room): (u8, &Binds)| {
        let ask = b.ask().unwrap();
        let from = Device::parse(&actor(who)).unwrap();
        b.ingest(&a.missing(&ask, BUDGET).unwrap(), room, Some(&from)).unwrap()
    };
    let from = |a: &Db, b: &Db, who: u8| by(a, b, (who, &room));
    write(&wes, "c0", "a");
    from(&wes, &eve, 5);
    write(&wes, "c1", "milk");
    write(&wes, "c1", "tea");
    for n in 2..22 {
        write(&wes, &format!("c{n}"), "jam");
    }
    by(&wes, &ted, (5, &roomy));
    let through = from(&ted, &eve, 6);
    assert!(
        through.iter().any(|(_, t)| *t == Took::Over),
        "past his share: {through:?}"
    );
    let him = Device::parse(&actor(5)).unwrap();
    assert!(eve.ask().unwrap().muted.contains(&him), "her ask names him over");
    for _ in 0..3 {
        from(&wes, &eve, 5);
    }
    write(&wes, "c22", "fig");
    let last = from(&wes, &eve, 5);
    assert!(
        !last.is_empty() && last.iter().all(|(_, t)| *t == Took::Taken),
        "his next card taken: {last:?}"
    );
    let all = "SELECT id, title FROM cards ORDER BY id";
    assert_eq!(eve.read(all, &[]).unwrap(), wes.read(all, &[]).unwrap());
    assert!(!eve.ask().unwrap().muted.contains(&him), "named over no more");
}
