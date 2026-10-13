// SPDX-License-Identifier: AGPL-3.0-or-later
//! The command surface (DESIGN §5; §8, U2's inventory, the cli.rs row):
//! verbs parsed by hand, no framework, in the shape of upstream's
//! `commonwealth/crates/commonwealth-rails/src/cli.rs` and `main.rs` at
//! `e1a43bcd2` — an error string for a wrong argument, never a guess; exit
//! codes 0, 1 and 3; a tokio runtime built by hand (`main.rs` here).
//!
//! ```text
//! svrngs                                   run for every group this computer holds
//! svrngs new <name> [--as N] [--origin U]
//!                                          make a group, print its link, run
//! svrngs join <link> [--as N] [--no-run]   join a group by its link, run
//! svrngs link <group> [--origin U]
//!                                          a new link to a group, print it, run
//! svrngs share <port> [--name N] [--kind K] [--limit L] [--refuse P]... [--upload M] [--reads K]... [--sign-in S] [--ask W]
//!                                          lend a service on this computer
//! svrngs unshare <name>                    stop lending it
//! svrngs share --folder <dir> --group <group> keep a folder the same on every member's disk,
//!                                          asked of svrngs running here
//! svrngs room <name>...                    serve lends to this computer's network, run
//! svrngs remove <name> --group <group> [--joined W]
//!                                          remove someone from a group, or ask it,
//!                                          asked of svrngs running here
//! svrngs handover <name> --group <group> [--joined W]
//!                                          let someone else decide who is removed,
//!                                          asked of svrngs running here
//! svrngs relay <name> [--open] [--staging] a relay, for the groups this computer is in
//!                                          or everyone ([`crate::relay`])
//! ```
//!
//! `remove` and `handover` ask the node running here, as `share --folder`
//! does (U7; DESIGN §6 "Removal has one decider"; §10, 2026-10-09):
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
//! (bash-person (d)); one with a sign-in, none either, each person signed
//! in by it (mwp-1). The bare command prints what the people
//! of this computer's groups lend, asked of their computers as it starts.
//!
//! `room` serves the lends it names to this computer's own network while
//! it runs, as this computer's person (u6-1, `svrngs_node::room`), so a
//! device in the room reaches each with the lend's own app: it says who
//! can open them, and
//! refuses when this computer has no private address to serve them on.
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
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ed25519_dalek::VerifyingKey;
use std::num::{NonZeroU16, NonZeroU32};

use svrngs_node::folder::FolderAsk;
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::lend::{self, Kind, Lend, LendName, RefusedPath, Upload};
use svrngs_node::node::{Lends, Lent, Node, NodeError};
use svrngs_node::redeem::{ask, redeem, AskRequest, RedeemError, RedeemRequest, Redeemed};
use svrngs_node::removal::{Outcome, RemovalAsk, HANDOVERS_ROUTE, REMOVALS_ROUTE};
use svrngs_node::room;
use svrngs_node::{actor_of, Device, Joined, Person, SigningKey};
use svrngs_node::{MintInvite, Refused as CommitRefused};

use crate::group::{self, Name, Seed, SeedError};
use crate::identity::{self, KeyError};
use crate::issuer;
use crate::relay::{self, Relay};
use crate::share::{self, Share, ShareRefusal};
use svrngs_node::link::{client_link, https_link, Link, Origin, Token};

pub const USAGE: &str = "\
svrngs — keep what your group shares on your own computers.

  svrngs
      Run for every group this computer is in. Leave it running.

  svrngs new <name> [--as <your name>] [--origin <url>]
      Make a group called <name> and print its link: send it to the people
      you want in it. It works for a day, while this computer is on. The
      group's folder is put at ~/svrngs/<name>.
      --as is the name the group knows you by; else your full name as this
      computer shows it, or its user name.
      --origin also prints its links on a page address, from now on, for
      people who open them in a browser. On a phone each app then opens at
      an address of its own under that page's host, which must serve the
      page at every name under it, as group.svrnme.sh does.

  svrngs join <link> [--as <your name>] [--no-run]
      Join the group a link you were sent is for. The computer that sent it
      must be on. --as is the name the group knows you by; else your full
      name as this computer shows it, or its user name. The group's folder is put at ~/svrngs/<group>.
      --no-run stops once you are in, rather than
      run: for a computer where svrngs runs at login, as install.sh keeps it.

  svrngs link <group> [--device] [--origin <url>]
      Print a new link to <group>. Any link you sent before stops letting
      anyone new in; everyone it let in stays. <group> is its name, or the
      first 8 letters of its id when two groups share a name. --origin
      also prints it on a page address, from now on, as `new` does.
      --device makes it a link for another computer or phone of yours: it
      joins as you, asked no name.

  svrngs share <port> [--name <name>] [--kind <kind>] [--limit <n>] [--refuse <path>]... [--upload <Mbit/s>] [--reads <kind>]... [--sign-in <port>] [--ask <words>]
      Lend what runs on <port> on this computer to everyone in your groups:
      each reaches it through this computer, as themselves. It asks for the
      password the service takes on each request, if any; that stays on
      this computer, and nobody you lend it to receives it. For a service
      it knows keeps its own accounts it asks none, and says how to make
      each person an account there: each signs in once. For one it knows
      takes each person from svrngs (Navidrome on 4533, Calibre-Web on
      8083) it asks none, and says what to set in it. --name is what
      they see it as; else its kind. --kind says what it is, when svrngs
      does not know the port. --limit is how many people may use it at
      once; else everyone in your groups. --refuse keeps a path of it from them; say it again
      for more. --upload is this computer's upload, in Mbit/s, as a speed
      test states it: each person using it is told their share of it.
      --reads lets it open, as whoever opens it, what your groups lend of
      <kind>; say it again for more. --sign-in names the port of your
      program that signs each person in to it as themselves; then no
      password is asked, and nobody types one. --ask says, in your words,
      what an app receives from it: then an app uses it only once the
      person it acts for says yes, on their own home page, and stops when
      they say stop.

  svrngs unshare <name>
      Stop lending <name>.

  svrngs share --folder <dir> --group <group>
      Keep <dir> the same on every computer in <group> that shares a folder
      of its name, whenever svrngs runs here, from now on: anyone in it may
      add, change and delete in it, and nobody outside it receives a byte.
      <group> is its name, or the first 8 letters of its id, as `link`
      takes it.

  svrngs room <name>...
      Let anyone on this computer's network (its WiFi) open what your groups
      lend as <name>, as you, while this runs.

  svrngs remove <name> --group <group> [--joined <when>]
      Remove <name> from <group>: every computer and phone of theirs. One
      person decides who is removed — whoever made the group, until they
      hand that on — so from anyone else this asks them, and says who.
      What <name> already has stays with them; nothing new reaches them.
      Asked of svrngs running here. --joined says which of two people
      called <name> you mean: when they joined, as svrngs says it.

  svrngs handover <name> --group <group> [--joined <when>]
      Let <name> decide who is removed from <group> from now on, instead
      of you. Asked of svrngs running here. --joined as for remove.

  svrngs relay <name> [--open] [--staging]
      Relay at https://<name>/ for the computers and phones of the groups
      this computer is in, beside svrngs: a phone reaches a computer only
      through a relay. <name> must lead to this computer, which must take
      connections on TCP ports 80 and 443 and UDP port 7842. Its
      certificate is asked of Let's Encrypt, and kept in the data below.
      The group's page's install line, with --relay <name> after the
      page's address, keeps it running.
      --open relays for every computer and phone that reaches it, as
      svrngs's own relay does, each held to what a phone's film drew
      through a relay at its most, measured; it says how much as it starts.
      --staging asks Let's Encrypt's trial service, whose certificates no
      browser trusts: to try <name> first.

Data:    $SVRNGS_DIR, else ~/.svrngs.
Logging: RUST_LOG, default `error`.";

