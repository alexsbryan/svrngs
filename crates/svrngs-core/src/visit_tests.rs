// SPDX-License-Identifier: AGPL-3.0-or-later
//! A guest, for a visit (guests-visit; svrn-docs `rings/current/GUESTS.md`
//! §3, §5): a `Visit` the host's computer signs, folded beside membership,
//! so every copy agrees who is visiting as it agrees who is a member.
use super::*;
use crate::tests_support::{key, p, run, NS};
use crate::{actor_of, admit, Admission, Ed25519Verifier, Roster};

/// Friday evening, and the hour.
const FRI: i64 = 1_000_000;
const HOUR: i64 = 3_600;
/// Sunday evening: 48 hours on.
const SUNDAY: i64 = FRI + 48 * HOUR;

/// A house of three, as a seed of three keys: alex keeps it, kim and bo.
fn house() -> Roster {
    let mut m = BTreeMap::new();
    m.insert(p("alex"), vec![actor_of(&key(1))]);
    m.insert(p("kim"), vec![actor_of(&key(2))]);
    m.insert(p("bo"), vec![actor_of(&key(3))]);
    Roster::new(m)
}

fn folded(ops: &[Op<SignedOp>]) -> Admission {
    admit(&crate::ActSet::of(ops), &[], &house(), NS, &Ed25519Verifier)
}

/// `host` has `guest`, holding the fresh key `fresh`, over until `until`,
/// opening the host's own Jellyfin.
fn visit(host: u8, guest: &str, fresh: u8, until: i64) -> RailAct {
    let mut opens = BTreeMap::new();
    opens.insert(actor_of(&key(host)), ["jellyfin".into()].into());
    RailAct::Visit(crate::Visit {
        guest: p(guest),
        key: actor_of(&key(fresh)),
        opens,
        until,
        consent: crate::tests_support::consent(&key(fresh), &actor_of(&key(host))),
    })
}

/// Who the fold says is visiting at `at`: each guest's name, their host's
/// key and until when.
fn visiting(a: &Admission, at: i64) -> Vec<(String, String, i64)> {
    let m = a.membership();
    (m.visits.values().filter(|g| g.live(&m.cut, at)))
        .map(|g| (g.visit.guest.to_string(), g.host.clone(), g.visit.until))
        .collect()
}

/// **A guest, asked** (part (a)): Kim hosts Sam until Sunday, and the fold
/// says Sam is visiting, Kim's guest, until Sunday. Sam's key never
/// stands. Red at HEAD, since the record has no such act.
#[test]
fn kim_hosts_sam_until_sunday_and_the_fold_says_so() {
    let ops = run(&key(2), FRI, [visit(2, "sam", 9, SUNDAY)]);
    let a = folded(&ops);
    assert_eq!(
        visiting(&a, FRI + HOUR),
        [("sam".into(), actor_of(&key(2)), SUNDAY)],
        "Sam is visiting, Kim's guest, until Sunday: {:?}",
        a.gaps()
    );
    assert!(
        !a.membership().standing.contains(&actor_of(&key(9))),
        "a guest's key never stands"
    );
}

/// **A visit begins when its act is written** (guests-first, #214): Kim's
/// visit is timed as far past Kim's clock as svrngs times a new act, one
/// past a line [`crate::command::AHEAD_SECS`] ahead, and the fold says Sam
/// is visiting at Kim's clock; a visit timed one further is not begun, so
/// no clock far ahead lengthens a visit past its 48 hours. Planted: the
/// visit begun only at its own time (red at Kim's clock); begun at any time
/// (red one further).
#[test]
fn a_visit_is_on_from_when_its_act_is_written_and_no_earlier() {
    let ahead = crate::command::AHEAD_SECS + 1;
    let ops = run(&key(2), FRI + ahead, [visit(2, "sam", 9, SUNDAY)]);
    let a = folded(&ops);
    assert_eq!(
        visiting(&a, FRI).len(),
        1,
        "on at Kim's clock: {:?}",
        a.gaps()
    );
    assert!(
        visiting(&a, FRI - 1).is_empty(),
        "begun before it was written"
    );
}

/// The names of who is visiting at `at`, sorted: the fold keeps visits by
/// their act's id.
fn names(a: &Admission, at: i64) -> Vec<String> {
    let mut names: Vec<String> = visiting(a, at)
        .into_iter()
        .map(|(name, _, _)| name)
        .collect();
    names.sort();
    names
}

/// **Two a computer, and no cap on the group**: with three members, Kim's
/// Sam, Bo's Tia and Alex's Uma are all guests, since each computer has its
/// own two slots (the cap of guests fewer than members is gone); Kim's
/// third at once is no guest, and once Kim stops Sam, Kim's next is. Plant:
/// the fold counting no guests.
#[test]
fn two_guests_a_computer_and_no_cap_on_the_group() {
    let mut ops = run(&key(2), FRI, [visit(2, "sam", 9, SUNDAY)]);
    ops.extend(run(&key(3), FRI + 1, [visit(3, "tia", 10, SUNDAY)]));
    ops.extend(run(&key(1), FRI + 2, [visit(1, "uma", 11, SUNDAY)]));
    let a = folded(&ops);
    assert_eq!(
        names(&a, FRI + HOUR),
        ["sam", "tia", "uma"],
        "a guest of each of three members"
    );

    let kims = run(
        &key(2),
        FRI,
        [visit(2, "sam", 9, SUNDAY), visit(2, "tia", 10, SUNDAY)],
    );
    let mut third = kims.clone();
    third.push(crate::tests_support::after(
        &key(2),
        &kims[1],
        FRI + 2,
        visit(2, "uma", 11, SUNDAY),
    ));
    assert_eq!(
        names(&folded(&third), FRI + HOUR),
        ["sam", "tia"],
        "a host's third guest at once is none"
    );

    let stop = RailAct::Correct {
        corrects: kims[0].id.clone(),
        replacement: None,
    };
    let mut after_stop = kims.clone();
    after_stop.push(crate::tests_support::after(
        &key(2),
        &kims[1],
        FRI + 2,
        stop,
    ));
    let s = after_stop.last().unwrap().clone();
    after_stop.push(crate::tests_support::after(
        &key(2),
        &s,
        FRI + 3,
        visit(2, "uma", 11, SUNDAY),
    ));
    assert_eq!(
        names(&folded(&after_stop), FRI + HOUR),
        ["tia", "uma"],
        "a Stop frees its place"
    );
}

