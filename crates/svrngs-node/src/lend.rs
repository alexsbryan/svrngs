// SPDX-License-Identifier: AGPL-3.0-or-later
//! A lend (DESIGN §3): a service on this computer, on a localhost port, that
//! its person lends to every group the computer holds, kept as data — a
//! declaration file the member door reads on every request, so a running
//! node sees a lend declared or withdrawn with no restart. `svrngs share`
//! writes it through [`declare`], and `svrngs unshare` takes it back
//! through [`withdraw`], whether or not svrngs is running (u3-5).
//!
//! ```text
//!   <data dir>/lends/<lend>.json                       what the door may do
//!   <data dir>/choices/<lend>.json                     each person's yes and stop
//!   <data dir>/secrets/lends/<lend>/<header>   0600    the lender's secret
//! ```
//!
//! The declaration holds the port, the kind, how many people may use it at
//! once, the paths the door refuses and, when its lender states one, the
//! upload it may use (u3-6); the secret is apart, in
//! [`crate::declared`]'s store, so the file a person may show someone
//! carries none of it. Neither enters the record: what a computer lends is
//! asked of it, never stored (DESIGN §8, "Lends are asked, not stored"), and
//! [`Offered`] is its answer.

use std::num::{NonZeroU16, NonZeroU32, NonZeroU64};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A lend's name: the file its declaration is kept in, and the path segment
/// a member's request names it by (`/lend/<name>/…`). ASCII letters, digits,
/// `-` and `_`, 1..=64 bytes — upstream's rule for an app's name
/// (`commonwealth-transport/src/iroh_identity_forward.rs:153`,
/// `valid_app_name`), so a name cannot express a path, an encoded slash or a
/// stray byte, and one that could is refused, never repaired.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct LendName(String);

/// What a lend is, for the apps that read the list of what members lend
/// (DESIGN §3, "an opaque kind the app interprets"): a name of the same
/// rule as [`LendName`]'s, which svrngs carries and never reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Kind(String);

/// A name of a declared header's rule, the same bytes and bound — both
/// become a file name under the data dir and a token on the wire — parsed
/// once for [`LendName`] and [`Kind`], each refused by its own name.
macro_rules! header_named {
    ($($name:ident: $refused:ident, $not:literal);+) => {$(
        impl $name {
            pub fn parse(raw: &str) -> Result<Self, LendError> {
                if crate::declared::valid_header_name(raw) {
                    Ok(Self(raw.to_string()))
                } else {
                    tracing::debug!(target: "lend", raw, $not);
                    Err(LendError::$refused(raw.to_string()))
                }
            }
        }
        impl TryFrom<String> for $name {
            type Error = LendError;
            fn try_from(raw: String) -> Result<Self, LendError> {
                Self::parse(&raw)
            }
        }
        impl From<$name> for String {
            fn from(name: $name) -> String {
                name.0
            }
        }
        svrngs_core::text_newtype!($name);
    )+};
}
header_named!(LendName: NotAName, "lend: not a lend's name"; Kind: NotAKind, "lend: not a kind");

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

/// The upload a lend's lender says it may use, in bits a second: stated,
/// never measured (DESIGN §8, "The upload is declared"), so the share the
/// door tells each request is a declared figure and says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Upload(NonZeroU64);

impl Upload {
    /// `mbit` megabits a second, as a speed test states an upload.
    pub fn from_mbit(mbit: NonZeroU32) -> Self {
        Self(NonZeroU64::from(mbit).saturating_mul(NonZeroU64::new(1_000_000).expect("not 0")))
    }

    pub fn bps(self) -> u64 {
        self.0.get()
    }
}

impl std::fmt::Display for Upload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.bps() % 1_000_000 {
            0 => write!(f, "{} Mbit/s", self.bps() / 1_000_000),
            _ => write!(f, "{} bit/s", self.bps()),
        }
    }
}

