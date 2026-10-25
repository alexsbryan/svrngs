// SPDX-License-Identifier: AGPL-3.0-or-later
//! The write side. A command is decided over the group's admitted acts, under
//! the store's write lock, and [`commit`] is the one way lines reach a store:
//! a [`Store`] appends only [`Authored`] lines, which only `commit` makes.

use alloc::string::String;
use alloc::vec::Vec;

use crate::{actor_of, RingVerifier, Roster, SigningKey};
use crate::{admit, derive_id, sign_act, ActSet, Admission, Op, OpId, RailAct, SignedOp};

/// Which of the held acts a command's decision reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reads {
    /// The acts the register folds ([`crate::standing_reads`]).
    Membership,
    /// Every act held: a removal names the removed key's newest acts (U7).
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
    pub signer: &'a SigningKey,
    pub roster: &'a Roster,
    pub namespace: &'a str,
    pub verifier: &'a dyn RingVerifier,
    pub now: i64,
}

/// Why a commit wrote nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused<R> {
    /// The signer holds no standing in the group.
    NoStanding,
    /// The group's cuts do not settle, before the acts or after them
    /// (#124): nothing is judged on that, so nothing is written.
    Unsettled,
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

/// A signer's newest seq in a store, and each line held at it: where its
/// next line continues its chain. The store is the authority on its own lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tip {
    pub seq: u64,
    pub lines: Vec<(OpId, String)>,
}

/// The port the write side appends through.
pub trait Store {
    type Error;

    /// Under the write lock: hands `f` the lines held and `actor`'s [`Tip`],
    /// appends what it authored, and returns its answer only once the append
    /// succeeded.
    fn append_with<R>(
        &self,
        actor: &str,
        f: impl FnOnce(&ActSet, Option<Tip>) -> (Option<Authored>, R),
    ) -> Result<R, Self::Error>;
}

/// Decides `command` over what `store` holds and appends the acts it asks
/// for, all under one write lock.
pub fn commit<C: Command, S: Store>(
    store: &S,
    command: &C,
    author: &Author<'_>,
) -> Result<Result<Recorded, Refused<C::Refusal>>, S::Error> {
    let actor = actor_of(author.signer);
    store.append_with(&actor, |held, tip| {
        match authored(held, tip, command, author) {
            Ok(lines) => {
                let recorded = Recorded(lines.clone());
                (Some(Authored(lines)), Ok(recorded))
            }
            Err(why) => (None, Err(why)),
        }
    })
}

fn authored<C: Command>(
    held: &ActSet,
    tip: Option<Tip>,
    command: &C,
    a: &Author<'_>,
) -> Result<Vec<Op<SignedOp>>, Refused<C::Refusal>> {
    let actor = actor_of(a.signer);
    let before = admit(&read::<C>(held), &[], a.roster, a.namespace, a.verifier);
    settled(&before, a.namespace)?;
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
    let lines = sign_lines(held, tip, acts, a);
    let mut with = held.clone();
    for line in &lines {
        with.insert(line.clone());
    }
    let after = admit(&read::<C>(&with), &[], a.roster, a.namespace, a.verifier);
    settled(&after, a.namespace)?;
    let ids: Vec<OpId> = lines.iter().map(|l| l.id.clone()).collect();
    command.confirm(&after, &ids).map_err(Refused::Command)?;
    Ok(lines)
}

/// Refuses a fold whose cuts did not settle (#124), said at warn.
fn settled<R>(fold: &Admission, ns: &str) -> Result<(), Refused<R>> {
    if !fold.membership().unsettled {
        return Ok(());
    }
    tracing::warn!(target: "rail:door", ns, "commit refused — the group's removals do not come to one answer, so nothing is judged on them");
    Err(Refused::Unsettled)
}

fn read<C: Command>(held: &ActSet) -> ActSet {
    match C::READS {
        Reads::Membership => {
            let all: Vec<Op<SignedOp>> = held.iter().cloned().collect();
            ActSet::of(crate::standing_reads(&all))
        }
        Reads::All => held.clone(),
    }
}

/// How far past the writer's clock a line held may be timed and still be
/// followed. A clock this far out is wrong, not skewed — Kerberos refuses a
/// peer's past five minutes by default (MIT krb5's `clockskew`, 300 s) —
/// and following one carried every later act past a link's day (#188).
pub const AHEAD_SECS: i64 = 300;

/// Signs `acts` as the signer's next lines: after its newest seq, naming the
/// line there its signature verifies, and timed strictly after every line
/// held that its signer signed and that is timed at most [`AHEAD_SECS`]
/// past the writer's clock, since the rail orders by `(ts_unix, actor, seq,
/// id)`. A line nobody signed, or one timed further ahead, is not followed:
/// either would carry every act after it (#188).
/// They carry no view (arch-view): what the writer held was stamped on every
/// act and read only by the signature over it, 39% of a line at 2 members
/// and 88% at 25 (`demo/runs/view/2026-10-15/`); a cut carries its own
/// `through`, and a fork is named by the chain.
fn sign_lines(
    held: &ActSet,
    tip: Option<Tip>,
    acts: Vec<RailAct>,
    a: &Author<'_>,
) -> Vec<Op<SignedOp>> {
    let actor = actor_of(a.signer);
    let first_seq = tip.as_ref().map_or(0, |t| t.seq + 1);
    let mut prev = tip.and_then(|t| {
        t.lines
            .iter()
            .find(|(id, sig)| a.verifier.verify(&actor, id, sig))
            .or(t.lines.first())
            .map(|(id, _)| id.clone())
    });
    let reach = a.now.saturating_add(AHEAD_SECS);
    let signed = |op: &Op<SignedOp>| {
        a.verifier
            .verify(&op.actor, &derive_id(op, a.namespace), &op.kind.sig)
    };
    let (followed, passed): (Vec<_>, Vec<_>) = (held.iter().filter(|op| op.ts_unix >= a.now))
        .partition(|op| op.ts_unix <= reach && signed(op));
    let after = followed
        .iter()
        .map(|op| op.ts_unix.saturating_add(1))
        .max()
        .unwrap_or(i64::MIN);
    let ts_unix = a.now.max(after);
    tracing::debug!(target: "rail:door", ns = a.namespace, %actor, first_seq, ?prev,
        now = a.now, after, ts_unix, passed = passed.len(),
        "commit: follows the signer's newest act, timed after every act held its signer signed and at most AHEAD_SECS on, signing no view; `passed` lines ahead of the clock are not followed");
    let mut lines = Vec::with_capacity(acts.len());
    for (seq, act) in (first_seq..).zip(acts) {
        let op = sign_act(
            a.signer,
            a.namespace,
            ts_unix,
            seq,
            prev.take(),
            act,
            None,
            None,
            None,
        );
        prev = Some(op.id.clone());
        lines.push(op);
    }
    lines
}
