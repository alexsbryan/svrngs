//! One local, volume-only HTTP service. Systemd owns the restart limit;
//! its main process waits on an exit, not on a health observation.

use std::path::Path;

use anyhow::{Context, bail, ensure};
use svrngs_keeper::{
    card::{Card, Protocol},
    kept::{self, Kept, Places},
    machine,
    podman::Podman,
    runtime::{App, Health, Name},
    systemd::SystemdUser,
    unit,
};

pub fn card(path: &Path) -> anyhow::Result<Card> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let card = Card::parse(&text)?;
    if card.door.protocol != Protocol::Http
        || card.services.len() != 1
        || !card.asks.is_empty()
        || card.services[0].mounts.iter().any(|m| m.volume.is_none())
    {
        tracing::debug!(title = card.title, "run: refused unsupported container card");
        bail!("Run takes a card with one HTTP container, only named volumes, and no additional agreements.");
    }
    Ok(card)
}

pub fn install(systemd: &SystemdUser, places: &Places, card: &Card, name: Name) -> anyhow::Result<Kept> {
    let program = std::env::current_exe()?;
    let path = places.card_dir(&name).join("compose.yaml");
    fn unit_word(path: &Path) -> anyhow::Result<&str> {
        path.to_str()
            .filter(|s| path.is_absolute() && !s.chars().any(char::is_control))
            .context("the service needs an absolute UTF-8 path without control characters")
    }
    let owner = std::fs::read_to_string("/proc/sys/kernel/random/uuid")?;
    let content = unit::container(&name, card, unit_word(&program)?, unit_word(&path)?, owner.trim());
    Ok(kept::install(
        systemd,
        places,
        card,
        name,
        || kept::boot(systemd, super::terminal),
        &content,
    )?)
}

pub fn healthy(card: &Card, name: &Name) -> bool {
    let seen = App::new(&Podman::local(), card, name.clone()).health();
    tracing::debug!(?seen, "run: container readiness (never restarts the unit)");
    matches!(seen, Ok(Health::Healthy))
}

pub fn serve(args: &[&str]) -> anyhow::Result<()> {
    let [name, path, owner, revision, mode @ ..] = args else {
        bail!("serve-card is called by a kept card's unit");
    };
    let stopping = match mode {
        [] => false,
        ["--stop"] => true,
        // Older generated units still pass $MAINPID when it exists.
        ["--stop", main] if main.parse::<std::num::NonZeroU32>().is_ok() => true,
        _ => bail!("unexpected serve-card arguments"),
    };
    let name = Name::new(name)?;
    let path = Path::new(path);
    let card = card(path)?;
    if !stopping && card.version != *revision {
        tracing::debug!(%revision, read = card.version, "serve-card: file differs from the installed card");
        bail!("The stored card changed. Remove its setup before running a changed card.");
    }
    let podman = Podman::local();
    ensure!(
        podman.run(&["info", "--format", "{{.Host.Security.Rootless}}"])? == "true",
        "The service needs rootless Podman."
    );
    let app = App::new(&podman, &card, name);
    // Stop and launch cannot interleave, and the lock is not held while waiting.
    let lock = std::fs::File::create(path.with_extension("lock"))?;
    lock.lock()?;
    app.stop_owned(owner)?;
    if stopping {
        return Ok(());
    }
    machine::ensure(&podman, &card)?;
    app.pull()?;
    let containers = app.start_owned(card.door.port, owner)?;
    drop(lock);
    let [id] = &containers[..] else {
        unreachable!("one service in the parsed card");
    };
    tracing::debug!(%id, "serve-card: waiting for the detached container to exit");
    let exit = podman.run(&["wait", "--condition=exited", id])?;
    bail!("The container exited ({exit}).");
}
