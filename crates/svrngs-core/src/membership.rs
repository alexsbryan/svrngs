// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/membership.rs@e1a43bcd2
//! Membership: what one fold of a group's history concludes, as
//! [`crate::authority`]'s adapter projects Eudoxia's interpreter's facts
//! for every consumer (svrn-docs `rings/current/EUDOXIA_JUDGE.md` §2a).
//! Nothing here decides who stands; the interpreter does, and an older
//! svrngs's group stands nobody ([`crate::Roster::older`]).
//!
//! svrngs (bash-devices, DESIGN §6 "A key per device per group, bound to
//! the person"): a person is their first key in the group, and their name
//! is what that key was bound as. [`Membership::person_of`] answers it.
//!
//! svrngs (bash-same-name): two people of one name are told apart by when
//! each joined, which the people list says and a removal or a hand-on may
//! name. [`Membership::one_named`] answers which person a name and that
//! time name.
//!
//! svrngs (guests-visit): whether a [`RailAct::Visit`] is a visit is the
//! interpreter's, judged along its host computer's own chain: a slot of the
//! host's free behind it, within [`VISIT_LONGEST`], its key never a
//! member's nor another visit's. A visit ends at its `until`, at its host's
//! Stop (their `Correct` of it), or when the host's key is cut.
//! [`Guest::live`] answers whether a visit is on when.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::{command::AHEAD_SECS, op::Op};

use crate::{Device, OpId, Person, RailAct, SignedOp, Visit};

/// How long a visit may be, from its act's own time: the interpreter's.
pub use eudoxia_community::{GUESTS_DEFAULT, VISIT_LONGEST};

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

/// What one fold of a group's history concludes, as the interpreter's
/// facts project it (`authority/evidence.rs`).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    /// Who holds standing NOW, as the interpreter resolves it.
    pub standing: BTreeSet<String>,
    /// Derived ids of the acts the interpreter establishes, less those a
    /// counting `Correct` voids. Everything else is held and reported,
    /// never counted.
    pub counted: BTreeSet<OpId>,
    /// The void set — the targets of counting corrections, never an act
    /// that moves membership.
    pub voided: BTreeSet<OpId>,
    /// Every `Invite` held, by derived id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub invites: BTreeMap<OpId, Invited>,
    /// The keys whose counting `Admit` that last bound them names a
    /// browser (u4-1). Cumulative as [`Self::bindings`] is; ask
    /// [`Self::device`], which answers only for a standing key. Skipped
    /// when empty, so a fold with none serialises as before.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub browsers: BTreeSet<String>,
    /// The group's keeper now (U7), while the office is held.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keeper: Option<String>,
    /// The office's state beside [`Self::keeper`], which names a key only
    /// while it is held. Skipped when held.
    #[serde(default, skip_serializing_if = "Office::held")]
    pub office: Office,
    /// The statements that do not count, each with the interpreter's
    /// reason: what a refused write says. Never serialized.
    #[serde(skip)]
    pub uncounted: BTreeMap<OpId, crate::authority::SourceFailure>,
    /// Each key out, with the cut or leave that took it (U7).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cut: BTreeMap<String, Cut>,
    /// key → the relay its newest counting [`RailAct::Relay`] names, as
    /// written; a key whose newest names none is absent (relay-in-record).
    /// Cumulative as [`Self::bindings`] is; ask [`Self::relay`].
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub relays: BTreeMap<String, String>,
    /// Each `Visit` that is one, by its id (guests-visit); ask each
    /// [`Guest::live`] whether it is on. Skipped when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub visits: BTreeMap<OpId, Guest>,
    /// The relays the keeper's newest counting [`RailAct::Meets`] names, as
    /// written (relay fallbacks); `None` while the keeper has named none.
    /// Ask [`Self::meets_at`]. Skipped when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meets: Option<BTreeSet<String>>,
    /// The keeper's guest setting that stands, and how many it gives each
    /// computer; `None` while none is said. Skipped when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slots: Option<(OpId, u8)>,
}

/// Who holds a group's office (EUDOXIA_JUDGE §2c): held; frozen
/// by a fork or by competing recoveries; handed on by `by` to `to` with
/// `hand_on`, and neither accepted nor withdrawn; vacated by its holder's exit; or resting
/// on a line not held. Only held names a keeper; none falls back to another.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Office {
    #[default]
    Held,
    Frozen,
    Pending {
        hand_on: OpId,
        by: String,
        to: String,
    },
    Vacant,
    Missing,
}

impl Office {
    fn held(&self) -> bool {
        *self == Self::Held
    }
}

/// A guest, as a counting [`RailAct::Visit`] names them, hosted by the key
/// that signed it, from the act's time `at`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Guest {
    pub host: String,
    pub at: i64,
    pub visit: Visit,
    /// The time on the host's Stop, if they stopped it.
    pub stopped: Option<i64>,
}

