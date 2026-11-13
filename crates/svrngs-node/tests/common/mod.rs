//! What the integration tests share: one copy, where accept.rs and
//! node_sync.rs each wrote it out (REVIEW-audit-u1-auto-1).

use std::net::SocketAddr;

use svrngs_node::iroh::Endpoint;

/// iroh binds the wildcard; rewrite to loopback so the address is dialable
/// in-process.
pub fn loopback_sockets(endpoint: &Endpoint) -> Vec<SocketAddr> {
    endpoint
        .bound_sockets()
        .into_iter()
        .map(|mut a| {
            if a.ip().is_unspecified() {
                let ip = if a.is_ipv4() { "127.0.0.1" } else { "::1" };
                a.set_ip(ip.parse().unwrap());
            }
            a
        })
        .collect()
}

/// A computer that never answers, as an asleep one in the field: a socket
/// bound on loopback that nobody reads, so a dial to it is neither refused
/// nor answered and waits out its own timeout. Held for as long as it
/// should stay asleep.
#[allow(
    dead_code,
    reason = "each test binary compiles this file; only some sleep a computer"
)]
pub fn never_answers() -> (std::net::UdpSocket, Vec<SocketAddr>) {
    let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let at = socket.local_addr().unwrap();
    (socket, vec![at])
}

/// A port nothing listens on, as a stopped program's: bound, read and let
/// go, so a connection to it is refused. One copy, where home.rs and
/// lend_origin.rs each wrote it out (REVIEW-audit-v1ui-auto-3).
#[allow(
    dead_code,
    reason = "each test binary compiles this file; only some stop a program"
)]
pub fn stopped() -> SocketAddr {
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    free.local_addr().unwrap()
}

/// A port on this computer's loopback spliced to `to`'s door on `alpn`
/// ([`svrngs_node::node::splice`]), dialling as `key`: a member's, or a pass
/// holder's.
#[allow(
    dead_code,
    reason = "each test binary compiles this file; only some reach a door"
)]
pub async fn bridge_as(
    key: &svrngs_node::SigningKey,
    to: &svrngs_node::node::Node,
    alpn: svrngs_core::alpn::Alpn,
) -> Bridge {
    use iroh::endpoint::Builder as EndpointBuilder;
    use svrngs_node::guest::ring_crypto_provider;
    use svrngs_node::iroh::{EndpointAddr, SecretKey};
    let dialling = EndpointBuilder::empty()
        .crypto_provider(ring_crypto_provider())
        .secret_key(SecretKey::from_bytes(&key.to_bytes()))
        .bind()
        .await
        .unwrap();
    let mut target = EndpointAddr::new(to.endpoint().id());
    for sock in loopback_sockets(to.endpoint()) {
        target = target.with_ip_addr(sock);
    }
    Bridge(svrngs_node::node::splice(dialling, target, alpn.bytes()).await)
}

/// [`bridge_as`]'s port, read as the bridge's was.
#[allow(
    dead_code,
    reason = "each test binary compiles this file; only some reach a door"
)]
pub struct Bridge(std::net::SocketAddr);

#[allow(
    dead_code,
    reason = "each test binary compiles this file; only some reach a door"
)]
impl Bridge {
    pub fn local_addr(&self) -> std::net::SocketAddr {
        self.0
    }
}

/// `len` bytes no two runs agree on by accident: BLAKE3's output stream.
#[allow(
    dead_code,
    reason = "each test binary compiles this file; only the synced folder's write files"
)]
pub fn noise(len: usize, salt: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; len];
    blake3::Hasher::new()
        .update(salt)
        .finalize_xof()
        .fill(&mut out);
    out
}

/// What a test's subscriber writes, kept to be read: one copy, where
/// folders.rs and home.rs each wrote it out (REVIEW-audit-events-auto-2).
#[derive(Clone, Default)]
#[allow(
    dead_code,
    clippy::disallowed_types,
    reason = "a test's trace sink; each test binary compiles this file, only some read their trace"
)]
pub struct Said(pub std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for Said {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// What `filter` traces while the guards live, kept to be read: one
/// copy, where three tests in home.rs each wrote it out
/// (REVIEW-audit-v1ui-auto-2), moved here for carried.rs's (v1ui-polish (c)).
/// A second dispatcher kept alive: with one, tracing-core 0.1.36
/// (callsite.rs:545, `has_just_one`) asks only the registering thread's,
/// so a callsite another test's thread reaches first is cached unheard.
#[allow(
    dead_code,
    reason = "each test binary compiles this file, only some read their trace"
)]
pub fn heard(filter: &str) -> (Said, (tracing::subscriber::DefaultGuard, tracing::Dispatch)) {
    let said = Said::default();
    let writer = said.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);
    let two = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
    (said, (guard, two))
}

