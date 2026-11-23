//! What the crate's own items hold, where no test outside it reaches them:
//! each names the plant it was watched failing on.

use serde_json::json;

use super::*;

/// The board under `dir`, made from one table and holding one card.
const CARDS: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);";

fn board(dir: &Path) -> (States, Arc<Db>) {
    let states = States::new(dir, SigningKey::from_bytes(&[1; 32]));
    let db = states.carry("g", "board", CARDS).unwrap();
    let binds = Binds::new([(db.device.to_string(), db.device.to_string(), false)], 4 << 20);
    db.write(
        &json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]),
        &binds,
    )
    .unwrap();
    (states, db)
}

/// **A crash between a restore's moves leaves the app's name a whole
/// database** (the delta review's D6): the board restored whole into a
/// copy and its WAL folded in, as `States::restored` does, then as many of
/// `moves` made as a crash allowed, none, one or both. svrngs, started
/// again, opens the board by its name and schema, as the node's next ask
/// does, and finds its card, in the live file or the copy, never a fresh,
/// empty board; and a later restore, which drops what `board.db.old`
/// names, keeps it. Plant, watched failing: the live file renamed aside
/// first, as before (after one move the next ask made a fresh board under
/// the name, its read `rows: []`, the card left in `board.db.old` for the
/// next restore to delete).
#[test]
fn a_crash_between_a_restore_s_moves_leaves_the_app_s_name_a_whole_database() {
    for made in 0..=2 {
        let dir = tempfile::tempdir().unwrap();
        let (states, db) = board(dir.path());
        let path = dir.path().join("state/g/board.db");
        let (copy, old) = (path.with_extension("db.new"), path.with_extension("db.old"));
        db.conn().execute("VACUUM INTO ?1", [copy.to_string_lossy()]).unwrap();
        let restored = Connection::open(&copy).unwrap();
        restored.execute_batch("UPDATE svrngs_db SET file = NULL").unwrap();
        close(restored).unwrap();
        // As a restore does before its moves: the live file's WAL folded in.
        db.conn().execute_batch("PRAGMA journal_mode=DELETE").unwrap();
        for (step, from, to) in moves(&path, &copy, &old).into_iter().take(made) {
            step(from, to).unwrap();
        }
        drop((db, states));
        let states = States::new(dir.path(), SigningKey::from_bytes(&[1; 32]));
        let db = states.open("g", "board", Some(CARDS)).unwrap();
        let card = json!([{ "id": "c1", "title": "milk" }]);
        let cards = |db: &Db| db.read("SELECT id, title FROM cards", &[]).unwrap()["rows"].clone();
        assert_eq!(cards(&db), card, "after {made} moves");
        let wider = format!("{CARDS}\nCREATE INDEX cards_title ON cards (title);");
        states.widen(&db, &wider).unwrap();
        assert_eq!(cards(&db), card, "after {made} moves and a restore");
    }
}
