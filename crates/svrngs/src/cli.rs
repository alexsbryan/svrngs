// SPDX-License-Identifier: AGPL-3.0-or-later
//! The command surface (DESIGN §5; §8, U2's inventory, the cli.rs row):
//! verbs parsed by hand, no framework, in the shape of upstream's
//! `commonwealth/crates/commonwealth-rails/src/cli.rs` and `main.rs` at
//! `e1a43bcd2` — an error string for a wrong argument, never a guess; exit
//! codes 0, 1 and 3; a tokio runtime built by hand (`main.rs` here).
//!
//! ```text
//! svrngs                                   run for every group this computer holds
//! svrngs new <name> [--as N] [--origin U [--lends-origin A]]
//!                                          make a group, print its link, run
//! svrngs join <link> [--as N]              join a group by its link, run
//! svrngs link <group> [--origin U [--lends-origin A]]
//!                                          a new link to a group, print it, run
//! svrngs share <port> [--name N] [--kind K] [--limit L] [--refuse P]... [--upload M] [--reads K]...
//!                                          lend a service on this computer
//! svrngs unshare <name>                    stop lending it
//! ```
//!
//! `share` and `unshare` write a lend's declaration and its secret into the
//! data dir (`svrngs_node::lend`), which the door reads on every request,
//! so each works whether or not svrngs is running here (u3-5). `share`
//! then runs, as the other verbs do, unless svrngs already is. The secret
//! is read from a prompt, never from the command line, where any program on
//! this computer may read it (`ps`); a service that keeps its own accounts
//! is lent with none, and its lender told how to make each person one
//! (u5-4, [`share::accounts_of`]). The bare command prints what the people
//! of this computer's groups lend, asked of their computers as it starts.
//!
//! `new` and `link` always print the client's link, `svrngs://join/<token>`
//! (u2-8): nothing depends on a hosted page. A group may keep a page
//! origin, its setting on this computer (`group::write_origin`), and then
//! its https link is printed beside it: `new --origin` keeps one, `join`
//! keeps the one its https link was printed on and none from a client's
//! link, and `link --origin` changes it.
//!
//! A group with a page origin opens its apps on a phone at addresses apart
//! from the page's, which holds each person's key (u5-2; DESIGN §7), and
//! `new` and `link` say which way ([`Apps`]): each app at an address of its
//! own under the page's host, when that host serves the page at every name
//! under it, which the command checks by asking a name nobody made for it;
//! or every app under the one address `--lends-origin` names. A page's host
//! that does not, with no `--lends-origin`, is refused, naming both ways on.
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

use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::lend::{self, Kind, Lend, LendName, RefusedPath, Upload};
use svrngs_node::node::{Lends, Node, NodeError};
use svrngs_node::redeem::{ask, redeem, AskRequest, RedeemError, RedeemRequest, Redeemed};
use svrngs_node::{actor_of, Device, Person, SigningKey};
use svrngs_node::{MintInvite, Refused as CommitRefused};

use crate::group::{self, Name, Seed, SeedError};
use crate::identity::{self, KeyError};
use crate::issuer;
use crate::share::{self, Share, ShareRefusal};
use svrngs_node::link::{client_link, https_link, Link, Origin, Token};

