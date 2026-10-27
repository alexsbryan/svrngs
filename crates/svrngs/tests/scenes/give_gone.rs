//! **Give, scenes 4 and 5: the keeper is gone, then back**
//! (give-app-scenes; svrn-docs rings/current/GIVE.md §3 scenes 4 and 5),
//! with the stand-in card (apps/give/tests/standin): Dee and Bo share Elm,
//! and each runs Give's program for computers beside svrngs. Bo starts the
//! stand-in for the group, and Dee opens it: its page shows its one start.
//! Bo's computer copies it to the group, and the copy reaches Dee's.
//!
//! Bo's computer loses power: its svrngs, Give's program and the app stop
//! at once. Fifteen minutes later Dee's row reads "Stand-in’s computer
//! hasn’t answered since <time>. You have a copy from <time>. Keep it
//! here?" Dee presses **Keep it here**, and Dee's computer starts it from
//! that copy: its page shows Bo's start, and Dee's after it.
//!
//! Bo's computer comes back. It sees that Dee's keeps the app now, and
//! starts nothing and lends nothing. It keeps the state it had as "Stand-in
//! as Bo’s computer had it at <time>", which every row lists, with **Start
//! instead**. Dee presses it: Dee's computer starts from what Bo's had,
//! and Dee's own state since is listed the same way, "as your computer had
//! it". Nothing is merged and nothing is thrown away.
//!
//! The fifteen minutes are the page's alone: whether the computer that
//! keeps an app is gone is the fold's answer at the time the page asks with
//! (apps/give/fold `Keeping::gone`, [`GONE`]), so Dee's page is read with
//! its clock held that far ahead (the driver's `clock` step), and back at
//! now once Dee has said yes. svrngs, Give's program and the record keep
//! this host's time throughout. The copy GIVE.md §3 scene 4's third step
//! offers when one doesn't start is played by apps/give/tests/keep.rs
//! `the_keeper_gone_then_back` and page.test.mjs, against a record of the
//! test's own: here no copy fails.
//!
//! The invariant it relies on: **a computer that comes back to an app
//! another keeps now keeps what it had apart, as a copy anyone may start
//! instead**, so one copy runs and none is lost.

use std::path::PathBuf;
use std::time::Duration;

use serde_json::{json, Value};

use crate::give::{
    brings, give_at, give_built, keeps, no_podman, prefix, standin, steps, stop_cleanly, Removed,
};
use crate::give_move::{lenders, runs, starts};
use crate::harness::Engine;
use crate::people::{late, Browsers, Person, Scene};

/// apps/give/fold `GONE`, in milliseconds, and a minute past it: how far
/// ahead Dee's page reads its clock. The scenes link no part of Give.
const GONE: u64 = 15 * 60 * 1000;

/// How often Bo's computer copies the stand-in, in seconds: one copy
/// reaches Dee's computer before Bo's loses power, and no second is begun.
const EVERY: u64 = 30;

/// `who`'s folder of `group`, as its svrngs says.
async fn folder(who: &Person, group: &str) -> PathBuf {
    let groups = who.get("/v1/groups").await;
    let g = (groups["groups"].as_array().into_iter().flatten()).find(|g| g["id"] == group);
    let at = g.and_then(|g| g["folders"][0].as_str());
    PathBuf::from(
        at.unwrap_or_else(|| panic!("{} keeps a folder of {group}: {groups:#}", who.name)),
    )
}

/// Bo's computer's copies of the stand-in, as `who`'s record lists them.
async fn copies(who: &Person, group: &str) -> Vec<Value> {
    let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
    (log["ops"].as_array().into_iter().flatten())
        .filter(|o| o["payload"]["kind"] == "give/copied" && o["person"] == "Bo")
        .cloned()
        .collect()
}

/// Whether the copy at `from` in the group's folder is whole on `there`,
/// file for file as `here` wrote it.
fn whole(here: &std::path::Path, there: &std::path::Path, from: &str) -> bool {
    let Ok(files) = std::fs::read_dir(here.join(from)) else {
        return false;
    };
    files.flatten().all(|f| {
        let theirs = there.join(from).join(f.file_name());
        std::fs::read(f.path())
            .ok()
            .is_some_and(|b| std::fs::read(theirs).ok() == Some(b))
    })
}

