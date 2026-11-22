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

use rusqlite::types::Value as Sql;
use rusqlite::{Connection, OptionalExtension as _, Row, Transaction, TransactionBehavior::Immediate, params};
use serde::{Deserialize, Serialize};
use svrngs_core::set::{At, Nonce, Site, site};

use crate::{Db, Refused, fault};

/// A `col_version` or a causal length past this is refused for good: one at
/// `i64::MAX` freezes its cell (svrn-docs
/// `app-state-diligence/engine-and-sync.md`, C3), and every copy reads the
/// bound alike from the set.
const BOUND: i64 = 1 << 53;

/// One change set as it travels and is kept: its device, the nonce of the
/// database it was written in, its seq there, its schema's hash, its
/// changes, and the device's signature over them all (`At::message`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Set {
    pub device: String,
    pub nonce: Nonce,
    pub seq: u64,
    pub schema: [u8; 32],
    pub changes: Vec<u8>,
    pub sig: String,
}

/// A set's place: its site and its seq.
pub type Place = (Site, u64);

/// What a copy holds, as it asks a member: each site's mark, every set of
/// the site up to it taken here, refused here for good, or vouched for by a
/// member pulled from in full; the sets counted above a mark, `haves`; and
/// each set it keeps at a place where it keeps two, `twins`, by signature.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ask {
    pub marks: BTreeMap<Site, u64>,
    pub haves: BTreeSet<Place>,
    pub twins: BTreeSet<(Place, String)>,
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

/// One change row of a set, as `crsql_changes` carries it, less the site
/// and version its set names once.
#[derive(Serialize, Deserialize)]
struct Change {
    table: String,
    pk: Vec<u8>,
    cid: String,
    #[serde(with = "Value")]
    val: Sql,
    col_version: i64,
    cl: i64,
    seq: i64,
}

