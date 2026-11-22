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

use axum::http::request::Builder;
use hyper::client::conn::http1::SendRequest;

/// What a service answered an ask, its status and body; none when it didn't.
type Answered = Option<(u16, Vec<u8>)>;

/// HTTP/1 on one connection, each request sent on it in turn ([`sent`]).
pub(crate) type On = SendRequest<String>;

use crate::door::bound::{Bound, Unsent};
use crate::lend::SendRow;
use crate::lend::{
    self, Answer, Approved, Audience, Lend, LendError, LendName, Program, Raw, RefusedPath, Source,
    Upload,
};
use crate::uid::{Answering, Unread};
use eudoxia_community::binding::{self, Chosen, Held, Refused, Took};

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
/// - `sends: <method> <path>`: where an app its person opens on their own
///   computer may send to it ([`SendRow`]), kept in the lend only on its
///   lender's yes ([`sends_of`]; CONTRACTS §2, "The lend is the yes").
///
/// A group's own `programs/<word>.txt`, in the same form, is read after
/// these when `share` lends under its word ([`fits`], [`says`]; CONTRACTS §2).
/// Its first column is not read, `-` or a port alike: the port's hint
/// ([`well_known_kind`]) names the word when its lender types none, and a
/// group's file is found by that word, read through the running svrngs one
/// group and word at a time, so it can't be found by a port before the word
/// is known; and a hint from it would let one member's file name what runs
/// on another's computer, where the word a lend goes under is its lender's.
///
/// Data, not code, since a program's word is one the infra does not hold
/// (DESIGN §5, the nouns gate).
pub const PROGRAMS: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/programs.rs"));

/// How a kind [`PROGRAMS`] names signs each person in, and what its
/// lender does for it, in their words: either way it is lent with no
/// password, since one sent on each request makes everyone one person to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Signs<'a> {
    /// It keeps its own accounts: how its lender makes a person one.
    Accounts(&'a str),
    /// It takes the person from the door's `x-svrngs-user`: what its
    /// lender sets in it.
    Header(&'a str),
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
const CLAUSES: [&str; 6] = ["keeps", "readers", "carries", "accounts", "header", "sends"];

/// A group's own `programs/<word>.txt` that [`fits`], empty for none: kept
/// as long as the terms svrngs ships, by the one `share` that reads it.
pub type Terms = &'static str;

/// The header a readers' key goes in, how, `{}` the key, and the path it
/// is tried at as it is pasted, empty for none ([`key_of`], [`tried`]).
pub type Key<'a> = Option<(&'a str, &'a str, &'a str)>;

/// Whether `text`, a group's own `programs/<word>.txt`, holds as terms for
/// `word` (CONTRACTS §2, §6.2, §6.3), or what in it does not, in words its
/// writer can act on: each line a row of `word`, its clause one of
/// [`CLAUSES`] and said the way its reader reads it. For a word svrngs ships
/// terms for, `keeps:` and `sends:` alone, so a group's file only adds to
/// what ships and no member opens `/slots` by writing a shorter file. A
/// `readers:` holds the `{}` its credential goes in, its sign-in a path on
/// the lender's own service ([`readers_in`]). A `sends:` is one [`SendRow`],
/// to a path no `keeps:` holds back. `carries:` is never beside `keeps:`,
/// `readers:` or `sends:`: a lend of bytes has no door to hold them to.
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
            c if !CLAUSES.contains(&c) => "is no term svrngs holds anyone to: those are keeps:, readers:, sends:, carries:, accounts: and header:",
            c if shipped && !["keeps", "sends"].contains(&c) => "changes the terms svrngs ships for it: a group's file may only add keeps: paths and sends: rows",
            _ if what.is_empty() => "says nothing after its colon",
            "keeps" if what.split_whitespace().any(|p| RefusedPath::parse(p).is_err()) => "keeps something that is not a path: each starts with /",
            "readers" if key_in(what).is_none() && readers_in(what).is_none() => "is neither `key <header>` nor a sign-in on its own service: a path starting with /, and {} where its token goes",
            "carries" if !matches!(what, "tcp" | "udp") => "carries tcp or udp, nothing else",
            "sends" if SendRow::parse(what).is_err() => "sends one method that writes, POST, PUT, PATCH or DELETE, to one exact path: `sends: POST /v1/embeddings`",
            _ => {
                said.push(clause);
                continue;
            }
        };
        tracing::debug!(target: "share", %word, row = how, why, "share: a group's programs file refused, its row named");
        return Err(format!("`{kind} {how}` {why}"));
    }
    let door = ["keeps", "readers", "sends"];
    let bytes = said.contains(&"carries") && said.iter().any(|c| door.contains(c));
    let kept = kept_of(word, text);
    let hidden = sends_of(word, text)
        .into_iter()
        .find(|s| kept.iter().any(|k| k.covers(s.path())));
    tracing::debug!(target: "share", %word, ?said, bytes, ?hidden, "share: a group's programs file, each row in the closed set, or carries: beside a door's term, or a send to a path it keeps");
    if let Some(row) = hidden {
        return Err(format!(
            "`{word} sends: {row}` sends to a path its keeps: holds back from everyone"
        ));
    }
    let mixed =
        format!("`carries:` lends {word} as bytes, with no door for keeps:, readers: or sends:");
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
    /// `--pid <pid>@<start>`: the process its lender chose on the port,
    /// which must hold every socket there, itself or a child ([`approve`]).
    pub pid: Option<Pid>,
    /// `--replace`: its name moves here from another port (SHARE_BINDING
    /// §5.8); without it a name shared from another port is refused.
    pub replace: bool,
}

