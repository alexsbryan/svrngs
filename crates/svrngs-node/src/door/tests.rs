#![allow(clippy::disallowed_methods, reason = "a test dials what it runs")]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! The door's rules, one at a time; `tests/door/` runs them through the
//! door between two nodes.

use super::*;
use eudoxia_community::binding;

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

fn identity() -> Identity {
    [
        (
            HeaderName::from_static(PERSON_HEADER),
            HeaderValue::from_static("bo"),
        ),
        (
            HeaderName::from_static(KEY_HEADER),
            HeaderValue::from_static("0b0b"),
        ),
        (
            HeaderName::from_static(USER_HEADER),
            HeaderValue::from_static("bo-0b0b"),
        ),
        (
            HeaderName::from_static(DEVICE_HEADER),
            HeaderValue::from_static("0b0c"),
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
        ("x-svrngs-user", "ana"),
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
    assert_eq!(displaced, 5);
    assert_eq!(values(&h, PERSON_HEADER), ["bo"]);
    assert_eq!(values(&h, KEY_HEADER), ["0b0b"]);
    assert_eq!(values(&h, USER_HEADER), ["bo-0b0b"]);
    assert!(values(&h, "x-svrngs-anything").is_empty());
    assert_eq!(values(&h, "authorization"), ["the-holders"]);
    assert_eq!(values(&h, "range"), ["bytes=0-9"]);
    assert_eq!(values(&h, "cookie"), ["c"]);
}

/// Two keys are two accounts though their people share a name; an app's
/// own `ana` is never one of them; what a header-sign-in app may refuse,
/// or read as a pattern, is left out (moved from the Jellyfin middleware's
/// `viewer.rs`, bash-person (b)).
#[test]
fn each_key_names_an_account_of_its_own() {
    let name = |raw: &str, key: &str| user_name(&Person::parse(raw).unwrap(), key);
    let (a, b) = (name("Ana", &"ab".repeat(32)), name("Ana", &"cd".repeat(32)));
    assert_eq!(a, "Ana-abababababababab");
    assert_ne!(a, b);
    assert_ne!(name("ana", &"ab".repeat(32)), "ana");
    assert_eq!(
        name("Ana/<Ben>_%", &"ab".repeat(32)),
        "AnaBen-abababababababab"
    );
    assert_eq!(
        name("Zoë O'Neil.2", &"ab".repeat(32)),
        "ZoONeil.2-abababababababab"
    );
    assert_eq!(
        name("イチロー", &"ab".repeat(32)),
        "person-abababababababab"
    );
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

/// **A clock before 1970 is no time, never 0** (#203): read as 0, every
/// lapsed pass opened a lend, since `0 >= until` is false. Planted, watched
/// failing: `unix_secs` reading it as 0, as `unix_now` did at 0d0f4f9b.
#[test]
fn a_clock_before_1970_is_no_time() {
    let before = std::time::UNIX_EPOCH - Duration::from_secs(1);
    assert_eq!(unix_secs(before), None);
    assert_eq!(
        unix_secs(std::time::UNIX_EPOCH + Duration::from_secs(7)),
        Some(7)
    );
}

/// The scheme a lend's service is told is the one the browser used:
/// plain http where svrngs serves the lend itself — a `*.localhost` origin
/// on a computer, and an address, as the room serves each lend at
/// `http://<private address>:<port>/` (room.rs) — and https at a page
/// host's name, where a phone opens it. Red at dd7721c7: both addresses
/// were told `https`.
#[test]
fn the_scheme_told_is_the_one_the_browser_used() {
    for (host, scheme) in [
        ("umfrj1nx.localhost:6473", "http"),
        ("192.168.1.20:7123", "http"),
        ("[fd12:3456::20]:7123", "http"),
        ("umfrj1nx.svrnme.sh", "https"),
        ("umfrj1nx.group.example:8443", "https"),
    ] {
        assert_eq!(scheme_of(host), scheme, "{host}");
    }
}

/// On port 80 a service's own address names no port, so `Host` and `Origin`
/// read as a browser's would and a redirect on `http://localhost/` stays on
/// the lend. Planted, watched failing: `own_address` naming `:80`, and
/// `LendPath::parse` taking only `http://<host>:<port>`, which sent a visitor
/// to their own computer. A name that only starts the same is still refused.
#[test]
fn port_80_names_no_port() {
    let port = |n| NonZeroU16::new(n).unwrap();
    assert_eq!(own_address(port(80)), "localhost");
    assert_eq!(own_address(port(8096)), "localhost:8096");
    let path = |url, at: u16| LendPath::parse(url, port(at), None).map(|p| p.location([]));
    assert_eq!(
        path("http://localhost/session/new", 80).unwrap(),
        "/session/new"
    );
    assert_eq!(path("http://127.0.0.1:80/x", 80).unwrap(), "/x");
    assert!(path("http://localhost.evil.example/x", 80).is_err());
    assert!(path("http://localhost/x", 8096).is_err());
}

/// The rule of [`admit`] (svrn-docs `rings/current/CAPABILITY_CONNECTIONS_PLAN.md`
/// §2; `CONTRACTS.md` §6.7), row by row: another app's read goes as the
/// readers' account or bare, and nothing else of its ever reaches the
/// service; a send on a `sends:` row goes the same way, but only for an app
/// its group carries and never for a pass's holder: a lent app's send, and
/// a phone's or a pass and link's, are refused; a person's own and a lend's
/// own page are untouched. Planted: `sends` in `admit` true for any app
/// (the lent page's and the holder's sends let in).
#[test]
fn admit_is_the_rule_for_every_request_naming_another_app() {
    use Asked::{Read, Send, Write};
    let own = Label::of(&[1; 32], "shelf");
    let other = Label::of(&[2; 32], "picks");
    let carried = Label::carried("Taste", "the-group-s-id-0123");
    let refused = Err(rules::Refusal::ReadOnly);
    // The lender's door decides under its own key: a real one.
    let node = ed25519_dalek::SigningKey::from_bytes(&[7; 32])
        .verifying_key()
        .to_bytes();
    for (app, asked, readers, holder, want) in [
        (None, Write, false, false, Ok(Admit::Own)),
        (None, Write, true, false, Ok(Admit::Own)),
        (None, Read, true, false, Ok(Admit::Own)),
        (Some(&own), Write, true, false, Ok(Admit::Own)),
        (Some(&own), Read, false, false, Ok(Admit::Own)),
        (Some(&other), Read, true, false, Ok(Admit::Readers)),
        (Some(&other), Read, false, false, Ok(Admit::Bare)),
        (Some(&other), Write, true, false, refused),
        (Some(&other), Write, false, false, refused),
        (Some(&carried), Send, true, false, Ok(Admit::Readers)),
        (Some(&carried), Send, false, false, Ok(Admit::Bare)),
        (Some(&carried), Write, true, false, refused),
        (Some(&carried), Send, true, true, refused),
        (Some(&carried), Read, true, true, Ok(Admit::Readers)),
        (Some(&other), Send, true, false, refused),
        (Some(&other), Send, false, true, refused),
    ] {
        assert_eq!(
            {
                let carried = app.is_some_and(|a| matches!(a.whose(), Whose::Carried));
                let app = app.map(Label::as_str);
                rules::admit(&node, own.as_str(), app, asked, (readers, holder, carried))
            },
            want,
            "{app:?} {asked:?} {readers} {holder}"
        );
    }
}

// The door's decisions that `tests/door/` once judged only through two
// nodes, here over explicit inputs: a lend, a head, a rail on a tempdir and
// the time. What only a crossing shows, that `forward` and `lend_for` apply
// these to a dialer's request, stays in `tests/door/` (docs/seams.tsv).

fn key(seed: u8) -> crate::SigningKey {
    crate::SigningKey::from_bytes(&[seed; 32])
}

fn actor(seed: u8) -> String {
    crate::actor_of(&key(seed))
}

/// A lend on a port, lent to `groups`, as a declaration reads.
fn port_lend(name: &str, groups: &[&str]) -> Lend {
    Lend {
        name: LendName::parse(name).unwrap(),
        source: Source::Port(NonZeroU16::new(8096).unwrap()),
        kind: lend::Program::parse("k").unwrap(),
        terms: crate::lend::Terms {
            audience: lend::Audience::Selected(groups.iter().map(|g| g.to_string()).collect()),
            limit: NonZeroU16::new(3),
            refused: Vec::new(),
            upload: None,
            reads: Vec::new(),
            sign_in_with_svrngs: None,
            asks: None,
            sends: Vec::new(),
            approved: None,
        },
    }
}

/// The groups a computer whose key is `me` holds, each seeded with its
/// people and their keys, on a rail of its own in a tempdir.
type People<'a> = &'a [(&'a str, &'a [u8])];

fn held(me: u8, groups: &[(&str, People)]) -> (tempfile::TempDir, RingRail) {
    let dir = tempfile::tempdir().unwrap();
    let rail = RingRail::new(dir.path(), Arc::new(key(me)));
    for (group, people) in groups {
        let people = people.iter().map(|(p, keys)| {
            let keys = keys.iter().map(|k| actor(*k)).collect();
            (Person::parse(p).unwrap(), keys)
        });
        rail.set_seed(group, &crate::Roster::new(people.collect()))
            .unwrap();
    }
    (dir, rail)
}

/// `said` written to `group` by the rail's own key: its first line's id and
/// time.
fn wrote<C: svrngs_core::Command>(rail: &RingRail, group: &str, said: &C) -> (crate::OpId, i64)
where
    C::Refusal: std::fmt::Debug,
{
    let recorded = rail.commit(group, said).unwrap().unwrap();
    let line = &recorded.lines()[0];
    (line.id.clone(), line.ts_unix)
}

/// svrngs's own answer as the door marks it: its status, its kind and its
/// words.
fn said(r: &Response) -> (u16, Option<&str>, Option<&str>) {
    let kind = r.headers().get(REFUSED_HEADER).map(|v| v.to_str().unwrap());
    let words = r.extensions().get::<Words>().map(|w| w.0.as_str());
    (r.status().as_u16(), kind, words)
}

/// Every header under the prefix the client typed goes, whatever it names,
/// and a name a credential the door sends fills goes too, its spelling
/// aside: a forged app, group, pass or link, and a page's own `x-api-key`
/// where the readers' key is sent (tests/door at 5e20d0b3, main.rs:1431-1435,
/// 1561-1597, scope.rs:60, main.rs:1167, 1286-1287). Planted, watched
/// failing: `rewrite_headers` stripping no declared name (the page's own
/// `x-api-key` kept beside the readers').
#[test]
fn a_client_s_forged_door_headers_and_credential_names_never_reach_the_service() {
    let mut h = map(&[
        ("x-svrngs-app", "forged"),
        ("x-svrngs-group", "yard"),
        ("x-svrngs-pass", "a-pass"),
        ("x-svrngs-link", "a-link"),
        ("x-api-key", "the-page-s-own"),
        ("AUTHORIZATION", "viewer-upper"),
    ]);
    let readers = [(
        HeaderName::from_static("x-api-key"),
        HeaderValue::from_static("immich-api-key-for-apps"),
    )];
    assert_eq!(rewrite_headers(&mut h, identity(), &readers), 5);
    for gone in [
        "x-svrngs-app",
        "x-svrngs-group",
        "x-svrngs-pass",
        "x-svrngs-link",
    ] {
        assert!(values(&h, gone).is_empty(), "{gone}");
    }
    assert_eq!(values(&h, "x-api-key"), ["immich-api-key-for-apps"]);
    assert_eq!(
        values(&h, "authorization"),
        ["viewer-upper"],
        "no secret declares it"
    );
    let secret = [(
        HeaderName::from_static("authorization"),
        HeaderValue::from_static(r#"MediaBrowser Token="a-key-1234""#),
    )];
    rewrite_headers(&mut h, identity(), &secret);
    assert_eq!(
        values(&h, "authorization"),
        [r#"MediaBrowser Token="a-key-1234""#],
        "the lend's credential displaces the client's, verbatim"
    );
}

/// A GET naming a protocol without `Connection: upgrade` is only that GET:
/// it asks no switch, so the door reads it as a read, and its `Upgrade`
/// stops at the door; with `Connection: Upgrade` it asks one, which the door
/// reads as a write (tests/door at 5e20d0b3, main.rs:1365-1394). Planted, watched
/// failing: `upgrade_of` reading `Upgrade` whatever `Connection` names.
#[test]
fn only_a_connection_naming_upgrade_asks_to_switch_protocols() {
    let mut bare = map(&[("upgrade", "websocket")]);
    assert_eq!(upgrade_of(&bare), None);
    strip_hop_by_hop(&mut bare);
    assert!(values(&bare, "upgrade").is_empty());
    let asked = map(&[("connection", "Upgrade, close"), ("upgrade", "websocket")]);
    assert_eq!(upgrade_of(&asked).unwrap(), "websocket");
}

/// The lend's secret is read from its store at each ask: none declared,
/// none sent; declared, sent from the next ask, its value verbatim, quotes
/// and inner space kept (tests/door at 5e20d0b3, main.rs:399, 459-461). Planted, watched
/// failing: `secret_headers` trimming the value's quotes.
#[test]
fn a_secret_is_read_from_its_store_at_each_ask_and_sent_verbatim() {
    let dir = tempfile::tempdir().unwrap();
    let picky = LendName::parse("picky").unwrap();
    assert!(secret_headers(dir.path(), &picky).is_empty());
    crate::declared::write_declared_in(dir.path(), "X-Emby-Token", "the-holders-key").unwrap();
    let sent = secret_headers(dir.path(), &picky);
    assert_eq!(sent.len(), 1);
    assert_eq!(
        (sent[0].0.as_str(), sent[0].1.as_bytes()),
        ("x-emby-token", &b"the-holders-key"[..])
    );
    let quoted = r#"MediaBrowser Token="a-key-1234""#;
    crate::declared::write_declared_in(dir.path(), "authorization", quoted).unwrap();
    let sent = secret_headers(dir.path(), &picky);
    let auth = sent.iter().find(|(n, _)| n == "authorization").unwrap();
    assert_eq!(auth.1.as_bytes(), quoted.as_bytes());
}

/// Each answer svrngs makes in a lend's place is marked its own, by kind,
/// in the header and the body: a lend not declared, a path refused or one
/// that steps out, a service not answering, a removal in its words (tests/door at 5e20d0b3, at 5e20d0b3,
/// main.rs:519-531, 1779-1789). Planted, watched failing: `said` setting no
/// header.
#[tokio::test]
async fn svrngs_s_own_answers_are_marked_by_kind_in_header_and_body() {
    for (answer, status, kind) in [
        (
            LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing"),
            404,
            "unlent",
        ),
        (
            LendRefusal::Path.say(StatusCode::FORBIDDEN, "refused"),
            403,
            "path",
        ),
        (LendRefusal::bad_path("steps out"), 400, "path"),
        (
            LendRefusal::Away.say(StatusCode::BAD_GATEWAY, "asleep"),
            502,
            "away",
        ),
        (LendRefusal::closed("not yours"), 403, "closed"),
        (LendRefusal::fault("unreadable"), 500, "fault"),
        (gone(), 403, "removed"),
    ] {
        assert_eq!(said(&answer).0, status, "{kind}");
        assert_eq!(said(&answer).1, Some(kind));
        if kind == "removed" {
            assert!(said(&answer)
                .2
                .unwrap()
                .starts_with("You are no longer in this group"));
        }
        let body = axum::body::to_bytes(answer.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["kind"], kind, "{body}");
    }
}

/// A lend that signs people in with svrngs is told the browser's origin as
/// a proxy's forwarded headers, `http` for a `*.localhost` host or the
/// room's address and `https` for a page host's name; any other lend is told
/// none; a value the client typed is displaced either way (tests/door at 5e20d0b3, at 5e20d0b3,
/// oidc.rs:632-645, 669; main.rs:1712-1720). Planted, watched failing:
/// `tell_origin` telling every lend, as at 1efa9a2d.
#[test]
fn only_a_lend_that_signs_in_is_told_the_browser_s_origin() {
    for (host, signs_in, told) in [
        ("umfrj1nx.localhost:6473", true, Some("http")),
        ("umfrj1nx.svrngs.example", true, Some("https")),
        ("192.168.1.20:7123", true, Some("http")),
        ("umfrj1nx.localhost:6473", false, None),
    ] {
        let mut h = map(&[
            ("host", host),
            ("x-forwarded-host", "evil.example"),
            ("x-forwarded-proto", "https"),
        ]);
        tell_origin(&mut h, signs_in);
        let want_host: Vec<&str> = told.map(|_| host).into_iter().collect();
        assert_eq!(
            values(&h, "x-forwarded-host"),
            want_host,
            "{host} {signs_in}"
        );
        let want_proto: Vec<&str> = told.into_iter().collect();
        assert_eq!(
            values(&h, "x-forwarded-proto"),
            want_proto,
            "{host} {signs_in}"
        );
    }
}

/// A request from the lend's own pages is told the service's own address
/// as its `Origin`, as its `Host` is, so a service that takes a post only
/// from its own address takes them; any other origin is left as sent
/// (tests/door at 5e20d0b3, main.rs:1721-1727). Planted, watched failing: `own_origin`
/// comparing the whole host, port and all, to the label.
#[test]
fn the_lend_s_own_origin_is_told_as_the_service_s_own_address() {
    let lend = port_lend("shelf", &["house"]);
    let label = Label::of(&[1; 32], "shelf");
    let port = |n| NonZeroU16::new(n).unwrap();
    for (origin, at, told) in [
        (
            format!("http://{label}.localhost:6473"),
            8096,
            "http://localhost:8096".to_string(),
        ),
        (
            format!("https://{label}.svrngs.example"),
            80,
            "http://localhost".to_string(),
        ),
        (
            "http://picks.localhost:6473".into(),
            8096,
            "http://picks.localhost:6473".into(),
        ),
    ] {
        let mut h = map(&[("origin", &origin)]);
        own_origin(&mut h, &label, (&lend, port(at)));
        assert_eq!(values(&h, "origin"), [told.as_str()], "{origin}");
    }
}

/// A service's redirect to its own address comes back a path on the lend,
/// its query kept; one a browser reads as another host's, `//` or `/\` after
/// that address, is no path, so the door leaves it as sent (tests/door at 5e20d0b3, at 5e20d0b3,
/// main.rs:1646-1651, 1667-1680). Planted, watched failing: `LendPath::parse`
/// without its `//` and `\` check, as at 3f313267.
#[test]
fn a_redirect_off_the_lend_s_origin_is_never_made_a_path() {
    let port = NonZeroU16::new(8096).unwrap();
    let path = |url: &str| LendPath::parse(url, port, None).map(|p| p.location([]));
    assert_eq!(
        path("http://localhost:8096/landed?x=1").unwrap(),
        "/landed?x=1"
    );
    for off in ["//evil.example/cb", "/\\evil.example/cb"] {
        let sent = format!("http://localhost:8096{off}");
        assert_eq!(
            path(&sent).err(),
            Some("would send you to another address"),
            "{sent}"
        );
    }
}

/// The mixed-version fence on what is lent: an ask with no marker, or an
/// older one, or one that does not read, is not scope-aware; 5 and later are
/// (tests/door at 5e20d0b3, scope.rs:157-163). Planted, watched failing: 4 read as aware.
#[test]
fn a_list_ask_is_scope_aware_only_marked_at_5_or_later() {
    for (marker, aware) in [
        (None, false),
        (Some("4"), false),
        (Some("five"), false),
        (Some("5"), true),
        (Some("6"), true),
    ] {
        let mut h = HeaderMap::new();
        if let Some(m) = marker {
            h.insert(crate::version::SCOPE_HEADER, HeaderValue::from_static(m));
        }
        assert_eq!(scope_aware(&h), aware, "{marker:?}");
    }
}

/// A newcomer past the limit is told why, how many, and that the number is
/// the lender's choice, in a 503 marked `full` with the figures (tests/door at 5e20d0b3, at 5e20d0b3,
/// limit.rs:186-200). Planted, watched failing: "people" said of one.
#[tokio::test]
async fn a_newcomer_past_the_limit_is_told_the_number_and_whose_it_is() {
    let shelf = port_lend("shelf", &["house"]);
    let limit = |n| NonZeroU16::new(n).unwrap();
    let words = busy_words(&shelf, 3, limit(3));
    assert!(
        words.starts_with("shelf is lent to 3 people at once, and 3 are using it now"),
        "{words}"
    );
    assert!(words.ends_with(
        "That number is the choice of the person who lends it, and they can change it."
    ));
    let one = busy_words(&shelf, 1, limit(1));
    assert!(
        one.starts_with("shelf is lent to 1 person at once, and 1 is using it now"),
        "{one}"
    );
    let answer = busy(&shelf, 3, limit(3));
    assert_eq!(said(&answer).0, 503);
    assert_eq!(said(&answer).1, Some("full"));
    let body = axum::body::to_bytes(answer.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        (body["using"].as_u64(), body["limit"].as_u64()),
        (Some(3), Some(3))
    );
}

/// A guest is told as their host's guest, with an account of their own
/// that no member's can be; a member as themselves (tests/door at 5e20d0b3, at 5e20d0b3,
/// guests.rs:109-127; oidc.rs:879-884). Planted, watched failing: `told_as`
/// leaving off `.guest`.
#[test]
fn a_guest_is_told_as_their_host_s_guest_with_an_account_of_their_own() {
    let [sam, kim, bo] = ["Sam", "Kim", "bo"].map(|p| Person::parse(p).unwrap());
    let key = actor(9);
    let (told, user) = told_as(&sam, Some(&kim), &key);
    assert_eq!(told, "Sam (Kim's guest)");
    assert_eq!(user, format!("Sam-{}.guest", &key[..16]));
    assert!(!told.contains(&actor(1)) && !user.contains(&actor(1)));
    let (told, user) = told_as(&bo, None, &actor(2));
    assert_eq!((told.as_str(), user), ("bo", user_name(&bo, &actor(2))));
}

const HOUSE: &str = "house";

/// Kim's house as Kim's computer, the lender, holds it: Alex keeps it, Kim
/// (key 1) and Bo (key 2) stand in it.
fn kims_house() -> (tempfile::TempDir, RingRail) {
    held(
        1,
        &[(HOUSE, &[("Alex", &[7]), ("Kim", &[1]), ("Bo", &[2])])],
    )
}

/// Kim has Sam (visit key 9) over until `until`, the visit opening each
/// lender's lends `opens` names: its id and time.
fn sams_visit(rail: &RingRail, until: i64, opens: &[(u8, &str)]) -> (crate::OpId, i64) {
    let mut named = std::collections::BTreeMap::<String, std::collections::BTreeSet<String>>::new();
    for (lender, lend) in opens {
        named
            .entry(actor(*lender))
            .or_default()
            .insert(lend.to_string());
    }
    let visit = crate::Visit {
        guest: Person::parse("Sam").unwrap(),
        key: actor(9),
        opens: named,
        until,
        consent: svrngs_core::tests_support::consent(
            &crate::SigningKey::from_bytes(&[9; 32]),
            &crate::actor_of(rail.signer()),
        ),
        under: None,
    };
    let act = crate::RailAct::Visit(visit);
    wrote(rail, HOUSE, &svrngs_core::tests_support::Fixture { act })
}

/// `guest_of` at `now` on the visit Kim's rail holds now, from the door
/// whose key is `me`: who is let in, or the refusal's kind and words.
fn guest_at(rail: &RingRail, me: u8, lend: &Lend, now: i64) -> Result<String, (String, String)> {
    let visit = rail.visit_anywhere(&actor(9)).expect("the visit is held");
    match guest_of(&actor(me), visit, &actor(9), lend, now) {
        Ok((group, host, guest)) => Ok(format!("{guest} in {group}, {host}'s")),
        Err(r) => {
            let (_, kind, words) = said(&r);
            Err((kind.unwrap().to_string(), words.unwrap().to_string()))
        }
    }
}

/// A guest is let in only to a lend of their host's own, on a port, that
/// the visit names, in a group the lend still reaches, and only while the
/// visit is on: begun, though timed a minute ahead of the door's clock
/// (#214), and before its time, its host's Stop and its host's leave, each
/// end said (tests/door at 5e20d0b3, guests.rs:157, 197-205, 257-266; oidc.rs:797-803).
/// The time is an argument, so no visit is waited out. Planted, watched
/// failing, one at a time: `guest_of` not asking `stopped`; letting a visit
/// in at any lend it names, its host's or not (`own` true).
#[test]
fn a_guest_is_let_in_only_to_their_host_s_named_lend_while_the_visit_is_on() {
    let (_dir, kims) = kims_house();
    let until = unix_now().unwrap() + 24 * 3600;
    let (visit, at) = sams_visit(&kims, until, &[(1, "jellyfin"), (2, "jellyfin")]);
    let jellyfin = port_lend("jellyfin", &[HOUSE]);
    let sam = Ok("Sam in house, Kim's".to_string());
    let closed = |w: &str| Err(("closed".to_string(), w.to_string()));
    let ended = |w: &str| Err(("ended".to_string(), w.to_string()));
    assert_eq!(guest_at(&kims, 1, &jellyfin, at), sam);
    assert_eq!(
        guest_at(&kims, 1, &jellyfin, at - 60),
        sam,
        "timed a minute ahead"
    );
    assert_eq!(
        guest_at(&kims, 2, &jellyfin, at),
        closed("jellyfin is not Kim's to open to you"),
        "Bo's door"
    );
    assert_eq!(
        guest_at(&kims, 1, &port_lend("films", &[HOUSE]), at),
        closed("Kim did not open films to you")
    );
    let carried = Lend::carried("jellyfin", HOUSE).unwrap();
    let theirs = guest_at(&kims, 1, &carried, at).unwrap_err();
    assert!(
        theirs.1.ends_with("is not Kim's to open to you"),
        "{theirs:?}"
    );
    assert_eq!(
        guest_at(&kims, 1, &port_lend("jellyfin", &["yard"]), at),
        closed("jellyfin is no longer lent to your visit's group")
    );
    assert_eq!(
        guest_at(&kims, 1, &jellyfin, until),
        ended("your visit with Kim is over: its time is up")
    );
    let act = crate::RailAct::Correct {
        corrects: visit,
        replacement: None,
    };
    let (_, stopped) = wrote(&kims, HOUSE, &svrngs_core::tests_support::Fixture { act });
    assert_eq!(
        guest_at(&kims, 1, &jellyfin, stopped - 1),
        sam,
        "before its Stop"
    );
    assert_eq!(
        guest_at(&kims, 1, &jellyfin, stopped),
        ended("Kim ended your visit")
    );

    let (_dir, kims) = kims_house();
    sams_visit(&kims, until, &[(1, "jellyfin")]);
    let (_, left) = wrote(&kims, HOUSE, &crate::Leave(actor(1)));
    assert_eq!(
        guest_at(&kims, 1, &jellyfin, left),
        ended("Kim is no longer a member, so your visit ended")
    );
}

/// The withdrawal sweep and a guest's redemption ask [`guest_of`] again at
/// the door's clock: a live visit's guest is admitted to their host's
/// named lend, and not once the lend reaches the visit's group no more, nor
/// a key with no visit here (tests/door at 5e20d0b3, guests.rs:414-422; oidc.rs:797-803).
/// Planted, watched failing: `guest_admitted` answering true for any visit.
#[test]
fn a_guest_is_admitted_again_by_the_one_decision_that_let_them_in() {
    let (_dir, kims) = kims_house();
    sams_visit(&kims, unix_now().unwrap() + 24 * 3600, &[(1, "radio")]);
    assert!(guest_admitted(
        &kims,
        &actor(9),
        &port_lend("radio", &[HOUSE])
    ));
    assert!(!guest_admitted(&kims, &actor(9), &port_lend("radio", &[])));
    assert!(!guest_admitted(
        &kims,
        &actor(5),
        &port_lend("radio", &[HOUSE])
    ));
}

/// A lend reaches a member only through a group it is lent to that both
/// its lender and the member stand in, whatever else either stands in:
/// lent to yard alone, bo of the house and trip reaches nothing, and a
/// third group joined widens nothing; lent to the house and yard, bo
/// reaches it through the house, and the house alone is the group whose
/// page the service is told; lent to `bos`, which names bo and not ada,
/// nobody reaches it (tests/door at 5e20d0b3, scope.rs:60-70, 101-106, 141-148, 346-348,
/// 367-378). Planted, watched failing, one at a time: `reached` not asking
/// `stands_here`; `member_for` resolving through any group held here.
#[test]
fn a_lend_reaches_a_member_only_through_a_group_both_stand_in() {
    let (ada_bo, ada_cy): (People, People) = (
        &[("ada", &[1]), ("bo", &[2])],
        &[("ada", &[1]), ("cy", &[3])],
    );
    let (_dir, a) = held(
        1,
        &[
            (HOUSE, ada_bo),
            ("trip", ada_bo),
            ("yard", ada_cy),
            ("bos", &[("bo", &[2])]),
        ],
    );
    let through = |lend: &Lend, who: u8| {
        member_for(&a, lend, &actor(who)).map(|(g, m)| format!("{} in {g}", m.person))
    };
    let yard = port_lend("shelf", &["yard"]);
    assert_eq!(through(&yard, 2), None, "lent to yard alone");
    assert_eq!(through(&yard, 3).as_deref(), Some("cy in yard"));
    let both = port_lend("shelf", &[HOUSE, "yard"]);
    assert_eq!(through(&both, 2).as_deref(), Some("bo in house"));
    let told: Vec<String> = (a.namespaces().unwrap().into_iter())
        .filter(|g| reached(&a, &both, g) && a.standing(g).is_ok_and(|s| s.names(&actor(2))))
        .collect();
    assert_eq!(told, [HOUSE], "the groups whose page bo's request is told");
    assert!(stands_here(&a, HOUSE) && !stands_here(&a, "bos"));
    let bos = port_lend("shelf", &["bos"]);
    assert!(!reached(&a, &bos, "bos"));
    assert_eq!(
        through(&bos, 2),
        None,
        "a group the lender does not stand in"
    );
    a.set_seed(
        "club",
        &crate::Roster::new(
            ada_bo
                .iter()
                .map(|(p, k)| {
                    (
                        Person::parse(p).unwrap(),
                        k.iter().map(|k| actor(*k)).collect(),
                    )
                })
                .collect(),
        ),
    )
    .unwrap();
    assert_eq!(
        through(&yard, 2),
        None,
        "a third group joined widens nothing"
    );
}

/// A service is told, in `x-svrngs-pages`, the page of each group the
/// device asking stands in that the lend reaches, `none` for one keeping
/// no page, so an app writes a link into itself that opens on a phone
/// (events-links (a); browser/main.rs at 58b950a3,
/// `a_lent_app_writes_a_link_into_itself_that_opens_on_a_phone`, whose
/// computer was told `dweb=none house=<page>` and bo's phone the house's
/// alone). Planted, watched failing: a group keeping no page told unread
/// (`map_or("unread"…)`); every group held told, whoever asks (`stands`
/// without its `standing`).
#[test]
fn a_service_is_told_the_page_of_each_group_its_asker_stands_in() {
    let (ada_bo, ada): (People, People) = (&[("ada", &[1]), ("bo", &[2])], &[("ada", &[1])]);
    let (_dir, a) = held(1, &[(HOUSE, ada_bo), ("dweb", ada), ("trip", ada_bo)]);
    let page = "https://group.example:8443/";
    let origin = svrngs_core::link::Origin::parse(page).unwrap();
    crate::group::write_origin(&a.group_dir(HOUSE).unwrap(), &origin).unwrap();
    let app = port_lend("app", &[HOUSE, "dweb"]);
    let told = |who: u8| {
        let value = pages(&a, &DeviceKey(actor(who)), &app);
        value.map(|v| v.to_str().unwrap().to_string())
    };
    assert_eq!(
        told(1),
        Some(format!("dweb=none {HOUSE}={page}")),
        "the computer, in both groups the lend reaches"
    );
    assert_eq!(
        told(2),
        Some(format!("{HOUSE}={page}")),
        "bo's phone, in the house alone"
    );
}

/// One person's two devices are one person to a lend: each device's key
/// resolves to the person's first key, which the door counts, keeps choices
/// by and tells as `x-svrngs-key`, so one account (tests/door at 5e20d0b3, at 5e20d0b3,
/// main.rs:355-358; limit.rs:225-227; choices.rs:224). Planted, watched
/// failing: `member_for` answering the device's own key as the person's.
#[test]
fn one_person_s_two_devices_resolve_to_one_person_key() {
    let (_dir, a) = held(1, &[(HOUSE, &[("ada", &[1]), ("bo", &[2, 6])])]);
    let shelf = port_lend("shelf", &[HOUSE]);
    for device in [2, 6] {
        let (_, member) = member_for(&a, &shelf, &actor(device)).unwrap();
        assert_eq!(member.person_key, actor(2), "device {device}");
        assert_eq!(member.person.as_str(), "bo");
    }
    let mut counts = Counts::default();
    let mut one = port_lend("one", &[HOUSE]);
    one.terms.limit = NonZeroU16::new(1);
    let t0 = now();
    for device in [2, 6] {
        let (_, member) = member_for(&a, &one, &actor(device)).unwrap();
        let verdict = counts.ask(&one, &member.person_key, Whereabouts::Away, t0);
        assert!(
            matches!(verdict, Verdict::In { .. }),
            "device {device}: {verdict:?}"
        );
    }
}

/// An app reading a lend past its rate is refused before its person is
/// counted or its service asked, and told so in words, by whose computer,
/// with a `Retry-After` (browser::acts::a_phone_s_page_reading_past_the_
/// rate_is_refused_with_the_words, its door half: a phone's page read cy's
/// shelf 2,000 times). Ana's app reads shelf: its first READS_A_MINUTE
/// asks are let in, the next is `TooFast`, holding no use, while a read
/// with no app, the person's own, is never held to the rate; the
/// refusal is 429 `limit`, its wait in whole seconds.
#[tokio::test]
#[expect(
    clippy::disallowed_methods,
    reason = "the rate refills on the real clock"
)]
async fn an_app_past_its_rate_is_refused_with_the_words_and_when_to_read_again() {
    let shelf = port_lend("shelf", &["house"]);
    let app = Label::of(&[1; 32], "page");
    let using = Using::spawn();
    // The rate refills on the real clock (door.rs `now`), so what it lets
    // in is judged by how long the asks took, as Reads' own test does.
    let began = std::time::Instant::now();
    let (mut taken, mut refused) = (0u64, None);
    while refused.is_none() && taken < 2 * u64::from(READS_A_MINUTE) {
        let (verdict, held) = using
            .ask(&shelf, &actor(7), Whereabouts::Away, Some(&app))
            .await
            .unwrap();
        match verdict {
            Verdict::In { .. } => taken += 1,
            Verdict::TooFast(wait) => refused = Some((wait, held.is_none())),
            other => panic!("read {taken}: {other:?}"),
        }
    }
    let burst = u64::from(READS_A_MINUTE);
    let ms = u64::try_from(began.elapsed().as_millis()).unwrap();
    assert!(
        (burst..=burst + ms * burst / 60_000 + 1).contains(&taken),
        "{burst} at once, then ten a second: {taken} taken in {ms} ms"
    );
    let Some((wait, unheld)) = refused else {
        panic!("never refused: {taken} taken in {ms} ms")
    };
    assert!(unheld && wait > Duration::ZERO, "{wait:?}");
    let own = using.ask(&shelf, &actor(7), Whereabouts::Away, None).await;
    assert!(
        matches!(own, Some((Verdict::In { .. }, Some(_)))),
        "a person's own read is not an app's"
    );

    let cy = Person::parse("cy").ok();
    let answer = read_too_fast(cy, &shelf, Duration::from_millis(100));
    assert_eq!((said(&answer).0, said(&answer).1), (429, Some("limit")));
    assert_eq!(answer.headers()[header::RETRY_AFTER], "1");
    assert_eq!(
        said(&answer).2,
        Some("This app is reading shelf faster than an app may, about 600 times a minute, because cy’s computer serves it. It can read again in 1 second. Read once and keep the answer, rather than on a timer.")
    );
    let answer = read_too_fast(None, &shelf, Duration::from_millis(2_500));
    assert_eq!(answer.headers()[header::RETRY_AFTER], "3");
    assert!(said(&answer)
        .2
        .is_some_and(|w| w
            .contains("because the lender’s computer serves it. It can read again in 3 seconds.")));
}

/// The socket `id` alone on a share's port, on 127.0.0.1, in cgroup 4.
fn alone(id: u64) -> binding::Read {
    binding::Read {
        boot: None,
        lasting: true,
        sockets: vec![binding::Socket {
            id,
            cgroup: Some(4),
            holders: Default::default(),
            at: std::net::Ipv4Addr::LOCALHOST.into(),
            dual: false,
            peer: None,
        }],
        unlisted: false,
    }
}

/// **What the door writes to a share's port is what its lender approved**
/// (SHARE_BINDING §5.2): `decided` takes `binding::check`'s answer and holds
/// the rest. A lend with no approval is held unapproved; one whose port
/// reads another socket, or whose connection another took, is held as
/// another's; a port that reads nothing is away; a read that failed, and a
/// system with no reader, are unjudged; what was approved carries. Planted,
/// watched failing: a failed read carried as approved (`Err(_) => Ok(())`).
#[test]
fn what_the_door_writes_to_a_share_s_port_is_what_its_lender_approved() {
    use crate::uid::Unread;
    use bound::Unsent;
    use eudoxia_community::binding::{Read, Took};
    let dir = tempfile::tempdir().unwrap();
    let decided = |lend: &Lend, read| bound::decided(dir.path(), lend, read);
    let mut shelf = Lend::carried("books", "house").unwrap();
    shelf.source = Source::Port(std::num::NonZeroU16::new(9001).unwrap());
    let held = |h| Err(Unsent::Held(h));
    assert_eq!(
        decided(&shelf, Ok((alone(1), Took::Cgroup(4)))),
        held(Hold::Unapproved)
    );
    shelf.terms.approved = binding::approve(&alone(1), None).ok();
    assert_eq!(decided(&shelf, Ok((alone(1), Took::Cgroup(4)))), Ok(()));
    assert_eq!(decided(&shelf, Ok((alone(1), Took::Datagram))), Ok(()));
    assert_eq!(
        decided(&shelf, Ok((alone(2), Took::Cgroup(4)))),
        held(Hold::Another)
    );
    assert_eq!(
        decided(&shelf, Ok((alone(1), Took::Cgroup(5)))),
        held(Hold::Another)
    );
    let empty = Read {
        sockets: Vec::new(),
        ..alone(1)
    };
    assert_eq!(
        decided(&shelf, Ok((empty, Took::Datagram))),
        Err(Unsent::Away)
    );
    let failed = Err(Unread::Failed("this computer's sockets can't be listed"));
    assert_eq!(decided(&shelf, failed), held(Hold::Unjudged));
    assert_eq!(decided(&shelf, Err(Unread::NoReader)), held(Hold::Unjudged));
}

/// **A program not approved, once seen, holds a UDP share in its file until
/// its lender approves again** (SHARE_BINDING_REVIEWS §6 #3, §7 #1): nothing
/// there, a failed read, and a socket whose holder can't be read hold
/// nothing past themselves; another socket seen holds the share, read from
/// its file again, with the approved socket back alone and for the lender's
/// own list; an approval made since is never held for what was seen before
/// it, and carries. A TCP share's connection is judged alone: another
/// socket seen holds that one, and the approved one back alone carries. On
/// a Mac, a socket beside the approved one whose last user has exited is
/// unjudged, and holds nothing past itself. Planted, each watched failing:
/// `decided` not keeping the hold (the approved socket back alone carried);
/// `hold` not comparing the approval judged (the approval made since held);
/// the share's protocol not asked (the TCP share held in its file); a
/// socket with no holder read taken as another's (the Mac share held).
#[test]
fn a_hold_is_kept_in_the_lend_s_file_until_its_lender_approves_again() {
    use bound::Unsent;
    use eudoxia_community::binding::Took;
    let dir = tempfile::tempdir().unwrap();
    let mut shelf = Lend::carried("books", "house").unwrap();
    shelf.source = Source::Raw(Raw::Udp, std::num::NonZeroU16::new(9001).unwrap());
    shelf.terms.approved = binding::approve(&alone(1), None).ok();
    crate::lend::declare(dir.path(), &shelf, None).unwrap();
    let now = || crate::lend::read(dir.path(), &shelf.name).unwrap().unwrap();
    let decided =
        |lend: &Lend, id| bound::decided(dir.path(), lend, Ok((alone(id), Took::Datagram)));
    let empty = binding::Read {
        sockets: Vec::new(),
        ..alone(1)
    };
    let away = bound::decided(dir.path(), &now(), Ok((empty, Took::Datagram)));
    assert_eq!(away, Err(Unsent::Away));
    let failed = Err(crate::uid::Unread::Failed("unread"));
    let unjudged = bound::decided(dir.path(), &now(), failed);
    assert_eq!(unjudged, Err(Unsent::Held(Hold::Unjudged)));
    assert_eq!(decided(&now(), 1), Ok(()), "neither kept");
    assert_eq!(decided(&now(), 2), Err(Unsent::Held(Hold::Another)));
    let back = decided(&now(), 1);
    assert_eq!(
        back,
        Err(Unsent::Held(Hold::Another)),
        "the approved socket back alone"
    );
    let since = binding::approve(&alone(3), None).unwrap();
    crate::share::reapproved(dir.path(), &shelf.name, since).unwrap();
    let stale = decided(&shelf, 2);
    assert!(
        stale.is_err(),
        "a check judged against the approval before it"
    );
    assert_eq!(
        decided(&now(), 3),
        Ok(()),
        "the approval made since carries"
    );
    let mut page = Lend::carried("page", "house").unwrap();
    page.source = Source::Port(std::num::NonZeroU16::new(9002).unwrap());
    page.terms.approved = binding::approve(&alone(1), None).ok();
    crate::lend::declare(dir.path(), &page, None).unwrap();
    let page = || crate::lend::read(dir.path(), &page.name).unwrap().unwrap();
    assert_eq!(decided(&page(), 2), Err(Unsent::Held(Hold::Another)));
    assert_eq!(
        decided(&page(), 1),
        Ok(()),
        "a TCP share's connection alone"
    );
    let mut mac = alone(1);
    mac.lasting = false;
    mac.sockets[0].holders = [(5, 5)].into();
    let approved = binding::approve(&mac, None).unwrap();
    crate::share::reapproved(dir.path(), &shelf.name, approved).unwrap();
    let mut exited = mac.clone();
    exited.sockets.push(binding::Socket {
        holders: Default::default(),
        ..alone(2).sockets[0].clone()
    });
    let unjudged = bound::decided(dir.path(), &now(), Ok((exited, Took::Datagram)));
    assert_eq!(
        unjudged,
        Err(Unsent::Held(Hold::Unjudged)),
        "its last user exited"
    );
    assert_eq!(
        bound::decided(dir.path(), &now(), Ok((mac, Took::Datagram))),
        Ok(())
    );
}

/// **A hold waits a moment for another change holding the lend's lock**
/// (SHARE_BINDING_REVIEWS §7 #5): the lend's lock held by another change
/// for 5 ms, a program not approved seen meanwhile, and the hold is kept in
/// its file once that change lets go. Planted, watched failing: `hold`
/// asking once (the hold lost while the lock was held).
#[test]
fn a_hold_waits_a_moment_for_another_change_holding_the_lend_s_lock() {
    let dir = tempfile::tempdir().unwrap();
    let mut shelf = Lend::carried("books", "house").unwrap();
    shelf.source = Source::Raw(Raw::Udp, std::num::NonZeroU16::new(9001).unwrap());
    shelf.terms.approved = binding::approve(&alone(1), None).ok();
    crate::lend::declare(dir.path(), &shelf, None).unwrap();
    std::fs::create_dir_all(dir.path().join("locks")).unwrap();
    let lock = dir.path().join(format!("locks/{}.lock", shelf.name));
    let lock = std::fs::File::create(lock).unwrap();
    lock.lock().unwrap();
    let other = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(5));
        lock.unlock().unwrap();
    });
    let seen = Ok((alone(2), binding::Took::Datagram));
    let checked = bound::decided(dir.path(), &shelf, seen);
    other.join().unwrap();
    assert_eq!(checked, Err(bound::Unsent::Held(Hold::Another)));
    let kept = crate::lend::read(dir.path(), &shelf.name).unwrap().unwrap();
    assert!(
        kept.terms.approved.unwrap().held,
        "the hold kept once the lock let go"
    );
}

