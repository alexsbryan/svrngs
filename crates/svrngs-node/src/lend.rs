// SPDX-License-Identifier: AGPL-3.0-or-later
//! A lend (DESIGN §3): a service on this computer, on a localhost port, that
//! its person lends to every group the computer holds, kept as data — a
//! declaration file the member door reads on every request, so a running
//! node sees a lend declared or withdrawn with no restart. `svrngs share`
//! writes it through [`declare`], and `svrngs unshare` takes it back
//! through [`withdraw`], whether or not svrngs is running (u3-5), turning
//! it off for the room first ([`crate::room::withdraw`]). Or an
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

pub use eudoxia_community::binding::Approved;

/// A lend's name: the file its declaration is kept in, and the path segment
/// a member's request names it by (`/lend/<name>/…`). ASCII letters, digits,
/// `-` and `_`, 1..=64 bytes — upstream's rule for an app's name
/// (`commonwealth-transport/src/iroh_identity_forward.rs:153`,
/// `valid_app_name`), so a name cannot express a path, an encoded slash or a
/// stray byte, and one that could is refused, never repaired.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct LendName(String);

/// Which program a lend is, for the apps that read the list of what members
/// lend (DESIGN §3, "an opaque kind the app interprets"): a name of the same
/// rule as [`LendName`]'s, which svrngs carries and never reads. Open: every
/// Jellyfin shares the word, and it is the same word in a `reads.txt`, so
/// it owns nothing. On the wire and in a lend's file it is the field `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Program(String);

/// Which app: the name of its folder under `apps/`, exclusive in its group,
/// and of the part of the permanent record its entries go under,
/// `<name>/…`. Not [`Program`], whose word anyone may lend under, so that
/// a program's word never writes a part by accident; not
/// [`svrngs_core::app::App`] either, the lowercase name an app signs under.
///
/// ```
/// use svrngs_node::lend::AppName;
/// let money = AppName::parse("Money").unwrap();
/// assert_eq!(money.act(br#"{"op":{"kind":"note"}}"#).unwrap()["payload"]["kind"], "Money/note");
/// ```
///
/// A lend's word is a [`Program`], and a program does not write a part:
///
/// ```compile_fail
/// use svrngs_node::lend::{AppName, Program};
/// let lent = Program::parse("Money").unwrap();
/// AppName::act(&lent, br#"{"op":{"kind":"note"}}"#);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppName(String);

