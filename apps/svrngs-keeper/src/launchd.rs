// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0
fn launchd_plist_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "cannot resolve home directory".to_string())?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join("com.svrnmesh.daemon.plist"))
}

/// PATH as seen by the shell that ran `install-service`: host-kit's
/// (`host_kit::service::captured_path` says why it is captured).
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn captured_path() -> String {
    host_kit::service::captured_path()
}

// Both XML-carrying backends need this: the launchd plist and the Windows
// task definition break identically on a path containing `&`. Gated to
// macOS alone until the Windows backend landed, and the cross-compile gate
// is what said so rather than a Windows user.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(target_os = "macos")]
#[allow(clippy::disallowed_methods)] // real $HOME: fills the launchd plist's {HOME} placeholder
fn install_launchd(bin_path: &Path) -> Result<(), String> {
    // Remove any leftover legacy (com.sovereign.daemon) registration first so an
    // upgrading user doesn't end up with two daemons fighting over the API port.
    migrate_legacy_service();

    let home = dirs::home_dir().ok_or_else(|| "cannot resolve home directory".to_string())?;
    let plist_path = launchd_plist_path()?;

    // Resolve the data root (prefers ~/.svrnmesh, falls back to a populated
    // legacy ~/.sovereign). Using the resolved root for the daemon's logs +
    // working dir means we never pre-create an empty ~/.svrnmesh here, which
    // would otherwise defeat the startup data-dir migration's existence guard.
    let data_dir = sovereign_contracts::rebrand::svrnmesh_root();

    // Make sure logs directory exists — launchd refuses to start if
    // StandardOutPath's parent is missing.
    let logs_dir = data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| format!("create {}: {e}", logs_dir.display()))?;

    let content = LAUNCHD_TEMPLATE
        .replace("{BINARY}", &bin_path.to_string_lossy())
        .replace("{DATA_DIR}", &data_dir.to_string_lossy())
        .replace("{HOME}", &home.to_string_lossy())
        .replace("{PATH}", &xml_escape(&captured_path()));

    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    std::fs::write(&plist_path, content)
        .map_err(|e| format!("write {}: {e}", plist_path.display()))?;

    // If the agent is already loaded (re-setup), unload first so load picks
    // up the fresh binary path. Ignore errors — might not be loaded yet.
    let _ = std::process::Command::new("launchctl")
        .args(["unload", plist_path.to_string_lossy().as_ref()])
        .output();

    let out = std::process::Command::new("launchctl")
        .args(["load", plist_path.to_string_lossy().as_ref()])
        .output()
        .map_err(|e| format!("launchctl load: {e}"))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("launchctl load failed: {}", stderr.trim()));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_launchd() -> Result<(), String> {
    let plist_path = launchd_plist_path()?;
    if !plist_path.exists() {
        return Ok(());
    }

    // Best-effort unload; failure is not fatal (might already be unloaded).
    let _ = std::process::Command::new("launchctl")
        .args(["unload", plist_path.to_string_lossy().as_ref()])
        .output();

    std::fs::remove_file(&plist_path)
        .map_err(|e| format!("remove {}: {e}", plist_path.display()))?;

    Ok(())
}

// ─── Linux (systemd --user) ────────────────────────────────────────

#[cfg(target_os = "linux")]