/// **Every dial of a share's port goes through its check** (SHARE_BINDING
/// §5.7, SHARE_BINDING_REVIEWS §6 #12): clippy refuses each way to dial, to
/// send a datagram to an address, or to build an HTTP client (clippy.toml),
/// but where an `expect` allows it; each allowance of a disallowed method in
/// svrngs's product code is named here, by file and reason, the clock's and
/// randomness's among them. One added fails here until it is named, which
/// is when it is reviewed; an `allow`, an inner `#![expect]` or `#![allow]`,
/// and one of the lint's group (`clippy::style`, `clippy::all`) fail here
/// always. Planted, each watched failing: an unlisted `TcpStream::connect`
/// in share.rs `tried` (clippy, `-D warnings`); the same under an `expect`
/// of its own (here); a `#![expect(clippy::disallowed_methods)]` atop
/// share.rs, and an `#[allow(clippy::style)]` on `tried` (here).
#[test]
fn every_dial_of_a_share_s_port_goes_through_its_check() {
    let named: &[(&str, &str)] = &[
        // Dials, each writing nothing to a share's port unchecked.
        ("crates/svrngs-node/src/door/bound.rs", "dial: checked by bound"),
        ("crates/svrngs-node/src/door/bound.rs", "dial: a checked flow"),
        ("crates/svrngs-node/src/door.rs", "dial: unwritten probe"),
        ("crates/svrngs-node/src/door/udp.rs", "dial: member's programs"),
        ("crates/svrngs-node/src/share.rs", "dial: unwritten probe"),
        ("crates/svrngs-node/src/uid.rs", "dial: unwritten probe"),
        ("crates/svrngs-node/src/tiers.rs", "dial: a relay"),
        ("crates/svrngs-node/src/network.rs", "dial: the route out"),
        ("crates/svrngs-node/src/update.rs", "dial: the update page"),
        ("crates/svrngs-node/src/publication.rs", "dial: a group's page"),
        ("crates/svrngs-node/src/http.rs", "dial: the one client"),
        ("crates/svrngs-node/src/http.rs", "dial: the one client"),
        ("crates/svrngs/src/cli.rs", "dial: svrngs's own API"),
        ("crates/svrngs/src/cli.rs", "dial: a group's page"),
        // The clock and randomness, at their adapters (clippy.toml).
        ("apps/give/src/main.rs", "the keeper program's clock, read here and passed in"),
        ("apps/give/src/runtime.rs", "a snapshot is named for when it was taken"),
        ("apps/give/src/runtime.rs", "a snapshot times its pause"),
        ("apps/give/src/runtime.rs", "the runtime is podman's adapter: it waits on the clock"),
        ("crates/svrngs-node/src/api.rs", "draws the record's token"),
        ("crates/svrngs-node/src/blob.rs", "a get's lapse is read from the clock at the route, its adapter"),
        ("crates/svrngs-node/src/blob.rs", "a put's lapse is read from the clock at the route, its adapter"),
        ("crates/svrngs-node/src/blob.rs", "what is left of a lapse is read from the clock at the route, its adapter"),
        ("crates/svrngs-node/src/confined.rs", "the filesystem adapter draws an unguessable partial file name"),
        ("crates/svrngs-node/src/declared.rs", "the secret-file adapter draws a fresh partial name, never reusing a readable inode"),
        ("crates/svrngs-node/src/door.rs", "the door's and the relay's clock: whether a pass has lapsed, and until when a computer's own lasts"),
        ("crates/svrngs-node/src/folder.rs", "the fetch is an adapter: it times what a holder takes, and when one did not answer"),
        ("crates/svrngs-node/src/folder.rs", "the sweep's adapter reads when it asks, so a lapse is read against it"),
        ("crates/svrngs-node/src/folder.rs", "times each get's read of the record for its trace (perf-1); no decision reads it"),
        ("crates/svrngs-node/src/folder.rs", "times the fetch's dial, first byte and last byte for their trace (perf-1); no decision reads it"),
        ("crates/svrngs-node/src/group.rs", "draws a group's salt"),
        ("crates/svrngs-node/src/invite.rs", "a link's life starts now"),
        ("crates/svrngs-node/src/invite.rs", "draws a link's secret"),
        ("crates/svrngs-node/src/oidc/provider.rs", "the provider is the adapter that reads the clock for a code and a token"),
        ("crates/svrngs-node/src/ring_sync.rs", "times a round for its trace; no decision reads it"),
        ("crates/svrngs-node/src/session.rs", "the session adapter draws a boot challenge to prevent cross-restart replay"),
        ("crates/svrngs-node/src/synced.rs", "the loop is an adapter: it reads the clock for when the next sweep is due"),
        ("crates/svrngs-node/src/synced.rs", "times each fetch and each export for their traces (perf-1); no decision reads it"),
        ("crates/svrngs-node/src/synced.rs", "times each phase of a step for its trace (perf-1); no decision reads it"),
        ("crates/svrngs-node/src/uid.rs", "the reader's wait for an accept, at the edge"),
        ("crates/svrngs-node/src/webrtc.rs", "str0m's own clock: the peer's loop times itself, as the spike's"),
        ("crates/svrngs-node/src/webrtc.rs", "str0m's own clock: the peer's loop times itself, as the spike's"),
        ("crates/svrngs-runtime/src/journal.rs", "the write side's clock adapter"),
        ("crates/svrngs/src/cli.rs", "draws a name nobody made, to ask the page's host for the page there"),
        ("crates/svrngs/src/identity.rs", "draws a device key"),
    ];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut found, mut allowed) = (Vec::new(), Vec::new());
    let crates = ["svrngs-core", "svrngs-runtime", "svrngs-node", "svrngs"];
    let mut dirs: Vec<_> = crates
        .iter()
        .map(|c| root.join("crates").join(c).join("src"))
        .collect();
    dirs.push(root.join("apps/give/src"));
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let (path, name) = (
                entry.path(),
                entry.file_name().to_string_lossy().into_owned(),
            );
            if name.contains("test") {
                continue;
            } else if path.is_dir() {
                dirs.push(path);
                continue;
            } else if !name.ends_with(".rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            let at = path.strip_prefix(&root).unwrap().display().to_string();
            // Each attribute, inner or outer, naming the lint or a group
            // holding it: an outer `expect` of the lint alone is named,
            // and any other is refused.
            let lints = ["clippy::disallowed_methods", "clippy::style", "clippy::all"];
            for (start, _) in text.match_indices("#[").chain(text.match_indices("#![")) {
                let end = text[start..].find(")]").map_or(text.len(), |e| start + e);
                let attr = text[start..end]
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                if !lints.iter().any(|lint| attr.contains(lint)) {
                    continue;
                }
                let reason = attr
                    .split("reason = \"")
                    .nth(1)
                    .and_then(|r| r.split('"').next());
                let bare = attr.replace(' ', "");
                if bare.starts_with("#[expect(clippy::disallowed_methods,reason=") {
                    found.push((at.clone(), reason.unwrap_or_default().to_string()));
                } else {
                    allowed.push(format!("{at}: {attr}"));
                }
            }
        }
    }
    found.sort();
    let mut want: Vec<(String, String)> = named
        .iter()
        .map(|(f, r)| (f.to_string(), r.to_string()))
        .collect();
    want.sort();
    assert_eq!(
        allowed,
        Vec::<String>::new(),
        "an `allow`, an inner `expect`, or one of the lint's group, where only a named `expect` of the lint may be"
    );
    assert_eq!(found, want, "an allowance of a disallowed method named or gone: name each dial with why it writes nothing to a share's port unchecked");
}

