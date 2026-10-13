// SPDX-License-Identifier: AGPL-3.0-or-later
//! A group's seed kept on this computer (DESIGN §8, U2's inventory, the
//! `valid_namespace` row). The seed, its id — the hash that names the
//! group's directory — and the belief in a seed an issuer answers are
//! svrngs-core's since u4-5 (`svrngs_core::seed`), so a browser's page
//! reads a seed with the command's body; drawing a seed's salt and keeping
//! it on disk are this module's. Moved from the command (svrngs's group.rs
//! and cli.rs `keep_group`, now [`crate::node::Host::keep_group`]) into the
//! node, behaviour kept, so a running svrngs makes a group for the home page
//! as `svrngs new` does (start-group).
//!
//! The seed is written beside the group's record as `seed.json`, and the
//! register's seed, `roster.json`, is written from it by [`Seed::roster`]
//! alone: the founder's key bound to the founder's name (DESIGN §6). The
//! register reads `roster.json` as it did in U1; this module reads
//! `seed.json` to find a group by its name.
//!
//! Beside them, `origin` keeps the page this computer prints the group's
//! https links on (u2-7): `new --origin` writes it, `join` the one its
//! https link was printed on, and `link --origin` changes it. It is this
//! computer's setting, outside the seed and its id; a group that keeps
//! none prints the client's link alone (u2-8), so nothing depends on a
//! hosted page unless the group names one.
//!
//! And `relay`, where this computer meets the group (relay-default): `new
//! --relay` writes it, `join` the one its link carried. It is this
//! computer's choice, as the origin is; the group's record says where each
//! computer is reached, by its `Relay` act (relay-in-record). A group that
//! keeps none is met where the build meets its groups (svrngs's `cli::SHIPPED`):
//! n0's relays until the operator's switch, then svrngs's own.

use std::path::{Path, PathBuf};

use crate::namespaces_in;
/// The seed, its id and the belief in an issuer's answer are svrngs-core's
/// since u4-5, so a browser's page reads a seed with this body.
pub use crate::seed::{answered, Seed};
/// A group's name and a person's share the core's one rule (u2-9).
pub use crate::Name;
use ed25519_dalek::VerifyingKey;

use crate::iroh::RelayUrl;
use crate::link::Origin;
use crate::relay::n0_relays;

/// The one spelling of the seed's file name in a group's directory.
const SEED_FILE: &str = "seed.json";

// The one spelling of the kept origin's file name in a group's directory:
// the node's, which reads it for the home page too.
use crate::invite::{carried, kept_page, ORIGIN_FILE};

/// The one spelling of the kept relay's file name in a group's directory.
const RELAY_FILE: &str = "relay";

/// A new group's seed: `founder` names the person holding `key`, and 16
/// bytes are drawn so that a second group of the same name by the same
/// person is a second group.
#[expect(clippy::disallowed_methods, reason = "draws a group's salt")]
pub fn draw(name: Name, founder: Name, key: VerifyingKey) -> Seed {
    let mut salt = [0u8; 16];
    // As identity.rs: getrandom failing is a host with no entropy, and
    // no group can be made on it.
    getrandom::fill(&mut salt).expect("failed to draw a group's salt");
    Seed::from_parts(name, founder, key, salt)
}

/// Why a group's seed or origin could not be written or read.
#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot read {path}: {why}")]
    Read { path: PathBuf, why: String },
}

/// Write `seed` into `dir`, the group's directory, with the register's
/// seed beside it.
pub fn write_seed(dir: &Path, seed: &Seed) -> Result<(), SeedError> {
    keep(dir, dir.join(SEED_FILE), &seed.to_text())
}

/// Read the seed in `dir`, or `None` when the directory holds none.
pub fn read_seed(dir: &Path) -> Result<Option<Seed>, SeedError> {
    let path = dir.join(SEED_FILE);
    let Some(raw) = kept(&path)? else {
        return Ok(None);
    };
    Seed::from_text(&raw)
        .map(Some)
        .map_err(|why| SeedError::Read { path, why })
}

/// The seed in `dir` as written, or `None` when the directory holds none:
/// what the issuer's node answers a link's ask with
/// (`crate::redeem::SeedText`).
pub fn seed_text(dir: &Path) -> Result<Option<String>, String> {
    kept(&dir.join(SEED_FILE)).map_err(|e| e.to_string())
}

/// Keep `origin` in `dir`, the group's directory, as the page its links
/// print on from here.
pub fn write_origin(dir: &Path, origin: &Origin) -> Result<(), SeedError> {
    let path = dir.join(ORIGIN_FILE);
    tracing::debug!(target: "group", path = %path.display(), origin = origin.as_str(), "group: origin kept");
    keep(dir, path, origin.as_str())
}

