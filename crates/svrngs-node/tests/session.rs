// SPDX-License-Identifier: AGPL-3.0-or-later
//! A browser comes through an open (give-session; svrn-docs
//! `rings/current/GIVE.md` §5): the home page and every lend's origin
//! answer a browser only with a session an open on this computer made, and
//! a program only with this computer's token. Loopback alone let in a
//! container on a Mac running podman, which reaches the Mac's `127.0.0.1`
//! through gvproxy (GIVE.md §5, measured 2026-10-25): it could read the home
//! page's lists, say the person's yes, and use every lend as them.
//!
//! An open is a single-use code that expires ([`svrngs_node`]'s
//! `/v1/open`, asked with the token, as `svrngs open` and `install.sh`
//! ask it); the home page mints one for each lend it opens (`/to`). Alex's
//! computer, alone, lends `night`, whose service keeps every head it is
//! sent.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[allow(dead_code)]
#[path = "door/origins.rs"]
mod origins;
#[path = "common/pair.rs"]
mod pair;

use std::net::SocketAddr;
use std::time::Duration;

use pair::{alex_lets_sam_in, signing};
use svrngs_core::pass::Label;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// What came back: the status, the head and the body.
struct Got {
    status: u16,
    head: String,
    body: String,
}

impl Got {
    fn header(&self, name: &str) -> Option<&str> {
        self.head
            .lines()
            .filter_map(|l| l.split_once(':'))
            .find(|(n, _)| n.trim().eq_ignore_ascii_case(name))
            .map(|(_, v)| v.trim())
    }

    /// The session this answer set, as a `Cookie:` line sends it back.
    fn cookie(&self) -> String {
        let set = self
            .header("set-cookie")
            .unwrap_or_else(|| panic!("no cookie set: {}", self.head));
        format!("Cookie: {}\r\n", set.split(';').next().unwrap())
    }
}

/// One request to `api` at `host`, `extra` its other header lines.
async fn ask(
    api: SocketAddr,
    method: &str,
    host: &str,
    path: &str,
    extra: &str,
    body: &str,
) -> Got {
    let mut sock = tokio::net::TcpStream::connect(api).await.unwrap();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\n{extra}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    sock.write_all(request.as_bytes()).await.unwrap();
    let mut reply = Vec::new();
    tokio::time::timeout(Duration::from_secs(30), sock.read_to_end(&mut reply))
        .await
        .expect("an answer")
        .unwrap();
    let reply = String::from_utf8_lossy(&reply).into_owned();
    let (head, body) = reply.split_once("\r\n\r\n").unwrap();
    Got {
        status: head[9..12].parse().unwrap(),
        head: head.to_string(),
        body: body.to_string(),
    }
}

/// Alex's `night`, of kind `night`, at `port`.
fn declare(dir: &std::path::Path, port: u16) {
    let at = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let body = serde_json::json!({ "port": port, "kind": "night", "limit": 3 });
    std::fs::write(at.join("night.json"), body.to_string()).unwrap();
}

/// The path and query of `url`, `http://<host>/<path>`.
fn path_of(url: &str) -> String {
    let rest = url
        .strip_prefix("http://")
        .unwrap_or_else(|| panic!("not an address: {url}"));
    format!("/{}", rest.split_once('/').map_or("", |(_, p)| p))
}

const TYPED: &str =
    "Sec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-Dest: document\r\n";

/// What a browser sends of the tab's navigation from another site: a
/// link followed, or a redirect from another host.
const CROSS: &str =
    "Sec-Fetch-Site: cross-site\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-Dest: document\r\n";

/// What a browser sends of a frame's navigation from another site: never
/// sent on to the home page (origin.rs `lent`).
const FRAMED: &str =
    "Sec-Fetch-Site: cross-site\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-Dest: iframe\r\n";

/// The words a browser with no session is answered, naming the opener in
/// the person's words, which need no terminal (WHAT_CAN_ACT_AS_YOU §9).
fn opener_named(body: &str) -> bool {
    let opener = if cfg!(target_os = "macos") {
        "Open svrngs from your Applications"
    } else {
        "Open svrngs from your applications menu"
    };
    body.contains("Open svrngs from your computer") && body.contains(opener)
}

