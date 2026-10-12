// SPDX-License-Identifier: AGPL-3.0-or-later
//! A lend's sign-in (mwp-1; DESIGN §3, "The room", call 6): the lender's
//! program, on a port of this computer, that the door asks who each person
//! is to the lend, so each is signed in as themselves with nothing typed.
//!
//! **Asked** on a person's first request through the lend, before it is
//! carried: `POST /` at `127.0.0.1:<its port>`, with the request's
//! [`super::Identity`] as the door sets it for the service — the person's
//! key, the name to show and their account's name — and [`STAND_IN`] in [`STAND_IN_HEADER`], for a script
//! to store, and nothing else. **Answered** `200` and a JSON object, every
//! field required and none other ([`Answer`]): the person's `secret`, the
//! `headers` set on each of their requests, and a `script` or `null` (mwp-2
//! puts it in the lend's pages). Anything else is a
//! [`Refusal`], and the person's request is refused, never carried unsigned.
//!
//! **Kept** by [`SignIns`], one task that owns every answer, in memory for
//! this run of the node — never logged, never on disk, never on the wire
//! but to the service — which asks for each person apart, so one person's
//! slow sign-in holds nobody else's request, and a person's first requests
//! at once wait on one ask. A `401` from the service to a request signed
//! with a kept answer forgets it, and the next request asks again.
//!
//! **Swapped** ([`STAND_IN`]): in what the door carries in, the stand-in
//! in a query or a header's value becomes the asker's secret; in what it
//! carries out, every secret held for the lend becomes the stand-in, in
//! each header and in a textual body, streamed ([`Swapping`]).
//!
//! **Put first** (mwp-2): the person's `script`, in a `<script>` of the
//! door's ([`tag`]), first in `<head>` of each page answered to their `GET`
//! ([`first_in`]), so it runs before every script of the page's own.

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

/// The header the door's ask names [`STAND_IN`] in, so what an answer's
/// script stores is read from the ask, never spelled by the app (#90).
pub const STAND_IN_HEADER: &str = "x-svrngs-stand-in";

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
    /// Put first in the lend's pages to the person ([`tag`], [`first_in`]).
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
#[derive(Clone, Debug, thiserror::Error)]
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

/// How a request through a lend is signed, decided once as the door lets
/// the person in: by the lend's one secret, or by the person's sign-in at
/// its port. Each way is taken at one site, [`Signing::inward`] on the
/// request and [`Signing::outward`] on the answer.
pub(crate) enum Signing {
    /// The lend's one secret, as headers (`crate::declared`).
    Secret(Vec<(HeaderName, HeaderValue)>),
    /// The person, signed in by the lend's sign-in at the port.
    Person(NonZeroU16, Signed),
}

/// What a textual answer's body is read with: every secret held for the
/// lend, and the person's script for a page ([`Swapping`]).
pub(crate) type Read = (Vec<Secret>, Option<Vec<u8>>);

impl Signing {
    /// The target `path` and `query` are asked as, and the headers set on
    /// the request; `headers` made ready for them. Inward, the stand-in
    /// becomes the asker's secret, in the query and each header.
    pub(crate) fn inward(
        &self,
        headers: &mut HeaderMap,
        path: String,
        query: Option<&str>,
    ) -> (String, Vec<(HeaderName, HeaderValue)>) {
        let s = match self {
            Self::Secret(secret) => {
                let target = query.map_or(path.clone(), |q| format!("{path}?{q}"));
                return (target, secret.clone());
            }
            Self::Person(_, s) => s,
        };
        let own = s.answer.secret.as_str();
        let target = query.map_or(path.clone(), |q| {
            format!("{path}?{}", q.replace(STAND_IN, own))
        });
        swap_headers(headers, &[STAND_IN.as_bytes()], own.as_bytes());
        headers.remove(header::ACCEPT_ENCODING);
        // No copy kept from before the person's script, or for another's.
        if s.answer.script.is_some() && said(headers, header::ACCEPT).contains("text/html") {
            headers.remove(header::IF_NONE_MATCH);
            headers.remove(header::IF_MODIFIED_SINCE);
        }
        (target, s.answer.headers.clone())
    }

