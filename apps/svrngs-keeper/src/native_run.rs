//! Install and verify an explicit Run, from a command or a container card.
//! A command comes as the page's line (`--command`), or as the words after
//! `--` that a terminal's shell split (`svrngs keep`), which are shown, and
//! kept only once the person presses Enter there. Both are checked by [`given`]. Only what
//! is verified is kept: started under the service manager and seen as that
//! job answering on the lend's port. Anything else is stopped and removed,
//! its card too, and why is said with what it last printed (svrn-docs
//! rings/current/KEEPER_ENTRY_POINT.md F1).

use std::{
    num::NonZeroU16,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, bail};
use serde::Serialize;
use svrngs_keeper::{card::Card, given, host, kept::Places, runtime::Name};
#[cfg(target_os = "linux")]
use svrngs_keeper::{
    kept::{self, Boot},
    systemd::SystemdUser,
    unit,
};

#[cfg(target_os = "linux")]
use super::container_run;

enum Source<'a> {
    Command(given::Given),
    Card(#[cfg_attr(target_os = "macos", allow(dead_code))] &'a Path),
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome {
    /// Kept: verified, and when it comes back.
    Verified {
        unit: String,
        port: u16,
        back: &'static str,
    },
    /// Nothing kept, and why, in the person's words.
    Refused { why: String },
}

/// What `run` was asked, besides the words after `--`.
#[derive(Default)]
struct Asked<'a> {
    port: Option<u16>,
    folder: Option<&'a str>,
    card: Option<&'a Path>,
    line: Option<&'a str>,
    /// The group folders this computer syncs, which nothing kept may be in.
    synced: Vec<PathBuf>,
    json: bool,
    /// The terminal route, the words after `--`: what will run is shown,
    /// and Enter asked for, first.
    confirm: bool,
    /// The page's yes to starting before anyone logs in.
    at_boot: bool,
}

pub fn run(name: &str, args: &[&str]) -> anyhow::Result<()> {
    let name = Name::new(name)?;
    let mut options = args.splitn(2, |w| *w == "--");
    let mut flags = options.next().unwrap_or_default();
    let argv = options.next().filter(|a| !a.is_empty());
    let mut asked = Asked::default();
    while !flags.is_empty() {
        flags = match flags {
            ["--port", value, rest @ ..] if asked.port.is_none() => {
                let port = value.parse::<NonZeroU16>().context("give a port from 1 to 65535")?;
                asked.port = Some(port.get());
                rest
            }
            ["--folder", value, rest @ ..] if asked.folder.is_none() => {
                asked.folder = Some(value);
                rest
            }
            ["--card", value, rest @ ..] if asked.card.is_none() => {
                asked.card = Some(Path::new(value));
                rest
            }
            ["--command", value, rest @ ..] if asked.line.is_none() && argv.is_none() => {
                asked.line = Some(value);
                rest
            }
            ["--synced", value, rest @ ..] => {
                asked.synced.push(PathBuf::from(value));
                rest
            }
            ["--json", rest @ ..] if !asked.json => {
                asked.json = true;
                rest
            }
            ["--at-boot", rest @ ..] if !asked.at_boot => {
                asked.at_boot = true;
                rest
            }
            [flag, ..] => bail!("unexpected Run option: {flag}"),
            [] => unreachable!(),
        };
    }
    asked.confirm = argv.is_some();
    let port = asked.port.context("give the listening port with --port")?;
    let outcome = keep(&name, port, argv, &asked).unwrap_or_else(|why| {
        tracing::debug!(%why, "run: refused, nothing kept");
        Outcome::Refused {
            why: format!("{why:#}"),
        }
    });
    if asked.json {
        return super::json(&outcome);
    }
    match outcome {
        Outcome::Verified { back, .. } => {
            println!(
                "{} is running here on port {port}. It comes back {back}.",
                name.as_str()
            )
        }
        Outcome::Refused { why } => bail!("{why}"),
    }
    Ok(())
}

