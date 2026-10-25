//! The record over loopback (DESIGN §3; PLAN §4, U3's inventory, the u3-2 and
//! u3-3 rows): an app on a member's computer appends to its group, reads the
//! log and the people, and pushes and drains the live lane, on a listener
//! bound to 127.0.0.1, and nothing else reaches it.
//! Nodes in one process, no relay; peers are reached by seeded addresses,
//! as `group_not_mesh.rs` does.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use svrngs_node::iroh::{PublicKey, RelayConfig};
use svrngs_node::node::{Node, NodeError};
use svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};

const NS: &str = "house";

/// How long B is watched for an act after A's app appends it: a sixth of
/// the round's interval, so only a round the append itself woke can carry
/// the act inside it.
const ONE_ROUND: Duration = Duration::from_secs(10);

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// No relay and no address lookup: peers are reached by seeded addresses.
fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
}

/// No link is asked about here.
fn no_seed(_: &std::path::Path) -> Result<Option<String>, String> {
    Ok(None)
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from_bytes(&signing(seed).verifying_key().to_bytes()).unwrap()
}

async fn start(seed: u8, dir: &tempfile::TempDir) -> Node {
    start_on(seed, dir, "127.0.0.1:0".parse().unwrap())
        .await
        .expect("a node starts")
}

/// A group's seed: its founder's key alone (DESIGN §6).
fn founded_by(founder: u8) -> Roster {
    Roster::new(
        [(
            Person::parse("ada").unwrap(),
            vec![actor_of(&signing(founder))],
        )]
        .into_iter()
        .collect(),
    )
}

/// `node` holds `ns`, seeded with `founder`'s key.
fn hold(node: &Node, ns: &str, founder: u8) {
    node.rail().set_seed(ns, &founded_by(founder)).unwrap();
}

/// The founder admits `joining` as `person` through its door, as a test
/// admits a member by hand (PLAN §4, U1).
fn admit(node: &Node, person: &str, joining: u8) {
    admit_to(node, NS, person, joining)
}

/// [`admit`], to the group `ns`.
fn admit_to(node: &Node, ns: &str, person: &str, joining: u8) {
    let fixture = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse(person).unwrap(),
            key: actor_of(&signing(joining)),
            invite: None,
        },
    };
    node.rail().commit(ns, &fixture).unwrap().unwrap();
}

/// An app's HTTP client: loopback, never a proxy.
fn app() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// Whether `node` holds the act `id` in `ns`.
fn holds(node: &Node, ns: &str, id: &str) -> bool {
    svrngs_core::tests_support::wire_ops(&node.rail().admission(ns).unwrap())
        .iter()
        .any(|o| o.id.to_string() == id)
}

async fn start_on(seed: u8, dir: &tempfile::TempDir, api: SocketAddr) -> Result<Node, NodeError> {
    Node::start(
        dir.path().to_path_buf(),
        signing(seed),
        &no_relay(),
        no_seed,
        api,
    )
    .await
}

/// **The API is served on loopback alone** — upstream's bind guard
/// (`api.rs:167`, `check_loopback`): asked to listen on an address that is
/// not loopback, the node refuses to start, naming the address, and binds
/// nothing. Asked for loopback, it serves there.
#[tokio::test]
async fn a_listener_not_on_loopback_is_refused_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let wide: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let refused = match start_on(1, &dir, wide).await {
        Ok(_) => panic!("a node must not serve the record on {wide}"),
        Err(e) => e.to_string(),
    };
    assert!(
        refused.contains("0.0.0.0:0") && refused.contains("not a loopback address"),
        "{refused}"
    );

    let node = start_on(1, &dir, "127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    assert!(node.api_addr().ip().is_loopback(), "{}", node.api_addr());
    assert_ne!(node.api_addr().port(), 0, "the port bound is answered");
    node.endpoint().close().await;
}

