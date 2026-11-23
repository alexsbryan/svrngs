//! svrngs-db's own copy behind `Replica`, `DbCopy`: a database as a computer
//! keeps one (`svrngs_db::States`), written in the write form, its sets
//! sealed, signed and synced by svrngs-db's own sync, its ask, what it sends
//! and what it takes. The simulator's sync under `Variant::Vouched`, §4's
//! marks with the ask naming its haves and a mark adopted as svrngs-db
//! adopts one, is the reference for that
//! bookkeeping (`sync.rs`, `Book`): at every call a `DbCopy` sends or takes,
//! it is held to the sets the reference sends and takes, and to its marks
//! and haves after. Each run says its first disagreement (`watched`), as a
//! pair does for the engine (`live.rs`). Each copy prunes what stands no
//! more on it as svrngs-db does, in the write's or the call's own
//! transaction, so invariant 3 holds here whole: its kept sets cover its
//! clock rows, it keeps none that doesn't stand (`tests/sync.rs`,
//! `cost_follows_what_is_live_on_each_copy`),
//! and a newcomer synced from it ends equal to it.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;

use rusqlite::Connection;
use rusqlite::types::ValueRef;
use serde::{Deserialize, Serialize};
use serde_json::{Value as Json, json};
use svrngs_db::{Ask, Db, SigningKey, States, Took};

use crate::change::{Change, ChangeSet, SetId, Site, State, Val};
use crate::live::named;
use crate::replica::{Op, Replica};
use crate::rule::Rule;
use crate::sim::{self, Verdict};
use crate::sim::{SCHEMA, TABLES};
use crate::sync::{Book, Call, Carry, Ingested, Variant};
use crate::syncs;

const GROUP: &str = "g";
const APP: &str = "board";

/// Each table's key columns, in key order, as `SCHEMA` declares them.
const KEYS: &[(&str, &[&str])] = &[
    ("magnets", &["id"]),
    ("votes", &["person_key", "item"]),
    ("tiles", &["id"]),
];

/// What a run's copies share: each device's site in the simulator by its
/// key, each database's real site, and every set sealed in the run, as
/// svrngs-db signed it, by its name in the simulator.
#[derive(Default)]
pub struct Run {
    devices: BTreeMap<String, Site>,
    sites: BTreeMap<[u8; 16], Site>,
    sets: BTreeMap<SetId, svrngs_db::Set>,
}

/// svrngs-db's database, as one computer keeps it.
pub struct DbCopy {
    site: Site,
    db: Arc<Db>,
    /// A second connection to the same file, which reads its clocks and rows.
    side: Option<Connection>,
    run: Rc<RefCell<Run>>,
    _dir: tempfile::TempDir,
}

/// Copies for one run, each of its own computer, sharing what the run sealed.
pub fn maker() -> impl Fn(Site) -> DbCopy {
    let run = Rc::new(RefCell::new(Run::default()));
    move |site| DbCopy::new(site, &run)
}

/// A change row as svrngs-db writes one into a set, for a test to read.
#[derive(Serialize, Deserialize)]
struct Row {
    table: String,
    pk: Vec<u8>,
    cid: String,
    val: Kept,
    col_version: i64,
    cl: i64,
    seq: i64,
}