/// **A visit ends on time** (GUESTS §3.6; §7 bar 6's first half): a visit
/// of 48 hours is one, and one of 49 hours, or the 72 a page might offer,
/// is no visit, wherever it was written, since the fold reads the bound
/// and no page does. Plant: `until` unchecked.
#[test]
fn a_visit_past_48_hours_is_none() {
    for (hours, is) in [(48, true), (49, false), (72, false)] {
        let a = folded(&run(&key(2), FRI, [visit(2, "sam", 9, FRI + hours * HOUR)]));
        assert_eq!(
            !names(&a, FRI + HOUR).is_empty(),
            is,
            "a visit of {hours} hours"
        );
    }
    let a = folded(&run(&key(2), FRI, [visit(2, "sam", 9, SUNDAY)]));
    assert!(
        names(&a, SUNDAY).is_empty(),
        "at Sunday's end, nobody is visiting"
    );
}

/// **The host's leave, or their Stop, ends the visit** (GUESTS §3.1, §5;
/// §7 bar 4's core half): Kim leaves an hour after Sam arrives, and Sam is
/// visiting before and not after; Kim's Correct of the visit stops it, and
/// Bo's ends nothing. Plants: a host's leave leaving the visit live; the
/// Stop unread.
#[test]
fn the_hosts_leave_or_stop_ends_the_visit() {
    let came = run(&key(2), FRI, [visit(2, "sam", 9, SUNDAY)]);
    let mut left = came.clone();
    let leave = RailAct::Remove {
        key: actor_of(&key(2)),
        through: vec![came[0].id.clone()],
    };
    left.push(crate::tests_support::after(
        &key(2),
        &came[0],
        FRI + HOUR,
        leave,
    ));
    let a = folded(&left);
    assert!(
        a.membership().cut.contains_key(&actor_of(&key(2))),
        "Kim left: {:?}",
        a.gaps()
    );
    assert_eq!(names(&a, FRI + 1), ["sam"]);
    assert!(
        names(&a, FRI + 2 * HOUR).is_empty(),
        "Kim's leave ended Sam's visit"
    );

    let stop = RailAct::Correct {
        corrects: came[0].id.clone(),
        replacement: None,
    };
    let mut bos = came.clone();
    bos.extend(run(&key(3), FRI + 1, [stop.clone()]));
    assert_eq!(
        names(&folded(&bos), FRI + HOUR),
        ["sam"],
        "Bo's Correct stops nobody's guest"
    );
    let mut stopped = came.clone();
    stopped.push(crate::tests_support::after(
        &key(2),
        &came[0],
        FRI + 1,
        stop,
    ));
    let a = folded(&stopped);
    assert!(names(&a, FRI + HOUR).is_empty(), "Kim's Stop ended it");
    assert!(
        a.membership().counts(&came[0].id),
        "and the visit stays in the record, seen"
    );
}

/// **A visit's key is fresh, and never stands** (GUESTS §3.3, §3.8,
/// §3.9): a visit naming Bo's key, or the key of an earlier visit, is no
/// visit; and no visit binds its key. Plant: the key's freshness unread.
#[test]
fn a_visits_key_is_fresh_and_never_stands() {
    let a = folded(&run(&key(2), FRI, [visit(2, "bo", 3, SUNDAY)]));
    assert!(
        names(&a, FRI + 1).is_empty(),
        "a member's key is never a guest's"
    );
    let ops = run(
        &key(2),
        FRI,
        [visit(2, "sam", 9, FRI + 2), visit(2, "sam", 9, SUNDAY)],
    );
    let a = folded(&ops);
    assert!(
        names(&a, FRI + HOUR).is_empty(),
        "a key that visited before is not fresh"
    );
    assert!(!a.membership().standing.contains(&actor_of(&key(9))));
}

/// **A copy of a guest's key takes nothing**: Kim has Tia over; Bo writes a
/// visit of his own with Tia's key and Kim's consent copied. Bo's is no
/// visit, since Tia's key consented to Kim alone, and Tia stays Kim's
/// guest. Plant: the consent's message naming no host
/// (`visit_consent_message`), so Bo's copy verifies.
#[test]
fn another_member_s_copy_of_a_guest_s_key_takes_nothing() {
    let kims = run(&key(2), FRI, [visit(2, "tia", 10, SUNDAY)]);
    let RailAct::Visit(tia) = &kims[0].kind.act else {
        unreachable!("a visit")
    };
    let mut ops = kims.clone();
    ops.extend(run(&key(3), FRI + 1, [RailAct::Visit(tia.clone())]));
    let a = folded(&ops);
    assert_eq!(names(&a, FRI + HOUR), ["tia"], "Tia stays Kim's guest");
    let m = a.membership();
    assert!(
        m.visits.values().all(|g| g.host == actor_of(&key(2))),
        "Kim's alone"
    );
    assert!(!m.counts(&ops[1].id), "Bo's copy counts nowhere");
}
