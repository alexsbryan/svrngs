//! On a Mac, podman runs its containers in a small Linux machine of its own.
//! Give makes it when there is none and starts it when it is stopped, sized
//! for what it keeps, so nobody opens a terminal (GIVE.md §3, "The one
//! install"). On Linux podman needs no machine.
//!
//! The decision is [`plan`], pure over what `podman machine list` says; the
//! commands that carry it out are [`ensure`].
//!
//! A card whose image runs an amd64 build, as Factorio's does since its arm64
//! build segfaulted making the map (cards/factorio/README.md), runs on a Mac
//! with Apple silicon under Rosetta in the machine, which podman 5.6 and
//! later leave off. The machine give makes has it on; on one give did not
//! make, with it off, such a card is refused before it starts, by
//! [`needs_rosetta`] and the machine's own `Rosetta`, so it never crashes
//! there instead (give-install (c)).

use crate::card::Card;
use crate::podman::{Podman, PodmanError};

/// What `podman machine init` reads after the person's own containers.conf
/// (containers.conf(5), `CONTAINERS_CONF_OVERRIDE`), so the machine give makes
/// has Rosetta on. Podman keeps it with the machine
/// (`AppleHypervisor.Vfkit.Rosetta` in its JSON, `Rosetta` in `podman machine
/// inspect`, podman 6.0.1 here).
pub const ROSETTA_ON: &str = "[machine]\nrosetta = true\n";

/// The memory `podman machine init` gives a machine when not told
/// (podman-machine-init(1), `--memory`: 2048 MiB). Give asks for this, for
/// the machine's own life, plus what the cards it keeps declare.
pub const PODMANS_OWN: u64 = 2048 << 20;

/// One row of `podman machine list --format json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machine {
    pub name: String,
    pub default: bool,
    pub running: bool,
    /// Bytes.
    pub memory: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// It runs, and has room.
    Ready,
    /// None here: make one with this much memory, then start it.
    Init { memory: u64 },
    /// Stopped, with room: start it.
    Start { name: String },
    /// Stopped, too small: give it this much, then start it.
    Grow { name: String, memory: u64 },
    /// Running and too small. Growing it would stop what runs in it now,
    /// which is the person's to decide, so give says so and waits.
    TooSmall { has: u64, needs: u64 },
}

#[derive(Debug, thiserror::Error)]
pub enum MachineError {
    #[error(transparent)]
    Podman(#[from] PodmanError),
    #[error("podman's list of machines was not what give reads: {0}")]
    Unreadable(String),
    #[error(
        "This computer gives Podman {} GB, and keeping this needs {} GB. Quit Podman Desktop, or anything you run in Podman, and try again: give will make room.",
        gb(*has),
        gb(*needs)
    )]
    TooSmall { has: u64, needs: u64 },
    #[error(
        "This runs on this computer through Rosetta, which Podman has turned off here. Turn on Rosetta in Podman Desktop’s settings, then start Podman again."
    )]
    NoRosetta,
    #[error("give could not write {0}, which podman reads to make its machine: {1}")]
    Conf(String, String),
}

/// Whether `card` needs Rosetta here: on a Mac with Apple silicon (`os` and
/// `arch` as `std::env::consts` names them), a service that names the amd64
/// build. Elsewhere podman runs that build natively, or not at all, and
/// needs no machine.
pub fn needs_rosetta(card: &Card, os: &str, arch: &str) -> bool {
    let amd64 = card
        .services
        .iter()
        .any(|s| s.platform.as_deref() == Some("linux/amd64"));
    os == "macos" && arch == "aarch64" && amd64
}

/// Whether the machine podman uses has Rosetta off, as `podman machine
/// inspect` says it: never, where there is no machine, since the one give
/// makes has it on.
pub fn rosetta_off(podman: &Podman) -> Result<bool, MachineError> {
    let machines = read(&podman.run(&["machine", "list", "--format", "json"])?)?;
    chosen(&machines).map_or(Ok(false), |m| off_in(podman, &m.name))
}

/// Whether the machine `name` has Rosetta off.
fn off_in(podman: &Podman, name: &str) -> Result<bool, MachineError> {
    let inspected: serde_json::Value = serde_json::from_str(&podman.run(&["machine", "inspect", name])?)
        .map_err(|e| MachineError::Unreadable(format!("podman machine inspect: {e}")))?;
    let on = inspected[0]["Rosetta"].as_bool();
    tracing::debug!(machine = name, rosetta = ?on, "podman's machine: Rosetta");
    on.map(|on| !on)
        .ok_or_else(|| MachineError::Unreadable(format!("{name} says nothing of Rosetta")))
}

