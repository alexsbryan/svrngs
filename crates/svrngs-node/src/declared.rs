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
    harden_dir(dir)?;
    write_owner_only(&path, value.trim())?;
    harden_file(&path)
}

#[cfg(unix)]
fn write_owner_only(path: &Path, value: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?
        .write_all(value.as_bytes())
}

#[cfg(unix)]
fn harden_file(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
}

#[cfg(unix)]
fn harden_dir(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn write_owner_only(path: &Path, value: &str) -> std::io::Result<()> {
    std::fs::write(path, value)
}

#[cfg(not(unix))]
fn harden_file(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn harden_dir(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_declaration_and_trims() {
        let dir = tempfile::tempdir().unwrap();
        write_declared_in(dir.path(), "X-Emby-Token", "  abc123\n").unwrap();
        assert_eq!(
            read_declared_in(dir.path()),
            vec![("x-emby-token".to_string(), "abc123".to_string())]
        );
    }

    /// Trimming is for the trailing newline an editor adds, not for the
    /// credential's own shape. Jellyfin 12 wants `MediaBrowser Token="<key>"`
    /// in one header value, so an inner space and two quotes have to survive
    /// the store or the declaration is unusable on every current Jellyfin.
    #[test]
    fn a_value_with_inner_spaces_and_quotes_survives_the_store() {
        let dir = tempfile::tempdir().unwrap();
        let credential = r#"MediaBrowser Token="a-key-1234""#;
        write_declared_in(dir.path(), "Authorization", &format!("{credential}\n")).unwrap();
        assert_eq!(
            read_declared_in(dir.path()),
            vec![("authorization".to_string(), credential.to_string())]
        );
    }

    #[test]
    fn an_absent_directory_is_empty_not_an_error() {
        assert!(read_declared_in(Path::new("/nonexistent/media/secrets")).is_empty());
    }

    #[test]
    fn an_empty_value_deletes_the_declaration() {
        let dir = tempfile::tempdir().unwrap();
        write_declared_in(dir.path(), "authorization", "x").unwrap();
        write_declared_in(dir.path(), "authorization", "  ").unwrap();
        assert!(read_declared_in(dir.path()).is_empty());
        // Idempotent: deleting what is already gone is not a failure.
        write_declared_in(dir.path(), "authorization", "").unwrap();
    }

    /// The filename reaches a request head, so it must not be able to name a
    /// path or end a line. Rejected, never repaired.
    #[test]
    fn a_name_that_could_escape_the_directory_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        for hostile in [
            "../../etc/passwd",
            "a/b",
            "x\r\ny",
            "has space",
            "",
            "..",
            ".",
        ] {
            assert!(
                write_declared_in(dir.path(), hostile, "v").is_err(),
                "{hostile:?} must be refused"
            );
        }
        assert!(read_declared_in(dir.path()).is_empty());
    }

    /// A file dropped in by hand with an unusable name is skipped rather than
    /// poisoning every other declaration on the node.
    #[test]
    fn a_hand_dropped_file_with_a_bad_name_is_skipped_not_fatal() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("not a header"), "v").unwrap();
        write_declared_in(dir.path(), "x-emby-token", "good").unwrap();
        assert_eq!(
            read_declared_in(dir.path()),
            vec![("x-emby-token".to_string(), "good".to_string())]
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_declaration_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        write_declared_in(dir.path(), "x-emby-token", "abc").unwrap();
        let mode = std::fs::metadata(dir.path().join("x-emby-token"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    /// svrngs: the directory is owner-only too, and so is a file that was
    /// already there with wider modes. The failing input: a secret written
    /// over a hand-dropped 0644 file, left 0644.
    #[cfg(unix)]
    #[test]
    fn the_declaration_dir_and_a_rewritten_file_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("secrets");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("authorization");
        std::fs::write(&file, "old").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        write_declared_in(&dir, "authorization", "new").unwrap();
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&dir), 0o700);
        assert_eq!(mode(&file), 0o600);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "new");
    }

    #[test]
    fn order_is_stable_so_a_head_is_reproducible() {
        let dir = tempfile::tempdir().unwrap();
        write_declared_in(dir.path(), "z-last", "1").unwrap();
        write_declared_in(dir.path(), "a-first", "2").unwrap();
        let names: Vec<String> = read_declared_in(dir.path())
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        assert_eq!(names, vec!["a-first".to_string(), "z-last".to_string()]);
    }
}