/// A real UDP service on 127.0.0.1, its share approved as the wire reads
/// it, and a flow to it that has sent one datagram: the service, where the
/// flow's answers go, the wire, its computer's key and transport, the share,
/// the folder its lend is kept in, and the flow.
#[allow(clippy::type_complexity)]
async fn a_flow() -> (
    tokio::net::UdpSocket,
    std::net::SocketAddr,
    (crate::node::Wire, crate::iroh::PublicKey),
    Lend,
    tempfile::TempDir,
    bound::Flow,
) {
    let service = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let at = service.local_addr().unwrap();
    let wire = crate::node::Wire::default();
    let key = crate::iroh::SecretKey::from_bytes(&[7; 32]).public();
    let net = wire.net(key);
    let mut shelf = Lend::carried("books", "house").unwrap();
    shelf.source = Source::Raw(Raw::Udp, std::num::NonZeroU16::new(at.port()).unwrap());
    let read = crate::iroh::Net::read_port(&net, Raw::Udp, at.port(), None).await;
    shelf.terms.approved = binding::approve(&read.unwrap().0, None).ok();
    let dir = tempfile::tempdir().unwrap();
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let using = Using::spawn();
    let flow = bound::Flow::open(socket, (at, &using, &shelf.name))
        .await
        .unwrap();
    let (_queue, mut more) = tokio::sync::mpsc::channel(1);
    let judged = (&net as &dyn crate::iroh::Net, dir.path(), &shelf);
    flow.send(judged, b"hi".to_vec(), &mut more).await.unwrap();
    let mut buf = [0u8; 16];
    let (_, from) = service.recv_from(&mut buf).await.unwrap();
    (service, from, (wire, key), shelf, dir, flow)
}

