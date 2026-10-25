// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/admit.rs@e1a43bcd2
//! [`admit`] — turn a bag of journal lines into the one order every node
//! applies them in, and say what is missing.
//!
//! # The cut this module is one half of
//!
//! There were thirteen ways a ring op could fail to count, and they were one
//! enum. Eight of them — a torn line, a bad signature, a stranger's key, a
//! rewritten id, a hole in a sequence, an equivocated sequence number, a
//! correction pointing at nothing, a line from a newer build — are about
//! **delivery and authenticity**, and not one of them knows what an expense
//! is. The other five were about money.
//!
//! That was one type doing two jobs, and the tell was that the money half
//! could not move: an app that lends drills instead of splitting groceries
//! needs every rule in this file and none of the other five. So the rail
//! keeps this half, in Rust, where the signature checking and the convergence
//! property live; the app keeps its own half, in its own vocabulary, over the
//! payloads this function hands back.
//!
//! # The one property this file exists to have
//!
//! **The result is a function of the op SET.** Not of arrival order, not of
//! file position, not of the transport's `timestamp`. Nineteen laptops gossip
//! the same ops in nineteen different orders; if admission depended on order,
//! two housemates would read different numbers off the same journal and the
//! whole thing would be worse than the spreadsheet it replaces.
//!
//! Everything below that looks fussy is in service of that one property:
//!
//! | Rule | Without it |
//! |---|---|
//! | dedupe by re-derived [`OpId`] | a replayed op is counted twice |
//! | total order `(ts_unix, actor, seq, id)` | tie-breaking differs per node, and one actor's own burst inside a second folds against its causality |
//! | void set built from ALL corrections at once | a correction that arrives before its target does nothing on one node and something on another |
//! | corrections never resurrect | un-voiding depends on which correction is "last" |
//! | gaps sorted before returning | the *report* differs even when the payloads agree |
//!
//! Proving it here rather than over balances makes it a **stronger** claim
//! and a cheaper one: the exhaustive permutation test no longer has to pick a
//! tenant to be true about.
//!
//! # Why the void set is in the rail and not in the app
//!
//! "This earlier act was wrong, and it never comes back" is not an expense
//! rule. A tool-lending board needs it the moment somebody writes *I returned
//! the drill* and then *no I didn't*. Leaving it to the app means every
//! author re-derives the one rule that makes the void set commutative — build
//! it from every correction at once, never walk for liveness — and the ones
//! who get it wrong get an app that converges in testing and diverges in a
//! house.
//!
//! # And the property it refuses to fake
//!
//! An admission over ops that have not all arrived is an admission over a
//! subset. It returns [`RailGap`]s rather than a bare list, because a rail
//! that cannot say "I may be missing something" lets an app state a wrong
//! total with complete confidence, which is the failure ARCH §18.3 names.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::op::{Op, OpId};
use crate::sig::derive_id;

use crate::payload::Payload;
use crate::{Person, RailAct, RingVerifier, Roster, SignedOp};