/// Text held whole once `$fits` says it fits, parsed once, each refused by
/// its own name: [`LendName`], [`Program`] and [`AppName`], a declared
/// header's rule's name, the same bytes and bound — both become a file name
/// under the data dir and a token on the wire — and [`SendRow`].
macro_rules! header_named {
    ($($name:ident: $fits:path, $refused:ident, $not:literal);+) => {$(
        impl $name {
            pub fn parse(raw: &str) -> Result<Self, LendError> {
                if $fits(raw) {
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
header_named!(
    LendName: crate::declared::valid_header_name, NotAName, "lend: not a lend's name";
    Program: crate::declared::valid_header_name, NotAProgram, "lend: not a program's name";
    AppName: crate::declared::valid_header_name, NotAProgram, "lend: not an app's name";
    SendRow: SendRow::fits, NotASend, "lend: not a sends: row"
);

impl AppName {
    /// A lent app, by the word it is lent under, as the owner of
    /// `<word>/…` in the group whose folders are `runs`; none when a folder
    /// there has that name. A lent app writing its own entries holds an
    /// [`AppName`] and not a [`Program`], because it writes a part as a
    /// folder's app does; and this is the one way from a word to an owner
    /// of a part, since a word is anyone's until a group's own record says
    /// no folder claims it, and that word came over the wire, so it is
    /// asked each time.
    pub(crate) fn of_lent(word: &Program, runs: &svrngs_core::Runs) -> Option<Self> {
        (!runs.brought.contains_key(word.as_str())).then(|| Self(word.0.clone()))
    }

    /// The act this app's page asks for, `body` under `<name>/`
    /// ([`svrngs_core::grant::app_act`]): the one way a word reaches a part.
    pub fn act(&self, body: &[u8]) -> Option<serde_json::Value> {
        svrngs_core::grant::app_act(self.as_str(), body)
    }
}

/// A path the door refuses for one lend, and every path under it at a
/// segment boundary. Kept as the lender wrote it, compared as
/// [`Self::covers`] says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefusedPath(String);
svrngs_core::text_newtype!(RefusedPath);

impl RefusedPath {
    pub fn parse(raw: &str) -> Result<Self, LendError> {
        if raw.starts_with('/') && !raw.chars().any(char::is_control) {
            Ok(Self(raw.to_string()))
        } else {
            Err(LendError::NotAPath(raw.to_string()))
        }
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
        let (byte, read) = match (bytes[i], escaped(i)) {
            (b'%', Some(b)) => (b, 3),
            (b, _) => (b, 1),
        };
        decoded.push(byte);
        i += read;
    }
    let lower = String::from_utf8_lossy(&decoded).to_ascii_lowercase();
    let mut out: Vec<char> = lower.chars().collect();
    out.dedup_by(|a, b| *a == '/' && *b == '/');
    out.into_iter().collect()
}

/// A `sends:` row a lend accepts (svrn-docs `rings/current/CONTRACTS.md`
/// §2, §7): one method that writes and one exact path, `POST /v1/embeddings`,
/// another app on its person's own computer may send to. Its path is
/// `/`-led segments of `[A-Za-z0-9._~-]`, none empty, `.` or `..`, so no
/// row holds an encoding; it is compared byte for byte ([`Self::is`]), never
/// as [`RefusedPath::covers`] compares, since a decoder would widen an exact
/// allow. Parsed where a programs file is read and again as a lend's file is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SendRow(String);

impl SendRow {
    /// Whether `raw` is a sends: row: a write's method, then a path of
    /// unreserved segments.
    fn fits(raw: &str) -> bool {
        let (method, path) = raw.split_once(' ').unwrap_or_default();
        let segment = |s: &str| !matches!(s, "" | "." | "..") && s.bytes().all(unreserved);
        let at = path
            .strip_prefix('/')
            .is_some_and(|p| p.split('/').all(segment));
        let fits = at && ["POST", "PUT", "PATCH", "DELETE"].contains(&method);
        tracing::debug!(target: "lend", raw, fits, "lend: a sends: row, read or refused");
        fits
    }

    /// Whether a request of `method` at `path`, its spelling as it came and
    /// before any query, is this row, byte for byte.
    pub fn is(&self, method: &str, path: &str) -> bool {
        self.0.split_once(' ') == Some((method, path))
    }

    /// Its path, as a person is told where an app sends.
    pub fn path(&self) -> &str {
        self.0.split_once(' ').map_or("", |(_, path)| path)
    }
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
    /// `apps/<name>/` in a synced folder of the group whose id is the
    /// `String`, served by each computer holding it, from its store
    /// ([`crate::folder::Folders::carried`]): lent to that group alone.
    Carried(AppName, String),
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
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "Option<Vec<String>>", into = "Option<Vec<String>>")]
pub enum Audience {
    /// A declaration that predates selection, or one never stamped: no
    /// group is reached, and the person must choose. Never all groups
    /// (ARCH 6): an old broad declaration is paused, not inherited.
    #[default]
    Legacy,
    /// The groups the lender explicitly chose, by their stable IDs. Empty
    /// is paused to groups; a carried app's own group is separate and never
    /// stored here ([`Source::Carried`]).
    Selected(Vec<String>),
}

impl From<Option<Vec<String>>> for Audience {
    fn from(ids: Option<Vec<String>>) -> Self {
        ids.map_or(Audience::Legacy, Audience::Selected)
    }
}

impl From<Audience> for Option<Vec<String>> {
    fn from(audience: Audience) -> Self {
        audience.ids().map(<[String]>::to_vec)
    }
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
/// where it is answered from, what it is, and the terms it is lent on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lend {
    pub name: LendName,
    pub source: Source,
    pub kind: Program,
    pub terms: Terms,
}

/// The terms a lend is lent on, as its lender declared them: which groups
/// it is lent to, how many people may use it at once, what the door
/// refuses of it, the upload its lender declares, if any, and the rest.
/// None of them, for a carried app's own ([`Lend::carried`]). As its
/// declaration keeps them, beside its [`Head`]: a field it does not know is
/// refused, not ignored, since a misspelt `refused` read as no refusals
/// would open what the lender closed (ARCH 6). A limit absent is none —
/// everyone at once, never a number nobody chose — and an upload absent is
/// none declared.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terms {
    /// The groups its lender explicitly chose (LENDING_SCOPE), by their
    /// stable IDs. A carried app's audience is its own group
    /// ([`Source::Carried`]), never this. Absent is a legacy declaration
    /// that named none: paused, needing selection, never all groups (ARCH
    /// 6). `deny_unknown_fields` makes an old binary refuse a scoped file
    /// rather than share it globally on downgrade: written always, `null`
    /// for legacy, so every file this build writes is refused by an old one.
    #[serde(default, rename = "groups")]
    pub audience: Audience,
    /// How many people may use it at once (DESIGN §3, "Too many at once"):
    /// none when its lender names no number, and then everyone in its
    /// selected groups (bash-limit; the operator, 2026-10-12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<NonZeroU16>,
    /// Each read as [`RefusedPath::parse`] reads it ([`read`]).
    #[serde(default)]
    pub refused: Vec<RefusedPath>,
    /// None when its lender declares none: then no request through the
    /// door is told a share, rather than one of a figure nobody stated.
    #[serde(
        default,
        rename = "upload_bps",
        skip_serializing_if = "Option::is_none"
    )]
    pub upload: Option<Upload>,
    /// The kinds of lend its pages read (u5-2, `svrngs share --reads`):
    /// a phone's page passes whoever opens it to the group's lends of each
    /// kind too, as that person (DESIGN §3, "Lent apps", call 6). A carried
    /// app's are its reads.txt's, read as each person runs it
    /// ([`crate::folder::Folders::reads`]), and none here.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reads: Vec<Program>,
    /// Its sign-in with svrngs when it says so (SIGN_IN_WITH_SVRNGS §5,
    /// "Turning it on"): the client id and secret its lender copies into
    /// the app's own settings form. This is the one place the provider
    /// reads them. Read as [`SignInWithSvrngs::checked`] reads it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign_in_with_svrngs: Option<SignInWithSvrngs>,
    /// What an app receives of it, in its lender's words (sup-2, `svrngs
    /// share --ask`): a lend that asks lets an app in only on its person's
    /// yes ([`crate::door::choices`]). svrngs carries the words to every
    /// list and never reads them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asks: Option<String>,
    /// Its `sends:` rows, its lender's yes to its terms' (CONTRACTS §2, "The
    /// lend is the yes"): where a carried app may send to it, as its person
    /// says ([`crate::door::admit`]). None for a carried app's own. Each
    /// parsed as it is read ([`SendRow`]); an svrngs from before sends
    /// refuses a file that holds one, never reading it as a lend that
    /// sends nothing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sends: Vec<SendRow>,
    /// What its lender approved on its port (svrn-docs SHARE_BINDING §5.1):
    /// the door carries a connection only to that ([`crate::door::bound`]).
    /// None for a carried app, which svrngs serves itself, and for one lent
    /// before this, which is held until its lender approves what runs. An
    /// svrngs from before approvals refuses a file that holds one, so a
    /// downgrade lends nothing unbound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved: Option<Approved>,
    /// The unit its lender approved, renewed through the unit's restarts
    /// (svrn-docs KEEP_SHARING_ON_RESTART §2, door/bound.rs `renewed`):
    /// none when what they approved runs in no unit of theirs, or they
    /// asked to choose again after each. An svrngs from before refuses it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keep: Option<crate::job::Keep>,
    /// When it was first lent, in Unix seconds, which a group's page says
    /// ("Lent on 3 Oct"): written with a new lend and kept through each
    /// change ([`changed`]), so one lent before this says none, never
    /// today. An svrngs from before refuses it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
    /// Its lender's choice that it is for programs, such as a chat app, not
    /// for a tab (`POST /lent`): what it can do, never its kind's name, so
    /// its cell on a group's page leads with Play and the address it
    /// copies. An svrngs from before refuses it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub for_programs: bool,
    /// A disk snapshot for renewal, absent in a draft or during a concurrent edit.
    #[serde(skip)]
    pub revision: Option<u64>,
}

/// What a declaration says beside a lend's [`Terms`]: the port it is
/// answered on, `raw` absent for a page's, as every declaration before
/// give-udp is; and what it is, refused when absent, never taken as a
/// default nobody declared. First in the file, as it always was.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Head {
    port: NonZeroU16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    raw: Option<Raw>,
    kind: Program,
}

/// What V1.12.0 to V1.12.2 kept of the program a lend was lent from, by
/// these keys of its declaration: never read, so a lend from then reads as
/// one with nothing approved, held until its lender approves (SHARE_BINDING
/// §5.5); and kept as it was when the file is written again, so V1.12.2,
/// started again, finds its own and records nothing (SHARE_BINDING_REVIEWS
/// §6 #9).
const BEFORE: [&str; 3] = ["owner", "sandbox", "command"];

/// A declaration as it is written: its [`Head`], then its [`Terms`], then
/// the keys it kept from before approvals ([`BEFORE`]), each key where it
/// always was.
#[derive(Serialize)]
struct Declared<'a> {
    #[serde(flatten)]
    head: Head,
    #[serde(flatten)]
    terms: &'a Terms,
    #[serde(flatten)]
    before: serde_json::Map<String, serde_json::Value>,
}

