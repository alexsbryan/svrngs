// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Done, Hand, Job, Method, Plan, Reason, Report, Return, Runs};
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Review<'a> {
    pub promise: String,
    pub reason: Option<String>,
    pub impact: String,
    pub button: String,
    pub runs: Option<&'a str>,
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

    pub fn words(&self) -> Review<'_> {
        let name = &self.nature.name;
        let promise = self.promise(&self.audience());
        let reason = self.reason.map(|reason| {
            let why = match reason {
                Reason::Udp => "this program shares a UDP port".into(),
                Reason::OtherAccount(account) => format!("{name} runs as the account {account}"),
                Reason::Transient => format!("{name} has no saved setup to start it again"),
                Reason::Declined(why) => why.into(),
                Reason::MacContainer => "svrngs can’t verify what will answer on this port when the container starts again".into(),
                Reason::AppStart => format!("svrngs couldn’t verify when {name} will start again"),
                Reason::DoorStart => "svrngs couldn’t verify when sharing itself will start again".into(),
            };
            format!("svrngs can’t verify that sharing will come back by itself with this setup: {why}.")
        });
        let (impact, button) = match self.method {
            Method::Takeover(_) => (
                format!("svrngs will restart {name} now. Anyone using it will disconnect."),
                "Restart and share".into(),
            ),
            Method::Reuse(_) => (
                format!("{name} keeps running as it does now."),
                format!("Share {name}"),
            ),
            Method::ThisRun => (
                format!("svrngs won’t restart {name}."),
                "Share this run".into(),
            ),
        };
        let runs = match &self.nature.runs {
            Runs::ByHand(Hand::Known(recipe) | Hand::Copyable(recipe))
            | Runs::Container { recipe, .. } => Some(recipe.shown.as_str()),
            _ => None,
        };
        Review {
            promise,
            reason,
            impact,
            button,
            runs,
        }
    }

    pub fn report_words(&self, report: &Report) -> Option<Line> {
        use {Done::*, Job::*};
        let name = &self.nature.name;
        let audience = self.audience();
        let (text, button) = match report {
            Report::Job(Starting) => ("Starting…".into(), None),
            Report::Job(Started) | Report::Done(Verified) => return None,
            Report::Job(KeepsStopping { said }) => (format!("Not available: {name} {said}"), Some(format!("Start {name} again"))),
            Report::Job(StoppedBySharer) => ("Stopped by you, after a restart too.".into(), Some(format!("Start {name}"))),
            Report::Job(SetupChanged) => (format!("Closed until you review it: {name} changed since you shared it."), Some(REVIEW.into())),
            Report::Job(PortTaken { by }) => (format!("Closed: a program of the account {by} is on {name}’s port. It opens again by itself once {name} has it back."), None),
            Report::Job(SharingStopped) => (format!("{name} still runs here for you. {audience} can’t use it any more."), Some("Stop running it here".into())),
            Report::Done(RolledBack { said }) => (format!("{name} didn’t start in its new place, so yours was started again as it was. It last said: ‘{said}’"), None),
            Report::Done(Interrupted { .. }) => (format!("{name} isn’t running: moving it didn’t finish."), Some(format!("Start {name}"))),
            Report::Done(Published { .. }) => (format!("Shared with {audience}. {}", self.promise(&audience)), None),
            Report::Done(NotPublished { why }) => (format!("Running here; not shared. The share couldn’t be saved: {why}."), Some(REVIEW.into())),
        };
        Some(Line { text, button })
    }
}
