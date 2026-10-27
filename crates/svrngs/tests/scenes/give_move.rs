//! **Give, scene 3: moving it** (give-app-scenes; svrn-docs
//! rings/current/GIVE.md §3 scene 3), with the stand-in card
//! (apps/give/tests/standin): Cy and Bo share Elm, and each runs Give's
//! program for computers beside svrngs. Cy starts the stand-in for the
//! group, and Bo opens it: its page shows the one start it has had. Cy is
//! moving out: on the stand-in's row Cy presses **Move it to…**, then
//! **Bo’s computer**, and reads "Waiting for Bo to say yes." Bo's row asks
//! "Cy wants to move Stand-in to your computer." Bo presses **Keep it
//! here**. Cy's computer copies it one last time and stops it; Bo's starts
//! from exactly that copy. Each row then reads who keeps it, and Cy opens
//! it through Cy's own svrngs, now lent by Bo's computer: its page shows
//! the start it had on Cy's computer, and the one on Bo's after it.
//!
//! The scene runs Give as scene 1 does (give.rs): its program beside this
//! build's svrngs, on svrngs's surface alone. The copy goes from Cy's
//! computer to Bo's through the group's folder, as svrngs syncs it.
//!
//! The invariant it relies on: **the computer a move goes to starts from
//! the last copy of the one it leaves**, so the app's state after a move is
//! its state before it: nothing played is lost.

use std::time::Duration;

use serde_json::json;

use crate::give::{
    brings, give_at, give_built, keeps, no_podman, prefix, standin, steps, stop_cleanly, Removed,
};
use crate::harness::Engine;
use crate::people::{Browsers, Person, Scene};

