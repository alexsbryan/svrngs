//! **Movie night** (demo/b.md; its dry run, demo/runs/b/2026-10-06/):
//! Sam makes House on the group's page's address, lends his Jellyfin as
//! `share` lends it, and makes one account for the room in Jellyfin's
//! pages, as the line `share` printed tells him. Dee, in House on a
//! computer, turns Sam's Jellyfin on for the room, `svrngs room
//! jellyfin`, which prints the address the room reaches it at. The TV, a
//! browser at that address, is signed in by Quick Connect from Dee's
//! phone, as the room. Then Ana, new to all of it, opens the link on her
//! phone, gives her name, opens Sam's Jellyfin from the list, signs in as
//! the room, picks a film, casts it to the TV and presses Play.
//!
//! Two scenes play the evening. `night`, in the scenes step, ends at the
//! TV asking the film's stream at the room's address and receiving it,
//! 200 or 206 with a video/* type (person.mjs `streamed`): what the room
//! owns, which no decoder's pace changes. `night_played`, in scripts/
//! check.sh's `playback` step and nowhere else, plays it: the film plays
//! on the TV within three minutes of the link, and Ana pauses and seeks it
//! there from her phone within 10 s. Jellyfin hears of the TV's play, and
//! acts on a pause or a seek of it, only once the TV's <video> has
//! started, so that is a claim of playback, judged with the others in one
//! step (demo/runs/flakes/2026-11-12): a bar missed on a host past load.rs
//! `BUSY` a core is could-not-judge (people.rs `late`).
//!
//! Sam's Jellyfin is the films check's scratch one (films.rs
//! [`Jellyfin`]), never-ran where Jellyfin is not installed; the TV and
//! the phones are that check's, demo B's dry run's own driver
//! (apps/jellyfin/check/night.mjs, its parts `tv` and `five`).
//!
//! The invariant it relies on: **a device in the room reaches a lend
//! through the computer it was opened on** (room.rs, "a device in the
//! room is this person's own"): the TV is in no group and holds no key,
//! and receives Sam's film through Dee's computer; and nobody makes an
//! account: Jellyfin's users after are those Sam made (demo/b.md, "After").

use serde_json::{json, Value};
use svrngs_node::room::Served;

use crate::films::{node, Jellyfin};
use crate::harness::{relay_url, Engine};
use crate::people::{at_page_host, late, Scene};

/// `scheme://host:port` of the room's address `at`, as a browser writes a
/// response's origin.
fn origin(at: &str) -> &str {
    let host = at.find("://").map_or(0, |i| i + 3);
    at[host..].find('/').map_or(at, |end| &at[..host + end])
}

/// Chromecast's sender script, which jellyfin-web asks of Google on every
/// page it draws.
const CAST_SENDER: &str = "https://www.gstatic.com/cv/js/sender/v1/cast_sender.js";

/// Jellyfin's users by name, as Sam's own token reads them.
pub(crate) async fn users(jf: &Jellyfin) -> Vec<String> {
    let token = jf.read("secrets/admin_token");
    let auth = format!(
        r#"MediaBrowser Client="scenes", Device="Sam computer", DeviceId="scenes", Version="1", Token="{token}""#
    );
    let got = reqwest::Client::new()
        .get(format!("{}/Users", jf.at()))
        .header("authorization", auth)
        .send();
    let got = got.await.unwrap().bytes().await.unwrap();
    let got: Value = serde_json::from_slice(&got).unwrap();
    let names = got.as_array().into_iter().flatten();
    let mut names: Vec<String> = names
        .map(|u| u["Name"].as_str().unwrap_or_default().to_string())
        .collect();
    names.sort();
    names
}

scene!(
    /// Planted, watched failing: a hand edit, the room served on no
    /// network (room.rs `serves` answering `Served::Elsewhere`), reverted
    /// after. The network hidden (`serves` answering `Served::Unread`)
    /// is never-ran, with svrngs's words.
    night
);
async fn night(engine: Engine) {
    evening("night", engine, false).await
}

scene!(
    /// The evening played: the playback step's (scripts/check.sh
    /// `playback`, which runs `night_played::` alone, and the scenes step
    /// skips it).
    night_played
);
async fn night_played(engine: Engine) {
    evening("night_played", engine, true).await
}