/// A process its lender chose, by its id and its start, as `--pid
/// <pid>@<start>` names it: Linux's `/proc/<pid>/stat` field 22, a Mac's
/// start in microseconds ([`crate::uid::start`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pid {
    pub pid: u32,
    pub start: u64,
}

impl Pid {
    /// `<pid>@<start>`, both numbers, or none.
    pub fn parse(raw: &str) -> Option<Self> {
        let (pid, start) = raw.split_once('@')?;
        Some(Self {
            pid: pid.parse().ok()?,
            start: start.parse().ok()?,
        })
    }
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
    #[error("svrngs can't share port {0}: the program there doesn't run as you, and on a Mac svrngs can keep a share only to programs that run as you. Run it as you, then share it again.")]
    NotYours(NonZeroU16),
    #[error("svrngs can't share port {0} on this system: it can't read which program answers there, so it can't keep the share to it.")]
    NoReader(NonZeroU16),
    #[error("svrngs can't share port {0}: it can't read what answers there ({1}). Try again.")]
    Unread(NonZeroU16, &'static str),
    #[error("svrngs can't share port {0}: that program stopped; choose it again.")]
    Stopped(NonZeroU16),
    #[error("svrngs can't share port {0}: another program also listens on port {0}. Stop it, or choose that one, then share it again.")]
    AlsoListens(NonZeroU16),
    #[error("svrngs can't share port {0}: the program there listens only on {1}, and svrngs reaches a share at this computer's own address. Have it listen on 127.0.0.1, ::1 or every address, then share it again.")]
    Elsewhere(NonZeroU16, std::net::IpAddr),
    #[error("svrngs can't share UDP port {0}: svrngs sends a UDP share's datagrams to 127.0.0.1, and the program there doesn't listen on it. Have it listen on 127.0.0.1, or on every address, then share it again.")]
    UdpOnIpv6(NonZeroU16),
    #[error("svrngs can't share UDP port {0}: the program there listens on every address and lets other programs listen on the port beside it, so on a Mac svrngs can't keep the share to it alone. Have it listen on 127.0.0.1 for svrngs, then share it again.")]
    UdpBeside(NonZeroU16),
    #[error("svrngs can't share port {0}: what answers on port {0} changed; choose it again.")]
    Changed(NonZeroU16),
    #[error("{0} is already shared from {1}: give this one another name, or add --replace to share {0} from here instead.")]
    Taken(LendName, String),
    #[error("nothing is shared from a port here by the name {0}")]
    NotShared(LendName),
    #[error("{0} can't be changed on this computer: {1}")]
    Unkept(LendName, String),
}

/// The lend `name` declared in `data_dir`, and its port, as an approval
/// asks them (SHARE_BINDING §5.4); none for a carried app's.
pub fn shared(data_dir: &Path, name: &LendName) -> Result<(Lend, NonZeroU16), ShareRefusal> {
    let read = crate::lend::read(data_dir, name);
    let read = read.map_err(|e| ShareRefusal::Unkept(name.clone(), e.to_string()))?;
    let port = read.as_ref().and_then(Lend::port);
    read.zip(port)
        .ok_or_else(|| ShareRefusal::NotShared(name.clone()))
}

/// `approved` kept as the lend `name`'s, nothing else of it changed
/// (SHARE_BINDING §5.4).
pub fn reapproved(
    data_dir: &Path,
    name: &LendName,
    approved: Approved,
) -> Result<(), ShareRefusal> {
    let kept = crate::lend::mutate(data_dir, name, |l| l.terms.approved = Some(approved));
    tracing::debug!(target: "share", lend = %name, ?kept, "share: what answers on its port approved, only its approval changed");
    match kept {
        Ok(true) => Ok(()),
        Ok(false) => Err(ShareRefusal::NotShared(name.clone())),
        Err(e) => Err(ShareRefusal::Unkept(name.clone(), e.to_string())),
    }
}

/// What `read`, the `raw` share's port read ([`crate::uid::answering`]),
/// approves with `pid` when its lender chose a process (SHARE_BINDING §5.1,
/// [`eudoxia_community::binding::approve`]), and only what they were `shown`, when a
/// page showed it ([`token`]): the approval, and what its lender is shown
/// of each socket it covers, from the same read; or why not, in their
/// words. The one decider of whether an approval is taken: a lend's list
/// gives a token only where this takes it with none (door.rs `offered`).
pub fn approve(
    (port, raw): (NonZeroU16, Raw),
    read: Result<Answering, Unread>,
    pid: Option<Pid>,
    shown: Option<&str>,
) -> Result<(Approved, Vec<Answer>), ShareRefusal> {
    let read = read.map_err(|why| match why {
        Unread::NoReader => ShareRefusal::NoReader(port),
        Unread::Failed(why) => ShareRefusal::Unread(port, why),
    })?;
    let chosen = pid.map(|p| Chosen {
        pid: p.pid,
        start: p.start,
        now: read.start,
        parents: read.parents.clone(),
    });
    // What the lender is shown of each socket an approval covers, those the
    // check judges at the address it dials (each judged alone), or none when
    // one is shown nothing, which would be approved unseen: a Mac's whose
    // holder exited, or on Linux one of this account's whose program's files
    // can't be read.
    let covered = |a: &Approved| {
        let judged = |s: &binding::Socket| {
            let mut alone = read.read.clone();
            alone.sockets = vec![s.clone()];
            let on = binding::check(Some(a), &alone, &Took::Datagram) != Err(Held::Empty);
            on.then_some(s.id)
        };
        let ids: Vec<u64> = read.read.sockets.iter().filter_map(judged).collect();
        let of = read.shown.iter().filter(|(id, _)| ids.contains(id));
        let mut said: Vec<Answer> = of.clone().map(|(_, a)| a.clone()).collect();
        said.dedup();
        let seen = ids.iter().all(|id| of.clone().any(|(i, _)| i == id));
        seen.then_some(said)
    };
    let approved = match binding::approve(&read.read, chosen.as_ref()) {
        Err(Refused::Nothing) => Err(ShareRefusal::NothingListens(port)),
        Err(Refused::NotYours) => Err(ShareRefusal::NotYours(port)),
        Err(Refused::Stopped) => Err(ShareRefusal::Stopped(port)),
        Err(Refused::Another) => Err(ShareRefusal::AlsoListens(port)),
        Err(Refused::Elsewhere(at)) => Err(ShareRefusal::Elsewhere(port, at)),
        Ok(a) if raw == Raw::Udp && a.at.is_ipv6() => Err(ShareRefusal::UdpOnIpv6(port)),
        Ok(_) if raw == Raw::Udp && read.beside => Err(ShareRefusal::UdpBeside(port)),
        Ok(a) if shown.is_some_and(|t| t != token(&a)) => Err(ShareRefusal::Changed(port)),
        Ok(a) => covered(&a)
            .map(|said| (a, said))
            .ok_or(ShareRefusal::Unread(port, "its program can't be read")),
    };
    tracing::debug!(target: "share", port, ?raw, ?pid, ?shown, read = ?read.read, answers = ?read.shown, ?approved, "share: what answers on the port, approved with what each socket it covers is shown as, or why not");
    approved
}

/// An opaque name for what `approved` approves, the same for the same
/// sockets read and another once they differ, a hold kept since aside: what
/// a page shows answering, which approving it sends back, so what changed
/// since is refused (SHARE_BINDING_REVIEWS §6 #6).
pub fn token(approved: &Approved) -> String {
    let shown = Approved {
        held: false,
        ..approved.clone()
    };
    let bytes = serde_json::to_vec(&shown).expect("an approval serializes");
    hex::encode(ring::digest::digest(&ring::digest::SHA256, &bytes))
}

/// Whether `lend` may be declared over what `data_dir` holds by its name
/// (SHARE_BINDING §5.8): none, one of its port and protocol, or with
/// `replace`; a share of that name from another, or one that can't be read,
/// is refused, so a friend's state for the name never moves to another
/// service unasked.
pub fn renamed(data_dir: &Path, lend: &Lend, replace: bool) -> Result<(), ShareRefusal> {
    let had = crate::lend::read(data_dir, &lend.name);
    let had = had.map_err(|e| ShareRefusal::Unkept(lend.name.clone(), e.to_string()))?;
    let at = |l: &Lend| match l.source {
        Source::Port(port) => Some(format!("port {port}")),
        Source::Raw(raw, port) => Some(format!("{} port {port}", raw.said())),
        Source::Carried(..) => None,
    };
    let elsewhere = (had.as_ref().and_then(at)).filter(|had| Some(had) != at(lend).as_ref());
    tracing::debug!(target: "share", lend = %lend.name, ?elsewhere, replace, "share: a name already shared from another port, moved only when asked");
    match elsewhere {
        Some(at) if !replace => Err(ShareRefusal::Taken(lend.name.clone(), at)),
        _ => Ok(()),
    }
}

/// What a lender is told when svrngs sends `name` nothing of theirs, a key
/// or an account, because of `unsent` ([`crate::door::bound`]).
pub fn unsent_words(name: &LendName, unsent: Unsent) -> String {
    let why = match unsent {
        Unsent::Away => "nothing answers on its port",
        Unsent::Held(_) => {
            "what answers on its port isn’t what you approved: Keep sharing it first"
        }
    };
    format!("svrngs sent nothing to {name}: {why}.")
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
fn says<'a>(kind: &Program, word: &'a str, terms: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    let shipped = PROGRAMS.iter().map(|&(_, f)| -> &'a str { f });
    let rows = shipped.chain([terms]).flat_map(|f| rows_in(f).flatten());
    let kind = kind.clone();
    rows.filter_map(move |(_, k, how)| {
        let (what, rest) = how.split_once(':')?;
        (k == kind && what == word).then_some(rest.trim())
    })
}

/// The paths [`PROGRAMS`] and `terms` ([`says`]) keep back of every lend of
/// `kind`, its lender typing none; none when no row says `keeps:`. Why each
/// is kept back is the file's, beside its row.
pub fn kept_of(kind: &Program, terms: &str) -> Vec<RefusedPath> {
    let paths = says(kind, "keeps", terms).flat_map(str::split_whitespace);
    let parsed = |p: &str| RefusedPath::parse(p).expect("a file's paths start with /, as it fits");
    let kept: Vec<RefusedPath> = paths.map(parsed).collect();
    tracing::debug!(target: "share", kind = %kind, kept = kept.len(), group = !terms.is_empty(), "share: the paths the files keep back of the kind");
    kept
}

/// The `sends:` rows [`PROGRAMS`] and `terms` ([`says`]) say for `kind`:
/// where an app may send to a lend of it once its lender says yes, which
/// [`lend_of`] keeps and `svrngs share` asks at a terminal.
pub fn sends_of(kind: &Program, terms: &str) -> Vec<SendRow> {
    says(kind, "sends", terms)
        .filter_map(|s| SendRow::parse(s).ok())
        .collect()
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
pub fn accounts_of<'a>(kind: &Program, terms: &'a str) -> Option<Signs<'a>> {
    let accounts = says(kind, "accounts", terms).next().map(Signs::Accounts);
    let signs = accounts.or_else(|| says(kind, "header", terms).next().map(Signs::Header));
    tracing::debug!(target: "share", kind = %kind, ?signs, "share: how the kind signs each person in, if its files say");
    signs
}

/// Whether its lender may give headers its door adds to each request `lend`
/// is asked, as `svrngs share` asks for an Authorization and its row on the
/// home page for any: not when it signs people in with svrngs or itself
/// ([`accounts_of`]), where one header sent for all made everyone one
/// person, nor for a raw lend, which carries the person's own.
pub fn adds_headers(lend: &Lend, terms: &str) -> bool {
    let signs = lend.terms.sign_in_with_svrngs.is_some() || matches!(lend.source, Source::Raw(..));
    let signs = signs || accounts_of(&lend.kind, terms).is_some();
    tracing::debug!(target: "share", lend = %lend.name, signs, "share: headers its door adds asked of its lender, unless it signs people in or is raw");
    !signs
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
pub fn readers_of<'a>(kind: &Program, terms: &'a str) -> Option<ReadersSignIn<'a>> {
    says(kind, "readers", terms).next().and_then(readers_in)
}

