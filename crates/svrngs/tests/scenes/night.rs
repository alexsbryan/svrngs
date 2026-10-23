//! **Movie night** (demo/b.md; its dry run, demo/runs/b/2026-10-06/):
//! Sam makes House on the group's page's address, lends his Jellyfin as
//! `share` lends it, and makes one account for the room in Jellyfin's
//! pages, as the line `share` printed tells him. Dee, in House on a
//! computer, turns Sam's Jellyfin on for the room, `svrngs room
//! jellyfin`, which prints the address the room reaches it at. The TV, a
//! browser at that address, is signed in by Quick Connect from Dee's
//! phone, as the room. Then Ana, new to all of it, opens the link on her
//! phone, gives her name, opens Sam's Jellyfin from the list, signs in as
//! the room, picks a film, casts it to the TV, and pauses and seeks it
//! there from her phone.
//!
//! Sam's Jellyfin is the films check's scratch one (films.rs
//! [`Jellyfin`]), never-ran where Jellyfin is not installed; the TV and
//! the phones are that check's, demo B's dry run's own driver
//! (apps/jellyfin/check/night.mjs, its parts `tv` and `five`).
//!
//! The invariant it relies on: **a device in the room reaches a lend
//! through the computer it was opened on** (room.rs, "a device in the
//! room is this person's own"): the TV is in no group and holds no key,
//! and plays Sam's film through Dee's computer; and nobody makes an
//! account: Jellyfin's users after are those Sam made (demo/b.md, "After").

use std::path::Path;

use serde_json::{json, Value};

use crate::films::{node, Jellyfin, JELLYFIN};
use crate::harness::{relay_url, Engine};
use crate::people::Scene;

/// Jellyfin's users by name, as Sam's own token reads them.
async fn users(jf: &Jellyfin) -> Vec<String> {
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

in_both_engines!(
    /// Planted, watched failing: a hand edit, the room served on no
    /// network (room.rs `serves` answering `Served::Elsewhere`), reverted
    /// after.
    night
);
async fn night(engine: Engine) {
    let scene = Scene::new("night", engine).await;
    if !Path::new(JELLYFIN).exists() {
        return scene.never_ran(&format!(
            "Jellyfin is not installed on this computer ({JELLYFIN})"
        ));
    }
    let jf = Jellyfin::up(scene.scratch("jellyfin")).await;
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
        .command(&sam, &["share", &port, "--kind", "jellyfin"], "\n")
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
    let args = |link: &Value, people: Value| {
        json!({
            "room": room, "link": link, "server": jf.env("SERVER"),
            "profile": profile, "account": "room", "people": people,
        })
        .to_string()
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
    let ana = json!([{ "name": "Ana", "user": "ana", "engine": engine.as_str(), "device": device, "film": "Test Pattern Two" }]);
    let night = node("night.mjs", &["five", &args(&links["https"], ana)], &env).await;
    eprintln!("the evening: {night:#}");
    assert!(night.get("stopped").is_none(), "{night:#}");
    let ana = &night["people"][0];
    assert!(
        ana.get("stopped").is_none_or(Value::is_null),
        "Ana gets from the link to the film on the TV: {ana:#}"
    );
    assert_eq!(
        ana["within_3_min"], true,
        "Ana's film plays on the TV within three minutes of the link: {ana:#}"
    );
    assert_eq!(
        ana["pause_seek_within_10s"], true,
        "Ana pauses and seeks it on the TV from her phone: {ana:#}"
    );

    // Nobody made an account.
    assert_eq!(
        users(&jf).await,
        before,
        "Jellyfin's users are those Sam made"
    );
    drop((sam, dee, jf));
}
