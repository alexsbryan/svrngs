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
use svrngs_node::lend::Kind;

/// The program, its output read line by line as it comes.
struct Program {
    child: Child,
    lines: std::sync::mpsc::Receiver<String>,
    seen: Vec<String>,
}

impl Program {
    /// `svrngs <args>` over `dir`, its home `dir` too (a group's folder
    /// is made under it), tracing everything to `log`.
    fn start(dir: &Path, log: &Path, args: &[&str]) -> Self {
        let mut child = Process::new(env!("CARGO_BIN_EXE_svrngs"))
            .args(args)
            .env("SVRNGS_DIR", dir)
            .env("HOME", dir)
            .env("RUST_LOG", "debug")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(std::fs::File::create(log).unwrap())
            .spawn()
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
        kind: Some(Kind::parse("pad").unwrap()),
        limit: None,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        asks: None,
        sign_in: false,
    };
    let lend = share::lend_of(&asked, share::listens, None).expect("pad listens");
    cli::share_lend(dir.path(), &lend, None).unwrap();

    let new = ["new", "Lab", "--as", "Sam"];
    let mut made = Program::start(dir.path(), &logs.path().join("new"), &new);
    made.until("Leave this running");
    drop(made);

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
        let (status, page) = get(&home, "/", true, None);
        assert!(status.ends_with("200 OK"), "{run}: {home} {status}");
        assert!(
            page.contains("/home.js"),
            "{run}: {home} is not the home page"
        );
        let (status, body) = get(&lent, "/", false, None);
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
