// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail/src/journal.rs@e1a43bcd2
//! [`RingJournal`] — the single-writer append log behind a ring namespace.
//!
//! Its own file because it and `RingRail` together put `lib.rs` into the
//! 800-1200 approach band (ARCH §3.1). Re-exported at
//! `commonwealth_rail::RingJournal`, so no caller moves.

use super::*;
// The rail stamps its own lines: it may not link `commonwealth-core` for a
// clock (Cargo.toml, and the [[forbid]] in quality/ARCH_LAYERS.toml).
use std::time::{SystemTime, UNIX_EPOCH};

/// One namespace's append-only journal, plus its roster.
///
/// The `Mutex` is the single-writer rule the journal format needs: `O_APPEND`
/// makes one `write(2)` atomic, and this makes sure there is one. It is also
/// what makes the next sequence number safe to compute — fold the log, take
/// the highest, add one — with no window for two appends to pick the same one.
pub(crate) struct RingJournal {
    namespace: String,
    dir: PathBuf,
    #[expect(
        clippy::disallowed_types,
        reason = "the journal's single-writer lock: O_APPEND makes one write atomic, this makes there be one"
    )]
    log: Mutex<Writer>,
}

/// The log, and what the two write doors need to know about it.
struct Writer {
    oplog: Oplog<SignedOp>,
    held: Held,
}

/// The facts [`RingJournal::append_all`] and [`RingJournal::ingest_all`]
/// read off the log: each actor's newest seq, with the lines held at it, and
/// the ids already held; and the digest's fold ([`DigestFold`]), which
/// [`RingJournal::digest`] and [`RingJournal::ops_missing_from_within`] answer
/// from.
///
/// svrngs (U2): each id is the one the act's content derives ([`derive_id`]),
/// never the `id` field its line carries. A held set keyed by the line's field
/// skipped a peer's act as already held when another line merely claimed its
/// id, so the act never landed (DESIGN §6). And an id is held once a line of
/// it verifies: the signature is not part of the id, so a forged copy held
/// first made every later genuine one "held" and dropped it, its copy apart
/// from its peers' (#189). A line that does not verify is kept by its id and
/// signature (`forged`), so the same bytes are not appended twice. The lines
/// at an actor's newest seq are kept, id and signature, because the door's
/// next act names one of them as its previous act; only a line whose
/// signature verifies counts toward it, so a forged line moves no member's
/// next seq (U7, #16).
///
/// Folded from the file and caught up from its tail before each write
/// ([`Oplog::read_since`]), so a write costs the lines appended since the last
/// one rather than the whole log: re-reading it per write made the kv drain
/// cost rows x log lines a tick, and a door write grow with the journal
/// (pc-rails-journal-linear). The file stays the source of truth: lines
/// another process appended are in the tail, and a file renamed over the path
/// (a restore from a backup) restarts the fold from the top.
///
/// The digest is caught up the same way, for the same reason: rebuilt from
/// the whole log, every signature checked again, it cost an idle round at 25
/// members and 10,000 acts most of its 73 s of processor (svrn-docs
/// `rings/current/THE_CEILING.md`, fix 1). An act's signature is checked
/// here as the tail brings it in, under [`Ed25519Verifier`], the scheme
/// every caller of the digest judged by.
///
/// And each signature that verified is kept, so it is checked once, as its
/// line enters, and not again by the offer ([`Checked`]): the offer read
/// whole checked every signature it held again, every exchange with a peer
/// not in step (THE_CEILING.md fix 2). A verdict is a function of the actor,
/// the id and the signature alone, so one kept cannot go stale whatever
/// happens to the file; what restarts the fold restarts this with it all
/// the same, and a file replaced from outside has its lines checked again.
///
/// And the register ([`RingRail::standing`]) is caught up the same way: the
/// acts that move membership, a handful among thousands, and every act of a
/// key a `Remove` names (U7), kept as the tail brings them in, and the
/// register last folded from them, with the seed it was folded from. A key
/// named for the first time has its acts already held added from the whole
/// log, after the tail's, so they are not in the log's order, and an act
/// of it after the `Remove` in the same tail is kept twice; the fold reads
/// them as a set (`ActSet::of`), so neither moves it. Read whole on every
/// connection, sync request and round, it was what an idle round still
/// paid for (26 reads of 10,000 acts, 5.5 s; scale.rs, ceiling-3's runs;
/// THE_CEILING.md fix 3).
/// An act that moves membership voids the standing; a seed unlike the one it
/// was folded from folds it again.
#[derive(Default)]
struct Held {
    cursor: crate::oplog::Cursor,
    newest: std::collections::HashMap<String, Newest>,
    ids: std::collections::HashSet<OpId>,
    forged: std::collections::HashSet<(OpId, String)>,
    digest: DigestFold,
    passed: Passed,
    membership: Vec<Op<SignedOp>>,
    /// The keys some held `Remove` names: their every act is kept in
    /// `membership` too, since a cut covers a key's acts along their chain
    /// (U7; `svrngs_core::standing_reads`).
    named: std::collections::HashSet<String>,
    standing: Option<(Roster, crate::register::Register)>,
}

