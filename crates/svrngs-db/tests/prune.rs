//! Prune's work follows what a change outranks (svrn-docs
//! `rings/current/APP_STATE.md` §4, "What is kept"; §7, invariant 3). A kept
//! set stops standing only where a cell it holds is outranked, so prune
//! visits the sets that stood on the cells a write or a take outranks, and
//! the sets it keeps, never every set the copy keeps. Counted as prune says
//! it at debug, `visited`, never by the clock, so the bound holds on any
//! computer. A copy's 10,000 sets are written on a hundred computers and
//! taken: one computer's own 10,000 writes would each count every set it
//! keeps against its share of the database (`sync.rs`, `Db::held`), and take
//! a minute.

use std::sync::{Arc, LazyLock};

use serde_json::{Value, json};
use svrngs_db::{Binds, Db, Device, SigningKey, States};
use tracing::field::{Field, Visit};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt as _};

/// Magnets' table, as slice E's bounds run wrote it
/// (`demo/runs/app-state-bounds/2026-11-22/measure.rs`).
const SCHEMA: &str = "CREATE TABLE magnets (id TEXT NOT NULL PRIMARY KEY, position INTEGER);";
/// The computers that write W9's 10,000 rows (APP_STATE §11), and the rows
/// each writes, a set each.
const WRITERS: u8 = 100;
const EACH: u64 = 100;
/// The clock cells of a magnet: its row's own and its position.
const CELLS: u64 = 2;

/// Each prune's `visited` and `dropped`, as its debug event says them.
#[derive(Clone, Default)]
#[expect(
    clippy::disallowed_types,
    reason = "a test's trace sink, read on the thread that writes it"
)]
struct Prunes(Arc<std::sync::Mutex<Vec<(u64, u64)>>>);

#[derive(Default)]
struct Said(Option<u64>, Option<u64>);

impl Visit for Said {
    fn record_u64(&mut self, field: &Field, value: u64) {
        match field.name() {
            "visited" => self.0 = Some(value),
            "dropped" => self.1 = Some(value),
            _ => {}
        }
    }
    fn record_debug(&mut self, _: &Field, _: &dyn std::fmt::Debug) {}
}

impl<S: tracing::Subscriber> Layer<S> for Prunes {
    fn on_event(&self, event: &tracing::Event<'_>, _: Context<'_, S>) {
        let mut said = Said::default();
        event.record(&mut said);
        if let Said(Some(visited), Some(dropped)) = said {
            self.0.lock().unwrap().push((visited, dropped));
        }
    }
}

impl Prunes {
    /// The one prune since the last asked: what it visited and dropped.
    fn one(&self) -> (u64, u64) {
        let said: Vec<_> = self.0.lock().unwrap().drain(..).collect();
        assert_eq!(said.len(), 1, "one prune for one change: {said:?}");
        said[0]
    }
}

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

/// Ana's key, 1, and each writer's, 3 on, bound, in a request's room: made
/// once, as a key's derivation, a hundred times a write, would cost more
/// than the write.
static BINDS: LazyLock<Binds> = LazyLock::new(|| {
    let keys = [1]
        .into_iter()
        .chain(3..3 + WRITERS)
        .map(|n| svrngs_core::actor_of(&key(n)));
    Binds::new(keys.map(|k| (k.clone(), k, false)), 4 << 20)
});

/// The rows a write changed, as it answers them, once its prune visited
/// at most its set and the sets that stood on the cells of those rows.
fn write(db: &Db, ops: Value, prunes: &Prunes) -> (u64, u64) {
    let rows = db.write(&ops, &BINDS).unwrap().0["count"].as_u64().unwrap();
    let (visited, dropped) = prunes.one();
    assert!(
        visited <= 1 + CELLS * rows,
        "a write of {rows} rows had prune visit {visited} kept sets, past its own set and those standing on {} cells",
        CELLS * rows
    );
    (visited, dropped)
}

fn magnet(w: u8, n: u64) -> String {
    format!("m{w:03}-{n:03}")
}

fn moved(id: String, to: u64) -> Value {
    json!({ "update": "magnets", "key": { "id": id }, "set": { "position": to } })
}

