// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native evidence for Eudoxia's community interpreter. This file says what
//! verified original lines are, in the profile's terms; the interpreter alone
//! decides what they establish, and [`interpreted`] projects that once into
//! the [`Membership`] every consumer reads. svrngs's own checks stay here:
//! line signatures, key well-formedness, and the joiner's consent.
use super::SourceOffice;
use super::{AuthorityRef, Judged, LegacyState, Lines};
use super::{SourceFailure, SourceFault, SourceMember};
use crate::membership::{in_rail_order, invites, visiting, voids};
use crate::{ActSet, ContentHash, Cut, Device, Guest, Membership, Office, OpId, Person, RailAct};
use crate::{Journaled, Op, RingVerifier, Roster, SignedOp};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};
use eudoxia_community::{Act, Community, Evidence, Redemption, Roots, Statement, Vocabulary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Native;
impl Vocabulary for Native {
    type Id = OpId;
    type Person = Person;
    type Device = Device;
    type Hash = ContentHash;
    fn computer(device: &Device) -> bool {
        *device == Device::Computer
    }
}
fn refused<T>(why: SourceFault) -> Judged<T> {
    Err(SourceFailure::Refused { why })
}

fn member_key(raw: &str) -> bool {
    raw.len() == 64
        && raw.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        && hex::decode(raw)
            .ok()
            .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
            .and_then(|bytes| ed25519_dalek::VerifyingKey::from_bytes(&bytes).ok())
            .is_some_and(|key| !key.is_weak())
}

/// Explicit trusted initial arrangement, verified from native genesis. A
/// request never supplies this value to an owner.
#[derive(Debug, Clone)]
pub struct SourceRoots {
    pub(crate) namespace: String,
    pub(crate) model: Roots<Native>,
}
impl SourceRoots {
    /// A founded group's roots: the seed's, its first key the office's.
    pub(crate) fn seeded(seed: &Roster, namespace: &str) -> Judged<Self> {
        let office = seed.members.values().flatten().next();
        Self::genesis(namespace, seed, office.map_or("", String::as_str))
    }
    /// An adopted group's closure lines that the old fold counted.
    pub(crate) fn admitted(&self) -> BTreeSet<OpId> {
        self.model.admitted().cloned().collect()
    }
    pub(crate) fn genesis(namespace: &str, seed: &Roster, office: &str) -> Judged<Self> {
        let mut members = BTreeMap::new();
        for (person, keys) in &seed.members {
            let Some(first) = keys.first() else {
                continue;
            };
            for key in keys {
                if !member_key(key) {
                    return refused(SourceFault::Binding);
                }
                let member = SourceMember::root(first.clone(), person.clone(), Device::Computer);
                if members.insert(key.clone(), member).is_some() {
                    return refused(SourceFault::Root);
                }
            }
        }
        let model = Roots::genesis(members, office.into())?;
        let namespace = namespace.into();
        Ok(Self { namespace, model })
    }
}
impl core::ops::Deref for SourceRoots {
    type Target = Roots<Native>;
    fn deref(&self) -> &Roots<Native> {
        &self.model
    }
}

pub(super) fn originals<'a>(
    namespace: &str,
    held: &'a ActSet,
    verifier: &dyn RingVerifier,
) -> (Lines<'a>, BTreeSet<OpId>) {
    let (mut originals, mut invalid) = (Lines::new(), BTreeSet::new());
    for op in held.iter() {
        let id = crate::derive_id(op, namespace);
        if !verifier.verify(&op.actor, &id, &op.kind.sig) {
            invalid.insert(id);
            continue;
        }
        // Version is unsigned envelope metadata. Preserve a supported encoding
        // of this signed body when a peer also sends an unsupported alias.
        let rank = |op: &Op<SignedOp>| {
            (
                !(op.kind.line_version()..=SignedOp::UNDERSTANDS).contains(&op.v),
                op.v,
            )
        };
        if originals.get(&id).is_none_or(|old| rank(op) < rank(old)) {
            originals.insert(id, op);
        }
    }
    (originals, invalid)
}

