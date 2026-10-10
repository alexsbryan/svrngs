// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared folders (gf-1, group folders' second spike; DESIGN §9). A folder's
//! files go into this computer's iroh-blobs store by reference, and its act
//! names them by hash ([`svrngs_core::ShareFolder`]). The provider serves a
//! hash to a dialer standing in a group held here, and only while a
//! standing folder of this computer's names it, asked of the record on every
//! request, so a stop takes at the next one. The meeting point's endpoint,
//! on loopback, answers a named hash's HTTP ranges from this computer's
//! store, fetching what it lacks by verified range from a holder the record
//! names, one piece at a time as the response is read; bytes that do not
//! match the hash never reach the store, and the range is refused naming
//! each holder asked and why.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::response::{AppendHeaders, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use iroh::endpoint::Connection;
use iroh_blobs::api::blobs::{AddPathOptions, ImportMode};
use iroh_blobs::api::Store;
use iroh_blobs::protocol::{ChunkRanges, ChunkRangesExt, GetRequest};
use iroh_blobs::provider::events::{
    AbortReason, EventMask, EventSender, ObserveMode, ProviderMessage, RequestMode, RequestUpdate,
};
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::{BlobFormat, BlobsProtocol, Hash};
use std::convert::Infallible;

use svrngs_core::{Command, ContentHash, Recorded, ShareFolder, SharedFile, SharedFolder};
use tokio::sync::{watch, Notify};

use crate::iroh::{actor_key, Alpn, Endpoint};
use crate::node::ANSWER_WITHIN;
use crate::RingRail;

#[path = "synced.rs"]
mod synced;

/// What a fetch asks a holder for at once, and so the most the endpoint
/// holds of a response ahead of what it has sent.
const PIECE: u64 = 1 << 20;

/// A guard the run takes out to watch its plant pass (gf-1's README §3):
/// `SVRNGS_GF_PLANT`, read once at start; never set outside the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plant {
    None,
    /// A holder is waited on without [`ANSWER_WITHIN`] (B6).
    Unbounded,
    /// A folder's walk follows links (B7, the file beside the folder).
    FollowLinks,
    /// The provider serves any hash its store holds (B7, the stopped file).
    Unchecked,
    /// A synced directory is never walked (sy-1's S1).
    Deaf,
    /// The provider serves a synced file only to its writer's (S2).
    OwnOnly,
    /// A synced path's first head alone is placed (S3).
    LaterWins,
}

#[derive(Debug, thiserror::Error)]
pub enum FolderError {
    #[error("the folder {0} cannot be read: {1}")]
    Read(PathBuf, std::io::Error),
    #[error("the store refused {0}: {1}")]
    Store(PathBuf, String),
    #[error("the record refused the folder: {0}")]
    Record(String),
}

/// This computer's folders and its endpoint's memory: one per node.
#[derive(Clone)]
pub struct Folders {
    store: Store,
    rail: Arc<RingRail>,
    endpoint: Endpoint,
    actor: String,
    nudge: Arc<Notify>,
    plant: Plant,
    data_dir: PathBuf,
    /// Where the endpoint listens.
    pub(crate) at: std::net::SocketAddr,
    /// Each holder's open connection, and when one last did not answer.
    conns: watch::Sender<BTreeMap<String, Connection>>,
    unanswered: watch::Sender<BTreeMap<String, Instant>>,
}

