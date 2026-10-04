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
use std::path::{Path, PathBuf};
use std::time::Duration;

use ed25519_dalek::VerifyingKey;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::{Node, NodeError};
use svrngs_node::redeem::{redeem, RedeemError, RedeemRequest, Redeemed};
use svrngs_node::{actor_of, Ed25519Verifier, Person, RailAct, RailError, SigningKey};

use crate::group::{self, Name, Seed, SeedError};
use crate::identity::{self, KeyError};
use crate::link::{build_https_link, parse_https_link, Dial, Link, Origin};

pub const USAGE: &str = "\
svrngs — keep what your group shares on your own computers.

  svrngs
      Run for every group this computer is in. Leave it running.

  svrngs new <name> [--as <your name>] [--origin <url>]
      Make a group called <name> and print its link: send it to the people
      you want in it. It works for a day, while this computer is on.
      --as is the name the group knows you by; else this computer's user name.
      --origin prints the link on a page address of your own.

  svrngs join <link> [--as <your name>]
      Join the group a link you were sent is for. The computer that sent it
      must be on. --as is the name the group knows you by; else this
      computer's user name.

  svrngs link <group> [--origin <url>]
      Print a new link to <group>. Any link you sent before stops letting
      anyone new in; everyone it let in stays. <group> is its name, or the
      first 8 letters of its id when two groups share a name.

Data:    $SVRNGS_DIR, else ~/.svrngs.
Logging: RUST_LOG, default `error`.";

