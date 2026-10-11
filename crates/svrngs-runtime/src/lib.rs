// SPDX-License-Identifier: AGPL-3.0-or-later
//! The runtime: the only owner of each group's journal, its projections and
//! its writes. Adapters (svrngs-node) reach it through its queries and
//! `commit`, never through a journal.

pub use svrngs_core::*;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
#[expect(
    clippy::disallowed_types,
    reason = "the open-journal map; an actor per journal replaces it (DESIGN §5)"
)]
use std::sync::{Arc, Mutex};

// The one type the core does NOT re-export: it is the writer, and nothing
// above this crate opens a journal directly.
use crate::oplog::Oplog;

/// A namespace names a directory, so it may only be a plain name.
pub fn valid_namespace(ns: &str) -> bool {
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
    #[expect(
        clippy::disallowed_types,
        reason = "the open-journal map; an actor per journal replaces it (DESIGN §5)"
    )]
    open: Mutex<BTreeMap<String, Arc<RingJournal>>>,
    /// Told a group's name after the group's journal takes an act, written
    /// here or ingested from a peer (perf-2 d): what wakes a synced
    /// folder's apply, where it waited up to a second (svrngs-node
    /// synced.rs).
    on_write: OnceLock<Box<dyn Fn(&str) + Send + Sync>>,
}

impl RingRail {
    #[expect(clippy::disallowed_types, reason = "the open-journal map")]
    pub fn new(root: impl Into<PathBuf>, signer: Arc<dyn RingSigner>) -> Self {
        Self {
            root: root.into(),
            signer,
            open: Mutex::new(BTreeMap::new()),
            on_write: OnceLock::new(),
        }
    }

    /// `tell` told each group's name after its journal takes an act; one
    /// teller, the first set, for the rail's life.
    pub fn on_write(&self, tell: impl Fn(&str) + Send + Sync + 'static) {
        if self.on_write.set(Box::new(tell)).is_err() {
            tracing::warn!("ring rail: a second teller of writes refused; the first is kept");
        }
    }

    fn wrote(&self, namespace: &str) {
        if let Some(tell) = self.on_write.get() {
            tell(namespace);
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
    pub(crate) fn journal(&self, namespace: &str) -> Result<Arc<RingJournal>, RailError> {
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
/// The runtime's whole surface for adapters: queries, `commit`, the seal, and
/// the replication port. No adapter holds a journal.
///
/// ```
/// fn reads(r: &svrngs_runtime::RingRail) -> Result<svrngs_runtime::Admission, svrngs_runtime::RailError> {
///     r.admission("g")
/// }
/// ```
/// ```compile_fail
/// fn reads(r: &svrngs_runtime::RingRail) {
///     let _ = r.journal("g");
/// }
/// ```
impl RingRail {
    /// The group's admitted acts, from one read of its journal.
    pub fn admission(&self, namespace: &str) -> Result<Admission, RailError> {
        let journal = self.journal(namespace)?;
        journal.admit(&journal.roster_file()?, &Ed25519Verifier)
    }

    /// Decides `command` under the group's write lock and writes what it asks
    /// for, signed by this computer at the clock's time.
    pub fn commit<C: Command>(
        &self,
        namespace: &str,
        command: &C,
    ) -> Result<Result<Recorded, Refused<C::Refusal>>, RailError> {
        let journal = self.journal(namespace)?;
        let roster = journal.roster_file()?;
        let author = Author {
            signer: self.signer(),
            roster: &roster,
            namespace,
            verifier: &Ed25519Verifier,
            now: now_unix(namespace)?,
        };
        let done = svrngs_core::commit(&*journal, command, &author);
        if matches!(done, Ok(Ok(_))) {
            self.wrote(namespace);
        }
        done
    }

    /// The seal, then the prune it authorises.
    pub fn seal(&self, namespace: &str) -> Result<Sealed, RailError> {
        let journal = self.journal(namespace)?;
        journal.seal(self.signer(), &journal.roster_file()?, &Ed25519Verifier)
    }

    /// This computer's newest `Invite` naming the link's key `link`, and the
    /// group it is in.
    pub fn invite_for(&self, link: &str) -> Result<Option<(String, OpId)>, RailError> {
        let me = self.signer().actor();
        for namespace in self.namespaces()? {
            let journal = self.journal(&namespace)?;
            let (ops, _) = journal.read()?;
            let register = ActSet::of(ops.iter().filter(|op| op.kind.act.moves_membership()));
            let invites =
                admit(&register, &[], &journal.roster_file()?, &namespace, &Ed25519Verifier).project::<Invites>();
            if let Some(id) = invites.newest(&me, link) {
                tracing::debug!(target: "register", namespace, invite = %id,
                    "runtime: this computer's newest Invite naming the link's key");
                return Ok(Some((namespace, id.clone())));
            }
        }
        Ok(None)
    }

    /// The group's directory, where the command keeps its own files.
    pub fn group_dir(&self, namespace: &str) -> Result<PathBuf, RailError> {
        Ok(self.journal(namespace)?.dir().to_path_buf())
    }

    /// Keeps `roster` as the group's seed: founding or joining a group.
    pub fn set_seed(&self, namespace: &str, roster: &Roster) -> Result<(), RailError> {
        self.journal(namespace)?.set_roster(roster)
    }

    pub fn digest(&self, namespace: &str) -> Result<Digest, RailError> {
        self.journal(namespace)?.digest()
    }

    pub fn ops_missing_from_within(
        &self,
        namespace: &str,
        theirs: &Digest,
        budget_bytes: usize,
    ) -> Result<(Vec<Op<SignedOp>>, bool), RailError> {
        self.journal(namespace)?.ops_missing_from_within(theirs, budget_bytes)
    }

    pub fn ingest_all(&self, namespace: &str, ops: &[Op<SignedOp>]) -> Result<usize, RailError> {
        let taken = self.journal(namespace)?.ingest_all(ops)?;
        if taken > 0 {
            self.wrote(namespace);
        }
        Ok(taken)
    }

    /// Deletes what the group's admitted seals retire.
    pub fn compact(&self, namespace: &str) -> Result<Compaction, RailError> {
        let journal = self.journal(namespace)?;
        journal.compact(&journal.roster_file()?, &Ed25519Verifier)
    }
}

/// A group's directory under `root`, without a runtime: the command lists the
/// groups a computer keeps before it starts one.
pub fn group_dir_in(root: &Path, namespace: &str) -> Result<PathBuf, RailError> {
    Ok(RingJournal::open(root, namespace)?.dir().to_path_buf())
}

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
pub(crate) mod oplog;
pub(crate) use journal::{now_unix, RingJournal};
pub mod register;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_local_only;
#[cfg(test)]
mod tests_queries;
#[cfg(test)]
mod tests_write_index;
