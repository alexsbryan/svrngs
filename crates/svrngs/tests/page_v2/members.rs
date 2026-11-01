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