/// Protocol fixture: enroll an origin key and prove each request, or use an
/// independent read capability when another app is named. Browser-client
/// eligibility is tested in JavaScript; these tests exercise the node's checks.
#[allow(
    dead_code,
    clippy::disallowed_types,
    reason = "each test binary compiles this file; only some model a browser, whose enrolled keys one test's lock keeps"
)]
pub struct Browser<'a> {
    node: &'a svrngs_node::node::Node,
    api: SocketAddr,
    token: String,
    held: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

#[allow(
    dead_code,
    clippy::disallowed_types,
    reason = "each test binary compiles this file; only some model a browser, whose enrolled keys one test's lock keeps"
)]
impl<'a> Browser<'a> {
    pub fn of(node: &'a svrngs_node::node::Node) -> Self {
        let (api, token) = (node.api_addr(), node.token().trim().to_string());
        let held = std::sync::Mutex::default();
        Self {
            node,
            api,
            token,
            held,
        }
    }

    /// The listener it asks.
    pub fn api(&self) -> SocketAddr {
        self.api
    }

    /// The token's line, which a program on this computer sends.
    pub fn bearer(&self) -> String {
        format!("Authorization: Bearer {}\r\n", self.token)
    }

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

    /// Exercise reads with their own credential, never the target's origin key.
    pub fn path(&self, host: &str, path: &str, extra: &str) -> String {
        use svrngs_core::pass::Label;
        let target = host
            .split_once(".localhost")
            .and_then(|(n, _)| Label::parse(n));
        let framed = extra.contains("Sec-Fetch-Dest: iframe");
        let header = if framed { "referer" } else { "origin" };
        let from = extra
            .lines()
            .filter_map(|l| l.split_once(':'))
            .find(|(n, _)| n.eq_ignore_ascii_case(header));
        let from = from.and_then(|(_, v)| reqwest::Url::parse(v.trim()).ok());
        let app = from
            .filter(|u| u.scheme() == "http" && u.port() == Some(self.api.port()))
            .and_then(|u| Label::parse(u.host_str()?.strip_suffix(".localhost")?));
        match (app, target) {
            (Some(app), Some(target)) if app != target => format!(
                "/svrngs-as/{}{path}",
                self.node.read_capability(&app, &target)
            ),
            _ => path.into(),
        }
    }

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
        let held = self.held.lock().unwrap().get(name).cloned();
        let key = match held {
            Some(c) => c,
            None => {
                let lend = Some(name).filter(|n| *n != "svrngs");
                let c = opened_at(self.api, &self.token, lend).await;
                self.held.lock().unwrap().insert(name.into(), c.clone());
                c
            }
        };
        let navigate = extra.contains("Sec-Fetch-Mode: navigate");
        prove(self.api, &key, method, host, path, body, navigate).await
    }
}

/// The headers home/session-worker.js adds to a request from `host` (a
/// `<name>.localhost:<port>` origin) whose enrolled key is `key`, as it
/// computes them (session.js `proof`): the boot challenge read from
/// `/svrngs-session-epoch`, a timestamp and single-use nonce, the body's
/// SHA-256, and the write intent, `no` for a tab's navigation. The one
/// copy every Rust test that models a browser without one uses (the node's
/// tests here, and the scenes' people.rs by path).
#[allow(
    dead_code,
    clippy::disallowed_methods,
    reason = "each test binary compiles this file; a protocol-test adapter draws a fresh nonce and reads the clock"
)]
pub async fn prove(
    api: SocketAddr,
    key: &[u8],
    method: &str,
    host: &str,
    path: &str,
    body: &str,
    navigate: bool,
) -> String {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let epoch = client
        .get(format!("http://{api}/svrngs-session-epoch"))
        .header("host", host)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    signed(key, (method, host, path), body, navigate, &epoch)
}

