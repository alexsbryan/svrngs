// SPDX-License-Identifier: AGPL-3.0-or-later
//! A lend's sign-in (mwp-1; DESIGN §3, "The room", call 6): the lender's
//! program, on a port of this computer, that the door asks who each person
//! is to the lend, so each is signed in as themselves with nothing typed.
//!
//! **Asked** on a person's first request through the lend, before it is
//! carried: `POST /` at `127.0.0.1:<its port>`, with the request's
//! [`super::PERSON_HEADER`] and [`super::KEY_HEADER`] as the door sets them
//! for the service, and nothing else. **Answered** `200` and a JSON
//! object, every field required and none other ([`Answer`]): the person's
//! `secret`, the `headers` set on each of their requests, and a `script`
//! or `null` (mwp-2 puts it in the lend's pages). Anything else is a
//! [`Refusal`], and the person's request is refused, never carried unsigned.
//!
//! **Kept** by [`SignIns`], one task that owns every answer, in memory for
//! this run of the node — never logged, never on disk, never on the wire
//! but to the service — and asks one person at a time, so a person's first
//! requests at once ask once. A `401` from the service to a request signed
//! with a kept answer forgets it, and the next request asks again.
//!
//! **Swapped** ([`STAND_IN`]): in what the door carries in, the stand-in
//! in a query or a header's value becomes the asker's secret; in what it
//! carries out, every secret held for the lend becomes the stand-in, in
//! each header and in a textual body, streamed ([`Swapping`]).

use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroU16;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use std::time::Duration;

use axum::body::Body;
use axum::http::header::{self, HeaderMap, HeaderName, HeaderValue};
use axum::http::StatusCode;
use http_body_util::BodyExt;
use hyper::body::{Bytes, Frame};
use tokio::sync::{mpsc, oneshot};

use crate::lend::LendName;

/// What a browser holds in place of a person's secret, the same for every
/// lend and person: it names nobody, since the door puts in the secret of
/// whoever it let in, never by the stand-in.
pub const STAND_IN: &str = "svrngs-stand-in";

/// How long a sign-in may take to answer, asked and read.
const ASK_WITHIN: Duration = Duration::from_secs(30);

/// The most of an answer read: a secret, a few headers and a script.
const ANSWER_AT_MOST: usize = 1 << 20;

/// A person's secret: 16 to 512 bytes of letters, digits, `-`, `_`, `.` and
/// `~`, which a query, a header and a JSON string all carry unescaped, so
/// one search finds it in each, and swapping it for [`STAND_IN`] leaves a
/// header a header. Never `Debug`, so never in a trace.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Secret(String);

/// What a sign-in answered for one person, parsed.
#[derive(Clone)]
pub(crate) struct Answer {
    pub secret: Secret,
    /// Set on each of the person's requests, displacing the client's.
    pub headers: Vec<(HeaderName, HeaderValue)>,
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "kept now; put in the lend's pages by mwp-2")
    )]
    pub script: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    secret: String,
    headers: BTreeMap<String, String>,
    #[serde(deserialize_with = "Option::deserialize")]
    script: Option<String>,
}

impl Answer {
    /// `body` as a sign-in's answer, or why it is none. A serde error is
    /// told by its class and place, never its text, which can quote a value.
    pub(crate) fn parse(body: &[u8]) -> Result<Self, Refusal> {
        let bad = |why: String| Err(Refusal::Malformed(why));
        let wire: Wire = match serde_json::from_slice(body) {
            Ok(w) => w,
            Err(e) => return bad(format!("{:?} at {}:{}", e.classify(), e.line(), e.column())),
        };
        let plain = |b: u8| b.is_ascii_alphanumeric() || b"-_.~".contains(&b);
        if !(16..=512).contains(&wire.secret.len()) || !wire.secret.bytes().all(plain) {
            return bad("its secret is not 16 to 512 letters, digits, -, _, . or ~".into());
        }
        let mut headers = Vec::new();
        for (name, value) in wire.headers {
            let compared = super::compare_name(&name);
            let ours = compared.starts_with(super::IDENTITY_PREFIX) || compared == "host";
            match (HeaderName::try_from(&name), HeaderValue::try_from(value)) {
                (Ok(n), Ok(v)) if !ours && !super::HOP_BY_HOP.contains(&compared.as_str()) => {
                    headers.push((n, v))
                }
                _ => return bad(format!("its header {name:?} is not one a sign-in may set")),
            }
        }
        let script = wire
            .script
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if script.contains(&wire.secret.to_ascii_lowercase()) || script.contains("</script") {
            return bad("its script holds the secret or `</script`".into());
        }
        let secret = Secret(wire.secret);
        Ok(Self {
            secret,
            headers,
            script: wire.script,
        })
    }
}

