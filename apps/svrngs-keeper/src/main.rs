//! The `svrngs-keeper` command: keep a card running on this computer.
//!
//!   svrngs-keeper keep [--name <name>] <card>
//!   svrngs-keeper keep <port> [--lend <lend>] [--name <name>]
//!   svrngs-keeper facts <port> [--name <name>]
//!   svrngs-keeper list [--json]
//!   svrngs-keeper stop|start|restart|remove <name>
//!
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
mod takeover;

const USAGE: &str = "usage: svrngs-keeper keep [--name <name>] <card>
       svrngs-keeper keep <port> [--lend <lend>] [--name <name>]
       svrngs-keeper facts <port> [--name <name>]
       svrngs-keeper keep <port> [--name <name>] --yes --token <token>
       svrngs-keeper list [--json]
       svrngs-keeper stop|start|restart|remove <name>

  keep: run a native card (a compose.yaml with x-svrngs.native, or the
  folder it is in) here for good, as the systemd user unit
  svrngs-keeper-<name>.service. It starts again if it stops, and when you
  log in; when this computer starts, too, if you let your account run with
  nobody logged in (the first keep asks). <name> is the card's title
  unless --name gives one. The same card again changes nothing; a new one
  replaces it, and the one before is kept beside it as previous.yaml.

  keep <port>: keep what runs on <port> now, in place of the copy you
  started. llama-server is kept from the card svrngs-keeper ships for it;
  anything else is copied from how it was started, with none of your
  terminal's settings. It shows what it will run and asks first. Then the
  one you started stops, the kept one starts, and if it does not answer
  within 30 s yours is started again as it was. --lend <lend> names it
  (unless --name does) and then runs svrngs approve <lend>, so the lend
  follows the kept copy.

  facts: review a port as JSON. --yes needs the token from that review.
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
        Ok(code) => code,
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

fn run(args: &[String]) -> anyhow::Result<ExitCode> {
    let systemd = SystemdUser::system();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    #[cfg(target_os = "linux")]
    if let ["__rollback", path] = args.as_slice() {
        return takeover::exec_original(path.as_ref()).map(|()| ExitCode::SUCCESS);
    }
    let places = places()?;
    #[cfg(target_os = "linux")]
    takeover::reconcile(&systemd, &places)?;
    let port = |p: &str| p.parse::<u16>().ok().filter(|p| *p != 0);
    if let [verb @ ("keep" | "facts"), p, rest @ ..] = args.as_slice()
        && let Some(port) = port(p)
    {
        return at_port(&systemd, &places, verb, port, rest);
    }
    run_card(&systemd, &places, &args).map(|()| ExitCode::SUCCESS)
}

fn run_card(systemd: &SystemdUser, places: &Places, args: &[&str]) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    let _lock = match args {
        ["stop" | "start" | "restart" | "remove", name] => Some(takeover::lock_card(places, &Name::new(name)?)?),
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
        _ => anyhow::bail!("the commands are keep, facts, list, stop, start, restart and remove"),
    }
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
        takeover::lock_free_port(places, card.door.port)?,
        takeover::lock_card(places, &name)?,
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

#[cfg(not(target_os = "linux"))]
fn at_port(_: &SystemdUser, _: &Places, verb: &str, port: u16, _: &[&str]) -> anyhow::Result<ExitCode> {
    anyhow::bail!("{verb} {port} reads how a program was started from Linux's /proc, and this is not Linux")
}

