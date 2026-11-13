//! page-open's tests (main.rs): the phone page opens into its
//! group, and the home page reads its groups.

use serde_json::json;
use svrngs_core::link::Token;
use svrngs_node::SigningKey;

use crate::drive::drive;
use crate::harness::{relay, relay_url, site, Engine};

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
        relays: vec![relay_url(&relay).parse().expect("a URL")],
    }
    .encode();
    let args = json!({ "site": site(), "token": token });
    let seen = drive(engine, "pages/open.mjs", args, None).await;
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
    /// (c); PAGE_STATE §3, 6). The seam is the claim: a read again through
    /// the wasm endpoint the page bound, over a real relay. Bo's phone opens
    /// house through ada's computer, which lends nothing; the page is
    /// hidden, ada lends a shelf, and the page is shown again: it reads the
    /// group a second time, and the shelf is in that read (`list`'s lines).
    /// What the page says of each read, and that being shown asks the read
    /// again: web/tests/pages/enter.test.mjs ('back on screen the list is
    /// read again') and group-page.test.mjs ("Nobody in house lends
    /// anything yet."). Watched failing on v1 (`UI` flipped), which reads
    /// the group once (web/app.js@19b63b2b:436).
    the_phone_reads_its_group_again_when_shown
);
async fn the_phone_reads_its_group_again_when_shown(engine: Engine) {
    use std::time::Duration;
    use svrngs_node::{actor_of, iroh::RelayConfig, node::Node, Person, Roster};
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
    svrngs_core::tests_support::let_in_by("house", &bo, "bo", |act| {
        let act = svrngs_core::tests_support::Fixture { act };
        node.rail().commit("house", &act).unwrap().unwrap().lines()[0]
            .id
            .clone()
    });
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    let token = Token {
        issuer: ada.verifying_key(),
        secret: [0; 16],
        relays: vec![relay_url(&relay).parse().expect("a URL")],
    }
    .encode();
    let ready = dir.path().join("ready");
    let (go, release) = tokio::sync::oneshot::channel();
    let args = json!({ "site": site(), "key": hex::encode(bo.to_bytes()), "token": token, "ready": ready });
    let phone = tokio::spawn(drive(engine, "pages/open-shown.mjs", args, Some(release)));
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
    let body = json!({ "port": shelf.local_addr().unwrap().port(), "kind": "k", "refused": [], "groups": ["house"] });
    std::fs::write(lends.join("shelf.json"), body.to_string()).unwrap();
    go.send(()).unwrap();
    let seen = phone.await.unwrap();
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
    drop((node, shelf));
}