/// rusqlite's value, as postcard writes it.
#[derive(Serialize, Deserialize)]
#[serde(remote = "Sql")]
enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
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
/// set counted just above it; those at or below it no longer kept apart.
fn adopt(conn: &Connection, (site, theirs): Place) -> rusqlite::Result<()> {
    let mark = conn.query_row("SELECT mark FROM svrngs_marks WHERE site = ?1", [site], |r| r.get(0));
    let mut m = mark.optional()?.unwrap_or(0).max(theirs);
    let next = "DELETE FROM svrngs_above WHERE site = ?1 AND seq = ?2";
    while conn.execute(next, params![site, m + 1])? == 1 {
        m += 1;
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
/// cr-sqlite decides what wins.
fn keep(conn: &Connection, (set, site): (&Set, Site), rows: Option<&[Change]>, held: bool) -> rusqlite::Result<()> {
    let put = "INSERT INTO crsql_changes (\"table\", pk, cid, val, col_version, db_version, site_id, cl, seq, ts) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '0')";
    for c in rows.unwrap_or_default() {
        let p = params![c.table, c.pk, c.cid, c.val, c.col_version, set.seq, site, c.cl, c.seq];
        conn.prepare_cached(put)?.execute(p)?;
    }
    let bytes = postcard::to_allocvec(set).expect("a set is written in memory");
    let sql = "INSERT OR IGNORE INTO svrngs_sets (site, seq, sig, held, bytes) VALUES (?1, ?2, ?3, ?4, ?5)";
    conn.execute(sql, params![site, set.seq, set.sig, held, bytes])?;
    count(conn, (site, set.seq))
}

impl Db {
    /// What `set`, of this database's group and app, is signed as.
    fn at<'s>(&'s self, set: &'s Set) -> At<'s> {
        At {
            group: &self.group,
            app: &self.app,
            device: &set.device,
            nonce: &set.nonce,
            seq: set.seq,
            schema: &set.schema,
        }
    }

    /// Within a write's transaction on `conn`: its `crsql_changes` rows at
    /// this database's site, sealed as one set at the transaction's version,
    /// signed by this computer's key, kept and counted; none when it changed
    /// nothing. Refused, loudly, at a version this site's sets already hold.
    pub(crate) fn seal(&self, conn: &Connection) -> Result<Option<u64>, Refused> {
        let seq: u64 = conn
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
        tracing::debug!(target: "state", group = self.group, app = self.app, seq, rows = made.len(), reused, "state: a write's rows sealed as one set at its version, none when it changed nothing, or refused at a version this site's sets hold");
        if reused {
            tracing::error!(target: "state", group = self.group, app = self.app, seq, "state: cr-sqlite gave a write a version this database's own sets already hold, so the write is refused");
            return Err(Refused::Fault(format!(
                "this database gave a write the version {seq} again"
            )));
        }
        if made.is_empty() {
            return Ok(None);
        }
        let changes = postcard::to_allocvec(&made).expect("changes are written in memory");
        let (device, nonce, schema, sig) = (self.device.clone(), self.nonce, self.schema, String::new());
        let mut set = Set {
            device,
            nonce,
            seq,
            schema,
            changes,
            sig,
        };
        set.sig = self.at(&set).sign(&self.key, &set.changes);
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
        let named: BTreeSet<&String> = theirs.twins.iter().map(|(_, s)| s).collect();
        let kept = "SELECT site, seq, sig, bytes FROM svrngs_sets ORDER BY site, seq, sig";
        let kept = rows(&self.conn(), kept, [], |r| {
            Ok(((r.get(0)?, r.get(1)?), r.get(2)?, r.get(3)?))
        });
        let (mut sets, mut spent) = (Vec::new(), 0);
        for (place, sig, bytes) in kept.map_err(fault)? {
            let (site, seq): Place = place;
            let above = seq > theirs.marks.get(&site).copied().unwrap_or(0) && !theirs.haves.contains(&place);
            let (twin, sig): (bool, String) = (twins.contains(&place), sig);
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
            let place = (site(&self.group, &self.app, &set.device, &set.nonce), set.seq);
            let took = self.take(&mut tx, (set, place.0), bound).unwrap_or_else(|e| {
                tracing::warn!(target: "state", group = self.group, app = self.app, device = set.device, seq = set.seq, %e, "state: a set this copy couldn't apply, its mark held");
                Took::Fault
            });
            tracing::debug!(target: "state", group = self.group, app = self.app, device = set.device, seq = set.seq, ?took, "state: a set offered, and how it ended");
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
        let adopted = adopting.into_iter().try_for_each(|(s, m)| adopt(&tx, (*s, *m)));
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
        let sigs = "SELECT sig FROM svrngs_sets WHERE site = ?1 AND seq = ?2";
        let sigs: Vec<String> = rows(tx, sigs, params![site, set.seq], |r| r.get(0))?;
        let counted = "SELECT ?2 <= coalesce((SELECT mark FROM svrngs_marks WHERE site = ?1), 0) \
            OR EXISTS (SELECT 1 FROM svrngs_above WHERE site = ?1 AND seq = ?2)";
        let counted: bool = tx.query_row(counted, params![site, set.seq], |r| r.get(0))?;
        if sigs.contains(&set.sig) || counted && sigs.is_empty() {
            return Ok(Took::Held);
        }
        if !self.at(set).check(&set.changes, &set.sig) {
            return Ok(Took::NotItsDevice);
        }
        let bounded = |c: &Vec<Change>| c.iter().all(|c| c.col_version <= BOUND && c.cl <= BOUND);
        let parsed: Option<Vec<Change>> = postcard::from_bytes(&set.changes).ok().filter(bounded);
        if parsed.is_some() && !sigs.is_empty() {
            tracing::warn!(target: "state", group = self.group, app = self.app, device = set.device, seq = set.seq, "state: a second set at one site and seq, both taken: a copied or restored database runs beside its original");
        }
        // Rolled back and released as it drops on any `?`.
        let sp = tx.savepoint()?;
        let took = match parsed {
            None => count(&sp, (site, set.seq)).map(|()| Took::Refused)?,
            Some(_) if !bound(&set.device) => keep(&sp, (set, site), None, true).map(|()| Took::Waits)?,
            Some(rows) => keep(&sp, (set, site), Some(&rows), false).map(|()| Took::Taken)?,
        };
        sp.commit()?;
        let twin = took == Took::Taken && !sigs.is_empty();
        Ok(if twin { Took::Twin } else { took })
    }

    /// Each set waiting here whose device the record now binds, as `bound`
    /// says, applied and no longer held: judged again on each change to the
    /// record. How many.
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
        for set in held.filter(|s| bound(&s.device)) {
            let place = (site(&self.group, &self.app, &set.device, &set.nonce), set.seq);
            let rows: Vec<Change> = postcard::from_bytes(&set.changes).unwrap_or_default();
            let unheld = "UPDATE svrngs_sets SET held = 0 WHERE site = ?1 AND seq = ?2 AND sig = ?3";
            // Rolled back and released as it drops, unless all of it applied.
            let sp = conn.savepoint().map_err(fault)?;
            let applied = keep(&sp, (&set, place.0), Some(&rows), false);
            let applied = applied.and_then(|()| sp.execute(unheld, params![place.0, place.1, set.sig]));
            let applied = applied.and_then(|_| sp.commit());
            tracing::debug!(target: "state", group = self.group, app = self.app, device = set.device, seq = set.seq, applied = applied.is_ok(), "state: a set that waited for its device, now bound, applied; or held still, as this copy couldn't");
            taken += usize::from(applied.is_ok());
        }
        if taken > 0 {
            self.changed.send_modify(|n| *n += 1);
        }
        Ok(taken)
    }
}
