// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai oplog/src/lib.rs@e1a43bcd2
use super::*;
// Only the tests derive with these.
use serde::{Deserialize, Serialize};

// The log's test-only methods, out of oplog.rs so the budget counts no test code.
impl<K: Journaled> Oplog<K> {
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Append one op: [`Self::append_all`] of one, so a test writes a line
    /// as the journal does.
    pub fn append(&self, op: &Op<K>) -> Result<()> {
        self.append_all(std::slice::from_ref(op))
    }

    /// Every op in append order. A missing file is an empty log, not an error.
    ///
    /// Drops the skip list from [`Self::read_all_with_skips`]. That is the
    /// right call for a tenant deriving a *summary* it can restate later
    /// (governance, reconciliation, the meta-atlas bridge all re-run from the
    /// whole log). It is the WRONG call for a tenant whose answer is a number
    /// a person acts on — see the ring ledger, which reads the skips and
    /// reports them as gaps rather than confidently totalling a subset.
    pub fn read_all(&self) -> Result<Vec<Op<K>>> {
        self.read_all_with_skips().map(|(ops, _)| ops)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Probe {
    Touch { what: String },
}

impl Journaled for Probe {
    const FILE: &'static str = "probe_oplog.jsonl";
    const ID_PREFIX: &'static str = "probe";
    const LABEL: &'static str = "probe_oplog";
}

fn touch(what: &str) -> Probe {
    Probe::Touch { what: what.into() }
}

#[test]
fn id_is_derived_from_the_act_not_from_position() {
    let a = Op::new(touch("x"), 100, "human:alex");
    let b = Op::new(touch("x"), 100, "human:alex");
    let c = Op::new(touch("y"), 100, "human:alex");
    assert_eq!(a.id, b.id, "same act, same second, same actor => same id");
    assert_ne!(a.id, c.id, "a different act must not reuse the id");
    // svrngs (U2): the whole hash, 64 hex, where upstream wore the
    // prefix and kept 64 bits.
    assert_eq!(a.id.to_string().len(), 64);
}

/// THE WIRE TEST. `OpId` is content-addressed and **gossiped** — a peer
/// reads ids minted by a build it does not have, and a `Revert` targets one
/// by value. So the derivation is a wire format, and this pins it to a
/// literal rather than to a re-computation, which would agree with any
/// change it was asked to check.
///
/// If this fails, you changed the id derivation. That is a mesh-wide
/// breaking change and it needs a `Journaled::VERSION` bump and a fleet
/// rollout, not a new expected value here. (svrngs, U2: re-pinned when
/// `OpId` became the whole hash, before any journal outside tests
/// existed.)
#[test]
fn the_id_derivation_is_pinned_to_the_wire() {
    let op = Op::new(touch("x"), 100, "human:alex");
    assert_eq!(
        op.id.to_string(),
        "97c56dfd0781629dd3907ff711e43ba356dd2ea3efa1bcd5b308902c85826c9f",
        "the id derivation is <prefix>|<ts>|<actor>|<body-json> hashed with \
         ContentHash, whole — see the note on this test before touching any \
         of those five"
    );
}

#[test]
fn the_prefix_is_hashed_so_two_tenants_cannot_collide() {
    // Same ts, same actor, same JSON body, different tenant prefix.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "op", rename_all = "snake_case")]
    enum Other {
        Touch { what: String },
    }
    impl Journaled for Other {
        const FILE: &'static str = "other_oplog.jsonl";
        const ID_PREFIX: &'static str = "other";
        const LABEL: &'static str = "other_oplog";
    }
    let a = Op::new(touch("x"), 100, "ingest");
    let b = Op::new(Other::Touch { what: "x".to_string() }, 100, "ingest");
    assert_ne!(
        a.id, b.id,
        "the hashed input must include the prefix, not only the body"
    );
}

/// **The one shrinking operation, and it leaves nothing behind.** The
/// temp file is a sibling of the log, so a compaction that returned
/// without cleaning up would leave a `.compacting` file in every ring
/// directory — harmless, and exactly the kind of litter that later reads
/// as a crashed run.
#[test]
fn replace_all_swaps_the_whole_log_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path());
    let ops: Vec<Op<Probe>> = ["a", "b", "c"]
        .iter()
        .map(|w| Op::new(touch(w), 7, "human:alex"))
        .collect();
    log.append_all(&ops).unwrap();

