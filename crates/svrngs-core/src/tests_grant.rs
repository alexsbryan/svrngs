// SPDX-License-Identifier: AGPL-3.0-or-later
//! A holder's line: what the pass on it lets its signer write as the person
//! who signed the pass (capability-signed; demo/runs/authorship/2026-10-24/
//! DESIGN.md §3, §4). A phone signs its own acts in an app it opened with
//! that app's holder, and any computer carries the line unchanged: it counts
//! as the phone's person, only in that app's own part of the record, only
//! before the pass lapses, and never as a decision.
//!
//! Each test's kept half, a holder's write in its app counting as its
//! person, is red where no fold resolves a holder (an `UnknownSigner`); its
//! refused half turns red under a plant that takes the pass's word for more
//! than it grants.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};

use crate::pass::{Label, Pass};
use crate::tests_support::*;
use crate::*;

/// The holder a phone's lend origin keeps.
const HOLDER: u8 = 9;

/// The carried app `kind`'s lend on alex's computer: its name the kind's and
/// the group's first 16 letters, as `Lend::carried` names it (lend.rs).
fn app(kind: &str) -> Label {
    let lender = key(1).verifying_key().to_bytes();
    Label::of(&lender, &format!("{kind}-{}", NS.get(..16).unwrap_or(NS)))
}

/// `by`'s pass for `HOLDER` in the app `kind`, lapsing at `until`.
fn pass(by: u8, kind: &str, until: i64) -> Pass {
    Pass::sign(
        &key(by),
        &actor_of(&key(HOLDER)),
        (app(kind), vec![]),
        until,
    )
}

fn act(kind: &str, what: &str) -> RailAct {
    let payload = Payload::new(serde_json::json!({ "kind": kind, "what": what })).unwrap();
    RailAct::Record { payload }
}

/// `act`, signed by `holder` at `seq` after `prev`, carrying `pass`.
fn held(
    holder: u8,
    pass: &Pass,
    ts: i64,
    prev: Option<&Op<SignedOp>>,
    act: RailAct,
) -> Op<SignedOp> {
    let seq = prev.map_or(0, |p| p.kind.seq + 1);
    let (prev, sig, grant) = (
        prev.map(|p| p.id.clone()),
        String::new(),
        Some(pass.clone()),
    );
    let kind = SignedOp {
        seq,
        prev,
        sig,
        act,
        on_behalf_of: None,
        view: None,
        grant,
    };
    sign_op(&key(holder), NS, ts, kind)
}

/// Each applied act's `what` and its person.
fn whose(a: &Admission) -> Vec<(String, String)> {
    let what = |o: &AdmittedOp| {
        serde_json::to_value(o.payload.as_ref()?).ok()?["what"]
            .as_str()
            .map(str::to_string)
    };
    let applied = a.ops().iter().filter(|o| o.applies());
    applied
        .filter_map(|o| Some((what(o)?, o.person.to_string())))
        .collect()
}

fn mine(what: &str, person: &str) -> (String, String) {
    (what.into(), person.into())
}

/// **A phone's write counts as its person, whoever carries it.** Bo's phone
/// signs Money's acts with its holder; the line is the same bytes on every
/// computer, so every copy folds it as bo's, its actor still the holder.
#[test]
fn a_phone_s_write_counts_as_its_person_whoever_carries_it() {
    let bo = pass(2, "money", 2_000);
    let dinner = held(HOLDER, &bo, 100, None, act("money/expense", "dinner"));
    let taxi = held(
        HOLDER,
        &bo,
        101,
        Some(&dinner),
        act("money/expense", "taxi"),
    );
    let (one, two) = (
        admitted(&[dinner.clone(), taxi.clone()]),
        admitted(&[taxi, dinner]),
    );
    assert_eq!(
        whose(&one),
        vec![mine("dinner", "bo"), mine("taxi", "bo")],
        "{:?}",
        one.gaps()
    );
    assert_eq!(one.gaps(), &[] as &[RailGap]);
    assert_eq!(
        one.ops()[0].actor,
        actor_of(&key(HOLDER)),
        "the signer stays the line's"
    );
    assert_eq!(
        whose(&one),
        whose(&two),
        "two computers holding the line fold it alike"
    );
}

/// **A holder writes only its own app's part.** Bo's Money holder cannot
/// write Events' acts, nor an act of no app, as bo.
#[test]
fn a_holder_writes_only_its_own_app_s_part() {
    let bo = pass(2, "money", 2_000);
    let dinner = held(HOLDER, &bo, 100, None, act("money/expense", "dinner"));
    let coming = held(
        HOLDER,
        &bo,
        101,
        Some(&dinner),
        act("events/coming", "coming"),
    );
    let bare = held(HOLDER, &bo, 102, Some(&coming), act("money", "bare"));
    let a = admitted(&[dinner, coming, bare]);
    assert_eq!(whose(&a), vec![mine("dinner", "bo")], "{:?}", a.gaps());
}

