// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail/src/journal.rs@e1a43bcd2
use super::*;

impl RingJournal {
    /// A test's fixture, written through [`commit`](svrngs_core::commit) like
    /// every other act: never compiled outside tests.
    pub fn append(
        &self,
        act: RailAct,
        signer: &SigningKey,
        roster: &Roster,
        _verifier: &dyn RingVerifier,
    ) -> Result<Op<SignedOp>, RailError> {
        let fixture = svrngs_core::tests_support::Fixture { act };
        let author = svrngs_core::Author {
            signer,
            roster,
            namespace: &self.namespace,
            now: now_unix(&self.namespace)?,
        };
        match svrngs_core::commit(self, &fixture, &author)? {
            Ok(recorded) => Ok(recorded.lines()[0].clone()),
            Err(_) => Err(RailError::NotInRoster {
                actor: actor_of(signer),
                namespace: self.namespace.clone(),
            }),
        }
    }
}

/// **The fold is made from the lines [`Held`] kept, and is the fold of the
/// file read whole** (svrn-docs `rings/current/INCREMENTAL_FOLD.md` Stage 1):
/// after its own commits, another handle's append, a peer's line synced in,
/// a shorter log renamed over it as a restore would, and a commit after
/// that, the fold the register reads, kept by a commit or made at the read,
/// is `admit` over the file read whole.
#[test]
fn the_fold_of_the_kept_lines_is_the_fold_of_the_file_read_whole() {
    use crate::tests::Doors;
    use svrngs_core::tests_support::*;
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (
        RingJournal::open(dir.path(), NS).unwrap(),
        RingJournal::open(dir.path(), NS).unwrap(),
    );
    let r = ring();
    let same = |why: &str| {
        a.folded(&r).unwrap();
        let kept = a.log().held.kept.clone().map(|(_, fold)| fold);
        assert_eq!(kept, Some(a.admit(&r, &Ed25519Verifier).unwrap()), "{why}");
    };
    for _ in 0..3 {
        a.append(record("x"), &key(1), &r, &Ed25519Verifier).unwrap();
    }
    same("its own commits");
    b.append(record("y"), &key(2), &r, &Ed25519Verifier).unwrap();
    same("another handle's append, from the tail");
    assert!(a.ingest(&signed(&key(3), 9, 0, record("from a peer"))).unwrap());
    same("a peer's line synced in");
    let first = a.read().unwrap().0.remove(0);
    crate::oplog::Oplog::<SignedOp>::new(a.dir().to_path_buf())
        .replace_all(&[first])
        .unwrap();
    same("a shorter log renamed over it, as a restore would");
    a.append(record("z"), &key(1), &r, &Ed25519Verifier).unwrap();
    same("a commit after the restore");
}
