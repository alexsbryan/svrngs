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
    let epoch = (http().get(format!("http://{}/svrngs-session-epoch", node.api_addr())))
        .header("host", &host)
        .send()
        .await
        .unwrap();
    let epoch = epoch.text().await.unwrap();
    let proof = crate::common::signed(&key, (method, &host, path), body, false, &epoch);
    let mut headers = HeaderMap::new();
    for (name, value) in proof.lines().filter_map(|l| l.split_once(':')) {
        let name = axum::http::HeaderName::from_bytes(name.as_bytes()).unwrap();
        headers.insert(name, value.trim().parse().unwrap());
    }
    headers
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

/// No path on this test's wire from `from`'s computer to `to`'s, as when
/// `from` knows no address of `to` ([`Wire::part`]).
pub fn part(from: &Node, to: &Node) {
    WIRE.with(|w| w.part(from.net().id(), to.net().id()));
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
    pub fn header(mut self, name: &'static str, value: impl AsRef<str>) -> Self {
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
    pub fn json(self, value: &impl serde::Serialize) -> Self {
        let body = serde_json::to_vec(value).unwrap();
        self.header("content-type", "application/json").body(body)
    }
    /// Asked of the node's routes, from a program on its computer; or, at
    /// a [`bridge_as`] address, of the computer it reaches, as its key.
    /// Refused at the acceptor, it is the error a closed connection is.
    pub async fn send(self) -> Result<Answer, Unsent> {
        let mut req = axum::http::Request::builder()
            .method(self.method)
            .uri(&self.path);
        *req.headers_mut().unwrap() = self.headers;
        let mut req = req.body(axum::body::Body::from(self.body)).unwrap();
        let bridged = BRIDGES.with(|b| b.borrow().get(&self.at).cloned());
        let answer = if let Some((key, to, alpn)) = bridged {
            let wire = WIRE.with(Clone::clone);
            wire.ask_as(&key, &to, alpn, req).await?
        } else {
            let routes = APIS.with(|a| a.borrow().get(&self.at).map(|(_, r)| r.clone()));
            let routes = routes.unwrap_or_else(|| panic!("no node's API at {}", self.at));
            let caller = SocketAddr::from(([127, 0, 0, 1], 50_000));
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(caller));
            routes
                .oneshot(req)
                .await
                .unwrap_or_else(|never| match never {})
        };
        let (head, body) = answer.into_parts();
        let body = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        Ok(Answer {
            status: head.status,
            headers: head.headers,
            body,
        })
    }
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