/// Something the rail could not account for. Never fatal, always reported.
///
/// Every variant is about **delivery or authenticity**. Nothing here reads
/// inside a payload, which is why this enum is the same eight cases for an
/// expense book and a tool-lending board. An app's own reasons for refusing
/// an act are the app's to name, over the payloads [`admit`] returns.
///
/// Ordered and sorted before admission returns, so two nodes holding the same
/// ops produce a byte-identical report and not merely an equal set of acts.
///
/// Its `Display` is one sentence a person can act on: **the one renderer**
/// (ARCH §10.6). A gap is shown in three places — the `svrn ring log`
/// table, a ring app's own page, and the refusal the append door returns —
/// and each writing its own prose is how three of them end up saying
/// different things about the same condition. Before this existed the door
/// returned a serde dump (`{"gap":"bad_signature",…}`) straight at a
/// housemate.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    thiserror::Error,
)]
#[serde(tag = "gap", rename_all = "snake_case")]
pub enum RailGap {
    /// A journal line this build could not parse. From
    /// [`SkippedLine::Malformed`](crate::op::SkippedLine) — a torn
    /// write, or a payload that has no canonical form (see
    /// [`Payload`](crate::Payload)).
    #[error("journal line {line} could not be read")]
    MalformedLine { line: u64, error: String },
    /// A journal line written by a NEWER build. Reading it would be guessing;
    /// counting the rest and saying nothing would be worse (ARCH §18.3), so
    /// an un-upgraded node reports that its answer covers a strict subset.
    #[error("journal line {line} was written by a newer build — upgrade to read it")]
    NewerVersionLine { line: u64, v: u32 },
    /// The signature does not verify under the public key the line names.
    /// The op is not admitted.
    #[error("an op whose signature does not verify ({})", crate::short_id(&.id.to_string()))]
    BadSignature { id: OpId, actor: String },
    /// The signature verifies, but no one in the roster signs with that key.
    /// Self-certifying is not the same as being a member. Not admitted.
    #[error("an op signed by {}… — nobody in the roster claims that key", crate::actor_prefix(.actor))]
    UnknownSigner { id: OpId, actor: String },
    /// The `id` on the line is not the id its content derives. Admission uses
    /// the derived id, so this changes no outcome — it is reported because a
    /// peer writing lines whose id has been rewritten is a fact worth seeing.
    #[error("a journal line whose id ({}) does not match its content", crate::short_id(&.claimed.to_string()))]
    TamperedId { claimed: OpId, derived: OpId },
    /// This actor's ops jump over sequence numbers, `from` to `to` (both
    /// missing). Something they wrote has not reached us — the one condition
    /// that distinguishes "nothing happened" from "it never arrived."
    /// Counted from zero: a [`Seal`](crate::RailAct::Seal) retires nothing
    /// (arch-seal). One gap a run, not a seq: a seq is what a line claims,
    /// and a gap each let one line far past its chain cost every copy that
    /// many (#187).
    #[error("{}", hole(.actor, .from, .to))]
    SequenceHole { actor: String, from: u64, to: u64 },
    /// One actor used one sequence number for two different ops. Equivocation
    /// or a lost counter after a restart. Both count, each weighed as itself:
    /// forks void nothing (svrngs, U7; DESIGN §6), since excluding both let an
    /// author take back their own act on every copy by signing a second.
    #[error("{}… used one sequence number twice (#{seq}) — both acts count", crate::actor_prefix(.actor))]
    SequenceFork {
        actor: String,
        seq: u64,
        ids: Vec<OpId>,
    },
    /// A correction naming an op we do not hold. The correction itself folds;
    /// its citation resolves to nothing here. Harmless to the order (the void
    /// is recorded and applies the moment the target arrives), and it hides
    /// no act from the fold — if the target is real and missing, its author's
    /// run reports the [`SequenceHole`], and if that author is wholly absent
    /// this gap names the exact op to ask for. It is the ask, not silence,
    /// and a typo'd citation must not flip every node's completeness bit
    /// forever (ROOT_CAUSE_FIXES A2).
    #[error("a correction of {}, which this node does not hold", crate::short_id(&.missing.to_string()))]
    DanglingCorrection { by: OpId, missing: OpId },
    /// The signer is bound in the record but held no standing at this act's
    /// position — before their own [`Admit`](crate::RailAct::Admit) in the
    /// order (an act of a key the keeper cut is [`Self::Removed`]). The act
    /// counts for nothing and is named rather than dropped: it may be real
    /// data under an `Admit` this copy does not hold yet.
    #[error("an op by actor {actor} was written while they held no standing in the ring")]
    NotAMember { id: OpId, actor: String },
    /// The signer was cut by the group's keeper, and the cut does not cover
    /// this act (svrngs, U7; DESIGN §6): written after the removal, or
    /// beside or above the acts the removal names. Nothing undoes a cut, so
    /// the act is refused for good.
    #[error("an act by {}… after they were removed from the group does not count", crate::actor_prefix(.actor))]
    Removed { id: OpId, actor: String },
    /// A counting cut names an act of the removed key this copy does not
    /// hold, so the acts it covers cannot all be told yet (U7).
    #[error("a removal names an act ({}) this node does not hold yet", crate::short_id(&.missing.to_string()))]
    UnheldCut { by: OpId, missing: OpId },
    /// The act's `prev` is not the id of its author's act at `seq − 1`: it
    /// names a parent at seq 0, names none above it, or names one other than
    /// the act this copy holds there (svrngs, U2; DESIGN §6). The act is not
    /// admitted — it follows a history this copy does not hold, and may be
    /// real data on another branch. Judged against an act at `seq − 1` only
    /// when one is held; a parent not yet here is the [`Self::SequenceHole`].
    #[error("an act from {}… does not follow the one before it (#{seq})", crate::actor_prefix(.actor))]
    WrongPrevious { id: OpId, actor: String, seq: u64 },
    /// A standing member's [`Admit`](crate::RailAct::Admit) that redeems an
    /// [`Invite`](crate::RailAct::Invite), refused for the named fault
    /// (svrngs, U2; DESIGN §6 "A link admits each person once"). It gives
    /// its key no standing; it may be real data under an `Invite` this copy
    /// does not yet hold.
    #[error("a joining by link admitted by {}… does not count — {fault}", crate::actor_prefix(.actor))]
    RefusedRedemption {
        id: OpId,
        actor: String,
        fault: crate::membership::InviteFault,
    },
    /// The keeper's cuts did not settle to one answer (#124): who stands,
    /// and so which acts count, cannot be judged on this copy.
    #[error("the removals do not come to one answer, so who is in cannot be told")]
    Unsettled,
    /// A holder's line its pass does not grant (capability-signed; DESIGN
    /// §1): written outside its app's own part, after its pass lapsed, as a
    /// decision, or under a pass its person did not sign or that names
    /// another key. It counts as nobody's act.
    #[error("an entry an app wrote as {}… does not count — {why}", crate::actor_prefix(.actor))]
    Ungranted {
        id: OpId,
        actor: String,
        why: crate::grant::Why,
    },
    /// An act naming another person than its signer's, `named`, written
    /// past what the keeper's cutover covers (capability-signed; DESIGN
    /// §1): a computer's word for someone else, which counts as nobody's.
    #[error("an entry {}… wrote as {named} does not count — only their own phone or computer writes as them", crate::actor_prefix(.actor))]
    CarriedForAnother {
        id: OpId,
        actor: String,
        named: String,
    },
}

