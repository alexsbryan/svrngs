//! The film newcomer's real world. No native newcomer is pre-joined and no
//! request-app account is preaccepted. Its invitation is copied from the UI.
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::films::{node, Jellyfin};
use crate::harness::{relay_url, Engine, PAGE_HOST};
use crate::holyarrs::{check, Services};
use crate::people::{at_page_host, Person, Scene};
use crate::{foldered, opened, stay_up};

struct World {
    _scene: Scene,
    ana: Person,
    group: String,
    jf: Jellyfin,
    _services: Services,
    state: Value,
    _page: tokio::process::Child,
}

fn public(origin: &str, page: &str) -> String {
    let label = reqwest::Url::parse(origin)
        .unwrap()
        .host_str()
        .unwrap()
        .split('.')
        .next()
        .unwrap()
        .to_string();
    let port = reqwest::Url::parse(page).unwrap().port().unwrap();
    format!("https://{label}.{PAGE_HOST}:{port}")
}

async fn response(request: reqwest::RequestBuilder) -> Value {
    match request.send().await {
        Ok(r) => {
            let status = r.status().as_u16();
            match r.json::<Value>().await {
                Ok(body) => json!({ "status": status, "body": body }),
                Err(e) => json!({ "status": status, "could_not_read": e.to_string() }),
            }
        }
        Err(e) => json!({ "could_not_observe": e.to_string() }),
    }
}

