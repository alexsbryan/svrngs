//! Leaving a group on the home page (groups-leave (c); DESIGN §3 and §6,
//! "Leaving a group"; svrn-docs `rings/current/LEAVING.md` §5, bars 3 to
//! 5): Sam's computer makes Trip and Ana's joins it from Sam's link, each
//! in this process, met at a relay on 127.0.0.1, so nothing leaves this
//! computer. Each home page is driven in Playwright's headless Chromium
//! and its WebKit by `web/tests/home.mjs`, which fails naming the install
//! command when Playwright or its browsers are absent — it never skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
mod common;

use std::time::Duration;

use serde_json::{json, Value};
use svrngs::cli;
use svrngs::group::{self, Name};
use svrngs_node::link::{client_link, Link};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, SigningKey};

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

/// Drive `web/tests/home.mjs` in `engine` at `node`'s home page, as `args`
/// say.
async fn home_page(engine: harness::Engine, node: &Node, mut args: Value) -> Value {
    args["engine"] = json!(engine.as_str());
    args["ask"] = json!(svrngs_node::home::address(node.api_addr().port()));
    let out = tokio::process::Command::new("node")
        .arg(harness::repo().join("web/tests/home.mjs"))
        .arg(args.to_string())
        .output()
        .await
        .expect("node runs: the browser tests need Node 18 or later");
    assert!(
        out.status.success(),
        "the browser half failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(stdout.lines().last().expect("printed")).expect("JSON")
}

/// Every string among `seen[at]`, flattened.
fn words<'a>(seen: &'a Value, at: &str) -> Vec<&'a str> {
    fn walk<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
        match v {
            Value::String(s) => out.push(s),
            Value::Array(a) => a.iter().for_each(|v| walk(v, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(&seen[at], &mut out);
    out
}

in_both_engines!(
    /// **A leave is said as a leave, everywhere** (groups-leave (c)).
    /// Sam, who made Trip, is shown the hand-on beside the leave, in a
    /// person's words. Ana presses "Leave Trip" on her home page, whose
    /// words said before what stays and that this computer cannot come
    /// back yet; her page then says she left and lists Trip under "Groups
    /// you left". Sam's page says "Ana left on …", never "removed". Sam
    /// sends Ana a new link, and her page, pasted it, says she cannot join
    /// again from this computer yet, never "Join Trip". Red at 1676df59,
    /// its page showing no hand-on and no "Leave Trip". Planted, each watched failing: the
    /// removal's words reused for a leave (question.js `removal` as at
    /// 1676df59, "Ana removed Ana"); a computer that left let to redeem
    /// again (join.rs `asked` without its `Standing::Left` check, "Join
    /// Trip").
    leaving_on_the_home_page_is_said_as_a_leave_everywhere
);
async fn leaving_on_the_home_page_is_said_as_a_leave_everywhere(engine: harness::Engine) {
    let (_relay, ours, _sam_dir, sam) = common::sams_computer(6).await;
    let (trip, _) = cli::new_group(&sam, name("Trip"), name("Sam"), None, None, &ours)
        .await
        .expect("Sam makes Trip");
    let link = || {
        let made = sam.host().link(&trip.id(), false, None);
        client_link(&made.expect("Sam makes a link to Trip").token)
    };
    let ana_dir = tempfile::tempdir().unwrap();
    let ana_key = SigningKey::from_bytes(&[7; 32]);
    let ana = Node::start(
        ana_dir.path().to_path_buf(),
        ana_key.clone(),
        &ours,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("Ana's computer starts");
    tokio::time::timeout(Duration::from_secs(20), ana.endpoint().online())
        .await
        .expect("Ana's computer reaches the relay on 127.0.0.1");
    let (sent, as_ana) = (Link::parse(&link()).unwrap(), name("Ana"));
    ana.host()
        .join(&sent, &as_ana)
        .await
        .expect("Ana joins Trip");

    let seen = home_page(engine, &sam, json!({ "all": "#home section p" })).await;
    eprintln!("Sam's page, before: {seen:#}");
    let hand =
        "You keep the order of removals in Trip. Choose who keeps it after you, then leave: ";
    assert!(
        words(&seen, "all").iter().any(|w| w.starts_with(hand)),
        "{seen:#}"
    );

    let all = "#home > p, #home section h2, #home section p";
    let seen = home_page(
        engine,
        &ana,
        json!({ "all": all, "do": [["#home section button:has-text('Leave Trip')", null]] }),
    )
    .await;
    eprintln!("Ana's page, leaving: {seen:#}");
    let before = words(&seen, "all").join("\n");
    assert!(
        before.contains("Rejoining needs a new invitation, and this computer can’t rejoin yet."),
        "{seen:#}"
    );
    let after = words(&seen, "after").join("\n");
    assert!(after.contains("You left Trip."), "{seen:#}");
    assert!(after.contains("Groups you left"), "{seen:#}");
    assert!(after.contains("Trip (Sam’s, since"), "{seen:#}");

    let ana_out = || {
        let register = sam.rail().register(&trip.id()).unwrap();
        register.removed.contains_key(&actor_of(&ana_key))
    };
    let told = tokio::time::timeout(Duration::from_secs(30), async {
        while !ana_out() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
    assert!(
        told.await.is_ok(),
        "Sam's computer holds Ana's leave within 30 s"
    );
    let seen = home_page(engine, &sam, json!({ "all": "#home .removals li" })).await;
    eprintln!("Sam's page, after: {seen:#}");
    let removals = words(&seen, "all");
    assert!(
        removals.iter().any(|w| w.starts_with("Ana left on ")),
        "{seen:#}"
    );
    assert!(removals.iter().all(|w| !w.contains("removed")), "{seen:#}");

    let seen = home_page(
        engine,
        &ana,
        json!({ "all": "#join p, #join button", "do": [["#join input[name=link]", link(), true]] }),
    )
    .await;
    eprintln!("Ana's page, sent a new link: {seen:#}");
    let again = words(&seen, "after").join("\n");
    assert!(
        again.contains(
            "You’re no longer in Trip on this computer, and you can’t join it again from here yet."
        ),
        "{seen:#}"
    );
    assert!(!again.contains("Join Trip"), "{seen:#}");
    ana.endpoint().close().await;
    sam.endpoint().close().await;
}