/// [`ReadersSignIn`] as a `readers:` row says it, if it parses, its
/// `<sent>` holds the `{}` the token goes in, and its sign-in is a path on
/// the lender's own service: [`sign_in_readers`] writes it after the port,
/// so `@evil.example/x` would post the typed password to evil.example
/// (CONTRACTS §6.3).
fn readers_in(row: &str) -> Option<ReadersSignIn<'_>> {
    let asked = row.split_once(" | ");
    let (row, asked) = asked.map_or((row, None), |(r, a)| (r, Some(a)));
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
pub fn key_of<'a>(kind: &Program, terms: &'a str) -> Key<'a> {
    says(kind, "readers", terms).next().and_then(key_in)
}

/// The header, how (`{}` the key) and the path a `readers: key` row says,
/// `key <header> [<sent>] [<path>]`, if it [`key_header_fits`]; the one
/// reader of that form. Its last word, when it starts with `/`, is the path
/// on the lender's own service a key is tried at as it is pasted
/// ([`tried`]). A row naming a header it refuses is no row at all.
fn key_in(row: &str) -> Option<(&str, &str, &str)> {
    let row = row.strip_prefix("key ")?.trim();
    let at = |(_, p): &(&str, &str)| p.starts_with('/');
    let (row, path) = row.rsplit_once(' ').filter(at).unwrap_or((row, ""));
    let (header, sent) = row.split_once(' ').unwrap_or((row, "{}"));
    let fits = key_header_fits(header) && sent.contains("{}");
    tracing::debug!(target: "share", header, fits, "share: a readers' key row, read or refused");
    fits.then_some((header, sent, path))
}

