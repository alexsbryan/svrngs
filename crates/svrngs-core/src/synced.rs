// SPDX-License-Identifier: AGPL-3.0-or-later
//! A folder synced both ways (sy-1, the mesh-Dropbox spike; DESIGN §9).
//! Each change any member makes to a file names its path, its bytes by hash
//! and size (none for a delete) and its base, the entry it replaces as its
//! writer last held the path. An entry no other names as its base is a
//! head; [`Admission::layout`] places each path's live heads, the first by
//! key under its name and each other beside it named as a conflict, so no
//! edit is lost and every member places the same files. The record names
//! bytes and never what they are.

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::Infallible;

use serde::{Deserialize, Serialize};

use crate::command::{Command, Reads};
use crate::{Admission, ContentHash, Person};

/// One change in the record: whose, which act, which path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Entry {
    pub key: String,
    pub seq: u64,
    pub path: String,
}

/// One file's change, as its writer saw it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub path: String,
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
    pub path: String,
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

/// Every change, from the acts in admission's order: each entry another
/// names as its base, and each file written, unplaced, with its writer's
/// person. What [`Admission::layout`] reads.
pub(crate) type Written = (BTreeSet<Entry>, Vec<(Placed, Person)>);

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
fn beside(path: &str, person: &Person) -> String {
    let (dir, file) = path.rsplit_once('/').unwrap_or(("", path));
    let named = match file.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem} (conflict {person}).{ext}"),
        _ => format!("{file} (conflict {person})"),
    };
    match dir {
        "" => named,
        dir => format!("{dir}/{named}"),
    }
}

impl Admission {
    /// Every synced folder's files where the record places them: each
    /// path's live heads, the first by entry under its path and each other
    /// beside it; a path whose heads are all deletes is absent.
    pub fn layout(&self) -> Vec<Placed> {
        let (replaced, written) = &self.synced;
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
