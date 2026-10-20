// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/membership.rs@e1a43bcd2
//! Membership — the seed plus the acts that move it, in one function (ARCH
//! §10.6).
//!
//! `RING_APPLICATIONS.md` "Amendment 2026-09-18" is the design; this is its
//! one membership function, the piece `admit` calls and "the piece most likely
//! to be replaced", with the permutation property as its contract. Nothing
//! else on the journal changes membership. `p2panda-auth` was read first.
//!
//! The rule (svrngs, U7; DESIGN §6 "Removal has one decider"):
//!
//! 1. **One walk in the rail's order** `(ts_unix, actor, seq, id)`: an act
//!    counts iff its signer holds standing at the act's position — the
//!    seed's keys to start, plus keys admitted by counting `Admit`s, minus
//!    keys the keeper cut — and, for a cut key, iff the cut covers it. The
//!    keeper is one key at every position: the seed's (the maker's; for a
//!    seed of many keys, upstream's shape, the first key of the person whose
//!    name sorts first), moved only by the keeper's own counting
//!    [`RailAct::HandOn`] to a standing key. A counting [`RailAct::Remove`]
//!    by the keeper of a standing key other than its own is the cut; one by
//!    a key standing as the person it removes is that person's leave, and
//!    the cut too, by them alone (groups-leave; the operator, 2026-10-14:
//!    "Nobody's yes; keeper hands on first") — unless the keeper stands as
//!    that person and someone else stands, so a keeper leaves only once
//!    they hand on, or last; by anyone else it is an ask, and cuts nobody.
//!    A leave's `Remove` is an act of the key it cuts, so the cut covers it
//!    and the acts it descends from. A cut key's own acts after it count
//!    nowhere, so it cannot remove, ask or hand on.
//! 2. **The cut names acts by hash.** It covers the acts `through` names and
//!    every act they descend from by `prev`; any other act of the cut key
//!    counts nowhere, wherever its timestamp puts it. Which cuts count turns
//!    on the walk, and the walk on which acts the cuts cover, so the walk
//!    runs again with the cuts it found until they are the cuts it was
//!    given. A cut never voids a hand-on that made a keeper (the operator,
//!    2026-10-12): it covers each of the cut key's hand-ons that moved the
//!    keeper before it in the walk as though `through` named it, so a
//!    keeper's cut of whoever handed on to them stands (before, it did not
//!    cover that hand-on, undid itself on the next walk and that walk redid
//!    it, #124). The walk is bounded, and a function of the set, but need
//!    not settle: a keeper whose standing rests on an act of the cut key
//!    its cut does not cover still cycles. Past `ops.len() + 1` walks the
//!    answer is the last walk, [`Membership::unsettled`]: who stands is not
//!    judged on it.
//! 3. **Then the void set**: the targets of counting [`RailAct::Correct`]s,
//!    except an act that moves membership, which nothing undoes. Voids
//!    never resurrect, and they cannot reach standing, so they come after
//!    the walk. A voided act is not [`Membership::counted`].
//!
//! The permutation contract: the walk sorts, so every permutation of the op
//! set produces a byte-identical [`Membership`], gaps included.
//! `ra-membership-is-order-free`'s legs pin exactly that.
//!
//! svrngs (U2, DESIGN §6 "A link admits each person once"): an `Admit` that
//! redeems an [`RailAct::Invite`] counts only when its signer signed that
//! `Invite`, the `Invite` is held and counted earlier in the walk, the
//! `Admit` is timed before the `Invite`'s `until`, and the admitted key's
//! co-signature verifies. Otherwise it is [`Membership::refused`] with its
//! [`InviteFault`], and binds nothing. An `Admit` naming no `Invite` keeps
//! the rule above.
//!
//! An issuer has one open link (u2-4b, DESIGN §6 "A new link closes the old
//! one"): their newest counting `Invite` at a place in the walk. A
//! redemption under an older one of theirs, after the newer, is refused
//! [`InviteFault::Closed`]; what the older link admitted before keeps its
//! standing, since closing is a place in the order.
//!
//! svrngs (u4-1, DESIGN §3 "Joining", call 2): each standing key is held by
//! a [`Device`] — a browser's when the counting `Admit` that last bound it
//! redeemed a link naming a browser, a computer's otherwise: a seed key, an
//! `Admit` naming no `Invite`, and every redemption before line version 4.
//! [`Membership::device`] answers it.
//!
//! svrngs (bash-devices, DESIGN §6 "A key per device per group, bound to
//! the person"): a person is their first key in the group — a seed row's
//! first, or the key a plain `Admit` binds — and their name is what that
//! key was bound as. An `Invite` naming a person, a device link, counts only
//! when its signer stands as that person at its place in the walk; a key an
//! `Admit` under it binds is that person's, under their name, whatever name
//! the `Admit` carries. Any other `Admit` makes its key a new person, so two
//! people of one name are two people, never merged.
//! [`Membership::person_of`] answers it.
//!
//! svrngs (bash-same-name): two people of one name are told apart by when
//! each joined — the time on the `Admit` that made their first key a
//! person, or that they started the group, a seed row's — which the people
//! list says and a removal or a hand-on may name. [`Membership::one_named`]
//! answers which person a name and that time name.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::op::Op;

