// SPDX-License-Identifier: AGPL-3.0-or-later
//! Lend a service, u3-5's bar (PLAN §4, U3): Sam shares a service on
//! Sam's computer; Alex, in Sam's group, runs the bare command and reads
//! "Sam's <name>" with an address on Alex's own computer, and a request
//! there reaches Sam's service as Alex, with Sam's password added and none
//! of it sent back. Sam stops sharing, and the next ask lists nothing. The
//! command itself, as a person runs it: `share` with svrngs not running
//! (it runs) and running (it declares and leaves), the password read from
//! a prompt — never the command line — and found in no log line and no file
//! but its own 0600 store. Nodes in one process, no relay, for the first;
//! the program, for the second.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use svrngs::cli::{self, Refusal, Said, Verb};
use svrngs::group::{self, Name};
use svrngs::share::{self, Share};
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, RelayConfig};
use svrngs_node::lend::{LendName, Program as LentProgram};
use svrngs_node::link::{Link, KEPT_WORDS};
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

/// Sam's password for Sam's own service: Jellyfin 12's form.
const SECRET: &str = r#"MediaBrowser Token="sams-own-1234""#;

fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
}

/// What `svrngs share <port> --kind <kind>` asks of the service at
/// `origin`, nothing else said.
fn share_of(origin: SocketAddr, kind: &str) -> Share {
    Share {
        port: std::num::NonZeroU16::new(origin.port()).unwrap(),
        name: None,
        kind: Some(LentProgram::parse(kind).unwrap()),
        limit: None,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        asks: None,
        sign_in: false,
        readers: false,
        raw: None,
        audience: svrngs_node::lend::Audience::Legacy,
    }
}

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

struct Computer {
    node: Node,
    key: SigningKey,
    dir: tempfile::TempDir,
}

async fn computer(seed: u8) -> Computer {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[seed; 32]);
    let node = Node::start(
        dir.path().to_path_buf(),
        key.clone(),
        &no_relay(),
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    Computer { node, key, dir }
}

/// Tell `who` the loopback addresses Sam's key resolves to, as address
/// lookup or a relay would outside one process.
fn knows(who: &Computer, sam: &Node) {
    let mut addr = EndpointAddr::new(sam.endpoint().id());
    for mut a in sam.endpoint().bound_sockets() {
        if a.ip().is_unspecified() {
            a.set_ip(if a.is_ipv4() {
                "127.0.0.1".parse().unwrap()
            } else {
                "::1".parse().unwrap()
            });
        }
        addr = addr.with_ip_addr(a);
    }
    let book = MemoryLookup::new();
    book.add_endpoint_info(addr);
    who.node.endpoint().address_lookup().unwrap().add(book);
}

/// A service that keeps every request head it is sent and answers 200
/// `ok`, echoing any `authorization` it received, as a careless one might.
async fn service() -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let heads = Arc::new(Mutex::new(Vec::new()));
    let kept = heads.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let kept = kept.clone();
            tokio::spawn(async move {
                let mut head = Vec::new();
                while !head.ends_with(b"\r\n\r\n") {
                    let mut b = [0u8; 1];
                    if sock.read_exact(&mut b).await.is_err() {
                        return;
                    }
                    head.push(b[0]);
                }
                let head = String::from_utf8_lossy(&head).into_owned();
                let echo = header(&head, "authorization").unwrap_or_default();
                kept.lock().unwrap().push(head);
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nX-Seen: {echo}\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
                );
                let _ = sock.write_all(reply.as_bytes()).await;
            });
        }
    });
    (addr, heads)
}

fn header(head: &str, name: &str) -> Option<String> {
    head.lines()
        .filter_map(|l| l.split_once(':'))
        .find(|(n, _)| n.trim().eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim().to_string())
}

/// The http address a printed line names, if any.
fn address_in(line: &str) -> Option<String> {
    line.split_whitespace()
        .find(|w| w.starts_with("http://"))
        .map(str::to_string)
}

