// SPDX-License-Identifier: AGPL-3.0-or-later
//! The command surface (DESIGN §5; PLAN §4, U2's inventory, the cli.rs row):
//! verbs parsed by hand, no framework, in the shape of upstream's
//! `commonwealth/crates/commonwealth-rails/src/cli.rs` and `main.rs` at
//! `e1a43bcd2` — an error string for a wrong argument, never a guess; exit
//! codes 0, 1 and 3; a tokio runtime built by hand (`main.rs` here).
//!
//! ```text
//! svrngs                                   run for every group this computer holds
//! svrngs new <name> [--as N] [--origin U] [--relay R]...
//!                                          make a group, print its link, run
//! svrngs join <link> [--as N] [--no-run]   join a group by its link, run
//! svrngs link <group> [--origin U]
//!                                          a new link to a group, print it, run
//! svrngs share <port> --group G... [--name N] [--program P] [--pid I@S] [--replace] [--limit L] [--refuse P]... [--upload M] [--reads P]... [--ask W] [--sign-in-with-svrngs] [--readers] [--udp | --tcp]
//!                                          lend a service on this computer
//! svrngs unshare <name> [--group G]...     stop lending it, or to G only
//! svrngs approve <name> [--pid I@S] [--token T]
//!                                          say what answers now, and keep lending it
//!                                          to that on a y, or to what a list showed
//! svrngs share --folder <dir> --group <group> keep a folder the same on every member's disk,
//!                                          asked of svrngs running here
//! svrngs room <name>...                    open lends to this computer's WiFi,
//!                                          asked of svrngs running here, else run
//! svrngs remove <name> --group <group> [--joined W]
//!                                          remove someone from a group, or ask it,
//!                                          asked of svrngs running here
//! svrngs handover <name> --group <group> [--joined W]
//!                                          let someone else decide who is removed,
//!                                          asked of svrngs running here
//! svrngs accept <group>                    decide who is removed, handed to you,
//!                                          asked of svrngs running here
//! svrngs relay <name> [--open] [--staging] a relay, for the groups this computer is in
//!                                          or everyone ([`crate::relay`])
//! svrngs guide                             print the guide to building an app
//! svrngs check <folder>                    what in an app's page the policy refuses
//! ```
//!
//! `remove` and `handover` ask the node running here, as `share --folder`
//! does (U7; DESIGN §6 "Removal has one decider"; PLAN §3, 2026-10-09):
//! `svrngs_node::removal` commits them and answers what happened, worded
//! here. One person in a group decides who is removed from it — its maker
//! at first, until they hand that on — and anyone else's `remove` asks
//! them; the words name them. Of two people of one name, svrngs never
//! guesses which is meant: it says when each joined, which `--joined`
//! takes, as the home page lists them (bash-same-name).
//!
//! `share` and `unshare` write a lend's declaration and its secret into the
//! data dir (`svrngs_node::lend`), which the door reads on every request,
//! so each works whether or not svrngs is running here (u3-5). `share`
//! then runs, as the other verbs do, unless svrngs already is. The secret
//! is read from a prompt, never from the command line, where any program on
//! this computer may read it (`ps`); a service that keeps its own accounts
//! is lent with none, and its lender told how to make each person one
//! (u5-4, [`share::accounts_of`]); one that takes each person from the
//! door's header, none, and its lender told what to set in it
//! (bash-person (d)). The bare command prints what the people
//! of this computer's groups lend, asked of their computers as it starts.
//! If already running, it prints the home address and reminds how to open it.
//!
//! `room` turns the lends it names on for the room, served to this
//! computer's own network as this computer's person by svrngs running
//! here, which keeps them on across restarts on that network
//! (`svrngs_node::room`): asked as the home page asks it, of svrngs running
//! here, else of the svrngs it starts, so the room has one decider. It says
//! who can open them, and where each is served or why not.
//!
//! `new`, `join` and `link` are asked of svrngs running here, the one
//! writer of this computer's acts ([`asked`]), as `room` is: of the one
//! running already, and then the command says and leaves, running
//! nothing; else of the one it starts. Each is its route's
//! ([`svrngs_node::group::NEW_ROUTE`], [`svrngs_node::join::JOINS_ROUTE`],
//! [`svrngs_node::home::LINKS_ROUTE`]), so a verb is built once, there.
//! A `--origin` is checked here first ([`checked_page`]). One that answers
//! no join — a build from before the route — is told as too old, for
//! updating and restarting it is the way on from there.
//!
//! `new` and `link` always print the client's link, `svrngs://join/<token>`
//! (u2-8): nothing depends on a hosted page. A group may keep a page
//! origin, its setting on this computer (`group::write_origin`), and then
//! its https link is printed beside it: `new --origin` keeps one, `join`
//! keeps the one its https link was printed on and none from a client's
//! link, and `link --origin` changes it.
//!
//! A group with a page origin opens its apps on a phone at addresses apart
//! from the page's, which holds each person's key (u5-2; DESIGN §7): each
//! app at an address of its own under the page's host, which must serve the
//! page at every name under it. `new` and `link` check it by asking a name
//! nobody made for the page ([`checked_page`]), refuse a host that does not, saying
//! what it must do, and print where the apps open.
//!
//! What a person reads uses DESIGN §1's words alone: the group's name,
//! people's names, *link*, *computer*. The detail behind a refusal is
//! traced at debug, where any word may be used.
//!
//! # Exit codes
//!
//! `0` it worked · `1` it ran and refused (no such group, not in it, a
//! wrong argument) · `3` could-not-judge: a precondition of the run is
//! absent — no data dir, no readable key, no endpoint — so nothing was
//! attempted and nothing is claimed (ARCH 5).

use std::fs::File;
use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;

use std::num::NonZeroU16;

use svrngs_node::api::{ApiError, API_FILE};
use svrngs_node::door::bound::{Bound, Unsent};
use svrngs_node::folder::{FolderAsk, PROGRAMS_ROUTE};
use svrngs_node::home::WHEN;
use svrngs_node::home::{LinkMade, Linking, LINKS_ROUTE};
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::join::{JoinAsk, NotJoined, Outcome as JoinOutcome, JOINS_ROUTE};
use svrngs_node::lend::Answer;
use svrngs_node::lend::{
    self, Audience, Lend, LendName, Program, Raw, RefusedPath, SignInWithSvrngs, Source, Upload,
};
use svrngs_node::node::{Lends, Node, NodeError};
use svrngs_node::relay::n0_relays;
use svrngs_node::removal::{
    LeaveAsk, Outcome, RemovalAsk, ACCEPTS_ROUTE, HANDOVERS_ROUTE, LEAVES_ROUTE, REMOVALS_ROUTE,
};
use svrngs_node::room;
use svrngs_node::uid::{Answering, Unread, Withheld};
use svrngs_node::update;
use svrngs_node::{Joined, Person};

use crate::group::{self, Name, Seed};
use crate::identity::{self, KeyError};
use crate::relay::{self, Relay};
use crate::share::{self, Share, ShareRefusal, Terms};
use svrngs_node::link::{client_link, https_link, Link, Origin};
use svrngs_node::oidc::KeyFailure;

pub const USAGE: &str = include_str!("help.txt");

/// Where the record is served to this computer's apps the first time
/// svrngs starts over a data dir: loopback, any free port, which the bare
/// command prints (DESIGN §3; the u3-2 row) and the node writes to
/// `<data dir>/api` (`svrngs_node::api::API_FILE`, u5-1), where an app
/// svrngs did not start finds it, and every start after listens again
/// (start-address, [`svrngs_node::api::kept_address`]).
pub const API: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

/// svrngs's own relay, where a computer meets each group it holds that
/// keeps none of its own while the build ships [`MeetAt::Svrngs`], with no
/// address lookup (PLAN §3, the operator, 2026-10-12). It answers before
/// any build meeting its groups there is published (DESIGN §7,
/// `scripts/deploy-relay.sh`, `scripts/release.sh`).
pub const DEFAULT_RELAY: &str = "https://relay.svrnme.sh/";

/// Where a computer meets each group it holds that keeps no relay of its
/// own: the switch the operator flips for production (PLAN §2, §3, the
/// operator, 2026-10-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeetAt {
    /// n0's relays, its address lookup beside them.
    N0,
    /// svrngs's own, [`DEFAULT_RELAY`], and no lookup.
    Svrngs,
}

/// Where this build meets its groups: svrngs's, the operator's switch
/// (HUMAN-switch; PLAN §2), once its relay held svrn-docs
/// `rings/current/relay-load` and a real phone through it
/// (demo/runs/phone-home/2026-11-20), n0's public relays behind it only
/// while it does not answer (`svrngs_node::tiers::behind`).
/// `scripts/release.sh` reads this line, and publishes a build at
/// svrngs's only while its relay answers (DESIGN §7).
pub const SHIPPED: MeetAt = MeetAt::Svrngs;

/// What was typed.
#[derive(Debug, PartialEq, Eq)]
pub enum Verb {
    Run,
    New {
        name: Name,
        person: Option<Name>,
        /// `None` prints the client's link alone.
        origin: Option<Origin>,
        /// `--relay`, once for each: where this group meets, beside where
        /// the build meets every group ([`SHIPPED`]; relay fallbacks).
        relays: Vec<RelayUrl>,
    },
    Join {
        link: Box<Link>,
        person: Option<Name>,
        /// `--no-run` makes it false: stop once in, for svrngs kept
        /// running at login (install.sh; DESIGN §7).
        run: bool,
    },
    Link {
        group: String,
        /// `None` keeps the group's kept origin, if any.
        origin: Option<Origin>,
        /// `--device`: a device link, for another device of this
        /// computer's person (bash-devices).
        device: bool,
        /// `--relay`, once for each: where the group meets from now on,
        /// when its keeper changes it; empty leaves it.
        relays: Vec<RelayUrl>,
    },
    /// No secret: it is read from a prompt (the module docs). Each
    /// `--group`, a name or the start of an id, as typed: who can open it.
    Share(Share, Vec<String>),
    /// Each `--group` as typed: only those taken from its audience; none
    /// withdraws it.
    Unshare(LendName, Vec<String>),
    /// `approve`: what answers on the lend's port approved, with `--pid`
    /// the process its lender chose (svrn-docs SHARE_BINDING §5.4), and
    /// `--token` what a lend's list showed of it (`Offered::token`).
    Approve(LendName, Option<share::Pid>, Option<String>),
    /// `share --folder --group`, asked of svrngs running here: the
    /// directory, absolute, and the group as typed.
    Folder {
        dir: PathBuf,
        group: String,
    },
    /// The lends named, served to this computer's network.
    Room(Vec<LendName>),
    /// `remove` or `handover` (U7), asked of svrngs running here: which,
    /// the person, the group as typed, and of two people of one name, when
    /// the one meant joined (bash-same-name).
    Remove {
        kind: RemoveKind,
        person: Name,
        group: String,
        joined: Option<Joined>,
    },
    /// `leave` (groups-leave), asked of svrngs running here: the group
    /// as typed.
    Leave(String),
    /// `accept`, asked of svrngs running here: the group as typed, whose
    /// waiting hand-on this computer takes.
    Accept(String),
    /// `open` (give-session): the home page, in this computer's browser,
    /// or with `--print` shown for another.
    Open(Option<String>, bool),
    /// `update`: the newest svrngs from the page this computer came from.
    Update,
    /// `version`: this build's release, and the commit it was built from.
    Version,
    /// `restart`: svrngs stopped and started by what keeps it running.
    Restart,
    Relay(Relay),
    Help,
    /// `guide`: the guide to building an app, as this build holds it.
    Guide,
    /// `check`: what in the app at that folder the policy refuses.
    Check(PathBuf),
}

/// What `--program` (and its older spelling, `--kind`) and `--reads` want,
/// said alike.
const A_PROGRAM: &str = "a program's name: letters, digits, - and _ only";

/// Which of a keeper's two asks of a person in a group (U7): a closed set
/// (AGENTS 9), each posted to its own route on svrngs running here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveKind {
    /// `remove`: the person out of the group.
    Remove,
    /// `handover`: the keeper's office to the person.
    Handover,
}

