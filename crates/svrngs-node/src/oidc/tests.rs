// SPDX-License-Identifier: AGPL-3.0-or-later
//! The lender's key, its JWKS and the tokens it signs (SIGN_IN_WITH_SVRNGS
//! §5, §8 3): the key is owner-only and survives a restart; a token verifies
//! only under the key that signed it; the JWKS is that key.
use super::*;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_FIXED};

const SUB: &str = "9f2c0a1b3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8";
const ISSUER: &str = "http://127.0.0.1:51234/oidc/jellyfin";

fn claims() -> Claims {
    let who = svrngs_core::token::Subject {
        sub: SUB.into(),
        name: "Ana".into(),
        username: "Ana-9f2c0a1b3d4e5f60".into(),
        group: svrngs_core::token::Group::Member,
    };
    Claims::new(ISSUER, "svrngs-jellyfin", who, 1_767_000_000, 600)
}

/// `token`'s signature verifies over its signing input under `point`.
fn verifies(point: &[u8], token: &str) -> bool {
    let (input, signature) = token.rsplit_once('.').expect("three parts");
    let signature = URL_SAFE_NO_PAD.decode(signature).expect("base64url");
    UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, point)
        .verify(input.as_bytes(), &signature)
        .is_ok()
}

/// Given a lender's computer with no key, when it loads one, then the key is
/// a file only its user can read, and the same key comes back on restart.
#[cfg(unix)]
#[test]
fn the_key_is_owner_only_and_survives_a_restart() {
    let at = tempfile::tempdir().unwrap();
    let first = load_or_create(at.path()).unwrap();
    let path = key_dir(at.path()).join(KEY_FILE);
    let mode = |p: &Path| {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    };
    assert_eq!(mode(&path), 0o600, "the key is another user's to read");
    assert_eq!(mode(&key_dir(at.path())), 0o700);
    let second = load_or_create(at.path()).unwrap();
    assert_eq!(first.point, second.point, "the same key after a restart");
    assert_eq!(first.kid, second.kid);
}

/// A token is signed by this key and by no other: given two lenders'
/// computers, when each signs the same input, then each token verifies only
/// under its own point.
#[test]
fn a_token_signed_by_another_key_does_not_verify() {
    let ours = load_or_create(tempfile::tempdir().unwrap().path()).unwrap();
    let theirs = load_or_create(tempfile::tempdir().unwrap().path()).unwrap();
    let token = ours.token(claims()).unwrap();
    assert!(verifies(&ours.point, &token));
    assert!(!verifies(&theirs.point, &token), "another key verified it");
    let their_token = theirs.token(claims()).unwrap();
    assert!(!verifies(&ours.point, &their_token));
}

/// The JWKS is the key that signs: given a token, when its `kid` and its
/// `x`/`y` are read from the JWKS, then they name the signing key, and the
/// point they rebuild verifies the token it lists.
#[test]
fn the_jwks_key_verifies_the_token_it_lists() {
    let key = load_or_create(tempfile::tempdir().unwrap().path()).unwrap();
    let token = key.token(claims()).unwrap();
    let jwks: serde_json::Value = serde_json::from_str(&key.jwks()).unwrap();
    let listed = &jwks["keys"][0];
    assert_eq!(listed["kid"], key.kid);
    assert_eq!(listed["kty"], "EC");
    assert_eq!(listed["crv"], "P-256");
    let part = |name: &str| {
        URL_SAFE_NO_PAD
            .decode(listed[name].as_str().unwrap())
            .unwrap()
    };
    let (x, y) = (part("x"), part("y"));
    assert_eq!((x.len(), y.len()), (32, 32));
    let mut point = vec![0x04];
    point.extend_from_slice(&x);
    point.extend_from_slice(&y);
    assert_eq!(
        point, key.point,
        "the JWKS is another key than the signer's"
    );
    assert!(
        verifies(&point, &token),
        "the JWKS' key does not verify its token"
    );

    let (header, _) = token.split_once('.').unwrap();
    let header: serde_json::Value =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(header).unwrap()).unwrap();
    assert_eq!(header["kid"], key.kid, "the header names another kid");
}

