//! The native source interpreter uses actual signed native envelopes and roots.
use crate::authority::*;
use crate::*;
use alloc::{string::String, vec, vec::Vec};

pub(super) const GROUP: &str = "native-source-graph";
pub(super) fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}
pub(super) fn actor(n: u8) -> String {
    actor_of(&key(n))
}
pub(super) fn roots() -> SourceRoots {
    let seed = Roster::new(
        [
            (Person::parse("ana").unwrap(), vec![actor(1)]),
            (Person::parse("bo").unwrap(), vec![actor(2)]),
            (Person::parse("cy").unwrap(), vec![actor(3)]),
        ]
        .into(),
    );
    SourceRoots::genesis(GROUP, &seed, &actor(1)).unwrap()
}
impl SourceRoots {
    /// The same roots with one genesis key held by another device.
    pub(crate) fn device(mut self, key: &str, device: Device) -> Result<Self, SourceFailure> {
        let mut members = self.members().clone();
        let Some(member) = members.get_mut(key) else {
            return Err(SourceFailure::Refused {
                why: SourceFault::Binding,
            });
        };
        member.device = device;
        self.model = eudoxia_community::Roots::genesis(members, self.office().into())?;
        Ok(self)
    }
}
pub(super) fn signed(by: u8, seq: u64, prev: Option<OpId>, ts: i64, act: RailAct) -> Op<SignedOp> {
    sign_act(&key(by), GROUP, ts, seq, prev, act, None, None, None)
}
/// The genesis holder's hand-on to `to`, a line on its own chain.
pub(super) fn handover(
    to: u8,
    seq: u64,
    prev: Option<OpId>,
    ts: i64,
    _: &SourceRoots,
) -> Op<SignedOp> {
    signed(1, seq, prev, ts, RailAct::HandOn { key: actor(to) })
}
/// The recipient's Accept of `hand`, on its own chain.
pub(super) fn accept(by: u8, seq: u64, prev: Option<OpId>, hand: &Op<SignedOp>) -> Op<SignedOp> {
    let hand_on = hand.id.clone();
    signed(by, seq, prev, 11, RailAct::Accept { hand_on })
}
/// The new holder's cut of 3, after its Accept.
pub(super) fn cut(took: &Op<SignedOp>) -> Op<SignedOp> {
    let remove = RailAct::Remove {
        key: actor(3),
        through: Vec::new(),
    };
    signed(2, took.kind.seq + 1, Some(took.id.clone()), 12, remove)
}

#[test]
fn office_fork_freezes_its_power_without_selecting_a_cut_or_freezing_independent_data() {
    let roots = roots();
    let hand = handover(2, 0, None, 10, &roots);
    let fork = handover(3, 0, None, -500, &roots);
    let took = accept(2, 0, None, &hand);
    let cut = cut(&took);
    let data = signed(
        3,
        0,
        None,
        13,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"ordinary","value":"independent"}))
                .unwrap(),
        },
    );
    let one = ActSet::of([&hand, &took, &cut, &data]);
    let normal = NativeAuthority::new(&roots, &one, &Ed25519Verifier);
    assert!(
        !normal.members().contains_key(&actor(3)),
        "uncontested cut is the positive control"
    );
    assert!(normal.assess(&data.id).is_err());
    let all = ActSet::of([&hand, &took, &fork, &cut, &data]);
    let conflicted = NativeAuthority::new(&roots, &all, &Ed25519Verifier);
    assert!(matches!(
        conflicted.office(),
        SourceOffice::Frozen {
            source: AuthorityRef::Genesis,
            ..
        }
    ));
    assert!(conflicted.members().contains_key(&actor(3)));
    assert!(conflicted.assess(&data.id).is_ok());
    assert!(matches!(
        conflicted.assess(&cut.id),
        Err(SourceFailure::Frozen { .. })
    ));
    let reversed = ActSet::of([&data, &cut, &fork, &took, &hand, &fork]);
    assert_eq!(
        conflicted.office(),
        NativeAuthority::new(&roots, &reversed, &Ed25519Verifier).office()
    );
}

