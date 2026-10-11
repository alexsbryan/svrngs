// SPDX-License-Identifier: AGPL-3.0-or-later
//! A group's folders' bytes (gf-1, group folders' second spike, kept for
//! sy-1's synced folder; DESIGN §9). A synced folder is one group's
//! (syp-4): its files go into this computer's iroh-blobs store, and its
//! acts, written in that group alone, name them by hash
//! ([`svrngs_core::SyncFolder`]). The provider serves a hash to a dialer
//! only while a synced folder of a group the dialer stands in places it,
//! asked of the record on every request. A fetch takes what the store lacks
//! of a hash, verified, from a holder the record names; bytes that do not match
//! the hash never reach the store, and each holder asked is named with why
//! it gave nothing. A group whose record cannot be read is said by name,
//! never read as one placing nothing (#113, #117). The store keeps what a
//! layout places and a synced directory holds, and drops every other
//! version once what it keeps changes (syp-4 b). Streaming a folder by range on loopback,
//! and the one-way share it served, were dropped at syp-1 (operator,
//! 2026-10-10): a group's folder is synced onto its members' disks.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use iroh::endpoint::Connection;
use iroh_blobs::api::blobs::{AddPathOptions, ImportMode};
use iroh_blobs::api::remote::GetProgressItem;
use iroh_blobs::api::{Store, TempTag};
use iroh_blobs::protocol::GetRequest;
use iroh_blobs::provider::events::{
    AbortReason, ConnectMode, EventMask, EventSender, ObserveMode, ProviderMessage, RequestMode,
    RequestUpdate,
};
use iroh_blobs::store::fs::{options::Options, FsStore};
use iroh_blobs::store::{GcConfig, ProtectCb, ProtectOutcome};
use iroh_blobs::{BlobFormat, BlobsProtocol, Hash};
use n0_future::StreamExt as _;
use std::convert::Infallible;

use svrngs_core::{Command, ContentHash, Digest, Placed, Recorded};
use tokio::sync::{mpsc, oneshot, watch, Notify};

use crate::iroh::{actor_key, key_actor, Alpn};
use crate::node::{Host, ANSWER_WITHIN};

#[path = "synced.rs"]
mod synced;

/// How long a holder that did not answer within [`ANSWER_WITHIN`] is
/// taken as asleep: asked last, after every other holder, for this long
/// (syp-4 f). The one decider of who is asleep: the node's lends keep
/// none, asking every computer at once and waiting on none past
/// [`ANSWER_WITHIN`] (node.rs `Lends`), where a fetch asks its holders in
/// turn, and would wait out an asleep writer at every file.
const ASLEEP_FOR: Duration = Duration::from_secs(60);

/// How often the store asks whether what it keeps has changed, and drops
/// what it no longer keeps if so (syp-4 b).
const SWEEP: Duration = Duration::from_secs(1);

/// What the store's sweep asks the node: what to keep, or `None` to keep
/// everything this time.
type KeepAsk = oneshot::Sender<Option<HashSet<Hash>>>;

#[derive(Debug, thiserror::Error)]
pub enum FolderError {
    #[error("the folder {0} cannot be read: {1}")]
    Read(PathBuf, std::io::Error),
    #[error("the store refused {0}: {1}")]
    Store(PathBuf, String),
    #[error("the record refused the folder: {0}")]
    Record(String),
    #[error("the record of the group {0} cannot be read here: {1}")]
    Unread(String, String),
    #[error("this computer does not stand in the group {0}")]
    NotStanding(String),
    #[error("{0} cannot be read, so nothing it keeps is synced: {1}")]
    Kept(PathBuf, String),
    #[error("{0} is not synced: one directory keeps one group's folder, and this folder or this directory is kept already")]
    Synced(PathBuf),
}

