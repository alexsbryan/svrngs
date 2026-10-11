// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-media/src/declared.rs@e1a43bcd2
//! Headers this node adds to requests reaching a service it lends: the
//! lender's own credential for its own service, which the member door
//! injects after it has set who is asking, and which never leaves this
//! computer (DESIGN §3, §8 U3's inventory, its declared.rs row). It is never
//! logged, and never in the record: the record is copied to every member.
//!
//! Layout, one directory per lend, under the data dir
//! (`crate::lend::secret_dir`; upstream kept one, `~/.svrnmesh/secrets/media/`):
//!
//! ```text
//!   <data dir>/secrets/lends/<lend>/authorization    0600   the value
//!   <data dir>/secrets/lends/<lend>/                 0700
//! ```
//!
//! For Jellyfin 12 the value is the whole credential the header carries,
//! `MediaBrowser Token="<key>"`, not the bare key.

use std::path::Path;

/// A header name the wire can carry and a path cannot escape: RFC 9110 token
/// characters minus the ones no origin uses, lowercased.
///
/// This is the ONLY thing standing between a filename and a request head, so
/// it rejects rather than sanitizes — a name that needed cleaning is a name
/// nobody meant, and silently repairing it would produce a header the operator
/// did not write. `..`, `/`, and every control byte fail the same test as a
/// stray space.
pub fn valid_header_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Every declared header, as the member door adds them. Empty when the
/// directory is absent, which is the common case and not an error.
///
/// Order is stable (sorted by name) so a head is byte-reproducible across
/// runs; a directory iteration order is not.
pub fn read_declared_in(dir: &Path) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<(String, String)> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if !valid_header_name(&name) {
            tracing::warn!(
                target: "media",
                file = %name,
                "declared-header: SKIPPED — not a usable header name (letters, digits, - and _ only)"
            );
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(entry.path()) else {
            tracing::warn!(target: "media", header = %name, "declared-header: unreadable, skipped");
            continue;
        };
        let value = raw.trim().to_string();
        if value.is_empty() {
            continue;
        }
        // The value is a secret: log THAT one is set, never what it is. Same
        // discipline as `PUT /v1/mcp/servers/{name}/token`. (svrngs: at
        // debug, where upstream's was info: the door reads the store on
        // every request, so a running node sees a lend declared or
        // withdrawn without a restart, and an info line per request would
        // drown the log.)
        tracing::debug!(
            target: "media",
            header = %name,
            "declared-header: this node will add its own credential to requests reaching its origin"
        );
        out.push((name, value));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Store a declared header's value in `dir`, hardening the file and the
/// directory. Writing an empty value deletes the declaration — so `unpublish`
/// and "I typed the wrong key" are the same operation, and neither leaves a
/// stale secret behind.
pub fn write_declared_in(dir: &Path, name: &str, value: &str) -> std::io::Result<()> {
    let name = name.to_ascii_lowercase();
    if !valid_header_name(&name) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{name:?} is not a usable header name (letters, digits, - and _ only)"),
        ));
    }
    let path = dir.join(&name);
    if value.trim().is_empty() {
        return match std::fs::remove_file(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        };
    }
    // svrngs: a hardening that fails fails the write, where upstream
    // ignored it (`let _ =`) and left a secret readable by others, reported
    // as stored (ARCH 6). The file is created owner-only, so the value is
    // never in a wider file even for a moment; a file already there with
    // wider modes is narrowed after the write.
    std::fs::create_dir_all(dir)?;
    harden(dir, 0o700)?;
    write_owner_only(&path, value.trim())?;
    harden(&path, 0o600)
}

/// `value` at `path`, the file created owner-only (0600) where the system
/// has modes. (svrngs, lean-4: upstream wrote this, and each `harden`, once
/// per `cfg`.)
fn write_owner_only(path: &Path, value: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut open = std::fs::OpenOptions::new();
    open.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut open, 0o600);
    open.open(path)?.write_all(value.as_bytes())
}

/// `path` narrowed to `mode` where the system has modes; elsewhere nothing.
fn harden(path: &Path, mode: u32) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
