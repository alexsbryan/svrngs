//! **Keeping a group handed on** (EUDOXIA_JUDGE §2a, release 1; §2c's
//! hand-on and Accept): Sam makes Pier, a group founded sources-v1 and
//! decided by its interpreter. Ana joins by its link, and her phone by her
//! home page's device link. Sam hands keeping Pier on to Ana with
//! `svrngs handover`: it waits for her, and Sam's home page says so, with
//! Take it back. Ana's home page asks her, and she presses Accept it: she
//! keeps the order of removals now. Then Ana leaves Pier, and the command
//! says plainly that nobody decides who is removed from it until the
//! others agree who does.
//!
//! The invariant it relies on: **the order of removals passes only with
//! its recipient's own yes, on their own computer** (§2b, "The new keeper
//! consents"): between Sam's hand-on and Ana's Accept nobody keeps Pier.
//! Forks and recoveries have no page or command to write them, so they
//! are judged at the core's level (svrngs-core `tests_sources_live.rs`).

use std::time::Duration;

use serde_json::{json, Value};

use crate::harness::{relay_url, Engine};
use crate::people::{late, Browsers, Scene};

const GROUP: &str = "Pier";

/// `who`'s steps on their one page, in the house-rules run's driver; a
/// step's wait that ran out fails the scene as late, any other error as
/// failed.
async fn steps(browsers: &mut Browsers, who: &str, steps: Value) -> Vec<Value> {
    let got = browsers
        .tell(json!({ "do": "steps", "who": who, "steps": steps }))
        .await;
    let done = got["done"].as_array().cloned().unwrap_or_default();
    let last = done.last().cloned().unwrap_or(got.clone());
    let error = last["error"].as_str().unwrap_or_default();
    if got["failed"] == true && (error.contains("within") || error.contains("Timeout")) {
        late(format!(
            "{who}'s steps did not go as this scene says: {last:#}"
        ))
        .await;
    }
    assert!(got["failed"] != true, "{who}'s steps: {got:#}");
    done
}

scene!(
    /// Planted, watched failing: a hand edit, the hand-on passing the
    /// office with no Accept (eudoxia-community history.rs, the walk's
    /// `self.accepted(..)` made `.or(Some(id.clone()))`), reverted after:
    /// Sam's `svrngs handover` said Ana decides at once.
    keeping
);
async fn keeping(engine: Engine) {
    let scene = Scene::new("keeping", engine).await;
    let (page, _served) = scene.page().await;
    let mut browsers = Browsers::driven(
        "demo/runs/canon/2026-10-17-house/people.mjs",
        engine,
        &[
            ("sam", Engine::Chromium),
            ("ana", Engine::Chromium),
            ("anaphone", Engine::Webkit),
        ],
    )
    .await;
    let at = relay_url(&scene.relay);

    // 1. Sam makes Pier; Ana joins by its link.
    let new = [
        "new", GROUP, "--as", "Sam", "--origin", &page, "--relay", &at,
    ];
    let sam = scene.start("Sam", &new).await;
    let group = sam.group_named(GROUP).await;
    scene.installs_from("Ana", &page);
    let ana = scene.joins("Ana", &sam.invites(&group).await, GROUP).await;

    // 2. Ana adds her phone from her home page: it joins Pier as her.
    let device = json!({ "group": group, "device": true });
    let link = ana.on_home_page("/link", &device).await;
    let phone = browsers
        .tell(json!({ "do": "join", "who": "anaphone", "link": link["https"], "name": "Ana" }))
        .await;
    let status = phone["status"].as_str().unwrap_or_default();
    assert!(
        status.contains("in Pier") && status.contains("Ana"),
        "Ana's phone joins Pier as Ana: {phone:#}"
    );

    // 3. Sam hands keeping Pier on to Ana: it waits for her yes.
    let said = scene
        .command(&sam, &["handover", "Ana", "--group", GROUP], "\n")
        .await;
    assert!(
        said.contains("Ana decides who is removed from Pier once they accept it"),
        "{said}"
    );
    let done = steps(
        &mut browsers,
        "sam",
        json!([
            { "goto": sam.home_page() },
            { "wait": "You handed the order of removals in Pier to Ana\\. Nobody keeps it until they accept it on their computer, or you take it back", "within": 30_000 },
            { "count": "#home button:has-text('Take it back')" },
        ]),
    )
    .await;
    assert_eq!(done[2]["value"], 1, "Sam can take it back: {done:#?}");

    // 4. Ana's home page asks her, and she accepts it.
    let within = Duration::from_secs(60).as_millis() as u64;
    steps(
        &mut browsers,
        "ana",
        json!([
            { "goto": ana.home_page() },
            { "wait": "Sam handed you the order of removals in Pier\\. It is yours once you accept it", "within": within },
            { "click": "Accept it", "until": "You keep the order of removals in Pier" },
            // Before she presses Leave, her page says what leaving keeping it does.
            { "wait": "If you leave without handing it on, nobody keeps the order of removals until the others agree on who does\\.", "within": 5_000 },
        ]),
    )
    .await;

    // 5. Ana leaves Pier, keeping it: nobody decides who is removed now.
    let said = scene.command(&ana, &["leave", GROUP], "\n").await;
    assert!(
        said.contains("You left Pier")
            && said.contains(
                "nobody decides who is removed from it now: the others agree together on who does"
            ),
        "{said}"
    );
    drop((sam, ana));
}
