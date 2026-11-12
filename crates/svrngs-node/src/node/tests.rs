// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use axum::body::Body;
use axum::http::Request;
use serde_json::json;
use svrngs_core::tests_support::let_in;
use tower::ServiceExt;

/// Keep the real startup wiring, but stop background work so a stored wake
/// permit can be inspected without a round consuming it.
async fn write_fixture() -> (Asking, axum::Router, crate::seed::Seed, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[91; 32]);
    let seed = crate::group::draw(
        crate::Name::parse("House").unwrap(),
        crate::Name::parse("Ada").unwrap(),
        key.verifying_key(),
    );
    let node = Node::start(
        dir.path().into(),
        key,
        &RelayConfig {
            relay_urls: vec![],
            n0_services: false,
            loopback_only: true,
        },
        crate::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    drop(node._round);
    drop(node._tasks);
    node.host.keep_group(&seed, None, &[]).unwrap();
    let (ns, rail) = (seed.id(), &node.host.rail);
    let_in(
        (&ns, seed.policy(), &node.host.actor),
        (
            &SigningKey::from_bytes(&[93; 32]),
            "Cy",
            crate::Device::Computer,
        ),
        |mint| {
            rail.commit(&ns, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(rail.commit(&ns, redeem).unwrap().unwrap()),
    );
    let at = node.asking;
    while takes_wake(&at) {}
    (at, node.routes, seed, dir)
}

fn takes_wake(at: &Asking) -> bool {
    use std::future::Future;
    use std::task::{Context, Waker};
    std::pin::pin!(at.door.folders.nudge.notified())
        .poll(&mut Context::from_waker(Waker::noop()))
        .is_ready()
}

#[tokio::test]
async fn minting_a_link_wakes_sync() {
    let (at, _, seed, _dir) = write_fixture().await;
    at.host
        .mint(&seed.id(), vec![], None, crate::door::unix_now().unwrap())
        .unwrap();
    assert!(
        takes_wake(&at),
        "the Invite was written without waking sync"
    );
}

#[tokio::test]
async fn changing_the_meeting_wakes_sync() {
    let (at, _, seed, _dir) = write_fixture().await;
    at.host
        .meet(&seed.id(), &["http://127.0.0.1:9/".parse().unwrap()])
        .unwrap();
    assert!(takes_wake(&at), "the Meets was written without waking sync");
}

#[tokio::test]
async fn journal_changes_wake_folders_and_sync_but_no_ops_and_refusals_do_not() {
    let (at, _, seed, _dir) = write_fixture().await;
    let rail = &at.host.rail;
    let mut folders = at.door.folders.woken.subscribe();
    let before = folders.borrow_and_update().clone();
    let made = rail
        .commit(
            &seed.id(),
            &svrngs_core::tests_support::Batch(vec![svrngs_core::tests_support::record("local")]),
        )
        .unwrap()
        .unwrap();
    assert!(folders.has_changed().unwrap());
    assert_eq!(
        folders.borrow_and_update()[&seed.id()],
        before[&seed.id()] + 1
    );
    assert!(takes_wake(&at));
    assert!(!takes_wake(&at), "one change leaves one permit");
    assert_eq!(rail.ingest_all(&seed.id(), made.lines()).unwrap(), 0);
    assert_eq!(rail.ingest_all(&seed.id(), &[]).unwrap(), 0);
    assert!(rail
        .commit(&seed.id(), &svrngs_core::tests_support::Batch(vec![]))
        .unwrap()
        .unwrap()
        .is_empty());
    assert!(rail
        .commit(
            &seed.id(),
            &svrngs_core::HandOnTo {
                person: Person::parse("Nobody").unwrap(),
                joined: None,
            }
        )
        .unwrap()
        .is_err());
    assert!(!takes_wake(&at));
    assert!(!folders.has_changed().unwrap());
    // A peer's new line uses the same notification path, including folders.
    let peer = crate::sign_act(
        &SigningKey::from_bytes(&[93; 32]),
        &seed.id(),
        crate::door::unix_now().unwrap() + 1,
        0,
        None,
        svrngs_core::tests_support::record("peer"),
        None,
        None,
        None,
    );
    assert_eq!(rail.ingest_all(&seed.id(), &[peer]).unwrap(), 1);
    assert!(folders.has_changed().unwrap());
    assert!(takes_wake(&at));
}

async fn visiting(at: &Asking, group: &str) -> String {
    let page = crate::link::Origin::parse("https://house.example").unwrap();
    let link = at.host.link(group, false, Some(&page), Some(3600)).unwrap();
    let guest = SigningKey::from_bytes(&[92; 32]);
    let ask = crate::joining::AskRequest::sign(&link.token.key(), &guest);
    let (base, _stream) = crate::iroh::tests::serve_on_stream(
        crate::redeem::router(at.clone()),
        SecretKey::from_bytes(&guest.to_bytes()),
    )
    .await;
    let response = reqwest::Client::new()
        .post(format!("{base}/visit"))
        .json(&json!({ "ask": ask, "guest": "Bo" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    at.host
        .rail
        .register(group)
        .unwrap()
        .visits
        .keys()
        .next()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn redeeming_a_visit_wakes_sync() {
    let (at, _, seed, _dir) = write_fixture().await;
    visiting(&at, &seed.id()).await;
    assert!(takes_wake(&at), "the Visit was written without waking sync");
}

#[tokio::test]
async fn stopping_a_visit_wakes_sync() {
    let (at, routes, seed, dir) = write_fixture().await;
    let visit = visiting(&at, &seed.id()).await;
    while takes_wake(&at) {}
    let token = std::fs::read_to_string(dir.path().join("token")).unwrap();
    let req = Request::post("/visits")
        .header("host", format!("svrngs.localhost:{}", at.host.api_port))
        .header(
            "origin",
            format!("http://svrngs.localhost:{}", at.host.api_port),
        )
        .header("sec-fetch-site", "same-origin")
        .header("authorization", format!("Bearer {}", token.trim()))
        .header("content-type", "application/json")
        .extension(axum::extract::ConnectInfo(
            "127.0.0.1:1234".parse::<SocketAddr>().unwrap(),
        ))
        .body(Body::from(json!([seed.id(), visit]).to_string()))
        .unwrap();
    let response = routes.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        takes_wake(&at),
        "the visit's Correct was written without waking sync"
    );
}

async fn tracked_node(
    key: SigningKey,
    dir: &std::path::Path,
) -> (
    Node,
    tokio::sync::watch::Receiver<crate::ring_sync::Progress>,
) {
    let mut node = Node::start(
        dir.into(),
        key,
        &RelayConfig {
            relay_urls: vec![],
            n0_services: false,
            loopback_only: true,
        },
        crate::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let progress = Rounds::new(Default::default());
    let seen = progress.subscribe();
    // Replace only the interval and observe the real round; keep startup's
    // write callback and nudge, with no periodic round during this test.
    node._round = spawn_ring_sync_loop(
        node.host.clone(),
        Duration::from_secs(86400),
        node.folders().nudge.clone(),
        progress,
    );
    (node, seen)
}

async fn settled(rounds: &mut tokio::sync::watch::Receiver<crate::ring_sync::Progress>) -> u64 {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let p = *rounds.borrow_and_update();
            if p.begun > 0 && p.begun == p.ended {
                if tokio::time::timeout(Duration::from_millis(200), rounds.changed())
                    .await
                    .is_err()
                {
                    return p.ended;
                }
            } else {
                rounds.changed().await.unwrap();
            }
        }
    })
    .await
    .expect("journal notifications kept sync running after convergence")
}

#[tokio::test]
async fn a_write_reaches_a_peer_without_the_timer_and_both_rounds_settle() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let key = SigningKey::from_bytes(&[94; 32]);
    let other = SigningKey::from_bytes(&[95; 32]);
    let seed = crate::group::draw(
        crate::Name::parse("House").unwrap(),
        crate::Name::parse("Ada").unwrap(),
        key.verifying_key(),
    );
    let (ada, mut ar) = tracked_node(key, a.path()).await;
    let (bo, mut br) = tracked_node(other.clone(), b.path()).await;
    ada.host.keep_group(&seed, None, &[]).unwrap();
    let ns = seed.id();
    let_in(
        (&ns, seed.policy(), &ada.host.actor),
        (&other, "Bo", crate::Device::Computer),
        |mint| {
            ada.rail().commit(&ns, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(ada.rail().commit(&ns, redeem).unwrap().unwrap()),
    );
    bo.host.keep_group(&seed, None, &[]).unwrap();
    let held = ada
        .rail()
        .ops_missing_from_within(&seed.id(), &Default::default(), usize::MAX)
        .unwrap()
        .0;
    bo.rail().ingest_all(&seed.id(), &held).unwrap();
    for (from, to) in [(&ada, &bo), (&bo, &ada)] {
        from.host.transport.add_known_peer(
            to.endpoint().id(),
            crate::iroh::tests::loopback_sockets(to.endpoint()),
        );
    }
    let (before, _) = tokio::join!(settled(&mut ar), settled(&mut br));
    assert!(ada
        .rail()
        .commit(&seed.id(), &svrngs_core::tests_support::Batch(vec![]))
        .unwrap()
        .unwrap()
        .is_empty());
    assert_eq!(
        settled(&mut ar).await,
        before,
        "a successful no-op started another round"
    );
    let wrote = ada
        .rail()
        .commit(
            &seed.id(),
            &svrngs_core::tests_support::Batch(vec![svrngs_core::tests_support::record(
                "after the round fell asleep",
            )]),
        )
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while !bo
            .rail()
            .admission(&seed.id())
            .unwrap()
            .membership()
            .counts(&wrote.lines()[0].id)
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the write waited for the periodic timer");
    let (after, _) = tokio::join!(settled(&mut ar), settled(&mut br));
    assert!(after > before, "the writer never began another round");
    assert_eq!(
        ada.rail().digest(&seed.id()).unwrap(),
        bo.rail().digest(&seed.id()).unwrap()
    );
    ada.endpoint().close().await;
    bo.endpoint().close().await;
}

/// **A computer running an older svrngs is named so, never read field by
/// field** (arch-version (b); AUDIT §2.5). Its answer of what it lends
/// carries no version, as every build before arch-version's did, so it is
/// `Older`, though its lends would read. One carrying this build's version,
/// or a later one, is read; one at 1 to 3, before the holders' flag day
/// (version 4, HOLDER_APPROVAL), is older too; one carrying it whose lends are
/// not lends is refused, as before. Planted, watched failing: the version check taken
/// out of `answered`, which read the versionless answer as lends.
#[test]
fn a_computer_answering_an_older_version_is_named_older() {
    let lend = || {
        serde_json::json!({
            "name": "shelf", "kind": "k", "limit": null, "using": 0,
            "upload_bps": null, "reads": [],
        })
    };
    let said = |version: Option<u32>, lends: serde_json::Value| {
        let mut body = serde_json::json!({ "lends": [lends] });
        if let Some(v) = version {
            body["version"] = v.into();
        }
        answered("cy", &serde_json::to_vec(&body).unwrap())
    };
    assert!(
        matches!(said(None, lend()), Answered::Older),
        "the published build's"
    );
    let now = crate::version::VERSION;
    assert!(matches!(said(Some(now), lend()), Answered::Lends(l, _) if l.len() == 1));
    assert!(
        matches!(said(Some(now + 1), lend()), Answered::Lends(l, _) if l.len() == 1),
        "a later one"
    );
    assert!(
        matches!(said(Some(5), lend()), Answered::Lends(l, _) if l.len() == 1),
        "6 reads 5: a raise that is `Behind::Reads` cuts nobody off (version.rs STEPS)"
    );
    for before in 1..5 {
        assert!(
            matches!(said(Some(before), lend()), Answered::Older),
            "{before}: one before the holders' flag day (version 4) takes an app's entry unapproved"
        );
    }
    assert!(!crate::version::links(Some(1)) && crate::version::links(Some(now)));
    assert!(
        matches!(
            said(Some(now), serde_json::json!({ "name": "shelf" })),
            Answered::Refused
        ),
        "lends that are not lends"
    );
    assert!(matches!(answered("cy", b"not json"), Answered::Refused));
}

/// Mixed versions wait (svrn-docs EUDOXIA_JUDGE §2e §1, §6): a keeper's
/// Move goes on past a computer only when its answer reads line version 15,
/// so one answering 6, or not at all, is named (`Outcome::Behind`) and
/// nothing is signed. Planted: `removal::moves` made true for every answer.
#[test]
fn a_keepers_move_waits_on_a_computer_answering_version_6() {
    let at = |version: u32| {
        let body = serde_json::json!({ "version": version, "lends": [] });
        answered("bo", &serde_json::to_vec(&body).unwrap())
    };
    assert!(
        matches!(at(6), Answered::Lends(..)),
        "6 is read for its lends"
    );
    assert!(
        !crate::removal::moves(&at(6)),
        "but its keeper's Move waits on it"
    );
    assert!(!crate::removal::moves(&Answered::Unanswered));
    assert!(crate::removal::moves(&at(crate::version::VERSION)));
}

/// The relayed direction of the scope fence (LENDING_SCOPE §"Updates,
/// migration and mixed versions"): a computer still on 4 answers its lends
/// unscoped, every one to every group; this computer names it older and
/// lists none of them on, so `/people/lends` never relays an old broad
/// grant. A 5 answer whose lend names no `groups` is read but listed to
/// nobody. Planted: VERSION left at 4, which read the 4 answer as lends.
#[test]
fn an_unscoped_answer_from_a_computer_on_4_is_relayed_to_nobody() {
    let shelf = serde_json::json!({
        "name": "shelf", "kind": "k", "limit": null, "using": 0, "upload_bps": null, "reads": [],
    });
    let said = |v: u32| {
        answered(
            "cy",
            &serde_json::to_vec(&serde_json::json!({ "version": v, "lends": [shelf] })).unwrap(),
        )
    };
    assert!(
        matches!(said(4), Answered::Older),
        "a 4 answer is older, not read"
    );
    let Answered::Lends(mut lends, 5) = said(5) else {
        panic!("a 5 answer is read")
    };
    assert!(
        !lends[0].listed(|_| true),
        "a lend with no groups is listed to nobody"
    );
}
