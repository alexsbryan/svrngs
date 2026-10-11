// SPDX-License-Identifier: AGPL-3.0-or-later
//! A directory kept in step with its folder in one group, synced both ways
//! (sy-1, the mesh-Dropbox spike; syp-4; DESIGN §9). Once a second: the
//! directory is walked as a shared folder is, each new or changed file
//! copied into the store and one act written in the group of the changes,
//! each naming the entry it replaces ([`svrngs_core::SyncFolder`]); then
//! the group's layout ([`svrngs_core::Admission::layout`]) is written into
//! the directory — what it lacks fetched whole, verified, from a member
//! standing in the group, the writer first, and what it no longer places
//! deleted — never touching a
//! file changed on disk since it was last taken. A file is copied, not
//! referenced as a shared folder's is, since the store must keep a version
//! the directory has since overwritten: a conflict's other side. What was
//! last taken or written at each path, and from which entry, is kept in the
//! data dir, so an edit made while the node was stopped is sent with its
//! base; a kept state that cannot be read stops its directory's steps, said
//! by its name, never read as nothing held (syp-4 e). The directories it
//! syncs are kept in the data dir too, so a restart resumes each unasked,
//! one loop each. A step whose group's record, or who stands in it, cannot
//! be read stops before it sends or deletes anything, said by the group's
//! name.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use iroh_blobs::api::TempTag;
use iroh_blobs::Hash;
use serde::{Deserialize, Serialize};
use svrngs_core::{Change, ContentHash, Placed, SyncFolder, SyncPath};

use n0_future::{BufferedStreamExt as _, StreamExt as _};

use super::{FolderAsk, FolderError, Folders, PIECE};
use crate::register::Standing;

/// The keys standing in the group a step syncs with.
type Holders = BTreeSet<String>;

/// How many files a step fetches at once (syp-4 f): sy-1's apply fetched
/// each after the last, 0.25-0.38 s a file through relays at M1 (the sy
/// README §7). Set by the measurement in demo/runs/syp/2026-10-10/production.
const FETCHES_AT_ONCE: usize = 8;

/// Where the directories this computer syncs are kept.
const SYNCED: &str = "synced.json";

/// The directories this computer syncs, each by its group and folder, one
/// directory each, so one loop owns a directory and its kept state (#119);
/// or why the list kept cannot be read, so no ask writes over it.
pub(super) type Syncing = Result<BTreeMap<(String, String), PathBuf>, (PathBuf, String)>;

/// The folder `dir` keeps: its name.
fn folder_of(dir: &Path) -> String {
    dir.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// The directories kept under `data_dir`: none when none was ever kept, and
/// a list that cannot be read said by its path.
pub(super) fn syncing_in(data_dir: &Path) -> Syncing {
    let at = data_dir.join(SYNCED);
    let asks: Vec<FolderAsk> = match std::fs::read(&at) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| (at.clone(), e.to_string()))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err((at, e.to_string())),
    };
    let kept = asks
        .into_iter()
        .map(|a| ((a.group, folder_of(&a.dir)), a.dir));
    Ok(kept.collect())
}

/// `bytes` at `path` whole or not at all: a stop mid-write leaves the last
/// whole one, never a torn file the next start would refuse.
fn write_whole(path: &Path, bytes: &[u8]) -> Result<(), FolderError> {
    let part = path.with_extension("part");
    std::fs::write(&part, bytes).map_err(io(&part))?;
    std::fs::rename(&part, path).map_err(io(path))
}

/// What this computer last took or wrote at a path: the record's entry, and
/// the file's mtime then.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Held {
    p: Placed,
    mtime: SystemTime,
}

/// Whether the file at `path` is as `held` says: absent when nothing is.
fn unchanged(held: Option<&Held>, path: &Path) -> bool {
    match (held, std::fs::symlink_metadata(path)) {
        (None, meta) => meta.is_err(),
        (Some(h), Ok(m)) => m.len() == h.p.size && m.modified().ok() == Some(h.mtime),
        (Some(_), Err(_)) => false,
    }
}

fn io(path: &Path) -> impl Fn(std::io::Error) -> FolderError + '_ {
    move |e| FolderError::Read(path.to_path_buf(), e)
}

