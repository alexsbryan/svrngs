//! The conformance canary (DESIGN §8, "The canaries"; the u5-2 row): the
//! web carried whole to whoever opens a lend, every way a group opens one.
//! Sam's computer founds the group, and Bo's and Alex's are in it; Sam and
//! Bo each lend the canary (`apps/web-conformance`), each with a secret of
//! its own. Alex opens both: from Alex's computer, each at its origin on
//! Alex's loopback; and from a phone holding Alex's key in the group, each
//! at an address of its own under the page's host (the wildcard) —
//! Chromium, and WebKit as an iPhone (`web/tests/conformance.mjs`). Each
//! route's verdict is printed per way and kept in `target/conformance/` for
//! `scripts/check.sh`, and every route passes. WebKit's seeks back to back are
//! a rate, not a gate (`a_rate`): kept beside the same seeks straight at a
//! canary, and counted by scripts/check.sh. Neither lend's cookie reaches the
//! other's requests; Bo's door names Alex's key; no byte Alex receives
//! holds either secret; a lent page finds no group whose key it could
//! unseal; and on a computer a lent page's post to the record API writes
//! nothing. Everything runs on this computer: iroh's relay on 127.0.0.1,
//! three nodes on it, two canaries and the page.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "browser/harness.rs"]
#[macro_use]
mod harness;

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use harness::{relay, relay_url, repo, site, Engine};
use iroh_relay::server::Server;
use serde_json::{json, Value};
use svrngs_core::link::Token;
use svrngs_core::pass::Label;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend::{self, LendName};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, declared, Person, RailAct, Roster, SigningKey};
use tokio::io::AsyncBufReadExt;

const NS: &str = "house";
const SAMS: &str = "Bearer sams-secret-41";
const BOS: &str = "Bearer bos-secret-42";
/// The keys: Sam's, Bo's and Alex's computers, and Alex's phone.
const SAM: u8 = 1;
const BO: u8 = 2;
const ALEX: u8 = 3;
const PHONE: u8 = 7;
const MARK: &str = "written-from-a-lent-page";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn no_seed(_: &Path) -> Result<Option<String>, String> {
    Ok(None)
}