impl Lend {
    /// This lend's label on the computer whose key is `lender`: a carried
    /// app's the group's one ([`Label::carried`]), whichever computer
    /// carries it; a declared lend's its lender's. The one decider of a
    /// lend's label, its [`Offered`]'s too.
    pub fn label(&self, lender: &[u8; 32]) -> Label {
        let carried = match &self.source {
            Source::Carried(app, group) => Some((app.as_str(), group.as_str())),
            Source::Port(_) | Source::Raw(..) => None,
        };
        Label::lent(lender, self.name.as_str(), carried)
    }

    /// Whether `group` may reach this lend: a carried app only its own
    /// group, a declared lend only a group its lender chose (LENDING_SCOPE).
    pub fn reaches(&self, group: &str) -> bool {
        match &self.source {
            Source::Carried(_, own) => own == group,
            Source::Port(_) | Source::Raw(..) => self.terms.audience.reaches(group),
        }
    }

    /// The app `kind` carried in `group`: its name the kind's and the
    /// group id's first 16 letters, so a computer in two groups carrying an
    /// app of one name gives each a label of its own (COMMUNITY_APPS §7,
    /// found 2); `None` for a folder's name that is not a kind's, traced
    /// as left out (#167).
    pub fn carried(kind: &str, group: &str) -> Option<Self> {
        let name = svrngs_core::pass::carried_name(kind, group);
        let (Ok(name), Ok(app)) = (LendName::parse(&name), AppName::parse(kind)) else {
            tracing::debug!(target: "folder", kind, group, "carried: left out, a folder under apps/ whose name no app can have");
            return None;
        };
        Some(Self {
            name,
            // Listed under its own name: what a reader of it names.
            kind: Program(app.0.clone()),
            source: Source::Carried(app, group.into()),
            // A carried app's reach is its own group, by `Source::Carried`;
            // this audience is never consulted for one. Its reads are read
            // from its files as each person runs them, which a lookup reads
            // none of ([`crate::folder::Folders::reads`]).
            terms: Terms {
                audience: Audience::Selected(Vec::new()),
                ..Terms::default()
            },
        })
    }