/// Each line whose signature [`Ed25519Verifier`] passed as the tail brought
/// it in: its actor, the id its content derives and its signature, the three
/// the verifier judged.
type Passed = std::collections::HashSet<(String, OpId, String)>;

/// [`Ed25519Verifier`], answered from the signatures [`Held`] checked as each
/// line entered. A line this handle did not see enter is checked now, so it
/// answers as [`Ed25519Verifier`] does for every line, and trusts nothing it
/// did not check.
struct Checked<'a>(&'a Passed);

impl RingVerifier for Checked<'_> {
    fn name(&self) -> &'static str {
        "ed25519, each line checked once as it entered"
    }

    fn verify(&self, actor: &str, id: &OpId, sig_hex: &str) -> bool {
        if self.0.contains(&(actor.to_string(), id.clone(), sig_hex.to_string())) {
            return true;
        }
        let verified = Ed25519Verifier.verify(actor, id, sig_hex);
        tracing::debug!(actor, %id, verified,
            "ring rail: a signature not passed as its line entered, checked now");
        verified
    }
}

/// One actor's newest seq among the lines that actor signed, and each such
/// line held at it: the id its content derives and the signature it carries.
struct Newest {
    seq: u64,
    lines: Vec<(OpId, String)>,
}

impl Held {
    /// Fold in whatever the log gained since the last call; true when the
    /// fold began again from the top.
    fn catch_up(&mut self, oplog: &Oplog<SignedOp>, namespace: &str) -> Result<bool, RailError> {
        let tail = oplog
            .read_since(self.cursor)
            .map_err(|e| RailError::Io(e.to_string()))?;
        if tail.restarted {
            *self = Held::default();
        }
        // Held, but not verified under its claimed actor: no part of the
        // digest (svrngs-core sync.rs `digest`).
        let mut unverified = 0usize;
        let mut newly_named: Vec<String> = Vec::new();
        for op in &tail.ops {
            let id = derive_id(op, namespace);
            let line = (id.clone(), op.kind.sig.clone());
            let verified = Ed25519Verifier.verify(&op.actor, &id, &op.kind.sig);
            // Only a line its claimed actor signed moves that actor's newest
            // seq (U7; ralph/REVIEW_FINDINGS.md #16): a forged line above a
            // member's newest act left the door signing past it, holes that
            // never fill.
            match self.newest.get_mut(&op.actor) {
                _ if !verified => {}
                Some(n) if n.seq > op.kind.seq => {}
                Some(n) if n.seq == op.kind.seq => n.lines.push(line),
                _ => {
                    self.newest.insert(
                        op.actor.clone(),
                        Newest {
                            seq: op.kind.seq,
                            lines: vec![line],
                        },
                    );
                }
            }
            if verified {
                self.passed.insert((op.actor.clone(), id.clone(), op.kind.sig.clone()));
                self.digest.add(id.clone(), op);
            } else {
                self.forged.insert((id.clone(), op.kind.sig.clone()));
                unverified += 1;
            }
            if let Some(key) = op.kind.act.names().filter(|k| self.named.insert(k.to_string())) {
                newly_named.push(key.to_string());
            }
            if op.kind.act.moves_membership() || self.named.contains(&op.actor) {
                self.membership.push(op.clone());
                self.standing = None;
            }
            if verified {
                self.ids.insert(id);
            }
        }
        if !newly_named.is_empty() {
            // A key a `Remove` names for the first time: its acts already
            // held join the fold of standing, read once from the whole log.
            let (all, _) = oplog.read_all_with_skips().map_err(|e| RailError::Io(e.to_string()))?;
            let before = self.membership.len();
            self.membership.extend(
                all.into_iter()
                    .filter(|op| newly_named.contains(&op.actor) && !op.kind.act.moves_membership()),
            );
            tracing::debug!(
                namespace,
                ?newly_named,
                kept = self.membership.len() - before,
                "ring rail: a key a Remove names — its acts held join the fold of standing"
            );
        }
        tracing::debug!(
            namespace,
            folded = tail.ops.len(),
            unverified,
            restarted = tail.restarted,
            held = self.ids.len(),
            "ring rail: write index caught up with the log"
        );
        self.cursor = tail.cursor;
        Ok(tail.restarted)
    }