/// Nothing a person reads says a word DESIGN §1 keeps from them.
fn plain(said: &[String]) {
    for line in said {
        let lower = line.to_lowercase();
        for word in KEPT_WORDS {
            assert!(!lower.contains(word), "{line:?} says {word}");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn sam_shares_and_alex_reaches_it_as_alex_until_sam_stops() {
    let sam = computer(1).await;
    let (seed, printed) = cli::new_group(
        &sam.node,
        name("house"),
        name("Sam"),
        None,
        &[],
        &no_relay(),
    )
    .await
    .expect("Sam makes house");
    let alex = computer(2).await;
    knows(&alex, &sam.node);
    let link = Link::parse(&printed.client).unwrap();
    cli::join_group(&alex.node, &alex.key, &link, &name("Alex"))
        .await
        .expect("Alex is let in");

    // ── Sam shares the service on its port, as `svrngs share <port> --kind pad`.
    let (origin, heads) = service().await;
    let lent_to = svrngs_node::lend::Audience::Selected(vec![seed.id()]);
    let asked = Share {
        audience: lent_to,
        ..share_of(origin, "pad")
    };
    let lend = share::lend_of(
        &asked,
        share::listens,
        share::well_known_kind(asked.port),
        "",
    )
    .expect("something listens there, and its kind is given");
    cli::share_lend(sam.dir.path(), &lend, Some(SECRET)).unwrap();
    let mut said = vec![Said::Shared {
        lend: lend.clone(),
        secret: true,
        accounts: None,
        client: None,
        issuer: None,
        terms: Some(Default::default()),
    }
    .to_string()];

    // ── Alex's bare command: "Sam's pad", at an address on Alex's computer.
    let running = cli::running(alex.dir.path(), &alex.node).await.to_string();
    said.push(running.clone());
    let line = running
        .lines()
        .find(|l| l.starts_with("Sam's pad"))
        .unwrap_or_else(|| panic!("Alex is not told of Sam's pad: {running:?}"));
    let address = address_in(line).expect("an address to open");
    // The lend's origin on Alex's computer (u5-2): `<label>.localhost`, on
    // the port Alex's apps reach the groups at.
    let host = address
        .strip_prefix("http://")
        .and_then(|a| a.strip_suffix(&format!(":{}/", alex.node.api_addr().port())))
        .unwrap_or_else(|| panic!("an origin on Alex's computer: {address}"));
    assert!(host.ends_with(".localhost"), "{address}");

    let http = reqwest::Client::builder()
        .no_proxy()
        .resolve(host, alex.node.api_addr())
        .build()
        .unwrap();
    let answer = http
        .get(format!("{address}notes?x=1"))
        // A program Alex runs, which shows his token (give-session).
        .bearer_auth(alex.node.token().trim())
        .header("x-svrngs-person", "Sam")
        .send()
        .await
        .unwrap();
    assert_eq!(answer.status(), 200);
    for (n, v) in answer.headers() {
        assert!(
            !v.as_bytes()
                .windows(SECRET.len())
                .any(|w| w == SECRET.as_bytes()),
            "Alex received Sam's password in {n}"
        );
    }
    assert_eq!(answer.text().await.unwrap(), "ok");
    let head = heads
        .lock()
        .unwrap()
        .last()
        .cloned()
        .expect("Sam's service was asked");
    assert!(head.starts_with("GET /notes?x=1 HTTP/1.1"), "{head}");
    assert_eq!(
        header(&head, "x-svrngs-person").as_deref(),
        Some("Alex"),
        "{head}"
    );
    assert_eq!(
        header(&head, "authorization").as_deref(),
        Some(SECRET),
        "{head}"
    );

    // ── Sam stops; Alex's next ask lists nothing of Sam's.
    let stopped = cli::unshare(sam.dir.path(), &lend.name, &[]).expect("Sam lent pad");
    assert_eq!(stopped, Said::Unshared(lend.name.clone()));
    said.push(stopped.to_string());
    let again = cli::running(alex.dir.path(), &alex.node).await.to_string();
    assert!(
        !again.contains("Sam's"),
        "a stopped share is still listed: {again:?}"
    );
    said.push(again);
    let refused = cli::unshare(sam.dir.path(), &lend.name, &[]).unwrap_err();
    assert_eq!(refused.exit_code(), 1);
    said.push(refused.to_string());
    for r in [
        Refusal::Share(share::ShareRefusal::NothingListens(asked.port)),
        Refusal::Share(share::ShareRefusal::NoKind(asked.port)),
    ] {
        said.push(r.to_string());
    }
    plain(&said);
    let _ = seed;
    sam.node.endpoint().close().await;
    alex.node.endpoint().close().await;
}

/// **A friend's llama-server, lent by its kind alone** (start-llama (a)):
/// Sam types `--kind llama-server` and no path, and Alex's request to each
/// route start-measure (b) found shows one person's chats to the others or
/// changes the server for everyone (`demo/runs/start/2026-10-13/README.md`
/// §3 (b) 6 and §4) is refused in svrngs's words, never reaching it; every
/// other route a chat page or a program asks reaches it. The routes are
/// written here from the run, not read from the table, so a table that
/// drops one fails. Red at start-llama's base: nothing kept back, `GET
/// /slots` answered 200.
#[tokio::test(flavor = "multi_thread")]
async fn llama_server_lent_by_its_kind_keeps_back_what_shows_one_person_s_chats() {
    let sam = computer(3).await;
    let (lab, printed) =
        cli::new_group(&sam.node, name("Lab"), name("Sam"), None, &[], &no_relay())
            .await
            .expect("Sam makes Lab");
    let alex = computer(4).await;
    knows(&alex, &sam.node);
    let link = Link::parse(&printed.client).unwrap();
    cli::join_group(&alex.node, &alex.key, &link, &name("Alex"))
        .await
        .expect("Alex is let in");

    let (origin, heads) = service().await;
    let lent_to = svrngs_node::lend::Audience::Selected(vec![lab.id()]);
    let asked = Share {
        audience: lent_to,
        ..share_of(origin, "llama-server")
    };
    let lend = share::lend_of(
        &asked,
        share::listens,
        share::well_known_kind(asked.port),
        "",
    )
    .expect("something listens there, and its kind is given");
    cli::share_lend(sam.dir.path(), &lend, None).unwrap();
    let shared = Said::Shared {
        lend: lend.clone(),
        secret: false,
        accounts: None,
        client: None,
        issuer: None,
        terms: Some(Default::default()),
    }
    .to_string();

    let running = cli::running(alex.dir.path(), &alex.node).await.to_string();
    let address = running
        .lines()
        .find(|l| l.starts_with("Sam's llama-server"))
        .and_then(address_in)
        .unwrap_or_else(|| panic!("Alex is not told of Sam's llama-server: {running:?}"));
    let host = address
        .strip_prefix("http://")
        .and_then(|a| a.split_once(':'))
        .map(|(h, _)| h.to_string())
        .unwrap();
    let http = reqwest::Client::builder()
        .no_proxy()
        .resolve(&host, alex.node.api_addr())
        .build()
        .unwrap();
    let ask = |method: &str, path: &str| {
        let method = reqwest::Method::from_bytes(method.as_bytes()).unwrap();
        http.request(method, format!("{}{}", address.trim_end_matches('/'), path))
            .bearer_auth(alex.node.token().trim())
            .send()
    };

    let kept = [
        ("GET", "/slots", "/slots"),
        ("POST", "/slots/0?action=save", "/slots"),
        ("GET", "/v1/stream?conv_id=cy-conv-1", "/v1/stream"),
        ("DELETE", "/v1/stream?conv_id=cy-conv-2", "/v1/stream"),
        ("POST", "/v1/streams/lookup", "/v1/streams"),
        (
            "POST",
            "/v1/chat/completions/control",
            "/v1/chat/completions/control",
        ),
        ("GET", "/lora-adapters", "/lora-adapters"),
        ("POST", "/lora-adapters", "/lora-adapters"),
        ("GET", "/metrics", "/metrics"),
        ("GET", "/tools", "/tools"),
        ("POST", "/tools", "/tools"),
        ("GET", "/cors-proxy?url=http://example.com/", "/cors-proxy"),
        ("POST", "/models/load", "/models"),
        ("POST", "/models/unload", "/models"),
        ("GET", "/models/sse", "/models"),
    ];
    for (method, path, under) in kept {
        let answer = ask(method, path).await.unwrap();
        let (status, why) = (
            answer.status(),
            answer.headers().get("x-svrngs-refused").cloned(),
        );
        let body = answer.text().await.unwrap();
        assert_eq!(
            (status.as_u16(), why.as_ref().and_then(|v| v.to_str().ok())),
            (403, Some("path")),
            "{method} {path} was not kept back: {body}"
        );
        assert!(
            body.contains(&format!("llama-server is lent without {under}")),
            "{method} {path}: {body}"
        );
    }
    assert!(
        heads.lock().unwrap().is_empty(),
        "a route kept back reached llama-server"
    );

    let answered = [
        ("GET", "/"),
        ("GET", "/props"),
        ("GET", "/health"),
        ("GET", "/v1/models"),
        ("POST", "/v1/chat/completions"),
        ("POST", "/chat/completions"),
        ("POST", "/completion"),
        ("POST", "/v1/completions"),
        ("POST", "/v1/chat/completions/input_tokens"),
        ("POST", "/v1/responses"),
        ("POST", "/v1/embeddings"),
        ("POST", "/tokenize"),
        ("POST", "/apply-template"),
    ];
    for (method, path) in answered {
        let answer = ask(method, path).await.unwrap();
        assert_eq!(answer.status(), 200, "{method} {path} was refused");
        assert_eq!(answer.text().await.unwrap(), "ok", "{method} {path}");
    }
    assert_eq!(heads.lock().unwrap().len(), answered.len());
    assert!(
        shared.contains("They cannot open /slots, "),
        "Sam is not told what is kept back: {shared}"
    );
    plain(&[shared, running]);
    sam.node.endpoint().close().await;
    alex.node.endpoint().close().await;
}

/// The password has no flag: `share` refuses one, whatever it is called.
#[test]
fn share_takes_no_password_on_the_command_line() {
    let argv = |rest: &[&str]| {
        std::iter::once("svrngs".to_string())
            .chain(rest.iter().map(|s| s.to_string()))
            .collect::<Vec<_>>()
    };
    for flag in ["--secret", "--password", "--token", "--authorization"] {
        assert!(
            cli::parse(argv(&["share", "8096", flag, "x"])).is_err(),
            "{flag} is taken"
        );
    }
    let Ok(Verb::Share(asked, _)) = cli::parse(argv(&[
        "share", "8096", "--name", "box", "--limit", "2", "--refuse", "/admin",
    ])) else {
        panic!("share parses");
    };
    assert_eq!(asked.name, Some(LendName::parse("box").unwrap()));
    assert_eq!(asked.limit.map(|l| l.get()), Some(2));
    assert_eq!(asked.refused.len(), 1);
    assert_eq!(asked.upload, None, "no upload stated, none declared");
    assert!(cli::parse(argv(&["new", "x", "--limit", "2"])).is_err());
    let Ok(Verb::Share(asked, _)) = cli::parse(argv(&["share", "8096", "--upload", "30"])) else {
        panic!("share parses an upload");
    };
    assert_eq!(asked.upload.map(|u| u.bps()), Some(30_000_000));
    for bad in ["0", "-3", "fast", "2.5"] {
        assert!(
            cli::parse(argv(&["share", "8096", "--upload", bad])).is_err(),
            "--upload {bad} is not an upload"
        );
    }
    assert!(cli::parse(argv(&["link", "x", "--upload", "30"])).is_err());
    // The kinds a lend reads (u5-2), each said again for more.
    let Ok(Verb::Share(asked, _)) = cli::parse(argv(&[
        "share", "8096", "--reads", "films", "--reads", "music",
    ])) else {
        panic!("share parses what it reads");
    };
    let reads: Vec<&str> = asked.reads.iter().map(|k| k.as_str()).collect();
    assert_eq!(reads, ["films", "music"]);
    assert!(cli::parse(argv(&["share", "8096", "--reads", "a/b"])).is_err());
    assert!(cli::parse(argv(&["new", "x", "--reads", "films"])).is_err());
    // The hook's `--sign-in` is gone; `share_refuses_a_sign_in_the_lender_ran_itself` holds it.
    assert!(cli::parse(argv(&["room", "x", "--sign-in", "9000"])).is_err());
}

/// **A sign-in of the lender's own is refused** (oidc-hook-cut; the
/// operator, 2026-10-21): `share --sign-in <port>` named the hook a lend
/// declared, which sign-in with svrngs makes userless. It is refused in a
/// person's words, and the flag still reads its value, so `--sign-in` alone
/// says it wants one. Red at HEAD, where it lent the port. Planted: the arm
/// removed, so the flag is unknown rather than gone.
#[test]
fn share_refuses_a_sign_in_the_lender_ran_itself() {
    let argv = |rest: &[&str]| {
        std::iter::once("svrngs".to_string())
            .chain(rest.iter().map(|s| s.to_string()))
            .collect::<Vec<_>>()
    };
    let gone = "--sign-in is gone: svrngs no longer runs a program of yours to sign people in; sign-in with svrngs is the door's own";
    assert_eq!(
        cli::parse(argv(&["share", "8096", "--sign-in", "9000"])).err(),
        Some(gone.to_string())
    );
    assert_eq!(
        cli::parse(argv(&["share", "8096", "--sign-in"])).err(),
        Some("--sign-in wants a value".to_string()),
        "the retired flag still reads its value"
    );
    assert_eq!(
        cli::parse(argv(&["share", "8096", "--sign-in", "0"])).err(),
        Some(gone.to_string()),
        "the value is never read as a port"
    );
}

/// **A lend that asks** (sup-2 (a)): `share --ask "<words>"` keeps its
/// lender's words in the declaration, read back whole, and says them;
/// words that are none, or a control character, are refused, and `--ask`
/// is `share`'s alone. `unshare` takes it back. Planted, watched failing:
/// the declaration not writing `asks`.
#[test]
fn a_lend_that_asks_keeps_its_lender_s_words() {
    let argv = |rest: &[&str]| {
        std::iter::once("svrngs".to_string())
            .chain(rest.iter().map(|s| s.to_string()))
            .collect::<Vec<_>>()
    };
    let words = "the app receives ranked suggestions; your history stays where it is";
    let Ok(Verb::Share(asked, _)) = cli::parse(argv(&["share", "8096", "--ask", words])) else {
        panic!("share parses what it asks");
    };
    assert_eq!(asked.asks.as_deref(), Some(words));
    for bad in ["", "  ", "a\nb"] {
        assert!(
            cli::parse(argv(&["share", "8096", "--ask", bad])).is_err(),
            "{bad:?}"
        );
    }
    assert!(cli::parse(argv(&["link", "x", "--ask", words])).is_err());
    let dir = tempfile::tempdir().unwrap();
    let lend = share::lend_of(
        &asked,
        |_| true,
        Some(LentProgram::parse("taste").unwrap()),
        "",
    )
    .unwrap();
    cli::share_lend(dir.path(), &lend, None).unwrap();
    let name = LendName::parse("taste").unwrap();
    let read = svrngs_node::lend::read(dir.path(), &name).unwrap().unwrap();
    assert_eq!(
        read.asks.as_deref(),
        Some(words),
        "the words come back whole"
    );
    let said = Said::Shared {
        lend: read,
        secret: false,
        accounts: None,
        client: None,
        issuer: None,
        terms: Some(Default::default()),
    };
    assert!(said.to_string().contains(words), "{said}");
    cli::unshare(dir.path(), &name, &[]).unwrap();
    assert!(svrngs_node::lend::read(dir.path(), &name)
        .unwrap()
        .is_none());
}

#[path = "common/program.rs"]
mod program;
use program::Program;
#[path = "common/as_main.rs"]
mod as_main;

/// svrngs itself, as [`Program`] runs it (common/as_main.rs).
#[test]
#[ignore = "run by Program, in a process of its own, as svrngs"]
fn svrngs_here() {
    as_main::main()
}

/// Every file under `dir` but `except`, by path, holding `needle`.
fn holding(dir: &Path, needle: &[u8], except: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path != except {
                let mut bytes = Vec::new();
                if std::fs::File::open(&path)
                    .and_then(|mut f| f.read_to_end(&mut bytes))
                    .is_ok()
                    && bytes.windows(needle.len()).any(|w| w == needle)
                {
                    found.push(path.display().to_string());
                }
            }
        }
    }
    found
}

#[cfg(unix)]
fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

/// **A misspelled program is told at share time** (CAPABILITY_CONNECTIONS
/// §3): a typo matches no `keeps:` row and silently loses every protection
/// the table holds. A name the table has is told nothing; a slip, a case
/// or a letter or two, names the row it likely meant; any other name is
/// told to check its spelling, and is still lent (the lend's `kept_of`
/// empties, never a refusal). Planted: `share::unrecognized` answering
/// `None` for every name.
#[test]
fn a_misspelled_program_is_told_at_share_time() {
    let told = |name: &str| share::unrecognized(&LentProgram::parse(name).unwrap());
    assert_eq!(
        told("jellyfin"),
        None,
        "a name the table knows needs no word"
    );
    for (name, ends) in [
        ("llama-sever", " Did you mean llama-server?"),
        ("Jellyfin", " Did you mean jellyfin?"),
        ("jellyfim", " Did you mean jellyfin?"),
        ("wiki", ""),
    ] {
        let said = format!("svrngs doesn't know {name}, so your friends can reach every part of it that its own sign-in allows.{ends}");
        assert_eq!(told(name), Some(said), "{name}");
    }
}

/// `svrngs share` as a person runs it, twice: with svrngs not running it
/// declares the lend and runs; then, with that one running, a second
/// share declares its lend and leaves. Each reads the password from its
/// prompt, never the command line; it is in no log line, no output, and
/// no file but its own 0600 store.
#[cfg(unix)]
#[test]
fn share_runs_when_svrngs_does_not_and_leaves_when_it_does() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let first = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let second = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let (p1, p2) = (
        first.local_addr().unwrap().port().to_string(),
        second.local_addr().unwrap().port().to_string(),
    );

    holds_lab(dir.path(), logs.path());
    let argv1 = ["share", p1.as_str(), "--kind", "pad", "--group", "Lab"];
    assert!(!argv1.iter().any(|a| a.contains("sams-own")));
    let mut running = Program::start(
        dir.path(),
        &logs.path().join("first"),
        &argv1,
        &format!("{SECRET}\n"),
    );
    running.until("Leave this running");
    assert!(
        running
            .seen
            .iter()
            .any(|l| l.starts_with("You lend pad (port")
                && l.contains("to the groups you chose")
                && l.contains("as many at once as want it")),
        "{:?}",
        running.seen
    );

    let argv2 = [
        "share",
        p2.as_str(),
        "--program",
        "box",
        "--limit",
        "2",
        "--group",
        "Lab",
    ];
    let (code, said) = Program::start(
        dir.path(),
        &logs.path().join("second"),
        &argv2,
        "the-boxs-own\n",
    )
    .ends();
    assert_eq!(code, 0, "{said:?}");
    assert!(
        said.iter()
            .any(|l| l.contains("svrngs is running here already")),
        "{said:?}"
    );
    assert!(
        said.iter()
            .any(|l| l.starts_with("You lend box (port") && l.ends_with("up to 2 people at once.")),
        "{said:?}"
    );
    assert!(
        said.iter()
            .any(|l| l == "svrngs doesn't know box, so your friends can reach every part of it that its own sign-in allows."),
        "a name the table does not know is told at share time: {said:?}"
    );
    assert!(
        !said.iter().any(|l| l.contains("Leave this running")),
        "{said:?}"
    );

    let _ = running.child.kill();
    let (_, seen) = running.ends();

    for (lend, secret) in [("pad", SECRET), ("box", "the-boxs-own")] {
        let store = dir.path().join("secrets/lends").join(lend);
        let file = store.join("authorization");
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            secret,
            "{lend}'s password, as typed at the prompt"
        );
        assert_eq!(mode(&file), 0o600, "{lend}");
        assert_eq!(mode(&store), 0o700, "{lend}");
        assert!(
            dir.path()
                .join("lends")
                .join(format!("{lend}.json"))
                .exists(),
            "{lend} is declared"
        );
        assert_eq!(
            holding(dir.path(), secret.as_bytes(), &file),
            Vec::<String>::new(),
            "{lend}'s password is in a file outside its store"
        );
        assert_eq!(
            holding(logs.path(), secret.as_bytes(), Path::new("")),
            Vec::<String>::new(),
            "{lend}'s password is in a log line"
        );
        for line in seen.iter().chain(&said) {
            assert!(
                !line.contains(secret),
                "{lend}'s password was printed: {line}"
            );
        }
    }
    let log = std::fs::read_to_string(logs.path().join("first")).unwrap();
    assert!(
        log.contains("share: the lend decided"),
        "the log is traced at all, or the scan above proves nothing"
    );
    plain(&seen);
    drop((first, second));
}