    log.replace_all(&ops[2..]).unwrap();
    assert_eq!(log.read_all().unwrap(), ops[2..].to_vec());
    assert!(
        !dir.path().join("probe_oplog.compacting").exists(),
        "the temp file outlived the rename"
    );

    // Replacing with nothing is an EMPTY log, not a missing one: a
    // namespace whose every line was retired still exists.
    log.replace_all(&[]).unwrap();
    assert!(log.path().exists(), "the file was unlinked, not emptied");
    assert!(log.read_all().unwrap().is_empty());

    // And it is still a journal afterwards — append picks up from the
    // replaced file rather than from whatever was there before.
    let next = Op::new(touch("d"), 8, "human:alex");
    log.append(&next).unwrap();
    assert_eq!(log.read_all().unwrap(), vec![next]);
}

#[test]
fn round_trips_through_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path().join("atlas"));
    let op = Op::new(touch("x"), 7, "human:alex");
    log.append(&op).unwrap();
    let back = log.read_all().unwrap();
    assert_eq!(back, vec![op]);
    assert!(log.path().ends_with("probe_oplog.jsonl"));
}

#[test]
fn the_line_is_one_flat_object_with_the_kind_inlined() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path());
    log.append(&Op::new(touch("x"), 7, "ingest")).unwrap();
    let raw = fs::read_to_string(log.path()).unwrap();
    let v: serde_json::Value = serde_json::from_str(raw.trim()).unwrap();
    for key in ["id", "v", "ts_unix", "actor", "op", "what"] {
        assert!(v.get(key).is_some(), "line is missing {key}: {raw}");
    }
}

#[test]
fn a_missing_log_reads_as_empty_not_as_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path().join("nope"));
    assert!(log.read_all().unwrap().is_empty());
}

/// The §18.3 property: a reader that skipped lines can SAY so. Before
/// `read_all_with_skips` a corrupt log and a clean one were indistinguishable
/// from the return value.
#[test]
fn the_skipped_lines_are_returned_not_only_logged() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path());
    let mut future = Op::new(touch("x"), 7, "ingest");
    future.v = Probe::VERSION + 1;
    log.append(&future).unwrap();
    log.append(&Op::new(touch("y"), 8, "ingest")).unwrap();
    fs::write(
        log.path(),
        format!("{}\nnot json at all\n", fs::read_to_string(log.path()).unwrap().trim()),
    )
    .unwrap();

    let (ops, skipped) = log.read_all_with_skips().unwrap();
    assert_eq!(ops.len(), 1);
    assert_eq!(skipped.len(), 2, "both classes of skip must be reported");
    assert!(matches!(
        skipped[0],
        SkippedLine::NewerVersion { line: 1, v } if v == Probe::VERSION + 1
    ));
    assert!(matches!(skipped[1], SkippedLine::Malformed { line: 3, .. }));
    // A clean log reports nothing, so a non-empty list is real signal.
    let clean: Oplog<Probe> = Oplog::new(dir.path().join("clean"));
    clean.append(&Op::new(touch("z"), 9, "ingest")).unwrap();
    assert!(clean.read_all_with_skips().unwrap().1.is_empty());
}

/// `O_APPEND` guarantees one `write(2)` lands whole. Two calls per logical
/// line did not have that guarantee, so a concurrent writer could split a
/// line across another's. Pinning the shape, not the race: every byte of a
/// line — newline included — comes from one buffer.
#[test]
fn a_logical_line_including_its_newline_is_written_once() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path());
    log.append_all(&[Op::new(touch("a"), 1, "ingest"), Op::new(touch("b"), 2, "ingest")])
        .unwrap();
    let raw = fs::read_to_string(log.path()).unwrap();
    assert!(raw.ends_with('\n'), "every line is newline-terminated");
    assert_eq!(raw.lines().count(), 2);
    assert_eq!(log.read_all().unwrap().len(), 2);
}

