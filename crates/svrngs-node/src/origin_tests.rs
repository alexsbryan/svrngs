// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
#[test]
fn reading_a_lend_is_not_authority_to_sign_its_reply_and_a_kind_cannot_expand() {
    let model = Program::parse("model").unwrap();
    let money = Program::parse("Money").unwrap();
    assert!(!Appending::for_request(Shown::Token, &HeaderMap::new())
        .unwrap()
        .permits(&money));
    let mut headers = HeaderMap::new();
    headers.insert(APPEND_HEADER, HeaderValue::from_static("model"));
    let accepted = Appending::for_request(Shown::Token, &headers).unwrap();
    assert!(accepted.permits(&model));
    assert!(!accepted.permits(&money));
}

/// **An `Origin` naming another lend outweighs `Sec-Fetch-Site:
/// same-origin`.** WebKit's service worker is shown no `Origin` on a
/// navigation, so the session worker forwards another lend's form post
/// unproved; the browser then sends the post with its true `Origin` and
/// `Referer`, but `Sec-Fetch-Site: same-origin` and `Sec-Fetch-Dest:
/// empty`, the worker's own fetch's (measured 2026-11-02,
/// scratchpad probe2/navseen2.txt: WebKit `cross-post-fwd`). It is that
/// lend's page's, judged as its fetch; the lend's own `Origin`, or none,
/// is still its own page's. Red before: the post was taken for the lend's
/// own tab and sent homeward (navigations.rs, WebKit).
#[test]
fn an_origin_naming_another_lend_outweighs_a_same_origin_fetch_site() {
    let (own, page) = (Label::of(&[1; 32], "own"), Label::of(&[1; 32], "page"));
    let headers = |origin: Option<String>| {
        let mut h = HeaderMap::new();
        h.insert("host", format!("{own}.localhost:4321").parse().unwrap());
        for (n, v) in [
            ("sec-fetch-site", "same-origin"),
            ("sec-fetch-mode", "navigate"),
            ("sec-fetch-dest", "empty"),
        ] {
            h.insert(n, HeaderValue::from_static(v));
        }
        if let Some(o) = origin {
            h.insert(header::ORIGIN, HeaderValue::from_str(&o).unwrap());
            h.insert(header::REFERER, format!("{o}/").parse().unwrap());
        }
        h
    };
    let from_page = headers(Some(format!("http://{page}.localhost:4321")));
    assert_eq!(app_of(&from_page, &own), Some(page));
    let from_own = headers(Some(format!("http://{own}.localhost:4321")));
    assert_eq!(app_of(&from_own, &own), Some(own.clone()));
    assert_eq!(app_of(&headers(None), &own), Some(own.clone()));
}

/// **A phone holds what a computer holds, by one rule stated twice**
/// (capability connections §5 step 7; web/lend.js `inert`): the two
/// languages share no constant, so this reads the JS and fails when either
/// list changes alone, and when the phone stops keeping `Retry-After` on
/// svrngs's refusal as `inert` does. Planted: a type added to [`DATA`];
/// `'retry-after'` dropped from web/lend.js; each watched failing.
#[test]
fn a_phone_and_a_computer_hold_one_rule_for_a_lends_answer_at_a_page() {
    let lend = include_str!("../../../web/lend.js");
    let types: Vec<String> = DATA.iter().map(|d| format!("'{d}'")).collect();
    let data = format!("const DATA = [{}];", types.join(", "));
    assert!(lend.contains(&data), "web/lend.js DATA is not {data}");
    for (name, value) in INERT {
        let said = format!("'{}': '{value}'", name.as_str());
        assert!(lend.contains(&said), "web/lend.js INERT lacks {said}");
    }
    let kept = "res.headers.has(REFUSED) ? [REFUSED, 'retry-after'] : []";
    assert!(
        lend.contains(kept),
        "web/lend.js inert does not keep {kept}"
    );
}

/// **An app's routes parse once, as the paths they replaced were read**
/// (svrn-docs EXTENSION_POINT §8, ARCH 9): `/svrngs/acts` and the two
/// exact routes by their path alone, the query aside; a stop only with a
/// label and the suffix; any path beginning `/svrngs-lend` a read, its
/// label none when it names none and its tail the rest with the query, as
/// `routed` split it; every other path the app's own. Planted: the query
/// left on the path, and `/svrngs-reads?x=1` is the app's own; watched
/// failing.
#[test]
fn an_app_s_routes_parse_once_as_the_paths_they_replaced() {
    let films = Label::of(&[2; 32], "films");
    let stop = format!("/svrngs-reads/{films}/svrngs-stop");
    for (asked, route) in [
        ("/svrngs/acts", AppRoute::Acts),
        ("/svrngs/acts?n=1", AppRoute::Acts),
        ("/svrngs-reads", AppRoute::Reads),
        ("/svrngs-reads?x=1", AppRoute::Reads),
        ("/svrngs-reads/", AppRoute::Own),
        (&stop, AppRoute::Stop(films.clone())),
        ("/svrngs-reads/not a label/svrngs-stop", AppRoute::Own),
        ("/svrngs-to?lender=a&lend=b", AppRoute::To),
        ("/svrngs-lendx", AppRoute::Lend(None, "/".into())),
        ("/svrngs-lend/films", AppRoute::Lend(None, "/".into())),
        ("/index.html", AppRoute::Own),
        ("/", AppRoute::Own),
    ] {
        assert_eq!(AppRoute::of(asked), route, "{asked}");
    }
    let read = format!("/svrngs-lend/{films}/Items?x=/y");
    let tail = "/Items?x=/y".to_string();
    assert_eq!(AppRoute::of(&read), AppRoute::Lend(Some(films), tail));
}

// scripts/surface.txt's gate, beside `AppRoute::of`'s own test above: a
// module here, so the list costs surface.rs no line of code.
#[path = "surface_list_tests.rs"]
mod surface_list;