/// Parse `std::env::args()`. A wrong argument is an error string the caller
/// prints, never a guess at what was meant.
pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Verb, String> {
    let mut it = args.into_iter().skip(1);
    let Some(verb) = it.next() else {
        return Ok(Verb::Run);
    };
    let mut positional: Option<String> = None;
    let mut person = None;
    let (mut origin, mut relays) = (None, Vec::new());
    let (mut name, mut kind, mut limit, mut refused) = (None, None, None, Vec::new());
    let mut upload = None;
    let mut reads = Vec::new();
    let (mut folder, mut groups) = (None, Vec::new());
    let mut asks = None;
    let (mut pid, mut token) = (None, None);
    let mut replace = false;
    let mut sign_in = false;
    let mut readers = false;
    let mut carries = None;
    let mut run = true;
    let mut device = false;
    let mut print = false;
    let (mut open, mut staging, mut here) = (false, false, None);
    let mut joined = None;
    let mut more = Vec::new();
    while let Some(arg) = it.next() {
        // Every flag taking a value reads it here, and each refusal of
        // one is said the one way, `not`: `--relay` and `--joined` only
        // where they are flags, so elsewhere they stay unknown. The rest
        // are a pattern the compiler holds, never text split (#147).
        let valued = match arg.as_str() {
            "--relay" => matches!(verb.as_str(), "new" | "link"),
            "--at" => verb == "relay",
            "--joined" => matches!(verb.as_str(), "remove" | "handover"),
            "--token" => verb == "approve",
            "--as" | "--origin" | "--name" | "--program" | "--kind" | "--pid" | "--limit"
            | "--refuse" | "--upload" | "--reads" | "--sign-in" | "--folder" | "--group"
            | "--ask" => true,
            _ => false,
        };
        let raw = match valued {
            true => it.next().ok_or_else(|| format!("{arg} wants a value"))?,
            false => String::new(),
        };
        let not = |what: &str| format!("{arg} `{raw}` is not {what}");
        match arg.as_str() {
            "--as" => {
                person = Some(Name::parse(&raw).map_err(|why| not(&format!("a name: {why}")))?)
            }
            "--origin" => {
                origin = Some(
                    Origin::parse(&raw).ok_or_else(|| not("an http(s) address without a '#'"))?,
                )
            }
            "--relay" if valued => {
                let url = format!("{}{raw}", if raw.contains("://") { "" } else { "https://" });
                relays.push(svrngs_node::relay::parse(&url).map_err(|_| not("a relay's address"))?);
            }
            "--name" => name = Some(lend_name(&raw).map_err(|why| format!("--name {why}"))?),
            "--program" | "--kind" => kind = Some(Program::parse(&raw).map_err(|_| not(A_PROGRAM))?),
            "--pid" => {
                let what = "a process and its start: <pid>@<start>";
                pid = Some(share::Pid::parse(&raw).ok_or_else(|| not(what))?);
            }
            "--limit" => {
                let what = "a number of people: 1 or more";
                limit = Some(raw.parse::<NonZeroU16>().map_err(|_| not(what))?);
            }
            "--refuse" => {
                refused.push(RefusedPath::parse(&raw).map_err(|_| not("a path: it starts with /"))?)
            }
            "--upload" => {
                let what = "an upload: a whole number of Mbit/s, 1 or more";
                upload = Some(Upload::from_mbit(raw.parse().map_err(|_| not(what))?));
            }
            "--reads" => reads.push(Program::parse(&raw).map_err(|_| not(A_PROGRAM))?),
            "--sign-in" => {
                return Err("--sign-in is gone: svrngs no longer runs a program of yours to sign people in; sign-in with svrngs is the door's own".into())
            }
            "--folder" => folder = Some(raw),
            "--group" => groups.push(raw),
            "--ask" => {
                if raw.trim().is_empty() || raw.chars().any(char::is_control) {
                    return Err("--ask wants words: what an app receives from it".into());
                }
                asks = Some(raw);
            }
            "--joined" if valued => joined = Some(joined_of(&raw)?),
            "--token" if valued => token = Some(raw),
            "--no-run" if verb == "join" => run = false,
            "--device" if verb == "link" => device = true,
            "--print" if verb == "open" => print = true,
            "--sign-in-with-svrngs" if verb == "share" => sign_in = true,
            "--readers" if verb == "share" => readers = true,
            "--replace" if verb == "share" => replace = true,
            "--udp" | "--tcp" if verb == "share" => {
                let said = if arg == "--udp" { lend::Raw::Udp } else { lend::Raw::Tcp };
                if carries.replace(said).is_some() {
                    return Err("a port is lent as UDP or as TCP: give one of --udp and --tcp".into());
                }
            }
            "--open" if verb == "relay" => open = true,
            "--staging" if verb == "relay" => staging = true,
            // Unlisted in USAGE: plain HTTP, no certificate, for a relay on
            // a local network with no internet, where Let's Encrypt cannot
            // reach a name (`relay::At::Here`).
            "--at" if valued => {
                let what = "an address and port of this computer's, as 10.42.3.20:3340";
                here = Some(raw.parse::<SocketAddr>().map_err(|_| not(what))?);
            }
            "-h" | "--help" => return Ok(Verb::Help),
            other if other.starts_with('-') => return Err(format!("unknown argument `{other}`")),
            other if positional.is_none() => positional = Some(other.to_string()),
            other if verb == "room" => more.push(other.to_string()),
            other => return Err(format!("unexpected argument `{other}`")),
        }
    }
    let group = groups.last().cloned();
    if person.is_some() && !matches!(verb.as_str(), "new" | "join") {
        return Err("--as is a `new` and `join` flag".into());
    }
    if origin.is_some() && !matches!(verb.as_str(), "new" | "link") {
        return Err("--origin is a `new` and `link` flag".into());
    }
    // What only a page takes, which --udp and --tcp refuse below.
    let page = !refused.is_empty() || upload.is_some() || !reads.is_empty();
    if (page
        || name.is_some()
        || kind.is_some()
        || limit.is_some()
        || asks.is_some()
        || (pid.is_some() && verb != "approve")
        || sign_in)
        && verb != "share"
    {
        return Err(
            "--name, --program, --pid, --limit, --refuse, --upload, --reads, --ask and --sign-in-with-svrngs are `share` flags"
                .into(),
        );
    }
    if matches!(verb.as_str(), "remove" | "handover") && folder.is_none() {
        let raw = positional.ok_or(format!(
            "{verb} wants a name: `svrngs {verb} <name> --group <group>`"
        ))?;
        let person = Name::parse(&raw).map_err(|why| format!("`{raw}` is not a name: {why}"))?;
        let group = group.ok_or(format!(
            "{verb} wants --group: `svrngs {verb} <name> --group <group>`"
        ))?;
        let kind = match verb.as_str() {
            "remove" => RemoveKind::Remove,
            _ => RemoveKind::Handover,
        };
        return Ok(Verb::Remove {
            kind,
            person,
            group,
            joined,
        });
    }
    match (verb.as_str(), folder, group) {
        ("share", Some(dir), Some(group)) => {
            let dir = std::path::absolute(&dir).map_err(|e| format!("--folder `{dir}`: {e}"))?;
            return Ok(Verb::Folder { dir, group });
        }
        ("share", Some(_), None) => {
            return Err("--folder takes --group: a folder is one group's".into())
        }
        ("share" | "unshare", None, _) | (_, None, None) => {}
        _ => return Err("--folder is a `share` flag, and --group a `share`, `unshare`, `remove` and `handover` one".into()),
    }
    match verb.as_str() {
        "new" => {
            let raw = positional.ok_or("new wants a name: `svrngs new <name>`")?;
            let name = Name::parse(&raw).map_err(|why| format!("`{raw}` is not a name: {why}"))?;
            Ok(Verb::New {
                name,
                person,
                origin,
                relays,
            })
        }
        "join" => {
            let raw = positional.ok_or("join wants a link: `svrngs join <link>`")?;
            let link = Link::parse(&raw).ok_or_else(|| {
                format!("`{raw}` is not a whole link; copy all of it, to the end")
            })?;
            Ok(Verb::Join {
                link: Box::new(link),
                person,
                run,
            })
        }
        "link" => Ok(Verb::Link {
            group: positional.ok_or("link wants a group: `svrngs link <group>`")?,
            origin,
            device,
            relays,
        }),
        "share" => {
            if carries.is_some() && (page || asks.is_some() || sign_in) {
                return Err("--udp and --tcp lend a port as it is, with no paths, upload, reads, ask or sign-in: those are a page's".into());
            }
            let port = positional.ok_or("share wants a port: `svrngs share <port>`")?;
            Ok(Verb::Share(
                Share {
                    port: share::resolve_target(&port)?,
                    name,
                    kind,
                    limit,
                    refused,
                    upload,
                    reads,
                    asks,
                    sign_in,
                    readers,
                    raw: carries,
                    audience: Audience::Legacy,
                    pid,
                    replace,
                },
                groups,
            ))
        }
        "leave" => Ok(Verb::Leave(
            positional.ok_or("leave wants a group: `svrngs leave <group>`")?,
        )),
        "accept" => Ok(Verb::Accept(
            positional.ok_or("accept wants a group: `svrngs accept <group>`")?,
        )),
        "unshare" => {
            let raw = positional.ok_or("unshare wants a name: `svrngs unshare <name>`")?;
            Ok(Verb::Unshare(lend_name(&raw)?, groups))
        }
        "approve" => {
            let raw = positional.ok_or("approve wants a name: `svrngs approve <name>`")?;
            Ok(Verb::Approve(lend_name(&raw)?, pid, token))
        }
        "room" => {
            let first = positional.ok_or("room wants what to serve: `svrngs room <name>...`")?;
            let names = std::iter::once(first)
                .chain(more)
                .map(|raw| lend_name(&raw));
            Ok(Verb::Room(names.collect::<Result<_, _>>()?))
        }
        "relay" => {
            let at = match (here, positional) {
                (Some(_), _) if staging => return Err("--staging asks Let's Encrypt for a name's certificate, and a relay --at an address has none".into()),
                (Some(_), Some(raw)) => return Err(format!("a relay is at a name or --at an address, not both: `{raw}`")),
                (Some(addr), None) => relay::At::Here(addr),
                (None, raw) => {
                    let raw = raw.ok_or("relay wants the name it is reached at: `svrngs relay <name>`")?;
                    relay::At::Name { name: relay::name(&raw)?, staging }
                }
            };
            Ok(Verb::Relay(Relay { at, open }))
        }
        "open" => Ok(Verb::Open(positional, print)),
        "update" => Ok(Verb::Update),
        "version" | "--version" => Ok(Verb::Version),
        "restart" => Ok(Verb::Restart),
        "guide" => Ok(Verb::Guide),
        "check" => {
            let dir = positional.ok_or("check wants an app's folder: `svrngs check <folder>`")?;
            Ok(Verb::Check(dir.into()))
        }
        "help" | "-h" | "--help" => Ok(Verb::Help),
        other => Err(format!("unknown command `{other}`")),
    }
}

