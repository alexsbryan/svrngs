// SPDX-License-Identifier: AGPL-3.0-or-later
//! The write side. A command is decided over the group's admitted acts, under
//! the store's write lock, and [`commit`] is the one way lines reach a store:
//! a [`Store`] appends only [`Authored`] lines, which only `commit` makes.

use alloc::vec::Vec;

use crate::{admit, digest, sign_act, ActSet, Admission, Op, OpId, RailAct, SignedOp};
use crate::{RingSigner, RingVerifier, Roster};

/// Which of the held acts a command's decision reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reads {
    /// The acts the register folds ([`RailAct::moves_membership`]).
    Membership,
    /// Every act held.
    All,
}

/// A request to change a group's record. A type implements this once, so a
/// command has exactly one decider.
///
/// ```
/// fn needs<C: svrngs_core::Command>() {}
/// needs::<svrngs_core::Redeem>();
/// ```
/// ```compile_fail
/// fn needs<C: svrngs_core::Command>() {}
/// struct NotACommand;
/// needs::<NotACommand>();
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a Command, so it cannot reach the record",
    label = "commit takes a Command",
    note = "add `impl Command for {Self}` in svrngs-core: `decide` over the Admission, and `confirm` when the fold must judge the result"
)]
pub trait Command {
    type Refusal;
    const READS: Reads;

    /// The acts to write, decided over what the group holds now.
    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, Self::Refusal>;

    /// The fold's verdict on the record with `written` in it.
    fn confirm(&self, _after: &Admission, _written: &[OpId]) -> Result<(), Self::Refusal> {
        Ok(())
    }
}

/// The adapter's facts: who signs, against which seed, in which group, when.
pub struct Author<'a> {
    pub signer: &'a dyn RingSigner,
    pub roster: &'a Roster,
    pub namespace: &'a str,
    pub verifier: &'a dyn RingVerifier,
    pub now: i64,
    pub on_behalf_of: Option<&'a str>,
}

/// Why a commit wrote nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused<R> {
    /// The signer holds no standing in the group.
    NoStanding,
    Command(R),
}

/// Signed lines only [`commit`] makes: the one thing a [`Store`] may append.
///
/// ```
/// fn append(lines: &svrngs_core::Authored) -> usize { lines.lines().len() }
/// ```
/// ```compile_fail
/// let lines = svrngs_core::Authored(Vec::new());
/// ```
pub struct Authored(Vec<Op<SignedOp>>);

impl Authored {
    pub fn lines(&self) -> &[Op<SignedOp>] {
        &self.0
    }
}

/// Lines a store has appended. Only [`commit`] makes one, from what the store
/// returned after appending.
///
/// ```
/// fn proof(r: &svrngs_core::Recorded) -> usize { r.lines().len() }
/// ```
/// ```compile_fail
/// let r = svrngs_core::Recorded(Vec::new());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded(Vec<Op<SignedOp>>);

impl Recorded {
    pub fn lines(&self) -> &[Op<SignedOp>] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The port the write side appends through.
pub trait Store {
    type Error;

    /// Under the write lock: hands `f` the lines held, appends what it
    /// authored, and returns its answer only once the append succeeded.
    fn append_with<R>(
        &self,
        f: impl FnOnce(&ActSet) -> (Option<Authored>, R),
    ) -> Result<R, Self::Error>;
}

/// Decides `command` over what `store` holds and appends the acts it asks
/// for, all under one write lock.
pub fn commit<C: Command, S: Store>(
    store: &S,
    command: &C,
    author: &Author<'_>,
) -> Result<Result<Recorded, Refused<C::Refusal>>, S::Error> {
    store.append_with(|held| match authored(held, command, author) {
        Ok(lines) => {
            let recorded = Recorded(lines.clone());
            (Some(Authored(lines)), Ok(recorded))
        }
        Err(why) => (None, Err(why)),
    })
}

fn authored<C: Command>(
    held: &ActSet,
    command: &C,
    a: &Author<'_>,
) -> Result<Vec<Op<SignedOp>>, Refused<C::Refusal>> {
    let actor = a.signer.actor();
    let before = admit(&read::<C>(held), &[], a.roster, a.namespace, a.verifier);
    if !before.membership().standing.contains(&actor) {
        tracing::debug!(target: "rail:door", ns = a.namespace, %actor,
            "commit refused — the signer holds no standing in the group");
        return Err(Refused::NoStanding);
    }
    let acts = command.decide(&before).map_err(Refused::Command)?;
    if acts.is_empty() {
        tracing::debug!(target: "rail:door", ns = a.namespace, %actor,
            "commit: the decision asks for nothing, so nothing is written");
        return Ok(Vec::new());
    }
    let lines = sign_lines(held, acts, a);
    let mut with = held.clone();
    for line in &lines {
        with.insert(line.clone());
    }
    let after = admit(&read::<C>(&with), &[], a.roster, a.namespace, a.verifier);
    let ids: Vec<OpId> = lines.iter().map(|l| l.id.clone()).collect();
    command.confirm(&after, &ids).map_err(Refused::Command)?;
    Ok(lines)
}

fn read<C: Command>(held: &ActSet) -> ActSet {
    match C::READS {
        Reads::All => held.clone(),
        Reads::Membership => ActSet::of(held.iter().filter(|op| op.kind.act.moves_membership())),
    }
}

/// Signs `acts` as the signer's next lines: after its newest seq, naming the
/// line there its signature verifies, and timed strictly after every line
/// held, since the rail orders by `(ts_unix, actor, seq, id)`.
fn sign_lines(held: &ActSet, acts: Vec<RailAct>, a: &Author<'_>) -> Vec<Op<SignedOp>> {
    let actor = a.signer.actor();
    let newest = held
        .iter()
        .filter(|op| op.actor == actor)
        .map(|op| op.kind.seq)
        .max();
    let mut prev = newest.and_then(|seq| {
        let at: Vec<&Op<SignedOp>> = held
            .iter()
            .filter(|op| op.actor == actor && op.kind.seq == seq)
            .collect();
        at.iter()
            .find(|op| a.verifier.verify(&actor, &op.id, &op.kind.sig))
            .or(at.first())
            .map(|op| op.id.clone())
    });
    let first_seq = newest.map_or(0, |seq| seq + 1);
    let after = held
        .iter()
        .map(|op| op.ts_unix.saturating_add(1))
        .max()
        .unwrap_or(i64::MIN);
    let ts_unix = a.now.max(after);
    tracing::debug!(target: "rail:door", ns = a.namespace, %actor, first_seq, ?prev,
        now = a.now, after, ts_unix,
        "commit: follows the signer's newest act, timed after every act held");
    let all: Vec<Op<SignedOp>> = held.iter().cloned().collect();
    let view = digest(&all, a.namespace, a.verifier);
    let mut lines = Vec::with_capacity(acts.len());
    for (seq, act) in (first_seq..).zip(acts) {
        let op = sign_act(
            a.signer,
            a.namespace,
            ts_unix,
            seq,
            prev.take(),
            act,
            a.on_behalf_of,
            Some(view.clone()),
        );
        prev = Some(op.id.clone());
        lines.push(op);
    }
    lines
}