/// **A browser with no session, or a program with no token, is told to
/// open svrngs from the computer, at the home page and at a lend's
/// origin.** Refused: before this, a typed address (`Sec-Fetch-Site:
/// none`) was the person, and a request with no `Sec-Fetch-Site` at a
/// lend's origin was "a program on this computer, so the lend's own"
/// (origin.rs `app_of`), so a container read the home page's groups and
/// used Alex's lend as him. A program showing the token reaches the lend
/// as its own, and the token never reaches the lend's service. Planted:
/// the session check taken out of `only_itself`, and the home page
/// answers a typed address with no session 200.
#[tokio::test]
async fn a_browser_with_no_session_and_a_program_with_no_token_are_told_to_open_svrngs() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, _sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    let (service, heads) = origins::head_origin().await;
    declare(alex_dir.path(), service.port());
    let api = alex.api_addr();
    let port = api.port();
    let night = Label::of(&signing(1).verifying_key().to_bytes(), "night");
    let (home, lend) = (
        format!("svrngs.localhost:{port}"),
        format!("{night}.localhost:{port}"),
    );

    for (host, path, extra) in [
        (&home, "/", TYPED),
        (
            &home,
            "/v1/groups",
            "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\n",
        ),
        (&lend, "/", ""),
        (&lend, "/", FRAMED),
    ] {
        let got = ask(api, "GET", host, path, extra, "").await;
        assert_eq!(
            got.status, 401,
            "{host}{path} with no session or token: {}",
            got.body
        );
        assert!(
            opener_named(&got.body),
            "{host}{path} names the opener: {}",
            got.body
        );
    }
    // A typed address at a lend's origin, a bookmark, goes by the home
    // page's `/to` (origin.rs `homeward`), which refuses a browser with no
    // session there in the same words.
    let typed = ask(api, "GET", &lend, "/", TYPED, "").await;
    let to = format!("http://{home}/to?lend={night}&at=%2F");
    assert_eq!(
        (typed.status, typed.header("location")),
        (303, Some(to.as_str())),
        "{}",
        typed.head
    );
    let refused = ask(api, "GET", &home, &to[to.find("/to").unwrap()..], TYPED, "").await;
    assert!(
        refused.status == 401 && opener_named(&refused.body),
        "{}",
        refused.body
    );
    let program = ask(api, "GET", &lend, "/", "", "").await;
    let token_file = alex_dir.path().join("token");
    assert!(
        program.body.contains(&token_file.display().to_string()),
        "a program is told where the token is: {}",
        program.body
    );
    assert!(
        heads.lock().unwrap().is_empty(),
        "nothing reached the lend's service"
    );

    let bearer = format!("Authorization: Bearer {}\r\n", alex.token().trim());
    let reached = ask(api, "GET", &lend, "/as-a-program", &bearer, "").await;
    assert_eq!(
        (reached.status, reached.body.as_str()),
        (200, "ok"),
        "{}",
        reached.head
    );
    let heads = heads.lock().unwrap();
    assert!(
        heads.iter().any(|h| h.starts_with("GET /as-a-program")),
        "{heads:?}"
    );
    assert!(
        !heads.iter().any(|h| h.contains(alex.token().trim())),
        "the token never reaches the lend's service: {heads:?}"
    );
}

