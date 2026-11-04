//! The group's page in two tabs of one browser (groups-tabs (a);
//! ralph/REVIEW_FINDINGS.md #176): one person's key, from that browser's
//! store, and a relay that keeps one connection per key
//! (iroh-relay-1.0.2/src/server/clients.rs:80-92).

use super::*;

/// How long both tabs are asked after the second opens: past the 16-24 s
/// at which arch-stream's page endpoint took its key back
/// (demo/runs/stream/2026-10-15/README.md, 1).
const FOR: u64 = 60_000;

in_webkit!(
    /// **Two tabs of the group's page reach the group** (#176). Bo opens
    /// house's page, then opens it again in a second tab of the same
    /// browser. For 60 s each tab in turn asks "What apps use" of ada's
    /// shelf, which asks, so each dials ada's computer through its page's
    /// own endpoint. Every ask in tab one is answered, and in tab two too,
    /// or tab two says house is open in another tab and asks nothing; then
    /// its "Open it here" opens house there, answers an ask, and tab one
    /// says house is open in another tab (groups-tabs (b)). Red at
    /// c43d2cc7, each tab its own endpoint: tab one's every ask "did not
    /// answer" (demo/runs/tabs/2026-10-16/).
    the_group_s_page_in_two_tabs_reaches_the_group_from_one
);
async fn the_group_s_page_in_two_tabs_reaches_the_group_from_one(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let bo = signing(7);
    let ada = node(dir.path(), &relay, &bo).await;
    let shelf = page_origin("<!doctype html><title>ada's shelf</title>").await;
    let lends = svrngs_node::lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "port": shelf.port(), "kind": "k", "refused": [], "asks": "the shelf" });
    std::fs::write(lends.join("shelf.json"), body.to_string()).unwrap();
    // One app bo stopped, so each ask of what apps use answers a line.
    let name = svrngs_node::lend::LendName::try_from("shelf".to_string()).unwrap();
    let choices = svrngs_node::lend::choices_path(dir.path(), &name);
    std::fs::create_dir_all(choices.parent().unwrap()).unwrap();
    let stopped = json!({ actor_of(&bo): { label(9, "app"): { "said": "stop", "at": 1 } } });
    std::fs::write(&choices, stopped.to_string()).unwrap();

    let seen = drive(
        engine,
        "tabs.mjs",
        json!({
            "site": site(),
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "for": FOR,
        }),
        None,
    )
    .await;
    let asks = seen["asks"].as_array().cloned().unwrap_or_default();
    let summary: Vec<String> = asks
        .iter()
        .map(|a| {
            format!(
                "{} +{} ms ({} ms): {}",
                a["tab"], a["at"], a["ms"], a["words"]
            )
        })
        .collect();
    eprintln!(
        "the tabs' run ({}):\n  one: {}\n  two: {}\n  {}\n  taken: {}",
        engine.as_str(),
        seen["one"]["status"],
        seen["two"]["status"],
        summary.join("\n  "),
        json!({ "one": seen["taken"]["one"]["status"], "two": seen["taken"]["two"]["status"], "ask": seen["taken"]["ask"] })
    );
    const ELSEWHERE: &str = "You have house open in another tab";
    assert_eq!(seen["one"]["good"], true, "tab one lists house: {seen:#}");
    let two = seen["two"]["status"].as_str().unwrap_or_default();
    let offered = two == ELSEWHERE;
    assert!(
        offered || seen["two"]["status"] == seen["one"]["status"],
        "tab two lists house, or says it is open in another tab: {two:?}\n{seen:#}"
    );
    let answered = |a: &Value| {
        a["words"]
            .as_str()
            .is_some_and(|w| w.contains("was stopped using ada’s shelf"))
    };
    // Tab two, open elsewhere, shows no list to ask.
    let dropped: Vec<&String> = asks
        .iter()
        .zip(&summary)
        .filter(|(a, _)| !(answered(a) || offered && a["tab"] == "two" && a["words"].is_null()))
        .map(|(_, s)| s)
        .collect();
    assert!(
        asks.len() >= 4 && dropped.is_empty(),
        "every ask in each tab that lists house is answered for {FOR} ms; dropped:\n  {}\n{seen:#}",
        dropped
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    for (tab, after) in [("one", "oneAfter"), ("two", "twoAfter")] {
        assert_eq!(
            seen[after]["status"], seen[tab]["status"],
            "tab {tab} says the same after: {seen:#}"
        );
    }
    if offered {
        let taken = &seen["taken"];
        assert!(
            taken["two"]["status"]
                .as_str()
                .is_some_and(|s| s.starts_with("You’re in house"))
                && answered(&json!({ "words": taken["ask"] })),
            "tab two, opened here, lists house and reaches ada: {taken:#}"
        );
        assert_eq!(taken["one"]["status"], ELSEWHERE, "{taken:#}");
    }
    drop(ada);
}
