// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/mesh_media/offer.rs@e1a43bcd2
//! What a lend is decided from (DESIGN §3, "Lent apps"): the probe, the
//! well-known table and the decision `svrngs share` made of them, moved
//! here from the command (v1ui-lend (a)), behaviour kept, so the home page
//! asks them too and the node never depends on the command. `probe_origin`
//! came from `svrn mesh media offer` (offer.rs:94-110); the command keeps
//! `resolve_target`, the verb's shape, and asks the rest here.
//!
//! svrngs: then changed (u3-5). What is shared is a port alone, on
//! 127.0.0.1, which is all a lend's declaration holds and all the door
//! dials ([`crate::lend`]); upstream's loopback `host:port` goes. The
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
//! oidc-hook-cut: the sign-in a lender ran itself is gone, and with it
//! `--sign-in`, a helper's port and its steps; sign-in with svrngs is the
//! door's own (SIGN_IN_WITH_SVRNGS §5). A `helper` row the table still
//! holds is kept for `oidc-retire`, which removes it with its app, and no
//! reader here takes one.
//!
//! start-llama: a kind the table says keeps paths back ([`kept_of`]) is lent
//! without them, its lender typing none; and a kind known by its lender's
//! word alone has no port in the table, since one service's default port is
//! often another's.

use std::num::NonZeroU16;
use std::time::Duration;

use crate::lend::{Kind, Lend, LendName, RefusedPath, Upload};

/// The well-known ports and the kind of service that listens on each, one
/// `<port> <kind>` a line, then how its lender makes a person an account
/// in it if it keeps its own, `#` for a comment: data, not code, since a
/// kind is an app's word the infra does not hold (DESIGN §5, the nouns
/// gate).
pub const WELL_KNOWN: &str = include_str!("well_known.txt");

/// How a kind [`WELL_KNOWN`] names signs each person in, and what its
/// lender does for it, in their words: either way it is lent with no
/// password, since one sent on each request makes everyone one person to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Signs {
    /// It keeps its own accounts: how its lender makes a person one.
    Accounts(&'static str),
    /// It takes the person from the door's `x-svrngs-user`: what its
    /// lender sets in it.
    Header(&'static str),
}

/// What [`WELL_KNOWN`]'s first column says a row is (REVIEW_FINDINGS #169):
/// a kind at its well-known port; a kind known by its lender's word alone
/// (`-`); or a kind's sign-in helper (`helper`), which nothing reads now and
/// `oidc-retire` removes with its app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Port(NonZeroU16),
    Named,
    Helper,
}

