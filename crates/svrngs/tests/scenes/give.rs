//! **Give, scene 1** (give-app-keep; svrn-docs rings/current/GIVE.md §3
//! scene 1, §7a), with the stand-in card (apps/give/tests/standin): Cy and
//! Bo share Elm. Cy brings Give to Elm, `give page Elm`, and runs Give's
//! program for computers, `give keep`, beside svrngs. On Give's page, Cy
//! presses **Start it for the group** on the stand-in's row, which then
//! reads "kept by your computer"; Bo's reads "kept by Cy’s computer".
//! Cy's computer starts the app and lends it, and Bo opens it through his
//! own svrngs, its page showing the one start it has had.
//!
//! Give is an app on svrngs's surface (apps/give, which links no svrngs
//! crate): the scene runs its program as a person would, with this
//! build's svrngs on the PATH, and Give reaches svrngs only through its
//! record API, `svrngs share` and the group's folder. The app is the
//! stand-in, kept in rootless podman; where podman does not answer, the
//! scene is never-ran with why. Its pod and volume are named
//! `svrngs-test-*` and removed as the scene ends.
//!
//! The invariant it relies on: **an app is lent only by the computer the
//! group's record names to keep it** (apps/give/fold, the one fold the page
//! and the program both run), so every member reaches the one copy there
//! is.
//!
//! What it shares with Give's scenes 3 to 5 (give_move.rs, give_gone.rs)
//! is here: Give built, brought and run beside a person's svrngs, its pods
//! removed, and the steps pressed on its page.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::process::{Child, Command};

use crate::harness::{repo, Engine};
use crate::people::{late, Browsers, Person, Scene};

/// What the scene names the stand-in's pod and volume by on `who`'s
/// computer: the two share this host's podman.
pub(crate) fn prefix(who: &str) -> String {
    format!(
        "svrngs-test-scene-{}-{}-",
        std::process::id(),
        who.to_lowercase()
    )
}

/// The stand-in card's folder.
pub(crate) fn standin() -> PathBuf {
    repo().join("apps/give/tests/standin")
}

/// Give's program, built from this commit, as `cargo build -p give` leaves
/// it: where it is.
pub(crate) async fn give_built() -> PathBuf {
    let built = Command::new(env!("CARGO"))
        .args(["build", "--quiet", "-p", "give"])
        .current_dir(repo())
        .status()
        .await
        .expect("cargo runs");
    assert!(built.success(), "give builds");
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("../debug/give")
}

/// `give <args>` on `who`'s computer, beside the svrngs running there: this
/// build's svrngs first on the PATH, as an install leaves it. Its svrngs
/// is named by `SVRNGS_DIR`, which give and svrngs's own command read
/// first, and its `HOME` is this host's: podman finds its machine through
/// the person's own home (on a Mac, `~/.config/containers`), and with a
/// person's HOME of the scene's it answered "unable to connect to Podman
/// socket" (target/scenes/give-chromium/cy-give/keep.log, 2026-10-26).
pub(crate) fn run_give(program: &Path, who: &Person, scratch: &Path, args: &[&str]) -> Command {
    let svrngs = Path::new(env!("CARGO_BIN_EXE_svrngs")).parent().unwrap();
    let path = format!(
        "{}:{}",
        svrngs.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut c = Command::new(program);
    c.args(args)
        .env_clear()
        .env("HOME", std::env::var_os("HOME").unwrap_or_default())
        .env("SVRNGS_DIR", who.home.join(".svrngs"))
        .env("USER", who.name.to_lowercase())
        .env("PATH", path)
        .env("GIVE_DIR", scratch.join("give"))
        .env("GIVE_TEST_PREFIX", prefix(&who.name))
        .env("RUST_LOG", "give=debug")
        .kill_on_drop(true);
    c
}

/// The scene's pods and volumes removed, on each of the computers it
/// names, and each program stopped, however the scene ends.
pub(crate) struct Removed(pub Vec<Child>, pub &'static [&'static str]);

impl Drop for Removed {
    fn drop(&mut self) {
        for keeper in &mut self.0 {
            let _ = keeper.start_kill();
        }
        for who in self.1 {
            remove(&format!("{}standin", prefix(who)));
        }
    }
}

