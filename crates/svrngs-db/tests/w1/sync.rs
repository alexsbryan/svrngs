//! The sync's bookkeeping above the engine (svrn-docs APP_STATE.md §4, "What
//! travels"): each copy's marks, what a member sends a copy, what the copy
//! takes of it, and what a refusal does to a mark. The simulator's own sync,
//! where every set that arrives is taken and the meet sends what the other
//! hasn't taken, is the base (`sim.rs`). §4's marks, and the spike's rules
//! each put in their place, are its variants, as question 9's rules are the
//! model copy's (`rule.rs`).
//!
//! The spike is svrn-docs `rings/current/app-state-spike/spike.patch`, cited
//! by the patch's own lines. Its sync bookkeeping is the 185 lines from
//! `marks` to `take`'s end (1052-1062, 1091-1127, 1129-1169, 1171-1266).
//!
//! A device's `seq` is its set's `db_version` here: Fly's fork counts one
//! per site and a write's sets number 1, 2, 3 and on (`replica.rs`,
//! `ModelCopy::write`), as the spike numbers a device's sets from its own
//! mark (1020-1026).

use std::collections::BTreeSet;

use crate::change::{ChangeSet, SetId, Site, Val, pack};
use crate::replica::{Marks, Replica};

/// What a device's mark counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// §4's: every set of the device up to the mark was taken here, refused
    /// here for good, or vouched for by a member this copy pulled from in
    /// full. A set above mark + 1 is taken and kept while the mark waits for
    /// the gap (engine-and-sync C1).
    Contiguous,
    /// The spike's: the highest `seq` taken (1245-1248), so a set at or
    /// below it is held already and not taken (1180-1189).
    Highest,
}

/// What a refused set does to its device's mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// §4's "refused for good": counted in the mark, never applied.
    Counted,
    /// The spike's: its device gives nothing more in this call, and its mark
    /// holds (1143-1155), so the next call carries it again.
    Stops,
}

/// A sync: how marks count, what a refusal does, and what a copy refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sync {
    pub mark: Mark,
    pub refusal: Refusal,
    /// The spike's: a set is taken only while its device stands now
    /// (1190-1192). §4 takes every set of a removed device that reaches a copy
    /// ("A removed device").
    pub standing_now: bool,
    /// §4's: after a call from a member, a copy adopts the member's mark for
    /// each device none of whose sets in the call stopped. The call carried
    /// every set the member keeps above this copy's mark, so what it holds up
    /// to its own mark is here or outranked here. The spike adopts none.
    pub adopts: bool,
    /// The ask names the sets above its marks this copy holds, so a member
    /// sends none of those again. Neither §4 nor the spike asks so.
    pub haves: bool,
    /// svrngs-db's adoption: a member's mark for a device is taken only on
    /// the word of that device's own copy, or into a copy not yet synced
    /// from one that has, as a newcomer trusts its source (svrn-docs
    /// `app-state-diligence/b-sync-fixes-2026-11-22/`, D1), and only up to
    /// the highest seq of it this copy has counted, so it fills gaps and
    /// passes no set the copy would fetch (`sync-simplify-2026-11-22/`,
    /// change 2; svrngs-db sync.rs `claimed`). With sets pruned, a gap a
    /// pruned set leaves at an established copy closes only on its own
    /// device's word.
    pub vouched: bool,
}

