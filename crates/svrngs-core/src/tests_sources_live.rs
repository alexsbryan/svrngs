//! A sources-v1 group decided live by its interpreter (EUDOXIA_JUDGE §2a,
//! R2 swap): every act written through the write side ([`crate::commit`])
//! and read back through admission, as a computer does, never through
//! `NativeAuthority`. Founding, an invitation with the joiner's device
//! consent, a phone, data, a cut, a hand-on with its Accept, a withdrawal,
//! a holder's fork freezing the office while data and exits still count,
//! and a recovery out of a hand-on nobody accepted.
use crate::authority::{AuthorityChange, AuthorityRef, GroupPolicy, OfficeDispute};
use crate::authority::{PeopleSnapshot, RecoveryChoice};
use crate::seed::Seed;
use crate::tests_support::{key, p, record, Fixture, Memory};
use crate::*;
use alloc::{collections::BTreeSet, string::String};

const V1: Option<GroupPolicy> = Some(GroupPolicy::SourcesV1);

struct Group {
    roster: Roster,
    ns: String,
    store: Memory,
}

impl Group {
    /// Ana (key 1) founds it.
    fn founded() -> Self {
        let name = |n: &str| Name::parse(n).unwrap();
        let seed = Seed::from_parts(name("House"), name("Ana"), key(1).verifying_key(), [3; 16])
            .with_source_policy();
        let roster = seed.roster();
        assert_eq!(roster.policy, V1, "the seed's policy reaches its roster");
        Self {
            roster,
            ns: seed.id(),
            store: Memory::holding(&[]),
        }
    }
    fn by<C: Command>(
        &self,
        k: u8,
        now: i64,
        command: &C,
    ) -> Result<Recorded, Refused<C::Refusal>> {
        let author = Author {
            signer: &key(k),
            roster: &self.roster,
            namespace: &self.ns,
            now,
        };
        let Ok(done) = commit(&self.store, command, &author);
        done
    }
    fn read(&self) -> Admission {
        admit(
            &self.store.0.borrow(),
            &[],
            &self.roster,
            &self.ns,
            &Ed25519Verifier,
        )
    }
    /// `issuer` lets `joining` in under its link `link` as `person`.
    fn invite(&self, issuer: u8, link: u8, now: i64, device_of: Option<String>) -> OpId {
        let mint = MintInvite {
            key: actor_of(&key(link)),
            until: now + 3600,
            device_of,
        };
        self.by(issuer, now, &mint).unwrap().lines()[0].id.clone()
    }
    fn redeem(
        &self,
        issuer: u8,
        link: u8,
        joining: u8,
        person: &str,
        device: Device,
        now: i64,
    ) -> Result<Recorded, Refused<RedeemRefusal>> {
        let invite = self.invite(issuer, link, now, None);
        self.redeem_under(issuer, link, invite, joining, person, device, V1, now)
    }
    #[allow(clippy::too_many_arguments)]
    fn redeem_under(
        &self,
        issuer: u8,
        link: u8,
        invite: OpId,
        joining: u8,
        person: &str,
        device: Device,
        policy: Option<GroupPolicy>,
        now: i64,
    ) -> Result<Recorded, Refused<RedeemRefusal>> {
        let consent =
            Redemption::sign(&key(joining), &self.ns, &invite, &p(person), device, policy);
        let redeem = Redeem {
            invite,
            person: p(person),
            key: actor_of(&key(joining)),
            cosig: consent.sig,
            device,
            link: actor_of(&key(link)),
            issuer: actor_of(&key(issuer)),
        };
        self.by(issuer, now, &redeem)
    }
    fn tip(&self, k: u8) -> Op<SignedOp> {
        let me = actor_of(&key(k));
        let held = self.store.0.borrow();
        let mine = held.iter().filter(|op| op.actor == me);
        mine.max_by_key(|op| op.kind.seq).unwrap().clone()
    }
    /// A line `k` signs by hand after `prev`, beside what its computer wrote.
    fn beside(&self, k: u8, prev: &Op<SignedOp>, ts: i64, act: RailAct) -> Op<SignedOp> {
        let seq = prev.kind.seq + 1;
        let line = sign_act(
            &key(k),
            &self.ns,
            ts,
            seq,
            Some(prev.id.clone()),
            act,
            None,
            None,
            None,
        );
        self.store.0.borrow_mut().insert(line.clone());
        line
    }
    /// Who the snapshot frame sees: every chain's newest line, every person.
    fn snapshot(&self) -> PeopleSnapshot {
        let a = self.read();
        let m = a.membership();
        let through: BTreeSet<OpId> = (self.store.0.borrow().iter())
            .map(|op| op.id.clone())
            .collect();
        let people = (m.standing.iter())
            .filter_map(|k| m.person_of(k))
            .map(|(first, person)| (String::from(first), person.clone()))
            .collect();
        PeopleSnapshot { through, people }
    }
}

