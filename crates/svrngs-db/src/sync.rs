// SPDX-License-Identifier: AGPL-3.0-or-later
//! Change sets between copies (svrn-docs `rings/current/APP_STATE.md` §4,
//! "Sealed and signed", "What travels" and the four outcomes): a write's set
//! sealed and signed in the write's own transaction ([`Db::seal`]); what a
//! copy asks a member ([`Db::ask`]), what the member sends it
//! ([`Db::missing`]), and what the copy takes of that ([`Db::ingest`]).
//!
//! Forked from the app-state spike's `state.rs` (svrn-docs
//! `rings/current/app-state-spike/spike.patch`): its `marks` (1052-1062),
//! `missing` (1091-1127), `ingest` (1129-1169) and `take` (1171-1266), as
//! `app-state-diligence/b-spike-sync-2026-11-22/` measured them.
//! - Kept: what is fetched, in (site, seq) order, to a budget; a call's
//!   order and its one transaction, with a savepoint for each set; apply and
//!   keep; telling the subscription.
//! - Rewritten: the spike's highest-taken mark, which split 90,146 of
//!   100,000 seeds when sets travel one at a time, is now a contiguous mark
//!   per site. A set above it is counted apart until the gap fills, as W's
//!   simulator keeps them (`tests/w1/sync.rs`, `Book`). The ask names the
//!   sets it holds above its marks (`haves`) and its twins, and after a full
//!   call the copy adopts the member's marks. The spike's stop, which fetched
//!   a refused set every round, is now the four outcomes ([`Took`]).
//! - Kept, as the spike's `prune` (1268-1289): a set dropped once none of
//!   its values stands on this copy ([`Db::prune`]), in the transaction of
//!   the write or the call that outranked it.
//! - Dropped: its refusal of a device that doesn't stand now, which split
//!   copies (the operator's "A) No undoing"); its binding of a site to the
//!   first device a copy saw claim it, since a site is derived from its
//!   signer's key and its database's nonce (`svrngs_core::set::site`); and
//!   its sets as JSON with hex keys, which cost 3.8 times the data they keep.
//!   A set's changes, and what travels, are postcard's bytes.
//!
//! Owned rows are checked at the write and at apply by one function
//! (`owned.rs`): checked at apply alone, a set the writer's copy took would
//! be refused on every other copy.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::Ordering::Relaxed;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, OptionalExtension as _, Row, Transaction, TransactionBehavior::Immediate, params};
use serde::{Deserialize, Serialize};
use svrngs_core::set::{Site, site};

use crate::parse::{Change, Unfit, Valid, fit, parse};
pub use crate::wire::{Device, Head, Hex, Seq, Set, Sig};
use crate::{Db, Refused, fault, owned};
use eudoxia_community::db::Record;

impl ToSql for Seq {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok((self.0 as i64).into())
    }
}

impl FromSql for Seq {
    fn column_result(v: ValueRef<'_>) -> FromSqlResult<Self> {
        let n = i64::column_result(v)?;
        u64::try_from(n)
            .ok()
            .and_then(Self::new)
            .ok_or(FromSqlError::OutOfRange(n))
    }
}

impl<const N: usize> ToSql for Hex<N> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl<const N: usize> FromSql for Hex<N> {
    fn column_result(v: ValueRef<'_>) -> FromSqlResult<Self> {
        Self::parse(v.as_str()?).ok_or(FromSqlError::InvalidType)
    }
}

/// A set's place: its site and its seq.
pub type Place = (Site, Seq);

/// What a copy holds, as it asks a member: each site's mark, every set of
/// the site up to it taken here, refused here for good, or vouched for by a
/// member pulled from in full; the sets counted above a mark, `haves`; and
/// each set it keeps at a place where it keeps two, `twins`, by signature;
/// and each device past its share here, `muted`, which no member sends more
/// of ([`Binds::share`]).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ask {
    pub marks: BTreeMap<Site, Seq>,
    pub haves: BTreeSet<Place>,
    pub twins: BTreeSet<(Place, Sig)>,
    pub muted: BTreeSet<Device>,
}

/// What a member sends a copy: the sets the copy's ask says it lacks, the
/// member's own ask, and whether those are all it keeps that the copy lacks,
/// so the copy may adopt its marks; and whether the member's own copy has
/// synced, as only a full call from one that has makes a copy synced
/// (APP_STATE §7, invariant 8).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
    pub ask: Ask,
    pub sets: Vec<Set>,
    pub full: bool,
    pub synced: bool,
}

/// How a set offered to a copy ends (APP_STATE §4, the four outcomes, with
/// two ways it ends being here already), as a phone that carried it is told:
/// decided by the set and by what this copy's record binds and has cut, a
/// phone's by its pass too, never by whether a device stands now, so copies
/// that take sets in any order end alike ([`Binds`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Took {
    /// Applied and kept, counted in its site's mark.
    Taken,
    /// A second set at a place where this copy keeps one: taken too, and
    /// named in the ask, since a mark that counts the place fetches neither.
    Twin,
    /// Counted here already: passed by.
    Held,
    /// Its device isn't one this copy's record binds yet, or it names a
    /// table or column of a schema this copy doesn't hold yet: dropped as if
    /// it never came, its mark unmoved, so a member sends it again once the
    /// record here binds it (the record's exchange runs first in a dial), or
    /// once this copy takes the widest schema (`States::widen`).
    NotYet,
    /// Refused for good, alike on every copy: counted, never applied.
    Refused,
    /// Its signature isn't its device's: dropped as if it never came, its
    /// mark unmoved.
    NotItsDevice,
    /// This copy couldn't apply it: its mark holds, and the next call
    /// fetches it again.
    Fault,
    /// Its device is past its share of what this copy's ask may carry
    /// ([`Binds::share`]): dropped as if it never came, its mark unmoved.
    Over,
}