/// One verified line in the profile's terms. An Admit under an Invite inside
/// an adoption's closure may carry the old consent: its issuer's computer
/// may have redeemed before it held the Adopt (§2e rule 6).
fn statement(namespace: &str, op: &Op<SignedOp>, closure: &BTreeSet<OpId>) -> Statement<Native> {
    let act = match &op.kind.act {
        RailAct::Invite {
            until, device_of, ..
        } => Act::Invite {
            until: *until,
            device_of: device_of.clone(),
        },
        RailAct::Admit {
            person,
            key,
            invite,
        } => Act::Admit {
            person: person.clone(),
            key: key.clone(),
            sound: member_key(key),
            redemption: invite.as_ref().map(|r| {
                let message =
                    crate::source_redemption_message(namespace, &r.id, person, key, r.device);
                let old = closure.contains(&r.id) && r.verifies(namespace, person, key);
                Redemption {
                    invite: r.id.clone(),
                    device: r.device,
                    consent: old || crate::verify_hex(key, &message, &r.sig),
                }
            }),
        },
        RailAct::HandOn { key } => Act::HandOn { key: key.clone() },
        RailAct::Accept { hand_on } => Act::Accept {
            hand_on: hand_on.clone(),
        },
        RailAct::Remove { key, through } => Act::Remove {
            key: key.clone(),
            through: through.clone(),
        },
        RailAct::Meets { .. } => Act::Meets,
        RailAct::Correct {
            corrects,
            replacement,
        } => Act::Correct {
            corrects: corrects.clone(),
            replacement: replacement.as_ref().map(|p| p.app().into()),
        },
        RailAct::Record { payload } => Act::Data {
            part: payload.app().into(),
        },
        RailAct::Authority { change } => Act::Change(change.clone()),
        RailAct::Signers { through } | RailAct::Holders { through } => Act::Other {
            through: through.clone(),
        },
        _ => Act::Other {
            through: Vec::new(),
        },
    };
    Statement {
        author: op.actor.clone(),
        stands_as: op.stands_as().into(),
        prev: op.kind.prev.clone(),
        at: op.ts_unix,
        readable: (op.kind.line_version()..=SignedOp::UNDERSTANDS).contains(&op.v),
        delegated: op.kind.on_behalf_of.is_some(),
        act,
    }
}