use crate::{Device, OpId, Person, RailAct, Redemption, Roster, SignedOp};

/// Why a redeeming `Admit` does not count — one per condition of the rule
/// in the module docs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InviteFault {
    /// No `Invite` with that id is held here.
    Unheld,
    /// The `Admit`'s signer is not the `Invite`'s.
    NotIssuer,
    /// The `Admit` is timed at or after the `Invite`'s `until`.
    Lapsed,
    /// The admitted key's signature over the redemption does not verify.
    BadCosignature,
    /// The `Invite` did not count before the `Admit` in the rail's order.
    Uncounted,
    /// The issuer's newer `Invite`, `by`, counted before the `Admit` in the
    /// rail's order and closed this one (u2-4b).
    Closed { by: OpId },
}

impl core::fmt::Display for InviteFault {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Unheld => "this node does not hold its link",
            Self::NotIssuer => "its link was made by someone else",
            Self::Lapsed => "its link had lapsed",
            Self::BadCosignature => "the joining key did not sign it",
            Self::Uncounted => "its link did not count before it",
            Self::Closed { .. } => "its link was closed by a newer one",
        })
    }
}

/// When a person joined the group (bash-same-name): they started it, a
/// seed row's person, or the `Admit` that made them a person is timed at
/// `At`, the seconds its signer stamped.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Joined {
    Started,
    At(i64),
}

/// One stretch a person stood in the group (canon-spans): from when they
/// joined ([`Membership::joined`]), `None` for a seed row's person, to the
/// time on the act that cut the last of their keys, `None` while one
/// stands. Both are stamps on acts, never a computer's clock, so every copy
/// holding the acts says the same. A person who left and came back did so
/// by a new key, since a cut key is never let back in (`register.rs`,
/// `RedeemRefusal::Cut`): a new person of the same name, with a stretch of
/// their own.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Span {
    pub person: Person,
    pub joined: Option<i64>,
    pub left: Option<i64>,
}

