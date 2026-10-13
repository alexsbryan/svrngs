//! A group made on the home page (start-group; DESIGN §3, "The home
//! page"): on a computer whose svrngs runs as install.sh keeps it — the
//! bare `svrngs`, this build's, in a `HOME` of its own, where `svrngs new`
//! is refused while it runs (demo/runs/start/2026-10-13-home/NEEDS_HUMAN.md)
//! — the page, in Playwright's headless Chromium and in its WebKit, makes
//! a group by its name and the person's; its folder is placed at
//! `~/svrngs/<name>`, where `join` puts everyone's; a link the page makes
//! for it is redeemed by another computer, which then stands in it; and a
//! name already held is refused in a person's words. The group keeps the
//! page this computer's svrngs came from, as install.sh keeps it, so the
//! link is an https link on it. Neither computer is
//! restarted. Dee's computer holds a first group, Lab, made by `svrngs new
//! --relay` at a relay on 127.0.0.1, so it is met there alone, as Bo's
//! computer is: nothing leaves this computer. The browser half is
//! `web/tests/home.mjs`, which fails naming the install command when
//! Playwright or its browsers are absent — it never skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
mod common;

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs::cli;
use svrngs::group::Name;
use svrngs_node::link::Link;
use svrngs_node::{actor_of, SigningKey};
use tokio::io::AsyncBufReadExt;

/// `svrngs <args>`, this build's, with `home` its `HOME` and nothing else
/// of this computer's environment but its `PATH`.
fn svrngs(home: &Path, args: &[&str]) -> tokio::process::Command {
    let mut c = tokio::process::Command::new(env!("CARGO_BIN_EXE_svrngs"));
    c.args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true);
    c
}

/// Drive `web/tests/home.mjs` in `engine` at `page`, as `args` say.
async fn home_page(engine: harness::Engine, page: &str, mut args: Value) -> Value {
    args["engine"] = json!(engine.as_str());
    args["ask"] = json!(page);
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

in_both_engines!(
    /// **A group made on the home page, where svrngs runs from login**
    /// (start-group (c)). Planted, watched failing: the page as 99c8638
    /// left it, with no Make a group.
    the_home_page_makes_a_group_another_computer_joins_with_no_restart
);
async fn the_home_page_makes_a_group_another_computer_joins_with_no_restart(
    engine: harness::Engine,
) {
    let (relay, _ours, _bo_dir, bo) = common::sams_computer(4).await;
    let home = tempfile::tempdir().unwrap();
    let at = harness::relay_url(&relay);

    // Lab, Dee's first group, met at the relay here: `new` runs until it
    // is stopped, as a terminal's Ctrl-C stops it.
    let mut new = svrngs(home.path(), &["new", "Lab", "--as", "Dee", "--relay", &at])
        .spawn()
        .unwrap();
    let mut out = tokio::io::BufReader::new(new.stdout.take().unwrap()).lines();
    let made = tokio::time::timeout(Duration::from_secs(60), async {
        while let Ok(Some(line)) = out.next_line().await {
            if line.starts_with("Leave this running") {
                return true;
            }
        }
        false
    })
    .await;
    assert_eq!(made, Ok(true), "svrngs new Lab printed all it says");
    new.kill().await.unwrap();

    // The page this computer's svrngs came from, as install.sh keeps it
    // in the data dir: a group made on the home page keeps it, so its
    // links are https links on it.
    let data = home.path().join(".svrngs");
    let came_from = "http://127.0.0.1:9/";
    std::fs::write(data.join("origin"), came_from).unwrap();

    // svrngs as login keeps it: the bare command, left running.
    let mut running = svrngs(home.path(), &[]).spawn().unwrap();
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let api = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let api = std::fs::read_to_string(data.join("api")).unwrap_or_default();
            let api = api.trim().to_string();
            if http.get(format!("{api}/v1/groups")).send().await.is_ok() {
                break api;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("svrngs answers at the API it wrote");
    let port = api.rsplit(':').next().unwrap();
    let page = format!("http://svrngs.localhost:{port}/");

    let all = "#home section h2, #home section p, #home > p";
    let seen = home_page(
        engine,
        &page,
        json!({
            "all": all,
            "do": [
                ["#make input[name=name]", "Trip"],
                ["#make input[name=person]", "Dee"],
                ["#make button:has-text('Make a group')", null],
                ["#home section button:has-text('Make a link to invite someone to Trip')", null, true],
                ["#make input[name=name]", "trip"],
                ["#make input[name=person]", "Dee"],
                ["#make button:has-text('Make a group')", null, true],
            ],
        }),
    )
    .await;
    eprintln!("the home page's run: {seen:#}");

    // Made, and said so; its folder placed where everyone's is.
    let place = home.path().join("svrngs/Trip");
    let made = seen["after"][0].to_string();
    assert!(made.contains("You made Trip."), "{seen:#}");
    assert!(
        made.contains(&format!(
            "Trip’s folder is on this computer at {}",
            place.display()
        )),
        "{seen:#}"
    );
    assert!(place.is_dir(), "{} not made", place.display());

    // A link the page made for it, on the page svrngs came from,
    // redeemed by Bo's computer.
    let shown = seen["after"][1].to_string();
    let https = format!("Send this link to the person you are inviting: {came_from}#");
    assert!(shown.contains(&https), "{seen:#}");
    let at_link = shown.find(&format!("{came_from}#")).expect("a link shown");
    let raw = shown[at_link..].split([' ', '"']).next().unwrap();
    let link = Link::parse(raw).expect("the link shown is a link");
    let bo_key = SigningKey::from_bytes(&[4; 32]);
    let joined = cli::join_group(&bo, &bo_key, &link, &Name::parse("Bo").unwrap()).await;
    let Ok(cli::Said::Joined { seed, .. }) = joined else {
        panic!("Bo is let in: {joined:?}");
    };
    assert_eq!(seed.name().as_str(), "Trip");
    let standing = bo.rail().standing(&seed.id()).unwrap();
    assert!(standing.names(&actor_of(&bo_key)), "Bo stands in Trip");

    // A name already held, refused in a person's words.
    let refused = seen["after"][2].to_string();
    assert!(
        refused.contains("You are in Trip already: give this group another name."),
        "{seen:#}"
    );
    running.kill().await.unwrap();
    bo.endpoint().close().await;
}
