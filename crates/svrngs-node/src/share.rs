// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/mesh_media/offer.rs@e1a43bcd2
//! What a lend is decided from (DESIGN §3, "Lent apps"): the probe, the
//! well-known table and the decision `svrngs share` made of them, moved
//! here from the command (v1ui-lend (a)), behaviour kept, so the home page
//! asks them too and the node never depends on the command. `probe_origin`
//! came from `svrn mesh media offer` (offer.rs:94-110); the command keeps
//! `resolve_target`, the verb's shape, and asks the rest here.
//!
//! svrngs: then changed (u3-5). What is shared is a port alone, on
//! 127.0.0.1, which is all a lend's declaration holds and all the door
//! dials ([`crate::lend`]); upstream's loopback `host:port` goes. The
//! probe asks one port, the one given, and the well-known ports name the
//! kind of what listens there, from a table kept as data
//! ([`PROGRAMS`]), so a lend of a service svrngs recognises needs no
//! more than its port. [`lend_of`] decides the lend from what was typed and
//! what the probe saw, and touches nothing.
//!
//! u5-4: a kind the table says keeps its own accounts ([`accounts_of`]) is
//! lent with no password, and its lender told how to make each person an
//! account in it (DESIGN §3, "Lent apps", call 4: sign in once).
//! bash-person (d): so is a kind that takes the person from the door's
//! header, its lender told what to set in it ([`Signs::Header`]).
//!
//! oidc-hook-cut: the sign-in a lender ran itself is gone, and with it
//! `--sign-in`, a helper's port and its steps; sign-in with svrngs is the
//! door's own (SIGN_IN_WITH_SVRNGS §5). oidc-retire removed the last
//! `helper` row with its app, so a row is a kind at a port or by its name
//! alone, and nothing names a program beside a lend.
//!
//! start-llama: a kind the table says keeps paths back ([`kept_of`]) is lent
//! without them, its lender typing none; and a kind known by its lender's
//! word alone has no port in the table, since one service's default port is
//! often another's.

use std::num::NonZeroU16;
use std::path::Path;
use std::time::Duration;

use crate::lend::{Audience, Lend, LendError, LendName, Program, Raw, RefusedPath, Source, Upload};

/// The programs svrngs ships terms for, one file each under `src/programs/`,
/// `<word>.txt`, so adding a program is adding a file; each holds that word's
/// rows and no other's, refused at build (build.rs), and they are read in
/// file-name order, so which row comes first is not the filesystem's to say.
///
/// A row is `<port> <word>`, `-` for its port when it is known by its
/// lender's word alone, then how it signs each person in, if svrngs knows
/// (`Signs`), and `share` lends it with no password and prints the rest of
/// the line, in DESIGN §1's words; `#` starts a comment, and each row's
/// evidence is the comment above it. The rest of a row is a clause:
///
/// - `accounts: <how>`: it keeps its own accounts; how its lender makes one.
/// - `header: <what>`: it takes the person from the header svrngs's door
///   sets, `x-svrngs-user` (door.rs, `USER_HEADER`, an account's name unique
///   by the person's key); what its lender sets in it for that.
/// - `readers: <how>`: how `share --readers` signs in once as the account
///   apps reading it are let in as (`ReadersSignIn`), or `key <header>
///   [<sent>]`, a key its lender pastes, `{}` in `<sent>` the key (`key_of`).
/// - `keeps: <path>...`: the paths of it svrngs keeps from everyone it is
///   lent to, its lender typing none (`share` adds them to its `--refuse`
///   paths and says so): why, in the comment above it.
/// - `carries: tcp` or `udp`: lent as bytes, not as a page; `share` refuses
///   to lend it otherwise.
///
/// A group's own `programs/<word>.txt`, in the same form, is read after
/// these when `share` lends under its word ([`fits`], [`says`]; CONTRACTS §2).
///
/// Data, not code, since a program's word is one the infra does not hold
/// (DESIGN §5, the nouns gate).
pub const PROGRAMS: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/programs.rs"));

