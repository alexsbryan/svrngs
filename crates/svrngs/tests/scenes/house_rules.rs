//! **House rules** (demo/house-rules.md; its runs, demo/runs/canon/
//! 2026-10-17-house/ and 2026-10-17-doc/): Ana, Bo and Cy share Elm. Ana
//! brings House rules from her home page, Money and Events unticked, and
//! says the house decides by most of them agreeing. She proposes quiet
//! hours; Cy objects, with a reason; Bo agrees, and with Ana's that is two
//! of three, so it holds, on Bo's page and on Ana's. Then Bo leaves Elm
//! from his home page: the quiet hours still hold, and Ana's page says
//! who has a say now and that Bo had one until he left.
//!
//! The invariant it relies on: **a rule is counted with whoever held a
//! say when it was proposed** (demo/house-rules.md step 10; canon 0.5.0,
//! apps/canon/build.sh `PIN`): a person leaving never unmakes what the
//! house agreed while they were in it.

use serde_json::{json, Value};

use crate::harness::{repo, Engine};
use crate::people::{late, Browsers, Person, Scene};

const QUIET: &str = "Quiet hours are 10pm to 7am";
const WHY: &str = "I work late on Fridays";
const DAY: &str = r"\d{1,2} \w{3}";
/// The rules page drawn, its way of deciding chosen or still asked.
const DRAWN: &str = r"How your house decides\n|How does your house decide its rules\?";

/// The run's plants (run.py `PLANTS`), named by `SCENES_PLANT=house-rules:
/// <name>`, in apps/canon/house.js, which the build carries into svrngs
/// (crates/svrngs-node/build.rs): planted by hand and the tree put back
/// after, as the run did, since a carried app is built in. Here the scene
/// only checks the plant is in place when it is named.
fn planted() {
    let plant = match std::env::var("SCENES_PLANT").as_deref() {
        // A past entry counted only with the people in the house today.
        Ok("house-rules:now") => "(read.people ?? []).includes(o.on_behalf_of ?? o.person)",
        _ => return,
    };
    let house = std::fs::read_to_string(repo().join("apps/canon/house.js")).unwrap();
    assert!(
        house.contains(plant),
        "the plant is in apps/canon/house.js: {plant}"
    );
}

/// `steps` pressed on `who`'s one page by the run's driver: each step's
/// words. A step that fails fails the scene, with the page's words then.
async fn steps(browsers: &mut Browsers, who: &str, steps: Value) -> Vec<Value> {
    let got = browsers
        .tell(json!({ "do": "steps", "who": who, "steps": steps }))
        .await;
    let done = got["done"].as_array().cloned().unwrap_or_default();
    // A step's wait that ran out: the driver's own words for it
    // (`until`'s and a poll's "within", Playwright's "Timeout … exceeded").
    let last = done.last().cloned().unwrap_or(got.clone());
    let error = last["error"].as_str().unwrap_or_default();
    if got["failed"] == true
        && (error.contains(" ms") && error.contains("within") || error.contains("Timeout"))
    {
        late(format!(
            "{who}'s steps did not go as demo/house-rules.md says within their bound: {last:#}"
        ))
        .await;
    }
    assert!(
        got["failed"] != true && got["errors"].as_array().is_none_or(|e| e.is_empty()),
        "{who}'s steps did not go as demo/house-rules.md says: {:#}",
        done.last().cloned().unwrap_or(got.clone())
    );
    done
}

/// A step that the page says `re`, a JavaScript regular expression as the
/// run's driver reads one, within 5 s of the step before.
fn says(re: &str) -> Value {
    json!({ "wait": re, "within": 5_000 })
}

/// The address `who`'s own **Open House rules** holds: their computer's
/// copy (doc.py `rules_at`), as their home page lists it.
async fn rules_at(who: &Person, group: &str) -> String {
    let people = who.get(&format!("/v1/rail/people?namespace={group}")).await;
    let people = people["people"].as_array().cloned().unwrap_or_default();
    let key = people
        .iter()
        .find(|p| p["person"] == who.name.as_str() && p["device"] == "computer")
        .map(|p| p["key"].clone())
        .expect("the person's computer is in the group");
    let lends = who.get(&format!("/v1/lends?namespace={group}")).await;
    let lends = lends["lent"].as_array().cloned().unwrap_or_default();
    lends
        .iter()
        .find(|l| l["kind"] == "House-rules" && l["key"] == key)
        .and_then(|l| l["origin"].as_str())
        .unwrap_or_else(|| panic!("{} lists no House rules of its own: {lends:#?}", who.name))
        .to_string()
}