/// One person, one account: the token's `sub` is the door's person key, not
/// the device's (SIGN_IN_WITH_SVRNGS §5; ONE_PERSON_MANY_DEVICES §4).
#[test]
fn the_token_subs_the_person_not_a_device() {
    const DEVICE: &str = "1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f809";
    let key = load_or_create(tempfile::tempdir().unwrap().path()).unwrap();
    let token = key.token(claims()).unwrap();
    let (input, _) = token.rsplit_once('.').unwrap();
    let (_, payload) = input.split_once('.').unwrap();
    let payload: serde_json::Value =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap();
    assert_eq!(payload["sub"], SUB, "sub is the person's key");
    assert_ne!(payload["sub"], DEVICE, "sub is the device's key");
}

/// The front channel's answer to an ask whose `redirect_uri` is `redirect`
/// (form-encoded into the query here, as a client sends it), for Ana at a
/// lend on port 8096, the browser on `browser`: its status and `Location`.
fn authorized(redirect: &str, browser: Option<&str>) -> (u16, Option<String>) {
    let dir = tempfile::tempdir().unwrap();
    let key = std::sync::Arc::new(crate::SigningKey::from_bytes(&[1; 32]));
    let rail = std::sync::Arc::new(crate::RingRail::new(dir.path(), key));
    let provider = Provider::new(dir.path().to_path_buf(), 1, rail).unwrap();
    let lend = crate::lend::Lend::carried("media", "0123456789abcdef").unwrap();
    let client = SignInWithSvrngs {
        client_id: "svrngs-media".into(),
        client_secret: None,
    };
    let who = svrngs_core::token::Subject {
        sub: SUB.into(),
        name: "Ana".into(),
        username: "Ana-9f2c0a1b3d4e5f60".into(),
        group: svrngs_core::token::Group::Member,
    };
    let redirect: String = form_urlencoded::byte_serialize(redirect.as_bytes()).collect();
    let query =
        format!("response_type=code&client_id=svrngs-media&redirect_uri={redirect}&scope=openid");
    let port = std::num::NonZeroU16::new(8096).unwrap();
    let answer = authorize(
        &provider,
        &lend,
        &client,
        port,
        browser,
        (who, "0123456789abcdef".into(), SUB.into()),
        Some(&query),
    );
    let location = (answer.headers().get(axum::http::header::LOCATION))
        .map(|l| String::from_utf8_lossy(l.as_bytes()).into_owned());
    (answer.status().as_u16(), location)
}

/// A redirect a browser would read as another host's is refused 400 at the
/// authorize path, and no code is answered: `//` after the lend's own
/// address, on the service's or the browser's origin, or `\`, which a
/// browser reads as `/`. Red at 3f313267: each answered 302 with
/// `Location: //evil.example/cb?code=…`, or `/\evil.example/cb?code=…`, a
/// protocol-relative URL off the lend's origin (front.rs:59-69, 88).
#[test]
fn a_redirect_a_browser_reads_as_another_hosts_is_refused() {
    const BROWSER: &str = "http://umfrj1nx.localhost:6473";
    for (redirect, browser) in [
        ("http://localhost:8096//evil.example/cb", None),
        ("http://localhost:8096/\\evil.example/cb", None),
        (
            "http://umfrj1nx.localhost:6473//evil.example/cb",
            Some(BROWSER),
        ),
    ] {
        let (status, location) = authorized(redirect, browser);
        assert_eq!((status, location.as_deref()), (400, None), "{redirect}");
    }
}

/// A control byte in the redirect, `%0A` in the ask, is refused 400, not a
/// panic of the request's task. Red at 3f313267: it panicked at
/// front.rs:103-104's `expect`, "a path and a URL-safe code are a header".
#[test]
fn a_control_byte_in_the_redirect_is_refused_not_a_panic() {
    let (status, location) = authorized("http://localhost:8096/cb\n", None);
    assert_eq!((status, location.as_deref()), (400, None));
}

