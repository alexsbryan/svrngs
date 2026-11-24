//! Magnets' read on svrngs-db, as its route runs it (`Db::read`, the page's
//! own `BOARD` from apps/magnets/board.js), at its pack's 16 magnets and at
//! W9's 10,000 rows: each read's time, best, median and slowest of 1,000,
//! against `QUERY_MOST`, and its answer's bytes against `ANSWER_MOST`; and
//! a tap's write, median of 200. Run as a test of svrngs-db, copied to
//! crates/svrngs-db/tests/ for the run and taken out after:
//! `cargo test -p svrngs-db --test measure -- --nocapture`.
use std::time::Instant;

use serde_json::json;
use svrngs_db::{Binds, SigningKey, States};

const SCHEMA: &str = "CREATE TABLE magnets (id TEXT NOT NULL PRIMARY KEY, position INTEGER);";
const BOARD: &str = "SELECT id, position FROM magnets WHERE position IS NOT NULL ORDER BY position, id";

fn us(mut v: Vec<u128>) -> (u128, u128, u128) {
    v.sort();
    (v[0], v[v.len() / 2], v[v.len() - 1])
}

#[test]
fn measure() {
    for rows in [16usize, 10_000] {
        let dir = tempfile::tempdir().unwrap();
        let db = States::new(dir.path(), SigningKey::from_bytes(&[1; 32]))
            .carry("g", "magnets", SCHEMA)
            .unwrap();
        let binds = Binds::new([], 0);
        let mut writes = Vec::new();
        for n in 0..rows {
            let op = json!([{ "insert": "magnets", "row": { "id": format!("m{n:05}"), "position": n } }]);
            let t = Instant::now();
            db.write(&op, &binds).unwrap();
            writes.push(t.elapsed().as_micros());
        }
        let mut reads = Vec::new();
        let mut bytes = 0;
        for _ in 0..1000 {
            let t = Instant::now();
            let answer = db.read(BOARD, &[]).unwrap();
            reads.push(t.elapsed().as_micros());
            bytes = answer.to_string().len();
        }
        let (w, r) = (us(writes), us(reads));
        println!("MEASURE {rows} rows: a tap's write µs best/median/slowest {w:?}; BOARD read µs {r:?} (QUERY_MOST {} µs); answer {bytes} B (ANSWER_MOST {} B, ROWS_MOST {})",
            svrngs_db::QUERY_MOST.as_micros(), svrngs_db::ANSWER_MOST, svrngs_db::ROWS_MOST);
    }
}
