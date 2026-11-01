//! page-members' v2-only tests (main.rs): who is visiting, dropped from
//! the phone's page as their visit ends, with no reload.

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use svrngs_core::link::Token;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, RailAct, SigningKey};

use crate::drive::drive;
use crate::harness::{relay, relay_url, site_of, Engine};

/// Ada's computer holding house from its seed, as production holds a group
/// (browser/main.rs `house`), bo let in; on `crate::UI`. With the token of
/// a link ada's computer made, as the phone opens it.
async fn house(dir: &Path, relay: &iroh_relay::server::Server) -> (Node, String, String) {
    use svrngs_node::group::{draw, Name};
    let ada = SigningKey::from_bytes(&[1; 32]);
    let config = RelayConfig {
        relay_urls: vec![relay_url(relay)],
        n0_services: false,
        loopback_only: true,
    };
    let node = Node::start(
        dir.to_path_buf(),
        ada.clone(),
        &config,
        svrngs_node::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
    .ui(crate::UI);
    crate::drive::opens_at(node.api_addr().port(), &node.token());
    let seed = draw(
        Name::parse("house").unwrap(),
        Name::parse("ada").unwrap(),
        ada.verifying_key(),
    );
    node.host().keep_group(&seed, None, None).unwrap();
    let bo = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("bo").unwrap(),
            key: actor_of(&SigningKey::from_bytes(&[7; 32])),
            invite: None,
        },
    };
    node.rail().commit(&seed.id(), &bo).unwrap().unwrap();
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let token = Token {
        issuer: ada.verifying_key(),
        secret: [0; 16],
        relay: Some(relay_url(relay).parse().expect("a URL")),
    }
    .encode();
    (node, seed.id(), token)
}

/// Sam, ada's guest in group `id` for `secs` from now.
fn sam_visits(ada: &Node, id: &str, secs: i64) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let visit = svrngs_core::Visit {
        guest: Person::parse("Sam").unwrap(),
        key: actor_of(&SigningKey::from_bytes(&[9; 32])),
        opens: Default::default(),
        until: now.as_secs() as i64 + secs,
    };
    ada.rail().commit(id, &visit).unwrap().unwrap();
}

/// Each line of `seen`'s trace starting with `start`.
fn traced(seen: &Value, start: &str) -> usize {
    (seen["lines"].as_array().into_iter().flatten())
        .filter(|l| l.as_str().is_some_and(|l| l.starts_with(start)))
        .count()
}

