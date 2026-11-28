// SPDX-License-Identifier: AGPL-3.0-or-later
//! The home page's address, and each lend's on this computer, the same
//! each time svrngs starts over one data dir (start-address (b); DESIGN §3,
//! §7): start-measure watched both move across one restart, 64525 to 49821,
//! and every program set to the old one refused
//! (`demo/runs/start/2026-10-13/README.md` §3 (a) 5). And another program
//! holding that address, said in the command's words, never a new one
//! taken in silence. The program, as a person runs it.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command as Process, Stdio};
use std::time::{Duration, Instant};

use svrngs::cli;
use svrngs::share::{self, Share};
use svrngs_node::lend::Program as LentProgram;

#[path = "common/as_main.rs"]
mod as_main;

/// svrngs itself, as [`Program`] runs it (common/as_main.rs). `open` and a
/// second bare command, which start no node, are the shipped binary's.
#[test]
#[ignore = "run by Program, in a process of its own, as svrngs"]
fn svrngs_here() {
    as_main::main()
}

/// The program, its output read line by line as it comes.
struct Program {
    child: Child,
    lines: std::sync::mpsc::Receiver<String>,
    seen: Vec<String>,
}

impl Program {
    /// `svrngs <args>` over `dir`, its home `dir` too (a group's folder
    /// is made under it), tracing everything to `log`: as_main.rs's, so it
    /// meets at a relay on 127.0.0.1, never n0.
    fn start(dir: &Path, log: &Path, args: &[&str]) -> Self {
        let mut child = as_main::svrngs()
            .args(args)
            .env("SVRNGS_DIR", dir)
            .env("HOME", dir)
            .env("RUST_LOG", "debug")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(std::fs::File::create(log).unwrap())
            .spawn()
            .unwrap();
        let mut out = BufReader::new(child.stdout.take().unwrap());
        as_main::past_header(&mut out);
        let (tx, lines) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in out.lines().map_while(Result::ok) {
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

    /// Read its output until a line holds `text`; that line.
    fn until(&mut self, text: &str) -> String {
        loop {
            if let Some(l) = self.seen.iter().find(|l| l.contains(text)) {
                return l.clone();
            }
            match self.lines.recv_timeout(Duration::from_secs(30)) {
                Ok(l) => self.seen.push(l),
                Err(_) => panic!("no {text:?} in {:?}", self.seen),
            }
        }
    }
}

/// Dropped, the program is stopped: a test that fails before it stops one
/// would otherwise leave it running, orphaned, holding its port.
impl Drop for Program {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The http address a printed line names.
fn address_in(line: &str) -> String {
    line.split_whitespace()
        .find(|w| w.starts_with("http://"))
        .unwrap_or_else(|| panic!("no address in {line:?}"))
        .trim_end_matches('.')
        .to_string()
}

/// `svrngs open` hands the address over and leaves: `xdg-open` stays until a
/// browser it started closes, and that browser must hold no pipe of its
/// caller's, as install.sh's `$(svrngs open)` reads one to the end. An opener
/// that refuses still leaves the person the address. Planted, watched failing:
/// `Command::status` in `open`, which held here until the opener left.
#[cfg(unix)]
#[test]
fn an_opener_that_stays_does_not_hold_svrngs_open() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let fake = logs.path().join(opener);
    let stub = |body: &str| {
        std::fs::write(&fake, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o700)).unwrap();
    };
    let invoke = || {
        Process::new(env!("CARGO_BIN_EXE_svrngs"))
            .arg("open")
            .env("SVRNGS_DIR", dir.path())
            .env("HOME", dir.path())
            .env("PATH", logs.path())
            .output()
            .unwrap()
    };
    let mut running = Program::start(dir.path(), &logs.path().join("first"), &[]);
    running.until("//svrngs.localhost:");
    stub("echo browser noise\nexec /bin/sleep 20");
    let started = std::time::Instant::now();
    let held = invoke();
    assert!(
        started.elapsed().as_secs() < 10,
        "the opener held it: {:?}",
        started.elapsed()
    );
    let said = String::from_utf8_lossy(&held.stdout);
    assert_eq!(
        said, "Your home page is open in your browser.\n",
        "{held:?}"
    );
    stub("exit 3");
    let refused = invoke();
    let said = String::from_utf8_lossy(&refused.stdout);
    assert!(
        said.starts_with("Open this in your browser within a minute: http"),
        "{refused:?}"
    );
    // The opener is handed the home page, at the address the bare command
    // said (moved from a_second_bare_command_reminds_without_starting_or_opening).
    let home = address_in(&running.until("//svrngs.localhost:"));
    let marker = logs.path().join("opened");
    stub(&format!("printf '%s' \"$1\" > '{}'", marker.display()));
    let opened = invoke();
    assert!(opened.status.success(), "{opened:?}");
    assert!(std::fs::read_to_string(&marker).unwrap().starts_with(&home));
}

/// **A second bare command reminds where the home page is, and starts,
/// opens and rewrites nothing**: while svrngs runs here, a bare `svrngs`
/// says the home page's address from the one the first wrote, with how to
/// open it; `new` asks it (cli.rs `new_running`), and with nothing
/// answering at the address it wrote is told svrngs is not running, never
/// "stop it"; an address it cannot read is refused, never a reminder
/// invented. svrngs running here is what
/// a second command reads of it: its data dir's lock held (cli::claim),
/// and its address and token written as the node writes them (api.rs
/// `write_address`, `TOKEN_FILE`), so no node runs and nothing is dialled.
/// That a first bare command starts one and its home page answers at that
/// address is the_home_page_and_a_lend_answer_at_one_address_across_a_restart's;
/// that `open` hands the opener the home page,
/// an_opener_that_stays_does_not_hold_svrngs_open's. The plant is the old
/// Busy refusal on a second bare invocation.
#[cfg(unix)]
#[test]
fn a_second_bare_command_reminds_without_starting_or_opening() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let fake = logs.path().join(opener);
    let marker = logs.path().join("opened");
    std::fs::write(&fake, "#!/bin/sh\nprintf '%s' \"$1\" > \"$OPEN_MARKER\"\n").unwrap();
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o700)).unwrap();
    // The shipped binary: a command that starts no node opens no socket.
    let invoke = |args: &[&str]| {
        Process::new(env!("CARGO_BIN_EXE_svrngs"))
            .args(args)
            .env("SVRNGS_DIR", dir.path())
            .env("HOME", dir.path())
            .env("PATH", logs.path())
            .env("OPEN_MARKER", &marker)
            .env("RUST_LOG", "cli=debug")
            .output()
            .unwrap()
    };
    let running = cli::claim(dir.path()).expect("the data dir is free");
    let port = 47_999;
    let at = SocketAddr::from(([127, 0, 0, 1], port));
    svrngs_node::api::write_address(dir.path(), at).unwrap();
    let token = "a-token-svrngs-keeps-here";
    std::fs::write(dir.path().join(svrngs_node::api::TOKEN_FILE), token).unwrap();
    let home = format!("http://svrngs.localhost:{port}/");
    let api = dir.path().join("api");
    let saved = std::fs::read(&api).unwrap();
    let second = invoke(&[]);
    let said = String::from_utf8_lossy(&second.stdout);
    let traced = String::from_utf8_lossy(&second.stderr);
    assert!(second.status.success(), "{said}\n{traced}");
    assert!(said.contains(&home), "{said}");
    assert!(
        said.contains("Run `svrngs open` to open it in your browser."),
        "{said}"
    );
    assert!(
        traced.contains("cli: run — local home reminder"),
        "{traced}"
    );
    assert!(!said.contains(token) && !traced.contains(token));
    assert!(!marker.exists(), "the reminder launched a browser");
    assert!(
        matches!(
            cli::claim(dir.path()),
            Err(cli::Refusal::Refused(cli::Refused::Busy(_)))
        ),
        "the running svrngs keeps its data dir"
    );
    assert_eq!(std::fs::read(&api).unwrap(), saved);
    let other = invoke(&["new", "Lab", "--as", "Sam"]);
    let refused = String::from_utf8_lossy(&other.stderr);
    assert_eq!(other.status.code(), Some(3), "{refused}");
    assert!(
        refused.contains("svrngs is not running on this computer"),
        "{refused}"
    );
    assert!(!refused.contains("stop it"), "{refused}");
    assert!(
        refused.contains("cli: svrngs runs here already, so it is asked")
            && refused.contains("/v1/new"),
        "`new`, asked at its route: {refused}"
    );
    for broken in [None, Some("not an address")] {
        std::fs::remove_file(&api).unwrap();
        if let Some(text) = broken {
            std::fs::write(&api, text).unwrap();
        }
        let refused = invoke(&[]);
        assert!(!refused.status.success(), "{refused:?}");
        assert!(refused.stdout.is_empty(), "invented a reminder");
        std::fs::write(&api, &saved).unwrap();
    }
    std::fs::remove_file(&api).unwrap();
    std::fs::create_dir(&api).unwrap();
    let unreadable = invoke(&[]);
    assert!(!unreadable.status.success());
    assert!(unreadable.stdout.is_empty());
    std::fs::remove_dir(&api).unwrap();
    std::fs::write(&api, saved).unwrap();
    assert!(!marker.exists(), "nothing launched a browser");
    drop(running);
}

