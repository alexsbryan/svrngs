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
//! Owned rows are checked at the write and at apply in slice D, both at once
//! (MAGNETS_BUILD §3): checked at apply alone, a set the writer's copy took
//! would be refused on every other copy.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::Ordering::Relaxed;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, OptionalExtension as _, Row, Transaction, TransactionBehavior::Immediate, params};
use serde::{Deserialize, Serialize};
use svrngs_core::set::{At, Nonce, Site, site};

use crate::parse::{Change, Valid, fit, parse};
use crate::{Db, Refused, fault};

/// A set's seq at its database, and a mark, a have or a twin's place: from 1
/// to `i64::MAX`, what SQLite holds. Parsed as the wire is read, so a call
/// naming one past it doesn't decode, and no seq reaches a bind that rusqlite
/// would refuse (a `u64` past `i64::MAX`), or an add past it. Written as
/// its number, as postcard writes a newtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u64")]
pub struct Seq(u64);

impl Seq {
    /// `n` as a seq, if it is one.
    pub fn new(n: u64) -> Option<Self> {
        (1..=i64::MAX as u64).contains(&n).then_some(Self(n))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for Seq {
    type Error = String;
    fn try_from(n: u64) -> Result<Self, String> {
        Self::new(n).ok_or_else(|| format!("{n} is no seq: one is from 1 to {}", i64::MAX))
    }
}

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

/// `N` bytes in lower-case hex, as the record spells a key (`actor_of`) and
/// a signature: parsed as the wire is read, so a device is spelled as the
/// record spells it, and one signature never has two spellings. Written as
/// its text, as postcard writes a newtype.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Hex<const N: usize>(String);

/// A device's key, as the record spells it.
pub type Device = Hex<32>;

/// A device's signature over a set.
pub type Sig = Hex<64>;

impl<const N: usize> Hex<N> {
    /// `text` as `N` bytes of lower-case hex, if it is.
    pub fn parse(text: &str) -> Option<Self> {
        let hex = text.len() == 2 * N && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        hex.then(|| Self(text.into()))
    }
}

svrngs_core::text_newtype!(Device, Sig);

impl<const N: usize> TryFrom<String> for Hex<N> {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        Self::parse(&text).ok_or_else(|| format!("not {N} bytes of lower-case hex"))
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

/// A set's signed head: its device, the nonce of the database it was
/// written in, its seq there, its schema's hash, its changes' hash
/// (`svrngs_core::set::changes`), and the device's signature over them all
/// (`At::message`). Ordered as its fields are, so by device, nonce and seq
/// first.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Head {
    pub device: Device,
    pub nonce: Nonce,
    pub seq: Seq,
    pub schema: [u8; 32],
    pub changes: [u8; 32],
    pub sig: Sig,
}

/// One change set as it travels and is kept: its signed head, and the
/// changes whose hash it signs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Set {
    pub head: Head,
    pub changes: Vec<u8>,
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
/// two ways it ends being here already).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Took {
    /// Applied and kept, counted in its site's mark.
    Taken,
    /// A second set at a place where this copy keeps one: taken too, and
    /// named in the ask, since a mark that counts the place fetches neither.
    Twin,
    /// Counted here already: passed by.
    Held,
    /// Its device isn't one this copy's record binds yet: dropped as if it
    /// never came, its mark unmoved, so a member sends it again once the
    /// record here binds it (the record's exchange runs first in a dial).
    NotYet,
    /// Written under a schema this copy doesn't hold: kept, held, counted,
    /// never sent on.
    Waits,
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

/// The keys a group's record binds, standing or cut, as a set's device is
/// judged by them (APP_STATE §4, "A removed device"); and the room a call's
/// ask may fill beside its sets. Every key the record binds has an equal
/// share of the room ([`Self::share`]), so no one device can fill what every
/// copy's ask may carry and stall the app's sync for everyone.
pub struct Binds {
    keys: BTreeSet<String>,
    room: usize,
}

impl Binds {
    pub fn new(keys: impl IntoIterator<Item = String>, room: usize) -> Self {
        Self {
            keys: keys.into_iter().collect(),
            room,
        }
    }

    /// Whether the record binds `device`.
    fn binds(&self, device: &Device) -> bool {
        self.keys.contains(device.as_str())
    }

    /// Each one's share of the room, in rows of at most [`ROW_MOST`] bytes:
    /// so however full, the ask is at most the room.
    fn share(&self) -> u64 {
        (self.room / ROW_MOST / self.keys.len().max(1)) as u64
    }
}

/// What a call may still add to this copy's ask as it is taken, by the
/// keys the record binds: the rows each device holds ([`Db::room`]), and
/// the devices muted here.
struct Room<'b> {
    binds: &'b Binds,
    rows: BTreeMap<Device, u64>,
    muted: BTreeSet<Device>,
}

impl Room<'_> {
    /// Whether a set of `device`'s has room for the `adds` rows it brings:
    /// unmuted, in its share with one row left for its name should it be
    /// muted.
    fn fits(&self, device: &Device, adds: u64) -> bool {
        let held = self.rows.get(device).copied().unwrap_or(0);
        !self.muted.contains(device) && (adds == 0 || held + adds < self.binds.share())
    }

