//! Read Linux listeners and ask a known program what model it is serving.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::promote::{Listen, listens};

/// The sockets on `port`, including the kernel's socket cgroup witness.
pub fn ss(port: u16) -> Result<Vec<Listen>, String> {
    let out = std::process::Command::new("/usr/bin/ss")
        .args(["-ltnpH", "--cgroup", "sport", "=", &format!(":{port}")])
        .output()
        .map_err(|e| format!("ss: {e}"))?;
    if !out.status.success() {
        return Err(format!("ss failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let listeners = listens(&String::from_utf8_lossy(&out.stdout), port);
    tracing::debug!(port, ?listeners, "host:listeners");
    Ok(listeners)
}

pub fn busy(port: u16) -> Result<Option<String>, String> {
    Ok(ss(port)?.first().map(|s| {
        s.pids
            .first()
            .and_then(|pid| std::fs::read_to_string(format!("/proc/{pid}/comm")).ok())
            .map(|name| name.trim().chars().filter(|c| !c.is_control()).collect::<String>())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "another program".into())
    }))
}

/// Recognize llama-server, and resolve a relative model path in its folder.
pub fn llama_folder(pid: u32) -> Result<PathBuf, String> {
    let link = |file| {
        let path = format!("/proc/{pid}/{file}");
        std::fs::read_link(&path).map_err(|e| format!("{path}: {e}"))
    };
    if link("exe")?.file_name().and_then(|name| name.to_str()) != Some("llama-server") {
        return Err("no known program is listening".into());
    }
    link("cwd")
}

/// `GET /props` over HTTP/1.0. Bound the reply's size and the entire read.
#[expect(
    clippy::disallowed_methods,
    reason = "the local prefill probe sets its deadline at the edge"
)]
pub fn props(addr: &str, port: u16) -> Result<String, String> {
    let (status, body) = http(addr, port, "/props", true, Instant::now() + Duration::from_secs(3))?;
    if status != 200 {
        return Err(format!("/props did not answer 200 ({status})"));
    }
    crate::promote::props(&body)
}

#[expect(
    clippy::disallowed_methods,
    reason = "dial: local prefill or readiness, without share credentials"
)]
fn http(addr: &str, port: u16, path: &str, body: bool, end: Instant) -> Result<(u16, String), String> {
    let at = format!("{addr}:{port}");
    let sock = at.parse().map_err(|e| format!("{at}: {e}"))?;
    let left = || {
        end.saturating_duration_since(Instant::now())
            .max(Duration::from_micros(1))
    };
    let mut s =
        TcpStream::connect_timeout(&sock, left().min(Duration::from_secs(1))).map_err(|e| format!("{at}: {e}"))?;
    s.set_write_timeout(Some(left())).map_err(|e| e.to_string())?;
    write!(s, "GET {path} HTTP/1.0\r\nHost: {at}\r\n\r\n").map_err(|e| format!("{at}: {e}"))?;
    let mut got = Vec::new();
    let mut buf = [0; 8192];
    loop {
        let left = end.saturating_duration_since(Instant::now());
        if left.is_zero() || got.len() >= 1_048_576 {
            return Err(format!("{at} did not finish a bounded HTTP reply"));
        }
        s.set_read_timeout(Some(left)).map_err(|e| e.to_string())?;
        let n = s.read(&mut buf).map_err(|e| format!("{at}: {e}"))?;
        got.extend_from_slice(&buf[..n]);
        if n == 0 || (!body && got.windows(2).any(|w| w == b"\r\n")) {
            break;
        }
    }
    let got = String::from_utf8_lossy(&got);
    let (head, rest) = got.split_once("\r\n").ok_or("did not answer HTTP")?;
    let status = head
        .strip_prefix("HTTP/")
        .and_then(|h| h.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|s| (200..600).contains(s))
        .ok_or("did not answer HTTP")?;
    let text = if body {
        rest.split_once("\r\n\r\n")
            .map(|(_, b)| b)
            .or_else(|| rest.strip_prefix("\r\n"))
            .ok_or("did not finish its HTTP headers")?
    } else {
        ""
    };
    Ok((status, text.to_owned()))
}

/// Wait for an HTTP answer, not merely an open socket. A known model is
/// ready when its health endpoint no longer says it is loading.
#[expect(clippy::disallowed_methods, reason = "the local readiness probe waits at the edge")]
pub fn wait_for_answer(port: u16, llama: bool, within: Duration) -> bool {
    let end = Instant::now() + within;
    let path = if llama { "/health" } else { "/" };
    while Instant::now() < end {
        for addr in ["127.0.0.1", "[::1]"] {
            let answer = http(
                addr,
                port,
                path,
                false,
                end.min(Instant::now() + Duration::from_secs(1)),
            );
            tracing::debug!(port, addr, path, ?answer, "run: readiness answer");
            if answer.is_ok_and(|(status, _)| !llama || status == 200) {
                return true;
            }
        }
        std::thread::sleep(
            end.saturating_duration_since(Instant::now())
                .min(Duration::from_millis(250)),
        );
    }
    false
}
