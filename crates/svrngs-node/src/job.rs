// SPDX-License-Identifier: AGPL-3.0-or-later
//! The systemd user unit a program runs in (svrn-docs
//! KEEP_SHARING_ON_RESTART §2): read to name the job on a lend's port
//! ([`crate::lend::Answer::Job`]), to grant a lend renewal through the
//! unit's restarts ([`Keep`]), and to renew it ([`now`], door/bound.rs
//! `renewed`). Asked of `/usr/bin/systemctl --user`, never through `PATH`,
//! and of `/sys/fs/cgroup`: read now, never kept but as a grant. Linux
//! alone; elsewhere a unit is never named, so nothing is granted.

#[cfg(any(target_os = "macos", test))]
pub(crate) mod mac;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// What a lend's lender granted when they approved a unit's job: the unit,
/// and a hash of its files they control, those under `~/.config` (its
/// Quadlet source, its unit file, its drop-ins there). Systemd's generated
/// unit and vendor drop-ins are left out, so an update of Podman or the
/// distribution asks nothing again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keep {
    pub unit: String,
    pub files: String,
}

impl std::fmt::Display for Keep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.unit)
    }
}

/// A unit as its user manager says now, to renew: its grant as one made
/// now would be, and the cgroups of its subtree, this boot's ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Now {
    pub keep: Keep,
    pub cgroups: BTreeSet<u64>,
}

/// A unit read now to renew, or why it can't be.
pub type Read = Result<Now, &'static str>;

/// The unit of the user manager of `uid` whose cgroup `path` (a `0::`
/// line of `/proc/<pid>/cgroup`) lies in: the first that isn't a slice
/// below `user@<uid>.service`, when it is a service; none for a scope (a
/// terminal's, an app's), which systemd never starts again.
pub fn unit_of(path: &str, uid: u32) -> Option<&str> {
    let (_, below) = path.split_once(&format!("/user@{uid}.service/"))?;
    let unit = below.split('/').find(|c| !c.ends_with(".slice"))?;
    unit.ends_with(".service").then_some(unit)
}

/// The job process `pid` runs in, when it is a unit of the user manager
/// of `uid` ([`unit_of`], by its `/proc/<pid>/cgroup`), as its approval
/// names it: the unit, and its `Description`.
pub fn of(pid: u32, uid: Option<u32>) -> Option<crate::lend::Answer> {
    let cgroups = std::fs::read_to_string(format!("/proc/{pid}/cgroup")).ok()?;
    let unit = unit_of(cgroups.lines().find_map(|l| l.strip_prefix("0::"))?, uid?)?;
    // Unset, systemd says the unit's name.
    let description = show(unit).ok()?.remove("Description").filter(|d| d != unit);
    let unit = unit.to_owned();
    Some(crate::lend::Answer::Job { unit, description })
}

/// `unit` now: the grant an approval of its job makes now, and its
/// subtree's cgroups; or why it can't be granted or renewed: transient,
/// with no file of its lender's, or not running.
pub fn now(unit: &str) -> Read {
    let mut said = show(unit)?;
    if said.get("Transient").map(String::as_str) != Some("no") {
        return Err("it is transient, or its user manager doesn't know it");
    }
    let config = std::env::var("HOME").map(|h| h + "/.config/");
    let config = config.map_err(|_| "this account's home can't be read")?;
    let paths = ["SourcePath", "FragmentPath", "DropInPaths"].map(|k| said.remove(k));
    let paths = paths.iter().flatten().flat_map(|v| v.split_whitespace());
    let mut files = paths.filter(|p| p.starts_with(&config)).peekable();
    files.peek().ok_or("it has no file of yours")?;
    let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
    for file in files {
        let body = std::fs::read(file).map_err(|_| "one of its files can't be read")?;
        hash.update(format!("{file}\0{}\0", body.len()).as_bytes());
        hash.update(&body);
    }
    let (unit, files) = (unit.to_owned(), hex::encode(hash.finish()));
    let keep = Keep { unit, files };
    let cgroup = said.remove("ControlGroup").filter(|c| !c.is_empty());
    let mut stack = vec![format!("/sys/fs/cgroup{}", cgroup.ok_or("it isn't running")?).into()];
    let mut cgroups = BTreeSet::new();
    while let Some(dir) = stack.pop() {
        use std::os::unix::fs::MetadataExt as _;
        // A cgroup's id is its directory's inode (kernfs), as sock_diag's
        // `INET_DIAG_CGROUP_ID` names it.
        cgroups.extend(std::fs::metadata(&dir).map(|m| m.ino()));
        let subs = std::fs::read_dir(&dir).into_iter().flatten().flatten();
        stack.extend(subs.filter_map(|e| e.file_type().ok()?.is_dir().then(|| e.path())));
    }
    Ok(Now { keep, cgroups })
}

/// What `/usr/bin/systemctl --user show` says of `unit`, by property, each
/// read traced.
pub(crate) fn show(unit: &str) -> Result<BTreeMap<String, String>, &'static str> {
    let asked =
        "Description,Transient,SourcePath,FragmentPath,DropInPaths,ControlGroup,UnitFileState,WantedBy";
    let systemctl = std::process::Command::new("/usr/bin/systemctl")
        .args(["--user", "show", "-p", asked, "--", unit])
        .output();
    let said = match systemctl {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).into_owned()),
        _ => Err("its user manager can't be asked"),
    };
    tracing::debug!(target: "job", unit, ?said, "job: what the user manager says of a unit");
    let lines = said?;
    let pairs = lines.lines().filter_map(|l| l.split_once('='));
    Ok(pairs.map(|(k, v)| (k.into(), v.into())).collect())
}
