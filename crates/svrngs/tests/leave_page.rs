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
#[path = "common/home_page.rs"]
mod home_page;

use std::time::Duration;

use serde_json::{json, Value};
use svrngs::cli;
use svrngs::group::{self, Name};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::link::{client_link, Link};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, SigningKey};

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

/// Drive `web/tests/home.mjs` in `engine` at `node`'s home page, as `args`
/// say.
async fn home_of(engine: harness::Engine, node: &Node, args: Value) -> Value {
    let page = svrngs_node::home::address(node.api_addr().port());
    home_page::home_page(engine, &page, args).await
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
    /// Trip"). Her computer's old address of a lend of Sam's says she left,
    /// "You left Trip on 16 Oct." (#182): red at 169d7a8c in both, "nothing
    /// is lent here." (origin.rs `dialled`'s `Unlent`).
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

    let seen = home_of(engine, &sam, json!({ "all": "#home section p" })).await;
    eprintln!("Sam's page, before: {seen:#}");
    let hand =
        "You keep the order of removals in Trip. Choose who keeps it after you, then leave: ";
    assert!(
        words(&seen, "all").iter().any(|w| w.starts_with(hand)),
        "{seen:#}"
    );

    let all = "#home > p, #home section h2, #home section p";
    let seen = home_of(
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
    // An old address on her computer of a lend of Sam's says she left
    // (#182), never "nothing is lent here".
    let sams = actor_of(&SigningKey::from_bytes(&[6; 32]));
    let notes = svrngs_core::pass::Label::of_hex(&sams, "notes").unwrap();
    let old = svrngs_node::origin::origin(&notes, ana.api_addr().port());
    let seen = home_page::home_page(engine, &old, json!({ "heading": false, "read": "p" })).await;
    eprintln!("Ana's old address of Sam's notes: {seen:#}");
    let said = seen["read"].as_str().unwrap_or_default();
    assert!(said.starts_with("You left Trip on "), "{seen:#}");

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
    let seen = home_of(engine, &sam, json!({ "all": "#home .removals li" })).await;
    eprintln!("Sam's page, after: {seen:#}");
    let removals = words(&seen, "all");
    assert!(
        removals.iter().any(|w| w.starts_with("Ana left on ")),
        "{seen:#}"
    );
    assert!(removals.iter().all(|w| !w.contains("removed")), "{seen:#}");

    let seen = home_of(
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

/// `node`'s `/v1/groups`, what its home page reads: whether it says
/// `group` is one its person left.
async fn left_on(node: &Node, group: &str) -> bool {
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let url = format!("http://{}/v1/groups", node.api_addr());
    let held = http.get(url).send().await.unwrap().text().await.unwrap();
    let held: Value = serde_json::from_str(&held).unwrap();
    let groups = held["groups"].as_array().cloned().unwrap_or_default();
    groups.iter().any(|g| g["id"] == group && g["left"] == true)
}

/// Up to `within`, with a round on `node` each second if `rounds`:
/// whether its home page says it left `group`.
async fn until_left(node: &Node, group: &str, within: Duration, rounds: bool) -> bool {
    let t0 = std::time::Instant::now();
    while t0.elapsed() < within {
        if left_on(node, group).await {
            return true;
        }
        if rounds {
            let _ = tokio::time::timeout(Duration::from_secs(10), node.round()).await;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    left_on(node, group).await
}

/// Up to 30 s: whether `node` holds `key` out of `group`.
async fn out_of(node: &Node, group: &str, key: &str) -> bool {
    let out = || {
        node.rail()
            .register(group)
            .unwrap()
            .removed
            .contains_key(key)
    };
    let held = tokio::time::timeout(Duration::from_secs(30), async {
        while !out() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
    held.await.is_ok()
}

/// What a computer off holds, `from`, copied to `to` but its blob store,
/// which the stopped node in this process still keeps open: started there
/// with its key, it comes back on holding what it held.
fn held_again(from: &std::path::Path, to: &std::path::Path) {
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let (at, name) = (e.path(), e.file_name());
        if name == "blobs" {
            continue;
        }
        if at.is_dir() {
            std::fs::create_dir_all(to.join(&name)).unwrap();
            held_again(&at, &to.join(&name));
        } else {
            std::fs::copy(&at, to.join(&name)).unwrap();
        }
    }
}

/// A computer of `key`'s in `dir`, online at the relay `ours` names. Its
/// own round runs at its start and then only when a test asks one, as
/// `tests/scale.rs` passes a day, so what reaches it between is told.
async fn computer(dir: &std::path::Path, key: &SigningKey, ours: &RelayConfig) -> Node {
    let node = Node::start_every(
        dir.to_path_buf(),
        key.clone(),
        ours,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
        Duration::from_secs(86_400),
    )
    .await
    .expect("a computer starts");
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the computer reaches the relay on 127.0.0.1");
    node
}

in_both_engines!(
    /// **A leave reaches the leaver's other computer, on or off as it
    /// lands** (groups-leave-others (a), #179). Sam makes Films, Trip and
    /// Walk; Ana joins each from Sam's links on her first computer, and
    /// her second joins each from her first's links for her devices, as
    /// Ana. Ana leaves Films from her first computer, as `svrngs leave`
    /// asks, while her second is off; it comes back on, still in Trip and
    /// Walk at Sam's, and within 90 s its home page says she left Films.
    /// She presses "Leave Trip" on her first computer's home page while
    /// her second is on, and within 30 s, no round of its own run, its
    /// page says she left Trip: told by her first.
    /// She leaves Walk with her second off again, now in no group of
    /// Sam's there; back on, within 90 s its page says she left Walk, and
    /// lists all three under "Groups you left". Off is the node stopped
    /// and started again in this process from a copy of what it held, its
    /// blob store apart ([`held_again`]). Red at 1956f9bf in all three
    /// (demo/runs/leave-others/2026-10-16/a-red.out): the leave told only
    /// to the computers it did not cut (removal.rs `cut`, `left !=
    /// cut.contains(k)`), and no lender telling a computer it cut.
    a_leave_reaches_the_leavers_other_computer
);
async fn a_leave_reaches_the_leavers_other_computer(engine: harness::Engine) {
    use svrngs_node::removal::{LeaveAsk, Outcome, LEAVES_ROUTE};
    let (_relay, ours, _sam_dir, sam) = common::sams_computer(6).await;
    let mut groups = Vec::new();
    for g in ["Films", "Trip", "Walk"] {
        let (seed, _) = cli::new_group(&sam, name(g), name("Sam"), None, None, &ours)
            .await
            .expect("Sam makes a group");
        groups.push(seed.id());
    }
    let [films, trip, walk] = [0, 1, 2].map(|i| groups[i].clone());
    let (one_dir, two_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (one_key, two_key) = (
        SigningKey::from_bytes(&[7; 32]),
        SigningKey::from_bytes(&[8; 32]),
    );
    let one = computer(one_dir.path(), &one_key, &ours).await;
    let mut two = computer(two_dir.path(), &two_key, &ours).await;
    for g in &groups {
        let made = sam.host().link(g, false, None).expect("Sam makes a link");
        let sent = Link::parse(&client_link(&made.token)).unwrap();
        one.host()
            .join(&sent, &name("Ana"))
            .await
            .expect("Ana joins");
        let made = one
            .host()
            .link(g, true, None)
            .expect("Ana makes a link for her devices");
        let sent = Link::parse(&client_link(&made.token)).unwrap();
        two.host()
            .join(&sent, &name("whoever"))
            .await
            .expect("Ana's second computer joins as Ana");
    }
    let two_actor = actor_of(&two_key);
    let held = || {
        groups.iter().all(|g| {
            let s = sam.rail().standing(g).unwrap();
            let o = one.rail().standing(g).unwrap();
            s.member(&two_actor)
                .is_some_and(|m| m.person.as_str() == "Ana")
                && o.names(&two_actor)
        })
    };
    let settled = tokio::time::timeout(Duration::from_secs(60), async {
        while !held() {
            for n in [&sam, &one, &two] {
                let _ = tokio::time::timeout(Duration::from_secs(10), n.round()).await;
            }
        }
    });
    assert!(
        settled.await.is_ok(),
        "control: Sam's and Ana's first computer hold Ana's second as Ana"
    );
    // As `svrngs leave` asks: what happened.
    let leave = |g: &str| {
        let ask = reqwest::Client::builder().no_proxy().build().unwrap();
        let ask = ask.post(format!("http://{}{LEAVES_ROUTE}", one.api_addr()));
        let body = serde_json::to_string(&LeaveAsk {
            group: g.to_string(),
        });
        let sent = ask
            .header("content-type", "application/json")
            .body(body.unwrap())
            .send();
        async {
            serde_json::from_str::<Outcome>(&sent.await.unwrap().text().await.unwrap()).unwrap()
        }
    };

    // Off as the leave of Films lands, in Trip and Walk still.
    two.endpoint().close().await;
    drop(two);
    let left = leave(&films).await;
    assert_eq!(
        left,
        Outcome::Left,
        "Ana leaves Films from her first computer"
    );
    assert!(
        out_of(&sam, &films, &two_actor).await,
        "control: Sam's computer holds Ana's second out of Films within 30 s"
    );
    let again = tempfile::tempdir().unwrap();
    held_again(two_dir.path(), again.path());
    two = computer(again.path(), &two_key, &ours).await;
    assert!(
        until_left(&two, &films, Duration::from_secs(90), true).await,
        "back on, Ana's second computer says she left Films"
    );

    // On as the leave of Trip lands, pressed on her first's home page.
    let press = json!({ "all": "#home > p", "do": [["#home section button:has-text('Leave Trip')", null]] });
    let seen = home_of(engine, &one, press).await;
    eprintln!("Ana's first computer's page, leaving Trip: {seen:#}");
    assert!(
        until_left(&two, &trip, Duration::from_secs(30), false).await,
        "on, Ana's second computer says she left Trip"
    );

    // Off as the leave of Walk lands, in no group of Sam's there.
    two.endpoint().close().await;
    drop(two);
    assert_eq!(leave(&walk).await, Outcome::Left);
    assert!(
        out_of(&sam, &walk, &two_actor).await,
        "control: Sam's computer holds it"
    );
    let third = tempfile::tempdir().unwrap();
    held_again(again.path(), third.path());
    two = computer(third.path(), &two_key, &ours).await;
    assert!(
        until_left(&two, &walk, Duration::from_secs(90), true).await,
        "back on, Ana's second computer says she left Walk"
    );
    let seen = home_of(engine, &two, json!({ "all": "#left p" })).await;
    eprintln!("Ana's second computer's page: {seen:#}");
    let gone = words(&seen, "all");
    for g in ["Films", "Trip", "Walk"] {
        assert!(
            gone.iter()
                .any(|w| w.starts_with(&format!("{g} (Sam’s, since"))),
            "{seen:#}"
        );
    }
    for n in [&two, &one, &sam] {
        n.endpoint().close().await;
    }
}
