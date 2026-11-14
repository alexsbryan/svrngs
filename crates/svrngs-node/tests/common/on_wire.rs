//! Computers met on one [`Wire`] in this process (docs/CHECKS.md, harness
//! 3): no endpoint, no relay, no socket. Each node's record API is asked
//! of its routes by `oneshot`, as a program on its computer asks it at
//! `http://<its address>`: [`http`] stands in for that client, so a test
//! keeps its requests as it wrote them and reaches no port. That the API
//! is served on its port at all stays record_api.rs's and the CLI's.
#![allow(dead_code, reason = "each test binary that loads this asks some of it")]

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use svrngs_node::iroh::{Alpn, PublicKey, Unsent};
use svrngs_node::node::{Node, Wire};
use svrngs_node::redeem::SeedText;
use svrngs_node::SigningKey;
use tower::ServiceExt;

thread_local! {
    /// Each started node's routes by the address its API names, and the
    /// wire they meet on: one test's, since each `#[tokio::test]` runs on a
    /// thread of its own.
    static APIS: RefCell<HashMap<SocketAddr, (PublicKey, axum::Router)>> = RefCell::default();
    static WIRE: Wire = Wire::default();
    /// Each [`bridge_as`]'s address: the key it asks as, of whom, on what.
    static BRIDGES: RefCell<HashMap<SocketAddr, Bridged>> = RefCell::default();
}

type Bridged = (SigningKey, PublicKey, Alpn);

/// The computer `seed` signs as, over `dir`, on this test's wire, its
/// seeds read from each group's `seed.json` as the command keeps them.
pub async fn start(seed: u8, dir: &Path) -> Node {
    let wire = WIRE.with(Clone::clone);
    on(&wire, SigningKey::from_bytes(&[seed; 32]), dir, seed_file).await
}

fn seed_file(dir: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(dir.join("seed.json")) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// `key`'s node over `dir` on `wire`, its API asked by [`http`].
pub async fn on(wire: &Wire, key: SigningKey, dir: &Path, seeds: SeedText) -> Node {
    let node = Node::on_wire(wire, dir.to_path_buf(), key, seeds)
        .await
        .expect("a node starts on the wire");
    served(&node);
    node
}

/// common/mod.rs `proved` with `node` asked in-process: the origin key
/// the home page (`lend` none) or `lend`'s origin enrolls, as `opened_at`
/// enrolls it, and the lines `prove` signs with it.
pub async fn proved(
    node: &Node,
    lend: Option<&str>,
    method: &str,
    path: &str,
    body: &str,
) -> HeaderMap {
    let host = format!(
        "{}.localhost:{}",
        lend.unwrap_or("svrngs"),
        node.api_addr().port()
    );
    if path.starts_with("/svrngs-as/") {
        return HeaderMap::new();
    }
    let key = opened(node, lend).await;
    let proof = signed(node, &key, (method, &host, path), body, false).await;
    let mut headers = HeaderMap::new();
    for (name, value) in proof.lines().filter_map(|l| l.split_once(':')) {
        let name = axum::http::HeaderName::from_bytes(name.as_bytes()).unwrap();
        headers.insert(name, value.trim().parse().unwrap());
    }
    headers
}

/// common/mod.rs `prove` with `node` asked in-process: the boot challenge
/// read of its routes at `host`, then the lines common/mod.rs `signed`
/// signs with `key`.
async fn signed(
    node: &Node,
    key: &[u8],
    (method, host, path): (&str, &str, &str),
    body: &str,
    navigate: bool,
) -> String {
    let epoch = (http().get(format!("http://{}/svrngs-session-epoch", node.api_addr())))
        .header("host", host)
        .send()
        .await
        .unwrap();
    let epoch = epoch.text().await.unwrap();
    crate::common::signed(key, (method, host, path), body, navigate, &epoch)
}

/// common/mod.rs `Browser` with its node asked in-process: the same read
/// capability and token, each origin's key enrolled once by [`opened`],
/// each request's lines signed by [`signed`].
pub struct Browser<'a> {
    node: &'a Node,
    of: crate::common::Browser<'a>,
    held: RefCell<HashMap<String, Vec<u8>>>,
}

impl<'a> Browser<'a> {
    pub fn of(node: &'a Node) -> Self {
        let (of, held) = (crate::common::Browser::of(node), RefCell::default());
        Self { node, of, held }
    }

    /// The address its API is asked at.
    pub fn api(&self) -> SocketAddr {
        self.node.api_addr()
    }

    /// common/mod.rs `Browser::bearer`.
    pub fn bearer(&self) -> String {
        self.of.bearer()
    }

