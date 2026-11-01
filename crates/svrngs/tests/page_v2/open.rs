//! page-open's v2-only tests (main.rs): the phone page opens into its
//! group, and the home page reads its groups.

use serde_json::json;
use svrngs_core::link::Token;
use svrngs_node::SigningKey;

use crate::drive::drive;
use crate::harness::{relay, relay_url, site_of, Engine};

in_chromium!(
    /// The group's link on a computer, on v2 (page-open (a)): the line
    /// that gets svrngs, composed from the link's `#` part as v1's is
    /// (web/app.js `offer`), and Copy (ui/copy.js), whose press writes the
    /// line and says "Copied" in the button's place, its machine traced as
    /// `COPY:`; under the page's one policy, with no violation and no page
    /// error. Watched failing on v1 (`UI` flipped), which traces no COPY.
    the_computer_s_line_is_offered_and_copied_on_v2
);
async fn the_computer_s_line_is_offered_and_copied_on_v2(engine: Engine) {
    let relay = relay().await;
    let token = Token {
        issuer: SigningKey::from_bytes(&[1; 32]).verifying_key(),
        secret: [0; 16],
        relay: Some(relay_url(&relay).parse().expect("a URL")),
    }
    .encode();
    let args = json!({ "site": site_of(crate::UI), "token": token });
    let seen = drive(engine, "v2/open.mjs", args, None).await;
    let origin = seen["origin"].as_str().unwrap();
    let line = format!("curl -fsSL {origin}install.sh | sh -s -- '{origin}#{token}'");
    assert_eq!(seen["line"], line, "{seen:#}");
    assert!(
        seen["words"]
            .as_str()
            .unwrap()
            .starts_with("On this computer, paste this line"),
        "{seen:#}"
    );
    assert_eq!(seen["button"], "Copied", "{seen:#}");
    assert_eq!(seen["written"], line, "{seen:#}");
    assert_eq!(seen["copied"], line, "{seen:#}");
    assert_eq!(
        seen["violations"],
        json!([]),
        "the page's policy held: {seen:#}"
    );
    assert_eq!(seen["errors"], json!([]), "{seen:#}");
    let lines: Vec<&str> = seen["lines"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|l| l.as_str())
        .collect();
    let copy: Vec<&&str> = lines.iter().filter(|l| l.starts_with("COPY: ")).collect();
    assert_eq!(
        copy,
        [
            &"COPY: copy: idle -> copying, running write",
            &"COPY: copied: copying -> copied"
        ],
        "{seen:#}"
    );
}

in_webkit!(
    /// **Back on screen, the phone reads its group again** (page-open
    /// (c); PAGE_STATE §3, 6). Bo's phone opens house through ada's
    /// computer, which lends nothing: "Nobody in house lends anything
    /// yet." The page is hidden, ada lends a shelf, and the page is shown
    /// again: it reads the group a second time, through the endpoint it
    /// bound, and the shelf is in that read (`list`'s lines), so the
    /// response no longer says nobody lends. Watched failing on v1 (`UI`
    /// flipped), which reads the group once (web/app.js:436).
    the_phone_reads_its_group_again_when_shown
);
async fn the_phone_reads_its_group_again_when_shown(engine: Engine) {
    use std::time::Duration;
    use svrngs_node::{actor_of, iroh::RelayConfig, node::Node, Person, RailAct, Roster};
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, bo) = (
        SigningKey::from_bytes(&[1; 32]),
        SigningKey::from_bytes(&[7; 32]),
    );
    let config = RelayConfig {
        relay_urls: vec![relay_url(&relay)],
        n0_services: false,
        loopback_only: true,
    };
    let no_seed = |_: &std::path::Path| Ok(None);
    let node = Node::start(
        dir.path().to_path_buf(),
        ada.clone(),
        &config,
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    let seed = Roster::new(
        [(Person::parse("ada").unwrap(), vec![actor_of(&ada)])]
            .into_iter()
            .collect(),
    );
    node.rail().set_seed("house", &seed).unwrap();
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("bo").unwrap(),
            key: actor_of(&bo),
            invite: None,
        },
    };
    node.rail().commit("house", &admit).unwrap().unwrap();
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let token = Token {
        issuer: ada.verifying_key(),
        secret: [0; 16],
        relay: Some(relay_url(&relay).parse().expect("a URL")),
    }
    .encode();
    let ready = dir.path().join("ready");
    let (go, release) = tokio::sync::oneshot::channel();
    let args = json!({ "site": site_of(crate::UI), "key": hex::encode(bo.to_bytes()), "token": token, "ready": ready });
    let phone = tokio::spawn(drive(engine, "v2/open-shown.mjs", args, Some(release)));
    tokio::time::timeout(Duration::from_secs(120), async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("the phone opens house, and is hidden");
    // Something lent while the page is hidden: a lend's file, as `svrngs
    // share` writes it, its port one that listens.
    let shelf = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let lends = svrngs_node::lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "port": shelf.local_addr().unwrap().port(), "kind": "k", "refused": [] });
    std::fs::write(lends.join("shelf.json"), body.to_string()).unwrap();
    go.send(()).unwrap();
    let seen = phone.await.unwrap();
    assert_eq!(
        seen["before"]["status"], "You’re in house, as bo",
        "{seen:#}"
    );
    assert_eq!(
        seen["before"]["response"], "Nobody in house lends anything yet.",
        "{seen:#}"
    );
    assert_eq!(
        seen["after"]["status"], "You’re in house, as bo",
        "{seen:#}"
    );
    assert_eq!(
        seen["after"]["response"], "",
        "the shelf is lent now: {seen:#}"
    );
    let lines: Vec<&str> = seen["lines"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|l| l.as_str())
        .collect();
    let reads: Vec<&&str> = lines
        .iter()
        .filter(|l| l.starts_with("PAGE: read ("))
        .collect();
    assert_eq!(reads.len(), 2, "read once, then again when shown: {seen:#}");
    assert!(
        reads[0].contains(", 0 lent)") && reads[1].contains(", 1 lent)"),
        "{reads:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("PAGE: shown: opening/holding/read/")
                && l.ends_with("running read")),
        "{seen:#}"
    );
    drop((node, shelf));
}