/// The machine podman uses: the default one, else the first.
fn chosen(machines: &[Machine]) -> Option<&Machine> {
    machines.iter().find(|m| m.default).or(machines.first())
}

fn gb(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / (1u64 << 30) as f64)
}

/// What to do, given the machines here and the memory the cards declare
/// beyond podman's own. The default machine is the one podman uses.
pub fn plan(machines: &[Machine], declared: u64) -> Plan {
    let needs = PODMANS_OWN + declared;
    let Some(m) = chosen(machines) else {
        return Plan::Init { memory: needs };
    };
    match (m.running, m.memory >= needs) {
        (true, true) => Plan::Ready,
        (true, false) => Plan::TooSmall { has: m.memory, needs },
        (false, true) => Plan::Start { name: m.name.clone() },
        (false, false) => Plan::Grow {
            name: m.name.clone(),
            memory: needs,
        },
    }
}

/// Read `podman machine list --format json`.
pub fn read(json: &str) -> Result<Vec<Machine>, MachineError> {
    let rows: serde_json::Value = serde_json::from_str(json).map_err(|e| MachineError::Unreadable(e.to_string()))?;
    let rows = rows
        .as_array()
        .ok_or_else(|| MachineError::Unreadable("not a list".into()))?;
    rows.iter()
        .map(|r| {
            let memory = match &r["Memory"] {
                serde_json::Value::String(s) => s.parse().ok(),
                v => v.as_u64(),
            };
            Ok(Machine {
                name: r["Name"]
                    .as_str()
                    .ok_or_else(|| MachineError::Unreadable("a machine with no Name".into()))?
                    .to_string(),
                default: r["Default"].as_bool().unwrap_or(false),
                running: r["Running"].as_bool().unwrap_or(false),
                memory: memory.ok_or_else(|| MachineError::Unreadable("a machine with no Memory".into()))?,
            })
        })
        .collect()
}

/// Make podman's machine ready on a Mac for `card`; nothing to do elsewhere.
pub fn ensure(podman: &Podman, card: &Card) -> Result<(), MachineError> {
    if cfg!(target_os = "macos") {
        let rosetta = needs_rosetta(card, std::env::consts::OS, std::env::consts::ARCH);
        ensure_machine(podman, card.memory(), rosetta)
    } else {
        tracing::debug!("podman runs here without a machine");
        Ok(())
    }
}

/// [`ensure`]'s work, on any system: what a Mac runs, for a card that
/// declares `declared` bytes and needs Rosetta or not.
pub fn ensure_machine(podman: &Podman, declared: u64, rosetta: bool) -> Result<(), MachineError> {
    let machines = read(&podman.run(&["machine", "list", "--format", "json"])?)?;
    let plan = plan(&machines, declared);
    tracing::debug!(?machines, declared, rosetta, ?plan, "podman's machine");
    if rosetta
        && let Some(m) = chosen(&machines)
        && off_in(podman, &m.name)?
    {
        tracing::debug!(
            ?plan,
            "podman's machine: refused, the card needs Rosetta and the machine has it off"
        );
        return Err(MachineError::NoRosetta);
    }
    let mib = |bytes: u64| (bytes >> 20).to_string();
    match plan {
        Plan::Ready => {}
        Plan::Init { memory } => {
            let conf = std::env::temp_dir().join(format!("give-rosetta-{}.conf", std::process::id()));
            std::fs::write(&conf, ROSETTA_ON)
                .map_err(|e| MachineError::Conf(conf.display().to_string(), e.to_string()))?;
            let made = (podman.with("CONTAINERS_CONF_OVERRIDE", &conf)).run(&[
                "machine",
                "init",
                "--memory",
                &mib(memory),
                "--now",
            ]);
            let _ = std::fs::remove_file(&conf);
            made?;
        }
        Plan::Start { name } => {
            podman.run(&["machine", "start", &name])?;
        }
        Plan::Grow { name, memory } => {
            podman.run(&["machine", "set", "--memory", &mib(memory), &name])?;
            podman.run(&["machine", "start", &name])?;
        }
        Plan::TooSmall { has, needs } => return Err(MachineError::TooSmall { has, needs }),
    }
    Ok(())
}
