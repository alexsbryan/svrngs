// SPDX-License-Identifier: AGPL-3.0-or-later
//! Who the door lets in to a lend, and each one's share of its upload
//! (DESIGN §8, "Many people, one library" and U3's inventory, its
//! `fair_share_cap` row): a decision over what the door observed, pure and
//! synchronous, the time an argument. The door's task owns one [`Counts`]
//! and reads the clock ([`super::Using`]); nothing here does I/O.
//!
//! The shape is upstream's `fair_share_cap` and `try_grant`
//! (`serving-policy-core/src/fair_sched.rs:163`, `:448` at `e1a43bcd2`): a
//! budget divided evenly among the active, and a newcomer granted or shed.
//! Here the budget is the lender's declared upload and the active are
//! people, not slots or requests: a person with three requests in flight is
//! one person, so seeking costs nobody else a share. A lend whose lender
//! names no limit lets in everyone. Nothing queues — a
//! person past the limit is refused with how many are using it, and an app
//! turns that into a choice (DESIGN §8, "Too many at once").
//!
//! **Who is counted** on a lend: each person with a request through the door
//! to it in flight, or one that ended within [`WINDOW`] — a player between
//! requests is still watching. A person already counted is never refused.
//!
//! **Who is not**: a viewer whose path ends at a private, link-local or
//! unique-local address is taken to be in the lender's home, where the
//! lender's upload is not spent (DESIGN §8, "The local network is guessed").
//! iroh says nothing of homes; [`guess`] is a guess, and the door traces it
//! as one. A viewer guessed home is let in, uncounted, and told no share:
//! their path is not the upload.

use std::collections::BTreeMap;
use std::net::IpAddr;
use std::num::{NonZeroU16, NonZeroU32};
use std::time::{Duration, Instant};

use crate::lend::{Lend, LendName, Upload};

/// How long a person whose last request ended is still counted on a lend:
/// a player fetches the next part of a stream every few seconds, and
/// between two fetches its person is still watching.
pub const WINDOW: Duration = Duration::from_secs(30);

/// Where the door guesses a viewer is, from their path's address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whereabouts {
    /// In the lender's home: a private, link-local or unique-local address.
    Home,
    /// Anywhere else — a public address, a relayed path, or no path seen.
    Away,
}

/// `addr`, the address a viewer's path ends at, or `None` when it ends at a
/// relay or none was seen: home when it is private (10/8, 172.16/12,
/// 192.168/16), link-local (169.254/16, fe80::/10) or unique-local
/// (fc00::/7), an IPv4 address written as IPv6 read as IPv4; else away.
/// Loopback is away: no other computer reaches this one over it.
pub fn guess(addr: Option<IpAddr>) -> Whereabouts {
    let home = match addr.map(|a| a.to_canonical()) {
        Some(IpAddr::V4(v4)) => v4.is_private() || v4.is_link_local(),
        Some(IpAddr::V6(v6)) => v6.is_unique_local() || v6.is_unicast_link_local(),
        None => false,
    };
    if home {
        Whereabouts::Home
    } else {
        Whereabouts::Away
    }
}

/// The door's answer to a person asking for a lend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Let in and counted: `people` are counted with them, and each is told
    /// `share` bits a second of the declared upload — `None` when the lend
    /// declares none.
    In {
        people: NonZeroU32,
        share: Option<u64>,
    },
    /// Let in uncounted: guessed in the lender's home.
    Home,
    /// Refused: the lend's `limit` is reached by `using` people, none of
    /// them this one.
    Full { using: u32, limit: NonZeroU16 },
}

/// One person's use of one lend.
#[derive(Debug, Clone, Copy)]
struct Viewer {
    in_flight: usize,
    /// When their last request began or ended.
    last: Instant,
}

impl Viewer {
    fn counted(&self, now: Instant) -> bool {
        self.in_flight > 0 || now.saturating_duration_since(self.last) < WINDOW
    }
}

/// Who is using each lend, by person: the door's own state, never in the
/// record.
#[derive(Debug, Default)]
pub struct Counts(BTreeMap<LendName, BTreeMap<String, Viewer>>);

/// `upload` divided evenly among `people`, as `fair_share_cap` divides a
/// budget among the active — floored, with no floor of its own: a share
/// is what each may use, and an upload too small to divide is the lender's
/// to raise.
pub fn share_of(upload: Upload, people: NonZeroU32) -> u64 {
    upload.bps() / u64::from(people.get())
}

impl Counts {
    /// `actor`, at `whereabouts`, asks for `lend` at `now`. Let in, a
    /// request of theirs begins and they are counted until [`Self::end`]
    /// ends it and [`WINDOW`] passes; refused, nothing changes.
    pub fn ask(
        &mut self,
        lend: &Lend,
        actor: &str,
        whereabouts: Whereabouts,
        now: Instant,
    ) -> Verdict {
        if whereabouts == Whereabouts::Home {
            return Verdict::Home;
        }
        let viewers = self.0.entry(lend.name.clone()).or_default();
        viewers.retain(|_, v| v.counted(now));
        let known = viewers.contains_key(actor);
        if let Some(limit) = lend
            .limit
            .filter(|l| !known && viewers.len() >= usize::from(l.get()))
        {
            return Verdict::Full {
                using: u32::try_from(viewers.len()).unwrap_or(u32::MAX),
                limit,
            };
        }
        let viewer = viewers.entry(actor.to_string()).or_insert(Viewer {
            in_flight: 0,
            last: now,
        });
        viewer.in_flight += 1;
        viewer.last = now;
        let people = NonZeroU32::new(u32::try_from(viewers.len()).unwrap_or(u32::MAX))
            .expect("this person is counted");
        Verdict::In {
            people,
            share: lend.upload.map(|u| share_of(u, people)),
        }
    }

    /// A request `actor` began on `lend` ended at `now`, its answer whole
    /// or abandoned. One that was never let in is no one's.
    pub fn end(&mut self, lend: &LendName, actor: &str, now: Instant) {
        if let Some(v) = self.0.get_mut(lend).and_then(|p| p.get_mut(actor)) {
            v.in_flight = v.in_flight.saturating_sub(1);
            v.last = now;
        }
    }

    /// How many people are counted on `lend` at `now`.
    pub fn people(&self, lend: &LendName, now: Instant) -> u32 {
        let n = self
            .0
            .get(lend)
            .map_or(0, |p| p.values().filter(|v| v.counted(now)).count());
        u32::try_from(n).unwrap_or(u32::MAX)
    }
}

#[cfg(test)]
mod tests;