    /// common/mod.rs `Browser::path`.
    pub fn path(&self, host: &str, path: &str, extra: &str) -> String {
        self.of.path(host, path, extra)
    }

    /// common/mod.rs `Browser::shown`: a page's proof, else the token's line.
    pub async fn shown(
        &self,
        method: &str,
        host: &str,
        path: &str,
        extra: &str,
        body: &str,
    ) -> String {
        match extra.contains("Sec-Fetch-Site") {
            true => self.proof(method, host, path, extra, body).await,
            false => self.bearer(),
        }
    }

    /// common/mod.rs `Browser::proof`, its node asked in-process.
    pub async fn proof(
        &self,
        method: &str,
        host: &str,
        path: &str,
        extra: &str,
        body: &str,
    ) -> String {
        let name = host.split_once(".localhost").map(|(n, _)| n);
        let Some(name) =
            name.filter(|_| extra.contains("Sec-Fetch-Site") && !path.starts_with("/svrngs-as/"))
        else {
            return String::new();
        };
        let held = self.held.borrow().get(name).cloned();
        let key = match held {
            Some(key) => key,
            None => {
                let key = opened(self.node, Some(name).filter(|n| *n != "svrngs")).await;
                self.held.borrow_mut().insert(name.into(), key.clone());
                key
            }
        };
        let navigate = extra.contains("Sec-Fetch-Mode: navigate");
        signed(self.node, &key, (method, host, path), body, navigate).await
    }
}

/// common/mod.rs `opened_at`, its two asks of `node`'s routes: `/v1/open`
/// with the token, then the code redeemed by a POST from that origin.
async fn opened(node: &Node, lend: Option<&str>) -> Vec<u8> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let api = node.api_addr();
    let body = lend.map_or(serde_json::json!({}), |l| serde_json::json!({ "lend": l }));
    let open = http().post(format!("http://{api}/v1/open"));
    let open = open.bearer_auth(node.token().trim()).json(&body).send();
    let open: serde_json::Value = open.await.unwrap().json().await.unwrap();
    let url = open["open"].as_str().expect("an open's address");
    let (host, path) = url.trim_start_matches("http://").split_once('/').unwrap();
    let redeem = (http().post(format!("http://{api}/{path}")))
        .header("host", host)
        .header("origin", format!("http://{host}"))
        .header("sec-fetch-site", "same-origin");
    let answer = redeem.send().await.unwrap();
    assert!(
        answer.status().is_success(),
        "enrollment answered {}",
        answer.status()
    );
    let answer: serde_json::Value = answer.json().await.unwrap();
    URL_SAFE_NO_PAD
        .decode(answer["key"].as_str().expect("enrollment's origin key"))
        .unwrap()
}

/// `node`'s service at `port`, answered in this process ([`Wire::listen`],
/// harness 2): each connection its door opens there handed to `answer`, as
/// a listener's accepted socket is; no port is bound.
pub fn serve<F, Fut>(node: &Node, port: u16, answer: F)
where
    F: Fn(tokio::io::DuplexStream) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let mut at = listen(node.net().id(), port);
    tokio::spawn(async move {
        while let Some(conn) = at.recv().await {
            tokio::spawn(answer(conn));
        }
    });
}

/// The raw HTTP/1 request `text` asked of the routes at `api`, as a
/// program's socket sends it: the answer's head, its status line and a
/// line per header up to the blank line, and its body whole.
pub async fn raw(api: SocketAddr, text: &str) -> (String, String) {
    let answer = asked_raw(api, text)
        .send()
        .await
        .expect("a node's own routes answer");
    let said = head_of(answer.status, &answer.headers);
    (said, String::from_utf8_lossy(&answer.body).into_owned())
}

/// [`raw`] with the answer's body still to read, a stream held: its head
/// as [`raw`] says it; or the acceptor's refusal.
pub async fn raw_held(at: SocketAddr, text: &str) -> Result<(String, axum::body::Body), Unsent> {
    let (head, body) = asked_raw(at, text).sent().await?.into_parts();
    Ok((head_of(head.status, &head.headers), body))
}

/// The request a raw request's `text` is, asked at `at`.
fn asked_raw(at: SocketAddr, text: &str) -> Asked {
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((text, ""));
    let mut lines = head.split("\r\n");
    let mut line = lines.next().unwrap().split_whitespace();
    let (method, path) = (line.next().unwrap(), line.next().unwrap());
    let mut asked = http().request(method.parse().unwrap(), format!("http://{at}{path}"));
    for (name, value) in lines.filter_map(|l| l.split_once(':')) {
        let name = axum::http::HeaderName::from_bytes(name.trim().as_bytes()).unwrap();
        asked.headers.insert(name, value.trim().parse().unwrap());
    }
    asked.body(body)
}

