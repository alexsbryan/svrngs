// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail/src/lib.rs@e1a43bcd2
//! The ring journal on disk — `<root>/rings/<namespace>/ring_oplog.jsonl`.
//!
//! [`svrngs_core`] is the FOLD: vocabulary, signing, admission,
//! the sync digest, and not one line of I/O. This crate is the half that
//! touches a filesystem — one writer per namespace, the door that signs, and
//! the peer-ingest path. Everything the core exports is re-exported here, so
//! an application names one crate and a peer that only needs the fold (canon)
//! names the other.
//!
//! # The door is strict about what the rail knows, and silent about the rest
//!
//! [`RingJournal::append`] refuses what the rail can judge: a payload with no
//! canonical form (see [`Payload`]), and an attempt to author under a key
//! that holds no standing in the ring (svrngs: the register's — the seed and
//! its `Admit` and `Remove` acts, [`register`] — where upstream asked whether
//! the ring's roster carried the key). It says nothing about whether an amount
//! is positive or a borrower exists, because it cannot — those are the app's,
//! and the app owns one validator that its own door and its own reducer both
//! call, exactly as this module used to.
//!
//! The second of those is [`RailError::NotInRoster`] rather than a sentence,
//! so a renderer says it in its own words by matching the variant, never the
//! prose. (Upstream's reason was that the fix depended on whether the roster
//! was a file or DERIVED, `RingRail::roster_origin`; svrngs derives no
//! roster, and the fix is an `Admit`.)
//!
//! The journal is **truth**; the mesh store is only a transport buffer. In
//! production `MeshStore` is `in_memory()`, so anything treating it as
//! durable loses the log on restart.

pub use svrngs_core::*;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// The one type the core does NOT re-export: it is the writer, and nothing
// above this crate opens a journal directly.
use crate::oplog::Oplog;