/// This computer's folders and its fetches' memory: one per node.
#[derive(Clone)]
pub struct Folders {
    store: Store,
    /// The journals, the endpoint, this computer's key, and who stands
    /// where ([`Host::standings`]): the round's, read here too.
    host: Arc<Host>,
    nudge: Arc<Notify>,
    data_dir: PathBuf,
    /// The directories this computer syncs (syp-4 e), written by `both_ways`
    /// alone.
    syncing: watch::Sender<synced::Syncing>,
    /// Each holder's open connection, and when one last did not answer.
    conns: watch::Sender<BTreeMap<String, Connection>>,
    unanswered: watch::Sender<BTreeMap<String, Instant>>,
    /// Each group's layout as last read, with the record state it was read
    /// at (perf-2 c).
    layouts: watch::Sender<BTreeMap<String, Layout>>,
    /// Each group's acts taken since the node started, counted by the
    /// rail's teller: a synced directory's loop waits on its group's
    /// (perf-2 d).
    woken: watch::Sender<BTreeMap<String, u64>>,
}

/// A group's synced folders' files where its record places them, the set
/// of their hashes, and the record's digest when they were read: equal
/// digests hold equal acts (svrngs-core view.rs, `View`), as the store's
/// sweep reads them ([`Folders::read_at`]). Read again only when the
/// digest moved, which [`Folders::layout`] asks before each answer, so a
/// layout the record has moved past never answers.
#[derive(Clone)]
struct Layout {
    at: Digest,
    placed: Arc<[Placed]>,
    hashes: Arc<HashSet<ContentHash>>,
}

/// A file found under a synced directory: its path under it, where it is,
/// and its metadata then.
type Found = (String, PathBuf, std::fs::Metadata);

/// What the store's keep is read from: each synced group's digest, and
/// each synced directory's kept state by its size and mtime.
type ReadAt = (
    Vec<(String, Digest)>,
    Vec<(PathBuf, u64, std::time::SystemTime)>,
);

impl Folders {
    /// The store under `data_dir/blobs`, its sweep, and its provider.
    pub(crate) async fn start(
        data_dir: &Path,
        host: Arc<Host>,
        nudge: Arc<Notify>,
    ) -> Result<(Self, BlobsProtocol), String> {
        // The sweep runs on the store's own runtime and asks this node what
        // to keep; a sweep told nothing keeps everything.
        let (ask_keep, mut keep_asked) = mpsc::unbounded_channel::<KeepAsk>();
        let protect: ProtectCb = Arc::new(move |live| {
            let (tx, rx) = oneshot::channel();
            ask_keep.send(tx).ok();
            Box::pin(async move {
                let Ok(Some(keep)) = rx.await else {
                    return ProtectOutcome::Abort;
                };
                live.extend(keep);
                ProtectOutcome::Continue
            })
        });
        let root = data_dir.join("blobs");
        let mut options = Options::new(&root);
        options.gc = Some(GcConfig {
            interval: SWEEP,
            add_protected: Some(protect),
        });
        let fs = FsStore::load_with_opts(root.join("blobs.db"), options)
            .await
            .map_err(|e| format!("the blob store: {e}"))?;
        let store: Store = (*fs).clone();
        // Nothing of the folders' is tagged since syp-4 (b): a tag a spike's
        // take left would keep its version past every layout.
        let untagged = store.tags().delete_all().await;
        let untagged = untagged.map_err(|e| format!("the blob store's tags: {e}"))?;
        let folders = Folders {
            store: store.clone(),
            host,
            nudge,
            data_dir: data_dir.to_path_buf(),
            syncing: watch::Sender::new(synced::syncing_in(data_dir)),
            conns: watch::Sender::new(BTreeMap::new()),
            unanswered: watch::Sender::new(BTreeMap::new()),
            layouts: watch::Sender::new(BTreeMap::new()),
            woken: watch::Sender::new(BTreeMap::new()),
        };
        let woken = folders.woken.clone();
        folders.host.rail().on_write(move |ns| {
            woken.send_modify(|m| *m.entry(ns.into()).or_default() += 1);
        });
        tracing::info!(target: "folder", untagged, "folder: the store is open");
        let keeper = folders.clone();
        tokio::spawn(async move {
            let (mut last, mut read_at) = (None, None);
            while let Some(answer) = keep_asked.recv().await {
                // Each sweep reading every group's layout cost each node a
                // whole record's admission a second: 25 nodes of 10,000 acts
                // reached none of 24 in the ceiling's canary (syp-4).
                let at = keeper.read_at();
                if at.is_some() && at == read_at {
                    tracing::debug!(target: "folder", "folder: the store's sweep drops nothing — no record and no directory moved since what it keeps was read");
                    answer.send(None).ok();
                    continue;
                }
                let keep = keeper.keep();
                read_at = at.filter(|_| keep.is_some());
                let sweep = keep.is_some() && keep != last;
                if sweep {
                    let kept = keep.as_ref().map_or(0, HashSet::len);
                    tracing::debug!(target: "folder", kept, "folder: what the store keeps changed, so it drops every other version");
                    last.clone_from(&keep);
                }
                answer.send(keep.filter(|_| sweep)).ok();
            }
        });
        // The store's actor lives as long as the node.
        tokio::spawn(async move {
            let _keep = fs;
            std::future::pending::<()>().await
        });
        folders.resume();
        let blobs = provider(&store, folders.clone());
        Ok((folders, blobs))
    }