    /// Its port on this computer; none for a carried app.
    pub fn port(&self) -> Option<NonZeroU16> {
        match self.source {
            Source::Port(port) | Source::Raw(_, port) => Some(port),
            Source::Carried(..) => None,
        }
    }

    /// How its port's bytes travel: a UDP share's as datagrams, any other's
    /// on TCP.
    pub fn raw(&self) -> Raw {
        match self.source {
            Source::Raw(Raw::Udp, _) => Raw::Udp,
            _ => Raw::Tcp,
        }
    }

    /// The refused path covering `path`, if one does.
    pub fn refusing(&self, path: &str) -> Option<&RefusedPath> {
        self.terms.refused.iter().find(|r| r.covers(path))
    }
}

/// The characters a sign-in client's id and secret may be made of
/// (SIGN_IN_WITH_SVRNGS §5a, "Make the client id and secret from
/// `[A-Za-z0-9-._~]` only, because clients disagree on encoding Basic
/// credentials"): RFC 3986's unreserved set, and no others.
pub fn client_alphabet(raw: &str) -> bool {
    !raw.is_empty() && raw.bytes().all(unreserved)
}

/// RFC 3986's unreserved bytes: what a sign-in client and a `sends:` row's
/// path segment are each made of.
fn unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"-._~".contains(&b)
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
    pub kind: Program,
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
    pub reads: Vec<Program>,
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
    /// Its `sends:` rows ([`Lend::sends`]), so a person's computer shows
    /// where an app would send before they say yes. Absent for none, and
    /// passed over by an svrngs from before sends.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sends: Vec<SendRow>,
    /// When it was first lent ([`Terms::since`]), absent where its lender's
    /// declaration says none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
    /// Its lender's choice that it is for programs ([`Terms::for_programs`]).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub for_programs: bool,
    /// Whether its service took a connection on its lender's computer as it
    /// listed it (`door::offered`): `false` is a lend whose program is not
    /// running, so a friend is told before opening it, not by a 502. Absent
    /// for a carried app, which svrngs serves itself, for a UDP port, and
    /// from an svrngs from before this, which said nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answering: Option<bool>,
    /// Why nobody can open it (SHARE_BINDING §5.2, §5.5), its lender's own
    /// list says: absent when nothing holds it. A member is told none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held: Option<Hold>,
    /// What answers on its port now, as its lender approves it
    /// (SHARE_BINDING §5.4): in its lender's own list alone, never a
    /// member's; absent where it isn't read, and where it couldn't be
    /// ([`Offered::unread`]), so `[]` is nothing answering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answers: Option<Vec<Answer>>,
    /// Why what answers on its port couldn't be read, when it couldn't, on
    /// a held lend in its lender's own list: a closed set
    /// ([`crate::uid::Unread`]), so a couldn't-read is said as one, with its
    /// reason, never as nothing answering. Its lender's own computer alone
    /// writes it, so it is never read from another's answer.
    #[serde(default, skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub unread: Option<crate::uid::Unread>,
    /// An opaque name for what `answers` shows, which approving it sends
    /// back, so what changed since is refused, not approved unseen
    /// ([`crate::share::token`]); given only where approving it would be
    /// taken ([`crate::share::approve`]), so a page offers Keep sharing it
    /// by this alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// Why nobody can open a lend (SHARE_BINDING §5.2, §5.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Hold {
    /// Something its lender didn't approve answers on its port.
    Another,
    /// What answers on its port can't be read.
    Unjudged,
    /// Its lender approved nothing on its port yet.
    Unapproved,
    /// The unit its lender kept it to restarted with its files changed.
    Changed,
}

