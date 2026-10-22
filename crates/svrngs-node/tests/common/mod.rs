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