/// The stand-in's starts, as `who`'s browser reads them at `lent`, once
/// it serves them: each start adds its line before the app answers, so
/// what it serves is its whole state.
pub(crate) async fn starts(browsers: &mut Browsers, who: &str, lent: &str) -> Vec<String> {
    let at = format!("{}/starts", lent.trim_end_matches('/'));
    let done = steps(
        browsers,
        who,
        json!([{ "goto": at }, { "poll": at, "re": r"\d{2}:\d{2}:\d{2}", "within": 180_000, "every": 2_000 }]),
    )
    .await;
    let text = done
        .last()
        .and_then(|d| d["text"].as_str())
        .unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// Who lends the stand-in in `group`, as `who`'s svrngs lists it, once
/// the list is `want`, within 60 s.
pub(crate) async fn lenders(who: &Person, group: &str, want: &[&str]) {
    let mut got = Vec::new();
    for _ in 0..120 {
        let lends = who.get(&format!("/v1/lends?namespace={group}")).await;
        got = (lends["lent"].as_array().into_iter().flatten())
            .filter(|l| l["name"] == "standin")
            .filter_map(|l| l["person"].as_str().map(String::from))
            .collect::<Vec<_>>();
        if got == want {
            return;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!(
        "one copy, lent by the computer that keeps it: {} lists {got:?}, not {want:?}",
        who.name
    );
}

/// Whether the stand-in's pod runs on `who`'s computer: a pod Give
/// stopped stays, its state in its volumes, until it starts again.
pub(crate) fn runs(who: &str) -> bool {
    let pod = format!("{}standin", prefix(who));
    let state = std::process::Command::new("podman")
        .args(["pod", "inspect", "--format", "{{.State}}", &pod])
        .output();
    state.is_ok_and(|o| {
        o.status.success()
            && matches!(
                String::from_utf8_lossy(&o.stdout).trim(),
                "Running" | "Degraded"
            )
    })
}

scene!(
    /// Planted, watched failing: the computer a move goes to starts from
    /// fresh state, not the copy it was given (apps/give/src/keep.rs
    /// `start`, `(Some(from), false) => run.restore(..)` made
    /// `run.remove()`), and the stand-in Cy opens on Bo's computer shows
    /// one start, not Cy's and Bo's.
    give_move
);
async fn give_move(engine: Engine) {
    let scene = Scene::new("give_move", engine).await;
    if let Some(why) = no_podman() {
        return scene.never_ran(&why);
    }
    let program = give_built().await;
    let mut browsers = Browsers::driven(
        "demo/runs/canon/2026-10-17-house/people.mjs",
        engine,
        &[("cy", Engine::Chromium), ("bo", Engine::Chromium)],
    )
    .await;

    // Elm: Cy makes it; Bo joins by its link. Cy brings Give, and each runs
    // its program for computers.
    let (cy, elm) = scene.founds("Cy", "Elm").await;
    let bo = scene.joins("Bo", &cy.invites(&elm).await, "Elm").await;
    let card = standin();
    let card = card.to_str().unwrap();
    let scratch = scene.scratch("cy-give");
    brings(&program, &cy, &scratch, "Elm", card).await;
    let removed = Removed(
        vec![
            keeps(&program, &cy, &scratch, card, None),
            keeps(&program, &bo, &scene.scratch("bo-give"), card, None),
        ],
        &["cy", "bo"],
    );

    // Cy starts it for the group; Bo opens it on Cy's computer.
    let cys = give_at(&cy, &elm).await;
    let bos = give_at(&bo, &elm).await;
    steps(
        &mut browsers,
        "cy",
        json!([
            { "goto": cys },
            { "poll": cys, "re": "Start it for the group", "within": 60_000, "every": 2_000 },
            { "click": "Start it for the group", "in": "Stand-in", "until": "Stand-in · kept by your computer" },
        ]),
    )
    .await;
    let on_cys = bo.lent_by(&elm, "Cy", "standin").await;
    let before = starts(&mut browsers, "bo", &on_cys).await;
    eprintln!("Bo opens the stand-in on Cy's computer: {before:?}");

    // Cy moves it to Bo's computer, and waits for Bo's yes.
    steps(
        &mut browsers,
        "cy",
        json!([
            { "goto": cys },
            { "poll": cys, "re": "Move it to…", "within": 60_000, "every": 2_000 },
            { "click": "Move it to…", "in": "Stand-in", "until": "Move Stand-in to:" },
            { "click": "Bo’s computer", "in": "Stand-in", "until": "Waiting for Bo to say yes\\." },
        ]),
    )
    .await;

    // Bo is asked, and keeps it here.
    steps(
        &mut browsers,
        "bo",
        json!([
            { "goto": bos },
            { "poll": bos, "re": "Cy wants to move Stand-in to your computer\\.", "within": 60_000, "every": 2_000 },
            { "click": "Keep it here", "in": "Stand-in", "until": "Moving to your computer…|Stand-in · kept by your computer" },
            { "poll": bos, "re": "Stand-in · kept by your computer", "within": 120_000, "every": 2_000 },
        ]),
    )
    .await;
    steps(
        &mut browsers,
        "cy",
        json!([{ "poll": cys, "re": "Stand-in · kept by Bo’s computer", "within": 60_000, "every": 2_000 }]),
    )
    .await;

    // Cy opens it through Cy's own svrngs, lent by Bo's computer now: the
    // start it had on Cy's, and the one on Bo's after it.
    lenders(&cy, &elm, &["Bo"]).await;
    let on_bos = cy.lent_by(&elm, "Bo", "standin").await;
    let after = starts(&mut browsers, "cy", &on_bos).await;
    eprintln!("Cy opens the stand-in on Bo's computer: {after:?}");
    assert_eq!(
        after.get(..before.len()),
        Some(&before[..]),
        "Bo's computer starts from exactly Cy's last copy: before {before:?}, after {after:?}"
    );
    assert_eq!(
        after.len(),
        before.len() + 1,
        "and one start more: {after:?}"
    );
    assert!(
        !runs("Cy"),
        "Cy's computer, which keeps it no more, runs nothing"
    );

    stop_cleanly(removed).await;
}