/// The most bytes one row of a copy's ask costs in a call: a twin's place
/// and signature, a 16-byte site, a seq of up to 9 (postcard's varint of
/// `i64::MAX`) and a 130-byte signature. A site's mark, a have or a muted
/// device costs less. `tests/bounds.rs` holds it to postcard's encoding of
/// the largest of each.
pub const ROW_MOST: usize = 155;

/// The keys a group's record binds, standing or cut, each with its person's
/// first key and whether the office has cut it, by a cut whose reach the
/// record holds whole, as a set's device and the owned rows it changes are
/// judged by them (APP_STATE §4, "A removed device", "Who may write which
/// row"; Eudoxia's `db::Record`, `owned.rs`); and the room a call's
/// ask may fill beside its sets. It holds only what a record can gain, a
/// binding and a cut, never whether a device stands now, and gives them
/// only through Eudoxia's rules: so a set's outcome depends only on the set
/// and on facts the record can only gain, and the order sets arrive in
/// can't split copies. Whether a device still stands is the door's
/// question, which refuses a removed device's computer and phone, never
/// apply's. Every key the record binds has an equal
/// share of the room ([`Self::share`]), so no one device can fill what every
/// copy's ask may carry and stall the app's sync for everyone.
pub struct Binds {
    keys: BTreeMap<String, (String, bool)>,
    room: usize,
}

/// The record as Eudoxia's rules read it: each key's person, the office's
/// cuts, and a person's keys, all from the keys the record binds.
impl Record for Binds {
    fn person_of(&self, device: &str) -> Option<&str> {
        self.keys.get(device).map(|(person, _)| person.as_str())
    }
    fn cut(&self, device: &str) -> bool {
        self.keys.get(device).is_some_and(|(_, cut)| *cut)
    }
    fn keys_of(&self, person: &str) -> impl Iterator<Item = &str> {
        let theirs = move |(_, (p, _)): &(&String, &(String, bool))| p == person;
        self.keys.iter().filter(theirs).map(|(key, _)| key.as_str())
    }
}

impl Binds {
    /// Each key, its person's first key, and whether the office has cut it.
    pub fn new(keys: impl IntoIterator<Item = (String, String, bool)>, room: usize) -> Self {
        let keys = keys.into_iter().map(|(k, person, cut)| (k, (person, cut))).collect();
        Self { keys, room }
    }

    /// Whether a set `device` signed counts, by Eudoxia's rule: once the
    /// record binds it (`db::set`).
    fn binds(&self, device: &Device) -> bool {
        eudoxia_community::db::set(self, device.as_str()) == eudoxia_community::db::Verdict::Taken
    }

    /// Each one's share of the room, in rows of at most [`ROW_MOST`] bytes:
    /// so however full, the ask is at most the room.
    fn share(&self) -> u64 {
        (self.room / ROW_MOST / self.keys.len().max(1)) as u64
    }

    /// Each one's share of an app's database, in bytes of [`SIZE_MOST`].
    fn size_share(&self) -> u64 {
        SIZE_MOST / self.keys.len().max(1) as u64
    }
}

/// The most an app's database on one computer holds of its kept sets and
/// their sites, each bound key's an equal share ([`Binds::size_share`]), so
/// no one device fills the app for everyone (APP_STATE §7, invariant 9): this
/// number's one home. Provisional: about 7 times the spike's 8.6 MB a
/// computer at 25 devices and 10,000 rows (svrn-docs APP_STATE §9, step 2),
/// until W9's run at that scale sets it.
pub const SIZE_MOST: u64 = 64 << 20;

/// What a site costs every copy beside its sets, counted in its device's
/// share of [`SIZE_MOST`]: its rows of sites, marks and above, about 100
/// bytes for the database's life (svrn-docs APP_STATE §8, question 10), so a
/// device minting a nonce a set pays for each.
const SITE_COST: u64 = 100;

/// What a call may still add to this copy's ask as it is taken, by the
/// keys the record binds: the rows each device holds ([`Db::room`]), the
/// bytes it holds of the database, and the devices muted here.
struct Room<'b> {
    binds: &'b Binds,
    rows: Shares,
    size: Shares,
    muted: BTreeSet<Device>,
}

/// What each device holds at a copy: rows of its ask, or bytes.
type Shares = BTreeMap<Device, u64>;

impl Room<'_> {
    /// Whether a set of `device`'s has room for the `adds` rows it brings,
    /// and its `bytes`: unmuted, in its share of the ask with one row left
    /// for its name should it be muted, and in its share of the database.
    fn fits(&self, device: &Device, (adds, bytes): (u64, u64)) -> bool {
        let (held, size) = (self.rows.get(device), self.size.get(device));
        let sized = size.copied().unwrap_or(0) + bytes <= self.binds.size_share();
        !self.muted.contains(device) && (adds == 0 || held.copied().unwrap_or(0) + adds < self.binds.share()) && sized
    }

    fn add(&mut self, device: &Device, (adds, bytes): (u64, u64)) {
        *self.rows.entry(device.clone()).or_default() += adds;
        *self.size.entry(device.clone()).or_default() += bytes;
    }
}