/// An answer's status line and a line per header, to the blank line.
fn head_of(status: StatusCode, headers: &HeaderMap) -> String {
    let mut said = format!(
        "HTTP/1.1 {} {}\r\n",
        status.as_u16(),
        status.canonical_reason().unwrap_or_default()
    );
    for (name, value) in headers {
        said += &format!("{name}: {}\r\n", value.to_str().unwrap_or_default());
    }
    said + "\r\n"
}

/// No path on this test's wire from `from`'s computer to `to`'s, as when
/// `from` knows no address of `to` ([`Wire::part`]).
pub fn part(from: &Node, to: &Node) {
    WIRE.with(|w| w.part(from.net().id(), to.net().id()));
}

/// `key`'s computer on this test's wire answers no dial ([`Wire::asleep`]).
pub fn asleep(key: PublicKey) {
    WIRE.with(|w| w.asleep(key));
}

/// `from` asks `holder`'s provider for `hash` whole, into a store of its
/// own under `dir`: the payload's bytes, or why none came.
pub async fn get(
    from: &Node,
    holder: &Node,
    hash: iroh_blobs::Hash,
    dir: &Path,
) -> Result<u64, String> {
    let store = iroh_blobs::store::fs::FsStore::load(dir).await.unwrap();
    let holder = svrngs_node::iroh::Contact {
        key: holder.net().id(),
        relays: Default::default(),
        iroh_direct_addrs: Vec::new(),
    };
    let request = iroh_blobs::protocol::GetRequest::blob(hash);
    let get = from.net().get(&holder, &store, request).await;
    let get = get.map_err(|e| match e {
        svrngs_node::iroh::Unsent::NotAdmitted => "the dial: not admitted".to_string(),
        e => format!("the dial: {e}"),
    })?;
    let got = tokio::time::timeout(Duration::from_secs(10), get).await;
    let got = got.map_err(|_| "no answer".to_string())?;
    got.map(|s| s.payload_bytes_read)
        .map_err(|e| format!("{e:?}"))
}

/// The client a program on a computer asks its record API with.
pub fn http() -> Client {
    Client
}

pub struct Client;

impl Client {
    pub fn get(&self, url: impl AsRef<str>) -> Asked {
        Asked::new(Method::GET, url.as_ref())
    }
    pub fn post(&self, url: impl AsRef<str>) -> Asked {
        Asked::new(Method::POST, url.as_ref())
    }
    pub fn put(&self, url: impl AsRef<str>) -> Asked {
        Asked::new(Method::PUT, url.as_ref())
    }
    pub fn delete(&self, url: impl AsRef<str>) -> Asked {
        Asked::new(Method::DELETE, url.as_ref())
    }
    pub fn request(&self, method: Method, url: impl AsRef<str>) -> Asked {
        Asked::new(method, url.as_ref())
    }
}

/// `node`'s routes asked by [`http`] at its API's address, a node started
/// over QUIC included.
pub fn served(node: &Node) {
    let (at, key) = (node.api_addr(), node.net().id());
    let was = APIS.with(|a| a.borrow_mut().insert(at, (key, node.api_routes())));
    // The same key again is its computer restarted; another's would be
    // asked in its place.
    if let Some((other, _)) = was.filter(|(k, _)| *k != key) {
        panic!("{key} and {other} both answer at {at}");
    }
}

/// An address [`http`] asks `to`'s routes at on `alpn` as `key` dials it,
/// as common/mod.rs `bridge_as` splices a port to a QUIC stream: the
/// request decided by `to`'s acceptor, as any dialer's, then its router's.
pub async fn bridge_as(key: &SigningKey, to: &Node, alpn: Alpn) -> Bridge {
    BRIDGES.with(|b| {
        let mut b = b.borrow_mut();
        let at = SocketAddr::from(([127, 0, 0, 2], 1 + b.len() as u16));
        b.insert(at, (key.clone(), to.net().id(), alpn));
        Bridge(at)
    })
}

/// Whether `at` is a [`bridge_as`] address on this test's wire.
pub fn bridged(at: SocketAddr) -> bool {
    BRIDGES.with(|b| b.borrow().contains_key(&at))
}

/// `key`'s computer's connections at `port` on this test's wire
/// ([`Wire::listen`]): each one a pipe's end, as a listener accepts.
pub fn listen(
    key: PublicKey,
    port: u16,
) -> tokio::sync::mpsc::UnboundedReceiver<tokio::io::DuplexStream> {
    WIRE.with(|w| w.listen(key, port))
}

