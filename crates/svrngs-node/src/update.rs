// SPDX-License-Identifier: AGPL-3.0-or-later
//! Getting the newest svrngs from the page this computer came from
//! (PLAN §1, the update row): the page's `get/SHA256SUMS` is the one
//! decider of what the release is, and `release` in the data dir is the one
//! record of what this computer installed. A computer whose record is that
//! hash is current; one whose record differs downloads the tarball the
//! sums name, checks it, unpacks only `svrngs`, and renames the new one
//! over it. Nothing is guessed, and the page is not the trust: its sums
//! are read only once [`signed`] finds them signed by a key built into
//! svrngs, the signature made no earlier than the one this computer last
//! accepted, which `release` keeps (DESIGN §7, "A computer updates only
//! from a signed release"). `install.sh` trusts the page's HTTPS alone, as
//! the install line does.
//!
//! A refusal keeps the running binary. A check that cannot be judged — no
//! page kept, no line for this platform, a page that does not answer — is
//! never said as "current" (ARCH 6). The caller restarts the node into the
//! new binary; this module only puts it in place.
//!
//! Whoever installed svrngs updates it. A package that puts svrngs in
//! `<prefix>/bin` puts [`UPDATED_BY`] in `<prefix>/lib/svrngs`, saying how
//! it is updated; this svrngs then never replaces itself, and says that
//! instead (svrn-docs `rings/current/SVRNGS_PACKAGE.md`).

use std::fs::{self, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ring::digest::{digest, SHA256};
use svrngs_core::link::Origin;

pub mod signed;

/// What `install.sh` writes, and the updater reads and writes: the hash of
/// the tarball this computer installed, under the data dir, and on a second
/// line the time of the signature an update accepted it under, which
/// `install.sh` never writes. `web/install.sh` spells the same file in
/// shell; `tests/install.rs` pins the pair.
pub const RELEASE_FILE: &str = "release";

/// What a package beside svrngs's binary says of how it is updated, from
/// the binary's own directory: `/usr/bin/svrngs` reads
/// `/usr/lib/svrngs/updated-by`.
pub const UPDATED_BY: &str = "../lib/svrngs/updated-by";

/// Said when [`UPDATED_BY`] is there but says nothing that can be read.
const PACKAGED: &str = "svrngs updates with this computer's system updates.";

const LOCK_FILE: &str = "update.lock";
const MAX_SUMS: usize = 64 * 1024;
const MAX_SIG: usize = 4 * 1024;
const MAX_TARBALL: usize = 64 * 1024 * 1024;

struct Latest {
    want: String,
    tarball: String,
    base: String,
    installed: Option<String>,
    signed: signed::Signed,
}

/// Whether the page has a svrngs this computer is not running, or a
/// package updates this one, in the package's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    Current,
    Available,
    Packaged(String),
}

/// What an update did: nothing, the new binary put in place, or nothing
/// because a package updates this one, in its words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Current,
    Updated,
    Packaged(String),
}

/// What [`UPDATED_BY`] beside `exe` says, if it is there: its first line,
/// at most 300 characters, or [`PACKAGED`] when it says nothing.
pub fn packaged(exe: &Path) -> Option<String> {
    let said = fs::read_to_string(exe.parent()?.join(UPDATED_BY));
    tracing::debug!(target: "update", packaged = said.is_ok(), "update: whether a package updates this svrngs");
    let said = said.ok()?;
    let line = said.lines().map(str::trim).find(|l| !l.is_empty());
    Some(line.unwrap_or(PACKAGED).chars().take(300).collect())
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
    /// The page has no such file, as [`fetch`] reads a 404.
    #[error("the page answered 404 Not Found")]
    Missing,
    #[error("No update: {0} offers a new svrngs without the svrngs release key’s signature (get/SHA256SUMS.sig), so it can’t be told from anyone’s. Whoever runs that page can add the signature from the release.")]
    Unsigned(String),
    #[error("No update: {0} offers a new svrngs whose signature (get/SHA256SUMS.sig) is not the svrngs release key’s ({1}), so it can’t be told from anyone’s. Whoever runs that page can put the release’s own signature there.")]
    BadSignature(String, signed::Refusal),
    #[error("No update: {0} offers a svrngs signed earlier than the one this computer last installed, and svrngs never goes back. Whoever runs that page can put the newest release there.")]
    Older(String),
    #[error("{0}")]
    CouldNotJudge(String),
    #[error("{0}")]
    Refused(String),
}

impl UpdateError {
    /// Whether the update ran and refused, rather than being unable to
    /// judge the page (the command's exit 1, against exit 3).
    pub fn judged(&self) -> bool {
        matches!(
            self,
            Self::Mismatch
                | Self::Busy
                | Self::Refused(_)
                | Self::Unsigned(_)
                | Self::BadSignature(..)
                | Self::Older(_)
        )
    }
}