#[test]
fn a_bare_admit_has_no_source_proven_joining_consent() {
    let roots = roots();
    let bare = signed(
        1,
        0,
        None,
        10,
        RailAct::Admit {
            person: Person::parse("dee").unwrap(),
            key: actor(9),
            invite: None,
        },
    );
    let held = ActSet::of([&bare]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(!graph.members().contains_key(&actor(9)));
    assert_eq!(
        graph.assess(&bare.id),
        Err(SourceFailure::Refused {
            why: SourceFault::BareAdmit
        })
    );
}

#[test]
fn stale_or_invalid_successors_cannot_freeze_or_reinstall_the_original_holder() {
    let roots = roots();
    let hand = handover(2, 0, None, 10, &roots);
    let late = handover(3, 1, Some(hand.id.clone()), -900, &roots);
    let mut forged = handover(3, 0, None, 11, &roots);
    forged.kind.sig.replace_range(..2, "ff");
    let took = accept(2, 0, None, &hand);
    let held = ActSet::of([&hand, &took, &late, &forged]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(matches!(graph.office(), SourceOffice::Held { key, .. } if key == actor(2)));
    assert_eq!(
        graph.assess(&late.id),
        Err(SourceFailure::Refused {
            why: SourceFault::Stale
        })
    );
    let phone_roots = roots.clone().device(&actor(3), Device::Browser).unwrap();
    let phone = handover(3, 0, None, 10, &phone_roots);
    let held = ActSet::of([&phone]);
    assert!(
        matches!(NativeAuthority::new(&phone_roots, &held, &Ed25519Verifier).office(), SourceOffice::Held { key, .. } if key == actor(1))
    );
}

fn joining(issuer: u8, joining: u8, name: &str, device: Device) -> (Op<SignedOp>, Op<SignedOp>) {
    let link = RailAct::Invite {
        key: actor(100 + issuer),
        until: 100,
        device_of: None,
    };
    let invite = signed(issuer, 0, None, 10, link);
    let person = Person::parse(name).unwrap();
    let message = source_redemption_message(GROUP, &invite.id, &person, &actor(joining), device);
    let redemption = Redemption {
        id: invite.id.clone(),
        sig: sign_message(&key(joining), &message),
        device,
    };
    let admit = signed(
        issuer,
        1,
        Some(invite.id.clone()),
        20,
        RailAct::Admit {
            person,
            key: actor(joining),
            invite: Some(redemption),
        },
    );
    (invite, admit)
}

#[test]
fn joining_requires_its_real_invitation_source_and_device_specific_consent() {
    let roots = roots();
    let (invite, admit) = joining(1, 9, "dee", Device::Browser);
    let held = ActSet::of([&invite, &admit]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    let member = graph
        .identity(&actor(9), &AuthorityRef::Act(admit.id.clone()))
        .unwrap();
    assert_eq!(member.device, Device::Browser);
    assert_eq!(member.first, actor(9));
    let mut forged = admit.kind.clone();
    let RailAct::Admit {
        invite: Some(r), ..
    } = &mut forged.act
    else {
        unreachable!()
    };
    r.device = Device::Computer;
    let mut forged = unsigned_act(actor(1), GROUP, 20, forged);
    forged.kind.sig = sign_ring_op(&key(1), &forged.id);
    let held = ActSet::of([&invite, &forged]);
    assert_eq!(
        NativeAuthority::new(&roots, &held, &Ed25519Verifier)
            .identity(&actor(9), &AuthorityRef::Act(forged.id.clone())),
        Err(SourceFailure::Refused {
            why: SourceFault::Consent
        })
    );
}

#[test]
fn genuine_binding_conflict_blocks_that_key_but_invalid_unheld_claims_do_not() {
    let roots = roots();
    let (one, a) = joining(1, 9, "dee", Device::Computer);
    let (two, b) = joining(2, 9, "someone-else", Device::Computer);
    let data = signed(
        9,
        0,
        None,
        30,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"notes/item"})).unwrap(),
        },
    );
    let held = ActSet::of([&one, &a, &two, &b, &data]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(matches!(
        graph.identity(&actor(9), &AuthorityRef::Act(a.id.clone())),
        Err(SourceFailure::Frozen { .. })
    ));
    assert!(graph.assess(&data.id).is_err());
    assert_eq!(graph.members().len(), 3);
    let mut bogus = b.kind.clone();
    bogus.prev = Some(crate::tests_support::stray_id(17));
    let mut bogus = unsigned_act(actor(2), GROUP, 20, bogus);
    bogus.kind.sig = sign_ring_op(&key(2), &bogus.id);
    let held = ActSet::of([&one, &a, &bogus, &data]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph
        .identity(&actor(9), &AuthorityRef::Act(a.id.clone()))
        .is_ok());
    assert!(graph.assess(&data.id).is_ok());
}

