// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/publish_cmd.rs@e1a43bcd2
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/mesh_media/offer.rs@e1a43bcd2
//! `svrngs share` (DESIGN §8, U3's inventory, its publish_cmd and offer
//! row): the verb's shape and the probe, as upstream had them —
//! `resolve_target` from `svrn publish` (publish_cmd.rs:409-434, its tests
//! in `share/tests.rs`) and `probe_origin` from `svrn mesh media offer`
//! (offer.rs:94-110). (svrngs: each is `pub`, so the lib builds before its
//! caller lands, and the imports upstream's modules had are written here.)
//!
//! svrngs: then changed (u3-5). What is shared is a port alone, on
//! 127.0.0.1, which is all a lend's declaration holds and all the door
//! dials (`svrngs_node::lend`); upstream's loopback `host:port` goes. The
//! probe asks one port, the one given, and the well-known ports name the
//! kind of what listens there, from a table kept as data
//! ([`WELL_KNOWN`]), so a lend of a service svrngs recognises needs no
//! more than its port. [`lend_of`] decides the lend from what was typed and
//! what the probe saw, and touches nothing.

use std::num::NonZeroU16;
use std::time::Duration;

use svrngs_node::lend::{Kind, Lend, LendName, RefusedPath};

/// How many people may use a lend at once when its lender names no limit:
/// Demo A's three homes on Sam's one library (DESIGN §8). `share` prints
/// it, so it is never silent.
pub const DEFAULT_LIMIT: NonZeroU16 = NonZeroU16::new(3).expect("3 is not 0");

/// The well-known ports and the kind of service that listens on each, one
/// `<port> <kind>` a line, `#` for a comment: data, not code, since a kind
/// is an app's word the infra does not hold (DESIGN §5, the nouns gate).
pub const WELL_KNOWN: &str = include_str!("well_known.txt");

/// What `svrngs share` was told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share {
    pub port: NonZeroU16,
    pub name: Option<LendName>,
    pub kind: Option<Kind>,
    pub limit: Option<NonZeroU16>,
    pub refused: Vec<RefusedPath>,
}

/// Why a lend could not be decided, in a person's words.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ShareRefusal {
    #[error("nothing answers on port {0} on this computer: start it, then share it")]
    NothingListens(NonZeroU16),
    #[error("say what is on port {0}: `svrngs share {0} --kind <kind>`")]
    NoKind(NonZeroU16),
}

/// `5000` → port 5000 on this computer. (svrngs: a port alone, where
/// upstream also took a loopback `host:port`; the door dials 127.0.0.1.)
///
/// Loopback-only is not a limitation to relax later: the point of publishing
/// is that the app stays bound to this machine and is reached by mesh key. A
/// `0.0.0.0` target would already be on the LAN to anyone, with no identity
/// attached, which is the thing the mesh replaces.
pub fn resolve_target(port: &str) -> Result<NonZeroU16, String> {
    match port.parse::<u16>().ok().and_then(NonZeroU16::new) {
        Some(p) => Ok(p),
        None => Err(format!(
            "`{port}` is not a port on this computer: a number from 1 to 65535"
        )),
    }
}

/// The kind [`WELL_KNOWN`] names for `port`, if any.
pub fn well_known_kind(port: NonZeroU16) -> Option<Kind> {
    WELL_KNOWN
        .lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .find_map(|l| {
            let (p, kind) = l.split_once(char::is_whitespace)?;
            (p.parse::<u16>().ok()? == port.get()).then(|| Kind::parse(kind.trim()).ok())?
        })
}

/// Whether something on this computer accepts a TCP connection on `port`.
/// (svrngs: upstream's `probe_origin` over the one port given.)
pub fn listens(port: NonZeroU16) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port.get()));
    let up = std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok();
    tracing::debug!(target: "share", %addr, up, "share: probed the port");
    up
}

/// The lend `share` declares, given whether something listens on its port
/// and the kind the port is well known for: the kind given, else the
/// port's; the name given, else the kind; the limit given, else
/// [`DEFAULT_LIMIT`].
pub fn lend_of(
    share: &Share,
    listening: bool,
    well_known: Option<Kind>,
) -> Result<Lend, ShareRefusal> {
    if !listening {
        tracing::debug!(target: "share", port = share.port, "share: refused — nothing listens on the port");
        return Err(ShareRefusal::NothingListens(share.port));
    }
    let Some(kind) = share.kind.clone().or(well_known) else {
        tracing::debug!(target: "share", port = share.port, "share: refused — no kind given, and the port is not a well-known one");
        return Err(ShareRefusal::NoKind(share.port));
    };
    let name = match &share.name {
        Some(n) => n.clone(),
        None => LendName::parse(kind.as_str()).expect("a kind is a lend's name, by one rule"),
    };
    let limit = share.limit.unwrap_or(DEFAULT_LIMIT);
    tracing::debug!(target: "share", lend = %name, kind = %kind, port = share.port, limit,
        named = share.name.is_some(), kind_given = share.kind.is_some(), limit_given = share.limit.is_some(),
        "share: the lend decided");
    Ok(Lend {
        name,
        port: share.port,
        kind,
        limit,
        refused: share.refused.clone(),
    })
}

#[cfg(test)]
mod tests;
