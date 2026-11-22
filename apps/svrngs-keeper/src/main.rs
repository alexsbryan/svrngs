//! The `svrngs-keeper` command: keep a card running on this computer.
//!
//!   svrngs-keeper keep [--name <name>] <card>
//!   svrngs-keeper keep <port> [--lend <lend>] [--name <name>]
//!   svrngs-keeper list
//!   svrngs-keeper stop|start|remove <name>
//!
//! RUST_LOG=debug shows every decision, each refusal with what refused it.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use svrngs_keeper::kept::{self, Boot, Places, Stored};
use svrngs_keeper::systemd::SystemdUser;

const USAGE: &str = "usage: svrngs-keeper keep [--name <name>] <card>
       svrngs-keeper keep <port> [--lend <lend>] [--name <name>]
       svrngs-keeper list
       svrngs-keeper stop|start|remove <name>

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
    let places = places()?;
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let port = |p: &str| p.parse::<u16>().ok().filter(|p| *p != 0);
    if let ["keep", p, rest @ ..] = args.as_slice()
        && let Some(port) = port(p)
    {
        let (mut lend, mut name) = (None, None);
        for pair in rest.chunks(2) {
            match pair {
                ["--lend", l] => lend = Some(*l),
                ["--name", n] => name = Some(*n),
                _ => anyhow::bail!("keep <port> takes --lend <lend> and --name <name>"),
            }
        }
        return promote(&systemd, &places, port, lend, name);
    }
    run_card(&systemd, &places, &args).map(|()| ExitCode::SUCCESS)
}