/// A lend as its declaration says: where the service listens on this
/// computer, what it is, how many people may use it at once, what the
/// door refuses of it, and the upload its lender declares, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lend {
    pub name: LendName,
    /// The service's port on 127.0.0.1.
    pub port: NonZeroU16,
    pub kind: Kind,
    /// How many people may use it at once (DESIGN §8, "Too many at once").
    pub limit: NonZeroU16,
    pub refused: Vec<RefusedPath>,
    /// None when its lender declares none: then no request through the
    /// door is told a share, rather than one of a figure nobody stated.
    pub upload: Option<Upload>,
    /// The kinds of lend its pages read (u5-2, `svrngs share --reads`):
    /// a phone's page passes whoever opens it to the group's lends of each
    /// kind too, as that person (DESIGN §3, "Lent apps", call 6).
    pub reads: Vec<Kind>,
    /// The port on 127.0.0.1 of the lender's program that signs each person
    /// in (mwp-1, `svrngs share --sign-in`; `crate::door::sign_in`): a lend
    /// declares it or a secret, never both.
    pub sign_in: Option<NonZeroU16>,
    /// What an app receives of it, in its lender's words (sup-2, `svrngs
    /// share --ask`): a lend that asks lets an app in only on its person's
    /// yes ([`crate::door::choices`]). svrngs carries the words to every
    /// list and never reads them.
    pub asks: Option<String>,
}

impl Lend {
    /// The refused path covering `path`, if one does.
    pub fn refusing(&self, path: &str) -> Option<&RefusedPath> {
        self.refused.iter().find(|r| r.covers(path))
    }
}

/// The declaration's form on disk. A field it does not know is refused, not
/// ignored: a misspelt `refused` read as no refusals would open what the
/// lender closed (ARCH 6). A kind or limit absent is refused too, never
/// taken as a default nobody declared. An upload absent is none declared.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LendFile {
    port: NonZeroU16,
    kind: Kind,
    limit: NonZeroU16,
    #[serde(default)]
    refused: Vec<String>,
    #[serde(
        default,
        rename = "upload_bps",
        skip_serializing_if = "Option::is_none"
    )]
    upload: Option<Upload>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    reads: Vec<Kind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sign_in: Option<NonZeroU16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    asks: Option<String>,
}

/// A lend as its lender's computer answers a member who asks what it
/// lends: its public part, and how many people use it now (DESIGN §3, "the
/// services the group's members lend"). Parsed on the asker's side, so a
/// name that is not one never becomes an address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offered {
    pub name: LendName,
    pub kind: Kind,
    pub limit: NonZeroU16,
    /// The people the door counts on it now (`door::limit`): a request in
    /// flight, or one within its window.
    pub using: u32,
    /// The upload its lender declares (u5-1), so an app knows a lend's
    /// share before the door counts anyone on it. On the wire as
    /// `upload_bps`, `null` when none is declared, and required on read:
    /// an answer that leaves it out is not one, never read as none.
    #[serde(rename = "upload_bps", deserialize_with = "Option::deserialize")]
    pub upload: Option<Upload>,
    /// The kinds of lend it reads (u5-2): none from a computer before u5-2.
    #[serde(default)]
    pub reads: Vec<Kind>,
    /// Its lender's words for what an app receives, when it asks (sup-2):
    /// none from a computer before sup-2, which asks nobody, and absent
    /// from a list when it asks nobody.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asks: Option<String>,
}

impl Offered {
    /// Whether this lend's pages read `other`: the one decider of what a
    /// page reaches on a computer ([`crate::origin`]) and what
    /// `/svrngs-reads` tells it ([`crate::surface::reads`]).
    pub(crate) fn reads_one(&self, other: &Offered) -> bool {
        self.reads.contains(&other.kind)
    }
}

/// Why a lend could not be read.
#[derive(Debug, thiserror::Error)]
pub enum LendError {
    #[error("{0:?} is not a lend's name (letters, digits, - and _ only)")]
    NotAName(String),
    #[error("{0:?} is not a kind (letters, digits, - and _ only)")]
    NotAKind(String),
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
    #[error("{0} declares a secret and a sign-in: a lend declares one or the other")]
    SecretAndSignIn(LendName),
    #[error("{path} cannot be written: {source}")]
    Unwritable {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Where a computer keeps its lends' declarations.
pub fn lends_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("lends")
}

/// Where the lend `name`'s door keeps each person's choices
/// ([`crate::door::choices`]), beside its declaration.
pub fn choices_path(data_dir: &Path, name: &LendName) -> PathBuf {
    data_dir.join("choices").join(format!("{name}.json"))
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
        kind: file.kind,
        limit: file.limit,
        refused,
        upload: file.upload,
        reads: file.reads,
        sign_in: file.sign_in,
        asks: file.asks,
    }))
}

