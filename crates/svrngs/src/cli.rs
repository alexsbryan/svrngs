// SPDX-License-Identifier: AGPL-3.0-or-later
//! The command surface (DESIGN §5; PLAN §4, U2's inventory, the cli.rs row):
//! verbs parsed by hand, no framework, in the shape of upstream's
//! `commonwealth/crates/commonwealth-rails/src/cli.rs` and `main.rs` at
//! `e1a43bcd2` — an error string for a wrong argument, never a guess; exit
//! codes 0, 1 and 3; a tokio runtime built by hand (`main.rs` here).
//!
//! ```text
//! svrngs                                   run for every group this computer holds
//! svrngs new <name> [--as N] [--origin U] [--relay R]
//!                                          make a group, print its link, run
//! svrngs join <link> [--as N] [--no-run]   join a group by its link, run
//! svrngs link <group> [--origin U]
//!                                          a new link to a group, print it, run
//! svrngs share <port> [--name N] [--kind K] [--limit L] [--refuse P]... [--upload M] [--reads K]... [--ask W] [--sign-in-with-svrngs]
//!                                          lend a service on this computer
//! svrngs unshare <name>                    stop lending it
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
//! svrngs relay <name> [--open] [--staging] a relay, for the groups this computer is in
//!                                          or everyone ([`crate::relay`])
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
//!
//! `room` turns the lends it names on for the room, served to this
//! computer's own network as this computer's person by svrngs running
//! here, which keeps them on across restarts on that network
//! (`svrngs_node::room`): asked as the home page asks it, of svrngs running
//! here, else of the svrngs it starts, so the room has one decider. It says
//! who can open them, and where each is served or why not.
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
use std::num::NonZeroU16;

use svrngs_node::api::{ApiError, API_FILE};
use svrngs_node::folder::FolderAsk;
use svrngs_node::home::WHEN;
use svrngs_node::invite::{Linked, Unminted};
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::join::{Entered, NotJoined};
use svrngs_node::lend::{
    self, Kind, Lend, LendName, RefusedPath, SignInWithSvrngs, Source, Upload,
};
use svrngs_node::node::{Lends, Node, NodeError};
use svrngs_node::relay::n0_relays;
use svrngs_node::removal::{
    LeaveAsk, Outcome, RemovalAsk, HANDOVERS_ROUTE, LEAVES_ROUTE, REMOVALS_ROUTE,
};
use svrngs_node::room;
use svrngs_node::{Joined, Person, SigningKey};

use crate::group::{self, Name, Seed};
use crate::identity::{self, KeyError};
use crate::relay::{self, Relay};
use crate::share::{self, Share, ShareRefusal};
use svrngs_node::link::{client_link, https_link, Link, Origin};

