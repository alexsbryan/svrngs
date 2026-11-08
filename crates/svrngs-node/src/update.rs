// SPDX-License-Identifier: AGPL-3.0-or-later
//! Getting the newest svrngs from the page this computer came from
//! (PLAN §1, the update row): the page's `get/SHA256SUMS` is the one
//! decider of what the release is, and `release` in the data dir is the one
//! record of what this computer installed. A computer whose record is that
//! hash is current; one whose record differs downloads the tarball the
//! sums name, checks it, unpacks only `svrngs`, and renames the new one
//! over it. Nothing is signed and nothing is guessed: the page's HTTPS and
//! the same sums `install.sh` checks are the trust, as they are for the
//! install line.
//!
//! A refusal keeps the running binary. A check that cannot be judged — no
//! page kept, no line for this platform, a page that does not answer — is
//! never said as "current" (ARCH 6). The caller restarts the node into the
//! new binary; this module only puts it in place.

use std::fs::{self, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ring::digest::{digest, SHA256};
use svrngs_core::link::Origin;

/// What `install.sh` writes, and the updater reads and writes: the hash of
/// the tarball this computer installed, under the data dir. `web/install.sh`
/// spells the same file in shell; `tests/install.rs` pins the pair.
pub const RELEASE_FILE: &str = "release";

const LOCK_FILE: &str = "update.lock";
const MAX_SUMS: usize = 64 * 1024;
const MAX_TARBALL: usize = 64 * 1024 * 1024;

struct Latest {
    want: String,
    tarball: String,
    base: String,
    installed: Option<String>,
}

/// Whether the page has a svrngs this computer is not running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    Current,
    Available,
}

/// What an update did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Current,
    Updated,
}

/// Why an update was not judged, or was refused. The words are a person's
/// (DESIGN §1); what is behind each is traced where it is raised.
#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("this computer does not know the page svrngs came from, so it cannot update: run the install line from your group’s page once")]
    NoPage,
    #[error("this computer is not one the release is built for, so nothing changed")]
    Platform,
    #[error("the page lists no svrngs for this computer, so nothing changed")]
    NotListed,
    #[error("the new svrngs does not match the page’s checksum, so nothing changed")]
    Mismatch,
    #[error("an update is running already")]
    Busy,
    #[error("{0}")]
    CouldNotJudge(String),
    #[error("{0}")]
    Refused(String),
}

impl UpdateError {
    /// Whether the update ran and refused, rather than being unable to
    /// judge the page (the command's exit 1, against exit 3).
    pub fn judged(&self) -> bool {
        matches!(self, Self::Mismatch | Self::Busy | Self::Refused(_))
    }
}

/// Whether the page offers a svrngs this computer does not run.
pub async fn check(dir: &Path) -> Result<Check, UpdateError> {
    let client = client()?;
    let latest = latest(dir, &client).await?;
    let current = latest.installed.as_deref() == Some(latest.want.as_str());
    tracing::debug!(target: "update", page = %latest.base, want = %latest.want, installed = ?latest.installed, current, "update: the page’s release read");
    Ok(if current {
        Check::Current
    } else {
        Check::Available
    })
}

/// Put the page’s release in place over this computer’s svrngs, unless it
/// is the one running. The caller restarts the node after `Updated`.
pub async fn apply(dir: &Path) -> Result<Outcome, UpdateError> {
    let exe = std::env::current_exe().map_err(|e| {
        UpdateError::Refused(format!("this computer cannot find the svrngs it runs: {e}"))
    })?;
    apply_at(dir, &exe).await
}