/// How a kind [`PROGRAMS`] names signs each person in, and what its
/// lender does for it, in their words: either way it is lent with no
/// password, since one sent on each request makes everyone one person to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Signs {
    /// It keeps its own accounts: how its lender makes a person one.
    Accounts(&'static str),
    /// It takes the person from the door's `x-svrngs-user`: what its
    /// lender sets in it.
    Header(&'static str),
}

/// What [`PROGRAMS`]'s first column says a row is (REVIEW_FINDINGS #169):
/// a kind at its well-known port, or a kind known by its lender's word
/// alone (`-`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Port(NonZeroU16),
    Named,
}

/// [`PROGRAMS`]'s rows: the first column ([`At`]) and the kind, each
/// parsed once for every reader (REVIEW_FINDINGS #64, #169), and the rest
/// of the line, empty when there is none. A row whose port or kind does
/// not parse is none.
pub fn rows() -> impl Iterator<Item = (At, Program, &'static str)> {
    PROGRAMS.iter().flat_map(|&(_, f)| rows_in(f).flatten())
}

/// The rows of a program's file, `text`, as [`rows`] reads [`PROGRAMS`]'s
/// and [`fits`] a group's: each parsed ([`row`]), or the line that is not one.
pub fn rows_in(text: &str) -> impl Iterator<Item = Result<(At, Program, &str), &str>> {
    let lines = text.lines().map(|l| l.split('#').next().unwrap_or(""));
    let lines = lines.map(str::trim).filter(|l| !l.is_empty());
    lines.map(|l| row(l).ok_or(l))
}

/// A row's first column, its kind and the rest, if its port or `-` and its
/// kind parse.
fn row(l: &str) -> Option<(At, Program, &str)> {
    let (port, rest) = l.split_once(char::is_whitespace)?;
    let rest = rest.trim_start();
    let (kind, how) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let at = match port {
        "-" => At::Named,
        p => At::Port(p.parse().ok()?),
    };
    Some((at, Program::parse(kind).ok()?, how.trim()))
}

/// The clauses a program's file may hold (CONTRACTS §2), the one list: a
/// set closed, so a term svrngs can't enforce can't be written ([`fits`]).
const CLAUSES: [&str; 5] = ["keeps", "readers", "carries", "accounts", "header"];

/// A group's own `programs/<word>.txt` that [`fits`], empty for none: kept
/// as long as the terms svrngs ships, by the one `share` that reads it.
pub type Terms = &'static str;

/// The header a readers' key goes in, and how, `{}` the key ([`key_of`]).
pub type Key = Option<(&'static str, &'static str)>;

/// Whether `text`, a group's own `programs/<word>.txt`, holds as terms for
/// `word` (CONTRACTS §2, §6.2, §6.3), or what in it does not, in words its
/// writer can act on: each line a row of `word`, its clause one of
/// [`CLAUSES`] and said the way its reader reads it. For a word svrngs ships
/// terms for, `keeps:` alone, so a group's file only adds to what ships and
/// no member opens `/slots` by writing a shorter file. A `readers:` holds
/// the `{}` its credential goes in, its sign-in a path on the lender's own
/// service ([`readers_in`]). `carries:` is never beside `keeps:` or
/// `readers:`: a lend of bytes has no door to hold them to.
pub fn fits(word: &Program, text: &str) -> Result<(), String> {
    let shipped = rows().any(|(_, k, _)| k == *word);
    let mut said = Vec::new();
    for row in rows_in(text) {
        let shape = |l| format!("`{l}` is not a row: write one as `- {word} keeps: /path`");
        let (_, kind, how) = row.map_err(shape)?;
        let (clause, what) = how.split_once(':').unwrap_or((how, ""));
        let what = what.trim();
        let why = match clause {
            _ if kind != *word => "is another program's",
            c if !CLAUSES.contains(&c) => "is no term svrngs holds anyone to: those are keeps:, readers:, carries:, accounts: and header:",
            c if shipped && c != "keeps" => "changes the terms svrngs ships for it: a group's file may only add keeps: paths",
            _ if what.is_empty() => "says nothing after its colon",
            "keeps" if what.split_whitespace().any(|p| RefusedPath::parse(p).is_err()) => "keeps something that is not a path: each starts with /",
            "readers" if key_in(what).is_none() && readers_in(what).is_none() => "is neither `key <header>` nor a sign-in on its own service: a path starting with /, and {} where its token goes",
            "carries" if !matches!(what, "tcp" | "udp") => "carries tcp or udp, nothing else",
            _ => {
                said.push(clause);
                continue;
            }
        };
        tracing::debug!(target: "share", %word, row = how, why, "share: a group's programs file refused, its row named");
        return Err(format!("`{kind} {how}` {why}"));
    }
    let bytes = said.contains(&"carries") && said.iter().any(|c| ["keeps", "readers"].contains(c));
    tracing::debug!(target: "share", %word, ?said, bytes, "share: a group's programs file, each row in the closed set, or carries: beside a door's term");
    let mixed = format!("`carries:` lends {word} as bytes, with no door for keeps: or readers:");
    (!bytes).then_some(()).ok_or(mixed)
}

