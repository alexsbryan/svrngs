//! Exact original proposal/vote/commit evidence for the installed recovery rule.
use crate::authority::*;
use crate::tests_source_graph::{accept, actor, cut, handover, link, redeem, roots, signed, GROUP};
use crate::*;
use alloc::{collections::BTreeSet, vec, vec::Vec};

fn recovery(
    target: u8,
    dispute: OfficeDispute,
    snapshot: PeopleSnapshot,
    departed: BTreeSet<OpId>,
    voters: &[u8],
) -> (Op<SignedOp>, Vec<Op<SignedOp>>, Op<SignedOp>) {
    let proposal = signed(
        target,
        0,
        None,
        30,
        RailAct::Authority {
            change: AuthorityChange::RecoveryProposal {
                choice: RecoveryChoice {
                    dispute,
                    target: actor(target),
                    departed,
                    snapshot: snapshot.clone(),
                },
            },
        },
    );
    let votes: Vec<_> = voters
        .iter()
        .map(|voter| {
            signed(
                *voter,
                1,
                (*voter == target).then(|| proposal.id.clone()),
                31,
                RailAct::Authority {
                    change: AuthorityChange::RecoveryVote {
                        proposal: proposal.id.clone(),
                    },
                },
            )
        })
        .collect();
    let approvals = voters
        .iter()
        .zip(&votes)
        .map(|(voter, vote)| (actor(*voter), vote.id.clone()))
        .collect();
    let mut after = snapshot;
    after
        .through
        .extend(votes.iter().map(|vote| vote.id.clone()));
    let commit = signed(
        target,
        2,
        votes
            .iter()
            .find(|vote| vote.actor == actor(target))
            .map(|vote| vote.id.clone()),
        32,
        RailAct::Authority {
            change: AuthorityChange::RecoveryCommit {
                proposal: proposal.id.clone(),
                votes: approvals,
                snapshot: after,
            },
        },
    );
    (proposal, votes, commit)
}