fn keep(name: &Name, port: u16, argv: Option<&[&str]>, asked: &Asked) -> anyhow::Result<Outcome> {
    let source = match (argv, asked.line, asked.card) {
        (Some(_), None, None) | (None, Some(_), None) => Source::Command(given(port, argv, asked)?),
        (None, None, Some(file)) if asked.folder.is_none() => Source::Card(file),
        _ => bail!("give a command after --, or a compose.yaml with --card"),
    };
    let busy = || -> anyhow::Result<_> {
        let then = if asked.confirm { "run this again" } else { "press Start" };
        Ok(host::busy(port)
            .map_err(anyhow::Error::msg)?
            .map(|by| Outcome::Refused {
                why: format!("Port {port} is in use by {by}, the copy you started: stop it, then {then}."),
            }))
    };
    if let Some(outcome) = busy()? {
        return Ok(outcome);
    }
    let (card, llama) = match source {
        #[cfg(target_os = "macos")]
        Source::Card(_) => bail!("On this Mac, Run needs a native command, not a container card."),
        #[cfg(target_os = "linux")]
        Source::Card(file) => {
            let card = container_run::card(file)?;
            if port != card.door.port {
                tracing::debug!(port, card_port = card.door.port, "run: port differs from the card");
                bail!(
                    "The card listens on port {}; give that port with --port.",
                    card.door.port
                );
            }
            (card, false)
        }
        Source::Command(given) => {
            if asked.confirm {
                confirm(name, port, &given)?;
            }
            let program = given.program.to_str().context("the program's path is not UTF-8")?;
            let folder = given.folder.to_str().context("the folder's path is not UTF-8")?;
            let command: Vec<&str> = std::iter::once(program)
                .chain(given.args.iter().map(String::as_str))
                .collect();
            let mut native = serde_json::json!({"command": command, "folder": folder});
            // Started under the word it was given as, as a shell starts it.
            if given.argv0 != program {
                native["argv0"] = given.argv0.clone().into();
            }
            let text = serde_yaml::to_string(&serde_json::json!({"x-svrngs": {
                "title": name.as_str(), "about": "Run with svrngs", "door": {"port": port, "protocol": "http"},
                "state": [], "native": native
            }}))?;
            let llama = [Path::new(&given.argv0), &given.program]
                .iter()
                .any(|p| p.file_name().is_some_and(|p| p == "llama-server"));
            (Card::parse(&text)?, llama)
        }
    };
    let places = super::places()?;
    let _locks = (
        super::lock(&places, &format!("port-{port}"))?,
        super::lock(&places, &format!("name-{}", name.as_str()))?,
    );
    if let Some(outcome) = busy()? {
        return Ok(outcome);
    }
    install(&places, &card, name.clone(), llama, asked)
}

/// The program, its words and its folder, from the terminal's words (the
/// folder this was started in unless `--folder` names one) or the page's
/// line, checked to keep ([`given::check`]).
fn given(port: u16, argv: Option<&[&str]>, asked: &Asked) -> anyhow::Result<given::Given> {
    let (words, folder) = match (argv, asked.line) {
        (Some(argv), _) => (
            argv.iter().map(|w| w.to_string()).collect(),
            std::path::absolute(asked.folder.unwrap_or("."))?,
        ),
        (None, line) => {
            let home = std::env::var_os("HOME").context("HOME is not set, so ~ means no folder")?;
            given::from_line(
                line.unwrap_or_default(),
                asked.folder.unwrap_or_default(),
                Path::new(&home),
                port,
            )?
        }
    };
    let path = svrngs_keeper::systemd::captured_path();
    Ok(given::check(
        words,
        &folder,
        &asked.synced,
        &path,
        &given::Account::read()?,
    )?)
}

/// Show exactly what will run, and where, and wait for Enter: a line the
/// shell gave by mistake (an Up arrow too many, a `!!`) never runs unseen.
/// With no terminal to answer, nothing is kept.
fn confirm(name: &Name, port: u16, given: &given::Given) -> anyhow::Result<()> {
    use std::io::{BufRead, IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        tracing::debug!("run: refused, no terminal to confirm in");
        bail!(
            "svrngs keep asks you to press Enter in a terminal before it keeps anything, and there is no terminal here, so nothing was kept. Run it in a terminal, or use Keep it running on your home page."
        );
    }
    eprint!(
        "\nsvrngs will keep this running as {}, answering on port {port}, and start it again after a restart:\n\n    {}\n    in {}\n\nPress Enter to keep it, or Ctrl+C to leave everything as it is. ",
        name.as_str(),
        given.shown(),
        given.folder.display()
    );
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    let read = std::io::stdin().lock().read_line(&mut line);
    let yes = matches!(read, Ok(n) if n > 0) && line.trim().is_empty();
    tracing::debug!(?read, yes, "run: confirm answered");
    if !yes {
        bail!("Nothing was kept.");
    }
    Ok(())
}

/// What it last printed: its last five lines, each cut short and shown as
/// plain text, or none.
fn tail(text: &str) -> Option<String> {
    let lines: Vec<String> = text
        .lines()
        .map(|l| l.chars().filter(|c| !c.is_control()).take(300).collect::<String>())
        .filter(|l| !l.trim().is_empty() && !l.starts_with("-- "))
        .collect();
    let last = lines[lines.len().saturating_sub(5)..].join("\n");
    (!last.is_empty()).then_some(last)
}

/// Why nothing was kept, in the person's words.
fn unverified(port: u16, ready: bool, ended: Option<String>, said: Option<String>) -> String {
    let what = if ready {
        format!("Something answered on port {port}, but not the copy svrngs started")
    } else {
        format!("It didn't answer on port {port} within 30 seconds")
    };
    let ended = ended.map_or_else(String::new, |e| format!(", and {e}"));
    let said = said.map_or_else(
        || "It printed nothing svrngs could read.".into(),
        |w| format!("Its last words:\n{w}"),
    );
    format!("{what}{ended}, so nothing was kept. Check the command and folder, then start your own copy again. {said}")
}