/// What reads the page's file at a URL, at most so many bytes: [`fetch`],
/// or a test's page with no socket under it.
pub trait Get: AsyncFn(&str, usize) -> Result<Vec<u8>, UpdateError> {}
impl<F: AsyncFn(&str, usize) -> Result<Vec<u8>, UpdateError>> Get for F {}

/// Whether the page offers a svrngs this computer does not run, unless a
/// package updates this one; then the page is not asked.
pub async fn check(dir: &Path) -> Result<Check, UpdateError> {
    match std::env::current_exe().ok().as_deref().and_then(packaged) {
        Some(words) => Ok(Check::Packaged(words)),
        None => check_by(dir, fetch, signed::RELEASE_KEYS).await,
    }
}

/// [`check`], the page's files read by `get`, its sums signed by one of
/// `keys`.
pub async fn check_by(dir: &Path, get: impl Get, keys: &[[u8; 32]]) -> Result<Check, UpdateError> {
    let latest = latest(dir, &get, keys).await?;
    let current = latest.installed.as_deref() == Some(latest.want.as_str());
    tracing::debug!(target: "update", page = %latest.base, want = %latest.want, installed = ?latest.installed, created = latest.signed.created(), current, "update: the page’s release read");
    Ok(if current {
        Check::Current
    } else {
        Check::Available
    })
}

/// Put the page’s release in place over this computer’s svrngs, unless it
/// is the one running. The caller restarts the node after `Updated`.
pub async fn apply(dir: &Path) -> Result<Outcome, UpdateError> {
    let exe =
        std::env::current_exe().map_err(refused("this computer cannot find the svrngs it runs"))?;
    apply_at(dir, &exe, fetch, signed::RELEASE_KEYS).await
}

/// [`apply`] over `exe`, the page's files read by `get`, its sums signed by
/// one of `keys`: for a test whose svrngs is not the running one, whose
/// page is no socket, and whose release a key of its own signed.
pub async fn apply_at(
    dir: &Path,
    exe: &Path,
    get: impl Get,
    keys: &[[u8; 32]],
) -> Result<Outcome, UpdateError> {
    if let Some(words) = packaged(exe) {
        return Ok(Outcome::Packaged(words));
    }
    let _held = claim(dir)?;
    let latest = latest(dir, &get, keys).await?;
    if latest.installed.as_deref() == Some(latest.want.as_str()) {
        tracing::debug!(target: "update", tarball = %latest.tarball, "update: the release is the one this computer installed");
        return Ok(Outcome::Current);
    }
    // Asked before the download: a svrngs that cannot write beside itself
    // was put there by someone else, who updates it.
    let new = exe.with_extension("new");
    let place = File::create(&new).and_then(|_| fs::remove_file(&new));
    place.map_err(refused("whoever installed svrngs where it is updates it"))?;
    let url = format!("{}{}", latest.base, latest.tarball);
    let bytes = get(&url, MAX_TARBALL).await?;
    if hash(&bytes) != latest.want {
        tracing::debug!(target: "update", url, "update: refused — the tarball does not match the page’s checksum");
        return Err(UpdateError::Mismatch);
    }
    let temp = Temp::new(dir)?;
    let tarball = temp.0.join("svrngs.tar.gz");
    fs::write(&tarball, &bytes).map_err(refused("the update’s tarball cannot be kept"))?;
    let tar = Command::new("tar")
        .arg("-xzf")
        .arg(&tarball)
        .arg("-C")
        .arg(&temp.0)
        .arg("svrngs")
        .status()
        .map_err(refused("the update cannot be unpacked"))?;
    if !tar.success() {
        tracing::debug!(target: "update", url, "update: refused — the tarball did not unpack");
        return Err(UpdateError::Refused(
            "the new svrngs could not be unpacked".to_string(),
        ));
    }
    let unpacked = temp.0.join("svrngs");
    let meta = fs::symlink_metadata(&unpacked).map_err(refused("the new svrngs cannot be read"))?;
    if !meta.file_type().is_file() {
        tracing::debug!(target: "update", url, "update: refused — the tarball’s svrngs is not a regular file");
        return Err(UpdateError::Refused(
            "the new svrngs is not a regular file".to_string(),
        ));
    }
    fs::copy(&unpacked, &new).map_err(refused("the new svrngs cannot be put where svrngs runs"))?;
    fs::set_permissions(&new, std::os::unix::fs::PermissionsExt::from_mode(0o755))
        .map_err(refused("the new svrngs cannot be made runnable"))?;
    fs::rename(&new, exe).map_err(refused("the new svrngs cannot replace this one"))?;
    // Keep the hash of the release now in place, and when its signature was
    // made, the floor the next is held to. A failure here is said and not
    // fatal: the bytes are already the release, and the next check reads
    // the page and the running bytes again.
    let path = dir.join(RELEASE_FILE);
    let created = latest.signed.created();
    if let Err(e) = fs::write(&path, format!("{}\n{created}\n", latest.want)) {
        tracing::warn!(target: "update", path = %path.display(), error = %e, "update: the new svrngs is in place, but what it installed cannot be kept");
    }
    tracing::info!(target: "update", exe = %exe.display(), release = %latest.want, created, "update: the new svrngs is in place");
    Ok(Outcome::Updated)
}

