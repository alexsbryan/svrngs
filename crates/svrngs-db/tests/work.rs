//! A change's work follows what it changes (svrn-docs
//! `rings/current/APP_STATE.md` §4, "What is kept"; §7, invariants 3 and
//! 9), never all a copy keeps, as svrngs-db says it at debug, never by the
//! clock, so each bound holds on any computer:
//! - prune, `visited`: a kept set stops standing only where a cell it holds
//!   is outranked, so prune visits the sets that stood on the cells a write
//!   or a take outranks, and the sets it keeps;
//! - the seal's read of what its computer holds, `read`, in the steps
//!   SQLite took for it: one row of the totals SQLite keeps as each set is
//!   kept or dropped (`sync.rs`, `recorded`).

use std::sync::{Arc, LazyLock};

use serde_json::{Value, json};
use svrngs_db::{Binds, Db, Device, SigningKey, States};
use tracing::field::{Field, Visit};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt as _};

/// Magnets' table, as slice E's bounds run wrote it
/// (`demo/runs/app-state-bounds/2026-11-22/measure.rs`).
const SCHEMA: &str = "CREATE TABLE magnets (id TEXT NOT NULL PRIMARY KEY, position INTEGER);";
/// W9's rows (APP_STATE §11), each written as a set of its own.
const ROWS: u64 = 10_000;
/// The clock cells of a magnet: its row's own and its position.
const CELLS: u64 = 2;

/// Each prune's `visited` and `dropped`, and each seal's `read`, as their
/// debug events say them.
#[derive(Clone, Default)]
#[expect(
    clippy::disallowed_types,
    reason = "a test's trace sink, read on the thread that writes it"
)]
struct Heard(Arc<std::sync::Mutex<Prunes>>);

/// Each prune's visited and dropped, and each seal's read, in order.
type Prunes = (Vec<(u64, u64)>, Vec<i64>);

#[derive(Default)]
struct Said(Option<u64>, Option<u64>, Option<i64>);

impl Visit for Said {
    fn record_u64(&mut self, field: &Field, value: u64) {
        match field.name() {
            "visited" => self.0 = Some(value),
            "dropped" => self.1 = Some(value),
            _ => {}
        }
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == "read" {
            self.2 = Some(value);
        }
    }
    fn record_debug(&mut self, _: &Field, _: &dyn std::fmt::Debug) {}
}

impl<S: tracing::Subscriber> Layer<S> for Heard {
    fn on_event(&self, event: &tracing::Event<'_>, _: Context<'_, S>) {
        let mut said = Said::default();
        event.record(&mut said);
        let mut heard = self.0.lock().unwrap();
        if let Said(Some(visited), Some(dropped), _) = said {
            heard.0.push((visited, dropped));
        }
        heard.1.extend(said.2);
    }
}

impl Heard {
    /// The one prune since the last asked: what it visited and dropped; and
    /// each seal's read since.
    fn one(&self) -> ((u64, u64), Vec<i64>) {
        let (prunes, reads) = &mut *self.0.lock().unwrap();
        assert_eq!(prunes.len(), 1, "one prune for one change: {prunes:?}");
        (prunes.remove(0), std::mem::take(reads))
    }
}

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

/// Ana's and Ben's keys bound, in a request's room: made once, as a key's
/// derivation costs about what a write does.
static BINDS: LazyLock<Binds> = LazyLock::new(|| {
    let keys = [1, 2].map(|n| svrngs_core::actor_of(&key(n)));
    Binds::new(keys.map(|k| (k.clone(), k, false)), 4 << 20)
});

/// The rows a write changed, as it answers them, once its prune visited
/// at most its set and the sets that stood on the cells of those rows, and
/// its seal read what its computer holds in the steps each write's after the
/// first did: the first finds no row of its computer's yet. `reads` holds
/// each write's steps.
fn write(db: &Db, ops: Value, (heard, reads): (&Heard, &mut Vec<i64>)) -> (u64, u64) {
    let rows = db.write(&ops, &BINDS).unwrap().0["count"].as_u64().unwrap();
    let ((visited, dropped), read) = heard.one();
    assert!(
        visited <= 1 + CELLS * rows,
        "a write of {rows} rows had prune visit {visited} kept sets, past its own set and those standing on {} cells",
        CELLS * rows
    );
    let [steps] = read[..] else {
        panic!("one read of what this computer holds a write: {read:?}")
    };
    reads.push(steps);
    if let [_, second, .., _] = reads[..] {
        assert_eq!(
            steps,
            second,
            "write {} read what its computer holds in {steps} steps, the second in {second}: its read follows what the copy keeps",
            reads.len()
        );
    }
    (visited, dropped)
}