/// What one fold of the membership acts concludes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Membership {
    /// key → person, from the seed and every counting `Admit`. CUMULATIVE: a
    /// cut key keeps its binding — names stay stable and every past act keeps
    /// the person it always had (leg 4) — it only leaves [`Self::standing`].
    pub bindings: BTreeMap<String, Person>,
    /// key → the person's first key, for each key a seed row or a device
    /// link bound to a person it is not the first key of (bash-devices).
    /// Cumulative as [`Self::bindings`] is; ask [`Self::person_of`]. Skipped
    /// when empty, so a fold with none serialises as before.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub devices: BTreeMap<String, String>,
    /// first key → the time on the `Admit` that made it a person
    /// (bash-same-name); a seed row's has none. Cumulative as
    /// [`Self::bindings`] is; ask [`Self::joined`]. Skipped when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub admitted_at: BTreeMap<String, i64>,
    /// Who holds standing NOW: the seed plus counting `Admit`s, minus the
    /// keys the keeper cut.
    pub standing: BTreeSet<String>,
    /// Derived ids of the acts that count: the signer held standing at the
    /// act's position in the rail's order, a cut covers it if its signer was
    /// cut, and no counting `Correct` voids it. Everything else is held and
    /// reported, never counted.
    pub counted: BTreeSet<OpId>,
    /// The void set — the targets of counting corrections, never an act
    /// that moves membership.
    pub voided: BTreeSet<OpId>,
    /// Redeeming `Admit`s by a standing signer that do not count, each with
    /// its fault (svrngs, U2). Skipped when empty, so a fold with none
    /// serialises as before.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub refused: BTreeMap<OpId, InviteFault>,
    /// Every `Invite` held, by derived id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub invites: BTreeMap<OpId, Invited>,
    /// The keys whose counting `Admit` that last bound them names a
    /// browser (u4-1). Cumulative as [`Self::bindings`] is; ask
    /// [`Self::device`], which answers only for a standing key. Skipped
    /// when empty, so a fold with none serialises as before.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub browsers: BTreeSet<String>,
    /// The group's keeper now (U7): the seed's, or whoever it was handed
    /// on to. `None` only for an empty seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keeper: Option<String>,
    /// Each key the keeper cut, with the cut (U7).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cut: BTreeMap<String, Cut>,
    /// The cuts did not settle: each walk gave the other's (#124), so who
    /// stands cannot be judged, and this answer is only the last walk.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub unsettled: bool,
    /// key → the relay its newest counting [`RailAct::Relay`] names, as
    /// written; a key whose newest names none is absent (relay-in-record).
    /// Cumulative as [`Self::bindings`] is; ask [`Self::relay`].
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub relays: BTreeMap<String, String>,
}

/// One counting cut: the keeper's `Remove`, the acts it names in
/// `through` that this copy does not hold, so cannot yet cover, and the cut
/// key's hand-ons that made a keeper, which it covers as though named.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cut {
    pub by: OpId,
    /// The time on the act that cut it.
    pub at: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unheld: Vec<OpId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handed_on: Vec<OpId>,
    /// Whether the cut was its person's own leave (groups-leave): decided
    /// here once, by `leaves`, and read where a removal is said (#180).
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub left: bool,
}

/// An `Invite` as the register holds it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Invited {
    pub issuer: String,
    pub key: String,
    pub until: i64,
    /// A device link's person, by their first key (bash-devices).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_of: Option<String>,
}

/// A redemption judged on what it alone says: the `Invite` it redeems and
/// that `Invite`'s person, or why it does not count.
type Judged<'a> = Result<(&'a OpId, Option<&'a str>), InviteFault>;

impl Membership {
    /// Whether the act with this derived id counts.
    pub fn counts(&self, id: &OpId) -> bool {
        self.counted.contains(id)
    }