/// Why the command stopped: refused (exit 1), or not judged, a
/// precondition absent (exit 3), each by its type (the module docs, "Exit
/// codes"); `update`'s by its verdict. Its words are a person's (DESIGN
/// §1); what is behind each is traced at debug where it is raised.
#[derive(Debug, thiserror::Error)]
pub enum Refusal {
    #[error(transparent)]
    Refused(#[from] Refused),
    #[error(transparent)]
    Unjudged(#[from] Unjudged),
    /// `update` could not judge the page, or refused what it offered.
    #[error(transparent)]
    Update(#[from] svrngs_node::update::UpdateError),
}

/// Another module's refusal, by `?`, as the kind it is.
macro_rules! refusal_from {
    ($($e:ty => $kind:ident),*) => {
        $(impl From<$e> for Refusal {
            fn from(e: $e) -> Self {
                Refusal::$kind(e.into())
            }
        })*
    };
}
refusal_from!(KeyError => Unjudged, ApiError => Unjudged, KeyFailure => Unjudged, ShareRefusal => Refused);

/// It ran and refused: exit 1.
#[derive(Debug, thiserror::Error)]
pub enum Refused {
    #[error(
        "svrngs is already running on this computer ({0} is held); stop it, then run this again"
    )]
    Busy(PathBuf),
    #[error("svrngs is not kept running at login here, so nothing can restart it: stop it where it runs (Ctrl-C), then start it with `svrngs`")]
    NotKept,
    #[error("the svrngs running here is too old to do this: run `svrngs update`, then `svrngs restart`, then run this again")]
    Older,
    #[error("who are you in {0}? say `svrngs new {0} --as <your name>`")]
    NoName(Name),
    #[error("who are you? open the link again with `svrngs join <link> --as <your name>`")]
    NoNameToJoin,
    #[error("this link lets nobody new in: ask whoever sent it for a new one")]
    LinkRefused,
    #[error("someone in {group} already goes by {holder}, so you were not let in as them. If that's you, add this computer from one of yours: `svrngs link {group} --device` there makes a link it joins as you by. Else join under another name, with `--as <your name>` after the link")]
    NameTaken { holder: Name, group: Name },
    #[error("you're no longer in {0} on this computer, and you can't join it again from here yet: join from your phone or another computer instead")]
    Left(Name),
    #[error("you are in no group called {0} on this computer")]
    NoGroup(String),
    /// An older svrngs made the group: nothing in it changes now.
    #[error("{}", svrngs_node::made_older(.0.as_str()))]
    MadeOlder(Name),
    #[error("say which groups can open it: `svrngs share {0} --group <group>`, once for each")]
    NoAudience(NonZeroU16),
    #[error("more than one group here is called {name}; say which by the start of its id: {}", ids.join(", "))]
    Ambiguous { name: String, ids: Vec<String> },
    #[error("you are not in {0} on this computer, so you cannot send its link")]
    NotIn(Name),
    #[error(transparent)]
    Share(#[from] ShareRefusal),
    #[error("{0}'s readers were not let in: {1}")]
    Readers(LendName, String),
    /// A group's own terms for the program refused, in their words.
    #[error("{0}")]
    Terms(String),
    #[error("you lend nothing called {name} from this computer{}", lent_now(.lent))]
    NotLent { name: LendName, lent: Vec<LendName> },
    #[error("{0} gives each app no address of its own, so on a phone every app would open where your group's page keeps who each person is. Use a host that serves your page at every name under it, as group.svrnme.sh does")]
    NoAppAddress(String),
    /// `room` refused by svrngs running here, in its words.
    #[error("{0}")]
    Room(String),
    /// `check`'s findings, or the folder holding no page.
    #[error("{0}")]
    Unfit(String),
    #[error("the folder was refused: {0}")]
    Folder(String),
    /// `remove` or `handover` refused, in the words [`Said::Removal`] says.
    #[error("{0}")]
    Removal(String),
    /// `link --relay` refused, in the node's words.
    #[error("{0}")]
    Meet(String),
    #[error("you are in no group on this computer, so a relay for your groups would let nobody through: join one first, or say --open to relay for everyone")]
    RelayForNobody,
    #[error("svrngs approve shows you what answers on {0}'s port before it keeps sharing {0}, and there's no terminal here to ask you: run it in a terminal, or press Keep sharing it on your home page")]
    Unshown(LendName),
    /// Not svrngs itself, as this account, at the address svrngs wrote.
    #[error("{1} answers at {0}, where svrngs listens, so svrngs sent it nothing, not your token. Stop that program, then start svrngs again with `svrngs restart`; or, if it's another svrngs of yours, use that one")]
    Withheld(String, Withheld),
}

/// A precondition of the run is absent, so nothing is claimed: exit 3.
#[derive(Debug, thiserror::Error)]
pub enum Unjudged {
    #[error("no place for svrngs's data: set $SVRNGS_DIR, or $HOME for ~/.svrngs")]
    NoDataDir,
    #[error("cannot make {0}: {1}")]
    DataDir(PathBuf, std::io::Error),
    #[error(transparent)]
    Key(#[from] KeyError),
    #[error("this computer cannot be reached from here yet (RUST_LOG=debug says why)")]
    Start,
    #[error("Another program on this computer holds {0}, where svrngs keeps your home page, {home}, and what your groups lend: stop that program, then start svrngs again. Or delete {file}, and svrngs takes a new address as it starts, which bookmarks and programs set to this one do not reach",
        file = .1.display(), home = svrngs_node::home::address(.0.port()))]
    Taken(SocketAddr, PathBuf),
    #[error(transparent)]
    Api(#[from] ApiError),
    #[error(
        "the computer that sent this link cannot be reached; it may be off, so try again later"
    )]
    IssuerAway,
    #[error("cannot keep {0} on this computer (RUST_LOG=debug says why)")]
    Keep(String),
    #[error("cannot read the password: {0}")]
    Prompt(std::io::Error),
    #[error("cannot make a sign-in for it on this computer: {0}")]
    SignIn(#[from] KeyFailure),
    #[error("svrngs is not running on this computer: start it, then run this again")]
    NotRunning,
    #[error("svrngs running here did not open your home page: {0}")]
    Unopened(String),
    #[error("svrngs can't tell which program answers at {0}, where svrngs listens ({1}), so it sent it nothing: try again in a moment")]
    Unread(String, &'static str),
    #[error("cannot tell from here whether {0} gives each app an address of its own: the page does not open there (RUST_LOG=debug says why); try again")]
    CannotCheck(String),
    #[error("{0} cannot be changed on this computer (RUST_LOG=debug says why)")]
    Unchanged(Name),
    #[error("the groups this computer is in cannot be read on it ({0}), so a relay for them could not tell whom to let through")]
    GroupsUnread(String),
    #[error("the relay did not start: {0}")]
    RelayStart(String),
}

/// `raw` as a lend's name, or the one refusal every verb that takes one
/// gives (`--name`, `unshare`, `room`).
fn lend_name(raw: &str) -> Result<LendName, String> {
    LendName::parse(raw)
        .map_err(|_| format!("`{raw}` is not a name: letters, digits, - and _ only"))
}

fn lent_now(lent: &[LendName]) -> String {
    let names: Vec<&str> = lent.iter().map(LendName::as_str).collect();
    let said = (!lent.is_empty()).then(|| format!("; you lend {}", names.join(", ")));
    said.unwrap_or_default()
}

impl Refusal {
    /// 1 for a refusal, 3 for a precondition absent (the module docs).
    pub fn exit_code(&self) -> u8 {
        match self {
            Refusal::Update(e) if e.judged() => 1,
            Refusal::Refused(_) => 1,
            Refusal::Unjudged(_) | Refusal::Update(_) => 3,
        }
    }
}

/// The data dir: `$SVRNGS_DIR`, else `~/.svrngs`.
pub fn data_dir() -> Result<PathBuf, Refusal> {
    if let Some(dir) = std::env::var_os("SVRNGS_DIR").filter(|d| !d.is_empty()) {
        tracing::debug!(target: "cli", dir = ?dir, "cli: data dir from $SVRNGS_DIR");
        return Ok(PathBuf::from(dir));
    }
    let home = home().ok_or(Unjudged::NoDataDir)?;
    tracing::debug!(target: "cli", home = ?home, "cli: data dir ~/.svrngs");
    Ok(home.join(".svrngs"))
}

/// `$HOME`, read here alone, at the edge: the data dir is under it, and
/// the node places each group's folder in its `svrngs` (svrngs-node
/// folder.rs `places`). None when it is unset or empty.
fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

/// Hold `<root>/lock` for as long as the file lives: one svrngs per data
/// dir, so each group's record has one writer (RUST_ARCH 3).
pub fn claim(root: &Path) -> Result<File, Refusal> {
    svrngs_node::declared::secret_dir(root)
        .map_err(|e| Unjudged::DataDir(root.to_path_buf(), e))?;
    let path = root.join("lock");
    let file = File::create(&path).map_err(|e| Unjudged::DataDir(path.clone(), e))?;
    match file.try_lock() {
        Ok(()) => {
            tracing::debug!(target: "cli", lock = %path.display(), "cli: data dir claimed");
            Ok(file)
        }
        Err(e) => {
            tracing::debug!(target: "cli", lock = %path.display(), error = %e, "cli: refused — another svrngs holds the data dir");
            Err(Refused::Busy(path).into())
        }
    }
}

/// How this computer reaches the network: at the build's relays, where it
/// meets every group beside the group's own (relay fallbacks; svrn-docs
/// `rings/current/GROUP_RELAYS.md` §7), which the node joins to them
/// (svrngs-node group.rs `meeting`). At [`MeetAt::N0`], n0's relays with
/// n0's lookup; at [`MeetAt::Svrngs`], [`DEFAULT_RELAY`] and no lookup,
/// n0's only the emergency the node holds while ours does not answer.
pub fn reach(at: MeetAt) -> RelayConfig {
    let relay_urls = match at {
        MeetAt::N0 => (n0_relays().urls::<Vec<RelayUrl>>().iter())
            .map(ToString::to_string)
            .collect(),
        MeetAt::Svrngs => vec![DEFAULT_RELAY.to_string()],
    };
    tracing::debug!(target: "cli", ?at, ?relay_urls, "cli: the build's relays, where this computer meets every group beside the group's own");
    RelayConfig {
        relay_urls,
        n0_services: at == MeetAt::N0,
        loopback_only: false,
    }
}

/// The whole program, `Help` apart: claim the data dir, load this
/// computer's key, start the node, do the verb, and run until Ctrl-C.
/// `share` declares its lend first, and runs only when svrngs is not
/// running here already; `unshare` only withdraws one.
pub async fn run(command: Verb, root: PathBuf, relay: &RelayConfig) -> Result<(), Refusal> {
    print!("{}", answer(command, &root, relay).await?);
    Ok(())
}

/// What [`run`] says of `verb` over the data dir `here` names, each
/// ask of svrngs running there answered as `here` answers it: so a test
/// asks a node's routes in this process (docs/CHECKS.md, unit 7).
pub async fn answer(verb: Verb, here: &impl Here, relay: &RelayConfig) -> Result<String, Refusal> {
    let root = here.root().to_path_buf();
    let said = match verb {
        Verb::Help => format!("{USAGE}\n"),
        // The release (1.10.0 is V1.10) and the commit build-release.sh
        // built it from, which a build by hand has none of.
        Verb::Version => match option_env!("SVRNGS_COMMIT") {
            Some(commit) => format!("svrngs {} ({commit})\n", env!("CARGO_PKG_VERSION")),
            None => format!("svrngs {}\n", env!("CARGO_PKG_VERSION")),
        },
        Verb::Guide => svrngs_node::surface::GUIDE.to_string(),
        Verb::Check(dir) => checked(&dir)?,
        Verb::Relay(asked) => relay::run(asked, &root).await.map(|()| String::new())?,
        Verb::Unshare(name, groups) => unshare(&root, &name, &groups)?.to_string(),
        Verb::Approve(name, pid, token) => approve(here, &name, pid, token)?,
        Verb::Open(at, print) => open(here, at.as_deref(), print).await?,
        Verb::Update => match update::apply(&root).await? {
            update::Outcome::Current => "This computer has the latest svrngs.\n".into(),
            update::Outcome::Updated => "Updated. `svrngs restart` uses it.\n".into(),
            update::Outcome::Packaged(words) => format!("{words}\n"),
        },
        Verb::Restart => restart(&root)?,
        Verb::Folder { dir, group } => format!("{}\n", folder(here, dir, &group).await?),
        Verb::Remove {
            kind,
            person,
            group,
            joined,
        } => {
            let route = match kind {
                RemoveKind::Remove => REMOVALS_ROUTE,
                RemoveKind::Handover => HANDOVERS_ROUTE,
            };
            // Asked of svrngs running here at `route`, for the group named
            // as `link` names it: what happened, in a person's words.
            let seed = group_named(&root, &group)?;
            let asked = RemovalAsk {
                group: seed.id(),
                person: person.as_str().to_string(),
                joined,
            };
            let answer = here.ask(route, &asked).await?;
            let group = seed.name().clone();
            said(&seed, answer, |outcome| Said::Removal {
                group,
                person,
                outcome,
            })?
            .to_string()
        }
        Verb::Leave(ref group) | Verb::Accept(ref group) => {
            let route = match verb {
                Verb::Accept(_) => ACCEPTS_ROUTE,
                _ => LEAVES_ROUTE,
            };
            let seed = group_named(&root, group)?;
            let answer = here.ask(route, &LeaveAsk { group: seed.id() }).await?;
            let group = seed.name().clone();
            said(&seed, answer, |outcome| Said::Left { group, outcome })?.to_string()
        }
        Verb::Share(mut asked, groups) => {
            asked.audience = audience_of(&root, asked.port, &groups)?;
            let known = share::well_known_kind(asked.port);
            let claimed = claim(&root);
            let terms = match (&claimed, share::kind_of(&asked, known.clone())) {
                (Err(Refusal::Refused(Refused::Busy(_))), Some(kind)) => {
                    Some(terms_of(here, &kind, &groups).await?)
                }
                (claimed, kind) => {
                    tracing::debug!(target: "cli", busy = claimed.is_err(), ?kind, "cli: share — no group's terms asked: svrngs is not running here to read them, or no program is named");
                    None
                }
            };
            let theirs = terms.as_ref().map_or("", |(t, _)| *t);
            let mut lend = share::lend_of(&asked, known, theirs)?;
            share::renamed(&root, &lend, asked.replace)?;
            let read = here.answering(lend.raw(), asked.port, asked.pid.map(|p| p.pid));
            let (approved, answers) =
                share::approve((asked.port, lend.raw()), read, asked.pid, None)?;
            lend.terms.approved = Some(approved);
            if !lend.terms.sends.is_empty() && !sends_said(here, &lend)? {
                lend.terms.sends.clear();
            }
            let accounts = share::accounts_of(&lend.kind, theirs);
            let client = asked.sign_in.then(svrngs_node::oidc::client).transpose()?;
            if let Some(client) = &client {
                tracing::debug!(target: "cli", lend = %lend.name, "cli: share — it signs people in with svrngs: a client id and secret made for its app form");
                lend.terms.sign_in_with_svrngs = Some(client.clone());
            }
            let secret = match share::adds_headers(&lend, theirs) {
                false => {
                    tracing::debug!(target: "cli", lend = %lend.name, ?accounts, "cli: share — no password asked: it signs each person in, with svrngs, with accounts its lender makes (Accounts) or from the door's header (Header), or it is raw");
                    None
                }
                true => here.prompt(&lend.name, &format!("If {} takes a password on each request, paste what it takes in its Authorization header, then press Enter (Enter alone for none): ", lend.name), false)?,
            };
            share_lend(&root, &lend, secret.as_deref())?;
            if asked.readers {
                share_readers(here, &lend, (asked.port, theirs)).await?;
            }
            let sharing = Said::Shared {
                lend: Box::new(lend),
                answers,
                secret: secret.is_some(),
                accounts,
                client,
                issuer: None,
                terms,
            };
            match claimed {
                Err(Refusal::Refused(Refused::Busy(lock))) => {
                    tracing::debug!(target: "cli", lock = %lock.display(), "cli: share — svrngs runs here already, and reads the lend from its declaration");
                    let api = svrngs_node::api::kept_address(&root)?.ok_or(Unjudged::NotRunning)?;
                    issued(sharing, api).to_string()
                }
                claimed => return start(Verb::Run, root, relay, claimed?, Some(sharing)).await,
            }
        }
        other => match (claim(&root), other) {
            (Err(Refusal::Refused(Refused::Busy(_))), Verb::Run) => {
                let api = svrngs_node::api::kept_address(&root)?.ok_or(Unjudged::NotRunning)?;
                let home = svrngs_node::home::address(api.port());
                tracing::debug!(target: "cli", %home, "cli: run — local home reminder");
                format!("svrngs is already running. Your home page is at {home}\nRun `svrngs open` to open it in your browser.\n")
            }
            (
                Err(Refusal::Refused(Refused::Busy(lock))),
                verb @ (Verb::Room(_) | Verb::Join { .. } | Verb::New { .. } | Verb::Link { .. }),
            ) => {
                tracing::debug!(target: "cli", lock = %lock.display(), "cli: svrngs runs here already, so it is asked");
                asked(here, verb, relay).await?
            }
            (claimed, other) => return start(other, root, relay, claimed?, None).await,
        },
    };
    Ok(said)
}

/// `check`: what in the app at `dir` the policy every app's answer carries
/// refuses, each by file and line, refused when anything is, so an agent's
/// loop reads it by the exit code (svrn-docs
/// `rings/current/EXTENSION_POINT.md` §7, G3).
fn checked(dir: &Path) -> Result<String, Refusal> {
    let shown = dir.display();
    let none = format!("{shown} holds no index.html: an app is a folder with an index.html in it");
    let found = svrngs_node::folder::check::check(dir).ok_or(Refused::Unfit(none))?;
    let said: Vec<_> = (found.iter())
        .map(|(file, line, r)| format!("{file}:{line}: {}", r.words()))
        .collect();
    let fix = "this page asks for what apps can't load, so it won't work as it is";
    match said.is_empty() {
        true => Ok(format!("Nothing in {shown} that apps can't load.\n")),
        false => Err(Refused::Unfit(format!("{fix}:\n{}", said.join("\n"))).into()),
    }
}

/// `share --folder --group`, asked of the node running here
/// (`svrngs_node::folder::FolderAsk`): it alone writes this computer's acts,
/// the journal's next index being its process's (svrngs-node journal.rs:16-27).
/// The group is named as `link` names it ([`group::named`]).
async fn folder(here: &impl Here, dir: PathBuf, reference: &str) -> Result<String, Refusal> {
    let group = group_named(here.root(), reference)?.id();
    let asked = FolderAsk { dir, group };
    let (ok, text) = here.ask("/v1/folders", &asked).await?;
    tracing::debug!(target: "cli", ?asked, ok, "cli: the folder's ask answered");
    ok.then_some(text.clone())
        .ok_or(Refused::Folder(text).into())
}

/// `svrngs room`: the lends `names` names turned on for the room by
/// svrngs running here, asked as its home page asks it
/// ([`room::ROUTE`]), and the room as it stands then; else its refusal,
/// in its words.
async fn room(here: &impl Here, names: Vec<LendName>) -> Result<Said, Refusal> {
    let answer = here.ask(room::ROUTE, &room::RoomAsk::Names(names)).await?;
    tracing::debug!(target: "cli", ok = answer.0, text = answer.1, "cli: room — svrngs running here answered");
    let refused = |told: Option<Told>| Refused::Room(told.map_or(answer.1.clone(), |t| t.error));
    answered(&answer)
        .map(Said::Room)
        .map_err(|told| refused(told).into())
}

/// What svrngs running here refuses with ([`answered`]): its words, and
/// their kind, where its route says one.
#[derive(serde::Deserialize)]
struct Told {
    error: String,
    #[serde(default)]
    kind: String,
}

/// An answer of svrngs running here, read as what it answers; else its
/// refusal, `None` where it answered neither, as an older build does.
fn answered<T: serde::de::DeserializeOwned>(
    (ok, text): &(bool, String),
) -> Result<T, Option<Told>> {
    let read = serde_json::from_str(text).ok().filter(|_| *ok);
    read.ok_or_else(|| serde_json::from_str(text).ok())
}

/// What svrngs running here answered a removal, hand-on or leave in the
/// group `seed` names, `text`, said as `said` words it, or the refusal. An
/// outcome it answered as success is done (removal.rs `Outcome`'s status,
/// the one list of them); any other is refused.
fn said(
    seed: &Seed,
    (ok, text): (bool, String),
    said: impl FnOnce(Outcome) -> Said,
) -> Result<Said, Refusal> {
    let outcome: Outcome = serde_json::from_str(&text).map_err(|e| {
        tracing::debug!(target: "cli", error = %e, text = %text, "cli: refused — svrngs running here answered no outcome");
        Unjudged::Unchanged(seed.name().clone())
    })?;
    tracing::debug!(target: "cli", group = %seed.id(), ?outcome, ok, "cli: a removal, hand-on or leave asked");
    match outcome {
        _ if ok => Ok(said(outcome)),
        Outcome::Fault => Err(Unjudged::Unchanged(seed.name().clone()).into()),
        _ => Err(Refused::Removal(said(outcome).to_string().trim_end().to_string()).into()),
    }
}

/// `svrngs open`: an address with a code that opens the home page once,
/// as its session there (give-session), asked of svrngs running here and
/// handed to this computer's opener; shown instead with `print`, for
/// another browser, or where none opens. Any account can read the
/// opener's arguments, and the code lets in only a browser this account
/// runs (svrngs-node session.rs `Sessions::ours`).
async fn open(here: &impl Here, at: Option<&str>, print: bool) -> Result<String, Refusal> {
    let asked = serde_json::json!({ "at": at });
    let (ok, text) = here.ask("/v1/open", &asked).await?;
    let open: Option<serde_json::Value> = serde_json::from_str(&text).ok().filter(|_| ok);
    let at = open.and_then(|o| Some(o["open"].as_str()?.to_string()));
    let at = at.ok_or_else(|| Unjudged::Unopened(text.trim().into()))?;
    let opened = !print && here.browse(at.as_ref());
    tracing::debug!(target: "cli", opened, print, "cli: open — the home page, with a code for its session, given to the browser, else shown");
    Ok(match opened {
        true => "Your home page is open in your browser.\n".into(),
        false => format!("Open this in your browser within a minute: {at}\n"),
    })
}

/// `svrngs restart`: launchd or systemd, whichever keeps svrngs at login,
/// stops it as it stops it at logout and starts it, so this works on any
/// build that is running. One started by hand is kept by neither: refused.
fn restart(root: &Path) -> Result<String, Refusal> {
    use std::os::unix::fs::MetadataExt as _;
    let uid = std::fs::metadata(root).map_or(0, |dir| dir.uid());
    let gui = format!("gui/{uid}/sh.svrnme.svrngs");
    let (manager, how) = match cfg!(target_os = "macos") {
        true => ("launchctl", vec!["kickstart", "-k", &gui]),
        false => ("systemctl", vec!["--user", "restart", "svrngs.service"]),
    };
    let done = std::process::Command::new(manager).args(how).output();
    tracing::debug!(target: "cli", manager, ?done, "cli: restart — asked what keeps svrngs running");
    let said = "svrngs is restarting. Your home page comes back in a moment.\n";
    let kept = done.is_ok_and(|out| out.status.success());
    kept.then(|| said.to_string())
        .ok_or(Refused::NotKept.into())
}

/// A data dir as a verb reaches it, and what answers an ask of svrngs
/// running over it: whether it answered success, and its body. The one
/// boundary between the command and the process it asks (RUST_ARCH 7).
pub trait Here {
    fn root(&self) -> &Path;
    fn ask(&self, route: &str, body: &impl serde::Serialize) -> impl Future<Output = Asked>;
    /// What answers on `port`, as `raw`, to approve, and the start now of
    /// `pid` ([`svrngs_node::uid::answering`]).
    fn answering(&self, raw: Raw, port: NonZeroU16, pid: Option<u32>) -> Result<Answering, Unread>;
    /// `lend`'s port, connected and checked, for what its lender sends it
    /// ([`svrngs_node::door::bound`]).
    fn bound(&self, lend: &Lend) -> impl Future<Output = Result<Bound, Unsent>>;
    fn prompt(&self, name: &LendName, asked: &str, shown: bool) -> Prompted;
    /// Whether a person is at this terminal to answer a question: standard
    /// input is one.
    fn terminal(&self) -> bool {
        std::io::IsTerminal::is_terminal(&std::io::stdin())
    }
    /// Whether the program that opens addresses on this computer took `what`:
    /// the one way the command reaches a browser, so a test stands in for it.
    fn browse(&self, what: &std::ffi::OsStr) -> bool;
}

pub type Asked = Result<(bool, String), Refusal>;
pub type Prompted = Result<Option<String>, Refusal>;

/// The product's: POST `body` to `route` of svrngs running here, at the
/// API it wrote, showing the token it keeps beside it.
impl Here for PathBuf {
    fn root(&self) -> &Path {
        self
    }
    fn answering(&self, raw: Raw, port: NonZeroU16, pid: Option<u32>) -> Result<Answering, Unread> {
        svrngs_node::uid::answering(raw, port.get(), pid)
    }
    async fn bound(&self, lend: &Lend) -> Result<Bound, Unsent> {
        svrngs_node::door::bound::here(self, lend).await
    }
    /// macOS's `open`, `xdg-open` anywhere else (svrngs-node home.rs `opened`).
    fn browse(&self, what: &std::ffi::OsStr) -> bool {
        svrngs_node::home::opened(what).is_some()
    }
    /// Ask for the lend's password on this terminal, never echoed, and read
    /// one line from standard input: `None` for an empty line or none. The
    /// value is never traced, only whether there is one.
    fn prompt(&self, name: &LendName, asked: &str, shown: bool) -> Prompted {
        use std::io::{BufRead, IsTerminal, Write};
        let stdin = std::io::stdin();
        let terminal = stdin.is_terminal() && !shown;
        eprint!("{asked}");
        let _ = std::io::stderr().flush();
        let echo = |on: bool| {
            if terminal {
                let _ = std::process::Command::new("stty")
                    .arg(if on { "echo" } else { "-echo" })
                    .stdin(std::process::Stdio::inherit())
                    .status();
            }
        };
        echo(false);
        let mut line = String::new();
        let read = stdin.lock().read_line(&mut line);
        echo(true);
        eprintln!();
        read.map_err(|e| {
            tracing::debug!(target: "cli", error = %e, "cli: share — the password cannot be read");
            Unjudged::Prompt(e)
        })?;
        let secret = line.trim().to_string();
        tracing::debug!(target: "cli", lend = %name, given = !secret.is_empty(), terminal, "cli: share — the password asked for");
        Ok((!secret.is_empty()).then_some(secret))
    }
    /// Asked only of svrngs itself, as this account, at the address it
    /// wrote ([`svrngs_node::api::asked`]): anything else there is sent
    /// nothing, and the person is told what answers there.
    async fn ask(&self, route: &str, body: &impl serde::Serialize) -> Asked {
        let read = |file| std::fs::read_to_string(self.join(file)).unwrap_or_default();
        let (api, token) = (read(API_FILE), read(svrngs_node::api::TOKEN_FILE));
        let body = serde_json::to_string(body).expect("an ask is JSON");
        let asked = svrngs_node::api::asked(&api, route, token.trim(), body).await;
        tracing::debug!(target: "cli", api, route, status = ?asked.as_ref().map(|a| a.0), "cli: svrngs running here answered, or what answers there was sent nothing, and why");
        // Nothing there, as before; what answers there, refused; or that
        // it can't be told, so the person tries again.
        let (status, text) = asked.map_err(|why| match why {
            Withheld::Away => Refusal::from(Unjudged::NotRunning),
            Withheld::Unread(why) => Unjudged::Unread(api.trim().into(), why).into(),
            theirs => Refused::Withheld(api.trim().into(), theirs).into(),
        })?;
        let ok = (200..300).contains(&status);
        Ok((ok, String::from_utf8_lossy(&text).into()))
    }
}

/// Start the node over the data dir `claimed` holds, do the verb, and run
/// until Ctrl-C.
async fn start(
    command: Verb,
    root: PathBuf,
    relay: &RelayConfig,
    _claimed: File,
    sharing: Option<Said>,
) -> Result<String, Refusal> {
    let key = identity::load_or_generate_node_key(&root)?;
    // Where the API, the home page and each lend's address here listened
    // last, so a bookmark and a program's address survive a restart
    // (start-address, demo/runs/start/2026-10-13/README.md §4).
    let api = svrngs_node::api::kept_address(&root)?.unwrap_or(API);
    // The relays `new --relay` or a link names, given the endpoint as it is
    // built, so it does not first wait on the build's (relay fallbacks).
    let asked: Vec<_> = match &command {
        Verb::New { relays, .. } => relays.iter().cloned().map(Into::into).collect(),
        Verb::Join { link, .. } => link.token.relays.clone(),
        _ => Vec::new(),
    };
    let seeds = group::seed_text;
    let every = svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL;
    let places = home().map(|h| h.join("svrngs"));
    tracing::debug!(target: "cli", ?places, "cli: where a group's folder is placed, or none with no $HOME");
    let dirs = (root.clone(), places);
    let node = Node::start_every(dirs, key.clone(), relay, &asked, seeds, api, every);
    let node = node.await.map_err(|e: NodeError| {
        tracing::debug!(target: "cli", error = %e, "cli: refused — the node did not start");
        match e {
            NodeError::Api(ApiError::Taken(at)) => Unjudged::Taken(at, root.join(API_FILE)),
            NodeError::Api(e) => Unjudged::Api(e),
            _ => Unjudged::Start,
        }
    })?;
    if let Some(sharing) = sharing {
        println!("{}", issued(sharing, node.api_addr()));
    }
    let ran = serve(command, &root, &node, relay).await;
    // iroh says at error when an endpoint is dropped open, so it is closed
    // first, on a refusal as on Ctrl-C.
    node.net().close().await;
    ran.map(|()| String::new())
}

/// `origin`'s scheme and host, with its port: where a browser keeps what
/// a page there stores. `None` for what is not an http(s) address.
fn place(origin: &Origin) -> Option<(&str, &str)> {
    let (scheme, rest) = origin.as_str().split_once("://")?;
    let authority = rest.split(['/', '?']).next()?;
    let authority = authority.rsplit_once('@').map_or(authority, |(_, a)| a);
    Some((scheme, authority))
}

/// The page `page`, when its host gives each of the group's apps an
/// address of its own on a phone: a name nobody made under it serves the
/// page at its root as the page's own address does (u5-10: www.svrnme.sh/g/
/// resolved every name and served the page at none but its own); refused
/// when it does not, and not judged when the page's own address does not
/// give the page (u5-2). No page given, none is checked: `new` and `link`
/// take one only with `--origin`.
#[expect(
    clippy::disallowed_methods,
    reason = "draws a name nobody made, to ask the page's host for the page there"
)]
pub async fn checked_page(page: Option<Origin>) -> Result<Option<Origin>, Refusal> {
    let Some(page) = page else { return Ok(None) };
    // An address in brackets is IPv6's, a literal no name falls under.
    let named = place(&page).filter(|(_, a)| {
        let host = a.split(':').next().unwrap_or_default();
        !a.starts_with('[') && !host.is_empty() && host.parse::<std::net::IpAddr>().is_err()
    });
    let no = || Refusal::from(Refused::NoAppAddress(page.as_str().into()));
    let Some((scheme, authority)) = named else {
        tracing::debug!(target: "cli", page = page.as_str(), "cli: refused — the page is at an address no name falls under");
        return Err(no());
    };
    let unjudged = || Unjudged::CannotCheck(page.as_str().into());
    #[expect(clippy::disallowed_methods, reason = "dial: a group's page")]
    let client = svrngs_node::http::client().map_err(|error| {
            tracing::debug!(target: "cli", error, "cli: not judged — no client to ask the page's host with");
            unjudged()
        })?;
    let get = |url: String| {
        let client = &client;
        async move {
            let ask = client.get(&url).timeout(Duration::from_secs(10));
            let got = match ask.send().await {
                Ok(r) if r.status().is_success() => r.bytes().await.map_err(|e| format!("{e:?}")),
                Ok(r) => Err(r.status().to_string()),
                Err(e) => Err(format!("{e:?}")),
            };
            tracing::debug!(target: "cli", url, got = ?got.as_ref().map(|b| b.len()), "cli: asked for the page");
            got.ok()
        }
    };
    let ours = get(page.as_str().to_string()).await.ok_or_else(unjudged)?;
    let mut nobodys = [0u8; 16];
    getrandom::fill(&mut nobodys).expect("failed to draw a name nobody made");
    let theirs = get(format!("{scheme}://{}.{authority}/", hex::encode(nobodys))).await;
    if theirs.as_ref() != Some(&ours) {
        tracing::debug!(target: "cli", page = page.as_str(), served = ?theirs.map(|b| b.len()), "cli: refused — a name nobody made under the page's host does not serve the page");
        return Err(no());
    }
    tracing::debug!(target: "cli", page = page.as_str(), authority, "cli: a name nobody made under the page's host serves the page, so each app opens at an address of its own");
    Ok(Some(page))
}

