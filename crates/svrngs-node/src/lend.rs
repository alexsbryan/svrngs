// SPDX-License-Identifier: AGPL-3.0-or-later
//! A lend (DESIGN §3): a service on this computer, on a localhost port, that
//! its person lends to every group the computer holds, kept as data — a
//! declaration file the member door reads on every request, so a running
//! node sees a lend declared or withdrawn with no restart. `svrngs share`
//! writes it through [`declare`], and `svrngs unshare` takes it back
//! through [`withdraw`], whether or not svrngs is running (u3-5). Or an
//! app's folder the group carries, found where the group's folder places it
//! and never declared ([`Source::Carried`], v1ui-shelf (a)).
//!
//! ```text
//!   <data dir>/lends/<lend>.json                       what the door may do
//!   <data dir>/choices/<lend>.json                     each person's yes and stop
//!   <data dir>/secrets/lends/<lend>/<header>   0600    the lender's secret
//! ```
//!
//! The declaration holds the port, the kind, how many people may use it at
//! once if its lender names a number, the paths the door refuses and, when its lender states one, the
//! upload it may use (u3-6); the secret is apart, in
//! [`crate::declared`]'s store, so the file a person may show someone
//! carries none of it. Neither enters the record: what a computer lends is
//! asked of it, never stored (DESIGN §3, "Lends are asked, not stored"), and
//! [`Offered`] is its answer.

use std::num::{NonZeroU16, NonZeroU32, NonZeroU64};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use svrngs_core::pass::Label;

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
/// never measured (DESIGN §3, "The upload is declared"), so the share the
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

/// Where a lend's answers come from (v1ui-shelf (a); COMMUNITY_APPS §3,
/// "Port or folder"): a program on this computer, which goes with its
/// lender, or an app's folder the group carries, which stays with the
/// group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// The service's port on 127.0.0.1.
    Port(NonZeroU16),
    /// `apps/<kind>/` in a synced folder of the group whose id this is,
    /// served by each computer holding it, from its store
    /// ([`crate::folder::Folders::carried`]): lent to that group alone.
    Carried(String),
    /// A port on 127.0.0.1 carried as its own bytes, not a page's
    /// (give-udp, give-tcp; `svrngs share <port> --udp` or `--tcp`): the
    /// door's [`crate::alpn::Alpn::Port`], never its HTTP.
    Raw(Raw, NonZeroU16),
}

/// How a raw lend's bytes travel, as its declaration says: `"raw": "udp"`
/// or `"raw": "tcp"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Raw {
    Udp,
    Tcp,
}

impl Raw {
    /// Its name as a person reads it, and as the door's opening names it.
    pub fn said(self) -> &'static str {
        match self {
            Raw::Udp => "UDP",
            Raw::Tcp => "TCP",
        }
    }
}

/// Which groups a declared lend is lent to (LENDING_SCOPE §"Policy and
/// representation"): the explicit stable group IDs its lender chose, or the
/// legacy declaration that named none. A closed type (AGENTS 9) so a
/// selection is never confused with its absence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Audience {
    /// A declaration that predates selection, or one never stamped: no
    /// group is reached, and the person must choose. Never all groups
    /// (ARCH 6): an old broad declaration is paused, not inherited.
    Legacy,
    /// The groups the lender explicitly chose, by their stable IDs. Empty
    /// is paused to groups; a carried app's own group is separate and never
    /// stored here ([`Source::Carried`]).
    Selected(Vec<String>),
}

impl Audience {
    /// Whether `group` may reach a lend with this audience: only a selected
    /// ID, never the legacy declaration.
    pub fn reaches(&self, group: &str) -> bool {
        matches!(self, Audience::Selected(ids) if ids.iter().any(|id| id == group))
    }

    /// The IDs as the wire and the page read them: `None` for legacy (needs
    /// selection), the list — possibly empty, paused — for a selection.
    pub fn ids(&self) -> Option<&[String]> {
        match self {
            Audience::Legacy => None,
            Audience::Selected(ids) => Some(ids),
        }
    }
}