/// **A holder decides nothing.** No pass grants Decide (DESIGN §4): a
/// holder's `Admit` lets nobody in, and its `Relay` names nothing.
#[test]
fn a_holder_decides_nothing() {
    let bo = pass(2, "money", 2_000);
    let dinner = held(HOLDER, &bo, 100, None, act("money/expense", "dinner"));
    let eve = RailAct::Admit {
        person: p("eve"),
        key: actor_of(&key(7)),
        invite: None,
    };
    let let_in = held(HOLDER, &bo, 101, Some(&dinner), eve);
    let relay = RailAct::Relay {
        url: Some("https://relay.example".into()),
    };
    let says = held(HOLDER, &bo, 102, Some(&let_in), relay);
    let a = admitted(&[dinner, let_in, says]);
    assert_eq!(whose(&a), vec![mine("dinner", "bo")], "{:?}", a.gaps());
    assert!(!standing(&a, 7), "a holder's Admit lets nobody in");
    assert!(
        a.membership().relays.is_empty(),
        "a holder's Relay names nothing"
    );
}

/// **A pass writes nothing once it lapses.**
#[test]
fn a_lapsed_pass_writes_nothing() {
    let bo = pass(2, "money", 101);
    let dinner = held(HOLDER, &bo, 100, None, act("money/expense", "dinner"));
    let late = held(
        HOLDER,
        &bo,
        101,
        Some(&dinner),
        act("money/expense", "late"),
    );
    let a = admitted(&[dinner, late]);
    assert_eq!(whose(&a), vec![mine("dinner", "bo")], "{:?}", a.gaps());
}

/// **A pass is its signer's word, for its holder alone.** A pass whose
/// person was rewritten after signing, one naming another holder, and one
/// signed by a key that stands nowhere each write nothing.
#[test]
fn a_pass_is_its_signer_s_word_for_its_holder_alone() {
    let bo = pass(2, "money", 2_000);
    let dinner = held(HOLDER, &bo, 100, None, act("money/expense", "dinner"));
    let mut renamed = bo.clone();
    renamed.person = actor_of(&key(3));
    let as_cy = held(
        HOLDER,
        &renamed,
        101,
        Some(&dinner),
        act("money/expense", "as-cy"),
    );
    let other = Pass::sign(&key(2), &actor_of(&key(8)), (app("money"), vec![]), 2_000);
    let not_its = held(
        HOLDER,
        &other,
        102,
        Some(&as_cy),
        act("money/expense", "not-its"),
    );
    let stranger = held(
        6,
        &pass(7, "money", 2_000),
        100,
        None,
        act("money/expense", "stranger"),
    );
    let a = admitted(&[dinner, as_cy, not_its, stranger]);
    assert_eq!(whose(&a), vec![mine("dinner", "bo")], "{:?}", a.gaps());
}

/// **A holder corrects only its own app's part.** Bo's Money holder voids a
/// Money act, and leaves alex's Events act counted.
#[test]
fn a_holder_corrects_only_its_own_app_s_part() {
    let bo = pass(2, "money", 2_000);
    let coming = signed(&key(1), 99, 0, act("events/coming", "alex-coming"));
    let dinner = held(HOLDER, &bo, 100, None, act("money/expense", "dinner"));
    let undo = |corrects: &Op<SignedOp>| RailAct::Correct {
        corrects: corrects.id.clone(),
        replacement: None,
    };
    let void_coming = held(HOLDER, &bo, 101, Some(&dinner), undo(&coming));
    let void_dinner = held(HOLDER, &bo, 102, Some(&void_coming), undo(&dinner));
    let taxi = held(
        HOLDER,
        &bo,
        103,
        Some(&void_dinner),
        act("money/expense", "taxi"),
    );
    let a = admitted(&[coming, dinner, void_coming, void_dinner, taxi]);
    assert_eq!(
        whose(&a),
        vec![mine("alex-coming", "alex"), mine("taxi", "bo")],
        "{:?}",
        a.gaps()
    );
}

/// **A cut covers its person's holders.** The keeper's cut of bo names the
/// newest acts it held of bo's key and of bo's holder; the holder's acts it
/// covers count, and one written after it counts nowhere.
#[test]
fn a_cut_covers_its_person_s_holders() {
    let bo = pass(2, "money", 2_000);
    let own = signed(&key(2), 99, 0, act("money/expense", "own"));
    let dinner = held(HOLDER, &bo, 100, None, act("money/expense", "dinner"));
    let through = vec![own.id.clone(), dinner.id.clone()];
    let cut = signed(
        &key(1),
        101,
        0,
        RailAct::Remove {
            key: actor_of(&key(2)),
            through,
        },
    );
    let late = held(
        HOLDER,
        &bo,
        102,
        Some(&dinner),
        act("money/expense", "late"),
    );
    let a = admitted(&[own, dinner, cut, late]);
    assert_eq!(
        whose(&a),
        vec![mine("own", "bo"), mine("dinner", "bo")],
        "{:?}",
        a.gaps()
    );
    assert!(!standing(&a, 2));
}
