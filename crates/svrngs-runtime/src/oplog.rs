// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai oplog/src/lib.rs@e1a43bcd2
//! [`Op`] and [`Oplog`] — one append-only journal, three tenants.
//!
//! # Why this module exists
//!
//! Measured 2026-08-20: corpus-engine held **three** append-only-JSONL logs
//! with the same envelope, the same file IO and the same reader, written three
//! times in three sibling directories. The code said so itself —
//! `enrichment/governance.rs` carried the section header
//! `// ── Persistence (mirrors reconciliation/oplog.rs conventions) ─`, and
//! `meta_atlas/bridge/edges.rs` opened with "The oplog mirrors the *discipline*
//! of `crate::enrichment::reconciliation::oplog`". Mirroring a discipline by
//! retyping it is how a discipline drifts.
//!
//! | was | entry | store | had id? | had actor? | had version gate? |
//! |---|---|---|---|---|---|
//! | governance | `GovernanceOp` | `GovernanceOplog` | yes | yes | yes |
//! | reconciliation | `OplogEntry` | `OplogWriter` + `OplogReader` | **no** | **no** | **no** |
//! | meta-atlas bridge | `BridgeOp` | `BridgeOplog` | **no** | **no** | **no** |
//!
//! The asymmetry was maturity, not concept: governance is the complete
//! version and the two younger logs are the same envelope with the identity
//! and attribution half never built. That absence was not cosmetic —
//! `reconciliation::OpKind::Split` documented itself as reversible by "walking
//! backwards finding the matching `Merge`", which is unimplementable without
//! an id to match on. Governance already had the answer
//! (`GovernanceOpKind::Revert { targets: Vec<OpId> }`).
//!
//! Seven types collapse to two here, and the two younger logs inherit the id,
//! the attribution and the version gate they were each missing.
//!
//! # What an `Op` is, and what it deliberately is not
//!
//! The envelope carries **provenance only**: what this act is identified as,
//! when it happened, who did it, and which line format it was written in. The
//! act itself — and its rationale, its evidence, its subjects — belongs to the
//! tenant's `K`. That cut is why governance can keep `rationale` required on
//! `AcceptTension` and optional elsewhere while the other two tenants carry
//! one envelope-level rationale: the envelope does not have an opinion.
//!
//! `K` is `#[serde(flatten)]`ed, so a line reads as one flat object and the
//! two tenants whose lines already looked like that keep their wire form:
//!
//! ```json
//! {"id":"gov-2f1c…","v":1,"ts_unix":1787,"actor":"human:alex","op":"supersede","new_rule":"…"}
//! ```
//!
//! # Identity is a content hash, and there is one derivation
//!
//! [`OpId`] is `<prefix>-<16 hex>` where the hex is the truncated BLAKE3 of
//! `<prefix>|<ts>|<actor>|<body-json>` — so the id a `Revert` targets is
//! reproducible from the log bytes alone, with no positional or external
//! counter to drift (ARCH §7.5: identity from essence, never a counter or an
//! address). Before this module that derivation existed once, privately, in
//! `governance.rs`; the other two logs had no ids at all. It is now the one
//! decider (ARCH §10.6) and every tenant reaches it.
//!
//! # Why this is a crate and not a module
//!
//! It was `corpus-engine/src/oplog.rs` until 2026-09-04. `commonwealth-knowledge`'s
//! ring rail names five of these types and, to reach a 547-line journal, took a
//! dependency on `corpus-engine` and inherited its **578-crate closure**. The
//! rail's entire coupling to the knowledge layer was seven references to
//! [`Op`], [`OpId`] and [`SkippedLine`] — a journal is not knowledge-layer
//! machinery, and the closure was the proof.
//!
//! So this is a Tier-0 leaf whose only in-repo dependency is `kernel-types`.
//! That budget is the contract, not an accident: [`OpId`] is content-addressed
//! **and gossiped**, so a second derivation of it is a wire break with nothing
//! red anywhere (ARCH §10.6). The hash comes from the crate that owns exactly
//! one implementation of it, and [`the pinned wire test`](tests) fails if the
//! derivation ever moves.
//!
//! `corpus-engine` re-exports this crate as `corpus_engine::oplog`, so its
//! twenty-odd consumers did not change when it moved.
//!
//! # Naming — why `Op` and not `Record`
//!
//! Work order `nc-5-corpus` proposed minting `Record` ("an immutable fact with
//! provenance, id = ContentHash") as the register noun for this shape. The
//! order's own not-worth-continuing-if clause fired against it: a census of the
//! 48 first-party `*Record` definitions splits 21 append-only facts against 21
//! store/wire row DTOs, and `id = ContentHash` is true of about three of the
//! forty-eight. Naming this `Record` would have annexed forty-five types that
//! do not share the property. `Op`/`Oplog` is the word the three call sites
//! already used, so this converges the tree's own vocabulary rather than
//! introducing a fourth (ARCH §19).

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

