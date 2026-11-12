// SPDX-License-Identifier: AGPL-3.0-or-later
//! The write side. A command is decided over the group's admitted acts, under
//! the store's write lock, and [`commit`] is the one way lines reach a store:
//! a [`Store`] appends only [`Authored`] lines, which only `commit` makes.

use alloc::string::String;
use alloc::vec::Vec;

use crate::{actor_of, RingVerifier, Roster, SigningKey};
use crate::{admit_after, derive_id, sign_act, ActSet, Admission, Op, OpId, RailAct, SignedOp};

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
    pub now: i64,
}

/// What a store hands a commit under its write lock: the lines it holds,
/// the signer's [`Tip`], the verifier that judges them (a store may vouch
/// for each line it checked as it entered), and the fold it keeps of
/// exactly those lines with the seed it was folded under, if it keeps one.
pub struct Held<'a> {
    pub acts: &'a ActSet,
    pub tip: Option<Tip>,
    pub verifier: &'a dyn RingVerifier,
    pub kept: Option<(&'a Roster, &'a Admission)>,
}

/// Why a commit wrote nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused<R> {
    /// The signer holds no standing in the group.
    NoStanding,
    /// The group's cuts do not settle, before the acts or after them
    /// (#124): nothing is judged on that, so nothing is written.
    Unsettled,
    /// A sources-v1 group's interpreter would not count a line the command
    /// asks for, so nothing is written: the write side is the replay.
    Uncounted(Uncounted),
    Command(R),
}

/// Signed lines only [`commit`] makes: the one thing a [`Store`] may append.
///
/// ```
/// fn append(lines: &svrngs_core::Authored) -> usize { lines.lines().len() }
/// ```
/// ```compile_fail
/// let lines = svrngs_core::Authored(Vec::new(), None);
/// ```
pub struct Authored(Vec<Op<SignedOp>>, Option<(Roster, Admission)>);

impl Authored {
    pub fn lines(&self) -> &[Op<SignedOp>] {
        &self.0
    }

