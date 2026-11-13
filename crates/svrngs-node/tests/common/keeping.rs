//! A lent service that keeps what it is sent: carried.rs's, for its sends
//! (row 13, d05674c0), answered on the wire.

/// What a [`keep_one`] service kept, each request whole as it came.
pub type Kept = std::sync::Arc<std::sync::Mutex<Vec<String>>>;

/// A lent service's answer to `text`: JSON, or a page with a script in it
/// when the request says `script`, a send's answer svrngs must not hand a
/// page.
pub fn embedding(text: &str) -> (&'static str, String) {
    match text.contains("script") {
        true => ("text/html", "<script>alert(1)</script>".into()),
        false => (
            "application/json",
            r#"{"data":[{"embedding":[0.5,0.25]}]}"#.into(),
        ),
    }
}

/// A lent service's answer on one connection, a pipe's end on the wire
/// (common/on_wire.rs `serve`): the request kept in `kept`, whole as it
/// came (its head, then as much body as its length says), and answered 200
/// with `answer`'s type and body, given the request.
pub async fn keep_one(
    mut sock: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    kept: Kept,
    answer: fn(&str) -> (&'static str, String),
) {
    let mut read = Vec::new();
    let mut buf = [0u8; 65536];
    // The head, then as much body as its length says.
    loop {
        let Ok(n) = tokio::io::AsyncReadExt::read(&mut sock, &mut buf).await else {
            return;
        };
        read.extend_from_slice(&buf[..n]);
        let text = String::from_utf8_lossy(&read).to_string();
        let Some(end) = text.find("\r\n\r\n") else {
            if n == 0 {
                return;
            }
            continue;
        };
        let length = (text[..end].lines())
            .find_map(|l| {
                l.to_ascii_lowercase()
                    .strip_prefix("content-length:")
                    .map(|v| v.trim().parse::<usize>().unwrap_or(0))
            })
            .unwrap_or(0);
        if n == 0 || read.len() >= end + 4 + length {
            break;
        }
    }
    let text = String::from_utf8_lossy(&read).to_string();
    let (kind, body) = answer(&text);
    kept.lock().unwrap().push(text);
    let answer = format!("HTTP/1.1 200 OK\r\ncontent-type: {kind}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}", body.len());
    let _ = tokio::io::AsyncWriteExt::write_all(&mut sock, answer.as_bytes()).await;
}
