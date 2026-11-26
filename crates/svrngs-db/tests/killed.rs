//! A copy killed as it takes a member's sets (svrn-docs
//! `rings/current/APP_STATE.md` §11, W1, "It crashes a copy mid-apply"): a
//! copy in a process of its own takes a member's sets a few at a time, and
//! is killed by SIGKILL wherever it is, its file left as the kill leaves it.
//! A child process killed is no seam (docs/CHECKS.md, "Seam migration
//! ledger"): SQLite's file under a real kill, and no socket, so where a kill
//! lands changes no verdict.

#[path = "common/held.rs"]
mod held;

use std::path::Path;
use std::sync::Arc;

use serde_json::{Value, json};
use svrngs_db::{Db, SigningKey, States};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, n INTEGER);";
/// Where each run of the copy is killed, and what the member wrote, drawn
/// from this seed, so a seed names a loop.
const SEED: u64 = 0x5eed_0c0b;
/// How many times the loop kills the copy.
const KILLS: usize = 12;
/// How many writes the member makes before the copy takes any.
const WRITES: u64 = 300;
/// What one call carries: a set or a few, so a copy takes many calls.
const BUDGET: usize = 1 << 10;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

/// Every key these tests sign with, bound, as tests/sync.rs binds them.
fn bound() -> svrngs_db::Binds {
    let keys = (1..=3).map(|n| svrngs_core::actor_of(&key(n)));
    svrngs_db::Binds::new(keys.map(|k| (k.clone(), k, false)), 4 << 20)
}

/// Computer `n`'s database of the board in `g`, under `dir`: the member's
/// carried as the root, synced from the start; any other's made from it.
fn copy(dir: &Path, n: u8) -> Arc<Db> {
    let states = States::new(&dir.join(format!("computer-{n}")), key(n));
    match n {
        1 => states.carry("g", "board", SCHEMA).unwrap(),
        _ => states.open("g", "board", Some(SCHEMA)).unwrap(),
    }
}

fn cards(db: &Db) -> Value {
    let read = db.read("SELECT id, title, n FROM cards ORDER BY id", &[]).unwrap();
    assert_eq!(read["more"], false, "every card in one read");
    read["rows"].clone()
}

/// splitmix64's next draw from `state`, below `below`.
fn drawn(state: &mut u64, below: u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) % below
}

/// `to` given every set `from` keeps that it lacks, a budget at a time.
fn synced(from: &Db, to: &Db) {
    for _ in 0..10_000 {
        let call = from.missing(&to.ask().unwrap(), BUDGET).unwrap();
        if call.sets.is_empty() {
            return;
        }
        to.ingest(&call, &bound(), None).unwrap();
    }
    panic!("never synced");
}

/// Not a check: computer 2's copy in a process of its own, which the test
/// below starts and kills. Under `KILLED_DIR`, it takes computer 1's sets a
/// call of [`BUDGET`] at a time, saying `took <n>` on stdout as call `n` is
/// taken, and `done` once none is left. Without `KILLED_DIR`, nothing.
#[test]
#[ignore = "the copy the kill loop below starts and kills"]
fn taking_until_killed() {
    let Some(dir) = std::env::var_os("KILLED_DIR") else {
        return;
    };
    let (member, mine) = (copy(Path::new(&dir), 1), copy(Path::new(&dir), 2));
    for n in 0.. {
        let call = member.missing(&mine.ask().unwrap(), BUDGET).unwrap();
        if call.sets.is_empty() {
            println!("done");
            return;
        }
        mine.ingest(&call, &bound(), None).unwrap();
        println!("took {n}");
    }
}

