//! **Money** (apps/expenses; the shelf's first app): Ada makes house and
//! brings Money from her home page; Bo joins by the link her page makes.
//! Ada adds the groceries, paid by her; Bo, on his own computer, says he
//! was in on it, owes half, and says he paid it back, and both are square.
//! Then a program on Bo's computer tries a correction of Ada's groceries:
//! the record refuses it as it is written, saying whose entry it is, and
//! both pages still count the groceries. Ada's own correction of an entry
//! of hers still counts. Then Ada names Bo in on
//! dinner while Bo's Money is open: his page, not opened again, asks him;
//! her line says what he owes once he says he was in on it; and his half
//! counts once he says "Yes, I was in on dinner".
//!
//! The invariant it relies on: **a correction counts only from its
//! author's person**: house is sources-v1, and its interpreter counts a
//! `Correct` only from the person whose entry it names (EUDOXIA_JUDGE §2c,
//! Correct), so the write side refuses another's (`Refused::Uncounted`).

use std::time::Duration;

use serde_json::{json, Value};

use crate::harness::Engine;
use crate::people::{late, sees, Person, Scene};

/// `who`'s record holds Money's expense `what`, not voided: no correction
/// of it counted.
async fn kept(who: &Person, group: &str, what: &str) {
    let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
    let ops = log["ops"].as_array().cloned().unwrap_or_default();
    let it = ops.iter().find(|o| o["payload"]["description"] == what);
    assert!(
        it.is_some_and(|o| o["voided"] == false),
        "{}'s record keeps {what}, not voided: {log:#}",
        who.name
    );
}

/// The groceries, split between Ada and Bo, as Bo's page and then Ada's
/// say it. Bo's home page opens his own computer's copy of Money, listed
/// first (node.rs `lent`; question.js `once`), so his share is signed by
/// his computer and is his alone, no other computer's
/// (demo/runs/authorship/2026-10-24/DESIGN.md §7).
const BOTH: [&str; 2] = [
    "Ada paid $30.00 for the groceries, split between Ada, Bo.",
    "You paid $30.00 for the groceries, split between Ada, Bo.",
];

/// The words `who`'s Money says, opened from their home page (fence 2:
/// a lend opened from the home page), after `steps` on Money's page.
async fn opened(engine: Engine, who: &Person, steps: Value) -> Value {
    let url = money_at(engine, who).await;
    let seen = sees(engine, "carried.mjs", json!({ "url": url, "do": steps })).await;
    eprintln!("{}'s Money: {seen:#}", who.name);
    seen
}