pub const USAGE: &str = "\
svrngs — keep what your group shares on your own computers.

  svrngs
      Run for every group you are in. Leave it running.

  svrngs new <name> [--as <your name>] [--origin <url>] [--relay <name>]
      Make a group called <name> and print its link: send it to the people
      you want in it. It works for a day, while this computer is on. The
      group's folder is put at ~/svrngs/<name>.
      --as is the name the group knows you by; else your full name as this
      computer shows it, or its user name.
      --origin also prints its links on a page address, from now on, for
      people who open them in a browser. On a phone each app then opens at
      an address of its own under that page's host, which must serve the
      page at every name under it, as group.svrnme.sh does.
      --relay meets the group at your own relay, `svrngs relay <name> --open`
      (else newcomers are refused it), in place of the one svrngs uses: every computer
      its links let in meets it there too.

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

  svrngs share <port> [--name <name>] [--kind <kind>] [--limit <n>] [--refuse <path>]... [--upload <Mbit/s>] [--reads <kind>]... [--ask <words>] [--sign-in-with-svrngs]
      Lend what runs on <port> on this computer to everyone in your groups:
      each reaches it through this computer, as themselves. It asks for the
      password the service takes on each request, if any; that stays on
      this computer, and nobody you lend it to receives it. For a service
      it knows keeps its own accounts it asks none, and says how to make
      each person an account there: each signs in once. For one it knows
      takes each person from svrngs (Navidrome on 4533, Calibre-Web on
      8083) it asks none, and says what to set in it. --name is what
      they see it as; else its kind. --kind says what it is, when svrngs
      does not know the port; without it, svrngs says the kinds it knows.
      Of a kind it knows, it keeps back, and says, what shows one person's
      use of it to the others or changes it for everyone. --limit is how
      many people may use it at once; else everyone in your groups. --refuse keeps a path of it from them; say it again
      for more. --upload is this computer's upload, in Mbit/s, as a speed
      test states it: each person using it is told their share of it.
      --reads lets it open, as whoever opens it, what your groups lend of
      <kind>; say it again for more. --ask says, in your words,
      what an app receives from it: then an app uses it only once the
      person it acts for says yes, on their own home page, and stops when
      they say stop. --sign-in-with-svrngs lets an app that speaks OpenID
      Connect sign each person in as themselves, with nothing typed: it
      prints the issuer, a client id and a secret for the app's own
      settings, and keeps the secret on this computer.

  svrngs unshare <name>
      Stop lending <name>.

  svrngs share --folder <dir> --group <group>
      Keep <dir> the same on every computer in <group> that shares a folder
      of its name, whenever svrngs runs here, from now on: anyone in it may
      add, change and delete in it, and nobody outside it receives a byte.
      <group> is its name, or the first 8 letters of its id, as `link`
      takes it.

  svrngs room <name>...
      Open the lends named <name> to this computer's WiFi, so a TV, a phone
      or another computer here can open them as you. They stay open on that
      WiFi until you close them on your home page. Works whether or not
      svrngs is already running.

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

  svrngs leave <group>
      Leave <group> from every computer and phone of yours. What you have
      of it stays here; nothing new reaches you, nor what you lend them.
      Rejoining needs a new invitation, and not from this computer yet.
      If you decide who is removed, hand that on first.

  svrngs relay <name> [--open] [--staging]
      Relay at https://<name>/ for the computers and phones of the groups
      this computer is in, beside svrngs: a phone reaches a computer only
      through a relay. <name> must lead to this computer, which must take
      connections on TCP ports 80 and 443 and UDP port 7842. Its
      certificate is asked of Let's Encrypt, and kept in the data below.
      The group's page's install line, with --relay <name> after the
      page's address, keeps it running.
      --open relays for every computer and phone that reaches it, as
      svrngs's own relay does, each held to the most a phone was measured
      drawing through a relay in a second; it says how much as it starts.
      --staging asks Let's Encrypt's trial service, whose certificates no
      browser trusts: to try <name> first.

Data:    $SVRNGS_DIR, else ~/.svrngs.
Logging: RUST_LOG, default `error`.";

/// Where the record is served to this computer's apps the first time
/// svrngs starts over a data dir: loopback, any free port, which the bare
/// command prints (DESIGN §3; the u3-2 row) and the node writes to
/// `<data dir>/api` (`svrngs_node::api::API_FILE`, u5-1), where an app
/// svrngs did not start finds it, and every start after listens again
/// (start-address, [`svrngs_node::api::kept_address`]).
pub const API: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

/// svrngs's own relay, where a computer meets each group it holds that
/// keeps none of its own once the build ships [`MeetAt::Svrngs`], with no
/// address lookup (PLAN §3, the operator, 2026-10-12). It answers before
/// any build meeting its groups there is published (DESIGN §7,
/// `scripts/deploy-relay.sh`, `scripts/deploy-page.sh`).
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

/// Where this build meets its groups: n0's, chosen, until the operator
/// starts paying for svrngs's. Flipping it, then `scripts/deploy-relay.sh`,
/// then `scripts/deploy-page.sh`, which reads this line, is theirs, in that
/// order (DESIGN §7; PLAN §2).
pub const SHIPPED: MeetAt = MeetAt::N0;

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
        /// `--relay`: where this group is met, in place of where the build
        /// meets its groups ([`SHIPPED`]).
        relay: Option<RelayUrl>,
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
    Relay(Relay),
    Help,
}

