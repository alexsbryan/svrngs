// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/tests_support.rs@e1a43bcd2
//! Fixtures shared by the rail's test modules.
//!
//! One ring, one set of builders. Two copies of "what does a signed op look
//! like" would be two answers to the question the signature exists to settle
//! (ARCH §10.6), and the sync tests and the admission tests must be talking
//! about the same op or neither proves anything.
//!
//! The payloads here are deliberately NOT expenses. The rail does not know
//! what an expense is, and a fixture that quietly assumed one would let a
//! money rule creep back into this layer without anyone noticing.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::*;

pub const NS: &str = "house-things";

pub fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// A visit key's consent to be the guest of `host`, a computer's actor hex,
/// as a guest's page signs it.
pub fn consent(guest: &SigningKey, host: &str) -> String {
    let key = crate::actor_of(guest);
    crate::sign_message(guest, &crate::visit_consent_message(host, &key))
}

pub fn p(name: &str) -> Person {
    Person::parse(name).unwrap()
}

/// A ring of three, each signing with one key.
pub fn ring() -> Roster {
    let mut m = BTreeMap::new();
    m.insert(p("alex"), vec![actor_of(&key(1))]);
    m.insert(p("bo"), vec![actor_of(&key(2))]);
    m.insert(p("cy"), vec![actor_of(&key(3))]);
    Roster::new(m)
}

/// An arbitrary well-formed act. `what` is what distinguishes two of them.
pub fn payload(what: &str) -> Payload {
    Payload::new(serde_json::json!({ "kind": "thing", "what": what })).unwrap()
}

pub fn record(what: &str) -> RailAct {
    RailAct::Record {
        payload: payload(what),
    }
}

/// Build the op a node would have written, without going through the journal.
///
/// Its previous act (svrngs, U2) is none at seq 0 and [`absent_parent`]
/// above it: a parent no test holds, which admission does not judge — a
/// parent not held is the `SequenceHole`. A test that holds the parent builds
/// the act with [`after`] or [`run`], or admission names it `WrongPrevious`.
pub fn signed(k: &SigningKey, ts: i64, seq: u64, act: RailAct) -> Op<SignedOp> {
    signed_in(NS, k, ts, seq, act)
}

pub fn signed_in(ns: &str, k: &SigningKey, ts: i64, seq: u64, act: RailAct) -> Op<SignedOp> {
    signed_for(ns, k, ts, seq, act, None)
}

/// [`signed_in`], stating whose words the act was. [`sign_act`] and not a
/// local hash because the signed bytes have one definition (ARCH §10.6) — a
/// fixture that spelled them itself would keep verifying after the real rule
/// changed.
pub fn signed_for(
    ns: &str,
    k: &SigningKey,
    ts: i64,
    seq: u64,
    act: RailAct,
    on_behalf_of: Option<&str>,
) -> Op<SignedOp> {
    let prev = (seq > 0).then(absent_parent);
    sign_act(k, ns, ts, seq, prev, act, on_behalf_of, None, None)
}

/// The previous act [`signed`] names above seq 0: an id no act here derives.
pub fn absent_parent() -> OpId {
    stray_id(0xab)
}

/// The act `k` signs next after `prev`: one seq on, naming `prev`'s id.
pub fn after(k: &SigningKey, prev: &Op<SignedOp>, ts: i64, act: RailAct) -> Op<SignedOp> {
    after_in(NS, k, prev, ts, act)
}

pub fn after_in(
    ns: &str,
    k: &SigningKey,
    prev: &Op<SignedOp>,
    ts: i64,
    act: RailAct,
) -> Op<SignedOp> {
    assert_eq!(prev.actor, actor_of(k), "an act follows its own author's");
    sign_act(
        k,
        ns,
        ts,
        prev.kind.seq + 1,
        Some(prev.id.clone()),
        act,
        None,
        None,
        None,
    )
}

/// `acts` as one author writes them from seq 0, each a second after the
/// last from `ts`, each naming the one before.
pub fn run(k: &SigningKey, ts: i64, acts: impl IntoIterator<Item = RailAct>) -> Vec<Op<SignedOp>> {
    run_in(NS, k, ts, acts)
}