/// How a set is kept here, as `svrngs_sets.kept` says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kept {
    /// Applied, and sent to a member that lacks it.
    Taken,
    /// Refused for good: its place and signature alone, no bytes, so a
    /// second set at its place is judged, never passed by as held.
    Refused,
}

impl ToSql for Kept {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok((*self as i64).into())
    }
}

/// `sql`'s rows on `conn`, each read by `f`.
pub(crate) fn rows<T>(
    conn: &Connection,
    sql: &str,
    p: impl rusqlite::Params,
    f: impl FnMut(&Row) -> rusqlite::Result<T>,
) -> rusqlite::Result<Vec<T>> {
    conn.prepare_cached(sql)?.query_map(p, f)?.collect()
}

/// A set taken here, refused here for good, or waiting here, counted:
/// apart, above its site's mark, until the gap below it fills.
fn count(conn: &Connection, (site, seq): Place) -> rusqlite::Result<()> {
    let sql = "INSERT OR IGNORE INTO svrngs_above (site, seq) SELECT ?1, ?2 \
        WHERE ?2 > coalesce((SELECT mark FROM svrngs_marks WHERE site = ?1), 0)";
    conn.execute(sql, params![site, seq])?;
    adopt(conn, (site, 0))
}

/// `site`'s mark taken to `theirs` where that is higher, then past every
/// set counted just above it, stopping at the last seq there is; those at
/// or below it no longer kept apart.
fn adopt(conn: &Connection, (site, theirs): (Site, i64)) -> rusqlite::Result<()> {
    let mark = conn.query_row("SELECT mark FROM svrngs_marks WHERE site = ?1", [site], |r| r.get(0));
    let mut m: i64 = mark.optional()?.unwrap_or(0).max(theirs);
    let next = "DELETE FROM svrngs_above WHERE site = ?1 AND seq = ?2";
    while let Some(n) = m.checked_add(1)
        && conn.execute(next, params![site, n])? == 1
    {
        m = n;
    }
    let below = "DELETE FROM svrngs_above WHERE site = ?1 AND seq <= ?2";
    conn.execute(below, params![site, m])?;
    let set = "INSERT INTO svrngs_marks (site, mark) VALUES (?1, ?2) ON CONFLICT (site) DO UPDATE SET mark = ?2";
    if m > 0 {
        conn.execute(set, params![site, m])?;
    }
    Ok(())
}

/// `set`, at `site`, kept as `kept` says and counted, its bytes with it but
/// for a refusal's; and, `rows` given, merged into `crsql_changes` first as
/// `site`'s at its seq, where cr-sqlite decides what wins: only rows the
/// parse made, so nothing it didn't read reaches cr-sqlite. Merged with
/// CHECKs off, and on again after, whatever the merge did: a CHECK across two
/// columns that each copy's writes kept can be broken by their merge, and
/// copies would refuse each other's sets for good (svrn-docs
/// `magnets-spikes/README.md`, S5); a page's own write is still held to it.
fn keep(conn: &Connection, (set, site): (&Set, Site), rows: Option<&Valid>, kept: Kept) -> rusqlite::Result<()> {
    let put = "INSERT INTO crsql_changes (\"table\", pk, cid, val, col_version, db_version, site_id, cl, seq, ts) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '0')";
    let seq = set.head.seq;
    conn.pragma_update(None, "ignore_check_constraints", rows.is_some())?;
    let merged = rows.map_or(&[][..], |v| &v.0).iter().try_for_each(|c| {
        let p = params![c.table, c.pk, c.cid, c.val, c.col_version, seq, site, c.cl, c.seq];
        conn.prepare_cached(put)?.execute(p).map(drop)
    });
    conn.pragma_update(None, "ignore_check_constraints", false)?;
    merged?;
    let bytes = (kept != Kept::Refused).then(|| postcard::to_allocvec(set).expect("a set is written in memory"));
    let sql = "INSERT OR IGNORE INTO svrngs_sets (site, seq, sig, kept, bytes) VALUES (?1, ?2, ?3, ?4, ?5)";
    conn.execute(sql, params![site, seq, set.head.sig, kept, bytes])?;
    // The site's device, by which its share is counted and a muted device's
    // sets are found; and its home, from whom alone a word for its mark is
    // trusted: a phone's, as its set names it, else the device's own.
    let home = set.phone.as_ref().map_or(&set.head.device, |p| &p.home);
    let sited = "INSERT OR IGNORE INTO svrngs_sites (site, device, home) VALUES (?1, ?2, ?3)";
    conn.execute(sited, params![site, set.head.device, home])?;
    count(conn, (site, seq))
}