fn ana() -> String {
    actor_of(&key(1))
}
fn bo() -> String {
    actor_of(&key(2))
}
fn cy() -> String {
    actor_of(&key(3))
}
fn note(what: &str) -> Fixture {
    Fixture { act: record(what) }
}
fn authority(change: AuthorityChange) -> Fixture {
    Fixture {
        act: RailAct::Authority { change },
    }
}

/// Bo and Cy joined Ana's group by link; Bo's phone (key 4) by Bo's device link.
fn three() -> Group {
    let g = Group::founded();
    assert!(g.redeem(1, 21, 2, "Bo", Device::Computer, 100).is_ok());
    assert!(g.redeem(1, 22, 3, "Cy", Device::Computer, 110).is_ok());
    let link = g.invite(2, 23, 120, Some(bo()));
    let phone = g.redeem_under(2, 23, link, 4, "Bo", Device::Browser, V1, 121);
    assert!(phone.is_ok(), "{phone:?}");
    g
}

#[test]
fn a_founded_group_is_decided_by_its_interpreter_from_the_link_to_the_accept() {
    let g = Group::founded();
    let a = g.read();
    assert_eq!(a.membership().keeper, Some(ana()));
    assert_eq!(a.membership().office, Office::Held);

    // The joiner's consent covers its device: a legacy consent does not count.
    let invite = g.invite(1, 20, 90, None);
    let legacy = g.redeem_under(1, 20, invite, 9, "Eve", Device::Computer, None, 91);
    let uncounted = Refused::Command(RedeemRefusal::Uncounted);
    assert_eq!(legacy.err(), Some(uncounted));
    assert!(!g.read().membership().standing.contains(&actor_of(&key(9))));

    let g = three();
    let m = g.read().membership().clone();
    let phone = actor_of(&key(4));
    assert!(m
        .standing
        .is_superset(&[ana(), bo(), cy(), phone.clone()].into()));
    assert_eq!(m.device(&phone), Some(Device::Browser));
    assert_eq!(
        m.person_of(&phone).map(|(first, _)| first),
        Some(bo().as_str())
    );

    // Data, from the phone.
    let said = g.by(4, 130, &note("from bo's phone")).unwrap();
    assert!(g.read().membership().counts(&said.lines()[0].id));

    // A cut: the keeper removes Cy; Cy's lines after it count nowhere.
    assert!(g
        .by(
            1,
            140,
            &Removal {
                person: p("Cy"),
                joined: None
            }
        )
        .is_ok());
    let m = g.read().membership().clone();
    assert!(!m.standing.contains(&cy()));
    assert!(m.cut.get(&cy()).is_some_and(|c| !c.left));
    assert!(matches!(
        g.by(3, 141, &note("after the cut")),
        Err(Refused::NoStanding)
    ));

    // A hand-on waits on its recipient: nobody keeps the group meanwhile.
    assert!(g
        .by(
            1,
            150,
            &HandOnTo {
                person: p("Bo"),
                joined: None
            }
        )
        .is_ok());
    let m = g.read().membership().clone();
    assert_eq!(
        m.keeper, None,
        "a waiting hand-on never falls back to anyone"
    );
    assert!(matches!(&m.office, Office::Pending { by, to, .. } if *by == ana() && *to == bo()));
    // The holder takes it back: a hand-on to its own key.
    assert!(g.by(1, 151, &Take(ana())).is_ok());
    assert_eq!(g.read().membership().keeper, Some(ana()));
    // Handed on again, and Bo's own yes passes it.
    assert!(g
        .by(
            1,
            152,
            &HandOnTo {
                person: p("Bo"),
                joined: None
            }
        )
        .is_ok());
    assert!(matches!(
        g.by(3, 153, &Take(cy())),
        Err(Refused::NoStanding)
    ));
    assert!(g.by(2, 154, &Take(bo())).is_ok());
    assert_eq!(g.read().membership().keeper, Some(bo()));
    assert!(matches!(
        g.by(1, 155, &Take(ana())),
        Err(Refused::Command(HandOnRefusal::NotOffered))
    ));
}