pub fn run_in(
    ns: &str,
    k: &SigningKey,
    ts: i64,
    acts: impl IntoIterator<Item = RailAct>,
) -> Vec<Op<SignedOp>> {
    let mut out: Vec<Op<SignedOp>> = Vec::new();
    for (i, act) in acts.into_iter().enumerate() {
        let op = match out.last() {
            None => signed_in(ns, k, ts, 0, act),
            Some(prev) => after_in(ns, k, prev, ts + i as i64, act),
        };
        out.push(op);
    }
    out
}

/// A line that claims to be `claims`'s act but carries `signs`'s signature
/// over its id — the line a hostile peer pushes, which admission must refuse
/// as a bad signature. Made by [`sign_act`] like every other fixture, so the
/// forgery is of the act as the rail defines it, never of a local copy.
pub fn forged(
    signs: &SigningKey,
    claims: &SigningKey,
    ts: i64,
    seq: u64,
    prev: Option<OpId>,
    act: RailAct,
) -> Op<SignedOp> {
    let mut line = sign_act(claims, NS, ts, seq, prev, act, None, None, None);
    line.kind.sig = sign_ring_op(signs, &line.id);
    line
}

/// An id no act here derives: `fill` repeated over 32 bytes, read as a
/// line's id is.
pub fn stray_id(fill: u8) -> OpId {
    serde_json::from_value(serde_json::Value::String(format!("{fill:02x}").repeat(32))).unwrap()
}

/// Each op beside its id, for a function that takes ids its caller derived.
/// A fixture's line carries the id its content derives, so the line's own
/// serves here.
pub fn with_ids<'a>(
    ops: impl IntoIterator<Item = &'a Op<SignedOp>>,
) -> impl Iterator<Item = (&'a OpId, &'a Op<SignedOp>)> {
    ops.into_iter().map(|op| (&op.id, op))
}

pub fn admitted(ops: &[Op<SignedOp>]) -> Admission {
    admit(&crate::ActSet::of(ops), &[], &ring(), NS, &Ed25519Verifier)
}

/// The admitted acts as an app reads them off the wire: the only way a crate
/// above this one sees them.
pub fn wire_ops(a: &Admission) -> Vec<crate::AdmittedOp> {
    serde_json::from_value(serde_json::to_value(a.wire()).expect("the wire serialises"))
        .expect("an app reads back what the wire carries")
}

/// Every op `ops` holds that `theirs` says the peer is missing:
/// [`ops_missing_from_within`] with no budget, the honest TOTAL a test
/// measures. (Moved here from `sync.rs` in lean-3: nothing but tests asked
/// for it, and everything that puts ops on a wire uses the budgeted form.)
pub fn ops_missing_from(
    ops: &[Op<SignedOp>],
    namespace: &str,
    verifier: &dyn RingVerifier,
    theirs: &Digest,
) -> Vec<Op<SignedOp>> {
    ops_missing_from_within(ops, namespace, verifier, theirs, NO_BUDGET).0
}

/// The marks projection of a digest — what most sync bars assert on. The
/// content commitment is asserted where it is the point (`view`'s tests and
/// the fork bars); everywhere else the mark is the claim under test.
pub fn marks(d: &crate::Digest) -> BTreeMap<String, u64> {
    entries(d)
        .into_iter()
        .map(|(actor, view)| (actor, view.mark))
        .collect()
}

/// A digest's `{actor → View}`, read as a peer reads it: through the wire,
/// so the digest keeps its map to itself.
pub fn entries(d: &crate::Digest) -> BTreeMap<String, crate::View> {
    let mut wire = serde_json::to_value(d).expect("a digest serialises");
    serde_json::from_value(wire["entries"].take()).expect("a digest's entries read back")
}

/// The `what` of every act an app's reducer would see, in order — the shape
/// most of these tests assert on.
/// Whether `key(k)` stands in the register `a` folds.
pub fn standing(a: &Admission, k: u8) -> bool {
    a.membership().standing.contains(&actor_of(&key(k)))
}

pub fn applied(a: &Admission) -> Vec<String> {
    a.ops()
        .iter()
        .filter(|o| o.applies())
        .filter_map(|o| {
            serde_json::to_value(o.payload.as_ref()?)
                .ok()?
                .get("what")?
                .as_str()
                .map(str::to_string)
        })
        .collect()
}