/// The pod `pod` and its volumes, removed.
pub(crate) fn remove(pod: &str) {
    let podman = |args: &[&str]| std::process::Command::new("podman").args(args).output();
    if podman(&["pod", "exists", pod]).is_ok_and(|o| o.status.success()) {
        let _ = podman(&["pod", "stop", "--time", "0", pod]);
        let _ = podman(&["pod", "rm", pod]);
    }
    for v in ["data", "data-snapshot"] {
        let _ = podman(&["volume", "rm", "--force", &format!("{pod}-{v}")]);
    }
}

/// `who` runs Give's program for computers, `give keep`, beside svrngs,
/// copying what it keeps each `every` seconds where given (Give's
/// `GIVE_TEST_EVERY`), else as often as a person's does. Its log is
/// appended to, so a program run again keeps what it said before.
pub(crate) fn keeps(
    program: &Path,
    who: &Person,
    scratch: &Path,
    card: &str,
    every: Option<u64>,
) -> Child {
    std::fs::create_dir_all(scratch).unwrap();
    let log = (std::fs::OpenOptions::new().create(true).append(true))
        .open(scratch.join("keep.log"))
        .unwrap();
    let mut run = run_give(program, who, scratch, &["keep", "--card", card]);
    if let Some(every) = every {
        run.env("GIVE_TEST_EVERY", every.to_string());
    }
    run.stdin(Stdio::null())
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .spawn()
        .expect("give runs")
}

/// The steps the run's driver presses on `who`'s one page (house_rules.rs
/// `steps`): each step's words; a step that fails fails the scene.
pub(crate) async fn steps(browsers: &mut Browsers, who: &str, steps: Value) -> Vec<Value> {
    let got = browsers
        .tell(json!({ "do": "steps", "who": who, "steps": steps }))
        .await;
    let done = got["done"].as_array().cloned().unwrap_or_default();
    let last = done.last().cloned().unwrap_or(got.clone());
    let error = last["error"].as_str().unwrap_or_default();
    if got["failed"] == true
        && (error.contains(" ms") && error.contains("within") || error.contains("Timeout"))
    {
        late(format!(
            "{who}'s steps did not go as the scene says within their bound: {last:#}"
        ))
        .await;
    }
    assert!(
        got["failed"] != true && got["errors"].as_array().is_none_or(|e| e.is_empty()),
        "{who}'s steps did not go as the scene says: {last:#}; the page's errors: {:#}",
        got["errors"]
    );
    done
}