pub const USAGE: &str = "\
svrngs — keep what your group shares on your own computers.

  svrngs
      Run for every group this computer is in. Leave it running.

  svrngs new <name> [--as <your name>] [--origin <url> [--lends-origin <url>]]
      Make a group called <name> and print its link: send it to the people
      you want in it. It works for a day, while this computer is on.
      --as is the name the group knows you by; else this computer's user name.
      --origin also prints its links on a page address, from now on, for
      people who open them in a browser. On a phone each app then opens at
      an address of its own under that page's host, if it serves the page at
      every name under it (group.svrnme.sh does); else name a second address the apps
      open under with --lends-origin.

  svrngs join <link> [--as <your name>]
      Join the group a link you were sent is for. The computer that sent it
      must be on. --as is the name the group knows you by; else this
      computer's user name.

  svrngs link <group> [--origin <url> [--lends-origin <url>]]
      Print a new link to <group>. Any link you sent before stops letting
      anyone new in; everyone it let in stays. <group> is its name, or the
      first 8 letters of its id when two groups share a name. --origin
      also prints it on a page address, from now on, as `new` does.

  svrngs share <port> [--name <name>] [--kind <kind>] [--limit <n>] [--refuse <path>]... [--upload <Mbit/s>] [--reads <kind>]...
      Lend what runs on <port> on this computer to everyone in your groups:
      each reaches it through this computer, as themselves. It asks for the
      password the service takes on each request, if any; that stays on
      this computer, and nobody you lend it to receives it. For a service
      it knows keeps its own accounts it asks none, and says how to make
      each person an account there: each signs in once. --name is what
      they see it as; else its kind. --kind says what it is, when svrngs
      does not know the port. --limit is how many people may use it at
      once; else 3. --refuse keeps a path of it from them; say it again
      for more. --upload is this computer's upload, in Mbit/s, as a speed
      test states it: each person using it is told their share of it.
      --reads lets it open, as whoever opens it, what your groups lend of
      <kind>; say it again for more.

  svrngs unshare <name>
      Stop lending <name>.

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
        /// Where its apps open on a phone, apart from the page (u5-2).
        lends: Option<Origin>,
    },
    Join {
        link: Box<Link>,
        person: Option<Name>,
    },
    Link {
        group: String,
        /// `None` keeps the group's kept origin, if any.
        origin: Option<Origin>,
        /// With `origin`, where its apps open on a phone (u5-2).
        lends: Option<Origin>,
    },
    /// No secret: it is read from a prompt (the module docs).
    Share(Share),
    Unshare(LendName),
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
    let mut lends = None;
    let (mut name, mut kind, mut limit, mut refused) = (None, None, None, Vec::new());
    let mut upload = None;
    let mut reads = Vec::new();
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
            "--lends-origin" => {
                let raw = value("--lends-origin")?;
                lends = Some(Origin::parse(&raw).ok_or_else(|| {
                    format!("--lends-origin `{raw}` is not an http(s) address without a '#'")
                })?);
            }
            "--name" => {
                let raw = value("--name")?;
                name = Some(LendName::parse(&raw).map_err(|_| {
                    format!("--name `{raw}` is not a name: letters, digits, - and _ only")
                })?);
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
            "-h" | "--help" => return Ok(Verb::Help),
            other if other.starts_with('-') => return Err(format!("unknown argument `{other}`")),
            other if positional.is_none() => positional = Some(other.to_string()),
            other => return Err(format!("unexpected argument `{other}`")),
        }
    }
    if person.is_some() && !matches!(verb.as_str(), "new" | "join") {
        return Err("--as is a `new` and `join` flag".into());
    }
    if origin.is_some() && !matches!(verb.as_str(), "new" | "link") {
        return Err("--origin is a `new` and `link` flag".into());
    }
    if lends.is_some() && origin.is_none() {
        return Err("--lends-origin goes with --origin, on `new` or `link`".into());
    }
    if (name.is_some()
        || kind.is_some()
        || limit.is_some()
        || !refused.is_empty()
        || upload.is_some()
        || !reads.is_empty())
        && verb != "share"
    {
        return Err(
            "--name, --kind, --limit, --refuse, --upload and --reads are `share` flags".into(),
        );
    }
    match verb.as_str() {
        "new" => {
            let raw = positional.ok_or("new wants a name: `svrngs new <name>`")?;
            let name = Name::parse(&raw).map_err(|why| format!("`{raw}` is not a name: {why}"))?;
            Ok(Verb::New {
                name,
                person,
                origin,
                lends,
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
            })
        }
        "link" => Ok(Verb::Link {
            group: positional.ok_or("link wants a group: `svrngs link <group>`")?,
            origin,
            lends,
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
            }))
        }
        "unshare" => {
            let raw = positional.ok_or("unshare wants a name: `svrngs unshare <name>`")?;
            let name = LendName::parse(&raw)
                .map_err(|_| format!("`{raw}` is not a name: letters, digits, - and _ only"))?;
            Ok(Verb::Unshare(name))
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
    #[error("{0} gives each app no address of its own, so on a phone every app would open where your group's page keeps who each person is. Either name a second address for the apps, on any host: `--lends-origin <url>`; or use a host that serves your page at every name under it, as group.svrnme.sh does")]
    NoAppAddress(String),
    #[error("cannot tell from here whether {0} gives each app an address of its own: the page does not open there (RUST_LOG=debug says why); try again, or name a second address for the apps with `--lends-origin <url>`")]
    CannotCheck(String),
    #[error("{0} is the page's own address; the apps open under one apart from it, so none can reach what the page keeps")]
    AppsOnPage(String),
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
            | Refusal::NoGroup(_)
            | Refusal::Ambiguous { .. }
            | Refusal::NotIn(_)
            | Refusal::Share(_)
            | Refusal::NotLent { .. }
            | Refusal::NoAppAddress(_)
            | Refusal::AppsOnPage(_)
            | Refusal::Busy(_) => 1,
            Refusal::NoDataDir
            | Refusal::DataDir(..)
            | Refusal::Key(_)
            | Refusal::Start
            | Refusal::IssuerAway
            | Refusal::Prompt(_)
            | Refusal::KeepLend(_)
            | Refusal::CannotCheck(_)
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
        Verb::Unshare(name) => {
            print!("{}", unshare(&root, &name)?);
            return Ok(());
        }
        Verb::Share(asked) => {
            let lend = share::lend_of(
                &asked,
                share::listens(asked.port),
                share::well_known_kind(asked.port),
            )?;
            let accounts = share::accounts_of(&lend.kind);
            let secret = match accounts {
                Some(_) => {
                    tracing::debug!(target: "cli", lend = %lend.name, "cli: share — no password asked: its people sign in to it once, with accounts its lender makes");
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

/// Where a group's apps open on a phone, apart from its page, which holds
/// each person's key (u5-2; DESIGN §3, "Lent apps", call 1; §7): a closed
/// set, decided by [`pages`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Apps {
    /// Each at an address of its own, `<label>.<the page's host>`: the
    /// page's host serves the page at every name under it.
    OwnAddress,
    /// Every app under this one address, each at `l/<label>/` under it:
    /// the floor, any host.
    Under(Origin),
}

/// A group's page, and where its apps open on a phone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pages {
    pub page: Origin,
    pub apps: Apps,
}

impl Pages {
    fn lends(&self) -> Option<&Origin> {
        match &self.apps {
            Apps::OwnAddress => None,
            Apps::Under(lends) => Some(lends),
        }
    }
}

/// `origin`'s scheme and host, with its port: where a browser keeps what
/// a page there stores. `None` for what is not an http(s) address.
fn place(origin: &Origin) -> Option<(&str, &str)> {
    let (scheme, rest) = origin.as_str().split_once("://")?;
    let authority = rest.split(['/', '?']).next()?;
    let authority = authority.rsplit_once('@').map_or(authority, |(_, a)| a);
    Some((scheme, authority))
}

/// The page `page`, and where the group's apps open on a phone: under
/// `lends` when given — refused when it is the page's own address — else
/// each at an address of its own under the page's host, when a name nobody
/// made under it serves the page at its root as the page's own address
/// does (u5-10: www.svrnme.sh/g/ resolved every name and served the page at
/// none but its own); refused when it does not, and not judged when the
/// page's own address does not give the page (u5-2).
#[expect(
    clippy::disallowed_methods,
    reason = "draws a name nobody made, to ask the page's host for the page there"
)]
pub async fn pages(page: Origin, lends: Option<Origin>) -> Result<Pages, Refusal> {
    if let Some(lends) = lends {
        if place(&lends).map(|(s, a)| (s, a.to_ascii_lowercase()))
            == place(&page).map(|(s, a)| (s, a.to_ascii_lowercase()))
        {
            tracing::debug!(target: "cli", page = page.as_str(), "cli: refused — the apps' address is the page's");
            return Err(Refusal::AppsOnPage(lends.as_str().into()));
        }
        tracing::debug!(target: "cli", page = page.as_str(), lends = lends.as_str(), "cli: the apps open under the address given");
        return Ok(Pages {
            page,
            apps: Apps::Under(lends),
        });
    }
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
    let client = rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .and_then(rustls_platform_verifier::BuilderVerifierExt::with_platform_verifier)
    .map_err(|e| e.to_string())
    .and_then(|tls| {
        reqwest13::Client::builder()
            .tls_backend_preconfigured(tls.with_no_client_auth())
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())
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
    Ok(Pages {
        page,
        apps: Apps::OwnAddress,
    })
}

/// A link as printed: the client's, and the https one for a group that
/// keeps a page origin, with where its apps open on a phone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printed {
    pub client: String,
    pub https: Option<String>,
    pub apps: Option<Pages>,
}

