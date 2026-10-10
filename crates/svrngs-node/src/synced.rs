// SPDX-License-Identifier: AGPL-3.0-or-later
//! A directory kept in step with its folder, synced both ways (sy-1, the
//! mesh-Dropbox spike; DESIGN §9). Once a second: the directory is walked
//! as a shared folder is, each new or changed file copied into the store
//! and one act written of the changes, each naming the entry it replaces
//! ([`svrngs_core::SyncFolder`]); then the record's layout
//! ([`svrngs_core::Admission::layout`]) is written into the directory —
//! what it lacks fetched whole, verified, from any standing member, the
//! writer first, and what it no longer places deleted — never touching a
//! file changed on disk since it was last taken. A file is copied, not
//! referenced as a shared folder's is, since the store must keep a version
//! the directory has since overwritten: a conflict's other side. What was
//! last taken or written at each path, and from which entry, is kept in the
//! data dir, so an edit made while the node was stopped is sent with its
//! base.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use iroh_blobs::api::blobs::ImportMode;
use iroh_blobs::Hash;
use serde::{Deserialize, Serialize};
use svrngs_core::{Change, ContentHash, Placed, SyncFolder};

use super::{FolderError, Folders, Plant, PIECE};
use crate::register::Standing;

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
    /// Keep `dir` in step with the folder of its name, once a second while
    /// the node runs; its name.
    pub fn both_ways(&self, dir: PathBuf) -> String {
        let name = dir.file_name().unwrap_or_default().to_string_lossy();
        let (f, folder, name) = (self.clone(), name.to_string(), name.into_owned());
        tokio::spawn(async move {
            loop {
                if let Err(why) = f.step(&dir, &folder).await {
                    tracing::warn!(target: "folder", folder, %why, "synced: the step stopped short; the next one tries again");
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
        name
    }

    /// Every synced folder's files where the records held here place them.
    pub(super) fn placed(&self) -> Vec<Placed> {
        let mut placed: Vec<Placed> = self.admissions().flat_map(|a| a.layout()).collect();
        if self.plant == Plant::LaterWins {
            placed.retain(|p| !p.beside);
        }
        placed
    }

    async fn step(&self, dir: &Path, folder: &str) -> Result<(), FolderError> {
        let kept = self.data_dir.join(format!("synced-{folder}.json"));
        let read = std::fs::read(&kept).ok();
        let mut held: BTreeMap<String, Held> = read
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let mut changes = Vec::new();
        let mut taken = BTreeMap::new();
        if self.plant != Plant::Deaf {
            let mut seen = BTreeSet::new();
            for (rel, path, meta) in self.walk(dir)? {
                seen.insert(rel.clone());
                let before = held.get_mut(&rel);
                if unchanged(before.as_deref(), &path) {
                    continue;
                }
                let hash = self.take(&path, ImportMode::Copy).await?;
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
        }
        if !changes.is_empty() {
            let act = SyncFolder {
                folder: folder.into(),
                changes,
            };
            self.write(&act, folder, act.changes.len()).await?;
            for c in act.changes {
                tracing::info!(target: "folder", folder, path = c.path, now = ?c.now, "synced: a change sent");
                held.remove(&c.path);
            }
        }
        self.apply(dir, folder, &mut held, &taken).await;
        std::fs::write(&kept, serde_json::to_vec(&held).unwrap_or_default()).map_err(io(&kept))
    }

    /// The layout written into `dir`, what `held` says updated. A file just
    /// `taken` and sent is held as its entry, wherever the record places
    /// that entry: beside, when it met another edit, and the record's
    /// choice is then written at its path.
    async fn apply(
        &self,
        dir: &Path,
        folder: &str,
        held: &mut BTreeMap<String, Held>,
        taken: &BTreeMap<String, (ContentHash, SystemTime)>,
    ) {
        let placed: Vec<Placed> = self
            .placed()
            .into_iter()
            .filter(|p| p.folder == folder)
            .collect();
        let mut standing = BTreeSet::new();
        for ns in self.rail.namespaces().unwrap_or_default() {
            if let Ok(Standing::Members(m)) = self.rail.standing(&ns) {
                standing.extend(m.into_keys());
            }
        }
        let paths: BTreeSet<&String> = placed.iter().map(|p| &p.path).collect();
        for (rel, h) in held.clone() {
            let at = dir.join(&rel);
            if !paths.contains(&rel) && unchanged(Some(&h), &at) {
                let gone = std::fs::remove_file(&at);
                tracing::info!(target: "folder", folder, path = rel, ?gone, "synced: deleted, the record placing nothing there");
                held.remove(&rel);
            }
        }
        for (path, &(hash, mtime)) in taken {
            let mine =
                |q: &&Placed| q.entry.key == self.actor && &q.entry.path == path && q.hash == hash;
            if let Some(q) = placed.iter().find(mine) {
                let p = Placed {
                    path: path.clone(),
                    ..q.clone()
                };
                held.insert(path.clone(), Held { p, mtime });
            }
        }
        for p in placed {
            let (at, before) = (dir.join(&p.path), held.get(&p.path));
            if before.is_some_and(|h| h.p.entry == p.entry) || !unchanged(before, &at) {
                continue;
            }
            if before.is_none_or(|h| h.p.hash != p.hash) {
                let hash = Hash::from_bytes(*p.hash.as_bytes());
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
                    tracing::warn!(target: "folder", folder, path = p.path, %hash, ?why, "synced: not written — no member gave it, verified");
                    continue;
                }
                let wrote = self.store.blobs().export(hash, &at).finish().await;
                if let Err(why) = wrote {
                    tracing::warn!(target: "folder", folder, path = p.path, %hash, %why, "synced: not written into the directory");
                    continue;
                }
                tracing::info!(target: "folder", folder, path = p.path, %hash, from = p.entry.key, seq = p.entry.seq, beside = p.beside, "synced: written into the directory");
            }
            let Ok(mtime) = std::fs::metadata(&at).and_then(|m| m.modified()) else {
                continue;
            };
            held.insert(p.path.clone(), Held { p, mtime });
        }
    }
}