    /// The person `key`, an act's actor, resolves to through the record, cut
    /// or standing: their first key and their name (bash-devices).
    pub fn person_of<'a>(&'a self, key: &'a str) -> Option<(&'a str, &'a Person)> {
        let first = self.devices.get(key).map_or(key, String::as_str);
        Some((first, self.bindings.get(first)?))
    }

    /// The keys standing now as the person whose first key is `first`.
    pub fn devices_of<'a>(&'a self, first: &'a str) -> impl Iterator<Item = &'a String> {
        (self.standing.iter()).filter(move |k| self.person_of(k).is_some_and(|(f, _)| f == first))
    }

    /// When the person whose first key is `first` joined: at their
    /// `Admit`'s time, else they started the group, as a seed row's person
    /// is the one first key with none.
    pub fn joined(&self, first: &str) -> Joined {
        self.admitted_at
            .get(first)
            .map_or(Joined::Started, |t| Joined::At(*t))
    }

    /// Each person's stretch in the group, by their first key: from when
    /// they joined, and once none of their keys stands, to the latest cut
    /// of one, which, the walk being in time, cut their last.
    pub fn spans(&self) -> Vec<Span> {
        let people = (self.bindings.iter()).filter(|(k, _)| !self.devices.contains_key(*k));
        let stands = |first: &str| self.devices_of(first).next().is_some();
        let left = |first: &str| {
            let cuts = (self.cut.iter())
                .filter(|(k, _)| self.person_of(k).is_some_and(|(f, _)| f == first));
            (cuts.map(|(_, c)| c.at).max()).filter(|_| !stands(first))
        };
        let span = |(first, person): (&String, &Person)| Span {
            person: person.clone(),
            joined: self.admitted_at.get(first).copied(),
            left: left(first),
        };
        let spans = people.map(span).collect();
        tracing::debug!(target: "rail:membership", ?spans, "membership: each person's stretch, from when they joined to their last key's cut, or standing");
        spans
    }

    /// The one person standing under `name` who joined `when`, when it is
    /// asked, by their first key; else when each person standing under the
    /// name joined, earliest first — none when nobody does, more than one
    /// when a race or a member's hand gave two people one name. svrngs
    /// never guesses which of two (bash-same-name).
    pub fn one_named(&self, name: &Person, when: Option<Joined>) -> Result<&str, Vec<Joined>> {
        let all: BTreeSet<(Joined, &str)> = (self.keys_of(name))
            .filter_map(|k| self.person_of(k))
            .map(|(first, _)| (self.joined(first), first))
            .collect();
        let of: Vec<&str> = (all.iter())
            .filter(|(j, _)| when.is_none_or(|w| *j == w))
            .map(|(_, first)| *first)
            .collect();
        tracing::debug!(target: "rail:membership", %name, ?when, people = all.len(), asked = of.len(),
            "membership: the one person a name and when they joined name, or when each joined");
        match of[..] {
            [first] => Ok(first),
            _ => Err(all.iter().map(|(j, _)| *j).collect()),
        }
    }

    /// The keys standing now under the name `person`.
    pub fn keys_of<'a, 'p>(
        &'a self,
        person: &'p Person,
    ) -> impl Iterator<Item = &'a String> + use<'a, 'p> {
        self.standing
            .iter()
            .filter(move |k| self.bindings.get(*k) == Some(person))
    }

    /// The relay `key` is reached through, if it stands now and the
    /// record names one that reads as a URL; `None`, said, otherwise.
    pub fn relay(&self, key: &str) -> Option<crate::relay::Url> {
        let said = self.relays.get(key).filter(|_| self.standing.contains(key));
        tracing::debug!(target: "rail:membership", key, ?said, "membership: the relay the record names for a standing key");
        crate::relay::parse(said?).ok()
    }

    /// What holds `key`, if it stands now; `None` if it does not.
    pub fn device(&self, key: &str) -> Option<Device> {
        self.standing.contains(key).then(|| {
            if self.browsers.contains(key) {
                Device::Browser
            } else {
                Device::Computer
            }
        })
    }
}