/// Keep `key` for apps reading the lend `name`, as `how`, its program's
/// [`key_of`], says to send it ([`crate::lend::keep_readers`]); false when it names no
/// key or `key` is empty, so a blank paste can't delete what is kept.
pub fn keep_key(dir: &Path, name: &LendName, how: Key<'_>, key: &str) -> Result<bool, LendError> {
    let Some((header, sent, _)) = how.filter(|_| !key.is_empty()) else {
        return Ok(false);
    };
    crate::lend::keep_readers(dir, name, (header, &sent.replace("{}", key))).map(|()| true)
}

/// What the share's service answers `key` on `to`, its checked
/// connection, sent as `how` says, at its path, followed nowhere else: its
/// status; none when nothing answers. Asked once, as the key is pasted.
pub async fn tried(to: Bound, port: NonZeroU16, how: Key<'_>, key: &str) -> Option<u16> {
    let (header, sent, path) = how.filter(|(.., path)| !path.is_empty())?;
    let asked = axum::http::Request::get(path).header(header, sent.replace("{}", key));
    let answered = asked_on(to, port, asked, String::new()).await;
    let status = answered.map(|(s, _)| s);
    tracing::debug!(target: "share", port, path, ?status, "share: a pasted key tried once on its own service, or none answered");
    status
}