/// The held set's evidence under these roots. Each query asks the profile.
pub struct NativeAuthority<'a> {
    roots: &'a SourceRoots,
    evidence: Evidence<Native>,
}
impl<'a> NativeAuthority<'a> {
    /// Over lines already verified, and for a holder's, granted (R1's).
    fn of(roots: &'a SourceRoots, lines: &Lines<'_>, forged: BTreeSet<OpId>) -> Self {
        let held = lines
            .iter()
            .map(|(id, op)| (id.clone(), statement(&roots.namespace, op, roots.closure())))
            .collect();
        let evidence = Evidence { held, forged };
        Self { roots, evidence }
    }
    fn graph(&self) -> Community<'_, Native> {
        Community::new(&self.roots.model, &self.evidence)
    }
    /// The one projection of one fold of the facts: every binding, who
    /// stands, the office beside a keeper it names only while held, each key
    /// out by the Remove that took it, the statements that count; then what
    /// svrngs carries over the counted acts in the rail's order (relays,
    /// visits, cutovers, where the group meets) and the void set.
    fn membership(&self, lines: &Lines<'_>) -> Membership {
        let facts = self.graph().facts();
        let counts = |(id, verdict): &(&OpId, &Judged<()>)| {
            let not = |why: &_| tracing::debug!(target: "rail:source", %id, ?why, "source: a statement that does not count");
            verdict.as_ref().inspect_err(not).is_ok()
        };
        let counted = facts.verdicts().iter().filter(counts);
        let counted = counted.map(|(id, _)| id.clone()).collect();
        let failed = |(id, v): (&OpId, &Judged<()>)| Some((id.clone(), v.clone().err()?));
        let uncounted = facts.verdicts().iter().filter_map(failed).collect();
        let bound = facts.bound();
        let at = |id: &OpId| lines.get(id).map_or(0, |op| op.ts_unix);
        let of = |id: &OpId| lines.get(id).map(|op| (&op.actor, &op.kind.act));
        let (keeper, office) = match facts.office() {
            SourceOffice::Held { key, .. } => (Some(key.clone()), Office::Held),
            SourceOffice::Pending { hand_on, .. } => match of(hand_on) {
                Some((by, RailAct::HandOn { key })) => {
                    let (hand_on, by, to) = (hand_on.clone(), by.clone(), key.clone());
                    (None, Office::Pending { hand_on, by, to })
                }
                _ => (None, Office::Missing),
            },
            SourceOffice::Frozen { .. } => (None, Office::Frozen),
            SourceOffice::Vacant { .. } => (None, Office::Vacant),
            SourceOffice::Missing { .. } => (None, Office::Missing),
        };
        tracing::debug!(target: "rail:source", ?keeper, ?office, "source: the office, a keeper only while held");
        let cut = (facts.out().iter()).map(|(key, (by, left))| {
            let through = match of(by) {
                Some((_, RailAct::Remove { through, .. })) => through.as_slice(),
                _ => &[],
            };
            let unheld = through.iter().filter(|id| !lines.contains_key(*id));
            let cut = Cut {
                by: by.clone(),
                at: at(by),
                unheld: unheld.cloned().collect(),
                handed_on: Vec::new(),
                left: *left,
            };
            (key.clone(), cut)
        });
        let joined = |(key, b): (&String, &SourceMember)| match &b.source {
            AuthorityRef::Act(id) if b.first == *key => Some((key.clone(), at(id))),
            _ => None,
        };
        // An adopted group's closure is never projected again (§2e rule 5):
        // its starting state is the pin's, and only later lines move it.
        let start = (self.roots.adoption()).map_or_else(LegacyState::default, |a| a.state.clone());
        let inside = |id: &OpId| self.roots.closure().contains(id);
        let admitted_at = bound.iter().filter_map(joined).chain(start.joined);
        let mut visits = BTreeMap::new();
        for (id, stopped) in start.visits {
            if let Some(op) = lines.get(&id) {
                let RailAct::Visit(visit) = &op.kind.act else {
                    continue;
                };
                let guest = Guest {
                    host: op.actor.clone(),
                    at: op.ts_unix,
                    visit: visit.clone(),
                    stopped,
                };
                visits.insert(id, guest);
            }
        }
        let mut m = Membership {
            bindings: bound
                .iter()
                .map(|(k, b)| (k.clone(), b.person.clone()))
                .collect(),
            devices: (bound.iter().filter(|(k, b)| b.first != **k))
                .map(|(k, b)| (k.clone(), b.first.clone()))
                .collect(),
            admitted_at: admitted_at.collect(),
            standing: facts.standing().clone(),
            browsers: (bound.iter().filter(|(_, b)| b.device == Device::Browser))
                .map(|(k, _)| k.clone())
                .collect(),
            keeper,
            office,
            cut: cut.collect(),
            counted,
            uncounted,
            signers: start.signers,
            holders: start.holders,
            meets: start.meets,
            ..Membership::default()
        };
        let all = in_rail_order(lines.iter().map(|(id, op)| (id, *op)));
        let first = |k: &str| bound.get(k).map_or_else(|| k.into(), |b| b.first.clone());
        let people = || {
            m.standing
                .iter()
                .map(|k| first(k))
                .collect::<BTreeSet<_>>()
                .len()
        };
        for (id, op) in all.iter().filter(|(id, _)| m.counted.contains(*id)) {
            if !inside(id) {
                visiting(&mut visits, (id, op), &m.cut, &m.bindings, &first, &people);
            }
            match &op.kind.act {
                RailAct::Relay { url: Some(url) } => {
                    drop(m.relays.insert(op.actor.clone(), url.clone()))
                }
                RailAct::Relay { url: None } => drop(m.relays.remove(&op.actor)),
                _ if inside(id) => {}
                RailAct::Signers { .. } => drop(m.signers.get_or_insert((*id).clone())),
                RailAct::Holders { .. } => drop(m.holders.get_or_insert((*id).clone())),
                RailAct::Meets { relays } => m.meets = Some(relays.clone()),
                _ => {}
            }
        }
        (m.visits, m.invites) = (visits, invites(&all));
        voids(&all, &mut m);
        m
    }
}

/// A group's membership by its interpreter (EUDOXIA_JUDGE §2a, §2e): its
/// facts over the lines admission verified, chained and granted, from the
/// seed's roots or an adoption's, folded once and projected once. Roots
/// that cannot be given stand nobody up, said at warn: never a legacy fold
/// in their place.
pub(crate) fn interpreted(
    lines: &Lines<'_>,
    forged: BTreeSet<OpId>,
    roots: Judged<SourceRoots>,
) -> Membership {
    match roots {
        Ok(roots) => NativeAuthority::of(&roots, lines, forged).membership(lines),
        Err(why) => {
            tracing::warn!(target: "rail:source", ?why, "source: the seed gives no roots, so nobody stands");
            let office = Office::Missing;
            Membership {
                office,
                ..Membership::default()
            }
        }
    }
}

#[cfg(test)]
#[path = "evidence_tests.rs"]
mod tests;
