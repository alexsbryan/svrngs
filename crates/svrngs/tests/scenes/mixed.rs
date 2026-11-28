//! **Mixed** (svrn-docs `rings/current/APP_STATE.md` §11, W5, "No split on
//! an upgrade"): one group whose computers run two builds. Ana's and Cy's
//! run this build; Bo's runs the last release, [`LAST`], which keeps no app
//! database. The group forms, and each one's line in the record reaches the
//! other two, as it always did. Ana carries Magnets and moves a word, and
//! Cy's copy has it; Cy moves another, and Ana's has it. Bo's computer is in
//! each round's exchange of the record, and Ana's and Cy's each ask it once
//! a round for Magnets' database, at the members' route v1.13.3 doesn't
//! serve. Its no ends that exchange alone: every round still ends, each
//! member reached, Bo asked once in it and never again there. Bo's svrngs
//! goes on as it did: his line reaches the others after the moves, theirs
//! reach him, and his home page makes a link. Then Bo upgrades to this
//! build, his data dir kept, and his computer opens Magnets' database and
//! takes Ana's and Cy's moves.
//!
//! Last, the versions: Cy adds a column to Magnets' schema, a version 2
//! that runs for her at once as its author, and writes a note with it. Ana,
//! still on version 1, moves the same magnet, and the note stays on every
//! copy; her `SELECT *` gets every column of the widest schema, pinned here.
//!
//! The invariant it relies on: **no split on an upgrade** (APP_STATE §11,
//! W5, invariant 14). The record's format is v1.13.3's; a member that
//! doesn't answer the state route ends only that exchange; a schema only
//! adds, and every computer takes the widest a member runs.
//!
//! v1.13.3 runs as this build's people do: its own test binary as its
//! src/main.rs's `main`, at the scenes' build relay on 127.0.0.1
//! ([`last_release`]). The scene runs once a release, by name, out of the
//! gate (`mixed::chromium`).

use std::path::PathBuf;
use std::time::Duration;

use serde_json::{json, Value};

use crate::harness::{repo, Engine};
use crate::magnets::{asked, carry, put, shows, ANA_S, CY_S};
use crate::people::{late, Person, Scene};

/// The last release, which keeps no app database.
const LAST: &str = "v1.13.3";

/// [`LAST`]'s svrngs as `svrngs` here: its test binary that names its
/// src/main.rs's `main` in the ignored `svrngs_here` (its
/// tests/common/as_main.rs is this build's but for `installed_from`), at
/// `SCENES_LAST_RELEASE`, else where last-release.sh makes it, run at the
/// scenes' build relay as this build's people are. With neither, the scene
/// never ran, said in its place with the command that makes it.
fn last_release(scene: &Scene) -> Option<PathBuf> {
    let made = repo().join(format!("target/last-release/svrngs-{LAST}-test-key"));
    let exe = std::env::var_os("SCENES_LAST_RELEASE").map_or(made, PathBuf::from);
    if !exe.is_file() {
        scene.never_ran(&format!(
            "{LAST}'s svrngs isn't at {}: `sh crates/svrngs/tests/scenes/last-release.sh` makes it, or SCENES_LAST_RELEASE names it",
            exe.display()
        ));
        return None;
    }
    let dir = scene.scratch(LAST);
    std::fs::create_dir_all(&dir).unwrap();
    Some(crate::as_main::installed_from(
        &dir,
        &exe,
        crate::build_relay(),
    ))
}

/// `mark` written to `who`'s copy of `group`'s record, as a program on that
/// computer writes, as them.
async fn writes(who: &Person, group: &str, mark: &str) {
    let act = json!({ "op": "record", "payload": { "mixed": mark } });
    let path = format!("/v1/rail/append?namespace={group}");
    let (status, said) = who.post(&path, &act).await;
    assert!(
        status < 300,
        "{}'s record takes {mark}: {status} {said}",
        who.name
    );
}

