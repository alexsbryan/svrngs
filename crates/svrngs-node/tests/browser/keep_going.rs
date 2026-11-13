//! A phone keeps going when the internet goes out at home (svrn-docs
//! rings/current/KEEP_GOING_AT_HOME.md): what it saved while everything
//! answered is what it reopens on (web/tests/reopen.mjs).

use super::*;

/// The relay `node` is homed at now, if any.
fn home(node: &Node) -> Option<String> {
    node.endpoint()
        .addr()
        .relay_urls()
        .next()
        .map(ToString::to_string)
}

/// The browser half run on `args`, with what `cut` does done between its
/// browser's close and its reopening.
async fn reopened<F: std::future::Future<Output = ()> + Send + 'static>(
    engine: Engine,
    mut args: Value,
    script: &str,
    cut: F,
) -> Value {
    let dir = tempfile::tempdir().unwrap();
    let ready = dir.path().join("ready");
    args["ready"] = json!(ready);
    let (go, wait) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        cut.await;
        let _ = go.send(());
    });
    drive(engine, script, args, Some(wait)).await
}

in_webkit!(
    /// **A phone learns the group's relay with no new invitation**
    /// (KEEP_GOING_AT_HOME 4). Bo's link names only the public relay; the
    /// keeper has since said the group meets at the home relay. Opened once
    /// while the public relay answers, the phone keeps the home one on its
    /// group; the public relay goes and ada's computer moves; the browser
    /// reopened reaches the group through the relay no link names. Planted,
    /// watched failing: `/people/lends` naming no `meets`, so nothing is
    /// kept and the reopen ends in no relay answering.
    a_phone_learns_the_group_s_relay_with_no_new_invitation
);
async fn a_phone_learns_the_group_s_relay_with_no_new_invitation(engine: Engine) {
    let (public, house_relay) = (relay().await, relay().await);
    let dir = tempfile::tempdir().unwrap();
    let bo = signing(7);
    let (ada, id) = house(dir.path(), &public, &bo).await;
    let at_home = relay_url(&house_relay);
    let named: svrngs_node::iroh::RelayUrl = at_home.parse().unwrap();
    ada.host().meet(&id, &[named]).expect("ada keeps the group");

    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &public),
        "id": id,
    });
    let ada = std::sync::Arc::new(ada);
    let (moved, moves) = (ada.clone(), at_home.clone());
    let seen = reopened(engine, args, "reopen.mjs", async move {
        public.shutdown().await.unwrap();
        let start = std::time::Instant::now();
        while home(&moved).as_deref() != Some(moves.as_str()) {
            assert!(
                start.elapsed() < Duration::from_secs(90),
                "ada did not move"
            );
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await;
    eprintln!("{seen:#}");
    // What it keeps of the reply's `meets` is enter.test.mjs's ('the relays
    // a reply names are kept, ...'); here, that the relay it learned is one
    // its endpoint reaches the group through.
    assert!(
        seen["again"]["kind"].as_str().unwrap().contains("good"),
        "the reopened phone reached the group through the relay it learned: {seen:#}"
    );
}

/// The file at a lend's origin that says what it saved, by each cache.
fn pieces(saved: &Value) -> Vec<String> {
    let caches = saved.as_object().expect("the page's caches");
    assert_eq!(caches.len(), 1, "one build's cache: {saved:#}");
    let (name, files) = caches.iter().next().unwrap();
    assert!(name.starts_with("svrngs-web-"), "{saved:#}");
    let files = files.as_array().unwrap().iter();
    files.map(|f| f.as_str().unwrap().to_string()).collect()
}

// Chromium's, where the page's other tests are WebKit's: WebKit in a context
// with a profile, as Playwright 1.59 runs it on macOS, keeps nothing in the
// Cache API, `put` and `addAll` resolving and `keys()` answering none (probed
// 2026-11-06; its contexts without a profile keep them), so a browser closed
// and opened again is Chromium's here and a real iPhone's in the real run.
in_chromium!(
    /// **A phone reopens the group and its app with the public site gone**
    /// (KEEP_GOING_AT_HOME 10, 13). Opened once while everything answered,
    /// the browser is fully closed, the page's public site and the public
    /// relay both go, and ada's computer moves to the home relay. The
    /// reopened page reaches the group, the app reopens at its address on
    /// its pass, and from the group's list on a new one, which the phone
    /// signs itself. Every saved piece answers; each taken out in turn stops
    /// answering, which is the check the spec demands. Planted, watched
    /// failing: the shell left to the network, as lend.js was. The readiness
    /// check (29) names the app until it has been opened; planted, watched
    /// failing: the signer marking nothing.
    a_phone_reopens_the_group_and_its_app_with_the_public_site_gone
);
async fn a_phone_reopens_the_group_and_its_app_with_the_public_site_gone(engine: Engine) {
    let (public, house_relay) = (relay().await, relay().await);
    let dir = tempfile::tempdir().unwrap();
    let bo = signing(7);
    let (ada, id) = house(dir.path(), &public, &bo).await;
    let at_home = relay_url(&house_relay);
    let named: svrngs_node::iroh::RelayUrl = at_home.parse().unwrap();
    ada.host().meet(&id, &[named]).expect("ada keeps the group");
    let shelf = page_origin("<!doctype html><title>shelf</title><p>hi</p>").await;
    let lends = svrngs_node::lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "groups": [id], "port": shelf.port(), "kind": "k", "refused": [] });
    std::fs::write(lends.join("shelf.json"), body.to_string()).unwrap();

    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &public),
        "id": id,
        "open": "shelf",
        "siteDown": true,
    });
    let ada = std::sync::Arc::new(ada);
    let (moved, moves) = (ada.clone(), at_home.clone());
    let seen = reopened(engine, args, "reopen.mjs", async move {
        public.shutdown().await.unwrap();
        let start = std::time::Instant::now();
        while home(&moved).as_deref() != Some(moves.as_str()) {
            assert!(
                start.elapsed() < Duration::from_secs(90),
                "ada did not move"
            );
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await;
    eprintln!("{seen:#}");
    let lines = |of: &str| seen["first"]["ready"][of].to_string();
    assert!(
        !lines("kept").contains("not saved"),
        "the app opened once while online is saved: {seen:#}"
    );
    assert!(
        seen["again"]["kind"].as_str().unwrap().contains("good"),
        "the group reopened with the site gone: {seen:#}"
    );
    assert_eq!(seen["reopened"]["fresh"], true, "{seen:#}");
    let whole = seen["reopened"]["whole"].as_object().unwrap();
    assert!(!whole.is_empty(), "{seen:#}");
    for (piece, answered) in whole {
        assert_eq!(
            answered, 200,
            "{piece} answers with the site gone: {seen:#}"
        );
    }
    for (piece, answered) in seen["reopened"]["lost"].as_object().unwrap() {
        assert_ne!(answered, 200, "{piece}, missing, fails the check: {seen:#}");
    }
}

in_chromium!(
    /// **A new release replaces a lend's saved pages** (KEEP_GOING_AT_HOME
    /// 10; the discipline of web/sw.js). Opened at one build, then a new
    /// build is deployed: the page moves to it, and the app opened from it
    /// runs the new worker, whose cache is the new build's alone. Planted,
    /// watched failing: `activate` keeping every cache, and a worker
    /// answering a newer build's request with its own copy.
    a_new_release_replaces_a_lend_s_saved_pages
);
async fn a_new_release_replaces_a_lend_s_saved_pages(engine: Engine) {
    let public = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let bo = signing(7);
    let (_ada, id) = house(dir.path(), &public, &bo).await;
    let shelf = page_origin("<!doctype html><title>shelf</title><p>hi</p>").await;
    let lends = svrngs_node::lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "groups": [id], "port": shelf.port(), "kind": "k", "refused": [] });
    std::fs::write(lends.join("shelf.json"), body.to_string()).unwrap();
    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &public),
        "id": id,
        "open": "shelf",
        "rebuilt": true,
    });
    let seen = reopened(engine, args, "reopen.mjs", async {}).await;
    eprintln!("{seen:#}");
    assert_eq!(pieces(&seen["saved"]).len(), 9, "{seen:#}");
    let name = format!("svrngs-web-{}", seen["reopened"]["built"].as_str().unwrap());
    assert!(
        seen["reopened"]["saved"].get(&name).is_some(),
        "the new build's cache: {seen:#}"
    );
    assert_eq!(pieces(&seen["reopened"]["saved"]).len(), 9, "{seen:#}");
}