/// How long a link lets people in: upstream's one invite lapse
/// (`commonwealth-discovery/src/membership.rs:53` at `e1a43bcd2`, a day;
/// DESIGN §8, U2's inventory).
pub const LINK_LIFE_SECS: i64 = 24 * 60 * 60;

/// Where the record is served to this computer's apps: loopback, any free
/// port, which the bare command prints (DESIGN §3; the u3-2 row) and the
/// node writes to `<data dir>/api` (`svrngs_node::api::API_FILE`, u5-1),
/// where an app svrngs did not start finds it.
pub const API: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

/// How long `new` and `link` wait for this computer's address before
/// printing a link, when it has a relay to reach.
const ONLINE_WAIT: Duration = Duration::from_secs(10);

/// What was typed.
#[derive(Debug, PartialEq, Eq)]
pub enum Verb {
    Run,
    New {
        name: Name,
        person: Option<Name>,
        /// `None` prints the client's link alone.
        origin: Option<Origin>,
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
    },
    /// No secret: it is read from a prompt (the module docs).
    Share(Share),
    Unshare(LendName),
    /// `share --folder --group`, asked of svrngs running here: the
    /// directory, absolute, and the group as typed.
    Folder {
        dir: PathBuf,
        group: String,
    },
    /// The lends named, served to this computer's network.
    Room(Vec<LendName>),
    /// `remove` (U7), asked of svrngs running here: the person, the
    /// group as typed, and of two people of one name, when the one meant
    /// joined (bash-same-name).
    Remove {
        person: Name,
        group: String,
        joined: Option<Joined>,
    },
    /// `handover` (U7), likewise.
    Handover {
        person: Name,
        group: String,
        joined: Option<Joined>,
    },
    Relay(Relay),
    Help,
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
    let mut origin = None;
    let (mut name, mut kind, mut limit, mut refused) = (None, None, None, Vec::new());
    let mut upload = None;
    let mut reads = Vec::new();
    let mut sign_in = None;
    let (mut folder, mut group) = (None, None);
    let mut asks = None;
    let mut run = true;
    let mut device = false;
    let (mut open, mut staging) = (false, false);
    let mut joined = None;
    let mut more = Vec::new();
    while let Some(arg) = it.next() {
        let mut value = |flag: &str| it.next().ok_or_else(|| format!("{flag} wants a value"));
        match arg.as_str() {
            "--as" => {
                let raw = value("--as")?;
                person = Some(
                    Name::parse(&raw)
                        .map_err(|why| format!("--as `{raw}` is not a name: {why}"))?,
                );
            }
            "--origin" => {
                let raw = value("--origin")?;
                origin = Some(Origin::parse(&raw).ok_or_else(|| {
                    format!("--origin `{raw}` is not an http(s) address without a '#'")
                })?);
            }
            "--name" => {
                let raw = value("--name")?;
                name = Some(lend_name(&raw).map_err(|why| format!("--name {why}"))?);
            }
            "--kind" => {
                let raw = value("--kind")?;
                kind = Some(Kind::parse(&raw).map_err(|_| {
                    format!("--kind `{raw}` is not a kind: letters, digits, - and _ only")
                })?);
            }
            "--limit" => {
                let raw = value("--limit")?;
                limit = Some(raw.parse::<NonZeroU16>().map_err(|_| {
                    format!("--limit `{raw}` is not a number of people: 1 or more")
                })?);
            }
            "--refuse" => {
                let raw = value("--refuse")?;
                refused
                    .push(RefusedPath::parse(&raw).map_err(|_| {
                        format!("--refuse `{raw}` is not a path: it starts with /")
                    })?);
            }
            "--upload" => {
                let raw = value("--upload")?;
                upload = Some(Upload::from_mbit(raw.parse::<NonZeroU32>().map_err(|_| {
                    format!("--upload `{raw}` is not an upload: a whole number of Mbit/s, 1 or more")
                })?));
            }
            "--reads" => {
                let raw = value("--reads")?;
                reads.push(Kind::parse(&raw).map_err(|_| {
                    format!("--reads `{raw}` is not a kind: letters, digits, - and _ only")
                })?);
            }
            "--sign-in" => {
                let raw = value("--sign-in")?;
                sign_in =
                    Some(share::resolve_target(&raw).map_err(|why| format!("--sign-in {why}"))?);
            }
            "--folder" => folder = Some(value("--folder")?),
            "--group" => group = Some(value("--group")?),
            "--ask" => {
                let raw = value("--ask")?;
                if raw.trim().is_empty() || raw.chars().any(char::is_control) {
                    return Err("--ask wants words: what an app receives from it".into());
                }
                asks = Some(raw);
            }
            "--joined" if matches!(verb.as_str(), "remove" | "handover") => {
                joined = Some(joined_of(&value("--joined")?)?);
            }
            "--no-run" if verb == "join" => run = false,
            "--device" if verb == "link" => device = true,
            "--open" if verb == "relay" => open = true,
            "--staging" if verb == "relay" => staging = true,
            "-h" | "--help" => return Ok(Verb::Help),
            other if other.starts_with('-') => return Err(format!("unknown argument `{other}`")),
            other if positional.is_none() => positional = Some(other.to_string()),
            other if verb == "room" => more.push(other.to_string()),
            other => return Err(format!("unexpected argument `{other}`")),
        }
    }
    if person.is_some() && !matches!(verb.as_str(), "new" | "join") {
        return Err("--as is a `new` and `join` flag".into());
    }
    if origin.is_some() && !matches!(verb.as_str(), "new" | "link") {
        return Err("--origin is a `new` and `link` flag".into());
    }
    if (name.is_some()
        || kind.is_some()
        || limit.is_some()
        || !refused.is_empty()
        || upload.is_some()
        || !reads.is_empty()
        || sign_in.is_some()
        || asks.is_some())
        && verb != "share"
    {
        return Err(
            "--name, --kind, --limit, --refuse, --upload, --reads, --sign-in and --ask are `share` flags"
                .into(),
        );
    }
    let removes = matches!(verb.as_str(), "remove" | "handover");
    if removes && folder.is_none() {
        let raw = positional.ok_or(format!(
            "{verb} wants a name: `svrngs {verb} <name> --group <group>`"
        ))?;
        let person = Name::parse(&raw).map_err(|why| format!("`{raw}` is not a name: {why}"))?;
        let group = group.ok_or(format!(
            "{verb} wants --group: `svrngs {verb} <name> --group <group>`"
        ))?;
        return Ok(match verb.as_str() {
            "remove" => Verb::Remove {
                person,
                group,
                joined,
            },
            _ => Verb::Handover {
                person,
                group,
                joined,
            },
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
        ("share", None, Some(_)) => return Err("--group is a `share --folder` flag".into()),
        (_, None, None) => {}
        _ => return Err("--folder is a `share` flag, and --group a `share --folder`, `remove` and `handover` one".into()),
    }
    match verb.as_str() {
        "new" => {
            let raw = positional.ok_or("new wants a name: `svrngs new <name>`")?;
            let name = Name::parse(&raw).map_err(|why| format!("`{raw}` is not a name: {why}"))?;
            Ok(Verb::New {
                name,
                person,
                origin,
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
        }),
        "share" => {
            let raw = positional.ok_or("share wants a port: `svrngs share <port>`")?;
            Ok(Verb::Share(Share {
                port: share::resolve_target(&raw)?,
                name,
                kind,
                limit,
                refused,
                upload,
                reads,
                sign_in,
                asks,
            }))
        }
        "unshare" => {
            let raw = positional.ok_or("unshare wants a name: `svrngs unshare <name>`")?;
            Ok(Verb::Unshare(lend_name(&raw)?))
        }
        "room" => {
            let first = positional.ok_or("room wants what to serve: `svrngs room <name>...`")?;
            let names = std::iter::once(first)
                .chain(more)
                .map(|raw| lend_name(&raw));
            Ok(Verb::Room(names.collect::<Result<_, _>>()?))
        }
        "relay" => {
            let raw =
                positional.ok_or("relay wants the name it is reached at: `svrngs relay <name>`")?;
            let name = relay::name(&raw)?;
            let at = relay::At::Name { name, staging };
            Ok(Verb::Relay(Relay { at, open }))
        }
        "help" | "-h" | "--help" => Ok(Verb::Help),
        other => Err(format!("unknown command `{other}`")),
    }
}

/// Why the command stopped. Its words are a person's (DESIGN §1); what is
/// behind each is traced at debug where it is raised.
#[derive(Debug, thiserror::Error)]
pub enum Refusal {
    #[error("no place for svrngs's data: set $SVRNGS_DIR, or $HOME for ~/.svrngs")]
    NoDataDir,
    #[error("cannot make {0}: {1}")]
    DataDir(PathBuf, std::io::Error),
    #[error(
        "svrngs is already running on this computer ({0} is held); stop it, then run this again"
    )]
    Busy(PathBuf),
    #[error(transparent)]
    Key(#[from] KeyError),
    #[error("this computer cannot be reached from here yet (RUST_LOG=debug says why)")]
    Start,
    #[error("who are you in {0}? say `svrngs new {0} --as <your name>`")]
    NoName(Name),
    #[error("who are you? open the link again with `svrngs join <link> --as <your name>`")]
    NoNameToJoin,
    #[error("this link lets nobody new in: ask whoever sent it for a new one")]
    LinkRefused,
    #[error("someone in {group} already goes by {holder}, so you were not let in as them. If that's you, add this computer from one of yours: `svrngs link {group} --device` there makes a link it joins as you by. Else join under another name, with `--as <your name>` after the link")]
    NameTaken { holder: Name, group: Name },
    #[error(
        "the computer that sent this link cannot be reached; it may be off, so try again later"
    )]
    IssuerAway,
    #[error("cannot keep {0} on this computer (RUST_LOG=debug says why)")]
    Keep(Name),
    #[error("this computer is in no group called {0}")]
    NoGroup(String),
    #[error("more than one group here is called {name}; say which by the start of its id: {}", ids.join(", "))]
    Ambiguous { name: String, ids: Vec<String> },
    #[error("you are not in {0} on this computer, so you cannot send its link")]
    NotIn(Name),
    #[error(transparent)]
    Share(#[from] ShareRefusal),
    #[error("cannot read the password: {0}")]
    Prompt(std::io::Error),
    #[error("cannot keep {0} on this computer (RUST_LOG=debug says why)")]
    KeepLend(LendName),
    #[error("you lend nothing called {name} from this computer{}", lent_now(.lent))]
    NotLent { name: LendName, lent: Vec<LendName> },
    #[error("{0} gives each app no address of its own, so on a phone every app would open where your group's page keeps who each person is. Use a host that serves your page at every name under it, as group.svrnme.sh does")]
    NoAppAddress(String),
    #[error("this computer has no address on a home network, so nothing near it can reach it: join its WiFi or plug it in, then run this again")]
    NoRoomAddress,
    #[error(transparent)]
    Room(#[from] svrngs_node::room::RoomError),
    #[error("svrngs is not running on this computer, and only it syncs a folder: start it, then run this again")]
    NotRunning,
    #[error("the folder was refused: {0}")]
    Folder(String),
    #[error("cannot ask svrngs running here: no HTTP client ({0})")]
    NoClient(String),
    #[error("cannot tell from here whether {0} gives each app an address of its own: the page does not open there (RUST_LOG=debug says why); try again")]
    CannotCheck(String),
    /// `remove` or `handover` refused, in the words [`Said::Removal`] says.
    #[error("{0}")]
    Removal(String),
    #[error("{0} cannot be changed on this computer (RUST_LOG=debug says why)")]
    Unchanged(Name),
    #[error("this computer is in no group, so a relay for its groups would let nobody through: join one first, or say --open to relay for everyone")]
    RelayForNobody,
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
    if lent.is_empty() {
        String::new()
    } else {
        let names: Vec<&str> = lent.iter().map(LendName::as_str).collect();
        format!("; you lend {}", names.join(", "))
    }
}

impl Refusal {
    /// 1 for a refusal, 3 for a precondition absent (the module docs).
    pub fn exit_code(&self) -> u8 {
        match self {
            Refusal::NoName(_)
            | Refusal::NoNameToJoin
            | Refusal::LinkRefused
            | Refusal::NameTaken { .. }
            | Refusal::NoGroup(_)
            | Refusal::Ambiguous { .. }
            | Refusal::NotIn(_)
            | Refusal::Share(_)
            | Refusal::NotLent { .. }
            | Refusal::NoAppAddress(_)
            | Refusal::Room(_)
            | Refusal::Folder(_)
            | Refusal::Removal(_)
            | Refusal::RelayForNobody
            | Refusal::Busy(_) => 1,
            Refusal::NoDataDir
            | Refusal::DataDir(..)
            | Refusal::Key(_)
            | Refusal::Start
            | Refusal::IssuerAway
            | Refusal::Prompt(_)
            | Refusal::KeepLend(_)
            | Refusal::CannotCheck(_)
            | Refusal::NoRoomAddress
            | Refusal::NotRunning
            | Refusal::NoClient(_)
            | Refusal::Unchanged(_)
            | Refusal::RelayStart(_)
            | Refusal::Keep(_) => 3,
        }
    }
}

/// The data dir: `$SVRNGS_DIR`, else `~/.svrngs`.
pub fn data_dir() -> Result<PathBuf, Refusal> {
    if let Some(dir) = std::env::var_os("SVRNGS_DIR").filter(|d| !d.is_empty()) {
        tracing::debug!(target: "cli", dir = ?dir, "cli: data dir from $SVRNGS_DIR");
        return Ok(PathBuf::from(dir));
    }
    let home = std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .ok_or(Refusal::NoDataDir)?;
    tracing::debug!(target: "cli", home = ?home, "cli: data dir ~/.svrngs");
    Ok(PathBuf::from(home).join(".svrngs"))
}

/// Hold `<root>/lock` for as long as the file lives: one svrngs per data
/// dir, so each group's record has one writer (RUST_ARCH 3).
pub fn claim(root: &Path) -> Result<File, Refusal> {
    std::fs::create_dir_all(root).map_err(|e| Refusal::DataDir(root.to_path_buf(), e))?;
    let path = root.join("lock");
    let file = File::create(&path).map_err(|e| Refusal::DataDir(path.clone(), e))?;
    match file.try_lock() {
        Ok(()) => {
            tracing::debug!(target: "cli", lock = %path.display(), "cli: data dir claimed");
            Ok(file)
        }
        Err(e) => {
            tracing::debug!(target: "cli", lock = %path.display(), error = %e, "cli: refused — another svrngs holds the data dir");
            Err(Refusal::Busy(path))
        }
    }
}

/// The whole program, `Help` apart: claim the data dir, load this
/// computer's key, start the node, do the verb, and run until Ctrl-C.
/// `share` declares its lend first, and runs only when svrngs is not
/// running here already; `unshare` only withdraws one.
pub async fn run(command: Verb, root: PathBuf, relay: &RelayConfig) -> Result<(), Refusal> {
    let command = match command {
        Verb::Help => {
            println!("{USAGE}");
            return Ok(());
        }
        Verb::Relay(asked) => return relay::run(asked, &root).await,
        Verb::Unshare(name) => {
            print!("{}", unshare(&root, &name)?);
            return Ok(());
        }
        Verb::Folder { dir, group } => {
            println!("{}", folder(&root, dir, &group).await?);
            return Ok(());
        }
        Verb::Remove {
            person,
            group,
            joined,
        } => {
            let said = removal(&root, REMOVALS_ROUTE, person, joined, &group).await?;
            print!("{said}");
            return Ok(());
        }
        Verb::Handover {
            person,
            group,
            joined,
        } => {
            let said = removal(&root, HANDOVERS_ROUTE, person, joined, &group).await?;
            print!("{said}");
            return Ok(());
        }
        Verb::Share(asked) => {
            let lend = share::lend_of(&asked, share::listens, share::well_known_kind(asked.port))?;
            let accounts = share::accounts_of(&lend.kind).filter(|_| lend.sign_in.is_none());
            let secret = match accounts {
                _ if lend.sign_in.is_some() => {
                    tracing::debug!(target: "cli", lend = %lend.name, "cli: share — no password asked: its sign-in signs each person in");
                    None
                }
                Some(share::Signs::Accounts(_)) => {
                    tracing::debug!(target: "cli", lend = %lend.name, "cli: share — no password asked: its people sign in to it once, with accounts its lender makes");
                    None
                }
                Some(share::Signs::Header(_)) => {
                    tracing::debug!(target: "cli", lend = %lend.name, "cli: share — no password asked: it takes each person from the door's header");
                    None
                }
                None => prompt_secret(&lend.name)?,
            };
            share_lend(&root, &lend, secret.as_deref())?;
            let claimed = match claim(&root) {
                Err(Refusal::Busy(lock)) => {
                    tracing::debug!(target: "cli", lock = %lock.display(), "cli: share — svrngs runs here already, and reads the lend from its declaration");
                    None
                }
                claimed => Some(claimed?),
            };
            print!(
                "{}",
                Said::Shared {
                    lend,
                    secret: secret.is_some(),
                    accounts,
                    running: claimed.is_none()
                }
            );
            let Some(claimed) = claimed else {
                return Ok(());
            };
            println!();
            return start(Verb::Run, root, relay, claimed).await;
        }
        other => other,
    };
    let claimed = claim(&root)?;
    start(command, root, relay, claimed).await
}

/// `share --folder --group`, asked of the node running here
/// (`svrngs_node::folder::FolderAsk`): it alone writes this computer's acts,
/// the journal's next index being its process's (svrngs-node journal.rs:16-27).
/// The group is named as `link` names it ([`group::named`]).
async fn folder(root: &Path, dir: PathBuf, reference: &str) -> Result<String, Refusal> {
    let group = group_named(root, reference)?.id();
    let asked = FolderAsk { dir, group };
    let (ok, text) = ask_running(root, "/v1/folders", &asked).await?;
    tracing::debug!(target: "cli", ?asked, ok, "cli: the folder's ask answered");
    ok.then_some(text.clone()).ok_or(Refusal::Folder(text))
}

/// `remove` and `handover` (U7), asked of svrngs running here at `route`
/// for the group `reference` names, as `link` names it: what happened, in
/// a person's words, or the refusal.
async fn removal(
    root: &Path,
    route: &str,
    person: Name,
    joined: Option<Joined>,
    reference: &str,
) -> Result<Said, Refusal> {
    let seed = group_named(root, reference)?;
    let asked = RemovalAsk {
        group: seed.id(),
        person: person.as_str().to_string(),
        joined,
    };
    let (_, text) = ask_running(root, route, &asked).await?;
    let outcome: Outcome = serde_json::from_str(&text).map_err(|e| {
        tracing::debug!(target: "cli", error = %e, text, "cli: refused — svrngs running here answered no outcome");
        Refusal::Unchanged(seed.name().clone())
    })?;
    tracing::debug!(target: "cli", route, group = %seed.id(), %person, ?outcome, "cli: a removal asked");
    let said = Said::Removal {
        group: seed.name().clone(),
        person,
        outcome,
    };
    match &said {
        Said::Removal {
            outcome: Outcome::Out | Outcome::Asked { .. } | Outcome::Handed,
            ..
        } => Ok(said),
        Said::Removal {
            outcome: Outcome::Fault,
            ..
        } => Err(Refusal::Unchanged(seed.name().clone())),
        _ => Err(Refusal::Removal(said.to_string().trim_end().to_string())),
    }
}

/// POST `body` to `route` of svrngs running here, at the API it wrote:
/// whether it answered success, and its body.
async fn ask_running(
    root: &Path,
    route: &str,
    body: &impl serde::Serialize,
) -> Result<(bool, String), Refusal> {
    let api = std::fs::read_to_string(root.join(svrngs_node::api::API_FILE)).unwrap_or_default();
    let body = serde_json::to_string(body).expect("an ask is JSON");
    let to = format!("{}{route}", api.trim());
    let client = http()
        .and_then(|b| b.build().map_err(|e| e.to_string()))
        .map_err(|error| {
            tracing::debug!(target: "cli", error, "cli: refused — no client to ask svrngs running here with");
            Refusal::NoClient(error)
        })?;
    let sent = client.post(to).header("content-type", "application/json");
    let answer = sent.body(body).send().await.map_err(|e| {
        tracing::debug!(target: "cli", api, error = %e, "cli: refused — no svrngs answers at the API it wrote");
        Refusal::NotRunning
    })?;
    let ok = answer.status().is_success();
    let text = answer.text().await.unwrap_or_default();
    tracing::debug!(target: "cli", route, ok, "cli: svrngs running here answered");
    Ok((ok, text))
}

/// The command's one HTTP client, iroh's reqwest 0.13: its
/// `rustls-no-provider` brings no TLS provider, and a client built without
/// one panics, so its TLS is given ring and this computer's verifier here.
fn http() -> Result<reqwest13::ClientBuilder, String> {
    let ring = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    rustls::ClientConfig::builder_with_provider(ring)
        .with_safe_default_protocol_versions()
        .and_then(rustls_platform_verifier::BuilderVerifierExt::with_platform_verifier)
        .map(|tls| {
            reqwest13::Client::builder().tls_backend_preconfigured(tls.with_no_client_auth())
        })
        .map_err(|e| e.to_string())
}

/// Start the node over the data dir `claimed` holds, do the verb, and run
/// until Ctrl-C.
async fn start(
    command: Verb,
    root: PathBuf,
    relay: &RelayConfig,
    _claimed: File,
) -> Result<(), Refusal> {
    let key = identity::load_or_generate_node_key(&root)?;
    let node = Node::start(root.clone(), key.clone(), relay, group::seed_text, API)
        .await
        .map_err(|e: NodeError| {
            tracing::debug!(target: "cli", error = %e, "cli: refused — the node did not start");
            Refusal::Start
        })?;
    let ran = serve(command, &root, &node, &key, relay).await;
    // iroh says at error when an endpoint is dropped open, so it is closed
    // first, on a refusal as on Ctrl-C.
    node.endpoint().close().await;
    ran
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
/// give the page (u5-2).
#[expect(
    clippy::disallowed_methods,
    reason = "draws a name nobody made, to ask the page's host for the page there"
)]
pub async fn checked_page(page: Origin) -> Result<Origin, Refusal> {
    // An address in brackets is IPv6's, a literal no name falls under.
    let named = place(&page).filter(|(_, a)| {
        let host = a.split(':').next().unwrap_or_default();
        !a.starts_with('[') && !host.is_empty() && host.parse::<std::net::IpAddr>().is_err()
    });
    let no = || Refusal::NoAppAddress(page.as_str().into());
    let Some((scheme, authority)) = named else {
        tracing::debug!(target: "cli", page = page.as_str(), "cli: refused — the page is at an address no name falls under");
        return Err(no());
    };
    let unjudged = || Refusal::CannotCheck(page.as_str().into());
    let client = http()
        .and_then(|b| {
            let b = b.timeout(std::time::Duration::from_secs(10));
            b.build().map_err(|e| e.to_string())
        })
        .map_err(|error| {
            tracing::debug!(target: "cli", error, "cli: not judged — no client to ask the page's host with");
            unjudged()
        })?;
    let get = |url: String| {
        let client = &client;
        async move {
            let got = match client.get(&url).send().await {
                Ok(r) if r.status().is_success() => r.bytes().await.map_err(|e| format!("{e:?}")),
                Ok(r) => Err(r.status().to_string()),
                Err(e) => Err(format!("{e:?}")),
            };
            tracing::debug!(target: "cli", url, got = ?got.as_ref().map(|b| b.len()), "cli: asked for the page");
            got.ok()
        }
    };
    let Some(ours) = get(page.as_str().to_string()).await else {
        return Err(unjudged());
    };
    let mut nobodys = [0u8; 16];
    getrandom::fill(&mut nobodys).expect("failed to draw a name nobody made");
    let theirs = get(format!("{scheme}://{}.{authority}/", hex::encode(nobodys))).await;
    if theirs.as_ref() != Some(&ours) {
        tracing::debug!(target: "cli", page = page.as_str(), served = ?theirs.map(|b| b.len()), "cli: refused — a name nobody made under the page's host does not serve the page");
        return Err(no());
    }
    tracing::debug!(target: "cli", page = page.as_str(), authority, "cli: a name nobody made under the page's host serves the page, so each app opens at an address of its own");
    Ok(page)
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
    fn of(token: &Token, page: Option<&Origin>) -> Self {
        Self {
            client: client_link(token),
            https: page.map(|p| https_link(p, token)),
            page: page.cloned(),
        }
    }
}

impl std::fmt::Display for Printed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  {}", self.client)?;
        if let Some(https) = &self.https {
            writeln!(f)?;
            writeln!(f, "or, to open it in a browser:")?;
            writeln!(f)?;
            writeln!(f, "  {https}")?;
        }
        if let Some((_, host)) = self.page.as_ref().and_then(place) {
            writeln!(f)?;
            writeln!(f, "On a phone, each app opens at an address of its own under {host}, which can see which app is opened.")?;
        }
        Ok(())
    }
}