/// GET `path` at `address`, as a browser typing it asks (`Sec-Fetch-Site:
/// none`, which the home page answers) or as a program does (`browser`
/// false: no `Sec-Fetch-*`, which a lend answers as its own), asked as
/// you by `token`, if any, with the code it makes for this connection
/// (svrngs-node's `as_you_on_std`): its status line and body.
fn get(address: &str, path: &str, browser: bool, token: Option<&str>) -> (String, String) {
    let authority = address
        .strip_prefix("http://")
        .and_then(|a| a.strip_suffix('/'))
        .unwrap_or_else(|| panic!("not an address: {address}"));
    let port = authority.rsplit_once(':').unwrap().1;
    let at: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let mut sock = TcpStream::connect_timeout(&at, Duration::from_secs(5))
        .unwrap_or_else(|e| panic!("{address} refused: {e}"));
    let fetch = if browser {
        "Sec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\n"
    } else {
        ""
    };
    let token = token.map_or(String::new(), |t| {
        svrngs_node::node::as_you_on_std(&mut sock, t)
    });
    write!(
        sock,
        "GET {path} HTTP/1.1\r\nHost: {authority}\r\n{fetch}{token}Connection: close\r\n\r\n"
    )
    .unwrap();
    let mut answer = String::new();
    sock.read_to_string(&mut answer).unwrap();
    let (head, body) = answer.split_once("\r\n\r\n").unwrap_or((&answer, ""));
    (
        head.lines().next().unwrap_or_default().to_string(),
        body.to_string(),
    )
}

/// Where this computer opens its own lend `name`, as `/v1/lends` at the
/// API in `dir` answers a program asking as you by `token`.
fn origin_of(dir: &Path, name: &str, token: &str) -> String {
    let api = std::fs::read_to_string(dir.join(svrngs_node::api::API_FILE)).unwrap();
    let api = format!("{}/", api.trim());
    let json = |path: &str| {
        let (status, body) = get(&api, path, false, Some(token));
        assert!(status.ends_with("200 OK"), "{path} {status} {body}");
        (
            serde_json::from_str::<serde_json::Value>(&body).unwrap(),
            body,
        )
    };
    let (groups, _) = json("/v1/groups");
    let id = groups["groups"][0]["id"].as_str().expect("Lab").to_string();
    let (lends, body) = json(&format!("/v1/lends?namespace={id}"));
    let lent = lends["lent"].as_array().cloned().unwrap_or_default();
    let named = lent.iter().find(|l| l["name"] == name);
    let origin = named.and_then(|l| l["origin"].as_str());
    origin
        .unwrap_or_else(|| panic!("no {name} in {body}"))
        .to_string()
}

/// A service answering every request 200 `ok`.
fn service() -> TcpListener {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let serving = listener.try_clone().unwrap();
    std::thread::spawn(move || {
        for mut sock in serving.incoming().map_while(Result::ok) {
            let mut head = Vec::new();
            let mut b = [0u8; 1];
            while !head.ends_with(b"\r\n\r\n") && sock.read_exact(&mut b).is_ok() {
                head.push(b[0]);
            }
            let _ = sock
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
        }
    });
    listener
}