/// The one membership function. `ops` must already be authenticated (the
/// caller's verifier judged the signatures), each beside the id its content
/// derives ([`crate::derive_id`]; svrngs, U2: the id needs the namespace, so
/// the caller that has it derives once); everything else is decided here,
/// and decided the same way for every arrival order. `namespace` is the one
/// a redemption's co-signature binds.
pub fn membership<'a>(
    ops: impl IntoIterator<Item = (&'a OpId, &'a Op<SignedOp>)>,
    seed: &Roster,
    namespace: &str,
) -> Membership {
    let mut ops: Vec<(&OpId, &Op<SignedOp>)> = ops.into_iter().collect();
    ops.sort_by_key(|(id, op)| (op.ts_unix, op.actor.clone(), op.kind.seq, *id));

    // ── the link: each redemption judged once on what it alone says ───
    let invites: BTreeMap<OpId, Invited> = ops
        .iter()
        .filter_map(|(id, op)| match &op.kind.act {
            RailAct::Invite {
                key,
                until,
                device_of,
            } => Some((
                (*id).clone(),
                Invited {
                    issuer: op.actor.clone(),
                    key: key.clone(),
                    until: *until,
                    device_of: device_of.clone(),
                },
            )),
            _ => None,
        })
        .collect();
    let redeems: BTreeMap<&OpId, Judged> = ops
        .iter()
        .filter_map(|(id, op)| match &op.kind.act {
            RailAct::Admit {
                person,
                key,
                invite: Some(r),
            } => Some((*id, redemption(op, person, key, r, &invites, namespace))),
            _ => None,
        })
        .collect();

    // ── what follows a hand-on in its author's chain keeps nothing ───
    //
    // An act descending by `prev` from a `HandOn` its own author signed
    // carries no keeper's authority, wherever its timestamp puts it: an
    // ex-keeper's backdated second hand-on moves nothing.
    let mut by_chain = ops.clone();
    by_chain.sort_by_key(|(id, op)| (op.actor.clone(), op.kind.seq, *id));
    // id → whether the act is a hand-on or follows one, along its chain.
    let mut handed: BTreeMap<&OpId, (bool, bool)> = BTreeMap::new();
    for (id, op) in &by_chain {
        let after = op
            .kind
            .prev
            .as_ref()
            .and_then(|p| handed.get(p))
            .is_some_and(|(is, after)| *is || *after);
        handed.insert(*id, (matches!(op.kind.act, RailAct::HandOn { .. }), after));
    }
    let kept: BTreeSet<&OpId> = handed
        .into_iter()
        .filter(|(_, (_, after))| !after)
        .map(|(id, _)| id)
        .collect();

    // ── the walk, again until the cuts it finds are the cuts it was given ──
    let mut covers: BTreeMap<String, BTreeSet<OpId>> = BTreeMap::new();
    let mut m = walk(&ops, seed, &redeems, &covers, &kept);
    for round in 0.. {
        let found = covered(&ops, &mut m.cut);
        let settled = found == covers;
        tracing::debug!(target: "rail:membership", round, cuts = found.len(), settled,
            "membership: the walk's cuts against the cuts it was given");
        if settled {
            break;
        }
        if round > ops.len() {
            tracing::warn!(target: "rail:membership", round,
                "membership: the cuts came to no one answer, each walk giving another's, so who stands cannot be judged");
            m.unsettled = true;
            break;
        }
        covers = found;
        m = walk(&ops, seed, &redeems, &covers, &kept);
    }

    // ── the void set, from the corrections that counted ──────────────
    let moves: BTreeSet<&OpId> = ops
        .iter()
        .filter(|(_, op)| op.kind.act.moves_membership())
        .map(|(id, _)| *id)
        .collect();
    for (id, op) in &ops {
        if let RailAct::Correct { corrects, .. } = &op.kind.act {
            if !m.counted.contains(*id) {
                continue;
            }
            if moves.contains(corrects) {
                tracing::debug!(target: "rail:membership", id = %id, target = %corrects,
                    "membership: a Correct of an act that moves membership voids nothing");
                continue;
            }
            m.voided.insert(corrects.clone());
        }
    }
    let voided = m.voided.clone();
    m.counted.retain(|id| !voided.contains(id));
    m.invites = invites;
    m
}

