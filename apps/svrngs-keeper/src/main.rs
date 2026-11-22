//! The `svrngs-keeper` command: keep a card running on this computer.
//!
//!   svrngs-keeper keep [--name <name>] <card>
//!   svrngs-keeper list
//!   svrngs-keeper stop|start|remove <name>
//!
//! RUST_LOG=debug shows every decision, each refusal with what refused it.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use svrngs_keeper::kept::{self, Places, Stored};
use svrngs_keeper::systemd::SystemdUser;

const USAGE: &str = "usage: svrngs-keeper keep [--name <name>] <card>
       svrngs-keeper list
       svrngs-keeper stop|start|remove <name>

  keep: run a native card (a compose.yaml with x-svrngs.native, or the
  folder it is in) here for good, as the systemd user unit
  svrngs-keeper-<name>.service. It starts when this computer starts, and
  again if it stops. <name> is the card's title unless --name gives one.
  The same card again changes nothing; a new one replaces it, and the one
  before is kept beside it as previous.yaml.

  list: each card kept here, and whether it runs.
  stop: stop it until you start it, at boot too. start: run it again.
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
    let systemd = SystemdUser::system();
    let places = places()?;
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["keep", "--name", name, card] | ["keep", card, "--name", name] => keep(&systemd, &places, card, Some(name)),
        ["keep", card] => keep(&systemd, &places, card, None),
        ["list"] => {
            let rows = kept::list(&systemd, &places)?;
            if rows.is_empty() {
                println!("svrngs-keeper keeps nothing here yet: svrngs-keeper keep <card>");
            }
            for row in rows {
                println!("{row}");
            }
            Ok(())
        }
        ["stop", name] => {
            let name = kept::stop(&systemd, &places, name)?;
            let name = name.as_str();
            println!("{name} is stopped until you start it again, at boot too: svrngs-keeper start {name}");
            Ok(())
        }
        ["start", name] => {
            let name = kept::start(&systemd, &places, name)?;
            println!("{} runs again, and starts when this computer starts.", name.as_str());
            Ok(())
        }
        ["remove", name] => {
            let dir = kept::remove(&systemd, &places, name)?;
            println!(
                "{name} is no longer run here. Its card stays in {}, and its folder is as it was.",
                dir.display()
            );
            Ok(())
        }
        _ => anyhow::bail!("the commands are keep, list, stop, start and remove"),
    }
}

fn keep(systemd: &SystemdUser, places: &Places, card: &str, name: Option<&str>) -> anyhow::Result<()> {
    let mut path = PathBuf::from(card);
    if path.is_dir() {
        path = path.join("compose.yaml");
    }
    let text = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let kept = kept::keep(systemd, places, &text, name)?;
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
    let starts = match systemd.lingers() {
        Ok(true) => "It starts when this computer starts, before anyone logs in,",
        Ok(false) => "It starts when you log in,",
        Err(_) => "It starts when you log in at least,",
    };
    println!("{starts} and starts again if it stops (4 times a minute at most).");
    Ok(())
}