/// What holds a lend's port, as read to show its lender, never compared
/// (SHARE_BINDING §5.1, I5): a process of svrngs's account, or another
/// account, whose programs are that account's to read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Answer {
    Program {
        pid: u32,
        /// Its start, as `--pid <pid>@<start>` takes it.
        start: u64,
        /// Its executable's file name.
        program: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        folder: Option<String>,
    },
    Account {
        account: String,
    },
    /// A unit of this account's user manager ([`crate::job`]).
    Job {
        unit: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

impl std::fmt::Display for Answer {
    /// "python3 in ~/dev/garden", "a program of the account _resolve", or
    /// "the service svrngs-keeper-coder.service (coder, kept by svrngs-keeper)".
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Answer::Program {
                program,
                folder: Some(folder),
                ..
            } => write!(f, "{program} in {folder}"),
            Answer::Program { program, .. } => f.write_str(program),
            Answer::Account { account } => write!(f, "a program of the account {account}"),
            Answer::Job { unit, description } => {
                let about = description
                    .as_ref()
                    .map_or(String::new(), |d| format!(" ({d})"));
                write!(f, "the service {unit}{about}")
            }
        }
    }
}

impl Offered {
    /// This lend's label as listed by the computer whose key is `lender`,
    /// 64 hex, at `version` ([`Lend::label`]): a carried app's at a computer
    /// below 6 its copy's own, as that computer serves it
    /// ([`crate::version::carried_once`]); `None` for a key that is not one.
    pub(crate) fn label(&self, lender: &str, version: u32) -> Option<Label> {
        let key: [u8; 32] = hex::decode(lender).ok()?.try_into().ok()?;
        let once = crate::version::carried_once(Some(version));
        let group = self.group.as_deref().filter(|_| once);
        let carried = group.map(|g| (self.kind.as_str(), g));
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
    #[error("{0:?} is not a program's name (letters, digits, - and _ only)")]
    NotAProgram(String),
    #[error("{0:?} is not a path: a refused path starts with / and holds no control character")]
    NotAPath(String),
    #[error("{0:?} is not a send: POST, PUT, PATCH or DELETE, a space, then a path such as /v1/embeddings, of letters, digits, . _ ~ and - between its slashes")]
    NotASend(String),
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
    /// Not withdrawn, as the room could not be told to turn it off first
    /// ([`crate::room::withdraw`]): why.
    #[error("{0} is still lent: this computer could not close it to its WiFi first ({1})")]
    Unclosed(LendName, String),
    #[error("{0} changed since you reviewed it")]
    RevisionChanged(LendName),
    #[error("{0} is not a readable share revision: {1}")]
    BadRevision(PathBuf, String),
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
pub fn keep_readers(data_dir: &Path, name: &LendName, pair: (&str, &str)) -> Result<(), LendError> {
    keep(data_dir, name, readers_dir(data_dir, name), pair)
}

/// Keep `(header, value)` as a header the lend `name`'s door adds to each
/// request ([`secret_dir`]), in place of one of that name, the others and its
/// readers' left as they are; an empty value stops adding it.
pub fn keep_header(data_dir: &Path, name: &LendName, pair: (&str, &str)) -> Result<(), LendError> {
    keep(data_dir, name, secret_dir(data_dir, name), pair)
}

fn keep(
    data_dir: &Path,
    name: &LendName,
    path: PathBuf,
    (header, value): (&str, &str),
) -> Result<(), LendError> {
    let _held = lock_lend(data_dir, name, true)?;
    let slot = lock_path(data_dir, name, "rev");
    advance(&slot, revision_at(&slot)?)?;
    crate::declared::write_declared_in(&path, header, value)
        .map_err(|source| LendError::Unwritable { path, source })
}

/// The lend `name` as its declaration says, or `None` when this computer
/// lends nothing by that name.
pub fn read(data_dir: &Path, name: &LendName) -> Result<Option<Lend>, LendError> {
    let seen = stored(data_dir, name, None)?;
    if seen.as_ref().is_none_or(|l| l.terms.keep.is_none()) {
        return Ok(seen);
    }
    // Read the revision first: a later write can only make it stale, never
    // pair the old declaration with a new revision. Busy does not authorize renewal.
    match revision(data_dir, name) {
        Ok(rev) => stored(data_dir, name, Some(rev)),
        Err(LendError::Busy { .. }) => Ok(seen),
        Err(e) => Err(e),
    }
}

fn stored(
    data_dir: &Path,
    name: &LendName,
    revision: Option<u64>,
) -> Result<Option<Lend>, LendError> {
    let path = lends_dir(data_dir).join(format!("{name}.json"));
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(target: "lend", lend = %name, "lend: none declared by that name");
            return Ok(None);
        }
        Err(source) => return Err(LendError::Unreadable { path, source }),
    };
    let parsed = parse(&text).map_err(|source| LendError::Malformed { path, source });
    let (Head { port, raw, kind }, mut terms, _) = parsed?;
    for refused in &terms.refused {
        RefusedPath::parse(refused.as_str())?;
    }
    let client = terms.sign_in_with_svrngs.map(SignInWithSvrngs::checked);
    terms.sign_in_with_svrngs = client.transpose()?;
    terms.revision = revision;
    Ok(Some(Lend {
        name: name.clone(),
        source: raw.map_or(Source::Port(port), |r| Source::Raw(r, port)),
        kind,
        terms,
    }))
}