    /// Every synced folder's files where the record of `group` places
    /// them, read whole only when its digest moved since the last read: a
    /// whole read once a get was 6.5 ms at 1,000 files and 399.6 at 100,000
    /// (perf-1's README §9). A record that cannot be read is said by the
    /// group's name, never read as one placing nothing (#113, #117).
    fn layout(&self, group: &str) -> Result<Layout, FolderError> {
        let unread =
            |e: svrngs_runtime::RailError| FolderError::Unread(group.into(), e.to_string());
        // The digest before the acts, so a layout is never older than the
        // digest it is kept with: an act between the two is read again.
        let at = self.host.rail().digest(group).map_err(unread)?;
        let kept = self.layouts.borrow().get(group).cloned();
        if let Some(kept) = kept.filter(|l| l.at == at) {
            return Ok(kept);
        }
        let placed: Arc<[Placed]> = self
            .host
            .rail()
            .admission(group)
            .map_err(unread)?
            .layout()
            .into();
        let hashes = Arc::new(placed.iter().map(|p| p.hash).collect());
        tracing::debug!(target: "folder", namespace = group, placed = placed.len(), "folder: the layout read again, the record having moved");
        let read = Layout { at, placed, hashes };
        self.layouts
            .send_modify(|m| drop(m.insert(group.into(), read.clone())));
        Ok(read)
    }

    /// Every file under `from`, by its path under the synced directory,
    /// `from_rel` being `from`'s. A link is not followed, since it may reach
    /// outside the folder.
    fn walk(&self, from: &Path, from_rel: &str) -> Result<Vec<Found>, FolderError> {
        let mut files = Vec::new();
        let mut walk = vec![from.to_path_buf()];
        while let Some(at) = walk.pop() {
            let entries = std::fs::read_dir(&at).map_err(|e| FolderError::Read(at.clone(), e))?;
            for entry in entries.flatten() {
                let path = entry.path();
                let meta = std::fs::symlink_metadata(&path)
                    .map_err(|e| FolderError::Read(path.clone(), e))?;
                if meta.is_dir() {
                    walk.push(path);
                } else if !meta.is_file() {
                    tracing::debug!(target: "folder", path = %path.display(), "folder: not shared — a link, which may reach outside the folder");
                } else {
                    let rel = path.strip_prefix(from).unwrap_or(&path).to_string_lossy();
                    let rel = match from_rel {
                        "" => rel.into_owned(),
                        _ => format!("{from_rel}/{rel}"),
                    };
                    files.push((rel, path, meta));
                }
            }
        }
        Ok(files)
    }