/// **An app's append goes at once** (the u3-2 row): A founds the group and
/// admits B; an app on A's computer appends a record to A's group over
/// loopback; the round, woken by the append and nothing else, carries it to
/// B inside [`ONE_ROUND`], a sixth of the interval — the test neither
/// sleeps for an interval nor runs a round. Both nodes' start rounds run
/// over no group first, so none is in flight when the app writes. The
/// failing input: the nudge the node builds dropped, as it was before
/// u3-2 (B does not see the act within one round's wait).
#[tokio::test]
async fn an_app_appends_on_loopback_and_the_woken_round_carries_it_to_b() {
    assert!(ONE_ROUND < DEFAULT_RING_SYNC_INTERVAL);
    let (da, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (start(1, &da).await, start(2, &db).await);
    // On this one-thread runtime, a yield runs every ready task once: each
    // node's start round finds no group and returns before any await, and
    // its loop then waits on the interval or the nudge.
    tokio::task::yield_now().await;
    hold(&a, NS, 1);
    admit(&a, "bo", 2);
    hold(&b, NS, 1);
    a.transport()
        .add_known_peer(key(2), loopback_sockets(b.endpoint()));

    let answer = app()
        .post(format!(
            "http://{}/v1/rail/append?namespace={NS}",
            a.api_addr()
        ))
        .json(&serde_json::json!({ "op": "record", "payload": { "n": 1 } }))
        .send()
        .await
        .expect("A's record answers on loopback");
    assert_eq!(answer.status(), reqwest::StatusCode::OK);
    let out: serde_json::Value = answer.json().await.unwrap();
    let id = out["id"].as_str().unwrap().to_string();
    assert_eq!(out["actor"], actor_of(&signing(1)), "A's computer signs");
    assert!(holds(&a, NS, &id), "control: A holds the act");
    assert!(!holds(&b, NS, &id), "control: B does not yet");

    let since = Instant::now();
    while !holds(&b, NS, &id) {
        assert!(
            since.elapsed() < ONE_ROUND,
            "B did not see the app's act within {ONE_ROUND:?}: nothing woke the round"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    for n in [&a, &b] {
        n.endpoint().close().await;
    }
}

/// **A group this computer does not hold is refused** (the u3-2 row): an
/// app's append naming one is answered 404 `not_held`, nothing is opened on
/// disk, and the group A does hold is untouched. The failing input: the
/// register's gate skipped in `journal_of` (crates/svrngs-node/src/rail.rs),
/// which the unit test there watched fail.
#[tokio::test]
async fn an_append_naming_a_group_not_held_is_refused() {
    let da = tempfile::tempdir().unwrap();
    let a = start(1, &da).await;
    hold(&a, NS, 1);
    let answer = app()
        .post(format!(
            "http://{}/v1/rail/append?namespace=elsewhere",
            a.api_addr()
        ))
        .json(&serde_json::json!({ "op": "record", "payload": { "n": 1 } }))
        .send()
        .await
        .unwrap();
    assert_eq!(answer.status(), reqwest::StatusCode::NOT_FOUND);
    let out: serde_json::Value = answer.json().await.unwrap();
    assert_eq!(out["kind"], "not_held", "{out}");
    assert!(!da.path().join("rings/elsewhere").exists());
    // No entry written: the keeper's cutover, which a's round writes, is
    // the record's own, and carries none (capability-signed).
    let entries = svrngs_core::tests_support::wire_ops(&a.rail().admission(NS).unwrap());
    assert!(entries.iter().all(|o| o.payload.is_none()), "{entries:?}");
    a.endpoint().close().await;
}

/// **The people come from the register, with their keys** (the u3-2 row;
/// DESIGN §3): A founds the group as ada and admits B as bo; the people
/// door on A's loopback lists both, each with its key's hex, so an app
/// needs no second source. The failing input: the people read from the
/// seed alone, which the unit test in crates/svrngs-node/src/rail/tests.rs
/// watched fail (bo absent).
#[tokio::test]
async fn the_people_read_lists_a_and_b_with_their_keys() {
    let da = tempfile::tempdir().unwrap();
    let a = start(1, &da).await;
    hold(&a, NS, 1);
    admit(&a, "bo", 2);
    let out: serde_json::Value = app()
        .get(format!(
            "http://{}/v1/rail/people?namespace={NS}",
            a.api_addr()
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // When bo joined: the time on the `Admit` (bash-same-name); ada
    // started the group.
    let bo_joined = (out["people"].as_array().unwrap().iter())
        .find(|p| p["person"] == "bo")
        .map(|p| p["joined"].clone())
        .unwrap();
    assert!(bo_joined["at"].is_i64(), "{out}");
    let mut want = vec![
        serde_json::json!({ "person": "ada", "key": actor_of(&signing(1)), "person_key": actor_of(&signing(1)), "joined": "started", "device": "computer" }),
        serde_json::json!({ "person": "bo", "key": actor_of(&signing(2)), "person_key": actor_of(&signing(2)), "joined": bo_joined, "device": "computer" }),
    ];
    want.sort_by_key(|p| p["key"].as_str().unwrap().to_string());
    assert_eq!(out["people"], serde_json::Value::from(want), "{out}");
    a.endpoint().close().await;
}

/// The record API's router over a rail of its own, with no listener: a
/// request is handed whatever caller address the test names, as the
/// listener hands every request its own. The endpoint is kept for the
/// transport the live lane dials through.
async fn api_alone(dir: &tempfile::TempDir) -> (axum::Router, svrngs_node::iroh::Endpoint) {
    let endpoint = svrngs_node::iroh::build_relayed_endpoint(
        svrngs_node::iroh::SecretKey::from_bytes(&signing(1).to_bytes()),
        Vec::new(),
        &no_relay(),
    )
    .await
    .unwrap();
    let router = svrngs_node::api::router(
        svrngs_node::rail::RailApi {
            rail: std::sync::Arc::new(svrngs_node::RingRail::new(
                dir.path(),
                std::sync::Arc::new(signing(1)),
            )),
            nudge: std::sync::Arc::new(tokio::sync::Notify::new()),
            live: Default::default(),
            transport: std::sync::Arc::new(svrngs_node::iroh::IrohTransport::new(endpoint.clone())),
        },
        axum::Router::new(),
        axum::Router::new(),
    );
    (router, endpoint)
}

/// A request to the people door from `caller`, if any; its status and body.
async fn ask_as(router: axum::Router, caller: Option<SocketAddr>) -> (u16, serde_json::Value) {
    use tower::ServiceExt;
    let mut request = axum::http::Request::get(format!("/v1/rail/people?namespace={NS}"))
        .body(axum::body::Body::empty())
        .unwrap();
    if let Some(addr) = caller {
        request
            .extensions_mut()
            .insert(axum::extract::ConnectInfo(addr));
    }
    let answer = router.oneshot(request).await.unwrap();
    let status = answer.status().as_u16();
    let body = axum::body::to_bytes(answer.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

/// **A caller not on loopback is refused by name** (the u3-2 row): each
/// request's caller is asked what the bind is asked (`check_loopback`), so a
/// caller at a network address is refused 403 `not_loopback`, the refusal
/// naming its address, and a caller whose address is unknown is refused
/// too; a caller at 127.0.0.1, or at `::ffff:127.0.0.1`, reaches the door.
/// The failing input: the loopback check skipped, every caller served.
#[tokio::test]
async fn a_caller_not_on_loopback_is_refused_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let (router, _endpoint) = api_alone(&dir).await;
    for caller in ["192.168.1.5:50000", "[2001:db8::5]:50000"] {
        let (status, out) = ask_as(router.clone(), Some(caller.parse().unwrap())).await;
        assert_eq!(status, 403, "{caller}: {out}");
        assert_eq!(out["kind"], "not_loopback", "{caller}");
        let named = out["error"].as_str().unwrap();
        assert!(
            named.contains(caller),
            "the refusal names {caller}: {named}"
        );
    }
    let (status, out) = ask_as(router.clone(), None).await;
    assert_eq!(
        (status, &out["kind"]),
        (403, &serde_json::json!("not_loopback")),
        "{out}"
    );
    for caller in ["127.0.0.1:50000", "[::ffff:127.0.0.1]:50000", "[::1]:50000"] {
        let (status, out) = ask_as(router.clone(), Some(caller.parse().unwrap())).await;
        assert_eq!(
            (status, &out["kind"]),
            (404, &serde_json::json!("not_held")),
            "{caller} reaches the door, which holds no group: {out}"
        );
    }
}

/// A live push from an app on `node`'s computer to `ns`: its status and body.
async fn push(node: &Node, ns: &str, payload: Vec<u8>) -> (u16, serde_json::Value) {
    let answer = app()
        .post(format!(
            "http://{}/v1/rail/live?namespace={ns}",
            node.api_addr()
        ))
        .body(payload)
        .send()
        .await
        .unwrap();
    (answer.status().as_u16(), answer.json().await.unwrap())
}

/// What an app on `node`'s computer drains from `ns`'s live lane.
async fn drain(node: &Node, ns: &str) -> serde_json::Value {
    app()
        .get(format!(
            "http://{}/v1/rail/live?namespace={ns}",
            node.api_addr()
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

/// What `node`'s app drains from `ns`, each payload's text alone.
async fn drained(node: &Node, ns: &str) -> Vec<String> {
    let got = drain(node, ns).await;
    assert_eq!(got["dropped"], 0, "{got}");
    let each = got["payloads"].as_array().unwrap().iter();
    each.map(|p| p["payload"].as_str().unwrap().to_string())
        .collect()
}

/// A founds `ns` and admits B as bo by an act, so B stands by the register
/// and not by the seed; B holds `ns` too, and A can reach B.
async fn a_and_b_in(ns: &str, a: &Node, b: &Node) {
    hold(a, ns, 1);
    admit_to(a, ns, "bo", 2);
    hold(b, ns, 1);
    a.transport()
        .add_known_peer(key(2), loopback_sockets(b.endpoint()));
}

/// **A pushes on loopback and B drains the payload** (the u3-3 row): an app
/// on A's computer pushes to the group; A offers it over iroh to every key
/// standing in it but its own — B, admitted by an act, not named in the
/// seed — and says B took it; an app on B's computer drains it, once.
/// Nothing is buffered on A. The failing input: the fan-out sent to the
/// seed's keys instead of the register's (B is not offered it).
#[tokio::test]
async fn a_pushes_on_loopback_and_b_drains_the_payload() {
    let (da, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (start(1, &da).await, start(2, &db).await);
    a_and_b_in(NS, &a, &b).await;

    let (status, out) = push(&a, NS, b"AQID".to_vec()).await;
    assert_eq!(status, 200, "{out}");
    assert_eq!(out["delivered"], 1, "{out}");
    assert_eq!(out["peers"][0]["node"], actor_of(&signing(2)), "{out}");
    assert_eq!(out["peers"][0]["name"], "bo", "{out}");

    assert_eq!(drained(&b, NS).await, ["AQID"]);
    assert!(drained(&b, NS).await.is_empty(), "a drain empties");
    assert!(drained(&a, NS).await.is_empty(), "A does not hear itself");
    for n in [&a, &b] {
        n.endpoint().close().await;
    }
}

/// **A payload drained names who sent it** (onair-live; ON_AIR §4.3): the
/// group's seed names ada by key 1 and bo by keys 2 and 6, so bo's first
/// key is 2; B is bo's second device, key 6. A's payload drained on B names
/// key 1 and person key 1; B's drained on A names key 6 and bo's first, 2,
/// as `/v1/rail/people` names them. The failing input: the drain without
/// its sender (the payload alone, as u3-3 left it).
#[tokio::test]
async fn a_payload_drained_names_its_sender_and_their_person() {
    let (da, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (start(1, &da).await, start(6, &db).await);
    let seed = Roster::new(
        [
            (Person::parse("ada").unwrap(), vec![actor_of(&signing(1))]),
            (
                Person::parse("bo").unwrap(),
                vec![actor_of(&signing(2)), actor_of(&signing(6))],
            ),
        ]
        .into_iter()
        .collect(),
    );
    for n in [&a, &b] {
        n.rail().set_seed(NS, &seed).unwrap();
    }
    a.transport()
        .add_known_peer(key(6), loopback_sockets(b.endpoint()));
    b.transport()
        .add_known_peer(key(1), loopback_sockets(a.endpoint()));

    let (ada, bo) = (actor_of(&signing(1)), actor_of(&signing(2)));
    let bos_device = actor_of(&signing(6));
    for (from, to, payload, sender, person) in [
        (&a, &b, "AQID", &ada, &ada),
        (&b, &a, "BAUG", &bos_device, &bo),
    ] {
        let (status, out) = push(from, NS, payload.into()).await;
        assert_eq!(status, 200, "{out}");
        assert_eq!(
            drain(to, NS).await["payloads"],
            serde_json::json!([{ "payload": payload, "key": sender, "person_key": person }]),
            "{payload} names who sent it"
        );
    }
    for n in [&a, &b] {
        n.endpoint().close().await;
    }
}

/// **A payload over 4096 bytes is refused** (the u3-3 row): A's push door
/// answers 413 naming the cap and sends nothing, so B drains nothing; one
/// of exactly 4096 bytes goes. The failing input: the push door's cap
/// skipped (the payload goes out, and B's live route refuses it in turn).
#[tokio::test]
async fn a_live_payload_over_4096_bytes_is_refused() {
    let (da, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (start(1, &da).await, start(2, &db).await);
    a_and_b_in(NS, &a, &b).await;

    let (status, out) = push(&a, NS, vec![b'x'; 4097]).await;
    assert_eq!(status, 413, "{out}");
    assert!(out["error"].as_str().unwrap().contains("4096"), "{out}");
    assert!(drained(&b, NS).await.is_empty());

    let (status, out) = push(&a, NS, vec![b'x'; 4096]).await;
    assert_eq!(
        (status, &out["delivered"]),
        (200, &serde_json::json!(1)),
        "{out}"
    );
    assert_eq!(drained(&b, NS).await[0].len(), 4096);
    for n in [&a, &b] {
        n.endpoint().close().await;
    }
}

/// **A drain takes only its own group's payloads** (the u3-3 row): A and B
/// are both in two groups; A pushes one payload to each; B's drain of one
/// group answers that group's payload alone, and the other's is still there
/// for its own drain. The failing input: the live route buffering every
/// payload under one group whatever its envelope names.
#[tokio::test]
async fn a_drain_takes_only_its_own_groups_payloads() {
    let (da, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (start(1, &da).await, start(2, &db).await);
    a_and_b_in(NS, &a, &b).await;
    a_and_b_in("yard", &a, &b).await;

    for (ns, payload) in [(NS, "SA"), ("yard", "WQ")] {
        let (status, out) = push(&a, ns, payload.into()).await;
        assert_eq!(
            (status, &out["delivered"]),
            (200, &serde_json::json!(1)),
            "{ns}: {out}"
        );
    }
    assert_eq!(drained(&b, "yard").await, ["WQ"]);
    assert_eq!(drained(&b, NS).await, ["SA"]);
    for n in [&a, &b] {
        n.endpoint().close().await;
    }
}

/// `node` holds `ns`, seeded with `founder`'s key, under the founder's name.
fn hold_as(node: &Node, ns: &str, founder: u8, name: &str) {
    let roster = Roster::new(
        [(
            Person::parse(name).unwrap(),
            vec![actor_of(&signing(founder))],
        )]
        .into_iter()
        .collect(),
    );
    node.rail().set_seed(ns, &roster).unwrap();
}

/// **A dialer standing in another group A holds is refused** (the u3-3
/// row): A holds `house` and `yard`; C stands in `yard` alone, so A's
/// acceptor admits C. C's own copy of `house`, which C founded, names A, so
/// C's app pushes to `house` and C offers it to A: A's live route refuses
/// it 403 as the sync route would, C's answer says A did not take it and
/// why, and A's app drains nothing from `house`. C's push to `yard` reaches
/// A. The failing input: the live route's standing check skipped (A
/// buffers C's payload in a group C is not in).
#[tokio::test]
async fn a_dialer_standing_in_another_group_a_holds_is_refused() {
    let (da, dc) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, c) = (start(1, &da).await, start(3, &dc).await);
    hold(&a, NS, 1);
    hold(&a, "yard", 1);
    admit_to(&a, "yard", "cy", 3);
    hold_as(&c, NS, 3, "cy");
    admit_to(&c, NS, "ada", 1);
    hold(&c, "yard", 1);
    c.transport()
        .add_known_peer(key(1), loopback_sockets(a.endpoint()));

    let (status, out) = push(&c, NS, b"AQID".to_vec()).await;
    assert_eq!(status, 200, "{out}");
    assert_eq!(out["delivered"], 0, "{out}");
    assert_eq!(out["peers"][0]["node"], actor_of(&signing(1)), "{out}");
    let why = out["peers"][0]["error"].as_str().unwrap();
    assert!(why.contains("403"), "A refused it: {why}");
    assert!(drained(&a, NS).await.is_empty());

    let (status, out) = push(&c, "yard", b"BAUG".to_vec()).await;
    assert_eq!(
        (status, &out["delivered"]),
        (200, &serde_json::json!(1)),
        "control: {out}"
    );
    assert_eq!(drained(&a, "yard").await, ["BAUG"]);
    for n in [&a, &c] {
        n.endpoint().close().await;
    }
}

/// **An app has this computer's key sign bytes for it** (onair-sign (c);
/// svrn-docs `rings/current/ON_AIR.md` §4.2): `POST /v1/sign?app=notes`
/// answers the key that signs this computer's acts and a signature that
/// checks for `notes` over those bytes alone, with the key, the app and the
/// bytes; a name that is not an app's, or none, is refused 400, saying so;
/// and a page in a browser is refused 403 before the route, as it is
/// before the record's.
#[tokio::test]
async fn an_app_has_this_computers_key_sign_its_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let node = start(1, &dir).await;
    let sign = |name: &str| {
        app()
            .post(format!("http://{}/v1/sign?app={name}", node.api_addr()))
            .body("line 812")
    };
    let answer = sign("notes").send().await.unwrap();
    assert_eq!(answer.status(), 200);
    let out: serde_json::Value = answer.json().await.unwrap();
    let (key, sig) = (out["key"].as_str().unwrap(), out["sig"].as_str().unwrap());
    assert_eq!(key, actor_of(&signing(1)), "the key that signs its acts");
    let notes = svrngs_core::app::App::parse("notes").unwrap();
    assert!(notes.check(key, b"line 812", sig), "{out}");
    assert!(!notes.check(key, b"line 813", sig));
    for not in ["Notes", "no:tes", ""] {
        let answer = sign(not).send().await.unwrap();
        assert_eq!(answer.status(), 400, "{not:?}");
        let out: serde_json::Value = answer.json().await.unwrap();
        assert!(
            out["error"].as_str().unwrap().contains("not an app's name"),
            "{out}"
        );
    }
    let page = sign("notes").header("origin", "http://elsewhere.example");
    assert_eq!(
        page.send().await.unwrap().status(),
        403,
        "a page is refused"
    );
}
