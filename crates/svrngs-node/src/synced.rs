// SPDX-License-Identifier: AGPL-3.0-or-later
//! A directory kept in step with its folder in one group, synced both ways
//! (sy-1, the mesh-Dropbox spike; syp-4; DESIGN §9). Each time the OS says
//! files under it changed (perf-2 f), the group's record takes an act
//! (perf-2 d), at its start and each ten minutes, when it is walked
//! whole, and again, a second and doubling to fifteen, while a file the
//! layout places is not written (syp-6 a): each new or changed file
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
//! data dir, read at a loop's first step and written when it changes, so
//! an edit made while the node was stopped is sent with its base, found by
//! the walk at its start; a kept state that cannot be read stops its directory's steps, said
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
use n0_future::{BufferedStreamExt as _, StreamExt as _};
use notify::{RecursiveMode, Watcher as _};
use serde::{Deserialize, Serialize};
use svrngs_core::{Change, ContentHash, Placed, SyncFolder, SyncPath};
use tokio::sync::mpsc;

use super::{FolderAsk, FolderError, Folders};

/// The keys standing in the group a step syncs with.
type Holders = BTreeSet<String>;

/// Where the directories this computer syncs are kept.
const SYNCED: &str = "synced.json";

/// How many changed files a step takes into the store at once: one after
/// another, 100,000 took 98.4 s at load 70 (perf-1's README §9; perf-2 e).
const TAKE_AT_ONCE: usize = 8;

/// How many files a step fetches at once: one after another, a member
/// joining 10,000 files waited each behind the last (perf-2 g).
const FETCH_AT_ONCE: usize = 8;

/// How often a watched directory is walked whole, for what the OS's events
/// did not say (perf-2 f): beside its start and every time the OS says it
/// dropped some. A directory the OS cannot watch is walked each second, as
/// every directory was before.
const SWEEP_EVERY: Duration = Duration::from_secs(600);

/// How long a step waits after the OS's first event, so a burst of writes
/// is one step and one act.
const GATHER_FOR: Duration = Duration::from_millis(50);

/// How soon a step that left a file the layout places unwritten, no member
/// having given it, is woken again, doubling at each such step up to
/// [`RETRY_UP_TO`] (syp-6 a): with no OS event or act to wake it, the next
/// was the sweep, 600 s on, and syp-5's S3 conflict was never fetched
/// (`demo/runs/syp/2026-10-10/rerun/README.md` §6, item 4).
const RETRY_FIRST: Duration = Duration::from_secs(1);

/// The longest wait between two asks of a file not written: iroh drops a
/// dead process's path within its 15 s path idle timeout
/// (iroh-1.0.2/src/socket.rs:117; at 15.6-17.6 s in syp-5's
/// `run-s3-2/ana.log:443`, `:448`), so a member killed and started again is
/// dialled within one wait of that, where a longer one only delays a
/// holder iroh can already reach.
const RETRY_UP_TO: Duration = Duration::from_secs(15);

/// What a step looks at on disk.
enum Look {
    /// Every file under the directory: the sweep.
    Everything,
    /// The paths the OS said changed, each a file, a directory moved or
    /// written whole, or gone; none when the record woke the step.
    Paths(BTreeSet<PathBuf>),
}