    /// The files at or under each of `paths` the OS named under `dir`, as
    /// [`Self::walk`] gives them, and each path's own path under `dir`: a
    /// file held under one but not found is a delete. A file written is
    /// itself; a directory moved in, or written whole, is walked; a path
    /// gone, a file deleted or renamed away or a directory moved out, finds
    /// nothing, so what was held under it is deleted. A save by rename names
    /// both its paths, the new file found at the one it replaced.
    fn look_at(
        &self,
        dir: &Path,
        paths: &BTreeSet<PathBuf>,
    ) -> Result<(Vec<Found>, Vec<String>), FolderError> {
        // The OS names a path as it resolves it (macOS: /private/var for
        // /var), so it is matched against the directory resolved too.
        let real = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        let (mut files, mut scopes) = (Vec::new(), Vec::new());
        for path in paths {
            let Some(rel) = [&real, dir].iter().find_map(|d| path.strip_prefix(d).ok()) else {
                continue;
            };
            let rel = rel.to_string_lossy().into_owned();
            match std::fs::symlink_metadata(path) {
                Ok(m) if m.is_dir() => files.extend(self.walk(path, &rel)?),
                Ok(m) if m.is_file() => files.push((rel.clone(), path.clone(), m)),
                Ok(_) => {
                    tracing::debug!(target: "folder", path = %path.display(), "folder: not shared — a link, which may reach outside the folder")
                }
                Err(_) => {}
            }
            scopes.push(rel);
        }
        Ok((files, scopes))
    }

    /// The file at `path` copied into the store, held from the sweep
    /// while its tag lives. Copied, not referenced: the store serves a
    /// version after the directory has overwritten it — to a member still
    /// fetching it, or as a conflict's other side — and a referenced file
    /// edited in place would answer its old hash with new bytes, which
    /// every fetch refuses, so the version would reach no one.
    async fn take(&self, path: &Path) -> Result<TempTag, FolderError> {
        let options = AddPathOptions {
            path: path.to_path_buf(),
            mode: ImportMode::Copy,
            format: BlobFormat::Raw,
        };
        let tag = self.store.add_path_with_opts(options).temp_tag().await;
        tag.map_err(|e| FolderError::Store(path.to_path_buf(), e.to_string()))
    }

    /// The groups a directory here syncs: a version is in the store only
    /// as one of their directories took or fetched it, so no other group's
    /// layout is read for what it keeps. `Err` when the list cannot be read.
    fn synced_groups(&self) -> Result<BTreeSet<String>, String> {
        let syncing = self.syncing.borrow();
        let kept = syncing
            .as_ref()
            .map_err(|(at, e)| format!("{}: {e}", at.display()))?;
        Ok(kept.keys().map(|(group, _)| group.clone()).collect())
    }

    /// What [`Self::keep`] is read from, cheaply: each synced group's
    /// digest, since equal digests hold equal acts (svrngs-core view.rs,
    /// `View`), and each kept state's size and mtime, since a step writes
    /// its kept state whenever it changes (perf-2 f), not each kept state
    /// read whole each second. `None` when any cannot be read, so `keep` is
    /// read and says why.
    fn read_at(&self) -> Option<ReadAt> {
        let groups = self.synced_groups().ok()?;
        let rail = self.host.rail();
        let digests = groups
            .into_iter()
            .map(|ns| rail.digest(&ns).ok().map(|d| (ns, d)));
        Some((digests.collect::<Option<_>>()?, self.kept_at().ok()?))
    }

    /// What the store keeps: every version a synced folder's layout places
    /// in a group a directory here syncs, a conflict's other side among
    /// them, and every version a synced directory holds as last taken or
    /// written;
    /// `None`, said, when any of it cannot be read, so nothing is dropped
    /// on a guess.
    fn keep(&self) -> Option<HashSet<Hash>> {
        let groups = self.synced_groups();
        let groups = groups.inspect_err(|e| tracing::warn!(target: "folder", error = %e, "folder: the store keeps everything — the directories synced here cannot be listed")).ok()?;
        let (mut keep, mut why) = (HashSet::new(), None);
        for ns in groups {
            match self.layout(&ns) {
                Ok(read) => keep.extend(read.hashes.iter().map(|h| Hash::from(*h.as_bytes()))),
                Err(e) => why = Some(e),
            }
        }
        match self.held_everywhere() {
            Ok(held) => keep.extend(held.iter().map(|h| Hash::from(*h.as_bytes()))),
            Err(e) => why = Some(e),
        }
        if let Some(why) = why {
            tracing::warn!(target: "folder", %why, "folder: the store keeps everything — what it keeps cannot be read");
            return None;
        }
        Some(keep)
    }