    /// The answer's head made ready for the person, who asked by `GET`
    /// when `get`: a header holding the one secret dropped; or, signed in,
    /// a kept answer the service refused forgotten, each secret held for
    /// the lend swapped for the stand-in, and their script made for a page.
    /// Answers what the body is read with, if it is, or the door's refusal
    /// of an answer it cannot read.
    pub(crate) fn outward(
        self,
        parts: &mut axum::http::response::Parts,
        get: bool,
        sign_ins: &SignIns,
        (lend, actor): (&LendName, &super::PersonKey),
    ) -> Result<Option<Read>, axum::response::Response> {
        let (port, s) = match self {
            Self::Secret(secret) => {
                let echoed = super::drop_echoed_secret(&mut parts.headers, &secret);
                if !echoed.is_empty() {
                    tracing::warn!(target: "door", lend = %lend, headers = ?echoed,
                        "door: the lend's service answered with its secret in a header — dropped, never sent on");
                }
                return Ok(None);
            }
            Self::Person(port, s) => (port, s),
        };
        if parts.status == StatusCode::UNAUTHORIZED {
            sign_ins.forget(lend, port, actor, &s.answer.secret);
        }
        let swapped = swap_headers(&mut parts.headers, &bytes(&s.secrets), STAND_IN.as_bytes());
        tracing::debug!(target: "door", lend = %lend, swapped, "door: secrets in the answer's headers swapped for the stand-in");
        // A page to the person gets their script first in its `<head>`.
        let page = said(&parts.headers, header::CONTENT_TYPE).to_ascii_lowercase();
        let script = (s.answer.script.as_deref())
            .filter(|_| get && page.starts_with("text/html") && parts.status == StatusCode::OK)
            .map(|script| {
                let policy = said(&parts.headers, header::CONTENT_SECURITY_POLICY);
                let (tag, runs) = tag(script, &policy);
                if !runs {
                    tracing::warn!(target: "door", lend = %lend, ?policy,
                        "door: the page's script policy runs no inline script, so the person's script is put in it and will not run");
                }
                tracing::debug!(target: "door", %actor, lend = %lend, "door: a page to the person, their script put first in it, kept by no cache");
                parts.headers.remove(header::ETAG);
                parts.headers.remove(header::LAST_MODIFIED);
                let no_store = HeaderValue::from_static("no-store");
                parts.headers.insert(header::CACHE_CONTROL, no_store);
                tag
            });
        // Outward, a textual body is read for every secret held for the lend.
        if parts.status == StatusCode::SWITCHING_PROTOCOLS || !textual(&parts.headers) {
            return Ok(None);
        }
        if compressed(&parts.headers) {
            tracing::warn!(target: "door", lend = %lend, encoding = ?parts.headers.get(header::CONTENT_ENCODING),
                "door: refused — a textual answer came compressed, so it cannot be read for a secret, and is not sent on");
            return Err(super::LendRefusal::SignIn.say(
                StatusCode::BAD_GATEWAY,
                format!("{lend} answered in a form the door cannot read"),
            ));
        }
        tracing::debug!(target: "door", lend = %lend, "door: a textual answer, its body read for every secret held for the lend");
        parts.headers.remove(header::CONTENT_LENGTH);
        Ok(Some((s.secrets, script)))
    }
}

/// Which answer: a lend, its sign-in's port, and a person's key.
type Whose = (LendName, NonZeroU16, super::PersonKey);

/// Who waits on a person's signed-in self.
type Waiter = oneshot::Sender<Result<Signed, Refusal>>;

enum Ask {
    Of(Whose, super::Identity, Waiter),
    /// A sign-in's answer for a person, asked off the task.
    Asked(Whose, Result<Answer, Refusal>),
    Forget(Whose, Secret),
}