#[derive(Serialize, Deserialize)]
enum Kept {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

thread_local! {
    static RUN: RefCell<String> = const { RefCell::new(String::new()) };
    static PARTED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// What `f` returns, run as `run`, and each copy's first disagreement with
/// the reference's bookkeeping in it.
pub fn watched<T>(run: String, f: impl FnOnce() -> T) -> (T, Vec<String>) {
    RUN.set(run);
    PARTED.take();
    let out = f();
    (out, PARTED.take())
}

fn json(v: &Val) -> Json {
    match v {
        Val::Null => Json::Null,
        Val::Integer(i) => json!(i),
        Val::Real(r) => json!(r),
        Val::Text(t) => json!(t),
        Val::Blob(b) => json!({ "base64": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b) }),
    }
}

fn val(v: ValueRef) -> Val {
    match v {
        ValueRef::Null => Val::Null,
        ValueRef::Integer(i) => Val::Integer(i),
        ValueRef::Real(r) => Val::Real(r),
        ValueRef::Text(t) => Val::Text(String::from_utf8(t.to_vec()).expect("text is UTF-8")),
        ValueRef::Blob(b) => Val::Blob(b.to_vec()),
    }
}

/// `key`'s columns named, for the write form.
fn keyed(table: &str, key: &[Val]) -> serde_json::Map<String, Json> {
    let names = KEYS.iter().find(|(t, _)| *t == table).expect("a table of SCHEMA").1;
    names.iter().zip(key).map(|(n, v)| (n.to_string(), json(v))).collect()
}

/// `ops` in svrngs's write form (APP_STATE §4, "The write form").
fn form(ops: &[Op]) -> Json {
    let named = |cols: &[(&str, Val)]| {
        cols.iter()
            .map(|(c, v)| (c.to_string(), json(v)))
            .collect::<serde_json::Map<_, _>>()
    };
    let each = ops.iter().map(|op| match op {
        Op::Insert {
            table,
            key,
            row,
            on_conflict,
        } => {
            assert!(
                row.iter().all(|(c, _)| on_conflict.contains(c)),
                "the write form sets every column it names"
            );
            let mut all = keyed(table, key);
            all.extend(named(row));
            json!({ "insert": table, "row": all })
        }
        Op::Update { table, key, set } => json!({ "update": table, "key": keyed(table, key), "set": named(set) }),
        Op::Delete { table, key } => json!({ "delete": table, "key": keyed(table, key) }),
    });
    Json::Array(each.collect())
}

impl DbCopy {
    pub fn new(site: Site, run: &Rc<RefCell<Run>>) -> DbCopy {
        let dir = tempfile::tempdir().expect("a directory for the copy");
        let key = SigningKey::from_bytes(&[site.0[0]; 32]);
        let device = svrngs_core::actor_of(&key);
        // Copy 1 carries the board, so its copy is the root every other is
        // synced from (APP_STATE §7, invariant 8).
        let states = States::new(dir.path(), key);
        let db = if site == Site::of(1) {
            states.carry(GROUP, APP, SCHEMA)
        } else {
            states.open(GROUP, APP, Some(SCHEMA))
        };
        let db = db.expect("the database opens");
        let side = Connection::open(dir.path().join(format!("state/{GROUP}/{APP}.db"))).expect("a second connection");
        let nonce: [u8; 16] = side
            .query_row("SELECT nonce FROM svrngs_db", [], |r| r.get(0))
            .expect("its nonce");
        let real = svrngs_core::set::site(GROUP, APP, &device, &nonce);
        let mut shared = run.borrow_mut();
        shared.devices.insert(device, site);
        shared.sites.insert(real, site);
        drop(shared);
        DbCopy {
            site,
            db,
            side: Some(side),
            run: run.clone(),
            _dir: dir,
        }
    }

    /// The record as the run binds it: each device to the person whose
    /// rows it writes (`sync.rs`, `person`), none cut.
    fn binds(&self) -> svrngs_db::Binds {
        let run = self.run.borrow();
        let each = run
            .devices
            .iter()
            .map(|(k, s)| (k.clone(), crate::sync::person(*s), false));
        svrngs_db::Binds::new(each, 4 << 20)
    }

    fn side(&self) -> &Connection {
        self.side.as_ref().expect("open until the copy drops")
    }

    /// A real set as the simulator names it: its rows at its device's site
    /// and its seq; and it noted, as the run's.
    fn change_set(&self, set: &svrngs_db::Set) -> ChangeSet {
        let site = self.run.borrow().devices[set.head.device.as_str()];
        let rows: Vec<Row> = postcard::from_bytes(&set.changes).expect("a set's rows");
        let each = rows.into_iter().map(|r| Change {
            table: r.table,
            pk: r.pk,
            cid: r.cid,
            val: match r.val {
                Kept::Null => Val::Null,
                Kept::Integer(i) => Val::Integer(i),
                Kept::Real(f) => Val::Real(f),
                Kept::Text(t) => Val::Text(t),
                Kept::Blob(b) => Val::Blob(b),
            },
            col_version: r.col_version,
            db_version: set.head.seq.get() as i64,
            site,
            cl: r.cl,
            seq: r.seq,
        });
        let cs = ChangeSet::new(each.collect()).expect("a set has rows");
        self.run.borrow_mut().sets.insert(cs.id(), set.clone());
        cs
    }

    /// Every set this copy keeps, as svrngs-db keeps them.
    fn sets(&self) -> Vec<svrngs_db::Set> {
        self.db.missing(&Ask::default(), usize::MAX).expect("its sets").sets
    }