/// Every lend this computer declares, by name: each declaration read as
/// [`read`] reads one. A file under `lends/` whose name is not a lend's is
/// not one, and is passed over at debug.
pub fn all(data_dir: &Path) -> Result<Vec<Result<Lend, LendError>>, LendError> {
    let dir = lends_dir(data_dir);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(LendError::Unreadable { path: dir, source }),
    };
    let mut names: Vec<LendName> = Vec::new();
    for entry in entries.flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        match file.strip_suffix(".json").map(LendName::parse) {
            Some(Ok(name)) => names.push(name),
            _ => tracing::debug!(target: "lend", file, "lend: not a declaration, passed over"),
        }
    }
    names.sort();
    Ok(names
        .iter()
        .filter_map(|name| read(data_dir, name).transpose())
        .collect())
}

/// Declare `lend` on this computer, with `secret` — a header and its value
/// — as what the door adds to each request it carries to the service, or
/// none. The secret is written first, into [`crate::declared`]'s store,
/// then the declaration, through a file renamed into place, so the door
/// never reads half of one, nor a new one before its secret. A lend
/// declared again replaces the last, its secret too: none given removes
/// the one kept. A secret beside a sign-in is refused, nothing written.
pub fn declare(
    data_dir: &Path,
    lend: &Lend,
    secret: Option<(&str, &str)>,
) -> Result<(), LendError> {
    if secret.is_some() && lend.sign_in.is_some() {
        tracing::debug!(target: "lend", lend = %lend.name, "lend: refused — a secret and a sign-in both");
        return Err(LendError::SecretAndSignIn(lend.name.clone()));
    }
    let secrets = secret_dir(data_dir, &lend.name);
    let unwritable = |path: &Path| {
        let path = path.to_path_buf();
        move |source| LendError::Unwritable { path, source }
    };
    if secrets.exists() {
        std::fs::remove_dir_all(&secrets).map_err(unwritable(&secrets))?;
    }
    if let Some((header, value)) = secret {
        crate::declared::write_declared_in(&secrets, header, value)
            .map_err(unwritable(&secrets))?;
    }
    let dir = lends_dir(data_dir);
    std::fs::create_dir_all(&dir).map_err(unwritable(&dir))?;
    let file = LendFile {
        port: lend.port,
        kind: lend.kind.clone(),
        limit: lend.limit,
        refused: lend.refused.iter().map(|r| r.0.clone()).collect(),
        upload: lend.upload,
        reads: lend.reads.clone(),
        sign_in: lend.sign_in,
        asks: lend.asks.clone(),
    };
    let text = serde_json::to_string_pretty(&file).expect("a declaration always serializes");
    crate::declared::write_whole(&dir.join(format!("{}.json", lend.name)), &text)
        .map_err(|(path, source)| LendError::Unwritable { path, source })?;
    tracing::debug!(target: "lend", lend = %lend.name, port = lend.port, kind = %lend.kind,
        limit = lend.limit, refused = lend.refused.len(), upload = ?lend.upload.map(Upload::bps),
        secret = secret.is_some(), sign_in = ?lend.sign_in, asks = lend.asks.is_some(),
        "lend: declared");
    Ok(())
}

/// Take back the lend `name`: its declaration, then its choices, then its
/// secret, so a lend shared again asks again. `false`
/// when this computer lends nothing by that name.
pub fn withdraw(data_dir: &Path, name: &LendName) -> Result<bool, LendError> {
    let path = lends_dir(data_dir).join(format!("{name}.json"));
    let declared = match std::fs::remove_file(&path) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(source) => return Err(LendError::Unwritable { path, source }),
    };
    let choices = choices_path(data_dir, name);
    match std::fs::remove_file(&choices) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            return Err(LendError::Unwritable {
                path: choices,
                source: e,
            })
        }
        _ => {}
    }
    let secrets = secret_dir(data_dir, name);
    match std::fs::remove_dir_all(&secrets) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(LendError::Unwritable {
                path: secrets,
                source,
            })
        }
    }
    tracing::debug!(target: "lend", lend = %name, declared, "lend: withdrawn");
    Ok(declared)
}

#[cfg(test)]
mod tests;
