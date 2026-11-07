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
    let lines = |of: &str| seen["first"]["ready"][of].to_string();
    assert!(
        !lines("kept").contains("not saved"),
        "the app opened once while online is saved: {seen:#}"
    );
    assert!(
        lines("denied").contains("home network") && !lines("granted").contains("home network"),
        "the permission is said until it is given: {seen:#}"
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

/// A lent app: `/` its page, a POST taken, counted and held unanswered, and a
/// read of `/took` how many it took. The count, read by the test too.
async fn takes_posts() -> (u16, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (listener, took) = (
        tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap(),
        std::sync::Arc::new(AtomicUsize::new(0)),
    );
    let (port, count) = (listener.local_addr().unwrap().port(), took.clone());
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let count = count.clone();
            tokio::spawn(async move {
                let Some(head) = origins::read_head(&mut sock).await else {
                    return;
                };
                if head.starts_with("POST") {
                    count.fetch_add(1, Ordering::SeqCst);
                    return std::future::pending().await;
                }
                let body = if head.starts_with("GET /took") {
                    format!("{} taken", count.load(Ordering::SeqCst))
                } else {
                    "<!doctype html><title>takes</title><p>hi</p>".to_string()
                };
                let type_of = if body.starts_with('<') {
                    "text/html"
                } else {
                    "text/plain"
                };
                let answer = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: {type_of}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(answer.as_bytes()).await;
            });
        }
    });
    (port, took)
}