/// The evening, as `name`: `played`, the film playing on the TV and Ana
/// pausing and seeking it; else up to the TV receiving its stream.
async fn evening(name: &str, engine: Engine, played: bool) {
    let scene = Scene::new(name, engine).await;
    let jf = match Jellyfin::up(scene.scratch("jellyfin")).await {
        Ok(jf) => jf,
        Err(why) => return scene.never_ran(&why),
    };
    let (page, _served) = scene.page().await;
    let at = relay_url(&scene.relay);

    // Before the evening. Sam makes House on the page's address, lends
    // Jellyfin, and makes the room its account.
    let sam = scene
        .start(
            "Sam",
            &[
                "new", "House", "--as", "Sam", "--origin", &page, "--relay", &at,
            ],
        )
        .await;
    let group = sam.group_named("House").await;
    let port = jf.port.to_string();
    let shared = scene
        .command(
            &sam,
            &["share", &port, "--kind", "jellyfin", "--group", "House"],
            "\n",
        )
        .await;
    let how = shared
        .lines()
        .find(|l| l.contains("asks each person to sign in once"))
        .unwrap_or_else(|| panic!("share tells Sam nothing of accounts: {shared}"));
    let secrets = jf.scratch.join("r/secrets");
    let secrets = ("SECRETS", secrets.to_string_lossy().to_string());
    let made = node(
        "lender.mjs",
        &[how, &jf.at(), "sam", "room"],
        &[secrets.clone()],
    )
    .await;
    assert!(
        made.get("stopped").is_none(),
        "Sam makes the room's account: {made:#}"
    );
    let before = users(&jf).await;

    // Dee joins on a computer and turns Sam's Jellyfin on for the room.
    let links = sam
        .on_home_page("/link", &json!({ "group": group, "device": false }))
        .await;
    let link = links["client"].as_str().unwrap();
    let dee = scene.joins("Dee", link, "House").await;
    // Her phone's link, made on her home page for her own devices. Her
    // computer keeps no page of House's, so her page gives the link alone;
    // on the group's page it is `<page>#<token>` (svrngs-core link.rs
    // `https_link`), as Sam's page gives his.
    let device = json!({ "group": group, "device": true });
    let hers = dee.on_home_page("/link", &device).await;
    let token = hers["client"]
        .as_str()
        .unwrap()
        .trim_start_matches("svrngs://join/");
    let hers = json!(format!("{page}#{token}"));
    dee.lent_by(&group, "Sam", "jellyfin").await;
    let said = scene.command(&dee, &["room", "jellyfin"], "").await;
    // The room is served on the network a computer reads (room.rs
    // `serves`): on a host whose network cannot be read, or that has no
    // address on it, the evening cannot be played, and is never-ran.
    for unserved in [Served::Unread, Served::NoAddress] {
        if said.contains(&unserved.said()) {
            return scene.never_ran(&format!(
                "this host's network cannot be served on: svrngs says {:?}",
                unserved.said()
            ));
        }
    }
    let line = said.lines().find(|l| l.starts_with("Sam's jellyfin: "));
    let room = line
        .and_then(|l| l.split_whitespace().find(|w| w.starts_with("http://")))
        .unwrap_or_else(|| panic!("Dee's computer opens Sam's jellyfin to the room: {said}"));
    let env = [
        secrets,
        ("JF", jf.at()),
        ("ADMIN", jf.read("secrets/admin_token")),
        (
            "DUMP",
            scene.scratch("received").to_string_lossy().to_string(),
        ),
    ];
    // The TV's profile, kept on disk as a TV's app keeps its sign-in; at a
    // path with `..` in it Chromium opened no store ("Internal error
    // opening backing store for indexedDB.open") and the TV forgot it.
    let profile = scene.scratch("tv");
    std::fs::create_dir_all(&profile).unwrap();
    let profile = profile.canonicalize().unwrap();
    // The page's links as a phone's browser reaches the page (people.rs
    // `at_page_host`).
    let args = |link: &Value, people: Value| {
        at_page_host(
            json!({
                "room": room, "link": link, "server": jf.env("SERVER"),
                "profile": profile, "account": "room", "people": people,
                "until": if played { Value::Null } else { json!("streamed") },
            })
            .to_string(),
        )
    };

    // The TV, at the room's address, signed in by Quick Connect from
    // Dee's phone, nothing typed on the TV.
    let tv = node("night.mjs", &["tv", &args(&hers, json!([]))], &env).await;
    eprintln!("the TV, before the evening: {tv:#}");
    assert!(
        tv.get("stopped").is_none() && tv["tv_signed_in_ms"].is_number(),
        "the TV is signed in as the room by Dee's phone: {tv:#}"
    );
    assert_eq!(
        tv["kept"]["signed_in_as"], "room",
        "the TV is in Jellyfin as the room: {tv:#}"
    );

    // The evening: Ana, from the link to her film on the TV.
    let device = match engine {
        Engine::Chromium => json!("Pixel 7"),
        Engine::Webkit => Value::Null,
    };
    let ana = json!([{ "name": "Ana", "user": "ana", "engine": engine.as_str(), "device": device, "film": "Test Pattern Two", "film_id": jf.env("TWO") }]);
    let night = node("night.mjs", &["five", &args(&links["https"], ana)], &env).await;
    eprintln!("the evening: {night:#}");
    assert!(night.get("stopped").is_none(), "{night:#}");
    let ana = &night["people"][0];
    assert!(
        ana.get("stopped").is_none_or(Value::is_null),
        "Ana gets from the link to the film on the TV: {ana:#}"
    );
    if played {
        if ana["within_3_min"] != true {
            late(format!(
                "Ana's film does not play on the TV within three minutes of the link: {ana:#}"
            ))
            .await;
        }
        if ana["pause_seek_within_10s"] != true {
            late(format!(
                "Ana does not pause and seek it on the TV from her phone within 10 s: {ana:#}"
            ))
            .await;
        }
    } else {
        // The TV received her film at the room's address, and asked nowhere
        // else: it reaches the lend through Dee's computer alone.
        assert!(
            ana["streamed"]["ms"].is_number(),
            "the TV receives Ana's film after her Play: {ana:#}"
        );
        assert_eq!(
            ana["streamed"]["origin"].as_str(),
            Some(origin(room)),
            "the TV's film comes from the room's address: {ana:#}"
        );
        // Off it, the one script jellyfin-web asks of Google on any page,
        // its Chromecast sender (the Mac, 2026-11-12, every run:
        // demo/runs/flakes/2026-11-12/mac/scenes.log): Jellyfin's page's own
        // ask, not a lend's, and nothing of Sam's Jellyfin.
        let elsewhere: Vec<&Value> = (night["asked"]["elsewhere"].as_array().into_iter())
            .flatten()
            .filter(|a| a["url"] != CAST_SENDER)
            .collect();
        assert!(
            elsewhere.is_empty(),
            "the TV asks nothing off the room's address: {night:#}"
        );
    }

    // Nobody made an account.
    assert_eq!(
        users(&jf).await,
        before,
        "Jellyfin's users are those Sam made"
    );
    drop((sam, dee, jf));
}
