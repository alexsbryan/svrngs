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
    let provider = Provider::new(dir.path().to_path_buf(), 1).unwrap();
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
    let answer = authorize(&provider, &lend, &client, port, browser, who, Some(&query));
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
