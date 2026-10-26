//! `give check <card dir>`: an author's proof that a card can go on the
//! shelf (GIVE.md §4, "`svrngs card check`"). On this computer it reads the
//! card, starts the app from its pinned image, waits for it to be healthy,
//! snapshots it, stops it, restores the snapshot, proves the state restored
//! is the state snapshotted, and starts it again until it is healthy. It
//! removes all it made, and names the step that failed.

use std::fmt;
use std::path::Path;

use crate::card::{Card, Transport};
use crate::machine;
use crate::podman::Podman;
use crate::runtime::{Agreed, App, Name};

/// Each step of a check, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Read,
    Asks,
    Machine,
    Pull,
    Start,
    Healthy,
    Snapshot,
    Stop,
    Restore,
    StartAgain,
    HealthyAgain,
    Remove,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Step::Read => "read the card",
            Step::Asks => "its asks",
            Step::Machine => "podman's machine",
            Step::Pull => "pull by digest",
            Step::Start => "start",
            Step::Healthy => "healthy",
            Step::Snapshot => "snapshot",
            Step::Stop => "stop",
            Step::Restore => "restore",
            Step::StartAgain => "start again",
            Step::HealthyAgain => "healthy again",
            Step::Remove => "remove what the check made",
        })
    }
}

/// The step that failed, and why.
#[derive(Debug)]
pub struct Failed {
    pub step: Step,
    pub why: String,
}

impl fmt::Display for Failed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.step, self.why)
    }
}

/// Run the check on the card in `dir`, under `name`, writing its snapshot
/// under `scratch`, which it makes and removes. `agree` is the yes of whoever runs it to the card's
/// asks. Each step passed is told to `passed`, with what it found.
pub fn check(
    podman: &Podman,
    dir: &Path,
    name: Name,
    scratch: &Path,
    agree: bool,
    passed: &mut dyn FnMut(Step, String),
) -> Result<Card, Failed> {
    let fail = |step: Step| {
        move |why: &dyn fmt::Display| Failed {
            step,
            why: why.to_string(),
        }
    };
    let text = std::fs::read_to_string(dir.join("compose.yaml"))
        .map_err(|e| fail(Step::Read)(&format!("{}: {e}", dir.join("compose.yaml").display())))?;
    let card = Card::parse(&text).map_err(|e| fail(Step::Read)(&e))?;
    passed(Step::Read, card.title.clone());
    if !card.asks.is_empty() && !agree {
        let asks: Vec<String> = card.asks.iter().map(|a| format!("{} {}", a.say, a.link)).collect();
        let why = format!("{} Read them, then run the check with --agree.", asks.join(" "));
        tracing::debug!(card = card.title, "refused: its asks are not agreed");
        return Err(Failed { step: Step::Asks, why });
    }
    passed(Step::Asks, format!("{} agreed", card.asks.len()));
    let app = App::new(podman, &card, name);
    let run = steps(podman, &card, &app, scratch, passed);
    let removed = app.remove();
    let _ = std::fs::remove_dir_all(scratch);
    run?;
    removed.map_err(|e| fail(Step::Remove)(&e))?;
    passed(Step::Remove, String::new());
    Ok(card)
}

fn steps(
    podman: &Podman,
    card: &Card,
    app: &App,
    scratch: &Path,
    passed: &mut dyn FnMut(Step, String),
) -> Result<(), Failed> {
    let fail = |step: Step| {
        move |why: &dyn fmt::Display| Failed {
            step,
            why: why.to_string(),
        }
    };
    machine::ensure(podman, card.memory()).map_err(|e| fail(Step::Machine)(&e))?;
    passed(Step::Machine, String::new());
    app.pull().map_err(|e| fail(Step::Pull)(&e))?;
    passed(
        Step::Pull,
        card.services
            .iter()
            .map(|s| s.image.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
    let at = free_port(card.door.protocol.transport()).map_err(|e| fail(Step::Start)(&e))?;
    let agreed = Agreed::to(&card.asks);
    app.start(at, &agreed).map_err(|e| fail(Step::Start)(&e))?;
    passed(Step::Start, format!("its door on 127.0.0.1:{at}"));
    app.wait_healthy().map_err(|e| fail(Step::Healthy)(&e))?;
    passed(Step::Healthy, String::new());
    let snapshot = app.snapshot(scratch).map_err(|e| fail(Step::Snapshot)(&e))?;
    passed(
        Step::Snapshot,
        format!("{:?}, paused {:?}", snapshot.copies, snapshot.paused),
    );
    app.stop().map_err(|e| fail(Step::Stop)(&e))?;
    passed(Step::Stop, String::new());
    app.restore(&snapshot.dir).map_err(|e| fail(Step::Restore)(&e))?;
    let differs = app
        .differs(&snapshot.dir, scratch)
        .map_err(|e| fail(Step::Restore)(&e))?;
    if !differs.is_empty() {
        tracing::debug!(?differs, "the state restored is not the state snapshotted");
        return Err(fail(Step::Restore)(&format!(
            "the state restored is not the snapshot's: {}",
            differs.join("; ")
        )));
    }
    passed(Step::Restore, "the state is the snapshot's".into());
    app.start(at, &agreed).map_err(|e| fail(Step::StartAgain)(&e))?;
    passed(Step::StartAgain, String::new());
    app.wait_healthy().map_err(|e| fail(Step::HealthyAgain)(&e))?;
    passed(Step::HealthyAgain, String::new());
    Ok(())
}

/// A port nothing on this computer's loopback holds now, for the door.
fn free_port(transport: Transport) -> std::io::Result<u16> {
    match transport {
        Transport::Tcp => std::net::TcpListener::bind("127.0.0.1:0")?
            .local_addr()
            .map(|a| a.port()),
        Transport::Udp => std::net::UdpSocket::bind("127.0.0.1:0")?.local_addr().map(|a| a.port()),
    }
}
