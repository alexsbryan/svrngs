// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Done, Method, Plan, Reason, Return, Runs};
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Review {
    pub promise: String,
    pub reason: Option<String>,
    pub impact: String,
    pub button: String,
}

pub const REVIEW_AGAIN: &str = "Something changed since you reviewed this.";
pub(crate) const RUN_UNREAD: &str =
    "svrngs couldn’t verify this. Check it from your terminal, then look again.";
pub(crate) const STARTING: &str = "Starting…";
pub(crate) const SHARED_WHILE: &str = "Shared while you run it.";

/// The port of a lend shared only while its program runs, which the home
/// page offers to have svrngs keep running (lent-row.js `Lent`): one
/// kept to no job's restarts, on a system whose keeper can run it.
pub(crate) fn shared_while(lend: &crate::lend::Lend) -> Option<std::num::NonZeroU16> {
    let crate::lend::Source::Port(port) = lend.source else {
        return None;
    };
    (cfg!(any(target_os = "linux", target_os = "macos")) && lend.terms.keep.is_none())
        .then_some(port)
}

#[derive(serde::Deserialize)]
pub(super) struct Kept {
    name: String,
    port: u16,
    status: Status,
}

#[derive(serde::Deserialize)]
struct Status {
    running: Running,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Running {
    Since(Option<String>),
    Starting,
    Stopped,
    Failed(String),
}

pub(super) fn owned(Kept { name, port, status }: Kept) -> serde_json::Value {
    use {Action::*, Running::*};
    let (text, action) = match status.running {
        Since(Some(at)) => (format!("Run by svrngs, since {at}."), Stop),
        Since(None) => ("Run by svrngs.".into(), Stop),
        Starting => (STARTING.into(), Stop),
        Stopped => ("Stopped.".into(), Start),
        Failed(why) => (format!("{name} stopped ({why}). {RUN_UNREAD}"), Start),
    };
    // Its button names its action, Start or Stop, and the program.
    let button = format!("{action:?} {name}");
    serde_json::json!({ "name": name, "port": port, "text": text, "action": action, "button": button })
}

#[derive(Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Action {
    Start,
    Stop,
}

impl Action {
    pub(crate) fn verb(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
        }
    }
}

pub(super) fn run_form(rows: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({ "starting": STARTING, "rows": rows })
}

impl Plan<'_> {
    fn audience(&self) -> String {
        let names: Vec<_> = self.terms.groups.values().map(String::as_str).collect();
        names.join(", ")
    }

    fn promise(&self, audience: &str) -> String {
        let after = match self.when {
            Return::Startup => "this computer starts, even before you sign in.",
            Return::AfterUnlock => "you unlock the disk.",
            Return::SignIn => "you sign in.",
            Return::ThisRun | Return::Unknown => {
                let now = "can use this copy now. If it stops, you’ll need to share it again.";
                return format!("{audience} {now}");
            }
        };
        format!("{audience} can open it again after {after}")
    }

    pub fn words(&self) -> Review {
        let name = &self.nature.name;
        let promise = self.promise(&self.audience());
        let reason = self.reason.map(|reason| {
            let why = match reason {
                Reason::Udp => "this program shares a UDP port".into(),
                Reason::OtherAccount(account) => format!("{name} runs as the account {account}"),
                Reason::Transient => format!("{name} has no saved setup to start it again"),
                Reason::ByHand => format!("{name} was started by hand"),
                Reason::Container => "svrngs can’t verify what will answer on this port when the container starts again".into(),
                Reason::AppStart => format!("svrngs couldn’t verify when {name} will start again"),
                Reason::DoorStart => "svrngs couldn’t verify when sharing itself will start again".into(),
            };
            format!("svrngs can’t verify that sharing will come back by itself with this setup: {why}.")
        });
        let (impact, button) = match self.method {
            Method::Reuse(job) => (
                format!("Trust your service {job} to answer after a restart. The share can follow it even if the program at the same path is replaced."),
                format!("Share {name}"),
            ),
            Method::ThisRun => (
                format!("svrngs won’t restart {name}."),
                "Share this run".into(),
            ),
        };
        Review {
            promise,
            reason,
            impact,
            button,
        }
    }

    pub fn report_words(&self, report: &Done) -> String {
        use Done::*;
        let name = &self.nature.name;
        let audience = self.audience();
        match report {
            Published { .. } => {
                let mut text = format!("Shared with {audience}. {}", self.promise(&audience));
                if matches!(self.nature.runs, Runs::ByHand) {
                    text += &format!("\n{SHARED_WHILE} To have svrngs run it: svrngs run {name} --port {} -- <the command you start it with>.", self.nature.port);
                }
                text
            }
            NotPublished { why } if matches!(self.method, Method::Reuse(_)) => {
                format!("Running here; not shared. The share couldn’t be saved: {why}.")
            }
            NotPublished { why } => format!("The share couldn’t be saved: {why}."),
            Unverified => format!("Not shared; svrngs couldn't verify {name} is answering."),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Started {
    Verified { port: std::num::NonZeroU16 },
    Unverified { state: Option<String>, ready: bool },
    Busy { why: String },
}

#[derive(Debug)]
pub enum RunView {
    Page,
    Terminal,
}

pub(super) fn started(name: &str, view: RunView, started: Started) -> Result<String, String> {
    use Started::*;
    tracing::debug!(target: "plan", ?started, ?view, "plan: Run verdict");
    match started {
        Verified { port } => Ok(match view {
            RunView::Page => format!("svrngs runs {name} now."),
            RunView::Terminal => format!("{name} is running here. To share it: `svrngs share {port} --program <program> --name {name} --group <group>`.\n"),
        }),
        Unverified { state, ready } => Err(format!("{name}'s setup was saved and can start again. Nothing was shared. To turn it off, choose Stop. Reported state: {}. The port {}.",
            state.as_deref().unwrap_or("unknown"), if ready { "answered" } else { "wasn't ready" })),
        Busy { why } => Err(why),
    }
}

#[cfg(test)]
mod tests;