/// Whether `rel` is `scope` or under it; every path is under `""`.
fn under(rel: &SyncPath, scope: &str) -> bool {
    let rel = rel.as_str();
    scope.is_empty()
        || rel
            .strip_prefix(scope)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// The directories this computer syncs, each by its group and folder, one
/// directory each, so one loop owns a directory and its kept state (#121);
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

/// `bytes` at `path` whole or not at all ([`crate::declared::write_whole`]):
/// a stop mid-write leaves the last whole one, never a torn file the next
/// start would refuse.
fn write_whole(path: &Path, bytes: &[u8]) -> Result<(), FolderError> {
    crate::declared::write_whole(path, bytes).map_err(|(at, e)| FolderError::Read(at, e))
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
        let standing = self.host.rail.standing(&group);
        let standing = standing.map_err(|e| FolderError::Unread(group.clone(), e.to_string()))?;
        if !standing.names(&self.host.actor) {
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

    /// `dir` stepped when the OS says a file under it changed, at once when
    /// its group's record takes an act, after [`RETRY_FIRST`] and longer
    /// while a file the layout places is not written, and walked whole at
    /// its start and each [`SWEEP_EVERY`]: its one loop, holding its kept
    /// state. A step
    /// each second walked every file, 1,952 ms of each at 100,000 files,
    /// and its wait was 958 ms of a change's 1,182 at another member
    /// (perf-1's README §9 T1, T5; perf-2 d, f).
    #[expect(
        clippy::disallowed_methods,
        reason = "the loop is an adapter: it reads the clock for when the next sweep is due"
    )]
    fn keep_in_step(&self, dir: PathBuf, group: String, folder: String) {
        tracing::info!(target: "folder", namespace = group, folder, dir = %dir.display(), "synced: the directory kept in step with the group's folder");
        let f = self.clone();
        let mut woken = self.woken.subscribe();
        tokio::spawn(async move {
            let count = |m: &BTreeMap<String, u64>| m.get(&group).copied().unwrap_or(0);
            // `tx` lives with the loop, so a watcher that could not start
            // leaves the channel open and the sweep its only waker.
            let (tx, mut events) = mpsc::unbounded_channel();
            let told = tx.clone();
            let watching = notify::recommended_watcher(move |e| drop(told.send(e)))
                .and_then(|mut w| w.watch(&dir, RecursiveMode::Recursive).map(|()| w));
            let every = match &watching {
                Ok(_) => {
                    tracing::debug!(target: "folder", namespace = group, folder, "synced: the OS's file events watched, the directory walked whole each {SWEEP_EVERY:?}");
                    SWEEP_EVERY
                }
                Err(why) => {
                    tracing::warn!(target: "folder", namespace = group, folder, %why, "synced: the OS's file events cannot be watched here, so the directory is walked each second");
                    Duration::from_secs(1)
                }
            };
            let (mut held, mut look, mut due) = (None, Look::Everything, Instant::now());
            // When the step is woken again for a file not written; none
            // while every file is, so an idle directory wakes nothing.
            let mut retry: Option<Duration> = None;
            loop {
                if matches!(look, Look::Everything) {
                    due = Instant::now() + every;
                }
                // Read before the step, so an act taken during it wakes the next.
                let seen = count(&woken.borrow_and_update());
                match f.step(&dir, &group, &folder, &mut held, &look).await {
                    Ok(0) => retry = None,
                    Ok(unwritten) => {
                        let next = retry.map_or(RETRY_FIRST, |d| (d * 2).min(RETRY_UP_TO));
                        retry = Some(next);
                        tracing::debug!(target: "folder", namespace = group, folder, unwritten, again_in = ?next, "synced: files the layout places here not written, so the step is woken again");
                    }
                    Err(why) => {
                        // Read again from the last whole kept state, as each
                        // step read it before: what this one changed is not kept.
                        held = None;
                        tracing::warn!(target: "folder", namespace = group, folder, %why, "synced: the step stopped short; the next one tries again, its kept state read again");
                    }
                }
                look = tokio::select! {
                    first = events.recv() => {
                        tokio::time::sleep(GATHER_FOR).await;
                        let mut said = vec![first];
                        while let Ok(e) = events.try_recv() {
                            said.push(Some(e));
                        }
                        let mut paths = BTreeSet::new();
                        let mut whole = false;
                        for e in said.into_iter().flatten() {
                            match e {
                                Ok(e) if !e.need_rescan() => paths.extend(e.paths),
                                _ => whole = true,
                            }
                        }
                        if whole {
                            tracing::debug!(target: "folder", namespace = group, folder, "synced: the OS says it dropped events, so the directory is walked whole");
                            Look::Everything
                        } else {
                            tracing::debug!(target: "folder", namespace = group, folder, paths = paths.len(), "synced: the step woken, the OS saying files changed");
                            Look::Paths(paths)
                        }
                    }
                    // The guard `wait_for` answers is dropped inside, never
                    // held across the other branch's settle.
                    () = async { drop(woken.wait_for(|m| count(m) != seen).await) } => {
                        tracing::debug!(target: "folder", namespace = group, folder, "synced: the step woken, the group's record having taken an act");
                        Look::Paths(BTreeSet::new())
                    }
                    () = async {
                        match retry {
                            Some(d) => tokio::time::sleep(d).await,
                            None => std::future::pending().await,
                        }
                    } => {
                        tracing::debug!(target: "folder", namespace = group, folder, "synced: the step woken again, a file the layout places here not written");
                        Look::Paths(BTreeSet::new())
                    }
                    _ = tokio::time::sleep_until(due.into()) => Look::Everything,
                };
            }
        });
    }

    /// Who stands in `group`, as [`crate::node::Host::standings`] reads it;
    /// a group it leaves out is one whose register cannot be read here.
    pub(super) fn holders(&self, group: &str) -> Result<Holders, FolderError> {
        let groups = self.host.standings().unwrap_or_default();
        let found = groups.into_iter().find(|(ns, _)| ns == group);
        let unread = || FolderError::Unread(group.into(), "who stands in it cannot be read".into());
        Ok(found.ok_or_else(unread)?.1.into_keys().collect())
    }

    /// Each synced directory's kept state on this computer, by its path.
    fn kept_files(&self) -> Result<Vec<PathBuf>, FolderError> {
        let unread = |e| FolderError::Read(self.data_dir.clone(), e);
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&self.data_dir).map_err(unread)? {
            let path = entry.map_err(unread)?.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with("synced-") && name.ends_with(".json") {
                out.push(path);
            }
        }
        Ok(out)
    }

    /// Each kept state's path, size and mtime: whether one moved.
    pub(super) fn kept_at(&self) -> Result<Vec<(PathBuf, u64, SystemTime)>, FolderError> {
        let stat = |path: PathBuf| {
            let meta = std::fs::metadata(&path).map_err(io(&path))?;
            let mtime = meta.modified().map_err(io(&path))?;
            Ok((path, meta.len(), mtime))
        };
        self.kept_files()?.into_iter().map(stat).collect()
    }

    /// Every version a synced directory on this computer holds, as each
    /// was last taken or written: what the store keeps beside the layouts.
    pub(super) fn held_everywhere(&self) -> Result<Vec<ContentHash>, FolderError> {
        let mut out = Vec::new();
        for path in self.kept_files()? {
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
    async fn step(
        &self,
        dir: &Path,
        group: &str,
        folder: &str,
        held: &mut Option<BTreeMap<SyncPath, Held>>,
        look: &Look,
    ) -> Result<usize, FolderError> {
        let t0 = Instant::now();
        let ms = |t: Instant| t.elapsed().as_millis() as u64;
        // Read before anything is sent or deleted, so a group that cannot
        // be read stops the step by its name.
        let (holders, placed) = (self.holders(group)?, self.layout(group)?);
        tracing::debug!(target: "folder", folder, placed = placed.placed.len(), holders = holders.len(), ms = ms(t0), "synced: the layout read");
        let kept = self.data_dir.join(format!("synced-{group}-{folder}.json"));
        // Read at the loop's first step, then held here and written when it
        // changes (perf-2 f). Nothing kept is nothing held; a kept state that
        // cannot be read is never read as that, or every file would go again
        // with no base, so each step asks again until it can be read.
        let held = match held {
            Some(held) => held,
            None => held.insert(match std::fs::read(&kept) {
                Ok(bytes) => serde_json::from_slice(&bytes)
                    .map_err(|e| FolderError::Kept(kept.clone(), e.to_string()))?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
                Err(e) => return Err(FolderError::Kept(kept.clone(), e.to_string())),
            }),
        };
        let was = held.clone();
        let mut changes = Vec::new();
        let mut taken = BTreeMap::new();
        let (mut seen, mut found) = (BTreeSet::new(), Vec::new());
        // What this step took or fetched, held from the store's sweep until
        // the kept state names it.
        let mut hold: Vec<TempTag> = Vec::new();
        let (read_ms, t) = (ms(t0), Instant::now());
        // What is looked at: every file, or what the OS named; and under
        // which paths a file held but not found is a delete.
        let (walked, scopes) = match look {
            Look::Everything => (self.walk(dir, "")?, vec![String::new()]),
            Look::Paths(paths) => self.look_at(dir, paths)?,
        };
        let (walk_ms, t) = (ms(t), Instant::now());
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
            if !seen.insert(rel.clone()) || unchanged(held.get(&rel), &path) {
                continue;
            }
            let was = held.get(&rel).map(|h| (h.p.size, h.mtime));
            tracing::debug!(target: "folder", folder, path = %rel, size = meta.len(), mtime = ?meta.modified().ok(), ?was, "synced: found on disk, changed since it was last held");
            found.push((rel, path, meta));
        }
        // Taken several at once, each answered in the walk's order, never
        // the order the imports finished: the act names them so (perf-2 e).
        let took = n0_future::stream::iter(found)
            .map(|(rel, path, meta)| async move {
                let took = Instant::now();
                let tag = self.take(&path, &meta).await;
                (rel, path, meta, tag, ms(took))
            })
            .buffered_ordered(TAKE_AT_ONCE)
            .collect::<Vec<_>>()
            .await;
        for (rel, path, meta, tag, took_ms) in took {
            // Held as it was, so the next step, which the write wakes, takes it.
            let Some(tag) = tag? else {
                tracing::debug!(target: "folder", folder, path = %rel, ms = took_ms, "synced: not taken — written as it was taken; the next step takes it");
                continue;
            };
            let hash = tag.hash();
            hold.push(tag);
            tracing::debug!(target: "folder", folder, path = %rel, %hash, ms = took_ms, "synced: taken into the store");
            let hash = ContentHash::from_bytes(hash.into());
            let mtime = meta.modified().map_err(io(&path))?;
            match held.get_mut(&rel) {
                Some(h) if h.p.hash == hash => {
                    tracing::debug!(target: "folder", folder, path = %rel, "synced: not sent — the bytes held, touched; its new mtime held");
                    h.mtime = mtime;
                }
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
                .filter(|(rel, _)| !seen.contains(*rel) && scopes.iter().any(|s| under(rel, s)))
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
        let placed: Vec<&Placed> = placed
            .placed
            .iter()
            .filter(|p| p.folder == folder)
            .collect();
        let (fetched, unwritten) = self
            .apply(dir, folder, &placed, &holders, held, &taken)
            .await;
        hold.extend(fetched);
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
        let wrote = match *held == was {
            true => Ok(()),
            false => write_whole(&kept, &serde_json::to_vec(&held).unwrap_or_default()),
        };
        drop(hold);
        // Each file looked at once, though the OS named it and its directory.
        let files = seen.len();
        tracing::debug!(target: "folder", folder, files, sent, unwritten, read_ms, walk_ms, take_ms, write_ms, apply_ms, kept_ms = ms(t), ms = ms(t0), "synced: a step");
        wrote.map(|()| unwritten)
    }

    /// The layout written into `dir`, what `held` says updated. A file just
    /// `taken` and sent is held as its entry, wherever the record places
    /// that entry: beside, when it met another edit, and the record's
    /// choice is then written at its path. What it fetched, held from the
    /// store's sweep for the step to drop once the kept state names it, and
    /// how many files it placed here it could not write.
    async fn apply(
        &self,
        dir: &Path,
        folder: &str,
        placed: &[&Placed],
        standing: &Holders,
        held: &mut BTreeMap<SyncPath, Held>,
        taken: &BTreeMap<SyncPath, (ContentHash, SystemTime)>,
    ) -> (Vec<TempTag>, usize) {
        let mut hold = Vec::new();
        tracing::debug!(target: "folder", folder, placed = placed.len(), "synced: the layout to write");
        let paths: BTreeSet<&SyncPath> = placed.iter().map(|p| &p.path).collect();
        for (rel, h) in held.clone() {
            let at = dir.join(rel.as_str());
            if paths.contains(&rel) {
                continue;
            }
            if unchanged(Some(&h), &at) {
                let gone = std::fs::remove_file(&at);
                tracing::info!(target: "folder", folder, path = %rel, ?gone, "synced: deleted, the record placing nothing there");
                held.remove(&rel);
            } else {
                tracing::debug!(target: "folder", folder, path = %rel, "synced: not deleted, the record placing nothing there — the disk changed since it was last held");
            }
        }
        // This computer's entries by path and hash, each file taken found by
        // key: a scan of every entry for each was 95.3 s of 100,000 files'
        // first step (perf-1's README §9; perf-2 b).
        let mut mine: BTreeMap<(&SyncPath, ContentHash), &Placed> = BTreeMap::new();
        for &q in placed.iter().filter(|q| q.entry.key == self.host.actor) {
            mine.entry((&q.entry.path, q.hash)).or_insert(q);
        }
        for (path, &(hash, mtime)) in taken {
            if let Some(q) = mine.get(&(path, hash)) {
                let p = Placed {
                    path: path.clone(),
                    ..(*q).clone()
                };
                held.insert(path.clone(), Held { p, mtime });
            }
        }
        let mut fetches = Vec::new();
        for &p in placed {
            let (at, before) = (dir.join(p.path.as_str()), held.get(&p.path));
            if before.is_some_and(|h| h.p.entry == p.entry) {
                continue;
            }
            if !unchanged(before, &at) {
                tracing::debug!(target: "folder", folder, path = %p.path, "synced: not written — the disk changed since it was last held, and is never written over");
                continue;
            }
            if before.is_none_or(|h| h.p.hash != p.hash) {
                fetches.push((p.clone(), at));
                continue;
            }
            match std::fs::metadata(&at).and_then(|m| m.modified()) {
                Ok(mtime) => {
                    let p = p.clone();
                    held.insert(p.path.clone(), Held { p, mtime });
                }
                Err(why) => {
                    tracing::warn!(target: "folder", folder, path = %p.path, %why, "synced: the bytes held, now another entry's, but its mtime cannot be read, so it is held as before");
                }
            }
        }
        // Fetched FETCH_AT_ONCE at once, each written when it is held
        // (perf-2 g).
        let fetched = n0_future::stream::iter(fetches)
            .map(|(p, at)| async move {
                let tag = self.fetch_placed(folder, &p, &at, standing).await;
                (p, at, tag)
            })
            .buffered_unordered(FETCH_AT_ONCE)
            .collect::<Vec<_>>()
            .await;
        let mut unwritten = 0;
        for (p, at, tag) in fetched {
            let Some(tag) = tag else {
                unwritten += 1;
                continue;
            };
            hold.push(tag);
            match std::fs::metadata(&at).and_then(|m| m.modified()) {
                Ok(mtime) => drop(held.insert(p.path.clone(), Held { p, mtime })),
                Err(why) => {
                    tracing::warn!(target: "folder", folder, path = %p.path, %why, "synced: written, but its mtime cannot be read, so it is not held");
                    unwritten += 1;
                }
            }
        }
        (hold, unwritten)
    }

    /// `p`'s version fetched from a member standing in the group, its
    /// writer first, and written at `at`; held from the store's sweep by
    /// the tag it answers with, or `None`, said, when none gave it or it
    /// could not be written.
    #[expect(
        clippy::disallowed_methods,
        reason = "times each fetch and each export for their traces (perf-1); no decision reads it"
    )]
    async fn fetch_placed(
        &self,
        folder: &str,
        p: &Placed,
        at: &Path,
        standing: &Holders,
    ) -> Option<TempTag> {
        let ms = |t: Instant| t.elapsed().as_millis() as u64;
        let hash = Hash::from_bytes(*p.hash.as_bytes());
        tracing::debug!(target: "folder", folder, path = %p.path, %hash, size = p.size, from = p.entry.key, seq = p.entry.seq, "synced: to fetch, the layout placing it here");
        let tag = self.store.tags().temp_tag(hash).await;
        let tag = tag.inspect_err(|why| tracing::warn!(target: "folder", folder, path = %p.path, %hash, %why, "synced: not fetched — the store cannot hold it from its sweep")).ok()?;
        let fetched = Instant::now();
        let mut from = vec![p.entry.key.clone()];
        from.extend(standing.iter().filter(|k| **k != p.entry.key).cloned());
        if let Err(why) = self.hold(hash, &from).await {
            tracing::warn!(target: "folder", folder, path = %p.path, %hash, ?why, "synced: not written — no member gave it, verified");
            return None;
        }
        let (fetch_ms, exported) = (ms(fetched), Instant::now());
        let wrote = self.store.blobs().export(hash, at).finish().await;
        if let Err(why) = wrote {
            tracing::warn!(target: "folder", folder, path = %p.path, %hash, %why, "synced: not written into the directory");
            return Some(tag);
        }
        tracing::info!(target: "folder", folder, path = %p.path, %hash, from = p.entry.key, seq = p.entry.seq, beside = p.beside, fetch_ms, export_ms = ms(exported), "synced: written into the directory");
        Some(tag)
    }
}
