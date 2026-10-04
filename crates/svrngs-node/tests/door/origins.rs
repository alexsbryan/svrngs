// SPDX-License-Identifier: AGPL-3.0-or-later
// `picky_origin`: ported from commonwealth-ai commonwealth/crates/commonwealth-rails/tests/two_daemons.rs@e1a43bcd2
// `byte_at`, `parse_range`, `serve_http`: ported from commonwealth-ai commonwealth/crates/commonwealth-transport/examples/media_bridge_bench.rs@e1a43bcd2
//! The services the door's tests lend: upstream's two fixtures (DESIGN §8,
//! U3's inventory, its fixture row), and two of the door's own.
//!
//! Edits to upstream's: `picky_origin` (two_daemons.rs:208-255) kept but for
//! `use` lines and the header it judges, a parameter here; the Range origin
//! (media_bridge_bench.rs:93-100, 211-325) loses its pacing (`Pacing`, the
//! stall and the rate), its 256 KiB `CHUNK` kept, and serves one file from a
//! listener on 127.0.0.1:0 this file binds (`range_origin`), where upstream's
//! `origin` read its port and file from flags.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Read/write granularity. 256 KiB is large enough that the syscall rate
/// is not the ceiling and small enough that a stall is still visible as a
/// gap between arrivals rather than smeared inside one read.
const CHUNK: usize = 256 * 1024;

/// Byte at absolute offset `i`. Position-dependent so ANY range can be
/// verified without holding the file, and so a splice that reorders,
/// duplicates or drops a window is caught at the first wrong byte rather
/// than only in a whole-file digest.
pub fn byte_at(i: u64) -> u8 {
    (i.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
        >> 33) as u8
}

/// An origin that authenticates its OWN clients, like Jellyfin does. Answers
/// 200 only when `X-Emby-Token` carries exactly `token`, 401 otherwise —
/// including when the header is absent, and including when it appears twice
/// with the wrong value first.
///
/// It is the JUDGE of this test. A permissive origin would return 200 whether
/// or not the header arrived, which is exactly the blunt predicate the first
/// draft of this campaign shipped with.
/// (svrngs: the header it judges is `header`, lowercase.)
pub async fn picky_origin(header: &'static str, token: &'static str) -> std::net::SocketAddr {
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => panic!("the picky origin could not bind: {e}"),
    };
    let addr = match listener.local_addr() {
        Ok(a) => a,
        Err(e) => panic!("the picky origin has no local addr: {e}"),
    };
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                let head = String::from_utf8_lossy(&buf[..n]).to_ascii_lowercase();
                let want = format!("{header}: {token}\r\n");
                // EXACTLY ONE token line, and it is the holder's. Two lines
                // would mean the viewer's copy survived beside ours.
                let ok = head.matches(&format!("{header}:")).count() == 1 && head.contains(&want);
                let body = if ok { "OK" } else { "NO" };
                let status = if ok { "200 OK" } else { "401 Unauthorized" };
                let _ = sock
                    .write_all(
                        format!(
                            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        )
                        .as_bytes(),
                    )
                    .await;
                let _ = sock.flush().await;
            });
        }
    });
    addr
}

/// Upstream's Range origin over one file of `total` bytes at `file`, on a
/// port of its own.
pub async fn range_origin(file: std::path::PathBuf, total: u64) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().unwrap();
    let file = Arc::new(file);
    tokio::spawn(async move {
        loop {
            let Ok((sock, _peer)) = listener.accept().await else {
                break;
            };
            let file = Arc::clone(&file);
            tokio::spawn(async move {
                let _ = serve_http(sock, file.to_str().unwrap(), total).await;
            });
        }
    });
    addr
}

/// Parse `Range: bytes=a-b` / `bytes=a-` / `bytes=-n` into an inclusive
/// byte range. `None` for an absent header; `Err` for one that is present
/// and unsatisfiable — those are different answers and the caller must
/// distinguish them (416 vs 200).
fn parse_range(h: &str, total: u64) -> Option<Result<(u64, u64), ()>> {
    let spec = h.strip_prefix("bytes=")?.trim();
    let (a, b) = spec.split_once('-')?;
    let (start, end) = match (a.trim(), b.trim()) {
        ("", "") => return Some(Err(())),
        ("", n) => {
            let n: u64 = n.parse().ok()?;
            if n == 0 || total == 0 {
                return Some(Err(()));
            }
            (total.saturating_sub(n), total - 1)
        }
        (s, "") => {
            let s: u64 = s.parse().ok()?;
            (s, total.saturating_sub(1))
        }
        (s, e) => {
            let s: u64 = s.parse().ok()?;
            let e: u64 = e.parse().ok()?;
            (s, e.min(total.saturating_sub(1)))
        }
    };
    if start > end || start >= total {
        return Some(Err(()));
    }
    Some(Ok((start, end)))
}

