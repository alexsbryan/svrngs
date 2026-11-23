// SPDX-License-Identifier: AGPL-3.0-or-later
//! A preview is a fold of facts, not permission to run a program or publish a share.
//! See svrn-docs KEEP_IT_RUNNING_RELEASE §3.1. Only the owners can attest the
//! inputs, or supply a binding for a listener that has yet to start.

pub mod run;
mod words;
pub use words::{Line, Review, REVIEW_AGAIN};

use crate::{
    job::Keep,
    lend::{LendName, Raw},
    share::Pid,
};
use serde::Serialize;
use std::{collections::BTreeMap, num::NonZeroU16};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Nature(Nature),
    Starts { app: Return, door: Return },
    Terms(Terms),
    Revision(u64),
    Job(Job),
    Done(Done),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Nature {
    pub name: LendName,
    pub port: NonZeroU16,
    pub platform: Platform,
    pub raw: Option<Raw>,
    pub runs: Runs,
    /// Ports with several holders are witnessed by the binding alone.
    pub pid: Option<Pid>,
    /// The current approval fingerprint from `share::token`, never a new grant.
    pub binding: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Platform {
    Linux,
    Mac,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Runs {
    ByHand(Hand),
    Container { pod: String, recipe: Recipe },
    Kept(Keep),
    OwnService(Keep),
    OtherAccount(String),
    Transient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Hand {
    Known(Recipe),
    Copyable(Recipe),
    Declined(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Recipe {
    pub fingerprint: String,
    pub shown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Terms {
    /// Stable group id to the name shown at review. Order of selection isn't consent.
    pub groups: BTreeMap<String, String>,
    pub limit: Option<NonZeroU16>,
    pub availability: Availability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Availability {
    Keep,
    ThisRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Return {
    Startup,
    AfterUnlock,
    SignIn,
    ThisRun,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Job {
    Starting,
    Started,
    /// The owner's observation, including the measured count and its window.
    KeepsStopping {
        said: String,
    },
    StoppedBySharer,
    SetupChanged,
    PortTaken {
        by: String,
    },
    SharingStopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    Verified,
    RolledBack { said: String },
    Interrupted { stage: Stage },
    Published { rev: u64 },
    NotPublished { why: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Staged,
    Recorded,
    Stopped,
    Started,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Report {
    Job(Job),
    Done(Done),
    RunningNotShared { why: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    nature: Option<Nature>,
    starts: Option<(Return, Return)>,
    terms: Option<Terms>,
    rev: Option<u64>,
    pub report: Option<Report>,
}

pub fn step(mut state: State, event: Event) -> State {
    match event {
        Event::Nature(nature) => state.nature = Some(nature),
        Event::Starts { app, door } => state.starts = Some((app, door)),
        Event::Terms(terms) => state.terms = Some(terms),
        Event::Revision(rev) => state.rev = Some(rev),
        Event::Job(job) => state.report = Some(Report::Job(job)),
        Event::Done(Done::NotPublished { why })
            if state.report == Some(Report::Done(Done::Verified)) =>
        {
            tracing::debug!(target: "plan", %why, "plan: verified takeover could not be shared");
            state.report = Some(Report::RunningNotShared { why });
        }
        Event::Done(done) => state.report = Some(Report::Done(done)),
    }
    state
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Method<'a> {
    Reuse(&'a Keep),
    Takeover(&'a Recipe),
    ThisRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reason<'a> {
    Udp,
    OtherAccount(&'a str),
    Transient,
    Declined(&'a str),
    MacContainer,
    AppStart,
    DoorStart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan<'a> {
    pub when: Return,
    pub method: Method<'a>,
    reason: Option<Reason<'a>>,
    pub nature: &'a Nature,
    pub terms: &'a Terms,
    pub if_rev: u64,
    pub token: String,
}

pub enum Step<'a> {
    Keeper(&'a Recipe),
    /// `Reuse` carries the grant to verify. `Takeover` needs a fresh binding
    /// and grant from its result. `ThisRun` carries only the current binding.
    Access(&'a Plan<'a>),
}

impl Plan<'_> {
    /// Access always goes last; a takeover must first return a fresh witness.
    pub fn steps(&self) -> impl Iterator<Item = Step<'_>> {
        let takeover = match self.method {
            Method::Takeover(recipe) => Some(Step::Keeper(recipe)),
            Method::Reuse(_) | Method::ThisRun => None,
        };
        takeover.into_iter().chain([Step::Access(self)])
    }
}

pub fn plan(state: &State) -> Option<Plan<'_>> {
    let (Some(nature), Some((app, door)), Some(terms), Some(if_rev)) =
        (&state.nature, state.starts, &state.terms, state.rev)
    else {
        tracing::debug!(target: "plan", "plan: incomplete observation");
        return None;
    };
    if terms.groups.is_empty() {
        tracing::debug!(target: "plan", "plan: no groups chosen");
        return None;
    }
    let proposed = match (nature.raw, &nature.runs, nature.platform) {
        (Some(Raw::Udp), _, _) => Err(Reason::Udp),
        (_, Runs::Kept(job) | Runs::OwnService(job), _) => Ok(Method::Reuse(job)),
        (_, Runs::ByHand(Hand::Known(recipe) | Hand::Copyable(recipe)), _)
        | (_, Runs::Container { recipe, .. }, Platform::Linux) => Ok(Method::Takeover(recipe)),
        (_, Runs::ByHand(Hand::Declined(why)), _) => Err(Reason::Declined(why)),
        (_, Runs::Container { .. }, Platform::Mac) => Err(Reason::MacContainer),
        (_, Runs::OtherAccount(account), _) => Err(Reason::OtherAccount(account)),
        (_, Runs::Transient, _) => Err(Reason::Transient),
    };
    let later = app.max(door);
    let verified = match later {
        Return::ThisRun | Return::Unknown if app >= door => Err(Reason::AppStart),
        Return::ThisRun | Return::Unknown => Err(Reason::DoorStart),
        _ if nature.platform == Platform::Mac => Ok(Return::SignIn),
        _ => Ok(later),
    };
    // Deployment refusals explain the fallback before an unverified start does.
    let proposed = proposed.and_then(|method| verified.map(|when| (when, method)));
    let (when, method, reason) = match (terms.availability, proposed) {
        (Availability::ThisRun, _) => (Return::ThisRun, Method::ThisRun, None),
        (Availability::Keep, Err(why)) => (Return::ThisRun, Method::ThisRun, Some(why)),
        (Availability::Keep, Ok((when, method))) => (when, method, None),
    };
    let action = match method {
        Method::Reuse(_) => "reuse",
        Method::Takeover(_) => "takeover",
        Method::ThisRun => "this run",
    };
    tracing::debug!(target: "plan", ?app, ?door, ?when, %action, ?reason, "plan: verified return and least disruption");
    let basis = (nature, terms, if_rev, (app, door, when), &method);
    let bytes = serde_json::to_vec(&("svrngs share plan v1", basis)).expect("plan facts serialize");
    let token = hex::encode(ring::digest::digest(&ring::digest::SHA256, &bytes));
    Some(Plan {
        when,
        method,
        reason,
        nature,
        terms,
        if_rev,
        token,
    })
}
