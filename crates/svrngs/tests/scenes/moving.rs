//! **Moving an old group** (EUDOXIA_JUDGE §2e §4): Sam's Pier was made
//! before groups were founded sources-v1, and Ana joins it by its link.
//! Sam's home page offers Move Pier, and he presses it: every computer in
//! Pier reads the new lines, gets ready, and Sam's signs the move. From
//! then on a hand-on waits: `svrngs handover Ana` says Ana decides once she
//! accepts it, where the old way handed it on at once.
//!
//! The invariant it relies on: **a group is on exactly one decider at a
//! time, and its keeper's own press moves it** (§2a, §2e rule 4): before
//! the move a hand-on passes at once, after it the recipient's yes is
//! needed, on every computer alike.

use std::sync::Arc;

use serde_json::{json, Value};

use crate::harness::{relay_url, Engine};
use crate::people::{late, Browsers, Scene};

const GROUP: &str = "Pier";

/// `who`'s steps on their one page, as keeping.rs runs them.
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

/// Sam's computer as an older svrngs left it: his key, and Pier made with
/// no policy in its seed, meeting at the scene's relay.
fn sams_old_pier(scene: &Scene, at: &str) {
    let data = scene.scratch("sam").join(".svrngs");
    let key = svrngs::identity::load_or_generate_node_key(&data).unwrap();
    let name = |n: &str| svrngs_node::Name::parse(n).unwrap();
    let seed =
        svrngs_node::seed::Seed::from_parts(name(GROUP), name("Sam"), key.verifying_key(), [7; 16]);
    assert_eq!(seed.policy(), None, "an old group");
    let rail = svrngs_node::RingRail::new(&data, Arc::new(key));
    let dir = rail.group_dir(&seed.id()).unwrap();
    svrngs_node::group::write_seed(&dir, &seed).unwrap();
    rail.set_seed(&seed.id(), &seed.roster()).unwrap();
    let meets = [svrngs_core::relay::parse(at).unwrap()].into();
    rail.commit(&seed.id(), &svrngs_node::MeetsAt(meets))
        .unwrap()
        .unwrap();
}

scene!(
    /// Planted, watched failing: svrngs-core legacy.rs `decider` made to
    /// return the old fold for every Adopt, reverted after: Sam's press
    /// said Pier cannot move.
    moving
);
async fn moving(engine: Engine) {
    let scene = Scene::new("moving", engine).await;
    let (page, _served) = scene.page().await;
    let mut browsers = Browsers::driven(
        "demo/runs/canon/2026-10-17-house/people.mjs",
        engine,
        &[("sam", Engine::Chromium)],
    )
    .await;
    let at = relay_url(&scene.relay);

    // 1. Sam's old Pier, and Ana joins it by its link.
    scene.installs_from("Sam", &page);
    sams_old_pier(&scene, &at);
    let sam = scene.start("Sam", &[]).await;
    let group = sam.group_named(GROUP).await;
    scene.installs_from("Ana", &page);
    let _ana = scene.joins("Ana", &sam.invites(&group).await, GROUP).await;

    // 2. Sam's home page offers to move Pier, and he presses Move.
    steps(
        &mut browsers,
        "sam",
        json!([
            { "goto": sam.home_page() },
            { "wait": "Pier keeps who decides removals the way svrngs first did", "within": 30_000 },
            { "click": "Move Pier", "until": "Pier moved\\. From now on, handing on the order of removals waits until the other person accepts it", "within": 90_000 },
        ]),
    )
    .await;
    let said = scene.command(&sam, &["adopt", GROUP], "\n").await;
    assert!(said.contains("Pier moved"), "{said}");

    // 3. A hand-on now waits for Ana's yes.
    let said = scene
        .command(&sam, &["handover", "Ana", "--group", GROUP], "\n")
        .await;
    assert!(
        said.contains("Ana decides who is removed from Pier once they accept it"),
        "{said}"
    );
}
