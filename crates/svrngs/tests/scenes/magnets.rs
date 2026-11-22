//! **Magnets** (apps/magnets; svrn-docs `rings/current/MAGNETS.md`, its
//! build `MAGNETS_BUILD.md` §3, slice A; demo/magnets.md): a fridge of word
//! magnets in a group of one, on a real svrngs. Ana makes Fridge and carries
//! Magnets in its folder, as a creator carries a folder app. She opens it and
//! puts three words after "I can explain the". She restarts her svrngs,
//! opens it again, and the sentence is still there: it lives in Magnets'
//! database on her computer, and her group's record has no line from it.
//! Then Magnets' own tests, the board's laws and its page in Chromium and in
//! WebKit as an iPhone, pass against her real route as they pass against
//! the stand-in (web/tests/route.mjs). This is the scene's first beat; the
//! lid closing on computers is its second, the next slice's.
//!
//! The invariant it relies on: **the database is never in the record**
//! (APP_STATE §7, invariant 1): a move is a write by key to the app's own
//! database, beside the record, kept across a restart.

use std::path::Path;
use std::process::Stdio;

use serde_json::{json, Value};

use crate::harness::{browsers, repo, Engine};
use crate::people::{sees, Person, Scene};

/// What Ana puts after the prompt.
const WORDS: [&str; 3] = ["haunted", "lasagna", "situation"];

/// Magnets' shipped files, its folder but its tests, put in `folder`'s
/// `apps/magnets/`, as Ana carries it.
fn carry(folder: &Path) {
    let to = folder.join("apps/magnets");
    std::fs::create_dir_all(&to).unwrap();
    for file in std::fs::read_dir(repo().join("apps/magnets")).unwrap() {
        let file = file.unwrap().path();
        if file.is_file() {
            std::fs::copy(&file, to.join(file.file_name().unwrap())).unwrap();
        }
    }
}

/// The words of the sentence the page `seen` said, after its prompt: none
/// where it shows its blank, `____`.
fn sentence(seen: &Value) -> Vec<String> {
    let said = seen["said"].as_str().unwrap_or_default();
    let after = said.split_once("I can explain the").map(|(_, s)| s);
    let words = after.and_then(|s| s.split("Your group sees this sentence").next());
    let words = words.unwrap_or_default().split_whitespace();
    words.filter(|w| *w != "____").map(str::to_string).collect()
}

/// `who`'s svrngs's port and token, from its data dir, as a program there
/// reads them.
fn api(who: &Person) -> (u16, String) {
    let dir = who.home.join(".svrngs");
    let at = std::fs::read_to_string(dir.join("api")).unwrap();
    let port = at.trim().rsplit(':').next().unwrap().trim_end_matches('/');
    let token = std::fs::read_to_string(dir.join("token")).unwrap();
    (port.parse().unwrap(), token.trim().to_string())
}

/// `body` posted to Magnets' `svrngs/sql` at `origin` on `who`'s computer,
/// as a program there asks it, as them: the answer.
async fn asked(who: &Person, origin: &str, body: Value) -> Value {
    let ((port, token), url) = (api(who), reqwest::Url::parse(origin).unwrap());
    let host = format!("{}:{port}", url.host_str().unwrap());
    let ask = axum::http::Request::post("/svrngs/sql").header("host", host);
    let ask = ask.header("content-type", "application/json");
    let ask = ask.body(body.to_string()).unwrap();
    let reply = svrngs_node::node::as_you_at(([127, 0, 0, 1], port).into(), &token, ask).await;
    let said = String::from_utf8_lossy(reply.body()).to_string();
    assert!(
        reply.status().is_success(),
        "{}'s svrngs/sql: {said}",
        who.name
    );
    serde_json::from_str(&said).unwrap()
}

/// The lines of `group`'s record on `who`'s computer.
async fn record(who: &Person, group: &str) -> Vec<Value> {
    let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
    log["ops"].as_array().cloned().unwrap_or_default()
}