#[test]
fn recovery_requires_every_other_snapshot_person_and_never_the_forkers_vote() {
    let roots = roots();
    let one = handover(2, 0, None, 10, &roots);
    let two = handover(3, 0, None, 11, &roots);
    let branches = [one.id.clone(), two.id.clone()].into();
    let dispute = OfficeDispute::Claims {
        source: AuthorityRef::Genesis,
        branches,
    };
    let mut snapshot = roots.snapshot();
    snapshot.through.extend([one.id.clone(), two.id.clone()]);
    let (pending, votes, insufficient) = recovery(
        2,
        dispute.clone(),
        snapshot.clone(),
        Default::default(),
        &[2],
    );
    let held = ActSet::of([&one, &two, &pending, &votes[0], &insufficient]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(matches!(graph.office(), SourceOffice::Frozen { .. }));
    assert!(
        matches!(graph.assess(&insufficient.id), Err(SourceFailure::MissingApprovals { people, .. }) if people == [actor(3)].into())
    );
    let (proposal, votes, recovered) = recovery(
        2,
        dispute.clone(),
        snapshot.clone(),
        Default::default(),
        &[2, 3],
    );
    let mut original = vec![&one, &two, &proposal, &recovered];
    original.extend(&votes);
    let held = ActSet::of(original);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(
        matches!(graph.office(), SourceOffice::Held { source: AuthorityRef::Act(id), key, .. } if id == recovered.id && key == actor(2))
    );
    assert!(graph.assess(&recovered.id).is_ok());
    let forker_vote = signed(
        1,
        1,
        Some(one.id.clone()),
        31,
        RailAct::Authority {
            change: AuthorityChange::RecoveryVote {
                proposal: proposal.id.clone(),
            },
        },
    );
    let mut all = held;
    all.insert(forker_vote.clone());
    assert_eq!(
        NativeAuthority::new(&roots, &all, &Ed25519Verifier).assess(&forker_vote.id),
        Err(SourceFailure::Refused {
            why: SourceFault::Recovery
        })
    );
    let (bad, votes, denied) = recovery(1, dispute, snapshot, Default::default(), &[2, 3]);
    let held = ActSet::of([&one, &two, &bad, &denied, &votes[0], &votes[1]]);
    assert!(
        matches!(
            NativeAuthority::new(&roots, &held, &Ed25519Verifier).office(),
            SourceOffice::Frozen { .. }
        ),
        "the equivocating key cannot be reinstalled"
    );
}

#[test]
fn only_a_real_personal_exit_can_narrow_the_frozen_constituency() {
    let roots = roots();
    let one = handover(2, 0, None, 10, &roots);
    let two = handover(3, 0, None, 11, &roots);
    let took = accept(2, 0, None, &one);
    let branch_cut = cut(&took);
    let dispute = OfficeDispute::Claims {
        source: AuthorityRef::Genesis,
        branches: [one.id.clone(), two.id.clone()].into(),
    };
    let mut snapshot = roots.snapshot();
    snapshot
        .through
        .extend([one.id.clone(), two.id.clone(), branch_cut.id.clone()]);
    let (bad, votes, denied) = recovery(
        2,
        dispute.clone(),
        snapshot.clone(),
        [branch_cut.id.clone()].into(),
        &[2],
    );
    let held = ActSet::of([&one, &two, &took, &branch_cut, &bad, &denied, &votes[0]]);
    assert!(matches!(
        NativeAuthority::new(&roots, &held, &Ed25519Verifier).office(),
        SourceOffice::Frozen { .. }
    ));
    let exit = signed(
        3,
        0,
        None,
        25,
        RailAct::Remove {
            key: actor(3),
            through: Vec::new(),
        },
    );
    snapshot.through.insert(exit.id.clone());
    snapshot.people.remove(&actor(3));
    let (proposal, votes, recovered) =
        recovery(2, dispute, snapshot, [exit.id.clone()].into(), &[2]);
    let held = ActSet::of([
        &one,
        &two,
        &took,
        &branch_cut,
        &exit,
        &proposal,
        &recovered,
        &votes[0],
    ]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph.assess(&exit.id).is_ok());
    assert!(matches!(graph.office(), SourceOffice::Held { key, .. } if key == actor(2)));
}

#[test]
fn an_explicit_vacancy_uses_the_same_snapshot_rule_without_a_seed_fallback() {
    let roots = roots();
    let exit = signed(
        1,
        0,
        None,
        20,
        RailAct::Remove {
            key: actor(1),
            through: Vec::new(),
        },
    );
    let mut snapshot = roots.snapshot();
    snapshot.people.remove(&actor(1));
    snapshot.through.insert(exit.id.clone());
    let dispute = OfficeDispute::Vacancy {
        source: AuthorityRef::Genesis,
        exits: [exit.id.clone()].into(),
    };
    let (proposal, votes, recovered) =
        recovery(2, dispute, snapshot, [exit.id.clone()].into(), &[2, 3]);
    let held = ActSet::of([&exit, &proposal, &votes[0], &votes[1], &recovered]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph.assess(&recovered.id).is_ok());
    assert!(!graph.members().contains_key(&actor(1)));
    assert!(
        matches!(graph.office(), SourceOffice::Held { source: AuthorityRef::Act(id), key, .. } if id == recovered.id && key == actor(2))
    );
}

#[test]
fn devices_are_not_extra_recovery_votes_and_an_empty_remaining_people_set_cannot_rescue() {
    let seed = Roster::new(
        [
            (Person::parse("ana").unwrap(), vec![actor(1)]),
            (Person::parse("bo").unwrap(), vec![actor(2), actor(4)]),
            (Person::parse("cy").unwrap(), vec![actor(3)]),
        ]
        .into(),
    );
    let roots = SourceRoots::genesis(GROUP, &seed, &actor(1))
        .unwrap()
        .device(&actor(4), Device::Browser)
        .unwrap();
    let one = handover(2, 0, None, 10, &roots);
    let two = handover(3, 0, None, 11, &roots);
    let mut snapshot = roots.snapshot();
    snapshot.through.extend([one.id.clone(), two.id.clone()]);
    let dispute = OfficeDispute::Claims {
        source: AuthorityRef::Genesis,
        branches: [one.id.clone(), two.id.clone()].into(),
    };
    let (proposal, votes, denied) = recovery(2, dispute, snapshot, Default::default(), &[2, 4]);
    let mut forged = denied.kind.clone();
    let RailAct::Authority {
        change: AuthorityChange::RecoveryCommit {
            votes: approvals, ..
        },
    } = &mut forged.act
    else {
        unreachable!()
    };
    let phone = approvals.remove(&actor(4)).unwrap();
    approvals.insert(actor(3), phone);
    let forged = signed(2, 2, forged.prev.clone(), 32, forged.act);
    let held = ActSet::of([&one, &two, &proposal, &votes[0], &votes[1], &forged]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert_eq!(
        graph.assess(&forged.id),
        Err(SourceFailure::Refused {
            why: SourceFault::Recovery
        })
    );

    let seed = Roster::new(
        [(
            Person::parse("only-person").unwrap(),
            vec![actor(1), actor(2), actor(3)],
        )]
        .into(),
    );
    let roots = SourceRoots::genesis(GROUP, &seed, &actor(1)).unwrap();
    let one = handover(2, 0, None, 10, &roots);
    let two = handover(3, 0, None, 11, &roots);
    let mut snapshot = roots.snapshot();
    snapshot.through.extend([one.id.clone(), two.id.clone()]);
    let dispute = OfficeDispute::Claims {
        source: AuthorityRef::Genesis,
        branches: [one.id.clone(), two.id.clone()].into(),
    };
    let (proposal, votes, denied) = recovery(2, dispute, snapshot, Default::default(), &[2, 3]);
    let held = ActSet::of([&one, &two, &proposal, &votes[0], &votes[1], &denied]);
    assert_eq!(
        NativeAuthority::new(&roots, &held, &Ed25519Verifier).assess(&denied.id),
        Err(SourceFailure::Refused {
            why: SourceFault::Recovery
        })
    );
}

#[test]
fn competing_recovery_commits_freeze_the_common_source_and_a_known_duplicate_is_stale() {
    let roots = roots();
    let one = handover(2, 0, None, 10, &roots);
    let two = handover(3, 0, None, 11, &roots);
    let mut snapshot = roots.snapshot();
    snapshot.through.extend([one.id.clone(), two.id.clone()]);
    let dispute = OfficeDispute::Claims {
        source: AuthorityRef::Genesis,
        branches: [one.id.clone(), two.id.clone()].into(),
    };
    let (proposal, votes, recovered) = recovery(2, dispute, snapshot, Default::default(), &[2, 3]);
    let late = signed(
        2,
        3,
        Some(recovered.id.clone()),
        34,
        recovered.kind.act.clone(),
    );
    let held = ActSet::of([
        &one, &two, &proposal, &votes[0], &votes[1], &recovered, &late,
    ]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(
        matches!(graph.office(), SourceOffice::Held { source: AuthorityRef::Act(id), .. } if id == recovered.id)
    );
    assert_eq!(
        graph.assess(&late.id),
        Err(SourceFailure::Refused {
            why: SourceFault::Stale
        })
    );
    let competing = signed(
        2,
        2,
        recovered.kind.prev.clone(),
        35,
        recovered.kind.act.clone(),
    );
    let held = ActSet::of([
        &one, &two, &proposal, &votes[0], &votes[1], &recovered, &competing,
    ]);
    assert!(matches!(
        NativeAuthority::new(&roots, &held, &Ed25519Verifier).office(),
        SourceOffice::Frozen {
            source: AuthorityRef::Genesis,
            conflict: OfficeConflict::Recoveries { .. }
        }
    ));
}

/// The office fork both recovery-contest tests start from, and dee, admitted
/// by 3 after `before` on 3's chain.
fn forked_with_dee(
    before: Option<&Op<SignedOp>>,
) -> (
    SourceRoots,
    Op<SignedOp>,
    Op<SignedOp>,
    Op<SignedOp>,
    Op<SignedOp>,
) {
    let roots = roots();
    let one = handover(2, 0, None, 10, &roots);
    let two = handover(3, 0, None, 11, &roots);
    let seq = before.map_or(0, |line| line.kind.seq + 1);
    let invite = link(3, seq, before, 3);
    let admit = redeem(seq + 1, &invite, &invite, 9, 20);
    (roots, one, two, invite, admit)
}
fn recovered_over(
    roots: &SourceRoots,
    lines: &[&Op<SignedOp>],
    one: &Op<SignedOp>,
    two: &Op<SignedOp>,
    position: &Op<SignedOp>,
) -> (Op<SignedOp>, SourceOffice, Result<(), SourceFailure>) {
    let dispute = OfficeDispute::Claims {
        source: AuthorityRef::Genesis,
        branches: [one.id.clone(), two.id.clone()].into(),
    };
    let mut snapshot = roots.snapshot();
    snapshot
        .through
        .extend([one.id.clone(), two.id.clone(), position.id.clone()]);
    let (proposal, votes, commit) = recovery(2, dispute, snapshot, Default::default(), &[2, 3]);
    let mut all: Vec<_> = lines.to_vec();
    all.extend([one, two, &proposal, &commit]);
    all.extend(&votes);
    let held = ActSet::of(all);
    let graph = NativeAuthority::new(roots, &held, &Ed25519Verifier);
    (commit.clone(), graph.office(), graph.assess(&commit.id))
}

#[test]
fn a_person_admitted_under_an_invite_the_snapshot_had_seen_contests_a_recovery_without_them() {
    let (roots, one, two, invite, admit) = forked_with_dee(None);
    let (_, office, commit) = recovered_over(&roots, &[&invite, &admit], &one, &two, &invite);
    assert!(matches!(office, SourceOffice::Frozen { .. }));
    assert!(
        matches!(commit, Err(SourceFailure::Frozen { branches, .. }) if branches.contains(&admit.id))
    );
}

#[test]
fn a_person_admitted_under_an_invite_after_the_snapshot_contests_nothing() {
    let record = signed(
        3,
        0,
        None,
        5,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"notes/item"})).unwrap(),
        },
    );
    let (roots, one, two, invite, admit) = forked_with_dee(Some(&record));
    let lines = [&record, &invite, &admit];
    let (commit, office, assessed) = recovered_over(&roots, &lines, &one, &two, &record);
    assert!(
        matches!(office, SourceOffice::Held { source: AuthorityRef::Act(id), key } if id == commit.id && key == actor(2))
    );
    assert!(assessed.is_ok());
}