/// What a verb tells the person, in DESIGN §1's words alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Said {
    /// `new`: the group made, and its first link.
    Made { seed: Seed, link: Printed },
    /// `link`: a new link to a group.
    /// A new link; a device link's person when it is one (bash-devices).
    Linked {
        seed: Seed,
        link: Printed,
        device: Option<Person>,
    },
    /// `join`: let in by the link's issuer, as `person`.
    Joined { seed: Seed, person: Name },
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
        api: SocketAddr,
        lends: Lends,
    },
    /// `share`: `lend` declared, with a secret or none; `accounts`, how it
    /// signs each person in and what its lender does for that, when the
    /// table of well-known ports says;
    /// `running` when svrngs runs here already, so it is lent now.
    Shared {
        lend: Lend,
        secret: bool,
        accounts: Option<share::Signs>,
        running: bool,
    },
    /// `unshare`: no longer lent.
    Unshared(LendName),
    /// `room`: each lend served to this computer's network, and where.
    Room(Vec<(Lent, SocketAddr)>),
    /// `remove` or `handover` (U7): what happened to `person` in `group`.
    Removal {
        group: Name,
        person: Name,
        outcome: Outcome,
    },
}

impl std::fmt::Display for Said {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Said::Made { seed, link } => {
                writeln!(f, "Made {}, with you as {}.", seed.name(), seed.founder())?;
                writeln!(f, "Send this link to the people you want in it. It works for a day, while this computer is on:")?;
                writeln!(f)?;
                write!(f, "{link}")
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
                writeln!(f, "It works for a day, while this computer is on:")?;
                writeln!(f)?;
                write!(f, "{link}")
            }
            Said::Joined { seed, person } => {
                writeln!(f, "You're in {}, as {person}.", seed.name())
            }
            Said::AlreadyIn(seed) => {
                writeln!(f, "You're in {} already, on this computer.", seed.name())
            }
            Said::Folder {
                group,
                dir,
                placed: Ok(()),
            } => writeln!(
                f,
                "{group}'s folder is {}: what anyone in {group} puts there reaches this computer, and what you put there reaches theirs.",
                dir.display()
            ),
            Said::Folder {
                group,
                dir,
                placed: Err(why),
            } => writeln!(
                f,
                "{group}'s folder is not at {}: {why}. `svrngs share --folder <dir> --group {group}` keeps it where you choose.",
                dir.display()
            ),
            Said::Running { groups, api, lends } => {
                if groups.is_empty() {
                    writeln!(
                        f,
                        "This computer is in no group yet: make one with `svrngs new <name>`."
                    )?;
                } else {
                    let names: Vec<&str> = groups.iter().map(Name::as_str).collect();
                    writeln!(f, "Running for {}.", names.join(", "))?;
                }
                writeln!(
                    f,
                    "Apps on this computer reach your groups at http://{api}."
                )?;
                writeln!(
                    f,
                    "Your groups, and what apps use of them, open at http://svrngs.localhost:{}/.",
                    api.port()
                )?;
                for lent in &lends.lent {
                    let (name, kind) = (&lent.offered.name, &lent.offered.kind);
                    let what = if name.as_str() == kind.as_str() {
                        name.to_string()
                    } else {
                        format!("{name} ({kind})")
                    };
                    writeln!(
                        f,
                        "{}'s {what}, in your browser: {}",
                        lent.person, lent.address
                    )?;
                }
                let unlisted = &lends.unlisted;
                for person in &unlisted.unanswered {
                    writeln!(
                        f,
                        "{person}'s computer did not answer, so what they lend is not listed."
                    )?;
                }
                for person in &unlisted.unheard {
                    writeln!(f, "{person}'s computer has not heard of this computer yet, so what they lend is not listed yet.")?;
                }
                for person in &unlisted.refused {
                    writeln!(f, "{person}'s computer answered without saying what they lend, so it is not listed.")?;
                }
                Ok(())
            }
            Said::Shared {
                lend,
                secret,
                accounts,
                running,
            } => {
                let (name, port) = (&lend.name, lend.port);
                match lend.limit {
                    Some(n) => writeln!(f, "Lent {name} (port {port}) to everyone in your groups, up to {n} {} at once.",
                        if n.get() == 1 { "person" } else { "people" })?,
                    None => writeln!(f, "Lent {name} (port {port}) to everyone in your groups, as many at once as want it. \
                        To let fewer in at once, change it on your home page, or `svrngs share {port} --limit <number>`.")?,
                }
                match accounts {
                    Some(share::Signs::Accounts(how)) => writeln!(
                        f,
                        "{} asks each person to sign in once, with an account you make them. In {}: {how}. Then tell each person theirs.",
                        lend.name, lend.kind
                    )?,
                    Some(share::Signs::Header(what)) => writeln!(
                        f,
                        "{} signs each person in as themselves, with nothing typed, once it takes them from svrngs: in {}, {what}.",
                        lend.name, lend.kind
                    )?,
                    None => {}
                }
                if let Some(port) = lend.sign_in {
                    writeln!(
                        f,
                        "Each person is signed in to it as themselves by what answers on port {port}, with nothing typed."
                    )?;
                }
                for path in &lend.refused {
                    writeln!(f, "They cannot open {}.", path.as_str())?;
                }
                if let Some(words) = &lend.asks {
                    writeln!(
                        f,
                        "An app uses it only once the person it acts for says yes to: \"{words}\"."
                    )?;
                }
                for kind in &lend.reads {
                    writeln!(
                        f,
                        "It can open the {kind} your groups lend, as whoever opens it."
                    )?;
                }
                if let Some(upload) = lend.upload {
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
                if *running {
                    writeln!(
                        f,
                        "svrngs is running here already, so they can open it now."
                    )?;
                }
                writeln!(f, "`svrngs unshare {}` stops lending it.", lend.name)
            }
            Said::Unshared(name) => writeln!(f, "Stopped lending {name}."),
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
                    Outcome::NotInGroup => writeln!(f, "Nobody called {person} is in {group}."),
                    Outcome::Which { joined } => {
                        let each: Vec<String> = joined.iter().map(|j| format!("`--joined {}`", joined_said(j))).collect();
                        writeln!(f, "svrngs cannot tell which person called {person} in {group} you mean, so nothing changed. Say which by when they joined, adding one of these to the command: {}.", each.join(", "))
                    }
                    Outcome::Decides => writeln!(f, "{person} decides who is removed from {group}, so cannot be removed: hand that to someone else first, with `svrngs handover <name> --group {group}`."),
                    Outcome::NotDecider { decides: who } => writeln!(f, "Only {} can let someone else decide who is removed from {group}.", decides(who)),
                    Outcome::NotDeciderUnread => writeln!(f, "You do not decide who is removed from {group}, and this computer could not read who is in the group, so cannot say who does (RUST_LOG=debug says why)."),
                    Outcome::NotIn => writeln!(f, "You are no longer in {group}."),
                    Outcome::Fault => writeln!(f, "{group} cannot be changed on this computer (RUST_LOG=debug says why)"),
                }
            }
            Said::Room(served) => {
                writeln!(f, "Anyone on this computer's network (its WiFi) can open these, as you, while this runs; nobody off it can:")?;
                for (lent, at) in served {
                    writeln!(f, "{}'s {}: http://{at}/", lent.person, lent.offered.name)?;
                }
                Ok(())
            }
        }
    }
}