#[cfg(target_os = "linux")]
fn install(places: &Places, card: &Card, name: Name, llama: bool, asked: &Asked) -> anyhow::Result<Outcome> {
    let systemd = SystemdUser::system();
    let port = card.door.port;
    let unit = unit::unit_name(&name);
    let loaded = systemd.show(&unit, &["LoadState"]).map_err(anyhow::Error::msg)?;
    if places.unit_path(&name).exists() || loaded.trim() != "LoadState=not-found" {
        tracing::debug!(%unit, state = loaded.trim(), "run: existing unit left alone");
        bail!(
            "{} already has a setup here. Start it again, or remove its setup before running a new command.",
            name.as_str()
        );
    }
    let native = card.native.is_some();
    let kept = if native {
        // Linger changes the whole account, so it is asked (or the page's
        // yes taken) now and turned on only once the program is verified.
        let boot = kept::boot(&systemd, |q| match (asked.at_boot, asked.confirm) {
            (true, _) => Some(true),
            (false, true) => super::terminal(q),
            (false, false) => None,
        });
        let kept = kept::keep_card(&systemd, places, card, name.clone(), || Boot::Login { unread: None })?;
        (kept, boot)
    } else {
        let kept = container_run::install(&systemd, places, card, name.clone())?;
        (kept, Boot::Login { unread: None })
    };
    let (kept, boot) = kept;
    let ready =
        host::wait_for_answer(port, llama, Duration::from_secs(30)) && (native || container_run::healthy(card, &name));
    let shown = systemd.show(
        &kept.unit,
        &["ActiveState", "ControlGroup", "ExecMainCode", "ExecMainStatus"],
    );
    let property = |key| shown.as_ref().ok()?.lines().find_map(|line| line.strip_prefix(key));
    let state = property("ActiveState=");
    let group = property("ControlGroup=")
        .map(Path::new)
        .filter(|g| g.is_absolute() && *g != Path::new("/"));
    // The same kernel socket cgroup evidence Share reads, without any approval.
    let owned = ready
        && state == Some("active")
        && group.is_some_and(|group| {
            host::ss(port).is_ok_and(|listeners| {
                !listeners.is_empty()
                    && listeners.iter().all(|socket| {
                        socket
                            .cgroup
                            .as_deref()
                            .is_some_and(|path| Path::new(path).starts_with(group))
                    })
            })
        });
    tracing::debug!(%unit, ?shown, ready, owned, "run: installed unit and port observed");
    if owned {
        if boot == Boot::TurnOn {
            systemd.ensure_linger("svrngs-keeper", "programs it keeps");
        }
        // Read again: a yes logind refused starts it at login all the same.
        let back = Boot::read(systemd.lingers()).back();
        return Ok(Outcome::Verified {
            unit: kept.unit,
            port,
            back,
        });
    }
    let ended = match (property("ExecMainCode="), property("ExecMainStatus=")) {
        (Some("1"), Some(status)) => Some(format!("it stopped with exit status {status}")),
        (Some("2" | "3"), Some(signal)) => Some(format!("it was stopped by signal {signal}")),
        _ => None,
    };
    let journal = std::process::Command::new("journalctl")
        .args([
            "--user",
            "--unit",
            &kept.unit,
            "--lines",
            "5",
            "--output",
            "cat",
            "--no-pager",
        ])
        .output();
    let said = journal
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| tail(&String::from_utf8_lossy(&out.stdout)));
    systemd
        .uninstall(&kept.unit, &places.unit_path(&name))
        .map_err(anyhow::Error::msg)?;
    places.forget(&name, &kept.stored)?;
    tracing::debug!(%unit, ?ended, "run: not verified, so its unit and card were removed");
    Ok(Outcome::Refused {
        why: unverified(port, ready, ended, said),
    })
}

#[cfg(target_os = "macos")]
fn install(places: &Places, card: &Card, name: Name, llama: bool, _: &Asked) -> anyhow::Result<Outcome> {
    use svrngs_keeper::{kept::Running, launchd};
    let stored = places.install_agent(card, &name)?;
    let port = card.door.port;
    let ready = host::wait_for_answer(port, llama, Duration::from_secs(30));
    let agent = places.agent(&name)?;
    let shown = agent.observe();
    let state = shown.as_ref().ok().map(|(status, _)| match status.running {
        Running::Since(_) => "running",
        Running::Starting => "starting",
        Running::Stopped | Running::Failed(_) => "stopped",
    });
    let witness = shown
        .as_ref()
        .ok()
        .and_then(|(_, pid)| *pid)
        .filter(|_| ready)
        .map(|pid| host::owns(port, pid));
    let owned = matches!(witness, Some(Ok(true)));
    tracing::debug!(
        port,
        ?shown,
        ?state,
        ?witness,
        ready,
        owned,
        "run: installed agent and port observed"
    );
    if owned {
        return Ok(Outcome::Verified {
            unit: launchd::label(&name),
            port,
            back: "when you log in",
        });
    }
    let ended = agent.last_exit().map(|code| format!("its last exit code was {code}"));
    let said = std::fs::read_to_string(agent.log()).ok().and_then(|t| tail(&t));
    agent.remove().map_err(anyhow::Error::msg)?;
    places.forget(&name, &stored)?;
    tracing::debug!(port, ?ended, "run: not verified, so its agent and card were removed");
    Ok(Outcome::Refused {
        why: unverified(port, ready, ended, said),
    })
}
