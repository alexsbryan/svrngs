// SPDX-License-Identifier: AGPL-3.0-or-later
//! A group's folders' bytes (gf-1, group folders' second spike, kept for
//! sy-1's synced folder; DESIGN §9). A synced folder's files go into this
//! computer's iroh-blobs store, and its acts name them by hash
//! ([`svrngs_core::SyncFolder`]). The provider serves a hash to a dialer
//! standing in a group held here, and only while a synced folder's layout
//! places it, asked of the record on every request. A fetch takes a range
//! of a hash by verified range from a holder the record names; bytes that
//! do not match the hash never reach the store, and each holder asked is
//! named with why it gave nothing. Streaming a folder by range on loopback,
//! and the one-way share it served, were dropped at syp-1 (operator,
//! 2026-10-10): a group's folder is synced onto its members' disks.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use iroh::endpoint::Connection;
use iroh_blobs::api::blobs::{AddPathOptions, ImportMode};
use iroh_blobs::api::remote::GetProgressItem;
use iroh_blobs::api::Store;
use iroh_blobs::protocol::{ChunkRanges, ChunkRangesExt, GetRequest};
use iroh_blobs::provider::events::{
    AbortReason, EventMask, EventSender, ObserveMode, ProviderMessage, RequestMode, RequestUpdate,
};
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::{BlobFormat, BlobsProtocol, Hash};
use n0_future::StreamExt as _;
use std::convert::Infallible;

use svrngs_core::{Command, ContentHash, Recorded};
use tokio::sync::{watch, Notify};

use crate::iroh::{actor_key, Alpn, Endpoint};
use crate::node::ANSWER_WITHIN;
use crate::RingRail;

#[path = "synced.rs"]
mod synced;

/// What a fetch asks a holder for at once.
const PIECE: u64 = 1 << 20;

#[derive(Debug, thiserror::Error)]
pub enum FolderError {
    #[error("the folder {0} cannot be read: {1}")]
    Read(PathBuf, std::io::Error),
    #[error("the store refused {0}: {1}")]
    Store(PathBuf, String),
    #[error("the record refused the folder: {0}")]
    Record(String),
}

/// This computer's folders and its fetches' memory: one per node.
#[derive(Clone)]
pub struct Folders {
    store: Store,
    rail: Arc<RingRail>,
    endpoint: Endpoint,
    actor: String,
    nudge: Arc<Notify>,
    data_dir: PathBuf,
    /// Each holder's open connection, and when one last did not answer.
    conns: watch::Sender<BTreeMap<String, Connection>>,
    unanswered: watch::Sender<BTreeMap<String, Instant>>,
}

impl Folders {
    /// The store under `data_dir/blobs`, and its provider.
    pub(crate) async fn start(
        data_dir: &Path,
        rail: Arc<RingRail>,
        endpoint: Endpoint,
        actor: String,
        nudge: Arc<Notify>,
    ) -> Result<(Self, BlobsProtocol), String> {
        let fs = FsStore::load(data_dir.join("blobs"))
            .await
            .map_err(|e| format!("the blob store: {e}"))?;
        let store: Store = (*fs).clone();
        let folders = Folders {
            store: store.clone(),
            rail,
            endpoint,
            actor,
            nudge,
            data_dir: data_dir.to_path_buf(),
            conns: watch::Sender::new(BTreeMap::new()),
            unanswered: watch::Sender::new(BTreeMap::new()),
        };
        tracing::info!(target: "folder", "folder: the store is open");
        // The store's actor lives as long as the node.
        tokio::spawn(async move {
            let _keep = fs;
            std::future::pending::<()>().await
        });
        let blobs = provider(&store, folders.clone());
        Ok((folders, blobs))
    }