/// For each cut, the acts it covers: the acts of the cut key its `through`
/// or its `handed_on` names and every act they descend from by `prev`,
/// among `ops`. A name this copy does not hold goes in the cut's `unheld`.
fn covered(
    ops: &[(&OpId, &Op<SignedOp>)],
    cuts: &mut BTreeMap<String, Cut>,
) -> BTreeMap<String, BTreeSet<OpId>> {
    let mut out = BTreeMap::new();
    for (key, cut) in cuts.iter_mut() {
        let chain: BTreeMap<&OpId, Option<&OpId>> = ops
            .iter()
            .filter(|(_, op)| &op.actor == key)
            .map(|(id, op)| (*id, op.kind.prev.as_ref()))
            .collect();
        let Some(RailAct::Remove { through, .. }) = ops
            .iter()
            .find(|(id, _)| **id == cut.by)
            .map(|(_, op)| &op.kind.act)
        else {
            continue;
        };
        let mut seen = BTreeSet::new();
        cut.unheld.clear();
        // A leave is an act of the key it cuts: it covers itself.
        let own = chain.contains_key(&cut.by).then(|| cut.by.clone());
        for tip in through.iter().chain(&cut.handed_on).chain(&own) {
            if !chain.contains_key(tip) {
                cut.unheld.push(tip.clone());
            }
            let mut at = Some(tip);
            while let Some(id) =
                at.filter(|id| chain.contains_key(id) && seen.insert((*id).clone()))
            {
                at = chain[id];
            }
        }
        out.insert(key.clone(), seen);
    }
    out
}