/// The service's own address is the door's one set of hosts, `localhost`,
/// `127.0.0.1` and `[::1]`, case ignored, as a `Location` the door makes a
/// path reads it: a redirect on any is answered relative to the lend's
/// origin. Red at 3f313267: front.rs:59 took `http://localhost:<port>`
/// alone, exact, and refused each of these 400.
#[test]
fn the_services_own_address_is_the_doors_one_set_of_hosts() {
    for redirect in [
        "http://127.0.0.1:8096/cb",
        "http://[::1]:8096/cb",
        "HTTP://LocalHost:8096/cb",
    ] {
        let (status, location) = authorized(redirect, None);
        assert_eq!(status, 302, "{redirect}: {location:?}");
        assert!(
            location
                .as_deref()
                .is_some_and(|l| l.starts_with("/cb?code=")),
            "{redirect}: {location:?}"
        );
    }
}

/// The browser at `host` as the door names it to the front channel
/// (door.rs `lend_for`): its scheme, as `scheme_of` reads the host, and the host.
fn browser_at(host: &str) -> String {
    format!("{}://{host}", crate::door::scheme_of(host))
}

/// The authorize path takes a redirect on the lend's own origin, the
/// browser's, as it takes one on the service's own address: a computer's
/// `<label>.localhost` in http, a page host's name in https, the room's
/// address in http; answered relative to that origin. Another lend's
/// origin, the label on a host that is not the browser's, and the browser's
/// host in the other scheme are refused 400, no code (tests/door
/// oidc.rs:536-593, 686-688). Planted, watched failing: `LendPath::parse`
/// reading no browser origin, as the front channel before oidc-jellyfin.
#[test]
fn the_authorize_path_takes_the_lend_s_own_origin_and_no_other() {
    const AT: &str = "umfrj1nx.localhost:6473";
    for host in [AT, "umfrj1nx.svrngs.example", "192.168.1.20:7123"] {
        let browser = browser_at(host);
        let (status, location) = authorized(&format!("{browser}/signin-oidc"), Some(&browser));
        assert_eq!(status, 302, "{host}: {location:?}");
        assert!(
            location
                .as_deref()
                .is_some_and(|l| l.starts_with("/signin-oidc?code=")),
            "{host}: {location:?}"
        );
    }
    for redirect in [
        "http://pickslbl.localhost:6473/signin-oidc",
        "http://umfrj1nx.other.example/signin-oidc",
        "https://umfrj1nx.localhost:6473/signin-oidc",
    ] {
        let (status, location) = authorized(redirect, Some(&browser_at(AT)));
        assert_eq!((status, location.as_deref()), (400, None), "{redirect}");
    }
}

const CLIENT: &str = "svrngs-media";
const CLIENT_SECRET: &str = "media-secret_2026.10";
const REDIRECT: &str = "http://localhost:8096/signin-oidc";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
/// `S256` of [`VERIFIER`].
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

/// Ada's computer (key 1), holding the house with bo (key 2), lending
/// `media` and `other` to it, each signing people in with svrngs; and its
/// provider, on the same data dir and rail.
struct Lender {
    dir: tempfile::TempDir,
    rail: std::sync::Arc<crate::RingRail>,
    provider: Provider,
}

fn lender() -> Lender {
    let dir = tempfile::tempdir().unwrap();
    let key = |seed: u8| crate::SigningKey::from_bytes(&[seed; 32]);
    let rail = std::sync::Arc::new(crate::RingRail::new(dir.path(), key(1).into()));
    let people = [("ada", 1), ("bo", 2)].map(|(p, k)| {
        (
            crate::Person::parse(p).unwrap(),
            vec![crate::actor_of(&key(k))],
        )
    });
    rail.set_seed("house", &crate::Roster::new(people.into_iter().collect()))
        .unwrap();
    let lends = crate::lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    for (name, client) in [("media", CLIENT), ("other", "svrngs-other")] {
        let body = serde_json::json!({
            "port": 8096, "kind": "k", "groups": ["house"],
            "sign_in_with_svrngs": { "client_id": client, "client_secret": CLIENT_SECRET },
        });
        std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
    }
    let provider = Provider::new(dir.path().to_path_buf(), 1, rail.clone()).unwrap();
    Lender {
        dir,
        rail,
        provider,
    }
}

