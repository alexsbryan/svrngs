//! The ingest rule (DESIGN C1), for one writer's journal: what a copy holds.
//!
//! [`Journal::decide`] judges a [`VerifiedAct`] against rules 2–4 and returns
//! the outcome as a value; [`Journal::apply`] holds only what `decide` passed,
//! because only `decide` makes a [`Holdable`]. Removal and the rest of the
//! register are E1 (DESIGN D7 E1.3).

use std::collections::BTreeMap;

use crate::chain::{ActId, Actor, Body, Draft, Genesis, GroupId, Interaction, Role, VerifiedAct};
use crate::guests::GuestBook;
use crate::payload::PayloadError;

/// How far ahead of this copy's clock an act's `ts` may be, in seconds
/// (C1 rule 4; DESIGN D10.7: skew past it refuses acts).
pub const MAX_SKEW_SECS: u64 = 300;

/// Why an act is not held yet. It is offered again, never dropped as bad.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Why {
    #[error("{0:?} is named neither in the genesis nor by an Admit from a Person key")]
    Unbound(Actor),
    #[error("its prev {0:?} is not held")]
    PrevNotHeld(ActId),
    #[error("its prev {0:?} is another actor's act")]
    PrevNotOwn(ActId),
    #[error("seq {seq} does not follow its prev's {prev_seq}")]
    SeqNotNext { seq: u64, prev_seq: u64 },
    #[error("ts {ts} is past this copy's clock {now} plus {MAX_SKEW_SECS} s")]
    FromTheFuture { ts: u64, now: u64 },
    #[error("ts {ts} is before {floor}, its prev's or its key's admission")]
    BeforeItsFloor { ts: u64, floor: u64 },
}

/// The outcome of [`Journal::decide`]: the event a shell logs and acts on.
#[derive(Debug)]
pub enum Ingest {
    Held(Holdable),
    AlreadyHeld(ActId),
    OfferAgain(Why),
}

/// An act `decide` passed. Hold it with [`Journal::apply`].
#[derive(Debug)]
pub struct Holdable(Box<VerifiedAct>);

impl Holdable {
    pub fn act(&self) -> &VerifiedAct {
        &self.0
    }
}

/// A key's standing: its role, and the earliest `ts` its acts may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    pub role: Role,
    pub since: u64,
}

/// One group's held acts, beginning with its genesis.
#[derive(Debug, Clone)]
pub struct Journal {
    genesis: Genesis,
    group: GroupId,
    acts: BTreeMap<ActId, VerifiedAct>,
    standing: BTreeMap<Actor, Standing>,
    heads: BTreeMap<Actor, ActId>,
    taken: BTreeMap<Interaction, ActId>,
    guests: GuestBook,
}

impl Journal {
    pub fn new(genesis: Genesis) -> Self {
        let standing = BTreeMap::from([(
            genesis.key,
            Standing {
                role: Role::Person,
                since: 0,
            },
        )]);
        Self {
            group: genesis.group_id(),
            genesis,
            acts: BTreeMap::new(),
            standing,
            heads: BTreeMap::new(),
            taken: BTreeMap::new(),
            guests: GuestBook::default(),
        }
    }

    /// Rules 2–4 of ingest. `now` is this copy's clock, in Unix seconds.
    pub fn decide(&self, act: VerifiedAct, now: u64) -> Ingest {
        if self.acts.contains_key(act.id()) {
            return Ingest::AlreadyHeld(*act.id());
        }
        let Some(standing) = self.standing.get(act.actor()) else {
            return Ingest::OfferAgain(Why::Unbound(*act.actor()));
        };
        let mut floor = standing.since;
        if let Some(prev_id) = act.prev() {
            let Some(prev) = self.acts.get(prev_id) else {
                return Ingest::OfferAgain(Why::PrevNotHeld(*prev_id));
            };
            if prev.actor() != act.actor() {
                return Ingest::OfferAgain(Why::PrevNotOwn(*prev_id));
            }
            if act.seq() != prev.seq() + 1 {
                return Ingest::OfferAgain(Why::SeqNotNext {
                    seq: act.seq(),
                    prev_seq: prev.seq(),
                });
            }
            floor = floor.max(prev.ts());
        }
        if act.ts() > now.saturating_add(MAX_SKEW_SECS) {
            return Ingest::OfferAgain(Why::FromTheFuture { ts: act.ts(), now });
        }
        if act.ts() < floor {
            return Ingest::OfferAgain(Why::BeforeItsFloor { ts: act.ts(), floor });
        }
        Ingest::Held(Holdable(Box::new(act)))
    }

    /// Hold what [`Journal::decide`] passed.
    pub fn apply(&mut self, held: Holdable) {
        let act = *held.0;
        let id = *act.id();
        // A fork leaves two acts on one seq (C1: forks void nothing); the head
        // a writer extends is the higher (seq, id), the same on every copy.
        let newer = self
            .heads
            .get(act.actor())
            .and_then(|h| self.acts.get(h))
            .is_none_or(|h| (act.seq(), id) > (h.seq(), *h.id()));
        if newer {
            self.heads.insert(*act.actor(), id);
        }
        let signer = self.standing.get(act.actor()).map(|s| s.role);
        if let (Body::Admit(a), Some(Role::Person)) = (act.body(), signer) {
            let s = self.standing.entry(a.key).or_insert(Standing {
                role: a.role,
                since: act.ts(),
            });
            s.since = s.since.min(act.ts());
        }
        if let (Some(o), Some(Role::Bridge)) = (act.body().on_behalf_of(), signer) {
            // Lowest id, so the answer is a function of the set, not arrival.
            let t = self.taken.entry(o.interaction.clone()).or_insert(id);
            *t = (*t).min(id);
        }
        if let (Body::Guest(g), Some(Role::Bridge)) = (act.body(), signer) {
            self.guests.hold((act.ts(), *act.actor(), act.seq(), id), g);
        }
        self.acts.insert(id, act);
    }

    /// The next act `actor` writes: its seq and prev from its head, and
    /// `ts = max(now, its last, its admission)` (DESIGN D2).
    pub fn draft(&self, actor: &Actor, body: &Body, now: u64) -> Result<Draft, PayloadError> {
        let since = self.standing.get(actor).map_or(0, |s| s.since);
        match self.heads.get(actor).and_then(|h| self.acts.get(h)) {
            None => Draft::new(*actor, 1, None, now.max(since), body),
            Some(h) => Draft::new(*actor, h.seq() + 1, Some(*h.id()), now.max(h.ts()).max(since), body),
        }
    }

    /// The act a Bridge-role key already wrote for this chat interaction.
    pub fn taken(&self, interaction: &Interaction) -> Option<&ActId> {
        self.taken.get(interaction)
    }

    /// The chat speakers the bridge acts for, from Bridge-role acts only.
    pub fn guests(&self) -> &GuestBook {
        &self.guests
    }

    pub fn standing(&self, actor: &Actor) -> Option<Standing> {
        self.standing.get(actor).copied()
    }

    pub fn genesis(&self) -> &Genesis {
        &self.genesis
    }

    pub fn group(&self) -> &GroupId {
        &self.group
    }

    pub fn get(&self, id: &ActId) -> Option<&VerifiedAct> {
        self.acts.get(id)
    }

    pub fn acts(&self) -> impl Iterator<Item = &VerifiedAct> {
        self.acts.values()
    }

    pub fn len(&self) -> usize {
        self.acts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.acts.is_empty()
    }
}