impl Folders {
    /// Keep `dir` in step with the folder of its name in `group`, once a
    /// second while the node runs and after each start; its name, or why
    /// not: this computer does not stand in the group, the directories it
    /// syncs cannot be read, or the folder or the directory is synced
    /// already elsewhere. Asked again as it is, it is answered as it is.
    pub fn both_ways(&self, dir: PathBuf, group: String) -> Result<String, FolderError> {
        let folder = folder_of(&dir);
        let standing = self.host.rail().standing(&group);
        let standing = standing.map_err(|e| FolderError::Unread(group.clone(), e.to_string()))?;
        if !matches!(&standing, Standing::Members(m) if m.contains_key(self.host.actor())) {
            tracing::debug!(target: "folder", namespace = group, folder, "synced: refused — this computer does not stand in the group");
            return Err(FolderError::NotStanding(group));
        }
        let key = (group.clone(), folder.clone());
        let mut added = Ok(false);
        self.syncing.send_if_modified(|kept| {
            added = match kept {
                Err((at, why)) => Err(FolderError::Kept(at.clone(), why.clone())),
                Ok(kept) if kept.get(&key) == Some(&dir) => Ok(false),
                Ok(kept) if kept.contains_key(&key) || kept.values().any(|d| d == &dir) => {
                    Err(FolderError::Synced(dir.clone()))
                }
                Ok(kept) => {
                    let mut asks: Vec<FolderAsk> = kept
                        .iter()
                        .map(|((group, _), dir)| FolderAsk {
                            dir: dir.clone(),
                            group: group.clone(),
                        })
                        .collect();
                    asks.push(FolderAsk {
                        dir: dir.clone(),
                        group: group.clone(),
                    });
                    let bytes = serde_json::to_vec(&asks).unwrap_or_default();
                    write_whole(&self.data_dir.join(SYNCED), &bytes)
                        .map(|()| kept.insert(key, dir.clone()).is_none())
                }
            };
            matches!(added, Ok(true))
        });
        match added {
            Ok(true) => self.keep_in_step(dir, group, folder.clone()),
            Ok(false) => {
                tracing::debug!(target: "folder", namespace = group, folder, "synced: kept in step already, as asked")
            }
            Err(why) => {
                tracing::warn!(target: "folder", namespace = group, folder, %why, "synced: refused");
                return Err(why);
            }
        }
        Ok(folder)
    }

    /// Every directory kept as synced resumed, at the node's start; none
    /// when the list kept cannot be read, said by its path.
    pub(super) fn resume(&self) {
        let kept = self.syncing.borrow().clone();
        match kept {
            Ok(kept) => {
                for ((group, folder), dir) in kept {
                    self.keep_in_step(dir, group, folder);
                }
            }
            Err((at, why)) => {
                tracing::warn!(target: "folder", at = %at.display(), why, "synced: the directories this computer syncs cannot be read, so none resumes and none is added")
            }
        }
    }