    /// `act` written in `group` alone, and the round nudged; what it
    /// recorded.
    async fn write<C: Command<Refusal = Infallible>>(
        &self,
        act: &C,
        group: &str,
        folder: &str,
        files: usize,
    ) -> Result<Recorded, FolderError> {
        let done = match self.host.rail().commit(group, act) {
            Ok(Ok(done)) => done,
            Ok(Err(why)) => {
                tracing::warn!(target: "folder", namespace = group, folder, ?why, "folder: not written — this computer does not stand in the folder's group");
                return Err(FolderError::NotStanding(group.into()));
            }
            Err(e) => return Err(FolderError::Record(e.to_string())),
        };
        tracing::info!(target: "folder", namespace = group, folder, files, "folder: written to the record");
        self.nudge.notify_one();
        Ok(done)
    }

    /// Whether a synced folder of a group `dialer` stands in places `hash`,
    /// whoever wrote it: who stands where is [`Host::standings`]'s, and a
    /// group whose record cannot be read places nothing it serves, said by
    /// its name.
    fn serves(&self, hash: Hash, dialer: &str) -> bool {
        let hash = &ContentHash::from_bytes(hash.into());
        let groups = self.host.standings().unwrap_or_default();
        for (ns, _) in groups.iter().filter(|(_, m)| m.contains_key(dialer)) {
            match self.layout(ns) {
                Ok(read) if read.hashes.contains(hash) => {
                    tracing::debug!(target: "folder", namespace = ns, dialer, "folder: the dialer stands in a group whose synced folder places the hash");
                    return true;
                }
                Ok(_) => {}
                Err(why) => {
                    tracing::warn!(target: "folder", namespace = ns, dialer, %why, "folder: a group the dialer stands in cannot be read, so nothing of it is served")
                }
            }
        }
        false
    }

