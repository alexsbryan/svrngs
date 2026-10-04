// SPDX-License-Identifier: AGPL-3.0-or-later
//! The door's rules, one at a time; `tests/door/` runs them through the
//! door between two nodes.

use super::*;

fn map(pairs: &[(&str, &str)]) -> HeaderMap {
    let mut h = HeaderMap::new();
    for (n, v) in pairs {
        h.append(
            HeaderName::try_from(*n).unwrap(),
            HeaderValue::try_from(*v).unwrap(),
        );
    }
    h
}

fn identity() -> [(HeaderName, HeaderValue); 2] {
    [
        (
            HeaderName::from_static(PERSON_HEADER),
            HeaderValue::from_static("bo"),
        ),
        (
            HeaderName::from_static(KEY_HEADER),
            HeaderValue::from_static("0b0b"),
        ),
    ]
}

fn values<'a>(h: &'a HeaderMap, name: &str) -> Vec<&'a str> {
    h.get_all(name)
        .iter()
        .map(|v| v.to_str().unwrap())
        .collect()
}

/// Upstream's comparison (`compare_name`): the three spellings
/// `rewrite_head`'s test displaces (iroh_identity_forward.rs:653-662)
/// compare as the declared name. hyper refuses the second and third before
/// the door sees them; `tests/door/` watches that.
#[test]
fn a_name_compares_case_padding_and_control_byte_blind() {
    for spelling in ["AUTHORIZATION", "  authorization  ", "Authoriz\u{7f}ation"] {
        assert_eq!(compare_name(spelling), "authorization", "{spelling:?}");
    }
}

/// Everything under the prefix goes, whatever follows it, and the door's
/// own pair is the one left; a secret's name displaces the client's copy.
/// The failing input: a client's `x-svrngs-person` kept.
#[test]
fn the_client_s_identity_and_secret_names_are_displaced() {
    let mut h = map(&[
        ("x-svrngs-person", "forged"),
        ("X-SVRNGS-KEY", "forged"),
        ("x-svrngs-anything", "nope"),
        ("authorization", "the-viewers"),
        ("range", "bytes=0-9"),
        ("cookie", "c"),
    ]);
    let secret = [(
        HeaderName::from_static("authorization"),
        HeaderValue::from_static("the-holders"),
    )];
    let displaced = rewrite_headers(&mut h, identity(), &secret);
    assert_eq!(displaced, 4);
    assert_eq!(values(&h, PERSON_HEADER), ["bo"]);
    assert_eq!(values(&h, KEY_HEADER), ["0b0b"]);
    assert!(values(&h, "x-svrngs-anything").is_empty());
    assert_eq!(values(&h, "authorization"), ["the-holders"]);
    assert_eq!(values(&h, "range"), ["bytes=0-9"]);
    assert_eq!(values(&h, "cookie"), ["c"]);
}

/// Hop-by-hop headers stop at the door, with any `Connection` names.
#[test]
fn hop_by_hop_headers_stop_at_the_door() {
    let mut h = map(&[
        ("connection", "keep-alive, x-private"),
        ("keep-alive", "timeout=5"),
        ("x-private", "1"),
        ("upgrade", "websocket"),
        ("if-range", "\"etag\""),
    ]);
    assert_eq!(strip_hop_by_hop(&mut h), 4);
    assert_eq!(values(&h, "if-range"), ["\"etag\""]);
}

/// A response header holding the secret is dropped; the rest pass.
#[test]
fn a_response_header_echoing_the_secret_is_dropped() {
    let mut h = map(&[
        ("x-echo", "seen MediaBrowser Token=\"k\" here"),
        ("content-range", "bytes 0-9/10"),
    ]);
    let secret = [(
        HeaderName::from_static("authorization"),
        HeaderValue::from_static("MediaBrowser Token=\"k\""),
    )];
    assert_eq!(drop_echoed_secret(&mut h, &secret), ["x-echo"]);
    assert_eq!(values(&h, "content-range"), ["bytes 0-9/10"]);
}

#[test]
fn a_path_that_can_step_out_is_seen() {
    for bad in [
        "/lend/a/../b",
        "/lend/a/%2e%2e/b",
        "/lend/a/x%2Fy",
        "/lend/%2E/",
    ] {
        assert!(can_step_out(bad), "{bad}");
    }
    assert!(!can_step_out("/lend/a/b..c/d"));
}

#[test]
fn the_first_segment_names_the_lend_and_the_rest_is_the_service_s() {
    assert_eq!(
        split_lend("/lend/shelf/web/index.html"),
        Some(("shelf", "/web/index.html".into()))
    );
    assert_eq!(split_lend("/lend/shelf"), Some(("shelf", "/".into())));
    assert_eq!(split_lend("/lend/shelf/"), Some(("shelf", "/".into())));
    assert_eq!(split_lend("/other"), None);
}