/// Sam's computer at `dir` holds Lab, made as `svrngs new Lab --as Sam`
/// makes it: `share` lends only to a group it names (LENDING_SCOPE).
fn holds_lab(dir: &Path, logs: &Path) {
    let mut made = Program::start(dir, &logs.join("new"), &["new", "Lab", "--as", "Sam"], "");
    made.until("Leave this running");
    let _ = made.child.kill();
    let _ = made.ends();
}

/// What svrngs running over `dir` answers `GET /v1/room`, at the API it
/// wrote, as the home page reads it.
fn room_of(dir: &Path) -> serde_json::Value {
    let api = std::fs::read_to_string(dir.join("api")).unwrap();
    let token = std::fs::read_to_string(dir.join("token")).unwrap();
    let mut sock = std::net::TcpStream::connect(api.trim().trim_start_matches("http://")).unwrap();
    let ask = format!("GET /v1/room HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n");
    sock.write_all(ask.as_bytes()).unwrap();
    let mut answer = String::new();
    sock.read_to_string(&mut answer).unwrap();
    let body = answer.split("\r\n\r\n").nth(1).unwrap_or_default();
    serde_json::from_str(body).unwrap_or_else(|_| panic!("the room said {answer:?}"))
}

/// **`svrngs room` asks the svrngs running here, and the room stays on
/// across a restart** (the room's report, 1 and 2: on a computer
/// install.sh set up, its LaunchAgent runs svrngs and holds the data dir,
/// so `room` refused, Busy; and the room lived only while `room` ran).
/// With `svrngs share` running and lending `pad`, `svrngs room pad` turns
/// it on there, says who can open it and where, and leaves, exit 0,
/// running nothing. That svrngs stops, and the bare command starts over
/// the same data, as login starts it: asked nothing, its room serves `pad`
/// where it said, on the same network (`/v1/room`, and its address takes a
/// connection). With nothing running, `svrngs room pad` runs svrngs and
/// says the same. Planted, watched failing: cli.rs as e01c070a left it,
/// which claimed the data dir first and refused, "svrngs is already
/// running on this computer".
#[cfg(unix)]
#[test]
fn room_asks_svrngs_running_here_and_keeps_it_across_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let pad = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = pad.local_addr().unwrap().port().to_string();
    holds_lab(dir.path(), logs.path());
    let argv = ["share", port.as_str(), "--kind", "pad", "--group", "Lab"];
    let mut running = Program::start(dir.path(), &logs.path().join("share"), &argv, "pads\n");
    running.until("Leave this running");
    let told = "Whenever svrngs runs here, anyone on this computer's WiFi can open these as you; nobody off it can. Each stays on the WiFi where you turned it on, until you close it to this WiFi on your home page.";

    let (code, said) =
        Program::start(dir.path(), &logs.path().join("room"), &["room", "pad"], "").ends();
    eprintln!("svrngs room pad, svrngs running: {said:#?}");
    assert_eq!(code, 0, "{said:?}");
    assert!(said.iter().any(|l| l.starts_with(told)), "{said:?}");
    let line = said
        .iter()
        .find(|l| l.starts_with("Sam's pad: "))
        .expect("pad said");
    assert!(
        !said.iter().any(|l| l.contains("Leave this running")),
        "{said:?}"
    );
    let room = room_of(dir.path());
    assert_eq!(room[0]["name"], "pad", "{room}");
    assert_eq!(&room[0]["said"], &line["Sam's pad: ".len()..], "{room}");
    let _ = running.child.kill();
    running.ends();

    let mut again = Program::start(dir.path(), &logs.path().join("again"), &[], "");
    again.until("Leave this running");
    // Its owner reads the network as it starts, the moment after.
    let mut kept = room_of(dir.path());
    for _ in 0..50 {
        if kept[0]["served"] == room[0]["served"] {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
        kept = room_of(dir.path());
    }
    assert_eq!(kept[0]["served"], room[0]["served"], "{kept}");
    if let Some(at) = room[0]["served"]["at"].as_str() {
        let at: std::net::SocketAddr = at.parse().unwrap();
        assert!(
            std::net::TcpStream::connect_timeout(&at, Duration::from_secs(5)).is_ok(),
            "not served again at {at}"
        );
    }
    let _ = again.child.kill();
    again.ends();

    let mut alone = Program::start(dir.path(), &logs.path().join("alone"), &["room", "pad"], "");
    alone.until("Leave this running");
    assert!(
        alone.seen.iter().any(|l| l.starts_with(told)),
        "{:?}",
        alone.seen
    );
    assert!(alone.seen.iter().any(|l| l == line), "{:?}", alone.seen);
    let _ = alone.child.kill();
    alone.ends();
    drop(pad);
}