/// A lend as its declaration says, or as its group's folder carries it:
/// where it is answered from, what it is, which groups it is lent to, how
/// many people may use it at once, what the door refuses of it, and the
/// upload its lender declares, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lend {
    pub name: LendName,
    pub source: Source,
    pub kind: Kind,
    /// The groups its lender explicitly chose (LENDING_SCOPE). A carried
    /// app's audience is its own group ([`Source::Carried`]), never this.
    pub audience: Audience,
    /// How many people may use it at once (DESIGN §3, "Too many at once"):
    /// none when its lender names no number, and then everyone in its
    /// selected groups (bash-limit; the operator, 2026-10-12).
    pub limit: Option<NonZeroU16>,
    pub refused: Vec<RefusedPath>,
    /// None when its lender declares none: then no request through the
    /// door is told a share, rather than one of a figure nobody stated.
    pub upload: Option<Upload>,
    /// The kinds of lend its pages read (u5-2, `svrngs share --reads`):
    /// a phone's page passes whoever opens it to the group's lends of each
    /// kind too, as that person (DESIGN §3, "Lent apps", call 6). A carried
    /// app's are its reads.txt's, read as each person runs it
    /// ([`crate::folder::Folders::reads`]), and none here.
    pub reads: Vec<Kind>,
    /// Its sign-in with svrngs when it says so (SIGN_IN_WITH_SVRNGS §5,
    /// "Turning it on"): the client id and secret its lender copies into
    /// the app's own settings form. This is the one place the provider
    /// reads them.
    pub sign_in_with_svrngs: Option<SignInWithSvrngs>,
    /// What an app receives of it, in its lender's words (sup-2, `svrngs
    /// share --ask`): a lend that asks lets an app in only on its person's
    /// yes ([`crate::door::choices`]). svrngs carries the words to every
    /// list and never reads them.
    pub asks: Option<String>,
}

impl Lend {
    /// This lend's label on the computer whose key is `lender`: a carried
    /// app's the group's one ([`Label::carried`]), whichever computer
    /// carries it; a declared lend's its lender's. The one decider of a
    /// lend's label, its [`Offered`]'s too.
    pub fn label(&self, lender: &[u8; 32]) -> Label {
        let carried = match &self.source {
            Source::Carried(group) => Some((self.kind.as_str(), group.as_str())),
            Source::Port(_) | Source::Raw(..) => None,
        };
        Label::lent(lender, self.name.as_str(), carried)
    }

    /// Whether `group` may reach this lend: a carried app only its own
    /// group, a declared lend only a group its lender chose (LENDING_SCOPE).
    pub fn reaches(&self, group: &str) -> bool {
        match &self.source {
            Source::Carried(own) => own == group,
            Source::Port(_) | Source::Raw(..) => self.audience.reaches(group),
        }
    }

    /// The app `kind` carried in `group`: its name the kind's and the
    /// group id's first 16 letters, so a computer in two groups carrying an
    /// app of one name gives each a label of its own (COMMUNITY_APPS §7,
    /// found 2); `None` for a folder's name that is not a kind's, traced
    /// as left out (#167).
    pub fn carried(kind: &str, group: &str) -> Option<Self> {
        let name = svrngs_core::pass::carried_name(kind, group);
        let (Ok(name), Ok(parsed)) = (LendName::parse(&name), Kind::parse(kind)) else {
            tracing::debug!(target: "folder", kind, group, "carried: left out, a folder under apps/ whose name no app can have");
            return None;
        };
        Some(Self {
            name,
            source: Source::Carried(group.into()),
            kind: parsed,
            // A carried app's reach is its own group, by `Source::Carried`;
            // this audience is never consulted for one.
            audience: Audience::Selected(Vec::new()),
            limit: None,
            refused: Vec::new(),
            upload: None,
            // Read from its files as each person runs them, which a lookup
            // reads none of ([`crate::folder::Folders::reads`]).
            reads: Vec::new(),
            sign_in_with_svrngs: None,
            asks: None,
        })
    }

    /// The refused path covering `path`, if one does.
    pub fn refusing(&self, path: &str) -> Option<&RefusedPath> {
        self.refused.iter().find(|r| r.covers(path))
    }
}

/// The declaration's form on disk. A field it does not know is refused, not
/// ignored: a misspelt `refused` read as no refusals would open what the
/// lender closed (ARCH 6). A kind absent is refused too, never taken as a
/// default nobody declared. A limit absent is none — everyone at once,
/// never a number nobody chose — and an upload absent is none declared.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LendFile {
    port: NonZeroU16,
    /// Absent for a page's port, as every declaration before give-udp is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    raw: Option<Raw>,
    kind: Kind,
    /// The groups this lend is lent to (LENDING_SCOPE), by their stable
    /// IDs. Absent is a legacy declaration that named none: paused, needing
    /// selection, never all groups (ARCH 6). Present and empty is paused;
    /// `deny_unknown_fields` makes an old binary refuse a scoped file rather
    /// than share it globally on downgrade. Written always, `null` for
    /// legacy, so every file this build writes is refused by an old one.
    #[serde(default)]
    groups: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    limit: Option<NonZeroU16>,
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
    sign_in_with_svrngs: Option<SignInWithSvrngs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    asks: Option<String>,
}

