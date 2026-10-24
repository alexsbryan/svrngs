//! **The group's life** (demo/c.md; its run, demo/runs/c/dry-2026-10-12-
//! rerun/): Sam makes lake-trip on the group's page's address and lends
//! his films; Ana joins by its link, lends her music and puts the trip's
//! thirty photos in the group's folder on her disk. Alex, Bo and Dee join
//! one at a time: each home page lists the group, its folder and both
//! lends, Sam's films open from it, and the trip is on their disk. Cy
//! joins on a phone, through the group's page, and opens Sam's films
//! there; Alex, who joined by the client's link, adds his phone from his
//! home page. Sam removes Bo. Ana's computer stops, as a lid shut; Eve joins
//! and gets every photo all the same, from whoever holds them.
//!
//! Sam's films and Ana's music here are a page each served on a port, not
//! Jellyfin and Navidrome: svrngs lends any app
//! on a port, and the films scenes (demo/a.md, demo/b.md) play Jellyfin
//! where it is installed.
//!
//! The invariant it relies on: **a removal cuts the removed person off
//! everywhere, and nothing new reaches them** (demo/c.md, "Removed"):
//! within 30 s of Sam's `svrngs remove Bo`, Sam's films and Ana's music
//! each refuse Bo with svrngs's refusal, Bo's home page says he is no
//! longer in lake-trip, and a photo Sam puts in the folder after reaches
//! every member's disk but Bo's.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::harness::{relay_url, Engine};
use crate::people::{late, Browsers, Person, Scene};

const GROUP: &str = "lake-trip";
const PHOTOS: usize = 30;

/// The trip's photo `n`, bytes of its own.
fn photo(n: usize) -> Vec<u8> {
    format!("lake-trip photo {n}\n").repeat(100).into_bytes()
}

/// The trip as it lies in `who`'s group folder.
fn trip(who: &Person) -> PathBuf {
    who.home.join("svrngs").join(GROUP).join("trip")
}

