//! Install and verify an explicit Run, from a command or a container card.

use std::{num::NonZeroU16, path::Path, time::Duration};

use anyhow::{Context, bail};
use serde::Serialize;
use svrngs_keeper::{card::Card, host, kept::Places, runtime::Name};
#[cfg(target_os = "linux")]
use svrngs_keeper::{kept, systemd::SystemdUser, unit};

#[cfg(target_os = "linux")]
use super::container_run;

enum Source<'a> {
    Command(&'a [&'a str], Option<&'a Path>),
    Card(#[cfg_attr(target_os = "macos", allow(dead_code))] &'a Path),
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome<'a> {
    Verified {
        unit: &'a str,
        port: u16,
    },
    Unverified {
        state: Option<String>,
        ready: bool,
        port: u16,
    },
    Busy {
        why: String,
    },
}

pub fn run(name: &str, args: &[&str]) -> anyhow::Result<()> {
    let name = Name::new(name)?;
    let mut options = args.splitn(2, |w| *w == "--");
    let mut flags = options.next().unwrap_or_default();
    let argv = options.next().filter(|a| !a.is_empty());
    let (mut port, mut folder, mut card_file, mut json) = (None, None, None, false);
    let mut quoted = None;
    while !flags.is_empty() {
        flags = match flags {
            ["--port", value, rest @ ..] if port.is_none() => {
                port = Some(value.parse::<NonZeroU16>().context("give a port from 1 to 65535")?);
                rest
            }
            ["--folder", value, rest @ ..] if folder.is_none() => {
                folder = Some(Path::new(value));
                rest
            }
            ["--card", value, rest @ ..] if card_file.is_none() => {
                card_file = Some(Path::new(value));
                rest
            }
            ["--command", value, rest @ ..] if quoted.is_none() && argv.is_none() => {
                quoted = Some(
                    shlex::split(value)
                        .filter(|a| !a.is_empty())
                        .context("give a command with complete quotes")?,
                );
                rest
            }
            ["--json", rest @ ..] if !json => {
                json = true;
                rest
            }
            [flag, ..] => bail!("unexpected Run option: {flag}"),
            [] => unreachable!(),
        };
    }
    let words = quoted
        .as_ref()
        .map(|args| args.iter().map(String::as_str).collect::<Vec<_>>());
    let source = match (argv.or(words.as_deref()), card_file, folder) {
        (Some(command), None, folder) => Source::Command(command, folder),
        (None, Some(file), None) => Source::Card(file),
        _ => bail!("give a command after --, or a compose.yaml with --card"),
    };
    let port = port.context("give the listening port with --port")?.get();
    let say = |outcome| {
        if json {
            return super::json(&outcome);
        }
        match outcome {
            Outcome::Verified { .. } => println!("{} is running here on port {port}.", name.as_str()),
            Outcome::Unverified { state, ready, .. } => {
                let state = state.as_deref().unwrap_or("unknown");
                bail!(
                    "{} was saved and can start again, but wasn't verified on port {port} (state: {state}; ready: {ready}). To turn it off, run svrngs-keeper stop {}.",
                    name.as_str(),
                    name.as_str()
                )
            }
            Outcome::Busy { why } => bail!("{why}"),
        }
        Ok(())
    };
    let busy = || -> anyhow::Result<_> {
        Ok(host::busy(port).map_err(anyhow::Error::msg)?.map(|by| Outcome::Busy {
            why: format!("Port {port} is in use by {by}, the copy you started: stop it, then press Start."),
        }))
    };
    if let Some(outcome) = busy()? {
        return say(outcome);
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
        Source::Command(command, folder) => {
            let folder = std::path::absolute(folder.unwrap_or(Path::new(".")))?;
            let folder = folder.to_str().context("the folder's path is not UTF-8")?;
            let text = serde_yaml::to_string(&serde_json::json!({"x-svrngs": {
                "title": name.as_str(), "about": "Run with svrngs", "door": {"port": port, "protocol": "http"},
                "state": [], "native": {"command": command, "folder": folder}
            }}))?;
            let llama = Path::new(command[0]).file_name().is_some_and(|p| p == "llama-server");
            (Card::parse(&text)?, llama)
        }
    };
    let places = super::places()?;
    let _locks = (
        super::lock(&places, &format!("port-{port}"))?,
        super::lock(&places, &format!("name-{}", name.as_str()))?,
    );
    if let Some(outcome) = busy()? {
        return say(outcome);
    }
    let (unit, ready, owned, state) = install(&places, &card, name.clone(), llama)?;
    if owned {
        say(Outcome::Verified { unit: &unit, port })
    } else {
        say(Outcome::Unverified { state, ready, port })
    }
}

#[cfg(target_os = "linux")]
fn install(
    places: &Places,
    card: &Card,
    name: Name,
    llama: bool,
) -> anyhow::Result<(String, bool, bool, Option<String>)> {
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
        kept::keep_card(&systemd, places, card, name.clone(), || {
            kept::boot(&systemd, super::terminal)
        })?
    } else {
        container_run::install(&systemd, places, card, name.clone())?
    };
    let ready =
        host::wait_for_answer(port, llama, Duration::from_secs(30)) && (native || container_run::healthy(card, &name));
    let shown = systemd.show(&kept.unit, &["ActiveState", "ControlGroup"]);
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
    Ok((kept.unit, ready, owned, state.map(str::to_owned)))
}

#[cfg(target_os = "macos")]
fn install(
    places: &Places,
    card: &Card,
    name: Name,
    llama: bool,
) -> anyhow::Result<(String, bool, bool, Option<String>)> {
    use svrngs_keeper::{kept::Running, launchd};
    places.install_agent(card, &name)?;
    let port = card.door.port;
    let ready = host::wait_for_answer(port, llama, Duration::from_secs(30));
    let shown = places.agent(&name)?.observe();
    let state = shown.as_ref().ok().map(|(status, _)| {
        match status.running {
            Running::Since(_) => "running",
            Running::Starting => "starting",
            Running::Stopped | Running::Failed(_) => "stopped",
        }
        .to_owned()
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
        ?witness,
        ready,
        owned,
        "run: installed agent and port observed"
    );
    Ok((launchd::label(&name), ready, owned, state))
}