/// **A server's datagram socket for a flow, made in the approved cgroup, is
/// carried** (eudoxia 4560910, `Took::Flow`): rootless Podman's pasta answers
/// each flow on a published UDP port from a socket of its own on that port,
/// connected to the flow's socket alone. Two flows open on a lend, each such
/// socket in the approved socket's cgroup, a check of either carries; one in
/// another cgroup holds it. Planted, each watched failing: `Took::Datagram`
/// passed for a flow (the server's socket held); only the flow's own address
/// named (the other flow's socket held).
#[tokio::test]
async fn a_server_s_socket_for_a_flow_in_the_approved_cgroup_is_carried() {
    let service = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let at = service.local_addr().unwrap();
    let wire = crate::node::Wire::default();
    let key = crate::iroh::SecretKey::from_bytes(&[8; 32]).public();
    let net = wire.net(key);
    let mut shelf = Lend::carried("game", "house").unwrap();
    shelf.source = Source::Raw(Raw::Udp, std::num::NonZeroU16::new(at.port()).unwrap());
    shelf.terms.approved = binding::approve(&alone(1), None).ok();
    let dir = tempfile::tempdir().unwrap();
    let (using, judged) = (
        Using::spawn(),
        (&net as &dyn crate::iroh::Net, dir.path(), &shelf),
    );
    let pasta = |answering: Vec<(std::net::SocketAddr, u64)>| {
        let mut read = alone(1);
        for (n, (peer, cgroup)) in (2..).zip(answering) {
            let socket = binding::Socket {
                cgroup: Some(cgroup),
                peer: Some(peer),
                ..alone(n).sockets[0].clone()
            };
            read.sockets.push(socket);
        }
        wire.reads(key, at.port(), Ok((read, binding::Took::Datagram)));
    };
    pasta(Vec::new());
    let (mut flows, mut from) = (Vec::new(), Vec::new());
    for _ in 0..2 {
        let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let flow = bound::Flow::open(socket, (at, &using, &shelf.name))
            .await
            .unwrap();
        let (_queue, mut more) = tokio::sync::mpsc::channel(1);
        flow.send(judged, b"hi".to_vec(), &mut more).await.unwrap();
        from.push(service.recv_from(&mut [0u8; 16]).await.unwrap().1);
        flows.push(flow);
    }
    pasta(vec![(from[0], 4)]);
    assert_eq!(flows[0].check(judged).await, Ok(()), "its server's socket");
    pasta(vec![(from[0], 4), (from[1], 4)]);
    assert_eq!(
        flows[0].check(judged).await,
        Ok(()),
        "beside the other flow's"
    );
    pasta(vec![(from[0], 5)]);
    assert_eq!(
        flows[0].check(judged).await,
        Err(bound::Unsent::Held(Hold::Another)),
        "one in another cgroup"
    );
}