    /// [`Self::catch_up`] past a write that already landed. A failure here
    /// must not report the write as failed, so it voids the fold instead: the
    /// next write re-reads the whole log, the cost before this index.
    fn after_write(&mut self, oplog: &Oplog<SignedOp>, namespace: &str) {
        if let Err(e) = self.catch_up(oplog, namespace) {
            tracing::warn!(
                namespace,
                error = %e,
                "ring rail: the write landed, the index could not read it back; the next write \
                 re-reads the whole log"
            );
            *self = Held::default();
        }
    }
}

impl RingJournal {
    /// Open (lazily — nothing is created until the first append) the journal
    /// for `namespace` under `root`.
    #[expect(clippy::disallowed_types, reason = "the journal's single-writer lock")]
    pub fn open(root: &Path, namespace: &str) -> Result<Self, RailError> {
        if !valid_namespace(namespace) {
            return Err(RailError::BadNamespace(namespace.to_string()));
        }
        let dir = ring_dir(root, namespace);
        Ok(Self {
            namespace: namespace.to_string(),
            log: Mutex::new(Writer {
                oplog: Oplog::new(dir.clone()),
                held: Held::default(),
            }),
            dir,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// This namespace's roster file. One spelling of `roster.json`, so the
    /// reader, the writer and the error message that names it cannot drift
    /// onto three files (ARCH §10.6).
    pub fn roster_path(&self) -> PathBuf {
        self.dir.join("roster.json")
    }

    fn log(&self) -> std::sync::MutexGuard<'_, Writer> {
        // A panic in another appender must not take the journal offline. The
        // log is on disk; the one in-memory state, the write index, may have
        // been left half-folded, so it starts again from the file.
        self.log.lock().unwrap_or_else(|e| {
            let mut writer = e.into_inner();
            writer.held = Held::default();
            writer
        })
    }

    /// Every op on disk, plus the lines that could not be read.
    pub fn read(&self) -> Result<(Vec<Op<SignedOp>>, Vec<SkippedLine>), RailError> {
        self.log()
            .oplog
            .read_all_with_skips()
            .map_err(|e| RailError::Io(e.to_string()))
    }

    /// The acts in the order every node applies them, and what could not be
    /// accounted for.
    ///
    /// `verifier` is passed rather than held, so the journal on disk has no
    /// opinion about which scheme judged the signatures on it — the caller
    /// that reads an answer is the one that says what it trusted.
    /// [`Ed25519Verifier`] is the shipped one.
    pub fn admit(&self, roster: &Roster, verifier: &dyn RingVerifier) -> Result<Admission, RailError> {
        let (ops, skipped) = self.read()?;
        Ok(admit(
            &crate::ActSet::of(&ops),
            &skipped,
            roster,
            &self.namespace,
            verifier,
        ))
    }

    /// What this node can honestly claim to need nothing below, per actor —
    /// the ~600-byte payload a peer needs in order to work out what to send
    /// back. See [`digest`] for why the mark is contiguous and why an op that
    /// does not verify under its claimed actor counts for nothing.
    ///
    /// Caught up from the log's tail ([`Held`]); its line is the one the
    /// scale instrument counts as the digest, kept or, when the fold began
    /// again from the top, rebuilt (`crates/svrngs-node/tests/scale.rs`).
    pub fn digest(&self) -> Result<Digest, RailError> {
        let mut guard = self.log();
        let log = &mut *guard;
        let restarted = log.held.catch_up(&log.oplog, &self.namespace)?;
        tracing::debug!(namespace = %self.namespace, restarted, "ring rail: digest caught up with the log");
        Ok(log.held.digest.digest())
    }

    /// The whole register, from the fold [`Self::folded`] keeps (U7).
    pub(crate) fn register(&self, seed: &Roster) -> Result<crate::register::Register, RailError> {
        self.folded(seed).map(|(register, _)| register)
    }

    /// The group's register from `seed` ([`crate::register::register_in`]),
    /// who stands and who was cut, over the acts that move membership as
    /// [`Held`] caught them up, judged by [`Checked`]; and true when it was
    /// answered from the register kept since the last call, false when
    /// folded now.
    pub(crate) fn folded(&self, seed: &Roster) -> Result<(crate::register::Register, bool), RailError> {
        let mut guard = self.log();
        let log = &mut *guard;
        log.held.catch_up(&log.oplog, &self.namespace)?;
        let held = &mut log.held;
        if let Some((_, register)) = held.standing.as_ref().filter(|(from, _)| from == seed) {
            return Ok((register.clone(), true));
        }
        let register = crate::register::register_in(&held.membership, seed, &self.namespace, &Checked(&held.passed))?;
        held.standing = Some((seed.clone(), register.clone()));
        Ok((register, false))
    }

    /// Every op this node holds that `theirs` says the peer is missing,
    /// stopped at `budget_bytes` of serialised ops; author-blind, so a node
    /// republishes what it HOLDS, and a housemate who leaves the ring does
    /// not take their half of the journal with them. Returns `(ops, more)` — see
    /// [`ops_missing_from_within`] for why the truncation is in the return
    /// type and why repeating this terminates.
    ///
    /// This is what every caller that sends ops over the wire uses. The
    /// budget itself is NOT decided here: it is derived from the receiver's
    /// body limit by the crate that owns that limit
    /// (`sovereign_daemon::routes_internal::RING_SYNC_OPS_BUDGET_BYTES`), so
    /// the rail stays a crate a ring app can lift without an HTTP server.
    ///
    /// A peer whose digest is ours is answered from the kept fold, unread,
    /// when [`DigestFold::missing_nothing`] says the slow path would send it
    /// nothing; every other peer is answered from the whole log, every
    /// signature judged by [`Checked`]: the ones passed as their lines entered
    /// are not checked again.
    pub fn ops_missing_from_within(
        &self,
        theirs: &Digest,
        budget_bytes: usize,
    ) -> Result<(Vec<Op<SignedOp>>, bool), RailError> {
        let mut guard = self.log();
        let log = &mut *guard;
        let restarted = log.held.catch_up(&log.oplog, &self.namespace)?;
        tracing::debug!(namespace = %self.namespace, restarted, "ring rail: digest caught up with the log");
        let newest = log.held.newest.iter().map(|(actor, n)| (actor.as_str(), n.seq));
        if log.held.digest.missing_nothing(theirs, newest) {
            tracing::debug!(namespace = %self.namespace, "ring rail: offer answered unread, the peer's digest is ours");
            return Ok((Vec::new(), false));
        }
        let (ops, _) = log
            .oplog
            .read_all_with_skips()
            .map_err(|e| RailError::Io(e.to_string()))?;
        tracing::debug!(namespace = %self.namespace, lines = ops.len(), "ring rail: offer folded from the whole log");
        Ok(ops_missing_from_within(
            &ops,
            &self.namespace,
            &Checked(&log.held.passed),
            theirs,
            budget_bytes,
        ))
    }

    /// Append a batch of peer ops, skipping the ones already held. Returns
    /// how many were new.
    ///
    /// One append for the whole batch, and the held ids from the write index
    /// ([`Held`]) rather than a read of the whole log: a boot republish hands
    /// over the entire journal, and re-reading it per op or per batch would
    /// make catching up grow with a file that only ever grows.
    pub fn ingest_all(&self, ops: &[Op<SignedOp>]) -> Result<usize, RailError> {
        if ops.is_empty() {
            return Ok(0);
        }
        let mut guard = self.log();
        let log = &mut *guard;
        log.held.catch_up(&log.oplog, &self.namespace)?;
        let mut batch = std::collections::BTreeSet::new();
        let mut fresh: Vec<Op<SignedOp>> = Vec::new();
        for op in ops {
            // Also dedupes WITHIN the batch: a peer that sends the same op
            // twice in one body must not get two lines out of it. By the id
            // its content derives, never the line's own (svrngs, U2), held
            // once a line of it verified; else by id and signature, so a
            // forged copy never stands in for the act (#189).
            let line = (derive_id(op, &self.namespace), op.kind.sig.clone());
            if !log.held.ids.contains(&line.0) && !log.held.forged.contains(&line) && batch.insert(line) {
                fresh.push(op.clone());
            }
        }
        if fresh.is_empty() {
            tracing::debug!(
                namespace = %self.namespace,
                offered = ops.len(),
                "ring rail: peer batch held nothing new"
            );
            return Ok(0);
        }
        log.oplog.append_all(&fresh).map_err(|e| RailError::Io(e.to_string()))?;
        log.held.after_write(&log.oplog, &self.namespace);
        tracing::debug!(
            namespace = %self.namespace,
            offered = ops.len(),
            ingested = fresh.len(),
            "ring rail: ingested a peer batch"
        );
        Ok(fresh.len())
    }

    pub fn roster_file(&self) -> Result<Roster, RailError> {
        Ok(self.seed()?.unwrap_or_default())
    }

    /// The roster file as the register reads it, the group's seed: `None`
    /// when there is no file, so the register can say "no seed" rather than
    /// read an empty roster as its answer (ARCH 6). [`Self::roster_file`] is
    /// this with an absent file read as empty.
    pub fn seed(&self) -> Result<Option<Roster>, RailError> {
        let path = self.roster_path();
        match std::fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw)
                .map(Some)
                .map_err(|e| RailError::Io(format!("{}: {e}", path.display()))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(RailError::Io(format!("{}: {e}", path.display()))),
        }
    }

    pub fn set_roster(&self, roster: &Roster) -> Result<(), RailError> {
        std::fs::create_dir_all(&self.dir).map_err(|e| RailError::Io(e.to_string()))?;
        let raw = serde_json::to_string_pretty(roster).map_err(|e| RailError::Io(e.to_string()))?;
        std::fs::write(self.roster_path(), raw).map_err(|e| RailError::Io(e.to_string()))
    }
}

/// The wall clock in Unix seconds: the one place the write side reads it.
/// Unreadable is named, never defaulted (ROOT_CAUSE_FIXES C4).
#[expect(clippy::disallowed_methods, reason = "the write side's clock adapter")]
pub(crate) fn now_unix(namespace: &str) -> Result<i64, RailError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|why| {
            tracing::warn!(target: "rail:door", ns = %namespace, %why,
                "append refused — the wall clock is unreadable");
            RailError::Clock
        })
}