/// Sam's computer makes a group and lends a service; the bare command
/// runs, as the LaunchAgent runs it, is stopped, as `launchctl bootout`
/// stops it, and starts again over the same data dir, as `KeepAlive` does.
/// The home page and the lend answer at the same address both times, and
/// the record's token is the same file (give-token): a program that read
/// it before the restart asks with it after. The failing input: a token
/// minted at every start, which the second run's ask refuses.
#[cfg(unix)]
#[test]
fn the_home_page_and_a_lend_answer_at_one_address_across_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let pad = service();
    let asked = Share {
        port: std::num::NonZeroU16::new(pad.local_addr().unwrap().port()).unwrap(),
        name: None,
        kind: Some(LentProgram::parse("pad").unwrap()),
        limit: None,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        asks: None,
        sign_in: false,
        readers: false,
        raw: None,
        audience: svrngs_node::lend::Audience::Legacy,
        pid: None,
        replace: false,
    };
    let mut lend = share::lend_of(&asked, None, "").expect("pad listens");
    svrngs_node::node::approve(&mut lend);
    svrngs_node::lend::declare(dir.path(), &lend, None).unwrap();

    let new = ["new", "Lab", "--as", "Sam"];
    let mut made = Program::start(dir.path(), &logs.path().join("new"), &new);
    made.until("Leave this running");
    drop(made);
    // Lent to Lab, the group just made (LENDING_SCOPE).
    let lab = svrngs::group::groups(dir.path())[0].id();
    assert!(svrngs_node::lend::set_audience(dir.path(), &lend.name, vec![lab]).unwrap());

    let mut at = Vec::new();
    let mut kept = None;
    for run in ["first", "again"] {
        let mut svrngs = Program::start(dir.path(), &logs.path().join(run), &[]);
        let home = address_in(&svrngs.until("//svrngs.localhost:"));
        let token = std::fs::read_to_string(dir.path().join("token")).unwrap();
        let read_before = kept.get_or_insert(token.clone());
        assert_eq!(
            &token, read_before,
            "{run}: the token moved across a restart"
        );
        let lent = origin_of(dir.path(), "pad", read_before);
        // As a program the person runs, asking as them: the address is
        // what this test holds, and a browser's session is session.rs's
        // (give-session).
        let (status, page) = get(&home, "/", true, Some(read_before));
        assert!(status.ends_with("200 OK"), "{run}: {home} {status}");
        assert!(
            page.contains("/home.js"),
            "{run}: {home} is not the home page"
        );
        let (status, body) = get(&lent, "/", false, Some(read_before));
        assert!(status.ends_with("200 OK"), "{run}: {lent} {status}");
        assert_eq!(body, "ok", "{run}: {lent}");
        drop(svrngs);
        at.push((home, lent));
    }
    assert_eq!(at[0], at[1], "the addresses moved across a restart");
    drop(pad);
}

/// svrngs started once, then stopped; another program takes its address;
/// svrngs started again says so, in the command's words, and stops —
/// never starting at another address in silence.
#[cfg(unix)]
#[test]
fn another_program_holding_the_address_is_said_and_svrngs_stops() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let mut first = Program::start(dir.path(), &logs.path().join("first"), &[]);
    let home = address_in(&first.until("//svrngs.localhost:"));
    drop(first);
    let port = home.trim_end_matches('/').rsplit_once(':').unwrap().1;
    let held = TcpListener::bind(format!("127.0.0.1:{port}"))
        .unwrap_or_else(|e| panic!("the test cannot hold {port}: {e}"));

    let log = logs.path().join("again");
    let mut again = Program::start(dir.path(), &log, &[]);
    let deadline = Instant::now() + Duration::from_secs(30);
    let code = loop {
        if let Some(status) = again.child.try_wait().unwrap() {
            break status.code();
        }
        if let Ok(l) = again.lines.recv_timeout(Duration::from_millis(200)) {
            again.seen.push(l);
        }
        assert!(
            Instant::now() < deadline,
            "svrngs ran on with another program at {port}: {:?}",
            again.seen
        );
    };
    let said = std::fs::read_to_string(&log).unwrap();
    let refusal = said
        .lines()
        .find(|l| l.starts_with("svrngs: "))
        .unwrap_or_else(|| panic!("nothing said: {:?} {said}", again.seen));
    assert_eq!(code, Some(3), "{refusal}");
    assert!(
        refusal.contains(&format!(
            "Another program on this computer holds 127.0.0.1:{port}"
        )),
        "{refusal}"
    );
    assert!(refusal.contains(&home), "{refusal}");
    drop(held);
}

/// The token each data dir [`data_dir_at`] makes keeps.
const TOKEN: &str = "the-token-svrngs-keeps";

/// What [`not_svrngs`] answers as svrngs's `/v1/open` would.
const OPENED: &str = "HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n{\"open\": \"http://\"}";

/// What [`not_svrngs`] answers as an older svrngs answers `POST
/// /v1/proof`, which it has no route for (1.13.2, 9be0fcc7, asked
/// 2026-11-21).
const OLDER: &str = "HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\n\r\n";

/// A program on 127.0.0.1 that is not svrngs, run as `argv` before Python:
/// it says its port, then for each connection answers its first request
/// `answer`, reads what else it is sent until the connection closes, and
/// writes how many bytes it was sent and what.
fn not_svrngs(before: &[&str], answer: &str) -> (Child, u16, BufReader<std::process::ChildStdout>) {
    not_svrngs_holding(before, answer, None)
}

/// [`not_svrngs`], holding the data dir `lock` as a running svrngs holds
/// its own (`cli::claim`), when one is given.
fn not_svrngs_holding(
    before: &[&str],
    answer: &str,
    lock: Option<&Path>,
) -> (Child, u16, BufReader<std::process::ChildStdout>) {
    let script = "import fcntl, os, socket\nheld = os.environ.get('LOCK') and open(os.environ['LOCK'], 'w')\nheld and fcntl.flock(held, fcntl.LOCK_EX)\ns = socket.socket()\ns.bind(('127.0.0.1', 0))\ns.listen()\nprint(s.getsockname()[1], flush=True)\nwhile True:\n    c, _ = s.accept()\n    c.settimeout(2)\n    got = b''\n    try:\n        while b'\\r\\n\\r\\n' not in got:\n            b = c.recv(65536)\n            if not b: break\n            got += b\n        c.sendall(os.environ['ANSWER'].encode())\n        while True:\n            b = c.recv(65536)\n            if not b: break\n            got += b\n    except OSError: pass\n    print(len(got), got, flush=True)\n    c.close()";
    let (program, rest) = before.split_first().unwrap_or((&"python3", &[]));
    let mut child = Process::new(program)
        .args(rest)
        .args(if before.is_empty() {
            &[][..]
        } else {
            &["python3"][..]
        })
        .args(["-I", "-c", script])
        .env("ANSWER", answer)
        .envs(lock.map(|l| ("LOCK", l)))
        .current_dir("/")
        .stdout(Stdio::piped())
        .spawn()
        .expect("python3 runs");
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    out.read_line(&mut port).unwrap();
    let port = port
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("no port: {port:?}"));
    (child, port, out)
}

/// What [`not_svrngs`] says it was sent on the one connection it was
/// asked on, which it says once that connection closes; then it stops.
fn told(python: &mut Child, said: &mut BufReader<std::process::ChildStdout>) -> String {
    let mut seen = String::new();
    said.read_line(&mut seen).unwrap();
    python.kill().unwrap();
    assert!(!seen.is_empty(), "Python said nothing of what it was sent");
    seen
}

/// [`not_svrngs`] in this test's own process, which runs svrngs's own
/// executable (as_main.rs): one connection, and all it was sent.
fn not_svrngs_here() -> (u16, std::thread::JoinHandle<String>) {
    let ours = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = ours.local_addr().unwrap().port();
    let heard = std::thread::spawn(move || {
        let (mut c, _) = ours.accept().unwrap();
        c.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let (mut got, mut buf, mut answered) = (Vec::new(), [0; 65536], false);
        while let Ok(n @ 1..) = c.read(&mut buf) {
            got.extend_from_slice(&buf[..n]);
            if !answered && got.windows(4).any(|w| w == b"\r\n\r\n") {
                let body = r#"{"open": "http://"}"#;
                let answer = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                );
                answered = c.write_all(answer.as_bytes()).is_ok();
            }
        }
        String::from_utf8_lossy(&got).into_owned()
    });
    (port, heard)
}