/// The origin kept in `dir`, or `None` when the directory keeps none, as
/// the node reads it for the home page too ([`crate::invite::kept_page`]).
pub fn read_origin(dir: &Path) -> Result<Option<Origin>, SeedError> {
    crate::invite::kept_page(dir).map_err(|why| SeedError::Read {
        path: dir.join(ORIGIN_FILE),
        why,
    })
}

/// Keep `relay` in `dir`, the group's directory, as where this computer
/// meets the group.
pub fn write_relay(dir: &Path, relay: &RelayUrl) -> Result<(), SeedError> {
    let path = dir.join(RELAY_FILE);
    tracing::debug!(target: "group", path = %path.display(), %relay, "group: relay kept");
    keep(dir, path, &relay.to_string())
}

/// The relay kept in `dir`, read dotless, or `None` when it keeps none.
pub fn read_relay(dir: &Path) -> Result<Option<RelayUrl>, SeedError> {
    let path = dir.join(RELAY_FILE);
    let Some(raw) = kept(&path)? else {
        return Ok(None);
    };
    crate::relay::parse(&raw)
        .map(Some)
        .map_err(|e| SeedError::Read {
            path,
            why: e.to_string(),
        })
}

/// Write `contents` to `path`, a file in `dir`, the group's directory,
/// making the directory first: how the seed and the origin are kept.
fn keep(dir: &Path, path: PathBuf, contents: &str) -> Result<(), SeedError> {
    std::fs::create_dir_all(dir)
        .and_then(|()| std::fs::write(&path, contents))
        .map_err(|source| SeedError::Write { path, source })
}

/// What `path` holds, or `None` when there is no file there: how the seed
/// and the origin are read back. Any other failure is a [`SeedError::Read`]
/// naming the file, never an absence (ARCH 6).
fn kept(path: &Path) -> Result<Option<String>, SeedError> {
    match std::fs::read_to_string(path) {
        Ok(raw) => Ok(Some(raw)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(SeedError::Read {
            path: path.to_path_buf(),
            why: e.to_string(),
        }),
    }
}

impl crate::node::Host {
    /// Keep `seed`'s group on this computer: its journal, the seed beside it,
    /// `origin` and `relay` when given — the relay `new --relay` names or a
    /// joined link carried, where the group is met (svrngs's `cli::reach`) —
    /// and the roster the seed founds. What `new` and `join` both write, in
    /// one order, so a group kept by either is the same.
    pub fn keep_group(
        &self,
        seed: &Seed,
        origin: Option<&Origin>,
        relay: Option<&RelayUrl>,
    ) -> Result<(), String> {
        let rail = &self.rail;
        let dir = rail.group_dir(&seed.id()).map_err(|e| e.to_string())?;
        write_seed(&dir, seed)
            .and_then(|()| match origin {
                Some(origin) => write_origin(&dir, origin),
                None => Ok(()),
            })
            .and_then(|()| relay.map_or(Ok(()), |relay| write_relay(&dir, relay)))
            .map_err(|e: SeedError| e.to_string())?;
        rail.set_seed(&seed.id(), &seed.roster())
            .map_err(|e| e.to_string())
    }
}

/// Where `new` and joining put a group's folder (v1-onboarding (d);
/// bash-first-minute (d)): `~/svrngs/`
/// and the group's name, which every member who joined derives alike, so
/// their folders are one; `/`, `\` and `:` read as `-` and leading dots
/// dropped, so the name stays one directory under it, else the id's start.
pub fn folder_place(seed: &Seed) -> Option<PathBuf> {
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty())?;
    let name = seed.name().as_str().replace(['/', '\\', ':'], "-");
    let name = match name.trim_start_matches('.') {
        "" => seed.id()[..8].to_string(),
        name => name.to_string(),
    };
    Some(PathBuf::from(home).join("svrngs").join(name))
}

impl crate::folder::Folders {
    /// The group's folder kept the same at [`folder_place`] as everyone's,
    /// as `share --folder` keeps one; or why not, said beside the join or
    /// the `new`, which stands either way. Moved from svrngs's cli.rs
    /// `folder_at_join`, which asked it of this node's own API.
    pub fn place(&self, seed: &Seed) -> (PathBuf, Result<(), String>) {
        let Some(dir) = folder_place(seed) else {
            tracing::debug!(target: "group", group = %seed.id(), "group: no $HOME, so no folder placed");
            let why = "this computer names no home folder ($HOME)".to_string();
            return (PathBuf::from("~/svrngs"), Err(why));
        };
        let placed = std::fs::create_dir_all(&dir).map_err(|e| e.to_string());
        let kept = |()| {
            self.both_ways(dir.clone(), seed.id())
                .map_err(|e| e.to_string())
        };
        let placed = placed.and_then(kept).map(drop);
        tracing::debug!(target: "group", group = %seed.id(), dir = %dir.display(), ?placed, "group: the group's folder placed, or why not");
        (dir, placed)
    }
}

