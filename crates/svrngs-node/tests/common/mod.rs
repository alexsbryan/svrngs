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

/// A bridge on this computer's loopback to `to`'s door on `alpn`,
/// dialling as `key`: a member's, or a pass holder's.
#[allow(
    dead_code,
    reason = "each test binary compiles this file; only some bridge to a door"
)]
pub async fn bridge_as(
    key: &svrngs_node::SigningKey,
    to: &svrngs_node::node::Node,
    alpn: svrngs_core::alpn::Alpn,
) -> svrngs_node::iroh::HttpBridge {
    use iroh::endpoint::Builder as EndpointBuilder;
    use svrngs_node::guest::ring_crypto_provider;
    use svrngs_node::iroh::{EndpointAddr, HttpBridge, SecretKey};
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
    HttpBridge::spawn_preferring(dialling, target, alpn.bytes(), None)
        .await
        .unwrap()
}
