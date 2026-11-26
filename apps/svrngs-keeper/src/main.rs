//! The `svrngs-keeper` command: keep a card running on this computer.
//! RUST_LOG=debug shows every decision, each refusal with what refused it.
#![deny(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use svrngs_keeper::kept::{self, Places};
#[cfg(not(target_os = "macos"))]
use svrngs_keeper::kept::{Boot, Stored};
use svrngs_keeper::runtime::Name;
#[cfg(not(target_os = "macos"))]
use svrngs_keeper::systemd::SystemdUser;

#[cfg(target_os = "linux")]
mod container_run;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod native_run;

const USAGE: &str = "usage: svrngs-keeper keep [--name <name>] <card>
       svrngs-keeper run <name> --port <port> [--folder <folder>] [<check>…] -- <command>
       svrngs-keeper run <name> --port <port> --folder <folder> --command <line> [<check>…]
       svrngs-keeper run <name> --port <port> --card <compose.yaml> [--json]
       svrngs-keeper list [--json]
       svrngs-keeper stop|start|restart|remove <name>

  keep (Linux): run a native card (a compose.yaml with x-svrngs.native, or the
  folder it is in) here for good, as the systemd user unit
  svrngs-keeper-<name>.service. It starts again if it stops, and when you
  log in; when this computer starts, too, if you let your account run with
  nobody logged in (the first keep asks). <name> is the card's title
  unless --name gives one. The same card again changes nothing; a new one
  replaces it, and the one before is kept beside it as previous.yaml.

  run: start a program under the service manager and keep it only once it
  answers on <port> as that job; anything else is stopped and removed, and
  why is said. The words after -- are run as given, in <folder> or the one
  this starts in; a --command line is split as a shell would split it, ~
  read as your home, and anything only a shell understands refused. Its
  checks: --synced <folder> (again for more) names a folder nothing kept
  may be in; --at-boot says yes to starting before anyone logs in (Linux);
  --json answers in JSON, its refusals too. Words after -- are shown, and
  kept only once you press Enter in a terminal.

  list: each card kept here, and whether it runs.
  stop: stop it until you start it, including after signing in. start: run it again.
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
    let args = match args.as_slice() {
        [flag, build, rest @ ..] if flag == "--for-svrngs" => {
            if build != option_env!("SVRNGS_COMMIT").unwrap_or("dev") {
                eprintln!("Keeper is from a different svrngs build.");
                return ExitCode::from(78);
            }
            rest
        }
        args => args,
    };
    match run(args) {
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
    let home = PathBuf::from(home);
    let places = Places::of(
        &home,
        var("XDG_DATA_HOME").as_deref(),
        var("XDG_CONFIG_HOME").as_deref(),
    );
    #[cfg(target_os = "macos")]
    let places = Places {
        units: home.join("Library/LaunchAgents"),
        ..places
    };
    Ok(places)
}

fn run(args: &[String]) -> anyhow::Result<()> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        ["run", name, rest @ ..] => return native_run::run(name, rest),
        #[cfg(target_os = "linux")]
        ["serve-card", rest @ ..] => return container_run::serve(rest),
        _ => (),
    }
    #[cfg(target_os = "macos")]
    {
        run_agent(&places()?, &args)
    }
    #[cfg(not(target_os = "macos"))]
    {
        run_card(&SystemdUser::system(), &places()?, &args)
    }
}

#[cfg(target_os = "macos")]
fn run_agent(places: &Places, args: &[&str]) -> anyhow::Result<()> {
    match args {
        ["list"] | ["list", "--json"] => {
            let rows = kept::listed(places, |name| {
                let agent = places.agent(name).map_err(|e| e.to_string())?;
                if agent.installed() {
                    agent.observe().map(|(status, _)| Some(status))
                } else {
                    Ok(None)
                }
            })?;
            if args.len() == 2 {
                return json(&rows);
            }
            for row in rows {
                println!("{row}");
            }
        }
        [verb @ ("start" | "stop" | "restart" | "remove"), name] => {
            let name = Name::new(name)?;
            let _lock = lock(places, &format!("name-{}", name.as_str()))?;
            let agent = places.agent(&name)?;
            match *verb {
                "start" => agent.start(),
                "stop" => agent.stop(),
                "restart" => agent.restart(),
                _ => agent.remove(),
            }
            .map_err(anyhow::Error::msg)?;
            println!("{}: {verb} requested", name.as_str());
        }
        _ => anyhow::bail!("on this Mac, use run, list, stop, start, restart or remove"),
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
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
        _ => anyhow::bail!("the commands are run, keep, list, stop, start, restart and remove"),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn lock(places: &Places, key: &str) -> anyhow::Result<std::fs::File> {
    // Card commands still serialize writes, without reading any live process.
    let dir = places.data.join("locks");
    kept::private_dir(&places.data)?;
    kept::private_dir(&dir)?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(key))?;
    file.try_lock()
        .with_context(|| format!("keeper lock {key} could not be taken"))?;
    Ok(file)
}

#[cfg(not(target_os = "macos"))]
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
    let kept = kept::keep_card(systemd, places, &card, name, || kept::boot(systemd, terminal))?;
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
#[cfg(not(target_os = "macos"))]
fn terminal(question: &str) -> Option<bool> {
    use std::io::{IsTerminal, Write};
    std::io::stdin().is_terminal().then(|| {
        eprint!("{question} [y/N] ");
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
        matches!(line.trim(), "y" | "Y" | "yes")
    })
}
