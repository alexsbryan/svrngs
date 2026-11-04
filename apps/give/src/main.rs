//! The `give` command.
//!
//!   give check [--agree] [--name <name>] <card dir>
//!   give keep [--card <card dir>]...
//!   give page <group> [--card <card dir>]...
//!
//! RUST_LOG=debug shows every decision, each refusal with the field or the
//! fact that refused it.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use give::check::{Step, check};
use give::keep::{EVERY, Keeper, ROUND};
use give::podman::Podman;
use give::record::{Record, data_dir};
use give::runtime::Name;
use give::shelf::Shelf;

const USAGE: &str = "usage: give check [--agree] [--name <name>] <card dir>
       give keep [--card <card dir>]...
       give page <group> [--card <card dir>]...

  check: start the app a card describes from its pinned image, wait until
  it is healthy, snapshot it, stop it, restore the snapshot and start it
  again; then remove all of it. --agree says yes to the card's asks, once
  you have read them. --name runs it under another name than
  svrngs-check-<dir>.

  keep: keep, on this computer, each app your groups' records name it for,
  beside the svrngs running here: lend it while it is kept here, copy its
  state to the group every few minutes and as it stops, and stop it when
  another computer keeps it. Ctrl-C stops every app it keeps, each copied
  to the group first. --card puts a card on the shelf beside give's own.

  page: bring Give's page to <group> (its name or id), with the shelf of
  give's cards and any --card, through the group's folder on this computer.";

/// Asked to stop, by SIGINT or SIGTERM: the keeper program then stops each
/// app it keeps cleanly, with a last snapshot, and exits.
static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn asked_to_stop(_: libc::c_int) {
    STOP.store(true, Ordering::SeqCst);
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error")),
        )
        .with_writer(std::io::stderr)
        .init();
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let rest: Vec<String> = args.collect();
    let ran = match command.as_deref() {
        Some("check") => run_check(rest),
        Some("keep") => run_keep(rest),
        Some("page") => run_page(rest),
        _ => Err(anyhow::anyhow!("the commands are check, keep and page")),
    };
    match ran {
        Ok(code) => code,
        Err(why) => {
            eprintln!("give: {why:#}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run_check(args: Vec<String>) -> anyhow::Result<ExitCode> {
    let mut args = args.into_iter();
    let (mut agree, mut name, mut dir) = (false, None, None);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--agree" => agree = true,
            "--name" => name = Some(args.next().ok_or_else(|| anyhow::anyhow!("--name needs a name"))?),
            _ if dir.is_none() && !a.starts_with('-') => dir = Some(PathBuf::from(a)),
            _ => anyhow::bail!("{a} is not an argument check takes"),
        }
    }
    let dir = dir.ok_or_else(|| anyhow::anyhow!("check needs a card's directory"))?;
    let kind = dir
        .canonicalize()?
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let name = Name::new(&name.unwrap_or_else(|| format!("svrngs-check-{kind}")))?;
    let scratch = std::env::temp_dir().join(format!("give-check-{}", std::process::id()));
    let mut passed = |step: Step, what: String| {
        let what = if what.is_empty() {
            String::new()
        } else {
            format!(": {what}")
        };
        println!("passed  {step}{what}");
    };
    match check(&Podman::default(), &dir, name, &scratch, agree, &mut passed) {
        Ok(card) => {
            println!("{} can go on the shelf.", card.title);
            Ok(ExitCode::SUCCESS)
        }
        Err(failed) => {
            println!("failed  {failed}");
            Ok(ExitCode::FAILURE)
        }
    }
}

/// The `--card` folders, and the one word left over, if any.
fn cards_and(args: Vec<String>, takes_one: bool) -> anyhow::Result<(Vec<PathBuf>, Option<String>)> {
    let (mut cards, mut one, mut args) = (Vec::new(), None, args.into_iter());
    while let Some(a) = args.next() {
        match a.as_str() {
            "--card" => cards.push(PathBuf::from(args.next().context("--card needs a card's directory")?)),
            _ if takes_one && one.is_none() && !a.starts_with('-') => one = Some(a),
            _ => anyhow::bail!("{a} is not an argument this command takes"),
        }
    }
    Ok((cards, one))
}

fn record() -> anyhow::Result<Record> {
    let dir = data_dir().context("neither SVRNGS_DIR nor HOME is set, so svrngs's data dir is unknown")?;
    Ok(Record::at(&dir)?)
}

/// give's own files on this computer: `$GIVE_DIR`, else `~/.give`.
fn give_dir() -> anyhow::Result<PathBuf> {
    let set = std::env::var_os("GIVE_DIR")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from);
    let home = || std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".give"));
    set.or_else(home)
        .context("neither GIVE_DIR nor HOME is set, so give has nowhere to keep its own files")
}

#[expect(
    clippy::disallowed_methods,
    reason = "the keeper program's clock, read here and passed in"
)]
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn run_keep(args: Vec<String>) -> anyhow::Result<ExitCode> {
    let (cards, _) = cards_and(args, false)?;
    let shelf = Shelf::with(&cards)?;
    // What a test names its pods and volumes by, `svrngs-test-*` (CHARTER
    // fence 11), and how often it copies; a person's give uses give's own.
    let prefix = std::env::var("GIVE_TEST_PREFIX").unwrap_or_else(|_| "give-".into());
    let every = (std::env::var("GIVE_TEST_EVERY").ok().and_then(|s| s.parse().ok())).map_or(EVERY, Duration::from_secs);
    let mut keeper = Keeper::new(record()?, shelf, prefix, give_dir()?, every);
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let handler = asked_to_stop as extern "C" fn(libc::c_int) as libc::sighandler_t;
        // SAFETY: the handler only stores to an atomic, which is
        // async-signal-safe.
        unsafe { libc::signal(signal, handler) };
    }
    println!("Keeping what your groups give this computer to keep. Ctrl-C stops it.");
    let mut me = None;
    while !STOP.load(Ordering::SeqCst) {
        match keeper.round(now()) {
            Ok(key) => me = Some(key),
            Err(why) => eprintln!("give: {why}"),
        }
        let mut waited = Duration::ZERO;
        while waited < ROUND && !STOP.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(100));
            waited += Duration::from_millis(100);
        }
    }
    if let Some(me) = me {
        keeper.stop_all(&me);
    }
    println!("Stopped.");
    Ok(ExitCode::SUCCESS)
}

fn run_page(args: Vec<String>) -> anyhow::Result<ExitCode> {
    let (cards, group) = cards_and(args, true)?;
    let group = group.context("page needs a group: its name or its id")?;
    let shelf = Shelf::with(&cards)?;
    let groups = record()?.groups()?;
    let mut named = groups.groups.iter().filter(|g| g.id == group || g.name == group);
    let found = named
        .next()
        .with_context(|| format!("this computer is in no group called {group}"))?;
    anyhow::ensure!(
        named.next().is_none(),
        "two groups here are called {group}: name it by its id"
    );
    let folder = (found.folder.as_ref()).with_context(|| {
        format!(
            "this computer keeps no folder of {}, so it cannot bring the page there",
            found.name
        )
    })?;
    let wrote = give::page::bring(folder, &shelf)?;
    tracing::debug!(group = found.id, wrote = wrote.len(), "page: brought");
    println!(
        "Give is in {}: everyone in it can open it in a few seconds.",
        found.name
    );
    Ok(ExitCode::SUCCESS)
}