#[test]
fn personal_exit_is_unconditional_and_vacates_the_office_when_only_phones_remain() {
    let roots = roots()
        .device(&actor(2), Device::Browser)
        .unwrap()
        .device(&actor(3), Device::Browser)
        .unwrap();
    let exit = signed(
        1,
        0,
        None,
        10,
        RailAct::Remove {
            key: actor(1),
            through: Vec::new(),
        },
    );
    let late = signed(
        1,
        1,
        Some(exit.id.clone()),
        -1000,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"notes/late"})).unwrap(),
        },
    );
    let held = ActSet::of([&exit, &late]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph.assess(&exit.id).is_ok());
    assert!(!graph.members().contains_key(&actor(1)));
    assert_eq!(graph.members().len(), 2);
    assert!(matches!(
        graph.office(),
        SourceOffice::Vacant {
            source: AuthorityRef::Genesis,
            ..
        }
    ));
    assert!(
        graph.assess(&late.id).is_err(),
        "a display-time backdate cannot undo exit"
    );
}

#[test]
fn exit_and_handover_equivocation_stays_frozen_but_never_traps_personal_exit() {
    let roots = roots();
    let handed = handover(2, 0, None, 10, &roots);
    let exit = signed(
        1,
        0,
        None,
        9,
        RailAct::Remove {
            key: actor(1),
            through: Vec::new(),
        },
    );
    let held = ActSet::of([&handed, &exit]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph.assess(&exit.id).is_ok());
    assert!(!graph.members().contains_key(&actor(1)));
    assert!(matches!(
        graph.office(),
        SourceOffice::Frozen {
            source: AuthorityRef::Genesis,
            ..
        }
    ));
}

#[test]
fn a_binding_cannot_outlive_its_uncovered_invitation_and_admission_support() {
    let roots = roots();
    let hand = handover(2, 0, None, 10, &roots);
    let (invite, admit) = joining(1, 9, "dee", Device::Computer);
    let took = accept(2, 0, None, &hand);
    let removal = signed(
        2,
        1,
        Some(took.id.clone()),
        30,
        RailAct::Remove {
            key: actor(1),
            through: vec![],
        },
    );
    let data = signed(
        9,
        0,
        None,
        40,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"notes/item"})).unwrap(),
        },
    );
    let held = ActSet::of([&hand, &took, &invite, &admit, &removal, &data]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph.assess(&admit.id).is_err());
    assert!(graph.assess(&data.id).is_err());
    assert!(
        !graph.members().contains_key(&actor(9)),
        "refused admission cannot supply current standing"
    );

    // The cut explicitly retains a legitimate earlier admission. The inviter's
    // later removal does not remove the independently joined person.
    let removal = signed(
        2,
        1,
        Some(took.id.clone()),
        30,
        RailAct::Remove {
            key: actor(1),
            through: vec![admit.id.clone()],
        },
    );
    let held = ActSet::of([&hand, &took, &invite, &admit, &removal, &data]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph.assess(&admit.id).is_ok());
    assert!(graph.assess(&data.id).is_ok());
    assert!(graph.members().contains_key(&actor(9)));
}

#[test]
fn personal_exit_withdraws_uncovered_binding_support_but_not_covered_joining() {
    let roots = roots();
    let (invite, admit) = joining(2, 9, "dee", Device::Computer);
    let exit = signed(
        2,
        2,
        None,
        30,
        RailAct::Remove {
            key: actor(2),
            through: Vec::new(),
        },
    );
    let data = signed(
        9,
        0,
        None,
        40,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"notes/item"})).unwrap(),
        },
    );
    let held = ActSet::of([&invite, &admit, &exit, &data]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph.assess(&admit.id).is_err());
    assert!(
        graph.assess(&data.id).is_err(),
        "a retained identity name is not retained authority"
    );
    assert!(!graph.members().contains_key(&actor(9)));

    let exit = signed(2, 2, Some(admit.id.clone()), 30, exit.kind.act.clone());
    let held = ActSet::of([&invite, &admit, &exit, &data]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph.assess(&data.id).is_ok());
    assert!(graph.members().contains_key(&actor(9)));
    assert!(!graph.members().contains_key(&actor(2)));
}

#[test]
fn source_assessment_does_not_treat_every_signed_member_control_as_established() {
    let roots = roots();
    let data = signed(
        1,
        0,
        None,
        20,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"notes/item"})).unwrap(),
        },
    );
    let mut future = data.clone();
    future.v = SignedOp::UNDERSTANDS + 1;
    let held = ActSet::of([&future]);
    assert!(NativeAuthority::new(&roots, &held, &Ed25519Verifier)
        .assess(&future.id)
        .is_err());
    let mut forged = data.kind;
    forged.on_behalf_of = Some("somebody-else".into());
    let mut forged = unsigned_act(actor(1), GROUP, 20, forged);
    forged.kind.sig = sign_ring_op(&key(1), &forged.id);
    let held = ActSet::of([&forged]);
    assert_eq!(
        NativeAuthority::new(&roots, &held, &Ed25519Verifier).assess(&forged.id),
        Err(SourceFailure::Refused {
            why: SourceFault::Use
        })
    );
}

