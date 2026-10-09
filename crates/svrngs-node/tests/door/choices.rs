// SPDX-License-Identifier: AGPL-3.0-or-later
//! A person's yes and stop, kept and decided at the lend's door (sup-2;
//! svrn-docs `rings/current/SCOPED_USE_MADE_PRODUCTION.md` §4): each row of
//! its table, and each route, through the door between two nodes.

use super::*;
use svrngs_core::alpn::Alpn;
use svrngs_core::pass::{Label, Pass};
use svrngs_node::iroh::{
    ring_crypto_provider, EndpointAddr, EndpointBuilder, HttpBridge, SecretKey,
};

const WORDS: &str = "the app receives ranked suggestions; your history stays where it is";

fn kind(reply: &Reply) -> Option<String> {
    header_values(&reply.text, "x-svrngs-refused")
        .first()
        .map(|k| k.to_string())
}

async fn post(addr: SocketAddr, path: &str, headers: &str, body: &str) -> Reply {
    raw_to(
        addr,
        format!(
            "POST {path} HTTP/1.1\r\nHost: local\r\n{headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
    .await
}

/// **Each row of the table, and each route.** A lends `taste`, asking in
/// its lender's words, and `shelf`, asking nobody; bo is B's person. As a
/// member with no pass, no app is named: `shelf` carried, `taste` refused
/// `unasked`. With B's own pass for an app, `night`: `unasked` before any
/// yes; bo's yes by `POST /choices/taste` lets it in, its service told the
/// app; a pass's yes, on the members' protocol or the holders', is refused
/// and nothing kept; the pass's stop, on either, refuses the next request
/// `stopped`, and `GET /choices/taste` answers bo's stop. `taste`'s own
/// pages (a pass whose app is `taste`) are carried whatever bo said. The
/// lists carry its words. `unshare` takes the choices with the lend, so
/// declared again it asks again. Planted, each watched failing: a stop not
/// kept; a pass's yes accepted.
#[tokio::test]
async fn a_lend_that_asks_lets_in_only_the_app_its_person_said_yes_to() {
    let (pair, heads) = lending_heads().await;
    let (at, taste_heads) = origins::head_origin().await;
    let declare = || {
        let body =
            serde_json::json!({ "port": at.port(), "kind": "taste", "limit": 3, "asks": WORDS });
        let dir = lend::lends_dir(pair.dir.path());
        std::fs::write(dir.join("taste.json"), body.to_string()).unwrap();
    };
    declare();
    let a = *pair.a.endpoint().id().as_bytes();
    let (taste, night) = (Label::of(&a, "taste"), Label::of(&a, "night"));
    let bo = actor_of(&signing(2));
    let own = |app: &Label| {
        let pass = Pass::sign(
            &signing(2),
            &bo,
            (app.clone(), vec![taste.clone()]),
            unix_now() + 300,
        );
        format!("x-svrngs-pass: {}\r\n", pass.encode())
    };

    let shelf = pair.get("/lend/shelf/x", "").await;
    assert_eq!(
        (shelf.status, kind(&shelf)),
        (200, None),
        "a lend that asks nobody: {}",
        shelf.text
    );
    assert!(header_values(&last(&heads), "x-svrngs-app").is_empty());
    let none = pair.get("/lend/taste/x", "").await;
    assert_eq!(
        (none.status, kind(&none)),
        (403, Some("unasked".into())),
        "no app named: {}",
        none.text
    );
    let unasked = pair.get("/lend/taste/x", &own(&night)).await;
    assert_eq!(
        (unasked.status, kind(&unasked)),
        (403, Some("unasked".into())),
        "nothing said: {}",
        unasked.text
    );
    assert!(
        taste_heads.lock().unwrap().is_empty(),
        "no refused request reached taste"
    );

    let yes = format!(r#"{{"app":"{night}","said":"yes"}}"#);
    let said = post(pair.bridge, "/choices/taste", "", &yes).await;
    assert_eq!(said.status, 200, "bo's yes: {}", said.text);
    let carried = pair.get("/lend/taste/x", &own(&night)).await;
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

    let holder = signing(9);
    let dialling = EndpointBuilder::empty()
        .crypto_provider(ring_crypto_provider())
        .secret_key(SecretKey::from_bytes(&holder.to_bytes()))
        .bind()
        .await
        .unwrap();
    let mut target = EndpointAddr::new(pair.a.endpoint().id());
    for sock in loopback_sockets(pair.a.endpoint()) {
        target = target.with_ip_addr(sock);
    }
    let holders = HttpBridge::spawn_preferring(dialling, target, Alpn::Lend.bytes(), None)
        .await
        .unwrap();
    let phones = Pass::sign(
        &signing(2),
        &actor_of(&holder),
        (night.clone(), vec![taste.clone()]),
        unix_now() + 300,
    );
    let phones = format!("x-svrngs-pass: {}\r\n", phones.encode());

    for (route, at, pass) in [
        ("members'", pair.bridge, own(&night)),
        ("holders'", holders.local_addr(), phones.clone()),
    ] {
        let refused = post(at, &format!("/choices/{taste}"), &pass, r#"{"said":"yes"}"#).await;
        assert_eq!(
            (refused.status, kind(&refused)),
            (403, Some("closed".into())),
            "{route} pass's yes: {}",
            refused.text
        );
        let refused = post(at, &format!("/choices/{taste}"), &pass, &yes).await;
        assert_eq!(
            refused.status, 403,
            "{route} pass's yes, naming an app: {}",
            refused.text
        );
    }
    let still = pair.get("/lend/taste/x", &own(&night)).await;
    assert_eq!(
        still.status, 200,
        "no pass's yes or stop was kept: {}",
        still.text
    );

    for (route, at, pass) in [
        ("members'", pair.bridge, own(&night)),
        ("holders'", holders.local_addr(), phones),
    ] {
        let said = post(pair.bridge, "/choices/taste", "", &yes).await;
        assert_eq!(said.status, 200, "bo's yes again: {}", said.text);
        let stopped = post(
            at,
            &format!("/choices/{taste}"),
            &pass,
            r#"{"said":"stop"}"#,
        )
        .await;
        assert_eq!(stopped.status, 200, "{route} pass's stop: {}", stopped.text);
        let after = pair.get("/lend/taste/x", &own(&night)).await;
        assert_eq!(
            (after.status, kind(&after)),
            (403, Some("stopped".into())),
            "{route} after the stop: {}",
            after.text
        );
    }
    let mine = pair.get("/choices/taste", "").await;
    assert_eq!(mine.status, 200, "{}", mine.text);
    let (_, body) = mine.text.split_once("\r\n\r\n").unwrap();
    let mine: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(mine["choices"][night.as_str()]["said"], "stop", "{mine}");

    let pages = pair.get("/lend/taste/x", &own(&taste)).await;
    assert_eq!(
        (pages.status, kind(&pages)),
        (200, None),
        "taste's own pages: {}",
        pages.text
    );

    let lends = pair.get("/lends", "").await;
    assert!(
        lends.text.contains(&format!(r#""asks":"{WORDS}""#)),
        "{}",
        lends.text
    );

    lend::withdraw(pair.dir.path(), &LendName::parse("taste").unwrap()).unwrap();
    assert!(!lend::choices_path(pair.dir.path(), &LendName::parse("taste").unwrap()).exists());
    declare();
    let again = pair.get("/lend/taste/x", &own(&night)).await;
    assert_eq!(
        (again.status, kind(&again)),
        (403, Some("unasked".into())),
        "shared again: {}",
        again.text
    );
    pair.close().await;
}