/// What `svrngs share` was told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share {
    pub port: NonZeroU16,
    pub name: Option<LendName>,
    pub kind: Option<Program>,
    pub limit: Option<NonZeroU16>,
    pub refused: Vec<RefusedPath>,
    /// This computer's upload, as its lender states it; none stated, none
    /// declared (u3-6).
    pub upload: Option<Upload>,
    /// The kinds of lend its pages read (u5-2).
    pub reads: Vec<Program>,
    /// What an app receives of it, in its lender's words (sup-2).
    pub asks: Option<String>,
    /// `--sign-in-with-svrngs`: it signs each person in as themselves, with
    /// nothing typed (SIGN_IN_WITH_SVRNGS §5 "Turning it on").
    pub sign_in: bool,
    /// `--readers`: the account apps reading it are let in as, asked for
    /// and kept (`lend::readers_dir`).
    pub readers: bool,
    /// `--udp` or `--tcp`: lent as its own datagrams or bytes, not as a
    /// page (give-udp, give-tcp).
    pub raw: Option<Raw>,
    /// The groups it is lent to (LENDING_SCOPE): explicit stable IDs its
    /// lender chose, or [`Audience::Legacy`] when none was given, which is
    /// paused until selection, never all groups.
    pub audience: Audience,
}

/// Why a lend could not be decided, in a person's words.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ShareRefusal {
    #[error("nothing answers on port {0} on this computer: start it, then share it")]
    NothingListens(NonZeroU16),
    #[error("say which program is on port {0}: `svrngs share {0} --program <name>`, its own name if it is one svrngs knows: {kinds}", kinds = PROGRAMS.iter().map(|(word, _)| *word).collect::<Vec<_>>().join(", "))]
    NoKind(NonZeroU16),
    #[error("your group's programs/{0}.txt says {0} is lent as {1}, not as a page: `svrngs share <port> --{1} --program {0} --group <group>`")]
    Carries(Program, &'static str),
}

/// The kind [`PROGRAMS`] names for `port`, if any.
pub fn well_known_kind(port: NonZeroU16) -> Option<Program> {
    rows().find_map(|(at, kind, _)| (at == At::Port(port)).then_some(kind))
}

/// What its lender is told of a program name [`PROGRAMS`] has no row
/// for, `None` for one it has: a typo costs every protection the table
/// keeps for that program, so a person's mistake is said at share time,
/// never refused (any app on a port stays lendable). A name differing from
/// a row's only in case or by a slip names the row it likely meant.
pub fn unrecognized(kind: &Program) -> Option<String> {
    if rows().any(|(_, known, _)| known == *kind) {
        return None;
    }
    let near = rows().find(|(_, known, _)| slipped(known.as_str(), kind.as_str()));
    tracing::debug!(target: "share", kind = %kind, near = ?near.as_ref().map(|(_, k, _)| k.as_str()), "share: a program the table does not know, its lender told");
    let mean = near.map_or(String::new(), |(_, known, _)| {
        format!(" Did you mean {known}?")
    });
    Some(format!(
        "svrngs doesn't know {kind}, so your friends can reach every part of it that its own sign-in allows.{mean}"
    ))
}

/// Whether `said` is `known` with the case changed or a letter or two
/// slipped: what the two share, each end, leaves at most two letters over.
fn slipped(known: &str, said: &str) -> bool {
    let (known, said) = (known.to_ascii_lowercase(), said.to_ascii_lowercase());
    let (a, b) = (known.as_bytes(), said.as_bytes());
    let head = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let tail = a[head..].iter().rev().zip(b[head..].iter().rev());
    a.len().max(b.len()) - head - tail.take_while(|(x, y)| x == y).count() <= 2
}