async fn start(dir: &Path, relay: &Server, me: u8) -> Node {
    let node = Node::start(
        dir.to_path_buf(),
        signing(me),
        &RelayConfig::from_parts(vec![relay_url(relay)], Some("none"))
            .unwrap()
            .on_loopback_only(),
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    let seed = Roster::new(
        [(Person::parse("sam").unwrap(), vec![actor_of(&signing(SAM))])]
            .into_iter()
            .collect(),
    );
    node.rail().set_seed(NS, &seed).unwrap();
    tokio::time::timeout(Duration::from_secs(20), node.endpoint().online())
        .await
        .expect("the node reaches the relay on 127.0.0.1");
    node
}

/// A canary on this computer, as `who`, and the port it listens on.
struct Canary {
    _child: tokio::process::Child,
    port: u16,
}

async fn canary(who: &str) -> Canary {
    let mut child = tokio::process::Command::new("node")
        .arg(repo().join("apps/web-conformance/serve.mjs"))
        .arg(who)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs the canary: Node 18 or later");
    let mut lines = tokio::io::BufReader::new(child.stdout.take().unwrap()).lines();
    let first = lines
        .next_line()
        .await
        .unwrap()
        .expect("the canary prints its port");
    let port = serde_json::from_str::<Value>(&first).unwrap()["port"]
        .as_u64()
        .unwrap() as u16;
    Canary {
        _child: child,
        port,
    }
}

/// What a canary was asked, read from it directly.
async fn seen(c: &Canary) -> Vec<Value> {
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    http.get(format!("http://127.0.0.1:{}/seen", c.port))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

/// Sam's group: Bo's and Alex's computers and Alex's phone let in; each
/// computer holding Sam's record and knowing where the others listen; Sam
/// and Bo each lending the canary as `app`, each with its secret.
struct Group {
    sam: Node,
    bo: Node,
    alex: Node,
    canaries: (Canary, Canary),
    _dirs: Vec<tempfile::TempDir>,
}

async fn group(relay: &Server) -> Group {
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let sam = start(dirs[0].path(), relay, SAM).await;
    for (person, key) in [("bo", BO), ("alex", ALEX), ("alex", PHONE)] {
        let admit = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse(person).unwrap(),
                key: actor_of(&signing(key)),
                invite: None,
            },
            on_behalf_of: None,
        };
        sam.rail().commit(NS, &admit).unwrap().unwrap();
    }
    let bo = start(dirs[1].path(), relay, BO).await;
    let alex = start(dirs[2].path(), relay, ALEX).await;
    let addrs = |n: &Node| n.endpoint().addr().ip_addrs().copied().collect::<Vec<_>>();
    for (node, name) in [(&bo, "bo"), (&alex, "alex")] {
        let synced = node.sync_with(NS, &sam.endpoint().addr()).await;
        assert!(
            synced.stop.is_none(),
            "{name} syncs Sam's record: {synced:?}"
        );
    }
    for (a, b) in [
        (&sam, &bo),
        (&sam, &alex),
        (&alex, &bo),
        (&bo, &alex),
        (&alex, &sam),
        (&bo, &sam),
    ] {
        a.transport().add_known_peer(b.endpoint().id(), addrs(b));
    }
    let canaries = (canary("sam").await, canary("bo").await);
    for (dir, port, secret) in [
        (&dirs[0], canaries.0.port, SAMS),
        (&dirs[1], canaries.1.port, BOS),
    ] {
        let at = lend::lends_dir(dir.path());
        std::fs::create_dir_all(&at).unwrap();
        let body = json!({ "port": port, "kind": "app", "limit": 3 });
        std::fs::write(at.join("app.json"), body.to_string()).unwrap();
        let secrets = lend::secret_dir(dir.path(), &LendName::parse("app").unwrap());
        declared::write_declared_in(&secrets, "authorization", secret).unwrap();
    }
    Group {
        sam,
        bo,
        alex,
        canaries,
        _dirs: dirs,
    }
}

/// How a group opens its lends (DESIGN §7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Way {
    Computer,
    Wildcard,
}

impl Way {
    fn as_str(self) -> &'static str {
        match self {
            Way::Computer => "computer",
            Way::Wildcard => "wildcard",
        }
    }
}

/// The canary's routes, as the row names them, each with its check.
const ROUTES: [(&str, &str); 11] = [
    ("/-absolute paths", "root"),
    ("WebSocket echo, OPEN read", "websocket"),
    ("64 MiB read by Range", "range"),
    ("seeks back to back", "seeks"),
    ("64 MiB uploaded, its length told", "upload"),
    ("its own service worker", "worker"),
    ("SharedArrayBuffer under COOP/COEP", "isolated"),
    ("Host check, absolute redirect", "host"),
    ("cookie sign-in, read unreloaded", "signin"),
    ("post, Origin checked", "guarded"),
    ("local storage", "storage"),
];

/// WebKit's seeks back to back are a rate, not a gate (u5-11; operator,
/// 2026-10-06): through a lend it takes a closed range's parts in serial
/// 64 KiB reads, a round trip each, and its last seek missed 5 s in 1 of 8
/// tries, where straight at the canary over the same link it missed in
/// none (demo/runs/u5/2026-10-06-seek-stall/README.md). Its verdict is
/// printed `, a rate` and never fails the test; `rate` keeps it.
fn a_rate(engine: Engine, check: &str) -> bool {
    matches!(engine, Engine::Webkit) && check == "seeks"
}

