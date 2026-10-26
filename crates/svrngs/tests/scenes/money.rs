//! **Money** (apps/expenses; the shelf's first app): Ada makes house and
//! brings Money from her home page; Bo joins by the link her page makes.
//! Ada adds the groceries, paid by her; Bo, on his own computer, says he
//! was in on it, owes half, and says he paid it back, and both are square.
//! Then a program on Bo's computer writes a correction of Ada's groceries:
//! it is not counted, on either page, and says whose it was. Ada's own
//! correction of an entry of hers still counts.
//!
//! The invariant it relies on: **a correction counts only from its
//! author** (capability-money; expenses.js `voids`). The record voids an
//! act whoever corrects it, so a fold that read the record's `voided`
//! alone would let any member erase anyone's expense.

use std::time::Duration;

use serde_json::{json, Value};

use crate::harness::Engine;
use crate::people::{late, sees, Person, Scene};

/// Once `who`'s record says Money's expense `what` voided, within 60 s:
/// the record voids an act whoever corrects it (svrngs-core admit.rs,
/// `voided`).
async fn voided(who: &Person, group: &str, what: &str) {
    let voided = |o: &Value| o["payload"]["description"] == what && o["voided"] == true;
    for _ in 0..120 {
        let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
        if log["ops"]
            .as_array()
            .is_some_and(|ops| ops.iter().any(voided))
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
    late(format!(
        "{}'s record does not void {what} within 60 s: {log:#}",
        who.name
    ))
    .await
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
    let page = who.home_page();
    let opened = sees(
        engine,
        "home.mjs",
        json!({ "ask": page, "heading": false, "open": "Open Money" }),
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
    let seen = sees(engine, "carried.mjs", json!({ "url": url, "do": steps })).await;
    eprintln!("{}'s Money: {seen:#}", who.name);
    seen
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
    /// Planted, watched failing: expenses.js's `actsOf` reading the
    /// record's `voided` as before fd3347ac, so Bo's correction erases
    /// Ada's groceries on both pages.
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
        ["text=Say you were in on it"],
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
    assert!(
        status < 300,
        "the record takes Bo's correction, as it takes any member's act: {status} {said}"
    );
    // Ada's record voids the groceries, as it voids an act whoever
    // corrects it: its log no longer holds them.
    voided(&ada, &house, "the groceries").await;

    let refused =
        "1 entry can’t be counted: Bo tried to change Ada’s entry, which only Ada can change.";
    for (who, both) in [(&bo, BOTH[0]), (&ada, BOTH[1])] {
        let seen = opened(engine, who, json!([])).await;
        let said = seen["said"].as_str().unwrap_or_default();
        assert!(
            said.contains(both) && said.contains("You’re square with everyone."),
            "{}'s Money still counts the groceries: {seen:#}",
            who.name
        );
        assert!(
            said.contains(refused),
            "{}'s Money says whose the correction was: {seen:#}",
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
    assert!(said.contains(refused), "{seen:#}");

    drop((ada, bo));
}
