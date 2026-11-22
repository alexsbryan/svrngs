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

/// svrngs itself, as [`Program`] runs it (common/as_main.rs). A second bare
/// command, which starts no node, is the shipped binary's; `open` is this
/// one, as the svrngs it asks is, since it shows the token only to svrngs
/// itself (svrngs-node uid.rs `itself`).
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
    // The program the running svrngs is ([`svrngs_here`]): `open` sends
    // its ask only to svrngs itself, the same executable, as this account.
    let invoke = || {
        let mut out = as_main::svrngs()
            .arg("open")
            .env("SVRNGS_DIR", dir.path())
            .env("HOME", dir.path())
            .env("PATH", logs.path())
            .output()
            .unwrap();
        let mut said = &out.stdout[..];
        as_main::past_header(&mut said);
        out.stdout = said.to_vec();
        out
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
/// false: no `Sec-Fetch-*`, which a lend answers as its own), showing the
/// record `token` if any: its status line and body.
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
    let token = token.map_or(String::new(), |t| format!("Authorization: Bearer {t}\r\n"));
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
/// API in `dir` answers a program showing `token`.
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
    cli::share_lend(dir.path(), &lend, None).unwrap();

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
        // As a program the person runs, showing the token: the address is
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

/// A program on 127.0.0.1 that is not svrngs, run as `argv` before Python:
/// it says its port, then for each connection writes how many bytes it was
/// sent and what, and answers as svrngs's `/v1/open` would.
fn not_svrngs(before: &[&str]) -> (Child, u16, BufReader<std::process::ChildStdout>) {
    let script = "import socket\ns = socket.socket()\ns.bind(('127.0.0.1', 0))\ns.listen()\nprint(s.getsockname()[1], flush=True)\nwhile True:\n    c, _ = s.accept()\n    c.settimeout(2)\n    got = b''\n    try:\n        while b'\\r\\n\\r\\n' not in got:\n            b = c.recv(65536)\n            if not b: break\n            got += b\n    except OSError: pass\n    print(len(got), got, flush=True)\n    c.sendall(b'HTTP/1.1 200 OK\\r\\nContent-Length: 18\\r\\n\\r\\n{\"open\": \"http://\"}')\n    c.close()";
    let (program, rest) = before.split_first().unwrap_or((&"python3", &[]));
    let mut child = Process::new(program)
        .args(rest)
        .args(if before.is_empty() {
            &[][..]
        } else {
            &["python3"][..]
        })
        .args(["-I", "-c", script])
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

/// A data dir whose `api` names 127.0.0.1:`port` and whose token is known.
fn data_dir_at(port: u16) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("api"), format!("http://127.0.0.1:{port}\n")).unwrap();
    std::fs::write(dir.path().join("token"), "the-token-svrngs-keeps\n").unwrap();
    dir
}

/// `svrngs open --print` over `dir`, as the applications menu's opener runs
/// it, but with no browser: its words, or its refusal's, and exit code.
async fn opened(dir: &Path) -> Result<String, (u8, String)> {
    let relay = svrngs_node::iroh::RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: true,
    };
    let said = cli::answer(cli::Verb::Open(None, true), &dir.to_path_buf(), &relay).await;
    said.map_err(|r| (r.exit_code(), r.to_string()))
}

/// **Only svrngs itself, as you, is shown your token** (V1.13.3's second
/// practice review, finding 1): while svrngs is down, a program that isn't
/// svrngs holds its address — Python, a program of yours — and `svrngs
/// open`, as the applications menu and install.sh run it, sends it
/// nothing, not one byte, and says what answers there and what to do
/// (exit 1); svrngs's own program at that address, this test's, which runs
/// svrngs's code here, is sent the ask with the token. Planted, watched
/// failing: V1.13.3's candidate sent Python `POST /v1/open` with
/// `authorization: Bearer the-token-svrngs-keeps`.
#[tokio::test(flavor = "multi_thread")]
async fn only_svrngs_itself_as_you_is_shown_the_token() {
    let (mut python, port, mut sent) = not_svrngs(&[]);
    let dir = data_dir_at(port);
    let refused = opened(dir.path()).await.expect_err("refused");
    python.kill().unwrap();
    let mut seen = String::new();
    sent.read_to_string(&mut seen).unwrap();
    assert!(
        seen.lines().all(|l| l.starts_with("0 ")),
        "Python was sent: {seen}"
    );
    assert_eq!(refused.0, 1, "{}", refused.1);
    let at = format!("http://127.0.0.1:{port}");
    assert!(
        refused.1.contains("a program of yours that isn't "),
        "{}",
        refused.1
    );
    assert!(
        refused.1.contains(&format!(
            "answers at {at}, where svrngs listens, so svrngs sent it nothing"
        )),
        "{}",
        refused.1
    );
    assert!(refused.1.contains("svrngs restart"), "{}", refused.1);

    let ours = TcpListener::bind("127.0.0.1:0").unwrap();
    let dir = data_dir_at(ours.local_addr().unwrap().port());
    let asked = std::thread::spawn(move || {
        let (mut c, _) = ours.accept().unwrap();
        let mut head = vec![0; 4096];
        let n = c.read(&mut head).unwrap();
        let body = r#"{"open": "http://svrngs.localhost:1/svrngs-open?code=c"}"#;
        let answer = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        c.write_all(answer.as_bytes()).unwrap();
        String::from_utf8_lossy(&head[..n]).into_owned()
    });
    let said = opened(dir.path())
        .await
        .unwrap_or_else(|(_, r)| panic!("{r}"));
    assert!(said.contains("svrngs-open?code=c"), "{said}");
    let head = asked.join().unwrap();
    assert!(
        head.contains("authorization: Bearer the-token-svrngs-keeps"),
        "{head}"
    );
}

/// **Another account's program at svrngs's address is sent nothing**, on a
/// real socket of another uid: a uid of this account's own subordinate
/// range (/etc/subuid), as `unshare --map-auto` gives it without root,
/// read by sock_diag. By hand on Linux: it needs newuidmap and a range in
/// /etc/subuid. Planted, watched failing: V1.13.3's candidate sent it `POST
/// /v1/open` with the token.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "by hand on Linux: unshare --map-auto and a range in /etc/subuid"]
async fn another_account_s_program_at_svrngs_s_address_is_sent_nothing() {
    let theirs = [
        "unshare",
        "--map-auto",
        "--map-root-user",
        "setpriv",
        "--reuid=1",
        "--regid=1",
        "--clear-groups",
    ];
    let (mut python, port, mut sent) = not_svrngs(&theirs);
    let dir = data_dir_at(port);
    let refused = opened(dir.path()).await.expect_err("refused");
    python.kill().unwrap();
    let mut seen = String::new();
    sent.read_to_string(&mut seen).unwrap();
    assert!(
        seen.lines().all(|l| l.starts_with("0 ")),
        "their program was sent: {seen}"
    );
    assert_eq!(refused.0, 1, "{}", refused.1);
    assert!(
        refused.1.starts_with("A program of the account "),
        "{}",
        refused.1
    );
    eprintln!("RESULT: {}", refused.1);
}