/// What [`PROGRAMS`]'s rows for `kind`, then `terms`, its group's own file
/// that [`fits`] (empty for none), say in the clause `word`, past the `:`,
/// shipped first — the one reader of a clause, for [`accounts_of`],
/// [`kept_of`] and [`readers_of`]. A kind may have a row for each clause.
fn says(kind: &Program, word: &'static str, terms: Terms) -> impl Iterator<Item = &'static str> {
    let (kind, rows) = (kind.clone(), rows().chain(rows_in(terms).flatten()));
    rows.filter_map(move |(_, k, how)| {
        let (what, rest) = how.split_once(':')?;
        (k == kind && what == word).then_some(rest.trim())
    })
}

/// The paths [`PROGRAMS`] and `terms` ([`says`]) keep back of every lend of
/// `kind`, its lender typing none; none when no row says `keeps:`. Why each
/// is kept back is the file's, beside its row.
pub fn kept_of(kind: &Program, terms: Terms) -> Vec<RefusedPath> {
    let paths = says(kind, "keeps", terms).flat_map(str::split_whitespace);
    let parsed = |p: &str| RefusedPath::parse(p).expect("a file's paths start with /, as it fits");
    let kept: Vec<RefusedPath> = paths.map(parsed).collect();
    tracing::debug!(target: "share", kind = %kind, kept = kept.len(), group = !terms.is_empty(), "share: the paths the files keep back of the kind");
    kept
}

/// How a service of `kind` signs each person in, if [`PROGRAMS`] or
/// `terms` say ([`says`]):
/// with accounts of its own, each signing in once with one its lender made,
/// or from the door's header. Either is lent with no password: one sent on
/// each request makes everyone one person to it, so one person's sign-in
/// ended everyone's, and its token reached their browsers
/// (`demo/runs/a/dry-2026-10-04/README.md:39-50`;
/// `demo/runs/u5/2026-10-05-jellyfin/README.md`, finding 3). A row saying
/// neither `accounts:` nor `header:` says nothing.
pub fn accounts_of(kind: &Program, terms: Terms) -> Option<Signs> {
    let accounts = says(kind, "accounts", terms).next().map(Signs::Accounts);
    let signs = accounts.or_else(|| says(kind, "header", terms).next().map(Signs::Header));
    tracing::debug!(target: "share", kind = %kind, ?signs, "share: how the kind signs each person in, if its files say");
    signs
}

/// How [`PROGRAMS`] says a service trades the account its lender keeps
/// for readers, a name and a password, for what an app reading it is let in
/// with (`svrngs share --readers`, `lend::readers_dir`). Its row:
/// `readers: <path> <name field> <password field> <token field> <sent>`,
/// `{}` in `<sent>` the token, then ` | <Authorization>` if the sign-in
/// wants one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadersSignIn<'a> {
    pub path: &'a str,
    pub name: &'a str,
    pub password: &'a str,
    pub token: &'a str,
    pub sent: &'a str,
    pub asked: Option<&'a str>,
}

/// [`ReadersSignIn`] for `kind`, if its files' row says one that
/// [`readers_in`] reads.
pub fn readers_of(kind: &Program, terms: Terms) -> Option<ReadersSignIn<'static>> {
    says(kind, "readers", terms).next().and_then(readers_in)
}

/// [`ReadersSignIn`] as a `readers:` row says it, if it parses, its
/// `<sent>` holds the `{}` the token goes in, and its sign-in is a path on
/// the lender's own service: [`sign_in_readers`] writes it after the port,
/// so `@evil.example/x` would post the typed password to evil.example
/// (CONTRACTS §6.3).
fn readers_in(row: &str) -> Option<ReadersSignIn<'_>> {
    let (row, asked) = row
        .split_once(" | ")
        .map_or((row, None), |(r, a)| (r, Some(a)));
    let words: Vec<&str> = row.splitn(5, ' ').collect();
    let [path, name, password, token, sent] = words.try_into().ok()?;
    let fits = path.starts_with('/') && sent.contains("{}");
    tracing::debug!(target: "share", path, fits, "share: a readers' sign-in row, read or refused");
    fits.then_some(ReadersSignIn {
        path,
        name,
        password,
        token,
        sent,
        asked,
    })
}