#[test]
fn a_holders_fork_freezes_the_office_while_data_and_exits_still_count() {
    let g = three();
    let tip = g.tip(1);
    let meets = |relay: &str| RailAct::Meets {
        relays: [String::from(relay)].into(),
    };
    g.beside(1, &tip, 200, meets("https://one.example/"));
    g.beside(1, &tip, 200, meets("https://two.example/"));
    let m = g.read().membership().clone();
    assert_eq!(m.office, Office::Frozen);
    assert_eq!(m.keeper, None, "a frozen office names no keeper");
    // Office powers wait; data and a person's own exit still count.
    let cut = g.by(
        1,
        210,
        &Removal {
            person: p("Cy"),
            joined: None,
        },
    );
    assert!(cut.is_err(), "{cut:?}");
    let said = g.by(2, 211, &note("still counts")).unwrap();
    assert!(g.read().membership().counts(&said.lines()[0].id));
    assert!(g.by(3, 212, &Leave(cy())).is_ok());
    let m = g.read().membership().clone();
    assert!(!m.standing.contains(&cy()));
    assert!(m.cut.get(&cy()).is_some_and(|c| c.left));
}

#[test]
fn a_hand_on_nobody_accepts_is_recovered_and_a_late_accept_beside_it_freezes() {
    let g = three();
    let hand = g
        .by(
            1,
            300,
            &HandOnTo {
                person: p("Bo"),
                joined: None,
            },
        )
        .unwrap();
    let hand_on = hand.lines()[0].id.clone();
    let dispute = OfficeDispute::Unaccepted {
        source: AuthorityRef::Genesis,
        hand_on: hand_on.clone(),
    };
    // Ana's computer is gone; Bo never accepts. Cy proposes to keep it.
    let choice = RecoveryChoice {
        dispute,
        target: cy(),
        departed: BTreeSet::new(),
        snapshot: g.snapshot(),
    };
    let proposal = g.by(
        3,
        310,
        &authority(AuthorityChange::RecoveryProposal { choice }),
    );
    let proposal = proposal.unwrap().lines()[0].id.clone();
    let vote = |k: u8, now| {
        let voted = g.by(
            k,
            now,
            &authority(AuthorityChange::RecoveryVote {
                proposal: proposal.clone(),
            }),
        );
        (actor_of(&key(k)), voted.unwrap().lines()[0].id.clone())
    };
    let votes = [vote(2, 311), vote(3, 312)].into();
    let commit = AuthorityChange::RecoveryCommit {
        proposal: proposal.clone(),
        votes,
        snapshot: g.snapshot(),
    };
    let recovered = g.by(3, 320, &authority(commit));
    assert!(recovered.is_ok(), "{recovered:?}");
    let m = g.read().membership().clone();
    assert_eq!(m.keeper, Some(cy()));
    assert_eq!(m.office, Office::Held);

    // Bo's Accept, signed before Bo's computer heard of the recovery: both
    // held, neither is picked, and the office freezes (P6).
    let tip = g.tip(2);
    g.beside(2, &tip, 330, RailAct::Accept { hand_on });
    let m = g.read().membership().clone();
    assert_eq!(m.office, Office::Frozen);
    assert_eq!(m.keeper, None);
}

#[test]
fn a_visit_and_its_hosts_stop_count_in_a_sources_group() {
    let g = three();
    let visit = Visit {
        guest: p("Gil"),
        key: actor_of(&key(40)),
        opens: Default::default(),
        until: 3_000,
    };
    let came = g.by(1, 400, &visit).unwrap().lines()[0].id.clone();
    assert!(g.read().membership().visits.contains_key(&came), "a visit");
    let stop = Fixture {
        act: RailAct::Correct {
            corrects: came.clone(),
            replacement: None,
        },
    };
    assert!(g.by(1, 410, &stop).is_ok(), "the host's Stop counts");
    let m = g.read().membership().clone();
    assert!(m.visits[&came].stopped.is_some(), "and stops it");
}