impl Db {
    /// Each set this copy took dropped once none of its values stands here:
    /// no clock row of any of its tables names its site and seq (svrn-docs
    /// APP_STATE §4, "What is kept"; the spike's `prune`, spike.patch:1268-1289).
    /// Sound because nothing is undone: what it drops is outranked by what
    /// stands (§4, "A removed device", "Why"). Copies may keep different
    /// sets, so each prunes by its own clocks (§7, invariant 3), and two
    /// sets at one place stay while either's values stand. Its place stays
    /// counted, so a copy that never took it is vouched past it by its
    /// site's own computer, once that copy counts a set above it. A set
    /// refused holds no value here and is kept as it is. How many it
    /// dropped.
    fn prune(&self, conn: &Connection) -> rusqlite::Result<usize> {
        let clock = |t: &String| format!("\"{}__crsql_clock\"", t.replace('"', "\"\""));
        let stands = self.tables.keys().map(|t| {
            format!(
                "SELECT 1 FROM {} c WHERE c.site_id = o.ordinal AND c.db_version = s.seq",
                clock(t)
            )
        });
        let stands = stands.collect::<Vec<_>>().join(" UNION ALL ");
        let sql = format!(
            "DELETE FROM svrngs_sets WHERE rowid IN (SELECT s.rowid FROM svrngs_sets s \
            LEFT JOIN crsql_site_id o ON o.site_id = s.site WHERE s.kept = ?1 AND NOT EXISTS ({stands}))"
        );
        let dropped = conn.execute(&sql, [Kept::Taken])?;
        tracing::debug!(target: "state", group = self.group, app = self.app, dropped, "state: the sets none of whose values stands on this copy, dropped");
        Ok(dropped)
    }

    /// Whether `set`'s head is signed by its device, over this database's
    /// group and app, and its changes are the ones it signs.
    fn signed(&self, set: &Set) -> bool {
        let (head, names) = (&set.head, (self.group.as_str(), self.app.as_str()));
        // A phone's is its open's approver's, where its pass holds
        // (wire.rs `Phoned::signer`); any other set its device's own.
        let own = || Some((head.device.to_string(), head.at(names).message(&head.changes)));
        let signer = set.phone.as_ref().map_or_else(own, |p| p.signer(head, names));
        let signed = signer.is_some_and(|(by, message)| svrngs_core::verify_hex(&by, &message, head.sig.as_str()));
        signed && svrngs_core::set::changes(&set.changes) == head.changes
    }

    /// The site of `head`'s database, derived from its device and nonce.
    fn site_of(&self, head: &Head) -> Site {
        site(&self.group, &self.app, head.device.as_str(), &head.nonce)
    }

    /// Within a write's transaction on `conn`: its `crsql_changes` rows at
    /// this database's site, sealed as one set at the transaction's version,
    /// signed by this computer's key, kept and counted; none when it changed
    /// nothing. Refused, loudly, at a version this site's sets already hold;
    /// and, in words for the page, where the parse every other copy runs
    /// would refuse the set, or Eudoxia's rule, by the record `binds`, would
    /// not take this computer's change to an owned row (`owned.rs`), so the
    /// writer's copy never takes what no other copy would.
    pub(crate) fn seal(&self, conn: &Connection, binds: &Binds) -> Result<Option<Seq>, Refused> {
        let seq: Seq = conn
            .query_row("SELECT crsql_peek_next_db_version()", [], |r| r.get(0))
            .map_err(fault)?;
        let made = self.changes(conn, seq).map_err(fault)?;
        // Its own mark counts every version it sealed, a set pruned since too.
        let reused = "SELECT ?2 <= coalesce((SELECT mark FROM svrngs_marks WHERE site = ?1), 0) \
            OR EXISTS (SELECT 1 FROM svrngs_sets WHERE site = ?1 AND seq = ?2)";
        let reused: bool = conn
            .query_row(reused, params![self.site, seq], |r| r.get(0))
            .map_err(fault)?;
        tracing::debug!(target: "state", group = self.group, app = self.app, seq = seq.get(), rows = made.len(), reused, "state: a write's rows sealed as one set at its version, none when it changed nothing, or refused at a version this site's sets hold");
        if reused {
            tracing::error!(target: "state", group = self.group, app = self.app, seq = seq.get(), "state: cr-sqlite gave a write a version this database's own sets already hold, so the write is refused");
            return Err(Refused::Fault(format!(
                "this database gave a write the version {} again",
                seq.get()
            )));
        }
        if made.is_empty() {
            return Ok(None);
        }
        let made = fit(made, &self.tables).map_err(Refused::Unkept)?;
        owned::mine(&made, &self.tables, binds, &self.device)?;
        let changes = postcard::to_allocvec(&made.0).expect("changes are written in memory");
        // In this computer's share of the app's database, as every copy
        // counts it at apply, so no copy keeps a set the others drop.
        let (_, size, _) = self.held(conn).map_err(fault)?;
        if size.get(&self.device).copied().unwrap_or(0) + changes.len() as u64 > binds.size_share() {
            return Err(Refused::Full(SIZE_MOST >> 20));
        }
        let set = Set::signed(
            &self.key,
            (&self.group, &self.app),
            (self.nonce, seq, self.schema),
            changes,
        );
        keep(conn, (&set, self.site), None, Kept::Taken).map_err(fault)?;
        self.prune(conn).map_err(fault)?;
        Ok(Some(seq))
    }