/// Where `who`'s browser opens their own computer's copy of Give, as their
/// home page lists it: a write is signed by the computer the page is open
/// on (crates/svrngs-node/src/carried.rs `acts`), so Cy starts it there.
pub(crate) async fn give_at(who: &Person, group: &str) -> String {
    let people = who.get(&format!("/v1/rail/people?namespace={group}")).await;
    let people = people["people"].as_array().cloned().unwrap_or_default();
    let key = (people.iter())
        .find(|p| p["person"] == who.name.as_str() && p["device"] == "computer")
        .map(|p| p["key"].clone())
        .expect("the person's computer is in the group");
    for _ in 0..120 {
        let lends = who.get(&format!("/v1/lends?namespace={group}")).await;
        let lends = lends["lent"].as_array().cloned().unwrap_or_default();
        let own = lends
            .iter()
            .find(|l| l["kind"] == "give" && l["key"] == key);
        if let Some(at) = own.and_then(|l| l["origin"].as_str()) {
            return at.to_string();
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    late(format!(
        "{} lists no Give of its own within 60 s; its log is {}",
        who.name,
        who.log().display()
    ))
    .await
}

scene!(
    /// Planted, watched failing: the program lends what the record names
    /// another computer for (apps/give/src/keep.rs `plan`, its `k.keeper
    /// == me` dropped), and Bo's computer, which keeps nothing, lends the
    /// stand-in too.
    give
);
/// Why no app can be kept on this host, or none when podman answers.
pub(crate) fn no_podman() -> Option<String> {
    let ready = std::process::Command::new("podman")
        .args(["info", "--format", "{{.Host.Security.Rootless}}"])
        .output();
    match ready {
        Ok(o) if o.status.success() => None,
        Ok(o) => Some(format!(
            "podman does not answer here: {}",
            String::from_utf8_lossy(&o.stderr).trim()
        )),
        Err(e) => Some(format!(
            "podman is not installed here ({e}), so no app can be kept"
        )),
    }
}

/// `who` brings Give to `group` (`give page <group>`), as its README says
/// anyone does once.
pub(crate) async fn brings(program: &Path, who: &Person, scratch: &Path, group: &str, card: &str) {
    std::fs::create_dir_all(scratch).unwrap();
    let brought = run_give(program, who, scratch, &["page", group, "--card", card])
        .output()
        .await
        .unwrap();
    let said = String::from_utf8_lossy(&brought.stdout);
    assert!(
        brought.status.success() && said.contains(&format!("Give is in {group}")),
        "{} brings Give to {group}: {said}{}",
        who.name,
        String::from_utf8_lossy(&brought.stderr)
    );
}

/// Each program, asked to stop, stops cleanly, with a last copy of what
/// it keeps; then the pods and volumes removed.
pub(crate) async fn stop_cleanly(mut removed: Removed) {
    for mut keeper in std::mem::take(&mut removed.0) {
        let id = keeper.id().unwrap().to_string();
        assert!(std::process::Command::new("kill")
            .args(["-TERM", &id])
            .status()
            .unwrap()
            .success());
        let stopped = tokio::time::timeout(Duration::from_secs(120), keeper.wait()).await;
        assert!(
            stopped.is_ok_and(|s| s.is_ok_and(|s| s.success())),
            "each program stops cleanly"
        );
    }
    drop(removed);
}

async fn give(engine: Engine) {
    let scene = Scene::new("give", engine).await;
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

    // Elm: Cy makes it; Bo joins by its link.
    let (cy, elm) = scene.founds("Cy", "Elm").await;
    let bo = scene.joins("Bo", &cy.invites(&elm).await, "Elm").await;
    let card = standin();
    let card = card.to_str().unwrap();

    // Cy brings Give to Elm; Cy and Bo each run its program for computers.
    let scratch = scene.scratch("cy-give");
    brings(&program, &cy, &scratch, "Elm", card).await;
    let removed = Removed(
        vec![
            keeps(&program, &cy, &scratch, card, None),
            keeps(&program, &bo, &scene.scratch("bo-give"), card, None),
        ],
        &["cy", "bo"],
    );

    // Cy starts the stand-in for the group, on Cy's computer's own Give.
    let at = give_at(&cy, &elm).await;
    let poll = |re: &str| json!({ "poll": at.clone(), "re": re, "within": 60_000, "every": 2_000 });
    steps(
        &mut browsers,
        "cy",
        json!([
            { "goto": at },
            poll("Start it for the group"),
            { "click": "Start it for the group", "in": "Stand-in", "until": "kept by" },
            { "wait": "Stand-in · kept by your computer", "within": 10_000 },
        ]),
    )
    .await;

    // Bo reads who keeps it, and opens it through his own svrngs.
    let bos = give_at(&bo, &elm).await;
    steps(
        &mut browsers,
        "bo",
        json!([{ "goto": bos }, { "poll": bos, "re": "Stand-in · kept by Cy’s computer", "within": 60_000, "every": 2_000 }]),
    )
    .await;
    let lent = bo.lent_by(&elm, "Cy", "standin").await;
    let starts = format!("{}starts", lent.trim_end_matches('/').to_string() + "/");
    let done = steps(
        &mut browsers,
        "bo",
        json!([{ "goto": starts }, { "poll": starts, "re": r"\d{2}:\d{2}:\d{2}", "within": 180_000, "every": 2_000 }]),
    )
    .await;
    eprintln!(
        "Bo opens the stand-in Cy's computer keeps: {:#}",
        done.last().cloned().unwrap_or_default()
    );

    // Only Cy's computer lends it: Bo's lists no stand-in of its own.
    let lends = bo.get(&format!("/v1/lends?namespace={elm}")).await;
    let lenders: Vec<_> = (lends["lent"].as_array().into_iter().flatten())
        .filter(|l| l["name"] == "standin")
        .map(|l| l["person"].clone())
        .collect();
    assert_eq!(
        lenders,
        [json!("Cy")],
        "one copy, lent by the computer that keeps it: {lends:#}"
    );
    let bos_pod = format!("{}standin", prefix("Bo"));
    let runs = std::process::Command::new("podman")
        .args(["pod", "exists", &bos_pod])
        .status()
        .unwrap();
    assert!(
        !runs.success(),
        "Bo's computer, which keeps nothing, runs nothing"
    );

    // Each program, asked to stop, stops cleanly: Cy's with a last copy.
    stop_cleanly(removed).await;
}
