// SPDX-License-Identifier: AGPL-3.0-or-later
//! The home page (sup-2; DESIGN §3, "The home page"):
//! `http://svrngs.localhost:<api port>/`, on the record API's listener.
//! Alex founds a group and lets Sam in. Alex lends `night`, a page reading
//! kind `taste`, and a `taste` that asks; Sam lends a `taste` that asks.
//! Two nodes in one process on one wire (common/on_wire.rs; docs/CHECKS.md,
//! harness 3), no endpoint, each API asked of its routes as a browser on
//! its computer asks it. A lent service is answered in this process where
//! a test's stub is origins.rs's (`head_on`, harness 2); a key's or an
//! account's try, the port probe and the resolver still reach a loopback
//! port, as svrngs asks them there.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[path = "common/on_wire.rs"]
mod on_wire;
#[allow(dead_code)]
#[path = "door/origins.rs"]
mod origins;
#[allow(
    dead_code,
    reason = "this file's computers are on the wire; pair.rs's over QUIC are its other loaders'"
)]
#[path = "common/pair.rs"]
mod pair;

use std::time::Duration;

use common::heard;
use on_wire::{http, Browser};
use pair::{let_sam_in, signing, NS};

use iroh_relay::server::{RelayConfig, Server, ServerConfig};
use svrngs_core::link::Link;
use svrngs_core::pass::Label;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, group, lend, Person, Roster, SigningKey};

const WORDS: &str = "the app receives ranked suggestions; your history stays where it is";

#[allow(dead_code)]
#[path = "common/computers.rs"]
mod computers;
#[path = "home/folder.rs"]
mod folder;
#[path = "home/opens.rs"]
mod opens;
#[path = "home/publication.rs"]
mod publication;
#[path = "home/shelf.rs"]
mod shelf;

fn declare(
    dir: &std::path::Path,
    name: &str,
    port: u16,
    kind: &str,
    reads: &[&str],
    asks: Option<&str>,
) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let body = serde_json::json!({ "port": port, "kind": kind, "limit": 3, "reads": reads, "asks": asks, "groups": [NS] });
    std::fs::write(
        at.join(format!("{name}.json")),
        svrngs_node::node::approving(&body),
    )
    .unwrap();
}

/// A lent service at `port` on `node`, answered in this process as
/// origins.rs `head_origin` answers (on_wire.rs `serve`, harness 2): no
/// port is bound; each request's head kept.
fn head_on(node: &Node, port: u16) -> std::sync::Arc<std::sync::Mutex<Vec<String>>> {
    let kept: std::sync::Arc<std::sync::Mutex<Vec<String>>> = std::sync::Arc::default();
    let heads = std::sync::Arc::clone(&kept);
    on_wire::serve(node, port, move |s| origins::answer_head(s, heads.clone()));
    kept
}

/// pair.rs `alex_lets_sam_in` with both computers on this test's wire,
/// where Alex's reaches Sam's by key alone.
async fn alex_lets_sam_in(alex_dir: &std::path::Path, sam_dir: &std::path::Path) -> (Node, Node) {
    let alex = on_wire::start(1, alex_dir).await;
    let sam = on_wire::start(2, sam_dir).await;
    let_sam_in(&alex, &sam);
    (alex, sam)
}

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

    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).unwrap_or_else(|e| panic!("{e}: {}", self.body))
    }
}

/// One request from `browser` at `host`, `extra` its other header lines,
/// with the session svrngs opened there when `extra` is a page's: asked of
/// the routes at the browser's node's API, as its listener would be asked.
async fn ask(
    browser: &Browser<'_>,
    method: &str,
    host: &str,
    path: &str,
    extra: &str,
    body: &str,
) -> Got {
    let path = browser.path(host, path, extra);
    let proof = browser.proof(method, host, &path, extra, body).await;
    let mut lines = axum::http::HeaderMap::new();
    for (name, value) in (extra.lines().chain(proof.lines())).filter_map(|l| l.split_once(':')) {
        let name = axum::http::HeaderName::from_bytes(name.trim().as_bytes()).unwrap();
        lines.append(name, value.trim().parse().unwrap());
    }
    let url = format!("http://{}{path}", browser.api());
    let asked = (http().request(method.parse().unwrap(), url))
        .header("host", host)
        .header("content-type", "application/json")
        .headers(lines)
        .body(body);
    let reply = tokio::time::timeout(Duration::from_secs(30), asked.send())
        .await
        .expect("an answer")
        .unwrap();
    let status = reply.status().as_u16();
    let mut head = format!("HTTP/1.1 {status}");
    for (name, value) in reply.headers() {
        head += &format!("\r\n{name}: {}", value.to_str().unwrap_or_default());
    }
    Got {
        status,
        head,
        body: reply.text().await.unwrap(),
    }
}