/// [`prove`]'s lines once the boot challenge `epoch` is read: what a test
/// whose node is asked in-process (on_wire.rs `proved`) signs too.
#[allow(
    dead_code,
    clippy::disallowed_methods,
    reason = "each test binary compiles this file; a protocol-test adapter draws a fresh nonce and reads the clock"
)]
pub fn signed(
    key: &[u8],
    (method, host, path): (&str, &str, &str),
    body: &str,
    navigate: bool,
    epoch: &str,
) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut nonce = [0; 16];
    getrandom::fill(&mut nonce).unwrap();
    let stamp = format!("{now}.{}", hex::encode(nonce));
    let digest = hex::encode(ring::digest::digest(&ring::digest::SHA256, body.as_bytes()).as_ref());
    let intent = if navigate { "no" } else { "yes" };
    let message = format!("{host}\n{method}\n{path}\n{stamp}\n{digest}\n{intent}\n{epoch}");
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    let tag = URL_SAFE_NO_PAD.encode(ring::hmac::sign(&key, message.as_bytes()));
    format!("X-Svrngs-Browser: {stamp}.{tag}\r\nX-Svrngs-Body: {digest}\r\nX-Svrngs-Intent: {intent}\r\n")
}

/// [`prove`]'s lines as a reqwest caller sends them.
#[allow(
    dead_code,
    reason = "each test binary compiles this file; only some send by reqwest"
)]
pub fn headers(proof: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    for (name, value) in proof.lines().filter_map(|l| l.split_once(':')) {
        headers.insert(
            reqwest::header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
            value.trim().parse().unwrap(),
        );
    }
    headers
}

/// A proved own-origin request, including its body digest, for reqwest callers.
#[allow(
    dead_code,
    reason = "each test binary compiles this file; only some model a browser"
)]
pub async fn proved(
    node: &svrngs_node::node::Node,
    lend: Option<&str>,
    method: &str,
    path: &str,
    body: &str,
) -> reqwest::header::HeaderMap {
    let browser = Browser::of(node);
    let host = format!(
        "{}.localhost:{}",
        lend.unwrap_or("svrngs"),
        node.api_addr().port()
    );
    let proof = browser
        .proof(method, &host, path, "Sec-Fetch-Site: same-origin\r\n", body)
        .await;
    headers(&proof)
}

/// The key an open enrolls at the home page (`lend` none) or the lend
/// `lend`'s origin: `/v1/open`, asked with the token as `svrngs open` asks
/// it, its code redeemed by a POST from that origin, as home/session.js
/// redeems it (session.rs `redeem`).
#[allow(
    dead_code,
    reason = "each test binary compiles this file; only some model a browser"
)]
pub async fn opened_at(api: SocketAddr, token: &str, lend: Option<&str>) -> Vec<u8> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let client = reqwest::Client::builder().no_proxy();
    let client = client.redirect(reqwest::redirect::Policy::none());
    let client = client.build().unwrap();
    let body = lend.map_or(serde_json::json!({}), |l| serde_json::json!({ "lend": l }));
    let open = client.post(format!("http://{api}/v1/open"));
    // By hand, not reqwest's `json`, which the command's tests (scenes,
    // people.rs) build without, since they include this file by path.
    let open = open
        .bearer_auth(token)
        .header("content-type", "application/json");
    let open = open.body(body.to_string()).send().await.unwrap();
    let open: serde_json::Value = serde_json::from_slice(&open.bytes().await.unwrap()).unwrap();
    let url = open["open"].as_str().expect("an open's address");
    let (host, path) = url.trim_start_matches("http://").split_once('/').unwrap();
    let redeem = client.post(format!("http://{api}/{path}"));
    let redeem = redeem
        .header("host", host)
        .header("origin", format!("http://{host}"))
        .header("sec-fetch-site", "same-origin");
    let answer = redeem.send().await.unwrap();
    assert!(
        answer.status().is_success(),
        "enrollment answered {}",
        answer.status()
    );
    let answer: serde_json::Value = serde_json::from_slice(&answer.bytes().await.unwrap()).unwrap();
    URL_SAFE_NO_PAD
        .decode(answer["key"].as_str().expect("enrollment's origin key"))
        .unwrap()
}