/// A link as printed: the client's, and the https one for a group that
/// keeps a page origin, whose host its apps open under on a phone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printed {
    pub client: String,
    pub https: Option<String>,
    pub page: Option<Origin>,
}

impl Printed {
    /// As svrngs running here told it ([`new_running`], [`link_running`]):
    /// the page is the https link's origin.
    fn told(client: String, https: Option<String>) -> Self {
        let page = https
            .as_deref()
            .and_then(Link::parse)
            .and_then(|l| l.origin);
        Self {
            client,
            https,
            page,
        }
    }
}

impl std::fmt::Display for Printed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  {}", self.client)?;
        if let Some(https) = &self.https {
            writeln!(f, "\nor, to open it in a browser:\n\n  {https}")?;
        }
        if let Some((_, host)) = self.page.as_ref().and_then(place) {
            writeln!(f, "\nOn a phone, each app opens at an address of its own under {host}, which can see which app is opened.")?;
        }
        Ok(())
    }
}

/// What `share` says, `said`, once the node it started knows where its API
/// listens, `api`: a sign-in it made names its issuer, made from that port,
/// the same string the door and discovery say (SIGN_IN_WITH_SVRNGS §5
/// "Turning it on").
fn issued(mut said: Said, api: SocketAddr) -> Said {
    if let Said::Shared {
        lend,
        client: Some(_),
        issuer,
        ..
    } = &mut said
    {
        *issuer = Some(svrngs_node::oidc::issuer(api.port(), lend.name.as_str()));
    }
    said
}

