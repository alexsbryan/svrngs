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
/// id, so the act never landed (DESIGN §6). The lines at an actor's newest
/// seq are kept, id and signature, because the door's next act names one of
/// them as its previous act, and only one that verifies is the signer's own.
///
/// Folded from the file and caught up from its tail before each write
/// ([`Oplog::read_since`]), so a write costs the lines appended since the last
/// one rather than the whole log: re-reading it per write made the kv drain
/// cost rows x log lines a tick, and a door write grow with the journal
/// (pc-rails-journal-linear). The file stays the source of truth: lines
/// another process appended are in the tail, and a file renamed over the path
/// (a compaction, ours or theirs) restarts the fold from the top.
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
/// acts that move membership, a handful among thousands, kept in the log's
/// order as the tail brings them in, and the standing last folded from them,
/// with the seed it was folded from. Read whole on every connection, sync
/// request and round, it was what an idle round still paid for (26 reads
/// of 10,000 acts, 5.5 s; scale.rs, ceiling-3's runs; THE_CEILING.md fix 3).
/// An act that moves membership voids the standing; a seed unlike the one it
/// was folded from folds it again.
#[derive(Default)]
struct Held {
    cursor: crate::oplog::Cursor,
    newest: std::collections::HashMap<String, Newest>,
    ids: std::collections::HashSet<OpId>,
    digest: DigestFold,
    passed: Passed,
    membership: Vec<Op<SignedOp>>,
    standing: Option<(Roster, BTreeMap<String, crate::register::Member>)>,
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

/// One actor's newest seq on the log, and each line held at it: the id its
/// content derives and the signature it carries.
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
        for op in &tail.ops {
            let id = derive_id(op, namespace);
            let line = (id.clone(), op.kind.sig.clone());
            match self.newest.get_mut(&op.actor) {
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
            if Ed25519Verifier.verify(&op.actor, &id, &op.kind.sig) {
                self.passed.insert((op.actor.clone(), id.clone(), op.kind.sig.clone()));
                self.digest.add(id.clone(), op);
            } else {
                unverified += 1;
            }
            if op.kind.act.moves_membership() {
                self.membership.push(op.clone());
                self.standing = None;
            }
            self.ids.insert(id);
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

    /// Who stands in this group from `seed` ([`crate::register::standing_in`]),
    /// over the acts that move membership as [`Held`] caught them up, judged
    /// by [`Checked`]; and true when it was answered from the standing kept
    /// since the last call, false when folded now.
    pub(crate) fn standing(
        &self,
        seed: &Roster,
    ) -> Result<(BTreeMap<String, crate::register::Member>, bool), RailError> {
        let mut guard = self.log();
        let log = &mut *guard;
        log.held.catch_up(&log.oplog, &self.namespace)?;
        let held = &mut log.held;
        if let Some((folded_from, members)) = &held.standing {
            if folded_from == seed {
                return Ok((members.clone(), true));
            }
        }
        let members = crate::register::standing_in(&held.membership, seed, &self.namespace, &Checked(&held.passed))?;
        held.standing = Some((seed.clone(), members.clone()));
        Ok((members, false))
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
        // A local-only journal answers no peer ([`is_local_only`]).
        if is_local_only(&self.namespace) {
            tracing::debug!(
                namespace = %self.namespace,
                "ring rail: local-only journal, nothing offered to the peer"
            );
            return Ok((Vec::new(), false));
        }
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
        let mut batch: std::collections::BTreeSet<OpId> = std::collections::BTreeSet::new();
        let mut fresh: Vec<Op<SignedOp>> = Vec::new();
        for op in ops {
            // Also dedupes WITHIN the batch: a peer that sends the same op
            // twice in one body must not get two lines out of it. By the id
            // its content derives, never the line's own (svrngs, U2).
            let id = derive_id(op, &self.namespace);
            if !log.held.ids.contains(&id) && batch.insert(id) {
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

    /// Sign a [`Seal`](RailAct::Seal) and delete what it retires — the pair.
    ///
    /// **THE one seal-and-prune (ARCH §10.6).** Rung S taught the read path to
    /// skip a retired prefix; nothing removed one from disk, so a seal on its
    /// own changed no bytes anywhere, and a prune with no seal behind it
    /// reports the deleted range as missing, forever, on every node. Neither
    /// half is worth anything alone. The two callers that seal — the rail
    /// route an app reaches (`sovereign_daemon::routes_rail::append`) and
    /// cw-rails' plane seal (`commonwealth_rails::plane_seal`) — reach this
    /// rather than each writing the sequence themselves, because a second
    /// spelling of "seal, then compact, and a refused compaction is not a
    /// failed seal" is a decider with two answers.
    ///
    /// The seal's own failure IS an `Err`: nothing was written, so there is no
    /// half-done state to describe.
    pub fn seal(
        &self,
        signer: &dyn RingSigner,
        roster: &Roster,
        verifier: &dyn RingVerifier,
    ) -> Result<Sealed, RailError> {
        let author = svrngs_core::Author {
            signer,
            roster,
            namespace: &self.namespace,
            verifier,
            now: now_unix(&self.namespace)?,
        };
        let op = match svrngs_core::commit(self, &svrngs_core::Seal, &author)? {
            Ok(recorded) => recorded.lines()[0].clone(),
            Err(_) => {
                return Err(RailError::NotInRoster {
                    actor: signer.actor(),
                    namespace: self.namespace.clone(),
                });
            }
        };
        let retired = self.compact(roster, verifier);
        if let Err(e) = &retired {
            tracing::warn!(
                namespace = %self.namespace,
                error = %e,
                "ring rail: the seal is written, the prune it authorises was refused"
            );
        }
        Ok(Sealed { op, retired })
    }

    /// Delete every line a seal has retired, and report what went.
    ///
    /// **This is the half that makes the rail's storage bounded.** A journal
    /// that can only be appended to grows for as long as the ring is used, and
    /// every peer holds a full copy of everyone's history forever; the seal
    /// (rung S) taught the READ path to stop asking for a retired prefix, but
    /// nothing until now removed one from disk. Both halves are needed and
    /// neither is provable alone: a seal nobody prunes on saves no bytes, and a
    /// prune with no seal behind it produced ten thousand
    /// [`SequenceHole`](RailGap::SequenceHole)s in the ceiling measurement,
    /// because every node then reported the retired range as missing forever.
    ///
    /// # The floor is admission's, never re-derived here
    ///
    /// An op is dropped when its `seq` is strictly below its own actor's floor
    /// on [`Admission::floors`] — the map [`admit`] built from seals that
    /// passed the signature and roster checks. A prune that read the seals
    /// itself would be a second answer to "what is retired" (ARCH §10.6), and
    /// the two would part company at the worst possible place: a forged seal is
    /// refused by admission and would be believed by a naive re-read, so one
    /// pushed line would erase a member's history from disk on every node that
    /// received it. Deleting by the ADMITTED floor makes that unwritable rather
    /// than checked.
    ///
    /// The seal itself is kept — the floor IS its `seq`, and the comparison is
    /// strict — which is what keeps [`digest`]'s contiguous run non-empty for a
    /// compacted actor. Dropping it would make this node claim it holds nothing
    /// of that actor, and every peer would re-send the whole history back.
    ///
    /// It prunes **every** actor's retired prefix, not just this node's. A seal
    /// is a statement by its author that binds whoever admits it, so a peer
    /// stops storing what the author retired as soon as the seal reaches it.
    /// Author-only pruning would bound the writer's disk and nobody else's.
    ///
    /// # Two things it will not delete
    ///
    /// **A journal with unreadable lines is not rewritten at all.**
    /// [`SkippedLine`] carries a line number and a parse error, never the
    /// bytes, so a line this build cannot parse cannot survive a rewrite — and
    /// that includes a line from a NEWER format version, which is exactly the
    /// content an old build must not be the one to destroy. Refusing is the
    /// only honest option; silently dropping them would be a compaction that
    /// deletes the future (ARCH §18.3).
    ///
    /// **An op a surviving correction names is kept regardless of the floor.**
    /// Retired means "nobody will ask for this again", and a
    /// [`Correct`](RailAct::Correct) still pointing at it says otherwise.
    /// Without this clause, sealing over a corrected op turns its correction
    /// into a permanent [`DanglingCorrection`](RailGap::DanglingCorrection) —
    /// and the guard below would then refuse every future compaction of that
    /// ring, so the growth this method exists to stop would quietly come back.
    ///
    /// # It verifies its own result before committing it (ARCH §7)
    ///
    /// The kept set is re-admitted and compared against the admission we
    /// started from; if compaction would raise a gap the journal did not
    /// already have, nothing is written and the refusal names the gap. This is
    /// the invariant "a node may delete only what a floor covers" encoded so it
    /// cannot be forgotten, rather than left as a rule a future caller has to
    /// remember — and it costs one fold over an in-memory `Vec` that is by
    /// construction shorter than the one already read.
    ///
    /// Gaps may DISAPPEAR, and that is not refused: a line that failed the
    /// signature check below a floor its claimed author authenticated is one
    /// nobody will ever want, and keeping it would let anyone grow a peer's
    /// journal without bound by pushing junk under an old seal. The count is
    /// reported on [`Compaction::gaps_cleared`] rather than left silent.
    ///
    /// The writer lock is held across read, decide and replace, because
    /// `O_APPEND` orders appends against each other and does nothing for an
    /// append racing the rename underneath it.
    pub fn compact(&self, roster: &Roster, verifier: &dyn RingVerifier) -> Result<Compaction, RailError> {
        let mut log = self.log();
        let (ops, skipped) = log
            .oplog
            .read_all_with_skips()
            .map_err(|e| RailError::Io(e.to_string()))?;

        if !skipped.is_empty() {
            return Err(RailError::Rejected(format!(
                "`{}` holds {} line(s) this build cannot read, and a rewrite would \
                 destroy them — their bytes are not recoverable from the skip \
                 report. Nothing was deleted. Run `svrn ring log {}` to see what \
                 they are; a line from a newer format version means this node is \
                 the old one and must not be the one to compact.",
                self.namespace,
                skipped.len(),
                self.namespace,
            )));
        }

        let before = admit(&crate::ActSet::of(&ops), &skipped, roster, &self.namespace, verifier);
        let unchanged = |floors: Floors, kept: usize| Compaction {
            removed: 0,
            kept,
            gaps_cleared: 0,
            floors,
        };
        // The decision is the core's: `Compact` reads the seals in the record
        // and only `react` hands back the `Due` this deletion runs from.
        let due = match before.react(&svrngs_core::Compact) {
            Ok(Some(due)) => due,
            Ok(None) => {
                tracing::debug!(
                    namespace = %self.namespace,
                    held = ops.len(),
                    "ring rail: nothing sealed, nothing to compact"
                );
                return Ok(unchanged(Floors::new(), ops.len()));
            }
            Err(uncaused) => {
                return Err(RailError::Rejected(format!(
                    "compacting `{}` was decided from {} act(s) that do not stand in its record, \
                     so nothing was deleted",
                    self.namespace,
                    uncaused.0.len(),
                )));
            }
        };
        let retire = due.effect();
        let kept: Vec<Op<SignedOp>> = ops
            .iter()
            .filter(|op| retire.keeps(op, &self.namespace))
            .cloned()
            .collect();
        let removed = ops.len() - kept.len();
        if removed == 0 {
            tracing::debug!(
                namespace = %self.namespace,
                held = ops.len(),
                sealed = ?retire.floors(),
                "ring rail: every line held is at or above its floor"
            );
            return Ok(unchanged(retire.floors().clone(), kept.len()));
        }

        let after = admit(&crate::ActSet::of(&kept), &[], roster, &self.namespace, verifier);
        let raised = retire.raises(&after);
        if !raised.is_empty() {
            return Err(RailError::Rejected(format!(
                "compacting `{}` would raise {} gap(s) the journal does not have, \
                 so nothing was deleted. First: {}",
                self.namespace,
                raised.len(),
                raised[0],
            )));
        }
        let gaps_cleared = retire.clears(&after);

        log.oplog.replace_all(&kept).map_err(|e| RailError::Io(e.to_string()))?;
        // The rename already voids the cursor; this says so without relying
        // on the platform having inodes.
        log.held = Held::default();
        // INFO, not debug: this is the one call in the rail that destroys
        // something, and an operator reading logs after a shrinking journal
        // needs the floors it happened under without turning debug on.
        tracing::info!(
            namespace = %self.namespace,
            held = ops.len(),
            removed,
            kept = kept.len(),
            gaps_cleared,
            sealed = ?retire.floors(),
            causes = ?due.causes(),
            "ring rail: compacted"
        );
        Ok(Compaction {
            removed,
            kept: kept.len(),
            gaps_cleared,
            floors: retire.floors().clone(),
        })
    }

    /// The roster FILE. A missing file is an empty ring, not an error — and
    /// an empty ring admits nothing but gaps, which is the honest answer
    /// before anyone has been added.
    ///
    /// This is the storage half, and its name says so. (svrngs: the file is
    /// every group's seed, and who stands in the group is the register's,
    /// [`RingRail::standing`]; upstream's `RingRail::roster`, which chose
    /// between this file and a derived roster, went with the derived ones.)
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
