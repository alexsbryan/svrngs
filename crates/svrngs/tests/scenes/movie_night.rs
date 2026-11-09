//! **Movie Night finds a friend's films** (apps/movie-night; capability
//! connections §5 step 6, the shelf): Sam makes House and lends his
//! Jellyfin as it comes, `svrngs share <its port> --kind jellyfin`. In
//! Jellyfin's own pages he makes an account for his groups that may see
//! his films, and gives it to svrngs on his home page, typing its name and
//! password, with no terminal (§6 bar 1). He brings Movie Night to House
//! from his home page, as he brings Money, typing nothing. Ana, new to
//! House, joins by its link on a computer of her own. Opened by its address
//! before she has said yes, Movie Night tells her Sam's films are new and
//! to open it from her home page, not that nobody lends any. From her home
//! page, her yes to what it reads, she searches a film that is in Sam's
//! library and sees its card with Sam's copy under it. She opens that copy
//! and lands on Sam's Jellyfin, where signing in as herself is the films
//! scene's (films.rs).
//!
//! Sam's Jellyfin is the films check's scratch one (films.rs
//! [`Jellyfin`]), never-ran where Jellyfin is not installed. Ana's browser
//! is movie_night.mjs, every response it receives written down
//! (apps/jellyfin/check/person.mjs `record`).
//!
//! The invariant it relies on: **a carried app is let in as the account its
//! lender keeps for readers, and only to look** (origin.rs `routed`, door.rs
//! `admit`): no credential of Sam's reaches Ana's browser or her svrngs,
//! Jellyfin's users after are those Sam made, and the readers' account is
//! still signed in after Ana's visit.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::process::Command;

use crate::films::{holding, node, Jellyfin};
use crate::harness::{browsers, repo, Engine};
use crate::night::users;
use crate::people::{late, open_of, Scene};

/// What Ana types into Movie Night's search: a film in Sam's library.
const FILM: &str = "Test Pattern One";

