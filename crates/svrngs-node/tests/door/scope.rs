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
use super::{header_values, last, lending_heads, raw_to, signing, unix_now, Pair, NS};

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
    assert_eq!(lends["lends"][0]["groups"], serde_json::json!([NS]), "{lends}");
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
    assert!(!pages.contains("trip") && !pages.contains("yard"), "{pages}");
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