pub use svrngs_core::{Journaled, Op, SkippedLine};

/// What can go wrong writing or reading a journal.
///
/// Minted with the crate. It carries exactly the two failures the journal
/// actually produces, and `corpus-engine`'s `From` bridge maps them back onto
/// the `Error::Io` / `Error::Extraction` variants — with the same `Display`
/// text — that these call sites raised before the move, so no consumer can
/// observe the extraction (ARCH §18.3: the substitution would have to be
/// named, so there isn't one).
#[derive(Debug, thiserror::Error)]
pub enum OplogError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Serialising an op to its JSONL line failed. `log` is [`Journaled::LABEL`].
    #[error("{log}: serialise: {source}")]
    Serialise {
        log: &'static str,
        source: serde_json::Error,
    },
}

pub type Result<T> = std::result::Result<T, OplogError>;

// ── The journal ──────────────────────────────────────────────

/// Append-only JSONL journal at `<dir>/<K::FILE>`. One [`Op`] per line; the
/// file is the bytes-level record of every decision, and re-reading it is how
/// current state is derived rather than stored.
///
/// Reader and writer are one type. They were split for the reconciliation log
/// and joined for the other two, which bought nothing but a second name — an
/// `Oplog` holds a path and no handle, so it is as cheap to build as either
/// half was.
pub struct Oplog<K> {
    path: PathBuf,
    _kind: PhantomData<fn() -> K>,
}