/// A store held in memory, for driving [`commit`](crate::commit) without a disk.
pub struct Memory(pub core::cell::RefCell<crate::ActSet>);

impl Memory {
    pub fn holding(lines: &[Op<SignedOp>]) -> Self {
        Self(core::cell::RefCell::new(crate::ActSet::of(lines)))
    }
}

impl crate::Store for Memory {
    type Error = core::convert::Infallible;

    fn append_with<R>(
        &self,
        actor: &str,
        f: impl FnOnce(crate::Held<'_>) -> (Option<crate::Authored>, R),
    ) -> Result<R, Self::Error> {
        let held = self.0.borrow().clone();
        let tip = held
            .iter()
            .filter(|op| op.actor == actor)
            .map(|op| op.kind.seq)
            .max()
            .map(|seq| crate::Tip {
                seq,
                lines: held
                    .iter()
                    .filter(|op| op.actor == actor && op.kind.seq == seq)
                    .map(|op| (op.id.clone(), op.kind.sig.clone()))
                    .collect(),
            });
        let verifier = &crate::Ed25519Verifier;
        let (authored, answer) = f(crate::Held {
            acts: &held,
            tip,
            verifier,
            kept: None,
        });
        for line in authored.iter().flat_map(|a| a.lines()) {
            self.0.borrow_mut().insert(line.clone());
        }
        Ok(answer)
    }
}

/// Any act, written as a test's fixture through [`commit`](crate::commit).
pub struct Fixture {
    pub act: RailAct,
}

impl crate::Command for Fixture {
    type Refusal = core::convert::Infallible;

    fn decide(&self, _before: &Admission) -> Result<Vec<RailAct>, Self::Refusal> {
        Ok(vec![self.act.clone()])
    }
}

/// Several acts, written as one batch through [`commit`](crate::commit).
pub struct Batch(pub Vec<RailAct>);

impl crate::Command for Batch {
    type Refusal = core::convert::Infallible;