/// What a verb tells the person, in DESIGN §1's words alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Said {
    /// `new`: the group made, its first link, and the groups called so
    /// already (#138).
    Made {
        seed: Seed,
        link: Printed,
        beside: Vec<group::Namesake>,
    },
    /// `link`: a new link to a group.
    /// A new link; a device link's person when it is one (bash-devices).
    Linked {
        seed: Seed,
        link: Printed,
        device: Option<Person>,
    },
    /// `join`: let in by the link's issuer, as `person`.
    Joined {
        seed: Seed,
        person: Name,
    },
    /// `join` of a group this computer is in already.
    AlreadyIn(Seed),
    /// After `join`: the group's folder at `dir`, kept the same as
    /// everyone's, or why not.
    Folder {
        group: Name,
        dir: PathBuf,
        placed: Result<(), String>,
    },
    /// The bare command: the names of the groups it runs for, where this
    /// computer's apps reach them, and what the people in them lend.
    Running {
        groups: Vec<Name>,
        /// Groups an older svrngs made: held, and changing nothing.
        older: Vec<Name>,
        api: SocketAddr,
        lends: Lends,
    },
    /// `share`: `lend` declared, with a secret or none; `accounts`, how it
    /// signs each person in and what its lender does for that, when the
    /// table of well-known ports says.
    Shared {
        lend: Box<Lend>,
        /// What answers on its port, approved, as its lender is shown it.
        answers: Vec<Answer>,
        secret: bool,
        accounts: Option<share::Signs<'static>>,
        /// The client its lender copies into the app's form, when it signs
        /// people in with svrngs: shown once, then never again.
        client: Option<SignInWithSvrngs>,
        /// The issuer that client names, when it has one.
        issuer: Option<String>,
        /// Its groups' own terms for its program, as read ([`terms_of`]),
        /// empty for none, and who wrote them where; none when svrngs was
        /// not running to read them.
        terms: Option<Read>,
    },
    /// `unshare`: no longer lent.
    Unshared(LendName),
    Narrowed(LendName),
    /// `room`: each lend on for the room, and where it is served or why not.
    Room(Vec<room::InRoom>),
    /// `remove` or `handover` (U7): what happened to `person` in `group`.
    Removal {
        group: Name,
        person: Name,
        outcome: Outcome,
    },
    /// `leave` (groups-leave): what happened as you left `group`.
    Left {
        group: Name,
        outcome: Outcome,
    },
}

impl Said {
    /// The group's folder as svrngs running here told it ([`join_running`],
    /// [`new_running`]): where it is, and why not placed, if it was not.
    fn told_folder(seed: &Seed, dir: String, placed: Option<String>) -> Self {
        let (group, dir) = (seed.name().clone(), PathBuf::from(dir));
        let placed = placed.map_or(Ok(()), Err);
        Said::Folder { group, dir, placed }
    }
}

