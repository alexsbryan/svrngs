// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read the installed svrngs agent, not the app's job or its lend authority.

use std::path::Path;

#[cfg(target_os = "macos")]
pub fn door_at_login(pid: Option<u32>) -> Option<()> {
    let uid = crate::uid::mine().ok()?;
    let label = "sh.svrnme.svrngs"; // web/install.sh's own login service
    let target = format!("gui/{uid}/{label}");
    let path = std::path::PathBuf::from(std::env::var_os("HOME")?)
        .join(format!("Library/LaunchAgents/{label}.plist"));
    let out = std::process::Command::new("/bin/launchctl")
        .args(["print", &target])
        .output();
    let shown = out.as_ref().ok().filter(|o| o.status.success());
    let verified = shown
        .is_some_and(|o| registered(&String::from_utf8_lossy(&o.stdout), &path, pid))
        && path.is_file();
    tracing::debug!(target: "job", %target, ?pid, verified, "job: svrngs login registration");
    verified.then_some(())
}

fn registered(show: &str, path: &Path, pid: Option<u32>) -> bool {
    let field = |key| {
        show.lines()
            .find_map(|l| l.strip_prefix('\t')?.strip_prefix(key))
    };
    field("path = ").is_some_and(|p| Path::new(p) == path)
        && pid.is_none_or(|pid| field("pid = ").and_then(|p| p.parse().ok()) == Some(pid))
}

#[cfg(test)]
mod tests;