impl Folders {
    /// The store under `data_dir/blobs`, its provider, and the endpoint
    /// bound on loopback at `port` (`SVRNGS_FOLDER_PORT`, else any).
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
        let plant = match std::env::var("SVRNGS_GF_PLANT").as_deref() {
            Ok("unbounded") => Plant::Unbounded,
            Ok("follow-links") => Plant::FollowLinks,
            Ok("unchecked") => Plant::Unchecked,
            Ok("deaf") => Plant::Deaf,
            Ok("own-only") => Plant::OwnOnly,
            Ok("later-wins") => Plant::LaterWins,
            _ => Plant::None,
        };
        let port: u16 = std::env::var("SVRNGS_FOLDER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);
        let bound = crate::api::bind(([127, 0, 0, 1], port).into())
            .await
            .map_err(|e| e.to_string())?;
        let store: Store = (*fs).clone();
        let folders = Folders {
            store: store.clone(),
            rail,
            endpoint,
            actor,
            nudge,
            plant,
            data_dir: data_dir.to_path_buf(),
            at: bound.addr(),
            conns: watch::Sender::new(BTreeMap::new()),
            unanswered: watch::Sender::new(BTreeMap::new()),
        };
        tracing::info!(target: "folder", at = %folders.at, ?plant, "folder: the endpoint listens on loopback");
        let served = crate::api::serve(
            Router::new().fallback(bytes).with_state(folders.clone()),
            bound,
        );
        // The listener lives as long as the store's actor: both are the node's.
        tokio::spawn(async move {
            let _keep = (served, fs);
            std::future::pending::<()>().await
        });
        let blobs = provider(&store, folders.clone());
        Ok((folders, blobs))
    }

    /// Every standing folder in every group held here.
    pub fn all(&self) -> Vec<SharedFolder> {
        let namespaces = self.rail.namespaces().unwrap_or_default();
        namespaces
            .iter()
            .filter_map(|ns| self.rail.admission(ns).ok())
            .flat_map(|a| a.folders())
            .collect()
    }

    /// Share `dir`: each file under it into the store by reference, and one
    /// act naming them in every group held here.
    pub async fn share(&self, dir: &Path) -> Result<ShareFolder, FolderError> {
        let name = dir.file_name().unwrap_or_default().to_string_lossy();
        let mut files = Vec::new();
        for (rel, path, meta) in self.walk(dir)? {
            let hash = self.take(&path, ImportMode::TryReference).await?;
            tracing::debug!(target: "folder", path = %rel, %hash, size = meta.len(), "folder: a file taken by reference");
            files.push(SharedFile {
                path: rel,
                size: meta.len(),
                hash: ContentHash::from_bytes(hash.into()),
            });
        }
        files.sort();
        let act = ShareFolder {
            name: name.into_owned(),
            files,
        };
        self.write(&act, &act.name, act.files.len()).await?;
        Ok(act)
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
                let meta = match self.plant {
                    Plant::FollowLinks => std::fs::metadata(&path),
                    _ => std::fs::symlink_metadata(&path),
                };
                let meta = meta.map_err(|e| FolderError::Read(path.clone(), e))?;
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

    /// Stop sharing the folder `name`: an act naming no file.
    pub async fn unshare(&self, name: &str) -> Result<ShareFolder, FolderError> {
        let act = ShareFolder {
            name: name.into(),
            files: Vec::new(),
        };
        self.write(&act, name, 0).await?;
        Ok(act)
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

    /// Whether a standing folder of this computer's names `hash`.
    /// Or a synced folder places it, whoever wrote it (sy-1).
    fn shares(&self, hash: Hash) -> bool {
        let hash = &ContentHash::from_bytes(hash.into());
        let mine = |key: &str| key == self.actor;
        let placed = self.placed().into_iter().filter(|p| &p.hash == hash);
        self.all()
            .iter()
            .any(|f| mine(&f.key) && f.files.iter().any(|x| &x.hash == hash))
            || { placed }.any(|p| self.plant != Plant::OwnOnly || mine(&p.entry.key))
    }

    /// Make `[start, end)` of `hash` held here, verified, asking each holder
    /// in turn, one that did not answer lately last. `Ok` names the holder
    /// it came from, or none when the store held it; `Err` names each
    /// holder asked and why it gave nothing.
    #[expect(
        clippy::disallowed_methods,
        reason = "the endpoint is an adapter: it times what a holder takes, and when one did not answer"
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
        let within = if self.plant == Plant::Unbounded {
            Duration::from_secs(3600)
        } else {
            ANSWER_WITHIN
        };
        let mut why = Vec::new();
        for key in order {
            let t0 = Instant::now();
            let got = tokio::time::timeout(within, self.fetch(key, missing.clone())).await;
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
                    format!("did not answer within {within:?}")
                }
            };
            self.conns.send_if_modified(|m| m.remove(key).is_some());
            tracing::warn!(target: "folder", hash = %hash, holder = %key, start, end, why = %said, "folder: a holder gave nothing of the range");
            why.push((key.clone(), said));
        }
        Err(why)
    }

    async fn fetch(
        &self,
        key: &str,
        request: GetRequest,
    ) -> Result<iroh_blobs::get::Stats, String> {
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
        self.store
            .remote()
            .execute_get(conn, request)
            .await
            .map_err(|e| format!("{e:?}"))
    }
}

