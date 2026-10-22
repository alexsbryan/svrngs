//! A stock client's three asks of the provider's back channel
//! (SIGN_IN_WITH_SVRNGS §5 "The back channel", §5a "What the provider must
//! do"): discovery at the issuer's `/.well-known/openid-configuration`, the
//! keys, the token and userinfo, on the record API's listener, behind its
//! one loopback check and its page guard.

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_FIXED};
use serde_json::{json, Value};
use svrngs_core::token::Group;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

const LEND: &str = "media";
const PORT: u16 = 8096;
const CLIENT: &str = "svrngs-media";
const SECRET: &str = "media-secret_2026.10";
const PUBLIC: &str = "public";
const PUBLIC_CLIENT: &str = "svrngs-public";
const SUB: &str = "9f2c0a1b3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8";
const NAME: &str = "Ana";
const USER: &str = "Ana-9f2c0a1b3d4e5f60";
const REDIRECT: &str = "http://localhost:8096/signin-oidc";
const NONCE: &str = "n-0S6_WzA2Mj";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
/// `S256` of [`VERIFIER`]: the challenge a client sends.
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

async fn start(dir: &Path) -> Node {
    Node::start(
        dir.to_path_buf(),
        signing(1),
        &RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: true,
        },
        |_| Ok(None),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// `dir` lends `name` on `port`, with a sign-in-with-svrngs client `client`
/// and, when one is named, its secret.
fn signs_in(dir: &Path, name: &str, port: u16, client: &str, secret: Option<&str>) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let mut client = json!({ "client_id": client });
    if let Some(secret) = secret {
        client["client_secret"] = secret.into();
    }
    let body = json!({ "port": port, "kind": "media", "sign_in_with_svrngs": client });
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
}

fn http() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

async fn get(node: &Node, path: &str) -> (u16, Value) {
    get_url(&format!("http://{}{path}", node.api_addr())).await
}

/// `url` asked as the client asks discovery's own addresses.
async fn get_url(url: &str) -> (u16, Value) {
    let answer = tokio::time::timeout(Duration::from_secs(30), http().get(url).send())
        .await
        .expect("the ask finishes")
        .unwrap();
    let status = answer.status().as_u16();
    (status, answer.json().await.unwrap_or(Value::Null))
}

async fn get_with(node: &Node, path: &str, header: (&str, &str)) -> u16 {
    let answer = tokio::time::timeout(
        Duration::from_secs(30),
        http()
            .get(format!("http://{}{path}", node.api_addr()))
            .header(header.0, header.1)
            .send(),
    )
    .await
    .expect("the ask finishes")
    .unwrap();
    answer.status().as_u16()
}

/// The token route, asked as a stock client: `form` by
/// `application/x-www-form-urlencoded`, and Basic credentials when `basic`
/// is named.
async fn token(node: &Node, basic: Option<(&str, &str)>, form: &[(&str, &str)]) -> (u16, Value) {
    token_at(node, LEND, basic, form).await
}

/// The token route of the lend `lend`.
async fn token_at(
    node: &Node,
    lend: &str,
    basic: Option<(&str, &str)>,
    form: &[(&str, &str)],
) -> (u16, Value) {
    let mut request = http().post(format!("http://{}/oidc/{lend}/token", node.api_addr()));
    if let Some((id, secret)) = basic {
        request = request.basic_auth(id, Some(secret));
    }
    let answer = tokio::time::timeout(Duration::from_secs(30), request.form(form).send())
        .await
        .expect("the ask finishes")
        .unwrap();
    let status = answer.status().as_u16();
    (status, answer.json().await.unwrap_or(Value::Null))
}

/// The token route, asked with a Bearer access token (userinfo's own ask).
async fn userinfo(node: &Node, token: &str) -> (u16, Value) {
    let answer = tokio::time::timeout(
        Duration::from_secs(30),
        http()
            .get(format!("http://{}/oidc/{LEND}/userinfo", node.api_addr()))
            .bearer_auth(token)
            .send(),
    )
    .await
    .expect("the ask finishes")
    .unwrap();
    let status = answer.status().as_u16();
    (status, answer.json().await.unwrap_or(Value::Null))
}

fn decoded(part: &str) -> Value {
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(part).unwrap()).unwrap()
}

/// The token's payload, as a stock client reads it.
fn payload(token: &str) -> Value {
    let (input, _) = token.rsplit_once('.').unwrap();
    let (_, payload) = input.split_once('.').unwrap();
    decoded(payload)
}

/// `token`'s ES256 signature verifies over its signing input under the
/// JWKS' `x` and `y`.
fn verifies(jwks: &Value, token: &str) -> bool {
    let listed = &jwks["keys"][0];
    let part = |name: &str| {
        URL_SAFE_NO_PAD
            .decode(listed[name].as_str().unwrap())
            .unwrap()
    };
    let (x, y) = (part("x"), part("y"));
    let mut point = vec![0x04];
    point.extend_from_slice(&x);
    point.extend_from_slice(&y);
    let (input, signature) = token.rsplit_once('.').unwrap();
    let signature = URL_SAFE_NO_PAD.decode(signature).unwrap();
    UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, &point)
        .verify(input.as_bytes(), &signature)
        .is_ok()
}