/// Whether a `readers: key` row may name `header`. The door keeps its own
/// (`crate::door::IDENTITY_PREFIX`, its hop-by-hop list, RFC 9110 §7.6.1), and
/// the person's cookie, the address and the length are never a lend's secret.
pub fn key_header_fits(header: &str) -> bool {
    let name = header.to_ascii_lowercase();
    let door = name.starts_with(crate::door::IDENTITY_PREFIX)
        || crate::door::HOP_BY_HOP.contains(&name.as_str());
    crate::declared::valid_header_name(&name)
        && !door
        && !matches!(name.as_str(), "cookie" | "host" | "content-length")
}

/// The header a key its lender pastes goes in, and how (`{}` the key), if
/// `kind`'s files' `readers:` row is a `key` row [`key_in`] reads.
pub fn key_of(kind: &Program, terms: Terms) -> Key {
    says(kind, "readers", terms).next().and_then(key_in)
}

/// The header and how (`{}` the key) a `readers: key` row says, if it
/// [`key_header_fits`]; the one reader of that form. A row naming a header
/// it refuses is no row at all.
fn key_in(row: &str) -> Option<(&str, &str)> {
    let row = row.strip_prefix("key ")?.trim();
    let (header, sent) = row.split_once(' ').unwrap_or((row, "{}"));
    let fits = key_header_fits(header) && sent.contains("{}");
    tracing::debug!(target: "share", header, fits, "share: a readers' key row, read or refused");
    fits.then_some((header, sent))
}

/// Keep `key` for apps reading the lend `name`, as `how`, its program's
/// [`key_of`], says to send it ([`crate::lend::keep_readers`]); false when it names no
/// key or `key` is empty, so a blank paste can't delete what is kept.
pub fn keep_key(dir: &Path, name: &LendName, how: Key, key: &str) -> Result<bool, LendError> {
    let Some((header, sent)) = how.filter(|_| !key.is_empty()) else {
        return Ok(false);
    };
    crate::lend::keep_readers(dir, name, (header, &sent.replace("{}", key))).map(|()| true)
}

/// Sign in to the service at this computer's `port` as `name`, as `how`
/// says, and say what an app reading it is let in with; or why not, the
/// password never in it.
pub async fn sign_in_readers(
    port: NonZeroU16,
    how: ReadersSignIn<'_>,
    name: &str,
    password: &str,
) -> Result<String, String> {
    let body = serde_json::json!({ how.name: name, how.password: password });
    let url = format!("http://127.0.0.1:{port}{}", how.path);
    let mut ask = crate::http::client()?.post(url).body(body.to_string());
    ask = ask.header("content-type", "application/json");
    if let Some(asked) = how.asked {
        ask = ask.header("authorization", asked);
    }
    let answer = ask
        .send()
        .await
        .map_err(|_| "it did not answer".to_string())?;
    let status = answer.status();
    tracing::debug!(target: "share", port, path = how.path, %status, "share: the readers' account asked to sign in");
    if !status.is_success() {
        return Err(format!(
            "it did not take that name and password (it answered {status})"
        ));
    }
    let answer = answer.bytes().await.map_err(|e| e.to_string())?;
    let answer: serde_json::Value = serde_json::from_slice(&answer).unwrap_or_default();
    let token = answer[how.token].as_str();
    tracing::debug!(target: "share", port, path = how.path, given = token.is_some(),
        "share: the readers' account signed in, its token in the answer or not");
    let token = token.ok_or_else(|| format!("its answer has no {}", how.token))?;
    Ok(how.sent.replace("{}", token))
}

/// Whether something on this computer accepts a TCP connection on `port`.
/// (svrngs: upstream's `probe_origin` over the one port given.)
pub fn listens(port: NonZeroU16) -> bool {
    let up = answers(&port);
    tracing::debug!(target: "share", port, up, "share: probed the port");
    up
}

/// [`listens`], untraced, for [`listening`]'s thousands.
fn answers(port: &NonZeroU16) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port.get()));
    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok()
}

