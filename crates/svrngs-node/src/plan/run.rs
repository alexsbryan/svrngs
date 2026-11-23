// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    plan, step, Availability, Done, Event, Hand, Method, Nature, Plan, Platform, Recipe, Return,
    Runs, State, Step, Terms,
};
use crate::{
    job,
    lend::{self, Answer, Audience, Credentials, Lend, Raw, Source},
    share::{self, Granted, Pid, ShareRefusal},
    uid::{Answering, Unread},
};
use serde::Deserialize;
use std::{
    num::NonZeroU16,
    path::Path,
    process::{Command, Stdio},
};

pub enum Confirmation<T> {
    ReviewAgain(State, T),
    Finished(State, T),
    Unavailable,
}

/// Replay first; only a current review may call the effect owners in plan order.
pub fn confirm<T>(
    token: &str,
    replay: impl FnOnce() -> (State, T),
    mut run: impl FnMut(&mut T, Step<'_>) -> Done,
) -> Confirmation<T> {
    let (observed, mut witness) = replay();
    let Some(ready) = plan(&observed) else {
        return Confirmation::Unavailable;
    };
    if ready.token != token {
        tracing::debug!(target: "plan", "plan: confirm requires a new review");
        return Confirmation::ReviewAgain(observed, witness);
    }
    let mut result = observed.clone();
    for next in ready.steps() {
        let done = run(&mut witness, next);
        let proceed = done == Done::Verified;
        tracing::debug!(target: "plan", ?done, proceed, "plan: confirmed step returned");
        result = step(result, Event::Done(done));
        if !proceed {
            break;
        }
    }
    Confirmation::Finished(result, witness)
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
    successor: Option<String>,
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
            deployment(lend, port, &answers, &approved.cgroups)
        } else {
            (Runs::Transient, Return::Unknown, None, self.pid)
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
                pid,
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
            successor: None,
        };
        Ok((events.into_iter().fold(State::default(), step), witness))
    }

    pub fn execute(
        &self,
        next: Step<'_>,
        witness: &mut Witness,
        secret: Credentials<'_>,
        read: impl FnOnce(Option<Pid>) -> Read,
    ) -> Done {
        let port = self.lend.port().expect("a port share");
        match next {
            Step::Keeper(recipe) => match keeper([
                "keep",
                &port.to_string(),
                "--name",
                self.lend.name.as_str(),
                "--yes",
                "--token",
                &recipe.fingerprint,
            ]) {
                Ok(KeeperEnd::Verified { unit }) => {
                    // Record the keeper's answer separately from any failed access witness.
                    witness.successor = Some(unit);
                    Done::Verified
                }
                Ok(KeeperEnd::RolledBack { why }) => Done::RolledBack { said: why },
                Err(why) => Done::NotPublished { why },
            },
            Step::Access(ready) => {
                if matches!(ready.method, Method::Takeover(_)) {
                    let fresh =
                        share::approve((port, self.lend.raw()), read(None), None, None).ok();
                    let old = &witness.granted.0;
                    let now = witness
                        .successor
                        .as_deref()
                        .and_then(|unit| job::now(unit).ok());
                    let verified = fresh
                        .zip(now)
                        .filter(|((a, _), job)| a.at == old.at && in_job(&a.cgroups, job));
                    let Some(((approved, answers), now)) = verified else {
                        return Done::NotPublished {
                            why: "what answers on the port is no longer the verified program"
                                .into(),
                        };
                    };
                    witness.granted = (approved, Some(now.keep));
                    witness.answers = answers;
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
    lend: &Lend,
    port: NonZeroU16,
    answers: &[Answer],
    cgroups: &Groups,
) -> (Runs, Return, Option<job::Keep>, Option<Pid>) {
    let declined = |why| {
        (
            Runs::ByHand(Hand::Declined(why)),
            Return::Unknown,
            None,
            None,
        )
    };
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
        [Answer::Program { pid, start, .. }] if cfg!(target_os = "linux") => {
            let seen = Pid {
                pid: *pid,
                start: *start,
            };
            let facts = keeper(["facts", &port.to_string(), "--name", lend.name.as_str()]);
            let (offer, known) = match facts {
                Ok(KeeperFacts::Known(offer)) => (offer, true),
                Ok(KeeperFacts::Copyable(offer)) => (offer, false),
                Ok(KeeperFacts::Declined { why }) | Err(why) => return declined(why),
                Ok(KeeperFacts::Kept) => {
                    return declined(
                        "the current listener could not be tied to its saved setup".into(),
                    )
                }
            };
            if offer.listener != seen {
                return declined("the program changed while it was being read".into());
            }
            let mut shown = offer.what;
            for (label, omitted) in [
                ("flags", offer.omitted_flags),
                ("settings", offer.omitted_settings),
            ] {
                if !omitted.is_empty() {
                    shown += &format!(
                        "\nIt starts without the {label} your terminal added: {}.",
                        omitted.join(", ")
                    );
                }
            }
            let recipe = Recipe {
                fingerprint: offer.token,
                shown,
            };
            let hand = if known {
                Hand::Known(recipe)
            } else {
                Hand::Copyable(recipe)
            };
            (Runs::ByHand(hand), Return::SignIn, None, Some(seen))
        }
        _ => declined("this program could not be tied to one saved or restartable setup".into()),
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum KeeperFacts {
    Known(Offer),
    Copyable(Offer),
    Kept,
    Declined { why: String },
}

#[derive(Deserialize)]
struct Offer {
    listener: Pid,
    token: String,
    what: String,
    omitted_settings: Vec<String>,
    omitted_flags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum KeeperEnd {
    Verified { unit: String },
    RolledBack { why: String },
}

fn keeper<T: serde::de::DeserializeOwned, const N: usize>(args: [&str; N]) -> Result<T, String> {
    let read = || {
        let path = std::env::current_exe()?.with_file_name("svrngs-keeper");
        Command::new(path).args(args).stdin(Stdio::null()).output()
    };
    let out = read().map_err(|e| format!("svrngs couldn't ask what can run here: {e}"))?;
    let answer = serde_json::from_slice(&out.stdout)
        .map_err(|_| format!("svrngs couldn't verify what happened ({})", out.status));
    tracing::debug!(target: "plan", verb = args.first(), status = %out.status, parsed = answer.is_ok(), "plan: keeper answered");
    answer
}

/// The access step of a confirmed plan. The caller supplies the binding and
/// grant verified by the access owner, after takeover if one was needed.
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