/// Ana's browser, movie_night.mjs on `args`: the JSON line it printed.
async fn ana_sees(args: &Value) -> Value {
    let script = repo().join("crates/svrngs/tests/scenes/movie_night.mjs");
    let run = Command::new("node")
        .arg(script)
        .arg(args.to_string())
        .env("SVRNGS_BROWSERS", browsers())
        .kill_on_drop(true)
        .output();
    let out = tokio::time::timeout(Duration::from_secs(400), run);
    let out = out.await.expect("Ana's browser ends within 400 s").unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    let line = said.lines().last().unwrap_or_default();
    serde_json::from_str(line).unwrap_or_else(|_| {
        panic!(
            "movie_night.mjs printed no JSON: {said}{}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

/// The plant `SCENES_PLANT` names for this scene, if any.
fn planted(plant: &str) -> bool {
    std::env::var("SCENES_PLANT").as_deref() == Ok(&format!("movie_night:{plant}"))
}

/// Movie Night's files in Sam's group folder as the shelf brings them, but
/// for `plant.js`, which holds Sam's own token, so the page sends it on each
/// request (index.js `planted`): `movie_night:lender-token`.
fn bring_with_token(folder: &Path, token: &str) {
    let to = folder.join("apps/Movie-Night");
    std::fs::create_dir_all(&to).unwrap();
    let kept = repo().join("apps/movie-night");
    for file in std::fs::read_dir(&kept).unwrap().flatten() {
        if file.path().is_file() {
            std::fs::copy(file.path(), to.join(file.file_name())).unwrap();
        }
    }
    let plant = format!("export const PLANT = 'lender-token';\nexport const TOKEN = '{token}';\n");
    std::fs::write(to.join("plant.js"), plant).unwrap();
}

/// Who Jellyfin says `authorization` signs in as: its status and name.
async fn me(jf: &Jellyfin, authorization: &str) -> (u16, Value) {
    let got = reqwest::Client::new()
        .get(format!("{}/Users/Me", jf.at()))
        .header("authorization", authorization)
        .send()
        .await
        .unwrap();
    let status = got.status().as_u16();
    let got = got.bytes().await.unwrap();
    let got: Value = serde_json::from_slice(&got).unwrap_or(Value::Null);
    (status, got["Name"].clone())
}

scene!(
    /// Planted, watched failing, each by `SCENES_PLANT`: `movie_night:
    /// no-readers`, Sam never gives the account; `movie_night:
    /// lender-token`, Movie Night brought with Sam's own token in its
    /// plant.js. A `logout` plant is gone with the lent form: a carried
    /// app's POST is refused at its own origin before any door
    /// (origin.rs `routed`; `a_carried_app_reads_the_lends_it_declares_at_its_own_origin`'s
    /// `before: post` plant).
    movie_night
);
async fn movie_night(engine: Engine) {
    let scene = Scene::new("movie_night", engine).await;
    let jf = match Jellyfin::up(scene.scratch("jellyfin")).await {
        Ok(jf) => jf,
        Err(why) => return scene.never_ran(&why),
    };
    let secrets = jf.scratch.join("r/secrets");
    let admin = jf.read("secrets/admin_token");
    let dump = scene.scratch("received");

    // 1. Sam makes House and lends his Jellyfin, typing nothing.
    let (sam, group) = scene.founds("Sam", "House").await;
    let port = jf.port.to_string();
    let jellyfin = ["share", &port, "--kind", "jellyfin", "--group", "House"];
    let shared = scene.command(&sam, &jellyfin, "\n").await;
    assert!(shared.contains("You lend jellyfin"), "{shared}");
    let how = shared
        .lines()
        .find(|l| l.contains("asks each person to sign in once"))
        .unwrap_or_else(|| panic!("share tells Sam nothing of accounts: {shared}"));

    // 2. Sam makes the account for his groups in Jellyfin's own pages:
    // README's step 1, Dashboard, Users and +, a name and a password, the
    // films library ticked, Save, which are the clicks of the line share
    // printed, by which lender.mjs finds each.
    let secrets_env = [("SECRETS", secrets.to_string_lossy().to_string())];
    let made = node(
        "lender.mjs",
        &[how, &jf.at(), "sam", "readers"],
        &secrets_env,
    )
    .await;
    assert!(
        made.get("stopped").is_none(),
        "Sam makes the readers' account: {made:#}"
    );
    let password = jf.read("secrets/readers_pw");
    let before = users(&jf).await;

    // 3. Sam gives svrngs that account on his home page, its name and
    // password typed there, no terminal (home.rs `readers`).
    let kept = sam
        .home
        .join(".svrngs/secrets/lends/jellyfin/readers/authorization");
    if !planted("no-readers") {
        let to = json!({ "readers": { "name": "readers", "password": password } });
        sam.on_home_page("/lent", &json!({ "lend": "jellyfin", "to": to }))
            .await;
    }
    let readers = std::fs::read_to_string(&kept).unwrap_or_default();

    // 4. Sam brings Movie Night to House from his home page, typing nothing
    // (crates/svrngs-node/src/shelf.txt; carried.rs `bring`).
    if planted("lender-token") {
        let groups = sam.get("/v1/groups").await;
        let house = (groups["groups"].as_array().into_iter().flatten())
            .find(|g| g["id"] == group.as_str())
            .unwrap_or_else(|| panic!("Sam holds House: {groups:#}"));
        bring_with_token(Path::new(house["folders"][0].as_str().unwrap()), &admin);
    } else {
        let brought = sam
            .on_home_page("/shelf", &json!([group, ["Movie-Night"]]))
            .await;
        assert_eq!(
            brought,
            json!([{ "app": "Movie-Night", "already": false }]),
            "Sam's home page brings Movie Night"
        );
    }

    // 5. Ana joins House by its link, on a computer of her own. Opened by
    // its address, Movie Night is not yet opened from her home page; from
    // there it is, and she searches a film of Sam's.
    let link = sam.invites(&group).await;
    let ana = scene.joins("Ana", &link, "House").await;
    let jf_at = ana.lent_by(&group, "Sam", "jellyfin").await;
    let at = ana.opens(&group, "Movie-Night").await;
    let home = open_of(&ana.home_page()).expect("svrngs opens it").1;
    let args = json!({
        "engine": Engine::Chromium.as_str(), "home": home, "at": at, "opener": "Open Movie Night",
        "film": FILM, "landed": "Sign In", "dump": dump,
    });
    let seen = ana_sees(&args).await;
    eprintln!("what Ana saw: {seen:#}");
    if let Some(stopped) = seen.get("stopped") {
        let error = stopped["error"].as_str().unwrap_or_default();
        if error.contains("Timeout") || error.contains("exceeded") {
            late(format!(
                "Ana's steps did not go within their bounds: {seen:#}"
            ))
            .await;
        }
        panic!("Ana's steps did not go as the scene says: {seen:#}");
    }
    // Not yet opened from her home page, §4's line, never "nobody lends one".
    assert_eq!(
        seen["before"]["said"],
        "Sam’s films are new. Open Movie Night from your home page to search them.",
        "Movie Night says what to do before Ana's yes: {seen:#}"
    );
    // Without the readers' account, Movie Night says whose it is to open,
    // since Ana cannot (index.js `WORDS.closed`); her search then shows nothing.
    let said = seen["searched"]["said"].as_str().unwrap_or_default();
    if planted("no-readers") {
        assert_eq!(
            said, "Sam’s library doesn’t let Movie Night search it yet. Only Sam can change that.",
            "the refusal names who can open it"
        );
    }
    let cards = seen["searched"]["cards"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let card = cards
        .iter()
        .filter_map(Value::as_str)
        .find(|c| c.starts_with(FILM));
    assert!(
        card.is_some_and(
            |c| c.contains(&format!("{FILM} (2026)")) && c.contains("Watch from Sam’s library")
        ),
        "Ana's search shows Sam's film, his copy under it: {:#}",
        seen["searched"]
    );
    // The title typed whole finds that film alone, though Test Pattern Two
    // and Three are within its slips (apps/movie-night/match.js `closest`).
    assert_eq!(
        (cards.len(), said),
        (1, "Sam’s library: 1 film"),
        "Ana's search shows the one film she named: {:#}",
        seen["searched"]
    );

    // 6. She opens Sam's copy and lands on his Jellyfin.
    let landed = seen["landed"]["url"].as_str().unwrap_or_default();
    let host = |at: &str| at.split('/').nth(2).unwrap_or_default().to_string();
    assert_eq!(
        host(landed),
        host(&jf_at),
        "she lands on Sam's Jellyfin: {seen:#}"
    );
    // At the film's own page, once she signs in: Jellyfin's pages keep
    // where they were sent in their address's fragment, `#/details?id=`
    // (apps/movie-night/match.js `itemAt`).
    let copy = seen["copy"].as_str().unwrap_or_default();
    let film = copy.split("id=").nth(1).and_then(|r| r.split('&').next());
    let film = film.unwrap_or_else(|| panic!("the copy names its film: {copy}"));
    assert!(
        landed.contains(film),
        "she lands at the film she opened, {film}, not Jellyfin's home: {landed}"
    );

    // No credential of Sam's reached Ana: in nothing her browser received,
    // nor in her svrngs's log or data.
    let token = readers.split('"').nth(1).unwrap_or_default().to_string();
    for (what, secret) in [
        ("Sam's own token", &admin),
        ("the readers' token", &token),
        ("the readers' password", &password),
    ] {
        assert!(!secret.is_empty(), "{what} is known to the scene");
        let mut held = holding(&dump, secret.as_bytes());
        held.extend(holding(&ana.home, secret.as_bytes()));
        assert!(held.is_empty(), "{what} reached Ana: {held:?}");
    }

    // Reading stays reading: Jellyfin's users are those Sam made, and the
    // readers' account is still signed in after Ana's visit.
    assert_eq!(users(&jf).await, before, "nobody made or lost an account");
    let (status, name) = me(&jf, &readers).await;
    assert_eq!(
        (status, name),
        (200, json!("readers")),
        "the readers' token still signs in as readers after Ana's visit"
    );
    drop((sam, ana, jf));
}