/// How `--joined` takes when a person joined and a refusal says it, in
/// this computer's time to the second: the home page's form too (home.js
/// `people`, whose `sv-SE` locale writes it).
const WHEN: &str = "%Y-%m-%d %H:%M:%S";

/// `--joined`: `started`, for whoever started the group, or a time in
/// [`WHEN`]'s form on this computer's clock, as [`joined_said`] says it.
fn joined_of(raw: &str) -> Result<Joined, String> {
    if raw == "started" {
        return Ok(Joined::Started);
    }
    (chrono::NaiveDateTime::parse_from_str(raw, WHEN).ok())
        .and_then(|t| t.and_local_timezone(chrono::Local).single())
        .map(|t| Joined::At(t.timestamp()))
        .ok_or_else(|| format!("--joined `{raw}` is not when someone joined: `started`, or a time as svrngs says it, such as \"2026-10-12 14:03:22\""))
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

/// The header a lend's password is sent in: the one HTTP gives a
/// credential (RFC 9110 §11.6.2).
pub const SECRET_HEADER: &str = "authorization";

/// Ask for the lend's password on this terminal, never echoed, and read
/// one line from standard input: `None` for an empty line or none. The
/// value is never traced, only whether there is one.
fn prompt_secret(name: &LendName) -> Result<Option<String>, Refusal> {
    use std::io::{BufRead, IsTerminal, Write};
    let stdin = std::io::stdin();
    let terminal = stdin.is_terminal();
    eprint!(
        "If {name} takes a password on each request, paste what it takes in its Authorization header, then press Enter (Enter alone for none): "
    );
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
        Refusal::Prompt(e)
    })?;
    let secret = line.trim().to_string();
    tracing::debug!(target: "cli", lend = %name, given = !secret.is_empty(), terminal, "cli: share — the password asked for");
    Ok((!secret.is_empty()).then_some(secret))
}

