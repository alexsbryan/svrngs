// SPDX-License-Identifier: AGPL-3.0-or-later
//! A person's whole sign-in: the front channel on the lend's origin
//! (SIGN_IN_WITH_SVRNGS §5 "The front channel", §8 1, 6, 7): the app's
//! server sends the browser to `/.svrngs/oidc/authorize` on the lend's
//! own address, the door answers a one-time code in a relative `Location`
//! back to the app's callback, and the code redeems at the provider's
//! token route for the person the door resolved. The service never sees
//! the ask. One crossing holds that the door's authorize path answers its
//! own asker and refuses what the lend would; each rule of the front
//! channel and the token route is judged on its own, in process
//! (`src/oidc/tests.rs`). Each step names the test it came from, retired
//! at its move (docs/seams.tsv).

use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde_json::{json, Value};
use svrngs_core::alpn::Alpn;
use svrngs_core::pass::{Label, Pass};
use svrngs_node::Visit;

use super::*;

const LEND: &str = "media";
const CLIENT: &str = "svrngs-media";
const SECRET: &str = "media-secret_2026.10";
const PATH: &str = "/signin-oidc";
const STATE: &str = "st-0123456789";
const NONCE: &str = "n-0S6_WzA2Mj";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
/// `S256` of [`VERIFIER`].
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

/// A's declaration of `name` on `port`, signing people in with svrngs as
/// `client` with `secret` (SIGN_IN_WITH_SVRNGS §5 "Turning it on").
fn signs_in(pair: &Pair, name: &str, port: u16, client: &str, secret: &str) {
    signs_in_with(pair, name, port, client, secret, json!({}));
}