/// [`apply`] over `exe`, for a test whose svrngs is not the running one.
pub async fn apply_at(dir: &Path, exe: &Path) -> Result<Outcome, UpdateError> {
    let _held = claim(dir)?;
    let client = client()?;
    let latest = latest(dir, &client).await?;
    if latest.installed.as_deref() == Some(latest.want.as_str()) {
        tracing::debug!(target: "update", tarball = %latest.tarball, "update: the release is the one this computer installed");
        return Ok(Outcome::Current);
    }
    let url = format!("{}{}", latest.base, latest.tarball);
    let bytes = fetch(&client, &url, MAX_TARBALL).await?;
    if hash(&bytes) != latest.want {
        tracing::debug!(target: "update", url, "update: refused — the tarball does not match the page’s checksum");
        return Err(UpdateError::Mismatch);
    }
    let temp = Temp::new(dir)?;
    let tarball = temp.0.join("svrngs.tar.gz");
    fs::write(&tarball, &bytes)
        .map_err(|e| UpdateError::Refused(format!("the update’s tarball cannot be kept: {e}")))?;
    let tar = Command::new("tar")
        .arg("-xzf")
        .arg(&tarball)
        .arg("-C")
        .arg(&temp.0)
        .arg("svrngs")
        .status()
        .map_err(|e| UpdateError::Refused(format!("the update cannot be unpacked: {e}")))?;
    if !tar.success() {
        tracing::debug!(target: "update", url, "update: refused — the tarball did not unpack");
        return Err(UpdateError::Refused(
            "the new svrngs could not be unpacked".to_string(),
        ));
    }
    let unpacked = temp.0.join("svrngs");
    let meta = fs::symlink_metadata(&unpacked)
        .map_err(|e| UpdateError::Refused(format!("the new svrngs cannot be read: {e}")))?;
    if !meta.file_type().is_file() {
        tracing::debug!(target: "update", url, "update: refused — the tarball’s svrngs is not a regular file");
        return Err(UpdateError::Refused(
            "the new svrngs is not a regular file".to_string(),
        ));
    }
    let program = fs::read(&unpacked)
        .map_err(|e| UpdateError::Refused(format!("the new svrngs cannot be read: {e}")))?;
    let new = exe.with_extension("new");
    fs::write(&new, &program).map_err(|e| {
        UpdateError::Refused(format!(
            "the new svrngs cannot be put where svrngs runs: {e}"
        ))
    })?;
    fs::set_permissions(&new, std::os::unix::fs::PermissionsExt::from_mode(0o755)).map_err(
        |e| UpdateError::Refused(format!("the new svrngs cannot be made runnable: {e}")),
    )?;
    fs::rename(&new, exe).map_err(|e| {
        UpdateError::Refused(format!("the new svrngs cannot replace this one: {e}"))
    })?;
    write_release(dir, &latest.want);
    tracing::info!(target: "update", exe = %exe.display(), release = %latest.want, "update: the new svrngs is in place");
    Ok(Outcome::Updated)
}

/// The page’s release and this computer’s record of what it installed.
async fn latest(dir: &Path, client: &reqwest13::Client) -> Result<Latest, UpdateError> {
    let page = page(dir)?;
    let platform =
        platform_of(std::env::consts::OS, std::env::consts::ARCH).ok_or(UpdateError::Platform)?;
    let tarball = format!("svrngs-{platform}.tar.gz");
    let base = format!("{}/get/", page.as_str().trim_end_matches('/'));
    let sums = fetch(client, &format!("{base}SHA256SUMS"), MAX_SUMS).await?;
    let sums = String::from_utf8(sums)
        .map_err(|e| UpdateError::CouldNotJudge(format!("the page’s answer is not text: {e}")))?;
    let want = wanted(&sums, &tarball)?;
    let installed = match fs::read_to_string(dir.join(RELEASE_FILE)) {
        Ok(raw) => Some(raw.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => {
            return Err(UpdateError::CouldNotJudge(format!(
                "what this computer last installed cannot be read: {e}"
            )))
        }
    };
    Ok(Latest {
        want,
        tarball,
        base,
        installed,
    })
}

/// The page this computer was installed from, kept by `install.sh`.
fn page(dir: &Path) -> Result<Origin, UpdateError> {
    match crate::invite::kept_page(dir) {
        Ok(Some(page)) => Ok(page),
        Ok(None) => {
            tracing::debug!(target: "update", "update: no page kept, so an update cannot be judged");
            Err(UpdateError::NoPage)
        }
        Err(e) => {
            tracing::debug!(target: "update", error = %e, "update: the kept page cannot be read");
            Err(UpdateError::CouldNotJudge(format!(
                "the page svrngs came from cannot be read: {e}"
            )))
        }
    }
}

/// The release’s name for this computer, as `install.sh` and
/// `scripts/build-release.sh` spell it: `<os>-<arch>`.
fn platform_of(os: &str, arch: &str) -> Option<String> {
    let os = match os {
        "macos" => "darwin",
        "linux" => "linux",
        _ => return None,
    };
    let arch = match arch {
        "aarch64" | "x86_64" => arch,
        _ => return None,
    };
    Some(format!("{os}-{arch}"))
}

/// The hash `SHA256SUMS` names for `tarball`, or why it names none.
fn wanted(sums: &str, tarball: &str) -> Result<String, UpdateError> {
    for line in sums.lines() {
        let mut parts = line.split_whitespace();
        let (Some(hash), Some(file)) = (parts.next(), parts.next()) else {
            continue;
        };
        if file.trim_start_matches('*') != tarball {
            continue;
        }
        if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            tracing::debug!(target: "update", tarball, hash, "update: refused — the page’s line for this computer is not a SHA-256");
            return Err(UpdateError::CouldNotJudge(
                "the page’s line for this computer is not a SHA-256".to_string(),
            ));
        }
        return Ok(hash.to_ascii_lowercase());
    }
    tracing::debug!(target: "update", tarball, "update: the page lists no tarball for this computer");
    Err(UpdateError::NotListed)
}

