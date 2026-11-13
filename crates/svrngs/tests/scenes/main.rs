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
// The host's load, judged as the browser tests judge it (people.rs `late`).
#[path = "../../../svrngs-node/tests/browser/load.rs"]
#[allow(dead_code)]
mod load;
mod people;
// Each person's svrngs, this binary run as `main` meeting at a relay on
// 127.0.0.1 (people.rs `program`; docs/CHECKS.md harness 8).
#[path = "../common/as_main.rs"]
mod as_main;
// The one helper that proves a request as home/session-worker.js does,
// for what a scene writes before a person's browser arrives (people.rs).
#[path = "../../../svrngs-node/tests/common/mod.rs"]
#[allow(dead_code)]
mod common;

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
// A beginner's folder app: private trial, sharing, phone write, update and removal.
mod creator;
// Events (demo/events.md): a dinner for two groups, each its own answers.
mod events;
// House rules (demo/house-rules.md): a rule agreed holds after one leaves.
mod house_rules;
// Grapevine (demo/grapevine.md): an ask carried a ring away, sealed replies.
mod grapevine;
// Grapevine's step 12 alone (redial.rs): a computer restarted while another
// keeps a connection to it, reached within a restart's bound.
mod redial;
// Doc (apps/doc): two editors converge, each line noted as its writer's.
mod doc;
// The group's life (demo/c.md): a link, a phone, a removal, a lid shut.
mod life;
// Keeping a group handed on: a hand-on, its Accept on a home page, a leave.
mod keeping;
// Films (demo/a.md): two people stream Sam's lent Jellyfin at once, each as themselves.
mod films;
// Movie night (demo/b.md): a newcomer from the link to the film on a TV.
mod night;
// Movie Night (apps/movie-night): a newcomer searches a friend's films.
mod movie_night;
// HolyArrs: request, approval, real download/import, discovery and the stream.
mod holyarrs;
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

/// svrngs itself, as each person's computer runs it ([`svrngs`]).
#[test]
#[ignore = "run by a scene's people, each in a process of its own, as svrngs"]
fn svrngs_here() {
    as_main::main()
}

/// The scenes' svrngs (people.rs `program`): a program named `svrngs` that
/// runs [`svrngs_here`] at this binary's build relay, made once
/// (as_main.rs `installed_at`). A script, so the relay is named whatever
/// environment a scene clears, and give and HolyArrs's driver run the same
/// svrngs, off the `PATH` or by its path. Then libtest's two lines, which
/// it prints before svrngs's own.
///
/// The build relay is the browser harness's (harness.rs `relay`), on
/// 127.0.0.1 behind its TLS front, not as_main.rs's plain one: a link
/// carries its issuer's home relay first (svrngs-core relay.rs `carried`),
/// and a phone's page over TLS reaches a relay over TLS alone. At
/// as_main.rs `build_relay`, every phone that joined failed ("A TLS error
/// caused the secure connection to fail", creator.rs:155, events.rs:134),
/// as their computers' home relay had been n0's until then.
fn svrngs() -> (std::path::PathBuf, &'static [&'static str]) {
    static MADE: std::sync::OnceLock<(tempfile::TempDir, std::path::PathBuf)> =
        std::sync::OnceLock::new();
    let (_, svrngs) = MADE.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
                let relay = harness::relay().await;
                tx.send(harness::relay_url(&relay)).unwrap();
                std::future::pending::<()>().await;
            });
        });
        let relay = rx.recv().expect("the build's relay said where it is");
        let dir = tempfile::tempdir().unwrap();
        let svrngs = as_main::installed_at(dir.path(), &relay);
        (dir, svrngs)
    });
    (svrngs.clone(), &as_main::HEADER)
}
