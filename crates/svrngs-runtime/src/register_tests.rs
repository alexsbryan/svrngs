// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{actor_of, Device, Ed25519Verifier, OpId, RailAct, Redemption};
use svrngs_core::tests_support::{after, key, record, signed, NS};

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

/// Leaving out every act but `Admit`, `Remove`, `Correct` and `Invite`
/// changes nobody's standing: records between the membership acts, an
/// `Admit` voided, a `Remove`, a record voided, and an `Admit` redeeming
/// an `Invite` fold to the same standing keys and persons from those acts
/// alone as from the whole journal. The failing inputs are a filter that
/// leaves out `Correct`, and one that leaves out `Invite`.
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
    let remove_cy = after(
        &key(1),
        &void_dee,
        106,
        RailAct::Remove {
            key: actor_of(&key(3)),
            through_seq: 0,
        },
    );
    let ops = vec![
        admit_bo,
        bo_writes.clone(),
        after(&key(2), &bo_writes, 102, admit_act("cy", 3)),
        admit_dee.clone(),
        void_dee,
        signed(&key(3), 105, 0, record("b")),
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
            let mut v = vec![actor_of(&key(1)), actor_of(&key(2)), actor_of(&key(5))];
            v.sort();
            v
        },
        "control: alex, bo and eve stand; cy was removed and dee's Admit voided"
    );
    assert_eq!(standing_in(&ops, &seed, NS, &Ed25519Verifier).unwrap(), expected);
}