#[tokio::test]
async fn run_routes_take_only_the_home_pages_closed_requests() {
    let dir = tempfile::tempdir().unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    let host = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{host}\r\n");
    let foreign = "Sec-Fetch-Site: cross-site\r\nSec-Fetch-Mode: cors\r\nOrigin: https://elsewhere.invalid\r\n";
    let browser = Browser::of(&alex);
    for (path, body) in [
        (
            "/run",
            r#"{"name":"test","port":1,"folder":".","command":"/bin/false","argv":["/bin/true"]}"#,
        ),
        ("/run/action", r#"["test","remove"]"#),
    ] {
        let got = ask(&browser, "POST", &host, path, &own, body).await;
        assert_eq!(got.status, 422, "{path}: {}", got.body);
        let got = ask(&browser, "POST", &host, path, foreign, body).await;
        assert_eq!(got.status, 403, "{path}: {}", got.body);
    }
    alex.net().close().await;
}

/// **The home page answers only itself, shows what `/v1/lends` answers, and
/// takes only the person's yes and stop.** Typed, it serves its page and
/// module, never to be framed, and every answer, its session's bootstrap
/// and a refusal included, keeps its opener policy, so no app that opens it
/// or that it opens holds a handle on it; a lend's page navigated to `/ask`
/// gets the page too. Its `/v1/groups` and `/v1/lends` answer what the record API's
/// answer an app, the same body. A lend's page reading its list is refused
/// `closed`, and so is a post with no exact `Origin` or not from its own
/// origin: a lend's page's yes. The person's yes, from its own origin, lets
/// `night` into Alex's own `taste` (kept here) and Sam's (posted to Sam's
/// door), which refused it `unasked` before, and `GET /choices` asks Sam's
/// door what apps use. `/svrngs-reads` at `night` gives each asked lend's
/// words and its `ask` address. `night`'s page stops itself at its origin,
/// at both, and is refused `stopped` after. Planted, each watched failing:
/// a lend page's yes accepted; a lend page reading the home page's list;
/// the home page framed; its opener policy left out (home.rs `HEADS`).
#[tokio::test]
async fn the_home_page_answers_only_itself_and_takes_only_the_person_s_choices() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    let mut heads = std::collections::BTreeMap::new();
    for (who, (node, dir), name, kind, reads, asks, port) in [
        (
            1,
            (&alex, alex_dir.path()),
            "night",
            "night",
            &["taste"][..],
            None,
            9001,
        ),
        (
            1,
            (&alex, alex_dir.path()),
            "taste",
            "taste",
            &[],
            Some(WORDS),
            9002,
        ),
        (
            2,
            (&sam, sam_dir.path()),
            "taste",
            "taste",
            &[],
            Some(WORDS),
            9002,
        ),
    ] {
        declare(dir, name, port, kind, reads, asks);
        heads.insert((who, name), head_on(node, port));
    }
    let api = alex.api_addr();
    let browser = &Browser::of(&alex);
    let port = api.port();
    let label = |seed: u8, name: &str| Label::of(&signing(seed).verifying_key().to_bytes(), name);
    let (night, alexs, sams) = (label(1, "night"), label(1, "taste"), label(2, "taste"));
    let home = format!("svrngs.localhost:{port}");
    let at = |l: &Label| format!("{l}.localhost:{port}");
    let typed = "Sec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\n";
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let from_night = format!(
        "Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{}\r\n",
        at(&night)
    );

    let page = ask(browser, "GET", &home, "/", typed, "").await;
    assert_eq!(page.status, 200, "{}", page.body);
    assert!(page.body.contains(r#"src="/home.js""#), "{}", page.body);
    // oidc-retire: the kind-keyed sign-in helper route is gone, so Jellyfin
    // answers as any kind with none (SIGN_IN_WITH_SVRNGS §7 3). A planted
    // `/helper/{kind}` route made this red before it passed.
    let jellyfin = ask(browser, "GET", &home, "/helper/jellyfin", typed, "").await;
    let any = ask(browser, "GET", &home, "/helper/unheard-of", typed, "").await;
    assert_eq!(jellyfin.status, 404, "{}", jellyfin.body);
    assert_eq!(
        jellyfin.status, any.status,
        "a helper is no kind of its own: {} / {}",
        jellyfin.body, any.body
    );
    for got in [
        &page,
        &ask(browser, "GET", &home, "/v1/groups", &from_night, "").await,
        &ask(browser, "GET", &home, "/svrngs-session.js", typed, "").await,
    ] {
        assert_eq!(
            got.header("content-security-policy"),
            Some("default-src 'self'; frame-ancestors 'none'"),
            "never framed, answered or refused: {}",
            got.head
        );
        assert_eq!(
            got.header("cross-origin-opener-policy"),
            Some("same-origin"),
            "no opener's handle, answered or refused: {}",
            got.head
        );
    }
    let module = ask(browser, "GET", &home, "/home.js", &own, "").await;
    assert_eq!(
        (module.status, module.header("content-type")),
        (200, Some("text/javascript"))
    );
    // The tab's icon, which the page links: the mark's small cut, from its
    // own origin, as its policy allows. Red with the route planted out.
    let icon = ask(browser, "GET", &home, "/favicon.svg", typed, "").await;
    assert_eq!(
        (icon.status, icon.header("content-type")),
        (200, Some("image/svg+xml"))
    );
    assert!(
        page.body.contains(r#"href="/favicon.svg""#),
        "{}",
        page.body
    );
    assert!(
        icon.body.contains(r#"aria-label="svrngs""#),
        "{}",
        icon.body
    );
    let navigated = format!(
        "Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: navigate\r\nOrigin: http://{}\r\n",
        at(&night)
    );
    let asking = ask(
        browser,
        "GET",
        &home,
        &format!("/ask?lend={sams}&app={night}&back=/x"),
        &navigated,
        "",
    )
    .await;
    assert_eq!(
        asking.status, 200,
        "a lend's page sends its person to ask: {}",
        asking.body
    );
    let elsewhere = ask(browser, "GET", "svrngs.localhost:1", "/", typed, "").await;
    assert_eq!(
        elsewhere.status, 401,
        "only at its own port: {}",
        elsewhere.body
    );

    for path in [
        "/v1/groups".to_string(),
        format!("/v1/lends?namespace={NS}"),
    ] {
        let shown = ask(browser, "GET", &home, &path, &own, "").await;
        let program = svrngs_node::node::AS_YOU;
        let answered = ask(browser, "GET", "127.0.0.1", &path, program, "").await;
        assert_eq!(
            (shown.status, shown.json()),
            (answered.status, answered.json()),
            "{path}: one decider"
        );
        let read = ask(browser, "GET", &home, &path, &from_night, "").await;
        assert_eq!(
            (read.status, read.header("x-svrngs-refused")),
            (403, Some("closed")),
            "a lend's page reading {path}: {}",
            read.body
        );
    }
    let shown = ask(
        browser,
        "GET",
        &home,
        &format!("/v1/lends?namespace={NS}"),
        &own,
        "",
    )
    .await
    .json();
    let lent = shown["lent"].as_array().unwrap();
    assert!(
        lent.iter()
            .any(|l| l["name"] == "taste" && l["person"] == "sam" && l["asks"] == WORDS),
        "{shown:#}"
    );

    let fetch = |l: &Label| {
        let (host, extra) = (at(l), from_night.clone());
        async move { ask(browser, "GET", &host, "/x", &extra, "").await }
    };
    let refused = |got: &Got| {
        (
            got.status,
            got.header("x-svrngs-refused").map(str::to_string),
        )
    };
    for l in [&alexs, &sams] {
        assert_eq!(
            refused(&fetch(l).await),
            (403, Some("unasked".into())),
            "before any yes: {l}"
        );
    }
    let yes = |l: &Label| format!(r#"{{"lend":"{l}","app":"{night}","said":"yes"}}"#);
    for (row, extra) in [
        ("night's page", from_night.clone()),
        (
            "the home page's Origin, from another site",
            format!("Sec-Fetch-Site: same-site\r\nOrigin: http://{home}\r\n"),
        ),
        (
            "its own origin with no Origin",
            "Sec-Fetch-Site: same-origin\r\n".into(),
        ),
        ("a program, as you", browser.as_you()),
    ] {
        let posted = ask(browser, "POST", &home, "/choices", &extra, &yes(&alexs)).await;
        assert_eq!(
            refused(&posted),
            (403, Some("closed".into())),
            "{row}'s yes: {}",
            posted.body
        );
    }
    assert_eq!(
        refused(&fetch(&alexs).await),
        (403, Some("unasked".into())),
        "no refused yes was kept"
    );

    for l in [&alexs, &sams] {
        let said = ask(browser, "POST", &home, "/choices", &own, &yes(l)).await;
        assert_eq!(said.status, 200, "the person's yes at {l}: {}", said.body);
        let carried = fetch(l).await;
        assert_eq!(
            refused(&carried),
            (200, None),
            "after the yes: {}",
            carried.body
        );
    }
    let kept = ask(
        browser,
        "GET",
        &home,
        &format!("/choices?lend={sams}"),
        &own,
        "",
    )
    .await;
    assert_eq!(
        kept.json()["choices"][night.as_str()]["said"],
        "yes",
        "{}",
        kept.body
    );

    let page = "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\n";
    let reads = ask(browser, "GET", &at(&night), "/svrngs-reads", page, "")
        .await
        .json();
    for l in [&alexs, &sams] {
        let one = reads["lends"]
            .as_array()
            .unwrap()
            .iter()
            // Its origin, with the capability night's page shows there
            // (give-session; tests/session.rs).
            .find(|r| {
                (r["address"].as_str().unwrap_or_default())
                    .starts_with(&format!("http://{}/svrngs-as/", at(l)))
            });
        let one = one.unwrap_or_else(|| panic!("{reads:#}"));
        assert_eq!(one["asks"], WORDS);
        assert_eq!(
            one["ask"],
            format!("http://{home}/ask?lend={l}&app={night}")
        );
        assert_eq!(
            one["stop"],
            format!("http://{}/svrngs-reads/{l}/svrngs-stop", at(&night)),
            "the stop address, beside ask, at night's own origin"
        );
    }

    let same = "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\n";
    for l in [&alexs, &sams] {
        let stop = ask(
            browser,
            "POST",
            &at(&night),
            &format!("/svrngs-reads/{l}/svrngs-stop"),
            same,
            "",
        )
        .await;
        assert_eq!(
            stop.status, 200,
            "night's page stops itself at {l}: {}",
            stop.body
        );
        assert_eq!(
            refused(&fetch(l).await),
            (403, Some("stopped".into())),
            "after the stop: {l}"
        );
    }
    assert!(
        heads[&(2, "taste")].lock().unwrap().len() == 1,
        "Sam's taste was asked once, after the yes"
    );
    sam.net().close().await;
    alex.net().close().await;
}

/// A stand-in for Immich's `GET /api/users/me` on `listener`: 200 for
/// `x-api-key: k-1`, else 401.
fn key_stand_in(listener: std::net::TcpListener) {
    use std::io::{BufRead, Write};
    std::thread::spawn(move || {
        for mut sock in listener.incoming().flatten() {
            let mut head = String::new();
            let mut lines = std::io::BufReader::new(&sock);
            while !head.ends_with("\r\n\r\n") && lines.read_line(&mut head).unwrap_or(0) > 0 {}
            let lower = head.to_ascii_lowercase();
            let took =
                lower.starts_with("get /api/users/me ") && lower.contains("\r\nx-api-key: k-1\r\n");
            let status = if took { "200 OK" } else { "401 Unauthorized" };
            let reply =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            let _ = sock.write_all(reply.as_bytes());
        }
    });
}

/// A stand-in for Jellyfin's sign-in on `listener`: `tok-1` for a body
/// holding `pw-1`, else 401.
fn sign_in_stand_in(listener: std::net::TcpListener) {
    use std::io::{BufRead, Read, Write};
    std::thread::spawn(move || {
        for sock in listener.incoming().flatten() {
            let mut sock = std::io::BufReader::new(sock);
            let (mut head, mut length) = (String::new(), 0);
            while !head.ends_with("\r\n\r\n") {
                let mut line = String::new();
                if sock.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                let lower = line.to_ascii_lowercase();
                if let Some(n) = lower.strip_prefix("content-length:") {
                    length = n.trim().parse().unwrap_or(0);
                }
                head += &line;
            }
            let mut body = vec![0; length];
            let _ = sock.read_exact(&mut body);
            let (status, answer) = match String::from_utf8_lossy(&body).contains("pw-1") {
                true => ("200 OK", r#"{"AccessToken":"tok-1"}"#),
                false => ("401 Unauthorized", ""),
            };
            let n = answer.len();
            let reply = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {n}\r\nConnection: close\r\n\r\n{answer}"
            );
            let _ = sock.get_mut().write_all(reply.as_bytes());
        }
    });
}

/// **A readers' key is pasted on the home page, tried, kept, and never
/// told** (a lender with no terminal; `readers: key x-api-key
/// /api/users/me`, CONTRACTS §4): `/lent` says the program takes a key, and
/// how to make one (`signs.accounts`); `POST /lent` `{"key": ...}` tries the
/// key once at its service's `/api/users/me`, a stand-in taking `k-1` alone:
/// one it refuses is refused in words saying what it answered, and kept
/// nowhere; one it takes is kept as the file `x-api-key`, which no answer
/// reads back; a blank paste is told to paste one, and keeps and deletes
/// nothing; a key for a program whose file names none is refused. Planted,
/// watched failing: the change kept the key under `authorization`; and the
/// key kept whatever its service answered (the wrong one kept).
#[tokio::test]
async fn a_readers_key_is_pasted_on_the_home_page_and_never_told() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (_alex, sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    tokio::time::timeout(Duration::from_secs(30), sam.round())
        .await
        .expect("Sam's computer hears it lets Sam in");
    let at = lend::lends_dir(sam_dir.path());
    std::fs::create_dir_all(&at).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    key_stand_in(listener);
    on_wire::forward(&sam, port);
    for (name, kind) in [("photos", "immich"), ("notes", "notes")] {
        let body = serde_json::json!({ "port": port, "kind": kind, "groups": [NS] });
        std::fs::write(
            at.join(format!("{name}.json")),
            svrngs_node::node::approving(&body),
        )
        .unwrap();
    }
    let (browser, home) = (
        &Browser::of(&sam),
        format!("svrngs.localhost:{}", sam.api_addr().port()),
    );
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let kept = sam_dir
        .path()
        .join("secrets/lends/photos/readers/x-api-key");
    let paste = |lend: &str, key: &str| format!(r#"{{"lend":"{lend}","to":{{"key":"{key}"}}}}"#);

    let listed = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(listed["readers"]["photos"]["key"], true, "{listed:#}");
    assert_eq!(listed["readers"]["photos"]["kept"], false, "{listed:#}");
    assert!(
        listed["readers"]["photos"]["signs"]["accounts"]
            .as_str()
            .is_some_and(|how| how.contains("New API Key")),
        "the steps to make the key, from the table: {listed:#}"
    );
    assert_eq!(
        listed["readers"]["notes"],
        serde_json::Value::Null,
        "{listed:#}"
    );

    let blank = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &paste("photos", "  "),
    )
    .await;
    assert_eq!(blank.status, 403, "{}", blank.body);
    assert_eq!(
        blank.json()["error"],
        "Paste the key first.",
        "{}",
        blank.body
    );
    assert!(!kept.exists(), "a blank paste keeps nothing");
    let none = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &paste("notes", "k-1"),
    )
    .await;
    assert_eq!(none.status, 404, "{}", none.body);
    assert!(
        !sam_dir
            .path()
            .join("secrets/lends/notes/readers/x-api-key")
            .exists(),
        "a program with no key row keeps none"
    );

    let wrong = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &paste("photos", "k-2"),
    )
    .await;
    assert_eq!(wrong.status, 403, "{}", wrong.body);
    assert_eq!(
        wrong.json()["error"],
        "immich refused this key: it answered 401. Make a new API key in immich, then paste that.",
        "{}",
        wrong.body
    );
    assert!(!kept.exists(), "a key its service refused is kept nowhere");
    let right = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &paste("photos", "k-1"),
    )
    .await;
    assert_eq!(right.status, 204, "{}", right.body);
    assert_eq!(std::fs::read_to_string(&kept).unwrap(), "k-1");
    let again = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &paste("photos", "  "),
    )
    .await;
    assert_eq!(again.status, 403, "{}", again.body);
    assert_eq!(
        std::fs::read_to_string(&kept).unwrap(),
        "k-1",
        "a blank paste after one deletes nothing"
    );
    let listed = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(listed["readers"]["photos"]["kept"], true, "{listed:#}");
    assert!(
        !listed.to_string().contains("k-1"),
        "what is kept is never answered: {listed:#}"
    );

    // Whether it is for programs, its lender's choice from the same row
    // (a group's page leads its cell with Play): kept, and listed, until
    // turned off. Planted, watched failing: the change written to nothing.
    for on in [true, false] {
        let to = format!(r#"{{"lend":"photos","to":{{"for-programs":{on}}}}}"#);
        let said = ask(browser, "POST", &home, "/lent", &own, &to).await;
        assert_eq!(said.status, 204, "{}", said.body);
        let listed = ask(browser, "GET", &home, "/lent", &own, "").await.json();
        let photos = listed["lent"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["name"] == "photos");
        let shown = photos.map(|l| l["for_programs"] == true);
        assert_eq!(shown, Some(on), "{listed:#}");
    }
}

/// **A header the door adds to each request is given on the home page**
/// (DESIGN §3, call 4): a lender with no terminal lends a llama-server
/// started with `--api-key`. `/lent` names, for each lend that may have them,
/// the headers kept, never their values; a header the door keeps for itself
/// or a value with a line break is refused and keeps nothing; a program that
/// signs each person in is refused any; one given is kept as `svrngs share`
/// keeps it, and Sam's door sends it on Alex's next request with no restart,
/// in place of the one Alex typed; an empty value stops it. Planted, watched
/// failing: the change kept the header under the readers' directory; and the
/// door sent a hand-written `x-svrngs-key` beside its own.
#[tokio::test]
async fn a_header_the_door_adds_is_given_on_the_home_page() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    tokio::time::timeout(Duration::from_secs(30), sam.round())
        .await
        .expect("Sam's computer hears it lets Sam in");
    let heads = head_on(&sam, 9002);
    let at = lend::lends_dir(sam_dir.path());
    std::fs::create_dir_all(&at).unwrap();
    for (name, kind) in [("coder", "llama-server"), ("photos", "immich")] {
        let body = serde_json::json!({ "port": 9002, "kind": kind, "groups": [NS] });
        std::fs::write(
            at.join(format!("{name}.json")),
            svrngs_node::node::approving(&body),
        )
        .unwrap();
    }
    let (browser, home) = (
        &Browser::of(&sam),
        format!("svrngs.localhost:{}", sam.api_addr().port()),
    );
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let kept = sam_dir.path().join("secrets/lends/coder/authorization");
    let add = |lend: &str, name: &str, value: &str| {
        serde_json::json!({ "lend": lend, "to": { "header": { "name": name, "value": value } } })
            .to_string()
    };
    let coder = Label::of(&signing(2).verifying_key().to_bytes(), "coder");
    // What Sam's llama-server saw of Alex's next request, which carries an
    // Authorization of Alex's own.
    let alexs = || async {
        let host = format!("{coder}.localhost:{}", alex.api_addr().port());
        let typed =
            "Sec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\nAuthorization: Bearer alexs\r\n";
        let got = ask(&Browser::of(&alex), "GET", &host, "/v1/models", typed, "").await;
        assert_eq!(got.status, 200, "Alex, through Sam's door: {}", got.body);
        heads
            .lock()
            .unwrap()
            .pop()
            .unwrap_or_default()
            .to_ascii_lowercase()
    };
    let refused = |got: Got, words: &str| {
        assert_eq!(
            (got.status, got.json()["error"].as_str().map(str::to_owned)),
            (403, Some(words.to_owned())),
            "{}",
            got.body
        );
    };

    let listed = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(
        listed["headers"]["coder"],
        serde_json::json!([]),
        "{listed:#}"
    );
    assert_eq!(
        listed["headers"]["photos"],
        serde_json::Value::Null,
        "{listed:#}"
    );
    assert!(
        alexs()
            .await
            .contains("\r\nauthorization: bearer alexs\r\n"),
        "with none kept, Alex's own passes"
    );

    for (name, value, why) in [
        ("x-svrngs-key", "sam", "the door's own"),
        ("host", "elsewhere", "the address"),
        ("transfer-encoding", "chunked", "a hop-by-hop one"),
        (
            "Authorization",
            "Bearer abc123\r\nx-svrngs-key: sam",
            "a value with a line break",
        ),
    ] {
        let got = ask(
            browser,
            "POST",
            &home,
            "/lent",
            &own,
            &add("coder", name, value),
        )
        .await;
        assert_eq!(got.status, 403, "{why}: {}", got.body);
    }
    assert!(
        !sam_dir.path().join("secrets/lends/coder").exists(),
        "nothing refused is kept"
    );
    refused(
        ask(
            browser,
            "POST",
            &home,
            "/lent",
            &own,
            &add("photos", "Authorization", "Bearer abc123"),
        )
        .await,
        "It signs each person in, so svrngs adds no header to it.",
    );
    assert!(!sam_dir.path().join("secrets/lends/photos").exists());

    let given = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &add("coder", "Authorization", "Bearer abc123"),
    )
    .await;
    assert_eq!(given.status, 204, "{}", given.body);
    assert_eq!(std::fs::read_to_string(&kept).unwrap(), "Bearer abc123");
    let listed = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(
        listed["headers"]["coder"],
        serde_json::json!(["authorization"]),
        "{listed:#}"
    );
    assert!(
        !listed.to_string().contains("abc123"),
        "never answered: {listed:#}"
    );
    let head = alexs().await;
    assert!(
        head.contains("\r\nauthorization: bearer abc123\r\n"),
        "the door's, with no restart: {head}"
    );
    assert_eq!(
        head.matches("\r\nauthorization:").count(),
        1,
        "Alex's own displaced: {head}"
    );

    // A file put there by hand is held to the same names.
    std::fs::write(kept.with_file_name("x-svrngs-key"), "sam").unwrap();
    let head = alexs().await;
    assert_eq!(
        head.matches("\r\nx-svrngs-key:").count(),
        1,
        "the door's identity alone: {head}"
    );

    let stopped = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &add("coder", "authorization", ""),
    )
    .await;
    assert_eq!(stopped.status, 204, "{}", stopped.body);
    assert!(!kept.exists(), "an empty value stops adding it");
    assert!(alexs()
        .await
        .contains("\r\nauthorization: bearer alexs\r\n"));
    sam.net().close().await;
    alex.net().close().await;
}

