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
    let relay = RelayConfig::from_parts(Vec::new(), Some("none"))
        .unwrap()
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