    /// Make `hash` held here whole, verified, in one get of what the store
    /// lacks, asking each holder in turn, one that did not answer lately
    /// last; a holder that stops mid-file leaves what the store still lacks
    /// to the next (perf-2 a). `Ok` names the holder it came from, or none
    /// when the store held it; `Err` names each holder asked and why it gave
    /// nothing.
    #[expect(
        clippy::disallowed_methods,
        reason = "the fetch is an adapter: it times what a holder takes, and when one did not answer"
    )]
    async fn hold(
        &self,
        hash: Hash,
        holders: &[String],
    ) -> Result<Option<String>, Vec<(String, String)>> {
        let mut lacks = self.lacks(hash).await?;
        let mut order: Vec<&String> = holders.iter().filter(|k| *k != self.host.actor()).collect();
        let lately = self.unanswered.borrow().clone();
        order.sort_by_key(|k| lately.get(*k).is_some_and(|t| t.elapsed() < ASLEEP_FOR));
        let mut why = Vec::new();
        for key in order {
            let Some(missing) = lacks else {
                return Ok(None);
            };
            let t0 = Instant::now();
            let said = match self.fetch(key, missing).await {
                Ok(stats) => {
                    self.unanswered
                        .send_if_modified(|m| m.remove(key).is_some());
                    tracing::info!(target: "folder", hash = %hash, holder = %key, payload = stats.payload_bytes_read, other = stats.other_bytes_read, ms = t0.elapsed().as_millis() as u64, "folder: fetched, verified");
                    return Ok(Some(key.clone()));
                }
                Err(Nothing::Said(e)) => e,
                Err(Nothing::Silent) => {
                    // Kept whether new or not; nothing waits on a change.
                    let at = Instant::now();
                    self.unanswered
                        .send_if_modified(|m| m.insert(key.clone(), at).is_none());
                    format!("did not answer within {ANSWER_WITHIN:?}")
                }
            };
            self.conns.send_if_modified(|m| m.remove(key).is_some());
            tracing::warn!(target: "folder", hash = %hash, holder = %key, why = %said, "folder: a holder gave nothing of what the store lacks");
            why.push((key.clone(), said));
            lacks = self.lacks(hash).await?;
        }
        match lacks {
            None => Ok(None),
            Some(_) => Err(why),
        }
    }

    /// What of `hash` the store lacks, as a get; `None` when it holds it whole.
    async fn lacks(&self, hash: Hash) -> Result<Option<GetRequest>, Vec<(String, String)>> {
        let local = self
            .store
            .remote()
            .local_for_request(GetRequest::blob(hash))
            .await;
        let local =
            local.map_err(|e| vec![(self.host.actor().into(), format!("the store: {e}"))])?;
        Ok((!local.is_complete()).then(|| local.missing()))
    }

    /// `request` asked of `key` on its open connection, else one dialled;
    /// its dial, first byte and last byte, and the path iroh selected for
    /// the connection, traced (perf-1). The dial and the first byte wait
    /// [`ANSWER_WITHIN`], and each next byte as long after the last, never
    /// the whole file: 100 MiB at 25 MB/s takes 4 s (perf-1's README §10,
    /// candidate 1), and a large file's holder is not asleep.
    #[expect(
        clippy::disallowed_methods,
        reason = "times the fetch's dial, first byte and last byte for their trace (perf-1); no decision reads it"
    )]
    async fn fetch(
        &self,
        key: &str,
        request: GetRequest,
    ) -> Result<iroh_blobs::get::Stats, Nothing> {
        let t0 = Instant::now();
        let ms = |t: Instant| t.elapsed().as_micros() as f64 / 1000.0;
        let open = self
            .conns
            .borrow()
            .get(key)
            .filter(|c| c.close_reason().is_none())
            .cloned();
        let conn = match open {
            Some(c) => c,
            None => {
                let peer = actor_key(key).ok_or(Nothing::Said("not a key".into()))?;
                let dial = self.host.endpoint().connect(peer, Alpn::Blobs.bytes());
                let c = tokio::time::timeout(ANSWER_WITHIN, dial)
                    .await
                    .map_err(|_| Nothing::Silent)?
                    .map_err(|e| Nothing::Said(format!("the dial: {e}")))?;
                self.conns
                    .send_modify(|m| drop(m.insert(key.into(), c.clone())));
                c
            }
        };
        let dial_ms = ms(t0);
        let paths = conn.paths();
        let path = paths.iter().find(|p| p.is_selected());
        let (relayed, addr) = (
            path.as_ref().map(|p| p.is_relay()),
            path.as_ref().map(|p| format!("{:?}", p.remote_addr())),
        );
        let rtt_ms = path.as_ref().map(|p| p.rtt().as_micros() as f64 / 1000.0);
        drop(paths);
        let mut got = std::pin::pin!(self.store.remote().execute_get(conn, request).stream());
        let (mut first_ms, mut by) = (None, t0 + ANSWER_WITHIN);
        let done = loop {
            let Ok(next) = tokio::time::timeout_at(by.into(), got.next()).await else {
                break Err(Nothing::Silent);
            };
            match next {
                Some(GetProgressItem::Progress(_)) => {
                    first_ms = first_ms.or(Some(ms(t0)));
                    by = Instant::now() + ANSWER_WITHIN;
                }
                Some(GetProgressItem::Done(stats)) => break Ok(stats),
                Some(GetProgressItem::Error(e)) => break Err(Nothing::Said(format!("{e:?}"))),
                None => break Err(Nothing::Said("the get ended without a result".into())),
            }
        };
        tracing::debug!(target: "folder", holder = %key, dial_ms, first_ms, last_ms = ms(t0), ?relayed, addr, rtt_ms, ok = done.is_ok(), "folder: the fetch's phases, and the path iroh selected");
        done
    }
}

/// Why a holder gave nothing of a fetch.
enum Nothing {
    /// No byte within [`ANSWER_WITHIN`] of the ask or of the last byte:
    /// taken as asleep for [`ASLEEP_FOR`].
    Silent,
    /// An answer refused, or a dial that failed, by what it said.
    Said(String),
}