/// `svrngs share`'s write: `lend` declared in the data dir at `root`, with
/// `secret` sent as [`SECRET_HEADER`], or none.
pub fn share_lend(root: &Path, lend: &Lend, secret: Option<&str>) -> Result<(), Refusal> {
    lend::declare(root, lend, secret.map(|s| (SECRET_HEADER, s))).map_err(|e| {
        tracing::debug!(target: "cli", lend = %lend.name, error = %e, "cli: share — the lend cannot be kept");
        Refusal::KeepLend(lend.name.clone())
    })
}

/// `svrngs unshare`: withdraw the lend `name` from the data dir at `root`.
pub fn unshare(root: &Path, name: &LendName) -> Result<Said, Refusal> {
    let keep = |e: lend::LendError| {
        tracing::debug!(target: "cli", lend = %name, error = %e, "cli: unshare — the lend cannot be withdrawn");
        Refusal::KeepLend(name.clone())
    };
    if lend::withdraw(root, name).map_err(keep)? {
        return Ok(Said::Unshared(name.clone()));
    }
    let lent = lend::all(root)
        .map_err(keep)?
        .into_iter()
        .filter_map(Result::ok)
        .map(|l| l.name)
        .collect();
    tracing::debug!(target: "cli", lend = %name, "cli: unshare — refused, nothing lent by that name");
    Err(Refusal::NotLent {
        name: name.clone(),
        lent,
    })
}