/// The provider: every get asked of the record, and its bytes logged.
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
            if folders.plant != Plant::Unchecked && !folders.shares(hash) {
                tracing::info!(target: "folder", %hash, conn, request, ranges, "folder: refused — no standing folder of this computer's names the hash");
                asked.tx.send(Err(AbortReason::Permission)).await.ok();
                continue;
            }
            tracing::info!(target: "folder", %hash, conn, request, ranges, "folder: serving — a standing folder of this computer's names the hash");
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

/// `GET /<hash>[/<name>][?type=<content type>]`, a range of a hash a
/// standing folder names: 206 with its bytes, 404 for a hash no folder
/// names, 502 naming each holder when none gave the range's first piece.
#[expect(
    clippy::disallowed_methods,
    reason = "the endpoint is an adapter: it times each answer it logs"
)]
async fn bytes(State(f): State<Folders>, uri: Uri, headers: HeaderMap) -> Response {
    let said = |name| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string()
    };
    let (ua, asked) = (said(header::USER_AGENT), said(header::RANGE));
    let path = uri.path().trim_start_matches('/');
    let named = ContentHash::from_hex(path.split('/').next().unwrap_or(""));
    let folders = f.all();
    let file = named.and_then(|h| {
        folders
            .iter()
            .flat_map(|d| &d.files)
            .find(|x| x.hash == h)
            .cloned()
    });
    let Some(file) = file else {
        tracing::info!(target: "folder", path, %ua, "folder: refused — no standing folder names the hash");
        let refused = [("x-svrngs-refused", "not-shared")];
        return (
            StatusCode::NOT_FOUND,
            refused,
            "no folder shared in your groups names this hash\n",
        )
            .into_response();
    };
    let holders: Vec<String> = folders
        .iter()
        .filter(|d| d.files.iter().any(|x| x.hash == file.hash))
        .map(|d| d.key.clone())
        .collect();
    let size = file.size;
    let Some((start, end)) = range(&asked, size) else {
        return (
            StatusCode::RANGE_NOT_SATISFIABLE,
            [(header::CONTENT_RANGE, format!("bytes */{size}"))],
        )
            .into_response();
    };
    let hash = Hash::from_bytes(*file.hash.as_bytes());
    let first = (start + PIECE).min(end);
    if let Err(why) = f.hold(hash, &holders, start, first).await {
        tracing::warn!(target: "folder", %hash, start, end = first, %ua, ?why, "folder: refused — no holder gave the range, verified");
        let body = serde_json::json!({ "refused": "unverified-or-unanswered", "hash": file.hash, "range": [start, first], "holders": why });
        return (
            StatusCode::BAD_GATEWAY,
            [("x-svrngs-refused", "no-holder")],
            Json(body),
        )
            .into_response();
    }
    let served = Served {
        ua,
        hash,
        start,
        sent: 0,
        digest: blake3::Hasher::new(),
        t0: Instant::now(),
    };
    let stream = n0_future::stream::unfold(
        (f, holders, start, served),
        move |(f, holders, at, mut served)| async move {
            if at >= end {
                return None;
            }
            let to = (at + PIECE).min(end);
            let piece = match f.hold(hash, &holders, at, to).await {
                Err(why) => Err(format!("no holder gave the range, verified: {why:?}")),
                Ok(_) => (f
                    .store
                    .blobs()
                    .export_ranges(hash, at..to)
                    .concatenate()
                    .await)
                    .map_err(|e| e.to_string()),
            };
            let Ok(piece) = piece.inspect_err(|why| {
                tracing::warn!(target: "folder", %hash, start = at, end = to, why, "folder: refused mid-answer");
            }) else {
                return Some((Err(std::io::Error::other("no holder gave the range, verified")), (f, holders, end, served)));
            };
            served.sent += piece.len() as u64;
            served.digest.update(&piece);
            Some((Ok(bytes::Bytes::from(piece)), (f, holders, to, served)))
        },
    );
    let content = uri
        .query()
        .and_then(|q| q.strip_prefix("type="))
        .unwrap_or("application/octet-stream");
    let mut h = vec![
        (header::ACCEPT_RANGES, "bytes".to_string()),
        (header::CONTENT_LENGTH, (end - start).to_string()),
        (header::CONTENT_TYPE, content.replace("%2F", "/")),
    ];
    let status = match asked.is_empty() {
        true => StatusCode::OK,
        false => {
            h.push((
                header::CONTENT_RANGE,
                format!("bytes {start}-{}/{size}", end - 1),
            ));
            StatusCode::PARTIAL_CONTENT
        }
    };
    (status, AppendHeaders(h), Body::from_stream(stream)).into_response()
}

