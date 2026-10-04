// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/publish_cmd.rs@e1a43bcd2
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/mesh_media/offer.rs@e1a43bcd2
//! `svrngs share` (DESIGN §8, U3's inventory, its publish_cmd and offer
//! row): the verb's shape and the probe, as upstream had them —
//! `resolve_target` from `svrn publish` (publish_cmd.rs:409-434, its tests
//! in `share/tests.rs`) and `probe_origin` from `svrn mesh media offer`
//! (offer.rs:94-110). (svrngs: each is `pub`, so the lib builds before its
//! caller lands, and the imports upstream's modules had are written here.)

use std::net::SocketAddr;
use std::time::Duration;

/// `5000` → `127.0.0.1:5000`; a full `host:port` is taken as given and must be
/// loopback.
///
/// Loopback-only is not a limitation to relax later: the point of publishing
/// is that the app stays bound to this machine and is reached by mesh key. A
/// `0.0.0.0` target would already be on the LAN to anyone, with no identity
/// attached, which is the thing the mesh replaces.
pub fn resolve_target(port: &str) -> Result<SocketAddr, String> {
    if let Ok(p) = port.parse::<u16>() {
        if p == 0 {
            return Err("port 0 is not a port a server listens on".to_string());
        }
        return Ok(([127, 0, 0, 1], p).into());
    }
    let addr: SocketAddr = port
        .parse()
        .map_err(|_| format!("{port:?} is neither a port nor a host:port"))?;
    if !addr.ip().is_loopback() {
        return Err(format!(
            "{addr} is not loopback — publish binds a port ON THIS MACHINE to the mesh. \
             An app on a routable address is already reachable without us, and without \
             any identity attached to the caller."
        ));
    }
    Ok(addr)
}

/// Where a media server listens when nobody said: Jellyfin's HTTP port, then
/// its HTTPS port, on this machine's loopback. The address is the tool's to
/// know, not the person's to type (ring-room census, 99ca7e4cb).
pub const WELL_KNOWN_ORIGINS: [&str; 2] = ["127.0.0.1:8096", "127.0.0.1:8920"];

/// The first candidate something accepts a TCP connection on.
pub fn probe_origin(candidates: &[&str]) -> Option<std::net::SocketAddr> {
    candidates
        .iter()
        .filter_map(|c| c.parse::<std::net::SocketAddr>().ok())
        .find(|addr| {
            let up = std::net::TcpStream::connect_timeout(addr, Duration::from_millis(500)).is_ok();
            tracing::debug!(%addr, up, "media offer: probed a well-known origin");
            up
        })
}

#[cfg(test)]
mod tests;