    fn add(&mut self, device: &Device, adds: u64) {
        *self.rows.entry(device.clone()).or_default() += adds;
    }
}

/// How a set is kept here, as `svrngs_sets.kept` says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kept {
    /// Applied, and sent to a member that lacks it.
    Taken,
    /// Refused for good: its place and signature alone, no bytes, so a
    /// second set at its place is judged, never passed by as held.
    Refused,
    /// Waiting for this copy to hold the schema it was written under.
    Unschemed,
}

impl ToSql for Kept {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok((*self as i64).into())
    }
}

/// `sql`'s rows on `conn`, each read by `f`.
fn rows<T>(
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
/// parse made, so nothing it didn't read reaches cr-sqlite.
fn keep(conn: &Connection, (set, site): (&Set, Site), rows: Option<&Valid>, kept: Kept) -> rusqlite::Result<()> {
    let put = "INSERT INTO crsql_changes (\"table\", pk, cid, val, col_version, db_version, site_id, cl, seq, ts) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '0')";
    let seq = set.head.seq;
    for c in rows.map_or(&[][..], |v| &v.0) {
        let p = params![c.table, c.pk, c.cid, c.val, c.col_version, seq, site, c.cl, c.seq];
        conn.prepare_cached(put)?.execute(p)?;
    }
    let bytes = (kept != Kept::Refused).then(|| postcard::to_allocvec(set).expect("a set is written in memory"));
    let sql = "INSERT OR IGNORE INTO svrngs_sets (site, seq, sig, kept, bytes) VALUES (?1, ?2, ?3, ?4, ?5)";
    conn.execute(sql, params![site, seq, set.head.sig, kept, bytes])?;
    // The site's device, by which its share is counted, a muted device's
    // sets are found and a word for its mark is trusted.
    let sited = "INSERT OR IGNORE INTO svrngs_sites (site, device) VALUES (?1, ?2)";
    conn.execute(sited, params![site, set.head.device])?;
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
    /// site's own computer, once that copy counts a set above it. A set that
    /// waits, or was refused, holds no value here and is kept as it is. How
    /// many it dropped.
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

    /// What a set of this database's group and app, written by `device` in
    /// its database of `nonce` at `seq` under `schema`, is signed as.
    fn at<'s>(&'s self, (device, nonce, seq, schema): (&'s Device, &'s Nonce, Seq, &'s [u8; 32])) -> At<'s> {
        At {
            group: &self.group,
            app: &self.app,
            device: device.as_str(),
            nonce,
            seq: seq.get(),
            schema,
        }
    }

    /// Whether `set`'s head is signed by its device, over this database's
    /// group and app, and its changes are the ones it signs.
    fn signed(&self, set: &Set) -> bool {
        let head = &set.head;
        let at = self.at((&head.device, &head.nonce, head.seq, &head.schema));
        at.check(&head.changes, head.sig.as_str()) && svrngs_core::set::changes(&set.changes) == head.changes
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
    /// would refuse the set, so the writer's copy never takes what no other
    /// copy would.
    pub(crate) fn seal(&self, conn: &Connection) -> Result<Option<Seq>, Refused> {
        let seq: Seq = conn
            .query_row("SELECT crsql_peek_next_db_version()", [], |r| r.get(0))
            .map_err(fault)?;
        let changes = "SELECT \"table\", pk, cid, val, col_version, cl, seq FROM crsql_changes \
            WHERE db_version = ?1 AND site_id = crsql_site_id() ORDER BY seq";
        let made = rows(conn, changes, [seq], |r| {
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
        });
        let made = made.map_err(fault)?;
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
        let changes = postcard::to_allocvec(&made.0).expect("changes are written in memory");
        let hash = svrngs_core::set::changes(&changes);
        let sig = self
            .at((&self.device, &self.nonce, seq, &self.schema))
            .sign(&self.key, &hash);
        let sig = Sig::parse(&sig).expect("a signature is written in lower-case hex");
        let (device, nonce, schema) = (self.device.clone(), self.nonce, self.schema);
        let head = Head {
            device,
            nonce,
            seq,
            schema,
            changes: hash,
            sig,
        };
        keep(conn, (&Set { head, changes }, self.site), None, Kept::Taken).map_err(fault)?;
        self.prune(conn).map_err(fault)?;
        Ok(Some(seq))
    }

    /// What this copy holds, as it asks a member ([`Ask`]).
    pub fn ask(&self) -> Result<Ask, Refused> {
        let conn = self.conn();
        let place = |r: &Row| Ok((r.get(0)?, r.get(1)?));
        let marks = rows(&conn, "SELECT site, mark FROM svrngs_marks", [], place).map_err(fault)?;
        let haves = rows(&conn, "SELECT site, seq FROM svrngs_above", [], place).map_err(fault)?;
        let twins = "SELECT site, seq, sig FROM svrngs_sets WHERE (site, seq) IN \
            (SELECT site, seq FROM svrngs_sets GROUP BY site, seq HAVING count(*) > 1)";
        let twins = rows(&conn, twins, [], |r| Ok((place(r)?, r.get(2)?))).map_err(fault)?;
        let muted = rows(&conn, "SELECT device FROM svrngs_muted", [], |r| r.get(0)).map_err(fault)?;
        let (marks, haves, twins) = (
            marks.into_iter().collect(),
            haves.into_iter().collect(),
            twins.into_iter().collect(),
        );
        let muted = muted.into_iter().collect();
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
    /// member's ask doesn't name; none that waits here or was refused, nor
    /// any of a device the member's ask names muted; with this copy's ask,
    /// and whether it is all of them. A set that waits is sent by its own
    /// device's copy, or one that took it, never by one where it waits.
    pub fn missing(&self, theirs: &Ask, budget: usize) -> Result<Call, Refused> {
        let ask = self.ask()?;
        let twins: BTreeSet<&Place> = ask.twins.iter().chain(&theirs.twins).map(|(p, _)| p).collect();
        let named: BTreeSet<&Sig> = theirs.twins.iter().map(|(_, s)| s).collect();
        let kept = "SELECT s.site, s.seq, s.sig, s.bytes, h.device FROM svrngs_sets s JOIN svrngs_sites h \
            USING (site) WHERE s.kept = ?1 ORDER BY s.site, s.seq, s.sig";
        let kept = rows(&self.conn(), kept, [Kept::Taken], |r| {
            Ok(((r.get(0)?, r.get(1)?), r.get(2)?, r.get(3)?, r.get(4)?))
        });
        let (mut sets, mut spent, mut full) = (Vec::new(), 0, true);
        for (place, sig, bytes, device) in kept.map_err(fault)? {
            if theirs.muted.contains::<Device>(&device) {
                continue;
            }
            let (site, seq): Place = place;
            let above = theirs.marks.get(&site).is_none_or(|m| seq > *m) && !theirs.haves.contains(&place);
            let (twin, sig): (bool, Sig) = (twins.contains(&place), sig);
            if !above && (!twin || named.contains(&sig)) {
                continue;
            }
            let bytes: Vec<u8> = bytes;
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
    /// set that faulted here, failed its signature or waits here. Each set's
    /// place and how it ended. `bound` says whether the record binds a
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
        for set in sets {
            let place = (self.site_of(&set.head), set.head.seq);
            let took = self.take(&mut tx, (set, place.0), &mut room).unwrap_or_else(|e| {
                tracing::warn!(target: "state", group = self.group, app = self.app, device = %set.head.device, seq = set.head.seq.get(), %e, "state: a set this copy couldn't apply, its mark held");
                Took::Fault
            });
            tracing::debug!(target: "state", group = self.group, app = self.app, device = %set.head.device, seq = set.head.seq.get(), ?took, "state: a set offered, and how it ended");
            if matches!(
                took,
                Took::NotYet | Took::Waits | Took::NotItsDevice | Took::Fault | Took::Over
            ) {
                stopped.insert(place.0);
            }
            out.push((place, took));
        }
        let waiting = "SELECT site FROM svrngs_sets WHERE kept = ?1";
        let waiting = rows(&tx, waiting, [Kept::Unschemed], |r| r.get::<_, Site>(0));
        stopped.extend(waiting.map_err(fault)?);
        let marks = call.ask.marks.iter();
        let adopting = marks.filter(|(s, _)| call.full && **s != self.site && !stopped.contains(*s));
        tracing::debug!(target: "state", group = self.group, app = self.app, full = call.full, stopped = stopped.len(), "state: a full call's marks adopted as far as its word goes, but for this copy's own site, and each site's with a set that faulted here, failed its signature, isn't bound here yet or waits");
        for (s, m) in adopting {
            self.claimed(&tx, (*s, *m), (from, call.synced, &room)).map_err(fault)?;
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

    /// What a call may add to this copy's ask, as it begins ([`Room`]).
    fn room<'b>(&self, conn: &Connection, binds: &'b Binds) -> rusqlite::Result<Room<'b>> {
        let shares = "SELECT h.device, count(*) FROM (SELECT site FROM svrngs_sites UNION ALL SELECT site FROM svrngs_above \
            UNION ALL SELECT site FROM svrngs_sets s WHERE EXISTS (SELECT 1 FROM svrngs_sets t WHERE t.site = s.site \
            AND t.seq = s.seq AND t.sig <> s.sig)) AS r JOIN svrngs_sites h USING (site) GROUP BY h.device";
        let held = BTreeMap::from_iter(rows(conn, shares, [], |r| Ok((r.get(0)?, r.get(1)?)))?);
        let muted = BTreeSet::from_iter(rows(conn, "SELECT device FROM svrngs_muted", [], |r| r.get(0))?);
        let rows = held;
        Ok(Room { binds, rows, muted })
    }

    /// `set`, past its share of what this copy's ask may carry, dropped as if
    /// it never came; and its device muted here from then on, named in this
    /// copy's ask so that no member sends it more, and no more of its sets
    /// taken here. Its sets before stand. The seat's decision (svrn-docs
    /// APP_STATE Decisions, 2026-11-22): no one device can fill what every
    /// copy's ask may carry, and so stall the app's sync for everyone.
    fn over(&self, tx: &Transaction, set: &Set, room: &mut Room) -> rusqlite::Result<Took> {
        let (device, share) = (&set.head.device, room.binds.share());
        tracing::warn!(target: "state", group = self.group, app = self.app, %device, share, "state: a device past its share of what this copy's ask may carry, so its set is dropped, and the device muted here, named in the ask so no member sends it more; its sets before stand");
        let muted = "INSERT OR IGNORE INTO svrngs_muted (device) VALUES (?1)";
        if room.muted.insert(device.clone()) {
            tx.execute(muted, [device])?;
        }
        Ok(Took::Over)
    }

    /// The mark `m` a full call claims for `site`, adopted up to the highest
    /// seq of the site this copy has counted and no further, so it fills the
    /// gaps below what this copy holds and passes nothing it hasn't counted;
    /// and only where its device is unmuted here, and the caller is trusted
    /// for it: the site's own computer, which vouches for its own sets, or a
    /// synced caller into a copy not yet synced, as a newcomer trusts its
    /// source (APP_STATE §4, "What travels"). So no word takes a mark past a
    /// set this copy would fetch, and none makes an established copy skip a
    /// set it hasn't fetched yet.
    fn claimed(
        &self,
        tx: &Transaction,
        (site, m): (Site, Seq),
        (from, synced, room): (Option<&Device>, bool, &Room),
    ) -> rusqlite::Result<()> {
        let top = "SELECT device, max(coalesce((SELECT mark FROM svrngs_marks WHERE site = ?1), 0), \
            coalesce((SELECT max(seq) FROM svrngs_above WHERE site = ?1), 0)) FROM svrngs_sites WHERE site = ?1";
        let top = tx.query_row(top, [site], |r| Ok((r.get::<_, Device>(0)?, r.get::<_, i64>(1)?)));
        let Some((device, top)) = top.optional()? else {
            return Ok(());
        };
        let trusted = from == Some(&device) || synced && !self.synced.load(Relaxed);
        let muted = room.muted.contains(&device);
        tracing::debug!(target: "state", group = self.group, app = self.app, %device, mark = m.get(), top, trusted, muted, "state: a full call's mark for a site, adopted up to the highest seq of it this copy has counted, where its device is unmuted here and its caller is trusted for it");
        if !trusted || muted {
            return Ok(());
        }
        adopt(tx, (site, (m.get() as i64).min(top)))
    }

    /// `set`, at `site`, offered to this copy within the call's transaction
    /// `tx` (the spike's `take`). Whatever it writes, it writes in a
    /// savepoint of its own, released only once all of it is written, so a
    /// set is kept and counted together or not at all.
    fn take(&self, tx: &mut Transaction, (set, site): (&Set, Site), room: &mut Room) -> rusqlite::Result<Took> {
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
        if sigs.contains(&set.head.sig) || counted && sigs.is_empty() {
            return Ok(Took::Held);
        }
        // What it adds to this copy's ask: a site new here, a set counted
        // above a gap, a set at a place that keeps one, the first making two
        // rows of it. Past its device's share, it is dropped ([`Self::over`]).
        let device = &set.head.device;
        let adds = u64::from(!sited) + u64::from(set.head.seq.0 > mark + 1) + [0, 2, 1][sigs.len().min(2)];
        if !room.fits(device, adds) {
            return self.over(tx, set, room);
        }
        // One written under a schema this copy doesn't hold waits, unparsed:
        // its changes are read only against the tables of its own schema
        // (APP_STATE §4, the four outcomes). Any other is parsed before
        // cr-sqlite sees it.
        let waits = (set.head.schema != self.schema).then_some(Kept::Unschemed);
        let judged = waits.map_or_else(|| parse(&set.changes, &self.tables).map_err(|_| Kept::Refused), Err);
        if judged.is_ok() && !sigs.is_empty() {
            tracing::warn!(target: "state", group = self.group, app = self.app, device = %set.head.device, seq = set.head.seq.get(), "state: a second set at one site and seq, both taken: a copied or restored database runs beside its original");
        }
        // Rolled back and released as it drops on any `?`.
        let sp = tx.savepoint()?;
        let took = match judged {
            Ok(valid) => keep(&sp, (set, site), Some(&valid), Kept::Taken).map(|()| Took::Taken)?,
            Err(Kept::Refused) => keep(&sp, (set, site), None, Kept::Refused).map(|()| Took::Refused)?,
            Err(waits) => keep(&sp, (set, site), None, waits).map(|()| Took::Waits)?,
        };
        sp.commit()?;
        room.add(device, adds);
        let twin = took == Took::Taken && !sigs.is_empty();
        Ok(if twin { Took::Twin } else { took })
    }
}
