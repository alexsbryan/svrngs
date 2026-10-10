// SPDX-License-Identifier: AGPL-3.0-or-later
//! A shared folder (gf-1, group folders' second spike; DESIGN §9): a key's
//! folder, by name, and the files it holds, each by its path under the
//! folder, its size and the BLAKE3 hash iroh-blobs names its bytes by. The
//! newest list a key wrote for a name is the folder; an empty list is its
//! stop. The record names bytes and never what they are.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::Infallible;

use serde::{Deserialize, Serialize};

use crate::command::{Command, Reads};
use crate::{Admission, ContentHash, Person};

/// One file of a shared folder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SharedFile {
    /// Where it sits under the folder: a name to show, never a path to open.
    pub path: String,
    pub size: u64,
    /// The BLAKE3 hash of its bytes, which is the name iroh-blobs gives a
    /// raw blob.
    pub hash: ContentHash,
}

/// A folder as the record stands: whose, its name, its files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SharedFolder {
    /// The key that shared it, as the record spells an actor.
    pub key: String,
    pub person: Person,
    pub name: String,
    pub files: Vec<SharedFile>,
}

/// This computer's folder `name` now holds `files`; none stops it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareFolder {
    pub name: String,
    pub files: Vec<SharedFile>,
}

impl Command for ShareFolder {
    type Refusal = Infallible;
    const READS: Reads = Reads::Membership;

    fn decide(&self, _before: &Admission) -> Result<Vec<crate::RailAct>, Infallible> {
        Ok(vec![crate::RailAct::Folder {
            name: self.name.clone(),
            files: self.files.clone(),
        }])
    }
}

/// Each key's newest list per folder name, by its own seq, from the acts in
/// admission's order: what [`Admission::folders`] reads.
pub(crate) fn newest<'a>(
    acts: impl Iterator<Item = (&'a str, &'a Person, u64, &'a crate::RailAct)>,
) -> Vec<(u64, SharedFolder)> {
    let mut out: Vec<(u64, SharedFolder)> = Vec::new();
    for (key, person, seq, act) in acts {
        let crate::RailAct::Folder { name, files } = act else {
            continue;
        };
        let folder = SharedFolder {
            key: key.into(),
            person: person.clone(),
            name: name.clone(),
            files: files.clone(),
        };
        match out
            .iter_mut()
            .find(|(_, f)| f.key == key && &f.name == name)
        {
            Some(held) if held.0 < seq => *held = (seq, folder),
            Some(_) => {}
            None => out.push((seq, folder)),
        }
    }
    out
}

impl Admission {
    /// Every folder shared by a key standing in the group, as its newest
    /// list says; a stopped one is absent.
    pub fn folders(&self) -> Vec<SharedFolder> {
        let standing = &self.membership().standing;
        let shared: Vec<SharedFolder> = self
            .folders
            .iter()
            .filter(|(_, f)| !f.files.is_empty() && standing.contains(&f.key))
            .map(|(_, f)| f.clone())
            .collect();
        tracing::debug!(target: "rail:folder", written = self.folders.len(), shared = shared.len(),
            "folders: each key's newest list, its stops and keys no longer standing left out");
        shared
    }

    /// The keys whose standing folders name `hash`, in the record's order.
    pub fn holders(&self, hash: &ContentHash) -> Vec<String> {
        let mut keys: Vec<String> = Vec::new();
        for f in self.folders() {
            if f.files.iter().any(|x| &x.hash == hash) && !keys.contains(&f.key) {
                keys.push(f.key);
            }
        }
        keys
    }
}

#[cfg(test)]
#[path = "folder_tests.rs"]
mod tests;
