//! **Magnets** (apps/magnets; svrn-docs `rings/current/MAGNETS.md`, its
//! build `MAGNETS_BUILD.md` §3, slice A; demo/magnets.md): a fridge of word
//! magnets in a group of one, on a real svrngs. Ana makes Fridge and carries
//! Magnets in its folder, as a creator carries a folder app. She opens it and
//! puts three words after "I can explain the". She restarts her svrngs,
//! opens it again, and the sentence is still there: it lives in Magnets'
//! database on her computer, and her group's record has no line from it.
//! Then Magnets' own tests, the board's laws and its page in Chromium and in
//! WebKit as an iPhone, pass against her real route as they pass against
//! the stand-in (web/tests/route.mjs). This is the scene's first beat.
//!
//! The invariant it relies on: **the database is never in the record**
//! (APP_STATE §7, invariant 1): a move is a write by key to the app's own
//! database, beside the record, kept across a restart.
//!
//! **The lid closes** ([`magnets_lid`]; MAGNETS_BUILD §3, slices B and C,
//! proof 2 whole: two computers, a phone, the lid) is its second beat: Ana
//! moves a word, and once Bo's computer has it, Ana's lid closes. Cy joins
//! from her phone by Bo's link, opens the fridge from the group's page
//! through Bo's computer, and taps another word, her phone's own write,
//! signed on her phone. Ana opens her lid and sees it.
//! It relies on **a closing lid stranding no write** (APP_STATE §11, W2):
//! every computer whose person runs Magnets keeps its database, a page
//! opened there or not, and a write wakes the round, so a member holds it
//! within the round's seconds.
//!
//! **Show the source** ([`magnets_fork`]; MAGNETS_BUILD §3, slice D, proof
//! 3) is its third beat: Magnets' lines are counted by its own law, at most
//! 1,000. Ana copies it to a folder of her own and changes its prompt and
//! five words in `pack.json`, then brings the fork to a second group of
//! hers, Office, under the same folder name, while Magnets stays in Fridge.
//! Office's fridge opens fresh under its own prompt; Bo joins Office; a word
//! moved in each group appears in neither other's fridge, on any computer.
//! It relies on **each group's board being a database of its own** (APP_STATE
//! §4, "Where it lives"): a database is named by its group and its folder.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use serde_json::{json, Value};

use crate::harness::{browsers, relay_url, repo, Engine};
use crate::people::{late, sees, Browsers, Person, Scene};

/// What Ana puts after the prompt.
const WORDS: [&str; 3] = ["haunted", "lasagna", "situation"];