    /// Within a write's transaction on `conn`: its `crsql_changes` rows at
    /// this database's site and the transaction's version `seq`.
    pub(crate) fn changes(&self, conn: &Connection, seq: Seq) -> rusqlite::Result<Vec<Change>> {
        let changes = "SELECT \"table\", pk, cid, val, col_version, cl, seq FROM crsql_changes \
            WHERE db_version = ?1 AND site_id = crsql_site_id() ORDER BY seq";
        rows(conn, changes, [seq], |r| {
            let (table, pk, cid, val) = (r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?);
            let (col_version, cl, seq) = (r.get(4)?, r.get(5)?, r.get(6)?);
            Ok(Change {
                table,
                pk,
                cid,
                val,
                col_version,
                cl,
                seq,
            })
        })
    }

    /// What this copy holds, as it asks a member ([`Ask`]).
    pub fn ask(&self) -> Result<Ask, Refused> {
        let conn = self.conn();
        let place = |r: &Row| Ok((r.get(0)?, r.get(1)?));
        let marks = rows(&conn, "SELECT site, mark FROM svrngs_marks", [], place).map_err(fault)?;
        let haves = rows(&conn, "SELECT site, seq FROM svrngs_above", [], place).map_err(fault)?;
        // And every set kept at the site this copy retired as it was made
        // again, as its copy's twins may sit at any of those places.
        let twins = "SELECT site, seq, sig FROM svrngs_sets WHERE (site, seq) IN \
            (SELECT site, seq FROM svrngs_sets GROUP BY site, seq HAVING count(*) > 1) \
            OR site = (SELECT retired FROM svrngs_db)";
        let twins = rows(&conn, twins, [], |r| Ok((place(r)?, r.get(2)?))).map_err(fault)?;
        let (.., muted) = self.held(&conn).map_err(fault)?;
        let (marks, haves, twins) = (
            marks.into_iter().collect(),
            haves.into_iter().collect(),
            twins.into_iter().collect(),
        );
        Ok(Ask {
            marks,
            haves,
            twins,
            muted,
        })
    }

    /// What a member whose ask is `theirs` lacks of the sets this copy
    /// keeps, in (site, seq) order, stopped at `budget` bytes of changes:
    /// each above the member's mark for its site that it doesn't name among
    /// its haves, and each at a place where either keeps two that the
    /// member's ask doesn't name; none refused, nor any of a device the
    /// member's ask names muted; with this copy's ask, and whether it is all
    /// of them.
    pub fn missing(&self, theirs: &Ask, budget: usize) -> Result<Call, Refused> {
        let ask = self.ask()?;
        let twins: BTreeSet<&Place> = ask.twins.iter().chain(&theirs.twins).map(|(p, _)| p).collect();
        let named: BTreeSet<&Sig> = theirs.twins.iter().map(|(_, s)| s).collect();
        // Walked in order, each set's bytes read only once it is sent, and
        // none past the budget: a call copies only the batch it sends.
        let kept = "SELECT s.site, s.seq, s.sig, h.device, s.bytes FROM svrngs_sets s JOIN svrngs_sites h \
            USING (site) WHERE s.kept = ?1 ORDER BY s.site, s.seq, s.sig";
        let conn = self.conn();
        let mut kept = conn.prepare_cached(kept).map_err(fault)?;
        let mut kept = kept.query([Kept::Taken]).map_err(fault)?;
        let (mut sets, mut spent, mut full) = (Vec::new(), 0, true);
        while let Some(r) = kept.next().map_err(fault)? {
            let read = || Ok::<_, rusqlite::Error>(((r.get(0)?, r.get(1)?), r.get(2)?, r.get(3)?));
            let (place, sig, device): (Place, Sig, Device) = read().map_err(fault)?;
            let (site, seq) = place;
            let above = theirs.marks.get(&site).is_none_or(|m| seq > *m) && !theirs.haves.contains(&place);
            let named = !twins.contains(&place) || named.contains(&sig);
            if theirs.muted.contains(&device) || !above && named {
                continue;
            }
            let bytes: Vec<u8> = r.get(4).map_err(fault)?;
            if !sets.is_empty() && spent + bytes.len() > budget {
                full = false;
                break;
            }
            spent += bytes.len();
            sets.push(postcard::from_bytes(&bytes).map_err(fault)?);
        }
        let synced = self.synced.load(Relaxed);
        Ok(Call {
            ask,
            sets,
            full,
            synced,
        })
    }

