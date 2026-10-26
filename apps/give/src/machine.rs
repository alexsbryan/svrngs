//! On a Mac, podman runs its containers in a small Linux machine of its own.
//! Give makes it when there is none and starts it when it is stopped, sized
//! for what it keeps, so nobody opens a terminal (GIVE.md §3, "The one
//! install"). On Linux podman needs no machine.
//!
//! The decision is [`plan`], pure over what `podman machine list` says; the
//! commands that carry it out are [`ensure`].

use crate::podman::{Podman, PodmanError};

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
}

fn gb(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / (1u64 << 30) as f64)
}

/// What to do, given the machines here and the memory the cards declare
/// beyond podman's own. The default machine is the one podman uses.
pub fn plan(machines: &[Machine], declared: u64) -> Plan {
    let needs = PODMANS_OWN + declared;
    let Some(m) = machines.iter().find(|m| m.default).or(machines.first()) else {
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

/// Make podman's machine ready on a Mac; nothing to do elsewhere.
pub fn ensure(podman: &Podman, declared: u64) -> Result<(), MachineError> {
    if cfg!(target_os = "macos") {
        ensure_machine(podman, declared)
    } else {
        tracing::debug!("podman runs here without a machine");
        Ok(())
    }
}

/// [`ensure`]'s work, on any system: what a Mac runs.
pub fn ensure_machine(podman: &Podman, declared: u64) -> Result<(), MachineError> {
    let machines = read(&podman.run(&["machine", "list", "--format", "json"])?)?;
    let plan = plan(&machines, declared);
    tracing::debug!(?machines, declared, ?plan, "podman's machine");
    let mib = |bytes: u64| (bytes >> 20).to_string();
    match plan {
        Plan::Ready => {}
        Plan::Init { memory } => {
            podman.run(&["machine", "init", "--memory", &mib(memory), "--now"])?;
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
