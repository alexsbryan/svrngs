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

/// A scene played once, judged ([`people::judged`]) in its place,
/// `target/scenes/<scene>-chromium`, by the name the gate's "scenes" step
/// reads its tests by: each person in the engine their device is, a
/// computer's browser Chromium and a phone WebKit as an iPhone
/// (`Browsers::driven`, each person's engine). Until 2026-10-25 a scene was
/// played a second time with every computer in WebKit too; the operator:
/// each page in the engine it stands for, "we can't be so maximalist here".
macro_rules! scene {
    ($(#[$doc:meta])* $test:ident) => {
        $(#[$doc])*
        mod $test {
            use $crate::harness::Engine;
            #[tokio::test(flavor = "multi_thread")]
            async fn chromium() {
                let place = format!("{}-chromium", stringify!($test));
                $crate::people::judged(place, super::$test(Engine::Chromium)).await
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
// Grapevine's step 12 alone (redial.rs), out of the gate until give-redial
// (b) lands its remedy: at 15cacd50 it fails 1 run in 10 here, so every
// gate would carry it twice (demo/runs/redial/2026-10-27/NEEDS_HUMAN.md).
// mod redial;
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
// Ports (demo/factorio.md, demo/minecraft.md): each presses Play, bytes cross.
mod ports;
// Give, scene 1 (apps/give): Cy starts an app for the group, Bo opens it.
mod give;
// Give, scene 3: Cy moves it to Bo's computer, its state with it.
mod give_move;
// Give, scenes 4 and 5: Bo's computer gone, Dee keeps it; Bo's back.
mod give_gone;
