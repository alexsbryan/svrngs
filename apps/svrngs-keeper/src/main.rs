//! The `svrngs-keeper` command: keep a card running on this computer.
//! RUST_LOG=debug shows every decision, each refusal with what refused it.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use svrngs_keeper::kept::{self, Boot, Places, Stored};
use svrngs_keeper::runtime::Name;
use svrngs_keeper::systemd::SystemdUser;

#[cfg(target_os = "linux")]
mod facts;
#[cfg(target_os = "linux")]
mod native_run;

const USAGE: &str = "usage: svrngs-keeper keep [--name <name>] <card>
       svrngs-keeper run <name> --port <port> [--folder <folder>] [--json] -- <command>
       svrngs-keeper facts <port> [--name <name>]
       svrngs-keeper list [--json]
       svrngs-keeper stop|start|restart|remove <name>

  keep: run a native card (a compose.yaml with x-svrngs.native, or the
  folder it is in) here for good, as the systemd user unit
  svrngs-keeper-<name>.service. It starts again if it stops, and when you
  log in; when this computer starts, too, if you let your account run with
  nobody logged in (the first keep asks). <name> is the card's title
  unless --name gives one. The same card again changes nothing; a new one
  replaces it, and the one before is kept beside it as previous.yaml.

  facts: suggest a llama-server card to review, as JSON (Linux only).
  list: each card kept here, and whether it runs.
  stop: stop it until you start it, at boot too. start: run it again.
  restart: restart it if it is running.
  remove: run it here no longer. Its card and its folder stay.";

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error")),
        )
        .with_writer(std::io::stderr)
        .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("svrngs-keeper: {why:#}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn places() -> anyhow::Result<Places> {
    let home = std::env::var_os("HOME").context("HOME is not set, so there is no folder to keep cards in")?;
    let var = |k: &str| std::env::var(k).ok();
    Ok(Places::of(
        &PathBuf::from(home),
        var("XDG_DATA_HOME").as_deref(),
        var("XDG_CONFIG_HOME").as_deref(),
    ))
}

fn run(args: &[String]) -> anyhow::Result<()> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["facts", port] => return at_port(port, "llama-server"),
        ["facts", port, "--name", name] => return at_port(port, name),
        #[cfg(target_os = "linux")]
        ["run", name, rest @ ..] => return native_run::run(name, rest),
        _ => (),
    }
    run_card(&SystemdUser::system(), &places()?, &args)
}

#[cfg(target_os = "linux")]
fn at_port(port: &str, name: &str) -> anyhow::Result<()> {
    let port = port
        .parse::<std::num::NonZeroU16>()
        .context("give a port from 1 to 65535")?;
    let name = Name::new(name)?;
    json(&facts::read(port.get(), name.as_str()))
}

#[cfg(not(target_os = "linux"))]
fn at_port(_: &str, _: &str) -> anyhow::Result<()> {
    anyhow::bail!("facts needs Linux to read listeners and suggest a command")
}

fn run_card(systemd: &SystemdUser, places: &Places, args: &[&str]) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    let _lock = match args {
        ["stop" | "start" | "restart" | "remove", name] => {
            Some(lock(places, &format!("name-{}", Name::new(name)?.as_str()))?)
        }
        _ => None,
    };
    match args {
        ["keep", "--name", name, card] | ["keep", card, "--name", name] => keep(systemd, places, card, Some(name)),
        ["keep", card] => keep(systemd, places, card, None),
        ["list"] => {
            let rows = kept::list(systemd, places)?;
            if rows.is_empty() {
                println!("svrngs-keeper keeps nothing here yet: svrngs-keeper keep <card>");
            }
            for row in rows {
                println!("{row}");
            }
            Ok(())
        }
        ["list", "--json"] => json(&kept::list(systemd, places)?),
        ["stop", name] => {
            let name = kept::stop(systemd, places, name)?;
            let name = name.as_str();
            println!("{name} is stopped until you start it again, at boot too: svrngs-keeper start {name}");
            Ok(())
        }
        ["start", name] => {
            let name = kept::start(systemd, places, name)?;
            let starts = Boot::read(systemd.lingers()).starts();
            println!("{} runs again. {starts} and starts again if it stops.", name.as_str());
            Ok(())
        }
        ["restart", name] => {
            let name = kept::restart(systemd, places, name)?;
            println!("{} was asked to restart if running.", name.as_str());
            Ok(())
        }
        ["remove", name] => {
            let dir = kept::remove(systemd, places, name)?;
            println!(
                "{name} is no longer run here. Its card stays in {}, and its folder is as it was.",
                dir.display()
            );
            Ok(())
        }
        _ => anyhow::bail!("the commands are run, keep, facts, list, stop, start, restart and remove"),
    }
}

#[cfg(target_os = "linux")]
fn lock(places: &Places, key: &str) -> anyhow::Result<std::fs::File> {
    // Card commands still serialize writes, without reading any live process.
    let dir = places.data.join("locks");
    std::fs::create_dir_all(&dir)?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(key))?;
    file.try_lock()
        .with_context(|| format!("keeper lock {key} could not be taken"))?;
    Ok(file)
}

fn keep(systemd: &SystemdUser, places: &Places, card: &str, name: Option<&str>) -> anyhow::Result<()> {
    let mut path = PathBuf::from(card);
    if path.is_dir() {
        path = path.join("compose.yaml");
    }
    let text = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let card = svrngs_keeper::card::Card::parse(&text)?;
    let name = Name::new(&name.map_or_else(|| kept::name_of(&card.title), str::to_owned))?;
    #[cfg(target_os = "linux")]
    let _locks = (
        lock(places, &format!("port-{}", card.door.port))?,
        lock(places, &format!("name-{}", name.as_str()))?,
    );
    let kept = kept::keep_card(systemd, places, &text, &card, name, || kept::boot(systemd, terminal))?;
    let (name, port, unit) = (kept.name.as_str(), kept.port, &kept.unit);
    match kept.stored {
        Stored::New => println!("{name} is kept here now, for port {port}: the systemd unit {unit}."),
        Stored::Same => println!("{name} was kept here already from this same card, so nothing was reset."),
        Stored::Changed { previous, restarted } => println!(
            "{name} is kept from its new card{}; the card before it is in {}.",
            if restarted { ", and started again from it" } else { "" },
            previous.display()
        ),
    }
    // Read again: a yes logind refused starts it at login all the same.
    println!(
        "{} and starts again if it stops (4 times a minute at most).",
        Boot::read(systemd.lingers()).starts()
    );
    Ok(())
}

fn json(value: &impl serde::Serialize) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

/// Ask `question` if stdin is a terminal; `None` when it is not.
fn terminal(question: &str) -> Option<bool> {
    use std::io::{IsTerminal, Write};
    std::io::stdin().is_terminal().then(|| {
        print!("{question} Type y, then press Enter: ");
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
        matches!(line.trim(), "y" | "yes")
    })
}