/// **The account apps read through, set on the home page** (capability
/// connections §5 step 5, §6 bar 1): `GET /lent` says which lends can have
/// one, and whether one is kept, never what is kept. A wrong password is
/// refused in words and keeps nothing; a right one is signed in with once
/// and kept as `svrngs share --readers` keeps it; a program with no way to
/// sign in is refused. The password is kept nowhere. Planted, watched
/// failing: the change kept the password in the readers' file.
#[tokio::test]
async fn the_apps_account_is_given_on_the_home_page() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    tokio::time::timeout(Duration::from_secs(30), sam.round())
        .await
        .expect("Sam's computer hears it lets Sam in");
    let jellyfin = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = jellyfin.local_addr().unwrap().port();
    sign_in_stand_in(jellyfin);
    on_wire::forward(&sam, port);
    let at = lend::lends_dir(sam_dir.path());
    std::fs::create_dir_all(&at).unwrap();
    for (name, kind) in [("films", "jellyfin"), ("notes", "notes")] {
        let body = serde_json::json!({ "port": port, "kind": kind, "groups": [NS] });
        std::fs::write(
            at.join(format!("{name}.json")),
            svrngs_node::node::approving(&body),
        )
        .unwrap();
    }
    let (browser, home) = (
        &Browser::of(&sam),
        format!("svrngs.localhost:{}", sam.api_addr().port()),
    );
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let readers = sam_dir
        .path()
        .join("secrets/lends/films/readers/authorization");
    let give = |lend: &str, pw: &str| {
        format!(
            r#"{{"lend":"{lend}","to":{{"readers":{{"name":"films-group","password":"{pw}"}}}}}}"#
        )
    };

    let listed = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(listed["readers"]["films"]["kept"], false, "{listed:#}");
    assert!(
        listed["readers"]["films"]["signs"]["accounts"]
            .as_str()
            .is_some_and(|how| how.contains("Dashboard, Users")),
        "the steps to make the account, from the table: {listed:#}"
    );
    assert_eq!(
        listed["readers"]["notes"],
        serde_json::Value::Null,
        "{listed:#}"
    );

    let wrong = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &give("films", "not-it"),
    )
    .await;
    assert_eq!(wrong.status, 403, "{}", wrong.body);
    assert!(
        wrong.json()["error"]
            .as_str()
            .is_some_and(|e| e.starts_with("films didn’t let that account in:")),
        "{}",
        wrong.body
    );
    assert!(!readers.exists(), "a wrong password kept something");

    let none = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &give("notes", "pw-1"),
    )
    .await;
    assert_eq!(none.status, 403, "{}", none.body);

    let right = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &give("films", "pw-1"),
    )
    .await;
    assert_eq!(right.status, 204, "{}", right.body);
    assert_eq!(
        std::fs::read_to_string(&readers).unwrap(),
        r#"MediaBrowser Token="tok-1""#
    );
    let listed = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(listed["readers"]["films"]["kept"], true, "{listed:#}");
    assert!(
        !listed.to_string().contains("tok-1") && !listed.to_string().contains("pw-1"),
        "what is kept is never answered: {listed:#}"
    );
    for entry in walkdir(sam_dir.path()) {
        let held = std::fs::read(&entry).unwrap_or_default();
        assert!(
            !held.windows(4).any(|w| w == b"pw-1"),
            "the password is kept at {entry:?}"
        );
    }
    drop(alex);
}