/// One walk in the rail's order, a cut key's acts counting only when
/// `covers` holds them, and the keeper's acting as the keeper only when
/// `kept` holds them.
fn walk(
    order: &[(&OpId, &Op<SignedOp>)],
    seed: &Roster,
    redeems: &BTreeMap<&OpId, Judged>,
    covers: &BTreeMap<String, BTreeSet<OpId>>,
    kept: &BTreeSet<&OpId>,
) -> Membership {
    let mut bindings: BTreeMap<String, Person> = (seed.members.iter())
        .flat_map(|(person, keys)| keys.iter().map(move |k| (k.clone(), person.clone())))
        .collect();
    let mut devices: BTreeMap<String, String> = (seed.members.values())
        .flat_map(|keys| keys.iter().skip(1).map(|k| (k.clone(), keys[0].clone())))
        .collect();
    let mut admitted_at: BTreeMap<String, i64> = BTreeMap::new();
    let mut live: BTreeMap<String, Person> = bindings.clone();
    let mut keeper: Option<String> = seed.members.values().flatten().next().cloned();
    let mut cut: BTreeMap<String, Cut> = BTreeMap::new();
    let mut counted: BTreeSet<OpId> = BTreeSet::new();
    let mut refused: BTreeMap<OpId, InviteFault> = BTreeMap::new();
    let mut browsers: BTreeSet<String> = BTreeSet::new();
    let mut relays: BTreeMap<String, String> = BTreeMap::new();
    // Each issuer's newest counting `Invite` so far: their one open link.
    let mut open: BTreeMap<&str, &OpId> = BTreeMap::new();
    // Each key's hand-ons that moved the keeper so far, which a cut keeps.
    let mut made: BTreeMap<&str, Vec<OpId>> = BTreeMap::new();
    for (id, op) in order.iter().copied() {
        if !live.contains_key(&op.actor) {
            continue;
        }
        if covers.get(&op.actor).is_some_and(|c| !c.contains(id)) {
            tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor,
                "membership: an act of a cut key its cut does not cover counts nowhere");
            continue;
        }
        let fault = match redeems.get(id) {
            None => None,
            Some(Err(fault)) => Some(fault.clone()),
            Some(Ok((invite, _))) if !counted.contains(*invite) => Some(InviteFault::Uncounted),
            // Counted, so it opened: the issuer (the signer, past NotIssuer)
            // has an open link, and it is this one or a newer one.
            Some(Ok((invite, _))) => open
                .get(op.actor.as_str())
                .filter(|newest| **newest != *invite)
                .map(|newest| InviteFault::Closed {
                    by: (*newest).clone(),
                }),
        };
        if let Some(fault) = fault {
            tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor, ?fault, "membership: a redeeming Admit does not count");
            refused.insert(id.clone(), fault);
            continue;
        }
        if let RailAct::Invite {
            device_of: Some(of),
            ..
        } = &op.kind.act
        {
            let theirs = devices.get(&op.actor).unwrap_or(&op.actor) == of;
            tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor, device_of = %of, theirs,
                "membership: a device link counts only when its signer stands as its person");
            if !theirs {
                continue;
            }
        }
        counted.insert(id.clone());
        let keeps = keeper.as_deref() == Some(op.actor.as_str()) && kept.contains(id);
        match &op.kind.act {
            RailAct::Admit {
                person,
                key,
                invite,
            } => {
                // An `Admit` naming no `Invite` admits a computer: before a
                // redemption could name a browser, only a computer joined.
                let device = invite.as_ref().map_or(Device::Computer, |r| r.device);
                // Under a device link the key is its person's, under their
                // name; else it is a new person, its own first key.
                let of = redeems.get(id).and_then(|r| r.as_ref().ok()?.1);
                let person = of.and_then(|f| bindings.get(f)).unwrap_or(person).clone();
                match of {
                    Some(f) => devices.insert(key.clone(), String::from(f)),
                    None => {
                        admitted_at.insert(key.clone(), op.ts_unix);
                        devices.remove(key)
                    }
                };
                tracing::debug!(target: "rail:membership", key = %key, person = %person, device_of = ?of, ?device, "membership: standing gained");
                match device {
                    Device::Browser => browsers.insert(key.clone()),
                    Device::Computer => browsers.remove(key),
                };
                bindings.insert(key.clone(), person.clone());
                live.insert(key.clone(), person);
            }
            RailAct::Remove { key, .. } => {
                let of = |k: &str| String::from(devices.get(k).map_or(k, String::as_str));
                let alone = live.keys().all(|k| of(k) == of(&op.actor));
                let leaves = of(key) == of(&op.actor)
                    && (keeper.as_deref().map(of) != Some(of(key)) || alone);
                let cuts = (keeps && *key != op.actor || leaves) && live.contains_key(key);
                tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor, key = %key,
                    keeper = ?keeper, leaves, alone, cuts, kept = made.get(key.as_str()).map_or(0, Vec::len),
                    "membership: a Remove — the keeper's cuts, and a person's own leaves unless they keep and others stand; anyone else's is an ask and cuts nobody; a cut keeps the `kept` hand-ons of the cut key that made a keeper");
                if cuts {
                    let handed_on = made.get(key.as_str()).cloned().unwrap_or_default();
                    live.remove(key);
                    cut.insert(
                        key.clone(),
                        Cut {
                            by: id.clone(),
                            at: op.ts_unix,
                            unheld: Vec::new(),
                            handed_on,
                            left: leaves,
                        },
                    );
                }
            }
            RailAct::HandOn { key } => {
                let hands = keeps && *key != op.actor && live.contains_key(key);
                tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor, key = %key,
                    keeper = ?keeper, hands,
                    "membership: a HandOn — only the keeper's, to a standing key, moves the keeper");
                if hands {
                    keeper = Some(key.clone());
                    made.entry(op.actor.as_str()).or_default().push(id.clone());
                }
            }
            RailAct::Invite { .. } => {
                let older = open.insert(op.actor.as_str(), id);
                tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor, closes = ?older, "membership: a link opens, closing its issuer's older one");
            }
            RailAct::Relay { url } => {
                tracing::debug!(target: "rail:membership", id = %id, actor = %op.actor, ?url,
                    "membership: a key says which relay it is reached through, none withdrawing it");
                match url {
                    Some(url) => relays.insert(op.actor.clone(), url.clone()),
                    None => relays.remove(&op.actor),
                };
            }
            _ => {}
        }
    }
    Membership {
        bindings,
        devices,
        admitted_at,
        standing: live.into_keys().collect(),
        counted,
        voided: BTreeSet::new(),
        refused,
        invites: BTreeMap::new(),
        browsers,
        keeper,
        cut,
        unsettled: false,
        relays,
    }
}