    /// The real ask in the simulator's names: its marks, and its haves.
    fn book(&self) -> (BTreeMap<Site, i64>, BTreeSet<SetId>) {
        let ask = self.db.ask().expect("its ask");
        let run = self.run.borrow();
        let marks = ask.marks.iter().map(|(s, m)| (run.sites[s], m.get() as i64)).collect();
        let haves = ask.haves.iter().map(|(s, q)| (run.sites[s], q.get() as i64)).collect();
        (marks, haves)
    }

    /// The first disagreement with the reference, said once a run.
    fn parted(&self, what: String) {
        let run = RUN.with_borrow(String::clone);
        PARTED.with_borrow_mut(|p| {
            if p.is_empty() {
                p.push(format!("{run}, copy {}: {what}", self.site.0[0]));
            }
        });
    }
}

impl Drop for DbCopy {
    fn drop(&mut self) {
        let side = self.side.take().expect("open until the copy drops");
        if let Err(e) = svrngs_db::close(side)
            && !std::thread::panicking()
        {
            panic!("the copy's second connection closes: {e}");
        }
    }
}

fn ids(sets: &[ChangeSet]) -> Vec<SetId> {
    sets.iter().map(ChangeSet::id).collect()
}

fn names(ids: &BTreeSet<SetId>) -> Vec<String> {
    ids.iter().map(|id| named(*id)).collect()
}

impl Replica for DbCopy {
    fn site(&self) -> Site {
        self.site
    }

    fn write(&mut self, ops: &[Op]) -> Option<ChangeSet> {
        let last = "SELECT coalesce(max(rowid), 0) FROM svrngs_sets";
        let before: i64 = self.side().query_row(last, [], |r| r.get(0)).unwrap();
        let binds = self.binds();
        self.db
            .write(&form(ops), &binds)
            .unwrap_or_else(|r| panic!("the write {ops:?} is refused: {r}"));
        let sealed = "SELECT bytes FROM svrngs_sets WHERE rowid > ?1";
        let sealed: Option<Vec<u8>> = self.side().query_row(sealed, [before], |r| r.get(0)).ok();
        sealed.map(|b| self.change_set(&postcard::from_bytes(&b).expect("a kept set")))
    }

    fn take(&mut self, set: &ChangeSet) {
        panic!(
            "svrngs-db takes whole calls, under the sync's variants: {}",
            named(set.id())
        );
    }

    fn kept(&self) -> Vec<ChangeSet> {
        self.sets().iter().map(|s| self.change_set(s)).collect()
    }

