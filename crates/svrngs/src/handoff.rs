// SPDX-License-Identifier: AGPL-3.0-or-later
//! How `svrngs open` hands the browser its open (DESIGN §3, "An open's
//! code is never a program's argument"). An open's address carries a
//! one-use code, and any account on a computer can read a program's
//! arguments (`ps`, `/proc/<pid>/cmdline`), so the opener is never given
//! that address. It is
//! given the `file://` address of a page that sends the browser on to it,
//! written to a folder of the data dir only this account can read: the same
//! on every system, and there wherever svrngs is. A folder that cannot be
//! kept so refuses the open; the address is never an argument instead.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use url::Url;

/// The folder of the data dir the pages are written to, 0700, each page 0600.
pub const DIR: &str = "opens";

/// How long a page is kept: its open lapses a minute after it is minted
/// (svrngs-node oidc/provider.rs `CODE_LIFETIME`) and is spent once used, so
/// an older page holds nothing anyone can use.
pub const KEPT: Duration = Duration::from_secs(5 * 60);

/// The page that sends a browser on to `at`, by a refresh and by script,
/// with a link where it follows neither. `at` is an `http` address, whose
/// serialization percent-encodes `"`, `'`, `<`, `>` and spaces, so only `&`
/// is escaped where it stands in an attribute.
pub fn page(at: &Url) -> String {
    let attribute = at.as_str().replace('&', "&amp;");
    let script = serde_json::to_string(at.as_str()).expect("a string is JSON");
    format!(
        "<!doctype html><meta charset=utf-8><meta http-equiv=refresh content=\"0;url={attribute}\"><title>Opening svrngs</title><script>location.replace({script})</script><p><a href=\"{attribute}\">Open your home page</a></p>\n"
    )
}

/// [`page`] for `at`, written to `<root>/opens/<name>.html` after each page
/// there older than [`KEPT`] at `now` is cleared: the page's `file://`
/// address, or the path that could not be kept private and why. The folder
/// is made 0700 (an existing one narrowed, never widened) and must then be a
/// folder, not a link, that no other account can read; the page is born 0600.
pub fn hand(
    root: &Path,
    at: &Url,
    name: [u8; 16],
    now: SystemTime,
) -> Result<Url, (PathBuf, String)> {
    use std::os::unix::fs::PermissionsExt;
    let dir = root.join(DIR);
    let failed = |e: std::io::Error| (dir.clone(), e.to_string());
    svrngs_node::declared::secret_dir(&dir).map_err(failed)?;
    let kept = std::fs::symlink_metadata(&dir).map_err(failed)?;
    let mode = kept.permissions().mode() & 0o777;
    let private = kept.is_dir() && mode & 0o077 == 0;
    tracing::debug!(target: "cli", dir = %dir.display(), folder = kept.is_dir(), mode = format!("{mode:o}"), private, "cli: open — the folder its page is handed through, kept to this account or refused");
    if !private {
        return Err((
            dir,
            format!("it is not a folder only you can read (mode {mode:o})"),
        ));
    }
    clear(&dir, now);
    let path = dir.join(format!("{}.html", hex::encode(name)));
    let written = svrngs_node::declared::write_secret(&path, page(at).as_bytes());
    written.map_err(|(file, e)| (file, e.to_string()))?;
    let whole = std::fs::canonicalize(&path).map_err(|e| (path.clone(), e.to_string()))?;
    let address = Url::from_file_path(&whole)
        .map_err(|()| (whole.clone(), "it has no file:// address".into()))?;
    tracing::debug!(target: "cli", page = %whole.display(), "cli: open — its page written 0600, its file:// address the one handed to the opener");
    Ok(address)
}

/// Each file in `dir` last written more than [`KEPT`] before `now`, removed.
/// One that cannot be read or removed is said and left: the folder is
/// private, and its open has lapsed.
fn clear(dir: &Path, now: SystemTime) {
    let read = std::fs::read_dir(dir).inspect_err(|e| {
        tracing::debug!(target: "cli", dir = %dir.display(), error = %e, "cli: open — its folder not read, so no old page cleared");
    });
    let Ok(pages) = read else { return };
    for page in pages.flatten() {
        let written = page.metadata().and_then(|m| m.modified());
        let age = written
            .as_ref()
            .ok()
            .and_then(|w| now.duration_since(*w).ok());
        let old = age.is_some_and(|age| age > KEPT);
        let removed = old.then(|| std::fs::remove_file(page.path()));
        tracing::debug!(target: "cli", page = %page.path().display(), ?age, old, ?removed, "cli: open — a page in its folder, cleared once older than five minutes, kept else");
    }
}