/// The characters a sign-in client's id and secret may be made of
/// (SIGN_IN_WITH_SVRNGS §5a, "Make the client id and secret from
/// `[A-Za-z0-9-._~]` only, because clients disagree on encoding Basic
/// credentials"): RFC 3986's unreserved set, and no others.
pub fn client_alphabet(raw: &str) -> bool {
    !raw.is_empty()
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b))
}

/// A lend's OpenID Connect client, as its lender's declaration keeps it
/// (SIGN_IN_WITH_SVRNGS §5 "Turning it on", §8 9): the id and secret the
/// lender copies into the app's form. A client with no secret asks as a
/// public client. A field it does not know is refused, as the declaration
/// refuses one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignInWithSvrngs {
    pub client_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

impl SignInWithSvrngs {
    /// The client, when its id and secret are made from
    /// [`client_alphabet`]'s characters: a declaration hand-edited to
    /// another spelling is refused, never repaired.
    pub fn checked(self) -> Result<Self, LendError> {
        let ok = client_alphabet(&self.client_id)
            && self.client_secret.as_deref().is_none_or(client_alphabet);
        if ok {
            Ok(self)
        } else {
            tracing::debug!(target: "lend", "lend: refused — a sign-in client is not made from letters, digits, - . _ and ~");
            Err(LendError::NotAClient)
        }
    }
}

/// A lend as its lender's computer answers a member who asks what it
/// lends: its public part, and how many people use it now (DESIGN §3, "the
/// services the group's members lend"). Parsed on the asker's side, so a
/// name that is not one never becomes an address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offered {
    pub name: LendName,
    pub kind: Kind,
    /// None when its lender names no number: everyone at once. On the wire
    /// `null` then, and required on read, as `upload_bps` is.
    #[serde(deserialize_with = "Option::deserialize")]
    pub limit: Option<NonZeroU16>,
    /// The people the door counts on it now (`door::limit`): a request in
    /// flight, or one within its window.
    pub using: u32,
    /// The upload its lender declares (u5-1), so an app knows a lend's
    /// share before the door counts anyone on it. On the wire as
    /// `upload_bps`, `null` when none is declared, and required on read:
    /// an answer that leaves it out is not one, never read as none.
    #[serde(rename = "upload_bps", deserialize_with = "Option::deserialize")]
    pub upload: Option<Upload>,
    /// The kinds of lend it reads (u5-2).
    pub reads: Vec<Kind>,
    /// Its lender's words for what an app receives, when it asks (sup-2):
    /// absent when it asks nobody.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asks: Option<String>,
    /// The group that carries it, when its group's folder does (v1ui-shelf
    /// (a)): lent to that group alone, and one copy of it shown, whichever
    /// computer answers. Absent for a port's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// The groups this lend is lent to, by stable ID (LENDING_SCOPE): the
    /// audience a list discloses. `null` is a legacy declaration that needs
    /// selection; `[]` is paused. An absent field reads as `null`, from an
    /// answer that names no audience, and is listed to nobody — never taken
    /// as all groups (`Offered::listed`).
    #[serde(default, deserialize_with = "Option::deserialize")]
    pub groups: Option<Vec<String>>,
    /// How its bytes travel when it is a port's own, not a page's
    /// (give-play): what the home page offers Play for. Absent for a page,
    /// and from a computer whose svrngs lends no port.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<Raw>,
}

impl Offered {
    /// This lend's label as listed by the computer whose key is `lender`,
    /// 64 hex, at `version` ([`Lend::label`]): a carried app's at a computer
    /// below 6 its copy's own, as that computer serves it
    /// ([`crate::version::carried_once`]); `None` for a key that is not one.
    pub(crate) fn label(&self, lender: &str, version: u32) -> Option<Label> {
        let key: [u8; 32] = hex::decode(lender).ok()?.try_into().ok()?;
        let once = crate::version::carried_once(Some(version));
        let carried = self
            .group
            .as_deref()
            .filter(|_| once)
            .map(|g| (self.kind.as_str(), g));
        Some(Label::lent(&key, self.name.as_str(), carried))
    }