async fn observe(world: World, to: PathBuf) {
    let began = Instant::now();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let admin = format!(
        "MediaBrowser Token=\"{}\"",
        world.jf.read("secrets/admin_token")
    );
    let mut history = vec![];
    let mut previous = Value::Null;
    loop {
        let people = world
            .ana
            .get(&format!("/v1/rail/people?namespace={}", world.group))
            .await;
        let root = world.state["at"].as_str().unwrap();
        let cookie = world.state["adminCookie"].as_str().unwrap();
        let asks = response(
            http.get(format!("{root}/api/v1/asks"))
                .header("cookie", cookie),
        )
        .await;
        let users = response(
            http.get(format!("{root}/api/v1/users"))
                .header("cookie", cookie),
        )
        .await;
        let sessions = response(
            http.get(format!("{}/Sessions", world.jf.at()))
                .header("authorization", &admin),
        )
        .await;
        let sessions = match sessions["body"].as_array() {
            Some(xs) => {
                json!({ "status": sessions["status"], "sessions": xs.iter().map(|s| json!({ "user": s["UserName"], "item": s["NowPlayingItem"]["Id"], "title": s["NowPlayingItem"]["Name"], "position_ticks": s["PlayState"]["PositionTicks"], "paused": s["PlayState"]["IsPaused"] })).collect::<Vec<_>>() })
            }
            None => sessions,
        };
        let current =
            json!({ "people": people, "asks": asks, "users": users, "playback": sessions });
        if current != previous {
            history.push(json!({ "seconds": began.elapsed().as_secs_f64(), "observed": current }));
            let data = json!({ "simulated": true, "read_only": true, "expected": { "item": world.jf.env("ONE"), "viewer": "bo", "requester": "Bo", "group": world.group }, "observations": history });
            let next = to.with_extension("next");
            std::fs::write(&next, serde_json::to_vec_pretty(&data).unwrap()).unwrap();
            std::fs::rename(next, &to).unwrap();
            previous = current;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
struct Observing(tokio::task::JoinHandle<()>);
impl Drop for Observing {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub async fn world() {
    eprintln!(
        "film newcomer: product binary {:?}",
        std::env::var_os("SCENES_BINARY")
    );
    let scene = Scene::new("xenia-film", Engine::Chromium).await;
    let jf = Jellyfin::up(scene.scratch("jellyfin"))
        .await
        .expect("newcomer world needs the real scratch Jellyfin");
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
    assert!(up["never_ran"].is_null(), "{up}");
    let jf_setup =
        json!({ "port": jf.port, "token_file": jf.scratch.join("r/secrets/admin_token") });
    check(
        "services.mjs",
        &[
            "setup".into(),
            dir.clone(),
            jf_setup.to_string(),
            "--newcomer".into(),
        ],
    )
    .await;
    let (page, served) = scene.page().await;
    let relay = relay_url(&scene.relay);
    let ana = scene
        .start(
            "Ana",
            &[
                "new", "Friends", "--as", "Ana", "--origin", &page, "--relay", &relay,
            ],
        )
        .await;
    let group = ana.group_named("Friends").await;
    foldered(&ana, &group).await;
    let jf_port = jf.port.to_string();
    let shared = scene
        .command(
            &ana,
            &[
                "share",
                &jf_port,
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
    assert!(made.get("stopped").is_none(), "{made}");
    ana.on_home_page("/lent", &json!({ "lend": "jellyfin", "to": { "readers": { "name": "readers", "password": jf.read("secrets/readers_pw") } } })).await;
    scene
        .command(
            &ana,
            &[
                "share",
                &up["port"].to_string(),
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
    let groups = ana.get("/v1/groups").await;
    let folder = groups["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["id"] == group)
        .unwrap()["folders"][0]
        .as_str()
        .unwrap();
    for file in ["index.html", "index.js"] {
        let brought =
            std::fs::read(PathBuf::from(folder).join("apps/Movie-Night").join(file)).unwrap();
        let source =
            std::fs::read(crate::harness::repo().join("apps/movie-night").join(file)).unwrap();
        assert!(
            brought == source,
            "the running binary carries this source's {file}, not a stale build"
        );
    }
    let own = ana
        .on_home_page("/link", &json!({ "group": group, "device": true }))
        .await;
    let owner_link = at_page_host(own["https"].as_str().unwrap().to_string());
    let holy = ana.lent_by(&group, "Ana", "holyarrs").await;
    let library = ana.lent_by(&group, "Ana", "jellyfin").await;
    let night = ana.opens(&group, "Movie-Night").await;
    let mode = std::env::var("NEWCOMER_INVITE_MODE").unwrap_or_else(|_| "shared".into());
    let app = check("prepare-newcomer.mjs", &[json!({ "state": services.0.join("services.json"), "home": opened(&ana.home_page()), "owner_link": owner_link, "mode": mode }).to_string()]).await;
    // The ordinary invitation is made last, after Ana's own-device link was
    // consumed: another invitation by its issuer can supersede the earlier one.
    let ordinary = ana
        .on_home_page("/link", &json!({ "group": group, "device": false }))
        .await;
    let link = at_page_host(ordinary["https"].as_str().unwrap().to_string());
    if std::env::var("NEWCOMER_PLANT").as_deref() == Ok("closed-invitation") {
        ana.on_home_page("/link", &json!({ "group": group, "device": false }))
            .await;
    }
    let initial = format!("Our film group is here: {link} Try watching Test Pattern One from my films. For my films, your account is bo and your password is {}. Here's your invitation to make your own account for film requests: {}", jf.read("secrets/bo_pw"), app["invitation"].as_str().unwrap());
    let evidence = scene.scratch("evidence.json");
    std::fs::write(
        &evidence,
        r#"{"simulated":true,"observations":[],"status":"not sampled yet"}"#,
    )
    .unwrap();
    let raw_state: Value =
        serde_json::from_slice(&std::fs::read(services.0.join("services.json")).unwrap()).unwrap();
    let private = json!({ "ordinary": link, "app_invitation": app, "owner_home": opened(&ana.home_page()), "owner_link": owner_link, "services_state": services.0.join("services.json"), "jellyfin": jf.at(), "jellyfin_secrets": secrets, "evidence": evidence, "public_library": public(&library, &page), "public_requests": public(&holy, &page), "public_night": public(&night, &page) });
    let mut private = private;
    private["expected_item"] = json!(if std::env::var("NEWCOMER_PLANT").as_deref()
        == Ok("wrong-item")
    {
        jf.env("TWO")
    } else {
        jf.env("ONE")
    });
    private["owner_data"] = json!(ana.home.join(".svrngs"));
    private["product_binary"] =
        json!(std::env::var_os("SCENES_BINARY").map(|p| p.to_string_lossy().into_owned()));
    std::fs::write(
        scene.scratch("world-private.json"),
        serde_json::to_vec_pretty(&private).unwrap(),
    )
    .unwrap();
    let public_page = at_page_host(page.clone());
    let mut reach = vec![
        public_page.trim_end_matches('/').to_string(),
        public(&library, &page),
        public(&holy, &page),
        public(&night, &page),
        relay.trim_end_matches('/').to_string(),
        relay
            .replacen("http://", "https://", 1)
            .trim_end_matches('/')
            .to_string(),
        "https://usw1-1.relay.n0.iroh.link".into(),
    ];
    reach.sort();
    reach.dedup();
    // A step ends at visible progress, not just an origin's Opening page.
    // These checks recharge attention only; service observations decide success.
    let on =
        |origin: &str, expr: &str| format!("location.origin === {} && ({expr})", json!(origin));
    let viewing = public(&library, &page);
    let requesting = public(&holy, &page);
    let line = json!({
        "begins": "Bo has only the Friends chat in a fresh phone browser, outside Friends, with no request-app account. Ana's real media services and the host-created viewing account/invitation were prepared. No newcomer has joined or accepted an app invitation. One host; real applications and transport, explicit local metadata/source/tracker fixtures.",
        "cast": [{ "as": "Bo", "device": "phone", "url": null }],
        "channels": [{ "name": "Friends chat", "members": ["Bo"], "opening": [{ "from": "Ana", "text": initial }] }],
        "screens": [
            { "name": "Joining Friends", "text": "What’s your name?" },
            { "name": "New request account", "text": "Join HolyArrs" },
            { "name": "Requested film found", "expr": on(&requesting, "(document.body?.innerText ?? '').includes('Copper Lighthouse')") },
            { "name": "Request home", "expr": on(&requesting, "/Good .*, Bo/.test(document.body?.innerText ?? '')") },
            { "name": "Film found", "expr": on(&viewing, "(document.body?.innerText ?? '').includes('Test Pattern One')") },
            { "name": "Viewing sign-in", "expr": on(&viewing, "!!document.querySelector('#txtManualName')?.getBoundingClientRect().height") },
            { "name": "Friends", "text": "You’re in Friends, as Bo" },
            { "name": "Movie Night", "expr": on(&public(&night, &page), "document.querySelector('h1')?.innerText === 'Movie Night'") },
            { "name": "Ana’s films", "url": public(&library, &page) },
            { "name": "Film requests", "url": public(&holy, &page) }
        ],
        "scenes": [
            { "id": "watch", "minutes": 5, "talk": 100, "brief": [{ "from": "Ana", "text": "Have a look and try watching the film I mentioned. Tell me here which film you managed to watch, or where you got stuck." }], "feedback": ["What did you manage to watch?", { "ask": "Where did you get stuck or need another instruction?", "bad": "stuck" }], "reach": ["Joining Friends", "Friends", "Movie Night", "Viewing sign-in", "Film found"] },
            { "id": "request", "minutes": 5, "talk": 100, "brief": [{ "from": "Ana", "text": "Could you ask for Copper Lighthouse for next time? Use your invitation to make your own account for requests if you haven't yet. Tell me when your request is waiting, or what stopped you." }], "feedback": ["What happened to your request?", { "ask": "Was there a step you could not work out from the pages and my message?", "bad": "stuck" }], "reach": ["Friends", "New request account", "Request home", "Requested film found"] },
            { "id": "return", "minutes": 3, "talk": 100, "brief": [{ "from": "Ana", "text": "Come back through the original group link and check what happened to your film request. You shouldn't need to join as somebody new. Tell me what you find." }], "feedback": ["What does your request say now?", { "ask": "Did anything make you sign up again or leave you unsure where to go?", "bad": "stuck" }], "reach": ["Friends", "Request home", "Requested film found"] }
        ],
        "faults": [{ "text": "can’t be read just now" }, { "text": "Open svrngs from your computer" }],
        "reach": reach,
        "attachments": [{ "as": "Bo", "path": evidence }]
    });
    let world = World {
        _scene: scene,
        ana,
        group,
        jf,
        _services: services,
        state: raw_state,
        _page: served,
    };
    let observing = Observing(tokio::spawn(observe(world, evidence)));
    eprintln!("film newcomer: private world state is in target/scenes/xenia-film-chromium/world-private.json");
    stay_up(line, observing).await;
}
