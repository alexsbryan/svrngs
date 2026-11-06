// SPDX-License-Identifier: AGPL-3.0-or-later
//! Lending to selected groups, through the door (docs/LENDING_SCOPE.md,
//! "Proof before landing" 1 and 2). A holds three groups: the house (ada
//! and bo, [`Pair::new`]'s), trip (ada and bo) and yard (ada and cy). bo
//! stands in the house and trip, never yard; a lend lent only to yard is
//! bo's to neither list nor reach, by dial or by pass, whatever bo's client
//! types, and the service is never asked. Lent to the house and yard, bo
//! reaches it, and is told — and tells the service — only the groups both
//! share with it.

use svrngs_core::alpn::Alpn;
use svrngs_core::pass::{Label, Pass};
use svrngs_node::lend::{self, LendName};
use svrngs_node::{actor_of, Person, Roster};

use super::common::bridge_as;
use super::{header_values, last, lending_heads, raw_to, signing, start, unix_now, Pair, NS};

/// A's three groups, the shelf lent to `to`.
fn groups_held(pair: &Pair, to: &[&str]) {
    let roster = |people: &[(&str, u8)]| {
        let people = people
            .iter()
            .map(|(p, k)| (Person::parse(p).unwrap(), vec![actor_of(&signing(*k))]));
        Roster::new(people.collect())
    };
    pair.a
        .rail()
        .set_seed("trip", &roster(&[("ada", 1), ("bo", 2)]))
        .unwrap();
    pair.a
        .rail()
        .set_seed("yard", &roster(&[("ada", 1), ("cy", 3)]))
        .unwrap();
    to_groups(pair, to);
}

fn to_groups(pair: &Pair, to: &[&str]) {
    let shelf = LendName::parse("shelf").unwrap();
    let to = to.iter().map(|g| g.to_string()).collect();
    assert!(lend::set_audience(pair.dir.path(), &shelf, to).unwrap());
}

/// What `/lends` answers bo, marked as this build's.
async fn listed(pair: &Pair) -> serde_json::Value {
    let reply = pair.get("/lends", "x-svrngs-version: 5\r\n").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    serde_json::from_str(reply.text.split("\r\n\r\n").nth(1).unwrap()).unwrap()
}

/// A Trip-only member, by dial: the shelf, lent only to yard, is not in
/// what bo is told is lent, and a request for it — forged group and
/// person headers and all — is refused `closed` before the service is
/// asked. Planted: the dialer resolved through any group held here
/// (`standing_in` asked with every group eligible).
#[tokio::test]
async fn a_member_of_no_group_it_is_lent_to_neither_lists_nor_reaches_it() {
    let (pair, heads) = lending_heads().await;
    groups_held(&pair, &["yard"]);
    let lends = listed(&pair).await;
    assert_eq!(lends["lends"], serde_json::json!([]), "{lends}");
    let forged = "x-svrngs-group: yard\r\nx-svrngs-person: cy\r\n";
    let reply = pair.get("/lend/shelf/whoami", forged).await;
    assert_eq!(reply.status, 403, "{}", reply.text);
    assert!(
        reply.text.contains("x-svrngs-refused: closed"),
        "{}",
        reply.text
    );
    assert!(heads.lock().unwrap().is_empty(), "the service was asked");
    pair.close().await;
}

/// What a House member is told is lent: once the shelf is lent to the
/// house and yard, bo's list names the shelf with only the house of its
/// audience, the group they share, never yard. Planted: the whole audience
/// listed.
#[tokio::test]
async fn a_list_tells_a_member_only_the_groups_it_shares_with_the_lend() {
    let (pair, _heads) = lending_heads().await;
    groups_held(&pair, &[NS, "yard"]);
    let lends = listed(&pair).await;
    assert_eq!(
        lends["lends"][0]["groups"],
        serde_json::json!([NS]),
        "{lends}"
    );
    pair.close().await;
}

