// SPDX-License-Identifier: AGPL-3.0-or-later
//! A folder synced both ways (sy-1, the mesh-Dropbox spike; DESIGN §9).
//! Each change any member makes to a file names its path, its bytes by hash
//! and size (none for a delete) and its base, the entry it replaces as its
//! writer last held the path. An entry no other names as its base is a
//! head; [`Admission::layout`] places each path's live heads, the first by
//! key under its name and each other beside it named as a conflict, so no
//! edit is lost and every member places the same files. The record names
//! bytes and never what they are. A path is a [`SyncPath`], parsed on the
//! write that makes it and again when any copy reads the act (syp-4 c).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::Infallible;

use serde::{Deserialize, Serialize};

use crate::command::{Command, Reads};
use crate::{Admission, ContentHash, Person};

/// A path under a synced folder (syp-4 c): named parts joined by `/`, so
/// never empty, no root and no doubled `/` (an empty part), no `.` or `..`
/// (a part that climbs), no `\` or NUL (a separator or an end another
/// system reads), and no drive's prefix (`C:`) first. Made only by
/// [`Self::parse`]; deserialising one runs it, so an act naming
/// `../outside` is a [`crate::RailGap::MalformedLine`] on every copy, never
/// placed, and a member's disk is written only under the folder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SyncPath(String);

/// Why some text is not a [`SyncPath`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotASyncPath {
    #[error("a path under a folder cannot be empty")]
    Empty,
    #[error("`{0}` is not under the folder: it has an empty part, a root or a doubled `/`")]
    EmptyPart(String),
    #[error("`{0}` is not under the folder: it has a `.` or `..` part")]
    Climbs(String),
    #[error("`{0}` is not under the folder: it holds a `\\` or a NUL")]
    Separator(String),
    #[error("`{0}` is not under the folder: it starts with a drive")]
    Prefix(String),
}

impl SyncPath {
    /// A path under a folder from `raw`, or why it is not one.
    pub fn parse(raw: &str) -> Result<Self, NotASyncPath> {
        let parts: Vec<&str> = raw.split('/').collect();
        let drive = |p: &str| matches!(p.as_bytes(), [d, b':', ..] if d.is_ascii_alphabetic());
        let why = match () {
            _ if raw.is_empty() => NotASyncPath::Empty,
            _ if parts.contains(&"") => NotASyncPath::EmptyPart(raw.into()),
            _ if parts.iter().any(|p| matches!(*p, "." | "..")) => NotASyncPath::Climbs(raw.into()),
            _ if raw.contains(['\\', '\0']) => NotASyncPath::Separator(raw.into()),
            _ if drive(parts[0]) => NotASyncPath::Prefix(raw.into()),
            _ => return Ok(Self(raw.into())),
        };
        tracing::debug!(target: "rail:folder", raw, %why, "synced: not a path under the folder");
        Err(why)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SyncPath {
    type Error = NotASyncPath;
    fn try_from(raw: String) -> Result<Self, NotASyncPath> {
        Self::parse(&raw)
    }
}

impl From<SyncPath> for String {
    fn from(path: SyncPath) -> String {
        path.0
    }
}

impl core::fmt::Display for SyncPath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One change in the record: whose, which act, which path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Entry {
    pub key: String,
    pub seq: u64,
    pub path: SyncPath,
}

/// One file's change, as its writer saw it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub path: SyncPath,
    /// Its bytes now, by hash and size; none is its delete.
    pub now: Option<(ContentHash, u64)>,
    /// The entry it replaces; none for a path its writer held nothing at.
    pub base: Option<Entry>,
}

/// A file where the record places it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placed {
    pub folder: String,
    /// Where it sits under the folder: its entry's path, or beside it.
    pub path: SyncPath,
    pub hash: ContentHash,
    pub size: u64,
    pub entry: Entry,
    /// Beside its entry's path, named as a conflict.
    pub beside: bool,
}

/// This computer's changes to its folder `folder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncFolder {
    pub folder: String,
    pub changes: Vec<Change>,
}

impl Command for SyncFolder {
    type Refusal = Infallible;
    const READS: Reads = Reads::Membership;

