//! **Movie Night finds a friend's films** (apps/movie-night; DESIGN §3,
//! "Lent apps, as they come" 4; apps/jellyfin/README.md, "Let apps that
//! read it search it"): Sam makes House and lends his Jellyfin as it
//! comes, `svrngs share <its port> --kind jellyfin`. In Jellyfin's own
//! pages he makes an account for his groups that may see his films, and
//! gives it to svrngs, `svrngs share <its port> --readers`, typing its
//! name and password at the prompts. He lends Movie Night too, its page
//! served by a plain static file server, `svrngs share <its port> --kind
//! movie-night --reads jellyfin`. Ana, new to House, joins by its link on
//! a computer of her own, opens Movie Night from her home page, which is
//! her yes to what it reads, searches a film that is in Sam's library,
//! and sees its card with Sam's copy under it. She opens that copy and
//! lands on Sam's Jellyfin, where signing in as herself is the films
//! scene's (films.rs).
//!
//! Sam's Jellyfin is the films check's scratch one (films.rs
//! [`Jellyfin`]), never-ran where Jellyfin is not installed. Movie Night
//! is lent, not carried: a carried app's reads are not built (lend.rs,
//! a carried lend's `reads: Vec::new()`). Ana's browser is
//! movie_night.mjs, every response it receives written down
//! (apps/jellyfin/check/person.mjs `record`).
//!
//! The invariant it relies on: **a reading app is let in as the account
//! its lender keeps for readers, and only to look** (door.rs `reading`):
//! the readers' secret goes only to another app's GET or HEAD, so no
//! credential of Sam's reaches Ana's browser or her svrngs, Jellyfin's
//! users after are those Sam made, and the readers' account is still
//! signed in after Ana's visit.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::process::{Child, Command};

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

/// Movie Night's page, apps/movie-night's files as they are kept, copied
/// to `dir` and served there on 127.0.0.1 by a plain static file server,
/// as a person serves a page: its port, and the server, killed on drop.
/// `lender-token` writes Sam's own token into plant.js, which app.js then
/// sends on each request (apps/movie-night/plant.js); `logout` adds a
/// line to app.js that, once a card is drawn, asks each library it reads
/// to end its session, by a POST, as the readers' page in the spike did
/// (cc15b40f).
async fn served(dir: &Path, token: &str) -> (u16, Child) {
    std::fs::create_dir_all(dir).unwrap();
    let kept = repo().join("apps/movie-night");
    for file in std::fs::read_dir(&kept).unwrap().flatten() {
        if file.path().is_file() {
            std::fs::copy(file.path(), dir.join(file.file_name())).unwrap();
        }
    }
    if planted("lender-token") {
        let plant =
            format!("export const PLANT = 'lender-token';\nexport const TOKEN = '{token}';\n");
        std::fs::write(dir.join("plant.js"), plant).unwrap();
    }
    if planted("logout") {
        let ends = "\nnew MutationObserver(() => results.childElementCount && told().then((t) => t?.lends.forEach((l) => fetch(`${l.address}Sessions/Logout`, { method: 'POST' })))).observe(results, { childList: true });\n";
        let app = std::fs::read_to_string(dir.join("app.js")).unwrap();
        std::fs::write(dir.join("app.js"), app + ends).unwrap();
    }
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = free.local_addr().unwrap().port();
    drop(free);
    let server = Command::new("python3")
        .args([
            "-m",
            "http.server",
            &port.to_string(),
            "--bind",
            "127.0.0.1",
        ])
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .expect("python3 runs");
    for _ in 0..100 {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return (port, server);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    late(format!(
        "Movie Night's page was not served on {port} within 10 s"
    ))
    .await
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
    /// no-readers`, Sam never runs `share --readers`; `movie_night:
    /// lender-token`, Movie Night served with Sam's own token in its
    /// plant.js; `movie_night:logout`, Movie Night asking each library to
    /// end its session, with a hand edit of door.rs `reading` that lets a
    /// POST be a read, reverted after.
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

    // 3. Sam gives svrngs that account, typing its name and password.
    let kept = sam
        .home
        .join(".svrngs/secrets/lends/jellyfin/readers/authorization");
    if !planted("no-readers") {
        let readers = [&jellyfin[..], &["--readers"]].concat();
        let typed = format!("readers\n{password}\n");
        scene.command(&sam, &readers, &typed).await;
    }
    let readers = std::fs::read_to_string(&kept).unwrap_or_default();

    // 4. Sam lends Movie Night, which reads his and his friends' Jellyfins.
    let (mn_port, _page) = served(&scene.scratch("movie-night"), &admin).await;
    let mn_port = mn_port.to_string();
    let lends = [
        "share",
        &mn_port,
        "--kind",
        "movie-night",
        "--reads",
        "jellyfin",
        "--group",
        "House",
    ];
    let shared = scene.command(&sam, &lends, "\n").await;
    assert!(shared.contains("You lend movie-night"), "{shared}");

    // 5. Ana joins House by its link, on a computer of her own, opens
    // Movie Night from her home page, and searches a film of Sam's.
    let link = sam.invites(&group).await;
    let ana = scene.joins("Ana", &link, "House").await;
    let jf_at = ana.lent_by(&group, "Sam", "jellyfin").await;
    ana.lent_by(&group, "Sam", "movie-night").await;
    let home = open_of(&ana.home_page()).expect("svrngs opens it").1;
    let args = json!({
        "engine": Engine::Chromium.as_str(), "home": home, "opener": "Open Sam’s movie-night",
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
    // Without the readers' account, Movie Night says whose it is to open,
    // since Ana cannot (app.js `WORDS.closed`); her search then shows nothing.
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