/// What `--kind` and `--reads` want, said alike.
const A_KIND: &str = "a kind: letters, digits, - and _ only";

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
    let (mut origin, mut relay) = (None, None);
    let (mut name, mut kind, mut limit, mut refused) = (None, None, None, Vec::new());
    let mut upload = None;
    let mut reads = Vec::new();
    let (mut folder, mut group) = (None, None);
    let mut asks = None;
    let mut sign_in = false;
    let mut run = true;
    let mut device = false;
    let (mut open, mut staging) = (false, false);
    let mut joined = None;
    let mut more = Vec::new();
    while let Some(arg) = it.next() {
        // Every flag taking a value reads it here, and each refusal of
        // one is said the one way, `not`: `--relay` and `--joined` only
        // where they are flags, so elsewhere they stay unknown. The rest
        // are a pattern the compiler holds, never text split (#147).
        let valued = match arg.as_str() {
            "--relay" => verb == "new",
            "--joined" => matches!(verb.as_str(), "remove" | "handover"),
            "--as" | "--origin" | "--name" | "--kind" | "--limit" | "--refuse" | "--upload"
            | "--reads" | "--sign-in" | "--folder" | "--group" | "--ask" => true,
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
                relay =
                    Some(svrngs_node::relay::parse(&url).map_err(|_| not("a relay's address"))?);
            }
            "--name" => name = Some(lend_name(&raw).map_err(|why| format!("--name {why}"))?),
            "--kind" => kind = Some(Kind::parse(&raw).map_err(|_| not(A_KIND))?),
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
            "--reads" => reads.push(Kind::parse(&raw).map_err(|_| not(A_KIND))?),
            "--sign-in" => {
                return Err("--sign-in is gone: svrngs no longer runs a program of yours to sign people in; sign-in with svrngs is the door's own".into())
            }
            "--folder" => folder = Some(raw),
            "--group" => group = Some(raw),
            "--ask" => {
                if raw.trim().is_empty() || raw.chars().any(char::is_control) {
                    return Err("--ask wants words: what an app receives from it".into());
                }
                asks = Some(raw);
            }
            "--joined" if valued => joined = Some(joined_of(&raw)?),
            "--no-run" if verb == "join" => run = false,
            "--device" if verb == "link" => device = true,
            "--sign-in-with-svrngs" if verb == "share" => sign_in = true,
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
        || asks.is_some()
        || sign_in)
        && verb != "share"
    {
        return Err(
            "--name, --kind, --limit, --refuse, --upload, --reads, --ask and --sign-in-with-svrngs are `share` flags"
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
                relay,
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
                asks,
                sign_in,
            }))
        }
        "leave" => Ok(Verb::Leave(
            positional.ok_or("leave wants a group: `svrngs leave <group>`")?,
        )),
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
    #[error("Another program on this computer holds {0}, where svrngs keeps your home page, {home}, and what your groups lend: stop that program, then start svrngs again. Or delete {file}, and svrngs takes a new address as it starts, which bookmarks and programs set to this one do not reach",
        file = .1.display(), home = svrngs_node::home::address(.0.port()))]
    Taken(SocketAddr, PathBuf),
    #[error(transparent)]
    Api(#[from] ApiError),
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
    #[error("you're no longer in {0} on this computer, and you can't join it again from here yet: join from your phone or another computer instead")]
    Left(Name),
    #[error("you are in no group called {0} on this computer")]
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
    #[error("cannot make a sign-in for it on this computer: {0}")]
    SignIn(#[from] svrngs_node::oidc::KeyFailure),
    #[error("cannot read what you lend on this computer: {0}")]
    Lends(lend::LendError),
    #[error("you lend nothing called {name} from this computer{}", lent_now(.lent))]
    NotLent { name: LendName, lent: Vec<LendName> },
    #[error("{0} gives each app no address of its own, so on a phone every app would open where your group's page keeps who each person is. Use a host that serves your page at every name under it, as group.svrnme.sh does")]
    NoAppAddress(String),
    /// `room` refused by svrngs running here, in its words.
    #[error("{0}")]
    Room(String),
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
    #[error("you are in no group on this computer, so a relay for your groups would let nobody through: join one first, or say --open to relay for everyone")]
    RelayForNobody,
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
            | Refusal::Left(_)
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
            | Refusal::Taken(..)
            | Refusal::Api(_)
            | Refusal::IssuerAway
            | Refusal::Prompt(_)
            | Refusal::KeepLend(_)
            | Refusal::SignIn(_)
            | Refusal::Lends(_)
            | Refusal::CannotCheck(_)
            | Refusal::NotRunning
            | Refusal::NoClient(_)
            | Refusal::Unchanged(_)
            | Refusal::RelayStart(_)
            | Refusal::GroupsUnread(_)
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

/// How this computer reaches the network to do `command` over `root`: at
/// the relay `new --relay` names or a link it joins carries, and each
/// group's it holds — the one the group keeps, else where `at` meets the
/// groups that keep none: at [`MeetAt::N0`], n0's relays with n0's lookup
/// on, beside any kept, since a non-empty list replaces n0's
/// (`svrngs_node::guest::RelayConfig`); at [`MeetAt::Svrngs`],
/// [`DEFAULT_RELAY`] and no lookup, n0 none of the path. A computer
/// holding only groups that keep their own relays meets them there alone,
/// as iroh makes the nearest of its relays its home. A kept relay unread
/// is refused.
pub fn reach(root: &Path, command: &Verb, at: MeetAt) -> Result<RelayConfig, Refusal> {
    let asked = match command {
        Verb::New { relay, .. } => relay.clone(),
        Verb::Join { link, .. } => link.token.relay.clone().map(RelayUrl::from),
        _ => None,
    };
    let mut relay_urls: Vec<String> = asked.iter().map(ToString::to_string).collect();
    let mut unkept = false;
    for seed in group::groups(root) {
        let unread = |e: &dyn std::fmt::Display| keep_refused(&seed, &e.to_string());
        let dir = svrngs_node::group_dir_in(root, &seed.id()).map_err(|e| unread(&e))?;
        let Some(kept) = group::read_relay(&dir).map_err(|e| unread(&e))? else {
            tracing::debug!(target: "cli", group = %seed.id(), ?at, "cli: the group keeps no relay — met where the build meets its groups");
            unkept = true;
            continue;
        };
        relay_urls.push(kept.to_string());
    }
    if unkept || relay_urls.is_empty() {
        relay_urls.extend(match at {
            MeetAt::N0 => n0_relays()
                .urls::<Vec<RelayUrl>>()
                .iter()
                .map(ToString::to_string)
                .collect(),
            MeetAt::Svrngs => vec![DEFAULT_RELAY.to_string()],
        });
    }
    let mut seen = std::collections::BTreeSet::new();
    relay_urls.retain(|url| seen.insert(url.clone()));
    tracing::debug!(target: "cli", ?at, unkept, ?relay_urls, "cli: the relays this computer meets its groups at");
    Ok(RelayConfig {
        relay_urls,
        n0_services: at == MeetAt::N0,
        loopback_only: false,
    })
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
            let (_, text) = ask_running(&root, route, &asked).await?;
            let group = seed.name().clone();
            print!(
                "{}",
                said(&seed, &text, |outcome| Said::Removal {
                    group,
                    person,
                    outcome
                })?
            );
            return Ok(());
        }
        Verb::Leave(group) => {
            let seed = group_named(&root, &group)?;
            let (_, text) =
                ask_running(&root, LEAVES_ROUTE, &LeaveAsk { group: seed.id() }).await?;
            let group = seed.name().clone();
            print!(
                "{}",
                said(&seed, &text, |outcome| Said::Left { group, outcome })?
            );
            return Ok(());
        }
        Verb::Room(names) => match claim(&root) {
            Err(Refusal::Busy(lock)) => {
                tracing::debug!(target: "cli", lock = %lock.display(), "cli: room — svrngs runs here already, so it is asked");
                print!("{}", room(&root, names).await?);
                return Ok(());
            }
            claimed => return start(Verb::Room(names), root, relay, claimed?, None).await,
        },
        Verb::Share(asked) => {
            let known = share::well_known_kind(asked.port);
            let mut lend = share::lend_of(&asked, share::listens, known)?;
            let accounts = share::accounts_of(&lend.kind);
            let client = asked.sign_in.then(svrngs_node::oidc::client).transpose()?;
            if let Some(client) = &client {
                tracing::debug!(target: "cli", lend = %lend.name, "cli: share — it signs people in with svrngs: a client id and secret made for its app form");
                lend.sign_in_with_svrngs = Some(client.clone());
            }
            let secret = match (client.is_some(), accounts) {
                (true, _) => None,
                (false, Some(share::Signs::Accounts(_))) => {
                    tracing::debug!(target: "cli", lend = %lend.name, "cli: share — no password asked: its people sign in to it once, with accounts its lender makes");
                    None
                }
                (false, Some(share::Signs::Header(_))) => {
                    tracing::debug!(target: "cli", lend = %lend.name, "cli: share — no password asked: it takes each person from the door's header");
                    None
                }
                (false, None) => prompt_secret(&lend.name)?,
            };
            share_lend(&root, &lend, secret.as_deref())?;
            let sharing = Sharing {
                lend,
                secret: secret.is_some(),
                accounts,
                client,
            };
            return match claim(&root) {
                Err(Refusal::Busy(lock)) => {
                    tracing::debug!(target: "cli", lock = %lock.display(), "cli: share — svrngs runs here already, and reads the lend from its declaration");
                    let api = svrngs_node::api::kept_address(&root)?.ok_or(Refusal::NotRunning)?;
                    print!("{}", sharing.said(api));
                    Ok(())
                }
                claimed => start(Verb::Run, root, relay, claimed?, Some(sharing)).await,
            };
        }
        other => other,
    };
    let claimed = claim(&root)?;
    start(command, root, relay, claimed, None).await
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

/// `svrngs room`: the lends `names` names turned on for the room by
/// svrngs running here, asked as its home page asks it
/// ([`room::ROUTE`]), and the room as it stands then; else its refusal,
/// in its words.
async fn room(root: &Path, names: Vec<LendName>) -> Result<Said, Refusal> {
    let (ok, text) = ask_running(root, room::ROUTE, &room::RoomAsk::Names(names)).await?;
    let on = serde_json::from_str(&text).ok().filter(|_| ok);
    tracing::debug!(target: "cli", ok, text, "cli: room — svrngs running here answered");
    let refused = serde_json::from_str::<serde_json::Value>(&text).ok();
    let refused = refused.and_then(|r| Some(r["error"].as_str()?.to_string()));
    on.map(Said::Room)
        .ok_or(Refusal::Room(refused.unwrap_or(text)))
}

/// What svrngs running here answered a removal, hand-on or leave in the
/// group `seed` names, `text`, said as `said` words it, or the refusal.
fn said(seed: &Seed, text: &str, said: impl FnOnce(Outcome) -> Said) -> Result<Said, Refusal> {
    let outcome: Outcome = serde_json::from_str(text).map_err(|e| {
        tracing::debug!(target: "cli", error = %e, text, "cli: refused — svrngs running here answered no outcome");
        Refusal::Unchanged(seed.name().clone())
    })?;
    tracing::debug!(target: "cli", group = %seed.id(), ?outcome, "cli: a removal, hand-on or leave asked");
    match outcome {
        Outcome::Out | Outcome::Asked { .. } | Outcome::Handed | Outcome::Left => Ok(said(outcome)),
        Outcome::Fault => Err(Refusal::Unchanged(seed.name().clone())),
        _ => Err(Refusal::Removal(
            said(outcome).to_string().trim_end().to_string(),
        )),
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
    let client = http().map_err(|error| {
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
fn http() -> Result<reqwest13::Client, String> {
    let ring = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    rustls::ClientConfig::builder_with_provider(ring)
        .with_safe_default_protocol_versions()
        .and_then(rustls_platform_verifier::BuilderVerifierExt::with_platform_verifier)
        .map(|tls| {
            reqwest13::Client::builder().tls_backend_preconfigured(tls.with_no_client_auth())
        })
        .map_err(|e| e.to_string())
        .and_then(|b| b.build().map_err(|e| e.to_string()))
}

/// Start the node over the data dir `claimed` holds, do the verb, and run
/// until Ctrl-C.
async fn start(
    command: Verb,
    root: PathBuf,
    relay: &RelayConfig,
    _claimed: File,
    sharing: Option<Sharing>,
) -> Result<(), Refusal> {
    let key = identity::load_or_generate_node_key(&root)?;
    // Where the API, the home page and each lend's address here listened
    // last, so a bookmark and a program's address survive a restart
    // (start-address, demo/runs/start/2026-10-13/README.md §4).
    let api = svrngs_node::api::kept_address(&root)?.unwrap_or(API);
    let node = Node::start(root.clone(), key.clone(), relay, group::seed_text, api)
        .await
        .map_err(|e: NodeError| {
            tracing::debug!(target: "cli", error = %e, "cli: refused — the node did not start");
            match e {
                NodeError::Api(ApiError::Taken(at)) => Refusal::Taken(at, root.join(API_FILE)),
                NodeError::Api(e) => Refusal::Api(e),
                _ => Refusal::Start,
            }
        })?;
    if let Some(sharing) = sharing {
        print!("{}", sharing.said(node.api_addr()));
        println!();
    }
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
    let client = http().map_err(|error| {
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
    fn of(made: &Linked) -> Self {
        Self {
            client: client_link(&made.token),
            https: made.page.as_ref().map(|p| https_link(p, &made.token)),
            page: made.page.clone(),
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

/// What `share` says once the node it started knows where its API listens:
/// the lend it declared and the sign-in it made, if any
/// (SIGN_IN_WITH_SVRNGS §5 "Turning it on").
struct Sharing {
    lend: Lend,
    secret: bool,
    accounts: Option<share::Signs>,
    client: Option<SignInWithSvrngs>,
}

impl Sharing {
    /// The words for the API at `api`: the one issuer string is made from
    /// its port, the same string the door and discovery say.
    fn said(&self, api: SocketAddr) -> Said {
        Said::Shared {
            lend: self.lend.clone(),
            secret: self.secret,
            accounts: self.accounts,
            running: true,
            issuer: self
                .client
                .as_ref()
                .map(|_| svrngs_node::oidc::issuer(api.port(), self.lend.name.as_str())),
            client: self.client.clone(),
        }
    }
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
        /// The client its lender copies into the app's form, when it signs
        /// people in with svrngs: shown once, then never again.
        client: Option<SignInWithSvrngs>,
        /// The issuer that client names, when it has one.
        issuer: Option<String>,
    },
    /// `unshare`: no longer lent.
    Unshared(LendName),
    /// `room`: each lend on for the room, and where it is served or why not.
    Room(Vec<room::InRoom>),
    /// `remove` or `handover` (U7): what happened to `person` in `group`.
    Removal {
        group: Name,
        person: Name,
        outcome: Outcome,
    },
    /// `leave` (groups-leave): what happened as you left `group`.
    Left { group: Name, outcome: Outcome },
}

impl std::fmt::Display for Said {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Said::Made { seed, link, beside } => {
                writeln!(f, "Made {}, with you as {}.", seed.name(), seed.founder())?;
                if !beside.is_empty() {
                    writeln!(f, "{}", namesakes_said(beside, seed.name().as_str()))?;
                }
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
            Said::Running { groups, api, lends } => {
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
                secret,
                accounts,
                running,
                client,
                issuer,
            } => {
                // `share` lends a port; a group's folder carries an app.
                let (name, Source::Port(port)) = (&lend.name, &lend.source) else {
                    return writeln!(f, "{} is carried in its group's folder.", lend.name);
                };
                match lend.limit {
                    Some(n) => writeln!(f, "You lend {name} (port {port}) to everyone in your groups, up to {n} {} at once.",
                        if n.get() == 1 { "person" } else { "people" })?,
                    None => writeln!(f, "You lend {name} (port {port}) to everyone in your groups, as many at once as want it. \
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
                let table = share::kept_of(&lend.kind);
                let (kept, typed): (Vec<_>, Vec<_>) = lend.refused.iter().partition(|p| table.contains(p));
                for path in typed {
                    writeln!(f, "They cannot open {}.", path.as_str())?;
                }
                if !kept.is_empty() {
                    let paths: Vec<&str> = kept.iter().map(|p| p.as_str()).collect();
                    writeln!(f, "They cannot open {}: svrngs keeps these back whenever a {} is lent, since they show one person's use of it to the others or change it for everyone.",
                        paths.join(", "), lend.kind)?;
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
                if let (Some(client), Some(issuer)) = (client, issuer) {
                    writeln!(f, "It signs people in with svrngs. In the app's own settings, choose OpenID Connect, and fill in: issuer {issuer}, client id {}, client secret {}.",
                        client.client_id, client.client_secret.as_deref().unwrap_or_default())?;
                    writeln!(f, "{} Copy the secret now: it is not shown again.", svrngs_node::oidc::DEVIATIONS)?;
                }
                if *running {
                    writeln!(
                        f,
                        "svrngs is running here already, so they can open it now."
                    )?;
                }
                writeln!(f, "`svrngs unshare {}` stops lending it.", lend.name)
            }
            Said::Unshared(name) => writeln!(f, "You no longer lend {name}."),
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
                    Outcome::Own => writeln!(f, "{person} is you, and you cannot remove yourself from {group}, so nothing changed: `svrngs leave {group}` leaves it."),
                    Outcome::NotDecider { decides: who } => writeln!(f, "Only {} can let someone else decide who is removed from {group}.", decides(who)),
                    Outcome::NotDeciderUnread => writeln!(f, "You do not decide who is removed from {group}, and this computer could not read who is in the group, so cannot say who does (RUST_LOG=debug says why)."),
                    Outcome::NotIn | Outcome::Left => writeln!(f, "You are no longer in {group}."),
                    Outcome::Fault => writeln!(f, "{group} cannot be changed on this computer (RUST_LOG=debug says why)"),
                }
            }
            Said::Left { group, outcome } => match outcome {
                Outcome::Left => writeln!(f, "You left {group}, from every computer and phone of yours. Your copy of it and the files its folder gave you stay on this computer; nothing new reaches you, and what you lend no longer reaches its people. Rejoining needs a new invitation, and this computer cannot rejoin yet."),
                Outcome::Decides => writeln!(f, "You decide who is removed from {group}, so you cannot leave it yet: hand that to someone else first, with `svrngs handover <name> --group {group}`, then leave."),
                Outcome::NotIn => writeln!(f, "You are no longer in {group}."),
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
    let stays = !matches!(command, Verb::Join { run: false, .. });
    let said = match command {
        Verb::New {
            name,
            person,
            origin,
            relay: kept,
        } => {
            let person = person
                .or_else(user_name)
                .ok_or_else(|| Refusal::NoName(name.clone()))?;
            let page = match origin {
                Some(page) => Some(checked_page(page).await?),
                None => None,
            };
            // The one rule for a name, asked before the group is kept (#138).
            let beside = group::namesakes(node.rail().root(), &name, &svrngs_node::actor_of(key));
            let (seed, link) = new_group(node, name, person, page.as_ref(), kept, relay).await?;
            println!(
                "{}",
                Said::Made {
                    seed: seed.clone(),
                    link,
                    beside
                }
            );
            // The founder's folder, where every joiner's is, so what the
            // founder puts in it is shared with no `share --folder` (Demo
            // C's dry run, finding 7).
            folder_at_join(node, &seed)
        }
        Verb::Join { link, person, .. } => {
            let person = person.or_else(user_name).ok_or(Refusal::NoNameToJoin)?;
            let joined = join_group(node, key, &link, &person).await?;
            print!("{joined}");
            let (Said::Joined { seed, .. } | Said::AlreadyIn(seed)) = joined else {
                unreachable!("join_group says joined or in already")
            };
            folder_at_join(node, &seed)
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
        Verb::Room(names) => room(root, names).await?,
        Verb::Help
        | Verb::Share(_)
        | Verb::Unshare(_)
        | Verb::Relay(_)
        | Verb::Folder { .. }
        | Verb::Remove { .. }
        | Verb::Leave(_) => {
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

/// `svrngs join`: `link` redeemed as `person` by the node, which signs
/// with its own key, `key` ([`svrngs_node::join`]), once a relay the link
/// carries is waited for, in the command's words.
pub async fn join_group(
    node: &Node,
    key: &SigningKey,
    link: &Link,
    person: &Name,
) -> Result<Said, Refusal> {
    debug_assert_eq!(
        node.endpoint().id().as_bytes(),
        key.verifying_key().as_bytes()
    );
    // Its home relay reached before the exchange, so the joiner says it
    // there and the issuer can dial it back; else not before its first
    // round (demo/runs/relay-in-record/2026-10-12/: 60 s). Before the ask,
    // so a relay that does not answer is named before the issuer is dialled.
    let carried: Vec<String> = link.token.relay.iter().map(ToString::to_string).collect();
    wait_online(node, &carried).await;
    match node.host().join(link, person).await {
        Ok(Entered::In { seed, person }) => Ok(Said::Joined { seed, person }),
        Ok(Entered::AlreadyIn(seed)) => Ok(Said::AlreadyIn(seed)),
        Err(NotJoined::LinkRefused) => Err(Refusal::LinkRefused),
        Err(NotJoined::NameTaken { holder, group }) => Err(Refusal::NameTaken { holder, group }),
        Err(NotJoined::IssuerAway) => Err(Refusal::IssuerAway),
        Err(NotJoined::Keep(group)) => Err(Refusal::Keep(group)),
        Err(NotJoined::Left(group)) => Err(Refusal::Left(group)),
    }
}

/// The group's folder placed as everyone's, by the node
/// ([`svrngs_node::folder::Folders::place`]); or why not, said beside the
/// join or the `new`, which stands either way.
fn folder_at_join(node: &Node, seed: &Seed) -> Said {
    let (dir, placed) = node.folders().place(seed);
    let group = seed.name().clone();
    Said::Folder { group, dir, placed }
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
/// given, kept as its https links' page, and `kept`, when given, as where
/// it is met (`new --relay`), else the relay a group made on the home page
/// keeps ([`svrngs_node::node::Host::made_relay`]), once this computer has
/// reached one; and its first link.
pub async fn new_group(
    node: &Node,
    name: Name,
    person: Name,
    page: Option<&Origin>,
    kept: Option<RelayUrl>,
    relay: &RelayConfig,
) -> Result<(Seed, Printed), Refusal> {
    // The founder is the key this computer signs and dials with (PLAN
    // §4, U1's "One key signs and dials").
    let founder = VerifyingKey::from_bytes(node.endpoint().id().as_bytes())
        .expect("an iroh endpoint id is an Ed25519 key");
    let seed = group::draw(name, person, founder);
    let id = seed.id();
    wait_online(node, &relay.relay_urls).await;
    let asked = kept.is_some();
    let kept = kept.or_else(|| node.host().made_relay());
    let keep = node.host().keep_linked(&seed, page, kept.as_ref());
    let made = keep.map_err(|why| keep_refused(&seed, &why))?;
    tracing::debug!(target: "cli", group = %id, name = %seed.name(), page = ?page.map(Origin::as_str), ?kept, asked, "cli: group made, keeping the relay asked, else the one the home page's keeps");
    let made = made.map_err(|e| unminted(&seed, e))?;
    Ok((seed, Printed::of(&made)))
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

/// `svrngs link`: a new link to the group `reference` names, made by the
/// node ([`svrngs_node::node::Host::link`]) once this computer has reached
/// a relay: the client's link, and the https one on `page`, kept from now
/// on, when given, else on the group's kept origin, if it keeps one. With
/// `device` a device link (`svrngs link --device`; bash-devices): one this
/// computer's person's other computer or phone joins as them by, and that
/// person.
pub async fn link_to(
    node: &Node,
    reference: &str,
    page: Option<&Origin>,
    relay: &RelayConfig,
    device: bool,
) -> Result<(Seed, Printed, Option<Person>), Refusal> {
    let seed = group_named(node.rail().root(), reference)?;
    wait_online(node, &relay.relay_urls).await;
    let made = node.host().link(&seed.id(), device, page, None);
    let made = made.map_err(|e| unminted(&seed, e))?;
    Ok((seed, Printed::of(&made), made.device))
}

/// Why the node made no link, in the command's refusals.
fn unminted(seed: &Seed, e: Unminted) -> Refusal {
    match e {
        Unminted::NotIn => Refusal::NotIn(seed.name().clone()),
        Unminted::Unkept(why) => keep_refused(seed, &why),
    }
}

/// Wait up to [`ONLINE_WAIT`] for a home relay, when this computer has
/// relays to reach, saying which it `tried` when it reached none.
async fn wait_online(node: &Node, tried: &[String]) {
    if node.relayed() {
        let online = tokio::time::timeout(ONLINE_WAIT, node.endpoint().online())
            .await
            .is_ok();
        tracing::debug!(target: "cli", online, ?tried, "cli: waited for a relay");
        if !online && !tried.is_empty() {
            let secs = ONLINE_WAIT.as_secs();
            eprintln!("svrngs: after {secs} s, this computer has not yet reached the server your groups find it through. It goes on, and keeps trying; until it does, they may not reach you (RUST_LOG=debug names the server).");
        }
    }
}

fn keep_refused(seed: &Seed, why: &str) -> Refusal {
    tracing::debug!(target: "cli", group = %seed.id(), why, "cli: refused — the group cannot be kept here");
    Refusal::Keep(seed.name().clone())
}