#[test]
fn correction_uses_the_targets_source_person_and_part_not_a_claimed_name() {
    let seed = Roster::new(
        [
            (Person::parse("ana").unwrap(), vec![actor(1), actor(4)]),
            (Person::parse("bo").unwrap(), vec![actor(2)]),
        ]
        .into(),
    );
    let roots = SourceRoots::genesis(GROUP, &seed, &actor(1))
        .unwrap()
        .device(&actor(4), Device::Browser)
        .unwrap();
    let record = signed(
        1,
        0,
        None,
        10,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"notes/item","value":"original"}))
                .unwrap(),
        },
    );
    let correction = |by, kind| {
        signed(
            by,
            0,
            None,
            20,
            RailAct::Correct {
                corrects: record.id.clone(),
                replacement: Some(
                    Payload::new(serde_json::json!({"kind":kind,"value":"corrected"})).unwrap(),
                ),
            },
        )
    };
    let others = correction(2, "notes/item");
    let wrong_part = correction(4, "money/item");
    let own_phone = correction(4, "notes/item");
    let held = ActSet::of([&record, &others, &wrong_part, &own_phone]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(graph.assess(&own_phone.id).is_ok());
    assert!(
        graph.assess(&others.id).is_err(),
        "membership does not grant another person's correction"
    );
    assert!(
        graph.assess(&wrong_part.id).is_err(),
        "even an author's correction cannot move history into another app"
    );
    let missing = signed(
        4,
        0,
        None,
        20,
        RailAct::Correct {
            corrects: crate::tests_support::stray_id(71),
            replacement: None,
        },
    );
    let held = ActSet::of([&missing]);
    assert!(matches!(
        NativeAuthority::new(&roots, &held, &Ed25519Verifier).assess(&missing.id),
        Err(SourceFailure::Missing { .. })
    ));
}

#[test]
fn an_unsigned_future_version_alias_cannot_displace_the_verified_supported_original() {
    let roots = roots();
    let data = signed(
        1,
        0,
        None,
        20,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"notes/item"})).unwrap(),
        },
    );
    let mut held = ActSet::of([&data]);
    for version in (AUTHORITY_LINE_VERSION + 1)..=(AUTHORITY_LINE_VERSION + 32) {
        let mut alias = data.clone();
        alias.v = version;
        assert_eq!(
            derive_id(&alias, GROUP),
            data.id,
            "the envelope version is not a signed field"
        );
        held.insert(alias);
    }
    assert!(NativeAuthority::new(&roots, &held, &Ed25519Verifier)
        .assess(&data.id)
        .is_ok());
}

/// A link by `issuer`, after `prev` on its chain.
pub(super) fn link(issuer: u8, seq: u64, prev: Option<&Op<SignedOp>>, n: u8) -> Op<SignedOp> {
    let act = RailAct::Invite {
        key: actor(100 + n),
        until: 100,
        device_of: None,
    };
    signed(issuer, seq, prev.map(|p| p.id.clone()), 10, act)
}
/// `joining`'s key redeems `invite`, admitted by the invite's issuer at `ts`.
pub(super) fn redeem(
    seq: u64,
    prev: &Op<SignedOp>,
    invite: &Op<SignedOp>,
    joining: u8,
    ts: i64,
) -> Op<SignedOp> {
    let person = Person::parse("dee").unwrap();
    let device = Device::Computer;
    let message = source_redemption_message(GROUP, &invite.id, &person, &actor(joining), device);
    let redemption = Redemption {
        id: invite.id.clone(),
        sig: sign_message(&key(joining), &message),
        device,
    };
    let admit = RailAct::Admit {
        person,
        key: actor(joining),
        invite: Some(redemption),
    };
    let issuer = key_number(&invite.actor);
    signed(issuer, seq, Some(prev.id.clone()), ts, admit)
}
fn key_number(actor_hex: &str) -> u8 {
    (1..=9).find(|n| actor(*n) == actor_hex).unwrap()
}
fn joins(
    roots: &SourceRoots,
    lines: &[&Op<SignedOp>],
    admit: &Op<SignedOp>,
) -> Result<SourceMember, SourceFailure> {
    let held = ActSet::of(lines.iter().copied());
    NativeAuthority::new(roots, &held, &Ed25519Verifier)
        .identity(&actor(9), &AuthorityRef::Act(admit.id.clone()))
}

