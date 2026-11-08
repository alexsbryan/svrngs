// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-media/src/declared.rs@e1a43bcd2
//! Headers this node adds to requests reaching a service it lends: the
//! lender's own credential for its own service, which the member door
//! injects after it has set who is asking, and which never leaves this
//! computer (DESIGN §3; PLAN §4, U3's inventory, its declared.rs row). It is never
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
//!
//! (svrngs: and how this node writes any file it keeps whole,
//! [`write_whole`].)

use std::path::{Path, PathBuf};
#[cfg(unix)]
const SECRET_MODE: u32 = 0o600;
const DIRECTORY_MODE: u32 = 0o700;

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
    // A directory holds secrets of its own (`crate::lend::readers_dir`).
    for entry in entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| !t.is_dir()))
    {
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
    secret_dir(dir)?;
    write_secret(&path, value.trim().as_bytes()).map_err(|(_, e)| e)
}

/// Make private directories from their first creation. Existing owner rights
/// are never broadened: an operator's read-only directory stays read-only.
pub fn secret_dir(dir: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, DIRECTORY_MODE);
    builder.create(dir)?;
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(dir)?.permissions().mode() & DIRECTORY_MODE
    };
    #[cfg(not(unix))]
    let mode = DIRECTORY_MODE;
    tracing::debug!(target: "media", dir = %dir.display(), mode = format!("{mode:o}"), "secret-dir: kept to its owner, the owner's rights never widened");
    harden(dir, mode)
}

/// `bytes` at `path`, owner-only and whole: written to a fresh random partial
/// created 0600 and renamed into place, so no reader sees half of it, nor a
/// value another user can read — the one way this node writes a secret
/// (a lend's declaration, and the sign-in key). An error names the file
/// that failed, the partial or `path`. The parent is private before creation;
/// the partial is born 0600, synced, then renamed. No old inode is reused.
#[expect(
    clippy::disallowed_methods,
    reason = "the secret-file adapter draws a fresh partial name, never reusing a readable inode"
)]
pub fn write_secret(path: &Path, bytes: &[u8]) -> Result<(), (PathBuf, std::io::Error)> {
    let written = (|| {
        secret_dir(path.parent().unwrap_or(Path::new("."))).map_err(|e| (path.to_path_buf(), e))?;
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).map_err(|e| (path.to_path_buf(), std::io::Error::other(e)))?;
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let partial = path.with_file_name(format!(".{name}.{}.partial", hex::encode(nonce)));
        write_owner_only(&partial, bytes).map_err(|e| (partial.clone(), e))?;
        std::fs::rename(&partial, path).map_err(|e| (path.to_path_buf(), e))
    })();
    // The value is a secret: the file that failed and why, never what it holds.
    if let Err((file, why)) = &written {
        tracing::debug!(target: "media", path = %path.display(), failed = %file.display(), %why, "secret: refused — not written owner-only and whole, so nothing is reported stored");
    }
    written
}

/// `value` at `path`, the file created owner-only (0600) where the system
/// has modes. (svrngs, lean-4: upstream wrote this, and each `harden`, once
/// per `cfg`.)
fn write_owner_only(path: &Path, value: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut open = std::fs::OpenOptions::new();
    open.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut open, SECRET_MODE);
    let mut file = open.open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(SECRET_MODE))?;
    }
    file.write_all(value)?;
    file.sync_all()
}

/// `bytes` at `path`, whole: written to `.<its name>.partial` beside it and
/// renamed into place, so no reader sees half of it — the one way this node
/// writes a file whole (REVIEW_FINDINGS #107: the API's address, a lend's
/// declaration, its choices, and a synced folder's kept state,
/// events-economize). An error names the file that failed, the partial or
/// `path`.
pub(crate) fn write_whole(path: &Path, bytes: &[u8]) -> Result<(), (PathBuf, std::io::Error)> {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let partial = path.with_file_name(format!(".{name}.partial"));
    std::fs::write(&partial, bytes).map_err(|e| (partial.clone(), e))?;
    std::fs::rename(&partial, path).map_err(|e| (path.to_path_buf(), e))
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
