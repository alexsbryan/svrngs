//! page-join's tests (main.rs): on the home page, what is
//! refused is said once, and a group joined or made is drawn with no
//! reload.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

use crate::drive::drive;
use crate::harness::{relay, relay_url, Engine};

/// Ada's computer holding house from its seed (browser/main.rs `house`),
/// and her home page's address.
async fn house(dir: &Path, relay: &iroh_relay::server::Server) -> (Node, String) {
    let (node, home, _) = computer(dir, relay, 1, "house", "ada").await;
    (node, home)
}

/// The computer of key `me`, holding `group`, made by `person`: it, its
/// home page's address, and the group's id.
async fn computer(
    dir: &Path,
    relay: &iroh_relay::server::Server,
    me: u8,
    group: &str,
    person: &str,
) -> (Node, String, String) {
    use svrngs_node::group::{draw, Name};
    let ada = SigningKey::from_bytes(&[me; 32]);
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
    .expect("a node starts");
    crate::drive::opens_at(node.api_addr().port(), &node.token());
    let seed = draw(
        Name::parse(group).unwrap(),
        Name::parse(person).unwrap(),
        ada.verifying_key(),
    );
    node.host().keep_group(&seed, None, &[]).unwrap();
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let home = format!("http://svrngs.localhost:{}/", node.api_addr().port());
    (node, home, seed.id())
}

/// `press` pressed twice on ada's home page, each read from `read` once
/// the page traced a line starting with `line` (web/tests/pages/index.mjs,
/// page-index's driver), after `fill` typed: how many times each read
/// says `words`.
async fn twice(
    engine: Engine,
    fill: Value,
    press: &str,
    read: &str,
    line: &str,
    words: &str,
) -> (Vec<usize>, Value) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = house(dir.path(), &relay).await;
    let press = json!([press, null, line]);
    let steps = [fill, press.clone(), press]
        .into_iter()
        .filter(|s| !s.is_null())
        .collect::<Vec<_>>();
    let args = json!({ "home": home, "read": read, "steps": steps });
    let seen = drive(engine, "pages/index.mjs", args, None).await;
    let said = (seen["after"].as_array().into_iter().flatten())
        .map(|a| a["words"].to_string().matches(words).count())
        .collect();
    drop(ada);
    (said, seen)
}

in_chromium!(
    /// **A group not made is said once** (page-join (d)'s first fix). Ada
    /// presses "Make a group" twice with no name: the node refuses each,
    /// and the section says why once, beside the button, the second in the
    /// first's place. Watched failing on v1 (`UI` flipped), which appends
    /// it at each press (home.js@19b63b2b:421).
    a_group_not_made_is_said_once
);
async fn a_group_not_made_is_said_once(engine: Engine) {
    let (said, seen) = twice(
        engine,
        Value::Null,
        "#make button:has-text('Make a group')",
        "#make p",
        "MAKE: refused",
        "a name cannot be empty",
    )
    .await;
    assert_eq!(said, [1, 1], "said once, at each press: {seen:#}");
}

in_chromium!(
    /// **A link to nothing lent is said once** (page-join (f)'s fix). Ada
    /// pastes a link that opens nothing lent in her groups and presses
    /// "Open it" twice: "That is not a link to something lent in your
    /// groups." is said once. Watched failing on v1 (`UI` flipped), which
    /// appends it at each press (home.js@19b63b2b:397).
    a_link_to_nothing_lent_is_said_once
);
async fn a_link_to_nothing_lent_is_said_once(engine: Engine) {
    let fill = json!(["#open input", "http://page.example/#open=no-such"]);
    let words = "That is not a link to something lent in your groups.";
    let (said, seen) = twice(
        engine,
        fill,
        "#open button:has-text('Open it')",
        "#open p",
        "OPEN: open",
        words,
    )
    .await;
    assert_eq!(said, [1, 1], "said once, at each press: {seen:#}");
}

/// Ada's home page, with Sam's computer holding Trip, the `steps` taken,
/// given Sam's link to Trip, each read once `until`'s words hold it, with
/// what `held` holds (web/tests/pages/index.mjs): the reads, and whether
/// the page loaded once.
async fn drawn(
    engine: Engine,
    read: &str,
    steps: impl FnOnce(&str) -> Value,
    held: Value,
) -> Value {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = house(dir.path(), &relay).await;
    let sam_dir = tempfile::tempdir().unwrap();
    let (sam, _, trip) = computer(sam_dir.path(), &relay, 2, "Trip", "Sam").await;
    let made = (sam.host().link(&trip, false, None, None)).expect("Sam makes a link to Trip");
    let link = svrngs_node::link::client_link(&made.token);
    let args = json!({ "home": home, "read": read, "steps": steps(&link), "held": held });
    let seen = drive(engine, "pages/index.mjs", args, None).await;
    drop((ada, sam));
    seen
}