/// A stand-in sign-in on `listener`: each request's head and body kept,
/// answered `AccessToken` `tok-1` when its body holds `pw-1`, else 401.
fn sign_in_stand_in(listener: std::net::TcpListener) -> Arc<Mutex<Vec<(String, String)>>> {
    use std::io::Write;
    let asked = Arc::new(Mutex::new(Vec::new()));
    let kept = asked.clone();
    std::thread::spawn(move || {
        for sock in listener.incoming().flatten() {
            let mut sock = std::io::BufReader::new(sock);
            let mut head = String::new();
            while !head.ends_with("\r\n\r\n") {
                if std::io::BufRead::read_line(&mut sock, &mut head).unwrap_or(0) == 0 {
                    break;
                }
            }
            let length = header(&head, "content-length").and_then(|l| l.parse().ok());
            let mut body = vec![0; length.unwrap_or(0)];
            let _ = sock.read_exact(&mut body);
            let body = String::from_utf8_lossy(&body).into_owned();
            let (status, answer) = match body.contains("pw-1") {
                true => ("200 OK", r#"{"AccessToken":"tok-1"}"#),
                false => ("401 Unauthorized", ""),
            };
            let n = answer.len();
            let reply = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {n}\r\nConnection: close\r\n\r\n{answer}"
            );
            let _ = sock.get_mut().write_all(reply.as_bytes());
            if !head.is_empty() {
                kept.lock().unwrap().push((head, body));
            }
        }
    });
    asked
}

