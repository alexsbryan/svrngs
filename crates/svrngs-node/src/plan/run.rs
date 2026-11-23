// SPDX-License-Identifier: AGPL-3.0-or-later
use super::words::{self, RUN_UNREAD};
use super::{
    plan, step, Availability, Done, Event, Method, Nature, Plan, Platform, Return, RunView, Runs,
    State, Terms,
};
use crate::{
    door::bound,
    job,
    lend::{self, Answer, Audience, Credentials, Lend, Raw, Source},
    share::{self, Granted, Pid, ShareRefusal},
    uid::{Answering, Unread},
};
use std::{collections::BTreeSet, path::Path};

/// Run is local only; it takes no share witness or access terms.
pub async fn native<'a>(
    name: &'a str,
    view: RunView,
    args: impl IntoIterator<Item = &'a str>,
) -> Result<String, String> {
    let out = keeper(["run", name, "--json"].into_iter().chain(args)).await?;
    let started = serde_json::from_slice(&out).map_err(|_| RUN_UNREAD)?;
    words::started(name, view, started)
}

pub(crate) async fn keeper<'a>(args: impl IntoIterator<Item = &'a str>) -> Result<Vec<u8>, String> {
    if !cfg!(target_os = "linux") {
        return Err(RUN_UNREAD.into());
    }
    let at = std::env::current_exe().map_err(|_| RUN_UNREAD)?;
    let mut child = tokio::process::Command::new(at.with_file_name("svrngs-keeper"));
    let answer = tokio::time::timeout(
        std::time::Duration::from_secs(90),
        child.args(args).kill_on_drop(true).output(),
    )
    .await;
    tracing::debug!(target: "plan", status = ?answer.as_ref().map(|out| out.as_ref().map(|o| o.status)), "plan: keeper answered");
    match answer {
        Ok(Ok(out)) if out.status.success() => Ok(out.stdout),
        _ => Err(RUN_UNREAD.into()),
    }
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Row {
    Kept(super::words::Kept),
    Unread { name: String },
    Removed,
}

pub(crate) async fn rows() -> Result<serde_json::Value, String> {
    let rows: Vec<Row> =
        serde_json::from_slice(&keeper(["list", "--json"]).await?).map_err(|_| RUN_UNREAD)?;
    Ok(words::run_form(
        rows.into_iter()
            .filter_map(|row| match row {
                Row::Kept(kept) => Some(words::owned(kept)),
                Row::Unread { name } => {
                    Some(serde_json::json!({ "name": name, "text": RUN_UNREAD }))
                }
                Row::Removed => None,
            })
            .collect(),
    ))
}

/// Replay first; only a current review may publish access.
pub fn confirm(
    token: &str,
    observed: &State,
    publish: impl FnOnce(&Plan<'_>) -> Done,
) -> Option<Done> {
    let ready = plan(observed)?;
    if ready.token != token {
        tracing::debug!(target: "plan", "plan: confirm requires a new review");
        return None;
    }
    Some(publish(&ready))
}

pub struct Share<'a> {
    pub dir: &'a Path,
    pub lend: &'a Lend,
    pub terms: Terms,
    pub pid: Option<Pid>,
    pub replace: bool,
    pub door: Return,
    pub contract: &'a str,
}

type Read = Result<Answering, Unread>;
pub struct Witness {
    granted: Granted,
    pub answers: Vec<Answer>,
}

impl Share<'_> {
    pub fn observe(
        &self,
        read: impl FnOnce(Option<Pid>) -> Read,
    ) -> Result<(State, Witness), ShareRefusal> {
        let (dir, lend) = (self.dir, self.lend);
        let rev = lend::revision(dir, &lend.name)
            .map_err(|e| ShareRefusal::Unkept(lend.name.clone(), e.to_string()))?;
        share::renamed(dir, lend, self.replace)?;
        let port = lend.port().expect("a port share");
        let raw = match lend.source {
            Source::Raw(raw, _) => Some(raw),
            _ => None,
        };
        let (approved, answers) =
            share::approve((port, lend.raw()), read(self.pid), self.pid, None)?;
        let binding =
            serde_json::json!([share::token(&approved), lend.kind, self.contract]).to_string();
        let select = self.terms.availability == Availability::Keep && lend.raw() != Raw::Udp;
        let (runs, app, keep, pid) = if select {
            deployment(&answers, &approved.cgroups)
        } else {
            (Runs::Transient, Return::Unknown, None, None)
        };
        let platform = if cfg!(target_os = "macos") {
            Platform::Mac
        } else {
            Platform::Linux
        };
        let events = [
            Event::Revision(rev),
            Event::Nature(Nature {
                name: lend.name.clone(),
                port,
                platform,
                raw,
                runs,
                pid: self.pid.or(pid),
                binding,
            }),
            Event::Starts {
                app,
                door: self.door,
            },
            Event::Terms(self.terms.clone()),
        ];
        let witness = Witness {
            granted: (approved, keep),
            answers,
        };
        Ok((events.into_iter().fold(State::default(), step), witness))
    }

    pub fn execute(&self, ready: &Plan<'_>, witness: &Witness, secret: Credentials<'_>) -> Done {
        let (approved, keep) = &witness.granted;
        if keep
            .as_ref()
            .is_some_and(|keep| !bound::witnesses_job(ready.nature.port.get(), approved, keep))
        {
            return Done::Unverified;
        }
        publish(
            self.dir,
            ready,
            self.lend.clone(),
            witness.granted.clone(),
            secret,
        )
    }
}