impl std::fmt::Display for Said {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Said::Made { seed, link, beside } => {
                writeln!(f, "Made {}, with you as {}.", seed.name(), seed.founder())?;
                if !beside.is_empty() {
                    writeln!(f, "{}", namesakes_said(beside, seed.name().as_str()))?;
                }
                write!(f, "Send this link to the people you want in it. It works for a day, while this computer is on:\n\n{link}")
            }
            Said::Linked { seed, link, device } => {
                let group = seed.name();
                match device {
                    None => writeln!(f, "A new link to {group}. Any link you sent before stops letting anyone new in; everyone it let in stays.")?,
                    Some(person) => {
                        writeln!(f, "A link that adds another computer or phone of yours to {group}: open it there, and it joins as you, {person}, asked no name. Send it to nobody else: whoever opens it joins as you. Any link you sent before stops letting anyone new in; everyone it let in stays.")?;
                        writeln!(f, "A computer or phone already in {group} on its own stays as it joined.")?;
                    }
                }
                write!(f, "It works for a day, while this computer is on:\n\n{link}")
            }
            Said::Joined { seed, person } => {
                writeln!(f, "You're in {}, as {person}.", seed.name())
            }
            Said::AlreadyIn(seed) => {
                writeln!(f, "You're in {} already, on this computer.", seed.name())
            }
            Said::Folder { group, dir, placed } => match placed {
                Ok(()) => writeln!(
                    f,
                    "{group}'s folder is {}: what anyone in {group} puts there reaches this computer, and what you put there reaches theirs.",
                    dir.display()
                ),
                Err(why) => writeln!(
                    f,
                    "{group}'s folder is not at {}: {why}. `svrngs share --folder <dir> --group {group}` keeps it where you choose.",
                    dir.display()
                ),
            },
            Said::Running { groups, older, api, lends } => {
                for group in older {
                    writeln!(f, "{}", svrngs_node::made_older(group.as_str()))?;
                }
                if groups.is_empty() {
                    writeln!(
                        f,
                        "You are in no group yet: make one on your home page, at the address below, or with `svrngs new <name>`."
                    )?;
                } else {
                    let names: Vec<&str> = groups.iter().map(Name::as_str).collect();
                    writeln!(f, "Running for {}.", names.join(", "))?;
                }
                writeln!(f, "Apps on this computer reach your groups at http://{api}.")?;
                let home = svrngs_node::home::address(api.port());
                writeln!(f, "Your home page, the same address each time svrngs starts: {home}")?;
                for lent in &lends.lent {
                    let (name, kind, who, at) = (&lent.offered.name, &lent.offered.kind, &lent.person, &lent.address);
                    let what = (name.as_str() != kind.as_str()).then(|| format!(" ({kind})"));
                    writeln!(f, "{who}'s {name}{}, in your browser: {at}", what.unwrap_or_default())?;
                }
                let u = &lends.unlisted;
                for (people, why) in [
                    (&u.unanswered, "did not answer, so what they lend is not listed"),
                    (&u.unheard, "has not heard of yours yet, so what they lend is not listed yet"),
                    (&u.refused, "answered without saying what they lend, so it is not listed"),
                    (&u.older, "runs an older svrngs, so what they lend is not listed"),
                ] {
                    for person in people {
                        writeln!(f, "{}'s computer {why}.", person.person)?;
                    }
                }
                Ok(())
            }
            Said::Shared {
                lend,
                answers,
                secret,
                accounts,
                client,
                issuer,
                terms,
            } => {
                // `share` lends a port; a group's folder carries an app.
                let (name, (port, raw)) = match &lend.source {
                    Source::Port(port) => (&lend.name, (port, String::new())),
                    Source::Raw(raw, port) => (&lend.name, (port, format!("{} ", raw.said()))),
                    Source::Carried(..) => return writeln!(f, "{} is carried in its group's folder.", lend.name),
                };
                match lend.terms.limit {
                    Some(n) => writeln!(f, "You lend {name} ({raw}port {port}) to the groups you chose, up to {n} {} at once.",
                        if n.get() == 1 { "person" } else { "people" })?,
                    None => writeln!(f, "You lend {name} ({raw}port {port}) to the groups you chose, as many at once as want it. \
                        To let fewer in at once, change it on your home page, or `svrngs share {port} --group <group> --limit <number>`.")?,
                }
                let (each, beside, or) = if raw.is_empty() || raw.starts_with("TCP") {
                    ("before each connection it carries there", "While another program is on the port, it is closed. ", "")
                } else {
                    ("before and after each batch of datagrams it sends there", "", ", or another one is on the port")
                };
                writeln!(f, "Lent {name}: {}. svrngs checks port {port} {each}. {beside}If the program starts again{or}, {name} closes until you approve what runs there: `svrngs approve {name}`.", what_answers(answers))?;
                let ((theirs, by), kind) = (terms.clone().unwrap_or_default(), &lend.kind);
                // How its people sign in, in the words of who wrote the group's file.
                let shipped = share::accounts_of(kind, "");
                let by = Some(by).filter(|_| *accounts != shipped).unwrap_or_default();
                let unknown = raw.is_empty() && theirs.is_empty();
                if let Some(told) = share::unrecognized(&lend.kind).filter(|_| unknown) {
                    writeln!(f, "{told}")?;
                }
                if terms.is_none() {
                    writeln!(f, "svrngs wasn't running, so this keeps only the terms svrngs ships for {kind}. A programs/{kind}.txt in your groups' folders is read once it runs: share it again then.")?;
                }
                match accounts {
                    Some(share::Signs::Accounts(how)) => writeln!(
                        f,
                        "{} asks each person to sign in once, with an account you make them. In {}{by}: {how}. Then tell each person theirs.",
                        lend.name, lend.kind
                    )?,
                    Some(share::Signs::Header(what)) => writeln!(
                        f,
                        "{} signs each person in as themselves, with nothing typed, once it takes them from svrngs: in {}{by}, {what}.",
                        lend.name, lend.kind
                    )?,
                    None => {}
                }
                let (table, group) = (share::kept_of(kind, ""), share::kept_of(kind, theirs));
                let (kept, typed): (Vec<_>, Vec<_>) = lend.terms.refused.iter().partition(|p| table.contains(p));
                let said = format!(": your groups' programs/{kind}.txt keeps it back");
                for path in typed {
                    let why = if group.contains(path) { said.as_str() } else { "" };
                    writeln!(f, "They cannot open {}{why}.", path.as_str())?;
                }
                if !kept.is_empty() {
                    let paths: Vec<&str> = kept.iter().map(|p| p.as_str()).collect();
                    writeln!(f, "They cannot open {}: svrngs keeps these back whenever a {} is lent, since they show one person's use of it to the others or change it for everyone.",
                        paths.join(", "), lend.kind)?;
                }
                if let Some(words) = &lend.terms.asks {
                    writeln!(
                        f,
                        "An app uses it only once the person it acts for says yes to: \"{words}\"."
                    )?;
                }
                for kind in &lend.terms.reads {
                    writeln!(
                        f,
                        "It can open the {kind} your groups lend, as whoever opens it."
                    )?;
                }
                if let Some(upload) = lend.terms.upload {
                    writeln!(
                        f,
                        "Each is told their share of the {upload} upload you stated."
                    )?;
                }
                if *secret {
                    writeln!(
                        f,
                        "Its password stays on this computer; nobody you lend it to receives it."
                    )?;
                }
                if let (Some(client), Some(issuer)) = (client, issuer) {
                    writeln!(f, "Sign-in credentials created; app setup still required. Follow the app's OpenID Connect setup guide: install any required OpenID Connect plugin, then configure it with issuer {issuer}, client id {}, client secret {}. Reach the issuer from the app server on this computer, not the computer of the person opening the app; a container has its own loopback. Test sign-in through your group's app list after setup.",
                        client.client_id, client.client_secret.as_deref().unwrap_or_default())?;
                    writeln!(f, "{} Copy the secret now: it is not shown again. App setup guides: https://github.com/alexsbryan/svrngs/tree/main/apps", svrngs_node::oidc::DEVIATIONS)?;
                }
                writeln!(
                    f,
                    "svrngs is running here already, so they can open it now."
                )?;
                writeln!(f, "`svrngs unshare {}` stops lending it.", lend.name)
            }
            Said::Unshared(name) => writeln!(f, "You no longer lend {name}."),
            Said::Narrowed(name) => writeln!(f, "Those groups cannot open {name}."),
            Said::Removal {
                group,
                person,
                outcome,
            } => {
                let decides = |who: &Option<Person>| match who {
                    Some(p) => p.to_string(),
                    None => "whoever decides who is removed".into(),
                };
                match outcome {
                    Outcome::Out => writeln!(f, "{person} is no longer in {group}. What {person} already has stays with them; nothing new reaches them."),
                    Outcome::Asked { decides: who } => {
                        let who = decides(who);
                        writeln!(f, "You asked {who} to remove {person} from {group}. {person} stays in it until {who} does.")
                    }
                    Outcome::Handed => writeln!(f, "{person} now decides who is removed from {group}, instead of you."),
                    Outcome::Offered => writeln!(f, "{person} decides who is removed from {group} once they accept it, with `svrngs accept {group}` or on their home page. Until then nobody does; `svrngs accept {group}` here takes it back."),
                    Outcome::Uncounted => writeln!(f, "{group}'s record would not count that from you, so nothing changed (RUST_LOG=debug says why)."),
                    Outcome::NotInGroup => writeln!(f, "Nobody called {person} is in {group}."),
                    Outcome::Which { joined } => {
                        let each: Vec<String> = joined.iter().map(|j| format!("`--joined {}`", joined_said(j))).collect();
                        writeln!(f, "svrngs cannot tell which person called {person} in {group} you mean, so nothing changed. Say which by when they joined, adding one of these to the command: {}.", each.join(", "))
                    }
                    Outcome::Decides => writeln!(f, "{person} decides who is removed from {group}, so cannot be removed: hand that to someone else first, with `svrngs handover <name> --group {group}`."),
                    Outcome::Own => writeln!(f, "{person} is you, and you cannot remove yourself from {group}, so nothing changed: `svrngs leave {group}` leaves it."),
                    Outcome::NotDecider { decides: who } => writeln!(f, "Only {} can let someone else decide who is removed from {group}.", decides(who)),
                    Outcome::NotDeciderUnread => writeln!(f, "You do not decide who is removed from {group}, and this computer could not read who is in the group, so cannot say who does (RUST_LOG=debug says why)."),
                    Outcome::NotIn | Outcome::Left | Outcome::LeftVacant => writeln!(f, "You are no longer in {group}."),
                    _ => writeln!(f, "{group} cannot be changed on this computer (RUST_LOG=debug says why)"),
                }
            }
            Said::Left { group, outcome } => match outcome {
                Outcome::Left => writeln!(f, "You left {group}, from every computer and phone of yours. Your copy of it and the files its folder gave you stay on this computer; nothing new reaches you, and what you lend no longer reaches its people. Rejoining needs a new invitation, and this computer cannot rejoin yet."),
                Outcome::Decides => writeln!(f, "You decide who is removed from {group}, so you cannot leave it yet: hand that to someone else first, with `svrngs handover <name> --group {group}`, then leave."),
                Outcome::LeftVacant => writeln!(f, "You left {group}, from every computer and phone of yours, and nobody decides who is removed from it now: the others agree together on who does. Your copy of it and the files its folder gave you stay on this computer; nothing new reaches you. Rejoining needs a new invitation, and this computer cannot rejoin yet."),
                Outcome::NotIn => writeln!(f, "You are no longer in {group}."),
                Outcome::Taken => writeln!(f, "You decide who is removed from {group} now."),
                Outcome::NotOffered => writeln!(f, "Nobody handed you who is removed from {group}, so there is nothing to accept. Whoever decides it hands it on with `svrngs handover <name> --group {group}`."),
                Outcome::NotDecider { .. } => writeln!(f, "Only whoever decides who is removed from {group} can move it."),
                // Fault, and what the leave route never answers.
                _ => writeln!(f, "{group} cannot be changed on this computer (RUST_LOG=debug says why)"),
            },
            Said::Room(on) => {
                writeln!(f, "Whenever svrngs runs here, anyone on this computer's WiFi can open these as you; nobody off it can. Each stays on the WiFi where you turned it on, until you close it to this WiFi on your home page.")?;
                for l in on {
                    let whose = l.person.as_ref().map_or(String::new(), |p| format!("{p}'s "));
                    writeln!(f, "{whose}{}: {}", l.name, l.said)?;
                }
                Ok(())
            }
        }
    }
}