/// **The readers' account, signed in once** (MOVIE_NIGHT_FOR_TEN, "The
/// spike"): `share --kind jellyfin --readers` asks the account's name and
/// password and signs in as the table's row says, with the Authorization
/// Jellyfin 12.0.0 answered 400 without; it keeps what apps reading it are
/// let in with, owner-only, and the password nowhere. A wrong password is
/// refused by name, and nothing is kept for readers. Planted: the row's
/// Authorization dropped (the stand-in told no Client); the token field
/// misnamed (refused: its answer has no such field).
#[cfg(unix)]
#[test]
fn share_readers_signs_in_once_and_keeps_what_readers_are_let_in_with() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let stand_in = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = stand_in.local_addr().unwrap().port().to_string();
    let asked = sign_in_stand_in(stand_in);
    holds_lab(dir.path(), logs.path());
    let args = [
        "share",
        &port,
        "--kind",
        "jellyfin",
        "--group",
        "Lab",
        "--readers",
    ];
    let readers = dir
        .path()
        .join("secrets/lends/jellyfin/readers/authorization");

    let wrong = Program::start(
        dir.path(),
        &logs.path().join("wrong"),
        &args,
        "group\nnot-it\n",
    );
    let (code, seen) = wrong.ends();
    assert_eq!(code, 1, "{seen:?}");
    let said = std::fs::read_to_string(logs.path().join("wrong")).unwrap();
    let words = "jellyfin's readers were not let in: it did not take that name and password";
    assert!(said.contains(words), "{said}");
    assert!(
        !readers.exists(),
        "a wrong password kept something for readers"
    );

    let log = logs.path().join("right");
    let mut right = Program::start(dir.path(), &log, &args, "group\npw-1\n");
    right.until("Leave this running");
    let _ = right.child.kill();
    let (_, seen) = right.ends();
    let said = std::fs::read_to_string(&log).unwrap();
    let words = "Apps that read jellyfin are let in as that account";
    assert!(said.contains(words), "{said}");
    assert_eq!(
        std::fs::read_to_string(&readers).unwrap(),
        r#"MediaBrowser Token="tok-1""#
    );
    assert_eq!(mode(&readers), 0o600);
    let (head, body) = asked
        .lock()
        .unwrap()
        .last()
        .cloned()
        .expect("the sign-in was asked");
    assert!(
        head.starts_with("POST /Users/AuthenticateByName HTTP/1.1\r\n"),
        "{head}"
    );
    assert!(
        header(&head, "authorization")
            .is_some_and(|a| a.starts_with(r#"MediaBrowser Client="svrngs""#)),
        "{head}"
    );
    let body: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({ "Username": "group", "Pw": "pw-1" })
    );
    assert_eq!(
        holding(dir.path(), b"pw-1", Path::new("")),
        Vec::<String>::new()
    );
    assert!(
        !std::fs::read_to_string(&log).unwrap().contains("pw-1"),
        "the password was logged"
    );
    assert!(
        !seen.iter().any(|l| l.contains("pw-1")),
        "the password was said"
    );
}

