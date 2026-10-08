// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail/src/tests_write_index.rs@e1a43bcd2
//! The write index (pc-rails-journal-linear): a batch continues the actor's
//! counter, and a write reads only what the log gained since the last one.
//! A sibling of `tests.rs` so that file stays under the size ceiling.

use crate::*;

use crate::tests::Doors;
use svrngs_core::tests_support::*;

fn open(dir: &std::path::Path) -> RingJournal {
    RingJournal::open(dir, NS).unwrap()
}

/// A batch continues the actor's counter, contiguously, and admits like the
/// same acts appended one at a time.
#[test]
fn a_batch_append_continues_the_counter_contiguously() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    journal
        .append(record("x"), &key(1), &r, None, &Ed25519Verifier)
        .unwrap();
    let batch = vec![record("a"), record("b"), record("c")];
    let author = svrngs_core::Author {
        signer: &key(1),
        roster: &r,
        namespace: NS,
        verifier: &Ed25519Verifier,
        now: 1,
    };
    let recorded = svrngs_core::commit(&journal, &Batch(batch), &author).unwrap().unwrap();
    let ops = recorded.lines();
    assert_eq!(ops.iter().map(|o| o.kind.seq).collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(
        journal
            .append(record("y"), &key(1), &r, None, &Ed25519Verifier)
            .unwrap()
            .kind
            .seq,
        4
    );

    let f = journal.admit(&r, &Ed25519Verifier).unwrap();
    assert!(f.is_complete(), "{:?}", f.gaps());
    assert_eq!(wire_ops(&f).len(), 5);
}

/// **A write reads what the log gained since the last one, not the whole log**
/// (pc-rails-journal-linear): re-reading it per write made the kv drain cost
/// rows x log lines a tick. Shown by overwriting a folded line in place —
/// same length, same file — which a whole re-read would see as malformed and
/// hand its seq out again. Lines another writer appended, and a compaction
/// renamed over the log by another handle, are still seen.
#[test]
fn a_write_reads_only_what_the_log_gained_since_the_last_one() {
    use std::io::{Seek, SeekFrom, Write};
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    for _ in 0..3 {
        journal
            .append(record("x"), &key(1), &r, None, &Ed25519Verifier)
            .unwrap();
    }
    let path = journal.dir().join("ring_oplog.jsonl");
    let raw = std::fs::read_to_string(&path).unwrap();
    let last_start = raw.trim_end().rfind('\n').unwrap() + 1;
    let last_len = raw.len() - last_start - 1;
    let mut f = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
    f.seek(SeekFrom::Start(last_start as u64)).unwrap();
    f.write_all("#".repeat(last_len).as_bytes()).unwrap();
    drop(f);
    assert_eq!(
        journal
            .append(record("x"), &key(1), &r, None, &Ed25519Verifier)
            .unwrap()
            .kind
            .seq,
        3,
        "seq 2 was handed out again: the write re-read lines it had already folded"
    );

    // Another handle on the same file appends: this one's next write sees it.
    let other = open(dir.path());
    let theirs = other.append(record("y"), &key(2), &r, None, &Ed25519Verifier).unwrap();
    assert!(!journal.ingest(&theirs).unwrap(), "held from the tail");
    assert_eq!(
        journal
            .append(record("x"), &key(1), &r, None, &Ed25519Verifier)
            .unwrap()
            .kind
            .seq,
        4
    );

    // A compaction by the other handle renames a new file over the log; this
    // handle starts again from the top. On a fresh log: compaction refuses
    // one holding the line overwritten above.
    let fresh = tempfile::tempdir().unwrap();
    let (a, b) = (open(fresh.path()), open(fresh.path()));
    for _ in 0..3 {
        a.append(record("x"), &key(1), &r, None, &Ed25519Verifier).unwrap();
    }
    a.append(RailAct::Seal, &key(1), &r, None, &Ed25519Verifier).unwrap();
    let dropped = a.read().unwrap().0.remove(0);
    assert_eq!(b.compact(&r, &Ed25519Verifier).unwrap().removed, 3);
    assert!(
        a.ingest(&dropped).unwrap(),
        "a retired op is not held after the compaction, as before the index"
    );
    assert_eq!(
        a.append(record("x"), &key(1), &r, None, &Ed25519Verifier)
            .unwrap()
            .kind
            .seq,
        4
    );
}