async fn serve_http(mut sock: TcpStream, file: &str, total: u64) -> std::io::Result<()> {
    sock.set_nodelay(true).ok();
    // Read request head.
    let mut head = Vec::with_capacity(2048);
    let mut buf = [0u8; 1024];
    loop {
        let n = sock.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        head.extend_from_slice(&buf[..n]);
        if head.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if head.len() > 64 * 1024 {
            return Ok(());
        }
    }
    let text = String::from_utf8_lossy(&head).to_string();
    let range_hdr = text
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("range:"))
        .map(|l| l[6..].trim().to_string());

    let (status, start, len) = match range_hdr.as_deref().and_then(|h| parse_range(h, total)) {
        Some(Ok((s, e))) => ("206 Partial Content", s, e - s + 1),
        Some(Err(())) => {
            let body = format!(
                "HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */{total}\r\n\
                 Content-Length: 0\r\nConnection: close\r\n\r\n"
            );
            sock.write_all(body.as_bytes()).await?;
            return Ok(());
        }
        None => ("200 OK", 0, total),
    };

    let mut hdr = format!(
        "HTTP/1.1 {status}\r\nContent-Type: video/mp4\r\nAccept-Ranges: bytes\r\n\
         Content-Length: {len}\r\n"
    );
    if status.starts_with("206") {
        hdr.push_str(&format!(
            "Content-Range: bytes {}-{}/{}\r\n",
            start,
            start + len - 1,
            total
        ));
    }
    hdr.push_str("Connection: close\r\n\r\n");
    sock.write_all(hdr.as_bytes()).await?;

    let mut f = tokio::fs::File::open(file).await?;
    f.seek(std::io::SeekFrom::Start(start)).await?;
    let mut buf = vec![0u8; CHUNK];
    let mut sent: u64 = 0;
    while sent < len {
        let want = CHUNK.min((len - sent) as usize);
        let n = f.read(&mut buf[..want]).await?;
        if n == 0 {
            break;
        }
        sock.write_all(&buf[..n]).await?;
        sent += n as u64;
    }
    sock.flush().await?;
    let _ = sock.shutdown().await;
    Ok(())
}

/// The door's own: an origin that keeps every request head it is sent, for
/// the test to read, and answers 200 `ok` — echoing any `authorization` it
/// received in `x-seen-authorization`, as a careless service might, so every
/// test through it watches the door keep a secret from going back.
pub async fn head_origin() -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().unwrap();
    let heads = Arc::new(Mutex::new(Vec::new()));
    let kept = heads.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
                let Some(head) = read_head(&mut sock).await else {
                    return;
                };
                let seen: Vec<String> = head
                    .lines()
                    .filter_map(|l| l.split_once(':'))
                    .filter(|(n, _)| n.trim().eq_ignore_ascii_case("authorization"))
                    .map(|(_, v)| format!("x-seen-authorization: {}\r\n", v.trim()))
                    .collect();
                kept.lock().unwrap().push(head);
                let _ = sock
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\n{}Content-Length: 2\r\nConnection: close\r\n\r\nok",
                            seen.concat()
                        )
                        .as_bytes(),
                    )
                    .await;
                let _ = sock.shutdown().await;
            });
        }
    });
    (addr, heads)
}

/// The door's own: an origin that takes one upload, says when its body's
/// first byte arrives, and answers its length and SHA-256.
pub async fn upload_origin() -> (SocketAddr, tokio::sync::oneshot::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().unwrap();
    let (first, first_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let Ok((mut sock, _)) = listener.accept().await else {
            return;
        };
        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            if sock.read(&mut byte).await.unwrap_or(0) == 0 {
                return;
            }
            head.push(byte[0]);
        }
        let head = String::from_utf8_lossy(&head).to_ascii_lowercase();
        let length: u64 = head
            .lines()
            .find_map(|l| l.strip_prefix("content-length:"))
            .and_then(|v| v.trim().parse().ok())
            .expect("the upload's head carries its length");
        let (mut sha, mut got, mut first) = (Sha256::new(), 0u64, Some(first));
        let mut buf = vec![0u8; CHUNK];
        while got < length {
            let n = sock.read(&mut buf).await.unwrap_or(0);
            if n == 0 {
                break;
            }
            if let Some(tx) = first.take() {
                let _ = tx.send(());
            }
            sha.update(&buf[..n]);
            got += n as u64;
        }
        let body = format!("{got} {}", hex::encode(sha.finalize()));
        let _ = sock
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .as_bytes(),
            )
            .await;
        let _ = sock.shutdown().await;
    });
    (addr, first_rx)
}

/// A request head off `sock`, up to its blank line.
async fn read_head(sock: &mut TcpStream) -> Option<String> {
    let mut head = Vec::new();
    let mut buf = [0u8; 4096];
    while !head.windows(4).any(|w| w == b"\r\n\r\n") {
        let n = sock.read(&mut buf).await.ok()?;
        if n == 0 {
            return None;
        }
        head.extend_from_slice(&buf[..n]);
    }
    Some(String::from_utf8_lossy(&head).into_owned())
}