/// Each route's verdict for one lend: `passed` or `failed`; a rate's with
/// `, a rate` after it.
fn verdicts(engine: Engine, lend: &Value, other: &Value) -> Vec<(&'static str, String, String)> {
    ROUTES
        .iter()
        .map(|(route, check)| {
            let mut ok = lend[check]["ok"].as_bool() == Some(true);
            let mut saw = lend[check]["saw"].to_string();
            match *check {
                // The cookie sign-in: set and sent, then read by the page.
                "signin" => {
                    ok &= lend["cookie"]["ok"].as_bool() == Some(true);
                    saw = format!("{saw}; the page reads {}", lend["cookie"]["saw"]);
                }
                // Local storage reads back, and holds nothing of another lend's.
                "storage" => {
                    let before = lend["stored"]["saw"].as_str().unwrap_or("");
                    let theirs = other["stored"]["saw"].as_str().unwrap_or("");
                    ok &= before.is_empty() && theirs.is_empty();
                    saw = format!(
                        "{saw}; before it wrote, {before:?}; before the other wrote, {theirs:?}"
                    );
                }
                _ => {}
            }
            let verdict = if ok { "passed" } else { "failed" };
            let verdict = match a_rate(engine, check) {
                true => format!("{verdict}, a rate"),
                false => verdict.to_string(),
            };
            (*route, verdict, saw)
        })
        .collect()
}

/// The relay as a phone away from home reaches it (u5-11): every byte each
/// way held `DELAY`, and what the lender sends the phone, which crosses the
/// lender's home uplink, paced to `RATE` bytes a second with at most
/// `QUEUE` bytes, a second's, held at once, as a home router's buffer
/// holds them: this host's uplink answered in 959 ms under load
/// (demo/runs/a/2026-10-05/data/upload.txt). Demo
/// A's phones met a 50 Mbit/s uplink shared by three streams, each told a
/// third, 16,666,666 bit/s, and a relayed round trip of 166 ms to 2 s
/// (demo/runs/a/2026-10-05/README.md, "Each stream's share"); here one
/// phone gets that third, with a 300 ms round trip. Answers the address to
/// give the phone.
const RATE: u64 = 16_666_666 / 8;
const DELAY: Duration = Duration::from_millis(150);
const QUEUE: u32 = RATE as u32;

async fn uplink(relay: &str) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let to: std::net::SocketAddr = relay
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .parse()
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let at = listener.local_addr().unwrap();
    // One direction: each read held DELAY, then written at `rate`.
    async fn paced(
        mut from: tokio::net::tcp::OwnedReadHalf,
        mut to: tokio::net::tcp::OwnedWriteHalf,
        rate: u64,
    ) {
        let held = std::sync::Arc::new(tokio::sync::Semaphore::new(QUEUE as usize));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut buf = vec![0; 16 * 1024];
            while let Ok(n) = from.read(&mut buf).await {
                let Ok(room) = held.clone().acquire_many_owned(n as u32).await else {
                    break;
                };
                let due = tokio::time::Instant::now() + DELAY;
                if n == 0 || tx.send((due, buf[..n].to_vec(), room)).is_err() {
                    break;
                }
            }
        });
        while let Some((due, bytes, room)) = rx.recv().await {
            tokio::time::sleep_until(due).await;
            if to.write_all(&bytes).await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_micros(bytes.len() as u64 * 1_000_000 / rate)).await;
            drop(room);
        }
        let _ = to.shutdown().await;
    }
    tokio::spawn(async move {
        while let Ok((phone, _)) = listener.accept().await {
            let Ok(relay) = tokio::net::TcpStream::connect(to).await else {
                continue;
            };
            let ((pr, pw), (rr, rw)) = (phone.into_split(), relay.into_split());
            // The phone's sending crosses the lender's downlink, unpaced here.
            tokio::spawn(paced(pr, rw, u64::MAX / 1_000_000));
            tokio::spawn(paced(rr, pw, RATE));
        }
    });
    format!("http://{at}/")
}