/// `--joined`: `started`, for whoever started the group, or a time in the
/// home page's form ([`WHEN`]), as [`joined_said`] says it.
fn joined_of(raw: &str) -> Result<Joined, String> {
    if raw == "started" {
        return Ok(Joined::Started);
    }
    (chrono::DateTime::parse_from_str(raw, WHEN).ok())
        .map(|t| Joined::At(t.timestamp()))
        .ok_or_else(|| format!("--joined `{raw}` is not when someone joined: `started`, or a time as svrngs says it, such as \"2026-10-12 14:03:22 -0700\""))
}

/// The groups a new one, `name`, shares its name with, each with who made
/// it and since when this computer has held it (#138), in the home page's
/// words to the letter (question.js `namesakes`, `whose` and `day`; #157,
/// pinned by tests/home_group.rs): "You’re already in Trip (Sam’s, since
/// 3 Oct). This is a second group called trip." — the page's apostrophe,
/// and a day as its number and its month's first three letters.
pub fn namesakes_said(beside: &[group::Namesake], name: &str) -> String {
    let at = |t| chrono::DateTime::from_timestamp(t, 0).map(|t| t.with_timezone(&chrono::Local));
    let said = |g: &group::Namesake| {
        let whose = (g.maker.as_ref()).map_or("yours".into(), |m| format!("{m}’s"));
        let since = (g.since.and_then(at)).map(|t| t.format(", since %-d %b").to_string());
        format!("{} ({whose}{})", g.name, since.unwrap_or_default())
    };
    let another = ["a second", "another"][usize::from(beside.len() > 1)];
    let all = beside.iter().map(said).collect::<Vec<_>>().join(" and ");
    format!("You’re already in {all}. This is {another} group called {name}.")
}

/// When a person joined, as a person adds it to the command.
fn joined_said(joined: &Joined) -> String {
    match joined {
        Joined::Started => "started".into(),
        Joined::At(t) => chrono::DateTime::from_timestamp(*t, 0).map_or_else(
            || "at a time this computer cannot write".into(),
            |t| format!("\"{}\"", t.with_timezone(&chrono::Local).format(WHEN)),
        ),
    }
}

/// Whether `lend`'s lender says yes, y at this terminal, to apps sending to
/// it where its terms say (svrn-docs `rings/current/CONTRACTS.md` §2, "The
/// lend is the yes"): with no terminal to answer, no, and said so. The
/// question opens with what it doesn't govern, since a lender read the
/// paths alone as all their group could reach (§10, "How a lightly
/// technical lender reads `sends:` at the yes"): people in the group reach
/// all but what's kept back, and only another app is held to reading.
fn sends_said(here: &impl Here, lend: &Lend) -> Result<bool, Refusal> {
    let paths: Vec<&str> = lend.terms.sends.iter().map(|s| s.path()).collect();
    let (name, at) = (&lend.name, paths.join(" and "));
    let terminal = here.terminal();
    let words = format!("People in your group can use all of {name} that isn't kept back. Apps your friends open on their computers can only read it, unless you let them also send to it at {at}; even then, each app sends only after the friend who opened it says yes. Let apps send? Type y, then press Enter: ");
    let yes = terminal && said_yes(here, name, &words)?;
    tracing::debug!(target: "cli", lend = %name, terminal, yes, "cli: share — its lender's yes to apps sending to it, or none");
    if !yes {
        eprintln!("Apps your friends open on their computers can only read {name}, not send to it at {at}; people in your group still use all of it that isn't kept back. To let apps send, run this again in a terminal and type y.");
    }
    Ok(yes)
}

/// Whether its lender, asked `words` at this terminal of the lend `name`,
/// types y or yes: anything else, nothing typed included, is no.
fn said_yes(here: &impl Here, name: &LendName, words: &str) -> Result<bool, Refusal> {
    let said = here.prompt(name, words, true)?.unwrap_or_default();
    Ok(["y", "yes"].contains(&said.to_lowercase().as_str()))
}

/// `svrngs share --readers`: the account apps reading `lend` are let in as,
/// signed in once where the table says how (`share::readers_of`), else
/// what its Authorization header takes, pasted; then kept.
async fn share_readers(
    here: &impl Here,
    lend: &Lend,
    at: (NonZeroU16, Terms),
) -> Result<(), Refusal> {
    let (port, terms) = at;
    let name = &lend.name;
    let ask = |words: String, shown| here.prompt(name, &words, shown);
    let refused = |why: String| Refused::Readers(name.clone(), why);
    let checked = || async {
        let to = here.bound(lend).await;
        to.map_err(|unsent| Refused::Readers(name.clone(), share::unsent_words(name, unsent)))
    };
    if let Some(how) = share::key_of(&lend.kind, terms) {
        let key = ask(format!("Paste the key you made in {name}: "), false)?;
        let key = key.ok_or_else(|| refused("nothing was given".into()))?;
        let tried = match how.2.is_empty() {
            true => None,
            false => share::tried(checked().await?, port, Some(how), &key).await,
        };
        let said = share::key_said(&lend.kind, tried);
        let said = said.map_err(&refused)?;
        share::keep_key(here.root(), name, Some(how), &key).map_err(|e| refused(e.to_string()))?;
        eprintln!("{said} Apps that read {name} are let in with that key, to look only; a person opening it still signs in as themselves.");
        return Ok(());
    }
    let line = match share::readers_of(&lend.kind, terms) {
        Some(how) => {
            let words =
                format!("The name of the account in {name} that apps reading it are let in as: ");
            let account = ask(words, true)?.unwrap_or_default();
            let password = ask("Its password: ".into(), false)?.unwrap_or_default();
            share::sign_in_readers(checked().await?, port, how, &account, &password)
                .await
                .map_err(refused)?
        }
        None => ask(
            format!(
                "Paste what an app reading {name} is let in with, in its Authorization header: "
            ),
            false,
        )?
        .ok_or_else(|| refused("nothing was given".into()))?,
    };
    lend::keep_readers(here.root(), name, (lend::SECRET_HEADER, &line))
        .map_err(|e| refused(e.to_string()))?;
    eprintln!("Apps that read {name} are let in as that account, to look only; a person opening it still signs in as themselves.");
    Ok(())
}

/// The terms the groups named `groups` hold for `kind` in their own
/// `programs/<kind>.txt`, asked of svrngs running here, which reads them
/// through each group's record and holds them to what a file may say
/// (folder/carried.rs `Folders::agreed`, CONTRACTS §2); empty for none; and
/// who wrote them where, as its lender is told it beside their words.
/// Refused, never read as no terms, in its words. Kept for the life of this
/// command, as the terms svrngs ships are.
async fn terms_of(here: &impl Here, kind: &Program, groups: &[String]) -> Result<Read, Refusal> {
    let named = |name: &String| group_named(here.root(), name).map(|g| g.id());
    let ids: Vec<String> = groups.iter().map(named).collect::<Result<_, _>>()?;
    let (ok, text) = here.ask(PROGRAMS_ROUTE, &(ids, kind)).await?;
    tracing::debug!(target: "cli", ?groups, %kind, ok, text, "cli: share — its groups' own terms for the program, read through their records, or why not");
    // An older svrngs answers what it can't read in words, not JSON.
    match (ok, serde_json::from_str::<(String, String)>(&text)) {
        (true, Ok((text, by))) => Ok((text.leak(), by)),
        (false, _) => Err(serde_json::from_str(&text)
            .map_or(Refused::Older, Refused::Terms)
            .into()),
        (true, Err(_)) => Err(Refused::Older.into()),
    }
}

/// A group's own terms ([`terms_of`]), and who wrote them where, as its
/// lender is told it after their `accounts:` or `header:` words.
type Read = (Terms, String);

/// `svrngs approve`: what answers on the port of the lend `name` approved,
/// with `pid` the process its lender chose, and nothing else of it changed
/// (svrn-docs SHARE_BINDING §5.4), only as its lender was shown it: by
/// `token`, what a lend's list showed; else at a terminal, what answers now
/// said first, approved on a y by that read's token, so what changed before
/// the y is refused; else refused, nothing approved unseen. What it approved,
/// said.
pub fn approve(
    here: &impl Here,
    name: &LendName,
    pid: Option<share::Pid>,
    token: Option<String>,
) -> Result<String, Refusal> {
    let (lend, port) = share::shared(here.root(), name)?;
    let read = || here.answering(lend.raw(), port, pid.map(|p| p.pid));
    let terminal = here.terminal();
    tracing::debug!(target: "cli", lend = %name, given = token.is_some(), terminal, "cli: approve — what answers approved by the token given, else said and asked at a terminal, else refused");
    let token = match token {
        Some(token) => token,
        None if !terminal => return Err(Refused::Unshown(name.clone()).into()),
        None => {
            let (seen, answers) = share::approve((port, lend.raw()), read(), pid, None)?;
            let asked = format!("Now answering on {name}'s port: {}. Keep sharing {name} with it? Type y, then press Enter: ", what_answers(&answers));
            if !said_yes(here, name, &asked)? {
                return Ok(format!("Nothing approved: {name} stays closed.\n"));
            }
            share::token(&seen)
        }
    };
    let (approved, answers) = share::approve((port, lend.raw()), read(), pid, Some(&token))?;
    share::reapproved(here.root(), name, approved)?;
    Ok(format!(
        "{name} is kept to {} now.\n",
        what_answers(&answers)
    ))
}

/// What answers on a port, as its lender is told it: each program in its
/// folder, or the account another's runs as, "and" between them. An
/// approval always has one: `share::approve` refuses a socket shown nothing.
fn what_answers(answers: &[Answer]) -> String {
    let said: Vec<String> = answers.iter().map(ToString::to_string).collect();
    said.join(" and ")
}

/// `svrngs share`'s write: `lend` declared in the data dir at `root`, with
/// `secret` sent as [`lend::SECRET_HEADER`], or none.
pub fn share_lend(root: &Path, lend: &Lend, secret: Option<&str>) -> Result<(), Refusal> {
    lend::declare(root, lend, secret.map(|s| (lend::SECRET_HEADER, s))).map_err(|e| {
        tracing::debug!(target: "cli", lend = %lend.name, error = %e, "cli: share — the lend cannot be kept");
        Unjudged::Keep(lend.name.to_string()).into()
    })
}

/// `svrngs unshare`: withdraw the lend `name` from the data dir at `root`.
pub fn unshare(root: &Path, name: &LendName, groups: &[String]) -> Result<Said, Refusal> {
    let keep = |e: lend::LendError| {
        tracing::debug!(target: "cli", lend = %name, error = %e, "cli: unshare — the lend cannot be withdrawn");
        Unjudged::Keep(name.to_string())
    };
    let ids = groups.iter().map(|g| group_named(root, g).map(|s| s.id()));
    let ids = ids.collect::<Result<Vec<_>, _>>()?;
    let narrowed = |lend: &mut Lend| {
        if let Audience::Selected(kept) = &mut lend.terms.audience {
            kept.retain(|id| !ids.contains(id));
        }
    };
    tracing::debug!(target: "cli", lend = %name, groups = ids.len(), "cli: unshare — those groups only, or none named and it is withdrawn");
    let (done, said): (_, fn(LendName) -> Said) = match ids.is_empty() {
        true => (lend::withdraw(root, name), Said::Unshared),
        false => (lend::mutate(root, name, narrowed), Said::Narrowed),
    };
    if done.map_err(keep)? {
        return Ok(said(name.clone()));
    }
    // Each lend that reads, its name.
    let lent = lend::all(root).map_err(keep)?.into_iter().flatten();
    let lent = lent.map(|l| l.name).collect();
    tracing::debug!(target: "cli", lend = %name, "cli: unshare — refused, nothing lent by that name");
    Err(Refused::NotLent {
        name: name.clone(),
        lent,
    }
    .into())
}

/// What the bare command prints: the groups held, where apps reach them,
/// and what the people in them lend, asked now.
pub async fn running(root: &Path, node: &Node) -> Said {
    let (older, groups): (Vec<_>, Vec<_>) = group::groups(root).into_iter().partition(Seed::older);
    let named = |seeds: Vec<Seed>| seeds.iter().map(|s| s.name().clone()).collect();
    Said::Running {
        groups: named(groups),
        older: named(older),
        api: node.api_addr(),
        lends: node.lends().await,
    }
}