scene!(
    /// Planted, watched failing: the route writing each move through the
    /// record's `append_act` as well (Ana's record gains a line a move);
    /// Magnets' database file kept in a directory a restart empties (the
    /// sentence gone after the restart); route.mjs's `database()` leaving
    /// the board as it found it (Magnets' tests fail on Ana's three words).
    magnets
);
async fn magnets(engine: Engine) {
    let scene = Scene::new("magnets", engine).await;
    let (ana, fridge) = scene.founds("Ana", "Fridge").await;
    carry(&ana.folder(&fridge).await);
    let origin = ana.opens(&fridge, "magnets").await;

    // Ana opens the fridge, its sentence empty, and puts three words in.
    let lines = record(&ana, &fridge).await;
    let taps: Vec<Value> = (WORDS.iter())
        .map(|w| json!([format!("#tray button:text-is('{w}')")]))
        .collect();
    let seen = sees(engine, "carried.mjs", json!({ "url": origin, "do": taps })).await;
    eprintln!("Ana's fridge: {seen:#}");
    assert_eq!(
        sentence(&json!({ "said": seen["first"] })),
        Vec::<String>::new(),
        "a fresh fridge: {seen:#}"
    );
    assert_eq!(sentence(&seen), WORDS, "Ana's sentence: {seen:#}");
    let board = json!({ "sql": "SELECT id, position FROM magnets ORDER BY position" });
    let kept = asked(&ana, &origin, board).await;
    let placed = json!([{ "id": WORDS[0], "position": 1 }, { "id": WORDS[1], "position": 2 }, { "id": WORDS[2], "position": 3 }]);
    assert_eq!(
        kept,
        json!({ "rows": placed, "more": false }),
        "kept in Magnets' database, not syncing"
    );
    let written = record(&ana, &fridge).await;
    let new: Vec<_> = written.iter().filter(|o| !lines.contains(o)).collect();
    assert!(
        new.is_empty(),
        "Ana's record has no line from the moves: {new:#?}"
    );

    // Ana restarts her svrngs, opens the fridge again, and her sentence is there.
    let ana = scene.again(ana, &[]).await;
    let seen = sees(engine, "carried.mjs", json!({ "url": origin, "do": [] })).await;
    assert_eq!(
        sentence(&seen),
        WORDS,
        "Ana's sentence after her restart: {seen:#}"
    );
    let new: Vec<_> = (record(&ana, &fridge).await.into_iter())
        .filter(|o| !lines.contains(o))
        .collect();
    let magnets = new
        .iter()
        .filter(|o| o.to_string().contains("magnets/"))
        .count();
    assert_eq!(
        magnets, 0,
        "no line of Magnets' in Ana's record after her restart: {new:#?}"
    );

    // Magnets' own tests, on Ana's real route (web/tests/route.mjs).
    let (port, token) = api(&ana);
    let db = ana
        .home
        .join(".svrngs/state")
        .join(&fridge)
        .join("magnets.db");
    assert!(db.is_file(), "Magnets' database at {}", db.display());
    let label = origin
        .trim_start_matches("http://")
        .split('.')
        .next()
        .unwrap();
    let route = json!({ "origin": origin, "port": port, "token": token, "db": db });
    let opens = json!([{ "port": port, "token": token, "names": [label] }]);
    let files = [
        "apps/magnets/tests/board.test.mjs",
        "apps/magnets/tests/page.test.mjs",
    ];
    let ran = tokio::process::Command::new("node")
        .current_dir(repo())
        .args(["--test", "--test-reporter=spec", "--test-timeout=120000"])
        .args(files)
        .env("SVRNGS_ROUTE", route.to_string())
        .env("SVRNGS_OPENS", opens.to_string())
        .env("SVRNGS_BROWSERS", browsers())
        .stdin(Stdio::null())
        .output()
        .await
        .expect("node runs Magnets' tests");
    let said = String::from_utf8_lossy(&ran.stdout);
    eprintln!(
        "Magnets' tests on Ana's route:\n{said}{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    assert!(
        ran.status.success(),
        "Magnets' tests on Ana's route: {said}"
    );
    // Five, and the states', whose switches are the stand-in's alone.
    assert!(
        said.contains("ℹ pass 5") && said.contains("ℹ skipped 1"),
        "{said}"
    );
    drop(ana);
}