/// A [`RailGap::SequenceHole`]'s words: one act missing, or a run of them.
fn hole(actor: &str, from: &u64, to: &u64) -> String {
    let who = crate::actor_prefix(actor);
    match from == to {
        true => alloc::format!("an op from {who}… has not reached this node yet (#{from})"),
        false => {
            alloc::format!("ops from {who}… have not reached this node yet (#{from} to #{to})")
        }
    }
}

/// What a gap means for the fold's answer — the one classification (ARCH
/// §10.6), asked the question `is_complete` exists to answer: **can this gap
/// be hiding acts that belong in the fold?**
///
/// The line matters more than it looks: a refusal is a refusal, never an
/// absence (ARCH §18.3), and an absence is never silence — but "the fold is
/// missing data" and "the record contradicts itself" are different facts and
/// one typo'd [`Correct`](crate::RailAct::Correct) used to masquerade as the
/// former forever. The split is by coverage, not by scariness: a forged act
/// is Absence precisely because it may be REAL data the fold refused (the
/// acts are real but they are a subset), while a tampered id, a dangling
/// citation or a fork is Contradiction because every act that could be
/// folded is folded — the gap is a fact about a claim the record makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapClass {
    /// The fold may lack acts that belong in it — an unreadable line, a
    /// refused or unattributable act (which may be real), a seq that never
    /// arrived. [`Admission::is_complete`] is false exactly
    /// on these.
    Absence,
    /// Every act that could be folded is folded: the gap is a claim the
    /// record makes that nothing here can stand behind — an id its content
    /// does not derive, a correction citing an op nobody holds, or two acts
    /// at one seq, both folded (U7). Named beside a complete answer.
    Contradiction,
}

impl RailGap {
    /// The one classification of what a gap means (ARCH §10.6). Consumers
    /// ask their own question of it — completeness is `class() ==
    /// Absence`; `ring checkpoint --verify` refuses a document on the gaps
    /// whose acts cannot stand — but there is one answer per kind here.
    pub fn class(&self) -> GapClass {
        match self {
            Self::MalformedLine { .. }
            | Self::NewerVersionLine { .. }
            | Self::BadSignature { .. }
            | Self::UnknownSigner { .. }
            | Self::NotAMember { .. }
            | Self::UnheldCut { .. }
            | Self::WrongPrevious { .. }
            | Self::RefusedRedemption { .. }
            | Self::Unsettled
            | Self::Ungranted {
                why: crate::grant::Why::Unheld,
                ..
            }
            | Self::SequenceHole { .. } => GapClass::Absence,
            Self::TamperedId { .. }
            | Self::DanglingCorrection { .. }
            | Self::SequenceFork { .. }
            | Self::Ungranted { .. }
            | Self::CarriedForAnother { .. }
            | Self::Removed { .. } => GapClass::Contradiction,
        }
    }
}

