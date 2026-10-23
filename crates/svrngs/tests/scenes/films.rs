//! **Films** (demo/a.md): Sam lends his Jellyfin to House as it comes,
//! `svrngs share <its port> --kind jellyfin`, typing nothing, and makes
//! Alex and Bo an account each in Jellyfin's own pages, as the line
//! `share` printed tells him (apps/jellyfin/README.md). Alex and Bo join
//! by the link Sam's home page makes, each on a computer of their own,
//! and open Sam's Jellyfin at their own computer's address of it. Each
//! signs in as themselves and plays a different film, and each seeks
//! with Jellyfin's own seek bar while the other plays on.
//!
//! Sam's Jellyfin is a scratch one, the films check's
//! (apps/jellyfin/check/check.sh `jellyfin`), on a port of its own for
//! each engine; the scene is never-ran where Jellyfin is not installed.
//! The people's browsers are that check's (drive.mjs, person.mjs).
//!
//! The invariant it relies on: **a lend carries nothing of its lender's
//! sign-in** (demo/a.md, "After"): each person is in Jellyfin as
//! themselves, on a device of their own, and Sam's password is in
//! nothing either browser received nor in either's svrngs log.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};

use crate::harness::{repo, Engine};
use crate::people::Scene;

/// Where the films check finds Jellyfin (check.sh `JF_APP`).
const JELLYFIN: &str = "/Applications/Jellyfin.app/Contents/MacOS/jellyfin";

/// check.sh `<step> <scratch> <out>`, with Jellyfin on `port`.
async fn check(step: &str, scratch: &Path, port: u16) -> std::process::Output {
    let script = repo().join("apps/jellyfin/check/check.sh");
    let run = tokio::process::Command::new(script)
        .args([
            step,
            &scratch.join("r").to_string_lossy(),
            &scratch.join("out").to_string_lossy(),
        ])
        .env("JF_PORT", port.to_string())
        .output();
    let ran = tokio::time::timeout(Duration::from_secs(300), run).await;
    ran.expect("check.sh ends within 300 s").unwrap()
}

/// Sam's scratch Jellyfin, its films and his account, up on `port`;
/// stopped by check.sh `down` when dropped.
struct Jellyfin {
    scratch: PathBuf,
    port: u16,
}

impl Jellyfin {
    async fn up(scratch: PathBuf) -> Jellyfin {
        // Its library's path as Jellyfin keeps it, with no `..` in it: a
        // library at `crates/svrngs/../../target/…` scans to no films.
        std::fs::create_dir_all(&scratch).unwrap();
        let scratch = scratch.canonicalize().unwrap();
        // A port nothing holds now, for this engine's Jellyfin alone.
        let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = free.local_addr().unwrap().port();
        drop(free);
        let jf = Jellyfin { scratch, port };
        let ran = check("jellyfin", &jf.scratch, port).await;
        assert!(
            ran.status.success(),
            "Sam's Jellyfin did not come up: {}",
            String::from_utf8_lossy(&ran.stderr)
        );
        jf
    }