scene!(
    /// Planted, watched failing: the run's `now` plant (run.py `PLANTS`),
    /// a past entry counted only with the people in the house today, by
    /// hand in apps/canon/house.js and `SCENES_PLANT=house-rules:now`:
    /// once Bo leaves, Ana's page has no rules.
    house_rules
);
async fn house_rules(engine: Engine) {
    planted();
    let scene = Scene::new("house_rules", engine).await;
    let mut browsers = Browsers::driven(
        "demo/runs/canon/2026-10-17-house/people.mjs",
        engine,
        &[
            ("ana", Engine::Chromium),
            ("bo", Engine::Chromium),
            ("cy", Engine::Chromium),
        ],
    )
    .await;

    // Elm: Ana makes it; Bo and Cy join by its link.
    let (ana, elm) = scene.founds("Ana", "Elm").await;
    let bo = scene.joins("Bo", &ana.invites(&elm).await, "Elm").await;
    let cy = scene.joins("Cy", &ana.invites(&elm).await, "Elm").await;
    let open_it = |who: &Person| json!({ "poll": who.home_page(), "re": "Open House rules", "within": 60_000, "every": 2_000 });

    // 5. Ana brings House rules, Money and Events unticked.
    let label = |app: &str| format!("#home label:has-text('{app}') input");
    let ticked = |app: &str| format!("#home label:has-text('{app}') input:checked");
    let done = steps(
        &mut browsers,
        "ana",
        json!([
            { "goto": ana.home_page() },
            { "wait": "Bring the ticked ones to Elm" },
            { "count": ticked("Money") },
            { "count": ticked("Events") },
            { "count": ticked("House rules") },
            { "uncheck": label("Money") },
            { "uncheck": label("Events") },
            { "click": "Bring the ticked ones to Elm", "until": "You brought|Nothing was brought|already" },
            says(r"You brought House rules to Elm: everyone in it can open it in a few seconds\."),
        ]),
    )
    .await;
    let ticked: Vec<_> = done[2..5].iter().map(|d| d["value"].clone()).collect();
    assert_eq!(
        ticked,
        [1, 1, 1],
        "step 5: Money, Events and House rules each ticked"
    );

    // 6. Ana says how the house decides: most of you agree.
    steps(&mut browsers, "ana", json!([open_it(&ana)])).await;
    let done = steps(
        &mut browsers,
        "ana",
        json!([
            { "goto": rules_at(&ana, &elm).await },
            { "wait": DRAWN, "within": 60_000 },
            says(r"How does your house decide its rules\?"),
            says(r"Most of you agree\."),
            { "count": "input[name=way]" },
            { "count": "input[name=way]:checked" },
            { "check": "input[name=way][value=most]" },
            { "click": "Make this how your house decides", "until": r"How your house decides\n" },
            says(r"A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them\."),
        ]),
    )
    .await;
    let ways: Vec<_> = done[4..6].iter().map(|d| d["value"].clone()).collect();
    assert_eq!(ways, [3, 0], "step 6: three ways, none ticked");

    // 7. Ana proposes quiet hours.
    steps(
        &mut browsers,
        "ana",
        json!([
            { "fill": "#rule", "value": QUIET },
            { "click": "Propose this rule to everyone in your house", "until": "You proposed it on" },
            says(&format!(r"What’s proposed\n+{QUIET}\n+You proposed it on {DAY}\.")),
        ]),
    )
    .await;

    // 8. Cy objects, with a reason: still proposed.
    steps(&mut browsers, "cy", json!([open_it(&cy)])).await;
    steps(
        &mut browsers,
        "cy",
        json!([
            { "goto": rules_at(&cy, &elm).await },
            { "wait": QUIET, "within": 60_000 },
            { "click": "Object", "in": QUIET, "until": r"Why do you object\?" },
            { "fill": "input.why", "value": WHY },
            { "click": "Send your objection", "until": "You object" },
            says(&format!("You object: “{WHY}”")),
            says(r"Your house has no rules yet\."),
        ]),
    )
    .await;

    // 9. Bo agrees: two of three, so it holds, on Bo's page and on Ana's.
    steps(&mut browsers, "bo", json!([open_it(&bo)])).await;
    steps(
        &mut browsers,
        "bo",
        json!([
            { "goto": rules_at(&bo, &elm).await },
            { "wait": format!("Cy objects: “{WHY}”"), "within": 60_000 },
            { "click": "Agree", "in": QUIET, "until": format!("Your house’s rules\n+{QUIET}") },
            says(&format!(r"{QUIET} \(Ana proposed it on {DAY}\)")),
        ]),
    )
    .await;
    let ana_rules = rules_at(&ana, &elm).await;
    steps(
        &mut browsers,
        "ana",
        json!([
            { "goto": &ana_rules },
            { "wait": DRAWN, "within": 60_000 },
            says(&format!(r"{QUIET} \(you proposed it on {DAY}\)")),
        ]),
    )
    .await;

    // 10. Bo leaves Elm; the quiet hours still hold on Ana's page, which
    // says who has a say now. The two are named in the order svrngs's
    // `spans` gives them (the doc run's README), so either order.
    steps(
        &mut browsers,
        "bo",
        json!([
            { "goto": bo.home_page() },
            { "wait": "Leave Elm" },
            { "click": "Leave Elm", "until": r"You left Elm\." },
        ]),
    )
    .await;
    steps(
        &mut browsers,
        "ana",
        json!([
            { "poll": ana_rules, "re": r"Everyone in your house has a say: (Cy and you|you and Cy)\.", "within": 60_000, "every": 2_000 },
            says(&format!(r"Your house’s rules\n+{QUIET} \(you proposed it on {DAY}\)")),
            says(&format!(r"Bo had a say from {DAY} until {DAY}, when they left\.")),
        ]),
    )
    .await;
    drop((ana, bo, cy));
}
