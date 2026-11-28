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
        Starting => ("Starting…".into(), Stop),
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

impl Plan<'_> {
    fn audience(&self) -> String {
        let names: Vec<_> = self.terms.groups.values().map(String::as_str).collect();
        names.join(", ")
    }

    fn promise(&self, audience: &str) -> String {
        let after = match self.when {
            Return::SignIn => "you sign in.",
            Return::ThisRun | Return::Unknown => {
                return format!("{audience} can use it while it’s running.")
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
                Reason::AppStart => format!("svrngs couldn’t verify when {name} will start again"),
                Reason::DoorStart => {
                    "svrngs couldn’t verify when sharing itself will start again".into()
                }
            };
            format!(
                "svrngs can’t verify that sharing will come back by itself with this setup: {why}."
            )
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
                    text += &format!("\n{SHARED_WHILE} To have svrngs keep it running, stop it, then put svrngs keep {} -- in front of the command you started it with.", self.nature.port);
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

/// What the keeper said of a Run: kept, verified, and when it comes back;
/// or why nothing was kept, in the person's words.
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Started {
    Verified { back: String },
    Refused { why: String },
}

#[derive(Debug)]
pub enum RunView {
    Page,
    Terminal,
}

pub(super) fn started(name: &str, view: RunView, started: Started) -> Result<String, String> {
    tracing::debug!(target: "plan", ?started, ?view, "plan: Run verdict");
    match started {
        Started::Verified { back } => Ok(format!("svrngs runs {name} now. It comes back {back}.")),
        Started::Refused { why } => Err(why),
    }
}

#[cfg(test)]
mod tests;
