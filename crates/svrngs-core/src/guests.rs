//! The guest book (DESIGN C5, D4): the chat speakers the bridge acts for,
//! keyed by the platform's user id, kept in the journal by the Bridge-role key.
//!
//! A speaker's entry is their guest acts folded in the record's order
//! `(ts, actor, seq, id)`, so it is a function of the set. A removal sticks
//! until a person re-adds them: only [`GuestEvent::Joined`] lifts it, and
//! being seen again — a late tap, a mention, a stored interaction — does not.

use std::collections::BTreeMap;

use crate::chain::{ActId, Actor, Guest, GuestEvent, Name, Speaker};

/// Where an act sits in the record's order.
type Order = (u64, Actor, u64, ActId);

/// A speaker as the guest book has them: their name, and the last event
/// that counted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestEntry {
    pub name: Name,
    pub last: GuestEvent,
}

impl GuestEntry {
    pub fn removed(&self) -> bool {
        self.last.removes()
    }
}

/// What writing a guest act would do. Only the first three write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestChange {
    /// They are in now and were not before: new, or rejoined after a removal.
    Arrives,
    Renamed,
    Removes,
    /// The book already says this.
    Unchanged,
    /// They were removed, and only a join lifts that.
    RemovalSticks,
}

impl GuestChange {
    pub const fn writes(self) -> bool {
        matches!(self, GuestChange::Arrives | GuestChange::Renamed | GuestChange::Removes)
    }
}

#[derive(Debug, Clone, Default)]
pub struct GuestBook {
    events: BTreeMap<Speaker, BTreeMap<Order, (GuestEvent, Name)>>,
}

impl GuestBook {
    pub(crate) fn hold(&mut self, at: Order, guest: &Guest) {
        self.events
            .entry(guest.speaker.clone())
            .or_default()
            .insert(at, (guest.event, guest.name.clone()));
    }

    pub fn entry(&self, speaker: &Speaker) -> Option<GuestEntry> {
        let mut entry: Option<GuestEntry> = None;
        for (event, name) in self.events.get(speaker)?.values() {
            if *event == GuestEvent::Seen && entry.as_ref().is_some_and(GuestEntry::removed) {
                continue;
            }
            entry = Some(GuestEntry {
                name: name.clone(),
                last: *event,
            });
        }
        entry
    }

    /// Whether the bridge may write for `speaker`. Someone it has never seen
    /// may: acting is how a person enters the book (C5).
    pub fn may_speak(&self, speaker: &Speaker) -> bool {
        !self.entry(speaker).is_some_and(|e| e.removed())
    }

    /// What `guest` would change, judged against the book as it stands.
    pub fn change(&self, guest: &Guest) -> GuestChange {
        let entry = self.entry(&guest.speaker);
        match (entry, guest.event) {
            (None, e) if e.removes() => GuestChange::Removes,
            (None, _) => GuestChange::Arrives,
            (Some(e), GuestEvent::Joined) if e.removed() => GuestChange::Arrives,
            (Some(e), GuestEvent::Seen) if e.removed() => GuestChange::RemovalSticks,
            (Some(e), ev) if ev.removes() => {
                if e.removed() {
                    GuestChange::Unchanged
                } else {
                    GuestChange::Removes
                }
            }
            (Some(e), _) if e.name != guest.name => GuestChange::Renamed,
            (Some(_), _) => GuestChange::Unchanged,
        }
    }

    /// Everyone in the book who is not removed, by speaker.
    pub fn present(&self) -> impl Iterator<Item = (&Speaker, GuestEntry)> {
        self.events
            .keys()
            .filter_map(|s| self.entry(s).filter(|e| !e.removed()).map(|e| (s, e)))
    }
}