/// A namespace names a directory, so it may only be a plain name.
fn valid_namespace(ns: &str) -> bool {
    !ns.is_empty()
        && ns.len() <= 64
        && ns
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

/// Every ring this node holds, as a directory. The ONE place the literal
/// `rings` is spelled — [`ring_dir`] and [`RingRail::namespaces`] are its two
/// callers and there is no third spelling in the workspace (ARCH §10.6).
fn rings_root(root: &Path) -> PathBuf {
    root.join("rings")
}

/// Where one ring namespace keeps its journal and its roster:
/// `<root>/rings/<namespace>/`.
///
/// A caller that holds a [`RingJournal`] reads [`RingJournal::dir`] or
/// [`RingJournal::roster_path`] instead — those are the same join and they
/// also carry the namespace check. This is for the caller that has a root and
/// a name and nothing else.
fn ring_dir(root: &Path, namespace: &str) -> PathBuf {
    rings_root(root).join(namespace)
}

// ── The rail's storage ───────────────────────────────────────

/// Every ring namespace this node holds, and the one key it signs with.
///
/// Installed once by the daemon. A namespace's journal is opened on first
/// touch and kept, so the write lock that serialises appends is per-namespace
/// and outlives a request — two ring apps do not contend, and one app's two
/// requests do.
pub struct RingRail {
    root: PathBuf,
    signer: Arc<dyn RingSigner>,
    open: Mutex<BTreeMap<String, Arc<RingJournal>>>,
}

impl RingRail {
    pub fn new(root: impl Into<PathBuf>, signer: Arc<dyn RingSigner>) -> Self {
        Self {
            root: root.into(),
            signer,
            open: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn signer(&self) -> &dyn RingSigner {
        self.signer.as_ref()
    }

    /// The root this rail journals under — for [`namespaces_in`], the
    /// unfiltered disk list a local-only rehydrate reads (fp-108).
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Every namespace this node holds a journal for, read from disk.
    ///
    /// From DISK and not from the open map, because on boot nothing has been
    /// touched yet — and boot is exactly when replication most needs the
    /// list. A node that came back from a week off has to offer its whole
    /// journal to peers before anyone asks it anything.
    ///
    /// A missing `rings/` directory is an empty list, not an error: a daemon
    /// that has never hosted a ring is a normal daemon.
    ///
    /// A local-only namespace ([`is_local_only`]) is journaled here but never
    /// offered, so it is not on this list.
    pub fn namespaces(&self) -> Result<Vec<String>, RailError> {
        let mut out = namespaces_in(&self.root)?;
        out.retain(|ns| {
            let local = is_local_only(ns);
            if local {
                tracing::debug!(namespace = %ns, "ring rail: local-only journal, not offered");
            }
            !local
        });
        Ok(out)
    }
}

/// Every ring namespace under `root`, without holding a [`RingRail`].
///
/// A reader that only wants to LIST rings has no business minting a signer,
/// and a second `root.join("rings")` somewhere else is how the literal gets a
/// second spelling (ARCH §10.6 — the module note above says this is the one
/// place it lives). `svrn mesh offers --why` is the caller that needed it:
/// resolving a remote seller's warrant means asking every ring this node
/// holds, and it holds no rail.
pub fn namespaces_in(root: &Path) -> Result<Vec<String>, RailError> {
    let dir = rings_root(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(RailError::Io(format!("{}: {e}", dir.display()))),
    };
    let mut out: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| e.file_name().into_string().ok())
        // A directory whose name this build would refuse to open is not a
        // namespace — skip it rather than surfacing a path we cannot use.
        .filter(|name| valid_namespace(name))
        .collect();
    out.sort();
    Ok(out)
}

impl RingRail {
    /// The journal for one namespace, opening it if this is the first touch.
    pub fn journal(&self, namespace: &str) -> Result<Arc<RingJournal>, RailError> {
        let mut open = self.open.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = open.get(namespace) {
            return Ok(existing.clone());
        }
        let journal = Arc::new(RingJournal::open(&self.root, namespace)?);
        open.insert(namespace.to_string(), journal.clone());
        Ok(journal)
    }
}

// ── What a compaction did ────────────────────────────────────
//
// `Compaction` moved to `commonwealth-rail-core` beside the `Floors` map it
// carries through: it is vocabulary, not storage, and the compact door's
// answer crosses back to a dialing client as this type. The glob re-export
// above keeps `commonwealth_rail::Compaction` resolving.

/// One [`RingJournal::seal`]: the seal act, and the prune it authorises.
///
/// **A seal and its prune are one act, and they are not one result.** The seal
/// is signed and on disk by the time the prune runs, so a refused prune is not
/// a failed seal — reporting it as one would tell a caller its seal did not
/// land when it did, and the retry would write a second one. So the pair is an
/// `Ok` carrying a `Result`: the caller has to look at `retired` to say what
/// happened, and cannot mistake "nothing was retired" for "the prune was
/// refused" (ARCH §18.3).
#[derive(Debug)]
pub struct Sealed {
    pub op: Op<SignedOp>,
    /// What [`RingJournal::compact`] did, or why it would not.
    pub retired: Result<Compaction, RailError>,
}

// ── The journal on disk ──────────────────────────────────────

// `RingJournal` lives in a sibling file: with `RingRail` it put this file into
// the 800-1200 approach band (ARCH §3.1). Re-exported, so the public path
// `commonwealth_rail::RingJournal` is unchanged.
mod journal;
pub mod oplog;
pub use journal::RingJournal;

// The v1 checkpoint document's one composer (moved from sovereign-mesh,
// phase-b pb-rails-parity).
pub mod ring_checkpoint;

// The dialling half of sync, forked in U1 (docs/DESIGN.md §8, U1's
// inventory): the sync bodies (`sovereign-peer-wire`), the one ALPN, the
// relayed endpoint and the loopback bridge (`mesh-reach::guest`), and the
// client side of the dial-by-key transport (`commonwealth-transport::iroh`).
pub mod alpn;
pub mod guest;
pub mod iroh;
pub mod peer_wire;

// Ring sync and its route, forked in U1 (docs/DESIGN.md §8, U1's inventory):
// the round and the exchange (`commonwealth-rails`' `ring_sync`), and the sync
// route (`ring_routes.rs`), which a caller serves as an `axum::Router`.
pub mod ring_routes;
pub mod ring_sync;

// The register, U1's (docs/DESIGN.md §6): who stands in a group, the one
// answer the round, the acceptor, the sync route and the door ask.
pub mod register;

// Redeeming a link, U2's (docs/DESIGN.md §6): the route served on the
// second ALPN, and the joiner's call to it.
pub mod redeem;

// The record over loopback, U3's (docs/DESIGN.md §8, U3's inventory): the
// rail's append, log and live doors (`commonwealth-rails`' `rail.rs`), on a
// listener that refuses any address but loopback (its `api.rs`), and the
// live lane's fan-out (`sovereign-daemon`'s `routes_rail_live.rs`).
pub mod api;
pub mod live;
pub mod rail;

// The node, U1's (docs/DESIGN.md §8): the rail, the endpoint, the accept loop
// serving the sync route and the round, in place of upstream's `RailsDaemon`.
pub mod node;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_local_only;
#[cfg(test)]
mod tests_write_index;
