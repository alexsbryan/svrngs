//! A lent service that keeps what it is sent: one copy, which carried.rs
//! wrote out for its sends (row 13, d05674c0) and the browser's sends share.

/// What a [`keeping_by`] service kept, each request whole as it came.
pub type Kept = std::sync::Arc<std::sync::Mutex<Vec<String>>>;

/// A lent service that keeps each request it is asked, whole as it came,
/// and answers JSON, or a page with a script in it when the request says
/// `script`: a send's answer svrngs must not hand a page.
pub async fn keeping() -> (u16, Kept) {
    keeping_by(|text| match text.contains("script") {
        true => ("text/html", "<script>alert(1)</script>".into()),
        false => (
            "application/json",
            r#"{"data":[{"embedding":[0.5,0.25]}]}"#.into(),
        ),
    })
    .await
}

/// A lent service that keeps each request it is asked, whole as it came
/// (its head, then as much body as its length says), and answers it 200
/// with `answer`'s type and body, given the request.
pub async fn keeping_by(answer: fn(&str) -> (&'static str, String)) -> (u16, Kept) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let asked = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let kept = asked.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
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
            });
        }
    });
    (port, asked)
}