/// The provider: every get asked of the record for its dialer, and its
/// bytes logged. Each connection's dialer is kept by its id from its
/// connect, which the provider awaits before it reads a request.
#[expect(
    clippy::disallowed_methods,
    reason = "times each get's read of the record for its trace (perf-1); no decision reads it"
)]
fn provider(store: &Store, folders: Folders) -> BlobsProtocol {
    let mask = EventMask {
        connected: ConnectMode::Intercept,
        get: RequestMode::InterceptLog,
        observe: ObserveMode::Intercept,
        ..EventMask::DEFAULT
    };
    let (events, mut rx) = EventSender::channel(32, mask);
    tokio::spawn(async move {
        let mut dialers: BTreeMap<u64, String> = BTreeMap::new();
        while let Some(message) = rx.recv().await {
            // Every other request, an observe or a get of many, is refused by
            // its answer dropped.
            let asked = match message {
                ProviderMessage::ClientConnected(c) => {
                    let (conn, id) = (c.inner.connection_id, c.inner.endpoint_id);
                    dialers.extend(id.map(|id| (conn, key_actor(&id))));
                    c.tx.send(Ok(())).await.ok();
                    continue;
                }
                ProviderMessage::ConnectionClosed(c) => {
                    dialers.remove(&c.inner.connection_id);
                    continue;
                }
                ProviderMessage::GetRequestReceived(asked) => asked,
                _ => continue,
            };
            let (hash, ranges) = (
                asked.inner.request.hash,
                format!("{:?}", asked.inner.request.ranges),
            );
            let (conn, request) = (asked.inner.connection_id, asked.inner.request_id);
            let t0 = Instant::now();
            let dialer = dialers.get(&conn);
            let shares = dialer.is_some_and(|d| folders.serves(hash, d));
            let record_ms = t0.elapsed().as_micros() as f64 / 1000.0;
            if !shares {
                tracing::info!(target: "folder", %hash, conn, request, ?dialer, ranges, record_ms, "folder: refused — no synced folder of a group the dialer stands in places the hash");
                asked.tx.send(Err(AbortReason::Permission)).await.ok();
                continue;
            }
            tracing::info!(target: "folder", %hash, conn, request, ?dialer, ranges, record_ms, "folder: serving — a synced folder of a group the dialer stands in places the hash");
            asked.tx.send(Ok(())).await.ok();
            let mut updates = asked.rx;
            tokio::spawn(async move {
                let mut last = 0;
                while let Ok(Some(update)) = updates.recv().await {
                    let (stats, done) = match update {
                        RequestUpdate::Progress(p) => {
                            last = p.end_offset;
                            continue;
                        }
                        RequestUpdate::Started(_) => continue,
                        RequestUpdate::Completed(c) => (c.stats, true),
                        RequestUpdate::Aborted(a) => (a.stats, false),
                    };
                    tracing::info!(target: "folder", %hash, conn, request, done, last, payload = stats.payload_bytes_sent, other = stats.other_bytes_sent, "folder: sent");
                }
            });
        }
    });
    BlobsProtocol::new(store, Some(events))
}

/// The record API's folder route: what this computer's apps ask. POST
/// keeps a directory in step with its folder.
pub(crate) fn router(folders: Folders) -> Router {
    Router::new()
        .route("/v1/folders", post(ask))
        .with_state(folders)
}

/// What `svrngs share --folder <dir> --group <group>` asks of the running
/// node, the one process that writes this computer's acts: keep `dir` in
/// step with the folder of its name in the group whose id is `group`
/// (sy-1; syp-4), `{"dir": dir, "group": id}`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FolderAsk {
    pub dir: PathBuf,
    pub group: String,
}

async fn ask(State(f): State<Folders>, Json(asked): Json<FolderAsk>) -> Response {
    match f.both_ways(asked.dir, asked.group) {
        Ok(name) => Json(serde_json::json!({ "name": name })).into_response(),
        Err(why) => (StatusCode::CONFLICT, why.to_string()).into_response(),
    }
}