/// **The digest is caught up from the tail, as the write index is**
/// (ceiling-2): after each change below, one handle's digest and its offer
/// to a peer in step are what the file read whole gives (svrngs-core
/// `digest`, `ops_missing_from_within`) — through its own appends, lines
/// another handle appended, a compaction another handle renamed over the
/// log, a shorter log renamed over that, and an act past a hole, its actor absent from the digest, which
/// an equal digest does not cover, so the offer reads the log and sends it.
#[test]
fn the_digest_is_caught_up_from_the_tail_and_restarts_on_a_replaced_log() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (open(dir.path()), open(dir.path()));
    let r = ring();
    let whole = |j: &RingJournal| {
        let ops = j.read().unwrap().0;
        let ours = svrngs_core::digest(&ops, NS, &Ed25519Verifier);
        let offer = svrngs_core::ops_missing_from_within(&ops, NS, &Ed25519Verifier, &ours, NO_BUDGET);
        (ours, offer)
    };
    let kept = |j: &RingJournal| {
        let ours = j.digest().unwrap();
        let offer = j.ops_missing_from_within(&ours, NO_BUDGET).unwrap();
        (ours, offer)
    };
    for _ in 0..3 {
        a.append(record("x"), &key(1), &r, None, &Ed25519Verifier).unwrap();
    }
    assert_eq!(kept(&a), whole(&a), "its own appends");
    b.append(record("y"), &key(2), &r, None, &Ed25519Verifier).unwrap();
    assert_eq!(kept(&a), whole(&a), "another handle's append, from the tail");
    a.append(RailAct::Seal, &key(1), &r, None, &Ed25519Verifier).unwrap();
    assert_eq!(b.compact(&r, &Ed25519Verifier).unwrap().removed, 3);
    assert_eq!(kept(&a), whole(&a), "a compaction renamed over the log");
    let first = a.read().unwrap().0.remove(0);
    crate::oplog::Oplog::<SignedOp>::new(a.dir().to_path_buf())
        .replace_all(&[first])
        .unwrap();
    assert_eq!(kept(&a), whole(&a), "a shorter log renamed over it, as a restore would");
    let past_a_hole = signed(&key(3), 9, 7, record("seqs 0 to 6 never came"));
    assert!(b.ingest(&past_a_hole).unwrap());
    let (ours, (sent, _)) = kept(&a);
    assert_eq!((ours.clone(), (sent.clone(), false)), whole(&a), "an act past a hole");
    assert_eq!(sent, [past_a_hole], "sent, though the digests are equal");
}

/// **Each signature is checked once, as its line enters** (ceiling-3): the
/// offer to a peer not in step judges what is held by the signatures passed
/// as their lines came in, not by checking them all again. A tampered act
/// (alex's own line, its words changed under his signature) and a replayed
/// one (alex's act signed in another group) are held, as every line pushed
/// is, and still count for nothing: the offer is the one the file read whole
/// gives under [`Ed25519Verifier`], the act the peer lacks and nothing of
/// alex's window, which a tampered act believed would make look forked.
#[test]
fn a_tampered_or_replayed_act_is_still_refused_once_signatures_are_kept() {
    let dir = tempfile::tempdir().unwrap();
    let journal = open(dir.path());
    let r = ring();
    let alex: Vec<_> = (0..3)
        .map(|_| {
            journal
                .append(record("x"), &key(1), &r, None, &Ed25519Verifier)
                .unwrap()
        })
        .collect();
    journal
        .append(record("y"), &key(2), &r, None, &Ed25519Verifier)
        .unwrap();
    let lacked = journal
        .append(record("y"), &key(2), &r, None, &Ed25519Verifier)
        .unwrap();
    let peer = svrngs_core::digest(&journal.read().unwrap().0[..4], NS, &Ed25519Verifier);
    let mut tampered = alex[1].clone();
    tampered.kind.act = record("words alex never signed");
    let replayed = svrngs_core::tests_support::signed_in("another-group", &key(1), 101, 2, record("x"));
    assert!(journal.ingest(&tampered).unwrap() && journal.ingest(&replayed).unwrap());

    let ops = journal.read().unwrap().0;
    let whole = svrngs_core::ops_missing_from_within(&ops, NS, &Ed25519Verifier, &peer, NO_BUDGET);
    let ours = journal.digest().unwrap();
    assert_eq!(ours, svrngs_core::digest(&ops, NS, &Ed25519Verifier));
    let offer = journal.ops_missing_from_within(&peer, NO_BUDGET).unwrap();
    assert_eq!(
        serde_json::to_string(&offer).unwrap(),
        serde_json::to_string(&whole).unwrap()
    );
    assert_eq!(
        offer,
        (vec![lacked], false),
        "the act the peer lacks, and no window of alex's"
    );
    let gaps = journal.admit(&r, &Ed25519Verifier).unwrap();
    let refused = gaps.gaps().iter().filter(|g| matches!(g, RailGap::BadSignature { .. }));
    assert_eq!(refused.count(), 2, "{:?}", gaps.gaps());
}