/// **A `sends:` is kept only on its lender's yes** (svrn-docs
/// `rings/current/CONTRACTS.md` §2, "The lend is the yes"): llama-server's
/// terms let apps send to `POST /v1/embeddings`, and `svrngs share` with
/// no terminal to answer y keeps none of it: the lend reads only, and its
/// lender is told so and how to say yes. Planted, watched failing:
/// `sends_said` answering yes with no terminal (the declaration holds the
/// row).
#[cfg(unix)]
#[test]
fn a_send_is_kept_only_on_its_lender_s_yes() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let stand_in = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = stand_in.local_addr().unwrap().port().to_string();
    let log = logs.path().join("share");
    holds_lab(dir.path(), logs.path());
    let mut running = Program::start(
        dir.path(),
        &log,
        &[
            "share",
            port.as_str(),
            "--program",
            "llama-server",
            "--group",
            "Lab",
        ],
        "\n",
    );
    running.until("Leave this running");
    let _ = running.child.kill();
    let _ = running.ends();
    let declared = std::fs::read_to_string(dir.path().join("lends/llama-server.json")).unwrap();
    assert!(!declared.contains("sends"), "{declared}");
    let log = std::fs::read_to_string(&log).unwrap();
    assert!(
        log.contains("llama-server is lent without sending: apps can only read it. To let them send, run this again in a terminal and type y."),
        "its lender is told: {log}"
    );
    assert!(
        log.contains("cli: share — its lender's yes to apps sending to it, or none"),
        "the branch is traced: {log}"
    );
    drop(stand_in);
}
