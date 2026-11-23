// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Done, Job, Method, Plan, Reason, Report, Return, Runs};
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Review {
    pub promise: String,
    pub reason: Option<String>,
    pub impact: String,
    pub button: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Line {
    pub text: String,
    pub button: Option<String>,
}

const REVIEW: &str = "Review sharing";
pub const REVIEW_AGAIN: &str = "Something changed since you reviewed this.";

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
            Method::Reuse(_) => (
                format!("{name} keeps running as it does now."),
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

    pub fn report_words(&self, report: &Report) -> Option<Line> {
        use {Done::*, Job::*};
        let name = &self.nature.name;
        let audience = self.audience();
        let (text, button) = match report {
            Report::Job(Starting) => ("Starting…".into(), None),
            Report::Job(Started) => return None,
            Report::Job(KeepsStopping { said }) => (format!("Not available: {name} {said}"), Some(format!("Start {name} again"))),
            Report::Job(StoppedBySharer) => ("Stopped by you, after a restart too.".into(), Some(format!("Start {name}"))),
            Report::Job(SetupChanged) => (format!("Closed until you review it: {name} changed since you shared it."), Some(REVIEW.into())),
            Report::Job(PortTaken { by }) => (format!("Closed: a program of the account {by} is on {name}’s port. It opens again by itself once {name} has it back."), None),
            Report::Job(SharingStopped) => (format!("{name} still runs here for you. {audience} can’t use it any more."), Some("Stop running it here".into())),
            Report::Done(Published { .. }) => {
                let mut text = format!("Shared with {audience}. {}", self.promise(&audience));
                if matches!(self.nature.runs, Runs::ByHand) {
                    text += &format!("\nShared while you run it. To have svrngs run it: svrngs run {name} --port {} -- <the command you start it with>.", self.nature.port);
                }
                (text, None)
            }
            Report::Done(NotPublished { why }) if matches!(self.method, Method::Reuse(_)) => (format!("Running here; not shared. The share couldn’t be saved: {why}."), Some(REVIEW.into())),
            Report::Done(NotPublished { why }) => (format!("The share couldn’t be saved: {why}."), Some(REVIEW.into())),
            Report::Done(Unverified) => (format!("Not shared; svrngs couldn't verify {name} is answering."), Some(REVIEW.into())),
        };
        Some(Line { text, button })
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Started {
    Verified,
    Unverified { running: bool, port: u16 },
    Busy { why: String },
}

pub(super) fn started(name: &str, started: Started) -> Result<String, String> {
    use Started::*;
    tracing::debug!(target: "plan", ?started, "plan: keeper Run verdict");
    match started {
        Verified => Ok(format!("{name} is running here. Share it separately with `svrngs share`.\n")),
        Unverified { running: true, port } => Err(format!("Not shared; {name} is running, but nothing answers on port {port} yet. Check the port in its command.")),
        Unverified { running: false, .. } => Err(format!("Not shared; svrngs couldn't verify {name} is answering.")),
        Busy { why } => Err(why),
    }
}
