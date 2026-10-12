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
//!
//! u5-4: a kind the table says keeps its own accounts ([`accounts_of`]) is
//! lent with no password, and its lender told how to make each person an
//! account in it (DESIGN §3, "Lent apps", call 4: sign in once).
//! bash-person (d): so is a kind that takes the person from the door's
//! header, its lender told what to set in it ([`Signs::Header`]).
//!
//! mwp-1: `--sign-in <port>` names the lender's program that signs each
//! person in (`svrngs_node::door::sign_in`), refused as a lend's port is
//! when nothing listens there.

use std::num::NonZeroU16;
use std::time::Duration;

use svrngs_node::lend::{Kind, Lend, LendName, RefusedPath, Upload};

/// The well-known ports and the kind of service that listens on each, one
/// `<port> <kind>` a line, then how its lender makes a person an account
/// in it if it keeps its own, `#` for a comment: data, not code, since a
/// kind is an app's word the infra does not hold (DESIGN §5, the nouns
/// gate).
pub const WELL_KNOWN: &str = include_str!("well_known.txt");

/// How a kind [`WELL_KNOWN`] names signs each person in, and what its
/// lender does for it, in their words: either way it is lent with no
/// password, since one sent on each request makes everyone one person to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signs {
    /// It keeps its own accounts: how its lender makes a person one.
    Accounts(&'static str),
    /// It takes the person from the door's `x-svrngs-user`: what its
    /// lender sets in it.
    Header(&'static str),
}

/// [`WELL_KNOWN`]'s rows: the port as written, the kind, parsed once for
/// both readers (REVIEW_FINDINGS #64), and the rest of the line, empty when
/// there is none. A row whose kind does not parse is none.
fn rows() -> impl Iterator<Item = (&'static str, Kind, &'static str)> {
    WELL_KNOWN
        .lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .filter_map(|l| {
            let (port, rest) = l.split_once(char::is_whitespace)?;
            let rest = rest.trim_start();
            let (kind, how) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            Some((port, Kind::parse(kind).ok()?, how.trim()))
        })
}

/// What `svrngs share` was told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share {
    pub port: NonZeroU16,
    pub name: Option<LendName>,
    pub kind: Option<Kind>,
    pub limit: Option<NonZeroU16>,
    pub refused: Vec<RefusedPath>,
    /// This computer's upload, as its lender states it; none stated, none
    /// declared (u3-6).
    pub upload: Option<Upload>,
    /// The kinds of lend its pages read (u5-2).
    pub reads: Vec<Kind>,
    /// The port of its sign-in, if its lender names one (mwp-1).
    pub sign_in: Option<NonZeroU16>,
    /// What an app receives of it, in its lender's words (sup-2).
    pub asks: Option<String>,
}

/// Why a lend could not be decided, in a person's words.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ShareRefusal {
    #[error("nothing answers on port {0} on this computer: start it, then share it")]
    NothingListens(NonZeroU16),
    #[error("say what is on port {0}: `svrngs share {0} --kind <kind>`")]
    NoKind(NonZeroU16),
    #[error("nothing answers on port {0}, its sign-in, on this computer: start it, then share")]
    NoSignIn(NonZeroU16),
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
    rows().find_map(|(p, kind, _)| (p.parse::<u16>().ok()? == port.get()).then_some(kind))
}

/// How a service of `kind` signs each person in, if [`WELL_KNOWN`] says:
/// with accounts of its own, each signing in once with one its lender made,
/// or from the door's header. Either is lent with no password: one sent on
/// each request makes everyone one person to it, so one person's sign-in
/// ended everyone's, and its token reached their browsers
/// (`demo/runs/a/dry-2026-10-04/README.md:39-50`;
/// `demo/runs/u5/2026-10-05-jellyfin/README.md`, finding 3). A row saying
/// neither `accounts:` nor `header:` says nothing.
pub fn accounts_of(kind: &Kind) -> Option<Signs> {
    let how = rows().find(|(_, k, _)| k == kind).map(|(_, _, how)| how);
    let signs = match how.and_then(|h| h.split_once(':')) {
        Some(("accounts", how)) => Some(Signs::Accounts(how.trim())),
        Some(("header", what)) => Some(Signs::Header(what.trim())),
        _ => None,
    };
    tracing::debug!(target: "share", kind = %kind, ?signs, "share: how the kind signs each person in, if the table says");
    signs
}

/// Whether something on this computer accepts a TCP connection on `port`.
/// (svrngs: upstream's `probe_origin` over the one port given.)
pub fn listens(port: NonZeroU16) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port.get()));
    let up = std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok();
    tracing::debug!(target: "share", %addr, up, "share: probed the port");
    up
}

/// The lend `share` declares, given what `listens` says of its port and
/// its sign-in's, and the kind the port is well known for: the kind given,
/// else the port's; the name given, else the kind; the limit given, else
/// none, which lets in everyone in its groups at once (bash-limit: no
/// number nobody chose; `share` says so).
pub fn lend_of(
    share: &Share,
    listens: impl Fn(NonZeroU16) -> bool,
    well_known: Option<Kind>,
) -> Result<Lend, ShareRefusal> {
    if let Some(port) = share.sign_in.filter(|p| !listens(*p)) {
        tracing::debug!(target: "share", port, "share: refused — nothing listens on the sign-in's port");
        return Err(ShareRefusal::NoSignIn(port));
    }
    if !listens(share.port) {
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
    let limit = share.limit;
    tracing::debug!(target: "share", lend = %name, kind = %kind, port = share.port, ?limit,
        named = share.name.is_some(), kind_given = share.kind.is_some(), limit_given = share.limit.is_some(),
        "share: the lend decided");
    Ok(Lend {
        name,
        port: share.port,
        kind,
        limit,
        refused: share.refused.clone(),
        upload: share.upload,
        reads: share.reads.clone(),
        sign_in: share.sign_in,
        asks: share.asks.clone(),
    })
}

#[cfg(test)]
mod tests;
