// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP over the dial (PLAN §4, U4's inventory, the u4-4 row): the client
//! a page speaks to a computer's door, one request per bi-stream of the
//! connection the dial made.
//!
//! hyper's HTTP/1 client writes the request and parses the response, as the
//! node's door parses the other end of the same stream with hyper's server
//! (crates/svrngs-node/src/door.rs); upstream read the response by hand,
//! whole, under a 512 KiB cap (ring-runtime/src/lib.rs:166-199), and only
//! ever sent a GET. Here any method; the request's body is read from the
//! page's `ReadableStream` as hyper sends it, and the response's body is
//! handed back as one, chunk by chunk, by content-length or chunked — so no
//! body is held whole on either side. Every header the page names goes as
//! named: `Range` and `If-Range` reach the door untouched, and every
//! response header, `Content-Range` with them, comes back.
//!
//! The send half stays open until the response ends: hyper shuts the write
//! side only once its read side is done, and the shutdown is the stream's
//! `finish` (noq's `poll_shutdown`). Finishing it first is what failed
//! upstream's first browser dial (`Ok(0 bytes)`, browser-dial-2).

use std::pin::Pin;
use std::task::{ready, Context, Poll};

use bytes::Bytes;
use futures_util::StreamExt;
use http_body_util::{BodyExt, Either, Empty, StreamBody};
use hyper::body::Frame;
use iroh::endpoint::{Connection, RecvStream, SendStream};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use wasm_bindgen::prelude::*;

use crate::emit;

/// The most a read off the stream hands hyper at once, and so the largest
/// chunk the page is handed of a response's body.
const READ_CHUNK: usize = 64 * 1024;

/// Where a response's `Set-Cookie` values are handed to the page (u5-2).
const COOKIES: &str = "x-svrngs-set-cookies";

/// The most a socket's answer's head may be before its blank line.
const HEAD_MOST: usize = 64 * 1024;

/// Why a request got no response.
#[derive(Debug, thiserror::Error)]
pub enum Refused {
    #[error("the method {0:?} is not an HTTP method")]
    Method(String),
    #[error("the path {0:?} is not a request target")]
    Path(String),
    #[error("the header {0:?} cannot be sent")]
    Header(String),
    #[error("the headers are not a list of [name, value] pairs")]
    Headers,
    #[error("the body cannot be read: {0}")]
    Locked(String),
    #[error("no stream could be opened to the computer: {0}")]
    Stream(String),
    #[error("the computer's answer is not HTTP: {0}")]
    Http(#[from] hyper::Error),
    #[error("the answer cannot be handed to the page: {0}")]
    Page(String),
}

/// Why the page's request body stopped short.
#[derive(Debug, thiserror::Error)]
pub enum BodyError {
    #[error("the page's body yielded a chunk that is not bytes")]
    NotBytes,
    #[error("the page's body failed: {0}")]
    Page(String),
}

/// One bi-stream as hyper's connection: reads off the receive half, writes
/// to the send half, and shuts down by finishing the send half.
struct Duplex {
    send: SendStream,
    recv: RecvStream,
    scratch: Box<[u8]>,
}

impl hyper::rt::Read for Duplex {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let want = buf.remaining().min(this.scratch.len());
        let mut read = ReadBuf::new(&mut this.scratch[..want]);
        ready!(AsyncRead::poll_read(Pin::new(&mut this.recv), cx, &mut read))?;
        buf.put_slice(read.filled());
        Poll::Ready(Ok(()))
    }
}

impl hyper::rt::Write for Duplex {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        AsyncWrite::poll_write(Pin::new(&mut self.get_mut().send), cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.get_mut().send), cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        emit("HTTP: the response has ended; finishing the request's send half");
        AsyncWrite::poll_shutdown(Pin::new(&mut self.get_mut().send), cx)
    }
}

type RequestBody = Either<Empty<Bytes>, StreamBody<PageBody>>;
type PageBody = futures_util::stream::Map<
    wasm_streams::readable::IntoStream<'static>,
    fn(Result<JsValue, JsValue>) -> Result<Frame<Bytes>, BodyError>,
>;

/// A chunk of the page's body as a frame hyper sends.
fn frame(chunk: Result<JsValue, JsValue>) -> Result<Frame<Bytes>, BodyError> {
    let chunk = chunk.map_err(|e| BodyError::Page(format!("{e:?}")))?;
    let bytes = chunk
        .dyn_into::<js_sys::Uint8Array>()
        .map_err(|_| BodyError::NotBytes)?;
    Ok(Frame::data(Bytes::from(bytes.to_vec())))
}

/// The `Host` a request is asked with when the page names none: HTTP/1.1
/// asks for one, and the door passes it to the service as is.
const UNNAMED_HOST: &str = "svrngs";