impl<K: Journaled> Oplog<K> {
    /// Point at the log inside `dir`. Does not touch the filesystem — the
    /// directory is created lazily on first append, so a caller may build one
    /// for a corpus whose atlas dir does not exist yet.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            path: dir.into().join(K::FILE),
            _kind: PhantomData,
        }
    }

    /// Append several ops in one open. Creates the parent directory on
    /// first write.
    pub fn append_all(&self, ops: &[Op<K>]) -> Result<()> {
        if ops.is_empty() {
            return Ok(());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(OplogError::Io)?;
        }
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(OplogError::Io)?;
        let mut buf = String::new();
        for op in ops {
            let line = serde_json::to_string(op).map_err(|e| OplogError::Serialise {
                log: K::LABEL,
                source: e,
            })?;
            buf.push_str(&line);
            buf.push('\n');
        }
        // ONE write for the whole batch, so `O_APPEND`'s atomicity covers it.
        // Two calls (the body, then the newline) left a window in which a
        // second writer could interleave and split a line in half.
        f.write_all(buf.as_bytes()).map_err(OplogError::Io)?;
        // Durability is the point of a journal. Without this a lid-slam or a
        // power cut loses the last ops that the caller was already told
        // succeeded — silently, which is the §18.3 shape.
        f.sync_data().map_err(OplogError::Io)?;
        tracing::debug!(
            log = K::LABEL,
            ops = ops.len(),
            path = %self.path.display(),
            "oplog: append_all"
        );
        Ok(())
    }

    /// Replace the whole log with exactly `ops`, atomically.
    ///
    /// **The only shrinking operation this journal has**, and the reason it
    /// exists is retention: an append-only file that can never lose a line
    /// grows without bound, and "delete the old prefix" is not expressible as
    /// an append. Every other method here adds; this one is how a tenant that
    /// has *earned* the right to forget actually forgets. What earns it is the
    /// tenant's business — the ring rail requires a signed seal
    /// (`commonwealth_rail::RingJournal::compact`) — and this method takes no
    /// opinion beyond writing the lines it was handed.
    ///
    /// # Atomic, because a half-written journal is worse than a long one
    ///
    /// The lines go to a sibling temp file, are fsynced, and are then renamed
    /// over the log. `rename(2)` within a directory is atomic, so a reader
    /// concurrent with this sees either the whole old log or the whole new
    /// one, and a crash at any point leaves one of the two on disk rather than
    /// a truncated file. The directory is fsynced afterwards so the rename
    /// itself survives a power cut — without that the data is durable and the
    /// name pointing at it is not, which on some filesystems leaves the log
    /// missing entirely.
    ///
    /// The temp file is `<FILE>.compacting` in the same directory rather than
    /// in a temp dir: `rename(2)` cannot cross a filesystem, and a data
    /// directory on its own volume is the normal deployment here.
    ///
    /// # This does NOT serialise against [`Self::append`]
    ///
    /// `O_APPEND` makes concurrent appends safe against each other; it does
    /// nothing for an append racing a rename, which would land the appended
    /// line in the file that is about to be replaced and lose it. The caller
    /// holds the writer lock for the read-decide-replace sequence — see
    /// `RingJournal::compact`, which does exactly that.
    pub fn replace_all(&self, ops: &[Op<K>]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(OplogError::Io)?;
        }
        let mut buf = String::new();
        for op in ops {
            let line = serde_json::to_string(op).map_err(|e| OplogError::Serialise {
                log: K::LABEL,
                source: e,
            })?;
            buf.push_str(&line);
            buf.push('\n');
        }
        let tmp = self.path.with_extension("compacting");
        {
            let mut f = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp)
                .map_err(OplogError::Io)?;
            f.write_all(buf.as_bytes()).map_err(OplogError::Io)?;
            f.sync_data().map_err(OplogError::Io)?;
        }
        fs::rename(&tmp, &self.path).map_err(OplogError::Io)?;
        // The rename is a directory change, and a directory change is not
        // covered by the file's own fsync above.
        if let Some(parent) = self.path.parent() {
            // Not worth FAILING a compaction over: the replacement lines are
            // already fsynced, so a lost rename leaves the pre-compaction log
            // in place — long, and correct. But not worth swallowing either
            // (ARCH §18.3): a host where this always fails is one where every
            // compaction is provisional, and the only way anyone learns that
            // is if it says so.
            let flushed = fs::File::open(parent).and_then(|dir| dir.sync_all());
            if let Err(e) = flushed {
                tracing::warn!(
                    log = K::LABEL,
                    dir = %parent.display(),
                    error = %e,
                    "oplog: replaced, but the directory was not flushed — the \
                     rename may not survive a power cut, in which case the \
                     pre-compaction log comes back"
                );
            }
        }
        tracing::info!(
            log = K::LABEL,
            ops = ops.len(),
            path = %self.path.display(),
            "oplog: replaced"
        );
        Ok(())
    }

    /// Every op in append order, **plus the lines this build could not use**.
    ///
    /// Two classes of line are skipped rather than defaulted (ARCH §18.3 —
    /// absence is reported, never defaulted): a line this build cannot parse,
    /// and a line declaring a `v` newer than `K::VERSION`. The second is the
    /// forward-compat gate: an older reader must not crash on a newer log, and
    /// must not silently misread it either.
    ///
    /// Both still warn. The warning was all a caller ever got, which means a
    /// caller could not *tell* — a log with a corrupt line and a log with none
    /// returned the identical value. Returning the skips is the difference
    /// between "here is the total" and "here is the total, and I could not
    /// read two lines."
    pub fn read_all_with_skips(&self) -> Result<(Vec<Op<K>>, Vec<SkippedLine>)> {
        if !self.path.exists() {
            return Ok((Vec::new(), Vec::new()));
        }
        let file = fs::File::open(&self.path).map_err(OplogError::Io)?;
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        let mut skipped = Vec::new();
        for (lineno, line) in reader.lines().enumerate() {
            let line = line.map_err(OplogError::Io)?;
            if line.trim().is_empty() {
                continue;
            }
            match self.parse_line(&line, lineno as u64 + 1) {
                Ok(op) => out.push(op),
                Err(skip) => skipped.push(skip),
            }
        }
        Ok((out, skipped))
    }

    /// The ops appended since `cursor`, and the cursor after them.
    ///
    /// For a tenant that keeps a fold of the log in memory and must not pay a
    /// whole read per write (the ring journal's next-seq and held-id index).
    /// Reads from the cursor's byte offset to the end of the last whole line,
    /// so a line another process is still writing is left for the next call.
    /// When the file is not the one the cursor was taken on (a
    /// [`Self::replace_all`] renamed a new one over it) or is shorter than the
    /// offset, the read starts again from the top and the returned
    /// [`Tail::restarted`] says the caller's fold is void. Lines are judged by
    /// the same rules as [`Self::read_all_with_skips`]; skipped ones are warned
    /// and counted in its trace, with line numbers counted from where this read began.
    pub fn read_since(&self, cursor: Cursor) -> Result<Tail<K>> {
        use std::io::{Read, Seek, SeekFrom};
        let mut file = match fs::File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Tail {
                    ops: Vec::new(),
                    cursor: Cursor::default(),
                    restarted: cursor != Cursor::default(),
                });
            }
            Err(e) => return Err(OplogError::Io(e)),
        };
        let meta = file.metadata().map_err(OplogError::Io)?;
        let file_id = file_identity(&meta);
        let restarted = file_id.is_none() || file_id != cursor.file || meta.len() < cursor.end;
        let start = if restarted { 0 } else { cursor.end };
        file.seek(SeekFrom::Start(start)).map_err(OplogError::Io)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(OplogError::Io)?;
        let whole = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
        let text = std::str::from_utf8(&bytes[..whole])
            .map_err(|e| OplogError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        let mut ops = Vec::new();
        let mut skipped = 0usize;
        for (lineno, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            match self.parse_line(line, lineno as u64 + 1) {
                Ok(op) => ops.push(op),
                Err(_) => skipped += 1,
            }
        }
        tracing::debug!(
            log = K::LABEL,
            path = %self.path.display(),
            from = start,
            bytes = whole,
            ops = ops.len(),
            skipped,
            restarted,
            "oplog: read since cursor"
        );
        Ok(Tail {
            ops,
            cursor: Cursor {
                file: file_id,
                end: start + whole as u64,
            },
            restarted,
        })
    }

    /// One line, judged: an op, or the reason it is not one. The one place
    /// both readers decide which lines count.
    fn parse_line(&self, line: &str, line_no: u64) -> std::result::Result<Op<K>, SkippedLine> {
        // The version gate runs BEFORE the body parse (leg 5 of
        // `ra-membership-is-order-free`): an act whose KIND this build
        // has never seen fails serde, and a parse-first gate would name
        // that Malformed exactly when NewerVersion is the honest answer.
        #[derive(serde::Deserialize)]
        struct LinePeek {
            v: u32,
        }
        if let Ok(peek) = serde_json::from_str::<LinePeek>(line) {
            if peek.v > K::UNDERSTANDS {
                tracing::warn!(
                    log = K::LABEL,
                    path = %self.path.display(),
                    line = line_no,
                    v = peek.v,
                    "oplog: skipping op from a newer format version"
                );
                return Err(SkippedLine::NewerVersion {
                    line: line_no,
                    v: peek.v,
                });
            }
        }
        serde_json::from_str::<Op<K>>(line).map_err(|err| {
            tracing::warn!(
                log = K::LABEL,
                path = %self.path.display(),
                line = line_no,
                "oplog: malformed line skipped ({err})"
            );
            SkippedLine::Malformed {
                line: line_no,
                error: err.to_string(),
            }
        })
    }
}

/// Where a [`Oplog::read_since`] stopped: which file it read, and the byte
/// after the last whole line. The default is "nothing read yet".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cursor {
    file: Option<(u64, u64)>,
    end: u64,
}

/// What [`Oplog::read_since`] found.
#[derive(Debug)]
pub struct Tail<K> {
    /// The ops past the cursor, in append order.
    pub ops: Vec<Op<K>>,
    /// Where the next read starts.
    pub cursor: Cursor,
    /// The read began at the top of the file, not at the cursor: whatever the
    /// caller folded from earlier reads no longer describes this file.
    pub restarted: bool,
}

/// The file's (device, inode), so a rename over the path is seen. Where the
/// platform has no such pair every read restarts, which is a whole read: the
/// cost this cursor exists to avoid, never a wrong answer.
#[cfg(unix)]
fn file_identity(meta: &fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((meta.dev(), meta.ino()))
}

#[cfg(not(unix))]
fn file_identity(_meta: &fs::Metadata) -> Option<(u64, u64)> {
    None
}

#[cfg(test)]
#[path = "oplog_tests.rs"]
mod tests;