/// A declaration's text as its [`Head`], its [`Terms`], and the keys it
/// kept from before approvals ([`BEFORE`]), each refused as its type says.
fn parse(
    text: &str,
) -> Result<(Head, Terms, serde_json::Map<String, serde_json::Value>), serde_json::Error> {
    let mut file: serde_json::Map<String, serde_json::Value> = serde_json::from_str(text)?;
    let mut take = |keys: &[&str]| {
        let taken = keys
            .iter()
            .filter_map(|k| Some((k.to_string(), file.remove(*k)?)));
        serde_json::Map::from_iter(taken)
    };
    let head = take(&["port", "raw", "kind"]);
    let before = take(&BEFORE);
    let head = serde_json::from_value(head.into())?;
    Ok((head, serde_json::from_value(file.into())?, before))
}

/// Lend each declaration from before groups, [`Audience::Legacy`] as
/// [`all`] reads it (no `groups` key, or the `"groups": null` a first 5
/// left), to `groups`, those this computer stands in now, under its lock
/// before it lends anything: it opens for the people it opened for, a
/// snapshot, so a group joined later is never added and its lender can pare
/// it down (LENDING_SCOPE §"Updates, migration and mixed versions"). No
/// group to name leaves `null`, read as needing selection. Either way an
/// svrngs before 5 refuses the file (`deny_unknown_fields`) rather than
/// lend it to every group. One that cannot be written is traced and left
/// legacy, which no group reaches.
pub fn stamp(data_dir: &Path, groups: &[String]) {
    let listed = all(data_dir).inspect_err(|e| tracing::warn!(target: "lend", error = %e, "lend: refused — the declarations cannot be listed, so none is stamped; each stays as it is, lent to no group"));
    for read in listed.into_iter().flatten() {
        let Ok(lend) = read.inspect_err(|e| tracing::warn!(target: "lend", error = %e, "lend: refused — a declaration cannot be read, so it is not stamped; read refuses it too, so nothing is lent from it")) else {
            continue;
        };
        if lend.terms.audience != Audience::Legacy {
            continue;
        }
        let name = lend.name;
        let lent = mutate(data_dir, &name, |lend| {
            tracing::info!(target: "lend", lend = %name, groups = groups.len(), "lend: a declaration from before groups goes to the groups this computer stands in now; in none, it needs selection");
            if lend.terms.audience == Audience::Legacy && !groups.is_empty() {
                lend.terms.audience = Audience::Selected(groups.to_vec());
            }
        });
        if let Err(e) = lent {
            tracing::warn!(target: "lend", lend = %name, error = %e, "lend: refused — a legacy declaration cannot be stamped, so it stays legacy, lent to no group");
        }
    }
}