impl Guest {
    /// Whether the visit is on at `at`: begun, and before its `until`, its
    /// host's Stop and the cut of its host's key, as `cut` holds them. It
    /// begins when its act is written: a new act is timed at most one past
    /// a line [`AHEAD_SECS`] ahead of its writer's clock (`command.rs`
    /// `sign_lines`), so a visit timed that far past `at` is begun
    /// (guests-first, #214).
    pub fn live(&self, cut: &BTreeMap<String, Cut>, at: i64) -> bool {
        let cut = cut.get(&self.host).map(|c| c.at);
        let ends = [Some(self.visit.until), self.stopped, cut];
        self.at <= at.saturating_add(AHEAD_SECS + 1) && ends.iter().flatten().all(|&e| at < e)
    }
}

/// One counting cut: the `Remove` that took the key, when, and the acts it
/// names in `through` that this copy does not hold, so cannot yet cover.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cut {
    pub by: OpId,
    /// The time on the act that cut it.
    pub at: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unheld: Vec<OpId>,
    /// Whether the cut was its person's own leave (groups-leave): decided
    /// once, by the interpreter, and read where a removal is said (#180).
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
        (self.admitted_at.get(first)).map_or(Joined::Started, |t| Joined::At(*t))
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
        (self.standing.iter()).filter(move |k| self.bindings.get(*k) == Some(person))
    }

    /// The relay `key` is reached through, if it stands now and the
    /// record names one that reads as a URL; `None`, said, otherwise.
    pub fn relay(&self, key: &str) -> Option<crate::relay::Url> {
        let said = self.relays.get(key).filter(|_| self.standing.contains(key));
        tracing::debug!(target: "rail:membership", key, ?said, "membership: the relay the record names for a standing key");
        crate::relay::parse(said?).ok()
    }

    /// The relays the keeper says the group meets at, each read dotless,
    /// one that reads as no URL left out and said ([`crate::relay::parse`]);
    /// `None` while the keeper has said none.
    pub fn meets_at(&self) -> Option<BTreeSet<crate::relay::Url>> {
        let said = self.meets.as_ref();
        tracing::debug!(target: "rail:membership", ?said, "membership: the relays the keeper says the group meets at");
        Some(
            said?
                .iter()
                .filter_map(|r| crate::relay::parse(r).ok())
                .collect(),
        )
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

/// `ops` in the rail's order, `(ts_unix, actor, seq, id)`.
pub(crate) fn in_rail_order<'a>(
    ops: impl IntoIterator<Item = (&'a OpId, &'a Op<SignedOp>)>,
) -> Vec<(&'a OpId, &'a Op<SignedOp>)> {
    let mut ops: Vec<_> = ops.into_iter().collect();
    ops.sort_by_key(|(id, op)| (op.ts_unix, op.actor.clone(), op.kind.seq, *id));
    ops
}

/// Every `Invite` held, by derived id: what a link says, whatever counts.
pub(crate) fn invites(ops: &[(&OpId, &Op<SignedOp>)]) -> BTreeMap<OpId, Invited> {
    let invited = |(id, op): &(&OpId, &Op<SignedOp>)| match &op.kind.act {
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
    };
    ops.iter().filter_map(invited).collect()
}

/// Rule 3, the void set: the targets of counting corrections, except an
/// act that moves membership, which nothing undoes; voided acts leave
/// [`Membership::counted`].
pub(crate) fn voids(ops: &[(&OpId, &Op<SignedOp>)], m: &mut Membership) {
    let moves: BTreeSet<&OpId> = ops
        .iter()
        .filter(|(_, op)| op.kind.act.moves_membership())
        .map(|(id, _)| *id)
        .collect();
    for (id, op) in ops {
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
    /// (`Invite`); or says where its signer is reached (`Relay`), or who
    /// is visiting (`Visit`), or cuts over to each act counting as its
    /// signer's alone (`Signers`), or where the group meets (`Meets`), each of which the register carries beside
    /// the keys. A `Correct` does not, since U7: it voids nothing that
    /// moves membership ([`voids`]). The match has no wildcard: a new kind
    /// of act does not compile until it is placed here.
    pub const fn moves_membership(&self) -> bool {
        match self {
            RailAct::Authority { .. }
            | RailAct::Accept { .. }
            | RailAct::Admit { .. }
            | RailAct::Remove { .. }
            | RailAct::HandOn { .. }
            | RailAct::Invite { .. }
            | RailAct::Relay { .. }
            | RailAct::Visit(_)
            | RailAct::Signers { .. }
            | RailAct::Holders { .. }
            | RailAct::Meets { .. }
            | RailAct::GuestSlots { .. } => true,
            RailAct::Record { .. }
            | RailAct::Correct { .. }
            | RailAct::Seal
            | RailAct::Synced { .. }
            | RailAct::Runs { .. } => false,
        }
    }
}

#[cfg(test)]
#[path = "membership_invite_tests.rs"]
mod invite_tests;

#[cfg(test)]
#[path = "visit_tests.rs"]
mod visit_tests;