/// What the bare command prints: the groups held, where apps reach them,
/// and what the people in them lend, asked now.
pub async fn running(root: &Path, node: &Node) -> Said {
    Said::Running {
        groups: group::groups(root)
            .iter()
            .map(|s| s.name().clone())
            .collect(),
        api: node.api_addr(),
        lends: node.lends().await,
    }
}

/// A verb on a started node, then the wait for Ctrl-C.
async fn serve(
    command: Verb,
    root: &Path,
    node: &Node,
    key: &SigningKey,
    relay: &RelayConfig,
) -> Result<(), Refusal> {
    let _room;
    let stays = !matches!(command, Verb::Join { run: false, .. });
    let said = match command {
        Verb::New {
            name,
            person,
            origin,
        } => {
            let person = match person {
                Some(p) => p,
                None => user_name().ok_or_else(|| Refusal::NoName(name.clone()))?,
            };
            let page = match origin {
                Some(page) => Some(checked_page(page).await?),
                None => None,
            };
            let (seed, link) = new_group(node, name, person, page.as_ref(), relay).await?;
            println!(
                "{}",
                Said::Made {
                    seed: seed.clone(),
                    link
                }
            );
            // The founder's folder, where every joiner's is, so what the
            // founder puts in it is shared with no `share --folder` (Demo
            // C's dry run, finding 7).
            folder_at_join(root, &seed).await
        }
        Verb::Join { link, person, .. } => {
            let person = match person {
                Some(p) => p,
                None => user_name().ok_or(Refusal::NoNameToJoin)?,
            };
            let joined = join_group(node, key, &link, &person).await?;
            print!("{joined}");
            let (Said::Joined { seed, .. } | Said::AlreadyIn(seed)) = joined else {
                unreachable!("join_group says joined or in already")
            };
            folder_at_join(root, &seed).await
        }
        Verb::Link {
            group,
            origin,
            device,
        } => {
            let page = match origin {
                Some(page) => Some(checked_page(page).await?),
                None => None,
            };
            let (seed, link, device) = link_to(node, &group, page.as_ref(), relay, device).await?;
            Said::Linked { seed, link, device }
        }
        Verb::Run => running(root, node).await,
        Verb::Room(names) => {
            let listed: Vec<_> = node.endpoint().addr().ip_addrs().map(|a| a.ip()).collect();
            let at = room::pick(&listed, room::route()).ok_or(Refusal::NoRoomAddress)?;
            _room = room::open(node, at, &names).await?;
            Said::Room(_room.served.clone())
        }
        Verb::Help
        | Verb::Share(_)
        | Verb::Unshare(_)
        | Verb::Relay(_)
        | Verb::Folder { .. }
        | Verb::Remove { .. }
        | Verb::Handover { .. } => {
            unreachable!("answered in `run`")
        }
    };
    print!("{said}");
    println!();
    if !stays {
        tracing::debug!(target: "cli", "cli: join --no-run — stopping once in");
        return Ok(());
    }
    println!("Leave this running; Ctrl-C stops it.");
    svrngs_node::node::asked_to_stop().await;
    tracing::debug!(target: "cli", "cli: stopping");
    Ok(())
}