in_chromium!(
    /// **A group joined is drawn with no reload** (page-join (c)'s fix).
    /// Ada pastes Sam's link to Trip and joins it as Dee: the page says
    /// "You’re in Trip, as Dee." and draws Trip, its shelf folded as just
    /// joined, having loaded once. Watched failing on v1 (`UI` flipped),
    /// which opens the page again (home.js@19b63b2b:409).
    a_group_joined_is_drawn_with_no_reload
);
async fn a_group_joined_is_drawn_with_no_reload(engine: Engine) {
    let steps = |link: &str| {
        json!([
            ["#join input[name=link]", link],
            ["#join input[name=person]", "Dee"],
            [
                "#join button:has-text('Join Trip as Dee')",
                null,
                "JOIN: joined",
                "Bring more to Trip"
            ]
        ])
    };
    let seen = drawn(engine, "#home > p, #home summary", steps, Value::Null).await;
    let words = seen["after"][0]["words"].to_string();
    let wrong: Vec<&str> = [
        (!words.contains("You’re in Trip, as Dee."), "said joined"),
        (!words.contains("Bring more to Trip"), "Trip drawn, folded"),
        (seen["loads"] != 1, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
}

in_chromium!(
    /// **A group joined by the running svrngs, not the page, is on the
    /// page when it is shown again** (join-while-running): ada's home page
    /// is open on house; through her running svrngs — asked as `svrngs
    /// join` now asks it, at the record's `/v1/joins` — she joins Sam's
    /// Trip as Dee; the page, hidden and shown again as a person coming
    /// back to its tab, reads its groups and draws Trip's section under
    /// house's, each of its own, with no reload. Planted, watched failing
    /// with the join made after the page was shown again: the page that
    /// read its groups once drew house alone.
    a_group_joined_by_the_running_svrngs_is_on_the_page_when_shown_again
);
async fn a_group_joined_by_the_running_svrngs_is_on_the_page_when_shown_again(
    engine: Engine,
) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, home) = house(dir.path(), &relay).await;
    let sam_dir = tempfile::tempdir().unwrap();
    let (sam, _, trip) = computer(sam_dir.path(), &relay, 2, "Trip", "Sam").await;
    let made = sam.host().link(&trip, false, None, None).expect("Sam makes a link to Trip");
    let link = svrngs_node::link::client_link(&made.token);
    let ready = dir.path().join("ready");
    let args = json!({
        "home": home,
        "read": "#home section h2",
        "steps": [],
        "ready": ready,
        "visible": ["HOME: visible:", "Trip"],
    });
    let (send, release) = tokio::sync::oneshot::channel::<()>();
    let driven = tokio::spawn(drive(engine, "pages/index.mjs", args, Some(release)));
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(90), async {
            while !ready.exists() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
        .await
        .is_ok(),
        "ada's page shows house within 90 s"
    );
    // Through the running svrngs, as `svrngs join` asks it: the join.
    let token = std::fs::read_to_string(dir.path().join(svrngs_node::api::TOKEN_FILE)).unwrap();
    let asked = serde_json::json!({ "link": link, "person": "Dee" });
    let joined: serde_json::Value = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("http://{}/v1/joins", ada.api_addr()))
        .header("content-type", "application/json")
        .bearer_auth(token.trim())
        .body(asked.to_string())
        .send()
        .await
        .expect("the running svrngs answers a join")
        .json()
        .await
        .unwrap_or(serde_json::Value::Null);
    assert_eq!(joined["outcome"], "in", "the running svrngs let ada in: {joined}");
    send.send(()).expect("the page is shown again");
    let seen = driven.await.expect("the page was driven");
    let headings: Vec<String> = seen["later"]["words"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|w| w.as_str().unwrap_or_default().to_string())
        .collect();
    let wrong: Vec<&str> = [
        (
            !headings.iter().any(|h| h.contains("house")),
            "house still drawn",
        ),
        (
            !headings.iter().any(|h| h.contains("Trip")),
            "Trip drawn after the show",
        ),
        (seen["later"]["how"] != "settled", "the read settled"),
        (seen["loads"] != 1, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
    drop((ada, sam));
}

in_chromium!(
    /// **A group made is drawn with no reload** (page-join (d)'s second
    /// fix). Ada makes Lab: the page says "You made Lab." and draws Lab,
    /// with its links, having loaded once. Watched failing on v1 (`UI`
    /// flipped), which opens the page again (home.js@19b63b2b:364).
    a_group_made_is_drawn_with_no_reload
);
async fn a_group_made_is_drawn_with_no_reload(engine: Engine) {
    let steps = |_: &str| {
        json!([
            ["#make input[name=name]", "Lab"],
            ["#make input[name=person]", "Ada"],
            [
                "#make button:has-text('Make a group')",
                null,
                "MAKE: made",
                "Invite someone to Lab"
            ]
        ])
    };
    let seen = drawn(
        engine,
        "#home > p, #home section button",
        steps,
        Value::Null,
    )
    .await;
    let words = seen["after"][0]["words"].to_string();
    let wrong: Vec<&str> = [
        (!words.contains("You made Lab."), "said made"),
        (!words.contains("Invite someone to Lab"), "Lab drawn"),
        (seen["loads"] != 1, "with no reload"),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
}

in_chromium!(
    /// **A group made or joined says it is being read until it is** (the
    /// operator, 2026-11-03: "Of course things should show as loading
    /// until they're loaded"; demo/runs/webkit-wall/2026-11-03/README.md,
    /// "The gate after (c)"). Once Ada's home page is drawn, every read of
    /// a group's people and of what they lend is held. She makes Lab, then
    /// joins Sam's Trip as Dee: each is drawn with "Reading Lab…", "Reading
    /// Trip…" under its heading, never its heading alone, while house,
    /// drawn before, keeps what it drew as it is read again. Let go, each
    /// is drawn read, its folder placed, and none says it is being read.
    /// Planted, watched failing: the page as 84b5a7b0 left it, Lab's and
    /// Trip's headings drawn with no line but their index's "Asking each
    /// computer in the group what it lends…".
    a_group_made_or_joined_says_it_is_being_read_until_it_is
);
async fn a_group_made_or_joined_says_it_is_being_read_until_it_is(engine: Engine) {
    let steps = |link: &str| {
        json!([
            ["#make input[name=name]", "Lab"],
            ["#make input[name=person]", "Ada"],
            [
                "#make button:has-text('Make a group')",
                null,
                "MAKE: made",
                "Reading Lab…"
            ],
            ["#join input[name=link]", link],
            ["#join input[name=person]", "Dee"],
            [
                "#join button:has-text('Join Trip as Dee')",
                null,
                "JOIN: joined",
                "Reading Trip…"
            ]
        ])
    };
    let held = json!([["/v1/rail/people", "/v1/lends"], "…"]);
    let seen = drawn(engine, "#home section[data-colour]", steps, held).await;
    eprintln!("the home page's run: {seen:#}");
    // The words of the section `head` starts, in the read `at`.
    let section = |at: &Value, head: &str| {
        (at["words"].as_array().into_iter().flatten())
            .filter_map(Value::as_str)
            .find(|words| words.starts_with(head))
            .map(String::from)
            .unwrap_or_default()
    };
    let (made, joined, released) = (&seen["after"][0], &seen["after"][1], &seen["released"]);
    let house = section(made, "house (yours");
    let wrong: Vec<&str> = [
        (
            !house.contains("Invite someone to house") || house.contains("Reading house…"),
            "house keeps what it drew while it is read again",
        ),
        (
            !section(made, "Lab (yours").contains("Reading Lab…"),
            "Lab says it is being read",
        ),
        (
            !section(joined, "Trip (Sam’s").contains("Reading Trip…"),
            "Trip says it is being read",
        ),
        (
            released["how"] != "settled",
            "let go, none says it is being read",
        ),
        (
            !section(released, "Lab (yours").contains("Lab’s folder is on this computer"),
            "let go, Lab drawn read",
        ),
        (
            !section(released, "Trip (Sam’s").contains("Trip’s folder is on this computer"),
            "let go, Trip drawn read",
        ),
    ]
    .into_iter()
    .filter_map(|(failed, what)| failed.then_some(what))
    .collect();
    assert!(wrong.is_empty(), "not so: {wrong:?}\n{seen:#}");
}

in_chromium!(
    /// **A link's copy is answered once** (page-join (e)'s fix). Ada makes
    /// a link to invite someone to house and presses "Copy the link"
    /// twice: the answer is said once beside it, here " Select it above to
    /// copy it.", as the test's browser refuses the clipboard, and
    /// " Copied." where it takes it (ui/copy.js, one answer of two). Watched
    /// failing on v1 (`UI` flipped), which appends it at each press
    /// (home.js@19b63b2b:137).
    a_link_s_copy_is_answered_once
);
async fn a_link_s_copy_is_answered_once(engine: Engine) {
    let invite = json!([
        "#home button:has-text('Invite someone to house')",
        null,
        "LINK: made"
    ]);
    let (said, seen) = twice(
        engine,
        invite,
        "#home p.link button:has-text('Copy the link')",
        "#home p.link",
        "COPY: unwritten",
        " Select it above to copy it.",
    )
    .await;
    assert_eq!(said, [0, 1, 1], "said once, at each press: {seen:#}");
}
