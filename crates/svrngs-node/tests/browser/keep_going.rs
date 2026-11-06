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
    drive(engine, "reopen.mjs", args, Some(wait)).await
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
    let seen = reopened(engine, args, async move {
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
    assert_eq!(
        seen["first"]["group"]["meets"],
        json!([at_home]),
        "{seen:#}"
    );
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
    /// failing: the shell left to the network, as lend.js was.
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
    let seen = reopened(engine, args, async move {
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
    let seen = reopened(engine, args, async {}).await;
    eprintln!("{seen:#}");
    assert_eq!(pieces(&seen["saved"]).len(), 9, "{seen:#}");
    let name = format!("svrngs-web-{}", seen["reopened"]["built"].as_str().unwrap());
    assert!(
        seen["reopened"]["saved"].get(&name).is_some(),
        "the new build's cache: {seen:#}"
    );
    assert_eq!(pieces(&seen["reopened"]["saved"]).len(), 9, "{seen:#}");
}