#[cfg(target_os = "linux")]
fn at_port(
    systemd: &SystemdUser,
    places: &Places,
    verb: &str,
    port: u16,
    mut args: &[&str],
) -> anyhow::Result<ExitCode> {
    use facts::{Draft, Facts};
    let (mut lend, mut name, mut token, mut yes) = (None, None, None, false);
    while !args.is_empty() {
        match args {
            ["--name", n, rest @ ..] if name.is_none() => {
                name = Some(*n);
                args = rest;
            }
            ["--lend", l, rest @ ..] if verb == "keep" && lend.is_none() => {
                lend = Some(*l);
                args = rest;
            }
            ["--token", t, rest @ ..] if verb == "keep" && token.is_none() => {
                token = Some(*t);
                args = rest;
            }
            ["--yes", rest @ ..] if verb == "keep" && !yes => {
                yes = true;
                args = rest;
            }
            _ => anyhow::bail!("unrecognized or repeated argument: {}", args[0]),
        }
    }
    anyhow::ensure!(
        yes == token.is_some() && !(yes && lend.is_some()),
        "--yes needs --token from facts and cannot be used with --lend"
    );
    let draft = Draft::read(systemd, &takeover::boot_id()?, port, name.or(lend));
    if verb == "facts" {
        json(&draft.map_or_else(|declined| Facts::refused(port, declined), Draft::facts))?;
        return Ok(ExitCode::SUCCESS);
    }
    let draft = match draft {
        Ok(draft) => draft,
        Err(why) => {
            tracing::debug!(%why, "takeover:declined");
            println!("svrngs-keeper can't keep port {port}: {why}. Nothing was changed.");
            return Ok(ExitCode::from(1));
        }
    };
    let offer = &draft.offer;
    let boot = if let Some(token) = token {
        anyhow::ensure!(token == offer.token, "what runs there changed; review it again");
        Boot::read(offer.linger.clone())
    } else {
        println!("svrngs-keeper will run {}: {}.", offer.name, offer.what);
        let flags = &offer.omitted_flags;
        if !flags.is_empty() {
            let them = if flags.len() == 1 { "it" } else { "them" };
            println!(
                "Yours was also started with {}: this one starts without {them}.",
                flags.join(", ")
            );
        }
        if !offer.omitted_settings.is_empty() {
            println!(
                "It starts with none of the settings your terminal added: {}.",
                offer.omitted_settings.join(", ")
            );
        }
        let boot = kept::boot(systemd, terminal);
        println!(
            "{} and starts again if it stops (4 times a minute at most).",
            boot.starts()
        );
        if !ask("The program you started stops, and this one takes its place. Keep it running?") {
            println!("Nothing was changed: yours runs as it was.");
            return Ok(ExitCode::from(1));
        }
        boot
    };
    let unit = svrngs_keeper::unit::unit_name(&draft.name);
    match takeover::take(systemd, places, &draft, boot)? {
        takeover::Outcome::Verified => {
            if yes {
                json(&serde_json::json!({"kind": "verified", "name": offer.name, "port": port, "unit": unit}))?;
                return Ok(ExitCode::SUCCESS);
            }
            println!(
                "{} runs here now, and it answers on port {port}: the systemd unit {unit}.",
                offer.name
            );
            Ok(approve(lend, port))
        }
        takeover::Outcome::Restored(why) => {
            if yes {
                json(&serde_json::json!({"kind": "rolled_back", "why": why}))?;
                return Ok(ExitCode::from(1));
            }
            println!(
                "The kept copy failed: {why}. So yours is started again as it was, with your terminal's settings, \
                and it answers on port {port}. See journalctl --user -u {unit}."
            );
            Ok(ExitCode::from(1))
        }
        takeover::Outcome::Unchanged(why) => anyhow::bail!("nothing was changed: {why}"),
    }
}

/// Ask `question` if stdin is a terminal; `None` when it is not.
fn terminal(question: &str) -> Option<bool> {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal().then(|| ask(question))
}

/// Ask `question`; yes is `y` or `yes` on its own line.
fn ask(question: &str) -> bool {
    use std::io::Write;
    print!("{question} Type y, then press Enter: ");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    matches!(line.trim(), "y" | "yes")
}

/// After the handover, `svrngs approve <lend>` in this same terminal, when
/// a lend was named: svrngs knows which lend the port is, and asks.
#[cfg(target_os = "linux")]
fn approve(lend: Option<&str>, port: u16) -> ExitCode {
    let Some(lend) = lend else {
        println!(
            "If port {port} is lent, svrngs holds its lend until you approve this one: svrngs approve <the lend>."
        );
        return ExitCode::SUCCESS;
    };
    match std::process::Command::new("svrngs").args(["approve", lend]).status() {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(s) => {
            println!("svrngs approve {lend} ended ({s}): run it again to keep lending {lend}.");
            ExitCode::from(1)
        }
        Err(e) => {
            println!("svrngs could not be run ({e}): svrngs approve {lend} keeps lending it.");
            ExitCode::from(1)
        }
    }
}