/// Every file under `dir`.
fn walkdir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    let mut files = Vec::new();
    while let Some(at) = stack.pop() {
        for entry in std::fs::read_dir(&at).into_iter().flatten().flatten() {
            let path = entry.path();
            match path.is_dir() {
                true => stack.push(path),
                false => files.push(path),
            }
        }
    }
    files
}

/// **A lend's limit is changed on the home page, and the door takes it with
/// no restart** (bash-limit (b)). Sam lends `shelf` to one person at once.
/// Sam opens it at his own computer and is let in; Alex, through hers, is
/// refused `full`, told the number is Sam's choice and his to change. A
/// change posted from anywhere but the home page's own origin is refused
/// `closed` and changes nothing; Sam's own, everyone, lets Alex in at the
/// door's next request; a number is kept; `GET /lent` says each; a lend
/// not lent is `unlent`; a stop withdraws it. Planted, watched failing:
/// `/lent` routed outside the home page's guard.
#[tokio::test]
async fn a_lend_s_limit_is_changed_on_the_home_page_and_the_door_takes_it() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    tokio::time::timeout(Duration::from_secs(30), sam.round())
        .await
        .expect("Sam's computer hears it lets Sam in");
    let _kept = head_on(&sam, 9001);
    let at = lend::lends_dir(sam_dir.path());
    std::fs::create_dir_all(&at).unwrap();
    let body = serde_json::json!({ "port": 9001, "kind": "books", "limit": 1, "groups": [NS] });
    std::fs::write(at.join("shelf.json"), svrngs_node::node::approving(&body)).unwrap();
    let shelf = Label::of(&signing(2).verifying_key().to_bytes(), "shelf");
    let name = lend::LendName::parse("shelf").unwrap();
    let limit = || {
        lend::read(sam_dir.path(), &name)
            .unwrap()
            .map(|l| l.terms.limit.map(|n| n.get()))
    };
    let typed = "Sec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\n";
    async fn open(node: &Node, shelf: &Label, typed: &str) -> Got {
        let browser = Browser::of(node);
        let host = format!("{shelf}.localhost:{}", node.api_addr().port());
        ask(&browser, "GET", &host, "/x", typed, "").await
    }
    let (browser, home) = (
        &Browser::of(&sam),
        format!("svrngs.localhost:{}", sam.api_addr().port()),
    );
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");

    let sams = open(&sam, &shelf, typed).await;
    assert_eq!(sams.status, 200, "Sam, the first: {}", sams.body);
    let alexs = open(&alex, &shelf, typed).await;
    assert_eq!(
        (alexs.status, alexs.header("x-svrngs-refused")),
        (503, Some("full")),
        "Alex, past the one: {}",
        alexs.body
    );
    assert!(
        alexs.json()["error"].as_str().unwrap().ends_with(
            "That number is the choice of the person who lends it, and they can change it."
        ),
        "{}",
        alexs.body
    );

    let everyone = r#"{"lend":"shelf","to":{"limit":null}}"#;
    for (row, extra) in [
        (
            "shelf's page",
            format!("Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{shelf}.localhost:{}\r\n", browser.api().port()),
        ),
        (
            "the home page's Origin, from another site",
            format!("Sec-Fetch-Site: same-site\r\nOrigin: http://{home}\r\n"),
        ),
        ("its own origin with no Origin", "Sec-Fetch-Site: same-origin\r\n".into()),
        ("a program, as you", browser.as_you()),
    ] {
        let posted = ask(browser, "POST", &home, "/lent", &extra, everyone).await;
        assert_eq!(
            (posted.status, posted.header("x-svrngs-refused")),
            (403, Some("closed")),
            "{row}'s change: {}",
            posted.body
        );
    }
    assert_eq!(limit(), Some(Some(1)), "no refused change was kept");

    let changed = ask(browser, "POST", &home, "/lent", &own, everyone).await;
    assert_eq!(changed.status, 204, "Sam's own change: {}", changed.body);
    let listed = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(listed["lent"][0]["name"], "shelf", "{listed:#}");
    assert_eq!(
        listed["lent"][0]["limit"],
        serde_json::Value::Null,
        "{listed:#}"
    );
    let alexs = open(&alex, &shelf, typed).await;
    assert_eq!(alexs.status, 200, "Alex, with no restart: {}", alexs.body);

    let two = r#"{"lend":"shelf","to":{"limit":2}}"#;
    assert_eq!(
        ask(browser, "POST", &home, "/lent", &own, two).await.status,
        204
    );
    assert_eq!(limit(), Some(Some(2)));
    let none = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        r#"{"lend":"pad","to":"stop"}"#,
    )
    .await;
    assert_eq!(
        (none.status, none.header("x-svrngs-refused")),
        (404, Some("unlent")),
        "{}",
        none.body
    );
    let stop = r#"{"lend":"shelf","to":"stop"}"#;
    assert_eq!(
        ask(browser, "POST", &home, "/lent", &own, stop)
            .await
            .status,
        204
    );
    assert_eq!(limit(), None, "no longer lent");
    let listed = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(listed["lent"], serde_json::json!([]), "{listed:#}");
    sam.net().close().await;
    alex.net().close().await;
}

/// **A lender turns on signing people in with svrngs from the home page**
/// (SIGN_IN_WITH_SVRNGS §5 "Turning it on", §8 9). Alex lends `shelf` with
/// no sign-in. `GET /lent` says so; `POST /lent` with `to: sign-in` makes a
/// client id and secret from the allowed alphabet, keeps them in the
/// declaration at 0600, and answers the issuer, the client and the
/// deviations in words once. After that `GET /lent` says the lend signs
/// people in, and never shows the secret again. Planted, watched failing:
/// the page's answer offering a client while the declaration holds none.
#[cfg(unix)]
#[tokio::test]
async fn a_lend_is_signed_in_with_svrngs_from_the_home_page() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    // A port never dialled: turning sign-in on asks nothing of the service.
    let lends = lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = serde_json::json!({ "port": 9, "kind": "books", "groups": [NS] });
    std::fs::write(
        lends.join("shelf.json"),
        svrngs_node::node::approving(&body),
    )
    .unwrap();
    let name = lend::LendName::parse("shelf").unwrap();
    let api = alex.api_addr();
    let browser = &Browser::of(&alex);
    let home = format!("svrngs.localhost:{}", api.port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let alphabet = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b))
    };

    let before = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(
        before["signing"],
        serde_json::json!([]),
        "nothing signs people in yet: {before:#}"
    );

    let turned = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        r#"{"lend":"shelf","to":"sign-in"}"#,
    )
    .await;
    assert_eq!(turned.status, 200, "turning it on: {}", turned.body);
    let shown = turned.json();
    let issuer = format!("http://127.0.0.1:{}/oidc/shelf", api.port());
    assert_eq!(shown["issuer"], issuer.as_str(), "{shown:#}");
    let id = shown["client_id"].as_str().expect("a client id");
    let secret = shown["client_secret"].as_str().expect("a secret");
    assert!(
        alphabet(id) && alphabet(secret),
        "the client is made from the allowed alphabet: {shown:#}"
    );
    let deviations = shown["deviations"].as_str().expect("the deviations");
    for word in ["plain http", "ES256", "no email"] {
        assert!(deviations.contains(word), "{deviations}");
    }

    let declared = lend::read(dir.path(), &name).unwrap().unwrap();
    let client = declared
        .terms
        .sign_in_with_svrngs
        .expect("the declaration holds the client");
    assert_eq!(client.client_id, id);
    assert_eq!(client.client_secret.as_deref(), Some(secret));
    let file = lends.join("shelf.json");
    assert_eq!(
        std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o600,
        "the declaration holding a secret is owner-only"
    );

    let after = ask(browser, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(after["signing"], serde_json::json!(["shelf"]), "{after:#}");
    assert!(
        !after.to_string().contains(secret),
        "the secret is never shown again: {after:#}"
    );
    alex.net().close().await;
}