impl RailAct {
    /// The key a `Remove` names, asked or cut; `None` for any other act.
    pub fn names(&self) -> Option<&str> {
        match self {
            RailAct::Remove { key, .. } => Some(key),
            _ => None,
        }
    }

    /// Whether this act moves membership: it binds (`Admit`), cuts or asks
    /// (`Remove`), hands the keeper on (`HandOn`) or opens a link
    /// (`Invite`); or says where its signer is reached (`Relay`), which the
    /// register carries beside each key, and which only a newer one
    /// replaces. A `Correct` does not, since U7: it voids nothing that
    /// moves membership. [`standing_reads`] keeps these, and the acts
    /// of every key a `Remove` names, so the list has one home beside the
    /// fold that reads it, and the match has no wildcard: a new kind of act
    /// does not compile until it is placed here.
    pub const fn moves_membership(&self) -> bool {
        match self {
            RailAct::Admit { .. }
            | RailAct::Remove { .. }
            | RailAct::HandOn { .. }
            | RailAct::Invite { .. }
            | RailAct::Relay { .. } => true,
            RailAct::Record { .. }
            | RailAct::Correct { .. }
            | RailAct::Seal
            | RailAct::Synced { .. }
            | RailAct::Runs { .. } => false,
        }
    }
}

/// The acts a fold of standing reads (U7): every act that moves membership,
/// and every act of a key some `Remove` names, since a cut covers that key's
/// acts along their chain by `prev`. A caller that folds standing from
/// fewer acts than it holds (svrngs-runtime's register) keeps these, and
/// [`RailAct::names`] is how it tells a key named as it goes.
pub fn standing_reads(ops: &[Op<SignedOp>]) -> impl Iterator<Item = &Op<SignedOp>> {
    let named: BTreeSet<&str> = ops.iter().filter_map(|op| op.kind.act.names()).collect();
    ops.iter()
        .filter(move |op| op.kind.act.moves_membership() || named.contains(op.actor.as_str()))
}

/// What a redeeming `Admit` says alone: the `Invite` it names, if that is
/// held, was signed by the `Admit`'s own signer, had not lapsed when the
/// `Admit` was timed, and the admitted key consented. Whether the `Invite`
/// counted is the walk's.
fn redemption<'a>(
    op: &Op<SignedOp>,
    person: &Person,
    key: &str,
    r: &'a Redemption,
    invites: &'a BTreeMap<OpId, Invited>,
    namespace: &str,
) -> Judged<'a> {
    let verdict = match invites.get(&r.id) {
        None => Err(InviteFault::Unheld),
        Some(i) if i.issuer != op.actor => Err(InviteFault::NotIssuer),
        Some(i) if op.ts_unix >= i.until => Err(InviteFault::Lapsed),
        Some(_) if !r.verifies(namespace, person, key) => Err(InviteFault::BadCosignature),
        Some(i) => Ok((&r.id, i.device_of.as_deref())),
    };
    tracing::debug!(target: "rail:membership", invite = %r.id, actor = %op.actor, key, ?verdict, "membership: a redemption judged on what it says");
    verdict
}

#[cfg(test)]
#[path = "membership_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "membership_invite_tests.rs"]
mod invite_tests;

#[cfg(test)]
#[path = "removal_tests.rs"]
mod removal_tests;
