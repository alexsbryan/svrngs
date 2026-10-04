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

use std::collections::BTreeMap;

use crate::*;

pub const NS: &str = "house-things";

pub fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

pub fn p(name: &str) -> Person {
    Person::from(name)
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
    admit(ops, &[], &ring(), NS, &Ed25519Verifier)
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
    a.applied()
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