/// Every lend this computer declares, by name: each declaration read as
/// [`read`] reads one. A file under `lends/` whose name is not a lend's is
/// not one, and is passed over at debug; an entry the listing can't read is
/// passed over at warn.
pub fn all(data_dir: &Path) -> Result<Vec<Result<Lend, LendError>>, LendError> {
    let dir = lends_dir(data_dir);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(LendError::Unreadable { path: dir, source }),
    };
    let mut names: Vec<LendName> = Vec::new();
    let listed = entries.filter_map(|e| {
        e.inspect_err(|e| tracing::warn!(target: "lend", dir = %dir.display(), error = %e, "lend: a declaration cannot be listed, so it is passed over")).ok()
    });
    for entry in listed {
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
    let _held = lock_lend(data_dir, &lend.name, true)?;
    let slot = lock_path(data_dir, &lend.name, "rev");
    advance(&slot, revision_at(&slot)?)?;
    replace_secret(data_dir, lend, secret)?;
    write(data_dir, lend)?;
    tracing::debug!(target: "lend", lend = %lend.name, source = ?lend.source, kind = %lend.kind,
        limit = ?lend.terms.limit, refused = lend.terms.refused.len(), upload = ?lend.terms.upload.map(Upload::bps),
        secret = secret.is_some(), asks = lend.terms.asks.is_some(),
        "lend: declared");
    Ok(())
}

type Header<'a> = (&'a str, &'a str);

