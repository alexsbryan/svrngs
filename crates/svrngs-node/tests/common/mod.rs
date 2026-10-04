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