/// The syncs the simulator runs, each by its name in the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    /// The simulator's own sync (`sim.rs`): every set that arrives is
    /// taken, and the meet sends each copy what it hasn't taken.
    Base,
    /// §4's marks: contiguous, a refusal for good counted, every removed
    /// device's set taken, a member's marks adopted after a call.
    Marks,
    /// `Marks`, the ask naming what it holds above its marks.
    Haves,
    /// `Marks` with the spike's highest-taken mark.
    Highest,
    /// `Marks` with the spike's refusal of a device that doesn't stand now,
    /// counted as §4 counts a refusal for good.
    Standing,
    /// `Standing`, the refusal stopping its device and holding its mark, as
    /// the spike's does.
    Stops,
    /// `Marks` adopting no member's marks.
    NoAdopt,
    /// `Haves`, a member's mark adopted only as svrngs-db adopts one
    /// ([`Sync::vouched`]): the sync svrngs-db's copies are held to.
    Vouched,
    /// Every rule of the spike's at once: the highest-taken mark, a refusal
    /// of a device that doesn't stand now, which stops it and holds its
    /// mark, and no adoption.
    Spike,
}

impl Variant {
    pub const ALL: [Variant; 9] = [
        Variant::Base,
        Variant::Marks,
        Variant::Haves,
        Variant::Highest,
        Variant::Standing,
        Variant::Stops,
        Variant::NoAdopt,
        Variant::Vouched,
        Variant::Spike,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Variant::Base => "base",
            Variant::Marks => "marks",
            Variant::Haves => "haves",
            Variant::Highest => "highest",
            Variant::Standing => "standing",
            Variant::Stops => "stops",
            Variant::NoAdopt => "no-adopt",
            Variant::Vouched => "vouched",
            Variant::Spike => "spike",
        }
    }

    /// A variant by its name, as `W1_SYNCS` lists them.
    pub fn named(name: &str) -> Variant {
        Variant::ALL
            .into_iter()
            .find(|v| v.name() == name)
            .unwrap_or_else(|| panic!("no sync {name}"))
    }

    /// Its bookkeeping, or none for the simulator's own.
    pub fn sync(self) -> Option<Sync> {
        let marks = Sync {
            mark: Mark::Contiguous,
            refusal: Refusal::Counted,
            standing_now: false,
            adopts: true,
            haves: false,
            vouched: false,
        };
        Some(match self {
            Variant::Base => return None,
            Variant::Marks => marks,
            Variant::Haves => Sync { haves: true, ..marks },
            Variant::Highest => Sync {
                mark: Mark::Highest,
                ..marks
            },
            Variant::Standing => Sync {
                standing_now: true,
                ..marks
            },
            Variant::Stops => Sync {
                standing_now: true,
                refusal: Refusal::Stops,
                ..marks
            },
            Variant::NoAdopt => Sync { adopts: false, ..marks },
            Variant::Vouched => Sync {
                haves: true,
                vouched: true,
                ..marks
            },
            Variant::Spike => Sync {
                mark: Mark::Highest,
                refusal: Refusal::Stops,
                standing_now: true,
                adopts: false,
                haves: false,
                vouched: false,
            },
        })
    }
}

/// How sets travel between copies before the meet, under a variant. The
/// meet is calls under both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carry {
    /// One at a time, as the simulator sends them and as a phone's sets are
    /// carried (engine-and-sync C1): any copy relays about half of what it
    /// keeps, and each set is reordered, duplicated or lost on its own.
    Sets,
    /// Calls, as the spike's computers sync (1355-1401, `missing` at
    /// 1094-1127): a member sends a copy every set it keeps above the copy's
    /// marks, in (device, seq) order, with its own marks, and the call
    /// arrives whole, twice, or not at all. The spike's budget is left out:
    /// it stops a call in (device, seq) order, so it leaves no gap.
    Calls,
}

impl Carry {
    pub const ALL: [Carry; 2] = [Carry::Sets, Carry::Calls];

    pub fn name(self) -> &'static str {
        match self {
            Carry::Sets => "sets",
            Carry::Calls => "calls",
        }
    }
}

/// A copy's marks, and under §4's the sets above them it has counted; and
/// whether it has synced in full with a copy that had, as svrngs-db says
/// (`Db::syncing`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Book {
    marks: Marks,
    above: BTreeSet<SetId>,
    synced: bool,
}

