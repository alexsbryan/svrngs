//! The one place a test asserts that a browser decodes a lent film: plays
//! it, plays again after each seek, and plays again once its worker was
//! stopped, in each engine (scripts/check.sh `playback`; demo/runs/flakes/
//! 2026-11-12). Every other test asserts what it owns of a film, its bytes
//! through the lend's worker (main.rs `film_answered`), which do not vary
//! with the host's load or its media stack; a decoder's stall under load
//! is said here alone. Run as a step of its own: `browser` skips
//! `playback::`, `playback` runs only it.
//!
//! A miss on a host whose load was past `load::BUSY` a core when the film
//! started is could-not-judge with that load, never a pass
//! (`target/could-not-judge/playback.txt`, which scripts/check.sh prints);
//! on a quieter host it fails.

use super::*;

/// A check the playback step could not judge, said on its output and added
/// to `target/could-not-judge/playback.txt`, which scripts/check.sh prints
/// after the step: never a pass (AGENTS.md 6).
fn unjudged(line: &str) {
    use std::io::Write;
    eprintln!("could-not-judge: {line}");
    let kept = repo().join("target/could-not-judge");
    std::fs::create_dir_all(&kept).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(kept.join("playback.txt"))
        .unwrap();
    writeln!(file, "{line}").unwrap();
}

in_both_engines!(
    /// A lent film plays under its worker (u4-6, u5-2; the runtime test's
    /// playing, moved here from main.rs, whose test now holds its bytes):
    /// ada's computer lends a page and a film, an H.264 mp4 from upstream's
    /// Range origin, and the phone, holding bo's key, opens it from its list
    /// at the lend's own origin, whose worker serves the `<video>` every byte
    /// range from ada's door. The film plays, and seeks three times, each
    /// playing again within five seconds (Demo A's seek bar, `seeks_judged`);
    /// the browser stops the worker before the third and the film plays on:
    /// Chromium stops it through DevTools, the page left playing, its worker
    /// running again and its endpoint opened again (`stopped_and_ran_again`);
    /// WebKit, whose worker Playwright cannot reach, is closed and opened
    /// again on its profile, and the film plays again at the lend's address.
    /// Every range the `<video>` asks reaches ada's door as a part of at
    /// most 2 MiB, naming bo's key: Chromium asks `bytes=N-`, WebKit
    /// `bytes=N-<the end>`, and WebKit read such a range through before it
    /// asked for a seek's bytes (ralph/REVIEW_FINDINGS.md #53). Planted: a
    /// `<video>`'s closed range to the end answered whole (b6ea0ab's); the
    /// film's ranges answered 200 whole, so no seek plays again in time.
    a_lent_film_plays_and_seeks_under_its_worker
);
async fn a_lent_film_plays_and_seeks_under_its_worker(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &bo).await;

    let mp4 = dir.path().join("title.mp4");
    let size = film(&mp4);
    let (film_origin, film_heads) = origins::kept_range_origin(mp4, size).await;
    let html = "<!doctype html><title>a page</title>\
                <video src=\"title.mp4\" playsinline preload=\"auto\"></video>";
    let page = one_origin(page_origin(html).await, film_origin).await;
    lend(dir.path(), "page", page.port());

    // The host's load as the film starts, which every miss is judged at.
    let began = load::load().await;
    let seen = drive(
        engine,
        "runtime.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "seeks": [40.0, 20.0, 52.0],
        }),
        None,
    )
    .await;
    let mut shown = seen.clone();
    shown["logs"] = json!(format!(
        "{} lines",
        seen["logs"].as_array().map_or(0, Vec::len)
    ));
    eprintln!("the film's run: {shown}");
    assert!(at_origin(&seen["opened"], &label(1, "page")), "{shown:#}");

    // The first play, the play again after the browser was opened again
    // (WebKit's), each a `play` that answers its ms within its 15 s; and the
    // seeks, each within five seconds (`seeks_judged`). Any miss on a host
    // past `load::BUSY` a core when the film started could not be judged.
    let name = format!(
        "a_lent_film_plays_and_seeks_under_its_worker::{}",
        engine.as_str()
    );
    let mut plays = vec![("first", &seen["first"])];
    if matches!(engine, Engine::Webkit) {
        assert_eq!(seen["reopened"], seen["opened"], "{shown:#}");
        plays.push(("once the browser was opened again", &seen["replayed"]));
    }
    let seeks = seen["seeks"].as_array().cloned().unwrap_or_default();
    assert_eq!(seeks.len(), 3, "{shown:#}");
    let cores = load::cores();
    let mut missed: Vec<String> = (plays.iter())
        .filter(|(_, p)| !p["ms"].is_u64())
        .map(|(when, p)| format!("the play {when}: {p}"))
        .collect();
    match seeks_judged(&seeks, &began, cores) {
        Ok(None) => {}
        Ok(Some(_)) => missed.push("a seek past five seconds".into()),
        Err(seeks) => missed.push(format!("the seeks {seeks}")),
    }
    if !missed.is_empty() {
        match load::past_busy(&began, cores) {
            Some(busy) => unjudged(&format!(
                "{name} plays, and each seek within five seconds: {busy}, the host at \
                 {began}; missed {}",
                missed.join("; ")
            )),
            None => panic!(
                "the film must play, and each seek play again within five seconds, the host \
                 at {began}; missed {}:\n{seen:#}",
                missed.join("; ")
            ),
        }
    }
    if matches!(engine, Engine::Chromium) {
        stopped_and_ran_again(&seen, &shown);
    }

    // What the `<video>` asked, as ada's door passed it on.
    assert!(
        film_heads
            .lock()
            .unwrap()
            .iter()
            .any(|h| h.to_ascii_lowercase().contains("\r\nrange: bytes=")),
        "the film's ranges must reach its service:\n{:#?}",
        film_heads.lock().unwrap()
    );
    let film_keys = keys(&film_heads);
    assert!(
        film_keys.iter().all(|k| *k == actor_of(&bo)),
        "ada's door must name bo's key: {film_keys:?}"
    );
    let asked = ranges(&film_heads);
    let long: Vec<&(u64, Option<u64>)> = asked
        .iter()
        .filter(|(from, to)| to.is_none_or(|to| to + 1 - from > 2 * MIB))
        .collect();
    assert!(
        long.is_empty(),
        "the film's ranges must reach its door as parts of at most 2 MiB, of {size} bytes; \
         these ran on (from, to): {long:?}\nall: {asked:?}"
    );
    drop(ada);
}