/// Magnets' shipped files, its folder but its tests, put in `folder`'s
/// `apps/magnets/`, as Ana carries it.
pub(crate) fn carry(folder: &Path) {
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
pub(crate) async fn asked(who: &Person, origin: &str, body: Value) -> Value {
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

/// Ana's magnet, then Cy's, by their ids in Magnets' pack
/// (apps/magnets/pack.json), each its word.
pub(crate) const ANA_S: &str = "goose";
pub(crate) const CY_S: &str = "moon";

/// The magnet `id` put at `position`, as a tap writes it (board.js `put`).
pub(crate) fn put(id: &str, position: i64) -> Value {
    json!({ "write": [{ "insert": "magnets", "row": { "id": id, "position": position } }] })
}

/// The sentence on `who`'s computer as a page asks it, by magnet, and
/// whether its copy says syncing.
async fn board(who: &Person, origin: &str) -> (Vec<String>, bool) {
    let read =
        json!({ "sql": "SELECT id FROM magnets WHERE position IS NOT NULL ORDER BY position, id" });
    let got = asked(who, origin, read).await;
    let rows = got["rows"].as_array().into_iter().flatten();
    let ids = rows.map(|r| r["id"].as_str().unwrap().to_string());
    (ids.collect(), got.get("syncing").is_some())
}

/// Within two minutes, two of a round's minutes, `who`'s sentence is
/// `want`, its copy synced; else the scene is late, saying what it shows.
pub(crate) async fn shows(who: &Person, origin: &str, want: &[&str]) {
    let want: Vec<String> = want.iter().map(|w| w.to_string()).collect();
    for _ in 0..120 {
        if board(who, origin).await == (want.clone(), false) {
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let (now, syncing) = board(who, origin).await;
    let log = who.log();
    late(format!(
        "{}'s fridge shows {now:?}, syncing {syncing}, never {want:?} synced; its log is {}",
        who.name,
        log.display()
    ))
    .await
}

scene!(
    /// Planted, each watched failing: the members' route answering no app
    /// database (ring_routes.rs `ring_state` refusing each as not held), so
    /// Bo's computer never has Ana's move; the group's signer signing no set
    /// (sign.js `setsSigned` refusing), so Cy's tap never moves.
    magnets_lid
);
async fn magnets_lid(engine: Engine) {
    let scene = Scene::new("magnets-lid", engine).await;
    let (page, _served) = scene.page().await;
    let relay = relay_url(&scene.relay);
    let made = [
        "new", "Fridge", "--as", "Ana", "--origin", &page, "--relay", &relay,
    ];
    let ana = scene.start("Ana", &made).await;
    let fridge = ana.group_named("Fridge").await;
    carry(&ana.folder(&fridge).await);
    let at_ana = ana.opens(&fridge, "magnets").await;
    // Bo's computer came from the group's page by its install line, so the
    // links it gives print on that page, and a phone opens them.
    scene.installs_from("Bo", &page);
    let bo = scene
        .joins("Bo", &ana.invites(&fridge).await, "Fridge")
        .await;
    let at_bo = bo.opens(&fridge, "magnets").await;

    // Ana moves a word, and once Bo's computer has it, her lid closes.
    assert_eq!(
        asked(&ana, &at_ana, put(ANA_S, 1)).await,
        json!({ "count": 1 })
    );
    shows(&bo, &at_bo, &[ANA_S]).await;
    ana.lid(true);

    // Cy joins from her phone by Bo's link, opens the fridge from the
    // group's page through Bo's computer, Ana's word on it, and taps
    // another word: her phone's own write, prepared by Bo's computer and
    // signed on her phone (svrn-docs APP_STATE §4, "Phones").
    let mut browsers = Browsers::driven(
        "demo/runs/canon/2026-10-17-house/people.mjs",
        engine,
        &[("cy", Engine::Webkit)],
    )
    .await;
    let link = (bo.on_home_page("/link", &json!({ "group": fridge, "device": false }))).await;
    let join = json!({ "do": "join", "who": "cy", "link": link["https"], "name": "Cy" });
    let joined = browsers.tell(join).await;
    let status = joined["status"].as_str().unwrap_or_default();
    assert!(status.contains("in Fridge"), "Cy's phone joins: {joined:#}");
    let taps = json!([
        { "lent": "magnets", "until": format!("the\\s+{ANA_S}") },
        { "click": CY_S, "until": format!("{ANA_S}\\s+{CY_S}") },
    ]);
    let moved = browsers
        .tell(json!({ "do": "steps", "who": "cy", "steps": taps }))
        .await;
    eprintln!("Cy's phone: {moved:#}");
    if moved["failed"] == true {
        late(format!(
            "Cy's phone didn't move {CY_S} within its bound: {moved:#}"
        ))
        .await;
    }
    shows(&bo, &at_bo, &[ANA_S, CY_S]).await;

    // Ana opens her lid, and her fridge shows Cy's word after hers.
    ana.lid(false);
    shows(&ana, &at_ana, &[ANA_S, CY_S]).await;
    let seen = sees(engine, "carried.mjs", json!({ "url": at_ana, "do": [] })).await;
    eprintln!("Ana's fridge, her lid open: {seen:#}");
    assert_eq!(
        sentence(&seen),
        [ANA_S, CY_S],
        "Ana sees Cy's word: {seen:#}"
    );
    drop((ana, bo));
}

/// The fork's prompt, and the five words it puts in place of Magnets'
/// first five, each a new id, as demo/magnets.md's proof 3 asks.
const PROMPT: &str = "Ask me about the";
const FORKED: [&str; 5] = ["kettle", "ferry", "spreadsheet", "cactus", "rumor"];

/// Magnets carried into `dir`'s `apps/magnets/`, then its pack changed:
/// its prompt, and its first five magnets new words with new ids, the rest
/// kept.
fn fork(dir: &Path) {
    carry(dir);
    let pack = dir.join("apps/magnets/pack.json");
    let mut read: Value = serde_json::from_str(&std::fs::read_to_string(&pack).unwrap()).unwrap();
    read["prompt"] = PROMPT.into();
    for (i, word) in FORKED.iter().enumerate() {
        read["magnets"][i] = json!({ "id": word, "word": word });
    }
    std::fs::write(&pack, serde_json::to_string_pretty(&read).unwrap()).unwrap();
}

scene!(
    /// Planted, watched failing: svrngs-db keeping an app's database in one
    /// file for every group (lib.rs `States::made`), and Office's fridge is
    /// refused, "this app's database here was made under another key".
    magnets_fork
);
async fn magnets_fork(engine: Engine) {
    let scene = Scene::new("magnets-fork", engine).await;

    // Magnets' lines, counted by its own law (apps/magnets/tests/lines.test.mjs).
    let counted = tokio::process::Command::new("node")
        .current_dir(repo())
        .args(["--test", "apps/magnets/tests/lines.test.mjs"])
        .stdin(Stdio::null())
        .output()
        .await
        .expect("node counts Magnets' lines");
    let said = String::from_utf8_lossy(&counted.stdout);
    assert!(counted.status.success(), "Magnets' count: {said}");
    let files = said
        .lines()
        .find_map(|l| l.strip_prefix("magnets: "))
        .expect("the count's files");
    let files: Vec<(String, u64)> = serde_json::from_str(files).unwrap();
    let lines: u64 = files.iter().map(|(_, n)| n).sum();
    eprintln!("Magnets ships {lines} lines: {files:?}");
    assert!(lines <= 1_000, "Magnets ships {lines} lines, past 1,000");

    // Ana's fridge in Fridge, with her word on it.
    let (ana, fridge) = scene.founds("Ana", "Fridge").await;
    carry(&ana.folder(&fridge).await);
    let at_fridge = ana.opens(&fridge, "magnets").await;
    assert_eq!(
        asked(&ana, &at_fridge, put(ANA_S, 1)).await,
        json!({ "count": 1 })
    );

    // Ana forks it in a folder of her own and brings the fork to Office.
    let relay = relay_url(&scene.relay);
    let ana = scene
        .again(ana, &["new", "Office", "--as", "Ana", "--relay", &relay])
        .await;
    let office = ana.group_named("Office").await;
    let own = scene.scratch("fork");
    fork(&own);
    let to = ana.folder(&office).await.join("apps/magnets");
    std::fs::create_dir_all(&to).unwrap();
    for file in std::fs::read_dir(own.join("apps/magnets")).unwrap() {
        let file = file.unwrap().path();
        std::fs::copy(&file, to.join(file.file_name().unwrap())).unwrap();
    }
    let at_office = ana.opens(&office, "magnets").await;
    assert_ne!(
        at_office, at_fridge,
        "the fork opens at an address of its own"
    );

    // Office's fridge opens fresh, under its own prompt, its own words in the
    // tray; Fridge's is as Ana left it.
    shows(&ana, &at_office, &[]).await;
    let seen = sees(engine, "carried.mjs", json!({ "url": at_office, "do": [] })).await;
    eprintln!("Office's fridge: {seen:#}");
    let page = seen["said"].as_str().unwrap_or_default();
    assert!(
        page.contains(PROMPT) && !page.contains("I can explain the"),
        "its own prompt: {seen:#}"
    );
    assert!(
        page.contains(FORKED[0]) && !page.contains(ANA_S),
        "its own words: {seen:#}"
    );
    shows(&ana, &at_fridge, &[ANA_S]).await;

    // Bo joins Office, and a word moved in each group shows in that group's
    // fridge alone, on every computer of it.
    let bo = scene
        .joins("Bo", &ana.invites(&office).await, "Office")
        .await;
    let at_bo = bo.opens(&office, "magnets").await;
    shows(&bo, &at_bo, &[]).await;
    assert_eq!(
        asked(&ana, &at_fridge, put("haunted", 2)).await,
        json!({ "count": 1 })
    );
    assert_eq!(
        asked(&bo, &at_bo, put(FORKED[0], 1)).await,
        json!({ "count": 1 })
    );
    shows(&ana, &at_office, &[FORKED[0]]).await;
    shows(&bo, &at_bo, &[FORKED[0]]).await;
    shows(&ana, &at_fridge, &[ANA_S, "haunted"]).await;
    let state = ana.home.join(".svrngs/state");
    for group in [&fridge, &office] {
        let db = state.join(group).join("magnets.db");
        assert!(
            db.is_file(),
            "a database of each group's own at {}",
            db.display()
        );
    }
    drop((ana, bo));
}