    fn standing(&self) -> BTreeSet<SetId> {
        let mut stmt = self
            .side()
            .prepare_cached("SELECT DISTINCT site_id, db_version FROM crsql_changes")
            .unwrap();
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, [u8; 16]>(0)?, r.get::<_, i64>(1)?)))
            .unwrap();
        let run = self.run.borrow();
        rows.map(|r| r.unwrap()).map(|(s, v)| (run.sites[&s], v)).collect()
    }

    fn state(&self) -> State {
        let mut state = State::new();
        for t in TABLES {
            let keys = KEYS.iter().find(|(n, _)| *n == t.name).unwrap().1;
            let quoted = |n: &[&str]| n.iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(", ");
            let sql = format!(
                "SELECT crsql_pack_columns({}), {} FROM \"{}\"",
                quoted(keys),
                quoted(t.cols),
                t.name
            );
            let mut stmt = self.side().prepare_cached(&sql).unwrap();
            let mut rows = stmt.query([]).unwrap();
            while let Some(r) = rows.next().unwrap() {
                let cols = t.cols.iter().enumerate();
                let values = cols.map(|(i, c)| (c.to_string(), val(r.get_ref(i + 1).unwrap())));
                let values = values.filter(|(_, v)| *v != Val::Null).collect();
                state.insert((t.name.to_owned(), r.get(0).unwrap()), values);
            }
        }
        state
    }

    fn removed(&mut self, _site: Site) {}

    fn takes_sets(&self) -> bool {
        false
    }

    /// The call taken whole by svrngs-db, as a computer takes one: its sets
    /// as svrngs-db signed them, and a call by marks full, one set carried
    /// alone not. What it took, refused and passed by must be the
    /// reference's, and so must its marks and haves after.
    fn called(&mut self, call: &Call, out: &Ingested, book: &Book) {
        let real = {
            let run = self.run.borrow();
            let to_real: BTreeMap<Site, [u8; 16]> = run.sites.iter().map(|(r, s)| (*s, *r)).collect();
            let sets = call.sets.iter().map(|s| run.sets[&s.id()].clone()).collect();
            let marks = call
                .marks
                .iter()
                .flatten()
                .map(|(s, m)| (to_real[s], svrngs_db::Seq::new(*m as u64).expect("a mark is a seq")))
                .collect();
            let ask = Ask {
                marks,
                ..Ask::default()
            };
            svrngs_db::Call {
                ask,
                sets,
                full: call.marks.is_some(),
                synced: call.marks.is_some() && call.synced,
            }
        };
        // The caller's own device, which vouches for its own site's mark.
        let from = {
            let run = self.run.borrow();
            let from = run.devices.iter().find(|(_, s)| Some(**s) == call.from);
            from.and_then(|(d, _)| svrngs_db::Device::parse(d))
        };
        let binds = self.binds();
        let got = self
            .db
            .ingest(&real, &binds, from.as_ref())
            .expect("svrngs-db takes the call");
        let run = self.run.borrow();
        let of = |t: Took| -> BTreeSet<SetId> {
            got.iter()
                .filter(|(_, k)| *k == t)
                .map(|((s, q), _)| (run.sites[s], q.get() as i64))
                .collect()
        };
        let set = |v: &[SetId]| v.iter().copied().collect::<BTreeSet<_>>();
        let mut wrong = Vec::new();
        for (what, real, want) in [
            ("took", &of(Took::Taken), &set(&out.taken)),
            ("refused", &of(Took::Refused), &set(&out.refused)),
            ("passed by", &of(Took::Held), &set(&out.held)),
        ] {
            if real != want {
                wrong.push(format!(
                    "svrngs-db {what} {:?}, the reference {:?}",
                    names(real),
                    names(want)
                ));
            }
        }
        let other = got
            .iter()
            .filter(|(_, t)| !matches!(t, Took::Taken | Took::Refused | Took::Held));
        wrong.extend(other.map(|((_, q), t)| format!("svrngs-db's set at seq {} ended {t:?}", q.get())));
        drop(run);
        let (marks, haves) = self.book();
        if marks != *book.marks() || haves != *book.above() {
            let said = format!("{marks:?} above {:?}", names(&haves));
            wrong.push(format!(
                "svrngs-db's marks {said}; the reference's {:?} above {:?}",
                book.marks(),
                names(book.above())
            ));
        }
        if !wrong.is_empty() {
            self.parted(format!(
                "taking {:?}: {}",
                ids(&call.sets).iter().map(|i| named(*i)).collect::<Vec<_>>(),
                wrong.join("; ")
            ));
        }
    }

    /// What svrngs-db sends `to` by `to`'s own ask must be what the
    /// reference sends, and whether it has synced what the reference's book
    /// says, as the call carries it.
    fn sent(&self, to: &DbCopy, call: &Call) {
        if call.synced == self.db.syncing() {
            let said = format!(
                "sending {}: svrngs-db synced {}, the reference's book {}",
                to.site.0[0],
                !self.db.syncing(),
                call.synced
            );
            self.parted(said);
        }
        let real = self
            .db
            .missing(&to.db.ask().expect("its ask"), usize::MAX)
            .expect("what it lacks");
        let run = self.run.borrow();
        let real: BTreeSet<SetId> = real
            .sets
            .iter()
            .map(|s| (run.devices[s.head.device.as_str()], s.head.seq.get() as i64))
            .collect();
        let want: BTreeSet<SetId> = ids(&call.sets).into_iter().collect();
        drop(run);
        if real != want {
            let said = format!(
                "sending {}: svrngs-db sends {:?}, the reference {:?}",
                to.site.0[0],
                names(&real),
                names(&want)
            );
            self.parted(said);
        }
    }
}

/// One seeded run on svrngs-db's copies under `carry`, met and judged, and
/// each copy's first disagreement with the reference's bookkeeping.
fn seeded(seed: u64, carry: Carry) -> (Verdict, Vec<String>) {
    watched(format!("seed {seed}, {}", carry.name()), || {
        sim::one_synced(seed, Rule::default(), Variant::Vouched, carry, &maker())
    })
}

