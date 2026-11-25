//! Read local listeners, how this account's listener was started, and
//! whether a started program answers.

use std::io::{Read, Write};
use std::net::TcpStream;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use crate::promote::{Listen, listens};
#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::{busy, listeners, owns, started};
#[cfg(target_os = "linux")]
pub use ss as listeners;

/// The sockets on `port`, including the kernel's socket cgroup witness.
#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
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

/// How this account's process `pid` was started: its arguments, from
/// `/proc/<pid>/cmdline`, and its folder, `/proc/<pid>/cwd`. Its arguments
/// can hold a secret (`--api-key`), so they are returned, never traced.
#[cfg(target_os = "linux")]
pub fn started(pid: u32) -> Result<(Vec<String>, PathBuf), String> {
    use std::os::unix::fs::MetadataExt;
    let at = |file: &str| format!("/proc/{pid}/{file}");
    let uid = |path: &str| {
        std::fs::metadata(path)
            .map(|m| m.uid())
            .map_err(|e| format!("{path}: {e}"))
    };
    if uid(&at(""))? != uid("/proc/self")? {
        tracing::debug!(pid, "host:started another account's");
        return Err(THEIRS.into());
    }
    let argv = crate::promote::cmdline(&std::fs::read(at("cmdline")).map_err(|e| e.to_string())?)?;
    let folder = std::fs::read_link(at("cwd")).map_err(|e| format!("its folder: {e}"))?;
    Ok((argv, folder))
}

/// Why another account's process is not read.
pub const THEIRS: &str = "another account runs it, and svrngs reads only how your own programs were started";

#[expect(
    clippy::disallowed_methods,
    reason = "dial: local readiness, without share credentials"
)]
fn http(addr: &str, port: u16, path: &str, end: Instant) -> Result<u16, String> {
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
        if n == 0 || got.windows(2).any(|w| w == b"\r\n") {
            break;
        }
    }
    let got = String::from_utf8_lossy(&got);
    let (head, _) = got.split_once("\r\n").ok_or("did not answer HTTP")?;
    head.strip_prefix("HTTP/")
        .and_then(|h| h.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|s| (200..600).contains(s))
        .ok_or_else(|| "did not answer HTTP".into())
}

/// Wait for an HTTP answer, not merely an open socket. A known model is
/// ready when its health endpoint no longer says it is loading.
#[expect(clippy::disallowed_methods, reason = "the local readiness probe waits at the edge")]
pub fn wait_for_answer(port: u16, llama: bool, within: Duration) -> bool {
    let end = Instant::now() + within;
    let path = if llama { "/health" } else { "/" };
    while Instant::now() < end {
        for addr in ["127.0.0.1", "[::1]"] {
            let answer = http(addr, port, path, end.min(Instant::now() + Duration::from_secs(1)));
            tracing::debug!(port, addr, path, ?answer, "run: readiness answer");
            if answer.is_ok_and(|status| !llama || status == 200) {
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
