use super::*;

fn request(method: &str, path: &str, port: u16, proof: Option<&str>) -> Request {
    let mut r = Request::builder()
        .method(method)
        .uri(path)
        .header("host", format!("svrngs.localhost:{port}"));
    if let Some(proof) = proof {
        r = r.header(PROOF, proof);
    }
    r.body(axum::body::Body::empty()).unwrap()
}

#[test]
fn cookies_never_authorize_and_proofs_bind_the_request_and_cannot_replay() {
    let dir = tempfile::tempdir().unwrap();
    let s = Sessions::new(crate::api::kept_token(dir.path()).unwrap(), 4321);
    // The cookie a browser opened before kept: the token's HMAC over the
    // host's first name, which no longer opens anything.
    let legacy = URL_SAFE_NO_PAD.encode(ring::hmac::sign(&s.key, HOME.as_bytes()));
    let mut cookie = request("GET", "/v1/groups", 4321, None);
    cookie
        .headers_mut()
        .insert("cookie", format!("svrngs={legacy}").parse().unwrap());
    assert_eq!(s.shown(&cookie, HOME), Shown::Neither);
    let now = crate::door::unix_now().unwrap();
    let nonce = "a".repeat(32);
    let message = format!(
        "svrngs.localhost:4321\nPOST\n/group\n{now}.{nonce}\n\n\n{}",
        s.epoch
    );
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &s.browser_key(HOME));
    let tag = URL_SAFE_NO_PAD.encode(ring::hmac::sign(&key, message.as_bytes()));
    let proof = format!("{now}.{nonce}.{tag}");
    for (method, path, port) in [
        ("GET", "/group", 4321),
        ("POST", "/group?other", 4321),
        ("POST", "/group", 1234),
    ] {
        assert_eq!(
            s.shown(&request(method, path, port, Some(&proof)), HOME),
            Shown::Neither
        );
    }
    assert_eq!(
        s.shown(&request("POST", "/group", 4321, Some(&proof)), HOME),
        Shown::Session
    );
    assert_eq!(
        s.shown(&request("POST", "/group", 4321, Some(&proof)), HOME),
        Shown::Neither
    );
    let restarted = Sessions::new(crate::api::kept_token(dir.path()).unwrap(), 4321);
    assert_eq!(
        restarted.shown(&request("POST", "/group", 4321, Some(&proof)), HOME),
        Shown::Neither,
        "the same enrolled key cannot replay a previous boot's proof"
    );
}