/// **Groups that cannot be read are said so** (ralph/REVIEW_FINDINGS.md
/// #103): with Alex's groups unlistable, a lent page's `/svrngs-reads` is
/// svrngs's `fault`, never "nothing is lent here". Watched failing first,
/// at 2e6edc7's `shared_with`, which read it as no group: 404 `unlent`.
#[tokio::test]
async fn groups_that_cannot_be_read_are_said_so() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    let seed = Roster::new(
        [(Person::parse("alex").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    alex.rail().set_seed(NS, &seed).unwrap();
    declare(dir.path(), "night", 1, "night", &[], None);
    let night = Label::of(&signing(1).verifying_key().to_bytes(), "night");
    let rings = dir.path().join("rings");
    std::fs::set_permissions(&rings, std::fs::Permissions::from_mode(0o000)).unwrap();
    let got = ask(
        &Browser::of(&alex),
        "GET",
        &format!("{night}.localhost:{}", alex.api_addr().port()),
        "/svrngs-reads",
        "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\n",
        "",
    )
    .await;
    std::fs::set_permissions(&rings, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        (got.status, got.header("x-svrngs-refused")),
        (500, Some("fault")),
        "{}",
        got.body
    );
    alex.net().close().await;
}

/// **svrngs's own answers at a lend's origin are readable by a page that
/// reads the lend, and by no other** (lean-4 (b); sup-4's follow-up 1,
/// `demo/runs/sup/2026-10-10/README.md`). Alex lends `night`, reading
/// `media`, and `telly` (asking), `shows` (refusing `/x`) and a `diary`
/// (`notes`); Sam lends `films` (refusing `/x`) and `reels`, its service
/// not listening. `night`'s page, by `fetch`, reads each refusal's kind —
/// its origin alone allowed, `x-svrngs-refused` exposed: `unasked` at
/// `telly`, `stopped` there once it stops itself, `path` at `shows` and at
/// Sam's `films`, `away` at Sam's `reels`. A service's own answer carries
/// what the service sent. The `diary`, which it does not read, is
/// `closed` and allows nobody; so is a fetch from the home page's origin,
/// and the home page allows `night` nothing. Planted, watched failing: the
/// allowance given to any origin, `*`.
#[tokio::test]
async fn svrngs_s_own_answers_are_readable_by_a_page_that_reads_the_lend_and_no_other() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alex, sam) = alex_lets_sam_in(alex_dir.path(), sam_dir.path()).await;
    // A port nothing serves on the wire: Sam's reels, stopped.
    let silent = 9099;
    for (dir, name, kind, reads, asks, port) in [
        (
            alex_dir.path(),
            "night",
            "night",
            &["media"][..],
            None,
            9001,
        ),
        (alex_dir.path(), "telly", "media", &[], Some(WORDS), 9002),
        (alex_dir.path(), "diary", "notes", &[], None, 9003),
    ] {
        head_on(&alex, port);
        declare(dir, name, port, kind, reads, asks);
    }
    for (node, dir, name, port, refused) in [
        (&alex, alex_dir.path(), "shows", 9004, &["/x"][..]),
        (&sam, sam_dir.path(), "films", 9001, &["/x"]),
        (&sam, sam_dir.path(), "reels", silent, &[]),
    ] {
        if port != silent {
            head_on(node, port);
        }
        let body = serde_json::json!({ "port": port, "kind": "media", "limit": 3, "refused": refused, "groups": [NS] });
        std::fs::create_dir_all(lend::lends_dir(dir)).unwrap();
        std::fs::write(
            lend::lends_dir(dir).join(format!("{name}.json")),
            svrngs_node::node::approving(&body),
        )
        .unwrap();
    }
    let api = alex.api_addr();
    let browser = &Browser::of(&alex);
    let port = api.port();
    let label = |seed: u8, name: &str| Label::of(&signing(seed).verifying_key().to_bytes(), name);
    let night = label(1, "night");
    let at = |l: &Label| format!("{l}.localhost:{port}");
    let page = format!("http://{}", at(&night));
    let home = format!("svrngs.localhost:{port}");
    let from = |origin: &str| {
        format!("Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: cors\r\nOrigin: {origin}\r\n")
    };
    let read = |got: &Got| {
        (
            got.status,
            got.header("x-svrngs-refused").map(str::to_string),
            got.header("access-control-allow-origin")
                .map(str::to_string),
            got.header("access-control-expose-headers")
                .map(str::to_string),
        )
    };
    let readable = |status: u16, kind: &str| {
        (
            status,
            Some(kind.to_string()),
            Some(page.clone()),
            Some("x-svrngs-refused".to_string()),
        )
    };
    let unread = |status: u16, kind: Option<&str>| (status, kind.map(str::to_string), None, None);
    let night_at = |l: Label, path: &'static str| {
        let (host, extra) = (at(&l), from(&page));
        async move { ask(browser, "GET", &host, path, &extra, "").await }
    };
    let rows = [
        (
            "telly, unasked",
            label(1, "telly"),
            "/x",
            readable(403, "unasked"),
        ),
        (
            "shows, a refused path",
            label(1, "shows"),
            "/x",
            readable(403, "path"),
        ),
        (
            "Sam's films, a refused path",
            label(2, "films"),
            "/x",
            readable(403, "path"),
        ),
        (
            "Sam's reels, not listening",
            label(2, "reels"),
            "/x",
            readable(502, "away"),
        ),
        (
            "Sam's films, its own answer",
            label(2, "films"),
            "/",
            unread(200, None),
        ),
        (
            "the diary, not read",
            label(1, "diary"),
            "/x",
            unread(403, Some("closed")),
        ),
    ];
    for (row, l, path, want) in rows {
        let got = night_at(l, path).await;
        eprintln!("{row}: {:?}", read(&got));
        assert_eq!(read(&got), want, "{row}: {}", got.body);
        if want.2.is_some() {
            assert!(
                got.head.to_ascii_lowercase().contains("vary: origin"),
                "{row}: {}",
                got.head
            );
        }
    }
    let same = "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\n";
    let telly = label(1, "telly");
    let stop = format!("/svrngs-reads/{telly}/svrngs-stop");
    let stopped = ask(browser, "POST", &at(&night), &stop, same, "").await;
    assert_eq!(stopped.status, 200, "night stops itself: {}", stopped.body);
    let got = night_at(telly.clone(), "/x").await;
    assert_eq!(read(&got), readable(403, "stopped"), "{}", got.body);

    let from_home = ask(
        browser,
        "GET",
        &at(&telly),
        "/x",
        &from(&format!("http://{home}")),
        "",
    )
    .await;
    assert_eq!(
        read(&from_home),
        unread(403, Some("closed")),
        "the home page's fetch"
    );
    let at_home = ask(browser, "GET", &home, "/v1/lends", &from(&page), "").await;
    assert_eq!(
        read(&at_home),
        unread(403, Some("closed")),
        "night's fetch at the home page"
    );
    sam.net().close().await;
    alex.net().close().await;
}

/// **The page's link for a group this computer does not hold is refused,
/// and traced on both sides** (#140): Alex's home page asks a link to
/// `elsewhere`, which Alex's computer does not hold, and is told 403, of
/// the kind `not_in` that `svrngs link` words as its own refusal; the
/// trace says the link refused it (invite.rs `Host::link`) and the page
/// said so (home.rs `link`), as `svrngs link` traces its own refusal. Red
/// on the plant the finding names, both branches untraced: neither line.
#[tokio::test]
async fn the_page_s_link_for_a_group_not_held_is_refused_and_traced() {
    let (said, _heard) = heard("link=debug,home=debug");
    let dir = tempfile::tempdir().unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    let home = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let body = r#"{"group":"elsewhere","device":false}"#;
    let got = ask(&Browser::of(&alex), "POST", &home, "/link", &own, body).await;
    let kind = serde_json::from_str::<serde_json::Value>(&got.body).unwrap()["kind"].clone();
    assert_eq!((got.status, kind), (403, "not_in".into()), "{}", got.body);
    let log = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let traced = |target: &str| {
        log.lines()
            .any(|l| l.contains(target) && l.contains("elsewhere"))
    };
    assert!(
        traced(" link: link: refused"),
        "the link's refusal traced: {log}"
    );
    assert!(
        traced(" home: home: refused"),
        "the page's refusal traced: {log}"
    );
    alex.net().close().await;
}