/// [`bridge_as`]'s address, read as the bridge's port was.
pub struct Bridge(SocketAddr);

impl Bridge {
    pub fn local_addr(&self) -> SocketAddr {
        self.0
    }
}

/// A request to `http://<address><path>`, built as reqwest builds one.
pub struct Asked {
    method: Method,
    at: SocketAddr,
    path: String,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl Asked {
    fn new(method: Method, url: &str) -> Self {
        let rest = url.strip_prefix("http://").expect("an http URL");
        let (at, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_str(at).unwrap());
        Self {
            method,
            // A name such as `svrngs.localhost` resolves to loopback, as
            // the page's names do.
            at: (at.parse().ok())
                .or_else(|| {
                    Some(SocketAddr::from((
                        [127, 0, 0, 1],
                        at.rsplit(':').next()?.parse().ok()?,
                    )))
                })
                .expect("an API's address"),
            path: if path.is_empty() {
                "/".into()
            } else {
                path.into()
            },
            headers,
            body: Vec::new(),
        }
    }
    pub fn bearer_auth(self, token: impl std::fmt::Display) -> Self {
        self.header("authorization", format!("Bearer {token}"))
    }
    pub fn header(mut self, name: &str, value: impl AsRef<str>) -> Self {
        let name = axum::http::HeaderName::from_bytes(name.as_bytes()).unwrap();
        self.headers
            .insert(name, HeaderValue::from_str(value.as_ref()).unwrap());
        self
    }
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        for (name, value) in &headers {
            self.headers.insert(name, value.clone());
        }
        self
    }
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }
    pub fn basic_auth(self, id: &str, secret: Option<&str>) -> Self {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let pair = STANDARD.encode(format!("{id}:{}", secret.unwrap_or_default()));
        self.header("authorization", format!("Basic {pair}"))
    }
    pub fn form(self, pairs: &[(&str, &str)]) -> Self {
        let body = (pairs.iter())
            .map(|(k, v)| format!("{}={}", encoded(k), encoded(v)))
            .collect::<Vec<_>>()
            .join("&");
        let kind = "application/x-www-form-urlencoded";
        self.header("content-type", kind).body(body)
    }
    pub fn json(self, value: &impl serde::Serialize) -> Self {
        let body = serde_json::to_vec(value).unwrap();
        self.header("content-type", "application/json").body(body)
    }
    /// Asked of the node's routes, from a program on its computer; or, at
    /// a [`bridge_as`] address, of the computer it reaches, as its key.
    /// Refused at the acceptor, it is the error a closed connection is.
    pub async fn send(self) -> Result<Answer, Unsent> {
        let (head, body) = self.sent().await?.into_parts();
        let body = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        Ok(Answer {
            status: head.status,
            headers: head.headers,
            body,
        })
    }

    /// [`Asked::send`], its answer's body still to read: a stream held.
    pub async fn sent(self) -> Result<axum::response::Response, Unsent> {
        let mut req = axum::http::Request::builder()
            .method(self.method)
            .uri(&self.path);
        *req.headers_mut().unwrap() = self.headers;
        let mut req = req.body(axum::body::Body::from(self.body)).unwrap();
        let bridged = BRIDGES.with(|b| b.borrow().get(&self.at).cloned());
        if let Some((key, to, alpn)) = bridged {
            let wire = WIRE.with(Clone::clone);
            return wire.ask_as(&key, &to, alpn, req).await;
        }
        let routes = APIS.with(|a| a.borrow().get(&self.at).map(|(_, r)| r.clone()));
        let routes = routes.unwrap_or_else(|| panic!("no node's API at {}", self.at));
        let caller = SocketAddr::from(([127, 0, 0, 1], 50_000));
        req.extensions_mut()
            .insert(axum::extract::ConnectInfo(caller));
        Ok(routes
            .oneshot(req)
            .await
            .unwrap_or_else(|never| match never {}))
    }
}

/// `text` percent-encoded as a form's value, as reqwest encodes it.
fn encoded(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'*' => {
                (b as char).to_string()
            }
            b' ' => "+".into(),
            b => format!("%{b:02X}"),
        })
        .collect()
}

/// An answer read whole, as reqwest's response is read.
pub struct Answer {
    status: StatusCode,
    headers: HeaderMap,
    body: axum::body::Bytes,
}

impl Answer {
    pub fn status(&self) -> StatusCode {
        self.status
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    pub async fn text(self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.to_vec())
    }
    pub async fn bytes(self) -> Result<axum::body::Bytes, std::convert::Infallible> {
        Ok(self.body)
    }
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}