    /// `call` taken in one transaction: its sets in (device, nonce, seq)
    /// order, each ending one of the four ways ([`Took`]); then, for a full
    /// call, the member's mark of each site as far as its word goes
    /// ([`Self::claimed`]), but for this copy's own site and each site with a
    /// set that faulted here, failed its signature, or isn't bound or schemed
    /// here yet. Each set's place and how it ended. `bound` says whether the record binds a
    /// device's key; `from` is the computer the call came from, where known.
    pub fn ingest(&self, call: &Call, binds: &Binds, from: Option<&Device>) -> Result<Vec<(Place, Took)>, Refused> {
        let mut sets: Vec<&Set> = call.sets.iter().collect();
        sets.sort_by(|a, b| a.head.cmp(&b.head));
        let mut conn = self.conn();
        // Rolled back as it drops on any `?` below, so a call that fails
        // leaves no transaction open (rusqlite 0.41 `transaction.rs:243`).
        let mut tx = conn.transaction_with_behavior(Immediate).map_err(fault)?;
        let mut room = self.room(&tx, binds).map_err(fault)?;
        let (mut out, mut stopped) = (Vec::new(), BTreeSet::new());
        let twins: BTreeSet<Place> = call.ask.twins.iter().map(|(place, _)| *place).collect();
        for set in sets {
            let place = (self.site_of(&set.head), set.head.seq);
            let took = self.take(&mut tx, (set, place.0), (&mut room, &twins)).unwrap_or_else(|e| {
                tracing::warn!(target: "state", group = self.group, app = self.app, device = %set.head.device, seq = set.head.seq.get(), %e, "state: a set this copy couldn't apply, its mark held");
                Took::Fault
            });
            tracing::debug!(target: "state", group = self.group, app = self.app, device = %set.head.device, seq = set.head.seq.get(), ?took, "state: a set offered, and how it ended");
            if matches!(took, Took::NotYet | Took::NotItsDevice | Took::Fault | Took::Over) {
                stopped.insert(place.0);
            }
            out.push((place, took));
        }
        let marks = call.ask.marks.iter();
        let adopting = marks.filter(|(s, _)| call.full && **s != self.site && !stopped.contains(*s));
        tracing::debug!(target: "state", group = self.group, app = self.app, full = call.full, stopped = stopped.len(), "state: a full call's marks adopted as far as its word goes, but for this copy's own site, and each site's with a set that faulted here, failed its signature, or isn't bound or schemed here yet");
        for (s, m) in adopting {
            self.claimed(&tx, (*s, *m), (from, call.synced)).map_err(fault)?;
        }
        let took = out.iter().any(|(_, t)| matches!(t, Took::Taken | Took::Twin));
        if took {
            self.prune(&tx).map_err(fault)?;
        }
        tx.commit().map_err(fault)?;
        let faulted = out.iter().any(|(_, t)| *t == Took::Fault);
        let first = call.full && call.synced && !faulted && !self.synced.swap(true, Relaxed);
        if first {
            let kept = conn.execute("UPDATE svrngs_db SET synced = 1", []);
            tracing::debug!(target: "state", group = self.group, app = self.app, kept = kept.is_ok(), "state: a copy's first full sync, so it is said syncing no more, and its pages told");
        }
        if call.full || faulted {
            self.faulted.store(faulted, Relaxed);
        }
        if first || took {
            self.changed.send_modify(|n| *n += 1);
        }
        Ok(out)
    }

    /// A set a phone signed and carried here on its own, the same bytes
    /// through any computer that keeps the app until one confirms them
    /// (APP_STATE §4, "Phones", step 3): taken as a call of that one set,
    /// never full, so it moves no mark past a gap, and ending one of the
    /// four ways as any set a member sends.
    pub fn carry(&self, set: Set, binds: &Binds) -> Result<Took, Refused> {
        let call = Call {
            sets: vec![set],
            ..Call::default()
        };
        let took = self.ingest(&call, binds, None)?;
        Ok(took.first().map_or(Took::Fault, |(_, took)| *took))
    }

