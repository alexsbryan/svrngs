// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    plan, step, Availability, Done, Event, Method, Nature, Plan, Platform, Return, Runs, State,
    Terms,
};
use crate::{
    job,
    lend::{self, Answer, Audience, Credentials, Lend, Raw, Source},
    share::{self, Granted, Pid, ShareRefusal},
    uid::{Answering, Unread},
};
use std::path::Path;

pub enum Confirmation<T> {
    ReviewAgain(State, T),
    Finished(State, T),
    Unavailable,
}

/// Replay first; only a current review may publish access.
pub fn confirm<T>(
    token: &str,
    replay: impl FnOnce() -> (State, T),
    publish: impl FnOnce(&T, &Plan<'_>) -> Done,
) -> Confirmation<T> {
    let (observed, witness) = replay();
    let Some(ready) = plan(&observed) else {
        return Confirmation::Unavailable;
    };
    if ready.token != token {
        tracing::debug!(target: "plan", "plan: confirm requires a new review");
        return Confirmation::ReviewAgain(observed, witness);
    }
    let done = publish(&witness, &ready);
    Confirmation::Finished(step(observed, Event::Done(done)), witness)
}

pub struct Share<'a> {
    pub dir: &'a Path,
    pub lend: &'a Lend,
    pub terms: Terms,
    pub pid: Option<Pid>,
    pub replace: bool,
    pub door: Return,
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
        let binding = share::token(&approved);
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
        publish(
            self.dir,
            ready,
            self.lend.clone(),
            witness.granted.clone(),
            secret,
        )
    }
}

pub fn service_return(unit: &str) -> Return {
    // Unit enablement proves a sign-in path; it doesn't prove unattended disk unlock.
    if job::show(unit).is_ok_and(|show| {
        show.get("UnitFileState").is_some_and(|v| v == "enabled")
            && show
                .get("WantedBy")
                .is_some_and(|v| v.split_whitespace().any(|t| t == "default.target"))
    }) {
        Return::SignIn
    } else {
        Return::Unknown
    }
}

type Groups = std::collections::BTreeSet<u64>;

fn in_job(cgroups: &Groups, job: &job::Now) -> bool {
    !cgroups.is_empty() && cgroups.is_subset(&job.cgroups)
}

fn deployment(
    answers: &[Answer],
    cgroups: &Groups,
) -> (Runs, Return, Option<job::Keep>, Option<Pid>) {
    match answers {
        [Answer::Job { unit, .. }] => match job::now(unit) {
            Ok(now) if in_job(cgroups, &now) => (
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
