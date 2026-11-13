//! demo/holyarrs.md: Bo asks Ana for a missing film, Ana approves, stock
//! HolyArrs imports a real local download, Movie Night finds it and Bo
//! presses Play: H5 is the film's stream received through Bo's lend, by
//! its worker on a phone (scene.mjs, person.mjs `streamed`). That a browser
//! decodes a lent film, and plays again within 5 s of a seek, is the
//! playback step's (crates/svrngs-node/tests/browser/playback.rs).
//! Metadata, Torznab and the tracker are explicit fixtures; the applications,
//! two torrent peers, group, door and browser are real.
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::process::Command;

use crate::films::{holding, node, Jellyfin};
use crate::harness::{browsers, relay_url, repo, Engine};
use crate::people::{at_page_host, open_of, Scene};

pub(crate) async fn check(script: &str, args: &[String]) -> Value {
    let run = Command::new("node")
        .arg(repo().join("apps/holyarrs/check").join(script))
        .args(args)
        .env("SVRNGS_BROWSERS", browsers())
        .kill_on_drop(true)
        .output();
    let out = tokio::time::timeout(Duration::from_secs(600), run)
        .await
        .expect("HolyArrs step ends within 600 s")
        .unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{script}: {said}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_str(said.lines().last().unwrap_or_default())
        .unwrap_or_else(|_| panic!("{script} prints JSON: {said}"))
}