/// One op that passed admission, ready for an app's reducer.
///
/// The rail has already done everything generic to it: the id is re-derived
/// from content, the signature verified, the signer looked up in the roster,
/// and the position in `Admission::ops` is the total order every node agrees
/// on. What is left — what the payload *means* — is the app's.
/// `Deserialize` as well as `Serialize` since cw-lift 5d, and the pair is the
/// point: `GET /v1/rail/log` ships `Admission::ops` verbatim, and a client that
/// wants to FOLD that answer — `svrn job status` is the first — has to be able
/// to read back exactly what this type wrote. The alternative was a second
/// struct in the CLI mirroring these fields, which is one wire shape with two
/// spellings and drifts the day a field is added here (ARCH §10.6).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdmittedOp {
    /// Content-derived id. This is what a correction names, and what an app
    /// hands back to [`RailAct::Correct`](crate::RailAct::Correct).
    pub id: OpId,
    /// The signing public key — the only field on the line a writer cannot
    /// forge for someone else (ARCH §18.1).
    pub actor: String,
    /// Who the roster says that key is. Present because admission already
    /// refused every key the roster does not know, so an app never has to
    /// render a balance against `node-44a1b3e8`.
    pub person: Person,
    pub seq: u64,
    pub ts_unix: i64,
    /// What this op voids, when it is a correction. The void is **already
    /// applied** — carried so an app can say *what changed*, never so it can
    /// re-derive the void set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corrects: Option<OpId>,
    /// `true` when a correction voided this op. It stays in the list so an
    /// app can render history, and the SDK's `fold` skips it. An app that
    /// walks `ops` itself instead of folding will double-count these — which
    /// is why the SDK ships the fold.
    pub voided: bool,
    /// The app's act. `None` is a correction that only voids, and states no
    /// replacement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Payload>,
    /// [`SignedOp::on_behalf_of`](crate::SignedOp::on_behalf_of), carried
    /// through unchanged. Carried and not interpreted: admission does not
    /// look it up, does not refuse on it, and `person` still names the key's
    /// owner. It is here because `admit` is the only place an `AdmittedOp` is
    /// built, so a reader that never sees the signed line — the log route,
    /// and every app behind it — could not otherwise reach the field at all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<String>,
}

impl AdmittedOp {
    /// Whether an app's reducer should see this op. The one definition of
    /// "surviving", so the SDK's fold and any Rust caller agree.
    pub fn applies(&self) -> bool {
        !self.voided && self.payload.is_some()
    }
}

/// What admission produced: the acts in their agreed order, and the honest
/// account of what could not be read. Only [`admit`] makes one.
///
/// ```
/// fn read(a: &svrngs_core::Admission) -> usize { a.held() }
/// ```
/// ```compile_fail
/// let a = svrngs_core::Admission { ..unimplemented!() };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admission {
    ops: Vec<AdmittedOp>,
    gaps: Vec<RailGap>,
    held: usize,
    membership: crate::membership::Membership,
    /// Every synced folder's change, in admission's order (sy-1).
    pub(crate) synced: crate::synced::Written,
    /// Each actor's newest acts held: chained, and named as `prev` by no
    /// other act of theirs. What a removal names (U7).
    tips: BTreeMap<String, Vec<OpId>>,
}

impl Admission {
    /// Every admitted op in the total order `(ts_unix, actor, seq, id)`,
    /// voided ones included and marked.
    pub(crate) fn ops(&self) -> &[AdmittedOp] {
        &self.ops
    }

    /// Everything the rail could not account for, sorted and deduplicated.
    pub fn gaps(&self) -> &[RailGap] {
        &self.gaps
    }

    /// The distinct lines admitted from; `held() - ops().len()` were refused.
    pub fn held(&self) -> usize {
        self.held
    }

    /// Who is in: the one membership walk admission made.
    ///
    /// ```
    /// fn standing(a: &svrngs_core::Admission) -> usize {
    ///     a.membership().standing.len()
    /// }
    /// ```
    pub fn membership(&self) -> &crate::membership::Membership {
        &self.membership
    }

    /// `actor`'s newest acts this copy holds, by id (U7).
    pub(crate) fn tips(&self, actor: &str) -> &[OpId] {
        self.tips.get(actor).map_or(&[], Vec::as_slice)
    }
}

impl Admission {
    /// Whether this answer covers every act that could be in it. `false` is
    /// the signal a UI must not hide: the acts are real but they are a
    /// subset — a gap of [`GapClass::Absence`] can be hiding acts that
    /// belong in the fold. [`GapClass::Contradiction`] gaps do not flip it:
    /// they are named beside a complete answer (a tampered id changes no
    /// outcome; a dangling citation is the ask, not a hole).
    pub fn is_complete(&self) -> bool {
        !self.gaps.iter().any(|g| g.class() == GapClass::Absence)
    }
}