in_webkit!(
    /// **A visit that ends while the phone's page is open drops from it**
    /// (page-members (b)). Sam is ada's guest for 20 s; bo's phone opens
    /// house and lists Sam visiting; as the visit ends, the page reads the
    /// group again by itself and lists nobody, with no reload. Watched
    /// failing on v1 (`UI` flipped), which reads the group once.
    a_visit_ended_while_open_drops_on_the_phone
);
async fn a_visit_ended_while_open_drops_on_the_phone(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, id, token) = house(dir.path(), &relay).await;
    // The site built first, which takes seconds the visit would lose.
    let site = site_of(crate::UI);
    sam_visits(&ada, &id, 20);
    let bo = hex::encode(SigningKey::from_bytes(&[7; 32]).to_bytes());
    let args = json!({ "site": site, "key": bo, "token": token, "shown": ".visiting li", "gone": ".visiting li", "at": "#visitors", "within": 40_000 });
    let seen = drive(engine, "v2/members.mjs", args, None).await;
    let shown = seen["shown"].as_array().cloned().unwrap_or_default();
    let wrong: Vec<&str> = [
        (
            !(shown.len() == 1
                && shown[0]
                    .as_str()
                    .is_some_and(|l| l.starts_with("Sam, ada’s guest, until "))),
            "Sam is listed visiting, ada's guest",
        ),
        (
            seen["went"] != true,
            "Sam is listed no more once the visit ends",
        ),
        (seen["reloads"] != 0, "with no reload"),
        (
            traced(&seen, "PAGE: reread (a visit ended") != 1,
            "the page read the group again as the visit ended",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

in_chromium!(
    /// **A guest's Stop on the home page ends the visit with no reload**
    /// (page-members (b)). Sam is ada's guest for an hour; ada's home page
    /// lists Sam under "Who you have over"; ada presses Stop, and once
    /// the Stop's own time has come the groups are read again and Sam is
    /// listed nowhere, where v1 reloaded the page (home.js:163). Watched
    /// failing on v1 (`UI` flipped).
    a_guest_s_stop_ends_the_visit_with_no_reload
);
async fn a_guest_s_stop_ends_the_visit_with_no_reload(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, id, _) = house(dir.path(), &relay).await;
    sam_visits(&ada, &id, 3600);
    let home = format!("http://svrngs.localhost:{}/", ada.api_addr().port());
    let args = json!({ "home": home, "shown": "#home .yours li", "press": ["#home .yours button"], "gone": "#home .yours li, #home .visiting li", "within": 20_000 });
    let seen = drive(engine, "v2/members.mjs", args, None).await;
    let shown = seen["shown"].as_array().cloned().unwrap_or_default();
    let wrong: Vec<&str> = [
        (
            !(shown.len() == 1
                && shown[0].as_str().is_some_and(|l| {
                    l.starts_with("Sam, until ") && l.ends_with("Stop Sam’s visit")
                })),
            "Sam is listed under Who you have over, with Stop",
        ),
        (seen["went"] != true, "Sam is listed nowhere once stopped"),
        (seen["reloads"] != 0, "with no reload"),
        (
            traced(&seen, "VISITS: waited") != 1,
            "the Stop waited for its time, then read again",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    let stopped = ada.rail().register(&id).unwrap();
    assert!(
        stopped.visits.values().all(|g| g.stopped.is_some()),
        "{stopped:#?}"
    );
    drop(ada);
}

/// Ada lending notes and a guest's link to house for `secs` made, as her
/// home page makes one (`/link` with `visit`), its page any https one: the
/// token, which the phone opens as `#visit=<token>`.
fn ada_has_a_guest(dir: &Path, ada: &Node, id: &str, secs: i64) -> String {
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "port": 22999, "kind": "k", "limit": 3, "refused": [], "reads": [] });
    std::fs::write(lends.join("notes.json"), body.to_string()).unwrap();
    let page = svrngs_core::link::Origin::parse("https://house.example/").unwrap();
    let linked = (ada.host().link(id, false, Some(&page), Some(secs))).expect("a guest's link");
    linked.token.encode()
}

/// Sam, in a browser holding nothing, at ada's guest's link of `secs`,
/// their name given once asked, then `watch` as members.mjs says.
/// Ada's computer, the group's id, what was seen, and ada's data dir, held.
async fn sam_at_the_link(
    engine: Engine,
    secs: i64,
    watch: Value,
) -> (Node, String, Value, tempfile::TempDir) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, id, _) = house(dir.path(), &relay).await;
    let site = site_of(crate::UI);
    let mut args = json!({ "site": site, "visit": ada_has_a_guest(dir.path(), &ada, &id, secs), "name": "Sam" });
    args.as_object_mut()
        .unwrap()
        .extend(watch.as_object().unwrap().clone());
    let seen = drive(engine, "v2/members.mjs", args, None).await;
    drop(relay);
    (ada, id, seen, dir)
}