/// Given a lend that signs people in with svrngs, when a stock client asks
/// discovery, the keys, the token and userinfo, then discovery names the
/// authorize address on the lend and the rest at the issuer, ES256 alone and
/// no PAR; the keys verify the ID token; the token exchange is the stock
/// one; and userinfo answers the eight claims. Planted: an issuer with a
/// trailing slash or a second string (the claims' `iss` compared byte for
/// byte); a page's request answered; a code used twice; a code for one
/// client redeemed by another; a verifier with the challenge not checked; a
/// nonce dropped; a claim missing from userinfo; RS256 or a PAR endpoint
/// listed.
#[tokio::test]
async fn discovery_the_keys_the_token_and_userinfo_as_a_stock_client_reads_them() {
    let dir = tempfile::tempdir().unwrap();
    signs_in(dir.path(), LEND, PORT, CLIENT, Some(SECRET));
    signs_in(dir.path(), PUBLIC, 8097, PUBLIC_CLIENT, None);
    let node = start(dir.path()).await;
    let api = node.api_addr().port();
    let issuer = format!("http://127.0.0.1:{api}/oidc/{LEND}");

    let (status, discovery) = get(
        &node,
        &format!("/oidc/{LEND}/.well-known/openid-configuration"),
    )
    .await;
    assert_eq!(status, 200, "discovery: {discovery:#}");
    assert_eq!(discovery["issuer"], issuer, "one issuer string, exact");
    assert!(
        !discovery["issuer"].as_str().unwrap().ends_with('/'),
        "the issuer has a trailing slash: {discovery:#}"
    );
    assert_eq!(
        discovery["authorization_endpoint"], "http://localhost:8096/.svrngs/oidc/authorize",
        "the authorize address is the lend's own"
    );
    assert_eq!(discovery["token_endpoint"], format!("{issuer}/token"));
    assert_eq!(discovery["userinfo_endpoint"], format!("{issuer}/userinfo"));
    assert_eq!(discovery["jwks_uri"], format!("{issuer}/keys"));
    assert_eq!(
        discovery["id_token_signing_alg_values_supported"],
        json!(["ES256"]),
        "ES256 alone: {discovery:#}"
    );
    assert!(
        !discovery.to_string().contains("RS256"),
        "discovery lists RS256: {discovery:#}"
    );
    assert!(
        !discovery
            .to_string()
            .to_lowercase()
            .contains("pushed_authorization"),
        "discovery lists a PAR endpoint: {discovery:#}"
    );
    assert_eq!(
        discovery["code_challenge_methods_supported"],
        json!(["S256"]),
        "PKCE S256: {discovery:#}"
    );
    let methods = discovery["token_endpoint_auth_methods_supported"]
        .as_array()
        .unwrap();
    for method in ["client_secret_basic", "client_secret_post", "none"] {
        assert!(
            methods.iter().any(|m| m == method),
            "discovery does not list {method}: {discovery:#}"
        );
    }

    let (status, jwks) = get_url(discovery["jwks_uri"].as_str().unwrap()).await;
    assert_eq!(status, 200, "the keys: {jwks:#}");
    assert_eq!(jwks["keys"].as_array().unwrap().len(), 1, "{jwks:#}");
    assert_eq!(jwks["keys"][0]["kty"], "EC");
    assert_eq!(jwks["keys"][0]["crv"], "P-256");

    // The stock exchange: a code with an S256 challenge and a nonce, redeemed
    // by the client that asked for it, with its verifier.
    let code = node.oidc_code(
        LEND,
        CLIENT,
        SUB,
        NAME,
        USER,
        Group::Member,
        REDIRECT,
        Some(CHALLENGE),
        Some(NONCE),
    );
    let (status, answer) = token(
        &node,
        Some((CLIENT, SECRET)),
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", REDIRECT),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(status, 200, "the token exchange: {answer:#}");
    assert_eq!(answer["token_type"], "Bearer");
    let id_token = answer["id_token"].as_str().expect("an id token");
    let access = answer["access_token"].as_str().expect("an access token");
    assert!(
        verifies(&jwks, id_token),
        "the JWKS does not verify the ID token"
    );
    let claims = payload(id_token);
    assert_eq!(
        claims["iss"], discovery["issuer"],
        "the ID token's iss is discovery's, byte for byte"
    );
    assert_eq!(claims["aud"], CLIENT);
    assert_eq!(claims["sub"], SUB);
    assert_eq!(claims["name"], NAME);
    assert_eq!(claims["preferred_username"], USER);
    assert_eq!(claims["groups"], json!(["member"]));
    assert_eq!(claims["nonce"], NONCE, "the nonce is not echoed");
    assert!(claims["exp"].as_i64().unwrap() > claims["iat"].as_i64().unwrap());

    let (status, info) = userinfo(&node, access).await;
    assert_eq!(status, 200, "userinfo: {info:#}");
    for claim in [
        "sub",
        "iss",
        "aud",
        "exp",
        "iat",
        "name",
        "preferred_username",
        "groups",
    ] {
        assert_eq!(
            info[claim], claims[claim],
            "userinfo is missing {claim}: {info:#}"
        );
    }
    assert!(
        info.get("nonce").is_none(),
        "userinfo carries the nonce: {info:#}"
    );

    // A code is used once: the very same code again is refused.
    let (status, again) = token(
        &node,
        Some((CLIENT, SECRET)),
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", REDIRECT),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(status, 400, "a code was redeemed twice: {again:#}");
    assert_eq!(again["error"], "invalid_grant", "{again:#}");

    // A code minted for this lend's client is not another client's.
    let foreign = node.oidc_code(
        LEND,
        "svrngs-another",
        SUB,
        NAME,
        USER,
        Group::Member,
        REDIRECT,
        None,
        None,
    );
    let (status, answer) = token(
        &node,
        Some((CLIENT, SECRET)),
        &[
            ("grant_type", "authorization_code"),
            ("code", &foreign),
            ("redirect_uri", REDIRECT),
        ],
    )
    .await;
    assert_eq!(status, 400, "another client redeemed the code: {answer:#}");

    // A bound challenge is checked: the wrong verifier is refused.
    let bound = node.oidc_code(
        LEND,
        CLIENT,
        SUB,
        NAME,
        USER,
        Group::Member,
        REDIRECT,
        Some(CHALLENGE),
        None,
    );
    let (status, answer) = token(
        &node,
        Some((CLIENT, SECRET)),
        &[
            ("grant_type", "authorization_code"),
            ("code", &bound),
            ("redirect_uri", REDIRECT),
            (
                "code_verifier",
                "the-wrong-verifier-at-least-43-characters-long",
            ),
        ],
    )
    .await;
    assert_eq!(status, 400, "a wrong verifier passed: {answer:#}");

    // A nonce is echoed when sent, and omitted otherwise.
    let plain = node.oidc_code(
        LEND,
        CLIENT,
        SUB,
        NAME,
        USER,
        Group::Member,
        REDIRECT,
        None,
        None,
    );
    let (status, answer) = token(
        &node,
        Some((CLIENT, SECRET)),
        &[
            ("grant_type", "authorization_code"),
            ("code", &plain),
            ("redirect_uri", REDIRECT),
        ],
    )
    .await;
    assert_eq!(status, 200, "{answer:#}");
    let claims = payload(answer["id_token"].as_str().unwrap());
    assert!(
        claims.get("nonce").is_none(),
        "a nonce nobody sent is echoed: {claims:#}"
    );

    // `client_secret_post` is a stock method too.
    let posted = node.oidc_code(
        LEND,
        CLIENT,
        SUB,
        NAME,
        USER,
        Group::Member,
        REDIRECT,
        None,
        None,
    );
    let (status, answer) = token(
        &node,
        None,
        &[
            ("grant_type", "authorization_code"),
            ("code", &posted),
            ("redirect_uri", REDIRECT),
            ("client_id", CLIENT),
            ("client_secret", SECRET),
        ],
    )
    .await;
    assert_eq!(status, 200, "client_secret_post: {answer:#}");

    // A public client asks with no secret.
    let public_code = node.oidc_code(
        PUBLIC,
        PUBLIC_CLIENT,
        SUB,
        NAME,
        USER,
        Group::Member,
        REDIRECT,
        None,
        None,
    );
    let (status, answer) = token_at(
        &node,
        PUBLIC,
        None,
        &[
            ("grant_type", "authorization_code"),
            ("code", &public_code),
            ("redirect_uri", REDIRECT),
            ("client_id", PUBLIC_CLIENT),
        ],
    )
    .await;
    assert_eq!(status, 200, "a public client: {answer:#}");

    // A page in a browser is refused, as every route on this listener is.
    let status = get_with(
        &node,
        &format!("/oidc/{LEND}/.well-known/openid-configuration"),
        ("origin", "http://elsewhere.example"),
    )
    .await;
    assert_eq!(status, 403, "a page's request was answered");

    // And a caller off this computer is refused before any route runs.
    {
        use tower::ServiceExt;
        let port = node.api_addr().port();
        let request =
            axum::http::Request::get(format!("/oidc/{LEND}/.well-known/openid-configuration"))
                .header("host", format!("127.0.0.1:{port}"))
                .body(axum::body::Body::empty())
                .unwrap();
        let mut request = request;
        let caller: SocketAddr = "10.0.0.9:50000".parse().unwrap();
        request
            .extensions_mut()
            .insert(axum::extract::ConnectInfo(caller));
        let answer =
            tokio::time::timeout(Duration::from_secs(30), node.api_routes().oneshot(request))
                .await
                .expect("the ask finishes")
                .unwrap();
        assert_eq!(
            answer.status().as_u16(),
            403,
            "a caller off this computer was answered"
        );
    }

    node.endpoint().close().await;
}