in_chromium!(
    /// **What was done on the home page is said once** (page-open (e);
    /// CHARTER, settled 9). Ada's computer holds house; her home page is
    /// opened at `/?made=Trip`, as making a group goes on to it, and says
    /// "You made Trip." It asks `/v1/groups` once, and house's people
    /// before what they lend. Loaded again, it asks the same, and says it
    /// no more: the address forgot it. Watched failing on v1 (`UI`
    /// flipped), which says it again on every load (home.js:363).
    the_home_page_says_what_was_done_once
);
async fn the_home_page_says_what_was_done_once(engine: Engine) {
    use std::time::Duration;
    use svrngs_node::{actor_of, iroh::RelayConfig, node::Node, Person, Roster};
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = SigningKey::from_bytes(&[1; 32]);
    let config = RelayConfig {
        relay_urls: vec![relay_url(&relay)],
        n0_services: false,
        loopback_only: true,
    };
    let no_seed = |_: &std::path::Path| Ok(None);
    let node = Node::start(
        dir.path().to_path_buf(),
        ada.clone(),
        &config,
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
    .ui(crate::UI);
    let seed = Roster::new(
        [(Person::parse("ada").unwrap(), vec![actor_of(&ada)])]
            .into_iter()
            .collect(),
    );
    node.rail().set_seed("house", &seed).unwrap();
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let port = node.api_addr().port();
    crate::drive::opens_at(port, &node.token());
    let args = json!({ "ask": format!("http://svrngs.localhost:{port}/?made=Trip") });
    let seen = drive(engine, "v2/open-home.mjs", args, None).await;
    let reads = |load: &str| -> Vec<String> {
        let sent = seen[load]["sent"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|p| p.as_str());
        sent.filter(|p| p.starts_with("/v1/"))
            .map(String::from)
            .collect()
    };
    for load in ["first", "again"] {
        let reads = reads(load);
        let at = |route: &str| reads.iter().position(|p| p == route);
        let groups = reads.iter().filter(|p| *p == "/v1/groups").count();
        assert_eq!(groups, 1, "{load}: one read of the groups: {seen:#}");
        let (people, lends) = (at("/v1/rail/people"), at("/v1/lends"));
        assert!(
            people.is_some() && people < lends,
            "{load}: people before lends: {seen:#}"
        );
    }
    let said = |load: &str| seen[load]["said"].to_string();
    assert!(
        said("first").contains("You made Trip. Invite someone to it below."),
        "{seen:#}"
    );
    assert!(
        !said("again").contains("You made Trip"),
        "said once: {seen:#}"
    );
    drop(node);
}