/// The one HTTP client, from [`crate::http`].
fn client() -> Result<reqwest13::Client, UpdateError> {
    crate::http::client().map_err(|e| {
        tracing::debug!(target: "update", error = %e, "update: no HTTP client can be made");
        UpdateError::CouldNotJudge(format!("this computer cannot ask the page: {e}"))
    })
}

/// `url`’s bytes, refused past `max` and on any answer but success.
async fn fetch(client: &reqwest13::Client, url: &str, max: usize) -> Result<Vec<u8>, UpdateError> {
    let answer = client
        .get(url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            tracing::debug!(target: "update", url, error = %e, "update: the page did not answer");
            UpdateError::CouldNotJudge(format!("cannot reach the page svrngs came from: {e}"))
        })?;
    if !answer.status().is_success() {
        tracing::debug!(target: "update", url, status = %answer.status(), "update: the page answered with no file");
        return Err(UpdateError::CouldNotJudge(format!(
            "the page answered {}",
            answer.status()
        )));
    }
    let bytes = answer.bytes().await.map_err(|e| {
        tracing::debug!(target: "update", url, error = %e, "update: the page’s answer stopped before it ended");
        UpdateError::CouldNotJudge(format!(
            "the page’s answer stopped before it ended: {e}"
        ))
    })?;
    if bytes.len() > max {
        tracing::debug!(target: "update", url, len = bytes.len(), max, "update: refused — the page’s answer is larger than this update reads");
        return Err(UpdateError::CouldNotJudge(format!(
            "the page’s answer is larger than {max} bytes"
        )));
    }
    tracing::debug!(target: "update", url, len = bytes.len(), "update: the page’s file read");
    Ok(bytes.to_vec())
}

/// The SHA-256 of `bytes`, as the page spells it.
fn hash(bytes: &[u8]) -> String {
    hex::encode(digest(&SHA256, bytes).as_ref())
}

/// Why the update’s place, its lock or its folder, cannot be kept.
fn unplaced(e: std::io::Error) -> UpdateError {
    UpdateError::CouldNotJudge(format!("this computer cannot keep the update’s place: {e}"))
}

/// The update’s place, held for as long as the update runs: one update at
/// a time, in this process and in any other using the same data dir.
fn claim(dir: &Path) -> Result<File, UpdateError> {
    let path = dir.join(LOCK_FILE);
    let file = File::create(&path).map_err(unplaced)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => {
            tracing::debug!(target: "update", path = %path.display(), "update: refused — another update holds this computer’s place");
            Err(UpdateError::Busy)
        }
        Err(std::fs::TryLockError::Error(e)) => Err(unplaced(e)),
    }
}

/// A directory the update unpacks in, removed when it is dropped.
struct Temp(PathBuf);

impl Temp {
    fn new(dir: &Path) -> Result<Self, UpdateError> {
        let path = dir.join("update");
        let _ = fs::remove_dir_all(&path);
        fs::create_dir(&path).map_err(unplaced)?;
        Ok(Self(path))
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Keep the hash of the release now in place. A failure here is said and
/// not fatal: the bytes are already the release, and the next check reads
/// the page and the running bytes again.
fn write_release(dir: &Path, hash: &str) {
    let path = dir.join(RELEASE_FILE);
    if let Err(e) = fs::write(&path, hash) {
        tracing::warn!(target: "update", path = %path.display(), error = %e, "update: the new svrngs is in place, but what it installed cannot be kept");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_release_s_name_is_install_sh_s_for_each_platform() {
        assert_eq!(
            platform_of("macos", "aarch64").as_deref(),
            Some("darwin-aarch64")
        );
        assert_eq!(
            platform_of("macos", "x86_64").as_deref(),
            Some("darwin-x86_64")
        );
        assert_eq!(
            platform_of("linux", "aarch64").as_deref(),
            Some("linux-aarch64")
        );
        assert_eq!(
            platform_of("linux", "x86_64").as_deref(),
            Some("linux-x86_64")
        );
        assert_eq!(platform_of("windows", "x86_64"), None);
        assert_eq!(platform_of("linux", "riscv64"), None);
    }

    #[test]
    fn the_sums_line_for_this_computer_is_read_and_any_other_is_not() {
        let sums = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  svrngs-darwin-aarch64.tar.gz\n\
                    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb *give-darwin-aarch64.tar.gz\n";
        assert_eq!(
            wanted(sums, "svrngs-darwin-aarch64.tar.gz").unwrap(),
            "a".repeat(64)
        );
        assert!(matches!(
            wanted(sums, "svrngs-linux-x86_64.tar.gz"),
            Err(UpdateError::NotListed)
        ));
        assert!(matches!(
            wanted(
                "not-a-sum  svrngs-darwin-aarch64.tar.gz\n",
                "svrngs-darwin-aarch64.tar.gz"
            ),
            Err(UpdateError::CouldNotJudge(_))
        ));
    }
}
