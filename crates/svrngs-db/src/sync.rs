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

/// One change set as it travels and is kept: its device, the nonce of the
/// database it was written in, its seq there, its schema's hash, its
/// changes, and the device's signature over them all (`At::message`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Set {
    pub device: Device,
    pub nonce: Nonce,
    pub seq: Seq,
    pub schema: [u8; 32],
    pub changes: Vec<u8>,
    pub sig: Sig,
}

/// A set's place: its site and its seq.
pub type Place = (Site, Seq);

/// What a copy holds, as it asks a member: each site's mark, every set of
/// the site up to it taken here, refused here for good, or vouched for by a
/// member pulled from in full; the sets counted above a mark, `haves`; and
/// each set it keeps at a place where it keeps two, `twins`, by signature.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ask {
    pub marks: BTreeMap<Site, Seq>,
    pub haves: BTreeSet<Place>,
    pub twins: BTreeSet<(Place, Sig)>,
}

/// What a member sends a copy: the sets the copy's ask says it lacks, the
/// member's own ask, and whether those are all it keeps that the copy lacks,
/// so the copy may adopt its marks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
    pub ask: Ask,
    pub sets: Vec<Set>,
    pub full: bool,
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
    /// Its device isn't in this copy's record yet: kept, held, counted, and
    /// judged again by [`Db::rejudge`].
    Waits,
    /// Refused for good, alike on every copy: counted, never applied.
    Refused,
    /// Its signature isn't its device's: dropped as if it never came, its
    /// mark unmoved.
    NotItsDevice,
    /// This copy couldn't apply it: its mark holds, and the next call
    /// fetches it again.
    Fault,
}

/// Whether the record binds a device's key, as [`Db::ingest`] is told.
type Bound<'b> = &'b dyn Fn(&str) -> bool;

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

/// `set`, at `site`, kept and counted, `held` while it waits; and, `rows`
/// given, merged into `crsql_changes` first as `site`'s at its seq, where
/// cr-sqlite decides what wins: only rows the parse made, so nothing it
/// didn't read reaches cr-sqlite.
fn keep(conn: &Connection, (set, site): (&Set, Site), rows: Option<&Valid>, held: bool) -> rusqlite::Result<()> {
    let put = "INSERT INTO crsql_changes (\"table\", pk, cid, val, col_version, db_version, site_id, cl, seq, ts) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '0')";
    for c in rows.map_or(&[][..], |v| &v.0) {
        let p = params![c.table, c.pk, c.cid, c.val, c.col_version, set.seq, site, c.cl, c.seq];
        conn.prepare_cached(put)?.execute(p)?;
    }
    let bytes = postcard::to_allocvec(set).expect("a set is written in memory");
    let sql = "INSERT OR IGNORE INTO svrngs_sets (site, seq, sig, held, bytes) VALUES (?1, ?2, ?3, ?4, ?5)";
    conn.execute(sql, params![site, set.seq, set.sig, held, bytes])?;
    count(conn, (site, set.seq))
}