/// Leg 5 of `ra-membership-is-order-free`, at the envelope level: a line
/// declaring a version this build does not understand is skipped BEFORE
/// its body is parsed — never as `MalformedLine`, never as an admitted
/// op. A parse-first gate gets this exactly backwards for the lines that
/// matter most: an act whose KIND a build has never seen fails serde and
/// reads as Malformed, when the version bump exists to name it.
#[test]
fn a_newer_version_line_is_skipped_before_its_body_is_parsed() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path());
    fs::write(
        log.path(),
        format!(
            r#"{{"id":"probe-0000000000000000","v":{},"ts_unix":7,"actor":"ingest","kind":{{"op":"an_act_this_build_has_never_seen"}}}}"#,
            Probe::VERSION + 1
        ),
    )
    .unwrap();
    let (ops, skipped) = log.read_all_with_skips().unwrap();
    assert!(ops.is_empty(), "nothing admitted");
    assert!(
        matches!(skipped.as_slice(), [SkippedLine::NewerVersion { v, .. }] if *v == Probe::VERSION + 1),
        "the version gate names it — never a Malformed misread of the unknown kind: {skipped:?}"
    );
}

#[test]
fn a_line_from_a_newer_format_is_skipped_not_misread() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path());
    let mut future = Op::new(touch("x"), 7, "ingest");
    future.v = Probe::VERSION + 1;
    log.append(&future).unwrap();
    log.append(&Op::new(touch("y"), 8, "ingest")).unwrap();
    let back = log.read_all().unwrap();
    assert_eq!(back.len(), 1, "the future-version line must not be read");
    assert_eq!(back[0].ts_unix, 8);
}

#[test]
fn a_malformed_line_is_skipped_and_the_rest_survive() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path());
    log.append(&Op::new(touch("x"), 7, "ingest")).unwrap();
    fs::write(
        log.path(),
        format!("{}\nnot json at all\n", fs::read_to_string(log.path()).unwrap().trim()),
    )
    .unwrap();
    assert_eq!(log.read_all().unwrap().len(), 1);
}

#[test]
fn append_all_writes_every_op_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path());
    let ops = vec![Op::new(touch("a"), 1, "ingest"), Op::new(touch("b"), 2, "ingest")];
    log.append_all(&ops).unwrap();
    assert_eq!(log.read_all().unwrap(), ops);
    // Empty batch must not create the file.
    let empty: Oplog<Probe> = Oplog::new(dir.path().join("empty"));
    empty.append_all(&[]).unwrap();
    assert!(!empty.path().exists());
}

/// A cursor reads only what was appended after it, leaves a half-written
/// line for the next read, and starts over when a replace renames a new
/// file over the log.
#[test]
fn read_since_reads_the_tail_and_restarts_on_a_replace() {
    let dir = tempfile::tempdir().unwrap();
    let log: Oplog<Probe> = Oplog::new(dir.path());
    let first = log.read_since(Cursor::default()).unwrap();
    assert!(first.ops.is_empty() && !first.restarted);

    let a = Op::new(touch("a"), 1, "ingest");
    let b = Op::new(touch("b"), 2, "ingest");
    log.append_all(&[a.clone(), b.clone()]).unwrap();
    let both = log.read_since(first.cursor).unwrap();
    assert_eq!(both.ops, vec![a.clone(), b.clone()]);
    assert!(both.restarted, "the file did not exist at the first cursor");

    let c = Op::new(touch("c"), 3, "ingest");
    log.append(&c).unwrap();
    let mut f = OpenOptions::new().append(true).open(log.path()).unwrap();
    f.write_all(b"{\"id\":\"half").unwrap();
    let tail = log.read_since(both.cursor).unwrap();
    assert_eq!(tail.ops, vec![c.clone()], "only the line after the cursor");
    assert!(!tail.restarted);
    let len = std::fs::metadata(log.path()).unwrap().len();
    assert_eq!(
        tail.cursor.end,
        len - b"{\"id\":\"half".len() as u64,
        "a half-written line is not judged yet: the cursor stops before it"
    );

    log.replace_all(&[c.clone()]).unwrap();
    let again = log.read_since(tail.cursor).unwrap();
    assert!(again.restarted, "a replaced file voids the cursor");
    assert_eq!(again.ops, vec![c]);
}