fn moved(n: u64, to: u64) -> Value {
    json!({ "update": "magnets", "key": { "id": format!("m{n:05}") }, "set": { "position": to } })
}

/// What `to` takes of what `from`, computer `n`, keeps and it lacks, in one
/// call: at most each set it takes and the sets that stood on the cells of
/// `rows`.
fn take((from, n): (&Db, u8), to: &Db, (rows, heard): (u64, &Heard)) -> (u64, u64) {
    let call = from.missing(&to.ask().unwrap(), usize::MAX).unwrap();
    let by = Device::parse(&svrngs_core::actor_of(&key(n))).unwrap();
    let took = to.ingest(&call, &BINDS, Some(&by)).unwrap();
    let ((visited, dropped), _) = heard.one();
    let most = took.len() as u64 + CELLS * rows;
    assert!(
        visited <= most,
        "a take of {} sets changing {rows} rows had prune visit {visited} kept sets, past {most}",
        took.len()
    );
    (visited, dropped)
}

/// **A change's work follows what it changes, at 10,000 rows** (svrn-docs
/// `app-state-diligence/prune-cost-2026-11-23/`): Ana writes W9's 10,000
/// magnets, a set each. Each write's prune visits at most its own set and
/// the sets standing on the cells it changes, and its seal reads what Ana's
/// computer holds in the steps the second write's did, the first finding
/// no row of hers yet. At 10,000, a magnet
/// moved visits its set and the insert it outranks, and drops the insert,
/// whose one cell was its position: cr-sqlite keeps a cell of the row's own
/// only once it is deleted (`after_insert.rs`). Moved again, the same for
/// the first move; five moved at once, six visited and five dropped; one
/// taken off drops the set that wrote it. Ben takes all of hers in one
/// call, which visits each set it keeps. He moves one and takes one off,
/// and Ana's take of his two sets visits four of her 10,000 and drops the
/// two inserts they outranked. Plants, each watched failing: the prune that
/// visited every kept set (`visited` 4 at Ana's fourth write); the seal's
/// read of every kept set's bytes, `Db::held`'s count before its totals.
#[test]
fn a_change_s_work_follows_what_it_changes_at_ten_thousand_rows() {
    let heard = Heard::default();
    let _heard = tracing::subscriber::set_default(tracing_subscriber::registry().with(heard.clone()));
    // A second dispatcher kept alive, so a callsite is asked of this
    // thread's (svrngs-node tests/common/mod.rs, `heard`).
    let _two = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
    let dir = tempfile::tempdir().unwrap();
    let states = |n: u8| States::new(&dir.path().join(format!("computer-{n}")), key(n));
    let ana = states(1).carry("g", "magnets", SCHEMA).unwrap();
    let ben = states(2).open("g", "magnets", Some(SCHEMA)).unwrap();
    let (mut at_ana, mut at_ben) = (Vec::new(), Vec::new());
    for n in 0..ROWS {
        let op = json!([{ "insert": "magnets", "row": { "id": format!("m{n:05}"), "position": n } }]);
        write(&ana, op, (&heard, &mut at_ana));
    }
    let mut one = |n, to| write(&ana, json!([moved(n, to)]), (&heard, &mut at_ana));
    assert_eq!(one(7, 1), (2, 1), "its insert, outranked whole");
    assert_eq!(one(7, 2), (2, 1), "the first move, outranked whole");
    let five: Vec<_> = (10..15).map(|n| moved(n, n + 1)).collect();
    let five = write(&ana, Value::Array(five), (&heard, &mut at_ana));
    assert_eq!(five, (6, 5), "five inserts, outranked whole");
    let off = |n: u64| json!([{ "delete": "magnets", "key": { "id": format!("m{n:05}") } }]);
    let off20 = write(&ana, off(20), (&heard, &mut at_ana));
    assert_eq!(off20, (2, 1), "the set that wrote it, outranked whole");
    let kept = ana.missing(&ben.ask().unwrap(), usize::MAX).unwrap().sets.len() as u64;
    assert_eq!(take((&ana, 1), &ben, (ROWS, &heard)), (kept, 0), "each set it keeps");
    write(&ben, json!([moved(30, 3)]), (&heard, &mut at_ben));
    write(&ben, off(31), (&heard, &mut at_ben));
    let (visited, dropped) = take((&ben, 2), &ana, (2, &heard));
    assert_eq!(
        (visited, dropped),
        (4, 2),
        "four of {kept}, two of them the inserts it outranked"
    );
}