/// Where `who`'s Money opens from their home page.
async fn money_at(engine: Engine, who: &Person) -> String {
    let page = who.home_page();
    let opened = sees(
        engine,
        "home.mjs",
        json!({ "ask": page, "heading": false, "open": "Money" }),
    );
    let opened = opened.await;
    let url = opened["opened"]["url"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert_eq!(
        opened["opened"]["title"], "Money",
        "{} opens Money from the home page: {opened:#}",
        who.name
    );
    url
}

/// Once `who`'s record holds an act of Money's for which `holds` is true,
/// within 60 s: that act.
async fn held(who: &Person, group: &str, holds: impl Fn(&Value) -> bool) -> Value {
    for _ in 0..120 {
        let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
        let ops = log["ops"].as_array().into_iter().flatten();
        if let Some(op) = ops.into_iter().find(|o| holds(o)) {
            return op.clone();
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    let log = who.log();
    late(format!(
        "{}'s record does not hold it within 60 s; its log is {}",
        who.name,
        log.display()
    ))
    .await
}

scene!(
    /// Planted, watched failing: svrngs-core command.rs `Uncounted`'s
    /// words made the one generic sentence for every refusal, so Bo is not
    /// told whose entry it is.
    money
);
async fn money(engine: Engine) {
    let scene = Scene::new("money", engine).await;
    let (ada, house) = scene.founds("Ada", "house").await;

    // Ada brings Money from her home page.
    // Ticked first: the shelf's boxes start unticked (the operator, 2026-10-26).
    let bring = json!([
        ["#home label:has-text('Money') input", true],
        ["button:has-text('Bring the ticked ones')", null, true]
    ]);
    let brought = sees(engine, "home.mjs", json!({ "ask": ada.home_page(), "heading": false, "all": "#home section h3 ~ p", "do": bring })).await;
    assert!(
        brought["after"][0]
            .to_string()
            .contains("You brought Money to house"),
        "Ada's home page says Money is brought: {brought:#}"
    );

    let bo = scene.joins("Bo", &ada.invites(&house).await, "house").await;

    // Ada adds the groceries, paid by her.
    let steps = json!([["#what", "the groceries"], ["#amount", "30"], ["#add"]]);
    let seen = opened(engine, &ada, steps).await;
    let after = seen["after"][0].as_str().unwrap_or_default();
    assert!(
        after.contains("You paid $30.00 for the groceries, split between Ada."),
        "Ada sees her groceries: {seen:#}"
    );

    // Bo, on his computer: in on it, owes half, pays it back, square.
    bo.opens(&house, "Money").await;
    held(&bo, &house, |o| {
        o["payload"]["description"] == "the groceries"
    })
    .await;
    let steps = json!([
        ["text=Say you were in on the groceries"],
        ["text=Say you paid Ada $15.00"]
    ]);
    let seen = opened(engine, &bo, steps).await;
    let shared = seen["after"][0].as_str().unwrap_or_default();
    assert!(
        shared.contains("You owe $15.00.") && shared.contains(BOTH[0]),
        "Bo sees his half: {seen:#}"
    );
    let settled = seen["after"][1].as_str().unwrap_or_default();
    assert!(
        settled.contains("You’re square with everyone."),
        "Bo is square: {seen:#}"
    );

    // A program on Bo's computer corrects Ada's groceries away.
    let groceries = held(&bo, &house, |o| {
        o["payload"]["description"] == "the groceries"
    })
    .await;
    let erase = json!({ "op": "correct", "corrects": groceries["id"], "replacement": null });
    let (status, said) = bo
        .post(&format!("/v1/rail/append?namespace={house}"), &erase)
        .await;
    // The record refuses it as it is written, saying whose entry it is and
    // what to do; nothing reaches either computer.
    let whose = "Only Ada can change Ada’s entry, so the record would not count Bo’s change. Ask Ada to change it.";
    assert_eq!(status, 422, "{said}");
    assert_eq!(
        said["error"], whose,
        "Bo is told whose the entry is: {said}"
    );
    for (who, both) in [(&bo, BOTH[0]), (&ada, BOTH[1])] {
        kept(who, &house, "the groceries").await;
        let seen = opened(engine, who, json!([])).await;
        let said = seen["said"].as_str().unwrap_or_default();
        assert!(
            said.contains(both) && said.contains("You’re square with everyone."),
            "{}'s Money still counts the groceries: {seen:#}",
            who.name
        );
    }

    // Ada's own correction of an entry of hers still counts (fence 2).
    let steps = json!([["#what", "a lamp"], ["#amount", "12"], ["#add"]]);
    opened(engine, &ada, steps).await;
    let lamp = held(&ada, &house, |o| o["payload"]["description"] == "a lamp").await;
    let undo = json!({ "op": "correct", "corrects": lamp["id"], "replacement": null });
    let (status, said) = ada
        .post(&format!("/v1/rail/append?namespace={house}"), &undo)
        .await;
    assert!(status < 300, "{status} {said}");
    let seen = opened(engine, &ada, json!([])).await;
    let said = seen["said"].as_str().unwrap_or_default();
    assert!(
        !said.contains("a lamp"),
        "Ada's own correction counts: {seen:#}"
    );

    // Ada names Bo in on dinner (the operator, 2026-10-26; demo/xenia/
    // findings/split-leaves-out-ana.md) while Bo's Money is open: his page,
    // not opened again, asks him (the fifth set: Ana never asked, 0 of 6,
    // demo/xenia/findings/index.md); her line says he owes his half once he
    // says so, and it counts only then (Kim expected it at once, the fifth
    // set's `together` run 2). Planted, watched failing: Money's page as
    // 5c6e3b0d left it, drawn once at load, and "waiting on Bo".
    let drawn = tempfile::tempdir().unwrap();
    let ready = drawn.path().join("bo-money-drawn");
    let url = money_at(engine, &bo).await;
    let ask = "Ada says you were in on dinner: $10.00.";
    let watching = sees(
        engine,
        "carried.mjs",
        json!({ "url": url, "do": [["wait", ask]], "ready": ready }),
    );
    let steps = json!([
        ["#what", "dinner"],
        ["#amount", "20"],
        ["label:has-text('Bo') input", true],
        ["#add"]
    ]);
    let adding = async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        opened(engine, &ada, steps).await
    };
    let (watched, seen) = tokio::join!(watching, adding);
    eprintln!("Bo's Money, open as Ada adds dinner: {watched:#}");
    let after = seen["after"][0].as_str().unwrap_or_default();
    assert!(
        after.contains("You paid $20.00 for dinner, split between Ada; Bo owes you $10.00 once they say they were in on it."),
        "Ada's line says what Bo owes once he says so: {seen:#}"
    );
    let first = watched["first"].as_str().unwrap_or_default();
    assert!(!first.contains("dinner"), "{watched:#}");
    assert!(
        watched["after"][0].to_string().contains(ask),
        "Bo is asked on the page he has open: {watched:#}"
    );
    held(&bo, &house, |o| o["payload"]["description"] == "dinner").await;
    let seen = opened(engine, &bo, json!([["text=Yes, I was in on dinner"]])).await;
    let yes = seen["after"][0].as_str().unwrap_or_default();
    assert!(
        yes.contains("You owe $10.00.") && yes.contains("split between Ada, Bo."),
        "Bo's own yes is his half: {seen:#}"
    );
    drop((ada, bo));
}