/// A data dir whose `api` names 127.0.0.1:`port` and whose token is [`TOKEN`].
fn data_dir_at(port: u16) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("api"), format!("http://127.0.0.1:{port}\n")).unwrap();
    std::fs::write(dir.path().join("token"), format!("{TOKEN}\n")).unwrap();
    dir
}

/// What a program at svrngs's address was sent: none of the token, not one
/// byte of it, and no `authorization` at all.
fn sent_none_of_the_token(seen: &str) {
    let shown = seen.contains(TOKEN) || seen.to_ascii_lowercase().contains("authorization");
    assert!(!shown, "it was sent the token: {seen}");
}

/// No relay, for a command asked in this process that dials none.
fn no_relay() -> svrngs_node::iroh::RelayConfig {
    svrngs_node::iroh::RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: true,
    }
}

/// `svrngs open --print` over `dir`, as the applications menu's opener runs
/// it, but with no browser: its words, or its refusal's, and exit code.
async fn opened(dir: &Path) -> Result<String, (u8, String)> {
    let said = cli::answer(cli::Verb::Open(None, true), &dir.to_path_buf(), &no_relay()).await;
    said.map_err(|r| (r.exit_code(), r.to_string()))
}

/// The command's refusal of what answers at 127.0.0.1:`port` without
/// proving it is svrngs: exit 1, in words that say what to do.
fn unproven(port: u16, (code, said): (u8, String)) {
    assert_eq!(code, 1, "{said}");
    let at = format!("http://127.0.0.1:{port}");
    let words = format!("Something other than your svrngs answers at {at}, where svrngs listens, so svrngs sent it none of your token.");
    assert!(said.contains(&words), "{said}");
    assert!(said.contains("`svrngs restart`"), "{said}");
}

/// **Only svrngs, proving it holds your token, is sent it** (V1.13.3's fix
/// review, finding 4): while svrngs is down, a program holding its address
/// that can't prove it holds the token is asked for the proof alone and
/// sent none of the token, and `svrngs open`, as the applications menu and
/// install.sh run it, says that something other than your svrngs answers
/// there and what to do (exit 1). Two such programs: Python of this
/// account, and one running svrngs's own file, as this test's listener
/// does, as a program in a container or a sandbox that can't read the
/// token can. Planted, watched failing: 588b0abc's process check sent this
/// test's listener `POST /v1/open` with `authorization: Bearer
/// the-token-svrngs-keeps`, since it runs svrngs's executable.
#[tokio::test(flavor = "multi_thread")]
async fn only_svrngs_proving_it_holds_your_token_is_sent_it() {
    let (port, heard) = not_svrngs_here();
    let refused = opened(data_dir_at(port).path()).await;
    sent_none_of_the_token(&heard.join().unwrap());
    unproven(port, refused.expect_err("refused"));

    let (mut python, port, mut sent) = not_svrngs(&[], OPENED);
    let refused = opened(data_dir_at(port).path()).await;
    let seen = told(&mut python, &mut sent);
    sent_none_of_the_token(&seen);
    assert!(seen.contains("POST /v1/proof HTTP/1.1"), "{seen}");
    unproven(port, refused.expect_err("refused"));
}

/// `svrngs ask <args>` over `dir`, the shipped binary as a script runs
/// it: its exit code, what it printed, and what it said.
fn ask(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let ran = Process::new(env!("CARGO_BIN_EXE_svrngs"))
        .arg("ask")
        .args(args)
        .env("SVRNGS_DIR", dir)
        .env("HOME", dir)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    (ran.status.code(), text(&ran.stdout), text(&ran.stderr))
}