    /// What a call may add to this copy's ask, as it begins ([`Room`]): the
    /// shares the record gives at it kept, by which the ask names the
    /// devices over them now ([`Self::held`]).
    fn room<'b>(&self, conn: &Connection, binds: &'b Binds) -> rusqlite::Result<Room<'b>> {
        self.shares[0].store(binds.share(), Relaxed);
        self.shares[1].store(binds.size_share(), Relaxed);
        let (rows, size, muted) = self.held(conn)?;
        Ok(Room {
            binds,
            rows,
            size,
            muted,
        })
    }

    /// The rows of the ask and the bytes of the database each device holds
    /// at this copy now, and the devices over either share now, by the
    /// shares of this copy's last call: no stored mute, so a device's sets
    /// come again once what it holds here falls (svrn-docs
    /// `app-state-diligence/invariants-2026-11-22/` §2.1). Over its rows, it
    /// has none left beside the row that names it.
    fn held(&self, conn: &Connection) -> rusqlite::Result<(Shares, Shares, BTreeSet<Device>)> {
        // Each row the ask names, by the device of its site: the sites, the
        // sets counted above a mark, each twin, and each set kept at the site
        // this file retired as it was made again, which the ask names as
        // twins too ([`Self::ask`]).
        let shares = "SELECT h.device, count(*) FROM (SELECT site FROM svrngs_sites UNION ALL SELECT site FROM svrngs_above \
            UNION ALL SELECT site FROM svrngs_sets s WHERE EXISTS (SELECT 1 FROM svrngs_sets t WHERE t.site = s.site \
            AND t.seq = s.seq AND t.sig <> s.sig) OR site = (SELECT retired FROM svrngs_db)) AS r \
            JOIN svrngs_sites h USING (site) GROUP BY h.device";
        let held = Shares::from_iter(rows(conn, shares, [], |r| Ok((r.get(0)?, r.get(1)?)))?);
        // And the bytes of the database each holds: its kept sets, and its sites.
        let size = "SELECT h.device, coalesce(sum(length(s.bytes)), 0) + ?1 * count(DISTINCT h.site) \
            FROM svrngs_sites h LEFT JOIN svrngs_sets s USING (site) GROUP BY h.device";
        let size = Shares::from_iter(rows(conn, size, [SITE_COST], |r| Ok((r.get(0)?, r.get(1)?)))?);
        let (share, sized) = (self.shares[0].load(Relaxed), self.shares[1].load(Relaxed));
        let over = (held.iter()).filter(|(_, n)| **n + 1 >= share);
        let over = over.chain(size.iter().filter(|(_, b)| **b > sized));
        let over = over.map(|(d, _)| d.clone()).collect();
        Ok((held, size, over))
    }

    /// `set`, past its share of what this copy's ask may carry, or of the
    /// database, dropped as if it never came; its device named over in this
    /// copy's ask while what it holds here is past its share, so no member
    /// sends it more, and no more of its sets taken in this call. Its sets
    /// before stand. The seat's decision (svrn-docs APP_STATE Decisions,
    /// 2026-11-22): no one device can fill what every copy's ask may carry,
    /// and so stall the app's sync for everyone.
    fn over(&self, set: &Set, room: &mut Room) -> Took {
        let (device, share) = (&set.head.device, room.binds.share());
        tracing::warn!(target: "state", group = self.group, app = self.app, %device, share, "state: a device past its share of what this copy's ask may carry, or of the database, so its set is dropped, and the device named over in the ask while it holds that much here; its sets before stand");
        room.muted.insert(device.clone());
        Took::Over
    }

    /// The place this computer, the home of the phone `phone` here, hands its
    /// next set (svrn-docs `app-state-diligence/sync-simplify-2026-11-22/`,
    /// change 4): in the phone's epoch here, the seq after every one handed
    /// or counted, so none is handed twice; or the next epoch's first, a site
    /// of its own, once a set of the site is counted above one missing here,
    /// as the site's mark would stall on that gap for good. Handed and kept
    /// in one transaction, so two prepares at once hand two places.
    pub(crate) fn place(&self, phone: &str) -> rusqlite::Result<(u64, u64)> {
        let mut conn = self.conn();
        let tx = conn.transaction_with_behavior(Immediate)?;
        let was = "SELECT epoch, handed FROM svrngs_homes WHERE device = ?1";
        let was = tx.query_row(was, [phone], |r| Ok((r.get(0)?, r.get(1)?))).optional()?;
        // A phone's first epoch here is drawn from this file's nonce, so a
        // file made again with a fresh one (`States::remake`) hands places at
        // sites the file it was copied from never hands; below 2^52, so a
        // page's JSON keeps it whole.
        let first = u64::from_be_bytes(self.nonce[..8].try_into().expect("a nonce is 16 bytes")) >> 12;
        let (epoch, handed): (u64, u64) = was.unwrap_or((first, 0));
        let site = site(
            &self.group,
            &self.app,
            phone,
            &crate::wire::nonce(self.device.as_str(), epoch),
        );
        let counted = "SELECT coalesce((SELECT mark FROM svrngs_marks WHERE site = ?1), 0), \
            EXISTS (SELECT 1 FROM svrngs_above WHERE site = ?1)";
        let (mark, gap): (u64, bool) = tx.query_row(counted, [site], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let (epoch, next) = if gap {
            (epoch + 1, 1)
        } else {
            (epoch, handed.max(mark) + 1)
        };
        tracing::debug!(target: "state", group = self.group, app = self.app, phone, epoch, next, gap, "state: the place this computer, a phone's home, hands its next set: past every one handed, or a fresh epoch past a gap");
        let hand = "INSERT INTO svrngs_homes (device, epoch, handed) VALUES (?1, ?2, ?3) \
            ON CONFLICT (device) DO UPDATE SET epoch = ?2, handed = ?3";
        tx.execute(hand, params![phone, epoch, next])?;
        tx.commit()?;
        Ok((epoch, next))
    }

    /// The mark `m` a full call claims for `site`, adopted up to the highest
    /// seq of the site this copy has counted and no further, so it fills the
    /// gaps below what this copy holds and passes nothing it hasn't counted;
    /// and only where the caller is trusted
    /// for it: the site's home, the computer of its own database or a
    /// phone's home, which vouches for its sets as it applied them first,
    /// or a synced caller into a copy not yet synced, as a newcomer trusts
    /// its source (APP_STATE §4, "What travels"). So no word takes a mark past a
    /// set this copy would fetch, and none makes an established copy skip a
    /// set it hasn't fetched yet.
    fn claimed(
        &self,
        tx: &Transaction,
        (site, m): (Site, Seq),
        (from, synced): (Option<&Device>, bool),
    ) -> rusqlite::Result<()> {
        let top = "SELECT device, home, max(coalesce((SELECT mark FROM svrngs_marks WHERE site = ?1), 0), \
            coalesce((SELECT max(seq) FROM svrngs_above WHERE site = ?1), 0)) FROM svrngs_sites WHERE site = ?1";
        let top = tx.query_row(top, [site], |r| {
            Ok((r.get::<_, Device>(0)?, r.get::<_, Device>(1)?, r.get::<_, i64>(2)?))
        });
        let Some((device, home, top)) = top.optional()? else {
            return Ok(());
        };
        // For a device over its share too: adoption only takes rows away.
        let trusted = from == Some(&home) || synced && !self.synced.load(Relaxed);
        tracing::debug!(target: "state", group = self.group, app = self.app, %device, mark = m.get(), top, trusted, "state: a full call's mark for a site, adopted up to the highest seq of it this copy has counted, where its caller is trusted for it");
        if !trusted {
            return Ok(());
        }
        adopt(tx, (site, (m.get() as i64).min(top)))
    }

    /// `set`, at `site`, offered to this copy within the call's transaction
    /// `tx` (the spike's `take`). Whatever it writes, it writes in a
    /// savepoint of its own, released only once all of it is written, so a
    /// set is kept and counted together or not at all.
    fn take(
        &self,
        tx: &mut Transaction,
        (set, site): (&Set, Site),
        (room, twins): (&mut Room, &BTreeSet<Place>),
    ) -> rusqlite::Result<Took> {
        // Its signature first, before anything in it is read.
        if !self.signed(set) {
            return Ok(Took::NotItsDevice);
        }
        // One whose device the record here doesn't bind yet is dropped as if
        // it never came, nothing of it kept or counted. The record's exchange
        // runs first in every dial, so its key's admission travels ahead of
        // it, and a call by marks sends it again once that lands here.
        if !room.binds.binds(&set.head.device) {
            return Ok(Took::NotYet);
        }
        let sigs = "SELECT sig FROM svrngs_sets WHERE site = ?1 AND seq = ?2";
        let sigs: Vec<Sig> = rows(tx, sigs, params![site, set.head.seq], |r| r.get(0))?;
        let counted = "SELECT ?2 <= coalesce(mark, 0) OR EXISTS (SELECT 1 FROM svrngs_above WHERE site = ?1 AND seq = ?2), \
            coalesce(mark, 0), EXISTS (SELECT 1 FROM svrngs_sites WHERE site = ?1) \
            FROM (SELECT (SELECT mark FROM svrngs_marks WHERE site = ?1) AS mark)";
        let (counted, mark, sited): (bool, u64, bool) = tx.query_row(counted, params![site, set.head.seq], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
        // A set at this copy's own site above every version it sealed was
        // sealed by another file of its nonce and key, a copy of this one run
        // beside it, as only such a file could seal it (svrn-docs
        // `app-state-diligence/sync-simplify-2026-11-22/`, change 5): set
        // aside, its mark held, and this file flagged to be made again
        // (`States::remake`), after which the set is another site's here.
        if site == self.site && set.head.seq.get() > mark {
            tracing::warn!(target: "state", group = self.group, app = self.app, seq = set.head.seq.get(), mark, "state: a set at this copy's own site above every version it sealed, so this file was copied from another computer's: it is made again");
            tx.execute("UPDATE svrngs_db SET copied = 1", [])?;
            self.copied.store(true, Relaxed);
            return Ok(Took::Fault);
        }
        // Counted here with no set kept at its place, it was pruned here,
        // and passed by: but for one at a place the call names a twin, which
        // is judged, so a twin reaches a copy that pruned its pair.
        let pruned = counted && sigs.is_empty() && !twins.contains(&(site, set.head.seq));
        if sigs.contains(&set.head.sig) || pruned {
            return Ok(Took::Held);
        }
        // What it adds to this copy's ask: a site new here, a set counted
        // above a gap, a set at a place that keeps one, the first making two
        // rows of it. Past its device's share, it is dropped ([`Self::over`]).
        let device = &set.head.device;
        let adds = u64::from(!sited) + u64::from(set.head.seq.0 > mark + 1) + [0, 2, 1][sigs.len().min(2)];
        let adds = (adds, set.changes.len() as u64 + SITE_COST * u64::from(!sited));
        if !room.fits(device, adds) {
            return Ok(self.over(set, room));
        }
        // Parsed before cr-sqlite sees it. One written under another schema
        // that names a table or column this copy lacks waits for the widest
        // schema here (`States::widen`), dropped as one not bound yet is and
        // sent again by marks once this copy takes it: never refused for
        // good, as a copy that holds that schema takes it. One under another
        // schema that names only what this copy holds, a narrower version's,
        // is judged as any.
        let judged = parse(&set.changes, &self.tables);
        let lacks = |u: &Unfit| matches!(u, Unfit::NoTable(_) | Unfit::NoColumn(_));
        if set.head.schema != self.schema && judged.as_ref().is_err_and(lacks) {
            return Ok(Took::NotYet);
        }
        let judged = judged.map_err(|_| Kept::Refused);
        // The owned rows it changes, by Eudoxia's rule (`owned.rs`): one that
        // waits for the record's cut of a row's person is dropped as one not
        // bound yet is, and sent again by marks once the Remove lands here.
        let Some(judged) = owned::kept(judged, &self.tables, room.binds, device) else {
            return Ok(Took::NotYet);
        };
        if judged.is_ok() && !sigs.is_empty() {
            tracing::warn!(target: "state", group = self.group, app = self.app, device = %set.head.device, seq = set.head.seq.get(), "state: a second set at one site and seq, both taken: a copied or restored database runs beside its original");
        }
        // Rolled back and released as it drops on any `?`.
        let sp = tx.savepoint()?;
        let took = match judged {
            Ok(valid) => keep(&sp, (set, site), Some(&valid), Kept::Taken).map(|()| Took::Taken)?,
            Err(refused) => keep(&sp, (set, site), None, refused).map(|()| Took::Refused)?,
        };
        sp.commit()?;
        room.add(device, adds);
        let twin = took == Took::Taken && !sigs.is_empty();
        Ok(if twin { Took::Twin } else { took })
    }
}