/// [`WELL_KNOWN`]'s rows: the first column ([`At`]) and the kind, each
/// parsed once for every reader (REVIEW_FINDINGS #64, #169), and the rest
/// of the line, empty when there is none. A row whose port or kind does
/// not parse is none.
pub fn rows() -> impl Iterator<Item = (At, Kind, &'static str)> {
    WELL_KNOWN
        .lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .filter_map(|l| {
            let (port, rest) = l.split_once(char::is_whitespace)?;
            let rest = rest.trim_start();
            let (kind, how) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            let at = match port {
                "-" => At::Named,
                "helper" => At::Helper,
                p => At::Port(p.parse().ok()?),
            };
            Some((at, Kind::parse(kind).ok()?, how.trim()))
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
    /// What an app receives of it, in its lender's words (sup-2).
    pub asks: Option<String>,
}

/// Why a lend could not be decided, in a person's words.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ShareRefusal {
    #[error("nothing answers on port {0} on this computer: start it, then share it")]
    NothingListens(NonZeroU16),
    #[error("say what is on port {0}: `svrngs share {0} --kind <kind>`, its own name if it is one svrngs knows: {kinds}", kinds = known_kinds())]
    NoKind(NonZeroU16),
}

/// The kind [`WELL_KNOWN`] names for `port`, if any.
pub fn well_known_kind(port: NonZeroU16) -> Option<Kind> {
    rows().find_map(|(at, kind, _)| (at == At::Port(port)).then_some(kind))
}

/// The kinds [`WELL_KNOWN`] names, for a lender asked to say which.
fn known_kinds() -> String {
    rows()
        .filter(|(at, ..)| *at != At::Helper)
        .map(|(_, kind, _)| kind.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// What [`WELL_KNOWN`]'s row for `kind` says past it: the word before its
/// first `:` and the rest — the one reader of that clause, for
/// [`accounts_of`] and [`kept_of`].
fn says(kind: &Kind) -> Option<(&'static str, &'static str)> {
    let (_, _, how) = rows().find(|(at, k, _)| k == kind && *at != At::Helper)?;
    how.split_once(':').map(|(what, rest)| (what, rest.trim()))
}

/// The paths [`WELL_KNOWN`] keeps back of every lend of `kind`, its lender
/// typing none; none when its row says no `keeps:`. Why each is kept back
/// is the table's, beside its row.
pub fn kept_of(kind: &Kind) -> Vec<RefusedPath> {
    let kept: Vec<RefusedPath> = match says(kind) {
        Some(("keeps", paths)) => paths
            .split_whitespace()
            .map(|p| {
                RefusedPath::parse(p).expect("the table's paths start with /, as its test holds")
            })
            .collect(),
        _ => Vec::new(),
    };
    tracing::debug!(target: "share", kind = %kind, kept = kept.len(), "share: the paths the table keeps back of the kind");
    kept
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
    let signs = match says(kind) {
        Some(("accounts", how)) => Some(Signs::Accounts(how)),
        Some(("header", what)) => Some(Signs::Header(what)),
        _ => None,
    };
    tracing::debug!(target: "share", kind = %kind, ?signs, "share: how the kind signs each person in, if the table says");
    signs
}

/// Whether something on this computer accepts a TCP connection on `port`.
/// (svrngs: upstream's `probe_origin` over the one port given.)
pub fn listens(port: NonZeroU16) -> bool {
    let up = answers(&port);
    tracing::debug!(target: "share", port, up, "share: probed the port");
    up
}

/// [`listens`], untraced, for [`listening`]'s thousands.
fn answers(port: &NonZeroU16) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port.get()));
    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok()
}

/// The ports something on this computer listens on, each probed as
/// [`listens`] probes one, a thread to each thousand (v1ui-lend (b): the
/// home page's "Already running here"). From 1024, below which only the
/// system's own services listen unless it is told otherwise, to 49151,
/// past which it hands ports to its own connections (IANA's dynamic
/// range, RFC 6335 §6), so what is offered is what a person started.
pub fn listening() -> Vec<NonZeroU16> {
    let ports: Vec<NonZeroU16> = (1024..=49151).filter_map(NonZeroU16::new).collect();
    let found: Vec<NonZeroU16> = std::thread::scope(|s| {
        let asked: Vec<_> = (ports.chunks(1000))
            .map(|c| s.spawn(|| c.iter().copied().filter(answers).collect::<Vec<_>>()))
            .collect();
        asked
            .into_iter()
            .flat_map(|h| h.join().expect("a probe does not panic"))
            .collect()
    });
    tracing::debug!(target: "share", probed = ports.len(), ?found, "share: what listens on this computer");
    found
}

/// The lend `share` declares, given what `listens` says of its port, and
/// the kind the port is well known for: the kind given, else the port's; the
/// name given, else the kind; the limit given, else none, which lets in
/// everyone in its groups at once (bash-limit: no number nobody chose;
/// `share` says so); the paths refused, those given and those the table
/// keeps back of the kind.
pub fn lend_of(
    share: &Share,
    listens: impl Fn(NonZeroU16) -> bool,
    well_known: Option<Kind>,
) -> Result<Lend, ShareRefusal> {
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
    let mut refused = share.refused.clone();
    for path in kept_of(&kind) {
        if !refused.contains(&path) {
            refused.push(path);
        }
    }
    tracing::debug!(target: "share", lend = %name, kind = %kind, port = share.port, ?limit, refused = refused.len(),
        named = share.name.is_some(), kind_given = share.kind.is_some(), limit_given = share.limit.is_some(),
        "share: the lend decided");
    Ok(Lend {
        name,
        source: crate::lend::Source::Port(share.port),
        kind,
        limit,
        refused,
        upload: share.upload,
        reads: share.reads.clone(),
        sign_in_with_svrngs: None,
        asks: share.asks.clone(),
    })
}