/// What `to` takes of what `from`, computer `n`, keeps and it lacks, in one
/// call: at most each set it takes and the sets that stood on the cells of
/// `rows`.
fn take((from, n): (&Db, u8), to: &Db, (rows, prunes): (u64, &Prunes)) -> (u64, u64) {
    let call = from.missing(&to.ask().unwrap(), usize::MAX).unwrap();
    let by = Device::parse(&svrngs_core::actor_of(&key(n))).unwrap();
    let took = to.ingest(&call, &BINDS, Some(&by)).unwrap();
    let (visited, dropped) = prunes.one();
    let most = took.len() as u64 + CELLS * rows;
    assert!(
        visited <= most,
        "a take of {} sets changing {rows} rows had prune visit {visited} kept sets, past {most}",
        took.len()
    );
    (visited, dropped)
}

/// **Prune visits what a change outranks, at 10,000 rows** (svrn-docs
/// `app-state-diligence/prune-cost-2026-11-23/`): a hundred computers each
/// write a hundred magnets, a set each, and Ana takes each one's in a call.
/// Each write's prune visits at most its own set and the sets standing on
/// the cells it changes, and each take's the sets it takes and those
/// standing on theirs. At Ana's 10,000, a magnet moved visits its set and
/// the insert it outranks, and drops the insert, whose one cell was its
/// position: cr-sqlite keeps a cell of the row's own only once it is
/// deleted (`after_insert.rs`). Moved again, the same for the first move;
/// five moved at once, six visited and five dropped; one taken off drops
/// the set that wrote it. The last writer moves one and takes one off, and
/// Ana's take of the two sets visits four of her 10,000 and drops the two
/// inserts they outranked. Plant, watched failing: the prune before it,
/// which visited every kept set (`visited` 4 at the first writer's fourth
/// write).
#[test]
fn prune_visits_what_a_change_outranks_at_ten_thousand_rows() {
    let prunes = Prunes::default();
    let _heard = tracing::subscriber::set_default(tracing_subscriber::registry().with(prunes.clone()));
    // A second dispatcher kept alive, so a callsite is asked of this
    // thread's (svrngs-node tests/common/mod.rs, `heard`).
    let _two = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
    let dir = tempfile::tempdir().unwrap();
    let states = |n: u8| States::new(&dir.path().join(format!("computer-{n}")), key(n));
    let ana = states(1).carry("g", "magnets", SCHEMA).unwrap();
    let mut last = None;
    for w in 3..3 + WRITERS {
        let writer = states(w).open("g", "magnets", Some(SCHEMA)).unwrap();
        for n in 0..EACH {
            let op = json!([{ "insert": "magnets", "row": { "id": magnet(w, n), "position": n } }]);
            write(&writer, op, &prunes);
        }
        assert_eq!(take((&writer, w), &ana, (EACH, &prunes)), (EACH, 0), "each set it took");
        last = Some((writer, w));
    }
    let all = || ana.missing(&svrngs_db::Ask::default(), usize::MAX).unwrap().sets.len();
    assert_eq!(all(), 10_000, "Ana keeps 10,000 sets");
    let one = |n, to| write(&ana, json!([moved(magnet(3, n), to)]), &prunes);
    assert_eq!(one(7, 1), (2, 1), "its insert, outranked whole");
    assert_eq!(one(7, 2), (2, 1), "the first move, outranked whole");
    let five: Vec<_> = (10..15).map(|n| moved(magnet(3, n), n + 1)).collect();
    assert_eq!(
        write(&ana, Value::Array(five), &prunes),
        (6, 5),
        "five inserts, outranked whole"
    );
    let off = |w, n| json!([{ "delete": "magnets", "key": { "id": magnet(w, n) } }]);
    assert_eq!(
        write(&ana, off(3, 20), &prunes),
        (2, 1),
        "the set that wrote it, outranked whole"
    );
    let (writer, w) = last.unwrap();
    write(&writer, json!([moved(magnet(w, 30), 3)]), &prunes);
    write(&writer, off(w, 31), &prunes);
    let (visited, dropped) = take((&writer, w), &ana, (2, &prunes));
    assert_eq!(
        (visited, dropped),
        (4, 2),
        "two of {}, the inserts of the two magnets it changed",
        all()
    );
}