/// Hand `each` every `[name, value]` pair of `headers`, as named and in
/// order, with the header hyper reads it as; refused at the first that is
/// not one. Answers whether one is `Host`. [`request`] and [`socket`] ask
/// it, so both send the same headers (REVIEW-audit-u5-auto-1 #56).
fn headers_of(
    headers: &js_sys::Array,
    mut each: impl FnMut(&str, &str, hyper::header::HeaderName, hyper::header::HeaderValue),
) -> Result<bool, Refused> {
    let mut host = false;
    for pair in headers.iter() {
        let pair = pair.dyn_into::<js_sys::Array>().map_err(|_| Refused::Headers)?;
        let (Some(name), Some(value)) = (pair.get(0).as_string(), pair.get(1).as_string()) else {
            return Err(Refused::Headers);
        };
        let header =
            hyper::header::HeaderName::from_bytes(name.as_bytes()).map_err(|_| Refused::Header(name.clone()))?;
        let parsed = hyper::header::HeaderValue::from_str(&value).map_err(|_| Refused::Header(name.clone()))?;
        host |= header == hyper::header::HOST;
        each(&name, &value, header, parsed);
    }
    Ok(host)
}

/// The request hyper sends: `method` and `path`, every header in `headers`
/// (an array of `[name, value]`) as named and in order, a `Host` only when
/// none is named, and `body` read as it is sent.
fn request(
    method: &str,
    path: &str,
    headers: &js_sys::Array,
    body: Option<web_sys::ReadableStream>,
) -> Result<hyper::Request<RequestBody>, Refused> {
    let method = hyper::Method::from_bytes(method.as_bytes()).map_err(|_| Refused::Method(method.to_string()))?;
    let uri: hyper::Uri = path.parse().map_err(|_| Refused::Path(path.to_string()))?;
    let mut named = hyper::HeaderMap::new();
    let host = headers_of(headers, |_, _, header, value| {
        named.append(header, value);
    })?;
    if !host {
        named.insert(
            hyper::header::HOST,
            hyper::header::HeaderValue::from_static(UNNAMED_HOST),
        );
    }
    let body = match body {
        None => Either::Left(Empty::new()),
        Some(raw) => {
            let stream = wasm_streams::ReadableStream::from_raw(raw)
                .try_into_stream()
                .map_err(|(e, _)| Refused::Locked(String::from(e.message())))?;
            let frame: fn(_) -> _ = frame;
            Either::Right(StreamBody::new(stream.map(frame)))
        }
    };
    let mut request = hyper::Request::new(body);
    *request.method_mut() = method;
    *request.uri_mut() = uri;
    *request.version_mut() = hyper::Version::HTTP_11;
    *request.headers_mut() = named;
    Ok(request)
}

/// Statuses whose response has no body, which a page's `Response` refuses
/// one for (Fetch's "null body status").
fn null_body(status: u16) -> bool {
    matches!(status, 101 | 103 | 204 | 205 | 304)
}