impl Printed {
    fn of(token: &Token, pages: Option<&Pages>) -> Self {
        Self {
            client: client_link(token),
            https: pages.map(|p| https_link(&p.page, p.lends(), token)),
            apps: pages.cloned(),
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
        match self.apps.as_ref().map(|p| (&p.apps, place(&p.page))) {
            Some((Apps::OwnAddress, Some((_, host)))) => {
                writeln!(f)?;
                writeln!(f, "On a phone, each app opens at an address of its own under {host}, which can see which app is opened.")?;
            }
            Some((Apps::Under(lends), _)) => {
                writeln!(f)?;
                writeln!(f, "On a phone, every app opens under {}: they share it, so an app there can see what another keeps there.", lends.as_str())?;
            }
            _ => {}
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
    Linked { seed: Seed, link: Printed },
    /// `join`: let in by the link's issuer, as `person`.
    Joined { seed: Seed, person: Name },
    /// `join` of a group this computer is in already.
    AlreadyIn(Seed),
    /// The bare command: the names of the groups it runs for, where this
    /// computer's apps reach them, and what the people in them lend.
    Running {
        groups: Vec<Name>,
        api: SocketAddr,
        lends: Lends,
    },
    /// `share`: `lend` declared, with a secret or none; `accounts`, how its
    /// lender makes each person an account in it, when it keeps its own;
    /// `running` when svrngs runs here already, so it is lent now.
    Shared {
        lend: Lend,
        secret: bool,
        accounts: Option<&'static str>,
        running: bool,
    },
    /// `unshare`: no longer lent.
    Unshared(LendName),
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
            Said::Linked { seed, link } => {
                writeln!(
                    f,
                    "A new link to {}. Any link you sent before stops letting anyone new in; everyone it let in stays.",
                    seed.name()
                )?;
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
                for person in &lends.unanswered {
                    writeln!(
                        f,
                        "{person}'s computer did not answer, so what they lend is not listed."
                    )?;
                }
                Ok(())
            }
            Said::Shared {
                lend,
                secret,
                accounts,
                running,
            } => {
                writeln!(
                    f,
                    "Lent {} (port {}) to everyone in your groups, up to {} {} at once.",
                    lend.name,
                    lend.port,
                    lend.limit,
                    if lend.limit.get() == 1 {
                        "person"
                    } else {
                        "people"
                    }
                )?;
                if let Some(how) = accounts {
                    writeln!(
                        f,
                        "{} asks each person to sign in once, with an account you make them. In {}: {how}. Then tell each person theirs.",
                        lend.name, lend.kind
                    )?;
                }
                for path in &lend.refused {
                    writeln!(f, "They cannot open {}.", path.as_str())?;
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
        }
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
    let said = match command {
        Verb::New {
            name,
            person,
            origin,
            lends,
        } => {
            let person = match person {
                Some(p) => p,
                None => user_name().ok_or_else(|| Refusal::NoName(name.clone()))?,
            };
            let pages = match origin {
                Some(page) => Some(pages(page, lends).await?),
                None => None,
            };
            let (seed, link) = new_group(node, name, person, pages.as_ref(), relay).await?;
            Said::Made { seed, link }
        }
        Verb::Join { link, person } => {
            let person = match person {
                Some(p) => p,
                None => user_name().ok_or(Refusal::NoNameToJoin)?,
            };
            join_group(node, key, &link, &person).await?
        }
        Verb::Link {
            group,
            origin,
            lends,
        } => {
            let pages = match origin {
                Some(page) => Some(pages(page, lends).await?),
                None => None,
            };
            let (seed, link) = new_link(node, &group, pages.as_ref(), relay).await?;
            Said::Linked { seed, link }
        }
        Verb::Run => running(root, node).await,
        Verb::Help | Verb::Share(_) | Verb::Unshare(_) => {
            unreachable!("answered in `run`")
        }
    };
    print!("{said}");
    println!();
    println!("Leave this running; Ctrl-C stops it.");
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::warn!(target: "cli", error = %e, "cli: cannot wait for Ctrl-C, so stopping now");
    }
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
    .map_err(not_let_in)?;
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
    let request = RedeemRequest::sign(
        &link_key,
        key,
        &id,
        &asked.invite,
        &Person::from(person.clone()),
        Device::Computer,
    );
    let redeemed = redeem(node.endpoint(), issuer.clone(), &request)
        .await
        .map_err(not_let_in)?;
    tracing::debug!(target: "cli", group = %id, redeemed = ?redeemed, "cli: join — let in");
    let pages = link.origin.clone().map(|page| Pages {
        page,
        apps: link.lends.clone().map_or(Apps::OwnAddress, Apps::Under),
    });
    keep_group(node, &seed, pages.as_ref())?;
    if link.origin.is_none() {
        tracing::debug!(target: "cli", group = %id, "cli: join — a client's link, so no origin is kept");
    }
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
        person: person.clone(),
    })
}

/// The issuer did not let this computer in: it refused, or never answered.
fn not_let_in(e: RedeemError) -> Refusal {
    tracing::debug!(target: "cli", error = %e, "cli: join — not let in");
    match e {
        RedeemError::Refused(_) => Refusal::LinkRefused,
        RedeemError::Unjudged | RedeemError::NoAnswer(_) => Refusal::IssuerAway,
    }
}

/// This computer's user name, as the name a group knows its founder or a
/// joiner by when `--as` names none. `new` and `join` print it, so it is
/// never silent.
fn user_name() -> Option<Name> {
    let raw = std::env::var("USER").ok()?;
    tracing::debug!(target: "cli", user = %raw, "cli: the founder's name from $USER");
    Name::parse(&raw).ok()
}

/// `svrngs new`: a seed drawn for `name`, founded by this computer's key
/// as `person`, written beside the group's record with `pages`, when
/// given, kept as its https links' page and where its apps open, and its
/// first link.
pub async fn new_group(
    node: &Node,
    name: Name,
    person: Name,
    pages: Option<&Pages>,
    relay: &RelayConfig,
) -> Result<(Seed, Printed), Refusal> {
    // The founder is the key this computer signs and dials with (DESIGN
    // §8, "One key signs and dials").
    let founder = VerifyingKey::from_bytes(node.endpoint().id().as_bytes())
        .expect("an iroh endpoint id is an Ed25519 key");
    let seed = group::draw(name, person, founder);
    let id = seed.id();
    keep_group(node, &seed, pages)?;
    tracing::debug!(target: "cli", group = %id, name = %seed.name(), pages = ?pages, "cli: group made");
    let token = mint(node, &seed, relay).await?;
    Ok((seed, Printed::of(&token, pages)))
}

/// `svrngs link`: a new link to the group `reference` names: the client's
/// link, and the https one on `pages`, kept from now on, when given, else
/// on the group's kept origin, if it keeps one, its apps where it keeps
/// them.
pub async fn new_link(
    node: &Node,
    reference: &str,
    pages: Option<&Pages>,
    relay: &RelayConfig,
) -> Result<(Seed, Printed), Refusal> {
    let held = group::groups(node.rail().root());
    let seed = match group::named(&held, reference).as_slice() {
        [] => {
            tracing::debug!(target: "cli", reference, held = held.len(), "cli: refused — no group by that name");
            return Err(Refusal::NoGroup(reference.to_string()));
        }
        [one] => (*one).clone(),
        many => {
            let ids = many.iter().map(|s| s.id()[..8].to_string()).collect();
            tracing::debug!(target: "cli", reference, n = many.len(), "cli: refused — the name is ambiguous");
            return Err(Refusal::Ambiguous {
                name: reference.to_string(),
                ids,
            });
        }
    };
    let dir = node
        .rail()
        .group_dir(&seed.id())
        .map_err(|e| keep_refused(&seed, &e.to_string()))?;
    let refused = |e: group::SeedError| keep_refused(&seed, &e.to_string());
    let pages = match pages {
        Some(given) => {
            keep_pages(&dir, given).map_err(refused)?;
            tracing::debug!(target: "cli", group = %seed.id(), pages = ?given, "cli: link — the group's origin changed");
            Some(given.clone())
        }
        None => match group::read_origin(&dir).map_err(refused)? {
            Some(page) => {
                let apps = group::read_lends(&dir).map_err(refused)?;
                tracing::debug!(target: "cli", group = %seed.id(), origin = page.as_str(), lends = ?apps.as_ref().map(Origin::as_str), "cli: link — on the group's kept origin too");
                Some(Pages {
                    page,
                    apps: apps.map_or(Apps::OwnAddress, Apps::Under),
                })
            }
            None => {
                tracing::debug!(target: "cli", group = %seed.id(), "cli: link — the group keeps no origin, so the client's link alone");
                None
            }
        },
    };
    let token = mint(node, &seed, relay).await?;
    Ok((seed, Printed::of(&token, pages.as_ref())))
}

/// Append an `Invite` for a fresh link's key that lapses in
/// [`LINK_LIFE_SECS`], signed by this computer through the group's door,
/// and answer the link's token. The issuer's newer `Invite` closes their
/// last one from its place in the order (u2-4b, `membership`).
#[expect(
    clippy::disallowed_methods,
    reason = "the command's clock: a link's life starts now"
)]
async fn mint(node: &Node, seed: &Seed, relay: &RelayConfig) -> Result<Token, Refusal> {
    let token = issuer::mint(node.endpoint().id(), issuer_relay(node, relay).await);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|_| keep_refused(seed, "the wall clock is before 1970"))?;
    let mint = MintInvite {
        key: actor_of(&token.key()),
        until: now + LINK_LIFE_SECS,
    };
    let group = seed.id();
    let recorded = match node.rail().commit(&group, &mint) {
        Ok(Ok(recorded)) => recorded,
        Ok(Err(CommitRefused::NoStanding)) => {
            tracing::debug!(target: "cli", group = %group, "cli: refused — this computer does not stand in the group");
            return Err(Refusal::NotIn(seed.name().clone()));
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
    let endpoint = node.endpoint();
    if relay.n0_services || !relay.relay_urls.is_empty() {
        let online = tokio::time::timeout(ONLINE_WAIT, endpoint.online())
            .await
            .is_ok();
        tracing::debug!(target: "cli", online, "cli: waited for a relay before printing the link");
    }
    link_relay(relay, endpoint.addr().relay_urls().next().cloned())
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
fn keep_group(node: &Node, seed: &Seed, pages: Option<&Pages>) -> Result<(), Refusal> {
    let rail = node.rail();
    let dir = rail
        .group_dir(&seed.id())
        .map_err(|e| keep_refused(seed, &e.to_string()))?;
    group::write_seed(&dir, seed)
        .and_then(|()| match pages {
            Some(pages) => keep_pages(&dir, pages),
            None => Ok(()),
        })
        .map_err(|e: SeedError| keep_refused(seed, &e.to_string()))?;
    rail.set_seed(&seed.id(), &seed.roster())
        .map_err(|e| keep_refused(seed, &e.to_string()))
}

/// Keep `pages` in the group's directory `dir`: its page origin, and its
/// lends origin or none.
fn keep_pages(dir: &Path, pages: &Pages) -> Result<(), group::SeedError> {
    group::write_origin(dir, &pages.page)?;
    group::write_lends(dir, pages.lends())
}

fn keep_refused(seed: &Seed, why: &str) -> Refusal {
    tracing::debug!(target: "cli", group = %seed.id(), why, "cli: refused — the group cannot be kept here");
    Refusal::Keep(seed.name().clone())
}