/// **A UDP share's answers are carried back only through its check**
/// (SHARE_BINDING §5.2): a datagram the share's port answers a flow is
/// given once the port's sockets are checked as for one sent; what was
/// approved, it is given; another socket on the port, the answer is dropped
/// and the flow told it is held. Planted, watched failing: the check
/// skipped in `Flow::recv` (the answer given while another is on the port).
#[tokio::test]
async fn a_udp_share_s_answers_come_back_only_through_its_check() {
    let (service, from, (wire, key), shelf, dir, mut flow) = a_flow().await;
    let net = wire.net(key);
    let judged = (&net as &dyn crate::iroh::Net, dir.path(), &shelf);
    service.send_to(b"one", from).await.unwrap();
    assert_eq!(flow.recv(judged).await, Ok(&b"one"[..]));
    let port = from.port();
    let other = alone(u64::MAX);
    wire.reads(
        key,
        shelf.port().unwrap().get(),
        Ok((other, binding::Took::Datagram)),
    );
    service.send_to(b"two", from).await.unwrap();
    let got = flow.recv(judged).await.map(<[u8]>::to_vec);
    assert_eq!(
        got,
        Err(bound::Unsent::Held(Hold::Another)),
        "dropped, and held ({port})"
    );
}

/// **A UDP answer read while its check is cut short is carried at the next
/// read** (SHARE_BINDING_REVIEWS §6 #4): a flow's `select!` drops what it
/// polls when another branch wins, here mid-check, and the answer read off
/// the socket before it is still given, once checked. Planted, watched
/// failing: `Flow::recv` keeping the answer in its own future, not the flow
/// (the second read waits for an answer never sent again).
#[tokio::test]
async fn a_udp_answer_read_while_its_check_is_cut_short_is_carried_at_the_next() {
    let (service, from, (wire, key), shelf, dir, mut flow) = a_flow().await;
    let net = wire.net(key);
    let judged = (&net as &dyn crate::iroh::Net, dir.path(), &shelf);
    let port = shelf.port().unwrap().get();
    wire.slow(key, port, std::time::Duration::from_millis(300));
    service.send_to(b"one", from).await.unwrap();
    let within = std::time::Duration::from_millis(100);
    let cut = tokio::time::timeout(within, flow.recv(judged)).await;
    assert!(cut.is_err(), "the check is cut short");
    wire.slow(key, port, std::time::Duration::ZERO);
    let again = tokio::time::timeout(std::time::Duration::from_secs(2), flow.recv(judged));
    assert_eq!(
        again.await.map(|got| got.map(<[u8]>::to_vec)),
        Ok(Ok(b"one".to_vec()))
    );
}