    /// The record of every group held here, read for its folder. A group
    /// that cannot be listed or admitted is skipped, untraced: #113, and
    /// #117 for the files a synced directory then deletes.
    fn admissions(&self) -> impl Iterator<Item = svrngs_core::Admission> + '_ {
        let namespaces = self.rail.namespaces().unwrap_or_default();
        namespaces
            .into_iter()
            .filter_map(|ns| self.rail.admission(&ns).ok())
    }

    /// Every file under `dir`, by its path under it. A link is not
    /// followed, since it may reach outside the folder.
    fn walk(&self, dir: &Path) -> Result<Vec<(String, PathBuf, std::fs::Metadata)>, FolderError> {
        let mut files = Vec::new();
        let mut walk = vec![dir.to_path_buf()];
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
                    let rel = path.strip_prefix(dir).unwrap_or(&path).to_string_lossy();
                    files.push((rel.into_owned(), path, meta));
                }
            }
        }
        Ok(files)
    }

    /// The file at `path` into the store by reference, by its hash.
    async fn take(&self, path: &Path, mode: ImportMode) -> Result<Hash, FolderError> {
        let options = AddPathOptions {
            path: path.to_path_buf(),
            mode,
            format: BlobFormat::Raw,
        };
        let tag = self.store.add_path_with_opts(options).with_tag().await;
        let tag = tag.map_err(|e| FolderError::Store(path.to_path_buf(), e.to_string()))?;
        Ok(tag.hash)
    }

    /// `act` written in every group held here where this computer stands,
    /// and the round nudged; what it recorded in each.
    async fn write<C: Command<Refusal = Infallible>>(
        &self,
        act: &C,
        folder: &str,
        files: usize,
    ) -> Result<Vec<Recorded>, FolderError> {
        let mut recorded = Vec::new();
        for ns in self
            .rail
            .namespaces()
            .map_err(|e| FolderError::Record(e.to_string()))?
        {
            match self.rail.commit(&ns, act) {
                Ok(Ok(done)) => {
                    tracing::info!(target: "folder", namespace = %ns, folder, files, "folder: written to the record");
                    recorded.push(done);
                }
                Ok(Err(why)) => {
                    tracing::debug!(target: "folder", namespace = %ns, ?why, "folder: not written — this computer does not stand there")
                }
                Err(e) => return Err(FolderError::Record(e.to_string())),
            }
        }
        self.nudge.notify_one();
        Ok(recorded)
    }

    /// Whether a synced folder places `hash`, whoever wrote it (sy-1).
    fn shares(&self, hash: Hash) -> bool {
        let hash = &ContentHash::from_bytes(hash.into());
        self.placed().iter().any(|p| &p.hash == hash)
    }

    /// Make `[start, end)` of `hash` held here, verified, asking each holder
    /// in turn, one that did not answer lately last. `Ok` names the holder
    /// it came from, or none when the store held it; `Err` names each
    /// holder asked and why it gave nothing.
    #[expect(
        clippy::disallowed_methods,
        reason = "the fetch is an adapter: it times what a holder takes, and when one did not answer"
    )]
    async fn hold(
        &self,
        hash: Hash,
        holders: &[String],
        start: u64,
        end: u64,
    ) -> Result<Option<String>, Vec<(String, String)>> {
        let want = GetRequest::blob_ranges(hash, ChunkRanges::bytes(start..end));
        let local = self.store.remote().local_for_request(want).await;
        let local = local.map_err(|e| vec![(self.actor.clone(), format!("the store: {e}"))])?;
        if local.is_complete() {
            return Ok(None);
        }
        let missing = local.missing();
        let mut order: Vec<&String> = holders.iter().filter(|k| **k != self.actor).collect();
        let lately = self.unanswered.borrow().clone();
        order.sort_by_key(|k| {
            lately
                .get(*k)
                .is_some_and(|t| t.elapsed() < Duration::from_secs(60))
        });
        let mut why = Vec::new();
        for key in order {
            let t0 = Instant::now();
            let got = tokio::time::timeout(ANSWER_WITHIN, self.fetch(key, missing.clone())).await;
            let said = match got {
                Ok(Ok(stats)) => {
                    self.unanswered
                        .send_if_modified(|m| m.remove(key).is_some());
                    tracing::info!(target: "folder", hash = %hash, holder = %key, start, end, payload = stats.payload_bytes_read, other = stats.other_bytes_read, ms = t0.elapsed().as_millis() as u64, "folder: fetched, verified");
                    return Ok(Some(key.clone()));
                }
                Ok(Err(e)) => e,
                Err(_) => {
                    // Kept whether new or not; nothing waits on a change.
                    let at = Instant::now();
                    self.unanswered
                        .send_if_modified(|m| m.insert(key.clone(), at).is_none());
                    format!("did not answer within {ANSWER_WITHIN:?}")
                }
            };
            self.conns.send_if_modified(|m| m.remove(key).is_some());
            tracing::warn!(target: "folder", hash = %hash, holder = %key, start, end, why = %said, "folder: a holder gave nothing of the range");
            why.push((key.clone(), said));
        }
        Err(why)
    }

    /// `request` asked of `key` on its open connection, else one dialled;
    /// its dial, first byte and last byte, and the path iroh selected for
    /// the connection, traced (perf-1).
    #[expect(
        clippy::disallowed_methods,
        reason = "times the fetch's dial, first byte and last byte for their trace (perf-1); no decision reads it"
    )]
    async fn fetch(
        &self,
        key: &str,
        request: GetRequest,
    ) -> Result<iroh_blobs::get::Stats, String> {
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
                let peer = actor_key(key).ok_or("not a key")?;
                let c = self
                    .endpoint
                    .connect(peer, Alpn::Blobs.bytes())
                    .await
                    .map_err(|e| format!("the dial: {e}"))?;
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
        let mut first_ms = None;
        let done = loop {
            match got.next().await {
                Some(GetProgressItem::Progress(_)) => {
                    first_ms = first_ms.or(Some(ms(t0)));
                }
                Some(GetProgressItem::Done(stats)) => break Ok(stats),
                Some(GetProgressItem::Error(e)) => break Err(format!("{e:?}")),
                None => break Err("the get ended without a result".into()),
            }
        };
        tracing::debug!(target: "folder", holder = %key, dial_ms, first_ms, last_ms = ms(t0), ?relayed, addr, rtt_ms, ok = done.is_ok(), "folder: the fetch's phases, and the path iroh selected");
        done
    }
}

