// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read an exact login registration, never discover agents by name alone.

#[cfg(target_os = "macos")]
use super::{Keep, Read, Scope};
#[cfg(target_os = "macos")]
use crate::lend::{Approved, Lend, Raw};
use std::path::Path;
#[cfg(target_os = "macos")]
use std::path::PathBuf;

#[cfg(target_os = "macos")]
pub(crate) const PREFIX: &str = "svrngs-keeper-"; // keeper's launchd::label wire name

#[cfg(target_os = "macos")]
pub fn door_at_login(pid: Option<u32>) -> Option<()> {
    registered_agent("sh.svrnme.svrngs", pid).ok().map(|_| ())
}

#[cfg(target_os = "macos")]
fn registered_agent(label: &str, pid: Option<u32>) -> Result<(PathBuf, String), &'static str> {
    let uid = crate::uid::mine()?;
    let domain = format!("gui/{uid}");
    let target = format!("{domain}/{label}");
    let home = std::env::var_os("HOME").ok_or("this account's home can't be read")?;
    let path = PathBuf::from(home).join(format!("Library/LaunchAgents/{label}.plist"));
    let answer = |out: std::io::Result<std::process::Output>| {
        String::from_utf8(out.ok().filter(|o| o.status.success())?.stdout).ok()
    };
    let shown = answer(
        std::process::Command::new("/bin/launchctl")
            .args(["print", &target])
            .output(),
    );
    let verified = shown.as_ref().is_some_and(|o| registered(o, &path, pid))
        && path.symlink_metadata().is_ok_and(|m| m.is_file())
        && answer(
            std::process::Command::new("/bin/launchctl")
                .args(["print-disabled", &domain])
                .output(),
        )
        .is_some_and(|overrides| {
            let key = format!("\"{label}\" => ");
            overrides
                .lines()
                .filter_map(|l| l.trim().strip_prefix(&key))
                .all(|v| v == "enabled")
        });
    tracing::debug!(target: "job", %target, ?pid, verified, "job: launchd login registration");
    shown
        .filter(|_| verified)
        .map(|out| (path, out))
        .ok_or("it has no verified login registration")
}

fn registered(show: &str, path: &Path, pid: Option<u32>) -> bool {
    field(show, "path = ").is_some_and(|p| Path::new(p) == path)
        && field(show, "properties = ").is_some_and(|p| p.split(" | ").any(|v| v == "runatload"))
        && pid.is_none_or(|pid| running(show) == Some(pid))
}

fn field<'a>(show: &'a str, key: &str) -> Option<&'a str> {
    show.lines()
        .find_map(|l| l.strip_prefix('\t')?.strip_prefix(key))
}

fn running(show: &str) -> Option<u32> {
    field(show, "pid = ")?.parse().ok()
}

/// The name is a lookup hint; the door still judges the actual listener and taker.
#[cfg(target_os = "macos")]
pub fn now(label: &str) -> super::Read {
    let name = label.strip_prefix(PREFIX).ok_or("not a keeper service")?;
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("not a keeper service name");
    }
    let (path, show) = registered_agent(label, None)?;
    let pid = running(&show).ok_or("it isn't running")?;
    let start = crate::uid::start(pid).ok_or("its process can't be read")?;
    let body = std::fs::read(path).map_err(|_| "its plist can't be read")?;
    let files = hex::encode(ring::digest::digest(&ring::digest::SHA256, &body));
    let keep = super::Keep {
        unit: label.into(),
        files,
    };
    Ok(super::Now {
        keep,
        scope: super::Scope::Agent((pid, start)),
    })
}

/// A Mac lend's grant through restarts, the one decider of it for Share
/// (plan/run.rs `observe`) and an approval (share.rs `kept`): where its
/// lender chose to keep sharing it (`chose`) and it is no UDP share, the
/// launch agent svrngs-keeper runs it under, its label [`PREFIX`] and the
/// lend's name, as `read` says it now ([`now`]), when every process
/// `approved` holds, one at least, descends from that agent's root
/// ([`crate::uid::descends`]). Else none, and why, traced.
#[cfg(target_os = "macos")]
pub fn kept(read: fn(&str) -> Read, lend: &Lend, approved: &Approved, chose: bool) -> Option<Keep> {
    let chosen = chose && lend.raw() != Raw::Udp;
    let held = &approved.processes;
    let theirs = |root| !held.is_empty() && held.iter().all(|&p| crate::uid::descends(p, root));
    let keep = match chosen.then(|| read(&format!("{PREFIX}{}", lend.name))) {
        None => Err("its lender chooses again after each restart, or it is UDP"),
        Some(now) => now.and_then(|now| match now.scope {
            Scope::Agent(root) if theirs(root) => Ok(now.keep),
            Scope::Agent(_) => Err("it approves no process, or one that isn't the agent's"),
            Scope::Cgroups(_) => Err("it isn't a launch agent"),
        }),
    };
    tracing::debug!(target: "job", lend = %lend.name, chose, ?held, ?keep, "job: a Mac lend's grant, the launch agent svrngs runs it under, or why not");
    keep.ok()
}

#[cfg(test)]
mod tests;