/// **What a UDP flow's checks cost, each way** (SHARE_BINDING §10;
/// SHARE_BINDING_REVIEWS §6 #4, #12): a flow to a real service on this
/// computer, read by its own reader, the service answering each datagram,
/// a member's datagrams asked at 60 and then 120 a second for five seconds
/// each, through the loop `flowing` runs: how many reached the service and
/// came back, and this process's processor time. Printed, not asserted: run
/// in release with `--ignored --nocapture`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "measures for ten seconds: run by hand, in release, with --nocapture"]
async fn what_a_udp_flow_s_checks_cost_each_way() {
    let cpu = || {
        let pid = std::process::id().to_string();
        let ps = std::process::Command::new("ps")
            .args(["-o", "time=", "-p", &pid])
            .output();
        let said = String::from_utf8_lossy(&ps.unwrap().stdout)
            .trim()
            .to_string();
        said.split(':').fold(0.0, |at, part| {
            at * 60.0 + part.parse::<f64>().unwrap_or(0.0)
        })
    };
    for rate in [60u32, 120] {
        let (service, from, (wire, key), shelf, dir, mut flow) = a_flow().await;
        let net = wire.net(key);
        let judged = (&net as &dyn crate::iroh::Net, dir.path(), &shelf);
        let service = std::sync::Arc::new(service);
        let heard = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let (echo, counted) = (service.clone(), heard.clone());
        let echoing = tokio::spawn(async move {
            let mut buf = [0u8; 64];
            while let Ok((n, at)) = echo.recv_from(&mut buf).await {
                counted.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let _ = echo.send_to(&buf[..n], at).await;
            }
        });
        let _ = from;
        let (queue, mut packets) = tokio::sync::mpsc::channel::<Vec<u8>>(256);
        let asking = tokio::spawn(async move {
            let mut every = tokio::time::interval(std::time::Duration::from_secs(1) / rate);
            let (mut asked, mut dropped) = (0u32, 0u32);
            for _ in 0..rate * 5 {
                every.tick().await;
                asked += 1;
                dropped += u32::from(queue.try_send(b"ping".to_vec()).is_err());
            }
            (asked, dropped)
        });
        let (began, used) = (std::time::Instant::now(), cpu());
        let mut back = 0u32;
        let end = tokio::time::sleep(std::time::Duration::from_millis(5_500));
        tokio::pin!(end);
        loop {
            tokio::select! {
                got = packets.recv() => {
                    let Some(packet) = got else { break };
                    flow.send(judged, packet, &mut packets).await.unwrap();
                }
                got = flow.recv(judged) => back += u32::from(got.is_ok()),
                () = &mut end => break,
            }
        }
        let (asked, dropped) = asking.await.unwrap();
        let (spent, wall) = (cpu() - used, began.elapsed().as_secs_f64());
        let mut late = 0u32;
        let within = std::time::Duration::from_secs(1);
        while let Ok(got) = tokio::time::timeout(within, flow.recv(judged)).await {
            late += u32::from(got.is_ok());
        }
        let heard = heard.load(std::sync::atomic::Ordering::Relaxed);
        echoing.abort();
        println!(
            "{rate}/s each way: {asked} asked, {dropped} dropped, {heard} reached the service ({:.0}/s), {back} came back ({:.0}/s), {late} more after; {:.0}% of a processor",
            f64::from(heard) / wall,
            f64::from(back) / wall,
            spent / wall * 100.0
        );
    }
}