fn run_card(systemd: &SystemdUser, places: &Places, args: &[&str]) -> anyhow::Result<()> {
    let (systemd, places) = (systemd, places);
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
        ["remove", name] => {
            let dir = kept::remove(systemd, places, name)?;
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
    let kept = kept::keep(systemd, places, &text, name, || kept::boot(systemd, terminal))?;
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

#[cfg(not(target_os = "linux"))]
fn promote(_: &SystemdUser, _: &Places, port: u16, _: Option<&str>, _: Option<&str>) -> anyhow::Result<ExitCode> {
    anyhow::bail!("keep {port} reads how a program was started from Linux's /proc, and this is not Linux")
}

/// `keep <port>`: show what would be kept, ask, stop the one running, keep
/// its card, and check the kept one answers; else start the one before
/// again as it was (src/promote.rs decides, src/host.rs acts).
#[cfg(target_os = "linux")]
fn promote(
    systemd: &SystemdUser,
    places: &Places,
    port: u16,
    lend: Option<&str>,
    name: Option<&str>,
) -> anyhow::Result<ExitCode> {
    use svrngs_keeper::{host, promote as p};
    let surveyed = host::survey(port).and_then(|(addr, procs)| match p::declines(&procs, port, host::ephemeral()) {
        Some(declined) => Err(declined),
        None => Ok((addr, procs)),
    });
    let (addr, procs) = match surveyed {
        Ok(found) => found,
        Err(declined) => {
            tracing::debug!(?declined, "declined");
            println!("svrngs-keeper can't keep port {port}: {declined}. Nothing was changed.");
            return Ok(ExitCode::from(1));
        }
    };
    let (listener, top) = (&procs[0], &procs[procs.len() - 1]);
    let home = std::env::var("HOME").unwrap_or_default();
    let dial = p::dial(&addr);
    // svrngs names it by its lend; else its program does.
    let name = name.or(lend).map_or_else(|| p::name_for(&top.argv), str::to_string);
    let props = if p::known(&procs) {
        host::get(&dial, port, "/props").and_then(|(status, body)| match status {
            200 => p::props(&body),
            s => Err(format!("/props answered {s}")),
        })
    } else {
        Err("svrngs-keeper ships no card for it".into())
    };
    let program = top.argv.first().map_or("", |a| a.rsplit('/').next().unwrap_or(a));
    let (card, http, what) = match &props {
        Ok(props) => {
            let model = p::model(listener, props, &home);
            let what = format!(
                "llama-server with your model {model} on port {port}, context {}",
                props.n_ctx
            );
            (p::llama_card(&name, listener, &addr, port, props, &home), true, what)
        }
        Err(why) => {
            tracing::debug!(why, "copied, not composed");
            let http = host::get(&dial, port, "/").is_ok();
            let what = format!(
                "{program} as it was started, on port {port}:\n  {}\n  in {}",
                p::shown(&top.argv),
                p::homed(&top.cwd, &home)
            );
            (p::copy_card(&name, top, port, http, &home), http, what)
        }
    };
    // Read before anything stops: a card keep would refuse is refused now.
    svrngs_keeper::card::Card::parse(&card)?;
    let named = svrngs_keeper::runtime::Name::new(&name)?;
    let unit = svrngs_keeper::unit::unit_name(&named);
    println!("svrngs-keeper will run {name}: {what}.");
    if props.is_ok() {
        let flags = p::not_carried(&listener.argv);
        if !flags.is_empty() {
            let them = if flags.len() == 1 { "it" } else { "them" };
            println!(
                "Yours was also started with {}: this one starts without {them}.",
                flags.join(", ")
            );
        }
    }
    let manager = systemd.environment().unwrap_or_else(|why| {
        tracing::debug!(why, "the manager's environment was not read, so every setting is named");
        String::new()
    });
    let added = p::added(top.env.as_deref().unwrap_or_default(), &manager);
    if !added.is_empty() {
        println!(
            "It starts with none of the settings your terminal added: {}.",
            added.join(", ")
        );
    }
    let boot = kept::boot(systemd, terminal);
    println!(
        "{} and starts again if it stops (4 times a minute at most).",
        boot.starts()
    );
    if !ask(&format!(
        "The {program} you started stops, and this one takes its place. Keep it running?"
    )) {
        println!("Nothing was changed: yours runs as it was.");
        return Ok(ExitCode::from(1));
    }
    if let Err(why) = host::stop(&procs, port) {
        println!("svrngs-keeper could not stop yours ({why}), so nothing was kept.");
        return Ok(ExitCode::from(1));
    }
    let kept = kept::keep(systemd, places, &card, Some(&name), || boot);
    let (mut up, mut gave_up) = (false, false);
    if kept.is_ok() {
        host::within(30, || {
            up = host::held_by(port, &unit, &listener.exe, &dial, http);
            // A unit that hit its start limit will not start again by itself.
            gave_up = !up
                && systemd
                    .show(&unit, &["ActiveState"])
                    .is_ok_and(|s| s.contains("=failed"));
            up || gave_up
        });
    }
    if up {
        println!("{name} runs here now, and it answers on port {port}: the systemd unit {unit}.");
        return Ok(approve(lend, port));
    }
    let why = match &kept {
        Err(why) => format!("it could not be kept ({why})"),
        Ok(_) if gave_up => format!("{program} under {unit} kept stopping, so systemd stopped starting it"),
        Ok(_) => format!("{program} under {unit} did not answer on port {port} within 30 s"),
    };
    let why = format!("{why} (journalctl --user -u {unit} says what it said)");
    tracing::debug!(why, "rolled back");
    let _ = kept::remove(systemd, places, &name);
    // A failed unit stays listed as failed until it is reset.
    let _ = systemd.ctl(&["--user", "reset-failed", &unit]);
    let dir = places.card_dir(&named);
    let log = dir.join("original.log");
    let _ = std::fs::create_dir_all(&dir);
    let again = host::start_again(top, &log).map(|pid| (pid, host::within(30, || host::answers(&dial, port, http))));
    match again {
        Ok((pid, true)) => println!(
            "The kept copy failed: {why}. So yours is started again as it was, with your terminal's settings, \
             and it answers on port {port} (pid {pid}, its output in {}). Its card is in {}.",
            log.display(),
            dir.display()
        ),
        Ok((pid, false)) => println!(
            "The kept copy failed: {why}. Yours is started again as it was (pid {pid}), but it does not answer \
             on port {port} yet; its output is in {}.",
            log.display()
        ),
        Err(e) => println!("The kept copy failed: {why}. And yours could not be started again: {e}."),
    }
    Ok(ExitCode::from(1))
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
