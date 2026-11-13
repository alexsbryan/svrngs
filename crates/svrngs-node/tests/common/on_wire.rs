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
use svrngs_node::node::{Node, Wire};
use svrngs_node::redeem::SeedText;
use svrngs_node::SigningKey;
use tower::ServiceExt;

thread_local! {
    /// Each started node's routes by the address its API names, and the
    /// wire they meet on: one test's, since each `#[tokio::test]` runs on a
    /// thread of its own.
    static APIS: RefCell<HashMap<SocketAddr, axum::Router>> = RefCell::default();
    static WIRE: Wire = Wire::default();
}

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
    APIS.with(|a| a.borrow_mut().insert(node.api_addr(), node.api_routes()));
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
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }
    pub fn json(self, value: &impl serde::Serialize) -> Self {
        let body = serde_json::to_vec(value).unwrap();
        self.header("content-type", "application/json").body(body)
    }
    /// Asked of the node's routes, from a program on its computer.
    pub async fn send(self) -> Result<Answer, std::convert::Infallible> {
        let routes = APIS.with(|a| a.borrow().get(&self.at).cloned());
        let routes = routes.unwrap_or_else(|| panic!("no node's API at {}", self.at));
        let mut req = axum::http::Request::builder()
            .method(self.method)
            .uri(&self.path);
        *req.headers_mut().unwrap() = self.headers;
        let mut req = req.body(axum::body::Body::from(self.body)).unwrap();
        let caller = SocketAddr::from(([127, 0, 0, 1], 50_000));
        req.extensions_mut()
            .insert(axum::extract::ConnectInfo(caller));
        let answer = routes
            .oneshot(req)
            .await
            .unwrap_or_else(|never| match never {});
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