/// What [`not_svrngs`] answers as svrngs's `/v1/groups` would.
const GROUPS: &str = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\n{\"groups\":[]}";

/// **`svrngs ask` sends a program squatting svrngs's address none of the
/// token, nor a code** ("Never send the key", V1.13.3): Python holding the
/// address while svrngs is down, answering every first request as
/// svrngs's `/v1/groups` would, is sent `POST /v1/proof` alone, and
/// `svrngs ask /v1/groups`, as a script runs it, prints nothing and exits
/// 1, saying something other than your svrngs answers there. Planted,
/// watched failing: 96ea3da7 knew no `ask` (exit 1, "unknown command
/// `ask`"); and an ask sending the token as Bearer after the challenge's
/// answer, whatever it said, sent Python `authorization: Bearer
/// the-token-svrngs-keeps`.
#[test]
fn svrngs_ask_sends_a_squatter_none_of_the_token() {
    let (mut python, port, mut sent) = not_svrngs(&[], GROUPS);
    let dir = data_dir_at(port);
    let (code, printed, said) = ask(dir.path(), &["/v1/groups"]);
    // What Python was sent, said once that connection closed; none in 5 s
    // is nothing asked of it.
    let ((tx, heard), pid) = (std::sync::mpsc::channel(), python.id().to_string());
    std::thread::spawn(move || tx.send(told(&mut python, &mut sent)));
    let seen = heard
        .recv_timeout(Duration::from_secs(5))
        .unwrap_or_else(|_| {
            Process::new("kill").arg(&pid).status().unwrap();
            panic!("Python was asked nothing: {code:?} {said}")
        });
    sent_none_of_the_token(&seen);
    assert!(seen.contains("POST /v1/proof HTTP/1.1"), "{seen}");
    assert_eq!(printed, "", "{said}");
    unproven(port, (code.unwrap_or_default() as u8, said));
}

/// **`svrngs ask` prints svrngs's answer, and says its refusal** ("Never
/// send the key", V1.13.3), of svrngs running over its data dir: a route
/// got, its JSON printed whole on a line of its own, exit 0; JSON posted to
/// a route, its answer printed; a route svrngs refuses, nothing printed,
/// exit 1, svrngs's status said; and what isn't a route or JSON refused
/// before anything is asked, saying how to ask, a route http can't send
/// among them. As apps/jellyfin/folders runs it. Planted, watched failing:
/// a route with a space in it asked unparsed, and said as svrngs not
/// running, exit 3.
#[test]
fn svrngs_ask_prints_svrngs_s_answer_and_says_its_refusal() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let mut running = Program::start(dir.path(), &logs.path().join("svrngs"), &[]);
    running.until("//svrngs.localhost:");
    let (code, printed, said) = ask(dir.path(), &["/v1/groups"]);
    assert_eq!(code, Some(0), "{said}");
    let groups: serde_json::Value = serde_json::from_str(&printed).unwrap();
    assert!(groups["groups"].is_array(), "{printed}");
    assert!(printed.ends_with("}\n"), "{printed:?}");
    let (code, printed, said) = ask(dir.path(), &["/v1/open", r#"{"at": "/"}"#]);
    assert_eq!(code, Some(0), "{said}");
    assert!(printed.contains("svrngs-open?code="), "{printed}");
    let (code, printed, said) = ask(dir.path(), &["/v1/nothing-here"]);
    assert_eq!((code, printed.as_str()), (Some(1), ""), "{said}");
    assert!(
        said.starts_with("svrngs: svrngs refused /v1/nothing-here: 404"),
        "{said}"
    );
    for wrong in [
        &["v1/groups"][..],
        &["/v1/open", "{at"],
        &["/v1/open", "{}", "{}"],
        &[],
        &["/v1/rail/log?namespace=my group"],
    ] {
        let (code, printed, said) = ask(dir.path(), wrong);
        assert_eq!((code, printed.as_str()), (Some(1), ""), "{wrong:?}: {said}");
        assert!(said.contains("svrngs ask"), "{wrong:?}: {said}");
    }
    // A route http can't send is said as one, with how to write it, never
    // as svrngs not running.
    let (_, _, said) = ask(dir.path(), &["/v1/rail/log?namespace=my group"]);
    assert!(said.contains("as %20"), "{said}");
}

/// `svrngs ask /v1/groups` over `dir`, asked every 50 ms until svrngs
/// answers it: never refused, exit 1, on the way. What it was told, and how
/// long the ask that was answered waited. (Not `open`, which starts svrngs
/// itself where none holds the data dir, as a test's program can't.)
async fn asked_until_answered(dir: &Path) -> String {
    let (began, mut told) = (Instant::now(), Vec::new());
    loop {
        assert!(
            began.elapsed() < Duration::from_secs(60),
            "never answered: {told:?}"
        );
        let asked = Instant::now();
        let verb = cli::Verb::Ask("/v1/groups".into(), None);
        let said = cli::answer(verb, &dir.to_path_buf(), &no_relay()).await;
        match said.map_err(|r| (r.exit_code(), r.to_string())) {
            Ok(said) => {
                assert!(said.contains("\"groups\""), "{said}");
                return format!(
                    "{told:?}, then answered after waiting {:?}",
                    asked.elapsed()
                );
            }
            Err((1, why)) => panic!("svrngs refused {:?} in: {why}", began.elapsed()),
            Err((code, why)) => told.push(format!("{code}: {why}")),
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// **svrngs is never refused while it starts** (V1.13.3's fix review,
/// findings 1 and 2): svrngs holds its address well before it takes a
/// connection there (svrngs-node node.rs `start_on`, the endpoint and the
/// folders between), so a command, asked every 50 ms, is told svrngs
/// isn't running until it holds its address, then waits for its proof and
/// is answered: as svrngs starts again over its data dir, and while it is
/// held there for a second, holding its address and taking no connection
/// (`kill -STOP`), as a slow start holds it (some 300 ms on a Mac with its
/// groups, its log said, 2026-11-21). Planted, watched failing on a Mac:
/// 588b0abc's process check found no process holding a connection svrngs
/// hadn't taken within 100 ms, and refused svrngs as "A program of
/// another account" (exit 1), which aborts install.sh's update.
#[tokio::test(flavor = "multi_thread")]
async fn svrngs_starting_is_never_refused() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let mut first = Program::start(dir.path(), &logs.path().join("first"), &[]);
    first.until("//svrngs.localhost:");
    drop(first);
    let mut again = Program::start(dir.path(), &logs.path().join("again"), &[]);
    let restarted = asked_until_answered(dir.path()).await;
    again.until("//svrngs.localhost:");
    let pid = again.child.id().to_string();
    let signal = move |how: &str| Process::new("kill").args([how, &pid]).status().unwrap();
    assert!(signal("-STOP").success());
    let resumed = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        signal("-CONT")
    });
    let held = asked_until_answered(dir.path()).await;
    assert!(resumed.join().unwrap().success());
    eprintln!("RESULT: restarted, {restarted}; held a second, {held}");
}

/// **Another account's program at svrngs's address is sent none of the
/// token**, on a real socket of another uid: a uid of this account's own
/// subordinate range (/etc/subuid), as `unshare --map-auto` gives it
/// without root. By hand on Linux: it needs newuidmap and a range in
/// /etc/subuid. Planted, watched failing: V1.13.3's first candidate sent it
/// `POST /v1/open` with the token.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "by hand on Linux: unshare --map-auto and a range in /etc/subuid"]
async fn another_account_s_program_at_svrngs_s_address_is_sent_none_of_the_token() {
    let theirs = [
        "unshare",
        "--map-auto",
        "--map-root-user",
        "setpriv",
        "--reuid=1",
        "--regid=1",
        "--clear-groups",
    ];
    let (mut python, port, mut sent) = not_svrngs(&theirs, OPENED);
    let refused = opened(data_dir_at(port).path()).await;
    let seen = told(&mut python, &mut sent);
    sent_none_of_the_token(&seen);
    eprintln!("RESULT: sent {seen:?}, then {refused:?}");
    unproven(port, refused.expect_err("refused"));
}

/// `launchctl` and `systemctl` in a folder of their own, the one `PATH` a
/// command run with [`restarted`] has, so no real one is reached: each
/// writes what it was asked to `said`, and to a restart of svrngs, as
/// `svrngs restart` asks it, runs `then`.
fn restarting(stubs: &Path, then: &str) {
    use std::os::unix::fs::PermissionsExt;
    let said = stubs.join("said");
    let body = format!("#!/bin/sh\necho \"$*\" >>'{}'\ncase \"$*\" in\n  \"kickstart -k \"*|\"--user restart svrngs.service\") {then} ;;\nesac\n", said.display());
    for name in ["launchctl", "systemctl"] {
        std::fs::write(stubs.join(name), &body).unwrap();
        std::fs::set_permissions(stubs.join(name), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// `svrngs open --print` over `dir`, the shipped program, with `PATH` the
/// stubs alone ([`restarting`]): its output, and the restarts it asked for.
fn restarted(dir: &Path, stubs: &Path) -> (std::process::Output, Vec<String>) {
    let out = Process::new(env!("CARGO_BIN_EXE_svrngs"))
        .args(["open", "--print"])
        .env("SVRNGS_DIR", dir)
        .env("HOME", dir)
        .env("PATH", stubs)
        .output()
        .unwrap();
    let said = std::fs::read_to_string(stubs.join("said")).unwrap_or_default();
    (out, said.lines().map(String::from).collect())
}

/// svrngs's token for a data dir it runs over, its 32 bytes as svrngs
/// writes them (api.rs `kept_token`), and `api` naming 127.0.0.1:`port`.
fn kept_at(dir: &Path, port: u16) {
    std::fs::write(dir.join("api"), format!("http://127.0.0.1:{port}\n")).unwrap();
    std::fs::write(dir.join("token"), "A".repeat(43)).unwrap();
}

/// What svrngs's command asks to restart svrngs, as `svrngs restart` does
/// (cli.rs `restart`): launchd's kickstart of its agent for this account,
/// or systemd's restart of its unit.
fn restart_asked() -> String {
    let id = Process::new("id").arg("-u").output().unwrap();
    let uid = String::from_utf8_lossy(&id.stdout).trim().to_string();
    match cfg!(target_os = "macos") {
        true => format!("kickstart -k gui/{uid}/sh.svrnme.svrngs"),
        false => "--user restart svrngs.service".into(),
    }
}

/// What keeps svrngs running at login in `home` runs it over `dir`, as
/// install.sh writes its LaunchAgent or systemd unit.
fn kept_by_login(home: &Path, dir: &Path) {
    #[cfg(target_os = "macos")]
    let unit = home.join("Library/LaunchAgents/sh.svrnme.svrngs.plist");
    #[cfg(not(target_os = "macos"))]
    let unit = home.join(".config/systemd/user/svrngs.service");
    std::fs::create_dir_all(unit.parent().unwrap()).unwrap();
    #[cfg(target_os = "macos")]
    plist::to_file_xml(
        &unit,
        &serde_json::json!({"EnvironmentVariables": {"SVRNGS_DIR": dir}}),
    )
    .unwrap();
    #[cfg(not(target_os = "macos"))]
    std::fs::write(
        unit,
        format!("[Service]\nEnvironment=SVRNGS_DIR={}\n", dir.display()),
    )
    .unwrap();
}

/// **An older svrngs is restarted once into this one, and then asked**
/// (the operator, 2026-11-21: "Restart automatically"): after an update
/// put a new svrngs in place and the old one runs on, holding its data dir,
/// as pacman leaves it until `svrngs restart`, `svrngs open`, as the
/// applications menu runs it, is answered 404 at `POST /v1/proof` by the
/// old one (1.13.2's answer), sends it none of the token, restarts svrngs
/// as `svrngs restart` does, once, as the login service runs it over this
/// data dir, and once the new svrngs proves itself, opens the home page,
/// saying that alone. Here the old one is Python answering 404 and holding
/// the data dir's lock, and the restart stops it and starts this svrngs
/// over the data dir, at its address. Planted, watched failing: dd2efe63
/// refused, "the svrngs running here is older than this command: run
/// `svrngs restart`", and asked for no restart.
#[test]
fn an_older_svrngs_is_restarted_once_then_asked() {
    let (dir, stubs) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let lock = dir.path().join("lock");
    let (mut old, port, mut sent) = not_svrngs_holding(&[], OLDER, Some(&lock));
    kept_at(dir.path(), port);
    kept_by_login(dir.path(), dir.path());
    let svrngs = as_main::installed(stubs.path());
    let (pid, log) = (stubs.path().join("pid"), stubs.path().join("log"));
    // Python stopped, and gone or a zombie, its socket closed either way;
    // then this svrngs started over the data dir, at its address.
    let then = format!(
        "/bin/kill {old}; while /bin/ps -o stat= -p {old} | /usr/bin/grep -qv Z; do /bin/sleep 0.05; done; SVRNGS_DIR='{dir}' HOME='{dir}' '{svrngs}' >'{log}' 2>&1 & echo $! >'{pid}'",
        old = old.id(),
        dir = dir.path().display(),
        svrngs = svrngs.display(),
        log = log.display(),
        pid = pid.display()
    );
    restarting(stubs.path(), &then);
    let (out, asked) = restarted(dir.path(), stubs.path());
    let _ = old.kill();
    let mut seen = String::new();
    sent.read_to_string(&mut seen).unwrap();
    let started = std::fs::read_to_string(&pid).unwrap_or_default();
    let _ = Process::new("/bin/kill").arg(started.trim()).status();
    sent_none_of_the_token(&seen);
    assert!(seen.contains("POST /v1/proof HTTP/1.1"), "{seen}");
    assert_eq!(
        asked,
        [restart_asked()],
        "one restart, as `svrngs restart` asks it"
    );
    let (said, told) = (
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(out.status.success(), "{said}\n{told}");
    assert!(said.contains("svrngs-open?code="), "{said}");
    assert!(told.trim().is_empty(), "{told}");
}

/// **What answers 404 where no svrngs holds the data dir is not svrngs,
/// and svrngs is not restarted for it** (the 1.13.3 proof review, finding
/// 3): Python holding svrngs's address answers 404 at `POST /v1/proof`, as
/// most programs would; the data dir's lock is free, so no svrngs runs
/// over it, and the command restarts nothing, sends none of the token and
/// says something other than svrngs answers there (exit 1). Planted,
/// watched failing: 4ece5ae5 restarted svrngs for it.
#[test]
fn a_program_answering_404_is_sent_none_of_the_token_and_restarts_nothing() {
    let (dir, stubs) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (mut squatter, port, mut sent) = not_svrngs(&[], OLDER);
    kept_at(dir.path(), port);
    kept_by_login(dir.path(), dir.path());
    restarting(stubs.path(), "");
    let (out, asked) = restarted(dir.path(), stubs.path());
    squatter.kill().unwrap();
    let mut seen = String::new();
    sent.read_to_string(&mut seen).unwrap();
    sent_none_of_the_token(&seen);
    assert!(seen.contains("POST /v1/proof HTTP/1.1"), "{seen}");
    assert_eq!(asked, Vec::<String>::new(), "nothing restarted");
    let told = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{told}");
    unproven(port, (1, told.trim().trim_start_matches("svrngs: ").into()));
}

/// **An older svrngs started by hand, too old to be asked to stop, is
/// said with the one step that stops it** (the 1.13.3 proof review,
/// finding 9): Python answering 404 holds this data dir's lock, as an
/// older svrngs started by hand over it would, and the login service runs
/// svrngs over another; restarting that would restart someone else's
/// svrngs, and svrngs signals no program itself (KEEP_IT_RUNNING_RELEASE
/// N1), so the command restarts nothing, sends none of the token, and says
/// the svrngs here didn't stop when asked and the step that stops it, once
/// (exit 1): with the token beside its address, and with none, as a svrngs
/// from before the token (v1.3) leaves its data dir. Planted, watched
/// failing: 4ece5ae5 restarted the login service's svrngs; 0ba18ef9 said
/// "cannot read or keep this computer's token" with no token, and "stop
/// that svrngs where it runs (Ctrl-C)" with one.
#[test]
fn an_older_svrngs_started_by_hand_is_named_and_how_to_stop_it_said() {
    for token in [true, false] {
        let (dir, stubs, other) = (
            tempfile::tempdir().unwrap(),
            tempfile::tempdir().unwrap(),
            tempfile::tempdir().unwrap(),
        );
        let lock = dir.path().join("lock");
        let (mut old, port, mut sent) = not_svrngs_holding(&[], OLDER, Some(&lock));
        kept_at(dir.path(), port);
        if !token {
            std::fs::remove_file(dir.path().join("token")).unwrap();
        }
        kept_by_login(dir.path(), other.path());
        restarting(stubs.path(), "");
        let (out, asked) = restarted(dir.path(), stubs.path());
        old.kill().unwrap();
        let mut seen = String::new();
        sent.read_to_string(&mut seen).unwrap();
        sent_none_of_the_token(&seen);
        assert_eq!(asked, Vec::<String>::new(), "nothing restarted");
        let told = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{told}");
        let words = format!("svrngs: The svrngs running here didn't stop when asked, as one older than this one ({}) can't: stop it with `kill $(lsof -t {})`, then run this again", svrngs_node::api::build(), lock.display());
        assert_eq!(told.trim(), words, "token: {token}");
    }
}

/// **A data dir whose lock svrngs can't look at restarts nothing** (the
/// never-send-the-key review, finding 4): what answers 404 at svrngs's
/// address, the login service running svrngs over this data dir, and its
/// `lock` a folder, which no lock can be taken of: svrngs can't tell
/// whether an older svrngs holds the data dir, so it restarts nothing,
/// sends none of the token, and says so (exit 3). Planted, watched failing:
/// 96ea3da7 read any failure to take the lock as an older svrngs holding
/// it, and asked for a restart.
#[test]
fn a_lock_svrngs_cannot_look_at_restarts_nothing() {
    let (dir, stubs) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    std::fs::create_dir(dir.path().join("lock")).unwrap();
    let (mut old, port, mut sent) = not_svrngs(&[], OLDER);
    kept_at(dir.path(), port);
    kept_by_login(dir.path(), dir.path());
    restarting(stubs.path(), "");
    let (out, asked) = restarted(dir.path(), stubs.path());
    old.kill().unwrap();
    let mut seen = String::new();
    sent.read_to_string(&mut seen).unwrap();
    sent_none_of_the_token(&seen);
    assert_eq!(asked, Vec::<String>::new(), "nothing restarted");
    let told = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(3), "{told}");
    let lock = dir.path().join("lock");
    let words = format!(
        "can't tell whether an older svrngs holds {}",
        lock.display()
    );
    assert!(
        told.contains(&words) && told.contains("restarted nothing"),
        "{told}"
    );
}

/// **svrngs starting waits out a command's look at its data dir's lock**
/// (the 1.13.3 proof review, finding 10): a command that hears 404 looks
/// whether a svrngs holds this data dir by taking its lock a moment, so a
/// svrngs starting then finds it held; it looks again a moment on, and
/// takes it. Planted, watched failing: 4ece5ae5's `claim` refused at once,
/// "svrngs is already running on this computer".
#[test]
fn a_start_waits_out_a_moment_s_hold_of_its_data_dir() {
    let dir = tempfile::tempdir().unwrap();
    let looked = std::fs::File::create(dir.path().join("lock")).unwrap();
    looked.try_lock().unwrap();
    let let_go = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(10));
        drop(looked);
    });
    let claimed = cli::claim(dir.path());
    let_go.join().unwrap();
    assert!(
        claimed.is_ok(),
        "{:?}",
        claimed.err().map(|r| r.to_string())
    );
}

/// **A data dir with no token is svrngs not running there, and nothing is
/// dialled** (the 1.13.3 proof review, finding 4): with no `token` beside
/// `api` and no svrngs holding the data dir, `svrngs ask` asks nothing
/// of what answers at svrngs's address, and says svrngs isn't running here
/// (exit 3); never that something other than svrngs answers there, nor a
/// file's error. (`open` starts svrngs then:
/// open_starts_svrngs_where_none_runs_and_opens_it.) Planted, watched failing: dd2efe63 asked Python for a
/// proof under no token and said "Something other than your svrngs
/// answers"; 0ba18ef9 said "cannot read or keep this computer's token".
#[tokio::test(flavor = "multi_thread")]
async fn a_data_dir_with_no_token_is_svrngs_not_running_and_nothing_is_asked() {
    let held = TcpListener::bind("127.0.0.1:0").unwrap();
    held.set_nonblocking(true).unwrap();
    let dir = data_dir_at(held.local_addr().unwrap().port());
    std::fs::remove_file(dir.path().join("token")).unwrap();
    let verb = cli::Verb::Ask("/v1/groups".into(), None);
    let refused = cli::answer(verb, &dir.path().to_path_buf(), &no_relay()).await;
    let asked = held.accept().map(|(_, from)| from);
    assert!(asked.is_err(), "it was asked, from {asked:?}");
    let refused = refused.expect_err("refused");
    let said = refused.to_string();
    assert_eq!(refused.exit_code(), 3, "{said}");
    assert!(
        said.contains("svrngs is not running on this computer"),
        "{said}"
    );
}

/// **`svrngs open` starts svrngs where none runs, and opens it**, as the
/// applications menu runs it after a restart of the computer: nothing
/// keeps svrngs at login, no svrngs holds the data dir and it holds no
/// token yet, so `open` starts this program over it apart from its
/// terminal, waits for it to answer as this build, and gives the open
/// alone, one line. Planted, watched failing: 0ba18ef9 said "svrngs is not
/// running on this computer: start it, then run this again" (exit 3).
#[test]
fn open_starts_svrngs_where_none_runs_and_opens_it() {
    let (dir, program) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let svrngs = as_main::installed(program.path());
    let out = Process::new(&svrngs)
        .args(["open", "--print"])
        .env("SVRNGS_DIR", dir.path())
        .env("HOME", dir.path())
        .output()
        .unwrap();
    let holder = holding(dir.path());
    let _ = Process::new("kill").arg(&holder).status();
    let said = String::from_utf8_lossy(&out.stdout);
    let said = said.lines().filter(|l| !as_main::HEADER.contains(l));
    let said: Vec<&str> = said.collect();
    assert!(out.status.success(), "{said:?} {out:?}");
    assert!(!holder.is_empty(), "no svrngs holds the data dir");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(
        said[0].starts_with("Open this in your browser within a minute: http://svrngs.localhost:"),
        "{said:?}"
    );
}

/// The pid of the program holding `dir`'s lock, or none (`lsof`).
fn holding(dir: &Path) -> String {
    let lsof = Process::new("lsof")
        .arg("-t")
        .arg(dir.join("lock"))
        .output();
    String::from_utf8_lossy(&lsof.unwrap().stdout)
        .trim()
        .to_string()
}

/// **`svrngs restart` stops a svrngs nothing keeps at login, and starts
/// this one in its place**: svrngs started by hand over a data dir, no
/// login service keeping it, answers; `svrngs restart` asks it, as you, to
/// stop (surface.rs `STOP_ROUTE`), which it does as on Ctrl-C, exit 0, its
/// lock let go; waits for the lock to be free, starts this program over the
/// data dir apart from its terminal, and once it answers says one line,
/// `svrngs <build> is running.`. No signal is sent but the test's own, to
/// end the second. Planted, watched failing: the stop route served without
/// the token (record_api.rs, every record route); the restart starting
/// svrngs without waiting for the lock.
#[test]
fn restart_stops_svrngs_started_by_hand_and_starts_this_one() {
    let (dir, program) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let svrngs = as_main::installed(program.path());
    let here = |args: &[&str]| {
        let mut c = Process::new(&svrngs);
        c.args(args)
            .env("SVRNGS_DIR", dir.path())
            .env("HOME", dir.path());
        c.stdin(Stdio::null())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());
        c
    };
    let mut first = here(&[]).spawn().unwrap();
    let began = Instant::now();
    while !here(&["ask", "/v1/groups"]).status().unwrap().success() {
        assert!(began.elapsed() < Duration::from_secs(30), "never answered");
        std::thread::sleep(Duration::from_millis(100));
    }
    let restarted = here(&["restart"]).output().unwrap();
    let (stopped, holder) = (first.wait().unwrap(), holding(dir.path()));
    let _ = Process::new("kill").arg(&holder).status();
    let said = String::from_utf8_lossy(&restarted.stdout);
    let said: Vec<&str> = (said.lines().filter(|l| !as_main::HEADER.contains(l))).collect();
    let running = format!("svrngs {} is running.", svrngs_node::api::build());
    assert!(restarted.status.success(), "{said:?} {restarted:?}");
    assert_eq!(said, [running.as_str()], "{restarted:?}");
    assert!(
        stopped.success(),
        "the first stopped as on Ctrl-C: {stopped:?}"
    );
    let first = first.id().to_string();
    assert!(
        !holder.is_empty() && holder != first,
        "{holder} holds it, not {first}"
    );
}

/// **A token svrngs can't read is said with the step that makes a new
/// one, and nothing is dialled**: `token` a folder, as no svrngs writes
/// it, so reading it fails for a reason other than its absence; the
/// command asks nothing at svrngs's address, and says which file, why, and
/// to delete it and run `svrngs restart` (exit 3). Planted, watched
/// failing: 0ba18ef9's words, "cannot read or keep this computer's token",
/// and no step.
#[tokio::test(flavor = "multi_thread")]
async fn an_unreadable_token_is_said_with_the_step_that_makes_a_new_one() {
    let held = TcpListener::bind("127.0.0.1:0").unwrap();
    held.set_nonblocking(true).unwrap();
    let dir = data_dir_at(held.local_addr().unwrap().port());
    let token = dir.path().join("token");
    std::fs::remove_file(&token).unwrap();
    std::fs::create_dir(&token).unwrap();
    let refused = opened(dir.path()).await;
    let asked = held.accept().map(|(_, from)| from);
    assert!(asked.is_err(), "it was asked, from {asked:?}");
    let (code, said) = refused.expect_err("refused");
    assert_eq!(code, 3, "{said}");
    let words = format!(
        "svrngs can't read or write its token, {} (",
        token.display()
    );
    assert!(said.starts_with(&words), "{said}");
    let step = "). svrngs makes a new one where there is none: delete that file, then run `svrngs restart`";
    assert!(said.ends_with(step), "{said}");
}

/// **What answers at svrngs's address is read no further than an answer
/// svrngs gives the command** (the 1.13.3 proof review, finding 7): Python
/// answering the proof with 64 MiB stops being read past the command's
/// bound, and the command says something other than svrngs answers.
/// Planted, watched failing: dd2efe63 read all 64 MiB.
#[tokio::test(flavor = "multi_thread")]
async fn an_endless_answer_is_read_no_further_than_svrngs_s_own() {
    let script = "import socket\ns = socket.socket()\ns.bind(('127.0.0.1', 0))\ns.listen()\nprint(s.getsockname()[1], flush=True)\nc, _ = s.accept()\nc.recv(65536)\nsent = 0\ntry:\n    c.sendall(b'HTTP/1.1 200 OK\\r\\nContent-Length: 67108864\\r\\n\\r\\n')\n    while sent < 67108864:\n        sent += c.send(bytes(65536))\nexcept OSError: pass\nprint(sent, flush=True)";
    let mut python = Process::new("python3")
        .args(["-I", "-c", script])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut out = BufReader::new(python.stdout.take().unwrap());
    let mut port = String::new();
    out.read_line(&mut port).unwrap();
    let dir = data_dir_at(port.trim().parse().unwrap());
    let refused = opened(dir.path()).await;
    let mut sent = String::new();
    out.read_line(&mut sent).unwrap();
    let _ = python.kill();
    let _ = python.wait();
    let sent: usize = sent.trim().parse().unwrap();
    assert!(sent < 16 << 20, "the command read {sent} bytes of it");
    unproven(port.trim().parse().unwrap(), refused.expect_err("refused"));
}

/// **A connection svrngs's address doesn't take is waited for no longer
/// than a proof** (the 1.13.3 proof review, finding 12): on Linux a
/// listener whose queue is full drops each new connection's handshake, so
/// a connect waits on it some two minutes; the command gives up within
/// its proof's wait, 10 s, and says nothing proved itself (exit 3). A Mac
/// takes the connection, so this is Linux's. Planted, watched failing:
/// dd2efe63's connect had no bound.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread")]
async fn a_connection_nothing_takes_is_waited_for_no_longer_than_a_proof() {
    let script = "import socket, time\ns = socket.socket()\ns.bind(('127.0.0.1', 0))\ns.listen(0)\nprint(s.getsockname()[1], flush=True)\ntime.sleep(60)";
    let mut python = Process::new("python3")
        .args(["-I", "-c", script])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut port = String::new();
    BufReader::new(python.stdout.take().unwrap())
        .read_line(&mut port)
        .unwrap();
    let port: u16 = port.trim().parse().unwrap();
    let at = SocketAddr::from(([127, 0, 0, 1], port));
    let filled: Vec<_> = (0..8)
        .filter_map(|_| TcpStream::connect_timeout(&at, Duration::from_millis(200)).ok())
        .collect();
    let dir = data_dir_at(port);
    let asked = tokio::time::timeout(Duration::from_secs(30), opened(dir.path())).await;
    let _ = python.kill();
    let _ = python.wait();
    drop(filled);
    let (code, said) = asked
        .expect("the command gave up within 30 s")
        .expect_err("refused");
    assert_eq!(code, 3, "{said}");
    assert!(
        said.contains("proved it is your svrngs within 10 seconds"),
        "{said}"
    );
}
