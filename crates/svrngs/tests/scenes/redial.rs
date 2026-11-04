//! The grapevine scene's step 12 alone, with no browser (give-redial): Jo's
//! svrngs restarts, killed outright as a crash or an upgrade kills it, while
//! Sam's keeps a connection to its old self, and Sam writes at once. The
//! write reaches Jo's copy within the scene's bound, [`RESTARTED_WITHIN`],
//! not at Sam's next round a minute on. How long it took is printed, so
//! runs give a rate (demo/runs/redial/). `SCENES_TRACED` adds its filters to
//! each person's traces, as `iroh::_events=debug` adds iroh's path choices.

use std::time::{Duration, Instant};

use serde_json::json;

use crate::grapevine::RESTARTED_WITHIN;
use crate::harness::Engine;
use crate::people::{late, Person, Scene};

mod restarted {
    #[tokio::test(flavor = "multi_thread")]
    async fn alone() {
        crate::people::judged("redial-chromium".into(), super::restarted()).await
    }
}

/// Whether `who`'s copy of `group` holds `mark`.
async fn holds(who: &Person, group: &str, mark: &str) -> bool {
    let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
    log["ops"].to_string().contains(mark)
}

/// `mark` written to Sam's copy of `group`, as a program on Sam's computer
/// writes, and how long until Jo's copy holds it, up to `limit`.
async fn reached(
    sam: &Person,
    jo: &Person,
    group: &str,
    mark: &str,
    limit: Duration,
) -> Option<Duration> {
    let act = json!({ "op": "record", "payload": { "redial": mark } });
    let (status, said) = sam
        .post(&format!("/v1/rail/append?namespace={group}"), &act)
        .await;
    assert!(status < 300, "Sam's record takes the act: {status} {said}");
    let t0 = Instant::now();
    while !holds(jo, group, mark).await {
        if t0.elapsed() >= limit {
            return None;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Some(t0.elapsed())
}

async fn restarted() {
    let scene = Scene::new("redial", Engine::Chromium).await;
    let (sam, page) = scene.founds("Sam", "1400 Page").await;
    let link = sam.invites(&page).await;
    let jo = scene.joins("Jo", &link, "1400 Page").await;
    // Sam's round dials Jo and keeps the connection (iroh.rs `kept`).
    let before = reached(&sam, &jo, &page, "before", Duration::from_secs(60)).await;
    assert!(
        before.is_some(),
        "Sam's act reaches Jo before any restart; Sam's log is {}",
        sam.log().display()
    );

    let jo = scene.again(jo, &[]).await;
    // A round is 60 s (`DEFAULT_RING_SYNC_INTERVAL`): waited past it, so a
    // slow run says how slow.
    let took = reached(&sam, &jo, &page, "after", Duration::from_secs(70)).await;
    eprintln!("redial: Sam's write reached Jo, restarted, in {took:?}");
    if took.is_none_or(|t| t > RESTARTED_WITHIN) {
        late(format!(
            "Sam's write is not in Jo's copy {RESTARTED_WITHIN:?} after Jo restarted (took {took:?}); Sam's log is {}",
            sam.log().display()
        ))
        .await;
    }
    drop((sam, jo));
}
