// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{actor_of, Device, Ed25519Verifier, OpId, RailAct, Redemption};
use svrngs_core::tests_support::{after, key, record, ring, signed, NS};

fn admit_act(person: &str, joining: u8) -> RailAct {
    RailAct::Admit {
        person: Person::parse(person).unwrap(),
        key: actor_of(&key(joining)),
        invite: None,
    }
}

fn void(target: &OpId) -> RailAct {
    RailAct::Correct {
        corrects: target.clone(),
        replacement: None,
    }
}

/// Leaving out every act `standing_reads` does not keep changes nobody's
/// standing: records between the membership acts, an `Admit` corrected
/// (which voids nothing since U7), the keeper's `Remove` covering cy's
/// `Admit` of fay through cy's later record, a record voided, and an
/// `Admit` redeeming an `Invite` fold to the same standing keys and persons
/// from those acts alone as from the whole journal. The failing inputs are
/// a filter that leaves out `Invite`, and one that leaves out the acts of a
/// key a `Remove` names (U7: fay falls, her admission no longer covered).
#[test]
fn standing_from_the_membership_acts_alone_is_the_journals() {
    let seed = Roster::new(
        [(Person::parse("alex").unwrap(), vec![actor_of(&key(1))])]
            .into_iter()
            .collect(),
    );
    let admit_bo = signed(&key(1), 100, 0, admit_act("bo", 2));
    let bo_writes = signed(&key(2), 101, 0, record("a"));
    let admit_dee = after(&key(1), &admit_bo, 103, admit_act("dee", 4));
    let void_dee = after(&key(1), &admit_dee, 104, void(&admit_dee.id));
    let cy_admits_fay = signed(&key(3), 105, 0, admit_act("fay", 6));
    let cy_writes = after(&key(3), &cy_admits_fay, 105, record("b"));
    let remove_cy = after(
        &key(1),
        &void_dee,
        106,
        RailAct::Remove {
            key: actor_of(&key(3)),
            through: vec![cy_writes.id.clone()],
        },
    );
    let ops = vec![
        admit_bo,
        bo_writes.clone(),
        after(&key(2), &bo_writes, 102, admit_act("cy", 3)),
        admit_dee.clone(),
        void_dee,
        cy_admits_fay,
        cy_writes,
        remove_cy.clone(),
        after(&key(1), &remove_cy, 107, void(&bo_writes.id)),
    ];
    let inv = after(
        &key(1),
        ops.last().unwrap(),
        108,
        RailAct::Invite {
            key: actor_of(&key(9)),
            until: 200,
        },
    );
    let eve = Person::parse("eve").unwrap();
    let redeem_eve = after(
        &key(1),
        &inv,
        109,
        RailAct::Admit {
            invite: Some(Redemption::sign(&key(5), NS, &inv.id, &eve, Device::Computer)),
            person: eve,
            key: actor_of(&key(5)),
        },
    );
    let ops = [ops, vec![inv, redeem_eve]].concat();
    let whole = admit(&crate::ActSet::of(&ops), &[], &seed, NS, &Ed25519Verifier).project::<Membership>();
    let expected: BTreeMap<String, Member> = whole
        .standing
        .iter()
        .map(|k| {
            let member = Member {
                person: whole.bindings[k].clone(),
                device: Device::Computer,
            };
            (k.clone(), member)
        })
        .collect();
    assert_eq!(
        expected.keys().cloned().collect::<Vec<_>>(),
        {
            let mut v = [1, 2, 4, 5, 6].map(|k| actor_of(&key(k))).to_vec();
            v.sort();
            v
        },
        "control: alex, bo, dee, eve and fay stand; cy was removed"
    );
    assert_eq!(standing_in(&ops, &seed, NS, &Ed25519Verifier).unwrap(), expected);
}

