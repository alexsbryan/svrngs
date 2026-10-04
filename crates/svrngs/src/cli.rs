// SPDX-License-Identifier: AGPL-3.0-or-later
//! The command surface (DESIGN §5; §8, U2's inventory, the cli.rs row):
//! verbs parsed by hand, no framework, in the shape of upstream's
//! `commonwealth/crates/commonwealth-rails/src/cli.rs` and `main.rs` at
//! `e1a43bcd2` — an error string for a wrong argument, never a guess; exit
//! codes 0, 1 and 3; a tokio runtime built by hand (`main.rs` here).
//!
//! ```text
//! svrngs                                   run for every group this computer holds
//! svrngs new <name> [--as N] [--origin U]  make a group, print its link, run
//! svrngs join <link> [--as N]              join a group by its link, run
//! svrngs link <group> [--origin U]         a new link to a group, print it, run
//! ```
//!
//! `new` and `link` always print the client's link, `svrngs://join/<token>`
//! (u2-8): nothing depends on a hosted page. A group may keep a page
//! origin, its setting on this computer (`group::write_origin`), and then
//! its https link is printed beside it: `new --origin` keeps one, `join`
//! keeps the one its https link was printed on and none from a client's
//! link, and `link --origin` changes it.
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
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::node::{Node, NodeError};
use svrngs_node::redeem::{ask, redeem, AskRequest, RedeemError, RedeemRequest, Redeemed};
use svrngs_node::{actor_of, Person, SigningKey};
use svrngs_node::{MintInvite, Refused as CommitRefused};

use crate::group::{self, Name, Seed, SeedError};
use crate::identity::{self, KeyError};
use crate::link::{self, client_link, https_link, Link, Origin, Token};

pub const USAGE: &str = "\
svrngs — keep what your group shares on your own computers.

  svrngs
      Run for every group this computer is in. Leave it running.

  svrngs new <name> [--as <your name>] [--origin <url>]
      Make a group called <name> and print its link: send it to the people
      you want in it. It works for a day, while this computer is on.
      --as is the name the group knows you by; else this computer's user name.
      --origin also prints its links on a page address, from now on, for
      people who open them in a browser.

  svrngs join <link> [--as <your name>]
      Join the group a link you were sent is for. The computer that sent it
      must be on. --as is the name the group knows you by; else this
      computer's user name.

  svrngs link <group> [--origin <url>]
      Print a new link to <group>. Any link you sent before stops letting
      anyone new in; everyone it let in stays. <group> is its name, or the
      first 8 letters of its id when two groups share a name. --origin
      also prints it on a page address, from now on.

Data:    $SVRNGS_DIR, else ~/.svrngs.
Logging: RUST_LOG, default `error`.";

/// How long a link lets people in: upstream's one invite lapse
/// (`commonwealth-discovery/src/membership.rs:53` at `e1a43bcd2`, a day;
/// DESIGN §8, U2's inventory).
pub const LINK_LIFE_SECS: i64 = 24 * 60 * 60;

/// Where the record is served to this computer's apps: loopback, any free
/// port, which the bare command prints (DESIGN §3; the u3-2 row). A port
/// an app finds without being told waits for an app that needs one.
pub const API: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

/// How long `new` and `link` wait for this computer's address before
/// printing a link, when it has a relay to reach.
const ONLINE_WAIT: Duration = Duration::from_secs(10);

/// What was typed.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
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
    },
    Link {
        group: String,
        /// `None` keeps the group's kept origin, if any.
        origin: Option<Origin>,
    },
    Help,
}