in_webkit!(
    /// **An action whose answer is lost is not sent again, and the person
    /// can check** (KEEP_GOING_AT_HOME 19). Bo's phone posts an entry to an
    /// app on ada's computer, which takes it; the phone's way to the relay
    /// is cut before any answer. The page is told, once. The way comes
    /// back; a read says one was taken, and the app took one POST in all,
    /// the reads and the dial the worker made afresh sending none of it
    /// again. Planted, watched failing in web/tests/lost-answer.test.mjs,
    /// where a retry can land: here a resent post would need its stream,
    /// which is spent.
    an_action_whose_answer_is_lost_is_not_sent_again_and_can_be_checked
);
async fn an_action_whose_answer_is_lost_is_not_sent_again_and_can_be_checked(engine: Engine) {
    use std::sync::atomic::Ordering;
    let public = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let bo = signing(7);
    let (_ada, id) = house(dir.path(), &public, &bo).await;
    let (port, took) = takes_posts().await;
    let lends = svrngs_node::lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "groups": [id], "port": port, "kind": "k", "refused": [] });
    std::fs::write(lends.join("takes.json"), body.to_string()).unwrap();
    let (way, cut) = super::cuttable::cuttable(harness::relay_at(&public)).await;
    let scratch = tempfile::tempdir().unwrap();
    let ready = scratch.path().join("ready");
    let token = Token {
        issuer: signing(1).verifying_key(),
        secret: [0; 16],
        relays: vec![way.parse().expect("a URL")],
    };
    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token.encode(),
        "id": id,
        "open": "takes",
        "ready": ready,
    });
    let (go, release) = tokio::sync::oneshot::channel();
    let phone = tokio::spawn(drive(engine, "lost-answer.mjs", args, Some(release)));
    tokio::time::timeout(Duration::from_secs(120), async {
        while took.load(Ordering::SeqCst) == 0 {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("the app takes the post");
    cut.send_replace(true);
    tokio::time::timeout(Duration::from_secs(120), async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("the page is told");
    cut.send_replace(false);
    go.send(()).unwrap();
    let seen = phone.await.unwrap();
    eprintln!("{seen:#}");
    assert_eq!(seen["posted"]["status"], 502, "{seen:#}");
    assert_eq!(seen["posted"]["refused"], "away", "{seen:#}");
    assert_eq!(seen["checked"]["text"], "1 taken", "{seen:#}");
    assert_eq!(took.load(Ordering::SeqCst), 1, "{seen:#}");
}

/// `n` bytes of their own, by `seed`, as a file a person adds.
fn file_of(seed: u8, n: usize) -> Vec<u8> {
    let step = |i: usize| (i.wrapping_mul(31) ^ (i >> 7)) as u8;
    (0..n).map(|i| step(i).wrapping_add(seed)).collect()
}

/// A computer of `seed`'s group, its build's relay `public`, the group met
/// at `own`, as relay_fallback.rs stands them up: no address told it, so
/// every first dial is a relay's.
async fn member(
    dir: &Path,
    me: u8,
    public: &str,
    seed: &svrngs_node::group::Seed,
    own: &svrngs_core::relay::Url,
) -> Node {
    let node = started(dir, me, public).await;
    let own = std::slice::from_ref(own);
    node.host().keep_group(seed, None, own).expect("kept");
    node
}

/// A computer's svrngs started on what `dir` holds, its build's relay `public`.
async fn started(dir: &Path, me: u8, public: &str) -> Node {
    Node::start(
        dir.to_path_buf(),
        signing(me),
        &RelayConfig {
            relay_urls: vec![public.to_string()],
            n0_services: false,
            loopback_only: true,
        },
        svrngs_node::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a computer starts")
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

/// Who `node` says stands in `group`, each with their keys.
fn people(node: &Node, group: &str) -> String {
    let read = node.rail().admission(group).unwrap();
    format!("{:?}", read.membership().standing)
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
    /// **The scene, through the home relay, and back** (KEEP_GOING_AT_HOME
    /// 11, 14; "How we'll know" steps 3 to 7). Ada and Cy at home, Dee let in
    /// and away, her computer off until the end (a stopped node holds its
    /// store in this process, so hers starts once), Bo on a phone that
    /// opened Money once. The public relay and
    /// the page's site go; the two computers move. A named file on Ada's
    /// disk arrives with the same bytes on Cy's, and another the other way.
    /// The phone, its browser closed and opened again, writes in Money, and
    /// the line is on both computers' records, carried by the home relay,
    /// the phone's only way (a page has no direct path). The public relay
    /// comes back at its address, nobody joining again: both computers go
    /// back, Dee starts, and everything made in the outage reaches her with
    /// the same group, people and keys. The page left open is the same page,
    /// never loaded again, and writes once more. Chromium's, as the reopen
    /// test is, for WebKit's saved profile. What it cannot show: whether two
    /// computers on one host spoke through the relay or found each other
    /// directly (demo/runs/mesh-offline/2026-11-01 finding 4). Planted,
    /// each watched failing: `/people/lends` naming no `meets`, so the
    /// reopened phone reaches nobody; the app's page reloaded on the way
    /// back; a byte of the file differing.
    the_house_keeps_going_through_the_home_relay_and_comes_back
);
async fn the_house_keeps_going_through_the_home_relay_and_comes_back(engine: Engine) {
    use svrngs_node::group;
    let (public, house_relay) = (relay().await, relay().await);
    let (inner, public_at) = (public.http_addr().unwrap(), relay_url(&public));
    let (home_at, bo) = (relay_url(&house_relay), signing(7));
    let own: svrngs_core::relay::Url = home_at.parse().unwrap();
    let dirs = [(); 4].map(|()| tempfile::tempdir().unwrap());
    let (ada, id) = house(dirs[0].path(), &public, &bo).await;
    let named: svrngs_node::iroh::RelayUrl = home_at.parse().unwrap();
    ada.host().meet(&id, &[named]).expect("ada keeps the group");
    for (person, me) in [("cy", 2), ("dee", 3)] {
        let act = RailAct::Admit {
            person: Person::parse(person).unwrap(),
            key: actor_of(&signing(me)),
            invite: None,
        };
        let fixture = svrngs_core::tests_support::Fixture { act };
        ada.rail().commit(&id, &fixture).unwrap().unwrap();
    }
    let seed = (group::groups(ada.rail().root()).into_iter())
        .find(|s| s.id() == id)
        .expect("ada holds house");
    let cy = member(dirs[1].path(), 2, &public_at, &seed, &own).await;
    let both = [&ada, &cy];
    let before = people(&ada, &id);
    until(&both, "both hold who is in", || people(&cy, &id) == before).await;
    let at = |dir: &tempfile::TempDir| dir.path().join("house");
    for (computer, dir) in [(&ada, &dirs[0]), (&cy, &dirs[1])] {
        std::fs::create_dir(at(dir)).unwrap();
        keep_folder_in(computer, &at(dir), &id).await;
    }
    let script = "window.title = document.title = 'Money';";
    let page = "<!doctype html><title>…</title><script type=module src=money.js></script>";
    super::carried::carry(
        &at(&dirs[0]),
        "Money",
        &[("index.html", page.into()), ("money.js", script.into())],
    );
    until(&both, "Money reaches cy", || {
        at(&dirs[1]).join("apps/Money/money.js").exists()
    })
    .await;
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
    moved_to(&cy, &home_at).await;

    // A named file each way, the same bytes.
    let (from_ada, from_cy) = (file_of(1, 300_000), file_of(2, 300_000));
    let (at_ada, at_cy) = (at(&dirs[0]), at(&dirs[1]));
    std::fs::write(at_ada.join("from-ada.bin"), &from_ada).unwrap();
    let same = |path: &Path, bytes: &[u8]| std::fs::read(path).is_ok_and(|b| b == bytes);
    until(&both, "ada's file on cy's disk", || {
        same(&at_cy.join("from-ada.bin"), &from_ada)
    })
    .await;
    std::fs::write(at_cy.join("from-cy.bin"), &from_cy).unwrap();
    until(&both, "cy's file on ada's disk", || {
        same(&at_ada.join("from-cy.bin"), &from_cy)
    })
    .await;

    // The phone, reopened on what it saved, writes; both records hold it.
    std::fs::write(phone.path().join("gone"), "").unwrap();
    let during = told(phone.path(), "wrote").await;
    until(&both, "the phone's line on both computers", || {
        holds(&ada, &id, outage) && holds(&cy, &id, outage)
    })
    .await;

    // The internet comes back at its address, and Dee with it.
    let mut config = iroh_relay::server::ServerConfig::default();
    config.relay = Some(iroh_relay::server::RelayConfig::new(inner));
    let _back = Server::spawn(config).await.expect("the relay is back");
    moved_to(&ada, &public_at).await;
    moved_to(&cy, &public_at).await;
    let dee = member(dirs[2].path(), 3, &public_at, &seed, &own).await;
    let everyone = [&ada, &cy, &dee];
    until(&everyone, "dee hears who is in", || {
        people(&dee, &id) == before
    })
    .await;
    let at_dee = at(&dirs[2]);
    std::fs::create_dir(&at_dee).unwrap();
    keep_folder_in(&dee, &at_dee, &id).await;
    std::fs::write(phone.path().join("back"), "").unwrap();
    let seen = driven.await.unwrap();
    eprintln!("{during}\n{seen:#}");
    assert!(during.contains("\"status\":200"), "{during}");
    assert_eq!(seen["after"]["status"], 200, "{seen:#}");
    let stayed = &seen["stayed"];
    assert_eq!(stayed["stayed"], "the same page", "{seen:#}");
    assert_eq!(stayed["loads"], 1, "{seen:#}");
    assert_eq!(seen["same"], true, "{seen:#}");
    until(&everyone, "all of it on all three", || {
        everyone
            .iter()
            .all(|n| holds(n, &id, outage) && holds(n, &id, after))
            && same(&at_dee.join("from-ada.bin"), &from_ada)
            && same(&at_dee.join("from-cy.bin"), &from_cy)
    })
    .await;
    for computer in everyone {
        assert_eq!(people(computer, &id), before, "the same people and keys");
        let held = group::groups(computer.rail().root());
        assert!(held.iter().any(|s| s.id() == id), "the same group");
    }
}