/// `asked`, with `body`, sent on `to` as HTTP/1 to the service at `port`:
/// its answer's status and body; none when it doesn't answer.
async fn asked_on(to: Bound, port: NonZeroU16, asked: Builder, body: String) -> Answered {
    sent(&mut http_on(to.stream()).await?, port.get(), asked, body).await
}

/// HTTP/1 on the one connection `to`, each request [`sent`] on it in turn:
/// a share's port's, and svrngs's own API's, whose ask follows its proof
/// on the same connection (`crate::api::asked`).
pub(crate) async fn http_on(to: impl crate::iroh::Duplex + 'static) -> Option<On> {
    let io = hyper_util::rt::TokioIo::new(to);
    let (sender, conn) = hyper::client::conn::http1::handshake(io).await.ok()?;
    tokio::spawn(conn);
    Some(sender)
}

/// `asked`, with `body`, sent on `on` to the service at `port`: its
/// answer's status and body; none when it doesn't answer.
pub(crate) async fn sent(on: &mut On, port: u16, asked: Builder, body: String) -> Answered {
    let asked = asked
        .header("host", format!("127.0.0.1:{port}"))
        .body(body)
        .ok()?;
    let answer = on.send_request(asked).await.ok()?;
    let status = answer.status().as_u16();
    let body = http_body_util::BodyExt::collect(answer.into_body())
        .await
        .ok()?;
    Some((status, body.to_bytes().to_vec()))
}