/// The provider: every get asked of the record, and its bytes logged.
#[expect(
    clippy::disallowed_methods,
    reason = "times each get's read of the record for its trace (perf-1); no decision reads it"
)]
fn provider(store: &Store, folders: Folders) -> BlobsProtocol {
    let mask = EventMask {
        get: RequestMode::InterceptLog,
        observe: ObserveMode::Intercept,
        ..EventMask::DEFAULT
    };
    let (events, mut rx) = EventSender::channel(32, mask);
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            // Every other request, an observe or a get of many, is refused by
            // its answer dropped.
            let ProviderMessage::GetRequestReceived(asked) = message else {
                continue;
            };
            let (hash, ranges) = (
                asked.inner.request.hash,
                format!("{:?}", asked.inner.request.ranges),
            );
            let (conn, request) = (asked.inner.connection_id, asked.inner.request_id);
            let t0 = Instant::now();
            let shares = folders.shares(hash);
            let record_ms = t0.elapsed().as_micros() as f64 / 1000.0;
            if !shares {
                tracing::info!(target: "folder", %hash, conn, request, ranges, record_ms, "folder: refused — no synced folder places the hash");
                asked.tx.send(Err(AbortReason::Permission)).await.ok();
                continue;
            }
            tracing::info!(target: "folder", %hash, conn, request, ranges, record_ms, "folder: serving — a synced folder places the hash");
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

/// What `svrngs share --folder <dir> --both-ways` asks of the running node,
/// the one process that writes this computer's acts: `{"bothways": dir}`.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FolderAsk {
    /// Keep a directory in step with the folder of its name (sy-1).
    BothWays(PathBuf),
}

async fn ask(State(f): State<Folders>, Json(asked): Json<FolderAsk>) -> Json<serde_json::Value> {
    let FolderAsk::BothWays(dir) = asked;
    Json(serde_json::json!({ "name": f.both_ways(dir) }))
}