/// Parse `std::env::args()`. A wrong argument is an error string the caller
/// prints, never a guess at what was meant.
pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Command, String> {
    let mut it = args.into_iter().skip(1);
    let Some(verb) = it.next() else {
        return Ok(Command::Run);
    };
    let mut positional: Option<String> = None;
    let mut person = None;
    let mut origin = None;
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
            "-h" | "--help" => return Ok(Command::Help),
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
    match verb.as_str() {
        "new" => {
            let raw = positional.ok_or("new wants a name: `svrngs new <name>`")?;
            let name = Name::parse(&raw).map_err(|why| format!("`{raw}` is not a name: {why}"))?;
            Ok(Command::New {
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
            Ok(Command::Join {
                link: Box::new(link),
                person,
            })
        }
        "link" => Ok(Command::Link {
            group: positional.ok_or("link wants a group: `svrngs link <group>`")?,
            origin,
        }),
        "help" | "-h" | "--help" => Ok(Command::Help),
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
            | Refusal::Busy(_) => 1,
            Refusal::NoDataDir
            | Refusal::DataDir(..)
            | Refusal::Key(_)
            | Refusal::Start
            | Refusal::IssuerAway
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
pub async fn run(command: Command, root: PathBuf, relay: &RelayConfig) -> Result<(), Refusal> {
    if command == Command::Help {
        println!("{USAGE}");
        return Ok(());
    }
    let _claim = claim(&root)?;
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

/// A link as printed: the client's, and the https one for a group that
/// keeps a page origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printed {
    pub client: String,
    pub https: Option<String>,
}

impl Printed {
    fn of(token: &Token, origin: Option<&Origin>) -> Self {
        Self {
            client: client_link(token),
            https: origin.map(|o| https_link(o, token)),
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
    /// The bare command: the names of the groups it runs for, and where
    /// this computer's apps reach them.
    Running { groups: Vec<Name>, api: SocketAddr },
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
            Said::Running { groups, api } => {
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
                )
            }
        }
    }
}

/// A verb on a started node, then the wait for Ctrl-C.
async fn serve(
    command: Command,
    root: &Path,
    node: &Node,
    key: &SigningKey,
    relay: &RelayConfig,
) -> Result<(), Refusal> {
    let said = match command {
        Command::New {
            name,
            person,
            origin,
        } => {
            let person = match person {
                Some(p) => p,
                None => user_name().ok_or_else(|| Refusal::NoName(name.clone()))?,
            };
            let (seed, link) = new_group(node, name, person, origin.as_ref(), relay).await?;
            Said::Made { seed, link }
        }
        Command::Join { link, person } => {
            let person = match person {
                Some(p) => p,
                None => user_name().ok_or(Refusal::NoNameToJoin)?,
            };
            join_group(node, key, &link, &person).await?
        }
        Command::Link { group, origin } => {
            let (seed, link) = new_link(node, &group, origin.as_ref(), relay).await?;
            Said::Linked { seed, link }
        }
        Command::Run => Said::Running {
            groups: group::groups(root)
                .iter()
                .map(|s| s.name().clone())
                .collect(),
            api: node.api_addr(),
        },
        Command::Help => unreachable!("answered above"),
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
    let issuer = link.token.issuer_addr();
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
    );
    let redeemed = redeem(node.endpoint(), issuer.clone(), &request)
        .await
        .map_err(not_let_in)?;
    tracing::debug!(target: "cli", group = %id, redeemed = ?redeemed, "cli: join — let in");
    keep_group(node, &seed, link.origin.as_ref())?;
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
/// as `person`, written beside the group's record with `origin`, when
/// given, kept as its https links' page, and its first link.
pub async fn new_group(
    node: &Node,
    name: Name,
    person: Name,
    origin: Option<&Origin>,
    relay: &RelayConfig,
) -> Result<(Seed, Printed), Refusal> {
    // The founder is the key this computer signs and dials with (DESIGN
    // §8, "One key signs and dials").
    let founder = VerifyingKey::from_bytes(node.endpoint().id().as_bytes())
        .expect("an iroh endpoint id is an Ed25519 key");
    let seed = Seed::draw(name, person, founder);
    let id = seed.id();
    keep_group(node, &seed, origin)?;
    tracing::debug!(target: "cli", group = %id, name = %seed.name(), origin = ?origin.map(Origin::as_str), "cli: group made");
    let token = mint(node, &seed, relay).await?;
    Ok((seed, Printed::of(&token, origin)))
}

/// `svrngs link`: a new link to the group `reference` names: the client's
/// link, and the https one on `origin`, kept from now on, when given, else
/// on the group's kept origin, if it keeps one.
pub async fn new_link(
    node: &Node,
    reference: &str,
    origin: Option<&Origin>,
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
    let origin = match origin {
        Some(given) => {
            group::write_origin(&dir, given).map_err(|e| keep_refused(&seed, &e.to_string()))?;
            tracing::debug!(target: "cli", group = %seed.id(), origin = given.as_str(), "cli: link — the group's origin changed");
            Some(given.clone())
        }
        None => match group::read_origin(&dir).map_err(|e| keep_refused(&seed, &e.to_string()))? {
            Some(kept) => {
                tracing::debug!(target: "cli", group = %seed.id(), origin = kept.as_str(), "cli: link — on the group's kept origin too");
                Some(kept)
            }
            None => {
                tracing::debug!(target: "cli", group = %seed.id(), "cli: link — the group keeps no origin, so the client's link alone");
                None
            }
        },
    };
    let token = mint(node, &seed, relay).await?;
    Ok((seed, Printed::of(&token, origin.as_ref())))
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
    let token = link::mint(node.endpoint().id(), issuer_relay(node, relay).await);
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
    let configured = || relay.relay_urls.first().and_then(|r| r.parse().ok());
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