/// What a lender is told of the key their `name`, its program's word,
/// answered `status` to ([`tried`]), and whether it is kept: refused, 401 or 403, it
/// is not, so a mistyped paste never replaces a key that works, as a
/// readers' account that is not let in is not kept (home.rs `readers`).
pub fn key_said(name: &Program, status: Option<u16>) -> Result<String, String> {
    match status {
        Some(s @ (401 | 403)) => Err(format!("{name} refused this key: it answered {s}. Make a new API key in {name}, then paste that.")),
        Some(s) if s < 300 => Ok(format!("{name} took this key.")),
        Some(s) => Ok(format!("{name} answered {s} to this key, so svrngs keeps it, not knowing it works.")),
        None => Ok(format!("{name} didn't answer, so svrngs keeps the key, not knowing it works.")),
    }
}

/// Sign in to the share's service as `name` on `to`, its checked
/// connection, as `how` says, and say what an app reading it is let in
/// with; or why not, the password never in it.
pub async fn sign_in_readers(
    to: Bound,
    port: NonZeroU16,
    how: ReadersSignIn<'_>,
    name: &str,
    password: &str,
) -> Result<String, String> {
    let body = serde_json::json!({ how.name: name, how.password: password });
    let mut ask = axum::http::Request::post(how.path).header("content-type", "application/json");
    if let Some(asked) = how.asked {
        ask = ask.header("authorization", asked);
    }
    let answered = asked_on(to, port, ask, body.to_string()).await;
    let (status, answer) = answered.ok_or_else(|| "it did not answer".to_string())?;
    tracing::debug!(target: "share", port, path = how.path, status, "share: the readers' account asked to sign in");
    if !(200..300).contains(&status) {
        return Err(format!(
            "it did not take that name and password (it answered {status})"
        ));
    }
    let answer: serde_json::Value = serde_json::from_slice(&answer).unwrap_or_default();
    let token = answer[how.token].as_str();
    tracing::debug!(target: "share", port, path = how.path, given = token.is_some(),
        "share: the readers' account signed in, its token in the answer or not");
    let token = token.ok_or_else(|| format!("its answer has no {}", how.token))?;
    Ok(how.sent.replace("{}", token))
}