/// The seeds `range` on svrngs-db's copies, carried both ways, shared among
/// the computer's threads: each failure, W1's or the bookkeeping's.
fn on_seeds(range: std::ops::Range<u64>) -> Vec<String> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    std::thread::scope(|s| {
        let each = (0..threads).map(|t| {
            let range = range.clone();
            s.spawn(move || {
                let mut failed = Vec::new();
                for seed in range.filter(|seed| seed % threads == t) {
                    for carry in Carry::ALL {
                        let (v, parted) = seeded(seed, carry);
                        failed.extend(v.failure.as_ref().map(|(class, e)| format!("{}: {e}", class.name())));
                        failed.extend(parted);
                        if v.passed + v.refetched + v.resent + v.late > 0 {
                            failed.push(format!("seed {seed}, {}: {v:?}", carry.name()));
                        }
                    }
                }
                failed
            })
        });
        let each: Vec<_> = each.collect();
        each.into_iter()
            .flat_map(|h| h.join().expect("a thread of seeds"))
            .collect()
    })
}

/// **svrngs-db's sync is the reference's, at every call, on the gate's
/// seeds** (APP_STATE §11, W1, on computers), its first 100 here, 9 to 10 s
/// on twelve threads, all 1,000 with `W1_DB_SEEDS=0..1000`, 105 s (svrn-docs
/// `app-state-diligence/b-sync-build-2026-11-22/`): each seed's copies are
/// svrngs-db's own databases, sending and taking by its own ask, carried a
/// set at a time and by calls; at every call each sends what the
/// reference's bookkeeping sends, takes, refuses and passes by what it
/// does, and ends with its marks and haves; every copy ends on the
/// reference, its kept sets cover its clock rows, a newcomer synced from it
/// is equal to it, and in the ten rounds after the meet nothing is passed,
/// fetched again, sent again or taken late (invariants 3, as far as it goes
/// without prune, 11 and 12). Plants, each watched failing: the spike's
/// highest-taken mark in svrngs-db (`adopt` taking the mark to the seq
/// counted); the ask naming no haves.
#[test]
fn svrngs_db_s_sync_is_the_reference_s_on_the_gate_s_seeds() {
    let failed = on_seeds(syncs::range("W1_DB_SEEDS", "0..100"));
    assert!(failed.is_empty(), "{} failed:\n{}", failed.len(), failed.join("\n"));
}

/// The soak on svrngs-db's copies, out of the gate: `W1_DB_SEEDS`, 0..100,000
/// by default, run on the Fedora box.
#[test]
#[ignore = "the soak on svrngs-db's copies, not the gate's"]
fn svrngs_db_s_sync_soak() {
    let failed = on_seeds(syncs::range("W1_DB_SEEDS", "0..100000"));
    assert!(failed.is_empty(), "{} failed:\n{}", failed.len(), failed.join("\n"));
}

/// W's sync cases on svrngs-db's copies, each held at every call: two sets
/// carried apart (invariant 11), a removed device's set relayed (invariant
/// 6), a set outranked and a member's marks adopted. The refused case
/// refuses an owned row, which svrngs-db refuses with slice D; its 2^53
/// refusal is `tests/sync.rs`'s invariant 12.
#[test]
fn w_s_sync_cases_hold_on_svrngs_db_s_copies() {
    let sync = Variant::Vouched.sync().expect("§4's marks, as svrngs-db adopts them");
    let cases: [(&str, CaseOf); 3] = [
        ("two sets carried", syncs::carried::<DbCopy>),
        ("a removed device's set", syncs::cut::<DbCopy>),
        ("a set outranked by one refused", syncs::outranked::<DbCopy>),
    ];
    let mut failed = Vec::new();
    for (name, case) in cases {
        let ((said, v), parted) = watched(name.to_owned(), || case(sync, &maker()));
        println!("{name}, svrngs-db's copies:\n{said}");
        failed.extend(v.err().map(|e| format!("{name}: {e}")));
        failed.extend(parted);
    }
    assert!(failed.is_empty(), "{failed:#?}");
}

type CaseOf = fn(crate::sync::Sync, &dyn Fn(Site) -> DbCopy) -> (String, Result<(), String>);

