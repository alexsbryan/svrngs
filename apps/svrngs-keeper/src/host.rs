//! Read Linux listeners and ask a known program what model it is serving.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::promote::{Listen, listens};

/// The sockets on `port`, from `ss -ltnpH sport = :<port>`.
pub fn ss(port: u16) -> Result<Vec<Listen>, String> {
    let out = std::process::Command::new("ss")
        .args(["-ltnpH", "sport", "=", &format!(":{port}")])
        .output()
        .map_err(|e| format!("ss: {e}"))?;
    if !out.status.success() {
        return Err(format!("ss failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let listeners = listens(&String::from_utf8_lossy(&out.stdout), port);
    tracing::debug!(port, ?listeners, "host:listeners");
    Ok(listeners)
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
    reason = "dial: the local llama-server's read-only /props prefill, without share credentials"
)]
pub fn props(addr: &str, port: u16) -> Result<String, String> {
    let at = format!("{addr}:{port}");
    let sock = at.parse().map_err(|e| format!("{at}: {e}"))?;
    let mut s = TcpStream::connect_timeout(&sock, Duration::from_secs(1)).map_err(|e| format!("{at}: {e}"))?;
    let end = Instant::now() + Duration::from_secs(3);
    s.set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;
    write!(s, "GET /props HTTP/1.0\r\nHost: {at}\r\n\r\n").map_err(|e| format!("{at}: {e}"))?;
    let mut got = Vec::new();
    let mut buf = [0; 8192];
    loop {
        let left = end.saturating_duration_since(Instant::now());
        if left.is_zero() || got.len() >= 1_048_576 {
            return Err(format!("{at} did not finish a bounded HTTP reply"));
        }
        s.set_read_timeout(Some(left)).map_err(|e| e.to_string())?;
        let n = s.read(&mut buf).map_err(|e| format!("{at}: {e}"))?;
        if n == 0 {
            break;
        }
        got.extend_from_slice(&buf[..n]);
    }
    let got = String::from_utf8_lossy(&got);
    let (head, body) = got.split_once("\r\n\r\n").ok_or("/props did not answer HTTP")?;
    let status = head.strip_prefix("HTTP/").and_then(|h| h.split_whitespace().nth(1));
    if status != Some("200") {
        return Err(format!("/props did not answer 200 ({status:?})"));
    }
    crate::promote::props(body)
}