/// A verb on a started node, asked of it as of svrngs running here
/// ([`asked`]), then the wait for Ctrl-C.
async fn serve(
    mut command: Verb,
    root: &Path,
    node: &Node,
    relay: &RelayConfig,
) -> Result<(), Refusal> {
    let stays = !matches!(command, Verb::Join { run: false, .. });
    if let Verb::Join { link, person, .. } = &mut command {
        *person = Some(
            person
                .take()
                .or_else(user_name)
                .ok_or(Refused::NoNameToJoin)?,
        );
        // Its home relay reached before the exchange, so the joiner says it
        // there and the issuer can dial it back; else not before its first
        // round (demo/runs/relay-in-record/2026-10-12/: 60 s). Before the ask,
        // so a relay that does not answer is named before the issuer is dialled.
        let carried: Vec<String> = link.token.relays.iter().map(ToString::to_string).collect();
        node.host().meet_at(&link.token.relays).await;
        let online = node.host().online().await;
        tracing::debug!(target: "cli", online, tried = ?carried, "cli: join — waited for a relay");
        unreached(online, &carried);
    }
    let said = match command {
        Verb::Run => running(root, node).await.to_string(),
        verb => asked(&root.to_path_buf(), verb, relay).await?,
    };
    println!("{said}");
    if !stays {
        tracing::debug!(target: "cli", "cli: join --no-run — stopping once in");
        return Ok(());
    }
    println!("Leave this running; Ctrl-C stops it.");
    svrngs_node::node::asked_to_stop().await;
    tracing::debug!(target: "cli", "cli: stopping");
    Ok(())
}

/// A join the node did not let this computer in by, in the command's
/// refusals.
impl From<NotJoined> for Refusal {
    fn from(why: NotJoined) -> Self {
        match why {
            NotJoined::LinkRefused => Refused::LinkRefused.into(),
            NotJoined::NameTaken { holder, group } => Refused::NameTaken { holder, group }.into(),
            NotJoined::IssuerAway => Unjudged::IssuerAway.into(),
            NotJoined::Keep { group } => Unjudged::Keep(group.to_string()).into(),
            NotJoined::Left { group } => Refused::Left(group).into(),
            NotJoined::Older { group } => Refused::MadeOlder(group).into(),
        }
    }
}

/// `verb` asked of svrngs running over the data dir `here` names, the
/// one writer of this computer's acts: it makes the group, the link or the
/// join, or turns the room on, and the command words what it answered.
pub async fn asked(here: &impl Here, verb: Verb, relay: &RelayConfig) -> Result<String, Refusal> {
    let tried = &relay.relay_urls;
    Ok(match verb {
        Verb::Room(names) => room(here, names).await?.to_string(),
        Verb::Join { link, person, .. } => {
            let person = person.or_else(user_name).ok_or(Refused::NoNameToJoin)?;
            join_running(here, &link, &person).await?
        }
        Verb::Link {
            group,
            origin,
            device,
            relays,
        } => link_running(here, &group, origin, device, relays, tried).await?,
        Verb::New {
            name,
            person,
            origin,
            relays,
        } => new_running(here, name, person, origin, relays, tried).await?,
        _ => unreachable!("asked only of what svrngs running here does"),
    })
}

/// `svrngs join`, of the svrngs running here (join-while-running), or
/// of the one this command started ([`serve`]): the link asked at the
/// record's join route — svrngs running is the one writer of this
/// computer's acts, so it joins and places the group's folder, and the
/// command says what it answered. A link whose whole text
/// `Link::parse` here already proved is sent as it was given; the running
/// svrngs that answers none of it is an older build: [`Refused::Older`],
/// never the lock's own refusal, which would send someone to stop it.
async fn join_running(here: &impl Here, link: &Link, person: &Name) -> Result<String, Refusal> {
    let whole = match &link.origin {
        Some(page) => https_link(page, &link.token),
        None => client_link(&link.token),
    };
    let asked = JoinAsk {
        link: whole,
        person: person.as_str().to_string(),
    };
    let (_, text) = here.ask(JOINS_ROUTE, &asked).await?;
    let older = |why: String| {
        tracing::debug!(target: "cli", %why, text, "cli: the svrngs running here answered no join, as one of an older build does not");
        Refused::Older
    };
    let said: JoinOutcome = serde_json::from_str(&text).map_err(|e| older(e.to_string()))?;
    let held = |text: String| Seed::from_text(&text).map_err(&older);
    Ok(match said {
        JoinOutcome::In {
            seed,
            person,
            dir,
            placed,
        } => {
            let seed = held(seed)?;
            let joined = Said::Joined {
                seed: seed.clone(),
                person,
            };
            format!("{joined}{}", Said::told_folder(&seed, dir, placed))
        }
        JoinOutcome::AlreadyIn { seed, dir, placed } => {
            let seed = held(seed)?;
            let already = Said::AlreadyIn(seed.clone());
            format!("{already}{}", Said::told_folder(&seed, dir, placed))
        }
        JoinOutcome::Not(why) => return Err(why.into()),
    })
}

/// The name a group knows its founder or a joiner by when `--as` names
/// none: the full name of the account `$USER` names, as the system shows it
/// on its own screens, else `$USER` itself (bash-person (e); the login was
/// what people read of each other, Demo C's dry run, finding 4). `new` and
/// `join` print it, so it is never silent.
pub fn user_name() -> Option<Name> {
    let user = std::env::var("USER").ok()?;
    let full = full_name(&user);
    let taken =
        (full.as_deref().and_then(|f| Name::parse(f).ok())).or_else(|| Name::parse(&user).ok());
    tracing::debug!(target: "cli", %user, ?full, ?taken, "cli: the name with no --as — the account's full name if it is a name, else $USER");
    taken
}

/// The full name the system shows for the account `user`, if it shows
/// one: macOS's (`id -F`), or Linux's GECOS up to its first comma
/// (`getent passwd`).
fn full_name(user: &str) -> Option<String> {
    let mac = cfg!(target_os = "macos");
    let (cmd, args) = match mac {
        true => ("id", ["-F", user]),
        false => ("getent", ["passwd", user]),
    };
    let out = std::process::Command::new(cmd).args(args).output().ok();
    let text = String::from_utf8(out.filter(|o| o.status.success())?.stdout).ok()?;
    let name = match mac {
        true => text.trim(),
        false => text.split(':').nth(4)?.split(',').next()?.trim(),
    };
    (!name.is_empty()).then(|| name.to_string())
}

/// The groups `share` lends to, each `--group` the one [`group_named`]
/// finds: refused with the next command when none is named, never all
/// groups (LENDING_SCOPE §"User interface and commands").
pub fn audience_of(root: &Path, port: NonZeroU16, named: &[String]) -> Result<Audience, Refusal> {
    if named.is_empty() {
        tracing::debug!(target: "cli", port, "cli: share — refused, no group named to lend it to");
        return Err(Refused::NoAudience(port).into());
    }
    let ids = named.iter().map(|g| group_named(root, g).map(|s| s.id()));
    ids.collect::<Result<_, _>>().map(Audience::Selected)
}

/// The one group held under `root` that `reference` names, by its name or
/// the start of its id ([`group::named`]): `link`'s and `share --folder`'s.
pub fn group_named(root: &Path, reference: &str) -> Result<Seed, Refusal> {
    let held = group::groups(root);
    match group::named(&held, reference).as_slice() {
        [] => {
            tracing::debug!(target: "cli", reference, held = held.len(), "cli: refused — no group by that name");
            Err(Refused::NoGroup(reference.to_string()).into())
        }
        [one] if one.older() => {
            tracing::debug!(target: "cli", reference, "cli: refused — an older svrngs made the group, so nothing in it changes");
            Err(Refused::MadeOlder(one.name().clone()).into())
        }
        [one] => Ok((*one).clone()),
        many => {
            let ids = many.iter().map(|s| s.id()[..8].to_string()).collect();
            tracing::debug!(target: "cli", reference, n = many.len(), "cli: refused — the name is ambiguous");
            Err(Refused::Ambiguous {
                name: reference.to_string(),
                ids,
            }
            .into())
        }
    }
}

/// `svrngs new`, of the svrngs running here or started by this command,
/// the one writer of this computer's acts ([`group::NEW_ROUTE`]): its page
/// checked here first ([`checked_page`]); that it reached no relay said
/// where it `tried` any ([`unreached`]). One that answers no group is an older build, said as
/// [`link_running`] says it; its refusal, why traced, is that the group
/// can't be kept.
async fn new_running(
    here: &impl Here,
    name: Name,
    person: Option<Name>,
    origin: Option<Origin>,
    relays: Vec<RelayUrl>,
    tried: &[String],
) -> Result<String, Refusal> {
    let person = person
        .or_else(user_name)
        .ok_or_else(|| Refused::NoName(name.clone()))?;
    let page = checked_page(origin).await?;
    let asked = group::NewAsk {
        name: name.as_str().into(),
        person: person.as_str().into(),
        origin: page.map(|p| p.as_str().into()),
        relays,
    };
    let answer = here.ask(group::NEW_ROUTE, &asked).await?;
    let made: group::NewMade = answered(&answer).map_err(|told| -> Refusal {
        let Some(told) = told else { return Refused::Older.into() };
        tracing::debug!(target: "cli", why = told.error, "cli: refused — the svrngs running here made no group");
        Unjudged::Keep(name.to_string()).into()
    })?;
    let seed = Seed::from_text(&made.seed).map_err(|_| Refused::Older)?;
    tracing::debug!(target: "cli", online = made.online, ?tried, "cli: new — made by the svrngs running here, which had reached a relay or not");
    unreached(made.online, tried);
    let link = Printed::told(made.client, made.https);
    let folder = Said::told_folder(&seed, made.dir, made.placed);
    let beside = made.beside;
    Ok(format!("{}\n{folder}", Said::Made { seed, link, beside }))
}

/// `svrngs link`, of the svrngs running here or started by this command,
/// the one writer of this computer's acts
/// ([`svrngs_node::home::LINKS_ROUTE`]), its page checked here first:
/// not in the group, or unkept, in the command's words; any other
/// refusal, a non-keeper's among them, in its own; and that it reached no
/// relay as [`new_running`] says it. One that answers no link is an older
/// build, said as [`join_running`] says it.
async fn link_running(
    here: &impl Here,
    group: &str,
    origin: Option<Origin>,
    device: bool,
    relays: Vec<RelayUrl>,
    tried: &[String],
) -> Result<String, Refusal> {
    let page = checked_page(origin).await?;
    let seed = group_named(here.root(), group)?;
    let asked = Linking {
        group: seed.id(),
        device,
        relays,
        origin: page.map(|p| p.as_str().into()),
        ..Linking::default()
    };
    let answer = here.ask(LINKS_ROUTE, &asked).await?;
    tracing::debug!(target: "cli", ok = answer.0, text = answer.1, "cli: link — svrngs running here answered");
    let made: LinkMade = answered(&answer).map_err(|told| -> Refusal {
        let Some(Told { error, kind }) = told else { return Refused::Older.into() };
        tracing::debug!(target: "cli", error, kind, "cli: refused — svrngs running here made no link");
        match kind.as_str() {
            "not_in" => Refused::NotIn(seed.name().clone()).into(),
            "unkept" => Unjudged::Keep(seed.name().to_string()).into(),
            _ => Refused::Meet(error).into(),
        }
    })?;
    // An older svrngs says nothing of a relay: nor does the command, then.
    tracing::debug!(target: "cli", online = ?made.online, ?tried, "cli: link — made by the svrngs running here, which had reached a relay or not, or did not say");
    unreached(made.online.unwrap_or(true), tried);
    let link = Printed::told(made.client, made.https);
    let device = made.device;
    Ok(Said::Linked { seed, link, device }.to_string())
}

/// That this computer has reached no relay, said where it `tried` any,
/// once [`svrngs_node::node::Host::online`] came to none: here, by a
/// `join` that started svrngs ([`serve`]), or by the svrngs running here
/// ([`new_running`], [`link_running`]).
fn unreached(online: bool, tried: &[String]) {
    if !online && !tried.is_empty() {
        tracing::debug!(target: "cli", ?tried, "cli: this computer has reached no relay, and is told so");
        let secs = svrngs_node::node::ONLINE_WAIT.as_secs();
        eprintln!("svrngs: after {secs} s, this computer has not yet reached the server your groups find it through. It goes on, and keeps trying; until it does, they may not reach you (RUST_LOG=debug names the server).");
    }
}