/// Whether something on this computer accepts a TCP connection on `port`:
/// upstream's `probe_origin`, over each port [`listening`] asks.
#[expect(clippy::disallowed_methods, reason = "dial: unwritten probe")]
fn answers(port: &NonZeroU16) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port.get()));
    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok()
}

/// The ports something on this computer listens on, each probed as
/// [`answers`] probes one, a thread to each thousand (v1ui-lend (b): the
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

/// The lend `share` declares, given the kind its port is well known for,
/// before what answers there is approved ([`approve`]), which refuses a
/// port nothing answers on: the kind given, else the port's; the
/// name given, else the kind ([`kind_of`]); refused when `terms`, the
/// groups' own (CONTRACTS §2), carry it as other than it is lent; the
/// limit given, else none, which lets in
/// everyone in its groups at once (bash-limit: no number nobody chose;
/// `share` says so); the paths refused, those given and those the table
/// and `terms` keep back of the kind, copied into the lend, so its door
/// reads only its own lend's file and a file changed after changes nothing
/// until it is lent again (CONTRACTS §6.1); and where its terms let apps
/// send to it ([`sends_of`]), a page's alone, which its caller keeps only
/// on its lender's yes (CONTRACTS §2).
pub fn lend_of(
    share: &Share,
    well_known: Option<Program>,
    terms: &str,
) -> Result<Lend, ShareRefusal> {
    // The table's ports are a page's.
    let raw = share.raw;
    let Some(kind) = kind_of(share, well_known) else {
        tracing::debug!(target: "share", port = share.port, "share: refused — no kind given, and the port is not a well-known one");
        return Err(ShareRefusal::NoKind(share.port));
    };
    let name = (share.name.clone()).unwrap_or_else(|| {
        LendName::parse(kind.as_str()).expect("a kind is a lend's name, by one rule")
    });
    // The word as `fits` holds it, one of these two, kept as long as the refusal.
    let carries = ["tcp", "udp"]
        .into_iter()
        .find(|w| says(&kind, "carries", terms).any(|c| c == *w));
    if carries.is_some_and(|c| raw.is_none_or(|r| !r.said().eq_ignore_ascii_case(c))) {
        tracing::debug!(target: "share", kind = %kind, carries, "share: refused — its terms carry it as bytes, and it is lent otherwise");
        return Err(ShareRefusal::Carries(kind, carries.unwrap_or_default()));
    }
    let mut refused = share.refused.clone();
    for path in kept_of(&kind, terms).into_iter().filter(|_| raw.is_none()) {
        if !refused.contains(&path) {
            refused.push(path);
        }
    }
    let sends = sends_of(&kind, terms).into_iter().filter(|_| raw.is_none());
    let sends: Vec<SendRow> = sends.collect();
    tracing::debug!(target: "share", lend = %name, kind = %kind, port = share.port, limit = ?share.limit, refused = refused.len(),
        named = share.name.is_some(), kind_given = share.kind.is_some(), limit_given = share.limit.is_some(), sends = sends.len(),
        "share: the lend decided");
    Ok(Lend {
        name,
        source: raw.map_or(Source::Port(share.port), |r| Source::Raw(r, share.port)),
        kind,
        terms: lend::Terms {
            audience: share.audience.clone(),
            limit: share.limit,
            refused,
            upload: share.upload,
            reads: share.reads.clone(),
            asks: share.asks.clone(),
            sends,
            ..lend::Terms::default()
        },
    })
}
