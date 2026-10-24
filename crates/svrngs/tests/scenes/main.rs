//! The demo scenes, played (capability-scenes; the operator, 2026-10-23:
//! "Part of what keeps the platform accountable is that it keeps
//! supporting these use cases with hi fidelity"): each scene of `demo/`
//! end to end, every person a `svrngs` of this build with a `HOME` of
//! their own ([`people`]), in the scene's group, through the real door,
//! relay and record, in Playwright's Chromium and in its WebKit as an
//! iPhone. Each scene says, in its own words, what each person sees, and
//! the one invariant it relies on. `scripts/check.sh` runs this binary
//! in its "scenes" step, each scene's verdict on its own line.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
#[path = "../../../svrngs-node/tests/browser/drive.rs"]
mod drive;
// The host's load and the film's seek bar under it, the browser tests' own.
#[path = "../../../svrngs-node/tests/browser/load.rs"]
#[allow(dead_code)]
mod load;
mod people;

/// A scene in Chromium and in WebKit, as `in_both_engines!` plays a
/// browser test, each judged ([`people::judged`]) in its place,
/// `target/scenes/<scene>-<engine>`, by the name the gate's "scenes" step
/// reads its tests by.
macro_rules! scene {
    ($(#[$doc:meta])* $test:ident) => {
        $(#[$doc])*
        mod $test {
            use $crate::harness::Engine;
            async fn judged(engine: Engine) {
                let place = format!("{}-{}", stringify!($test), engine.as_str());
                $crate::people::judged(place, super::$test(engine)).await
            }
            #[tokio::test(flavor = "multi_thread")]
            async fn chromium() {
                judged(Engine::Chromium).await
            }
            #[tokio::test(flavor = "multi_thread")]
            async fn webkit() {
                judged(Engine::Webkit).await
            }
        }
    };
}

// Money (apps/expenses): a split, a settle, another's correction.
mod money;
// Events (demo/events.md): a dinner for two groups, each its own answers.
mod events;
// House rules (demo/house-rules.md): a rule agreed holds after one leaves.
mod house_rules;
// Grapevine (demo/grapevine.md): an ask carried a ring away, sealed replies.
mod grapevine;
// Doc (apps/doc): two editors converge, each line noted as its writer's.
mod doc;
// The group's life (demo/c.md): a link, a phone, a removal, a lid shut.
mod life;
// Films (demo/a.md): two people play and seek Sam's lent Jellyfin at once.
mod films;
// Movie night (demo/b.md): a newcomer from the link to the film on a TV.
mod night;
// The pantry (apps/pantry): friends' models and tools, found by protocol.
mod pantry;