/// **`/v1/open` mints a single-use code that becomes the session, and a
/// lend opened from the home page carries a code for its own origin.**
/// `svrngs open` asks `/v1/open` with the token; its address, opened,
/// sets the home page's session and goes on to the home page, which a
/// bookmark then opens with no code. The home page's `/to?lend=` mints a
/// code for that lend's origin alone, which sets its session there and
/// goes on to the path asked. A code redeemed twice, or at another
/// origin, is refused; the home page's session opens no lend's origin,
/// and no lend's session is ever carried to the lend's service; a
/// WebSocket's handshake, which carries no Sec-Fetch-Site, is judged by
/// its Origin. Planted:
/// `Codes::redeem` keeping a code after its use, and the replay opens the
/// home page.
#[tokio::test]
async fn an_open_s_code_becomes_the_session_once_and_each_lend_gets_its_own() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, _sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    let (service, heads) = origins::head_origin().await;
    declare(alex_dir.path(), service.port());
    let api = alex.api_addr();
    let port = api.port();
    let night = Label::of(&signing(1).verifying_key().to_bytes(), "night");
    let (home, lend) = (
        format!("svrngs.localhost:{port}"),
        format!("{night}.localhost:{port}"),
    );
    let bearer = format!("Authorization: Bearer {}\r\n", alex.token().trim());
    let local = api.to_string();

    let refused = ask(api, "POST", &local, "/v1/open", "", "{}").await;
    assert_eq!(
        refused.status, 401,
        "an open is the token's: {}",
        refused.body
    );
    let open = ask(api, "POST", &local, "/v1/open", &bearer, "{}").await;
    assert_eq!(open.status, 200, "{}", open.body);
    let url: serde_json::Value = serde_json::from_str(&open.body).unwrap();
    let url = url["open"]
        .as_str()
        .unwrap_or_else(|| panic!("no open: {}", open.body))
        .to_string();
    assert!(url.starts_with(&format!("http://{home}/")), "{url}");

    let opened = ask(api, "GET", &home, &path_of(&url), TYPED, "").await;
    assert_eq!(
        (opened.status, opened.header("location")),
        (303, Some("/svrngs-open?kept=%2F")),
        "{}",
        opened.head
    );
    let set = opened
        .header("set-cookie")
        .unwrap_or_default()
        .to_ascii_lowercase();
    for attribute in ["httponly", "samesite=lax", "path=/", "max-age="] {
        assert!(
            set.contains(attribute),
            "the session is kept, {attribute}: {set}"
        );
    }
    let session = opened.cookie();
    let page = ask(api, "GET", &home, "/", &format!("{TYPED}{session}"), "").await;
    assert_eq!(
        page.status, 200,
        "a bookmark opens the home page: {}",
        page.body
    );
    assert!(page.body.contains(r#"src="/home.js""#), "{}", page.body);
    let again = ask(api, "GET", &home, &path_of(&url), TYPED, "").await;
    assert_ne!(
        again.status, 303,
        "a replayed code is refused: {}",
        again.head
    );
    assert!(again.header("set-cookie").is_none(), "{}", again.head);

    let at = "/films?x=1";
    let to = format!(
        "/to?lend={night}&at={}",
        at.replace('?', "%3F").replace('=', "%3D")
    );
    let without = ask(api, "GET", &home, &to, TYPED, "").await;
    assert_eq!(
        without.status, 401,
        "a lend is opened only from an opened home page: {}",
        without.head
    );
    let going = ask(api, "GET", &home, &to, &format!("{TYPED}{session}"), "").await;
    assert_eq!(going.status, 303, "{}", going.head);
    let there = going.header("location").unwrap_or_default().to_string();
    assert!(there.starts_with(&format!("http://{lend}/")), "{there}");
    let elsewhere = ask(api, "GET", &home, &path_of(&there), TYPED, "").await;
    assert!(
        elsewhere.header("set-cookie").is_none(),
        "a lend's code opens no other origin: {}",
        elsewhere.head
    );
    let landed = ask(api, "GET", &lend, &path_of(&there), TYPED, "").await;
    assert_eq!(
        landed.status, 401,
        "spent at another origin, it is spent: {}",
        landed.head
    );

    let going = ask(api, "GET", &home, &to, &format!("{TYPED}{session}"), "").await;
    let there = going.header("location").unwrap_or_default().to_string();
    let landed = ask(api, "GET", &lend, &path_of(&there), TYPED, "").await;
    assert_eq!(
        (landed.status, landed.header("location")),
        (303, Some("/svrngs-open?kept=%2Ffilms%3Fx%3D1")),
        "{}",
        landed.head
    );
    let lends = landed.cookie();
    assert_ne!(lends, session, "each origin its own session");
    // Back to see the session kept (session.rs `redeem`), then on to the
    // path; a way back naming another host's path goes to this one's root.
    for (back, on) in [
        ("%2Ffilms%3Fx%3D1", at),
        ("%2F%2Felsewhere.example%2F", "/"),
    ] {
        let kept = format!("/svrngs-open?kept={back}");
        let kept = ask(api, "GET", &lend, &kept, &format!("{TYPED}{lends}"), "").await;
        assert_eq!(
            (kept.status, kept.header("location")),
            (303, Some(on)),
            "{}",
            kept.head
        );
    }
    let reached = ask(api, "GET", &lend, at, &format!("{TYPED}{lends}"), "").await;
    assert_eq!(
        (reached.status, reached.body.as_str()),
        (200, "ok"),
        "{}",
        reached.head
    );
    // A WebSocket's handshake, which Chromium sends with the cookie and
    // its page's Origin but no Sec-Fetch-Site (the doc scene's socket), is
    // its own page's, judged by that Origin (origin.rs app_of).
    let socket = format!("Origin: http://{lend}\r\n{lends}");
    let socket = ask(api, "GET", &lend, at, &socket, "").await;
    assert_eq!(
        (socket.status, socket.body.as_str()),
        (200, "ok"),
        "a socket from the lend's own page: {}",
        socket.head
    );
    let replay = ask(api, "GET", &lend, &path_of(&there), TYPED, "").await;
    assert_ne!(
        replay.status, 303,
        "a replayed code is refused: {}",
        replay.head
    );
    // The home page's session opens no lend: shown there, it is no session,
    // and the tab is sent to the home page's `/to` (origin.rs `homeward`).
    let homes = ask(api, "GET", &lend, at, &format!("{TYPED}{session}"), "").await;
    assert!(
        homes.status == 303
            && homes
                .header("location")
                .is_some_and(|l| l.starts_with(&format!("http://{home}/to?"))),
        "the home page's session opens no lend: {}",
        homes.head
    );

    let heads = heads.lock().unwrap();
    assert!(
        heads.iter().any(|h| h.starts_with("GET /films?x=1")),
        "{heads:?}"
    );
    let sent = |c: &str| c.trim_start_matches("Cookie: ").trim().to_string();
    assert!(
        !heads
            .iter()
            .any(|h| h.contains(&sent(&lends)) || h.contains(&sent(&session))),
        "no session reaches the lend's service: {heads:?}"
    );
}

/// **A read's address is no session** (DESIGN §3, "Lent apps", call 6): a
/// lend's page reads another lend at `/svrngs-as/<cap>/` there, the cap
/// this computer's for that pair alone (the browser tests, `navigations`,
/// `opens`, `acts`). Refused: a cap this computer did not give, which a
/// container on a Mac running podman forges with the page's `Origin`, and
/// nothing of it reaches the lend's service. A navigation under the prefix,
/// the person following a link, is sent to the home page's `/to` at the
/// rest of its path, so the home page's session decides and a cap never
/// sets a lend's. Planted: the cap's check taken for held, and the forged
/// one reaches the service.
#[tokio::test]
async fn a_read_s_address_is_held_to_its_capability_and_never_a_session() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, _sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    let (service, heads) = origins::head_origin().await;
    declare(alex_dir.path(), service.port());
    let api = alex.api_addr();
    let port = api.port();
    let night = Label::of(&signing(1).verifying_key().to_bytes(), "night");
    let page = Label::of(&signing(1).verifying_key().to_bytes(), "page");
    let lend = format!("{night}.localhost:{port}");
    let fetch = format!("Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{page}.localhost:{port}\r\n");

    let forged = ask(api, "GET", &lend, "/svrngs-as/forged/films", &fetch, "").await;
    assert_eq!(
        forged.status, 401,
        "a cap not this computer's: {}",
        forged.body
    );
    assert!(opener_named(&forged.body), "{}", forged.body);
    assert!(
        heads.lock().unwrap().is_empty(),
        "nothing reached the lend's service"
    );

    let followed = ask(api, "GET", &lend, "/svrngs-as/any/films?x=1", TYPED, "").await;
    assert_eq!(
        (followed.status, followed.header("location")),
        (
            303,
            Some(
                format!("http://svrngs.localhost:{port}/to?lend={night}&at=%2Ffilms%3Fx%3D1")
                    .as_str()
            )
        ),
        "{}",
        followed.head
    );
    assert!(followed.header("set-cookie").is_none(), "{}", followed.head);
}