/// Every door's `carriage` lines (u5-11), kept in
/// `target/conformance/carriage.log` for every test this binary runs: each
/// range asked, answered, sent whole or cut off, beside the worker's trace.
fn carriage() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let kept = repo().join("target/conformance");
        std::fs::create_dir_all(&kept).unwrap();
        let log = std::fs::File::create(kept.join("carriage.log")).unwrap();
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new("carriage=debug"))
            .with_writer(std::sync::Mutex::new(log))
            .with_ansi(false)
            .init();
    });
}

/// The clock and this host's load, `uptime`'s line after the UTC time:
/// every run that times seeks records it at its start and end (the u5
/// queue's rule; u5-4's one miss came at a load of 33.6 on 12 cores).
async fn load() -> String {
    let out = tokio::process::Command::new("sh")
        .args(["-c", "echo $(date -u +%FT%TZ) $(uptime)"])
        .output()
        .await
        .expect("sh runs date and uptime");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// WebKit's seeks back to back (`a_rate`), beside the same seeks straight
/// at a canary over the same link (`web/tests/control.mjs`), a try for
/// each lend, with the host's load from `began` to now: printed, kept in
/// `target/conformance/seeks-<way>.tsv` for this run and added to
/// `target/seeks-webkit.log`, every run's on this host, which
/// scripts/check.sh counts. The control must run; its seeks, and the
/// lend's, never fail the test.
async fn rate(way: Way, ran: &Value, began: &str) {
    let out = tokio::process::Command::new("node")
        .arg(repo().join("web/tests/control.mjs"))
        .args(["webkit", "2"])
        .output()
        .await
        .expect("node runs the control");
    assert!(
        out.status.success(),
        "the seeks straight at the canary did not run ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let straight: Value = serde_json::from_str(stdout.lines().last().expect("it printed")).unwrap();
    let judged = |s: &Value| {
        let again = s["again"]
            .as_u64()
            .map_or("none".to_string(), |ms| ms.to_string());
        let ok = if s["ok"] == json!(true) {
            "passed"
        } else {
            "failed"
        };
        format!("{again}\t{ok}")
    };
    let ended = load().await;
    let mut lines = String::new();
    for (i, who) in ["sam", "bo"].into_iter().enumerate() {
        lines.push_str(&format!(
            "{}\t{who}\tthrough its lend\t{}\tstraight at the canary\t{}\t{began}\t{ended}\n",
            way.as_str(),
            judged(&ran["lends"][who]["seeks"]),
            judged(&straight[i]),
        ));
    }
    print!("{lines}");
    let kept = repo().join("target");
    std::fs::write(
        kept.join(format!("conformance/seeks-{}.tsv", way.as_str())),
        &lines,
    )
    .unwrap();
    use std::io::Write;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(kept.join("seeks-webkit.log"))
        .and_then(|mut f| f.write_all(lines.as_bytes()))
        .unwrap();
}

fn label(n: &Node) -> String {
    Label::of(n.endpoint().id().as_bytes(), "app").to_string()
}

/// What the browser half (`web/tests/conformance.mjs`) takes to open `g`'s
/// lends `way` in `engine`, the phone's link through `relay`.
async fn browser_args(way: Way, engine: Engine, g: &Group, relay: &Server) -> Value {
    let api = g.alex.api_addr();
    let lends = json!([
        { "who": "sam", "button": "sam’s app", "label": label(&g.sam),
          "origin": format!("http://{}.localhost:{}/", label(&g.sam), api.port()) },
        { "who": "bo", "button": "bo’s app", "label": label(&g.bo),
          "origin": format!("http://{}.localhost:{}/", label(&g.bo), api.port()) },
    ]);
    let token = Token {
        issuer: signing(SAM).verifying_key(),
        secret: [0; 16],
        relay: Some(uplink(&relay_url(relay)).await.parse().unwrap()),
    }
    .encode();
    let append = json!({ "op": "record", "payload": { "mark": MARK } }).to_string();
    let mut args = json!({
        "engine": engine.as_str(), "mode": way.as_str(), "site": site(), "lends": lends,
        "key": hex::encode(signing(PHONE).to_bytes()), "token": token, "secrets": [SAMS, BOS],
    });
    if way == Way::Computer {
        args["api"] = json!(format!("http://{api}"));
        args["append"] = json!(append);
    }
    args
}

/// The browser half run with `args`: what it saw, its one JSON line.
async fn browse(args: &Value) -> Value {
    let out = tokio::process::Command::new("node")
        .arg(repo().join("web/tests/conformance.mjs"))
        .arg(args.to_string())
        .output()
        .await
        .expect("node runs: the browser tests need Node 18 or later");
    assert!(
        out.status.success(),
        "the browser half failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(stdout.lines().last().expect("it printed")).unwrap()
}

async fn conformance(way: Way, engine: Engine) {
    carriage();
    let began = load().await;
    let relay = relay().await;
    let g = group(&relay).await;
    let api = g.alex.api_addr();
    let ran = browse(&browser_args(way, engine, &g, &relay).await).await;
    let kept = repo().join("target/conformance");
    std::fs::create_dir_all(&kept).unwrap();
    std::fs::write(
        kept.join(format!("{}-{}.json", way.as_str(), engine.as_str())),
        format!("{ran:#}"),
    )
    .unwrap();
    let mut shown = ran.clone();
    shown["logs"] = json!(format!(
        "{} lines",
        ran["logs"].as_array().map_or(0, Vec::len)
    ));
    // Of each lend's requests, those its worker did not answer; its
    // memory's samples are in the kept JSON.
    for who in ["sam", "bo"] {
        let lend = &mut shown["lends"][who];
        let missed: Vec<Value> = lend["requests"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|r| r.as_str().unwrap_or("").ends_with("not by the worker"))
            .cloned()
            .collect();
        lend["requests"] = json!(missed);
        lend["memory"] = json!(format!(
            "{} samples",
            lend["memory"].as_array().map_or(0, Vec::len)
        ));
    }

    // Each route's verdict, printed and kept for scripts/check.sh.
    let mut table = String::new();
    let mut failed = Vec::new();
    for (who, other) in [("sam", "bo"), ("bo", "sam")] {
        for (route, verdict, saw) in verdicts(engine, &ran["lends"][who], &ran["lends"][other]) {
            let line = format!(
                "{:<9} {:<8} {who:<4} {route:<34} {verdict}",
                way.as_str(),
                engine.as_str()
            );
            table.push_str(&format!("{line}\n"));
            if verdict == "failed" {
                failed.push(format!("{line}: {saw}"));
            }
        }
    }
    print!("{table}");
    std::fs::write(
        kept.join(format!("{}-{}.txt", way.as_str(), engine.as_str())),
        &table,
    )
    .unwrap();
    assert!(
        failed.is_empty(),
        "routes failed:\n{}\n{shown:#}",
        failed.join("\n")
    );
    if matches!(engine, Engine::Webkit) {
        rate(way, &ran, &began).await;
    }

    // Neither lend's cookie reaches the other's requests; Bo's door names
    // Alex's key, the computer's or the phone's; the secret is the
    // service's and only its own.
    let alex = actor_of(&signing(if way == Way::Computer { ALEX } else { PHONE }));
    for (c, own, theirs) in [(&g.canaries.0, "sam", "bo"), (&g.canaries.1, "bo", "sam")] {
        let mut asked = seen(c).await;
        // The last is this test's own read of what it saw, not through any door.
        asked.retain(|s| s["path"] != "/seen");
        let cookies: Vec<&str> = asked
            .iter()
            .filter_map(|s| s["cookie"].as_str())
            .filter(|c| !c.is_empty())
            .collect();
        assert!(
            cookies
                .iter()
                .any(|c| c.contains(&format!("canary-{own}="))),
            "{own}'s canary saw its own cookie: {cookies:?}"
        );
        assert!(
            !cookies
                .iter()
                .any(|c| c.contains(&format!("canary-{theirs}"))),
            "{own}'s canary saw {theirs}'s cookie: {cookies:?}"
        );
        assert!(
            asked.iter().all(|s| s["key"] == json!(alex)),
            "{own}'s door names Alex's key {alex}: {asked:?}"
        );
        assert!(
            asked.iter().all(|s| s["secret"] == json!(true)),
            "{own}'s secret reached its service"
        );
    }
    assert_eq!(
        ran["leaked"],
        json!([]),
        "no byte Alex receives holds either secret:\n{shown:#}"
    );
    for (who, n) in [("sam", &g.sam), ("bo", &g.bo)] {
        let lend = &ran["lends"][who];
        let at = lend["at"].as_str().unwrap_or_default();
        let own = match way {
            Way::Computer => format!("http://{}.localhost:{}/", label(n), api.port()),
            Way::Wildcard => format!("http://{}.localhost:", label(n)),
        };
        if way != Way::Computer {
            assert_eq!(
                lend["unsealable"]["groups"],
                json!(0),
                "a lent page finds a group to unseal:\n{shown:#}"
            );
        } else {
            assert!(lend["recordPost"].is_string(), "{shown:#}");
        }
        assert!(
            at.starts_with(&own),
            "{who}'s lend opened at {at}, not its own address {own}…:\n{shown:#}"
        );
    }
    if way == Way::Computer {
        let http = reqwest::Client::builder().no_proxy().build().unwrap();
        let log = http
            .get(format!("http://{api}/v1/rail/log?namespace={NS}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(
            !log.contains(MARK),
            "a lent page's post reached the record: {log}"
        );
    }
    for n in [g.sam, g.bo, g.alex] {
        n.endpoint().close().await;
    }
}

in_both_engines!(
    /// From Alex's computer: each lend at its origin on Alex's loopback.
    on_a_computer
);
async fn on_a_computer(engine: Engine) {
    conformance(Way::Computer, engine).await
}

in_both_engines!(
    /// From a phone, each lend at an address of its own under the page's host.
    on_a_phone_with_the_wildcard
);
async fn on_a_phone_with_the_wildcard(engine: Engine) {
    conformance(Way::Wildcard, engine).await
}

/// The most, in MB, a WebKit content process may hold through the tries
/// (u5-12): the line WebKit ends a worker's process past. The process that
/// held both floor lends' workers exited at a footprint of 1,065 MB and
/// 1,393 MB resident ('Memory usage policy changed: Unrestricted ->
/// Conservative', 'Suspended WebProcess is exiting due to memory
/// pressure'); the same tries left it at 1,332 MB in runs WebKit did not end
/// it, and at 511 MB (fixed-1) and 670 MB (fixed-2) once a body went a
/// slice at a time (demo/runs/u5/2026-10-06-worker-gone/).
const HELD: u64 = 1024;

/// #65 (u5-12; `ralph/REVIEW_FINDINGS.md`): WebKit as an iPhone, the
/// canary's 64 MiB upload after its seeks, through each lend in turn,
/// `TRIES` times in all (10 if unset), with the wildcard (the floor's
/// runs went with it, u6-00).
/// Each try's verdict, what the page's own host was asked meanwhile that is
/// not its files — a request no lend's worker answered — and the host's
/// load at its start and end are printed and added to
/// `target/upload-after-seeks.log`; the run's whole JSON, each try's
/// requests and worker trace in it, is kept in `target/upload-after-seeks/`
/// beside WebKit's own log of the run (macOS `log stream`, its subsystem's
/// lines), which names what WebKit did with its workers. It fails if the
/// page's host answered any try's upload, or if any of the browser's
/// content processes, sampled each second, held `HELD` or more — or none
/// was sampled, which judges nothing. `ROUTES` (`seeks,upload` if
/// unset) names the canary's checks each try runs. Not in scripts/check.sh:
/// run by name, `cargo test -p svrngs-node --test conformance
/// upload_after_seeks -- --ignored`.
mod upload_after_seeks {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "u5-12's runs, by name"]
    async fn with_the_wildcard() {
        tries(Way::Wildcard).await
    }
}

async fn tries(way: Way) {
    let kept = repo().join("target/upload-after-seeks");
    std::fs::create_dir_all(&kept).unwrap();
    let webkit_log =
        std::fs::File::create(kept.join(format!("webkit-{}.log", way.as_str()))).unwrap();
    let _webkit = tokio::process::Command::new("/usr/bin/log")
        .args([
            "stream",
            "--style",
            "compact",
            "--predicate",
            "subsystem == \"com.apple.WebKit\"",
        ])
        .stdout(webkit_log)
        .kill_on_drop(true)
        .spawn()
        .ok();
    let n: usize = std::env::var("TRIES").map_or(10, |t| t.parse().expect("TRIES is a number"));
    let routes: Vec<String> = std::env::var("ROUTES")
        .unwrap_or("seeks,upload".into())
        .split(',')
        .map(String::from)
        .collect();
    let relay = relay().await;
    let g = group(&relay).await;
    let mut args = browser_args(way, Engine::Webkit, &g, &relay).await;
    let lends = args["lends"].as_array().unwrap().clone();
    args["lends"] = (0..n)
        .map(|i| {
            let mut lend = lends[i % 2].clone();
            lend["who"] = json!(format!("{} {}", lend["who"].as_str().unwrap(), i + 1));
            lend
        })
        .collect();
    args["routes"] = json!(routes);
    let ran = browse(&args).await;
    std::fs::write(
        kept.join(format!("{}.json", way.as_str())),
        format!("{ran:#}"),
    )
    .unwrap();
    let mut lines = String::new();
    let mut lost = Vec::new();
    let mut most = None;
    for i in 0..n {
        let who = args["lends"][i]["who"].as_str().unwrap();
        let seen = &ran["lends"][who];
        let host: Vec<&str> = seen["asked"]
            .as_array()
            .map(|h| h.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let checks: Vec<String> = routes
            .iter()
            .map(|r| format!("{r} {} {}", seen[r]["ok"], seen[r]["saw"]))
            .collect();
        // `<time> <pid>:<MB> …` each second: the most any process held.
        let held = seen["memory"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .flat_map(|l| l.split(' ').skip(1))
            .filter_map(|p| p.split_once(':')?.1.parse::<u64>().ok())
            .max();
        most = most.max(held);
        let line = format!(
            "{}\t{who}\t{}\tthe page's host was asked {:?}\tthe most a content process held {} MB\t{}\t{}\n",
            way.as_str(),
            checks.join("\t"),
            host,
            held.map_or("never sampled".into(), |m| m.to_string()),
            seen["load"][0].as_str().unwrap_or("?"),
            seen["load"][1].as_str().unwrap_or("?"),
        );
        if host.iter().any(|l| l.contains(" POST ")) || held.is_some_and(|m| m >= HELD) {
            lost.push(line.clone());
        }
        lines.push_str(&line);
    }
    print!("{lines}");
    use std::io::Write;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(repo().join("target/upload-after-seeks.log"))
        .and_then(|mut f| f.write_all(lines.as_bytes()))
        .unwrap();
    assert!(
        lost.is_empty(),
        "the page's host answered an upload, or a process held {HELD} MB or more:\n{}",
        lost.join("")
    );
    assert!(
        most.is_some(),
        "no content process was sampled, so what they held is not judged"
    );
    for n in [g.sam, g.bo, g.alex] {
        n.endpoint().close().await;
    }
}
