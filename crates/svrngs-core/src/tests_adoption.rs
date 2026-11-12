//! An old group crosses to the interpreter by its keeper's Adopt
//! (svrn-docs `rings/current/EUDOXIA_JUDGE.md` §2e), written and read back
//! through the write side and admission as a computer does. Each law of §2e
//! §6 that lives in the core, with the plant named beside it.
use crate::authority::{adoption, AuthorityChange, GroupPolicy, Unlawful};
use crate::seed::Seed;
use crate::tests_support::{key, p, record, Fixture, Memory};
use crate::*;
use alloc::{string::String, vec::Vec};

struct Old {
    roster: Roster,
    ns: String,
    store: Memory,
}

/// Sam (key 1) made it, before groups were founded sources-v1.
fn old() -> Old {
    let name = |n: &str| Name::parse(n).unwrap();
    let seed = Seed::from_parts(name("Pier"), name("Sam"), key(1).verifying_key(), [9; 16]);
    let roster = seed.roster();
    assert_eq!(roster.policy, None);
    Old {
        roster,
        ns: seed.id(),
        store: Memory::holding(&[]),
    }
}

fn who(k: u8) -> String {
    actor_of(&key(k))
}

impl Old {
    fn by<C: Command>(&self, k: u8, now: i64, c: &C) -> Result<Recorded, Refused<C::Refusal>> {
        let author = Author {
            signer: &key(k),
            roster: &self.roster,
            namespace: &self.ns,
            now,
        };
        let Ok(done) = commit(&self.store, c, &author);
        done
    }
    fn read(&self) -> Admission {
        let held = self.store.0.borrow();
        admit(&held, &[], &self.roster, &self.ns, &Ed25519Verifier)
    }
    fn invite(&self, issuer: u8, link: u8, now: i64) -> OpId {
        let mint = MintInvite {
            key: who(link),
            until: now + 3600,
            device_of: None,
        };
        self.by(issuer, now, &mint).unwrap().lines()[0].id.clone()
    }
    /// The Admit `issuer` writes for `joining` under `invite`, its consent
    /// as `policy` asks: the old one, or the device-bound one.
    fn admit(
        &self,
        issuer: u8,
        invite: &OpId,
        joining: u8,
        person: &str,
        device: Device,
    ) -> RailAct {
        self.admit_as(issuer, invite, joining, person, device, None)
    }
    fn admit_as(
        &self,
        _issuer: u8,
        invite: &OpId,
        joining: u8,
        person: &str,
        device: Device,
        policy: Option<GroupPolicy>,
    ) -> RailAct {
        let invite = Some(Redemption::sign(
            &key(joining),
            &self.ns,
            invite,
            &p(person),
            device,
            policy,
        ));
        RailAct::Admit {
            person: p(person),
            key: who(joining),
            invite,
        }
    }
    /// `issuer` lets `joining` in by a link of its own.
    fn join(&self, issuer: u8, joining: u8, person: &str, device: Device, now: i64) {
        let invite = self.invite(issuer, 100 + joining, now);
        let act = self.admit(issuer, &invite, joining, person, device);
        self.by(issuer, now, &Fixture { act }).unwrap();
        assert!(
            self.read().membership().standing.contains(&who(joining)),
            "{person} joined"
        );
    }
    fn ready(&self, k: u8, now: i64) {
        self.by(k, now, &Ready).unwrap();
    }
    fn adoption(&self, k: u8) -> Result<authority::LegacyAdoption, Unlawful> {
        adoption(
            &self.store.0.borrow(),
            &self.roster,
            &self.ns,
            &Ed25519Verifier,
            &who(k),
        )
    }
    /// `k` presses Move: the Adopt its computer draws, written.
    fn adopt(&self, k: u8, now: i64) -> OpId {
        let drawn = self.adoption(k).unwrap();
        let written = self.by(k, now, &Adopt(drawn));
        written.unwrap().lines()[0].id.clone()
    }
    fn tip(&self, k: u8) -> Op<SignedOp> {
        let held = self.store.0.borrow();
        let mine = held.iter().filter(|op| op.actor == who(k));
        mine.max_by_key(|op| op.kind.seq).unwrap().clone()
    }
    /// `k`'s newest line, if it signed any.
    fn tip_or_seed(&self, k: u8) -> Option<OpId> {
        let held = self.store.0.borrow();
        let mine = held.iter().filter(|op| op.actor == who(k));
        mine.max_by_key(|op| op.kind.seq).map(|op| op.id.clone())
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
    fn hand_on(&self, k: u8, to: &str, now: i64) -> Result<Recorded, Refused<HandOnRefusal>> {
        let to = HandOnTo {
            person: p(to),
            joined: None,
        };
        self.by(k, now, &to)
    }
}

/// Sam's group with Ana (key 2) and Bo (key 3) by Sam's links, every
/// standing computer ready.
fn pier() -> Old {
    let g = old();
    g.join(1, 2, "Ana", Device::Computer, 100);
    g.join(1, 3, "Bo", Device::Computer, 110);
    g.by(
        2,
        120,
        &Fixture {
            act: record("ana's note"),
        },
    )
    .unwrap();
    for k in [1, 2, 3] {
        g.ready(k, 200 + i64::from(k));
    }
    g
}

#[test]
fn an_old_group_moves_when_its_keeper_adopts_and_a_hand_on_then_waits() {
    let g = pier();
    let before = g.read().membership().clone();
    assert_eq!(
        before.decided,
        Decided::Legacy,
        "Readies alone move nothing"
    );
    let adopt = g.adopt(1, 300);
    let m = g.read().membership().clone();
    assert_eq!(m.decided, Decided::Adopted(adopt));
    assert_eq!(m.keeper, Some(who(1)));
    assert_eq!(m.standing, before.standing);
    assert_eq!(m.bindings, before.bindings);
    assert_eq!(m.admitted_at, before.admitted_at);
    // After it, a hand-on waits for its recipient, as in a founded group.
    assert!(g.hand_on(1, "Ana", 310).is_ok());
    let m = g.read().membership().clone();
    assert_eq!(m.keeper, None);
    assert!(matches!(&m.office, Office::Pending { to, .. } if *to == who(2)));
    assert!(g.by(2, 320, &Take(who(2))).is_ok());
    assert_eq!(g.read().membership().keeper, Some(who(2)));
}

/// Plant: skip `author == keeper` in `pin`'s checks.
#[test]
fn only_the_old_folds_keeper_adopts() {
    let g = pier();
    // Ana asks Bo to keep it: a hand-on that moves nothing, so Bo held the
    // office's name and is no Adopt dropped unread.
    let tip = g.tip(2);
    g.beside(2, &tip, 250, RailAct::HandOn { key: who(3) });
    assert_eq!(g.read().membership().keeper, Some(who(1)));
    // Every computer says Bo keeps it, by hand: only the fold says who does.
    for k in [1, 2, 3] {
        let tips = g.store.0.borrow().iter().map(|op| op.id.clone()).collect();
        let change = AuthorityChange::Ready {
            tips,
            keeper: who(3),
        };
        let act = RailAct::Authority { change };
        g.by(k, 260 + i64::from(k), &Fixture { act }).unwrap();
    }
    assert_eq!(g.adoption(3), Err(Unlawful::NotKeeper));
}

/// Plants: skip the HandOn-since check; skip the Ready coverage.
#[test]
fn no_ex_keeper_adopts_over_a_truncated_closure() {
    // Bo let Ana in after his Ready; Sam then handed on to Ana. Over heads
    // that stop at Bo's Ready, Ana never stood and Sam still keeps.
    let g = old();
    g.join(1, 3, "Bo", Device::Computer, 100);
    g.ready(1, 110);
    g.ready(3, 111);
    g.join(3, 2, "Ana", Device::Computer, 120);
    assert!(g.hand_on(1, "Ana", 130).is_ok());
    assert_eq!(g.read().membership().keeper, Some(who(2)));
    assert_eq!(g.adoption(1), Err(Unlawful::HandedOn));

    // A standing computer that never said it is ready, or what it holds.
    let g = old();
    g.join(1, 2, "Ana", Device::Computer, 100);
    g.ready(1, 110);
    assert_eq!(g.adoption(1), Err(Unlawful::Ready));
}

/// Plant: skip the old fold's `keeps` test at the Adopt's place.
#[test]
fn a_keeper_who_handed_on_and_got_it_back_does_not_adopt() {
    let g = pier();
    assert!(g.hand_on(1, "Ana", 230).is_ok());
    assert!(g.hand_on(2, "Sam", 231).is_ok());
    assert_eq!(g.read().membership().keeper, Some(who(1)));
    for k in [1, 2, 3] {
        g.ready(k, 240 + i64::from(k));
    }
    assert_eq!(g.adoption(1), Err(Unlawful::HandedOn));
}

/// A computer that sees no keeper says so, never an empty one: here a
/// founded group's hand-on waiting on its Accept.
#[test]
fn a_ready_names_the_keeper_or_is_not_written() {
    let name = |n: &str| Name::parse(n).unwrap();
    let seed = Seed::from_parts(name("Pier"), name("Sam"), key(1).verifying_key(), [9; 16]);
    let seed = seed.with_source_policy();
    let g = Old {
        roster: seed.roster(),
        ns: seed.id(),
        store: Memory::holding(&[]),
    };
    let invite = g.invite(1, 102, 100);
    let ana = g.admit_as(1, &invite, 2, "Ana", Device::Computer, seed.policy());
    g.by(1, 101, &Fixture { act: ana }).unwrap();
    g.ready(1, 102);
    let named = |op: &Op<SignedOp>| op.kind.act.ready().map(|(_, k)| String::from(k));
    assert_eq!(named(&g.tip(1)), Some(who(1)));
    assert!(g.hand_on(1, "Ana", 110).is_ok());
    assert_eq!(g.read().membership().keeper, None);
    assert!(matches!(
        g.by(1, 120, &Ready),
        Err(Refused::Command(Unlawful::NotKeeper))
    ));
}

/// The pin's bytes, as release 2 reads them: domain, group, NUL, bytes.
#[test]
fn the_result_pin_is_its_domain_group_nul_and_bytes() {
    let pin = authority::legacy_result_pin("pier", b"{\"membership\":{}}");
    // b3sum of the bytes `svrngs/legacy-result/v1\0pier\0{"membership":{}}`.
    let golden = "f134151aaa0c62e05b7d4bbb013056cf6fa6601838c62a321146312dde31e55d";
    assert_eq!(pin.to_hex(), golden);
}

/// Plant: skip the Computer check in `pin`.
#[test]
fn the_office_key_stands_and_is_a_computer() {
    let g = old();
    g.join(1, 2, "Ana", Device::Computer, 100);
    let link = g.invite(2, 50, 110);
    let phone = g.admit(2, &link, 5, "Ana", Device::Browser);
    g.by(2, 111, &Fixture { act: phone }).unwrap();
    // Sam hands the old office to Ana's phone, which the old fold allows.
    g.by(
        1,
        120,
        &Fixture {
            act: RailAct::HandOn { key: who(5) },
        },
    )
    .unwrap();
    assert_eq!(g.read().membership().keeper, Some(who(5)));
    for k in [1, 2] {
        g.ready(k, 130 + i64::from(k));
    }
    assert_eq!(g.adoption(5), Err(Unlawful::Computer));
}

/// Plant: skip the decider's own fork check (`RailGap::SequenceFork`).
#[test]
fn a_keeper_fork_before_move_makes_it_unlawful() {
    let g = pier();
    let tip = g.tip(1);
    g.beside(1, &tip, 250, record("one"));
    g.beside(1, &tip, 250, record("two"));
    for k in [1, 2, 3] {
        g.ready(k, 260 + i64::from(k));
    }
    assert_eq!(g.adoption(1), Err(Unlawful::Fork));
}

/// Plant: judge lawfulness over the held set, not the Adopt's closure.
#[test]
fn a_later_fork_or_a_rival_adopt_freezes_and_never_reverts() {
    let g = pier();
    let drawn = g.adoption(1).unwrap();
    let ready = g.tip(1);
    let adopt = g.adopt(1, 300);
    // Sam's key signs beside its own Adopt: an office fork, frozen.
    g.beside(
        1,
        &ready,
        301,
        RailAct::Meets {
            relays: Default::default(),
        },
    );
    let m = g.read().membership().clone();
    assert_eq!(m.decided, Decided::Adopted(adopt.clone()), "never reverts");
    assert_eq!(m.office, Office::Frozen);
    // A rival Adopt beside it: still the one decider, still frozen.
    let change = AuthorityChange::Adopt { adoption: drawn };
    let rival = g.beside(1, &ready, 302, RailAct::Authority { change });
    let m = g.read().membership().clone();
    assert!(m.decided.interpreted());
    assert!(matches!(&m.decided, Decided::Adopted(id) if *id == adopt.clone().min(rival.id)));
    assert_eq!(m.office, Office::Frozen);
}

/// Plant: stop comparing `out` against the old fold.
#[test]
fn the_pin_equals_the_old_fold() {
    let g = pier();
    let cut = Removal {
        person: p("Bo"),
        joined: None,
    };
    g.by(1, 250, &cut).unwrap();
    for k in [1, 2] {
        g.ready(k, 260 + i64::from(k));
    }
    let mut drawn = g.adoption(1).unwrap();
    assert!(drawn.out.contains_key(&who(3)));
    drawn.out.clear();
    assert!(matches!(
        g.by(1, 300, &Adopt(drawn.clone())),
        Err(Refused::Command(Unlawful::Pin))
    ));
    let change = AuthorityChange::Adopt { adoption: drawn };
    let act = RailAct::Authority { change };
    g.by(1, 301, &Fixture { act }).unwrap();
    assert_eq!(
        g.read().membership().decided,
        Decided::Legacy,
        "a wrong pin adopts nothing"
    );
}

/// Plant: re-project the closure (no pinned start, no closure skip).
#[test]
fn lines_inside_keep_their_meaning() {
    let g = pier();
    // Ana's Meets counted in the old fold as a line, and moved nothing.
    g.by(
        2,
        230,
        &Fixture {
            act: RailAct::Meets {
                relays: ["https://ana.example/".into()].into(),
            },
        },
    )
    .unwrap();
    // A second Bo, let in by Ana's own hand at another time.
    let link = g.invite(2, 60, 240);
    let second = g.admit(2, &link, 6, "Bo", Device::Computer);
    g.by(2, 241, &Fixture { act: second }).unwrap();
    let visit = Visit {
        guest: p("Gil"),
        key: who(40),
        opens: Default::default(),
        until: 100_000,
    };
    let came = g.by(1, 250, &visit).unwrap().lines()[0].id.clone();
    for k in [1, 2, 3, 6] {
        g.ready(k, 260 + i64::from(k));
    }
    let before = g.read().membership().clone();
    assert_eq!(before.meets, None);
    g.adopt(1, 300);
    let m = g.read().membership().clone();
    assert!(m.decided.interpreted());
    assert_eq!(m.meets, None, "a member's Meets stays powerless");
    let bos = m.one_named(&p("Bo"), None).unwrap_err();
    assert_eq!(bos.len(), 2);
    assert!(
        bos.iter().all(|j| matches!(j, Joined::At(_))),
        "two Bos, told apart: {bos:?}"
    );
    assert!(
        m.visits.get(&came).is_some_and(|g| g.live(&m.cut, 301)),
        "a live visit stays live"
    );
}

/// Plant: `open_link` reads only lines outside the closure.
#[test]
fn a_link_closed_inside_stays_closed() {
    let g = pier();
    let older = g.invite(1, 70, 230);
    g.invite(1, 71, 231);
    for k in [1, 2, 3] {
        g.ready(k, 240 + i64::from(k));
    }
    g.adopt(1, 300);
    let late = g.admit_as(
        1,
        &older,
        7,
        "Dee",
        Device::Computer,
        Some(GroupPolicy::SourcesV1),
    );
    let _ = g.by(1, 310, &Fixture { act: late });
    assert!(!g.read().membership().standing.contains(&who(7)));
}

/// Plants: drop the closure skip in `held()`; drop the `out` check.
#[test]
fn lines_outside_gain_nothing() {
    let g = pier();
    let cut = Removal {
        person: p("Bo"),
        joined: None,
    };
    g.by(1, 225, &cut).unwrap();
    // Sam's Admit of Dee, timed before the link it redeems: the old fold
    // had not counted that link yet, so Dee never stood.
    let link = g.invite(1, 90, 230);
    let dee = g.admit_as(
        1,
        &link,
        7,
        "Dee",
        Device::Computer,
        Some(GroupPolicy::SourcesV1),
    );
    g.beside(1, &g.tip(1), 150, dee);
    assert!(!g.read().membership().standing.contains(&who(7)));
    for k in [1, 2] {
        g.ready(k, 240 + i64::from(k));
    }
    g.adopt(1, 300);
    let m = g.read().membership().clone();
    assert_eq!((m.keeper, m.office), (Some(who(1)), Office::Held));
    assert!(
        !m.standing.contains(&who(7)),
        "an old line stays as the old fold read it"
    );
    // Bo's computer, cut inside, writes on: none of it counts.
    let tip = g.tip(3);
    let after = g.beside(3, &tip, 310, record("bo after"));
    let m = g.read().membership().clone();
    assert!(!m.counts(&after.id));
    assert!(!m.standing.contains(&who(3)));
}

/// Plant: refuse every old consent (`statement`'s `old` false).
#[test]
fn an_old_consent_admit_under_a_pre_adoption_link_counts() {
    let g = pier();
    let link = g.invite(1, 80, 230);
    for k in [1, 2, 3] {
        g.ready(k, 240 + i64::from(k));
    }
    g.adopt(1, 300);
    let redeemed = g.admit(1, &link, 8, "Dee", Device::Computer);
    assert!(g.by(1, 310, &Fixture { act: redeemed }).is_ok());
    assert!(g.read().membership().standing.contains(&who(8)));
}

#[test]
fn a_copy_missing_closure_lines_writes_no_membership_act() {
    let g = pier();
    let adopt = g.adopt(1, 300);
    // Ana's copy lacks Bo's Ready: it holds the Adopt and waits.
    let held: Vec<Op<SignedOp>> = g.store.0.borrow().iter().cloned().collect();
    let bos = |op: &&Op<SignedOp>| op.actor == who(3) && op.kind.act.ready().is_some();
    let copy = Old {
        roster: g.roster.clone(),
        ns: g.ns.clone(),
        store: Memory::holding(
            &held
                .iter()
                .filter(|op| !bos(op))
                .cloned()
                .collect::<Vec<_>>(),
        ),
    };
    assert_eq!(copy.read().membership().decided, Decided::Waiting(adopt));
    let link = MintInvite {
        key: who(90),
        until: 1_000,
        device_of: None,
    };
    let Err(Refused::Uncounted(waits)) = copy.by(2, 310, &link) else {
        panic!("a link is a membership act, and this copy waits");
    };
    assert!(
        waits.to_string().contains("lacks part of the group"),
        "{waits}"
    );
    assert!(copy
        .by(
            2,
            311,
            &Fixture {
                act: record("data goes on")
            }
        )
        .is_ok());
}

/// Plant: the joiner's consent ignores `Asked::adopted`.
#[test]
fn joining_after_adoption_uses_the_new_consent() {
    let g = pier();
    g.adopt(1, 300);
    let invite = g.invite(1, 81, 310);
    let asked = crate::joining::Asked {
        group: g.ns.clone(),
        seed: String::new(),
        invite: invite.clone(),
        person: None,
        adopted: true,
    };
    let consent = asked.policy(None);
    let joins = g.admit_as(1, &invite, 9, "Eve", Device::Computer, consent);
    assert!(g.by(1, 311, &Fixture { act: joins }).is_ok());
    assert!(g.read().membership().standing.contains(&who(9)));
}

/// A keeper who left while alone is still the old fold's keeper
/// (`membership.rs`, a lone leaver keeps the office), but stands nowhere, so
/// it moves nothing (B3). Plant: drop the standing test in `pin`.
#[test]
fn a_keeper_who_left_does_not_adopt() {
    let g = old();
    let tip = g.tip_or_seed(1);
    let leave = RailAct::Remove {
        key: who(1),
        through: tip.into_iter().collect(),
    };
    g.by(1, 120, &Fixture { act: leave }).unwrap();
    let m = g.read().membership().clone();
    assert!(!m.standing.contains(&who(1)), "Sam left");
    assert_eq!(
        m.keeper,
        Some(who(1)),
        "the old fold keeps a lone leaver as keeper"
    );
    assert_eq!(g.adoption(1).map(|_| ()), Err(Unlawful::NotKeeper));
}
