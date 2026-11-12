// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{actor_of, Device, Ed25519Verifier, RailAct};
use svrngs_core::tests_support::{key, record, NS};

/// The keys standing in `namespace` after `ops`, from `seed`: a fresh fold
/// of the whole file, which the tests hold the kept fold to (P5).
fn standing_in(ops: &[Op<SignedOp>], seed: &Roster, namespace: &str) -> BTreeMap<String, Member> {
    let a = crate::admit(&crate::ActSet::of(ops), &[], seed, namespace, &Ed25519Verifier);
    let removes: Vec<_> = (ops.iter())
        .filter(|o| matches!(o.kind.act, RailAct::Remove { .. }))
        .cloned()
        .collect();
    register_of(a.membership(), &removes, namespace).members
}

/// `joining` let in as `person` by a link this computer mints in `rail`.
fn let_in(rail: &RingRail, joining: u8, person: &str) {
    svrngs_core::tests_support::let_in(
        (NS, &actor_of(&key(1))),
        (&key(joining), person, Device::Computer),
        |mint| rail.commit(NS, mint).unwrap().unwrap().lines()[0].id.clone(),
        |redeem| drop(rail.commit(NS, redeem).unwrap().unwrap()),
    );
}

/// **The register is caught up from the tail** (ceiling-4): who stands, as
/// [`RingRail::standing`] answers from the journal's kept fold, is at every
/// step the standing [`standing_in`] folds from the file read whole and the
/// seed on disk, and it is folded again when any act arrived, since the
/// interpreter reads them all (G2), or the seed changed: own commits,
/// another handle's appends from the tail, a new seed, a shorter log
/// renamed over the path, and a tampered `Admit` ingested (held, and still
/// nobody's standing). The failing inputs are a fold that is not voided by
/// an act arriving, and one that ignores the seed.
#[test]
fn the_kept_register_is_the_whole_files_and_folds_again_when_membership_moves() {
    use crate::oplog::Oplog;
    use std::sync::Arc;
    use svrngs_core::tests_support::{invite_of, redeeming};
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
        let whole = standing_in(&other.read().unwrap().0, &seed, NS);
        let journal = rail.journal(NS).unwrap();
        assert_eq!(rail.standing(NS).unwrap(), Standing::Members(whole.clone()), "{why}");
        // `standing` above caught the fold up; this asks it again, unchanged.
        assert_eq!(
            journal.folded(&seed).map(|(r, kept)| (r.members, kept)).unwrap(),
            (whole.clone(), true),
            "{why}: kept after"
        );
        whole.len()
    };
    let kept_before = |why: &str, kept: bool| {
        let seed = other.seed().unwrap().unwrap();
        assert_eq!(rail.journal(NS).unwrap().folded(&seed).unwrap().1, kept, "{why}");
    };
    assert_eq!(step("nothing written"), 1);

    let_in(&rail, 2, "bo");
    kept_before("bo let in, committed", false);
    assert_eq!(step("bo let in, committed"), 2);

    let seed = || other.seed().unwrap().unwrap();
    other.append(record("x"), &key(2), &seed(), &Ed25519Verifier).unwrap();
    kept_before("a record, from another handle", false);
    assert_eq!(step("a record, from another handle"), 2);

    let link = other
        .append(invite_of(&key(9), i64::MAX, None), &key(1), &seed(), &Ed25519Verifier)
        .unwrap();
    let cy = redeeming(&link, "cy", &key(3), Device::Computer);
    let admit_cy = other.append(cy, &key(1), &seed(), &Ed25519Verifier).unwrap();
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
    assert_eq!(
        step("a shorter log renamed over it"),
        2,
        "bo's and cy's Admits are gone"
    );

    let mut tampered = admit_cy;
    tampered.kind.act = redeeming(&link, "eve", &key(5), Device::Computer);
    assert_eq!(
        rail.ingest_all(NS, &[tampered]).unwrap(),
        1,
        "held, as every pushed line is"
    );
    kept_before("a tampered Admit ingested", false);
    assert_eq!(step("a tampered Admit ingested"), 2, "eve stands nowhere");
}

/// A device standing in two groups a lend reaches as two different people
/// is refused, not resolved to whichever group lists first, so an allowed
/// request is never attributed to another account (LENDING_SCOPE §"One
/// policy"); one person in both resolves through the first eligible group,
/// and a group the lend does not reach never vouches. The failing input is
/// the first-match `find_map`, which answers `house`'s alex for key 2.
#[test]
fn a_device_two_eligible_groups_name_as_different_people_stands_in_neither() {
    use std::sync::Arc;
    let dir = tempfile::tempdir().unwrap();
    let rail = RingRail::new(dir.path(), Arc::new(key(1)));
    let roster = |person: &str, keys: &[u8]| {
        let keys = keys.iter().map(|k| actor_of(&key(*k))).collect();
        Roster::new([(Person::parse(person).unwrap(), keys)].into_iter().collect())
    };
    rail.set_seed("house", &roster("alex", &[1, 2])).unwrap();
    rail.set_seed("trip", &roster("bo", &[2])).unwrap();
    rail.set_seed("yard", &roster("alex", &[1, 2])).unwrap();
    let two = actor_of(&key(2));
    let person = |eligible: &[&str]| {
        let found = rail.standing_in(&two, |g| eligible.contains(&g));
        found.map(|(group, m)| (group, m.person_key))
    };
    assert_eq!(person(&["house", "trip"]), None, "two people for one device");
    assert_eq!(person(&["trip"]), Some(("trip".into(), two.clone())));
    let alex = Some(("house".into(), actor_of(&key(1))));
    assert_eq!(person(&["house", "yard"]), alex, "one person, the first group");
    assert_eq!(person(&["yard", "house"]), alex, "order of the ask changes nothing");
}