/// `svrngs join`: ask `link`'s issuer what it is for, proving the link's
/// key; believe the seed answered only when it hashes to the group named
/// (`group::answered`); redeem the link there as `person`, signing with
/// `key`, this computer's; keep the group's seed once let in, and the
/// origin of an https link; sync once from the issuer, whom this
/// computer's copy of the register may not know yet (DESIGN §8, U2).
/// Nothing is written until the issuer has let this computer in, so a
/// refused link leaves nothing behind.
pub async fn join_group(
    node: &Node,
    key: &SigningKey,
    link: &Link,
    person: &Name,
) -> Result<Said, Refusal> {
    let issuer = issuer::addr(&link.token);
    let link_key = link.token.key();
    let asked = ask(
        node.endpoint(),
        issuer.clone(),
        &AskRequest::sign(&link_key, key),
    )
    .await
    .map_err(|e| not_let_in(e, None))?;
    let Some(seed) = group::answered(&asked.group, &asked.seed) else {
        tracing::debug!(target: "cli", group = %asked.group, "cli: join — the issuer's answer is not believed, so nothing is redeemed");
        return Err(Refusal::LinkRefused);
    };
    let id = seed.id();
    if group::groups(node.rail().root())
        .iter()
        .any(|s| s.id() == id)
    {
        tracing::debug!(target: "cli", group = %id, "cli: join — this computer holds the group already, so nothing is redeemed");
        return Ok(Said::AlreadyIn(seed));
    }
    // A device link's person is who this computer joins as, asked no name
    // (bash-devices); else the name given.
    let person = asked.person.unwrap_or_else(|| Person::from(person.clone()));
    tracing::debug!(target: "cli", group = %id, %person, "cli: join — redeeming as this person");
    let request = RedeemRequest::sign(
        &link_key,
        key,
        &id,
        &asked.invite,
        &person,
        Device::Computer,
    );
    let redeemed = redeem(node.endpoint(), issuer.clone(), &request)
        .await
        .map_err(|e| not_let_in(e, Some(seed.name())))?;
    tracing::debug!(target: "cli", group = %id, redeemed = ?redeemed, "cli: join — let in");
    keep_group(node, &seed, link.origin.as_ref())?;
    if link.origin.is_none() {
        tracing::debug!(target: "cli", group = %id, "cli: join — a client's link, so no origin is kept");
    }
    // Its home relay reached before the exchange, so the joiner says it
    // there and the issuer can dial it back; else not before its first
    // round (demo/runs/relay-in-record/2026-10-12/: 60 s).
    wait_online(node, "joining").await;
    let synced = node.sync_with(&id, &issuer).await;
    match &synced.stop {
        None => {
            tracing::debug!(target: "cli", group = %id, pulled = synced.pulled, "cli: join — synced once from the issuer")
        }
        Some(stop) => {
            tracing::warn!(target: "cli", group = %id, stop = ?stop, pulled = synced.pulled, "cli: join — let in, and the first sync stopped; the round carries on")
        }
    }
    if matches!(redeemed, Redeemed::AlreadyIn) {
        tracing::debug!(target: "cli", group = %id, "cli: join — the issuer says this key stood already; this computer now keeps its copy");
    }
    Ok(Said::Joined {
        seed,
        person: person.name().clone(),
    })
}

/// Where `new` and joining put a group's folder (v1-onboarding (d);
/// bash-first-minute (d)): `~/svrngs/`
/// and the group's name, which every member who joined derives alike, so
/// their folders are one; `/`, `\` and `:` read as `-` and leading dots
/// dropped, so the name stays one directory under it, else the id's start.
fn folder_place(seed: &Seed) -> Option<PathBuf> {
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty())?;
    let name = seed.name().as_str().replace(['/', '\\', ':'], "-");
    let name = match name.trim_start_matches('.') {
        "" => seed.id()[..8].to_string(),
        name => name.to_string(),
    };
    Some(PathBuf::from(home).join("svrngs").join(name))
}

/// The group's folder kept the same at [`folder_place`] as everyone's,
/// asked of this node's own API as `share --folder` asks it; or why not,
/// said beside the join or the `new`, which stands either way.
async fn folder_at_join(root: &Path, seed: &Seed) -> Said {
    let group = seed.name().clone();
    let Some(dir) = folder_place(seed) else {
        tracing::debug!(target: "cli", group = %seed.id(), "cli: join — no $HOME, so no folder placed");
        let why = "this computer names no home folder ($HOME)".to_string();
        return Said::Folder {
            group,
            dir: PathBuf::from("~/svrngs"),
            placed: Err(why),
        };
    };
    let placed = match std::fs::create_dir_all(&dir) {
        Err(e) => Err(e.to_string()),
        Ok(()) => {
            let asked = FolderAsk {
                dir: dir.clone(),
                group: seed.id(),
            };
            match ask_running(root, "/v1/folders", &asked).await {
                Ok((true, _)) => Ok(()),
                Ok((false, why)) => Err(why),
                Err(e) => Err(e.to_string()),
            }
        }
    };
    tracing::debug!(target: "cli", group = %seed.id(), dir = %dir.display(), ?placed, "cli: join — the group's folder placed, or why not");
    Said::Folder { group, dir, placed }
}

/// The issuer did not let this computer in: it refused, or never answered.
fn not_let_in(e: RedeemError, group: Option<&Name>) -> Refusal {
    tracing::debug!(target: "cli", error = %e, "cli: join — not let in");
    match e {
        RedeemError::Refused(_) => Refusal::LinkRefused,
        RedeemError::NameTaken(holder) => match group {
            Some(group) => Refusal::NameTaken {
                holder,
                group: group.clone(),
            },
            // Only a redemption answers that a name is taken: to an ask
            // it is an answer this build cannot read.
            None => Refusal::IssuerAway,
        },
        RedeemError::Unjudged | RedeemError::NoAnswer(_) => Refusal::IssuerAway,
    }
}

/// The name a group knows its founder or a joiner by when `--as` names
/// none: the full name of the account `$USER` names, as the system shows it
/// on its own screens, else `$USER` itself (bash-person (e); the login was
/// what people read of each other, Demo C's dry run, finding 4). `new` and
/// `join` print it, so it is never silent.
fn user_name() -> Option<Name> {
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
    let (cmd, args) = if mac {
        ("id", ["-F", user])
    } else {
        ("getent", ["passwd", user])
    };
    let out = std::process::Command::new(cmd).args(args).output().ok();
    let text = String::from_utf8(out.filter(|o| o.status.success())?.stdout).ok()?;
    let name = if mac {
        text.trim()
    } else {
        text.split(':').nth(4)?.split(',').next()?.trim()
    };
    (!name.is_empty()).then(|| name.to_string())
}

/// `svrngs new`: a seed drawn for `name`, founded by this computer's key
/// as `person`, written beside the group's record with `page`, when
/// given, kept as its https links' page, and its first link.
pub async fn new_group(
    node: &Node,
    name: Name,
    person: Name,
    page: Option<&Origin>,
    relay: &RelayConfig,
) -> Result<(Seed, Printed), Refusal> {
    // The founder is the key this computer signs and dials with (DESIGN
    // §8, "One key signs and dials").
    let founder = VerifyingKey::from_bytes(node.endpoint().id().as_bytes())
        .expect("an iroh endpoint id is an Ed25519 key");
    let seed = group::draw(name, person, founder);
    let id = seed.id();
    keep_group(node, &seed, page)?;
    tracing::debug!(target: "cli", group = %id, name = %seed.name(), page = ?page.map(Origin::as_str), "cli: group made");
    let token = mint(node, &seed, relay, None).await?;
    Ok((seed, Printed::of(&token, page)))
}

/// The one group held under `root` that `reference` names, by its name or
/// the start of its id ([`group::named`]): `link`'s and `share --folder`'s.
fn group_named(root: &Path, reference: &str) -> Result<Seed, Refusal> {
    let held = group::groups(root);
    match group::named(&held, reference).as_slice() {
        [] => {
            tracing::debug!(target: "cli", reference, held = held.len(), "cli: refused — no group by that name");
            Err(Refusal::NoGroup(reference.to_string()))
        }
        [one] => Ok((*one).clone()),
        many => {
            let ids = many.iter().map(|s| s.id()[..8].to_string()).collect();
            tracing::debug!(target: "cli", reference, n = many.len(), "cli: refused — the name is ambiguous");
            Err(Refusal::Ambiguous {
                name: reference.to_string(),
                ids,
            })
        }
    }
}