/// **A large answer over the build's relay is refused while its share is
/// spent; a small one, and any over a direct path or a group's own relay,
/// is not** (share-refusal; RELAY_COMMONS design 3, "The refusal"): a
/// range is refused before its service is asked, a body over the door's
/// own line after it, and the words are the design's in the lender's name.
/// Planted, watched red: a range taken as small, where a film's range
/// passes.
#[test]
fn a_large_answer_over_the_build_s_relay_is_refused_while_its_share_is_spent() {
    assert!(over_share(true, true, None), "a range over the spent relay");
    assert!(
        !over_share(true, false, None),
        "a request with no range is asked"
    );
    assert!(
        !over_share(false, true, None),
        "a range over a direct path or a group's own relay is never refused"
    );
    assert!(
        over_share(true, false, Some(OVER_SHARE + 1)),
        "a body over the line"
    );
    assert!(
        !over_share(true, false, Some(OVER_SHARE)),
        "a body at the line passes"
    );
    assert!(
        !over_share(true, false, None),
        "a body of no known length passes"
    );
    assert!(
        !over_share(false, false, Some(OVER_SHARE + 1)),
        "over a direct path it is never refused"
    );

    let refused = LendRefusal::Share.say(
        StatusCode::TOO_MANY_REQUESTS,
        spent_words(Some(Person::parse("Sam").unwrap())),
    );
    assert_eq!(
        (said(&refused).0, said(&refused).1),
        (429, Some("share")),
        "429, marked as svrngs's own"
    );
    assert_eq!(
        said(&refused).2,
        Some("Sam’s computer has used its share of the public way for now. Play it at full speed on the same Wi-Fi as Sam’s computer, or in the svrngs app, soon.")
    );
    assert!(spent_words(None).starts_with("This computer has used its share"));
}