/// The stand-in's starts on Dee's computer, once they are not `not`.
async fn changed(
    browsers: &mut Browsers,
    dee: &Person,
    group: &str,
    not: &[String],
) -> Vec<String> {
    for _ in 0..60 {
        let on_dees = dee.lent_by(group, "Dee", "standin").await;
        let now = starts(browsers, "dee", &on_dees).await;
        if now != not {
            return now;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    late(format!(
        "the stand-in on Dee's computer still shows {not:?}"
    ))
    .await
}

scene!(
    /// Planted, each watched failing alone: the page never says a computer
    /// stopped answering (apps/give/page/index.js `takeover`, `if
    /// (!keeping.gone) return false` made `return false`), and Dee is never
    /// asked; then the computer that comes back never unlends the app
    /// (apps/give/src/keep.rs `had`, its `unshare` dropped), and svrngs,
    /// which keeps a lend across its restart, lends Bo's again.
    give_gone
);
async fn give_gone(engine: Engine) {
    let scene = Scene::new("give_gone", engine).await;
    if let Some(why) = no_podman() {
        return scene.never_ran(&why);
    }
    let program = give_built().await;
    let mut browsers = Browsers::driven(
        "demo/runs/canon/2026-10-17-house/people.mjs",
        engine,
        &[("dee", Engine::Chromium), ("bo", Engine::Chromium)],
    )
    .await;

    // Elm: Dee makes it; Bo joins by its link. Dee brings Give, and each
    // runs its program for computers.
    let (dee, elm) = scene.founds("Dee", "Elm").await;
    let bo = scene.joins("Bo", &dee.invites(&elm).await, "Elm").await;
    let card = standin();
    let card = card.to_str().unwrap();
    let (dees_scratch, bos_scratch) = (scene.scratch("dee-give"), scene.scratch("bo-give"));
    brings(&program, &dee, &dees_scratch, "Elm", card).await;
    let mut removed = Removed(
        vec![
            keeps(&program, &dee, &dees_scratch, card, None),
            keeps(&program, &bo, &bos_scratch, card, Some(EVERY)),
        ],
        &["dee", "bo"],
    );

    // Bo starts it for the group; Dee opens it on Bo's computer.
    let bos = give_at(&bo, &elm).await;
    let dees = give_at(&dee, &elm).await;
    steps(
        &mut browsers,
        "bo",
        json!([
            { "goto": bos },
            { "poll": bos, "re": "Start it for the group", "within": 60_000, "every": 2_000 },
            { "click": "Start it for the group", "in": "Stand-in", "until": "Stand-in · kept by your computer" },
        ]),
    )
    .await;
    let on_bos = dee.lent_by(&elm, "Bo", "standin").await;
    let bo_had = starts(&mut browsers, "dee", &on_bos).await;

    // Bo's computer copies it, and the copy reaches Dee's whole.
    let (bos_folder, dees_folder) = (folder(&bo, &elm).await, folder(&dee, &elm).await);
    let mut copy = None;
    for _ in 0..180 {
        let said = copies(&dee, &elm).await;
        let from = said
            .last()
            .and_then(|c| c["payload"]["from"].as_str())
            .map(String::from);
        if let Some(from) = from.filter(|f| whole(&bos_folder, &dees_folder, f)) {
            copy = Some(from);
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    let Some(copy) = copy else {
        late(format!(
            "no copy of Bo's reached Dee's computer whole within 90 s; Bo's says {:#?}",
            copies(&bo, &elm).await
        ))
        .await
    };
    eprintln!("Bo's copy {copy} is whole on Dee's computer");

    // Bo's computer loses power: its browser, svrngs, Give's program and
    // the app stop at once. (Give's page left open while its computer's
    // svrngs is down says nothing of it, and throws "Failed to fetch" each
    // round: a finding of this scene's first run, for its own row.)
    steps(&mut browsers, "bo", json!([{ "goto": "about:blank" }])).await;
    let mut bos_program = removed.0.pop().unwrap();
    let _ = bos_program.start_kill();
    let _ = bos_program.wait().await;
    let pod = format!("{}standin", prefix("Bo"));
    let killed = std::process::Command::new("podman")
        .args(["pod", "kill", &pod])
        .output()
        .unwrap();
    assert!(
        killed.status.success(),
        "{}",
        String::from_utf8_lossy(&killed.stderr)
    );
    bo.stop().await;
    let said = copies(&dee, &elm).await;
    let newest = said.last().and_then(|c| c["payload"]["from"].as_str());
    assert_eq!(
        newest,
        Some(copy.as_str()),
        "the copy Dee is offered is the one that reached Dee's computer"
    );

    // Fifteen minutes later, Dee is asked, and keeps it here. Bo's copy
    // is in Dee's record already, so the page that reads who keeps it
    // says, as it reads it, that Bo's computer stopped answering.
    let read = steps(
        &mut browsers,
        "dee",
        json!([
            { "clock": GONE + 60_000 },
            { "goto": dees },
            { "poll": dees, "re": "Stand-in · kept by Bo’s computer", "within": 60_000, "every": 2_000 },
        ]),
    )
    .await;
    let row = read
        .last()
        .and_then(|d| d["text"].as_str())
        .unwrap_or_default();
    let asked = [
        "Stand-in’s computer hasn’t answered since ",
        ". You have a copy from ",
        ". Keep it here?",
    ];
    assert!(
        asked.iter().all(|w| row.contains(w)),
        "Dee's page, fifteen minutes on, asks Dee to keep it: {row}"
    );
    steps(
        &mut browsers,
        "dee",
        json!([
            { "click": "Keep it here", "in": "Stand-in", "until": "Stand-in · kept by your computer" },
            { "clock": 0 },
        ]),
    )
    .await;

    // Dee's computer starts it from Bo's copy: Bo's start, and Dee's.
    let on_dees = dee.lent_by(&elm, "Dee", "standin").await;
    let dee_had = starts(&mut browsers, "dee", &on_dees).await;
    eprintln!("Dee keeps the stand-in from Bo's copy: {dee_had:?}");
    assert_eq!(
        dee_had[..1],
        bo_had[..],
        "Dee's computer starts from Bo's copy: {dee_had:?}, Bo's {bo_had:?}"
    );
    lenders(&dee, &elm, &["Dee"]).await;

    // Bo's computer comes back: it lends nothing, starts nothing, and keeps
    // what it had as a copy every row lists.
    let bo = scene.start("Bo", &[]).await;
    removed
        .0
        .push(keeps(&program, &bo, &bos_scratch, card, Some(EVERY)));
    let bos = give_at(&bo, &elm).await;
    steps(
        &mut browsers,
        "bo",
        json!([{ "poll": bos, "re": r"Stand-in · kept by Dee’s computer[\s\S]*Stand-in as your computer had it at \d{2}:\d{2}", "within": 180_000, "every": 2_000 }]),
    )
    .await;
    steps(
        &mut browsers,
        "dee",
        json!([{ "poll": dees, "re": r"Stand-in · kept by your computer[\s\S]*Stand-in as Bo’s computer had it at \d{2}:\d{2}", "within": 120_000, "every": 2_000 }]),
    )
    .await;
    lenders(&dee, &elm, &["Dee"]).await;
    assert!(!runs("Bo"), "Bo's computer, back, runs nothing");
    let on_dees = dee.lent_by(&elm, "Dee", "standin").await;
    let still = starts(&mut browsers, "dee", &on_dees).await;
    assert_eq!(
        still, dee_had,
        "Dee's state is untouched by Bo's coming back: nothing merged"
    );

    // Dee starts what Bo's computer had instead: Bo's start, then this one;
    // and Dee's own state since is kept, as Dee's computer had it.
    steps(
        &mut browsers,
        "dee",
        json!([
            { "goto": dees },
            { "click": "Start instead", "in": "Stand-in", "until": "Stand-in · kept by your computer" },
            { "poll": dees, "re": r"Stand-in as Bo’s computer had it at \d{2}:\d{2}[\s\S]*Stand-in as your computer had it at \d{2}:\d{2}|Stand-in as your computer had it at \d{2}:\d{2}[\s\S]*Stand-in as Bo’s computer had it", "within": 180_000, "every": 2_000 },
        ]),
    )
    .await;
    let now = changed(&mut browsers, &dee, &elm, &dee_had).await;
    eprintln!("Dee starts what Bo's computer had instead: {now:?}");
    assert_eq!(
        now.get(..1),
        Some(&bo_had[..]),
        "started from what Bo's had: {now:?}"
    );
    assert_eq!(now.len(), bo_had.len() + 1, "and one start more: {now:?}");

    stop_cleanly(removed).await;
}