impl Book {
    /// The book of the copy at `site`: copy 1's synced from the start, as
    /// the root the computer that carries the app makes (svrngs-db
    /// `States::carry`), every other's not.
    pub fn of(site: Site) -> Book {
        Book {
            synced: site == Site::of(1),
            ..Book::default()
        }
    }

    pub fn marks(&self) -> &Marks {
        &self.marks
    }

    /// The sets counted above their marks.
    pub fn above(&self) -> &BTreeSet<SetId> {
        &self.above
    }

    pub fn mark(&self, site: Site) -> i64 {
        self.marks.get(&site).copied().unwrap_or(0)
    }

    /// Whether the set counts here already: at or below its device's mark,
    /// or above it and counted.
    pub fn holds(&self, id: SetId) -> bool {
        id.1 <= self.mark(id.0) || self.above.contains(&id)
    }

    /// The highest seq of the site counted here: the highest set above its
    /// mark, or the mark.
    fn top(&self, site: Site) -> i64 {
        let above = self.above.iter().filter(|id| id.0 == site).map(|id| id.1).max();
        above.unwrap_or(0).max(self.mark(site))
    }

    /// A set taken here, or refused here for good, counted in its device's
    /// mark: the spike's takes the mark to it (1245-1248), §4's keeps it
    /// above the mark until the gap below it fills.
    fn count(&mut self, mark: Mark, id: SetId) {
        match mark {
            Mark::Highest => {
                let m = self.marks.entry(id.0).or_default();
                *m = (*m).max(id.1);
            }
            Mark::Contiguous => {
                self.above.insert(id);
                self.settle(id.0);
            }
        }
    }

    /// The device's mark taken up past every set counted just above it, and
    /// the sets at or below it no longer kept apart.
    fn settle(&mut self, site: Site) {
        let mut m = self.mark(site);
        while self.above.remove(&(site, m + 1)) {
            m += 1;
        }
        if m > 0 {
            self.marks.insert(site, m);
        }
        self.above.retain(|id| id.0 != site || id.1 > m);
    }

    /// A member's mark for a device, adopted where it is higher; whether it
    /// was.
    fn adopt(&mut self, mark: Mark, site: Site, theirs: i64) -> bool {
        if theirs <= self.mark(site) {
            return false;
        }
        self.marks.insert(site, theirs);
        if mark == Mark::Contiguous {
            self.settle(site);
        }
        true
    }
}

/// What a member sends a copy: sets, and with a call, the member's marks,
/// which copy it is, and whether its copy has synced.
#[derive(Clone, Debug)]
pub struct Call {
    pub sets: Vec<ChangeSet>,
    pub marks: Option<Marks>,
    pub from: Option<Site>,
    pub synced: bool,
}

impl Call {
    /// One set carried on its own, with no marks.
    pub fn carried(set: &ChangeSet) -> Call {
        Call {
            sets: vec![set.clone()],
            marks: None,
            from: None,
            synced: false,
        }
    }
}

/// What came of a call.
#[derive(Debug, Default)]
pub struct Ingested {
    /// The sets taken, in the order taken.
    pub taken: Vec<SetId>,
    /// Sets that counted here already: sent again.
    pub held: Vec<SetId>,
    /// Sets refused.
    pub refused: Vec<SetId>,
    /// Sets after a refusal of their device's in the call, under `Refusal::Stops`.
    pub stopped: Vec<SetId>,
}

/// The person whose device writes at `site` in the simulator: `person-0` at
/// site 1 (`sim.rs`, `op`).
fn person(site: Site) -> String {
    format!("person-{}", site.0[0] - 1)
}

/// Whether a set changes an owned row of another person's: a row of `votes`
/// whose `person_key` isn't its writer's. Every copy refuses one alike, for
/// good (APP_STATE §4, "Who may write which row"), and an honest device
/// writes none, so the simulator's writes never do.
pub fn breach(set: &ChangeSet) -> bool {
    let own = pack(&[Val::Text(person(set.id().0))]);
    set.changes()
        .iter()
        .any(|c| c.table == "votes" && !c.pk[1..].starts_with(&own[1..]))
}