// ── admission ────────────────────────────────────────────────

/// One op that passed the signature and roster checks, with the id the rail
/// actually uses.
struct Candidate<'a> {
    id: OpId,
    person: Person,
    op: &'a Op<SignedOp>,
}

/// Turn a bag of journal lines into the order every node applies them in.
///
/// `roster`, `namespace` and `verifier` are *parameters*, never folded state.
/// The namespace is bound into every signature, so it decides admission; the
/// verifier says whether a signature is real; the roster says which keys are
/// members. None of the three is ever handed to the app's reducer as anything
/// but the `person` on an already-admitted op — which is what keeps an app's
/// arithmetic a function of the op set even as people join and leave (pinned
/// by a test in the reference app).
///
/// The verifier is named rather than defaulted: which scheme judged these
/// signatures is a fact about the answer, and one that has to be greppable at
/// every call site instead of inherited from whatever the fold happened to be
/// compiled with. [`Ed25519Verifier`](crate::Ed25519Verifier) is the shipped
/// one.
pub fn admit(
    acts: &crate::ActSet,
    skipped: &[crate::op::SkippedLine],
    roster: &Roster,
    namespace: &str,
    verifier: &dyn RingVerifier,
) -> Admission {
    use crate::op::SkippedLine;

    let mut gaps: Vec<RailGap> = skipped
        .iter()
        .map(|s| match s {
            SkippedLine::Malformed { line, error } => RailGap::MalformedLine {
                line: *line,
                error: error.clone(),
            },
            SkippedLine::NewerVersion { line, v } => {
                RailGap::NewerVersionLine { line: *line, v: *v }
            }
        })
        .collect();

    // ── signature ───────────────────────────────────────────
    let mut authentic: BTreeMap<OpId, &Op<SignedOp>> = BTreeMap::new();
    for op in acts.iter() {
        let derived = derive_id(op, namespace);
        if derived != op.id {
            gaps.push(RailGap::TamperedId {
                claimed: op.id.clone(),
                derived: derived.clone(),
            });
        }
        // Whatever the verifier cannot vouch for is a gap, never an act. A
        // `false` here is a refusal and is reported as one — there is no
        // answer that means "could not tell, carry on" (ARCH §18.3). The
        // signature is over the DERIVED id (svrngs, U2), so it covers every
        // field the id does.
        if !verifier.verify(&op.actor, &derived, &op.kind.sig) {
            gaps.push(RailGap::BadSignature {
                id: derived,
                actor: op.actor.clone(),
            });
            continue;
        }
        // Dedupe: the same op reaching us twice is the normal case under
        // gossip, not an anomaly — and so is one act carrying two valid
        // signatures, which is one id.
        authentic.insert(derived, op);
    }

    // ── a holder: what the pass on its line grants (capability-signed) ──
    //
    // Judged before the chain and the walk, so a line its pass does not
    // grant is nobody's act, and the walk counts a granted one under its
    // pass's signer's standing ([`Op::stands_as`]). Each distinct pass is
    // verified once, however many lines carry it.
    let mut signed: BTreeMap<&crate::pass::Pass, bool> = BTreeMap::new();
    let mut ungranted = Vec::new();
    for (id, op) in &authentic {
        use crate::grant::{Capability::*, Grant, Why};
        let Some(pass) = &op.kind.grant else {
            continue;
        };
        let app = |id| (authentic.get(id)).map(|t| t.kind.act.payload().map_or("", Payload::app));
        let may = |cap| Grant(pass, namespace).may(&op.actor, cap, op.ts_unix);
        let verdict = match &op.kind.act {
            _ if !*signed.entry(pass).or_insert_with(|| pass.signed()) => Err(Why::Unsigned),
            RailAct::Record { payload } => may(Append(payload.app())),
            RailAct::Correct {
                corrects,
                replacement,
            } => match app(corrects) {
                Some(k) if replacement.as_ref().is_none_or(|r| r.app() == k) => may(Append(k)),
                held => Err(held.map_or(Why::Unheld, |_| Why::OtherKind)),
            },
            _ => may(Decide),
        };
        if let Err(why) = verdict {
            tracing::debug!(target: "rail:admit", id = %id, actor = %op.actor, person = %pass.person, %why,
                "admit refused — a holder's line its pass does not grant");
            ungranted.push((id.clone(), op.actor.clone(), why));
        }
    }
    for (id, actor, why) in ungranted {
        authentic.remove(&id);
        gaps.push(RailGap::Ungranted { id, actor, why });
    }

    // ── the chain: each act names its author's act at seq − 1 ─────────
    //
    // svrngs (U2, DESIGN §6). Judged over every authentic act at once, before
    // membership, so an act on a history this copy does not hold grants
    // nothing — and against the parent's DERIVED id, so the verdict is a
    // function of the set like everything else here. A parent not held is
    // not judged: its absence is the `SequenceHole` below. Two acts held at
    // `seq − 1` are a fork, and an act naming either follows a parent this
    // copy holds (forks void nothing, U7: the audit below names one).
    let mut at: BTreeMap<(&str, u64), BTreeSet<&OpId>> = BTreeMap::new();
    for (id, op) in &authentic {
        at.entry((op.actor.as_str(), op.kind.seq))
            .or_default()
            .insert(id);
    }
    let mut broken: Vec<OpId> = Vec::new();
    for (id, op) in &authentic {
        let seq = op.kind.seq;
        let follows = match (seq.checked_sub(1), op.kind.prev.as_ref()) {
            (None, prev) => prev.is_none(),
            (Some(_), None) => false,
            (Some(parent), Some(prev)) => at
                .get(&(op.actor.as_str(), parent))
                .is_none_or(|held| held.contains(prev)),
        };
        if !follows {
            tracing::debug!(
                target: "rail:admit",
                id = %id,
                actor = %op.actor,
                seq,
                prev = ?op.kind.prev,
                "admit refused — the act does not name its author's act at seq − 1"
            );
            gaps.push(RailGap::WrongPrevious {
                id: id.clone(),
                actor: op.actor.clone(),
                seq,
            });
            broken.push(id.clone());
        }
    }
    drop(at);
    for id in &broken {
        authentic.remove(id);
    }
    let parents: BTreeSet<(&str, &OpId)> = authentic
        .values()
        .filter_map(|op| Some((op.actor.as_str(), op.kind.prev.as_ref()?)))
        .collect();
    let mut tips: BTreeMap<String, Vec<OpId>> = BTreeMap::new();
    for (id, op) in &authentic {
        if !parents.contains(&(op.actor.as_str(), id)) {
            let key = op.stands_as().to_string();
            tips.entry(key).or_default().push(id.clone());
        }
    }

    // ── membership: the seed plus two act kinds, one function ─────────
    //
    // `roster` is the SEED (RING_APPLICATIONS.md "Amendment 2026-09-18"):
    // what is in the ring is decided by `membership`, and an act resolves
    // through the bindings its own record carries — so key churn cannot
    // re-flip the past (leg 4 of `ra-membership-is-order-free`).
    let m = crate::membership::membership(
        authentic.iter().map(|(id, op)| (id, *op)),
        roster,
        namespace,
    );

    if m.unsettled {
        gaps.push(RailGap::Unsettled);
    }
    for cut in m.cut.values() {
        gaps.extend(cut.unheld.iter().map(|missing| RailGap::UnheldCut {
            by: cut.by.clone(),
            missing: missing.clone(),
        }));
    }
    // ── per-actor sequence audit, over every act of a bound signer ───
    //
    // Counted, refused or removed alike: an act refused inside its author's
    // chain is held, so it is reported as what it is and never also as a
    // hole (U7; DESIGN §6). Before, the audit counted admitted acts only.
    let mut seqs: BTreeMap<&str, BTreeMap<u64, Vec<OpId>>> = BTreeMap::new();
    for (id, op) in authentic
        .iter()
        .filter(|(_, op)| m.person_of(op.stands_as()).is_some())
    {
        let op: &Op<SignedOp> = op;
        seqs.entry(op.actor.as_str())
            .or_default()
            .entry(op.kind.seq)
            .or_default()
            .push(id.clone());
    }
    // ── the keeper's cutover (capability-signed; DESIGN §1) ──────────
    //
    // An act naming another person than its signer's counts only where the
    // keeper's `Signers` covers it, by ancestry as a cut covers; until the
    // keeper writes one, as before.
    let cover = m.signers.as_ref().map(|by| {
        let mut at = match authentic.get(by).map(|op| &op.kind.act) {
            Some(RailAct::Signers { through }) => through.clone(),
            _ => Vec::new(),
        };
        let mut seen = BTreeSet::new();
        while let Some(id) = at.pop() {
            if let Some(op) = authentic.get(&id).filter(|_| seen.insert(id.clone())) {
                at.extend(op.kind.prev.clone());
            }
        }
        seen
    });
    let mut admitted: BTreeMap<OpId, Candidate<'_>> = BTreeMap::new();
    for (id, op) in authentic {
        let Some((_, person)) = m.person_of(op.stands_as()) else {
            gaps.push(RailGap::UnknownSigner {
                id,
                actor: op.actor.clone(),
            });
            continue;
        };
        if let Some(fault) = m.refused.get(&id) {
            gaps.push(RailGap::RefusedRedemption {
                id,
                actor: op.actor.clone(),
                fault: fault.clone(),
            });
            continue;
        }
        if !m.counted.contains(&id) && !m.voided.contains(&id) {
            let actor = op.actor.clone();
            gaps.push(match m.cut.contains_key(op.stands_as()) {
                true => RailGap::Removed { id, actor },
                false => RailGap::NotAMember { id, actor },
            });
            continue;
        }
        let named = op
            .kind
            .on_behalf_of
            .as_deref()
            .filter(|n| *n != person.as_str());
        let past = cover.as_ref().is_some_and(|c| !c.contains(&id));
        if let Some(named) = named {
            tracing::debug!(target: "rail:admit", id = %id, actor = %op.actor, named, cutover = cover.is_some(), past,
                "admit: an act naming another person counts only where the keeper's cutover covers it, and as before until there is one");
            if past {
                let actor = op.actor.clone();
                gaps.push(RailGap::CarriedForAnother {
                    id,
                    actor,
                    named: named.into(),
                });
                continue;
            }
        }
        admitted.insert(
            id.clone(),
            Candidate {
                id,
                person: person.clone(),
                op,
            },
        );
    }

    // Forks void nothing (svrngs, U7; DESIGN §6): both branches are held and
    // each act is weighed as itself, so no second act at a seq takes back the
    // first. The fork is named beside a complete answer.
    for (actor, by_seq) in &seqs {
        for (seq, ids) in by_seq {
            if ids.len() > 1 {
                let mut ids = ids.clone();
                ids.sort();
                tracing::debug!(target: "rail:admit", actor, seq, ?ids,
                    "admit: a fork — every branch counts");
                gaps.push(RailGap::SequenceFork {
                    actor: (*actor).to_string(),
                    seq: *seq,
                    ids,
                });
            }
        }
    }

    // ── the void set: commutative, and it never resurrects ───
    //
    // The set itself is computed in ONE place — `membership`, which applies
    // it before its walk so a correction that lands late still drops what it
    // targets and everything that target admitted. This pass only NAMES the
    // corrections whose target nobody here holds: the ask, never a hole
    // (GapClass::Contradiction).
    // Cloned, not moved: the whole walk — voided set included — is carried
    // out on `Admission` below, so the membership a reader sees is the one
    // this fold applied.
    let voided = m.voided.clone();
    for a in admitted.values() {
        if let RailAct::Correct { corrects, .. } = &a.op.kind.act {
            if !admitted.contains_key(corrects) {
                gaps.push(RailGap::DanglingCorrection {
                    by: a.id.clone(),
                    missing: corrects.clone(),
                });
            }
        }
    }

    // Holes are audited from zero. A seal is held as an act that does
    // nothing (arch-seal; svrn-docs A_RECORD_FOR_YEARS §6 q2): an older
    // build's seal set a floor that hid the holes below it, and here it
    // hides none, so a copy an older build pruned says what it is missing.
    let seals = admitted
        .values()
        .filter(|c| matches!(c.op.kind.act, RailAct::Seal));
    for c in seals {
        tracing::debug!(namespace, id = %c.id, actor = %crate::actor_prefix(&c.op.actor), seq = c.op.kind.seq,
            "ring rail: a seal is held and retires nothing — an older build's act, read as one that does nothing");
    }
    for (actor, by_seq) in &seqs {
        let mut next = 0u64;
        for &seq in by_seq.keys() {
            if seq > next {
                gaps.push(RailGap::SequenceHole {
                    actor: (*actor).to_string(),
                    from: next,
                    to: seq - 1,
                });
            }
            next = seq.saturating_add(1);
        }
    }

    // ── the content-derived total order ──────────────────────
    //
    // `seq` sits between the actor and the id, and it is not decoration.
    // `ts_unix` is SECOND resolution, so an actor writing twice inside one
    // second used to be ordered by content hash — arbitrary, and arbitrary
    // against its own causality. Observed 2026-09-09 on the work plane: a
    // `Submit` and the `Lease` + `Complete` that answered it landed in the
    // same second, folded lease-then-complete-then-submit, and both of the
    // later acts were reported `unreadable` ("no admitted submission opened
    // this handoff") — so a unit ran TWICE, once for the discarded pair and
    // once for the retry. The rail already holds each actor's `seq` and
    // already refuses a fork on it (`SequenceFork` above), so ordering one
    // actor's own acts by anything else discards information this function
    // has verified (ARCH §7.5 — order from essence, not from an address).
    //
    // It stays DETERMINISTIC: `seq` is on the wire and every node reads the
    // same value, and the id remains the last term so the comparator is total
    // even for a pair the fork check somehow let through. Only within-actor,
    // within-second order changes.
    let mut order: Vec<&Candidate<'_>> = admitted.values().collect();
    order.sort_by(|x, y| {
        (x.op.ts_unix, &x.op.actor, x.op.kind.seq, &x.id).cmp(&(
            y.op.ts_unix,
            &y.op.actor,
            y.op.kind.seq,
            &y.id,
        ))
    });

    let synced = crate::synced::written(order.iter().map(|a| (&a.person, a.op)));
    let out: Vec<AdmittedOp> = order
        .into_iter()
        .map(|a| {
            // Only a record or a correction carries an app's act: a seal
            // does nothing (arch-seal), so `applies()` is false and no
            // reducer sees it, and the register's acts carry meaning to
            // the MEMBERSHIP function and never to an app (no-re-division).
            let corrects = match &a.op.kind.act {
                RailAct::Correct { corrects, .. } => Some(corrects.clone()),
                _ => None,
            };
            let payload = a.op.kind.act.payload().cloned();
            AdmittedOp {
                id: a.id.clone(),
                actor: a.op.actor.clone(),
                person: a.person.clone(),
                seq: a.op.kind.seq,
                ts_unix: a.op.ts_unix,
                corrects,
                voided: voided.contains(&a.id),
                payload,
                on_behalf_of: a.op.kind.on_behalf_of.clone(),
            }
        })
        .collect();

    gaps.sort();
    gaps.dedup();

    let applied = out.iter().filter(|o| o.applies()).count();
    tracing::debug!(
        namespace,
        verifier = verifier.name(),
        held = acts.len(),
        admitted = out.len(),
        applied,
        voided = voided.len(),
        gaps = gaps.len(),
        "ring rail: admission"
    );
    if !gaps.is_empty() {
        // Louder than the summary above on purpose: an admission with gaps
        // covers a subset, and it is the one thing about this answer an
        // operator reading logs needs to see without turning debug on.
        tracing::warn!(
            namespace,
            gaps = gaps.len(),
            first = ?gaps.first(),
            "ring rail: this answer covers a subset — the rail could not account for every op"
        );
    }

    Admission {
        ops: out,
        gaps,
        held: acts.len(),
        membership: m,
        synced,
        tips,
    }
}