/// `bytes=a-b` or `bytes=a-` within `size`, as `[start, end)`; none asked
/// is the whole.
fn range(asked: &str, size: u64) -> Option<(u64, u64)> {
    if asked.is_empty() {
        return (size > 0).then_some((0, size));
    }
    let (a, b) = asked.strip_prefix("bytes=")?.split_once('-')?;
    let start: u64 = a.parse().ok()?;
    let end = match b {
        "" => size,
        b => b.parse::<u64>().ok()?.saturating_add(1).min(size),
    };
    (start < end).then_some((start, end))
}

/// One answer as it went: logged when its body ends or is dropped.
struct Served {
    ua: String,
    hash: Hash,
    start: u64,
    sent: u64,
    digest: blake3::Hasher,
    t0: Instant,
}

impl Drop for Served {
    fn drop(&mut self) {
        tracing::info!(target: "folder", hash = %self.hash, start = self.start, sent = self.sent, blake3 = %self.digest.finalize(), ms = self.t0.elapsed().as_millis() as u64, ua = %self.ua, "folder: served");
    }
}

/// The record API's folder routes: what this computer's apps ask. GET
/// names the endpoint, every standing folder and the holders lately
/// unanswered; POST shares a folder or stops one.
pub(crate) fn router(folders: Folders) -> Router {
    Router::new()
        .route("/v1/folders", get(list).post(ask))
        .with_state(folders)
}

async fn list(State(f): State<Folders>) -> Json<serde_json::Value> {
    let unanswered: Vec<String> = f.unanswered.borrow().keys().cloned().collect();
    Json(
        serde_json::json!({ "endpoint": format!("http://{}/", f.at), "folders": f.all(), "unanswered": unanswered }),
    )
}

/// What `svrngs share --folder` and `unshare --folder` ask of the running
/// node, the one process that writes this computer's acts: `{"share": dir}`
/// or `{"stop": name}`.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FolderAsk {
    Share(PathBuf),
    Stop(String),
    /// Keep a directory in step with the folder of its name (sy-1).
    BothWays(PathBuf),
}

async fn ask(State(f): State<Folders>, Json(asked): Json<FolderAsk>) -> Response {
    let done = match asked {
        FolderAsk::Share(dir) => f.share(&dir).await,
        FolderAsk::Stop(name) => f.unshare(&name).await,
        FolderAsk::BothWays(dir) => {
            return Json(serde_json::json!({ "name": f.both_ways(dir) })).into_response()
        }
    };
    match done {
        Ok(f) => Json(serde_json::json!({ "name": f.name, "files": f.files })).into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
    }
}