/// `svrngs link`: a new link to the group `reference` names: the client's
/// link, and the https one on `page`, kept from now on, when given, else
/// on the group's kept origin, if it keeps one.
pub async fn new_link(
    node: &Node,
    reference: &str,
    page: Option<&Origin>,
    relay: &RelayConfig,
) -> Result<(Seed, Printed), Refusal> {
    let (seed, printed, _) = link_to(node, reference, page, relay, false).await?;
    Ok((seed, printed))
}

/// [`new_link`], or with `device` a device link (`svrngs link --device`;
/// bash-devices): one this computer's person's other computer or phone
/// joins as them by, and that person.
pub async fn link_to(
    node: &Node,
    reference: &str,
    page: Option<&Origin>,
    relay: &RelayConfig,
    device: bool,
) -> Result<(Seed, Printed, Option<Person>), Refusal> {
    let seed = group_named(node.rail().root(), reference)?;
    let dir = node
        .rail()
        .group_dir(&seed.id())
        .map_err(|e| keep_refused(&seed, &e.to_string()))?;
    let refused = |e: group::SeedError| keep_refused(&seed, &e.to_string());
    let page = match page {
        Some(given) => {
            group::write_origin(&dir, given).map_err(refused)?;
            tracing::debug!(target: "cli", group = %seed.id(), origin = given.as_str(), "cli: link — the group's origin changed");
            Some(given.clone())
        }
        None => match group::read_origin(&dir).map_err(refused)? {
            Some(page) => {
                tracing::debug!(target: "cli", group = %seed.id(), origin = page.as_str(), "cli: link — on the group's kept origin too");
                Some(page)
            }
            None => {
                tracing::debug!(target: "cli", group = %seed.id(), "cli: link — the group keeps no origin, so the client's link alone");
                None
            }
        },
    };
    let person = match device {
        true => Some(own_person(node, &seed)?),
        false => None,
    };
    let token = mint(node, &seed, relay, person.as_ref().map(|(k, _)| k.clone())).await?;
    Ok((
        seed,
        Printed::of(&token, page.as_ref()),
        person.map(|(_, p)| p),
    ))
}

/// This computer's person in `seed`'s group, by their first key and name,
/// as the register names them (bash-devices): whom a device link adds a
/// device of. A computer that stands there as nobody makes none.
fn own_person(node: &Node, seed: &Seed) -> Result<(String, Person), Refusal> {
    let me = node.rail().signer().actor();
    let standing =
        (node.rail().standing(&seed.id())).map_err(|e| keep_refused(seed, &e.to_string()))?;
    let own = match standing {
        svrngs_node::register::Standing::Members(m) => {
            m.get(&me).map(|m| (m.person_key.clone(), m.person.clone()))
        }
        _ => None,
    };
    tracing::debug!(target: "cli", group = %seed.id(), ?own, "cli: link --device — this computer's person");
    own.ok_or_else(|| Refusal::NotIn(seed.name().clone()))
}

/// Append an `Invite` for a fresh link's key that lapses in
/// [`LINK_LIFE_SECS`], signed by this computer through the group's door,
/// and answer the link's token. The issuer's newer `Invite` closes their
/// last one from its place in the order (u2-4b, `membership`).
#[expect(
    clippy::disallowed_methods,
    reason = "the command's clock: a link's life starts now"
)]
async fn mint(
    node: &Node,
    seed: &Seed,
    relay: &RelayConfig,
    device_of: Option<String>,
) -> Result<Token, Refusal> {
    let token = issuer::mint(node.endpoint().id(), issuer_relay(node, relay).await);
    // Where this computer is reached, in the record before a joiner pulls
    // it, so the joiner can dial it back (relay-in-record).
    node.say_relay();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|_| keep_refused(seed, "the wall clock is before 1970"))?;
    let mint = MintInvite {
        key: actor_of(&token.key()),
        until: now + LINK_LIFE_SECS,
        device_of,
    };
    let group = seed.id();
    let recorded = match node.rail().commit(&group, &mint) {
        Ok(Ok(recorded)) => recorded,
        Ok(Err(CommitRefused::NoStanding)) => {
            tracing::debug!(target: "cli", group = %group, "cli: refused — this computer does not stand in the group");
            return Err(Refusal::NotIn(seed.name().clone()));
        }
        Ok(Err(CommitRefused::Unsettled)) => {
            return Err(keep_refused(
                seed,
                &svrngs_node::RailError::Unsettled(group).to_string(),
            ))
        }
        Ok(Err(CommitRefused::Command(never))) => match never {},
        Err(e) => return Err(keep_refused(seed, &e.to_string())),
    };
    tracing::debug!(target: "cli", group = %group, invite = %recorded.lines()[0].id, until = mint.until, "cli: link minted");
    Ok(token)
}

/// The relay a link carries: none from a computer with address lookup,
/// which finds it by key; else its home relay, waiting up to
/// [`ONLINE_WAIT`] for one when it has relays to reach.
async fn issuer_relay(node: &Node, relay: &RelayConfig) -> Option<RelayUrl> {
    wait_online(node, "printing the link").await;
    link_relay(relay, node.endpoint().addr().relay_urls().next().cloned())
}

/// Wait up to [`ONLINE_WAIT`] for a home relay, when this computer has
/// relays to reach, before `what`.
async fn wait_online(node: &Node, what: &str) {
    if node.relayed() {
        let online = tokio::time::timeout(ONLINE_WAIT, node.endpoint().online())
            .await
            .is_ok();
        tracing::debug!(target: "cli", online, what, "cli: waited for a relay");
    }
}

/// The relay a link carries, given how this computer reaches the network
/// and the home relay its endpoint reports: none with lookup on; else
/// `home`, or the first relay configured while the endpoint reports none.
pub fn link_relay(relay: &RelayConfig, home: Option<RelayUrl>) -> Option<RelayUrl> {
    if relay.n0_services {
        tracing::debug!(target: "cli", "cli: the link carries the key alone — lookup finds this computer");
        return None;
    }
    let configured = || {
        relay
            .relay_urls
            .first()
            .and_then(|r| svrngs_node::relay::parse(r).ok())
    };
    let carried = home.or_else(configured);
    match &carried {
        Some(url) => {
            tracing::debug!(target: "cli", relay = %url, "cli: no lookup, so the link carries the relay")
        }
        None => {
            tracing::warn!(target: "cli", "cli: no lookup and no relay — the link carries the key alone, and only a computer told this one's address can open it")
        }
    }
    carried
}

/// Keep `seed`'s group on this computer: its journal, the seed beside it,
/// `origin` when given, and the roster the seed founds. What `new` and
/// `join` both write, in one order, so a group kept by either is the same.
fn keep_group(node: &Node, seed: &Seed, origin: Option<&Origin>) -> Result<(), Refusal> {
    let rail = node.rail();
    let dir = rail
        .group_dir(&seed.id())
        .map_err(|e| keep_refused(seed, &e.to_string()))?;
    group::write_seed(&dir, seed)
        .and_then(|()| match origin {
            Some(origin) => group::write_origin(&dir, origin),
            None => Ok(()),
        })
        .map_err(|e: SeedError| keep_refused(seed, &e.to_string()))?;
    rail.set_seed(&seed.id(), &seed.roster())
        .map_err(|e| keep_refused(seed, &e.to_string()))
}

fn keep_refused(seed: &Seed, why: &str) -> Refusal {
    tracing::debug!(target: "cli", group = %seed.id(), why, "cli: refused — the group cannot be kept here");
    Refusal::Keep(seed.name().clone())
}