pub(crate) fn own_return() -> Return {
    #[cfg(target_os = "linux")]
    if let Some(Answer::Job { unit, .. }) = job::of(std::process::id(), crate::uid::mine().ok()) {
        return service_return(&unit);
    }
    Return::Unknown
}

pub fn service_return(unit: &str) -> Return {
    // Unit enablement proves a sign-in path; it doesn't prove unattended disk unlock.
    let Ok(show) = job::show(unit) else {
        return Return::Unknown;
    };
    let enabled = show.get("UnitFileState").map(String::as_str) == Some("enabled");
    let wanted = show
        .get("WantedBy")
        .is_some_and(|v| v.split_whitespace().any(|t| t == "default.target"));
    if enabled && wanted {
        Return::SignIn
    } else {
        Return::Unknown
    }
}

fn deployment(
    answers: &[Answer],
    cgroups: &BTreeSet<u64>,
) -> (Runs, Return, Option<job::Keep>, Option<Pid>) {
    match answers {
        [Answer::Job { unit, .. }] => match job::now(unit) {
            Ok(now) if !cgroups.is_empty() && cgroups.is_subset(&now.cgroups) => (
                Runs::OwnService(now.keep.clone()),
                service_return(unit),
                Some(now.keep),
                None,
            ),
            _ => (Runs::Transient, Return::Unknown, None, None),
        },
        [Answer::Account { account }] => (
            Runs::OtherAccount(account.clone()),
            Return::Unknown,
            None,
            None,
        ),
        [Answer::Program { pid, start, .. }] => (
            Runs::ByHand,
            Return::ThisRun,
            None,
            Some(Pid {
                pid: *pid,
                start: *start,
            }),
        ),
        _ => (Runs::Transient, Return::Unknown, None, None),
    }
}

/// The access step of a confirmed plan, with the binding and grant just observed.
pub fn publish(
    dir: &Path,
    plan: &Plan<'_>,
    mut lend: Lend,
    (approved, keep): Granted,
    secret: Credentials<'_>,
) -> Done {
    let (nature, port) = (plan.nature, plan.nature.port);
    lend.name = nature.name.clone();
    lend.source = nature
        .raw
        .map_or(Source::Port(port), |raw| Source::Raw(raw, port));
    lend.terms.approved = Some(approved);
    lend.terms.keep = keep.filter(|_| !matches!(plan.method, Method::ThisRun));
    lend.terms.audience = Audience::Selected(plan.terms.groups.keys().cloned().collect());
    lend.terms.limit = plan.terms.limit;
    let saved = lend::publish(dir, lend, plan.if_rev, secret);
    tracing::debug!(target: "plan", lend = %nature.name, ?saved, "plan: conditional share publication");
    match saved {
        Ok(rev) => Done::Published { rev },
        Err(e) => Done::NotPublished { why: e.to_string() },
    }
}