impl crate::node::Asking {
    /// A group made where svrngs runs, for the home page, as `svrngs new`
    /// makes one (start-group): `name`, founded by this computer's key as
    /// `person`, refused when one of its groups is called so already. It
    /// keeps the page this computer's svrngs came from, the data dir's
    /// `origin`, which install.sh writes, so the page's links to it are
    /// https links whose page gives the one line; and the relay its links
    /// carry, kept as a joiner keeps it ([`kept_relay`]): where this
    /// computer is met now, so the group is met with no restart, and after
    /// one alike. Its folder is placed as a joiner's is.
    pub(crate) fn found(&self, name: &str, person: &str) -> Result<Made, String> {
        let parse = |n: &str| Name::parse(n).map_err(|e| e.to_string());
        let (name, person, root) = (parse(name)?, parse(person)?, self.host.rail.root());
        let held = |n: &&Name| n.as_str().eq_ignore_ascii_case(name.as_str());
        if let Some(n) = groups(root).iter().map(Seed::name).find(held) {
            let why = format!("You are in {n} already: give this group another name.");
            return Err(why);
        }
        // The founder is the key this computer signs and dials with.
        let seed = draw(name, person, self.key.verifying_key());
        let page = kept_page(root)?;
        let home = self.host.endpoint.addr().relay_urls().next().cloned();
        let relay = kept_relay(carried(&self.host.relay, home), self.host.relay.n0_services);
        self.host.keep_group(&seed, page.as_ref(), relay.as_ref())?;
        tracing::debug!(target: "group", group = %seed.id(), ?page, ?relay, "group: made for the home page");
        Ok((self.folders.place(&seed), seed))
    }
}

/// A group made, after where its folder is and whether it was placed.
type Made = ((PathBuf, Result<(), String>), Seed);

/// The relay a joiner keeps from its link's `carried`, where it meets the
/// group from then on: at n0's (`n0`), none when it is one of n0's relays —
/// where the issuer's build met its groups, not the group's own, which kept
/// would outlast the operator's switch (DESIGN §10, 2026-10-13). At
/// svrngs's every relay is kept, n0's list never built there.
pub fn kept_relay(carried: Option<RelayUrl>, n0: bool) -> Option<RelayUrl> {
    let n0 = n0 && carried.as_ref().is_some_and(|r| n0_relays().contains(r));
    tracing::debug!(target: "group", ?carried, n0, "group: join — the link's relay kept, unless it is one of n0's at n0's");
    carried.filter(|_| !n0)
}

/// Every group this computer holds a seed for under `root`, the data dir.
/// A directory whose seed does not hash to its name is left out, at warn:
/// its record is not the group its seed names.
pub fn groups(root: &Path) -> Vec<Seed> {
    let namespaces = match namespaces_in(root) {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!(target: "group", error = %e, "group: cannot list the groups held");
            return Vec::new();
        }
    };
    let mut out = Vec::new();
    for ns in namespaces {
        // `namespaces_in` lists only names a journal opens, and opening
        // creates nothing: this is the rail's spelling of the directory.
        let dir = match crate::group_dir_in(root, &ns) {
            Ok(dir) => dir,
            Err(e) => {
                tracing::warn!(target: "group", namespace = %ns, error = %e, "group: a directory the rail will not open, left out");
                continue;
            }
        };
        match read_seed(&dir) {
            Ok(Some(seed)) if seed.id() == ns => out.push(seed),
            Ok(Some(seed)) => {
                tracing::warn!(target: "group", namespace = %ns, id = %seed.id(), "group: a seed that does not hash to its directory, left out")
            }
            Ok(None) => {
                tracing::debug!(target: "group", namespace = %ns, "group: a directory with no seed, left out")
            }
            Err(e) => {
                tracing::warn!(target: "group", namespace = %ns, error = %e, "group: an unreadable seed, left out")
            }
        }
    }
    out
}

/// Which groups `reference` names: a group's name, case-insensitively,
/// or a prefix of at least 8 of its id's hex — upstream's rule for naming
/// a mesh (`commonwealth-discovery/src/membership.rs:62-68` at
/// `e1a43bcd2`), so two groups of one name can be told apart.
pub fn named<'a>(groups: &'a [Seed], reference: &str) -> Vec<&'a Seed> {
    let needle = reference.trim().to_lowercase();
    groups
        .iter()
        .filter(|s| {
            !needle.is_empty()
                && (s.name().as_str().to_lowercase() == needle
                    || (needle.len() >= 8 && s.id().starts_with(&needle)))
        })
        .collect()
}