impl svrngs_core::Store for RingJournal {
    type Error = RailError;

    fn append_with<R>(
        &self,
        actor: &str,
        f: impl FnOnce(&ActSet, Option<svrngs_core::Tip>) -> (Option<svrngs_core::Authored>, R),
    ) -> Result<R, RailError> {
        let mut guard = self.log();
        let log = &mut *guard;
        log.held.catch_up(&log.oplog, &self.namespace)?;
        let (held_ops, _) = log
            .oplog
            .read_all_with_skips()
            .map_err(|e| RailError::Io(e.to_string()))?;
        let tip = log.held.newest.get(actor).map(|n| svrngs_core::Tip {
            seq: n.seq,
            lines: n.lines.clone(),
        });
        let (authored, answer) = f(&ActSet::of(&held_ops), tip);
        let Some(authored) = authored.filter(|a| !a.lines().is_empty()) else {
            return Ok(answer);
        };
        log.oplog
            .append_all(authored.lines())
            .map_err(|e| RailError::Io(e.to_string()))?;
        log.held.after_write(&log.oplog, &self.namespace);
        for op in authored.lines() {
            tracing::debug!(namespace = %self.namespace, id = %op.id, actor = %op.actor,
                seq = op.kind.seq, "ring rail: appended by commit");
        }
        Ok(answer)
    }
}

#[cfg(test)]
#[path = "journal_tests.rs"]
mod tests;
