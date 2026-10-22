// SPDX-License-Identifier: AGPL-3.0-or-later
//! A person's whole sign-in: the front channel on the lend's origin
//! (SIGN_IN_WITH_SVRNGS §5 "The front channel", §8 1, 6, 7): the app's
//! server sends the browser to `/.svrngs/oidc/authorize` on the lend's
//! own address, the door answers a one-time code in a relative `Location`
//! back to the app's callback, and the code redeems at the provider's
//! token route for the person the door resolved. The service never sees
//! the ask.

use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde_json::{json, Value};
use svrngs_core::alpn::Alpn;
use svrngs_core::pass::{Label, Pass};
use svrngs_node::{OpId, Visit};

use super::*;

const LEND: &str = "media";
const OTHER: &str = "other";
const CLIENT: &str = "svrngs-media";
const SECRET: &str = "media-secret_2026.10";
const OTHER_CLIENT: &str = "svrngs-other";
const OTHER_SECRET: &str = "other-secret_2026.10";
const PATH: &str = "/signin-oidc";
const STATE: &str = "st-0123456789";
const NONCE: &str = "n-0S6_WzA2Mj";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
/// `S256` of [`VERIFIER`].
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

/// A's declaration of `name` on `port`, signing people in with svrngs as
/// `client` with `secret` (SIGN_IN_WITH_SVRNGS §5 "Turning it on").
fn signs_in(pair: &Pair, name: &str, port: u16, client: &str, secret: &str) {
    let dir = lend::lends_dir(pair.dir.path());
    std::fs::create_dir_all(&dir).unwrap();
    let body = json!({
        "port": port,
        "kind": "k",
        "sign_in_with_svrngs": { "client_id": client, "client_secret": secret },
    });
    std::fs::write(dir.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// `u` percent-encoded enough for a query's value.
fn quoted(u: &str) -> String {
    u.replace(':', "%3A").replace('/', "%2F")
}

/// The authorize request's query for `port`'s client, with `extra` appended
/// as whole `&k=v` pairs.
fn ask(port: u16, client: &str, extra: &str) -> String {
    let redirect = quoted(&format!("http://localhost:{port}{PATH}"));
    format!(
        "response_type=code&client_id={client}&redirect_uri={redirect}\
         &scope=openid%20profile&state={STATE}\
         &code_challenge={CHALLENGE}&code_challenge_method=S256{extra}"
    )
}

/// `path` asked through `bridge`, with `headers` on the request.
async fn get(bridge: SocketAddr, path: &str, headers: &str) -> Reply {
    raw_to(
        bridge,
        format!("GET {path} HTTP/1.1\r\nHost: local\r\n{headers}Connection: close\r\n\r\n"),
    )
    .await
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

/// Given a lend that signs people in with svrngs, when a member's browser
/// reaches its authorize address through the door, then the door answers a
/// one-time code in a relative `Location` back to the app's callback, and
/// the code redeems at the token route for that person's claims; the
/// service is never asked. Planted and watched failing: a second person's
/// key or name in the ask (the door's `Asker` is the code's person); a code
/// minted for this lend presented at another's token route; a removed
/// member answered a code; an absolute `Location`; a redirect other than
/// the one the code was minted for.
#[tokio::test]
async fn a_members_authorize_answers_a_code_the_token_route_redeems() {
    let pair = Pair::new().await;
    let (origin, heads) = origins::head_origin().await;
    let port = origin.port();
    signs_in(&pair, LEND, port, CLIENT, SECRET);
    signs_in(&pair, OTHER, port, OTHER_CLIENT, OTHER_SECRET);
    let bo = actor_of(&signing(2));
    let cy = actor_of(&signing(3));
    let redirect = format!("http://localhost:{port}{PATH}");

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
    let said = pairs(query);
    assert_eq!(said.get("state").map(String::as_str), Some(STATE));
    let code = said.get("code").expect("a code").clone();
    assert!(!code.is_empty());
    assert!(
        heads.lock().unwrap().is_empty(),
        "the authorize ask reached the app's service: {:?}",
        heads.lock().unwrap()
    );

    // The code redeems, for bo, with bo's PKCE verifier; never for cy.
    let (status, answer) = token(
        &pair.a,
        LEND,
        CLIENT,
        SECRET,
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(status, 200, "{answer:#}");
    let claims = claims(answer["id_token"].as_str().expect("an id token"));
    assert_eq!(
        claims["sub"], bo,
        "the code is not the door's person: {claims:#}"
    );
    assert_ne!(claims["sub"], cy, "the ask chose the person: {claims:#}");
    assert_eq!(claims["name"], "bo");
    assert_eq!(claims["preferred_username"], format!("bo-{}", &bo[..16]));
    assert_eq!(claims["groups"], json!(["member"]));
    assert_eq!(
        claims["nonce"], NONCE,
        "the nonce is not echoed: {claims:#}"
    );

    // A fresh code, presented at another lend's token route, is refused.
    let fresh = |reply: &Reply| -> String {
        pairs(
            location(reply)
                .expect("a Location")
                .split_once('?')
                .unwrap()
                .1,
        )
        .remove("code")
        .expect("a code")
    };
    let reply = get(
        pair.bridge,
        &format!(
            "/lend/{LEND}/.svrngs/oidc/authorize?{}",
            ask(port, CLIENT, "")
        ),
        "",
    )
    .await;
    let (status, answer) = token(
        &pair.a,
        OTHER,
        OTHER_CLIENT,
        OTHER_SECRET,
        &[
            ("grant_type", "authorization_code"),
            ("code", &fresh(&reply)),
            ("redirect_uri", &redirect),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(status, 400, "another lend redeemed the code: {answer:#}");

    // Another login attempt's verifier does not answer the code's challenge.
    let reply = get(
        pair.bridge,
        &format!(
            "/lend/{LEND}/.svrngs/oidc/authorize?{}",
            ask(port, CLIENT, "")
        ),
        "",
    )
    .await;
    let (status, answer) = token(
        &pair.a,
        LEND,
        CLIENT,
        SECRET,
        &[
            ("grant_type", "authorization_code"),
            ("code", &fresh(&reply)),
            ("redirect_uri", &redirect),
            (
                "code_verifier",
                "another-login-attempt-s-verifier-43-characters",
            ),
        ],
    )
    .await;
    assert_eq!(status, 400, "another login redeemed the code: {answer:#}");

    // A fourth code, redeemed with a redirect it was not minted for.
    let reply = get(
        pair.bridge,
        &format!(
            "/lend/{LEND}/.svrngs/oidc/authorize?{}",
            ask(port, CLIENT, "")
        ),
        "",
    )
    .await;
    let (status, answer) = token(
        &pair.a,
        LEND,
        CLIENT,
        SECRET,
        &[
            ("grant_type", "authorization_code"),
            ("code", &fresh(&reply)),
            ("redirect_uri", "http://localhost:8096/elsewhere"),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(
        status, 400,
        "a redirect other than the code's was accepted: {answer:#}"
    );

    // bo is removed, and the door refuses the authorize address before a
    // code, without asking the service.
    let removal = svrngs_node::Removal {
        person: Person::parse("bo").unwrap(),
        joined: None,
    };
    pair.a.rail().commit(NS, &removal).unwrap().unwrap();
    let gone = bridge_as(&signing(2), &pair.a, Alpn::Member).await;
    let refused = get(
        gone.local_addr(),
        &format!(
            "/lend/{LEND}/.svrngs/oidc/authorize?{}",
            ask(port, CLIENT, "")
        ),
        "",
    )
    .await;
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

/// Kim's house: Alex keeps it, Kim's computer the lender, Bo's another
/// member's, each holding the group's seed.
struct House {
    kim: Node,
    bo: Node,
    dirs: [tempfile::TempDir; 2],
}

impl House {
    async fn new() -> Self {
        let dirs = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
        let kim = start(1, dirs[0].path()).await;
        let bo = start(2, dirs[1].path()).await;
        let people = [("Alex", 7), ("Kim", 1), ("Bo", 2)];
        let seed = Roster::new(
            (people.into_iter())
                .map(|(p, k)| (Person::parse(p).unwrap(), vec![actor_of(&signing(k))]))
                .collect(),
        );
        for node in [&kim, &bo] {
            node.rail().set_seed(NS, &seed).unwrap();
        }
        House { kim, bo, dirs }
    }

    /// Kim has Sam over until `until`, opening Kim's `lend`.
    fn visit(&self, until: i64, lend: &str) -> OpId {
        let mut opens = std::collections::BTreeMap::new();
        opens.insert(
            actor_of(self.kim.rail().signer()),
            [lend.to_string()].into(),
        );
        let act = RailAct::Visit(Visit {
            guest: Person::parse("Sam").unwrap(),
            key: actor_of(&signing(SAM)),
            opens,
            until,
        });
        let fixture = svrngs_core::tests_support::Fixture {
            act,
            on_behalf_of: None,
        };
        let recorded = self.kim.rail().commit(NS, &fixture).unwrap().unwrap();
        self.bo.rail().ingest_all(NS, recorded.lines()).unwrap();
        recorded.lines()[0].id.clone()
    }
}

/// Given Kim's lend signing people in with svrngs and Sam over as Kim's
/// guest, when Sam's browser reaches the authorize address with the visit's
/// pass, then the code redeems for the guest — the visit's key, Sam's
/// account and name — never for Kim or Kim's key.
#[tokio::test]
async fn a_guests_code_names_the_guest_never_the_host() {
    let house = House::new().await;
    let (origin, heads) = origins::head_origin().await;
    let port = origin.port();
    let dir = lend::lends_dir(house.dirs[0].path());
    std::fs::create_dir_all(&dir).unwrap();
    let body = json!({
        "port": port,
        "kind": "k",
        "sign_in_with_svrngs": { "client_id": CLIENT, "client_secret": SECRET },
    });
    std::fs::write(dir.join(format!("{LEND}.json")), body.to_string()).unwrap();
    house.visit(unix_now() + 24 * 3600, LEND);

    let label = Label::of(house.kim.endpoint().id().as_bytes(), LEND);
    let holder = signing(HOLDER);
    let pass = Pass::sign(
        &signing(SAM),
        &actor_of(&holder),
        (label.clone(), vec![]),
        unix_now() + 3600,
    );
    let bridge = bridge_as(&holder, &house.kim, Alpn::Lend).await;
    let reply = raw_to(
        bridge.local_addr(),
        format!(
            "GET /lend/{label}/.svrngs/oidc/authorize?{} HTTP/1.1\r\nHost: local\r\n\
             x-svrngs-pass: {}\r\nConnection: close\r\n\r\n",
            ask(port, CLIENT, ""),
            pass.encode()
        ),
    )
    .await;
    assert_eq!(reply.status, 302, "{}", reply.text);
    let loc = location(&reply).expect("a Location");
    let code = pairs(loc.split_once('?').unwrap().1)
        .remove("code")
        .expect("a code");
    assert!(
        heads.lock().unwrap().is_empty(),
        "the service was asked: {:?}",
        heads.lock().unwrap()
    );

    let redirect = format!("http://localhost:{port}{PATH}");
    let (status, answer) = token(
        &house.kim,
        LEND,
        CLIENT,
        SECRET,
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(status, 200, "{answer:#}");
    let claims = claims(answer["id_token"].as_str().expect("an id token"));
    let sam = actor_of(&signing(SAM));
    let kim = actor_of(&signing(1));
    assert_eq!(claims["sub"], sam, "{claims:#}");
    assert_ne!(claims["sub"], kim, "the guest's code names Kim: {claims:#}");
    assert_eq!(claims["name"], "Sam (Kim's guest)");
    assert_eq!(
        claims["preferred_username"],
        format!("Sam-{}.guest", &sam[..16])
    );
    assert_eq!(claims["groups"], json!(["guest"]));
    assert!(
        !answer.to_string().contains(&kim),
        "Kim's key reached the app"
    );

    house.kim.endpoint().close().await;
    house.bo.endpoint().close().await;
}