/// How long a link lets people in: upstream's one invite lapse
/// (`commonwealth-discovery/src/membership.rs:53` at `e1a43bcd2`, a day;
/// DESIGN §8, U2's inventory).
pub const LINK_LIFE_SECS: i64 = 24 * 60 * 60;

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
        origin: Origin,
    },
    Join {
        link: Box<Link>,
        person: Option<Name>,
    },
    Link {
        group: String,
        origin: Origin,
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
                person =
                    Some(Name::parse(&raw).ok_or_else(|| format!("--as `{raw}` is not a name"))?);
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
    let origin = origin.unwrap_or_default();
    match verb.as_str() {
        "new" => {
            let raw = positional.ok_or("new wants a name: `svrngs new <name>`")?;
            let name = Name::parse(&raw).ok_or_else(|| format!("`{raw}` is not a name"))?;
            Ok(Command::New {
                name,
                person,
                origin,
            })
        }
        "join" => {
            let raw = positional.ok_or("join wants a link: `svrngs join <link>`")?;
            let link = parse_https_link(&raw).ok_or_else(|| {
                format!("`{raw}` is not a whole link; copy all of it, from https to the end")
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
    #[error(
        "this computer has no address another can reach yet; check it is online and try again"
    )]
    Unreachable,
    #[error("who are you in {0}? say `svrngs new {0} --as <your name>`")]
    NoName(Name),
    #[error("who are you in {0}? open the link again with `svrngs join <link> --as <your name>`")]
    NoNameToJoin(Name),
    #[error("the link to {0} lets nobody new in: ask whoever sent it for a new one")]
    LinkRefused(Name),
    #[error("the computer that sent the link to {0} cannot be reached; it may be off, so try again later")]
    IssuerAway(Name),
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
            | Refusal::NoNameToJoin(_)
            | Refusal::LinkRefused(_)
            | Refusal::NoGroup(_)
            | Refusal::Ambiguous { .. }
            | Refusal::NotIn(_)
            | Refusal::Busy(_) => 1,
            Refusal::NoDataDir
            | Refusal::DataDir(..)
            | Refusal::Key(_)
            | Refusal::Start
            | Refusal::Unreachable
            | Refusal::IssuerAway(_)
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
    let node = Node::start(root.clone(), key.clone(), relay)
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

/// What a verb tells the person, in DESIGN §1's words alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Said {
    /// `new`: the group made, and its first link.
    Made { seed: Seed, url: String },
    /// `link`: a new link to a group.
    Linked { seed: Seed, url: String },
    /// `join`: let in by the link's issuer, as `person`.
    Joined { seed: Seed, person: Name },
    /// `join` of a group this computer is in already.
    AlreadyIn(Seed),
    /// The bare command: the names of the groups it runs for.
    Running(Vec<Name>),
}

impl std::fmt::Display for Said {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Said::Made { seed, url } => {
                writeln!(f, "Made {}, with you as {}.", seed.name(), seed.founder())?;
                writeln!(f, "Send this link to the people you want in it. It works for a day, while this computer is on:")?;
                writeln!(f)?;
                writeln!(f, "  {url}")
            }
            Said::Linked { seed, url } => {
                writeln!(
                    f,
                    "A new link to {}. Any link you sent before stops letting anyone new in; everyone it let in stays.",
                    seed.name()
                )?;
                writeln!(f, "It works for a day, while this computer is on:")?;
                writeln!(f)?;
                writeln!(f, "  {url}")
            }
            Said::Joined { seed, person } => {
                writeln!(f, "You're in {}, as {person}.", seed.name())
            }
            Said::AlreadyIn(seed) => {
                writeln!(f, "You're in {} already, on this computer.", seed.name())
            }
            Said::Running(names) if names.is_empty() => writeln!(
                f,
                "This computer is in no group yet: make one with `svrngs new <name>`."
            ),
            Said::Running(names) => {
                let names: Vec<&str> = names.iter().map(Name::as_str).collect();
                writeln!(f, "Running for {}.", names.join(", "))
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
            let (seed, url) = new_group(node, name, person, &origin, relay).await?;
            Said::Made { seed, url }
        }
        Command::Join { link, person } => {
            let person = match person {
                Some(p) => p,
                None => {
                    user_name().ok_or_else(|| Refusal::NoNameToJoin(link.seed.name().clone()))?
                }
            };
            join_group(node, key, &link, &person).await?
        }
        Command::Link { group, origin } => {
            let (seed, url) = new_link(node, &group, &origin, relay).await?;
            Said::Linked { seed, url }
        }
        Command::Run => Said::Running(
            group::groups(root)
                .iter()
                .map(|s| s.name().clone())
                .collect(),
        ),
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

/// `svrngs join`: redeem `link` at its issuer's computer as `person`,
/// signing with `key`, this computer's; keep the group's seed once let in;
/// sync once from the issuer, whom this computer's copy of the register may
/// not know yet (DESIGN §8, U2). The seed is written only once the issuer
/// has let this computer in, so a refused link leaves nothing behind.
pub async fn join_group(
    node: &Node,
    key: &SigningKey,
    link: &Link,
    person: &Name,
) -> Result<Said, Refusal> {
    let seed = &link.seed;
    let id = seed.id();
    if group::groups(node.rail().root())
        .iter()
        .any(|s| s.id() == id)
    {
        tracing::debug!(target: "cli", group = %id, "cli: join — this computer holds the group already, so nothing is redeemed");
        return Ok(Said::AlreadyIn(seed.clone()));
    }
    let request = RedeemRequest::sign(
        &link.secret,
        key,
        &id,
        &link.invite,
        &Person::from(person.as_str()),
    );
    let redeemed = redeem(node.endpoint(), link.issuer.addr().clone(), &request)
        .await
        .map_err(|e| {
            tracing::debug!(target: "cli", group = %id, error = %e, "cli: join — not let in");
            match e {
                RedeemError::Refused(_) => Refusal::LinkRefused(seed.name().clone()),
                RedeemError::Unjudged | RedeemError::NoAnswer(_) => {
                    Refusal::IssuerAway(seed.name().clone())
                }
            }
        })?;
    tracing::debug!(target: "cli", group = %id, redeemed = ?redeemed, "cli: join — let in");
    let journal = node
        .rail()
        .journal(&id)
        .map_err(|e| keep_refused(seed, &e.to_string()))?;
    group::write_seed(journal.dir(), seed)
        .map_err(|e: SeedError| keep_refused(seed, &e.to_string()))?;
    journal
        .set_roster(&seed.roster())
        .map_err(|e| keep_refused(seed, &e.to_string()))?;
    let synced = node.sync_with(&id, link.issuer.addr()).await;
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
        seed: seed.clone(),
        person: person.clone(),
    })
}

/// This computer's user name, as the name a group knows its founder or a
/// joiner by when `--as` names none. `new` and `join` print it, so it is
/// never silent.
fn user_name() -> Option<Name> {
    let raw = std::env::var("USER").ok()?;
    tracing::debug!(target: "cli", user = %raw, "cli: the founder's name from $USER");
    Name::parse(&raw)
}

/// `svrngs new`: a seed drawn for `name`, founded by this computer's key
/// as `person`, written beside the group's record, and its first link.
pub async fn new_group(
    node: &Node,
    name: Name,
    person: Name,
    origin: &Origin,
    relay: &RelayConfig,
) -> Result<(Seed, String), Refusal> {
    // The founder is the key this computer signs and dials with (DESIGN
    // §8, "One key signs and dials").
    let founder = VerifyingKey::from_bytes(node.endpoint().id().as_bytes())
        .expect("an iroh endpoint id is an Ed25519 key");
    let seed = Seed::draw(name, person, founder);
    let id = seed.id();
    let journal = node
        .rail()
        .journal(&id)
        .map_err(|e| keep_refused(&seed, &e.to_string()))?;
    group::write_seed(journal.dir(), &seed)
        .map_err(|e: SeedError| keep_refused(&seed, &e.to_string()))?;
    journal
        .set_roster(&seed.roster())
        .map_err(|e| keep_refused(&seed, &e.to_string()))?;
    tracing::debug!(target: "cli", group = %id, name = %seed.name(), "cli: group made");
    let url = mint(node, &seed, origin, relay).await?;
    Ok((seed, url))
}

/// `svrngs link`: a new link to the group `reference` names.
pub async fn new_link(
    node: &Node,
    reference: &str,
    origin: &Origin,
    relay: &RelayConfig,
) -> Result<(Seed, String), Refusal> {
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
    let url = mint(node, &seed, origin, relay).await?;
    Ok((seed, url))
}

/// Append an `Invite` for a fresh link key that lapses in
/// [`LINK_LIFE_SECS`], signed by this computer through the group's door,
/// and build the link. The issuer's newer `Invite` closes their last one
/// from its place in the order (u2-4b, `membership`).
async fn mint(
    node: &Node,
    seed: &Seed,
    origin: &Origin,
    relay: &RelayConfig,
) -> Result<String, Refusal> {
    let issuer = dial(node, relay).await?;
    let mut secret = [0u8; 32];
    getrandom::fill(&mut secret).expect("failed to draw a link's key");
    let secret = SigningKey::from_bytes(&secret);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|_| keep_refused(seed, "the wall clock is before 1970"))?;
    let invite = RailAct::Invite {
        key: actor_of(&secret),
        until: now + LINK_LIFE_SECS,
    };
    let journal = node
        .rail()
        .journal(&seed.id())
        .map_err(|e| keep_refused(seed, &e.to_string()))?;
    let op = journal
        .append(invite, node.rail().signer(), &seed.roster(), None, &Ed25519Verifier)
        .map_err(|e| match e {
            RailError::NotInRoster { .. } => {
                tracing::debug!(target: "cli", group = %seed.id(), error = %e, "cli: refused — this computer does not stand in the group");
                Refusal::NotIn(seed.name().clone())
            }
            other => keep_refused(seed, &other.to_string()),
        })?;
    tracing::debug!(target: "cli", group = %seed.id(), invite = %op.id, until = now + LINK_LIFE_SECS, "cli: link minted");
    let link = Link {
        seed: seed.clone(),
        secret,
        invite: op.id,
        issuer,
    };
    Ok(build_https_link(&link, origin))
}

/// This computer's dial, waiting up to [`ONLINE_WAIT`] for a relay when it
/// has one to reach.
async fn dial(node: &Node, relay: &RelayConfig) -> Result<Dial, Refusal> {
    let endpoint = node.endpoint();
    if relay.n0_services || !relay.relay_urls.is_empty() {
        let online = tokio::time::timeout(ONLINE_WAIT, endpoint.online())
            .await
            .is_ok();
        tracing::debug!(target: "cli", online, "cli: waited for a relay before printing the link");
    }
    Dial::of(&endpoint.addr()).ok_or_else(|| {
        tracing::debug!(target: "cli", "cli: refused — the endpoint has no address yet");
        Refusal::Unreachable
    })
}

fn keep_refused(seed: &Seed, why: &str) -> Refusal {
    tracing::debug!(target: "cli", group = %seed.id(), why, "cli: refused — the group cannot be kept here");
    Refusal::Keep(seed.name().clone())
}