in_webkit!(
    /// **A guest at their link is asked their name once, and is in**
    /// (page-members (c), web/app.js `visit`, :206-226, moved). Ada lends
    /// notes and makes a guest's link of an hour; Sam opens it in a browser
    /// holding nothing, is told "You’re invited over as a guest" and asked
    /// their name, gives it, and is in as ada's guest, ada's notes listed;
    /// ada's record holds one visit, Sam's. Red on v2 against the stub.
    a_guest_at_their_link_is_asked_their_name_once_and_is_in
);
async fn a_guest_at_their_link_is_asked_their_name_once_and_is_in(engine: Engine) {
    let watch =
        json!({ "shown": "#status.good, #status.bad", "said": "", "at": "#lent", "within": 1_000 });
    let (ada, id, seen, _dir) = sam_at_the_link(engine, 3600, watch).await;
    let shown = seen["shown"].as_array().cloned().unwrap_or_default();
    let visits = ada.rail().register(&id).unwrap().visits;
    let wrong: Vec<&str> = [
        (
            seen["asking"] != "You’re invited over as a guest",
            "Sam is invited over as a guest, asked their name",
        ),
        (
            !(shown.len() == 1
                && shown[0]
                    .as_str()
                    .is_some_and(|h| h.starts_with("You’re ada’s guest until "))),
            "Sam is in as ada's guest",
        ),
        (
            seen["text"].as_str().map(str::trim) != Some("ada’s notes (k)"),
            "ada's notes are listed for Sam to open",
        ),
        (
            !(visits.len() == 1 && visits.values().all(|g| g.visit.guest.as_str() == "Sam")),
            "ada's record holds one visit, Sam's",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

in_webkit!(
    /// **A guest's visit of 5 s ends while their page is shown**
    /// (page-members (c)'s fix). Sam, ada's guest, sees ada's notes
    /// listed; as the visit ends, the page says so and lists nothing, with
    /// no reload. Watched failing on v1 (`UI` flipped), which listed the
    /// lends until the link was opened again (web/app.js:221-224).
    a_guest_s_visit_of_5_s_ends_while_shown
);
async fn a_guest_s_visit_of_5_s_ends_while_shown(engine: Engine) {
    let watch = json!({ "shown": "#lent button", "gone": "#lent button", "at": "#status", "within": 15_000 });
    let (ada, _, seen, _dir) = sam_at_the_link(engine, 5, watch).await;
    let wrong: Vec<&str> = [
        (
            seen["shown"] != json!(["ada’s notes (k)"]),
            "ada's notes are listed while Sam visits",
        ),
        (
            seen["went"] != true,
            "nothing is listed once the visit ends",
        ),
        (
            !seen["text"]
                .as_str()
                .is_some_and(|t| t.starts_with("Your visit as ada’s guest ended ")),
            "the page says the visit has ended",
        ),
        (seen["reloads"] != 0, "with no reload"),
        (
            traced(&seen, "PAGE: ended") != 1,
            "the visit ended at its end, on the page's timer",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

in_chromium!(
    /// **A refused leave is said in its own words** (page-members (d)).
    /// ada's home page shows "Leave house"; its session lapses (its
    /// cookies cleared) and ada presses it: beside the button, house
    /// cannot be changed, as the session has ended, said as how to open
    /// it again. Watched failing on v1 (`UI` flipped), whose `asked`
    /// (home.js:42) reads the 401's page as JSON and says a parse error.
    a_refused_leave_is_said_in_its_own_words
);
async fn a_refused_leave_is_said_in_its_own_words(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, _, _) = house(dir.path(), &relay).await;
    let home = format!("http://svrngs.localhost:{}/", ada.api_addr().port());
    let leave = "#home section button:text-is('Leave house')";
    let args = json!({ "home": home, "shown": leave, "forget": true, "press": [leave], "said": "this page’s session has ended", "at": "#home .leave", "within": 15_000 });
    let seen = drive(engine, "v2/members.mjs", args, None).await;
    let said = "house cannot be changed on this computer now: this page’s session has ended. Open svrngs from your Applications";
    let wrong: Vec<&str> = [
        (seen["went"] != true, "the refusal is said beside Leave"),
        (
            !seen["text"].as_str().is_some_and(|t| t.contains(said)),
            "in its own words, how to open the page again",
        ),
        (seen["reloads"] != 0, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

in_chromium!(
    /// **Hand on, then leave, with no reload** (page-members (d)). ada
    /// keeps the order of removals in house, bo in it too; her home page
    /// offers to hand it on to bo. She presses "Hand it on": the groups
    /// are read again, so the offer goes, and beside Leave the page says
    /// bo keeps it now; she presses "Leave house", and the page says she
    /// left and lists house under "Groups you left", never reloading.
    /// Watched failing on v1 (`UI` flipped), whose offer stays until a
    /// reload (home.js:42 appends its words) and whose leave reloads.
    hand_on_then_leave_with_no_reload
);
async fn hand_on_then_leave_with_no_reload(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, _, _) = house(dir.path(), &relay).await;
    let home = format!("http://svrngs.localhost:{}/", ada.api_addr().port());
    let (hand, leave) = ("#home section p:has(> select)", "#home section p.leave");
    let press = json!([
        [format!("{hand} button"), hand],
        [format!("{leave} button"), leave]
    ]);
    let args = json!({ "home": home, "shown": hand, "press": press, "said": "Groups you left", "at": "#home", "within": 20_000 });
    let seen = drive(engine, "v2/members.mjs", args, None).await;
    let offered =
        "You keep the order of removals in house. Choose who keeps it after you, then leave: bo";
    let (handed, left) = (&seen["waited"][0], &seen["waited"][1]);
    let wrong: Vec<&str> = [
        (
            !seen["shown"][0]
                .as_str()
                .is_some_and(|t| t.starts_with(offered)),
            "ada is offered to hand it on to bo",
        ),
        (handed["went"] != true, "the offer goes once handed on"),
        (
            !handed["text"].as_str().is_some_and(|t| {
                t.contains(
                    "Leave house bo keeps the order of removals in house now, so you can leave it.",
                )
            }),
            "beside Leave, bo keeps it now",
        ),
        (left["went"] != true, "house's section goes once left"),
        (
            seen["went"] != true,
            "house is listed under Groups you left",
        ),
        (
            !seen["text"]
                .as_str()
                .is_some_and(|t| t.contains("You left house.")),
            "the page says ada left",
        ),
        (seen["reloads"] != 0, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

in_chromium!(
    /// **Two folder presses are said once** (page-members (f)). ada keeps
    /// house's folder; her home page offers to open it, and she presses
    /// "Open this folder" twice, each answered: the page says it opened
    /// once, the last press's answer, never one after the other. The
    /// openers are planted first on `PATH`, each answering done, so no
    /// window opens (browser/main.rs `the_home_page_opens_the_group_s_folder`).
    /// Watched failing on v1 (`UI` flipped), whose `p.append` (home.js:130)
    /// says each answer after the one before.
    two_folder_presses_are_said_once
);
async fn two_folder_presses_are_said_once(engine: Engine) {
    use std::os::unix::fs::PermissionsExt;
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, id, _) = house(dir.path(), &relay).await;
    let (folder, bin) = (dir.path().join("house"), dir.path().join("bin"));
    std::fs::create_dir(&folder).unwrap();
    std::fs::create_dir(&bin).unwrap();
    for opener in ["open", "xdg-open"] {
        std::fs::write(bin.join(opener), "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(bin.join(opener), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(bin.clone()).chain(std::env::split_paths(&path));
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    let kept = reqwest::Client::builder().no_proxy().build().unwrap();
    let kept = kept
        .post(format!("http://{}/v1/folders", ada.api_addr()))
        .bearer_auth(ada.token())
        .header("content-type", "application/json")
        .body(json!({ "dir": folder, "group": id }).to_string())
        .send()
        .await
        .unwrap();
    assert!(kept.status().is_success(), "{:?}", kept.text().await);
    let home = format!("http://svrngs.localhost:{}/", ada.api_addr().port());
    let open = "#home section button:text-is('Open this folder')";
    let args = json!({ "home": home, "shown": open, "press": [open, open], "said": "", "at": "#home p.folder + p", "traced": ["FOLDER: opened", 2], "within": 15_000 });
    let seen = drive(engine, "v2/members.mjs", args, None).await;
    let text = seen["text"].as_str().unwrap_or_default();
    let wrong: Vec<&str> = [
        (seen["went"] != true, "both presses are answered"),
        (
            text != "Open this folder Opened, in a window of its own.",
            "the page says it opened once",
        ),
        (seen["reloads"] != 0, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}

in_chromium!(
    /// **The door is judged again when the page is shown again**
    /// (page-members (f)). cy joins ada's house; ada's home page lights cy,
    /// "on now", bo drawn off beside, as bo's computer never answers.
    /// cy's computer goes off while the page is open; ada's page is hidden
    /// and shown again, and its door draws cy "did not answer", with no
    /// reload. The page read again on `visible` is page-open's (home.js
    /// `visible`, group.js GROUP), so this part is its test alone. Off, not
    /// back on, as nothing here restarts a node in process. Watched
    /// failing on v1 (`UI` flipped), whose page judges its door once.
    the_door_is_judged_again_when_shown_again
);
async fn the_door_is_judged_again_when_shown_again(engine: Engine) {
    use svrngs_node::group::Name;
    use svrngs_node::link::{client_link, Link};
    let relay = relay().await;
    let (dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ada, id, _) = house(dir.path(), &relay).await;
    let config = RelayConfig {
        relay_urls: vec![relay_url(&relay)],
        n0_services: false,
        loopback_only: true,
    };
    let cy = Node::start(
        cy_dir.path().to_path_buf(),
        SigningKey::from_bytes(&[3; 32]),
        &config,
        svrngs_node::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("cy's computer starts");
    tokio::time::timeout(Duration::from_secs(20), cy.endpoint().online())
        .await
        .expect("cy's computer reaches the relay on 127.0.0.1");
    let made = ada.host().link(&id, false, None, None).expect("a link");
    let sent = Link::parse(&client_link(&made.token)).unwrap();
    cy.host()
        .join(&sent, &Name::parse("cy").unwrap())
        .await
        .expect("cy joins house");
    let home = format!("http://svrngs.localhost:{}/", ada.api_addr().port());
    let hold = dir.path().join("hold");
    let args = json!({ "home": home, "shown": "#home .door li.off + li.on", "hold": hold, "gone": "#home .door li.off + li.on", "traced": ["GROUP: answered (lends 2)", 1], "at": "#home .door", "within": 30_000 });
    let driven = tokio::spawn(drive(engine, "v2/members.mjs", args, None));
    let shown = hold.with_extension("shown");
    let told = tokio::time::timeout(Duration::from_secs(90), async {
        while !shown.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    assert!(told.await.is_ok(), "ada's page lights cy within 90 s");
    cy.endpoint().close().await;
    drop(cy);
    std::fs::write(&hold, "").unwrap();
    let seen = driven.await.unwrap();
    let wrong: Vec<&str> = [
        (
            !seen["shown"][0]
                .as_str()
                .is_some_and(|t| t.starts_with("cy")),
            "cy is lit",
        ),
        (seen["went"] != true, "shown again, the door is read again"),
        (
            !seen["text"]
                .as_str()
                .is_some_and(|t| t.ends_with("cy\ndid not answer")),
            "cy did not answer",
        ),
        (seen["reloads"] != 0, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop(ada);
}