/// Every one of `names` on `who`'s disk with the bytes Ana put there,
/// within `within`; else the scene fails, naming what is missing.
async fn on_disk(who: &Person, names: &[(String, Vec<u8>)], within: Duration) -> Duration {
    let t0 = Instant::now();
    loop {
        let missing: Vec<_> = names
            .iter()
            .filter(|(n, bytes)| std::fs::read(trip(who).join(n)).ok().as_ref() != Some(bytes))
            .map(|(n, _)| n.clone())
            .collect();
        if missing.is_empty() {
            return t0.elapsed();
        }
        let lacks = || {
            format!(
                "{}'s disk lacks {} of the trip after {within:?}: {missing:?}; its log is {}",
                who.name,
                missing.len(),
                who.log().display()
            )
        };
        // Held at once, no wait: a plain assertion under any load.
        assert!(!within.is_zero(), "{}", lacks());
        if t0.elapsed() >= within {
            late(lacks()).await;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

/// An app of `who`'s on a port, one page saying whose `name` it is, lent
/// as `svrngs share <port> --kind <name>` lends it: the app, killed on drop.
async fn lends(scene: &Scene, who: &Person, name: &str) -> tokio::process::Child {
    let words = format!("{}’s {name}", who.name);
    let page = format!("<!doctype html><meta charset=utf-8><title>{words}</title><h1>{words}</h1>");
    let app = "const [page] = process.argv.slice(1); \
        const s = require('node:http').createServer((q, r) => r.writeHead(200, { 'content-type': 'text/html' }).end(page)); \
        s.listen(0, '127.0.0.1', () => console.log(s.address().port));";
    let mut served = tokio::process::Command::new("node")
        .args(["-e", app, &page])
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs");
    use tokio::io::AsyncBufReadExt;
    let mut printed = tokio::io::BufReader::new(served.stdout.take().unwrap()).lines();
    let port = printed
        .next_line()
        .await
        .unwrap()
        .expect("the app prints its port");
    let shared = scene
        .command(who, &["share", &port, "--kind", name, "--name", name], "\n")
        .await;
    assert!(shared.contains(&format!("You lend {name}")), "{shared}");
    served
}

/// `who`'s home page, which reads its groups again each second, once it
/// says `words`, within `within` and people.mjs's 90 s: what it says.
async fn home_says(browsers: &mut Browsers, who: &Person, words: &str, within: Duration) -> String {
    let t0 = Instant::now();
    let url = who.home_page();
    let got = browsers
        .tell(json!({ "do": "goto", "who": who.name.to_lowercase(), "url": url, "wait": words }))
        .await;
    let text = got["text"].as_str().unwrap_or_default().to_string();
    if t0.elapsed() >= within || !text.contains(words) {
        let took = t0.elapsed();
        late(format!(
            "{}'s home page did not say {words:?} within {within:?}, after {took:?}: {text}",
            who.name
        ))
        .await;
    }
    text
}

/// A newcomer's three things: the home page lists the group, its folder
/// and both lends; Sam's films open from it, as `who`; the trip on disk.
async fn newcomer(browsers: &mut Browsers, who: &Person, trip: &[(String, Vec<u8>)]) {
    // Each waited for: the page reads its groups and lends again each
    // second, and at a load of 155 it listed Sam's films before the
    // group's folder.
    for words in ["Sam’s films", GROUP, "lake-trip’s folder", "Ana’s music"] {
        home_says(browsers, who, words, Duration::from_secs(60)).await;
    }
    let got = browsers
        .tell(json!({ "do": "follow", "who": who.name.to_lowercase(), "link": "Open Sam’s films" }))
        .await;
    assert_eq!(
        got["status"], 200,
        "{} opens Sam's films: {got:#}",
        who.name
    );
    assert!(
        got["text"]
            .as_str()
            .unwrap_or_default()
            .contains("Sam’s films"),
        "{} sees Sam's films: {got:#}",
        who.name
    );
    on_disk(who, trip, Duration::from_secs(120)).await;
}

/// `who`'s browser at `at`, a lend's address on their computer, until
/// svrngs refuses them as removed or `within` passes: what it said.
async fn refused(browsers: &mut Browsers, who: &Person, at: &str, within: Duration) -> Value {
    let t0 = Instant::now();
    loop {
        let got = browsers
            .tell(json!({ "do": "goto", "who": who.name.to_lowercase(), "url": at }))
            .await;
        if got["refused"] == "removed" {
            return got;
        }
        if t0.elapsed() >= within {
            late(format!(
                "{} is not refused as removed at {at} after {within:?}: {got:#}",
                who.name
            ))
            .await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

scene!(
    /// Planted, watched failing: a hand edit, the door's and the origin's
    /// refusals of a removed person taken out (door.rs `gone`'s two
    /// callers, door.rs:1082, origin.rs:275), reverted after.
    life
);
async fn life(engine: Engine) {
    let scene = Scene::new("life", engine).await;
    let (page, _served) = scene.page().await;
    let mut browsers = Browsers::open(
        engine,
        &[
            ("alex", Engine::Chromium),
            ("bo", Engine::Chromium),
            ("dee", Engine::Chromium),
            ("eve", Engine::Chromium),
            ("cy", Engine::Webkit),
            ("alexphone", Engine::Webkit),
        ],
    )
    .await;
    let at = relay_url(&scene.relay);

    // 1. Sam makes the group on the group's page's address, and lends his films.
    let sam = scene
        .start(
            "Sam",
            &[
                "new", GROUP, "--as", "Sam", "--origin", &page, "--relay", &at,
            ],
        )
        .await;
    let group = sam.group_named(GROUP).await;
    let links = sam
        .on_home_page("/link", &json!({ "group": group, "device": false }))
        .await;
    let link = links["client"].as_str().unwrap();
    let _films = lends(&scene, &sam, "films").await;

    // 2. Ana joins, lends her music, and puts the trip in the group's folder.
    let ana = scene.joins("Ana", link, GROUP).await;
    let _music = lends(&scene, &ana, "music").await;
    let photos: Vec<(String, Vec<u8>)> = (1..=PHOTOS)
        .map(|n| (format!("IMG_{n:04}.jpg"), photo(n)))
        .collect();
    std::fs::create_dir_all(trip(&ana)).unwrap();
    for (name, bytes) in &photos {
        std::fs::write(trip(&ana).join(name), bytes).unwrap();
    }

    // 3. Alex, Bo and Dee, one at a time; Cy on a phone.
    let mut newcomers = vec![];
    scene.installs_from("Alex", &page);
    for name in ["Alex", "Bo", "Dee"] {
        let who = scene.joins(name, link, GROUP).await;
        newcomer(&mut browsers, &who, &photos).await;
        newcomers.push(who);
    }
    let cy = browsers
        .tell(json!({ "do": "join", "who": "cy", "link": links["https"], "name": "Cy" }))
        .await;
    let status = cy["status"].as_str().unwrap_or_default();
    assert!(status.contains("in lake-trip"), "Cy's phone joins: {cy:#}");
    let cy = browsers
        .tell(json!({ "do": "lend", "who": "cy", "button": "Sam’s films" }))
        .await;
    assert!(
        cy["text"]
            .as_str()
            .unwrap_or_default()
            .contains("Sam’s films"),
        "Cy's phone opens Sam's films through the group's page: {cy:#}"
    );
    let [alex, bo, dee] = <[Person; 3]>::try_from(newcomers).ok().unwrap();
    // Alex joined by the client's link, so his computer keeps no page of
    // lake-trip's, but it came from the group's page by the install line:
    // he adds his phone from his home page, and it opens the group.
    let device = json!({ "group": group, "device": true });
    let his = alex.on_home_page("/link", &device).await;
    assert!(
        his["https"].is_string(),
        "Alex's home page gives a link his phone opens: {his:#}"
    );
    let phone = browsers
        .tell(json!({ "do": "join", "who": "alexphone", "link": his["https"], "name": "Alex" }))
        .await;
    let status = phone["status"].as_str().unwrap_or_default();
    assert!(
        status.contains("in lake-trip") && status.contains("Alex"),
        "Alex's phone opens lake-trip as Alex: {phone:#}"
    );
    let films_at_bo = bo.lent_by(&group, "Sam", "films").await;
    let music_at_bo = bo.lent_by(&group, "Ana", "music").await;

    // 4. Sam removes Bo.
    let said = scene
        .command(&sam, &["remove", "Bo", "--group", GROUP], "\n")
        .await;
    assert!(said.contains("Bo is no longer in lake-trip."), "{said}");
    let within = Duration::from_secs(30);
    for at in [&films_at_bo, &music_at_bo] {
        let got = refused(&mut browsers, &bo, at, within).await;
        let text = got["text"].as_str().unwrap_or_default();
        assert!(
            text.contains("You are no longer in this group"),
            "Bo is told he is out, in a person's words: {got:#}"
        );
        assert!(
            !text.contains("’s films") && !text.contains("’s music"),
            "{got:#}"
        );
    }
    home_says(
        &mut browsers,
        &bo,
        "You are no longer in lake-trip.",
        within,
    )
    .await;

    // Nothing new reaches Bo: a photo Sam puts in after reaches every
    // member's disk but his.
    let after = vec![("after.jpg".to_string(), photo(0))];
    std::fs::create_dir_all(trip(&sam)).unwrap();
    std::fs::write(trip(&sam).join("after.jpg"), &after[0].1).unwrap();
    for who in [&ana, &alex, &dee] {
        on_disk(who, &after, Duration::from_secs(60)).await;
    }
    assert!(
        !trip(&bo).join("after.jpg").exists(),
        "Sam's photo after the removal reached Bo's disk"
    );
    on_disk(&bo, &photos, Duration::ZERO).await;

    // 5. Ana's lid shuts; Eve joins and gets every photo from whoever
    // holds them, and her home page says Ana's computer did not answer.
    ana.stop().await;
    let eve = scene.joins("Eve", link, GROUP).await;
    let took = on_disk(&eve, &photos, Duration::from_secs(120)).await;
    eprintln!("Eve's disk held the trip {took:?} after she was in, Ana's computer stopped");
    let asleep = "Ana's computer did not answer";
    home_says(&mut browsers, &eve, asleep, Duration::from_secs(60)).await;
    let got = browsers
        .tell(json!({ "do": "follow", "who": "eve", "link": "Open Sam’s films" }))
        .await;
    assert_eq!(got["status"], 200, "Sam's films play on for Eve: {got:#}");
    drop((sam, alex, bo, dee, eve));
}