/// Send one request on a new bi-stream of `conn` and answer the page's
/// `Response`, its body a `ReadableStream` read off the stream as the page
/// reads it.
pub async fn fetch(
    conn: Connection,
    method: String,
    path: String,
    headers: js_sys::Array,
    body: Option<web_sys::ReadableStream>,
) -> Result<web_sys::Response, Refused> {
    let request = request(&method, &path, &headers, body)?;
    let (send, recv) = conn.open_bi().await.map_err(|e| Refused::Stream(e.to_string()))?;
    emit(&format!("HTTP: {method} {path} — a bi-stream opened"));
    let duplex = Duplex {
        send,
        recv,
        scratch: vec![0; READ_CHUNK].into_boxed_slice(),
    };
    let (mut sender, connection) = hyper::client::conn::http1::handshake(duplex).await?;
    let label = format!("{method} {path}");
    wasm_bindgen_futures::spawn_local(async move {
        match connection.await {
            Ok(()) => emit(&format!("HTTP: {label} — the stream is done")),
            Err(e) => emit(&format!("HTTP: {label} — the stream ended in error: {e}")),
        }
    });
    let answer = sender.send_request(request).await?;
    // No second request rides this stream: with the sender gone, hyper
    // ends the connection once the response's body has been read.
    drop(sender);
    let (parts, incoming) = answer.into_parts();
    let status = parts.status.as_u16();
    emit(&format!("HTTP: {method} {path} answered {status}"));
    let headers = web_sys::Headers::new().map_err(|e| Refused::Page(format!("{e:?}")))?;
    // A page's `Response` drops `Set-Cookie`, so the lend's worker, which
    // keeps each lend's cookies (u5-2), is handed them under a name of its
    // own: a JSON array of each value.
    let cookies: Vec<&str> = parts
        .headers
        .get_all(hyper::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .collect();
    if !cookies.is_empty() {
        let json = serde_json::to_string(&cookies).unwrap_or_default();
        headers
            .append(COOKIES, &json)
            .map_err(|e| Refused::Page(format!("{e:?}")))?;
    }
    for (name, value) in &parts.headers {
        match value.to_str() {
            Ok(value) => headers
                .append(name.as_str(), value)
                .map_err(|e| Refused::Page(format!("{e:?}")))?,
            Err(_) => emit(&format!(
                "HTTP: {method} {path} — the header {name} is not text, so the page is not handed it"
            )),
        }
    }
    let init = web_sys::ResponseInit::new();
    init.set_status(status);
    init.set_status_text(parts.status.canonical_reason().unwrap_or(""));
    init.set_headers(&headers);
    let body = (!null_body(status)).then(|| {
        let chunks = incoming.into_data_stream().map(move |chunk| match chunk {
            Ok(bytes) => Ok(JsValue::from(js_sys::Uint8Array::from(&bytes[..]))),
            Err(e) => {
                emit(&format!("HTTP: {method} {path} — the body stopped short: {e}"));
                Err(JsValue::from(js_sys::Error::new(&e.to_string())))
            }
        });
        wasm_streams::ReadableStream::from_stream(chunks).into_raw()
    });
    web_sys::Response::new_with_opt_readable_stream_and_init(body.as_ref(), &init)
        .map_err(|e| Refused::Page(format!("{e:?}")))
}

/// Ask on a new bi-stream of `conn` to switch `path` to another protocol,
/// `headers` naming it, and answer `{status, headers, readable, writable}`:
/// the status and head of the answer, then the stream's raw bytes both
/// ways (u5-2). Written by hand, as the head is all there is to HTTP here;
/// the door's hyper reads it, and splices the stream once its service
/// switches.
pub async fn socket(conn: Connection, path: &str, headers: &js_sys::Array) -> Result<JsValue, Refused> {
    let mut head = format!("GET {path} HTTP/1.1\r\n");
    let host = headers_of(headers, |name, value, _, _| {
        head.push_str(&format!("{name}: {value}\r\n"))
    })?;
    if !host {
        head.push_str(&format!("Host: {UNNAMED_HOST}\r\n"));
    }
    head.push_str("\r\n");
    let (mut send, mut recv) = conn.open_bi().await.map_err(|e| Refused::Stream(e.to_string()))?;
    send.write_all(head.as_bytes())
        .await
        .map_err(|e| Refused::Stream(e.to_string()))?;
    let mut got = Vec::new();
    let end = loop {
        if let Some(at) = got.windows(4).position(|w| w == b"\r\n\r\n") {
            break at + 4;
        }
        if got.len() > HEAD_MOST {
            return Err(Refused::Stream("the answer's head never ended".into()));
        }
        match recv
            .read_chunk(READ_CHUNK)
            .await
            .map_err(|e| Refused::Stream(e.to_string()))?
        {
            Some(chunk) => got.extend_from_slice(&chunk),
            None => return Err(Refused::Stream("the stream ended before its answer did".into())),
        }
    };
    let text = String::from_utf8_lossy(&got[..end]).into_owned();
    let mut lines = text.split("\r\n");
    let status: u16 = lines
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Refused::Stream(format!("not an HTTP answer: {text}")))?;
    emit(&format!("HTTP: a socket at {path} answered {status}"));
    let named = js_sys::Array::new();
    for (name, value) in lines.filter_map(|l| l.split_once(':')) {
        named.push(&js_sys::Array::of2(&name.trim().into(), &value.trim().into()));
    }
    let first = Bytes::copy_from_slice(&got[end..]);
    let read = futures_util::stream::unfold((Some(first), recv), |(first, mut recv)| async move {
        if let Some(first) = first.filter(|f| !f.is_empty()) {
            return Some((Ok(JsValue::from(js_sys::Uint8Array::from(&first[..]))), (None, recv)));
        }
        match recv.read_chunk(READ_CHUNK).await {
            Ok(Some(chunk)) => Some((Ok(JsValue::from(js_sys::Uint8Array::from(&chunk[..]))), (None, recv))),
            Ok(None) => None,
            Err(e) => Some((Err(JsValue::from(js_sys::Error::new(&e.to_string()))), (None, recv))),
        }
    });
    let write = futures_util::sink::unfold(send, |mut send, chunk: JsValue| async move {
        let bytes = chunk
            .dyn_into::<js_sys::Uint8Array>()
            .map_err(|_| JsValue::from(js_sys::Error::new("a socket writes bytes")))?
            .to_vec();
        send.write_all(&bytes)
            .await
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
        Ok::<_, JsValue>(send)
    });
    let answer = js_sys::Object::new();
    let set = |k: &str, v: &JsValue| js_sys::Reflect::set(&answer, &k.into(), v).map(|_| ());
    set("status", &status.into())
        .and(set("headers", &named))
        .and(set(
            "readable",
            &wasm_streams::ReadableStream::from_stream(read).into_raw(),
        ))
        .and(set(
            "writable",
            &wasm_streams::WritableStream::from_sink(write).into_raw(),
        ))
        .map_err(|e| Refused::Page(format!("{e:?}")))?;
    Ok(answer.into())
}