    fn decide(&self, _before: &Admission) -> Result<Vec<RailAct>, Self::Refusal> {
        Ok(self.0.clone())
    }
}

// Op::new ported from commonwealth-ai oplog-types/src/lib.rs@e1a43bcd2, out of op.rs
// so the budget counts no test code.
impl<K: crate::Journaled> crate::Op<K> {
    /// Build an op, deriving its content-addressed [`OpId`] from the
    /// (prefix, ts, actor, body) tuple.
    ///
    /// svrngs: test-only, so no production path can build a ring act whose id
    /// hashes the line, signature included — a ring act's id is
    /// [`crate::act_id`]'s (U2). The journal's own tests build envelopes of
    /// other kinds with it.
    ///
    /// Two byte-identical acts at the same second by the same actor collide by
    /// design — callers append in real time, so this does not arise in
    /// practice (the same birthday-bound caveat the atom content-hash ids
    /// carry).
    pub fn new(kind: K, ts_unix: i64, actor: impl Into<String>) -> Self {
        let actor = actor.into();
        // serde_json writes fields in declaration order, so the body string —
        // and therefore the id — is deterministic across runs and builds. A
        // kind failing to serialise is unreachable; silent `""` would derive
        // an id over empty bytes (ROOT_CAUSE_FIXES C4).
        let body = serde_json::to_string(&kind)
            .expect("a kind serialises — an id over empty bytes is the named catastrophe");
        let input = alloc::format!("{}|{ts_unix}|{actor}|{body}", K::ID_PREFIX);
        let v = kind.line_version();
        Self {
            id: OpId::from_hash(ContentHash::of_str(&input)),
            v,
            ts_unix,
            actor,
            kind,
        }
    }
}

/// A redeeming `Admit` of `joining` as `person` under the `Invite` `inv`,
/// its joiner's consent covering `device`: how a key comes to stand now
/// that no bare `Admit` counts.
pub fn redeeming(
    inv: &Op<SignedOp>,
    person: &str,
    joining: &SigningKey,
    device: Device,
) -> RailAct {
    RailAct::Admit {
        person: p(person),
        key: actor_of(joining),
        invite: Some(Redemption::sign(joining, NS, &inv.id, &p(person), device)),
    }
}

/// An `Invite` of the link whose key is `link`, open until `until`; a device
/// link of the person whose first key is `device_of`.
pub fn invite_of(link: &SigningKey, until: i64, device_of: Option<String>) -> RailAct {
    RailAct::Invite {
        key: actor_of(link),
        until,
        device_of,
    }
}

/// `joining`, a computer, let in as `person` in `ns` by a link the
/// committer mints: `commit` writes each act as a fixture and answers its
/// line's id. The `Admit` is written as given, so no name is weighed, as a
/// key let in by hand was before release 2 counted no bare `Admit`.
pub fn let_in_by(
    ns: &str,
    joining: &SigningKey,
    person: &str,
    mut commit: impl FnMut(RailAct) -> OpId,
) {
    let link = SigningKey::from_bytes(&[0xa5; 32]);
    let invite = commit(invite_of(&link, i64::MAX, None));
    let consent = Redemption::sign(joining, ns, &invite, &p(person), Device::Computer);
    let admit = RailAct::Admit {
        person: p(person),
        key: actor_of(joining),
        invite: Some(consent),
    };
    commit(admit);
}

/// `joining` let in as `person`, held by `device`, by a link the issuer
/// (the committer's key, `issuer`) mints, as a link does with no network:
/// `mint` commits the [`MintInvite`] and answers its `Invite`'s id, and
/// `redeem` commits the redeeming [`Redeem`]. A group counts no `Admit`
/// but one under an invitation its joiner consented to.
pub fn let_in(
    (ns, issuer): (&str, &str),
    (joining, person, device): (&SigningKey, &str, Device),
    mint: impl FnOnce(&MintInvite) -> OpId,
    redeem: impl FnOnce(&Redeem),
) {
    let link = SigningKey::from_bytes(&[0xa5; 32]);
    let invite = mint(&MintInvite {
        key: actor_of(&link),
        until: i64::MAX,
        device_of: None,
    });
    let consent = Redemption::sign(joining, ns, &invite, &p(person), device);
    redeem(&Redeem {
        invite,
        person: p(person),
        key: actor_of(joining),
        cosig: consent.sig,
        device,
        link: actor_of(&link),
        issuer: issuer.into(),
    });
}

impl pass::Pass {
    /// This grant naming `line`, signed by `approver`, the key this pass
    /// names, when it may approve it at `now`, its own clock: before the pass
    /// lapses, by its holder, appending only its app's part (HOLDER_APPROVAL
    /// §2; [`approving`]). The holder's own timestamp is no evidence of when
    /// it wrote, so a line stamped after the pass lapses gets none either.
    /// Only tests approve so; a page approves through
    /// [`crate::authority::approve_in_open`].
    pub fn approve(
        &self,
        approver: &SigningKey,
        line: &Op<SignedOp>,
        group: &str,
        now: i64,
    ) -> Option<Self> {
        let mut pass = approving(self, line, group, now)?;
        (pass.approver.as_deref() == Some(actor_of(approver).as_str())).then_some(())?;
        pass.approval = Some(sign_message(approver, &pass.approval_message()));
        Some(pass)
    }
}

/// `pass` naming `line`, unsigned, when its approver may approve it at `now`.
fn approving(pass: &pass::Pass, line: &Op<SignedOp>, group: &str, now: i64) -> Option<pass::Pass> {
    use crate::authority::{entry, read, Lines};
    use alloc::string::ToString;
    use eudoxia_community::pass as r1;
    let (read, content) = (read(pass), crate::grant::bare_id(line, group));
    let text = content.to_string();
    let approves = r1::approving(&read, &entry(&read, line, group, &text, &Lines::new()), now);
    tracing::debug!(target: "rail:authority", actor = %line.actor, now, approves, "pass: an entry's approval, by the community profile's rules");
    approves.then(|| pass::Pass {
        op: Some(content),
        ..pass.clone()
    })
}

impl seed::Seed {
    /// The group's policy, as its id hashes it: only tests ask it apart.
    pub fn policy(&self) -> Option<authority::GroupPolicy> {
        self.policy
    }
}