    /// `dir` stepped once a second while the node runs: its one loop.
    fn keep_in_step(&self, dir: PathBuf, group: String, folder: String) {
        tracing::info!(target: "folder", namespace = group, folder, dir = %dir.display(), "synced: the directory kept in step with the group's folder");
        let f = self.clone();
        tokio::spawn(async move {
            loop {
                if let Err(why) = f.step(&dir, &group, &folder).await {
                    tracing::warn!(target: "folder", namespace = group, folder, %why, "synced: the step stopped short; the next one tries again");
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    /// Who stands in `group`, as [`crate::node::Host::standings`] reads it;
    /// a group it leaves out is one whose register cannot be read here.
    fn holders(&self, group: &str) -> Result<Holders, FolderError> {
        let groups = self.host.standings().unwrap_or_default();
        let found = groups.into_iter().find(|(ns, _)| ns == group);
        let unread = || FolderError::Unread(group.into(), "who stands in it cannot be read".into());
        Ok(found.ok_or_else(unread)?.1.into_keys().collect())
    }

    /// Every version a synced directory on this computer holds, as each
    /// was last taken or written: what the store keeps beside the layouts.
    pub(super) fn held_everywhere(&self) -> Result<Vec<ContentHash>, FolderError> {
        let unread = |e| FolderError::Read(self.data_dir.clone(), e);
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&self.data_dir).map_err(unread)? {
            let path = entry.map_err(unread)?.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !(name.starts_with("synced-") && name.ends_with(".json")) {
                continue;
            }
            let bytes = std::fs::read(&path).map_err(io(&path))?;
            let held: BTreeMap<SyncPath, Held> = serde_json::from_slice(&bytes)
                .map_err(|e| FolderError::Kept(path.clone(), e.to_string()))?;
            out.extend(held.into_values().map(|h| h.p.hash));
        }
        Ok(out)
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "times each phase of a step for its trace (perf-1); no decision reads it"
    )]
    async fn step(&self, dir: &Path, group: &str, folder: &str) -> Result<(), FolderError> {
        let t0 = Instant::now();
        let ms = |t: Instant| t.elapsed().as_millis() as u64;
        // Read before anything is sent or deleted, so a group that cannot
        // be read stops the step by its name.
        let (holders, placed) = (self.holders(group)?, self.layout(group)?);
        tracing::debug!(target: "folder", folder, placed = placed.len(), holders = holders.len(), ms = ms(t0), "synced: the layout read");
        let kept = self.data_dir.join(format!("synced-{group}-{folder}.json"));
        // Nothing kept is nothing held; a kept state that cannot be read is
        // never read as that, or every file would go again with no base.
        let mut held: BTreeMap<SyncPath, Held> = match std::fs::read(&kept) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| FolderError::Kept(kept.clone(), e.to_string()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(e) => return Err(FolderError::Kept(kept.clone(), e.to_string())),
        };
        let mut changes = Vec::new();
        let mut taken = BTreeMap::new();
        let mut seen = BTreeSet::new();
        // What this step took or fetched, held from the store's sweep until
        // the kept state names it.
        let mut hold: Vec<TempTag> = Vec::new();
        let (read_ms, t) = (ms(t0), Instant::now());
        let walked = self.walk(dir)?;
        let (files, walk_ms, t) = (walked.len(), ms(t), Instant::now());
        for (rel, path, meta) in walked {
            // Parsed on the write that makes it (syp-4 c): a name this
            // computer's disk holds that no folder carries is not sent.
            let rel = match SyncPath::parse(&rel) {
                Ok(rel) => rel,
                Err(why) => {
                    tracing::debug!(target: "folder", folder, path = rel, %why, "synced: not shared — not a path a folder carries");
                    continue;
                }
            };
            seen.insert(rel.clone());
            let before = held.get_mut(&rel);
            if unchanged(before.as_deref(), &path) {
                continue;
            }
            tracing::debug!(target: "folder", folder, path = %rel, size = meta.len(), "synced: found on disk, changed since it was last held");
            let took = Instant::now();
            let tag = self.take(&path).await?;
            let hash = tag.hash();
            hold.push(tag);
            tracing::debug!(target: "folder", folder, path = %rel, %hash, ms = ms(took), "synced: taken into the store");
            let hash = ContentHash::from_bytes(hash.into());
            let mtime = meta.modified().map_err(io(&path))?;
            match before {
                Some(h) if h.p.hash == hash => h.mtime = mtime,
                before => {
                    let base = before.map(|h| h.p.entry.clone());
                    let now = Some((hash, meta.len()));
                    changes.push(Change {
                        path: rel.clone(),
                        now,
                        base,
                    });
                    taken.insert(rel, (hash, mtime));
                }
            }
        }
        changes.extend(
            held.iter()
                .filter(|(rel, _)| !seen.contains(*rel))
                .map(|(rel, h)| {
                    let base = Some(h.p.entry.clone());
                    Change {
                        path: rel.clone(),
                        now: None,
                        base,
                    }
                }),
        );
        let (take_ms, t) = (ms(t), Instant::now());
        let sent = changes.len();
        if !changes.is_empty() {
            let act = SyncFolder {
                folder: folder.into(),
                changes,
            };
            self.write(&act, group, folder, act.changes.len()).await?;
            for c in act.changes {
                tracing::info!(target: "folder", folder, path = %c.path, now = ?c.now, base = ?c.base, "synced: a change sent");
                held.remove(&c.path);
            }
        }
        let (write_ms, t) = (ms(t), Instant::now());
        // Read again past this step's own act, which places what it took.
        // Should the record turn unreadable between the two reads, the kept
        // state stays as it was, and the next step sends the same change
        // again: a second head of this computer's, kept beside, never lost.
        let placed = match sent {
            0 => placed,
            _ => self.layout(group)?,
        };
        let placed = placed.into_iter().filter(|p| p.folder == folder).collect();
        hold.extend(
            self.apply(dir, folder, placed, &holders, &mut held, &taken)
                .await,
        );
        let (apply_ms, t) = (ms(t), Instant::now());
        // What this step took or fetched made durable before the kept state
        // names it: the store commits its writes in batches up to 500 ms
        // apart (iroh-blobs 0.103 `BatchOptions`), and a node stopped
        // between lost a fetched file's bytes (syp-4 d, tests/kept.rs).
        if !hold.is_empty() {
            let synced = self.store.sync_db().await;
            synced.map_err(|e| FolderError::Store(dir.into(), e.to_string()))?;
            tracing::debug!(target: "folder", folder, held = hold.len(), "synced: what the step took or fetched made durable");
        }
        let wrote = write_whole(&kept, &serde_json::to_vec(&held).unwrap_or_default());
        drop(hold);
        tracing::debug!(target: "folder", folder, files, sent, read_ms, walk_ms, take_ms, write_ms, apply_ms, kept_ms = ms(t), ms = ms(t0), "synced: a step");
        wrote
    }

    /// The layout written into `dir`, what `held` says updated. A file just
    /// `taken` and sent is held as its entry, wherever the record places
    /// that entry: beside, when it met another edit, and the record's
    /// choice is then written at its path. What it fetched, held from the
    /// store's sweep for the step to drop once the kept state names it.
    /// Up to [`FETCHES_AT_ONCE`] files are fetched at once.
    async fn apply(
        &self,
        dir: &Path,
        folder: &str,
        placed: Vec<Placed>,
        standing: &Holders,
        held: &mut BTreeMap<SyncPath, Held>,
        taken: &BTreeMap<SyncPath, (ContentHash, SystemTime)>,
    ) -> Vec<TempTag> {
        let mut hold = Vec::new();
        tracing::debug!(target: "folder", folder, placed = placed.len(), "synced: the layout to write");
        let paths: BTreeSet<&SyncPath> = placed.iter().map(|p| &p.path).collect();
        for (rel, h) in held.clone() {
            let at = dir.join(rel.as_str());
            if !paths.contains(&rel) && unchanged(Some(&h), &at) {
                let gone = std::fs::remove_file(&at);
                tracing::info!(target: "folder", folder, path = %rel, ?gone, "synced: deleted, the record placing nothing there");
                held.remove(&rel);
            }
        }
        for (path, &(hash, mtime)) in taken {
            let mine = |q: &&Placed| {
                q.entry.key == self.host.actor() && &q.entry.path == path && q.hash == hash
            };
            if let Some(q) = placed.iter().find(mine) {
                let p = Placed {
                    path: path.clone(),
                    ..q.clone()
                };
                held.insert(path.clone(), Held { p, mtime });
            }
        }
        let mut fetches = Vec::new();
        for p in placed {
            let (at, before) = (dir.join(p.path.as_str()), held.get(&p.path));
            if before.is_some_and(|h| h.p.entry == p.entry) || !unchanged(before, &at) {
                continue;
            }
            if before.is_none_or(|h| h.p.hash != p.hash) {
                fetches.push((p, at));
                continue;
            }
            let Ok(mtime) = std::fs::metadata(&at).and_then(|m| m.modified()) else {
                continue;
            };
            held.insert(p.path.clone(), Held { p, mtime });
        }
        let fetched: Vec<_> = n0_future::stream::iter(fetches)
            .map(|(p, at)| self.fetch_into(folder, p, at, standing))
            .buffered_unordered(FETCHES_AT_ONCE)
            .collect()
            .await;
        for (p, at, tag) in fetched.into_iter().flatten() {
            hold.push(tag);
            let Ok(mtime) = std::fs::metadata(&at).and_then(|m| m.modified()) else {
                continue;
            };
            held.insert(p.path.clone(), Held { p, mtime });
        }
        hold
    }

    /// `p` fetched whole, verified, from its writer first and then the
    /// group's other members, and written at `at`; with the tag that holds
    /// it from the store's sweep, or none, said why.
    #[expect(
        clippy::disallowed_methods,
        reason = "times each fetch and each export for their traces (perf-1); no decision reads it"
    )]
    async fn fetch_into(
        &self,
        folder: &str,
        p: Placed,
        at: PathBuf,
        standing: &Holders,
    ) -> Option<(Placed, PathBuf, TempTag)> {
        let ms = |t: Instant| t.elapsed().as_millis() as u64;
        let hash = Hash::from_bytes(*p.hash.as_bytes());
        tracing::debug!(target: "folder", folder, path = %p.path, %hash, size = p.size, from = p.entry.key, seq = p.entry.seq, "synced: to fetch, the layout placing it here");
        let tag = match self.store.tags().temp_tag(hash).await {
            Ok(tag) => tag,
            Err(why) => {
                tracing::warn!(target: "folder", folder, path = %p.path, %hash, %why, "synced: not fetched — the store cannot hold it from its sweep");
                return None;
            }
        };
        let fetched = Instant::now();
        let mut from = vec![p.entry.key.clone()];
        from.extend(standing.iter().filter(|k| **k != p.entry.key).cloned());
        let (mut got, mut from_at) = (Ok(None), 0);
        while from_at < p.size && got.is_ok() {
            got = self
                .hold(hash, &from, from_at, (from_at + PIECE).min(p.size))
                .await;
            from_at += PIECE;
        }
        if let Err(why) = got {
            tracing::warn!(target: "folder", folder, path = %p.path, %hash, ?why, "synced: not written — no member gave it, verified");
            return None;
        }
        let (fetch_ms, exported) = (ms(fetched), Instant::now());
        let wrote = self.store.blobs().export(hash, &at).finish().await;
        if let Err(why) = wrote {
            tracing::warn!(target: "folder", folder, path = %p.path, %hash, %why, "synced: not written into the directory");
            return None;
        }
        tracing::info!(target: "folder", folder, path = %p.path, %hash, from = p.entry.key, seq = p.entry.seq, beside = p.beside, fetch_ms, export_ms = ms(exported), "synced: written into the directory");
        Some((p, at, tag))
    }
}