/// [`signs_in`], the declaration's `extra` fields beside what it says.
fn signs_in_with(pair: &Pair, name: &str, port: u16, client: &str, secret: &str, extra: Value) {
    let dir = lend::lends_dir(pair.dir.path());
    std::fs::create_dir_all(&dir).unwrap();
    let mut body = json!({
        "port": port,
        "kind": "k",
        "groups": [NS],
        "sign_in_with_svrngs": { "client_id": client, "client_secret": secret },
    });
    let fields = body.as_object_mut().unwrap();
    fields.extend(extra.as_object().unwrap().clone());
    std::fs::write(dir.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// `u` percent-encoded enough for a query's value.
fn quoted(u: &str) -> String {
    u.replace(':', "%3A").replace('/', "%2F")
}

/// The authorize request's query for `port`'s client, with `extra` appended
/// as whole `&k=v` pairs.
fn ask(port: u16, client: &str, extra: &str) -> String {
    ask_to(&format!("http://localhost:{port}{PATH}"), client, extra)
}

/// The authorize request's query with `redirect` its `redirect_uri`.
fn ask_to(redirect: &str, client: &str, extra: &str) -> String {
    let redirect = quoted(redirect);
    format!(
        "response_type=code&client_id={client}&redirect_uri={redirect}\
         &scope=openid%20profile&state={STATE}\
         &code_challenge={CHALLENGE}&code_challenge_method=S256{extra}"
    )
}

/// `path` asked through `bridge` as the browser at `host`, with `headers` on
/// the request.
async fn get_at(bridge: SocketAddr, host: &str, path: &str, headers: &str) -> Reply {
    raw_to(
        bridge,
        format!("GET {path} HTTP/1.1\r\nHost: {host}\r\n{headers}Connection: close\r\n\r\n"),
    )
    .await
}

/// `path` asked through `bridge`, with `headers` on the request.
async fn get(bridge: SocketAddr, path: &str, headers: &str) -> Reply {
    get_at(bridge, "local", path, headers).await
}

/// The `Location` an answer carries, or none.
fn location(reply: &Reply) -> Option<String> {
    header_values(&reply.text, "location")
        .first()
        .map(|l| l.to_string())
}

/// The pairs in a query string, without percent-decoding: a code and a
/// state svrngs made are URL-safe.
fn pairs(query: &str) -> BTreeMap<String, String> {
    (query.split('&'))
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// The token route's answer to `form`, asked with `client`'s secret.
async fn token(
    node: &Node,
    lend: &str,
    client: &str,
    secret: &str,
    form: &[(&str, &str)],
) -> (u16, Value) {
    let answer = reqwest::Client::new()
        .post(format!("http://{}/oidc/{lend}/token", node.api_addr()))
        .basic_auth(client, Some(secret))
        .form(form)
        .send()
        .await
        .unwrap();
    let status = answer.status().as_u16();
    (status, answer.json().await.unwrap_or(Value::Null))
}

/// `token`'s claims, as a stock client reads them.
fn claims(token: &str) -> Value {
    let (input, _) = token.rsplit_once('.').unwrap();
    let (_, payload) = input.split_once('.').unwrap();
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap()
}

/// Planted and watched failing when each was its own test: a second
/// person's key or name in the ask (the door's `Asker` is the code's
/// person); a removed member answered a code; an absolute `Location`; the
/// front channel accepting only `http://localhost:<port>`; the door telling
/// a room's address https; the authorize path carried past a refused path
/// or an unsaid app; a guest's code naming the host.
#[tokio::test]
async fn a_members_authorize_answers_a_code_the_token_route_redeems() {
    let pair = Pair::new().await;
    let (origin, heads) = origins::head_origin().await;
    let port = origin.port();
    signs_in(&pair, LEND, port, CLIENT, SECRET);
    let bo = actor_of(&signing(2));
    let cy = actor_of(&signing(3));
    let redirect = format!("http://localhost:{port}{PATH}");
    let redeem = |code: String, redirect: String| {
        let a = &pair.a;
        async move {
            let form = [
                ("grant_type", "authorization_code"),
                ("code", code.as_str()),
                ("redirect_uri", redirect.as_str()),
                ("code_verifier", VERIFIER),
            ];
            token(a, LEND, CLIENT, SECRET, &form).await
        }
    };
    let code_of = |reply: &Reply| -> String {
        let loc = location(reply).expect("a Location");
        pairs(loc.split_once('?').expect("a query").1)
            .remove("code")
            .expect("a code")
    };

    // a_members_authorize_answers_a_code_the_token_route_redeems (oidc.rs:153-209):
    // bo's browser asks, with cy's key and name planted on the request.
    let reply = get(
        pair.bridge,
        &format!(
            "/lend/{LEND}/.svrngs/oidc/authorize?{}&sub=cy",
            ask(port, CLIENT, &format!("&nonce={NONCE}"))
        ),
        &format!("x-svrngs-key: {cy}\r\nx-svrngs-person: cy\r\n"),
    )
    .await;
    assert_eq!(reply.status, 302, "{}", reply.text);
    let loc = location(&reply).expect("a Location");
    assert!(
        loc.starts_with(PATH),
        "the redirect is not relative to the lend's origin: {loc}"
    );
    assert!(!loc.contains("http://"), "the redirect is absolute: {loc}");
    let (path, query) = loc.split_once('?').expect("a query on the callback");
    assert_eq!(path, PATH);
    assert_eq!(pairs(query).get("state").map(String::as_str), Some(STATE));
    let (status, answer) = redeem(code_of(&reply), redirect.clone()).await;
    assert_eq!(status, 200, "{answer:#}");
    let said = claims(answer["id_token"].as_str().expect("an id token"));
    assert_eq!(
        said["sub"], bo,
        "the code is not the door's person: {said:#}"
    );
    assert_ne!(said["sub"], cy, "the ask chose the person: {said:#}");
    assert_eq!(said["name"], "bo");
    assert_eq!(said["preferred_username"], format!("bo-{}", &bo[..16]));
    assert_eq!(said["groups"], json!(["member"]));
    assert_eq!(said["nonce"], NONCE, "the nonce is not echoed: {said:#}");

    // the_authorize_path_accepts_the_lends_own_origin_as_the_redirect (oidc.rs:519-566):
    // a computer's browser at <label>.localhost, a phone's at a page host, read as https;
    // and a_rooms_address_is_told_http_and_its_origin_passes_authorize (oidc.rs:675-688).
    let label = Label::of(pair.a.endpoint().id().as_bytes(), LEND);
    let api = pair.a.api_addr().port();
    for (browser, scheme) in [
        (format!("{label}.localhost:{api}"), "http"),
        (format!("{label}.svrngs.example"), "https"),
        ("192.168.1.20:7123".to_string(), "http"),
    ] {
        let at = format!("{scheme}://{browser}{PATH}");
        let path = format!(
            "/lend/{LEND}/.svrngs/oidc/authorize?{}",
            ask_to(&at, CLIENT, "")
        );
        let reply = get_at(pair.bridge, &browser, &path, "").await;
        assert_eq!(reply.status, 302, "on {browser}: {}", reply.text);
        let loc = location(&reply).expect("a Location");
        assert!(
            loc.starts_with(&format!("{PATH}?code=")),
            "on {browser}: {loc}"
        );
        let (status, answer) = redeem(code_of(&reply), at).await;
        assert_eq!(status, 200, "on {browser}: {answer:#}");
        assert_eq!(
            claims(answer["id_token"].as_str().unwrap())["sub"],
            bo,
            "on {browser}"
        );
    }

    // a_lend_that_refuses_or_asks_answers_the_authorize_path_no_code_of_its_own
    // (oidc.rs:347-417).
    let (refusing, asking) = ("refusing", "asking");
    signs_in_with(
        &pair,
        refusing,
        port,
        CLIENT,
        SECRET,
        json!({ "refused": ["/.svrngs"] }),
    );
    signs_in_with(
        &pair,
        asking,
        port,
        CLIENT,
        SECRET,
        json!({ "asks": "the app reads your films" }),
    );
    let a = *pair.a.endpoint().id().as_bytes();
    let (asks, night) = (Label::of(&a, asking), Label::of(&a, "night"));
    let passed = |app: &Label| {
        let pass = Pass::sign(
            &signing(2),
            &bo,
            (app.clone(), vec![asks.clone()]),
            unix_now() + 3600,
        );
        format!("x-svrngs-pass: {}\r\n", pass.encode())
    };
    let authorize = |lend: &str| {
        format!(
            "/lend/{lend}/.svrngs/oidc/authorize?{}",
            ask(port, CLIENT, "")
        )
    };
    for (row, lend, headers, refused) in [
        (
            "a path the lend refuses",
            refusing,
            String::new(),
            Some("path"),
        ),
        (
            "a member naming no app",
            asking,
            String::new(),
            Some("unasked"),
        ),
        (
            "another app, unsaid",
            asking,
            passed(&night),
            Some("unasked"),
        ),
        ("the lend's own page", asking, passed(&asks), None),
    ] {
        let reply = get(pair.bridge, &authorize(lend), &headers).await;
        match refused {
            Some(kind) => {
                assert_eq!(reply.status, 403, "{row}: {}", reply.text);
                assert!(location(&reply).is_none(), "{row}: {}", reply.text);
                assert_eq!(
                    header_values(&reply.text, "x-svrngs-refused"),
                    vec![kind],
                    "{row}"
                );
            }
            None => assert!(location(&reply).is_some(), "{row}: {}", reply.text),
        }
    }

    // a_guests_code_names_the_guest_never_the_host (oidc.rs:813-888): Sam,
    // ada's guest here, through the pass holders' protocol.
    let mut opens = BTreeMap::new();
    opens.insert(actor_of(&signing(1)), [LEND.to_string()].into());
    let act = RailAct::Visit(Visit {
        guest: Person::parse("Sam").unwrap(),
        key: actor_of(&signing(SAM)),
        opens,
        until: unix_now() + 24 * 3600,
    });
    pair.a
        .rail()
        .commit(NS, &svrngs_core::tests_support::Fixture { act })
        .unwrap()
        .unwrap();
    let holder = signing(HOLDER);
    let sams = Pass::sign(
        &signing(SAM),
        &actor_of(&holder),
        (label.clone(), vec![]),
        unix_now() + 3600,
    );
    let bridge = bridge_as(&holder, &pair.a, Alpn::Lend).await;
    let reply = raw_to(
        bridge.local_addr(),
        format!(
            "GET /lend/{label}/.svrngs/oidc/authorize?{} HTTP/1.1\r\nHost: local\r\n\
             x-svrngs-pass: {}\r\nConnection: close\r\n\r\n",
            ask(port, CLIENT, ""),
            sams.encode()
        ),
    )
    .await;
    assert_eq!(reply.status, 302, "{}", reply.text);
    let (status, answer) = redeem(code_of(&reply), redirect.clone()).await;
    assert_eq!(status, 200, "{answer:#}");
    let said = claims(answer["id_token"].as_str().expect("an id token"));
    let (sam, ada) = (actor_of(&signing(SAM)), actor_of(&signing(1)));
    assert_eq!(said["sub"], sam, "{said:#}");
    assert_eq!(said["name"], "Sam (ada's guest)");
    assert_eq!(
        said["preferred_username"],
        format!("Sam-{}.guest", &sam[..16])
    );
    assert_eq!(said["groups"], json!(["guest"]));
    assert!(
        !answer.to_string().contains(&ada),
        "the host's key reached the app"
    );

    // a_code_is_not_redeemed_once_its_group_is_no_longer_lent_to (oidc.rs:493-494):
    // no new code is minted once the lend leaves the house.
    let name = LendName::parse(LEND).unwrap();
    assert!(lend::set_audience(pair.dir.path(), &name, vec!["yard".into()]).unwrap());
    let path = format!(
        "/lend/{LEND}/.svrngs/oidc/authorize?{}",
        ask(port, CLIENT, "")
    );
    assert_eq!(get(pair.bridge, &path, "").await.status, 403);
    assert!(lend::set_audience(pair.dir.path(), &name, vec![NS.into()]).unwrap());

    // a_members_authorize_answers_a_code_the_token_route_redeems (oidc.rs:303-330):
    // bo removed, the door refuses the authorize address before a code.
    let removal = svrngs_node::Removal {
        person: Person::parse("bo").unwrap(),
        joined: None,
    };
    pair.a.rail().commit(NS, &removal).unwrap().unwrap();
    let gone = bridge_as(&signing(2), &pair.a, Alpn::Member).await;
    let refused = get(gone.local_addr(), &path, "").await;
    assert_eq!(refused.status, 403, "{}", refused.text);
    assert!(
        location(&refused).is_none(),
        "a removed member was answered a code: {}",
        refused.text
    );
    assert!(
        heads.lock().unwrap().is_empty(),
        "the service was asked: {:?}",
        heads.lock().unwrap()
    );
    pair.close().await;
}

/// Sam's visit key, which Sam's page makes fresh for this visit.
const SAM: u8 = 9;
/// The key a lend's origin on Sam's page dials with, holding Sam's pass.
const HOLDER: u8 = 10;
