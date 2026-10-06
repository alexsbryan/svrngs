// SPDX-License-Identifier: AGPL-3.0-or-later
//! Lend a service, u3-5's bar (DESIGN §8, U3): Sam shares a service on
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
use std::io::{BufRead, BufReader, Read, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::process::{Child, Command as Process, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use svrngs::cli::{self, Refusal, Said, Verb};
use svrngs::group::{self, Name};
use svrngs::share::{self, Share};
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, RelayConfig};
use svrngs_node::lend::{Kind, LendName};
use svrngs_node::link::{Link, KEPT_WORDS};
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

/// Sam's password for Sam's own service: Jellyfin 12's form.
const SECRET: &str = r#"MediaBrowser Token="sams-own-1234""#;

fn no_relay() -> RelayConfig {
    RelayConfig::from_parts(Vec::new(), Some("none")).unwrap()
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
    let (seed, printed) = cli::new_group(&sam.node, name("house"), name("Sam"), None, &no_relay())
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
    let asked = Share {
        port: std::num::NonZeroU16::new(origin.port()).unwrap(),
        name: None,
        kind: Some(Kind::parse("pad").unwrap()),
        limit: None,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
    };
    let lend = share::lend_of(
        &asked,
        share::listens(asked.port),
        share::well_known_kind(asked.port),
    )
    .expect("something listens there, and its kind is given");
    cli::share_lend(sam.dir.path(), &lend, Some(SECRET)).unwrap();
    let mut said = vec![Said::Shared {
        lend: lend.clone(),
        secret: true,
        accounts: None,
        running: true,
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
    let stopped = cli::unshare(sam.dir.path(), &lend.name).expect("Sam lent pad");
    assert_eq!(stopped, Said::Unshared(lend.name.clone()));
    said.push(stopped.to_string());
    let again = cli::running(alex.dir.path(), &alex.node).await.to_string();
    assert!(
        !again.contains("Sam's"),
        "a stopped share is still listed: {again:?}"
    );
    said.push(again);
    let refused = cli::unshare(sam.dir.path(), &lend.name).unwrap_err();
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
    let Ok(Verb::Share(asked)) = cli::parse(argv(&[
        "share", "8096", "--name", "box", "--limit", "2", "--refuse", "/admin",
    ])) else {
        panic!("share parses");
    };
    assert_eq!(asked.name, Some(LendName::parse("box").unwrap()));
    assert_eq!(asked.limit.map(|l| l.get()), Some(2));
    assert_eq!(asked.refused.len(), 1);
    assert_eq!(asked.upload, None, "no upload stated, none declared");
    assert!(cli::parse(argv(&["new", "x", "--limit", "2"])).is_err());
    let Ok(Verb::Share(asked)) = cli::parse(argv(&["share", "8096", "--upload", "30"])) else {
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
    let Ok(Verb::Share(asked)) = cli::parse(argv(&[
        "share", "8096", "--reads", "films", "--reads", "music",
    ])) else {
        panic!("share parses what it reads");
    };
    let reads: Vec<&str> = asked.reads.iter().map(|k| k.as_str()).collect();
    assert_eq!(reads, ["films", "music"]);
    assert!(cli::parse(argv(&["share", "8096", "--reads", "a/b"])).is_err());
    assert!(cli::parse(argv(&["new", "x", "--reads", "films"])).is_err());
}

/// The program, its output read line by line as it comes.
struct Program {
    child: Child,
    lines: std::sync::mpsc::Receiver<String>,
    seen: Vec<String>,
}

impl Program {
    /// `svrngs <args>` over `dir`, tracing everything to `log`, with `input`
    /// on its standard input.
    fn start(dir: &Path, log: &Path, args: &[&str], input: &str) -> Self {
        let mut child = Process::new(env!("CARGO_BIN_EXE_svrngs"))
            .args(args)
            .env("SVRNGS_DIR", dir)
            .env("RUST_LOG", "trace")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(std::fs::File::create(log).unwrap())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.stdout.take().unwrap();
        let (tx, lines) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    return;
                }
            }
        });
        Program {
            child,
            lines,
            seen: Vec::new(),
        }
    }

    /// Read its output until a line holds `text`.
    fn until(&mut self, text: &str) {
        while !self.seen.iter().any(|l| l.contains(text)) {
            match self.lines.recv_timeout(Duration::from_secs(30)) {
                Ok(l) => self.seen.push(l),
                Err(_) => panic!("no {text:?} in {:?}", self.seen),
            }
        }
    }

    /// Wait for it to end by itself; its exit code and whole output.
    fn ends(mut self) -> (i32, Vec<String>) {
        let status = self.child.wait().unwrap();
        while let Ok(l) = self.lines.recv_timeout(Duration::from_secs(5)) {
            self.seen.push(l);
        }
        (status.code().unwrap_or(-1), self.seen)
    }
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

    let argv1 = ["share", p1.as_str(), "--kind", "pad"];
    assert!(!argv1.iter().any(|a| a.contains("sams-own")));
    let mut running = Program::start(
        dir.path(),
        &logs.path().join("first"),
        &argv1,
        &format!("{SECRET}\n"),
    );
    running.until("Leave this running");
    assert!(
        running.seen.iter().any(|l| l.starts_with("Lent pad (port")),
        "{:?}",
        running.seen
    );

    let argv2 = ["share", p2.as_str(), "--kind", "box", "--limit", "2"];
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
        said.iter().any(|l| l.starts_with("Lent box (port")),
        "{said:?}"
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

/// A service that keeps its own accounts, on a port of a stand-in (u5-4;
/// DESIGN §3, "Lent apps", call 4): `share` asks no password, keeps none
/// though one is typed, and tells its lender how to make each person an
/// account there, in DESIGN §1's words.
#[cfg(unix)]
#[test]
fn a_service_with_its_own_accounts_is_lent_with_no_password() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let stand_in = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = stand_in.local_addr().unwrap().port().to_string();
    let log = logs.path().join("share");
    let mut running = Program::start(
        dir.path(),
        &log,
        &["share", port.as_str(), "--kind", "jellyfin"],
        &format!("{SECRET}\n"),
    );
    running.until("Leave this running");
    let _ = running.child.kill();
    let (_, seen) = running.ends();

    let how = seen
        .iter()
        .find(|l| l.starts_with("jellyfin asks each person to sign in once, with an account you make them. In jellyfin: "))
        .unwrap_or_else(|| panic!("the lender is not told how to make accounts: {seen:?}"));
    assert!(
        how.contains("Dashboard, Users and +") && how.ends_with("Then tell each person theirs."),
        "{how}"
    );
    assert!(
        !seen.iter().any(|l| l.contains("Its password stays")),
        "{seen:?}"
    );
    assert!(
        dir.path().join("lends/jellyfin.json").exists(),
        "jellyfin is declared"
    );
    assert!(
        !dir.path().join("secrets/lends/jellyfin").exists(),
        "a password was kept for jellyfin"
    );
    assert_eq!(
        holding(dir.path(), b"sams-own", Path::new("")),
        Vec::<String>::new()
    );
    let log = std::fs::read_to_string(&log).unwrap();
    assert!(
        !log.contains("takes a password"),
        "a password was asked: {log}"
    );
    assert!(
        log.contains("cli: share — no password asked"),
        "the branch is traced"
    );
    plain(&seen);
    drop(stand_in);
}