/// The groups the service is told a House member's request comes with:
/// the house's page, never trip's, which bo stands in and the shelf is not
/// lent to. Planted: every group bo stands in told (`pages` unfiltered).
#[tokio::test]
async fn the_service_is_told_only_the_groups_the_lend_reaches() {
    let (pair, heads) = lending_heads().await;
    groups_held(&pair, &[NS, "yard"]);
    let reply = pair.get("/lend/shelf/whoami", "").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let pages = header_values(&last(&heads), "x-svrngs-pages").join(" ");
    assert!(pages.starts_with(&format!("{NS}=")), "{pages}");
    assert!(
        !pages.contains("trip") && !pages.contains("yard"),
        "{pages}"
    );
    pair.close().await;
}

/// A stale pass: bo's pass for the shelf, signed while it was lent to the
/// house, opens nothing once it is lent only to yard, and the service is
/// not asked again. Planted: the signer resolved through any group.
#[tokio::test]
async fn a_pass_signed_by_a_member_of_no_group_it_is_lent_to_opens_nothing() {
    let (pair, heads) = lending_heads().await;
    let a = *pair.a.endpoint().id().as_bytes();
    let shelf = Label::of(&a, "shelf");
    let holder = signing(9);
    let bridge = bridge_as(&holder, &pair.a, Alpn::Lend).await;
    let bos = Pass::sign(
        &signing(2),
        &actor_of(&holder),
        (shelf.clone(), vec![]),
        unix_now() + 3600,
    );
    let ask = || {
        let pass = format!("x-svrngs-pass: {}\r\n", bos.encode());
        let head = format!(
            "GET /lend/{shelf}/whoami HTTP/1.1\r\nHost: local\r\n{pass}Connection: close\r\n\r\n"
        );
        raw_to(bridge.local_addr(), head)
    };
    assert_eq!(
        ask().await.status,
        200,
        "lent to the house, the pass opens it"
    );
    groups_held(&pair, &["yard"]);
    let asked = heads.lock().unwrap().len();
    let reply = ask().await;
    assert_eq!(reply.status, 403, "{}", reply.text);
    assert_eq!(
        heads.lock().unwrap().len(),
        asked,
        "the service was asked again"
    );
    pair.close().await;
}

/// The mixed-version fence: a list asked with no scope marker, or an older
/// one, is refused, so an old consumer never reads a scoped list as the
/// whole of what is lent; the marker opens nothing by itself. Planted: the
/// lender answering an unmarked ask.
#[tokio::test]
async fn a_list_asked_without_this_builds_scope_marker_is_refused() {
    let (pair, _heads) = lending_heads().await;
    for marker in ["", "x-svrngs-version: 4\r\n"] {
        for route in ["/lends", svrngs_node::node::PEOPLE_LENDS_ROUTE] {
            let reply = pair.get(route, marker).await;
            assert_eq!(reply.status, 403, "{route} {marker:?}: {}", reply.text);
        }
    }
    groups_held(&pair, &["yard"]);
    assert_eq!(listed(&pair).await["lends"], serde_json::json!([]));
    pair.close().await;
}