pub(crate) struct Services(pub(crate) PathBuf, pub(crate) bool);
impl Services {
    async fn finish(mut self) {
        check(
            "services.mjs",
            &["down".into(), self.0.to_string_lossy().into()],
        )
        .await;
        self.1 = true;
    }
}
impl Drop for Services {
    fn drop(&mut self) {
        if self.1 || !self.0.join("services.json").is_file() {
            return;
        }
        let stopped = std::process::Command::new("node")
            .arg(repo().join("apps/holyarrs/check/services.mjs"))
            .arg("down")
            .arg(&self.0)
            .output();
        if let Ok(out) = stopped {
            if !out.status.success() {
                eprintln!(
                    "HolyArrs cleanup failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
    }
}

scene!(holyarrs);
mod phone {
    #[tokio::test(flavor = "multi_thread")]
    async fn webkit() {
        crate::people::judged(
            "holyarrs-webkit".into(),
            super::holyarrs(crate::harness::Engine::Webkit),
        )
        .await;
    }
}
async fn holyarrs(engine: Engine) {
    let scene = Scene::new("holyarrs", engine).await;
    let jf = match Jellyfin::up(scene.scratch("jellyfin")).await {
        Ok(jf) => jf,
        Err(why) => return scene.never_ran(&why),
    };
    let services = Services(scene.scratch("services"), false);
    let dir = services.0.to_string_lossy().to_string();
    let up = check(
        "services.mjs",
        &[
            "up".into(),
            dir.clone(),
            jf.scratch.join("r/jf/media").to_string_lossy().into(),
        ],
    )
    .await;
    if let Some(why) = up["never_ran"].as_str() {
        return scene.never_ran(why);
    }
    let setup = json!({ "port": jf.port, "token_file": jf.scratch.join("r/secrets/admin_token") });
    check(
        "services.mjs",
        &["setup".into(), dir.clone(), setup.to_string()],
    )
    .await;

    let phone = matches!(engine, Engine::Webkit);
    let (ana, group, origin, _served) = if phone {
        let (origin, served) = scene.page().await;
        let relay = relay_url(&scene.relay);
        let ana = scene
            .start(
                "Ana",
                &[
                    "new", "Friends", "--as", "Ana", "--relay", &relay, "--origin", &origin,
                ],
            )
            .await;
        let group = ana.group_named("Friends").await;
        (ana, group, origin, Some(served))
    } else {
        let (ana, group) = scene.founds("Ana", "Friends").await;
        (ana, group, String::new(), None)
    };
    let port = jf.port.to_string();
    let shared = scene
        .command(
            &ana,
            &[
                "share",
                &port,
                "--program",
                "jellyfin",
                "--group",
                "Friends",
            ],
            "\n",
        )
        .await;
    let how = shared
        .lines()
        .find(|l| l.contains("asks each person to sign in once"))
        .unwrap();
    let secrets = jf.scratch.join("r/secrets");
    let made = node(
        "lender.mjs",
        &[how, &jf.at(), "sam", "bo", "readers"],
        &[("SECRETS", secrets.to_string_lossy().into())],
    )
    .await;
    assert!(
        made.get("stopped").is_none(),
        "Jellyfin accounts prepared: {made:#}"
    );
    let password = jf.read("secrets/readers_pw");
    if std::env::var("SCENES_PLANT").as_deref() != Ok("holyarrs:no-readers") {
        ana.on_home_page("/lent", &json!({ "lend": "jellyfin", "to": { "readers": { "name": "readers", "password": password } } })).await;
    }
    let port = up["port"].to_string();
    scene
        .command(
            &ana,
            &[
                "share",
                &port,
                "--program",
                "holyarrs",
                "--group",
                "Friends",
            ],
            "\n",
        )
        .await;
    ana.on_home_page("/shelf", &json!([group, ["Movie-Night"]]))
        .await;
    let link = ana.invites(&group).await;
    let bo = scene.joins("Bo", &link, "Friends").await;
    let phone_link = if phone {
        let device = bo
            .on_home_page("/link", &json!({ "group": group, "device": true }))
            .await;
        let token = device["client"]
            .as_str()
            .unwrap()
            .trim_start_matches("svrngs://join/");
        Some(at_page_host(format!("{origin}#{token}")))
    } else {
        None
    };
    let out = std::env::var_os("HOLYARRS_RECORD")
        .map(|p| repo().join(p).join(engine.as_str()))
        .unwrap_or_else(|| scene.scratch("evidence"));
    let home = |at: String| open_of(&at).expect("svrngs opens its page").1;
    let args = json!({
        "state": services.0.join("services.json"), "engine": engine.as_str(),
        "ana_home": home(ana.home_page()), "bo_home": home(bo.home_page()),
        "ana_at": home(ana.lent_by(&group, "Ana", "holyarrs").await),
        "bo_at": home(bo.lent_by(&group, "Ana", "holyarrs").await),
        "night": bo.opens(&group, "Movie-Night").await,
        "jf_secrets": secrets, "jf": jf.at(),
        "binary": crate::people::program().0, "ana_dir": ana.home,
        "dump": scene.scratch("received"), "out": out, "phone_link": phone_link,
        "plant": std::env::var("SCENES_PLANT").unwrap_or_default(),
    });
    let mut seen = check("scene.mjs", &[args.to_string()]).await;
    eprintln!("HolyArrs scene: {}", seen["steps"]);
    assert!(
        seen.get("stopped").is_none(),
        "HolyArrs scene completed: {seen:#}"
    );
    let state: Value =
        serde_json::from_slice(&std::fs::read(services.0.join("services.json")).unwrap()).unwrap();
    seen["versions"] = json!({ "images": state["images"], "engine": engine.as_str(), "observed": seen["runtime"], "svrngs_package": env!("CARGO_PKG_VERSION") });
    let reader = ana
        .home
        .join(".svrngs/secrets/lends/jellyfin/readers/authorization");
    let reader = std::fs::read_to_string(reader).unwrap_or_default();
    let reader_token = reader
        .split("Token=\"")
        .nth(1)
        .and_then(|v| v.split('"').next())
        .unwrap_or_default();
    for (name, secret) in [
        ("Jellyfin admin", jf.read("secrets/admin_token")),
        ("Jellyfin owner password", jf.read("secrets/sam_pw")),
        ("readers password", password),
        ("readers token", reader_token.to_string()),
        (
            "HolyArrs owner password",
            state["ana"]["password"].as_str().unwrap().to_string(),
        ),
    ] {
        assert!(!secret.is_empty(), "H7: {name} is known to the instrument");
        let mut held = holding(Path::new(args["dump"].as_str().unwrap()), secret.as_bytes());
        held.extend(holding(&bo.home, secret.as_bytes()));
        assert!(held.is_empty(), "H7: {name} reached Bo: {held:?}");
    }
    let steps = seen["steps"].as_array_mut().unwrap();
    steps.push(json!({ "bar": "H7", "verdict": "passed", "owner_credentials_checked": 5 }));
    steps.push(json!({ "bar": "H8", "verdict": "passed", "fixture": "metadata, Torznab and tracker", "people": "programs on one host" }));
    for bar in ["H1", "H2", "H3", "H4", "H5", "H6", "H7", "H8"] {
        assert_eq!(
            steps
                .iter()
                .filter(|s| s["bar"] == bar && s["verdict"] == "passed")
                .count(),
            1,
            "{bar} ran and passed exactly once"
        );
    }
    services.finish().await;
    std::fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&seen).unwrap(),
    )
    .unwrap();
    drop((bo, ana, jf));
}