/// **A copy killed as it takes a member's sets keeps each set whole, and
/// syncs on to equal the member's** (APP_STATE §11, W1; §7, invariant 3):
/// computer 1 writes [`WRITES`] times, cards inserted, retitled and deleted,
/// drawn from [`SEED`]. Computer 2's copy takes its sets in a process of
/// its own ([`taking_until_killed`]), a few a call, and is killed by
/// SIGKILL [`KILLS`] times, each after 1 to 8 calls are taken and up to
/// 2 ms more, drawn from the seed, so a kill lands anywhere in a call, its
/// commit among them; and started again over its file. Then, here, it syncs
/// on: its cards equal the member's, the member has nothing left to send
/// it, its totals are the count over its file, and a newcomer synced from
/// it alone ends equal to it, so every row it holds has its set kept.
/// Plant, watched failing: a set kept and committed before its rows are
/// merged, in a transaction of their own (the copy claims sets whose rows
/// it lacks, and never equals the member's).
#[cfg(unix)]
#[test]
fn a_copy_killed_as_it_takes_sets_keeps_each_whole_and_syncs_on() {
    use std::io::BufRead as _;
    use std::os::unix::process::ExitStatusExt as _;
    use std::sync::mpsc::RecvTimeoutError;
    use std::time::Duration;
    let dir = tempfile::tempdir().unwrap();
    let mut seed = SEED;
    {
        let member = copy(dir.path(), 1);
        for n in 0..WRITES {
            let card = |seed: &mut u64| format!("c{}", drawn(seed, n.max(1)));
            let op = match drawn(&mut seed, 6) {
                0 if n > 0 => {
                    json!([{ "update": "cards", "key": { "id": card(&mut seed) }, "set": { "title": "retitled", "n": n } }])
                }
                1 if n > 0 => json!([{ "delete": "cards", "key": { "id": card(&mut seed) } }]),
                _ => json!([{ "insert": "cards", "row": { "id": format!("c{n}"), "title": "new", "n": n } }]),
            };
            member.write(&op, &bound()).unwrap();
        }
        copy(dir.path(), 2);
    }
    let mut done = false;
    for run in 0..KILLS {
        let (after, wait) = (1 + drawn(&mut seed, 8), drawn(&mut seed, 2_000));
        let log = dir.path().join(format!("run-{run}.log"));
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "taking_until_killed", "--ignored", "--nocapture"])
            .env("KILLED_DIR", dir.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::fs::File::create(&log).unwrap())
            .spawn()
            .unwrap();
        let out = child.stdout.take().unwrap();
        let (said, heard) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                if said.send(line).is_err() {
                    return;
                }
            }
        });
        let mut taken = 0;
        loop {
            let line = match heard.recv_timeout(Duration::from_secs(60)) {
                Ok(line) => line,
                Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {
                    let _ = child.kill();
                    panic!(
                        "run {run}: the copy said nothing for 60 s; its log is {}",
                        log.display()
                    );
                }
            };
            if line.starts_with("took ") {
                taken += 1;
                if taken == after {
                    std::thread::sleep(Duration::from_micros(wait));
                    let _ = child.kill();
                }
            }
            done |= line == "done";
        }
        let ended = child.wait().unwrap();
        let log = std::fs::read_to_string(&log).unwrap_or_default();
        assert!(
            ended.signal() == Some(9) || done && ended.success(),
            "run {run} took sets until killed: {ended:?}\n{log}"
        );
        if done {
            break;
        }
    }
    assert!(
        !done,
        "the copy was killed {KILLS} times before it had every set: more writes, or fewer kills"
    );

    let (member, mine) = (copy(dir.path(), 1), copy(dir.path(), 2));
    synced(&member, &mine);
    let theirs = cards(&member);
    assert!(theirs.as_array().unwrap().len() > 100, "{theirs}");
    assert_eq!(
        cards(&mine),
        theirs,
        "the copy killed {KILLS} times equals the member's"
    );
    let left = member.missing(&mine.ask().unwrap(), usize::MAX).unwrap();
    assert!(
        left.sets.is_empty(),
        "nothing left to send it: {} sets",
        left.sets.len()
    );
    held::as_counted(&mine, &dir.path().join("computer-2/state/g/board.db"));
    let newcomer = copy(dir.path(), 3);
    synced(&mine, &newcomer);
    assert_eq!(
        cards(&newcomer),
        theirs,
        "a newcomer synced from the killed copy alone equals it"
    );
}
