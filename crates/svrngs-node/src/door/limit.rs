// SPDX-License-Identifier: AGPL-3.0-or-later
//! Who the door lets in to a lend, and each one's share of its upload
//! (DESIGN §3, "Many people, one library"; PLAN §4, U3's inventory, its
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
//! turns that into a choice (DESIGN §3, "Too many at once").
//!
//! **Who is counted** on a lend: each person with a request through the door
//! to it in flight, or one that ended within [`WINDOW`] — a player between
//! requests is still watching. A person already counted is never refused.
//!
//! **Who is not**: a viewer whose path ends at a private, link-local or
//! unique-local address is taken to be in the lender's home, where the
//! lender's upload is not spent (DESIGN §3, "The local network is guessed").
//! iroh says nothing of homes; [`crate::network::guess`] is a guess, and the door traces it
//! as one. A viewer guessed home is let in, uncounted, and told no share:
//! their path is not the upload.

use std::collections::BTreeMap;
use std::num::{NonZeroU16, NonZeroU32};
use std::time::{Duration, Instant};

use crate::lend::{Lend, LendName};
use crate::network::Whereabouts;
use svrngs_core::pass::Label;

/// How long a person whose last request ended is still counted on a lend:
/// a player fetches the next part of a stream every few seconds, and
/// between two fetches its person is still watching.
pub const WINDOW: Duration = Duration::from_secs(30);

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
    /// Refused: the app asking has read this lend as this person past
    /// [`READS_A_MINUTE`] ([`Reads::read`]); it may again after this long.
    TooFast(Duration),
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
        // The upload divided evenly among them, as `fair_share_cap` divides a
        // budget among the active — floored, with no floor of its own: a share
        // is what each may use, and an upload too small to divide is the
        // lender's to raise.
        Verdict::In {
            people,
            share: lend.upload.map(|u| u.bps() / u64::from(people.get())),
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

/// How many times an app may read one lend as one person at once, and,
/// since one more refills every `60 / READS_A_MINUTE` seconds, in a
/// minute: this number's one home (svrn-docs
/// `rings/current/EXTENSION_POINT.md` §7, G2). Movie Night reads each
/// library once a search (`apps/movie-night/index.js` `search`), and the
/// fastest searching on record is a driven run's, one every 15 s
/// (`demo/runs/mn/2026-10-08/README.md`, bar 1): 4 reads a minute. Ten
/// times that is 40. But a page showing a grid of pictures from a lend
/// reads each at once, and a read loop does no lasting harm (EXTENSION_POINT
/// §1): what this stops is a loop's 400 a second (24,824 in 60 s by hand,
/// `a_folder_app_reading_for_sixty_seconds`), so it is set well clear of
/// any page, 600 at once and ten a second, as the operator asked of the
/// write limit: a thin number is no footgun. No person's session of a
/// reading app is recorded; raising it is this line, and nothing kept
/// depends on it.
pub const READS_A_MINUTE: u32 = 600;

/// Each app's reads of each lend, by the person it reads as: when its next
/// read would be due were it reading at [`READS_A_MINUTE`] since it began,
/// a refill as `crate::folder::carried::wait_after` is (GCRA). Reads leave
/// no entry, so this is kept in memory alone, by the door's task: a restart
/// forgets it, and each app may read a whole burst again.
#[derive(Debug, Default)]
pub struct Reads(BTreeMap<(LendName, String, Label), Instant>);

impl Reads {
    /// The app `.2` reads the lend `.0` as the person `.1` at `now`: none
    /// when it may, and it is counted; else how long before it may.
    pub fn read(&mut self, at: (LendName, String, Label), now: Instant) -> Option<Duration> {
        let each = Duration::from_secs(60) / READS_A_MINUTE;
        // A due time past is a full burst, the same as none kept.
        self.0.retain(|_, due| *due > now);
        let due = self.0.get(&at).map_or(now, |d| (*d).max(now)) + each;
        let wait = due.saturating_duration_since(now + each * READS_A_MINUTE);
        if !wait.is_zero() {
            return Some(wait);
        }
        self.0.insert(at, due);
        None
    }
}

#[cfg(test)]
mod tests;
