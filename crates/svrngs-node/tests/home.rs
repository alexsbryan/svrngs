// SPDX-License-Identifier: AGPL-3.0-or-later
//! The home page (sup-2; DESIGN §3, "The home page"):
//! `http://svrngs.localhost:<api port>/`, on the record API's listener.
//! Alex founds a group and lets Sam in. Alex lends `night`, a page reading
//! kind `taste`, and a `taste` that asks; Sam lends a `taste` that asks.
//! Two nodes in one process, no relay, each on 127.0.0.1 alone, Sam
//! reached by seeded addresses, as `tests/lend_origin.rs`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[allow(dead_code)]
#[path = "door/origins.rs"]
mod origins;

use std::net::SocketAddr;
use std::time::Duration;

use common::loopback_sockets;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use svrngs_core::pass::Label;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};

const NS: &str = "house";
const WORDS: &str = "the app receives ranked suggestions; your history stays where it is";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn no_seed(_: &std::path::Path) -> Result<Option<String>, String> {
    Ok(None)
}

async fn start(seed: u8, dir: &std::path::Path) -> Node {
    let relay = RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
    .on_loopback_only();
    Node::start(
        dir.to_path_buf(),
        signing(seed),
        &relay,
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

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
    let body =
        serde_json::json!({ "port": port, "kind": kind, "limit": 3, "reads": reads, "asks": asks });
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
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
    let body = match head
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked")
    {
        true => unchunk(body),
        false => body.to_string(),
    };
    Got {
        status: head[9..12].parse().unwrap(),
        head: head.to_string(),
        body,
    }
}

fn unchunk(mut rest: &str) -> String {
    let mut out = String::new();
    while let Some((size, tail)) = rest.split_once("\r\n") {
        let n = usize::from_str_radix(size.trim(), 16).unwrap();
        if n == 0 {
            break;
        }
        out.push_str(&tail[..n]);
        rest = &tail[n + 2..];
    }
    out
}

/// **The home page answers only itself, shows what `/v1/lends` answers, and
/// takes only the person's yes and stop.** Typed, it serves its page and
/// module, never to be framed; a lend's page navigated to `/ask` gets the
/// page too. Its `/v1/groups` and `/v1/lends` answer what the record API's
/// answer an app, the same body. A lend's page reading its list is refused
/// `closed`, and so is a post with no exact `Origin` or not from its own
/// origin: a lend's page's yes. The person's yes, from its own origin, lets
/// `night` into Alex's own `taste` (kept here) and Sam's (posted to Sam's
/// door), which refused it `unasked` before, and `GET /choices` asks Sam's
/// door what apps use. `/svrngs-reads` at `night` gives each asked lend's
/// words and its `ask` address. `night`'s page stops itself at its origin,
/// at both, and is refused `stopped` after. Planted, each watched failing:
/// a lend page's yes accepted; a lend page reading the home page's list;
/// the home page framed.
#[tokio::test]
async fn the_home_page_answers_only_itself_and_takes_only_the_person_s_choices() {
    let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let alex = start(1, alex_dir.path()).await;
    let sam = start(2, sam_dir.path()).await;
    let seed = Roster::new(
        [(Person::parse("alex").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    for node in [&alex, &sam] {
        node.rail().set_seed(NS, &seed).unwrap();
    }
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("sam").unwrap(),
            key: actor_of(&signing(2)),
            invite: None,
        },
        on_behalf_of: None,
    };
    alex.rail().commit(NS, &admit).unwrap().unwrap();
    alex.transport()
        .add_known_peer(sam.endpoint().id(), loopback_sockets(sam.endpoint()));
    let mut heads = std::collections::BTreeMap::new();
    for (who, dir, name, kind, reads, asks) in [
        (1, alex_dir.path(), "night", "night", &["taste"][..], None),
        (1, alex_dir.path(), "taste", "taste", &[], Some(WORDS)),
        (2, sam_dir.path(), "taste", "taste", &[], Some(WORDS)),
    ] {
        let (at, kept) = origins::head_origin().await;
        declare(dir, name, at.port(), kind, reads, asks);
        heads.insert((who, name), kept);
    }
    let api = alex.api_addr();
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

    let page = ask(api, "GET", &home, "/", typed, "").await;
    assert_eq!(page.status, 200, "{}", page.body);
    assert!(page.body.contains(r#"src="/home.js""#), "{}", page.body);
    for got in [
        &page,
        &ask(api, "GET", &home, "/v1/groups", &from_night, "").await,
    ] {
        assert_eq!(
            got.header("content-security-policy"),
            Some("default-src 'self'; frame-ancestors 'none'"),
            "never framed, answered or refused: {}",
            got.head
        );
    }
    let module = ask(api, "GET", &home, "/home.js", &own, "").await;
    assert_eq!(
        (module.status, module.header("content-type")),
        (200, Some("text/javascript"))
    );
    let navigated = format!(
        "Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: navigate\r\nOrigin: http://{}\r\n",
        at(&night)
    );
    let asking = ask(
        api,
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
    let elsewhere = ask(api, "GET", "svrngs.localhost:1", "/", typed, "").await;
    assert_eq!(
        elsewhere.status, 403,
        "only at its own port: {}",
        elsewhere.body
    );

    for path in [
        "/v1/groups".to_string(),
        format!("/v1/lends?namespace={NS}"),
    ] {
        let shown = ask(api, "GET", &home, &path, &own, "").await;
        let answered = ask(api, "GET", "127.0.0.1", &path, "", "").await;
        assert_eq!(
            (shown.status, shown.json()),
            (answered.status, answered.json()),
            "{path}: one decider"
        );
        let read = ask(api, "GET", &home, &path, &from_night, "").await;
        assert_eq!(
            (read.status, read.header("x-svrngs-refused")),
            (403, Some("closed")),
            "a lend's page reading {path}: {}",
            read.body
        );
    }
    let shown = ask(
        api,
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
        async move { ask(api, "GET", &host, "/x", &extra, "").await }
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
        ("a program", String::new()),
    ] {
        let posted = ask(api, "POST", &home, "/choices", &extra, &yes(&alexs)).await;
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
        let said = ask(api, "POST", &home, "/choices", &own, &yes(l)).await;
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
        api,
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

    let reads = ask(api, "GET", &at(&night), "/svrngs-reads", "", "")
        .await
        .json();
    for l in [&alexs, &sams] {
        let one = reads["lends"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["address"] == format!("http://{}/", at(l)));
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
            api,
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
    sam.endpoint().close().await;
    alex.endpoint().close().await;
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
    let alex = start(1, alex_dir.path()).await;
    let sam = start(2, sam_dir.path()).await;
    let seed = Roster::new(
        [(Person::parse("alex").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    for node in [&alex, &sam] {
        node.rail().set_seed(NS, &seed).unwrap();
    }
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("sam").unwrap(),
            key: actor_of(&signing(2)),
            invite: None,
        },
        on_behalf_of: None,
    };
    alex.rail().commit(NS, &admit).unwrap().unwrap();
    alex.transport()
        .add_known_peer(sam.endpoint().id(), loopback_sockets(sam.endpoint()));
    sam.transport()
        .add_known_peer(alex.endpoint().id(), loopback_sockets(alex.endpoint()));
    tokio::time::timeout(Duration::from_secs(30), sam.round())
        .await
        .expect("Sam's computer hears it lets Sam in");
    let (shelf_at, _kept) = origins::head_origin().await;
    let at = lend::lends_dir(sam_dir.path());
    std::fs::create_dir_all(&at).unwrap();
    let body = serde_json::json!({ "port": shelf_at.port(), "kind": "books", "limit": 1 });
    std::fs::write(at.join("shelf.json"), body.to_string()).unwrap();
    let shelf = Label::of(&signing(2).verifying_key().to_bytes(), "shelf");
    let name = lend::LendName::parse("shelf").unwrap();
    let limit = || {
        lend::read(sam_dir.path(), &name)
            .unwrap()
            .map(|l| l.limit.map(|n| n.get()))
    };
    let typed = "Sec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\n";
    let open = |node: &Node| {
        let api = node.api_addr();
        let host = format!("{shelf}.localhost:{}", api.port());
        async move { ask(api, "GET", &host, "/x", typed, "").await }
    };
    let (api, home) = (
        sam.api_addr(),
        format!("svrngs.localhost:{}", sam.api_addr().port()),
    );
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");

    let sams = open(&sam).await;
    assert_eq!(sams.status, 200, "Sam, the first: {}", sams.body);
    let alexs = open(&alex).await;
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
            format!("Sec-Fetch-Site: same-site\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{shelf}.localhost:{}\r\n", api.port()),
        ),
        (
            "the home page's Origin, from another site",
            format!("Sec-Fetch-Site: same-site\r\nOrigin: http://{home}\r\n"),
        ),
        ("its own origin with no Origin", "Sec-Fetch-Site: same-origin\r\n".into()),
        ("a program", String::new()),
    ] {
        let posted = ask(api, "POST", &home, "/lent", &extra, everyone).await;
        assert_eq!(
            (posted.status, posted.header("x-svrngs-refused")),
            (403, Some("closed")),
            "{row}'s change: {}",
            posted.body
        );
    }
    assert_eq!(limit(), Some(Some(1)), "no refused change was kept");

    let changed = ask(api, "POST", &home, "/lent", &own, everyone).await;
    assert_eq!(changed.status, 204, "Sam's own change: {}", changed.body);
    let listed = ask(api, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(listed["lent"][0]["name"], "shelf", "{listed:#}");
    assert_eq!(
        listed["lent"][0]["limit"],
        serde_json::Value::Null,
        "{listed:#}"
    );
    let alexs = open(&alex).await;
    assert_eq!(alexs.status, 200, "Alex, with no restart: {}", alexs.body);

    let two = r#"{"lend":"shelf","to":{"limit":2}}"#;
    assert_eq!(
        ask(api, "POST", &home, "/lent", &own, two).await.status,
        204
    );
    assert_eq!(limit(), Some(Some(2)));
    let none = ask(
        api,
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
        ask(api, "POST", &home, "/lent", &own, stop).await.status,
        204
    );
    assert_eq!(limit(), None, "no longer lent");
    let listed = ask(api, "GET", &home, "/lent", &own, "").await.json();
    assert_eq!(listed["lent"], serde_json::json!([]), "{listed:#}");
    sam.endpoint().close().await;
    alex.endpoint().close().await;
}

/// **Groups that cannot be read are said so** (ralph/REVIEW_FINDINGS.md
/// #103): with Alex's groups unlistable, a lent page's `/svrngs-reads` is
/// svrngs's `fault`, never "nothing is lent here". Watched failing first,
/// at 2e6edc7's `shared_with`, which read it as no group: 404 `unlent`.
#[tokio::test]
async fn groups_that_cannot_be_read_are_said_so() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let alex = start(1, dir.path()).await;
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
        alex.api_addr(),
        "GET",
        &format!("{night}.localhost:{}", alex.api_addr().port()),
        "/svrngs-reads",
        "",
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
    alex.endpoint().close().await;
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
    let alex = start(1, alex_dir.path()).await;
    let sam = start(2, sam_dir.path()).await;
    let seed = Roster::new(
        [(Person::parse("alex").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    for node in [&alex, &sam] {
        node.rail().set_seed(NS, &seed).unwrap();
    }
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("sam").unwrap(),
            key: actor_of(&signing(2)),
            invite: None,
        },
        on_behalf_of: None,
    };
    alex.rail().commit(NS, &admit).unwrap().unwrap();
    alex.transport()
        .add_known_peer(sam.endpoint().id(), loopback_sockets(sam.endpoint()));
    // A port nothing listens on: bound, read and let go.
    let silent = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    for (dir, name, kind, reads, asks) in [
        (alex_dir.path(), "night", "night", &["media"][..], None),
        (alex_dir.path(), "telly", "media", &[], Some(WORDS)),
        (alex_dir.path(), "diary", "notes", &[], None),
    ] {
        let (at, _) = origins::head_origin().await;
        declare(dir, name, at.port(), kind, reads, asks);
    }
    for (dir, name, port, refused) in [
        (alex_dir.path(), "shows", None, &["/x"][..]),
        (sam_dir.path(), "films", None, &["/x"]),
        (sam_dir.path(), "reels", Some(silent), &[]),
    ] {
        let port = match port {
            Some(p) => p,
            None => origins::head_origin().await.0.port(),
        };
        let body =
            serde_json::json!({ "port": port, "kind": "media", "limit": 3, "refused": refused });
        std::fs::create_dir_all(lend::lends_dir(dir)).unwrap();
        std::fs::write(
            lend::lends_dir(dir).join(format!("{name}.json")),
            body.to_string(),
        )
        .unwrap();
    }
    let api = alex.api_addr();
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
        async move { ask(api, "GET", &host, path, &extra, "").await }
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
    let stopped = ask(api, "POST", &at(&night), &stop, same, "").await;
    assert_eq!(stopped.status, 200, "night stops itself: {}", stopped.body);
    let got = night_at(telly.clone(), "/x").await;
    assert_eq!(read(&got), readable(403, "stopped"), "{}", got.body);

    let from_home = ask(
        api,
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
    let at_home = ask(api, "GET", &home, "/v1/lends", &from(&page), "").await;
    assert_eq!(
        read(&at_home),
        unread(403, Some("closed")),
        "night's fetch at the home page"
    );
    sam.endpoint().close().await;
    alex.endpoint().close().await;
}