/// Why a sign-in did not sign a person in: a closed set, traced at warn on
/// the lender's side, never sent to the person.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Refusal {
    #[error("nothing answered on its port, or not within 30 s")]
    Unanswered,
    #[error("it answered {0}, not 200")]
    Status(u16),
    #[error("its answer is not a sign-in's: {0}")]
    Malformed(String),
    #[error("the door's keeper of sign-ins has stopped")]
    Gone,
}

/// A person signed in to a lend: their answer, and every secret the door
/// holds for the lend now, which its answers to anyone are swapped of.
pub(crate) struct Signed {
    pub answer: Answer,
    pub secrets: Vec<Secret>,
}

/// Which answer: a lend, its sign-in's port, and a person's key.
type Whose = (LendName, NonZeroU16, String);

enum Ask {
    Of(
        Whose,
        [(HeaderName, HeaderValue); 2],
        oneshot::Sender<Result<Signed, Refusal>>,
    ),
    Forget(Whose, Secret),
}

/// Every person's sign-in answer, owned by one task and told by messages
/// (RUST_ARCH 3, 4), as [`super::Using`] is.
#[derive(Clone)]
pub struct SignIns(mpsc::UnboundedSender<Ask>);

impl SignIns {
    /// The task that keeps the answers, ending when the last handle drops.
    pub fn spawn() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Ask>();
        tokio::spawn(async move {
            let mut kept: HashMap<Whose, Answer> = HashMap::new();
            while let Some(ask) = rx.recv().await {
                match ask {
                    Ask::Of(whose, identity, answer) => {
                        let (lend, port, actor) = &whose;
                        let found = match kept.get(&whose) {
                            Some(a) => {
                                tracing::debug!(target: "door", lend = %lend, actor, "door: sign-in — the person's kept answer");
                                Ok(a.clone())
                            }
                            None => ask_of(*port, identity, lend).await.inspect(|a| {
                                tracing::info!(target: "door", lend = %lend, actor, "door: sign-in — asked for the person on their first request, and kept");
                                kept.insert(whose.clone(), a.clone());
                            }),
                        };
                        let signed = found.map(|answer| Signed {
                            answer,
                            secrets: kept
                                .iter()
                                .filter(|((l, _, _), _)| l == lend)
                                .map(|(_, a)| a.secret.clone())
                                .collect(),
                        });
                        let _ = answer.send(signed);
                    }
                    Ask::Forget(whose, secret) => {
                        if kept.get(&whose).is_some_and(|a| a.secret == secret) {
                            tracing::info!(target: "door", lend = %whose.0, actor = whose.2,
                                "door: sign-in — the service refused a kept answer, so it is forgotten and asked again next");
                            kept.remove(&whose);
                        }
                    }
                }
            }
        });
        Self(tx)
    }

    /// The person `identity` names signed in to `lend`: kept, or asked of
    /// its sign-in at `port` now.
    pub(crate) async fn of(
        &self,
        lend: &LendName,
        port: NonZeroU16,
        actor: &str,
        identity: [(HeaderName, HeaderValue); 2],
    ) -> Result<Signed, Refusal> {
        let (answer, told) = oneshot::channel();
        let whose = (lend.clone(), port, actor.to_string());
        self.0
            .send(Ask::Of(whose, identity, answer))
            .map_err(|_| Refusal::Gone)?;
        told.await.map_err(|_| Refusal::Gone)?
    }

    /// Forget `actor`'s answer for `lend`, if it is still the one holding
    /// `secret`.
    pub(crate) fn forget(&self, lend: &LendName, port: NonZeroU16, actor: &str, secret: &Secret) {
        let whose = (lend.clone(), port, actor.to_string());
        let _ = self.0.send(Ask::Forget(whose, secret.clone()));
    }
}

/// Ask the sign-in at `port` for the person `identity` names.
async fn ask_of(
    port: NonZeroU16,
    identity: [(HeaderName, HeaderValue); 2],
    lend: &LendName,
) -> Result<Answer, Refusal> {
    let mut request = axum::http::Request::post("/")
        .header(header::HOST, format!("127.0.0.1:{port}"))
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("a fixed request is one");
    request.headers_mut().extend(identity);
    let to = std::net::SocketAddr::from(([127, 0, 0, 1], port.get()));
    let asked = async {
        let name = format!("{lend}'s sign-in");
        let reply = super::send_to(to, request, &name)
            .await
            .ok_or(Refusal::Unanswered)?;
        if reply.status() != StatusCode::OK {
            return Err(Refusal::Status(reply.status().as_u16()));
        }
        let body = http_body_util::Limited::new(reply.into_body(), ANSWER_AT_MOST)
            .collect()
            .await
            .map_err(|_| Refusal::Malformed("unread, or over 1 MiB".into()))?;
        Answer::parse(&body.to_bytes())
    };
    tokio::time::timeout(ASK_WITHIN, asked)
        .await
        .map_err(|_| Refusal::Unanswered)?
}