    fn at(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// What check.sh wrote to `<scratch>/r/<file>`.
    fn read(&self, file: &str) -> String {
        std::fs::read_to_string(self.scratch.join("r").join(file)).unwrap()
    }

    /// `key` of the films and the server check.sh found (its env).
    fn env(&self, key: &str) -> String {
        let env = self.read("env");
        let line = env.lines().find_map(|l| l.strip_prefix(&format!("{key}=")));
        line.unwrap_or_else(|| panic!("check.sh's env lacks {key}: {env}"))
            .to_string()
    }
}

impl Drop for Jellyfin {
    fn drop(&mut self) {
        let script = repo().join("apps/jellyfin/check/check.sh");
        let r = self.scratch.join("r");
        let _ = std::process::Command::new(script)
            .args([
                "down",
                &r.to_string_lossy(),
                &self.scratch.join("out").to_string_lossy(),
            ])
            .output();
    }
}

/// `node <script> <args>` of the films check, `env` given: its last line.
async fn node(script: &str, args: &[&str], env: &[(&str, String)]) -> Value {
    let mut run = tokio::process::Command::new("node");
    run.arg(repo().join("apps/jellyfin/check").join(script))
        .args(args);
    for (k, v) in env {
        run.env(k, v);
    }
    let out = tokio::time::timeout(Duration::from_secs(300), run.output());
    let out = out.await.expect("it ends within 300 s").unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    let line = said.lines().last().unwrap_or_default();
    serde_json::from_str(line).unwrap_or_else(|_| {
        panic!(
            "{script} printed no JSON: {said}{}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

/// Every file under `dir` that holds `secret`.
fn holding(dir: &Path, secret: &[u8]) -> Vec<PathBuf> {
    let mut found = vec![];
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(holding(&path, secret));
        } else if std::fs::read(&path)
            .unwrap_or_default()
            .windows(secret.len())
            .any(|w| w == secret)
        {
            found.push(path);
        }
    }
    found
}

in_both_engines!(
    /// Planted, watched failing: `SCENES_PLANT=films:token`, Sam's own
    /// Jellyfin token kept as the lend's secret, so the door sends it on
    /// every request (door.rs `secret_headers`).
    films
);
async fn films(engine: Engine) {
    let scene = Scene::new("films", engine).await;
    if !Path::new(JELLYFIN).exists() {
        return scene.never_ran(&format!(
            "Jellyfin is not installed on this computer ({JELLYFIN})"
        ));
    }
    let jf = Jellyfin::up(scene.scratch("jellyfin")).await;
    let secrets = jf.scratch.join("r/secrets");

    // 1. Sam makes House and lends his Jellyfin, typing nothing.
    let (sam, group) = scene.founds("Sam", "House").await;
    let port = jf.port.to_string();
    let shared = scene
        .command(&sam, &["share", &port, "--kind", "jellyfin"], "\n")
        .await;
    if std::env::var("SCENES_PLANT").as_deref() == Ok("films:token") {
        // The plant: Sam's token kept as the lend's secret, where the door
        // reads one on each request (lend.rs `secret_dir`), as check.sh
        // `plant token` keeps one; `share` asks a Jellyfin lender for none.
        let kept = sam.home.join(".svrngs/secrets/lends/jellyfin");
        std::fs::create_dir_all(&kept).unwrap();
        let device = r#"MediaBrowser Client="svrngs lend", Device="Sam computer", DeviceId="svrngs-lend-token", Version="1""#;
        let token = jf.read("secrets/admin_token");
        let kept = kept.join("authorization");
        std::fs::write(kept, format!("{device}, Token=\"{token}\"")).unwrap();
    }
    assert!(shared.contains("You lend jellyfin"), "{shared}");
    let how = shared
        .lines()
        .find(|l| l.contains("asks each person to sign in once"))
        .unwrap_or_else(|| panic!("share tells Sam nothing of accounts: {shared}"));

    // 2. Sam makes Alex and Bo an account each, as that line says.
    let secrets_env = [("SECRETS", secrets.to_string_lossy().to_string())];
    let made = node(
        "lender.mjs",
        &[how, &jf.at(), "sam", "alex", "bo"],
        &secrets_env,
    )
    .await;
    assert!(
        made.get("stopped").is_none(),
        "Sam makes the accounts: {made:#}"
    );

    // 3. Alex and Bo join, each on a computer of their own.
    let link = sam.invites(&group).await;
    let alex = scene.joins("Alex", &link, "House").await;
    let bo = scene.joins("Bo", &link, "House").await;
    let alex_at = alex.lent_by(&group, "Sam", "jellyfin").await;
    let bo_at = bo.lent_by(&group, "Sam", "jellyfin").await;

    // 4. Each signs in, plays a film of their own, and seeks while the
    // other plays on.
    let dump = scene.scratch("received");
    let people = json!({
        "way": "computer", "engine": engine.as_str(), "server": jf.env("SERVER"),
        "people": [
            { "name": "Alex", "user": "alex", "film": jf.env("ONE"), "seeks": [50, 15, 80], "at": alex_at },
            { "name": "Bo", "user": "bo", "film": jf.env("TWO"), "seeks": [60, 20, 75], "at": bo_at },
        ],
    });
    let env = [
        secrets_env[0].clone(),
        ("JF", jf.at()),
        ("ADMIN", jf.read("secrets/admin_token")),
        ("DUMP", dump.to_string_lossy().to_string()),
    ];
    let seen = node("drive.mjs", &[&people.to_string()], &env).await;
    eprintln!("what Alex and Bo saw: {seen:#}");
    assert!(seen.get("stopped").is_none(), "{seen:#}");
    for who in ["alex", "bo"] {
        let p = &seen["people"][who];
        assert!(p["play_ms"].is_number(), "{who}'s film plays: {p:#}");
        assert_eq!(
            p["signed_in_after"], who,
            "{who} is in Jellyfin as themselves: {p:#}"
        );
    }
    assert_eq!(
        seen["devices_apart"], true,
        "Alex and Bo are each a device of their own: {seen:#}"
    );
    for phase in ["first_seeks", "then_seeks"] {
        let p = &seen[phase];
        let seeks = p["seeks"].as_array().map_or(0, Vec::len);
        assert!(
            seeks == 3 && p["within_5s"] == 3,
            "each of {}'s seeks plays again within 5 s: {p:#}",
            p["seeker"]
        );
        // Judged by the film itself: never started over, never back, never
        // still 5 s. WebKit's watcher at times binds a second <video> while
        // the film plays on, before any of this (demo/runs/si/2026-10-08/
        // README.md, "What failed"), a count printed, not judged.
        let w = &p["watch"];
        eprintln!(
            "{}'s page bound {} new <video> while {} sought",
            p["watched"], w["element"], p["seeker"]
        );
        assert!(
            w["emptied"] == 0
                && w["loadstart"] == 0
                && w["back_s"].as_f64() <= Some(2.0)
                && w["still_ms"].as_u64() < Some(5000),
            "{}'s film plays on while {} seeks: {p:#}",
            p["watched"],
            p["seeker"]
        );
    }

    // Sam's password is in nothing they received, nor in their logs.
    let password = jf.read("secrets/sam_pw");
    let mut held = holding(&dump, password.as_bytes());
    for who in [&alex, &bo] {
        held.extend(holding(&who.home, password.as_bytes()));
    }
    assert!(held.is_empty(), "Sam's password reached them: {held:?}");
    drop((sam, alex, bo, jf));
}