    /// A sources-v1 group's fold with these lines in it, for a store to
    /// keep as the fold of what it now holds: one fold per write (P8).
    pub fn after(self) -> Option<(Roster, Admission)> {
        self.1
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

    /// Under the write lock: hands `f` what it holds for `actor` ([`Held`]),
    /// appends what it authored, and returns its answer only once the append
    /// succeeded.
    fn append_with<R>(
        &self,
        actor: &str,
        f: impl FnOnce(Held<'_>) -> (Option<Authored>, R),
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
    store.append_with(&actor, |held| match authored(held, command, author) {
        Ok(authored) => {
            let recorded = Recorded(authored.0.clone());
            (Some(authored), Ok(recorded))
        }
        Err(why) => (None, Err(why)),
    })
}

fn authored<C: Command>(
    held: Held<'_>,
    command: &C,
    a: &Author<'_>,
) -> Result<Authored, Refused<C::Refusal>> {
    let actor = actor_of(a.signer);
    let fold = |acts: &ActSet, known: Option<&OpId>| {
        let read = read::<C>(acts, a.roster);
        admit_after(&read, &[], a.roster, a.namespace, held.verifier, known)
    };
    // The interpreter reads every line, so the store's kept fold of what it
    // holds, which a store keeps only then, is this decision's, and the
    // write folds once (P8).
    let kept = held.kept.filter(|(seed, _)| *seed == a.roster);
    let before = kept.map_or_else(|| fold(held.acts, None), |(_, kept)| kept.clone());
    tracing::debug!(target: "rail:door", ns = a.namespace, %actor, kept = kept.is_some(), "commit: the fold decided over, the store's kept one when it keeps one");
    settled(&before, a.namespace)?;
    if !before.membership().standing.contains(&actor) {
        tracing::debug!(target: "rail:door", ns = a.namespace, %actor,
            "commit refused — the signer holds no standing in the group");
        return Err(Refused::NoStanding);
    }
    let acts = command.decide(&before).map_err(Refused::Command)?;
    // A held Adopt this copy lacks closure lines of decides nothing yet, so
    // no membership act is written over it (EUDOXIA_JUDGE §2e rule 7).
    let waits =
        (acts.iter().any(RailAct::moves_membership)).then_some(&before.membership().decided);
    if let Some(crate::Decided::Waiting(adopt)) = waits {
        tracing::debug!(target: "rail:door", ns = a.namespace, %actor, %adopt, "commit refused — the group's Adopt waits on lines this copy lacks");
        let sources = [crate::authority::AuthorityRef::Act(adopt.clone())].into();
        let why = Some(crate::authority::SourceFailure::Missing { sources });
        return Err(Refused::Uncounted(Uncounted {
            why,
            ..Default::default()
        }));
    }
    if acts.is_empty() {
        tracing::debug!(target: "rail:door", ns = a.namespace, %actor,
            "commit: the decision asks for nothing, so nothing is written");
        return Ok(Authored(Vec::new(), None));
    }
    let lines = sign_lines(held.acts, held.tip, acts, a, held.verifier);
    let mut with = held.acts.clone();
    for line in &lines {
        with.insert(line.clone());
    }
    let after = fold(&with, before.membership().decided.adopted());
    settled(&after, a.namespace)?;
    let ids: Vec<OpId> = lines.iter().map(|l| l.id.clone()).collect();
    command.confirm(&after, &ids).map_err(Refused::Command)?;
    // Write side equals replay (EUDOXIA_JUDGE §2): past the command's own
    // refusals, the group's interpreter says whether each line would count,
    // never a rule here.
    let v1 = after.membership().decided.interpreted();
    if let Some(id) = ids.iter().find(|id| v1 && !after.membership().counts(id)) {
        let why = after.membership().uncounted.get(id).cloned();
        tracing::debug!(target: "rail:door", ns = a.namespace, %actor, %id, ?why,
            "commit refused — the group's interpreter would not count the line");
        let line = lines.iter().find(|l| l.id == *id).expect("written");
        return Err(Refused::Uncounted(Uncounted::of(&after, line, why)));
    }
    Ok(Authored(lines, v1.then(|| (a.roster.clone(), after))))
}

/// Refuses a fold whose cuts did not settle (#124), said at warn.
fn settled<R>(fold: &Admission, ns: &str) -> Result<(), Refused<R>> {
    if !fold.membership().unsettled {
        return Ok(());
    }
    tracing::warn!(target: "rail:door", ns, "commit refused — the group's removals do not come to one answer, so nothing is judged on them");
    Err(Refused::Unsettled)
}

fn read<C: Command>(held: &ActSet, seed: &Roster) -> ActSet {
    match C::READS {
        Reads::Membership => {
            let all: Vec<Op<SignedOp>> = held.iter().cloned().collect();
            ActSet::of(crate::standing_reads(&all, seed))
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
    verifier: &dyn RingVerifier,
) -> Vec<Op<SignedOp>> {
    let actor = actor_of(a.signer);
    let first_seq = tip.as_ref().map_or(0, |t| t.seq + 1);
    let mut prev = tip.and_then(|t| {
        t.lines
            .iter()
            .find(|(id, sig)| verifier.verify(&actor, id, sig))
            .or(t.lines.first())
            .map(|(id, _)| id.clone())
    });
    let reach = a.now.saturating_add(AHEAD_SECS);
    let signed =
        |op: &Op<SignedOp>| verifier.verify(&op.actor, &derive_id(op, a.namespace), &op.kind.sig);
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

/// Why a sources-v1 group's interpreter would not count a line a command
/// asked for, with whose line it was and, for a correction, whose entry it
/// names. Its `Display` is the one sentence a person reads: what stopped,
/// why, and what to do.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Uncounted {
    pub why: Option<crate::authority::SourceFailure>,
    /// Who wrote the line, and whose entry a correction names, as named;
    /// "someone" when the record names nobody.
    pub by: String,
    pub owner: String,
}

impl Uncounted {
    fn of(
        after: &Admission,
        line: &Op<SignedOp>,
        why: Option<crate::authority::SourceFailure>,
    ) -> Self {
        let m = after.membership();
        let person = |key: &str| {
            m.person_of(key)
                .map_or("someone".into(), |(_, p)| p.as_str().into())
        };
        let target = match &line.kind.act {
            RailAct::Correct { corrects, .. } => after.ops().iter().find(|o| o.id == *corrects),
            _ => None,
        };
        let owner = target.map_or("someone".into(), |t| person(&t.actor));
        let by = person(line.stands_as());
        Self { why, by, owner }
    }
}

impl core::fmt::Display for Uncounted {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use crate::authority::{SourceFailure as Why, SourceFault as Fault};
        let (owner, by) = (&self.owner, &self.by);
        match &self.why {
            Some(Why::Refused { why: Fault::Correction }) => write!(f, "Only {owner} can change {owner}’s entry, so the record would not count {by}’s change. Ask {owner} to change it."),
            Some(Why::Refused { why: Fault::Office | Fault::Stale }) => f.write_str("Only whoever keeps the order of removals can do that, so the record would not count it. Ask them to."),
            Some(Why::Frozen { .. }) => f.write_str("Who keeps the order of removals is in dispute, so the record would not count this until the others agree who does."),
            Some(Why::Missing { .. }) => f.write_str("This computer lacks part of the group’s record this rests on, so it would not count yet. Try again once it has synced."),
            _ => f.write_str("The group’s record would not count this, so nothing changed."),
        }
    }
}