/// **The review's case, between computers alone** (APP_STATE §7, invariant
/// 6's twin; §4, "A removed device"), on svrngs-db's copies syncing by
/// their own full calls, as the round makes them: Mia places the cat at 1
/// and the four computers meet; the fourth goes offline. Jo moves the cat to
/// 2 and Sam to 3, and the three online meet after each. Sam is removed:
/// the members' sync refuses his computer from then on
/// (`the_state_route_serves_a_standing_member_alone`), so it neither calls
/// nor is called, and his sets stay bound in the record, as every device's
/// that was ever admitted. The offline computer returns, and Jo's, Mia's and
/// it meet by full calls. Sam's move outranks Jo's, so the online copies
/// prune Jo's set before the offline one returns, and it never sees it.
/// Every copy ends on Sam's 3, the reference over the three sets, and none
/// on 1, the value before Jo's move. Plants, each watched failing: the
/// spike's standing-now rule, the record binding Sam's key no more once he
/// is removed (the offline computer drops his move as not bound, and shows
/// 1); no prune (Jo's copy still keeps Jo's set).
#[test]
fn the_review_s_case_between_computers() {
    let make = maker();
    let mut copies: Vec<DbCopy> = (1..=4).map(|i| make(Site::of(i))).collect();
    let (jo, sam, mia, away) = (0, 1, 2, 3);
    // The record binds every key it ever admitted, a removed one's too: a
    // removal stops what the device writes from then on, at the members'
    // sync, and undoes nothing (Eudoxia's `db::set`).
    let key = |i: u8| svrngs_core::actor_of(&SigningKey::from_bytes(&[i; 32]));
    let (admitted, sam_s): (BTreeSet<String>, _) = ((1..=4).map(key).collect(), key(2));
    let removed = std::cell::Cell::new(false);
    let bound = svrngs_db::Binds::new(admitted.iter().map(|k| (k.clone(), k.clone(), false)), 4 << 20);
    let meet = |copies: &[DbCopy], who: &[usize]| {
        for _ in 0..10 {
            let before: Vec<Ask> = who.iter().map(|&c| copies[c].db.ask().unwrap()).collect();
            for &from in who {
                for &to in who.iter().filter(|&&t| t != from) {
                    let call = copies[from]
                        .db
                        .missing(&copies[to].db.ask().unwrap(), usize::MAX)
                        .unwrap();
                    copies[to].db.ingest(&call, &bound, None).unwrap();
                }
            }
            if who.iter().map(|&c| copies[c].db.ask().unwrap()).collect::<Vec<_>>() == before {
                return;
            }
        }
        panic!("the meet settles");
    };
    let moved = |pos| {
        vec![Op::insert(
            "magnets",
            vec![Val::text("cat")],
            vec![("pos", Val::Integer(pos))],
        )]
    };
    let first = copies[mia].write(&moved(1)).expect("the cat placed");
    meet(&copies, &[jo, sam, mia, away]);
    let jo_s = copies[jo].write(&moved(2)).expect("Jo's move");
    meet(&copies, &[jo, sam, mia]);
    let sam_set = copies[sam].write(&moved(3)).expect("Sam's move");
    meet(&copies, &[jo, sam, mia]);
    for (c, name) in [(jo, "Jo's"), (sam, "Sam's"), (mia, "Mia's")] {
        let kept = ids(&copies[c].kept());
        assert!(
            !kept.contains(&jo_s.id()),
            "{name} copy still keeps Jo's set: the case needs it pruned"
        );
    }
    removed.set(true);
    println!(
        "Sam ({}) removed: {}; Jo's, Mia's and the offline computer meet",
        &sam_s[..8],
        removed.get()
    );
    meet(&copies, &[jo, mia, away]);
    let want = crate::reference::state(TABLES, [&first, &jo_s, &sam_set]);
    let pos = |c: &DbCopy| c.db.read("SELECT pos FROM magnets WHERE id = 'cat'", &[]).unwrap()["rows"].clone();
    for (c, name) in [(jo, "Jo's"), (mia, "Mia's"), (away, "the offline")] {
        println!(
            "{name} copy: the cat at {}, keeping {:?}",
            pos(&copies[c]),
            ids(&copies[c].kept()).iter().map(|i| named(*i)).collect::<Vec<_>>()
        );
        assert_eq!(
            copies[c].state(),
            want,
            "{name} copy is not the reference's, Sam's 3: {}",
            pos(&copies[c])
        );
    }
}