/// A request through bo's bridge whose answer's head has come: the socket,
/// its body still to come.
async fn opened(pair: &Pair, head: &str) -> tokio::net::TcpStream {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut sock = tokio::net::TcpStream::connect(pair.bridge).await.unwrap();
    sock.write_all(format!("{head}Host: local\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let (mut seen, mut byte) = (Vec::new(), [0u8; 1]);
    while !seen.ends_with(b"\r\n\r\n") {
        assert_eq!(
            sock.read(&mut byte).await.unwrap(),
            1,
            "{:?}",
            String::from_utf8_lossy(&seen)
        );
        seen.push(byte[0]);
    }
    assert!(
        seen.starts_with(b"HTTP/1.1 20") || seen.starts_with(b"HTTP/1.1 101"),
        "{:?}",
        String::from_utf8_lossy(&seen)
    );
    sock
}

/// How long `sock` carries on before it ends, read as it comes, up to `within`.
async fn ends_within(
    sock: &mut tokio::net::TcpStream,
    within: std::time::Duration,
) -> Option<std::time::Duration> {
    use tokio::io::AsyncReadExt;
    let began = std::time::Instant::now();
    let mut buf = [0u8; 4096];
    let ended = async { while sock.read(&mut buf).await.is_ok_and(|n| n > 0) {} };
    tokio::time::timeout(within, ended)
        .await
        .ok()
        .map(|()| began.elapsed())
}

/// Withdrawal of what already streams (LENDING_SCOPE §"Existing streams
/// and app sessions"): bo streams the shelf and the radio, both lent to
/// the house, and holds a WebSocket to `live`. Once the shelf and `live`
/// are lent only to yard, the shelf's answer and the socket each end
/// within two seconds; the radio's stream, through the same bridge, still
/// carries bytes after, and a new request to it is answered. Planted: the
/// answer's body and the upgraded copy carried on to their end, unwatched.
#[tokio::test]
async fn a_stream_ends_within_two_seconds_of_its_group_taken_away_and_no_other() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let two = std::time::Duration::from_secs(2);
    let pair = Pair::new().await;
    let (stream, _) = super::origins::streaming_origin().await;
    let (live, _) = super::origins::upgrade_origin().await;
    for (name, port) in [
        ("shelf", stream.port()),
        ("radio", stream.port()),
        ("live", live.port()),
    ] {
        pair.lend(name, port, &[]);
    }
    groups_held(&pair, &[NS]);
    let mut shelf = opened(&pair, "GET /lend/shelf/stream HTTP/1.1\r\n").await;
    let mut radio = opened(&pair, "GET /lend/radio/stream HTTP/1.1\r\n").await;
    let upgrade = "GET /lend/live/socket HTTP/1.1\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n";
    let mut socket = opened(&pair, upgrade).await;
    socket.write_all(b"ping").await.unwrap();
    let mut echo = [0u8; 4];
    socket.read_exact(&mut echo).await.unwrap();

    to_groups(&pair, &["yard"]);
    let live = LendName::parse("live").unwrap();
    assert!(lend::set_audience(pair.dir.path(), &live, vec!["yard".into()]).unwrap());
    let cut = ends_within(&mut shelf, two).await;
    let closed = ends_within(&mut socket, two).await;
    eprintln!("the shelf's answer ended after {cut:?}, the socket after {closed:?}");
    assert!(
        cut.is_some(),
        "the shelf's answer still carries on after 2 s"
    );
    assert!(closed.is_some(), "the socket is still open after 2 s");
    let mut more = [0u8; 64];
    let carried = tokio::time::timeout(two, radio.read(&mut more)).await;
    assert!(
        matches!(carried, Ok(Ok(n)) if n > 0),
        "the radio stopped: {carried:?}"
    );
    assert_eq!(pair.get("/lend/radio/now", "").await.status, 200);
    pair.close().await;
}

/// The lender's own standing, asked at each request (LENDING_SCOPE §"One
/// policy": a chosen group in which both the lender and the asker stand):
/// the shelf lent only to bos, a group A holds that names bo and not ada,
/// is refused bo `closed`, and the service is not asked. Planted: the
/// audience asked alone, never whether the lender stands there.
#[tokio::test]
async fn a_group_the_lender_no_longer_stands_in_reaches_nothing() {
    let (pair, heads) = lending_heads().await;
    let bo = [(Person::parse("bo").unwrap(), vec![actor_of(&signing(2))])];
    pair.a
        .rail()
        .set_seed("bos", &Roster::new(bo.into_iter().collect()))
        .unwrap();
    to_groups(&pair, &["bos"]);
    let reply = pair.get("/lend/shelf/whoami", "").await;
    assert_eq!(reply.status, 403, "{}", reply.text);
    assert!(heads.lock().unwrap().is_empty(), "the service was asked");
    pair.close().await;
}

/// Joining a third group widens nothing (LENDING_SCOPE "Proof before
/// landing" 3): the shelf lent only to yard; A then holds club, ada and
/// bo's. The declaration's bytes are as they were, and bo, now in a third
/// group with the lender, still neither lists nor reaches it. Planted: a
/// lend reaching every group its lender stands in.
#[tokio::test]
async fn joining_a_third_group_leaves_the_declaration_and_reaches_nothing() {
    let (pair, heads) = lending_heads().await;
    groups_held(&pair, &["yard"]);
    let file = lend::lends_dir(pair.dir.path()).join("shelf.json");
    let before = std::fs::read(&file).unwrap();
    let club = [("ada", 1), ("bo", 2)]
        .map(|(p, k)| (Person::parse(p).unwrap(), vec![actor_of(&signing(k))]));
    pair.a
        .rail()
        .set_seed("club", &Roster::new(club.into_iter().collect()))
        .unwrap();
    assert_eq!(listed(&pair).await["lends"], serde_json::json!([]));
    let reply = pair.get("/lend/shelf/whoami", "").await;
    assert_eq!(reply.status, 403, "{}", reply.text);
    assert_eq!(
        std::fs::read(&file).unwrap(),
        before,
        "the declaration is unchanged"
    );
    assert!(heads.lock().unwrap().is_empty(), "the service was asked");
    pair.close().await;
}

/// A lend from before groups opens on the next start exactly as it did, to
/// the groups its computer stands in then, whichever way it was left: no
/// `groups` key (from an svrngs before 5) or `null` (stamped by the first
/// 5). Bo reaches both through the door. A group A holds but no longer
/// stands in is not named, nor is one it joins after, and a lend paused or
/// chosen on purpose is untouched. Planted: `stamp` writing `null` as
/// before, so nobody reaches either.
#[tokio::test]
async fn a_lend_from_before_groups_opens_as_it_did_and_can_then_be_pared_down() {
    let (origin, heads) = super::origins::head_origin().await;
    let port = origin.port();
    let (dir, db) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let rail = svrngs_runtime::RingRail::new(dir.path(), std::sync::Arc::new(signing(1)));
    let people = |list: &[(&str, u8)]| {
        Roster::new(
            list.iter()
                .map(|(p, k)| (Person::parse(p).unwrap(), vec![actor_of(&signing(*k))]))
                .collect(),
        )
    };
    let group = people(&[("ada", 1), ("bo", 2)]);
    for ns in [NS, "trip"] {
        rail.set_seed(ns, &group).unwrap();
    }
    rail.set_seed("gone", &people(&[("cy", 3)])).unwrap();
    let lends = lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let write = |name: &str, tail: &str| {
        let body = format!(r#"{{"port":{port},"kind":"k","refused":[]{tail}}}"#);
        std::fs::write(lends.join(format!("{name}.json")), body).unwrap();
    };
    write("old", "");
    write("stamped", r#","groups":null"#);
    write("paused", r#","groups":[]"#);
    write("chosen", r#","groups":["trip"]"#);

    let a = start(1, dir.path()).await;
    let audience = |name: &str| {
        let name = LendName::parse(name).unwrap();
        lend::read(dir.path(), &name).unwrap().unwrap().audience
    };
    let mut held = vec![NS.to_string(), "trip".to_string()];
    held.sort();
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
        lend::Audience::Selected(vec!["trip".to_string()])
    );

    let b = start(2, db.path()).await;
    let pair = Pair::dialling(a, b, dir, db).await;
    for name in ["old", "stamped"] {
        let reply = pair.get(&format!("/lend/{name}/whoami"), "").await;
        assert_eq!(reply.status, 200, "{name}: {}", reply.text);
        assert_eq!(header_values(&last(&heads), "x-svrngs-person"), vec!["bo"]);
    }
    let paused = pair.get("/lend/paused/whoami", "").await;
    assert_eq!(paused.status, 403, "{}", paused.text);

    pair.a.rail().set_seed("later", &group).unwrap();
    let old = audience_of(&pair, "old");
    assert!(!old.contains(&"later".to_string()), "{old:?}");
    pair.close().await;
}

fn audience_of(pair: &Pair, name: &str) -> Vec<String> {
    let name = LendName::parse(name).unwrap();
    let lend = lend::read(pair.dir.path(), &name).unwrap().unwrap();
    lend.audience.ids().unwrap().to_vec()
}
