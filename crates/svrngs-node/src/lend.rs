// SPDX-License-Identifier: AGPL-3.0-or-later
//! A lend (DESIGN §3): a service on this computer, on a localhost port, that
//! its person lends to every group the computer holds, kept as data — a
//! declaration file the member door reads on every request, so a running
//! node sees a lend declared or withdrawn with no restart. `svrngs share`
//! writes it (u3-5); here it is read.
//!
//! ```text
//!   <data dir>/lends/<lend>.json                       what the door may do
//!   <data dir>/secrets/lends/<lend>/<header>   0600    the lender's secret
//! ```
//!
//! The declaration holds the port and the paths the door refuses; the
//! secret is apart, in [`crate::declared`]'s store, so the file a person may
//! show someone carries none of it. Neither enters the record.

use std::num::NonZeroU16;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// A lend's name: the file its declaration is kept in, and the path segment
/// a member's request names it by (`/lend/<name>/…`). ASCII letters, digits,
/// `-` and `_`, 1..=64 bytes — upstream's rule for an app's name
/// (`commonwealth-transport/src/iroh_identity_forward.rs:153`,
/// `valid_app_name`), so a name cannot express a path, an encoded slash or a
/// stray byte, and one that could is refused, never repaired.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LendName(String);

impl LendName {
    pub fn parse(raw: &str) -> Result<Self, LendError> {
        // A declared header's rule, the same bytes and bound: both names
        // become a file name under the data dir and a token on the wire.
        if crate::declared::valid_header_name(raw) {
            Ok(Self(raw.to_string()))
        } else {
            tracing::debug!(target: "lend", raw, "lend: not a lend's name");
            Err(LendError::NotAName(raw.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for LendName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A path the door refuses for one lend, and every path under it at a
/// segment boundary. Kept as the lender wrote it, compared as
/// [`Self::covers`] says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefusedPath(String);

impl RefusedPath {
    pub fn parse(raw: &str) -> Result<Self, LendError> {
        if raw.starts_with('/') && !raw.chars().any(char::is_control) {
            Ok(Self(raw.to_string()))
        } else {
            Err(LendError::NotAPath(raw.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether `path` — a request's path, before its query — is this path or
    /// under it, as `route_by_prefix` matches a prefix (upstream's `covers`,
    /// `iroh_routed_forward.rs:304`): `/admin` covers `/admin` and
    /// `/admin/x`, never `/administer`. svrngs compares both sides
    /// percent-decoded, ASCII-case-blind and with repeated slashes as one,
    /// where upstream compared bytes: the services behind a lend read
    /// `/Admin`, `/%41dmin` and `//admin` as `/admin`, and a refusal a
    /// client could dodge by spelling is not one.
    pub fn covers(&self, path: &str) -> bool {
        let prefix = comparable(&self.0);
        let path = comparable(path);
        path == prefix
            || (path.starts_with(&prefix)
                && (prefix.ends_with('/') || path.as_bytes().get(prefix.len()) == Some(&b'/')))
    }
}

/// A path as [`RefusedPath::covers`] compares it: percent-decoded, lowercase,
/// each run of slashes one.
fn comparable(path: &str) -> String {
    let bytes = path.as_bytes();
    let hex = |b: u8| char::from(b).to_digit(16);
    let escaped = |i: usize| -> Option<u8> {
        let (h, l) = (hex(*bytes.get(i + 1)?)?, hex(*bytes.get(i + 2)?)?);
        u8::try_from(h * 16 + l).ok()
    };
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match (bytes[i], escaped(i)) {
            (b'%', Some(b)) => {
                decoded.push(b);
                i += 3;
            }
            (b, _) => {
                decoded.push(b);
                i += 1;
            }
        }
    }
    let mut out = String::with_capacity(decoded.len());
    for c in String::from_utf8_lossy(&decoded)
        .to_ascii_lowercase()
        .chars()
    {
        if !(c == '/' && out.ends_with('/')) {
            out.push(c);
        }
    }
    out
}

/// A lend as its declaration says: where the service listens on this
/// computer, and what the door refuses of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lend {
    pub name: LendName,
    /// The service's port on 127.0.0.1.
    pub port: NonZeroU16,
    pub refused: Vec<RefusedPath>,
}

impl Lend {
    /// The refused path covering `path`, if one does.
    pub fn refusing(&self, path: &str) -> Option<&RefusedPath> {
        self.refused.iter().find(|r| r.covers(path))
    }
}

/// The declaration's form on disk. A field it does not know is refused, not
/// ignored: a misspelt `refused` read as no refusals would open what the
/// lender closed (ARCH 6).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LendFile {
    port: NonZeroU16,
    #[serde(default)]
    refused: Vec<String>,
}

/// Why a lend could not be read.
#[derive(Debug, thiserror::Error)]
pub enum LendError {
    #[error("{0:?} is not a lend's name (letters, digits, - and _ only)")]
    NotAName(String),
    #[error("{0:?} is not a path: a refused path starts with / and holds no control character")]
    NotAPath(String),
    #[error("{path} cannot be read: {source}")]
    Unreadable {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path} is not a lend's declaration: {source}")]
    Malformed {
        path: PathBuf,
        source: serde_json::Error,
    },
}

/// Where a computer keeps its lends' declarations.
pub fn lends_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("lends")
}

/// Where the lend `name` keeps its secret: [`crate::declared`]'s store, one
/// directory per lend, apart from the declaration.
pub fn secret_dir(data_dir: &Path, name: &LendName) -> PathBuf {
    data_dir.join("secrets").join("lends").join(name.as_str())
}

/// The lend `name` as its declaration says, or `None` when this computer
/// lends nothing by that name.
pub fn read(data_dir: &Path, name: &LendName) -> Result<Option<Lend>, LendError> {
    let path = lends_dir(data_dir).join(format!("{name}.json"));
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(target: "lend", lend = %name, "lend: none declared by that name");
            return Ok(None);
        }
        Err(source) => return Err(LendError::Unreadable { path, source }),
    };
    let file: LendFile =
        serde_json::from_str(&text).map_err(|source| LendError::Malformed { path, source })?;
    let refused = file
        .refused
        .iter()
        .map(|r| RefusedPath::parse(r))
        .collect::<Result<_, _>>()?;
    Ok(Some(Lend {
        name: name.clone(),
        port: file.port,
        refused,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused(raw: &str) -> RefusedPath {
        RefusedPath::parse(raw).unwrap()
    }

    /// `covers` at a segment boundary, as upstream's: the path itself and
    /// what is under it, never a longer name beside it.
    #[test]
    fn a_refused_path_covers_itself_and_what_is_under_it() {
        let r = refused("/admin");
        assert!(r.covers("/admin"));
        assert!(r.covers("/admin/users"));
        assert!(!r.covers("/administer"));
        assert!(!r.covers("/"));
        assert!(refused("/admin/").covers("/admin/x"));
    }

    /// The failing inputs svrngs's comparison exists for: the spellings a
    /// service reads as the refused path.
    #[test]
    fn a_refused_path_is_not_dodged_by_case_encoding_or_slashes() {
        let r = refused("/Auth/Keys");
        for dodge in [
            "/auth/keys",
            "/AUTH/KEYS/1",
            "/%41uth/Keys",
            "//auth//keys",
            "/auth%2Fkeys",
        ] {
            assert!(r.covers(dodge), "{dodge}");
        }
    }

    #[test]
    fn a_lend_name_cannot_express_a_path() {
        assert!(LendName::parse("shelf-1_a").is_ok());
        for bad in ["", "..", "a/b", "a%2fb", "a b", "a\u{7f}", &"x".repeat(65)] {
            assert!(LendName::parse(bad).is_err(), "{bad:?}");
        }
    }

    /// A field the door does not know is refused, so a misspelt refusal is
    /// never read as none; a missing file is no lend, not an error.
    #[test]
    fn a_declaration_is_read_strictly() {
        let root = tempfile::tempdir().unwrap();
        let name = LendName::parse("shelf").unwrap();
        assert!(read(root.path(), &name).unwrap().is_none());
        std::fs::create_dir_all(lends_dir(root.path())).unwrap();
        let file = lends_dir(root.path()).join("shelf.json");
        std::fs::write(&file, r#"{"port": 8096, "refusd": ["/admin"]}"#).unwrap();
        assert!(matches!(
            read(root.path(), &name),
            Err(LendError::Malformed { .. })
        ));
        std::fs::write(&file, r#"{"port": 0}"#).unwrap();
        assert!(read(root.path(), &name).is_err(), "port 0 is no port");
        std::fs::write(&file, r#"{"port": 8096, "refused": ["/admin"]}"#).unwrap();
        let lend = read(root.path(), &name).unwrap().unwrap();
        assert_eq!(lend.port.get(), 8096);
        assert_eq!(
            lend.refusing("/Admin/x").map(RefusedPath::as_str),
            Some("/admin")
        );
    }
}