/// **A link the page makes before this computer has a home names the
/// home it comes to.** The first iroh report, which sets the home, takes
/// its whole 3 s where no IPv6 probe answers, and the page can ask sooner.
/// A relay of the build's, stopped, sorts first, as aps1-1 does among n0's;
/// a group that names none of its own sends a link with no home to the
/// sorted first, where this computer never registers, and a joiner who is
/// told nothing else waits out `ANSWERED_WITHIN`
/// (`home_group.rs`, `Bo is let in: Err(IssuerAway)`). Red, watched failing,
/// on the route with no wait: the link names the stopped relay.
#[tokio::test(flavor = "multi_thread")]
async fn a_link_the_page_makes_before_a_home_names_the_home_it_comes_to() {
    let relay = || async {
        let mut config = ServerConfig::default();
        config.relay = Some(RelayConfig::new(([127, 0, 0, 1], 0)));
        Server::spawn(config).await.expect("a relay on 127.0.0.1")
    };
    let mut relays = [relay().await, relay().await];
    let at = |r: &Server| format!("http://{}/", r.http_addr().unwrap());
    relays.sort_by_key(at);
    let [stopped, live] = relays;
    let urls = [at(&stopped), at(&live)];
    stopped.shutdown().await.unwrap();

    let computer = |seed: u8, relay_urls: Vec<String>, dir: std::path::PathBuf| async move {
        let reach = svrngs_node::iroh::RelayConfig {
            relay_urls,
            n0_services: false,
            loopback_only: true,
        };
        let key = SigningKey::from_bytes(&[seed; 32]);
        let any = "127.0.0.1:0".parse().unwrap();
        Node::start(dir, key, &reach, group::seed_text, any)
            .await
            .expect("a computer starts")
    };
    let dir = tempfile::tempdir().unwrap();
    let alex = computer(1, urls.to_vec(), dir.path().into()).await;
    let house = group::draw(
        group::Name::parse("house").unwrap(),
        group::Name::parse("alex").unwrap(),
        SigningKey::from_bytes(&[1; 32]).verifying_key(),
    );
    // Its API asked of its routes: the claim is the home iroh reports,
    // not the listener.
    on_wire::served(&alex);
    let kept = alex.host().keep_group(&house, None, &[]);
    kept.expect("Alex keeps a group that names no relay of its own");
    let home = || alex.endpoint().addr().relay_urls().next().cloned();
    assert_eq!(home(), None, "asked before this computer has a home");

    let page = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{page}\r\n");
    let body = serde_json::json!({ "group": house.id(), "device": false }).to_string();
    let got = ask(&Browser::of(&alex), "POST", &page, "/link", &own, &body).await;
    assert_eq!(got.status, 200, "{}", got.body);
    let link = Link::parse(got.json()["client"].as_str().unwrap()).expect("a link");
    let carried: Vec<String> = link.token.relays.iter().map(ToString::to_string).collect();
    assert_eq!(
        carried,
        [urls[1].clone()],
        "the relay this computer is met at"
    );

    let dir = tempfile::tempdir().unwrap();
    let bo = computer(2, vec![urls[1].clone()], dir.path().into()).await;
    let joined = bo
        .host()
        .join(&link, &group::Name::parse("bo").unwrap())
        .await;
    assert!(joined.is_ok(), "Bo is let in: {joined:?}");
    alex.endpoint().close().await;
    bo.endpoint().close().await;
}