#[test]
fn a_redemption_signed_at_or_after_its_links_until_is_expired() {
    let roots = roots();
    let invite = link(1, 0, None, 1);
    let timely = redeem(1, &invite, &invite, 9, 99);
    assert!(joins(&roots, &[&invite, &timely], &timely).is_ok());
    let late = redeem(1, &invite, &invite, 9, 100);
    assert_eq!(
        joins(&roots, &[&invite, &late], &late),
        Err(SourceFailure::Refused {
            why: SourceFault::Expired
        })
    );
}

#[test]
fn the_issuers_newer_link_closes_the_older_by_its_own_chain_not_by_time() {
    let roots = roots();
    let older = link(1, 0, None, 1);
    let newer = link(1, 1, Some(&older), 2);
    let after = redeem(2, &newer, &older, 9, 20);
    assert_eq!(
        joins(&roots, &[&older, &newer, &after], &after),
        Err(SourceFailure::Refused {
            why: SourceFault::Invitation
        })
    );
    let before = redeem(1, &older, &older, 9, 20);
    let closing = link(1, 2, Some(&before), 2);
    assert!(joins(&roots, &[&older, &before, &closing], &before).is_ok());
}

#[test]
fn a_redemption_on_a_fork_of_its_issuers_chain_beside_the_newer_link_freezes() {
    let roots = roots();
    let older = link(1, 0, None, 1);
    let newer = link(1, 1, Some(&older), 2);
    let beside = redeem(1, &older, &older, 9, 20);
    assert_eq!(
        joins(&roots, &[&older, &newer, &beside], &beside),
        Err(SourceFailure::Frozen {
            source: AuthorityRef::Act(older.id.clone()),
            branches: [beside.id.clone(), newer.id.clone()].into(),
        })
    );
}

#[test]
fn a_hand_on_with_neither_accept_nor_withdrawal_is_pending_and_the_holder_is_stale() {
    let roots = roots();
    let hand = handover(2, 0, None, 10, &roots);
    let meets = signed(
        1,
        1,
        Some(hand.id.clone()),
        12,
        RailAct::Meets {
            relays: Default::default(),
        },
    );
    let held = ActSet::of([&hand, &meets]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert_eq!(
        graph.office(),
        SourceOffice::Pending {
            source: AuthorityRef::Genesis,
            hand_on: hand.id.clone(),
        }
    );
    assert_eq!(
        graph.assess(&meets.id),
        Err(SourceFailure::Refused {
            why: SourceFault::Stale
        })
    );
}

#[test]
fn a_hand_on_withdrawn_by_the_holder_to_its_own_key_resumes_the_holder() {
    let roots = roots();
    let hand = handover(2, 0, None, 10, &roots);
    let back = handover(1, 1, Some(hand.id.clone()), 11, &roots);
    let meets = signed(
        1,
        2,
        Some(back.id.clone()),
        12,
        RailAct::Meets {
            relays: Default::default(),
        },
    );
    let held = ActSet::of([&hand, &back, &meets]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(matches!(graph.office(), SourceOffice::Held { key, .. } if key == actor(1)));
    assert!(graph.assess(&back.id).is_ok());
    assert!(graph.assess(&meets.id).is_ok());
}

#[test]
fn an_accept_and_a_withdrawal_of_one_hand_on_both_held_freeze_the_office() {
    let roots = roots();
    let hand = handover(2, 0, None, 10, &roots);
    let took = accept(2, 0, None, &hand);
    let back = handover(1, 1, Some(hand.id.clone()), 11, &roots);
    let held = ActSet::of([&hand, &took, &back]);
    let graph = NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert!(matches!(
        graph.office(),
        SourceOffice::Frozen {
            source: AuthorityRef::Genesis,
            conflict: OfficeConflict::Claims { branches, .. },
        } if branches == [took.id.clone(), back.id.clone()].into()
    ));
    let accepted = ActSet::of([&hand, &took]);
    assert!(matches!(
        NativeAuthority::new(&roots, &accepted, &Ed25519Verifier).office(),
        SourceOffice::Held { key, .. } if key == actor(2)
    ));
}

#[test]
fn an_authority_act_after_a_hole_in_its_authors_chain_is_missing() {
    let roots = roots();
    let hole = crate::tests_support::stray_id(41);
    let act = RailAct::Invite {
        key: actor(101),
        until: 100,
        device_of: None,
    };
    let invite = signed(1, 1, Some(hole.clone()), 10, act);
    let held = ActSet::of([&invite]);
    assert_eq!(
        NativeAuthority::new(&roots, &held, &Ed25519Verifier).assess(&invite.id),
        Err(SourceFailure::Missing {
            sources: [AuthorityRef::Act(hole)].into(),
        })
    );
}
