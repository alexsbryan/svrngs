// SPDX-License-Identifier: AGPL-3.0-or-later
//! Each family of the door's refusals, decided before the lend's service
//! is asked: one crossing, A's door over QUIC, holding that `lend_for`,
//! `holder_of`, `choices::check`, the acceptor and the listing routes ask
//! the decisions `src/door/tests.rs`, `src/door/choices/tests.rs`,
//! `src/door/limit/tests.rs`, `src/lend/tests.rs` and `src/node/tests.rs`
//! judge on their own, and answer by them. Each step names the test it came
//! from, retired at its move (docs/seams.tsv).

use super::*;
use svrngs_core::alpn::Alpn;
use svrngs_node::door::limit::WINDOW;
use svrngs_node::door::SHARE_HEADER;

const WORDS: &str = "the app receives ranked suggestions; your history stays where it is";
/// A third of 30 Mbit/s, in bits a second.
const A_THIRD: &str = "10000000";

/// `path` posted through `at` with a JSON `body`, `headers` added.
async fn post(at: SocketAddr, path: &str, headers: &str, body: &str) -> Reply {
    raw_to(
        at,
        format!(
            "POST {path} HTTP/1.1\r\nHost: local\r\n{headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
    .await
}

/// That `reply` is svrngs's refusal `want`, `status`, saying `words`.
fn refused(reply: &Reply, status: u16, want: &str, words: &str) {
    assert_eq!(reply.status, status, "{want}: {}", reply.text);
    assert_eq!(kind(reply).as_deref(), Some(want), "{}", reply.text);
    assert!(reply.text.contains(words), "{words}: {}", reply.text);
}

/// What A lends, as the dialer at `at` asks with `marker`.
async fn listed(at: SocketAddr, marker: &str) -> (u16, serde_json::Value) {
    let reply = ask(at, "GET", "/lends", marker).await;
    let body = reply.text.split_once("\r\n\r\n").map_or("", |(_, b)| b);
    (reply.status, serde_json::from_str(body).unwrap_or_default())
}

/// What `at` answers `request` on its connection, every byte: none when it
/// closes the connection unserved.
async fn served(at: SocketAddr, request: &str) -> String {
    let mut sock = tokio::net::TcpStream::connect(at).await.unwrap();
    sock.write_all(request.as_bytes()).await.unwrap();
    let mut reply = Vec::new();
    let read = tokio::time::timeout(Duration::from_secs(10), sock.read_to_end(&mut reply)).await;
    assert!(read.is_ok(), "neither answered nor closed: {request}");
    String::from_utf8_lossy(&reply).into_owned()
}

/// Whether A closes a connection Sam's visit key dials on `alpn` within
/// three seconds, having served it nothing.
async fn closes_to_sam(a: &Node, alpn: Alpn) -> bool {
    use svrngs_node::guest::ring_crypto_provider;
    use svrngs_node::iroh::{EndpointAddr, SecretKey};
    let sam = iroh::endpoint::Builder::empty()
        .crypto_provider(ring_crypto_provider())
        .secret_key(SecretKey::from_bytes(&signing(SAM).to_bytes()))
        .bind()
        .await
        .unwrap();
    let mut target = EndpointAddr::new(a.endpoint().id());
    for sock in common::loopback_sockets(a.endpoint()) {
        target = target.with_ip_addr(sock);
    }
    match sam.connect(target, alpn.bytes()).await {
        Err(_) => true,
        Ok(c) => tokio::time::timeout(Duration::from_secs(3), c.closed())
            .await
            .is_ok(),
    }
}

/// `who` streams `path` through `at`: the answer's head and first chunk
/// read, the rest left to come for as long as the socket is held.
async fn streaming(at: SocketAddr, path: &str) -> (tokio::net::TcpStream, String) {
    let mut sock = tokio::net::TcpStream::connect(at).await.unwrap();
    sock.write_all(format!("GET {path} HTTP/1.1\r\nHost: local\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let mut got = Vec::new();
    while !got.windows(5).any(|w| w == b"hello") {
        let mut buf = [0u8; 1024];
        let n = tokio::time::timeout(Duration::from_secs(20), sock.read(&mut buf))
            .await
            .expect("the stream starts")
            .unwrap();
        assert!(n > 0, "the stream ended early: {got:?}");
        got.extend_from_slice(&buf[..n]);
    }
    clean(&got);
    (sock, String::from_utf8_lossy(&got).into_owned())
}

fn share_in(reply: &str) -> Vec<&str> {
    header_values(reply.split("\r\n\r\n").next().unwrap(), SHARE_HEADER)
}

/// Planted, each watched failing when its test was its own: the service
/// asked a refused path; the refusal header off; the door taking a pass for
/// any lend, reading no link, taking any link that reads; an upgrade let
/// through as a read; the method left unchecked (another app's POST let in
/// as readers); a member's pass read whoever signed it; `admit`'s sends
/// true for any app; the dialer resolved through any group held here; the
/// lender answering an unmarked list; the audience asked alone, never
/// whether the lender stands there; `stamp` writing `null`; a lend reaching
/// every group its lender stands in; the guest's key put in `Standing`; a
/// stop not kept and a pass's yes accepted; choices kept per key; the share
/// divided by requests; the door counting each device's key; unreadable
/// declarations answered as lending nothing; the pass's removed check out
/// and a removed dialer closed like a stranger.
#[tokio::test]
async fn each_refusal_is_decided_before_the_service_is_asked() {
    let (said, _heard) = common::heard("node=debug");
    let (legacy, legacy_heads) = origins::head_origin().await;
    let world = World::new(|dir| {
        // a_lend_from_before_groups_opens_as_it_did_and_can_then_be_pared_down
        // (scope.rs:404-449): declarations from before groups, found at start.
        let lends = lend::lends_dir(dir);
        std::fs::create_dir_all(&lends).unwrap();
        let port = legacy.port();
        for (name, tail) in [
            ("old", ""),
            ("stamped", r#","groups":null"#),
            ("paused", r#","groups":[]"#),
            ("chosen", r#","groups":["trip"]"#),
        ] {
            let body = format!(r#"{{"port":{port},"kind":"k","refused":[]{tail}}}"#);
            std::fs::write(lends.join(format!("{name}.json")), body).unwrap();
        }
    })
    .await;
    let audience = |name: &str| {
        let name = LendName::parse(name).unwrap();
        lend::read(world.dir.path(), &name)
            .unwrap()
            .unwrap()
            .audience
    };
    let held = vec![NS.to_string(), "trip".to_string(), "yard".to_string()];
    for name in ["old", "stamped"] {
        assert_eq!(
            audience(name),
            lend::Audience::Selected(held.clone()),
            "{name}"
        );
    }
    assert_eq!(audience("paused"), lend::Audience::Selected(Vec::new()));
    assert_eq!(
        audience("chosen"),
        lend::Audience::Selected(vec!["trip".into()])
    );
    let bo = world.as_key(2, Alpn::Member).await;
    for name in ["old", "stamped"] {
        let reply = ask(bo, "GET", &format!("/lend/{name}/whoami"), "").await;
        assert_eq!(reply.status, 200, "{name}: {}", reply.text);
        assert_eq!(
            header_values(&last(&legacy_heads), "x-svrngs-person"),
            vec!["bo"]
        );
    }
    assert_eq!(ask(bo, "GET", "/lend/paused/whoami", "").await.status, 403);
    world
        .a
        .rail()
        .set_seed(
            "later",
            &Roster::new(
                [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
                    .into_iter()
                    .collect(),
            ),
        )
        .unwrap();
    let old = audience("old");
    assert!(
        !old.ids().unwrap().contains(&"later".to_string()),
        "{old:?}"
    );

    let (origin, heads) = origins::head_origin().await;
    let lent = |port: u16, groups: &[&str]| serde_json::json!({ "port": port, "kind": "k", "limit": 3, "refused": ["/Auth/Keys"], "groups": groups });
    let shelf_json = lent(origin.port(), &[NS]);
    world.lend("shelf", shelf_json.clone());
    world.secret("shelf", "authorization", SECRET);
    let shelf = world.label("shelf");
    let asked = || heads.lock().unwrap().len();

    // a_refused_path_is_refused (main.rs:471-494).
    for (path, status) in [
        ("/lend/shelf/Auth/Keys", 403),
        ("/lend/shelf/auth/keys/1", 403),
        ("/lend/shelf/%41uth/Keys", 403),
        ("/lend/shelf//Auth/Keys", 403),
        ("/lend/shelf/../x", 400),
        ("/lend/shelf/a%2fb", 400),
        ("/lend/shelf/%2e%2e/x", 400),
        ("/lend/nothing/x", 404),
        ("/lend/no..name/x", 404),
    ] {
        let reply = ask(bo, "GET", path, "").await;
        assert_eq!(reply.status, status, "{path}: {}", reply.text);
    }
    assert_eq!(asked(), 0, "the service was asked a refused path");
    let reply = ask(bo, "GET", "/lend/shelf/Auth/Keyshelf", "").await;
    assert_eq!(
        reply.status, 200,
        "control: a path beside the refused one is lent"
    );

    // svrngs_s_refusals_are_marked_its_own_and_a_lend_s_are_not (main.rs:510-535).
    world.lend(
        "strict",
        lent(origins::refusing_origin().await.port(), &[NS]),
    );
    let gone = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    world.lend("gone", lent(gone.local_addr().unwrap().port(), &[NS]));
    drop(gone);
    for (path, status, mark) in [
        ("/lend/nothing/x", 404, Some("unlent")),
        ("/lend/shelf/Auth/Keys", 403, Some("path")),
        ("/lend/shelf/../x", 400, Some("path")),
        ("/lend/gone/x", 502, Some("away")),
        ("/lend/strict/x", 403, None),
        ("/lend/shelf/x", 200, None),
    ] {
        let reply = ask(bo, "GET", path, "").await;
        assert_eq!(reply.status, status, "{path}: {}", reply.text);
        assert_eq!(kind(&reply).as_deref(), mark, "{path}: {}", reply.text);
        if let Some(kind) = mark {
            let body = reply.text.split_once("\r\n\r\n").unwrap().1;
            assert!(
                body.contains(&format!(r#""kind":"{kind}""#)),
                "{path}: {body}"
            );
        }
    }
    let strict = ask(bo, "GET", "/lend/strict/x", "").await;
    assert!(strict.text.contains("the app's own no"), "{}", strict.text);

    // a_pass_opens_only_the_lend_it_names_to_the_key_holding_it (main.rs:1192-1231).
    let (other, other_heads) = origins::head_origin().await;
    world.lend("other", lent(other.port(), &[NS]));
    let elsewhere = world.label("other");
    let holder = world.as_key(19, Alpn::Lend).await;
    let until = unix_now() + 3600;
    let bos = pass(2, 19, &shelf, &[], until);
    let before = asked();
    for (why, path, pass) in [
        (
            "another lend, by its label",
            format!("/lend/{elsewhere}/x"),
            bos.clone(),
        ),
        (
            "another lend, by its name",
            "/lend/other/x".into(),
            bos.clone(),
        ),
        ("no pass", format!("/lend/{shelf}/x"), String::new()),
        (
            "a lapsed pass",
            format!("/lend/{shelf}/x"),
            pass(2, 19, &shelf, &[], unix_now() - 1),
        ),
        (
            "signed by a key standing nowhere",
            format!("/lend/{shelf}/x"),
            pass(5, 19, &shelf, &[], until),
        ),
        (
            "held by another key",
            format!("/lend/{shelf}/x"),
            pass(2, 8, &shelf, &[], until),
        ),
    ] {
        let reply = ask(holder, "GET", &path, &pass).await;
        assert_eq!(reply.status, 403, "{why}: {}", reply.text);
        assert_eq!(kind(&reply).as_deref(), Some("pass"), "{why}");
    }
    assert!(
        other_heads.lock().unwrap().is_empty(),
        "nothing reached other"
    );

    // a_link_carries_a_pass_to_the_key_dialling_and_no_further (main.rs:1289-1315).
    let endpoint = world.as_key(10, Alpn::Lend).await;
    let carried = svrngs_core::pass::Pass::sign(
        &signing(2),
        &actor_of(&signing(19)),
        (shelf.clone(), vec![]),
        until,
    );
    let signed = |by: u8, mut link: svrngs_core::pass::Pass| {
        link.sig = svrngs_core::sign_message(&signing(by), &link.message());
        Some(link.encode())
    };
    let link = carried.link(&actor_of(&signing(10)));
    let mut wider = link.clone();
    wider.reads.push(elsewhere.clone());
    for (why, link) in [
        ("the pass alone, its holder not the dialer", None),
        ("a link naming a lend the pass does not", signed(19, wider)),
        (
            "a link signed by another key than the holder",
            signed(8, link),
        ),
        ("a link that does not read", Some("not-a-link".to_string())),
    ] {
        let link = link.map_or(String::new(), |l| format!("x-svrngs-link: {l}\r\n"));
        let headers = format!("x-svrngs-pass: {}\r\n{link}", carried.encode());
        let reply = ask(endpoint, "GET", &format!("/lend/{shelf}/whoami"), &headers).await;
        assert_eq!(reply.status, 403, "{why}: {}", reply.text);
        assert_eq!(kind(&reply).as_deref(), Some("pass"), "{why}");
    }
    assert_eq!(asked(), before, "no refused pass or link reached shelf");

    // an_app_reads_a_lend_bare_and_never_writes_or_upgrades_it (main.rs:1377-1395).
    let picks = world.label("picks");
    for (how, at, pass) in [
        (
            "a phone's pass",
            holder,
            pass(2, 19, &picks, &[&shelf], until),
        ),
        (
            "a computer's own pass",
            bo,
            pass(2, 2, &picks, &[&shelf], until),
        ),
    ] {
        let path = format!("/lend/{shelf}/whoami");
        let upgrade = format!("{pass}Connection: Upgrade, close\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\n");
        for (what, reply) in [
            ("POST /Sessions/Logout", ask(at, "POST", &path, &pass).await),
            (
                "a WebSocket upgrade",
                raw_to(
                    at,
                    format!("GET {path} HTTP/1.1\r\nHost: local\r\n{upgrade}\r\n"),
                )
                .await,
            ),
        ] {
            assert_eq!(reply.status, 403, "{how}: {what}: {}", reply.text);
            assert_eq!(kind(&reply).as_deref(), Some("closed"), "{how}: {what}");
        }
    }
    assert_eq!(asked(), before, "an app's write reached the service");

    // the_readers_secret_goes_only_to_another_app_s_get (main.rs:1491-1509).
    let reply = ask(
        holder,
        "POST",
        &format!("/lend/{shelf}/whoami"),
        &pass(2, 19, &picks, &[&shelf], until),
    )
    .await;
    refused(&reply, 403, "closed", "Apps can only read ada’s");

    // a_pass_s_app_is_told_and_a_client_s_own_stripped (main.rs:1572-1631): a
    // pass from before the app had a field of its own; a member's own pass
    // naming another lend, or signed by another key.
    let mut old_pass = serde_json::to_value(svrngs_core::pass::Pass::sign(
        &signing(2),
        &actor_of(&signing(19)),
        (picks.clone(), vec![shelf.clone()]),
        until,
    ))
    .unwrap();
    let fields = old_pass.as_object_mut().unwrap();
    fields.remove("app");
    let reads = fields.remove("reads").unwrap();
    fields.insert(
        "lends".into(),
        serde_json::json!([picks.as_str(), reads[0]]),
    );
    use base64::Engine as _;
    let old_pass = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&old_pass).unwrap());
    let reply = ask(
        holder,
        "GET",
        &format!("/lend/{shelf}/whoami"),
        &format!("x-svrngs-pass: {old_pass}\r\n"),
    )
    .await;
    assert_eq!(
        (reply.status, kind(&reply).as_deref()),
        (403, Some("pass")),
        "{}",
        reply.text
    );
    for (row, pass) in [
        (
            "naming another lend",
            pass(2, 2, &picks, &[], unix_now() + 300),
        ),
        (
            "signed by another key",
            pass(5, 2, &picks, &[&shelf], unix_now() + 300),
        ),
    ] {
        let reply = ask(bo, "GET", "/lend/shelf/whoami", &pass).await;
        assert_eq!(
            (reply.status, kind(&reply).as_deref()),
            (403, Some("pass")),
            "{row}: {}",
            reply.text
        );
    }

    // a_send_is_a_carried_app_s_from_its_person_s_own_computer (main.rs:1854-1885).
    let mut sends = shelf_json.clone();
    sends["sends"] = serde_json::json!(["POST /whoami"]);
    world.lend("shelf", sends);
    let taste = svrngs_core::pass::Label::carried("Taste", NS);
    let posted = |at: SocketAddr, pass: String, path: &'static str| {
        let shelf = shelf.clone();
        async move { post(at, &format!("/lend/{shelf}{path}"), &pass, "{}").await }
    };
    for (why, reply) in [
        (
            "a lent page's send",
            posted(bo, pass(2, 2, &picks, &[&shelf], until), "/whoami").await,
        ),
        (
            "a pass and link's send",
            posted(holder, pass(2, 19, &taste, &[&shelf], until), "/whoami").await,
        ),
        (
            "a send with a query",
            posted(
                bo,
                pass(2, 2, &taste, &[&shelf], until),
                "/whoami?_method=DELETE",
            )
            .await,
        ),
    ] {
        refused(&reply, 403, "closed", "only as POST /whoami, exactly");
        let _ = why;
    }
    world.lend("shelf", shelf_json.clone());
    assert_eq!(asked(), before, "a refused write or pass reached shelf");

    // Scope (docs/LENDING_SCOPE.md): a_member_of_no_group_it_is_lent_to_neither_lists_nor_reaches_it
    // (scope.rs:60-70), a_pass_signed_by_a_member_of_no_group_it_is_lent_to_opens_nothing
    // (scope.rs:141-148), a_group_the_lender_no_longer_stands_in_reaches_nothing
    // (scope.rs:346-348), a_list_asked_without_this_builds_scope_marker_is_refused
    // (scope.rs:157-163), joining_a_third_group_leaves_the_declaration_and_reaches_nothing
    // (scope.rs:367-378).
    let (scoped, scoped_heads) = origins::head_origin().await;
    world.lend("yardonly", lent(scoped.port(), &["yard"]));
    world.lend("bosonly", lent(scoped.port(), &["bos"]));
    world.lend("stale", lent(scoped.port(), &[NS]));
    let (status, lends) = listed(bo, "x-svrngs-version: 5\r\n").await;
    assert_eq!(status, 200);
    let names: Vec<&str> = lends["lends"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["name"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"yardonly"), "{names:?}");
    // Nor does it list one lent only to a group A no longer stands in, which
    // it would refuse to open (door.rs `reached`): bo stands in "bos", A not.
    assert!(!names.contains(&"bosonly"), "{names:?}");
    let forged = "x-svrngs-group: yard\r\nx-svrngs-person: cy\r\n";
    let reply = ask(bo, "GET", "/lend/yardonly/whoami", forged).await;
    assert_eq!(
        (reply.status, kind(&reply).as_deref()),
        (403, Some("closed")),
        "{}",
        reply.text
    );
    assert_eq!(ask(bo, "GET", "/lend/bosonly/whoami", "").await.status, 403);
    let stale = world.label("stale");
    let bos_stale = pass(2, 19, &stale, &[], until);
    assert_eq!(
        ask(holder, "GET", &format!("/lend/{stale}/whoami"), &bos_stale)
            .await
            .status,
        200
    );
    let name = LendName::parse("stale").unwrap();
    assert!(lend::set_audience(world.dir.path(), &name, vec!["yard".into()]).unwrap());
    let scoped_asked = scoped_heads.lock().unwrap().len();
    assert_eq!(
        ask(holder, "GET", &format!("/lend/{stale}/whoami"), &bos_stale)
            .await
            .status,
        403
    );
    for marker in ["", "x-svrngs-version: 4\r\n"] {
        for route in ["/lends", svrngs_node::node::PEOPLE_LENDS_ROUTE] {
            let reply = ask(bo, "GET", route, marker).await;
            assert_eq!(reply.status, 403, "{route} {marker:?}: {}", reply.text);
        }
    }
    let file = lend::lends_dir(world.dir.path()).join("yardonly.json");
    let declared_bytes = std::fs::read(&file).unwrap();
    let club = [("ada", 1), ("bo", 2)]
        .map(|(p, k)| (Person::parse(p).unwrap(), vec![actor_of(&signing(k))]));
    world
        .a
        .rail()
        .set_seed("club", &Roster::new(club.into_iter().collect()))
        .unwrap();
    let (_, lends) = listed(bo, "x-svrngs-version: 5\r\n").await;
    assert!(!lends.to_string().contains("yardonly"), "{lends}");
    assert_eq!(
        ask(bo, "GET", "/lend/yardonly/whoami", "").await.status,
        403
    );
    assert_eq!(
        std::fs::read(&file).unwrap(),
        declared_bytes,
        "the declaration is unchanged"
    );
    assert_eq!(
        scoped_heads.lock().unwrap().len(),
        scoped_asked,
        "the service was asked"
    );

    // A guest: sam_opens_no_lend_of_bos_nor_one_kim_did_not_open (guests.rs:197-205),
    // and sams_visit_key_syncs_nothing_and_reads_no_person_presence_or_record
    // (guests.rs:330-354).
    let (films, films_heads) = origins::head_origin().await;
    world.lend("films", lent(films.port(), &[NS]));
    world.visit(&["shelf"]);
    let sams_holder = world.as_key(11, Alpn::Lend).await;
    let films_label = world.label("films");
    let sams = pass(SAM, 11, &films_label, &[], until);
    let reply = ask(
        sams_holder,
        "GET",
        &format!("/lend/{films_label}/whoami"),
        &sams,
    )
    .await;
    refused(&reply, 403, "closed", "ada did not open films to you");
    assert!(
        films_heads.lock().unwrap().is_empty(),
        "ada's films was asked"
    );
    let sync = serde_json::to_string(&svrngs_node::peer_wire::RingSyncRequest {
        namespace: NS.into(),
        digest: Default::default(),
        ops: Vec::new(),
    })
    .unwrap();
    let posted_sync = |route: &str| {
        format!("POST {route} HTTP/1.1\r\nHost: local\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{sync}", sync.len())
    };
    let gets =
        |route: &str| format!("GET {route} HTTP/1.1\r\nHost: local\r\nConnection: close\r\n\r\n");
    let (sam_member, sam_holders) = (
        world.as_key(SAM, Alpn::Member).await,
        world.as_key(SAM, Alpn::Lend).await,
    );
    let mut was_served = Vec::new();
    for request in [
        posted_sync("/internal/ring/sync"),
        posted_sync("/internal/ring/live"),
        gets("/people/lends"),
        gets("/lends"),
        gets("/v1/rail/people"),
    ] {
        let line = request.lines().next().unwrap().to_string();
        let member = served(sam_member, &request).await;
        if !member.is_empty() {
            was_served.push(format!("members' protocol, {line}: {member}"));
        }
        let holders = served(sam_holders, &request).await;
        if !holders.starts_with("HTTP/1.1 404") && !holders.starts_with("HTTP/1.1 405") {
            was_served.push(format!("holders' protocol, {line}: {holders}"));
        }
    }
    if !closes_to_sam(&world.a, Alpn::Blobs).await {
        was_served.push("the blobs protocol kept Sam's connection".into());
    }
    assert!(
        was_served.is_empty(),
        "Sam's visit key was served: {was_served:#?}"
    );

    // a_lend_that_asks_lets_in_only_the_app_its_person_said_yes_to (choices.rs:62-200)
    // and a_persons_yes_on_one_device_holds_on_another (choices.rs:224).
    let (at, taste_heads) = origins::head_origin().await;
    let declare = || {
        world.lend("taste", serde_json::json!({ "port": at.port(), "kind": "taste", "limit": 3, "asks": WORDS, "groups": [NS] }))
    };
    declare();
    let (taste, night) = (world.label("taste"), world.label("night"));
    let own = |app: &svrngs_core::pass::Label| pass(2, 2, app, &[&taste], unix_now() + 300);
    let shelf_reply = ask(bo, "GET", "/lend/shelf/x", "").await;
    assert_eq!(
        (shelf_reply.status, kind(&shelf_reply)),
        (200, None),
        "a lend that asks nobody"
    );
    assert!(header_values(&last(&heads), "x-svrngs-app").is_empty());
    let none = ask(bo, "GET", "/lend/taste/x", "").await;
    assert_eq!(
        (none.status, kind(&none).as_deref()),
        (403, Some("unasked")),
        "no app named"
    );
    let unasked = ask(bo, "GET", "/lend/taste/x", &own(&night)).await;
    assert_eq!(
        (unasked.status, kind(&unasked).as_deref()),
        (403, Some("unasked")),
        "nothing said"
    );
    assert!(
        taste_heads.lock().unwrap().is_empty(),
        "no refused request reached taste"
    );
    let yes = format!(r#"{{"app":"{night}","said":"yes"}}"#);
    assert_eq!(
        post(bo, "/choices/taste", "", &yes).await.status,
        200,
        "bo's yes"
    );
    let carried = ask(bo, "GET", "/lend/taste/x", &own(&night)).await;
    assert_eq!(
        (carried.status, kind(&carried)),
        (200, None),
        "after the yes: {}",
        carried.text
    );
    assert_eq!(
        header_values(&last(&taste_heads), "x-svrngs-app"),
        vec![night.as_str()]
    );
    let bo6 = world.as_key(6, Alpn::Member).await;
    let sixes = pass(6, 6, &night, &[&taste], unix_now() + 300);
    let other_device = ask(bo6, "GET", "/lend/taste/x", &sixes).await;
    assert_eq!(
        (other_device.status, kind(&other_device)),
        (200, None),
        "bo's other device"
    );
    let phones = pass(2, 19, &night, &[&taste], unix_now() + 300);
    for (route, at, pass) in [
        ("members'", bo, own(&night)),
        ("holders'", holder, phones.clone()),
    ] {
        let reply = post(at, &format!("/choices/{taste}"), &pass, r#"{"said":"yes"}"#).await;
        assert_eq!(
            (reply.status, kind(&reply).as_deref()),
            (403, Some("closed")),
            "{route} pass's yes"
        );
        let reply = post(at, &format!("/choices/{taste}"), &pass, &yes).await;
        assert_eq!(
            reply.status, 403,
            "{route} pass's yes, naming an app: {}",
            reply.text
        );
    }
    assert_eq!(
        ask(bo, "GET", "/lend/taste/x", &own(&night)).await.status,
        200,
        "no pass's yes kept"
    );
    for (route, at, pass) in [("members'", bo, own(&night)), ("holders'", holder, phones)] {
        assert_eq!(
            post(bo, "/choices/taste", "", &yes).await.status,
            200,
            "bo's yes again"
        );
        let stopped = post(
            at,
            &format!("/choices/{taste}"),
            &pass,
            r#"{"said":"stop"}"#,
        )
        .await;
        assert_eq!(stopped.status, 200, "{route} pass's stop: {}", stopped.text);
        let after = ask(bo, "GET", "/lend/taste/x", &own(&night)).await;
        assert_eq!(
            (after.status, kind(&after).as_deref()),
            (403, Some("stopped")),
            "{route}"
        );
    }
    let mine = ask(bo, "GET", "/choices/taste", "").await;
    assert_eq!(mine.status, 200, "{}", mine.text);
    let mine: serde_json::Value =
        serde_json::from_str(mine.text.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(mine["choices"][night.as_str()]["said"], "stop", "{mine}");
    let pages = ask(bo, "GET", "/lend/taste/x", &own(&taste)).await;
    assert_eq!(
        (pages.status, kind(&pages)),
        (200, None),
        "taste's own pages"
    );
    let (_, lends) = listed(bo, "x-svrngs-version: 5\r\n").await;
    assert!(lends.to_string().contains(WORDS), "{lends}");
    lend::withdraw(world.dir.path(), &LendName::parse("taste").unwrap()).unwrap();
    assert!(!lend::choices_path(world.dir.path(), &LendName::parse("taste").unwrap()).exists());
    declare();
    let again = ask(bo, "GET", "/lend/taste/x", &own(&night)).await;
    assert_eq!(
        (again.status, kind(&again).as_deref()),
        (403, Some("unasked")),
        "shared again"
    );

    // one_person_on_two_devices_is_counted_once (limit.rs:225-227).
    let (one, _) = origins::head_origin().await;
    world.lend(
        "one",
        serde_json::json!({ "port": one.port(), "kind": "k", "limit": 1, "groups": [NS] }),
    );
    assert_eq!(ask(bo, "GET", "/lend/one/x", "").await.status, 200);
    assert_eq!(
        ask(bo6, "GET", "/lend/one/x", "").await.status,
        200,
        "bo's other device"
    );

    // a_removed_person_is_refused_as_removed_by_pass_and_by_dial (main.rs:1765-1797):
    // ron, of the house alone, removed.
    world.let_in(NS, 14, "ron");
    let rons_pass = pass(14, 19, &shelf, &[], until);
    let removal = svrngs_node::Removal {
        person: Person::parse("ron").unwrap(),
        joined: None,
    };
    world.a.rail().commit(NS, &removal).unwrap().unwrap();
    let before = asked();
    let by_pass = ask(holder, "GET", &format!("/lend/{shelf}/x"), &rons_pass).await;
    let by_dial = ask(
        world.as_key(14, Alpn::Member).await,
        "GET",
        "/lend/shelf/x",
        "",
    )
    .await;
    for (how, reply) in [("by pass", by_pass), ("by dial", by_dial)] {
        refused(&reply, 403, "removed", "You are no longer in this group");
        let _ = how;
    }
    assert_eq!(asked(), before, "the service was never asked");
    let trace = String::from_utf8(said.0.lock().unwrap().clone()).unwrap();
    let told = "node: the dialer was cut here, so it is told its cut";
    assert!(trace.contains(told), "{told}: {trace}");

    // three_streaming_are_told_a_third_and_a_fourth_waits_for_one_to_stop
    // (limit.rs:160-210): bo, cy (of yard), dee and eve.
    world.let_in(NS, 12, "dee");
    world.let_in(NS, 13, "eve");
    let (stream, stream_heads) = origins::streaming_origin().await;
    world.lend("stream", serde_json::json!({ "port": stream.port(), "kind": "k", "limit": 3, "upload_bps": 30_000_000, "groups": [NS, "yard"] }));
    let mut crowd = std::collections::BTreeMap::new();
    for (who, key) in [("bo", 2), ("cy", 3), ("dee", 12), ("eve", 13)] {
        crowd.insert(who, world.as_key(key, Alpn::Member).await);
    }
    let mut streams = Vec::new();
    for who in ["bo", "cy", "dee"] {
        streams.push(streaming(crowd[who], "/lend/stream/stream").await);
    }
    assert_eq!(
        share_in(&streams[2].1),
        vec![A_THIRD],
        "dee, the third, is told a third"
    );
    let told_the_service = || {
        let heads = stream_heads.lock().unwrap();
        header_values(heads.last().unwrap(), SHARE_HEADER).join(",")
    };
    for who in ["bo", "cy", "dee"] {
        let reply = ask(crowd[who], "GET", "/lend/stream/seek", "").await;
        assert_eq!(reply.status, 200, "{}", reply.text);
        assert_eq!(
            share_in(&reply.text),
            vec![A_THIRD],
            "{who}: {}",
            reply.text
        );
        assert_eq!(told_the_service(), A_THIRD, "{who}");
    }
    let full = ask(crowd["eve"], "GET", "/lend/stream/seek", "").await;
    refused(
        &full,
        503,
        "full",
        "stream is lent to 3 people at once, and 3 are using it now",
    );
    assert!(full
        .text
        .contains("That number is the choice of the person who lends it, and they can change it."));
    assert!(share_in(&full.text).is_empty(), "{}", full.text);
    drop(streams.remove(1));
    tokio::time::sleep(WINDOW + Duration::from_secs(2)).await;
    for who in ["eve", "bo", "dee"] {
        let reply = ask(crowd[who], "GET", "/lend/stream/seek", "").await;
        assert_eq!(reply.status, 200, "{who}: {}", reply.text);
        assert_eq!(
            share_in(&reply.text),
            vec![A_THIRD],
            "{who}: {}",
            reply.text
        );
    }
    drop(streams);

    // declarations_that_cannot_be_read_are_refused_on_both_routes_never_lent_as_none
    // (main.rs:1025-1036), last: the lends directory set aside.
    let lends = lend::lends_dir(world.dir.path());
    std::fs::rename(&lends, world.dir.path().join("lends-aside")).unwrap();
    std::fs::write(&lends, "not a directory").unwrap();
    for route in ["/lends", svrngs_node::node::PEOPLE_LENDS_ROUTE] {
        let reply = ask(bo, "GET", route, "x-svrngs-version: 5\r\n").await;
        assert_eq!(reply.status, 500, "{route}: {}", reply.text);
        assert!(
            reply
                .text
                .contains("the computer you reached cannot read what it lends"),
            "{route}: {}",
            reply.text
        );
    }
    world.a.endpoint().close().await;
}