/// **The register is caught up from the tail** (ceiling-4): who stands, as
/// [`RingRail::standing`] answers from the journal's kept fold, is at every
/// step the standing [`standing_in`] folds from the file read whole and the
/// seed on disk, and it is folded again exactly when an act that moves
/// membership arrived or the seed changed: own commits, another handle's
/// append from the tail, a record (kept), a new seed, a shorter log renamed
/// over the path, and a tampered `Admit` ingested (held, and still nobody's
/// standing). The failing inputs are a fold that is not voided by an act
/// that moves membership, and one that ignores the seed.
#[test]
fn the_kept_register_is_the_whole_files_and_folds_again_when_membership_moves() {
    use crate::oplog::Oplog;
    use std::sync::Arc;
    let dir = tempfile::tempdir().unwrap();
    let rail = RingRail::new(dir.path(), Arc::new(key(1)));
    let founded = |keys: &[u8]| {
        Roster::new(
            [(
                Person::parse("alex").unwrap(),
                keys.iter().map(|k| actor_of(&key(*k))).collect(),
            )]
            .into_iter()
            .collect(),
        )
    };
    rail.set_seed(NS, &founded(&[1])).unwrap();
    let other = crate::RingJournal::open(dir.path(), NS).unwrap();
    let step = |why: &str| {
        let seed = other.seed().unwrap().unwrap();
        let whole = standing_in(&other.read().unwrap().0, &seed, NS, &Ed25519Verifier).unwrap();
        let journal = rail.journal(NS).unwrap();
        assert_eq!(rail.standing(NS).unwrap(), Standing::Members(whole.clone()), "{why}");
        // `standing` above caught the fold up; this asks it again, unchanged.
        assert_eq!(
            journal.standing(&seed).unwrap(),
            (whole.clone(), true),
            "{why}: kept after"
        );
        whole.len()
    };
    let kept_before = |why: &str, kept: bool| {
        let seed = other.seed().unwrap().unwrap();
        assert_eq!(rail.journal(NS).unwrap().standing(&seed).unwrap().1, kept, "{why}");
    };
    assert_eq!(step("nothing written"), 1);

    rail.commit(NS, &svrngs_core::tests_support::Batch(vec![admit_act("bo", 2)]))
        .unwrap()
        .unwrap();
    kept_before("an Admit committed", false);
    assert_eq!(step("an Admit committed"), 2);

    other
        .append(record("x"), &key(2), &ring(), None, &Ed25519Verifier)
        .unwrap();
    kept_before("a record, from another handle", true);
    assert_eq!(step("a record, from another handle"), 2);

    let admit_cy = other
        .append(admit_act("cy", 3), &key(1), &ring(), None, &Ed25519Verifier)
        .unwrap();
    kept_before("another handle's Admit, from the tail", false);
    assert_eq!(step("another handle's Admit, from the tail"), 3);

    rail.set_seed(NS, &founded(&[1, 4])).unwrap();
    kept_before("a new seed", false);
    assert_eq!(step("a new seed"), 4);

    let first = other.read().unwrap().0.remove(0);
    Oplog::<SignedOp>::new(other.dir().to_path_buf())
        .replace_all(&[first])
        .unwrap();
    kept_before("a shorter log renamed over it", false);
    assert_eq!(step("a shorter log renamed over it"), 3, "cy's Admit is gone");

    let mut tampered = admit_cy;
    tampered.kind.act = admit_act("eve", 5);
    assert_eq!(
        rail.ingest_all(NS, &[tampered]).unwrap(),
        1,
        "held, as every pushed line is"
    );
    kept_before("a tampered Admit ingested", false);
    assert_eq!(step("a tampered Admit ingested"), 3, "eve stands nowhere");
}

/// **A cut key's earlier acts join the kept register** (U7 (c)). The kept
/// fold holds the acts that move membership; once a `Remove` names bo, bo's
/// record already held, which the cut names, is read in from the log, so
/// bo's `Admit` of fay, under the record in bo's chain, is covered and fay
/// stands, as in the whole file. The failing input is the kept fold without
/// that read: the cut names an act it does not hold, and fay falls.
#[test]
fn a_cut_keys_earlier_acts_join_the_kept_register() {
    use std::sync::Arc;
    let dir = tempfile::tempdir().unwrap();
    let rail = RingRail::new(dir.path(), Arc::new(key(1)));
    let alex = Roster::new(
        [(Person::parse("alex").unwrap(), vec![actor_of(&key(1))])]
            .into_iter()
            .collect(),
    );
    rail.set_seed(NS, &alex).unwrap();
    let bo_in = signed(&key(1), 100, 0, admit_act("bo", 2));
    let bo_admits_fay = signed(&key(2), 101, 0, admit_act("fay", 6));
    let bo_writes = after(&key(2), &bo_admits_fay, 102, record("bo-writes"));
    rail.ingest_all(NS, &[bo_in.clone(), bo_admits_fay, bo_writes.clone()])
        .unwrap();
    let Standing::Members(before) = rail.standing(NS).unwrap() else {
        panic!("held, seeded")
    };
    assert_eq!(before.len(), 3, "control: alex, bo and fay");
    let cut = after(
        &key(1),
        &bo_in,
        103,
        RailAct::Remove {
            key: actor_of(&key(2)),
            through: vec![bo_writes.id.clone()],
        },
    );
    rail.ingest_all(NS, &[cut]).unwrap();
    let journal = rail.journal(NS).unwrap();
    let whole = standing_in(&journal.read().unwrap().0, &alex, NS, &Ed25519Verifier).unwrap();
    let mut names: Vec<String> = whole.values().map(|m| m.person.to_string()).collect();
    names.sort();
    assert_eq!(names, ["alex", "fay"], "control: bo out, fay covered");
    assert_eq!(rail.standing(NS).unwrap(), Standing::Members(whole));
    let register = rail.register(NS).unwrap();
    assert_eq!(
        register.removed.into_iter().collect::<Vec<_>>(),
        [(actor_of(&key(2)), Person::parse("bo").unwrap())],
        "bo, cut, with bo's person"
    );
    assert_eq!(register.keeper, Some(actor_of(&key(1))), "alex keeps");
    assert_eq!(rail.register("not-held").unwrap(), Register::default());
}
