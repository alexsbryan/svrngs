//! Turn a command the person gave into the same native card `keep` installs.

use std::{num::NonZeroU16, path::Path};

use anyhow::{Context, bail};
use serde::Serialize;
use svrngs_keeper::{card::Card, host, kept, runtime::Name, systemd::SystemdUser, unit};

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome<'a> {
    Verified {
        unit: &'a str,
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
    let command = options
        .next()
        .filter(|a| !a.is_empty())
        .context("give a command after --")?;
    let (mut port, mut folder, mut json) = (None, None, false);
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
            ["--json", rest @ ..] if !json => {
                json = true;
                rest
            }
            [flag, ..] => bail!("unexpected Run option: {flag}"),
            [] => unreachable!(),
        };
    }
    let port = port.context("give the command's listening port with --port")?.get();
    let say = |outcome| {
        if json {
            return super::json(&outcome);
        }
        match outcome {
            Outcome::Verified { .. } => println!("{} is running here on port {port}.", name.as_str()),
            Outcome::Unverified { state, ready, .. } => {
                let state = state.as_deref().unwrap_or("unknown");
                bail!(
                    "Couldn't verify {} is answering (state: {state}; port {port} ready: {ready}).",
                    name.as_str()
                )
            }
            Outcome::Busy { why } => bail!("{why}"),
        }
        Ok(())
    };
    let busy = || -> anyhow::Result<_> {
        Ok(host::busy(port).map_err(anyhow::Error::msg)?.map(|by| Outcome::Busy {
            why: format!("Port {port} is in use by {by}, the copy you started: stop it, then press Run."),
        }))
    };
    if let Some(outcome) = busy()? {
        return say(outcome);
    }
    let folder = std::path::absolute(folder.unwrap_or(Path::new(".")))?;
    let folder = folder.to_str().context("the folder's path is not UTF-8")?;
    let text = serde_yaml::to_string(&serde_json::json!({"x-svrngs": {
        "title": name.as_str(), "about": "Run with svrngs", "door": {"port": port, "protocol": "http"},
        "state": [], "native": {"command": command, "folder": folder}
    }}))?;
    let card = Card::parse(&text)?;
    let (systemd, places) = (SystemdUser::system(), super::places()?);
    let _locks = (
        super::lock(&places, &format!("port-{port}"))?,
        super::lock(&places, &format!("name-{}", name.as_str()))?,
    );
    if let Some(outcome) = busy()? {
        return say(outcome);
    }
    let unit = unit::unit_name(&name);
    let loaded = systemd.show(&unit, &["LoadState"]).map_err(anyhow::Error::msg)?;
    if places.unit_path(&name).exists() || loaded.trim() != "LoadState=not-found" {
        tracing::debug!(%unit, state = loaded.trim(), "run: existing unit left alone");
        bail!(
            "{} already has a setup here. Start it again, or remove its setup before running a new command.",
            name.as_str()
        );
    }
    let kept = kept::keep_card(&systemd, &places, &text, &card, name.clone(), || {
        kept::boot(&systemd, super::terminal)
    })?;
    let llama = Path::new(command[0]).file_name().is_some_and(|p| p == "llama-server");
    let ready = host::wait_for_answer(port, llama, std::time::Duration::from_secs(30));
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
    if owned {
        say(Outcome::Verified { unit: &kept.unit })
    } else {
        say(Outcome::Unverified {
            state: state.map(str::to_owned),
            ready,
            port,
        })
    }
}