#[tokio::test]
async fn a_control_plane_body_must_match_its_proved_digest() {
    let dir = tempfile::tempdir().unwrap();
    let s = Sessions::new(crate::api::kept_token(dir.path()).unwrap(), 4321);
    let bytes = br#"{"said":"yes"}"#;
    let digest = hex::encode(ring::digest::digest(&ring::digest::SHA256, bytes).as_ref());
    let make = |body: &'static [u8]| {
        Request::builder()
            .method("POST")
            .uri("/choices")
            .header("x-svrngs-body", &digest)
            .body(axum::body::Body::from(body))
            .unwrap()
    };
    assert!(!s.body_matches(&mut make(br#"{"said":"stop"}"#)).await);
    let mut exact = make(bytes);
    assert!(s.body_matches(&mut exact).await);
    assert_eq!(
        axum::body::to_bytes(exact.into_body(), 1024)
            .await
            .unwrap()
            .as_ref(),
        bytes
    );
}

/// **One busy origin cannot spend another's window.** Each origin keeps
/// its own nonces, so one that shows [`PER_ORIGIN`] proofs in a minute is
/// refused its next while another's first is taken; a replay is refused
/// in its own origin only; a stale stamp's nonce is dropped, freeing its
/// room; and no more than [`ORIGINS`] origins hold nonces at once. Watched
/// failing on the one store all origins shared (4,096 nonces in all).
#[test]
fn one_busy_origin_cannot_spend_anothers_window() {
    let nonces = Nonces::default();
    let now = 1_000;
    for n in 0..PER_ORIGIN as u128 {
        assert_eq!(nonces.spend("busy", now, n, now), Ok(()));
    }
    assert_eq!(
        nonces.spend("busy", now, u128::MAX, now),
        Err(Spent::OriginFull)
    );
    assert_eq!(
        nonces.spend("quiet", now, 0, now),
        Ok(()),
        "the quiet origin's first proof"
    );
    assert_eq!(nonces.spend("quiet", now, 0, now), Err(Spent::Replayed));
    let later = now + FRESH;
    assert_eq!(
        nonces.spend("busy", later, u128::MAX, later),
        Ok(()),
        "the stale ones dropped"
    );
    for origin in 0..ORIGINS - 1 {
        assert_eq!(nonces.spend(&format!("o{origin}"), later, 0, later), Ok(()));
    }
    assert_eq!(
        nonces.spend("one-more", later, 0, later),
        Err(Spent::TooManyOrigins)
    );
}

/// A proof for `method path` at the home page of `s` (port 4321), as
/// home/session.js makes it, no body digest or intent.
fn proof_for(s: &Sessions, method: &str, path: &str, nonce: char) -> String {
    let now = crate::door::unix_now().unwrap();
    let nonce = nonce.to_string().repeat(32);
    let message = format!(
        "svrngs.localhost:4321\n{method}\n{path}\n{now}.{nonce}\n\n\n{}",
        s.epoch
    );
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &s.browser_key(HOME));
    let tag = URL_SAFE_NO_PAD.encode(ring::hmac::sign(&key, message.as_bytes()));
    format!("{now}.{nonce}.{tag}")
}

/// **A redirect's onward request is proved once, where and how the answer
/// says** (`Onwards`): by GET after a 303 to a POST, by the same method after
/// a 307, at an own-origin absolute address as at its path; never at a
/// redirect to another origin, and never twice. Red before `Onwards`: every
/// onward request `Neither`, as the conformance canary's `fetch('redirect')` was 401.
#[test]
fn a_redirect_s_next_onward_is_proved_once_where_and_how_the_answer_says() {
    let dir = tempfile::tempdir().unwrap();
    let s = Sessions::new(crate::api::kept_token(dir.path()).unwrap(), 4321);
    let to = |status: u16, at: &str| {
        Response::builder()
            .status(status)
            .header(header::LOCATION, at)
            .body(axum::body::Body::empty())
            .unwrap()
    };
    let post = axum::http::Method::POST;
    for (nonce, status, at, next, kept) in [
        ('a', 303, "/done", "GET", true),
        ('b', 307, "/again", "POST", true),
        ('c', 302, "http://svrngs.localhost:4321/own?x", "GET", true),
        ('d', 302, "http://other.localhost:4321/x", "GET", false),
        ('e', 302, "//evil.example/x", "GET", false),
    ] {
        let proof = proof_for(&s, "POST", "/form", nonce);
        assert_eq!(
            s.shown(&request("POST", "/form", 4321, Some(&proof)), HOME),
            Shown::Session
        );
        s.onward(HOME, &proof, &post, &to(status, at));
        let path = at
            .strip_prefix("http://svrngs.localhost:4321")
            .unwrap_or(at);
        let other = if next == "GET" { "POST" } else { "GET" };
        assert_eq!(
            s.shown(&request(other, path, 4321, Some(&proof)), HOME),
            Shown::Neither,
            "{status} {at}: not by {other}"
        );
        let onward = if kept { Shown::Session } else { Shown::Neither };
        assert_eq!(
            s.shown(&request(next, path, 4321, Some(&proof)), HOME),
            onward,
            "{status} {at}"
        );
        assert_eq!(
            s.shown(&request(next, path, 4321, Some(&proof)), HOME),
            Shown::Neither,
            "{status} {at}: once"
        );
    }
}

/// The home page's address, as the command and install.sh say it and the
/// page says it at its foot from the origin it was opened at
/// (web/tests/pages/home.test.mjs, 'the page says its own address at its
/// foot'): this host, the port it started on, and `/`.
#[test]
fn the_home_page_s_address_is_its_host_at_its_port() {
    assert_eq!(crate::home::address(4321), "http://svrngs.localhost:4321/");
}