/// Within two minutes, two of a round's minutes, each of `people`'s copies
/// of `group` holds every one of `marks`; else the scene is late.
async fn hold(people: &[&Person], group: &str, marks: &[String]) {
    for who in people {
        let mut lacks = marks.to_vec();
        for _ in 0..240 {
            let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
            let ops = log["ops"].to_string();
            lacks.retain(|m| !ops.contains(&format!("\"{m}\"")));
            if lacks.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        if !lacks.is_empty() {
            late(format!(
                "{}'s copy of the record lacks {lacks:?}; its log is {}",
                who.name,
                who.log().display()
            ))
            .await;
        }
    }
}

/// The key of `name`'s computer and that of their person, as `who`'s copy
/// of `group`'s record names them.
async fn keys(who: &Person, group: &str, name: &str) -> (String, String) {
    let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
    let people = log["people"].as_array().into_iter().flatten();
    let theirs = people.into_iter().find(|p| p["person"] == name);
    let theirs = theirs.unwrap_or_else(|| panic!("{name} in {}'s record: {log:#}", who.name));
    let key = |k: &str| theirs[k].as_str().unwrap().to_string();
    (key("key"), key("person_key"))
}

/// What `who`'s svrngs traced past `from` bytes of its log.
fn traced_since(who: &Person, from: u64) -> String {
    let all = std::fs::read(who.log()).unwrap();
    String::from_utf8_lossy(&all[from as usize..]).into_owned()
}

/// How long `who`'s log is now, where a later [`traced_since`] starts.
fn traced_to(who: &Person) -> u64 {
    std::fs::metadata(who.log()).unwrap().len()
}

/// The number `field=` says on `line`.
fn number(line: &str, field: &str) -> u64 {
    let at = line.split(&format!(" {field}=")).nth(1);
    let mut digits = at.unwrap_or_default().split(|c: char| !c.is_ascii_digit());
    digits
        .next()
        .unwrap_or_default()
        .parse()
        .unwrap_or(u64::MAX)
}

/// Each round `who`'s computer ended past `from` bytes of its log, at least
/// two within two of a round's minutes, reached both other members and no
/// member refused or failed it; and the state route of `bo`, the computer
/// whose build has none, was asked within those rounds once a round at most,
/// at a round's first chunk, and answered nothing. A round that ended late,
/// past the bound, is the scene's late.
async fn rounds_go_on(who: &Person, from: u64, bo: &str) {
    let ended = |log: &str| {
        let rounds = log
            .lines()
            .filter(|l| l.contains("ring sync: round namespaces="));
        rounds.map(str::to_string).collect::<Vec<_>>()
    };
    for _ in 0..240 {
        if ended(&traced_since(who, from)).len() >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    let log = traced_since(who, from);
    let rounds = ended(&log);
    if rounds.len() < 2 {
        late(format!(
            "{}'s computer ended {} rounds in two minutes past Ana's move, not two; its log is {}",
            who.name,
            rounds.len(),
            who.log().display()
        ))
        .await;
    }
    for round in &rounds {
        let (reached, failed) = (
            number(round, "peers_reached"),
            number(round, "peers_unreachable"),
        );
        let refused = number(round, "peers_refused");
        assert!(
            reached == 2 && failed == 0 && refused == 0,
            "{}'s round reached both other members, none failed or refused: {round}",
            who.name
        );
    }
    let peer = format!(" peer={bo} ");
    let asked: Vec<&str> = (log.lines())
        .filter(|l| l.contains("ring state: a chunk") && l.contains(&peer))
        .collect();
    eprintln!(
        "{}'s computer, past Ana's move: {} rounds ended, Bo's state route asked {} times: {asked:#?}",
        who.name,
        rounds.len(),
        asked.len()
    );
    assert!(
        !asked.is_empty(),
        "{}'s computer asked Bo's state route in its rounds",
        who.name
    );
    for call in &asked {
        assert!(
            number(call, "chunk") == 1 && call.contains(" took=None"),
            "{}'s computer asked Bo's state route once a round, at its first chunk, and took nothing: {call}",
            who.name
        );
    }
    // A round that began but didn't end yet may have asked too.
    assert!(
        asked.len() <= rounds.len() + 1,
        "{}'s computer asked Bo's state route {} times in {} rounds",
        who.name,
        asked.len(),
        rounds.len()
    );
    let record = |l: &&str| l.contains("ring sync: chunk") && l.contains(&peer);
    let exchanged = log.lines().any(|l| record(&l));
    assert!(
        exchanged,
        "{}'s computer exchanged the record with Bo's in those rounds",
        who.name
    );
}

/// Within a minute, `who`'s copy at `origin` answers `read` with `want`'s
/// rows; else the scene is late, saying what it answered.
async fn rows(who: &Person, origin: &str, read: &Value, want: &Value) {
    for _ in 0..120 {
        if &asked(who, origin, read.clone()).await["rows"] == want {
            return;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    let now = asked(who, origin, read.clone()).await;
    late(format!(
        "{}'s copy answers {now}, never {want}; its log is {}",
        who.name,
        who.log().display()
    ))
    .await
}

/// `who`'s computer traced, past `from` bytes of its log, the files of
/// Magnets that run as `person` as their yes, not their own change
/// (carried.rs `runs_as`).
fn runs_their_yes(who: &Person, from: u64, person: &str) -> bool {
    let log = traced_since(who, from);
    let own = " own=false";
    (log.lines()).any(|l| {
        l.contains("carried: the app's files now run as the person")
            && l.contains("\"magnets\"")
            && l.contains(person)
            && l.contains(own)
    })
}

/// The scene played as `scene!` plays one, in its place
/// `target/scenes/mixed-chromium`, but ignored: it compares a release with
/// the last one, so it is a release's recorded run (docs/DESIGN.md §7,
/// beside each target's tests), run by name with `--ignored`, and the
/// gate's scenes step lists it ignored and never runs it.
///
/// Planted, each watched failing, in ring_sync.rs `states`: a member that
/// doesn't answer the state route asked again until it does (`for chunk in
/// 1..`, `None => continue`), and Ana's computer asked Bo's 305,998 times
/// in one round that never ended, so Cy's fridge never showed goose; asked
/// again within the round's chunks alone (`None => continue`), and Ana's
/// computer asked it at a round's second chunk ([`rounds_go_on`]).
#[expect(
    clippy::module_inception,
    reason = "named as scene! names a scene's test, <file>::<scene>::<engine>, which docs/seams.tsv and the gate's step read"
)]
mod mixed {
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "a release's recorded run (docs/DESIGN.md §7): last-release.sh, then this by name with --ignored"]
    async fn chromium() {
        let played = super::mixed(crate::harness::Engine::Chromium);
        crate::people::judged("mixed-chromium".into(), played).await
    }
}
async fn mixed(engine: Engine) {
    let scene = Scene::new("mixed", engine).await;
    let Some(last) = last_release(&scene) else {
        return;
    };
    // Which build Bo's is, by its commit, said for the record; its last
    // line, past libtest's, whatever this build's program prints first.
    let home = scene.scratch(LAST);
    let mut version = std::process::Command::new(&last);
    let version = version.arg("version").env_clear().env("HOME", &home);
    let version = version.output().unwrap();
    let version = String::from_utf8_lossy(&version.stdout).into_owned();
    eprintln!("Bo's svrngs: {}", version.trim());
    scene.runs("Bo", Some(last));

    // The group forms: Ana's computer makes Fridge, Bo's and Cy's join by
    // her links, and each one's line reaches the other two.
    let (ana, fridge) = scene.founds("Ana", "Fridge").await;
    let bo = (scene.joins("Bo", &ana.invites(&fridge).await, "Fridge")).await;
    let cy = (scene.joins("Cy", &ana.invites(&fridge).await, "Fridge")).await;
    let marks = |when: &str| ["Ana", "Bo", "Cy"].map(|n| format!("{n} {when}"));
    for (who, mark) in [&ana, &bo, &cy].into_iter().zip(marks("joined")) {
        writes(who, &fridge, &mark).await;
    }
    hold(&[&ana, &bo, &cy], &fridge, &marks("joined")).await;
    let (bo_key, _) = keys(&ana, &fridge, "Bo").await;
    let (_, ana_person) = keys(&ana, &fridge, "Ana").await;

    // Ana carries Magnets and moves a word, and Cy's copy has it; Cy moves
    // another, and Ana's has it.
    carry(&ana.folder(&fridge).await);
    let at_ana = ana.opens(&fridge, "magnets").await;
    let at_cy = cy.opens(&fridge, "magnets").await;
    let (from_ana, from_cy) = (traced_to(&ana), traced_to(&cy));
    assert_eq!(
        asked(&ana, &at_ana, put(ANA_S, 1)).await,
        json!({ "count": 1 })
    );
    shows(&cy, &at_cy, &[ANA_S]).await;
    assert_eq!(
        asked(&cy, &at_cy, put(CY_S, 2)).await,
        json!({ "count": 1 })
    );
    shows(&ana, &at_ana, &[ANA_S, CY_S]).await;

    // Bo's svrngs goes on as it did: his line reaches Ana's and Cy's copies
    // after the moves, theirs reach his, and his home page makes a link.
    for (who, mark) in [&ana, &bo, &cy].into_iter().zip(marks("after")) {
        writes(who, &fridge, &mark).await;
    }
    hold(&[&ana, &bo, &cy], &fridge, &marks("after")).await;
    let link = bo.invites(&fridge).await;
    assert!(
        link.starts_with("svrngs://join/"),
        "Bo's home page makes a link: {link}"
    );
    let lends = bo.get(&format!("/v1/lends?namespace={fridge}")).await;
    eprintln!("What Bo's computer lists in Fridge: {lends:#}");

    // Every round Ana's and Cy's computers ended reached Bo's for the
    // record, and asked its state route once at most.
    rounds_go_on(&ana, from_ana, &bo_key).await;
    rounds_go_on(&cy, from_cy, &bo_key).await;
    let bo_traced = traced_since(&bo, 0);
    assert!(
        !bo_traced.contains("ring state:") && !bo.home.join(".svrngs/state").exists(),
        "Bo's {LAST} keeps no app database; its log is {}",
        bo.log().display()
    );

    // Bo upgrades to this build, his data dir kept, and his computer opens
    // Magnets' database and takes Ana's and Cy's moves.
    scene.runs("Bo", None);
    let bo = scene.again(bo, &[]).await;
    let at_bo = bo.opens(&fridge, "magnets").await;
    shows(&bo, &at_bo, &[ANA_S, CY_S]).await;

    // Cy adds a column to Magnets' schema, after its last, as the guide
    // says: version 2, which runs for her at once as its author, and she
    // writes a note with it.
    let schema = cy.folder(&fridge).await.join("apps/magnets/schema.sql");
    let v1 = std::fs::read_to_string(&schema).unwrap();
    let v2 = v1.replace("  position INTEGER\n", "  position INTEGER,\n  note TEXT\n");
    assert_ne!(v1, v2, "version 2 adds a note after position");
    let from_ana = traced_to(&ana);
    std::fs::write(&schema, v2).unwrap();
    let all = json!({ "sql": "SELECT * FROM magnets ORDER BY id" });
    let row = |id: &str, position: i64, note: Value| json!({ "id": id, "position": position, "note": note });
    let widened = json!([row(ANA_S, 1, Value::Null), row(CY_S, 2, Value::Null)]);
    rows(&cy, &at_cy, &all, &widened).await;
    let note = json!({ "write": [{ "update": "magnets", "key": { "id": ANA_S }, "set": { "note": "honk" } }] });
    assert_eq!(asked(&cy, &at_cy, note).await, json!({ "count": 1 }));

    // Ana, still on version 1, moves the same magnet once her copy has the
    // note, and it stays on every copy; her `SELECT *` gets every column of
    // the widest schema, pinned.
    let noted = json!([row(ANA_S, 1, json!("honk")), row(CY_S, 2, Value::Null)]);
    rows(&ana, &at_ana, &all, &noted).await;
    assert!(
        runs_their_yes(&ana, from_ana, &ana_person),
        "version 1 still runs for Ana, her yes; her log is {}",
        ana.log().display()
    );
    assert_eq!(
        asked(&ana, &at_ana, put(ANA_S, 3)).await,
        json!({ "count": 1 })
    );
    let moved = json!([row(ANA_S, 3, json!("honk")), row(CY_S, 2, Value::Null)]);
    assert_eq!(
        asked(&ana, &at_ana, all.clone()).await,
        json!({ "rows": moved, "more": false }),
        "Ana's version 1 kept the note, and its SELECT * reads every column"
    );
    rows(&cy, &at_cy, &all, &moved).await;
    rows(&bo, &at_bo, &all, &moved).await;
    drop((ana, bo, cy));
}