impl Lender {
    /// The code the door's front channel answers `who`, let in through the
    /// house on `device`, asking `media` with `extra` on the ask.
    fn code(&self, who: svrngs_core::token::Subject, device: &str, extra: &str) -> String {
        let name = crate::lend::LendName::parse("media").unwrap();
        let lend = crate::lend::read(self.dir.path(), &name).unwrap().unwrap();
        let client = lend.sign_in_with_svrngs.clone().unwrap();
        let redirect: String = form_urlencoded::byte_serialize(REDIRECT.as_bytes()).collect();
        let query = format!(
            "response_type=code&client_id={CLIENT}&redirect_uri={redirect}&scope=openid\
             &state=st-1&code_challenge={CHALLENGE}&code_challenge_method=S256{extra}"
        );
        let port = std::num::NonZeroU16::new(8096).unwrap();
        let group = ("house".to_string(), device.to_string());
        let answer = authorize(
            &self.provider,
            &lend,
            &client,
            port,
            None,
            (who, group.0, group.1),
            Some(&query),
        );
        assert_eq!(answer.status(), 302);
        let location = answer.headers()[axum::http::header::LOCATION]
            .to_str()
            .unwrap();
        let query = location.split_once('?').unwrap().1;
        let code = form_urlencoded::parse(query.as_bytes()).find(|(k, _)| k == "code");
        code.unwrap().1.into_owned()
    }

    /// The token route's answer for `lend`, asked in process, `form` beside
    /// the client's id and secret.
    async fn token(&self, lend: &str, form: &[(&str, &str)]) -> (u16, serde_json::Value) {
        use tower::ServiceExt;
        let client = if lend == "media" {
            CLIENT
        } else {
            "svrngs-other"
        };
        let body = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(form)
            .extend_pairs([("client_id", client), ("client_secret", CLIENT_SECRET)])
            .finish();
        let asked = axum::http::Request::post(format!("/oidc/{lend}/token"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(axum::body::Body::from(body))
            .unwrap();
        let answer = self.provider.clone().router().oneshot(asked).await.unwrap();
        let status = answer.status().as_u16();
        let body = axum::body::to_bytes(answer.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&body).unwrap_or_default())
    }

    async fn redeem(
        &self,
        lend: &str,
        code: &str,
        redirect: &str,
        verifier: &str,
    ) -> (u16, serde_json::Value) {
        let form = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect),
            ("code_verifier", verifier),
        ];
        self.token(lend, &form).await
    }
}

fn subject(
    sub: &str,
    name: &str,
    username: &str,
    group: svrngs_core::token::Group,
) -> svrngs_core::token::Subject {
    svrngs_core::token::Subject {
        sub: sub.into(),
        name: name.into(),
        username: username.into(),
        group,
    }
}

/// The code's claims, as a stock client reads the id token.
fn id_claims(answer: &serde_json::Value) -> serde_json::Value {
    let token = answer["id_token"].as_str().expect("an id token");
    let (input, _) = token.rsplit_once('.').unwrap();
    let (_, payload) = input.split_once('.').unwrap();
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap()
}