/// **A browser keeping no cookie for a lend is told why, after one hop,
/// never sent round again** (origin.rs `homeward`; session.rs `redeem`).
/// Alex's browser holds the home page's session and keeps no cookie for
/// `night`'s host, as one refusing cookies there does: its bookmark of
/// `night` at a path, followed as the browser follows each 303, a
/// navigation of the tab each time, goes once by the home page's `/to` and
/// the lend's open, which comes back to see its session kept, and finding
/// none says why, in the home page's words. Red on 3934e7c6: the lend's
/// open went straight on to the path, which sent it to `/to` again, round
/// to any browser's redirect limit. Planted: the way back to see it kept
/// taken out (the open going straight on), red the same way. Kept: a
/// browser that keeps it reaches the path, by one more hop.
#[tokio::test]
async fn a_browser_keeping_no_cookie_for_a_lend_is_told_why_never_sent_round() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, _sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    let (service, _heads) = origins::head_origin().await;
    declare(alex_dir.path(), service.port());
    let api = alex.api_addr();
    let port = api.port();
    let night = Label::of(&signing(1).verifying_key().to_bytes(), "night");
    let (home, lend) = (
        format!("svrngs.localhost:{port}"),
        format!("{night}.localhost:{port}"),
    );
    let bearer = format!("Authorization: Bearer {}\r\n", alex.token().trim());
    let open = ask(api, "POST", &api.to_string(), "/v1/open", &bearer, "{}").await;
    let url: serde_json::Value = serde_json::from_str(&open.body).unwrap();
    let opened = ask(
        api,
        "GET",
        &home,
        &path_of(url["open"].as_str().unwrap()),
        TYPED,
        "",
    )
    .await;
    let session = opened.cookie();

    // Each 303 followed as a browser follows it, a navigation of the tab,
    // the home page's session sent to its host, and the lend's cookie kept
    // only when `keeps`: where it lands, and the hops on the way.
    let follow = |keeps: bool| {
        let (home, lend, session) = (home.clone(), lend.clone(), session.clone());
        async move {
            let (mut host, mut path, mut kept, mut hops) = (
                lend.clone(),
                "/films?x=1".to_string(),
                String::new(),
                Vec::new(),
            );
            loop {
                let cookie = if host == home {
                    session.clone()
                } else {
                    kept.clone()
                };
                let got = ask(api, "GET", &host, &path, &format!("{CROSS}{cookie}"), "").await;
                hops.push(format!("{} {host}{path}", got.status));
                if keeps && host == lend && got.header("set-cookie").is_some() {
                    kept = got.cookie();
                }
                let Some(to) = got
                    .header("location")
                    .filter(|_| got.status == 303 && hops.len() < 20)
                else {
                    return (got, hops);
                };
                match to.strip_prefix("http://") {
                    Some(rest) => {
                        let (h, p) = rest.split_once('/').unwrap();
                        (host, path) = (h.to_string(), format!("/{p}"));
                    }
                    None => path = to.to_string(),
                }
            }
        }
    };
    let (got, hops) = follow(false).await;
    assert!(
        got.status == 401
            && opener_named(&got.body)
            && got
                .body
                .contains("This browser didn’t keep what svrngs needs to open this page")
            && hops.len() <= 5,
        "told why, after one hop, never sent round: {hops:#?}\n{}",
        got.body
    );
    let (got, hops) = follow(true).await;
    assert_eq!(
        (got.status, got.body.as_str()),
        (200, "ok"),
        "a browser that keeps it reaches the path: {hops:#?}"
    );
}