/// The page’s release, its sums read only once found signed by one of
/// `keys` no earlier than the last signature accepted here, and this
/// computer’s record of what it installed.
async fn latest(dir: &Path, get: &impl Get, keys: &[[u8; 32]]) -> Result<Latest, UpdateError> {
    let page = page(dir)?;
    let platform =
        platform_of(std::env::consts::OS, std::env::consts::ARCH).ok_or(UpdateError::Platform)?;
    let tarball = format!("svrngs-{platform}.tar.gz");
    let base = format!("{}/get/", page.as_str().trim_end_matches('/'));
    let sums = get(&format!("{base}SHA256SUMS"), MAX_SUMS).await?;
    let named = named(&page);
    let sig = match get(&format!("{base}SHA256SUMS.sig"), MAX_SIG).await {
        Err(UpdateError::Missing) => {
            tracing::debug!(target: "update", page = %named, "update: refused — the page serves its sums with no signature");
            return Err(UpdateError::Unsigned(named));
        }
        sig => sig?,
    };
    let signed = match signed::verify(&sums, &sig, keys) {
        Ok(signed) => signed,
        Err(why) => {
            tracing::debug!(target: "update", page = %named, ?why, "update: refused — the page’s sums are not signed by a release key");
            return Err(UpdateError::BadSignature(named, why));
        }
    };
    let (installed, floor) = installed(dir)?;
    if floor.is_some_and(|floor| signed.created() < floor) {
        tracing::debug!(target: "update", page = %named, created = signed.created(), ?floor, "update: refused — the page’s release was signed earlier than the one last installed");
        return Err(UpdateError::Older(named));
    }
    tracing::debug!(target: "update", created = signed.created(), ?floor, "update: the page’s sums are signed by a release key, no earlier than the last accepted");
    let sums = String::from_utf8(sums)
        .map_err(|e| UpdateError::CouldNotJudge(format!("the page’s answer is not text: {e}")))?;
    let want = wanted(&sums, &tarball)?;
    Ok(Latest {
        want,
        tarball,
        base,
        installed,
        signed,
    })
}

/// What this computer last installed, as `release` keeps it: the
/// tarball’s hash, and the time of the signature it was accepted under,
/// when one was kept.
fn installed(dir: &Path) -> Result<(Option<String>, Option<u32>), UpdateError> {
    let unread = |e: String| {
        UpdateError::CouldNotJudge(format!(
            "what this computer last installed cannot be read: {e}"
        ))
    };
    let raw = match fs::read_to_string(dir.join(RELEASE_FILE)) {
        Ok(raw) => raw,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok((None, None)),
        Err(e) => return Err(unread(e.to_string())),
    };
    let mut lines = raw.lines().map(str::trim);
    let hash = lines.next().map(str::to_ascii_lowercase);
    let floor = lines.next().filter(|line| !line.is_empty());
    let floor = floor.map(str::parse::<u32>).transpose();
    Ok((
        hash.filter(|hash| !hash.is_empty()),
        floor.map_err(|e| unread(e.to_string()))?,
    ))
}

/// The page as a person names it: its address with no scheme, nor a last
/// `/`.
fn named(page: &Origin) -> String {
    let raw = page.as_str();
    let named = raw.split_once("://").map_or(raw, |(_, rest)| rest);
    named.trim_end_matches('/').to_string()
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

/// `url`’s bytes, refused past `max` and on any answer but success, asked
/// with the one HTTP client, from [`crate::http`].
#[expect(clippy::disallowed_methods, reason = "dial: the update page")]
pub async fn fetch(url: &str, max: usize) -> Result<Vec<u8>, UpdateError> {
    let client = crate::http::client().map_err(|e| {
        tracing::debug!(target: "update", error = %e, "update: no HTTP client can be made");
        UpdateError::CouldNotJudge(format!("this computer cannot ask the page: {e}"))
    })?;
    let answer = client
        .get(url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            tracing::debug!(target: "update", url, error = %e, "update: the page did not answer");
            UpdateError::CouldNotJudge(format!("cannot reach the page svrngs came from: {e}"))
        })?;
    if answer.status() == reqwest13::StatusCode::NOT_FOUND {
        tracing::debug!(target: "update", url, "update: the page has no such file");
        return Err(UpdateError::Missing);
    }
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

/// The update refused where `what` could not be done, and the error why.
fn refused(what: &str) -> impl FnOnce(std::io::Error) -> UpdateError + '_ {
    move |e| UpdateError::Refused(format!("{what}: {e}"))
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

#[cfg(test)]
mod tests;