/// A code the front channel minted for the door's person redeems at the
/// token route, once, for that person's claims and the ask's nonce, never a
/// `sub` the ask names; at another lend's route, with another login's
/// verifier, or for another redirect, it is refused; once its person is
/// removed from the group it was minted through, or the lend is no longer
/// lent there, `invalid_grant` (tests/door oidc.rs:163-301, 445-456,
/// 481-491). In process, no socket. Planted, watched failing, one at a
/// time: the token route not asking the code's person's standing; not
/// asking `reached`.
#[tokio::test]
async fn a_code_redeems_for_the_door_s_person_only_while_the_door_would_let_them_in() {
    use svrngs_core::token::Group;
    let l = lender();
    let bo = crate::actor_of(&crate::SigningKey::from_bytes(&[2; 32]));
    let username = format!("bo-{}", &bo[..16]);
    let bos = || subject(&bo, "bo", &username, Group::Member);
    let code = l.code(bos(), &bo, "&nonce=n-0S6_WzA2Mj&sub=cy");
    let (status, answer) = l.redeem("media", &code, REDIRECT, VERIFIER).await;
    assert_eq!(status, 200, "{answer:#}");
    let claims = id_claims(&answer);
    assert_eq!(claims["sub"], bo.as_str(), "{claims:#}");
    assert_eq!(claims["name"], "bo");
    assert_eq!(claims["preferred_username"], username.as_str());
    assert_eq!(claims["groups"], serde_json::json!(["member"]));
    assert_eq!(claims["nonce"], "n-0S6_WzA2Mj");
    let again = l.redeem("media", &code, REDIRECT, VERIFIER).await;
    assert_eq!(again.0, 400, "a code redeems once: {:#}", again.1);

    for (why, lend, redirect, verifier) in [
        ("another lend's route", "other", REDIRECT, VERIFIER),
        (
            "another login's verifier",
            "media",
            REDIRECT,
            "another-login-attempt-s-verifier-43-characters",
        ),
        (
            "another redirect",
            "media",
            "http://localhost:8096/elsewhere",
            VERIFIER,
        ),
    ] {
        let code = l.code(bos(), &bo, "");
        let (status, answer) = l.redeem(lend, &code, redirect, verifier).await;
        assert_eq!(status, 400, "{why}: {answer:#}");
    }

    let invalid = Some("invalid_grant");
    let gone = l.code(bos(), &bo, "");
    let name = crate::lend::LendName::parse("media").unwrap();
    assert!(crate::lend::set_audience(l.dir.path(), &name, vec!["yard".into()]).unwrap());
    let (status, answer) = l.redeem("media", &gone, REDIRECT, VERIFIER).await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, invalid),
        "no longer lent: {answer:#}"
    );
    assert!(crate::lend::set_audience(l.dir.path(), &name, vec!["house".into()]).unwrap());

    let left = l.code(bos(), &bo, "");
    let removal = svrngs_core::Removal {
        person: crate::Person::parse("bo").unwrap(),
        joined: None,
    };
    l.rail.commit("house", &removal).unwrap().unwrap();
    let (status, answer) = l.redeem("media", &left, REDIRECT, VERIFIER).await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, invalid),
        "removed: {answer:#}"
    );
}

/// A guest's code names the guest, never the host, and is held to the
/// guest's visit by the door's one decision (`guest_admitted`): Sam's code,
/// Kim's visit live, redeems for Sam's visit key, account and name, group
/// `guest`; a guest code for a key no visit here names is refused for the
/// visit (tests/door oidc.rs:797-805, 873-888). Planted, watched failing:
/// the token route holding a guest's code to the audience alone.
#[tokio::test]
async fn a_guest_s_code_names_the_guest_and_holds_to_their_visit() {
    use svrngs_core::token::Group;
    let l = lender();
    let key = |seed: u8| crate::actor_of(&crate::SigningKey::from_bytes(&[seed; 32]));
    let (sam, ada) = (key(9), key(1));
    let mut opens = std::collections::BTreeMap::new();
    opens.insert(ada.clone(), ["media".to_string()].into());
    let visit = crate::Visit {
        guest: crate::Person::parse("Sam").unwrap(),
        key: sam.clone(),
        opens,
        until: crate::door::unix_now().unwrap() + 24 * 3600,
    };
    let act = svrngs_core::tests_support::Fixture {
        act: crate::RailAct::Visit(visit),
    };
    l.rail.commit("house", &act).unwrap().unwrap();
    let username = format!("Sam-{}.guest", &sam[..16]);
    let sams = subject(&sam, "Sam (ada's guest)", &username, Group::Guest);
    let code = l.code(sams, &sam, "");
    let (status, answer) = l.redeem("media", &code, REDIRECT, VERIFIER).await;
    assert_eq!(status, 200, "{answer:#}");
    let claims = id_claims(&answer);
    assert_eq!(claims["sub"], sam.as_str());
    assert_eq!(claims["preferred_username"], username.as_str());
    assert_eq!(claims["groups"], serde_json::json!(["guest"]));
    assert!(
        !answer.to_string().contains(&ada),
        "the host's key reached the app"
    );

    let nobody = key(5);
    let stray = subject(&nobody, "Nobody (ada's guest)", "x.guest", Group::Guest);
    let code = l.code(stray, &nobody, "");
    let (status, answer) = l.redeem("media", &code, REDIRECT, VERIFIER).await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, Some("invalid_grant"))
    );
    let why = answer["error_description"].as_str().unwrap_or_default();
    assert!(why.contains("guest"), "refused for the visit: {answer:#}");
}