/// `bytes` with each of `from` swapped for `to`, leftmost first; and how
/// many of `bytes` that covers. Unless `end`, a tail shorter than the
/// longest of `from` is held back, since a match may finish in what comes
/// next: one byte fewer than the longest.
pub(crate) fn swapped(bytes: &[u8], from: &[&[u8]], to: &[u8], end: bool) -> (Vec<u8>, usize) {
    let longest = from.iter().map(|f| f.len()).max().unwrap_or(0);
    let (mut out, mut i) = (Vec::with_capacity(bytes.len()), 0);
    while i < bytes.len() {
        if let Some(f) = from
            .iter()
            .find(|f| !f.is_empty() && bytes[i..].starts_with(f))
        {
            out.extend_from_slice(to);
            i += f.len();
        } else if !end && bytes.len() - i < longest {
            break;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    (out, i)
}

/// Each of `from` in `headers`' values swapped for `to`; how many changed.
/// A secret and the stand-in are both of a header's bytes, so a value
/// swapped is one still.
pub(crate) fn swap_headers(headers: &mut HeaderMap, from: &[&[u8]], to: &[u8]) -> usize {
    let mut n = 0;
    for value in headers.values_mut() {
        let (out, _) = swapped(value.as_bytes(), from, to, true);
        if out != value.as_bytes() {
            *value = HeaderValue::from_bytes(&out).unwrap_or(HeaderValue::from_static(""));
            n += 1;
        }
    }
    n
}

/// The secrets as [`swapped`] reads them.
pub(crate) fn bytes(secrets: &[Secret]) -> Vec<&[u8]> {
    secrets.iter().map(|s| s.0.as_bytes()).collect()
}

impl Secret {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether an answer is read for a secret, by its `Content-Type`: text,
/// JSON, XML, a playlist, JavaScript, and one naming none, read rather than
/// presumed bytes.
pub(crate) fn textual(headers: &HeaderMap) -> bool {
    let t = headers
        .get(header::CONTENT_TYPE)
        .map(|t| t.as_bytes())
        .unwrap_or_default();
    let t = String::from_utf8_lossy(t).to_ascii_lowercase();
    let t = t.split(';').next().unwrap_or_default().trim();
    t.is_empty()
        || t.starts_with("text/")
        || ["json", "xml", "mpegurl", "javascript"]
            .iter()
            .any(|e| t.ends_with(e))
}

/// Whether an answer's body is compressed, so cannot be read for a secret.
pub(crate) fn compressed(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_ENCODING)
        .is_some_and(|e| !e.as_bytes().eq_ignore_ascii_case(b"identity"))
}

/// A textual answer's body, each secret swapped for the stand-in as it
/// streams, a secret split across chunks found by [`swapped`]'s holding
/// back. A trailer cannot be swapped, so is not carried.
pub(crate) struct Swapping<B> {
    inner: B,
    secrets: Vec<Secret>,
    held: Vec<u8>,
    done: bool,
}

impl<B> Swapping<B> {
    pub(crate) fn new(inner: B, secrets: Vec<Secret>) -> Self {
        let (held, done) = (Vec::new(), false);
        Self {
            inner,
            secrets,
            held,
            done,
        }
    }
}

impl<B> hyper::body::Body for Swapping<B>
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
{
    type Data = Bytes;
    type Error = B::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, B::Error>>> {
        let this = &mut *self;
        while !this.done {
            match ready!(Pin::new(&mut this.inner).poll_frame(cx)) {
                Some(Ok(frame)) => this
                    .held
                    .extend_from_slice(&frame.into_data().unwrap_or_default()),
                Some(Err(e)) => return Poll::Ready(Some(Err(e))),
                None => this.done = true,
            }
            let end = this.done;
            let (out, used) = swapped(&this.held, &bytes(&this.secrets), STAND_IN.as_bytes(), end);
            this.held.drain(..used);
            if !out.is_empty() {
                return Poll::Ready(Some(Ok(Frame::data(out.into()))));
            }
        }
        Poll::Ready(None)
    }
}

#[cfg(test)]
mod tests;
