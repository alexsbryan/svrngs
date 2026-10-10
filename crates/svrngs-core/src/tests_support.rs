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
    sign_act(k, ns, ts, seq, prev, act, on_behalf_of, None)
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
    struct Impostor<'a> {
        signs: &'a SigningKey,
        claims: String,
    }
    impl RingSigner for Impostor<'_> {
        fn actor(&self) -> String {
            self.claims.clone()
        }
        fn sign(&self, id: &OpId) -> String {
            self.signs.sign(id)
        }
    }
    let impostor = Impostor {
        signs,
        claims: actor_of(claims),
    };
    sign_act(&impostor, NS, ts, seq, prev, act, None, None)
}

/// An id no act here derives: `fill` repeated over 32 bytes.
pub fn stray_id(fill: u8) -> OpId {
    OpId::from_hex(&format!("{fill:02x}").repeat(32)).unwrap()
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

/// The marks projection of a digest — what most sync bars assert on. The
/// content commitment is asserted where it is the point (`view`'s tests and
/// the fork bars); everywhere else the mark is the claim under test.
pub fn marks(d: &crate::Digest) -> BTreeMap<String, u64> {
    d.iter()
        .map(|(actor, view)| (actor.clone(), view.mark))
        .collect()
}

/// The `what` of every act an app's reducer would see, in order — the shape
/// most of these tests assert on.
pub fn applied(a: &Admission) -> Vec<String> {
    a.ops()
        .iter()
        .filter(|o| o.applies())
        .filter_map(|o| {
            o.payload
                .as_ref()?
                .as_value()
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
        f: impl FnOnce(&crate::ActSet, Option<crate::Tip>) -> (Option<crate::Authored>, R),
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
        let (authored, answer) = f(&held, tip);
        for line in authored.iter().flat_map(|a| a.lines()) {
            self.0.borrow_mut().insert(line.clone());
        }
        Ok(answer)
    }
}

/// Any act, written as a test's fixture through [`commit`](crate::commit).
pub struct Fixture {
    pub act: RailAct,
    pub on_behalf_of: Option<String>,
}

impl crate::Command for Fixture {
    type Refusal = core::convert::Infallible;
    const READS: crate::Reads = crate::Reads::Membership;

    fn decide(&self, _before: &Admission) -> Result<Vec<RailAct>, Self::Refusal> {
        Ok(vec![self.act.clone()])
    }

    fn on_behalf_of(&self) -> Option<&str> {
        self.on_behalf_of.as_deref()
    }
}

/// Several acts, written as one batch through [`commit`](crate::commit).
pub struct Batch(pub Vec<RailAct>);

impl crate::Command for Batch {
    type Refusal = core::convert::Infallible;
    const READS: crate::Reads = crate::Reads::Membership;

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