impl Db {
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
        let reused = "SELECT EXISTS (SELECT 1 FROM svrngs_sets WHERE site = ?1 AND seq = ?2)";
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
        let sig = self
            .at((&self.device, &self.nonce, seq, &self.schema))
            .sign(&self.key, &changes);
        let sig = Sig::parse(&sig).expect("a signature is written in lower-case hex");
        let (device, nonce, schema) = (self.device.clone(), self.nonce, self.schema);
        let set = Set {
            device,
            nonce,
            seq,
            schema,
            changes,
            sig,
        };
        keep(conn, (&set, self.site), None, false).map_err(fault)?;
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
        let (marks, haves, twins) = (
            marks.into_iter().collect(),
            haves.into_iter().collect(),
            twins.into_iter().collect(),
        );
        Ok(Ask { marks, haves, twins })
    }

    /// What a member whose ask is `theirs` lacks of the sets this copy
    /// keeps, in (site, seq) order, stopped at `budget` bytes of changes:
    /// each above the member's mark for its site that it doesn't name among
    /// its haves, and each at a place where either keeps two that the
    /// member's ask doesn't name; with this copy's ask, and whether it is all
    /// of them.
    pub fn missing(&self, theirs: &Ask, budget: usize) -> Result<Call, Refused> {
        let ask = self.ask()?;
        let twins: BTreeSet<&Place> = ask.twins.iter().chain(&theirs.twins).map(|(p, _)| p).collect();
        let named: BTreeSet<&Sig> = theirs.twins.iter().map(|(_, s)| s).collect();
        let kept = "SELECT site, seq, sig, bytes FROM svrngs_sets ORDER BY site, seq, sig";
        let kept = rows(&self.conn(), kept, [], |r| {
            Ok(((r.get(0)?, r.get(1)?), r.get(2)?, r.get(3)?))
        });
        let (mut sets, mut spent) = (Vec::new(), 0);
        for (place, sig, bytes) in kept.map_err(fault)? {
            let (site, seq): Place = place;
            let above = theirs.marks.get(&site).is_none_or(|m| seq > *m) && !theirs.haves.contains(&place);
            let (twin, sig): (bool, Sig) = (twins.contains(&place), sig);
            if !above && (!twin || named.contains(&sig)) {
                continue;
            }
            let bytes: Vec<u8> = bytes;
            if !sets.is_empty() && spent + bytes.len() > budget {
                return Ok(Call { ask, sets, full: false });
            }
            spent += bytes.len();
            sets.push(postcard::from_bytes(&bytes).map_err(fault)?);
        }
        Ok(Call { ask, sets, full: true })
    }

    /// `call` taken in one transaction: its sets in (device, nonce, seq)
    /// order, each ending one of the four ways ([`Took`]); then, for a full
    /// call, the member's mark of each site none of whose sets faulted here,
    /// failed its signature or waits here. Each set's place and how it
    /// ended. `bound` says whether the record binds a device's key.
    pub fn ingest(&self, call: &Call, bound: &dyn Fn(&str) -> bool) -> Result<Vec<(Place, Took)>, Refused> {
        let mut sets: Vec<&Set> = call.sets.iter().collect();
        sets.sort_by(|a, b| (&a.device, a.nonce, a.seq).cmp(&(&b.device, b.nonce, b.seq)));
        let mut conn = self.conn();
        // Rolled back as it drops on any `?` below, so a call that fails
        // leaves no transaction open (rusqlite 0.41 `transaction.rs:243`).
        let mut tx = conn.transaction_with_behavior(Immediate).map_err(fault)?;
        let (mut out, mut stopped) = (Vec::new(), BTreeSet::new());
        for set in sets {
            let place = (site(&self.group, &self.app, set.device.as_str(), &set.nonce), set.seq);
            let took = self.take(&mut tx, (set, place.0), bound).unwrap_or_else(|e| {
                tracing::warn!(target: "state", group = self.group, app = self.app, device = %set.device, seq = set.seq.get(), %e, "state: a set this copy couldn't apply, its mark held");
                Took::Fault
            });
            tracing::debug!(target: "state", group = self.group, app = self.app, device = %set.device, seq = set.seq.get(), ?took, "state: a set offered, and how it ended");
            if matches!(took, Took::Waits | Took::NotItsDevice | Took::Fault) {
                stopped.insert(place.0);
            }
            out.push((place, took));
        }
        let waiting = rows(&tx, "SELECT site FROM svrngs_sets WHERE held", [], |r| {
            r.get::<_, Site>(0)
        });
        stopped.extend(waiting.map_err(fault)?);
        let adopting = call
            .ask
            .marks
            .iter()
            .filter(|(s, _)| call.full && !stopped.contains(*s));
        tracing::debug!(target: "state", group = self.group, app = self.app, full = call.full, stopped = stopped.len(), "state: a full call's marks adopted, but each site's with a set that faulted here, failed its signature or waits");
        let adopted = adopting.into_iter().try_for_each(|(s, m)| adopt(&tx, (*s, m.0 as i64)));
        adopted.and_then(|()| tx.commit()).map_err(fault)?;
        let faulted = out.iter().any(|(_, t)| *t == Took::Fault);
        let first = call.full && !faulted && !self.synced.swap(true, Relaxed);
        if first {
            let kept = conn.execute("UPDATE svrngs_db SET synced = 1", []);
            tracing::debug!(target: "state", group = self.group, app = self.app, kept = kept.is_ok(), "state: a copy's first full sync, so it is said syncing no more, and its pages told");
        }
        if call.full || faulted {
            self.faulted.store(faulted, Relaxed);
        }
        if first || out.iter().any(|(_, t)| matches!(t, Took::Taken | Took::Twin)) {
            self.changed.send_modify(|n| *n += 1);
        }
        Ok(out)
    }

    /// `set`, at `site`, offered to this copy within the call's transaction
    /// `tx` (the spike's `take`). Whatever it writes, it writes in a
    /// savepoint of its own, released only once all of it is written, so a
    /// set is kept and counted together or not at all.
    fn take(&self, tx: &mut Transaction, (set, site): (&Set, Site), bound: Bound) -> rusqlite::Result<Took> {
        // Its signature first, before anything in it is read.
        let at = self.at((&set.device, &set.nonce, set.seq, &set.schema));
        if !at.check(&set.changes, set.sig.as_str()) {
            return Ok(Took::NotItsDevice);
        }
        let sigs = "SELECT sig FROM svrngs_sets WHERE site = ?1 AND seq = ?2";
        let sigs: Vec<Sig> = rows(tx, sigs, params![site, set.seq], |r| r.get(0))?;
        let counted = "SELECT ?2 <= coalesce((SELECT mark FROM svrngs_marks WHERE site = ?1), 0) \
            OR EXISTS (SELECT 1 FROM svrngs_above WHERE site = ?1 AND seq = ?2)";
        let counted: bool = tx.query_row(counted, params![site, set.seq], |r| r.get(0))?;
        if sigs.contains(&set.sig) || counted && sigs.is_empty() {
            return Ok(Took::Held);
        }
        // One whose device the record doesn't bind, or written under a schema
        // this copy doesn't hold, waits, unparsed: its changes are read only
        // against the tables of its own schema (APP_STATE §4, the four
        // outcomes). Any other is parsed before cr-sqlite sees it.
        let waits = !bound(set.device.as_str()) || set.schema != self.schema;
        let parsed = (!waits).then(|| parse(&set.changes, &self.tables));
        if matches!(parsed, Some(Ok(_))) && !sigs.is_empty() {
            tracing::warn!(target: "state", group = self.group, app = self.app, device = %set.device, seq = set.seq.get(), "state: a second set at one site and seq, both taken: a copied or restored database runs beside its original");
        }
        // Rolled back and released as it drops on any `?`.
        let sp = tx.savepoint()?;
        let took = match parsed {
            None => keep(&sp, (set, site), None, true).map(|()| Took::Waits)?,
            Some(Err(_)) => count(&sp, (site, set.seq)).map(|()| Took::Refused)?,
            Some(Ok(valid)) => keep(&sp, (set, site), Some(&valid), false).map(|()| Took::Taken)?,
        };
        sp.commit()?;
        let twin = took == Took::Taken && !sigs.is_empty();
        Ok(if twin { Took::Twin } else { took })
    }

    /// Each set waiting here whose device the record now binds, as `bound`
    /// says, and whose schema this copy holds, judged again: parsed, and
    /// applied and no longer held, or refused for good, kept no more and
    /// still counted. How many it applied.
    pub fn rejudge(&self, bound: &dyn Fn(&str) -> bool) -> Result<usize, Refused> {
        let mut conn = self.conn();
        let held = rows(&conn, "SELECT bytes FROM svrngs_sets WHERE held", [], |r| {
            r.get::<_, Vec<u8>>(0)
        });
        let held = held
            .map_err(fault)?
            .into_iter()
            .filter_map(|b| postcard::from_bytes::<Set>(&b).ok());
        let mut taken = 0;
        for set in held.filter(|s| bound(s.device.as_str()) && s.schema == self.schema) {
            let place = (site(&self.group, &self.app, set.device.as_str(), &set.nonce), set.seq);
            let parsed = parse(&set.changes, &self.tables);
            let unheld = "UPDATE svrngs_sets SET held = 0 WHERE site = ?1 AND seq = ?2 AND sig = ?3";
            let gone = "DELETE FROM svrngs_sets WHERE site = ?1 AND seq = ?2 AND sig = ?3";
            let at = params![place.0, place.1, set.sig];
            // Rolled back and released as it drops, unless all of it is written.
            let sp = conn.savepoint().map_err(fault)?;
            let judged = match &parsed {
                Ok(valid) => keep(&sp, (&set, place.0), Some(valid), false).and_then(|()| sp.execute(unheld, at)),
                Err(_) => sp.execute(gone, at),
            };
            let judged = judged.and_then(|_| sp.commit());
            let applied = judged.is_ok() && parsed.is_ok();
            tracing::debug!(target: "state", group = self.group, app = self.app, device = %set.device, seq = set.seq.get(), applied, refused = parsed.is_err(), written = judged.is_ok(), "state: a set that waited for its device, now bound, applied; refused for good as it doesn't parse; or held still, as this copy couldn't");
            taken += usize::from(applied);
        }
        if taken > 0 {
            self.changed.send_modify(|n| *n += 1);
        }
        Ok(taken)
    }
}