impl Sync {
    /// A copy's own write, counted in its own mark, as the spike's seal sets
    /// it (1039-1043).
    pub fn wrote(self, book: &mut Book, id: SetId) {
        book.count(self.mark, id);
    }

    /// What a member keeping `kept` sends a copy whose book is `to`: every set
    /// above the copy's mark for its device, an absent mark asking all
    /// (1099-1103), less those it holds above its mark where the ask names
    /// them; and the member's marks.
    pub fn call(self, kept: Vec<ChangeSet>, (site, from): (Site, &Book), to: &Book) -> Call {
        let mut sets: Vec<ChangeSet> = kept
            .into_iter()
            .filter(|s| {
                let id = s.id();
                id.1 > to.mark(id.0) && !(self.haves && to.above.contains(&id))
            })
            .collect();
        sets.sort_by_key(ChangeSet::id);
        Call {
            sets,
            marks: Some(from.marks.clone()),
            from: Some(site),
            synced: from.synced,
        }
    }

    /// What `from`, whose book is `fb`, sends `to`, whose book is `tb`
    /// ([`Sync::call`]), to which a copy that keeps its own marks is held
    /// (`Replica::sent`).
    pub fn call_from<R: Replica>(self, (from, fb): (&R, &Book), (to, tb): (&R, &Book)) -> Call {
        let call = self.call(from.kept(), (from.site(), fb), tb);
        from.sent(to, &call);
        call
    }

    /// Whether this copy refuses the set: an owned row of another person's,
    /// or a `col_version` or causal length past 2^53, always, every copy
    /// alike (APP_STATE §4, the four outcomes); and under the spike's rule a
    /// device that doesn't stand now.
    fn refuses(self, set: &ChangeSet, removed: &BTreeSet<Site>) -> bool {
        let unbounded = set.changes().iter().any(|c| c.col_version > 1 << 53 || c.cl > 1 << 53);
        breach(set) || unbounded || (self.standing_now && removed.contains(&set.id().0))
    }

    /// A call taken by `copy`: its sets in (device, seq) order (1135-1136),
    /// each one counted here already passed by (1187-1189), each refused one
    /// counted or stopping its device, each other one taken and counted; then,
    /// where the sync adopts, the member's marks of each device not stopped.
    pub fn ingest<R: Replica>(self, copy: &mut R, book: &mut Book, call: &Call, removed: &BTreeSet<Site>) -> Ingested {
        let mut sets: Vec<&ChangeSet> = call.sets.iter().collect();
        sets.sort_by_key(|s| s.id());
        let mut out = Ingested::default();
        let mut stopped: BTreeSet<Site> = BTreeSet::new();
        for set in sets {
            let id = set.id();
            if stopped.contains(&id.0) {
                out.stopped.push(id);
                continue;
            }
            if book.holds(id) {
                out.held.push(id);
                continue;
            }
            if self.refuses(set, removed) {
                out.refused.push(id);
                match self.refusal {
                    Refusal::Counted => book.count(self.mark, id),
                    Refusal::Stops => {
                        stopped.insert(id.0);
                    }
                }
                continue;
            }
            if copy.takes_sets() {
                copy.take(set);
            }
            book.count(self.mark, id);
            out.taken.push(id);
        }
        if self.adopts
            && let Some(theirs) = &call.marks
        {
            let newcomer = call.synced && !book.synced;
            for (site, m) in theirs {
                let vouched = !self.vouched || newcomer || call.from == Some(*site);
                let m = if self.vouched { (*m).min(book.top(*site)) } else { *m };
                if !stopped.contains(site) && vouched {
                    book.adopt(self.mark, *site, m);
                }
            }
            book.synced |= call.synced;
        }
        copy.called(call, &out, book);
        out
    }
}