fn replace_secret(dir: &Path, lend: &Lend, secret: Option<Header<'_>>) -> Result<(), LendError> {
    if secret.is_some() && lend.terms.sign_in_with_svrngs.is_some() {
        tracing::debug!(target: "lend", lend = %lend.name,
            "lend: refused — a lend declares its own password or a sign-in with svrngs, never both");
        return Err(LendError::TwoSecrets);
    }
    let secrets = secret_dir(dir, &lend.name);
    if secrets.exists() {
        std::fs::remove_dir_all(&secrets).map_err(unwritable(&secrets))?;
    }
    if let Some((header, value)) = secret {
        crate::declared::write_declared_in(&secrets, header, value)
            .map_err(unwritable(&secrets))?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub enum Credentials<'a> {
    Keep,
    Replace(Option<Header<'a>>),
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
    changed(data_dir, name, None, Credentials::Keep, |had| {
        had.map(|mut lend| {
            edit(&mut lend);
            lend
        })
    })
    .map(|rev| rev.is_some())
}

/// Publish the complete declaration at the reviewed revision. The existing
/// credential and choice files are outside this change.
pub(crate) fn publish(
    dir: &Path,
    lend: Lend,
    if_rev: u64,
    secret: Credentials<'_>,
) -> Result<u64, LendError> {
    changed(dir, &lend.name.clone(), Some(if_rev), secret, |_| {
        Some(lend)
    })
    .map(|rev| rev.expect("publication supplies a declaration"))
}

fn changed(
    dir: &Path,
    name: &LendName,
    if_rev: Option<u64>,
    secret: Credentials<'_>,
    edit: impl FnOnce(Option<Lend>) -> Option<Lend>,
) -> Result<Option<u64>, LendError> {
    let _held = lock_lend(dir, name, true)?;
    let slot = lock_path(dir, name, "rev");
    let rev = revision_at(&slot)?;
    if if_rev.is_some_and(|expected| expected != rev) {
        tracing::debug!(target: "lend", %name, ?if_rev, rev, "lend: publication refused at a stale revision");
        return Err(LendError::RevisionChanged(name.clone()));
    }
    let had = stored(dir, name, Some(rev))?;
    let since = (had.as_ref()).map_or_else(crate::door::unix_now, |l| l.terms.since);
    let Some(mut lend) = edit(had) else {
        return Ok(None);
    };
    tracing::debug!(target: "lend", %name, ?since, "lend: when it was first lent, kept, or now for a new one");
    lend.terms.since = since;
    let rev = advance(&slot, rev)?;
    if let Credentials::Replace(secret) = secret {
        replace_secret(dir, &lend, secret)?;
    }
    write(dir, &lend)?;
    Ok(Some(rev))
}

/// The current revision of this name, even when it was withdrawn. Observe
/// this before reading other preview facts; a subsequent edit invalidates it.
pub fn revision(dir: &Path, name: &LendName) -> Result<u64, LendError> {
    let Some(_held) = lock_lend(dir, name, false)? else {
        return Ok(0);
    };
    revision_at(&lock_path(dir, name, "rev"))
}

fn lock_path(dir: &Path, name: &LendName, ext: &str) -> PathBuf {
    dir.join("locks").join(format!("{name}.{ext}"))
}

fn revision_at(path: &Path) -> Result<u64, LendError> {
    crate::declared::read_json(path)
        .map(|rev| rev.unwrap_or(0))
        .map_err(|why| LendError::BadRevision(path.into(), why))
}

// Invalidate previews before touching access. If the subsequent write fails,
// the revision is spent, but the declaration is still the old one or absent.
fn advance(path: &Path, rev: u64) -> Result<u64, LendError> {
    let next = rev
        .checked_add(1)
        .ok_or_else(|| LendError::BadRevision(path.into(), "counter exhausted".into()))?;
    crate::declared::write_secret(path, next.to_string().as_bytes())
        .map_err(|(path, source)| LendError::Unwritable { path, source })?;
    // A withdrawal must not outlast its new counter after a power loss.
    #[cfg(unix)]
    for dir in path.ancestors().skip(1).take(2) {
        std::fs::File::open(dir)
            .and_then(|f| f.sync_all())
            .map_err(unwritable(dir))?;
    }
    tracing::debug!(target: "lend", file = %path.display(), rev = next, "lend: revision advanced");
    Ok(next)
}

/// Hold the per-lend lock while a read/modify/write of `name`'s
/// declaration runs; let it go when the returned [`Held`] drops. The
/// standard library's file locks, as `cli::claim` uses them: no new
/// lock crate (LENDING_SCOPE). A held lock is a visible busy, never a lost
/// update. A read never creates a lock; absence precedes all versioned writes.
fn lock_lend(data_dir: &Path, name: &LendName, writing: bool) -> Result<Option<Held>, LendError> {
    let path = lock_path(data_dir, name, "lock");
    let file = if writing {
        let dir = path.parent().expect("lock has a folder");
        std::fs::create_dir_all(dir).map_err(unwritable(dir))?;
        std::fs::File::create(&path).map_err(unwritable(&path))?
    } else {
        match std::fs::File::open(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            read => read.map_err(|source| LendError::Unreadable {
                path: path.clone(),
                source,
            })?,
        }
    };
    match if writing {
        file.try_lock()
    } else {
        file.try_lock_shared()
    } {
        Ok(()) => Ok(Some(Held(file))),
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

/// A lend's lock, held until this drops, which unlocks it before the file
/// closes: a program started while it was held shares the open file until
/// that program runs, so closing alone left the lock held a moment, and
/// the next change refused busy (lend/tests.rs
/// `a_change_let_go_is_never_held_by_a_program_starting_at_the_time`).
struct Held(std::fs::File);

impl Drop for Held {
    fn drop(&mut self) {
        let unlocked = self.0.unlock();
        tracing::debug!(target: "lend", ?unlocked, "lend: its lock let go");
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
        tracing::debug!(target: "lend", lend = %name, was = ?lend.terms.audience, now = groups.len(),
            "lend: the groups it is lent to changed");
        lend.terms.audience = Audience::Selected(groups);
    })
}

/// `lend`'s declaration, through a file renamed into place.
fn write(data_dir: &Path, lend: &Lend) -> Result<(), LendError> {
    let (port, raw) = match lend.source {
        Source::Port(port) => (port, None),
        Source::Raw(raw, port) => (port, Some(raw)),
        Source::Carried(..) => return Err(LendError::Carried(lend.name.clone())),
    };
    let dir = lends_dir(data_dir);
    std::fs::create_dir_all(&dir).map_err(unwritable(&dir))?;
    let path = dir.join(format!("{}.json", lend.name));
    let had = std::fs::read_to_string(&path).ok();
    let before = had
        .and_then(|had| parse(&had).ok())
        .map(|(.., before)| before);
    let head = Head {
        port,
        raw,
        kind: lend.kind.clone(),
    };
    let file = Declared {
        head,
        terms: &lend.terms,
        before: before.unwrap_or_default(),
    };
    let text = serde_json::to_string_pretty(&file).expect("a declaration always serializes");
    // A declaration holding the client secret is written the way this node
    // writes a secret: 0600, and whole (SIGN_IN_WITH_SVRNGS §5, "Turning it
    // on"). One with none keeps the ordinary whole write.
    let write = if lend.terms.sign_in_with_svrngs.is_some() {
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
/// lend back; busy while another holds it. svrngs takes a lend back through
/// [`crate::room::withdraw`], which turns it off for the room first.
pub fn withdraw(data_dir: &Path, name: &LendName) -> Result<bool, LendError> {
    let _held = lock_lend(data_dir, name, true)?;
    let slot = lock_path(data_dir, name, "rev");
    advance(&slot, revision_at(&slot)?)?;
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