    fn decide(&self, _before: &Admission) -> Result<Vec<crate::RailAct>, Infallible> {
        Ok(vec![crate::RailAct::Synced {
            folder: self.folder.clone(),
            changes: self.changes.clone(),
        }])
    }
}

/// An app's files by path, as one version of it holds them.
pub type Files = BTreeMap<SyncPath, ContentHash>;

/// What runs of each app a group carries (canon-pin; DESIGN §3), by its
/// kind and, with `None`, as it was brought: each path under `apps/<kind>/`
/// as the record first placed it, so a member's later change to a file
/// runs as nobody it was not brought as.
pub type Runs = BTreeMap<(String, Option<String>), Files>;

/// Every change, from the acts in admission's order: each entry another
/// names as its base, each file written, unplaced, with its writer's
/// person, and what runs of each app. What [`Admission::layout`] and
/// [`Admission::runs`] read.
pub(crate) type Written = (BTreeSet<Entry>, Vec<(Placed, Person)>, Runs);

pub(crate) fn written<'a>(
    acts: impl Iterator<Item = (&'a str, &'a Person, u64, &'a crate::RailAct)>,
) -> Written {
    let mut out = Written::default();
    for (key, person, seq, act) in acts {
        let crate::RailAct::Synced { folder, changes } = act else {
            continue;
        };
        for c in changes {
            out.0.extend(c.base.clone());
            let Some((hash, size)) = c.now else {
                continue;
            };
            let (path, key) = (c.path.clone(), key.into());
            if let ["apps", kind, _, ..] = path.as_str().split('/').collect::<Vec<_>>()[..] {
                let brought = out.2.entry((kind.into(), None)).or_default();
                brought.entry(path.clone()).or_insert(hash);
            }
            let entry = Entry {
                key,
                seq,
                path: path.clone(),
            };
            let (folder, beside) = (folder.clone(), false);
            let placed = Placed {
                folder,
                path,
                hash,
                size,
                entry,
                beside,
            };
            out.1.push((placed, person.clone()));
        }
    }
    out
}

/// `dir/notes.txt` written by Ben, beside its path: `dir/notes (conflict
/// Ben).txt`.
/// A `/` or `\\` in the person's name is written `∕`, so it makes no part:
/// a person cannot name a directory on another member's disk.
fn beside(path: &SyncPath, person: &Person) -> SyncPath {
    let person = person.as_str().replace(['/', '\\'], "∕");
    let (dir, file) = path.0.rsplit_once('/').unwrap_or(("", &path.0));
    let named = match file.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem} (conflict {person}).{ext}"),
        _ => format!("{file} (conflict {person})"),
    };
    SyncPath(match dir {
        "" => named,
        dir => format!("{dir}/{named}"),
    })
}

impl Admission {
    /// What runs of each app the group carries ([`Runs`]).
    pub fn runs(&self) -> &Runs {
        &self.synced.2
    }

    /// Every synced folder's files where the record places them: each
    /// path's live heads, the first by entry under its path and each other
    /// beside it; a path whose heads are all deletes is absent.
    pub fn layout(&self) -> Vec<Placed> {
        let (replaced, written, _) = &self.synced;
        let mut heads: Vec<&(Placed, Person)> = written
            .iter()
            .filter(|(p, _)| !replaced.contains(&p.entry))
            .collect();
        heads.sort_by_key(|(p, _)| (&p.folder, &p.entry.path, &p.entry));
        let mut out: Vec<Placed> = Vec::new();
        for (p, person) in heads {
            let mut p = p.clone();
            p.beside = out
                .last()
                .is_some_and(|l| l.folder == p.folder && l.entry.path == p.entry.path);
            if p.beside {
                p.path = beside(&p.path, person);
            }
            out.push(p);
        }
        tracing::debug!(target: "rail:folder", written = written.len(), replaced = replaced.len(), placed = out.len(),
            conflicts = out.iter().filter(|p| p.beside).count(), "synced: each path's heads placed, a second beside as a conflict");
        out
    }
}

#[cfg(test)]
#[path = "synced_tests.rs"]
mod tests;