/// **What a person is shown is named by a token that changes with it**
/// (SHARE_BINDING_REVIEWS §6 #6, share.rs `token`): the same sockets read
/// twice, the same token; another socket, another cgroup, another address,
/// another boot, or on a Mac the same pid started again, another; a hold
/// kept since, the same. Planted, watched failing: the token over `at`
/// alone (another socket, the same token).
#[test]
fn what_a_person_is_shown_is_named_by_a_token_that_changes_with_it() {
    use eudoxia_community::binding::{approve, Read};
    let token = |read: &Read| crate::share::token(&approve(read, None).unwrap());
    let read = Read {
        boot: Some(7),
        ..alone(10)
    };
    assert_eq!(token(&read), token(&read.clone()));
    let mut held = approve(&read, None).unwrap();
    held.held = true;
    assert_eq!(
        crate::share::token(&held),
        token(&read),
        "a hold isn't what was shown"
    );
    let with = |change: fn(&mut Read)| {
        let mut other = read.clone();
        change(&mut other);
        other
    };
    let differ = [
        with(|r| r.sockets[0].id = 11),
        with(|r| r.sockets[0].cgroup = Some(5)),
        with(|r| r.sockets[0].at = "::1".parse().unwrap()),
        with(|r| r.boot = Some(8)),
    ];
    for other in &differ {
        assert_ne!(token(other), token(&read), "{other:?}");
    }
    let on_mac = |start| Read {
        boot: None,
        lasting: false,
        sockets: vec![binding::Socket {
            holders: [(100, start)].into(),
            ..alone(9).sockets[0].clone()
        }],
        unlisted: false,
    };
    assert_eq!(token(&on_mac(1)), token(&on_mac(1)));
    assert_ne!(
        token(&on_mac(1)),
        token(&on_mac(2)),
        "the same pid started again"
    );
}
