//! A browser half run, and the JSON line it printed: the browser tests'
//! own, and the demo scenes' (crates/svrngs/tests/scenes), which include
//! this file by path.

use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;

use crate::harness::{browsers, repo, Engine};

/// Each computer a browser half may be opened at: its record API's port
/// and its token, the latest for a port first (give-session).
static COMPUTERS: std::sync::Mutex<Vec<(u16, String)>> = std::sync::Mutex::new(Vec::new());

/// The computer whose record API is at `port`, holding `token`: a browser
/// half whose args name it is opened there, as the person's browser is once
/// svrngs opened it ([`drive`]).
pub fn opens_at(port: u16, token: &str) {
    COMPUTERS
        .lock()
        .unwrap()
        .insert(0, (port, token.trim().into()));
}

/// An address that opens the host first named `name` (`svrngs`, or a
/// lend's label) on the computer at `port`, at the path `at`, minted by
/// its svrngs with its token as `svrngs open` asks it (give-session); none
/// for a computer [`opens_at`] never named.
#[allow(dead_code)]
pub fn open_at(port: u16, name: &str, at: &str) -> Option<String> {
    use std::io::{Read, Write};
    let computers = COMPUTERS.lock().unwrap().clone();
    let (_, token) = computers.into_iter().find(|(p, _)| *p == port)?;
    let body = if name == "svrngs" {
        json!({ "at": at })
    } else {
        json!({ "lend": name, "at": at })
    }
    .to_string();
    let mut sock = std::net::TcpStream::connect(("127.0.0.1", port)).ok()?;
    let ask = format!("POST /v1/open HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
    sock.write_all(ask.as_bytes()).ok()?;
    let mut reply = String::new();
    sock.read_to_string(&mut reply).ok()?;
    let answer: Value = serde_json::from_str(reply.split_once("\r\n\r\n")?.1).ok()?;
    Some(answer["open"].as_str()?.to_string())
}

/// What svrngs opens for a half run on `args`: each computer [`opens_at`]
/// named whose port they name, with its home page and each lend's host they
/// name there, as `[{port, token, names}]` (web/tests/page.mjs `launch`).
pub fn opens(args: &Value) -> Value {
    let said = args.to_string();
    let mut seen = std::collections::BTreeSet::new();
    let computers = COMPUTERS.lock().unwrap().clone();
    let named = computers.into_iter().filter_map(|(port, token)| {
        let at = format!(":{port}");
        let ends = |i: usize| !said[i + at.len()..].starts_with(|c: char| c.is_ascii_digit());
        let mut names = vec!["svrngs".to_string()];
        let hosts = said.match_indices(&at).filter(|(i, _)| ends(*i));
        let hosts: Vec<usize> = hosts.map(|(i, _)| i).collect();
        for i in &hosts {
            let Some(host) = said[..*i].strip_suffix(".localhost") else {
                continue;
            };
            let from = host
                .rfind(|c: char| !c.is_ascii_alphanumeric())
                .map_or(0, |b| b + 1);
            (!names.iter().any(|n| n == &host[from..])).then(|| names.push(host[from..].into()));
        }
        (!hosts.is_empty() && seen.insert(port))
            .then(|| json!({ "port": port, "token": token, "names": names }))
    });
    Value::Array(named.collect())
}

/// Run the browser half `web/tests/<script>` in `engine` on `args`, and
/// read the JSON line it printed. When `release` fires, the browser half is told `go`
/// (its upload's tail, u4-4).
pub async fn drive(
    engine: Engine,
    script: &str,
    mut args: Value,
    release: Option<tokio::sync::oneshot::Receiver<()>>,
) -> Value {
    args["engine"] = json!(engine.as_str());
    let mut child = tokio::process::Command::new("node")
        .arg(repo().join("web/tests").join(script))
        .arg(args.to_string())
        .env("SVRNGS_BROWSERS", browsers())
        .env("SVRNGS_OPENS", opens(&args).to_string())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs: the browser tests need Node 18 or later");
    let mut stdin = child.stdin.take().expect("stdin is piped");
    let told = tokio::spawn(async move {
        if let Some(Ok(())) = match release {
            Some(release) => Some(release.await),
            None => None,
        } {
            let _ = stdin.write_all(b"go\n").await;
            let _ = stdin.flush().await;
        }
        // Held open until the browser half is done: it reads stdin.
        std::future::pending::<()>().await;
    });
    let out = child
        .wait_with_output()
        .await
        .expect("the browser half ran");
    told.abort();
    assert!(
        out.status.success(),
        "the browser half failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.lines().last().expect("the browser half printed");
    serde_json::from_str(line).expect("the browser half printed JSON")
}