/// Every person's sign-in answer, owned by one task and told by messages
/// (RUST_ARCH 3, 4), as [`super::Using`] is. Each ask goes out on its own,
/// so one person's slow sign-in holds nobody else's request (#88).
#[derive(Clone)]
pub struct SignIns(mpsc::UnboundedSender<Ask>);

impl SignIns {
    /// The task that keeps the answers, ending when the last handle drops.
    pub fn spawn() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Ask>();
        let back = tx.downgrade();
        tokio::spawn(async move {
            let mut kept: HashMap<Whose, Answer> = HashMap::new();
            // Each ask out now, and who waits on its answer.
            let mut out: HashMap<Whose, Vec<Waiter>> = HashMap::new();
            while let Some(ask) = rx.recv().await {
                match ask {
                    Ask::Of(whose, identity, told) => {
                        let (lend, port, actor) = &whose;
                        if let Some(a) = kept.get(&whose) {
                            tracing::debug!(target: "door", lend = %lend, %actor, "door: sign-in — the person's kept answer");
                            let _ = told.send(Ok(signed(&kept, lend, a.clone())));
                        } else if let Some(waiting) = out.get_mut(&whose) {
                            tracing::debug!(target: "door", lend = %lend, %actor, "door: sign-in — the person's ask is out, so this request waits on it");
                            waiting.push(told);
                        } else {
                            let (port, lend, back) = (*port, lend.clone(), back.clone());
                            out.insert(whose.clone(), vec![told]);
                            tokio::spawn(async move {
                                let found = ask_of(port, identity, &lend).await;
                                if let Some(tx) = back.upgrade() {
                                    let _ = tx.send(Ask::Asked(whose, found));
                                }
                            });
                        }
                    }
                    Ask::Asked(whose, found) => {
                        let (lend, _, actor) = &whose;
                        let waiting = out.remove(&whose).unwrap_or_default();
                        if let Ok(a) = &found {
                            tracing::info!(target: "door", lend = %lend, %actor, requests = waiting.len(),
                                "door: sign-in — asked for the person on their first request, and kept");
                            kept.insert(whose.clone(), a.clone());
                        }
                        for told in waiting {
                            let _ = told.send(found.clone().map(|a| signed(&kept, lend, a)));
                        }
                    }
                    Ask::Forget(whose, secret) => {
                        if kept.get(&whose).is_some_and(|a| a.secret == secret) {
                            tracing::info!(target: "door", lend = %whose.0, actor = %whose.2,
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
        actor: &super::PersonKey,
        identity: super::Identity,
    ) -> Result<Signed, Refusal> {
        let (answer, told) = oneshot::channel();
        let whose = (lend.clone(), port, actor.clone());
        self.0
            .send(Ask::Of(whose, identity, answer))
            .map_err(|_| Refusal::Gone)?;
        told.await.map_err(|_| Refusal::Gone)?
    }

    /// Forget `actor`'s answer for `lend`, if it is still the one holding
    /// `secret`.
    pub(crate) fn forget(
        &self,
        lend: &LendName,
        port: NonZeroU16,
        actor: &super::PersonKey,
        secret: &Secret,
    ) {
        let whose = (lend.clone(), port, actor.clone());
        let _ = self.0.send(Ask::Forget(whose, secret.clone()));
    }
}

/// A person signed in to `lend` by `answer`, with every secret kept for it.
fn signed(kept: &HashMap<Whose, Answer>, lend: &LendName, answer: Answer) -> Signed {
    let secrets = kept.iter().filter(|((l, _, _), _)| l == lend);
    let secrets = secrets.map(|(_, a)| a.secret.clone()).collect();
    Signed { answer, secrets }
}

/// Ask the sign-in at `port` for the person `identity` names.
async fn ask_of(
    port: NonZeroU16,
    identity: super::Identity,
    lend: &LendName,
) -> Result<Answer, Refusal> {
    let mut request = axum::http::Request::post("/")
        .header(header::HOST, format!("127.0.0.1:{port}"))
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("a fixed request is one");
    request.headers_mut().extend(identity);
    let stand_in = HeaderValue::from_static(STAND_IN);
    (request.headers_mut()).insert(HeaderName::from_static(STAND_IN_HEADER), stand_in);
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

/// How much of a page is read for where its script goes; the rest streams.
const PAGE_READ: usize = 64 * 1024;

/// Where a script goes first in `page`, as `web/lend.js` puts its shim:
/// after the first `<head…>`, else `<html…>`, else `<!doctype…>`, else at
/// the start. `None` while a `<head…>` may yet come, unless `all` of the
/// page that is read is here.
pub(crate) fn first_in(page: &[u8], all: bool) -> Option<usize> {
    let lower = page.to_ascii_lowercase();
    let after = |tag: &[u8]| {
        let open = lower.windows(tag.len()).position(|w| w == tag)?;
        let close = lower[open..].iter().position(|&b| b == b'>')?;
        Some(open + close + 1)
    };
    let fallback = || after(b"<html").or_else(|| after(b"<!doctype")).unwrap_or(0);
    after(b"<head").or_else(|| all.then(fallback))
}

/// `script` in the door's `<script>` for a page whose answer's
/// `Content-Security-Policy` is `policy`, carrying its nonce when it names
/// one, as lend.js's shim does (`:250-257`); and whether the policy runs an
/// inline script, by nonce or `'unsafe-inline'` where it says what scripts
/// run.
pub(crate) fn tag(script: &str, policy: &str) -> (Vec<u8>, bool) {
    let nonce = (policy.split("'nonce-").nth(1))
        .and_then(|n| n.split_once('\''))
        .map(|(n, _)| n);
    let lower = policy.to_ascii_lowercase();
    let says = |d| lower.split(';').map(str::trim).find(|p| p.starts_with(d));
    let scripts = says("script-src ").or_else(|| says("default-src "));
    let runs = nonce.is_some() || scripts.is_none_or(|d| d.contains("'unsafe-inline'"));
    let open = nonce.map_or("<script>".into(), |n| format!("<script nonce=\"{n}\">"));
    (format!("{open}{script}</script>").into_bytes(), runs)
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
    let t = said(headers, header::CONTENT_TYPE).to_ascii_lowercase();
    let t = t.split(';').next().unwrap_or_default().trim();
    t.is_empty()
        || t.starts_with("text/")
        || ["json", "xml", "mpegurl", "javascript"]
            .iter()
            .any(|e| t.ends_with(e))
}

/// What `headers` say as `name`, the first of it; empty when they say none.
pub(crate) fn said(headers: &HeaderMap, name: HeaderName) -> String {
    String::from_utf8_lossy(headers.get(name).map(|v| v.as_bytes()).unwrap_or_default())
        .into_owned()
}

/// Whether an answer's body is compressed, so cannot be read for a secret.
pub(crate) fn compressed(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_ENCODING)
        .is_some_and(|e| !e.as_bytes().eq_ignore_ascii_case(b"identity"))
}

/// A textual answer's body, each secret swapped for the stand-in as it
/// streams, a secret split across chunks found by [`swapped`]'s holding
/// back, and a page's `script` put first in it ([`first_in`]), nothing sent
/// until it is. A trailer cannot be swapped, so is not carried.
pub(crate) struct Swapping<B> {
    inner: B,
    secrets: Vec<Secret>,
    held: Vec<u8>,
    done: bool,
    script: Option<Vec<u8>>,
}

impl<B> Swapping<B> {
    pub(crate) fn new(inner: B, secrets: Vec<Secret>, script: Option<Vec<u8>>) -> Self {
        let (held, done) = (Vec::new(), false);
        Self {
            inner,
            secrets,
            held,
            done,
            script,
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
            if let Some(script) = &this.script {
                let Some(at) = first_in(&this.held, end || this.held.len() >= PAGE_READ) else {
                    continue;
                };
                this.held.splice(at..at, script.iter().copied());
                this.script = None;
            }
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