/// The ports something on this computer listens on, each probed as
/// [`listens`] probes one, a thread to each thousand (v1ui-lend (b): the
/// home page's "Already running here"). From 1024, below which only the
/// system's own services listen unless it is told otherwise, to 49151,
/// past which it hands ports to its own connections (IANA's dynamic
/// range, RFC 6335 §6), so what is offered is what a person started.
pub fn listening() -> Vec<NonZeroU16> {
    let ports: Vec<NonZeroU16> = (1024..=49151).filter_map(NonZeroU16::new).collect();
    let found: Vec<NonZeroU16> = std::thread::scope(|s| {
        let asked: Vec<_> = (ports.chunks(1000))
            .map(|c| s.spawn(|| c.iter().copied().filter(answers).collect::<Vec<_>>()))
            .collect();
        asked
            .into_iter()
            .flat_map(|h| h.join().expect("a probe does not panic"))
            .collect()
    });
    tracing::debug!(target: "share", probed = ports.len(), ?found, "share: what listens on this computer");
    found
}

/// The kind a lend of `share` is of: the kind given, else the one its port
/// is well known for, `well_known`, a page's alone.
pub fn kind_of(share: &Share, well_known: Option<Program>) -> Option<Program> {
    let page = well_known.filter(|_| share.raw.is_none());
    share.kind.clone().or(page)
}

/// The lend `share` declares, given what `listens` says of its port, and
/// the kind the port is well known for: the kind given, else the port's; the
/// name given, else the kind ([`kind_of`]); refused when `terms`, the
/// groups' own (CONTRACTS §2), carry it as other than it is lent; the
/// limit given, else none, which lets in
/// everyone in its groups at once (bash-limit: no number nobody chose;
/// `share` says so); the paths refused, those given and those the table
/// and `terms` keep back of the kind, copied into the lend, so its door
/// reads only its own lend's file and a file changed after changes nothing
/// until it is lent again (CONTRACTS §6.1).
pub fn lend_of(
    share: &Share,
    listens: impl Fn(NonZeroU16) -> bool,
    well_known: Option<Program>,
    terms: Terms,
) -> Result<Lend, ShareRefusal> {
    // A UDP port answers no connection, so nothing tells whether a program
    // listens on it; a TCP port is probed as a page's is. The table's ports
    // are a page's.
    let raw = share.raw;
    if raw == Some(Raw::Udp) {
        tracing::debug!(target: "share", port = share.port, "share: a UDP port, lent unprobed");
    } else if !listens(share.port) {
        tracing::debug!(target: "share", port = share.port, "share: refused — nothing listens on the port");
        return Err(ShareRefusal::NothingListens(share.port));
    }
    let Some(kind) = kind_of(share, well_known) else {
        tracing::debug!(target: "share", port = share.port, "share: refused — no kind given, and the port is not a well-known one");
        return Err(ShareRefusal::NoKind(share.port));
    };
    let name = match &share.name {
        Some(n) => n.clone(),
        None => LendName::parse(kind.as_str()).expect("a kind is a lend's name, by one rule"),
    };
    let carries = says(&kind, "carries", terms).next();
    if carries.is_some_and(|c| raw.is_none_or(|r| !r.said().eq_ignore_ascii_case(c))) {
        tracing::debug!(target: "share", kind = %kind, carries, "share: refused — its terms carry it as bytes, and it is lent otherwise");
        return Err(ShareRefusal::Carries(kind, carries.unwrap_or_default()));
    }
    let limit = share.limit;
    let mut refused = share.refused.clone();
    for path in kept_of(&kind, terms).into_iter().filter(|_| raw.is_none()) {
        if !refused.contains(&path) {
            refused.push(path);
        }
    }
    tracing::debug!(target: "share", lend = %name, kind = %kind, port = share.port, ?limit, refused = refused.len(),
        named = share.name.is_some(), kind_given = share.kind.is_some(), limit_given = share.limit.is_some(),
        "share: the lend decided");
    Ok(Lend {
        name,
        source: raw.map_or(Source::Port(share.port), |r| Source::Raw(r, share.port)),
        kind,
        audience: share.audience.clone(),
        limit,
        refused,
        upload: share.upload,
        reads: share.reads.clone(),
        sign_in_with_svrngs: None,
        asks: share.asks.clone(),
    })
}