/// The body the id covers — the act, in declaration order, with
/// its payload canonical (see [`Payload`](crate::Payload)), followed by
/// `on_behalf_of` when the writer stated one, followed by `view` when the
/// writer committed to one, followed by `grant` when a holder signed it.
///
/// The name is inside the signature because a stamp a peer could strip or
/// rewrite in flight would attribute an act to whoever last handled it. The
/// view is inside for the same reason one field later: a rewritten view
/// would make every act claim whichever history suited the carrier. Both are
/// LAST-in-order and omitted when `None`, so the bytes for an act with
/// neither are byte-identical to what this function returned before the
/// fields existed and every op already on every replica verifies unchanged.
pub fn body_json(
    act: &RailAct,
    on_behalf_of: Option<&str>,
    view: Option<&crate::Digest>,
    grant: Option<&crate::pass::Pass>,
) -> String {
    // A borrowing mirror of `SignedOp`'s signed half rather than a second
    // spelling of the rule: the act flattens in exactly as it serialises
    // alone, and the two added fields sit after it.
    #[derive(serde::Serialize)]
    struct Body<'a> {
        #[serde(flatten)]
        act: &'a RailAct,
        #[serde(skip_serializing_if = "Option::is_none")]
        on_behalf_of: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        view: Option<&'a crate::Digest>,
        #[serde(skip_serializing_if = "Option::is_none")]
        grant: Option<&'a crate::pass::Pass>,
    }
    serde_json::to_string(&Body {
        act,
        on_behalf_of,
        view,
        grant,
    })
    // A `RailAct` failing to serialise is unreachable; silent `""` would be
    // catastrophic — empty bytes on the signature path (ROOT_CAUSE_FIXES C4,
    // substitution-gate's named target).
    .expect("a RailAct serialises — signing empty bytes is the named catastrophe")
}