    /// Whether this lend is listed to an asker the groups `stands` answers
    /// yes to (LENDING_SCOPE §"Enforcement and disclosure"): a carried app
    /// only in its own group (COMMUNITY_APPS §7, found 3), a declared lend
    /// only through a group it is actually lent to. A legacy declaration
    /// (`groups` null) and a paused one (`[]`) are never listed; the
    /// audience is never widened to every shared group. Its `groups` keep
    /// only those `stands` answers yes to: the rest of its audience is
    /// never told to the asker.
    pub(crate) fn listed(&mut self, stands: impl Fn(&str) -> bool) -> bool {
        if let Some(ids) = &mut self.groups {
            ids.retain(|id| stands(id));
        }
        match &self.group {
            Some(own) => stands(own),
            None => self.groups.as_ref().is_some_and(|ids| !ids.is_empty()),
        }
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
    #[error("{0} is carried in its group's folder, never declared")]
    Carried(LendName),
    #[error("a sign-in client's id and secret are made of letters, digits, - . _ and ~ only")]
    NotAClient,
    #[error("a lend declares its own password or a sign-in with svrngs, never both")]
    TwoSecrets,
    #[error("{path} cannot be written: {source}")]
    Unwritable {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path} is held by another change: {source}")]
    Busy {
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

/// Where the lend `name` keeps what another app reading it is let in as:
/// the lender's account for the group, sent only on that app's GET or HEAD
/// (`crate::door`'s carry), never on a person's own use of it. Inside the
/// lend's secret, so declaring the lend again or unsharing it ends it too.
pub fn readers_dir(data_dir: &Path, name: &LendName) -> PathBuf {
    secret_dir(data_dir, name).join("readers")
}

/// The header a lend's password is sent in: the one HTTP gives a
/// credential (RFC 9110 §11.6.2).
pub const SECRET_HEADER: &str = "authorization";

/// Keep `(header, value)` as what apps reading the lend `name` are let in
/// with ([`readers_dir`]), held as every change to the lend is.
pub fn keep_readers(
    data_dir: &Path,
    name: &LendName,
    (header, value): (&str, &str),
) -> Result<(), LendError> {
    let _held = lock_lend(data_dir, name)?;
    let path = readers_dir(data_dir, name);
    crate::declared::write_declared_in(&path, header, value)
        .map_err(|source| LendError::Unwritable { path, source })
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
    let sign_in_with_svrngs = file
        .sign_in_with_svrngs
        .map(SignInWithSvrngs::checked)
        .transpose()?;
    Ok(Some(Lend {
        name: name.clone(),
        source: (file.raw).map_or(Source::Port(file.port), |r| Source::Raw(r, file.port)),
        kind: file.kind,
        audience: file.groups.map_or(Audience::Legacy, Audience::Selected),
        limit: file.limit,
        refused,
        upload: file.upload,
        reads: file.reads,
        sign_in_with_svrngs,
        asks: file.asks,
    }))
}

/// Lend each declaration from before groups, one with no `groups` key or
/// the `"groups": null` a first 5 left, to `groups`, those this computer
/// stands in now, under its lock before it lends anything: it opens for
/// the people it opened for, a snapshot, so a group joined later is never
/// added and its lender can pare it down (LENDING_SCOPE §"Updates, migration
/// and mixed versions"). No group to name leaves `null`, read as needing
/// selection. Either way an svrngs before 5 refuses the file
/// (`deny_unknown_fields`) rather than lend it to every group. One that
/// cannot be written is traced and left legacy, which no group reaches.
pub fn stamp(data_dir: &Path, groups: &[String]) {
    let dir = lends_dir(data_dir);
    let Ok(entries) = std::fs::read_dir(&dir).inspect_err(|e| if e.kind() != std::io::ErrorKind::NotFound { tracing::warn!(target: "lend", dir = %dir.display(), error = %e, "lend: refused — the declarations cannot be listed, so none is stamped; each stays as it is, lent to no group") }) else {
        return;
    };
    let entries = entries.filter_map(|e| {
        e.inspect_err(|e| tracing::warn!(target: "lend", dir = %dir.display(), error = %e, "lend: a declaration cannot be listed, so it is not stamped")).ok()
    });
    for path in entries.map(|e| e.path()) {
        let name = path
            .file_name()
            .and_then(|f| f.to_str()?.strip_suffix(".json"));
        let Some(Ok(name)) = name.map(LendName::parse) else {
            continue;
        };
        let read = std::fs::read_to_string(&path).map_err(|e| e.to_string());
        let value = read
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).map_err(|e| e.to_string()));
        let Ok(value) = value.inspect_err(|e| tracing::warn!(target: "lend", file = %path.display(), error = e, "lend: refused — a declaration cannot be read, so it is not stamped; read refuses it too, so nothing is lent from it")) else {
            continue;
        };
        if value.get("groups").is_some_and(|g| !g.is_null()) {
            continue;
        }
        let lent = mutate(data_dir, &name, |lend| {
            tracing::info!(target: "lend", lend = %name, groups = groups.len(), "lend: a declaration from before groups goes to the groups this computer stands in now; in none, it needs selection");
            if lend.audience == Audience::Legacy && !groups.is_empty() {
                lend.audience = Audience::Selected(groups.to_vec());
            }
        });
        if let Err(e) = lent {
            tracing::warn!(target: "lend", lend = %name, error = %e, "lend: refused — a legacy declaration cannot be stamped, so it stays legacy, lent to no group");
        }
    }
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
/// the one kept.
pub fn declare(
    data_dir: &Path,
    lend: &Lend,
    secret: Option<(&str, &str)>,
) -> Result<(), LendError> {
    if secret.is_some() && lend.sign_in_with_svrngs.is_some() {
        tracing::debug!(target: "lend", lend = %lend.name,
            "lend: refused — a lend declares its own password or a sign-in with svrngs, never both");
        return Err(LendError::TwoSecrets);
    }
    // Held from the secret to the declaration, as every change is, so a
    // change that read the lend before never writes it back over this.
    let _held = lock_lend(data_dir, &lend.name)?;
    let secrets = secret_dir(data_dir, &lend.name);
    if secrets.exists() {
        std::fs::remove_dir_all(&secrets).map_err(unwritable(&secrets))?;
    }
    if let Some((header, value)) = secret {
        crate::declared::write_declared_in(&secrets, header, value)
            .map_err(unwritable(&secrets))?;
    }
    write(data_dir, lend)?;
    tracing::debug!(target: "lend", lend = %lend.name, source = ?lend.source, kind = %lend.kind,
        limit = ?lend.limit, refused = lend.refused.len(), upload = ?lend.upload.map(Upload::bps),
        secret = secret.is_some(), asks = lend.asks.is_some(),
        "lend: declared");
    Ok(())
}

/// Let in `limit` people at once to the lend `name`, none for everyone in
/// its selected groups (bash-limit; the home page's `/lent`): its
/// declaration rewritten whole, its secret and choices kept, so the door
/// takes it at its next request. `false` when this computer lends nothing
/// by that name. Serialized with every other change through [`mutate`], so
/// a limit edit never restores an audience another narrowed (LENDING_SCOPE
/// §"Updates, migration and mixed versions").
pub fn limit(
    data_dir: &Path,
    name: &LendName,
    limit: Option<NonZeroU16>,
) -> Result<bool, LendError> {
    mutate(data_dir, name, |lend| {
        tracing::debug!(target: "lend", lend = %name, was = ?lend.limit, now = ?limit, "lend: its limit changed");
        lend.limit = limit;
    })
}

/// Change the lend `name` in place: read it, let `edit` rewrite it, write it
/// whole — the whole read/modify/write held under one per-lend lock, so a
/// change never loses another's (LENDING_SCOPE). `false` when this computer
/// lends nothing by that name; an error when it cannot be read or written,
/// or the lock is held.
pub fn mutate(
    data_dir: &Path,
    name: &LendName,
    edit: impl FnOnce(&mut Lend),
) -> Result<bool, LendError> {
    let _held = lock_lend(data_dir, name)?;
    let Some(mut lend) = read(data_dir, name)? else {
        return Ok(false);
    };
    edit(&mut lend);
    write(data_dir, &lend).map(|()| true)
}

/// Hold the per-lend lock while a read/modify/write of `name`'s
/// declaration runs; let it go when the returned file drops. The standard
/// library's `File::try_lock`, as `cli::claim` uses it: no new lock crate
/// (LENDING_SCOPE). A held lock is a visible busy, never a lost update.
fn lock_lend(data_dir: &Path, name: &LendName) -> Result<std::fs::File, LendError> {
    let dir = data_dir.join("locks");
    std::fs::create_dir_all(&dir).map_err(unwritable(&dir))?;
    let path = dir.join(format!("{name}.lock"));
    let file = std::fs::File::create(&path).map_err(unwritable(&path))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(source) => {
            let source = match source {
                std::fs::TryLockError::WouldBlock => {
                    std::io::Error::new(std::io::ErrorKind::WouldBlock, "held")
                }
                std::fs::TryLockError::Error(e) => e,
            };
            tracing::debug!(target: "lend", lend = %name, lock = %path.display(), error = %source,
                "lend: refused — another change holds its lock");
            Err(LendError::Busy { path, source })
        }
    }
}

/// Set the groups the lend `name` is lent to, its other fields kept — its
/// secret, its sign-in client and its choices (LENDING_SCOPE §"Audience-only
/// edits retain credentials and choices"). Serialized like every change.
/// `false` when this computer lends nothing by that name.
pub fn set_audience(
    data_dir: &Path,
    name: &LendName,
    groups: Vec<String>,
) -> Result<bool, LendError> {
    mutate(data_dir, name, |lend| {
        tracing::debug!(target: "lend", lend = %name, was = ?lend.audience, now = groups.len(),
            "lend: the groups it is lent to changed");
        lend.audience = Audience::Selected(groups);
    })
}

/// `lend`'s declaration, through a file renamed into place.
fn write(data_dir: &Path, lend: &Lend) -> Result<(), LendError> {
    let (port, raw) = match lend.source {
        Source::Port(port) => (port, None),
        Source::Raw(raw, port) => (port, Some(raw)),
        Source::Carried(_) => return Err(LendError::Carried(lend.name.clone())),
    };
    let dir = lends_dir(data_dir);
    std::fs::create_dir_all(&dir).map_err(unwritable(&dir))?;
    let file = LendFile {
        port,
        raw,
        kind: lend.kind.clone(),
        groups: lend.audience.ids().map(<[String]>::to_vec),
        limit: lend.limit,
        refused: lend.refused.iter().map(|r| r.0.clone()).collect(),
        upload: lend.upload,
        reads: lend.reads.clone(),
        sign_in_with_svrngs: lend.sign_in_with_svrngs.clone(),
        asks: lend.asks.clone(),
    };
    let text = serde_json::to_string_pretty(&file).expect("a declaration always serializes");
    let path = dir.join(format!("{}.json", lend.name));
    // A declaration holding the client secret is written the way this node
    // writes a secret: 0600, and whole (SIGN_IN_WITH_SVRNGS §5, "Turning it
    // on"). One with none keeps the ordinary whole write.
    let write = if lend.sign_in_with_svrngs.is_some() {
        crate::declared::write_secret(&path, text.as_bytes())
    } else {
        crate::declared::write_whole(&path, text.as_bytes())
    };
    write.map_err(|(path, source)| LendError::Unwritable { path, source })
}

/// Take back the lend `name`: its declaration, then its choices, then its
/// secret, so a lend shared again asks again. `false`
/// when this computer lends nothing by that name. Held under its lock like
/// every change ([`mutate`]), so a change read before it cannot write the
/// lend back; busy while another holds it.
pub fn withdraw(data_dir: &Path, name: &LendName) -> Result<bool, LendError> {
    let _held = lock_lend(data_dir, name)?;
    let path = lends_dir(data_dir).join(format!("{name}.json"));
    let declared = removed(std::fs::remove_file(&path), &path)?;
    let choices = choices_path(data_dir, name);
    removed(std::fs::remove_file(&choices), &choices)?;
    let secrets = secret_dir(data_dir, name);
    removed(std::fs::remove_dir_all(&secrets), &secrets)?;
    tracing::debug!(target: "lend", lend = %name, declared, "lend: withdrawn");
    Ok(declared)
}

/// Whether `path` was there for `removed`, its removal, to take: none
/// there is no error, any other is [`unwritable`].
fn removed(removed: std::io::Result<()>, path: &Path) -> Result<bool, LendError> {
    match removed {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(unwritable(path)(source)),
    }
}

/// A change's refusal where `path` could not be written or removed.
fn unwritable(path: &Path) -> impl FnOnce(std::io::Error) -> LendError {
    let path = path.to_path_buf();
    move |source| LendError::Unwritable { path, source }
}

#[cfg(test)]
mod tests;