/// Whether `node`'s copy of `group` holds `words`.
fn holds(node: &Node, group: &str, words: &str) -> bool {
    let dir = node.rail().group_dir(group).unwrap();
    std::fs::read_to_string(dir.join("ring_oplog.jsonl"))
        .unwrap_or_default()
        .contains(words)
}

/// Rounds on `nodes` until `done`, within 120 s, saying `what` if not.
async fn until(nodes: &[&Node], what: &str, done: impl Fn() -> bool) {
    let start = std::time::Instant::now();
    while !done() {
        assert!(start.elapsed() < Duration::from_secs(120), "never: {what}");
        for node in nodes {
            let _ = tokio::time::timeout(Duration::from_secs(10), node.round()).await;
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

/// `node`'s home `want`, within 90 s.
async fn moved_to(node: &Node, want: &str) {
    let start = std::time::Instant::now();
    while home(node).as_deref() != Some(want) {
        let at = home(node);
        assert!(
            start.elapsed() < Duration::from_secs(90),
            "not at {want}: at {at:?}"
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// `file` in `dir` written, by polling for it: the browser half's step.
async fn told(dir: &Path, file: &str) -> String {
    let path = dir.join(file);
    tokio::time::timeout(Duration::from_secs(180), async {
        while !path.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("the phone never reached {file}"));
    std::fs::read_to_string(path).unwrap()
}

in_chromium!(
    /// **The phone keeps going through the home relay, and back**
    /// (KEEP_GOING_AT_HOME 11, 14; "How we'll know" steps 5 and 7): Bo's
    /// phone opened Money once, carried in house on Ada's computer, and
    /// its browser closed. The public relay and the page's site go; Ada's
    /// computer moves home. The phone, its browser opened again on what it
    /// saved, writes in Money, and the line is on Ada's record, carried by
    /// the home relay, the phone's only way (a page has no direct path).
    /// The public relay comes back at its address; Ada's computer goes
    /// back; the page left open is the same page, never loaded again, and
    /// writes once more. Chromium's, as the reopen test is, for WebKit's
    /// saved profile. The computers' half, a file each way and a member
    /// off through the outage catching up with the same group, people and
    /// keys, is relay_fallback.rs's and folders.rs's. Planted, each
    /// watched failing: `/people/lends` naming no `meets`, so the reopened
    /// phone reaches nobody; the app's page reloaded on the way back.
    the_house_keeps_going_through_the_home_relay_and_comes_back
);
async fn the_house_keeps_going_through_the_home_relay_and_comes_back(engine: Engine) {
    let (public, house_relay) = (relay().await, relay().await);
    let (inner, public_at) = (public.http_addr().unwrap(), relay_url(&public));
    let (home_at, bo) = (relay_url(&house_relay), signing(7));
    let dir = tempfile::tempdir().unwrap();
    let (ada, id) = house(dir.path(), &public, &bo).await;
    let named: svrngs_node::iroh::RelayUrl = home_at.parse().unwrap();
    ada.host().meet(&id, &[named]).expect("ada keeps the group");
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder_in(&ada, &folder, &id).await;
    let script = "window.title = document.title = 'Money';";
    let page = "<!doctype html><title>…</title><script type=module src=money.js></script>";
    super::carried::carry(
        &folder,
        "Money",
        &[("index.html", page.into()), ("money.js", script.into())],
    );
    super::carried::carried_in(&ada, "Money", &id).await;

    // The phone opens Money once and its browser closes.
    let phone = tempfile::tempdir().unwrap();
    let (outage, after) = ("bins in the outage", "bins after the outage");
    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &public),
        "id": id,
        "open": "Money",
        "dir": phone.path(),
        "outage": outage,
        "after": after,
    });
    let driven = tokio::spawn(drive(engine, "keeps-going.mjs", args, None));
    told(phone.path(), "closed").await;

    // The internet goes: the public relay, and the page's site with it.
    public.shutdown().await.unwrap();
    moved_to(&ada, &home_at).await;

    // The phone, reopened on what it saved, writes; Ada's record holds it.
    std::fs::write(phone.path().join("gone"), "").unwrap();
    let during = told(phone.path(), "wrote").await;
    until(&[&ada], "the phone's line on ada's record", || {
        holds(&ada, &id, outage)
    })
    .await;

    // The internet comes back at its address.
    let mut config = iroh_relay::server::ServerConfig::default();
    config.relay = Some(iroh_relay::server::RelayConfig::new(inner));
    let _back = Server::spawn(config).await.expect("the relay is back");
    moved_to(&ada, &public_at).await;
    std::fs::write(phone.path().join("back"), "").unwrap();
    let seen = driven.await.unwrap();
    eprintln!("{during}\n{seen:#}");
    assert!(during.contains("\"status\":200"), "{during}");
    assert_eq!(seen["after"]["status"], 200, "{seen:#}");
    let stayed = &seen["stayed"];
    assert_eq!(stayed["stayed"], "the same page", "{seen:#}");
    assert_eq!(stayed["loads"], 1, "{seen:#}");
    assert_eq!(seen["same"], true, "{seen:#}");
    until(&[&ada], "the page's second line on ada's record", || {
        holds(&ada, &id, after)
    })
    .await;
}

in_chromium!(
    /// **A phone with the page saved joins at home with the internet out**
    /// (KEEP_GOING_AT_HOME 3, 1): Pia opened the group's page once, in no
    /// group, and her browser closed. The public relay and the page's site
    /// go, and ada's computer moves to the home relay. Her invitation names
    /// both ways in: opened on the saved page she gives her name and is in
    /// the group, seen on ada's record. A planted bad input, kept as the
    /// control: the same link naming the public relay alone, on the same
    /// saved page, ends in the page's words and no one let in. Chromium's,
    /// as the reopen test is. A page saved means opened once: nothing here
    /// says a phone that never opened it can join (spec, "not promising").
    a_phone_with_the_page_saved_joins_at_home_with_the_internet_out
);
async fn a_phone_with_the_page_saved_joins_at_home_with_the_internet_out(engine: Engine) {
    let (public, house_relay) = (relay().await, relay().await);
    let dir = tempfile::tempdir().unwrap();
    let (ada, id) = house(dir.path(), &public, &signing(7)).await;
    let at_home = relay_url(&house_relay);
    let named: svrngs_node::iroh::RelayUrl = at_home.parse().unwrap();
    ada.host().meet(&id, &[named]).expect("ada keeps the group");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let (own, out): (svrngs_core::relay::Url, svrngs_core::relay::Url) = (
        at_home.parse().unwrap(),
        relay_url(&public).parse().unwrap(),
    );
    let link = (ada.host().mint(&id, vec![own, out.clone()], None, now)).unwrap();
    let old = Token {
        relays: vec![out],
        ..link.clone()
    };
    let args = json!({ "site": site(), "link": link.encode(), "old": old.encode() });
    let ada = std::sync::Arc::new(ada);
    let (moved, moves) = (ada.clone(), at_home.clone());
    let seen = reopened(engine, args, "join-saved.mjs", async move {
        public.shutdown().await.unwrap();
        moved_to(&moved, &moves).await;
    })
    .await;
    eprintln!("{seen:#}");
    assert!(
        seen["old"]["kind"]
            .as_str()
            .unwrap_or_default()
            .contains("bad"),
        "the old link reaches nobody: {seen:#}"
    );
    assert!(
        seen["joined"]["said"]
            .as_str()
            .unwrap_or_default()
            .contains("in house"),
        "{seen:#}"
    );
    assert!(holds(&ada, &id, "Pia"), "ada's record holds her: {seen:#}");
}