/// **Lending from the page, each refusal said apart and traced**
/// (v1ui-findings (b); #161): `POST /running`'s two closed sets, what the
/// page sent (axum's `JsonRejection`) and the lend's own refusal
/// (`ShareRefusal`), each said in its own words: a port of 0, which is
/// nothing running here; a name that is not one word; what it is, not
/// one word; a body that names no groups, which must be chosen; a body
/// that is not JSON; and nothing answering at a port.
/// Each is traced with what was refused. Red on the plant the finding
/// names: a port of 0, told as before that a name and what it is are
/// each one word, and untraced.
#[tokio::test]
async fn lending_from_the_page_says_each_refusal_apart_and_traces_it() {
    let (said, _heard) = heard("home=debug");
    let dir = tempfile::tempdir().unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    // The lender must hold the group it lends to; without one, an
    // explicit selection is refused before the port is ever checked.
    let seed = Roster::new(
        [(
            svrngs_node::Person::parse("alex").unwrap(),
            vec![svrngs_node::actor_of(&signing(1))],
        )]
        .into_iter()
        .collect(),
    );
    alex.rail().set_seed(NS, &seed).unwrap();
    let home = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    // A port nothing answers at: bound, then let go.
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = free.local_addr().unwrap().port();
    drop(free);
    let nothing = svrngs_node::uid::Answering::default();
    on_wire::answers(&alex, port, Ok(nothing));
    let asked = [
        (
            r#"{"port":0,"name":"pad","kind":"pad"}"#.to_string(),
            "That is not something running on this computer: look again, then lend it from the list.",
        ),
        (
            format!(r#"{{"port":{port},"name":"my pad","kind":"pad","groups":["house"]}}"#),
            "Give it a name of one word: letters and digits, with - or _ between.",
        ),
        (
            format!(r#"{{"port":{port},"name":"pad","kind":"a pad","groups":["house"]}}"#),
            "Say which program it is in one word: letters and digits, with - or _ between.",
        ),
        (
            format!(r#"{{"port":{port},"name":"pad","kind":"pad"}}"#),
            "Choose which groups can open this first.",
        ),
        (
            "port=8080".to_string(),
            "This computer could not read what the page sent: reload the page, then try again.",
        ),
        (
            format!(r#"{{"port":{port},"name":"pad","kind":"pad","groups":["house"]}}"#),
            &*format!("Nothing answers at http://localhost:{port} now: start it, then lend it."),
        ),
    ];
    for (body, words) in &asked {
        let got = ask(&Browser::of(&alex), "POST", &home, "/running", &own, body).await;
        assert_eq!(
            (got.status, got.json()["error"].as_str()),
            (400, Some(*words)),
            "{body}"
        );
    }
    let log = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let refused: Vec<&str> = (log.lines())
        .filter(|l| l.contains("home: refused — "))
        .collect();
    assert_eq!(refused.len(), asked.len(), "each refusal traced: {log}");
    assert!(
        refused.iter().all(|l| l.contains(" error=")),
        "each with what refused it: {log}"
    );
    assert!(refused[0].contains("expected a nonzero u16"), "{log}");
    alex.net().close().await;
}

/// **What the page lends is what it showed** (SHARE_BINDING §5.1): lending
/// from the page with the pid and start it showed approves the socket read
/// on the port, and nothing else; with none shown, it is refused 400 with
/// `svrngs share` as the fix; a pid that holds none of the port's sockets,
/// or one that started since, is refused 400 in `svrngs share --pid`'s
/// words; a pid without its start, or a start without its pid, refused 422
/// in the API's shape, as the approve route refuses it; a port only another
/// account's program answers, as a Mac reads it, refused with the fix to
/// run it as you. Nothing is lent but the one. Planted, each watched
/// failing: the pid not sent to the approval (lent with no pid); the holder
/// check skipped (lent with pid 999); the pid and start zipped, half a
/// process dropped (refused 400 as nothing shown).
#[tokio::test]
async fn what_the_page_lends_is_what_it_showed() {
    let dir = tempfile::tempdir().unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    let seed = Roster::new(
        [(
            svrngs_node::Person::parse("alex").unwrap(),
            vec![svrngs_node::actor_of(&signing(1))],
        )]
        .into_iter()
        .collect(),
    );
    alex.rail().set_seed(NS, &seed).unwrap();
    let home = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let running = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = running.local_addr().unwrap().port();
    let lent = |name: &str, shown: &str| {
        format!(r#"{{"port":{port},"name":"{name}","kind":"pad","groups":["{NS}"]{shown}}}"#)
    };
    let refused = |words: String| (400, Some(words));
    let lends = |body: String| {
        let alex = &alex;
        let (home, own) = (home.clone(), own.clone());
        async move {
            let got = ask(&Browser::of(alex), "POST", &home, "/running", &own, &body).await;
            (got.status, got.json()["error"].as_str().map(str::to_owned))
        }
    };
    let unshown = format!("svrngs can’t tell which program answers on port {port}, so it can’t be lent from here: `svrngs share {port} --group <group>` lends it.");
    assert_eq!(lends(lent("pad", "")).await, refused(unshown));
    let also = format!("svrngs can't share port {port}: another program also listens on port {port}. Stop it, or choose that one, then share it again.");
    assert_eq!(
        lends(lent("pad", r#","pid":999,"start":1"#)).await,
        refused(also)
    );
    let stopped = format!("svrngs can't share port {port}: that program stopped; choose it again.");
    assert_eq!(
        lends(lent("pad", r#","pid":1,"start":2"#)).await,
        refused(stopped)
    );
    let half = Some("send a process's pid with its start, or neither".to_string());
    for shown in [r#","pid":1"#, r#","start":1"#] {
        assert_eq!(
            lends(lent("pad", shown)).await,
            (422, half.clone()),
            "{shown}"
        );
    }
    let mut theirs = svrngs_node::uid::Answering::default();
    theirs.read.unlisted = true;
    on_wire::answers(&alex, port, Ok(theirs));
    let words = format!("svrngs can't share port {port}: the program there doesn't run as you, and on a Mac svrngs can keep a share only to programs that run as you. Run it as you, then share it again.");
    assert_eq!(
        lends(lent("pad", r#","pid":1,"start":1"#)).await,
        refused(words)
    );
    let pad = lend::LendName::parse("pad").unwrap();
    assert_eq!(lend::read(dir.path(), &pad).unwrap(), None, "nothing lent");
    on_wire::answers(&alex, port, Ok(svrngs_node::node::answering()));
    let body = lent("pad", r#","pid":1,"start":1"#);
    let review = ask(&Browser::of(&alex), "POST", &home, "/running", &own, &body).await;
    let token = review.json()["token"]
        .as_str()
        .expect("the page got a review")
        .to_owned();
    assert_eq!(
        lend::read(dir.path(), &pad).unwrap(),
        None,
        "the review alone shares nothing"
    );
    lend::withdraw(dir.path(), &pad).unwrap();
    let confirm = |token: &str| format!(r#"{},"token":"{token}"}}"#, body.trim_end_matches('}'));
    let again = ask(
        &Browser::of(&alex),
        "POST",
        &home,
        "/running/confirm",
        &own,
        &confirm(&token),
    )
    .await;
    assert_eq!(
        lend::read(dir.path(), &pad).unwrap(),
        None,
        "a stale review never publishes"
    );
    let fresh = again.json()["token"]
        .as_str()
        .expect("a fresh review")
        .to_owned();
    assert_ne!(fresh, token);
    let saved = ask(
        &Browser::of(&alex),
        "POST",
        &home,
        "/running/confirm",
        &own,
        &confirm(&fresh),
    )
    .await;
    assert_eq!(saved.status, 200, "{}", saved.json());
    let approved = lend::read(dir.path(), &pad)
        .unwrap()
        .unwrap()
        .terms
        .approved;
    assert_eq!(approved, Some(svrngs_node::node::approval()));
    alex.net().close().await;
}

/// **Keep sharing it approves only what the page showed** (SHARE_BINDING
/// §5.4; SHARE_BINDING_REVIEWS §6 #6, #7): shelf, held since something not
/// approved was seen on its port and kept so in its file, is listed held
/// with what answers now and a token for it, though only its approved
/// socket answers now. Approving with that token after what answers changed
/// is refused in words that say so; with none, refused; with the token of
/// what answers but asked to keep it to the job svrngs runs (`keep`, as the
/// page's Keep it running asks), refused, nothing saved, since svrngs keeps
/// no job of it (`share::kept`: on Linux a program in no unit, on a Mac no
/// launch agent of shelf's name); with that token alone, approved, and
/// listed held no more. Planted, each watched failing: `share::approve` not
/// comparing the token (the changed set approved); the list reading no hold
/// from the file (shelf listed open); the route ignoring `keep` (approved,
/// 200).
#[tokio::test]
async fn keep_sharing_it_approves_only_what_the_page_showed() {
    // A port nothing listens on, so the wire's socket 1 is what answers.
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = free.local_addr().unwrap().port();
    drop(free);
    let dir = tempfile::tempdir().unwrap();
    let lends = lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let held = r#"{"sockets":[1],"at":"127.0.0.1","held":true}"#;
    let body = format!(r#"{{"port":{port},"kind":"pad","groups":["{NS}"],"approved":{held}}}"#);
    std::fs::write(lends.join("shelf.json"), body).unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    let seed = Roster::new(
        [(
            svrngs_node::Person::parse("alex").unwrap(),
            vec![svrngs_node::actor_of(&signing(1))],
        )]
        .into_iter()
        .collect(),
    );
    alex.rail().set_seed(NS, &seed).unwrap();
    let home = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let (home, own) = (home.as_str(), own.as_str());
    let browser = &Browser::of(&alex);
    let row = || async {
        let path = format!("/v1/lends?namespace={NS}");
        let listed = ask(browser, "GET", home, &path, own, "").await.json();
        let rows = listed["lent"].as_array().cloned().unwrap_or_default();
        let row = rows.into_iter().find(|l| l["name"] == "shelf");
        row.unwrap_or_else(|| panic!("shelf listed: {listed:#}"))
    };
    let shown = row().await;
    assert_eq!(shown["held"], "another", "{shown:#}");
    assert_eq!(shown["answers"][0]["program"], "test", "{shown:#}");
    let token = shown["token"].as_str().unwrap().to_string();
    let approve = |body: String| async move {
        let got = ask(browser, "POST", home, "/v1/lends/shelf/approve", own, &body).await;
        (got.status, got.json()["error"].as_str().map(str::to_owned))
    };
    let mut changed = svrngs_node::node::answering();
    changed.read.sockets[0].id = 2;
    on_wire::answers(&alex, port, Ok(changed));
    let words = format!(
        "svrngs can't share port {port}: what answers on port {port} changed; choose it again."
    );
    let with = format!(r#"{{"token":"{token}","pid":1,"start":1}}"#);
    assert_eq!(approve(with.clone()).await, (403, Some(words)));
    let path = "/v1/lends/shelf/approve";
    let unshown = ask(browser, "POST", home, path, own, "null").await;
    assert_eq!(unshown.status, 422, "no token: {}", unshown.body);
    on_wire::answers(&alex, port, Ok(svrngs_node::node::answering()));
    let shelf = || lend::read(dir.path(), &lend::LendName::parse("shelf").unwrap());
    let kept = format!(r#"{{"token":"{token}","pid":1,"start":1,"keep":true}}"#);
    let unkept = "svrngs doesn't run what answers on shelf's port, so it wasn't approved. Check what answers there, then approve it.";
    assert_eq!(approve(kept).await, (403, Some(unkept.into())));
    let still = shelf().unwrap().unwrap().terms;
    assert!(still.approved.is_some_and(|a| a.held), "nothing approved");
    assert_eq!(still.keep, None, "nothing kept");
    assert_eq!(approve(with).await.0, 200);
    let shelf = shelf();
    let approved = shelf.unwrap().unwrap().terms.approved.unwrap();
    assert_eq!(approved, svrngs_node::node::approval(), "held no more");
    assert_eq!(row().await["held"], serde_json::Value::Null);
    alex.net().close().await;
}

/// **Keep sharing it is offered only where approving it would be taken**
/// (SHARE_BINDING §5.1, share.rs `approve` the one decider): shelf, held,
/// is listed with a token only for a read the approve route takes. Nothing
/// answering, a Mac's port only another account answers, a socket whose
/// program can't be read (a Mac's whose holder exited, or on Linux one of
/// this account's whose files can't be read), no reader and a read that
/// failed each get none; a read that couldn't be made, or a program that
/// couldn't, a Mac's of another account's among them, says why in `unread`,
/// apart from nothing answering (`answers: []`). Linux's socket of another account, shown as its account, gets one,
/// and its token alone approves it; beside it a socket on a LAN address,
/// which the door never dials, is neither shown nor approved. The route
/// refuses the unjudged socket's token in words, and a pid without its
/// start, or a start without its pid, 422 in the API's shape, the lend still
/// held. Planted, each watched failing: the route's pid and start zipped,
/// half a process dropped (approved, 200); the list's answers `[]` for a
/// read that failed, no `unread`; its token from `binding::approve` alone
/// (the unjudged socket given one); a token for every held lend; a socket
/// shown nothing let through (Linux's unread program given a token, said as
/// nothing answering); every socket's entry shown (the LAN one's program
/// listed beside the account); a Mac's port another account answers given
/// no `unread` (said as nothing answering).
#[tokio::test]
async fn keep_sharing_it_is_offered_only_where_approving_it_would_be_taken() {
    use serde_json::{json, Value};
    use svrngs_node::binding::{Read, Socket};
    use svrngs_node::uid::{Answering, Unread};
    use svrngs_node::{lend::Answer, share};
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = free.local_addr().unwrap().port();
    drop(free);
    let dir = tempfile::tempdir().unwrap();
    let lends = lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let held = r#"{"sockets":[1],"at":"127.0.0.1","held":true}"#;
    let body = format!(r#"{{"port":{port},"kind":"pad","groups":["{NS}"],"approved":{held}}}"#);
    std::fs::write(lends.join("shelf.json"), body).unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    let seed = Roster::new(
        [(
            svrngs_node::Person::parse("alex").unwrap(),
            vec![svrngs_node::actor_of(&signing(1))],
        )]
        .into_iter()
        .collect(),
    );
    alex.rail().set_seed(NS, &seed).unwrap();
    let home = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let (home, own) = (home.as_str(), own.as_str());
    let browser = &Browser::of(&alex);
    let row = || async {
        let path = format!("/v1/lends?namespace={NS}");
        let listed = ask(browser, "GET", home, &path, own, "").await.json();
        let rows = listed["lent"].as_array().cloned().unwrap_or_default();
        let row = rows.into_iter().find(|l| l["name"] == "shelf");
        row.unwrap_or_else(|| panic!("shelf listed: {listed:#}"))
    };
    let approve = |body: Value| async move {
        let path = "/v1/lends/shelf/approve";
        let got = ask(browser, "POST", home, path, own, &body.to_string()).await;
        (got.status, got.json()["error"].as_str().map(str::to_owned))
    };
    let shelf = lend::LendName::parse("shelf").unwrap();
    let approved = || {
        lend::read(dir.path(), &shelf)
            .unwrap()
            .unwrap()
            .terms
            .approved
    };
    let socket = |id, holders: &[(u32, u64)]| Socket {
        id,
        cgroup: None,
        holders: holders.iter().copied().collect(),
        at: std::net::Ipv4Addr::LOCALHOST.into(),
        dual: false,
        peer: svrngs_node::binding::Peer::Unread,
    };
    let read = |lasting, sockets| Read {
        boot: None,
        lasting,
        sockets,
        unlisted: false,
    };
    let answering = |read, shown| Answering {
        read,
        parents: Default::default(),
        start: None,
        shown,
        beside: false,
    };
    let mut theirs = Answering::default();
    theirs.read.unlisted = true;
    let unjudged = answering(read(false, vec![socket(3, &[])]), Vec::new());
    let account = Answer::Account {
        account: "_resolve".into(),
    };
    let elsewhere = Socket {
        at: "192.168.1.5".parse().unwrap(),
        ..socket(8, &[(2, 2)])
    };
    let lan = Answer::Program {
        pid: 2,
        start: 2,
        program: "other".into(),
        folder: None,
    };
    let sockets = vec![socket(7, &[]), elsewhere];
    let linux = answering(read(true, sockets), vec![(7, account), (8, lan)]);
    let unshown = answering(read(true, vec![socket(9, &[])]), Vec::new());
    let program = json!({ "why": "failed", "detail": "its program can't be read" });
    let listed = [
        (
            "nothing answers",
            Ok(Answering::default()),
            json!([]),
            Value::Null,
        ),
        (
            "a Mac's other account",
            Ok(theirs),
            Value::Null,
            json!({ "why": "failed", "detail": "a program of another account answers there" }),
        ),
        (
            "its program unread",
            Ok(unjudged.clone()),
            Value::Null,
            program.clone(),
        ),
        ("Linux's program unread", Ok(unshown), Value::Null, program),
        (
            "no reader",
            Err(Unread::NoReader),
            Value::Null,
            json!({ "why": "no_reader" }),
        ),
        (
            "a failed read",
            Err(Unread::Failed("the read stopped")),
            Value::Null,
            json!({ "why": "failed", "detail": "the read stopped" }),
        ),
    ];
    let mut wrong = Vec::new();
    for (case, read, answers, unread) in listed {
        on_wire::answers(&alex, port, read);
        let shown = row().await;
        let said = ["held", "answers", "unread", "token"].map(|f| shown[f].clone());
        if said != [json!("another"), answers, unread, Value::Null] {
            wrong.push(format!("{case}: {said:?}"));
        }
    }
    assert!(wrong.is_empty(), "each listed as it should be: {wrong:#?}");
    let token = share::token(&svrngs_node::binding::approve(&unjudged.read, None).unwrap());
    on_wire::answers(&alex, port, Ok(unjudged));
    let words = format!("svrngs can't share port {port}: it can't read what answers there (its program can't be read). Try again.");
    assert_eq!(approve(json!({ "token": token })).await, (403, Some(words)));
    on_wire::answers(&alex, port, Ok(svrngs_node::node::answering()));
    let token = row().await["token"].clone();
    assert!(token.is_string(), "a token for what answers");
    let half = Some("send a process's pid with its start, or neither".to_string());
    for asked in [
        json!({ "token": token, "pid": 1 }),
        json!({ "token": token, "start": 1 }),
    ] {
        assert_eq!(approve(asked.clone()).await, (422, half.clone()), "{asked}");
    }
    assert!(approved().unwrap().held, "still held");
    on_wire::answers(&alex, port, Ok(linux.clone()));
    let shown = row().await;
    assert_eq!(
        shown["answers"],
        json!([{ "account": "_resolve" }]),
        "{shown:#}"
    );
    let token = shown["token"]
        .as_str()
        .expect("a token for another account's socket");
    assert_eq!(approve(json!({ "token": token })).await, (200, None));
    let theirs = svrngs_node::binding::approve(&linux.read, None).ok();
    assert_eq!(approved(), theirs, "its socket approved by its token alone");
    assert_eq!(
        theirs.map(|a| a.sockets),
        Some([7].into()),
        "the LAN one not"
    );
    alex.net().close().await;
}

/// **Looking for what runs here, a failed read said and traced by which**
/// (v1ui-findings (d); #160): Alex's computer cannot read what it lends —
/// its lends' directory is a file — so `GET /running` refuses, saying
/// that read is the one that failed, and its trace carries the error.
/// Red on the plant: the refusal as before, "this computer could not be
/// asked", its warn naming neither read nor error.
#[tokio::test]
async fn looking_for_what_runs_here_says_which_read_failed_and_traces_why() {
    let (said, _heard) = heard("home=debug");
    let dir = tempfile::tempdir().unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    let lends = lend::lends_dir(dir.path());
    let _ = std::fs::remove_dir_all(&lends);
    std::fs::write(&lends, "not a directory").unwrap();
    let home = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own = "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\n";
    let got = ask(&Browser::of(&alex), "GET", &home, "/running", own, "").await;
    assert_eq!(
        (got.status, got.json()["error"].as_str()),
        (500, Some("it cannot read what you lend already")),
        "{}",
        got.body
    );
    let log = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let traced = (log.lines()).find(|l| l.contains("home: refused — "));
    let traced = traced.unwrap_or_else(|| panic!("the refusal traced: {log}"));
    assert!(
        traced.contains("it cannot read what you lend already")
            && traced.contains("cannot be read: Not a directory"),
        "which read, and its error: {traced}"
    );
    alex.net().close().await;
}

/// **The home page's modules are served, each as its file, and no other**
/// (home.rs `served`; page-scaffold (g), page-switchover). Every `.js` in
/// src/home/page/, src/home/ui/ and web/vendor/, read from the tree as
/// this test runs, is answered at `/page/`, `/ui/` and `/vendor/` with its
/// bytes, as JavaScript, under the page's one policy, so a module the
/// table leaves out fails here and not as a page that never draws. A name
/// not there, another case, another kind, and a `../` up to a file that is
/// there are each 404. Red on the plant: `served`'s table without `"wifi"`,
/// whose `/page/wifi.js` was 404.
#[tokio::test]
async fn the_home_page_s_modules_are_served_each_as_its_file_and_no_other() {
    let dir = tempfile::tempdir().unwrap();
    let node = on_wire::start(1, dir.path()).await;
    let (browser, port) = (&Browser::of(&node), node.api_addr().port());
    let home = format!("svrngs.localhost:{port}");
    let own = "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\n";
    let root = env!("CARGO_MANIFEST_DIR");
    let mut served = 0;
    for (at, kept) in [
        ("page", "src/home/page"),
        ("ui", "src/home/ui"),
        ("vendor", "../../web/vendor"),
    ] {
        for f in std::fs::read_dir(format!("{root}/{kept}")).unwrap() {
            let name = f.unwrap().file_name().into_string().unwrap();
            if !name.ends_with(".js") {
                continue;
            }
            let got = ask(browser, "GET", &home, &format!("/{at}/{name}?v=x"), own, "").await;
            let file = std::fs::read_to_string(format!("{root}/{kept}/{name}")).unwrap();
            assert_eq!(
                (got.status, got.header("content-type"), &got.body),
                (200, Some("text/javascript"), &file),
                "/{at}/{name}"
            );
            assert_eq!(
                got.header("content-security-policy"),
                Some("default-src 'self'; frame-ancestors 'none'"),
                "/{at}/{name}"
            );
            served += 1;
        }
    }
    assert!(served >= 30, "the page's modules, each read: {served}");
    for path in [
        "/page/../home.js",
        "/ui/../question.js",
        "/page/Home.js",
        "/page/home.mjs",
        "/ui/gone.js",
    ] {
        let got = ask(browser, "GET", &home, path, own, "").await;
        assert_eq!(got.status, 404, "{path}: {}", got.body);
    }
    node.net().close().await;
}

/// Every change to a lend's declaration holds its lock for the whole read,
/// change and write (LENDING_SCOPE §"Updates, migration and mixed
/// versions"): while another change holds it, a sign-in, a limit and an
/// audience edit are each answered 409, busy, and the declaration is left
/// as it was, never written from a stale read. Once it is let go each is
/// taken, and the audience edit keeps the sign-in client. Planted:
/// sign-in's read and write outside the lock, which wrote while it was held.
#[cfg(unix)]
#[tokio::test]
async fn a_change_while_another_holds_the_lend_is_busy_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    let lends = lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = serde_json::json!({ "port": 9, "kind": "books", "groups": [NS] });
    std::fs::write(
        lends.join("shelf.json"),
        svrngs_node::node::approving(&body),
    )
    .unwrap();
    let browser = &Browser::of(&alex);
    let home = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let to = |change: &str| format!(r#"{{"lend":"shelf","to":{change}}}"#);
    std::fs::create_dir_all(dir.path().join("locks")).unwrap();
    let held = std::fs::File::create(dir.path().join("locks/shelf.lock")).unwrap();
    held.try_lock().unwrap();
    let before = std::fs::read(lends.join("shelf.json")).unwrap();
    for change in [r#""sign-in""#, r#"{"limit":2}"#, r#"{"groups":[]}"#] {
        let said = ask(browser, "POST", &home, "/lent", &own, &to(change)).await;
        assert_eq!(said.status, 409, "{change}: {}", said.body);
    }
    let after = std::fs::read(lends.join("shelf.json")).unwrap();
    assert_eq!(
        after, before,
        "nothing is written while another change holds it"
    );
    drop(held);
    let signed = ask(browser, "POST", &home, "/lent", &own, &to(r#""sign-in""#)).await;
    assert_eq!(signed.status, 200, "{}", signed.body);
    let paused = ask(
        browser,
        "POST",
        &home,
        "/lent",
        &own,
        &to(r#"{"groups":[]}"#),
    )
    .await;
    assert_eq!(paused.status, 204, "{}", paused.body);
    let name = lend::LendName::parse("shelf").unwrap();
    let kept = lend::read(dir.path(), &name).unwrap().unwrap();
    assert_eq!(kept.terms.audience, lend::Audience::Selected(Vec::new()));
    assert!(
        kept.terms.sign_in_with_svrngs.is_some(),
        "the audience edit kept the client"
    );
    alex.net().close().await;
}

/// A selection names only groups the lender stands in now
/// (SCOPE_PARALLEL.md: validate explicit selections and lender standing):
/// alex's computer holds the house, where alex stands, and bos, which
/// names only sam; an audience of bos is refused 400 and the declaration
/// keeps the house; one of the house is taken. Planted: `chosen` asking
/// only whether the group is held.
#[tokio::test]
async fn an_audience_names_only_groups_the_lender_stands_in() {
    let dir = tempfile::tempdir().unwrap();
    let alex = on_wire::start(1, dir.path()).await;
    let roster = |p: &str, k: u8| {
        Roster::new(
            [(Person::parse(p).unwrap(), vec![actor_of(&signing(k))])]
                .into_iter()
                .collect(),
        )
    };
    alex.rail().set_seed(NS, &roster("alex", 1)).unwrap();
    alex.rail().set_seed("bos", &roster("sam", 2)).unwrap();
    let lends = lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = serde_json::json!({ "port": 9, "kind": "books", "groups": [NS] });
    std::fs::write(
        lends.join("shelf.json"),
        svrngs_node::node::approving(&body),
    )
    .unwrap();
    let browser = &Browser::of(&alex);
    let home = format!("svrngs.localhost:{}", alex.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let to = |g: &str| format!(r#"{{"lend":"shelf","to":{{"groups":["{g}"]}}}}"#);
    let bos = ask(browser, "POST", &home, "/lent", &own, &to("bos")).await;
    assert_eq!(bos.status, 400, "{}", bos.body);
    let name = lend::LendName::parse("shelf").unwrap();
    let kept = lend::read(dir.path(), &name)
        .unwrap()
        .unwrap()
        .terms
        .audience;
    assert_eq!(kept, lend::Audience::Selected(vec![NS.into()]));
    let house = ask(browser, "POST", &home, "/lent", &own, &to(NS)).await;
    assert_eq!(house.status, 204, "{}", house.body);
    alex.net().close().await;
}
