//! What the keeper program does each round, decided apart from doing it
//! (keep.rs `plan`): pure over the groups' folds and what runs here, so
//! each choice is tested without podman or svrngs.

use std::collections::BTreeMap;
use std::time::Duration;

use give::keep::{EARLY, EVERY, Held, RETRY, Running, Step, plan, resay};
use give_fold::fold;
use serde_json::{Value, json};

fn op(id: &str, actor: &str, payload: Value) -> Value {
    json!({ "id": id, "actor": actor, "person": actor, "person_key": actor, "ts_unix": 1, "voided": false, "payload": payload })
}

fn ready(id: &str, actor: &str) -> Value {
    op(id, actor, json!({ "kind": "ready" }))
}

fn started(id: &str, actor: &str, app: &str) -> Value {
    op(
        id,
        actor,
        json!({ "kind": "started", "app": app, "card": "v1", "agreed": true }),
    )
}

fn running(group: &str, act: &str, snapshot: u64) -> Running {
    Running {
        group: group.into(),
        act: act.into(),
        at: 40000,
        snapshot,
        began: 0,
        from: None,
    }
}

const NONE: BTreeMap<String, (String, u64)> = BTreeMap::new();
const HELD: BTreeMap<String, Held> = BTreeMap::new();

#[test]
fn a_computer_that_has_not_said_it_can_keep_says_so_and_starts_nothing() {
    let folds = [("g1".to_string(), fold(&[]))];
    assert_eq!(
        plan("a1", &folds, &BTreeMap::new(), &HELD, &NONE, 100, EVERY),
        [Step::Ready { group: "g1".into() }]
    );
}

#[test]
fn it_starts_what_the_record_names_it_for_and_nothing_named_for_another() {
    let kept = fold(&[
        ready("r1", "a1"),
        ready("r2", "b1"),
        started("s1", "a1", "mine"),
        started("s2", "b1", "theirs"),
    ]);
    let steps = plan(
        "a1",
        &[("g1".into(), kept.clone())],
        &BTreeMap::new(),
        &HELD,
        &NONE,
        100,
        EVERY,
    );
    assert_eq!(
        steps,
        [Step::Start {
            group: "g1".into(),
            app: "mine".into(),
            keeping: kept.apps["mine"].clone()
        }]
    );
}

#[test]
fn it_copies_what_it_keeps_each_interval_and_not_before() {
    let kept = fold(&[ready("r1", "a1"), started("s1", "a1", "mine")]);
    let folds = [("g1".to_string(), kept)];
    let here = BTreeMap::from([("mine".to_string(), running("g1", "s1", 100))]);
    let every = Duration::from_secs(300);
    assert_eq!(plan("a1", &folds, &here, &HELD, &NONE, 399, every), []);
    assert_eq!(
        plan("a1", &folds, &here, &HELD, &NONE, 400, every),
        [Step::Snapshot { app: "mine".into() }]
    );
}

#[test]
fn it_stops_what_another_computer_keeps_now() {
    let taken = op(
        "t1",
        "b1",
        json!({ "kind": "taken", "app": "mine", "card": "v1", "over": "s1", "from": "give/mine/a1/1", "sum": "5e2", "agreed": true }),
    );
    let kept = fold(&[ready("r1", "a1"), ready("r2", "b1"), started("s1", "a1", "mine"), taken]);
    let here = BTreeMap::from([("mine".to_string(), running("g1", "s1", 100))]);
    let steps = plan("a1", &[("g1".into(), kept)], &here, &HELD, &NONE, 101, EVERY);
    assert!(
        matches!(&steps[..], [Step::Stop { app, .. }] if app == "mine"),
        "{steps:?}"
    );
}

#[test]
fn it_keeps_one_app_of_a_kind_however_many_groups_name_it() {
    let one = fold(&[ready("r1", "a1"), started("s1", "a1", "mine")]);
    let two = fold(&[ready("r9", "a1"), started("s9", "a1", "mine")]);
    let steps = plan(
        "a1",
        &[("g1".into(), one), ("g2".into(), two)],
        &BTreeMap::new(),
        &HELD,
        &NONE,
        100,
        EVERY,
    );
    assert!(
        matches!(&steps[..], [Step::Refuse { group, why, .. }, Step::Start { group: g, .. }]
            if group == "g2" && why.contains("for g1 already") && g == "g1"),
        "{steps:?}"
    );
}

#[test]
fn a_start_that_failed_waits_before_the_same_act_is_tried_again() {
    let kept = fold(&[ready("r1", "a1"), started("s1", "a1", "mine")]);
    let folds = [("g1".to_string(), kept)];
    let failed = BTreeMap::from([("mine".to_string(), ("s1".to_string(), 100))]);
    assert_eq!(
        plan(
            "a1",
            &folds,
            &BTreeMap::new(),
            &HELD,
            &failed,
            100 + RETRY.as_secs() - 1,
            EVERY
        ),
        []
    );
    let again = plan(
        "a1",
        &folds,
        &BTreeMap::new(),
        &HELD,
        &failed,
        100 + RETRY.as_secs(),
        EVERY,
    );
    assert!(matches!(&again[..], [Step::Start { .. }]), "{again:?}");
}

fn offered(id: &str, actor: &str, app: &str, over: &str, to: &str) -> Value {
    op(
        id,
        actor,
        json!({ "kind": "offered", "app": app, "over": over, "to": to }),
    )
}

fn answered(id: &str, actor: &str, kind: &str, app: &str, offer: &str) -> Value {
    op(
        id,
        actor,
        json!({ "kind": kind, "app": app, "offer": offer, "agreed": true }),
    )
}

/// On the yes of the person of the computer it is offered to, the
/// computer that keeps an app moves it there (GIVE.md §3 scene 3).
#[test]
fn on_a_yes_it_moves_what_it_keeps_to_the_computer_offered() {
    let kept = fold(&[
        ready("r1", "a1"),
        ready("r2", "b1"),
        started("s1", "a1", "mine"),
        offered("o1", "a1", "mine", "s1", "b1"),
        answered("y1", "b1", "accepted", "mine", "o1"),
    ]);
    let here = BTreeMap::from([("mine".to_string(), running("g1", "s1", 100))]);
    let steps = plan("a1", &[("g1".into(), kept.clone())], &here, &HELD, &NONE, 101, EVERY);
    assert_eq!(
        steps,
        [Step::Move {
            app: "mine".into(),
            keeping: kept.apps["mine"].clone()
        }]
    );
    // The computer offered starts nothing until the move is written.
    assert_eq!(
        plan("b1", &[("g1".into(), kept)], &BTreeMap::new(), &HELD, &NONE, 101, EVERY),
        []
    );
}

/// An offer waiting, or said no to, moves nothing: the app stays kept
/// where it is, and is copied each interval as before.
#[test]
fn an_offer_waiting_or_said_no_to_moves_nothing() {
    let waiting = [
        ready("r1", "a1"),
        ready("r2", "b1"),
        started("s1", "a1", "mine"),
        offered("o1", "a1", "mine", "s1", "b1"),
    ];
    let mut no = waiting.to_vec();
    no.push(answered("n1", "b1", "declined", "mine", "o1"));
    let here = BTreeMap::from([("mine".to_string(), running("g1", "s1", 100))]);
    for ops in [waiting.to_vec(), no] {
        let folds = [("g1".to_string(), fold(&ops))];
        assert_eq!(plan("a1", &folds, &here, &HELD, &NONE, 101, EVERY), []);
        assert_eq!(
            plan("a1", &folds, &here, &HELD, &NONE, 100 + EVERY.as_secs(), EVERY),
            [Step::Snapshot { app: "mine".into() }]
        );
    }
}

/// A move that failed here waits before it is tried again, as a start does.
#[test]
fn a_move_that_failed_waits_before_it_is_tried_again() {
    let kept = fold(&[
        ready("r1", "a1"),
        ready("r2", "b1"),
        started("s1", "a1", "mine"),
        offered("o1", "a1", "mine", "s1", "b1"),
        answered("y1", "b1", "accepted", "mine", "o1"),
    ]);
    let folds = [("g1".to_string(), kept)];
    let here = BTreeMap::from([("mine".to_string(), running("g1", "s1", 100))]);
    let failed = BTreeMap::from([("mine".to_string(), ("o1".to_string(), 100))]);
    assert_eq!(
        plan("a1", &folds, &here, &HELD, &failed, 100 + RETRY.as_secs() - 1, EVERY),
        []
    );
    let again = plan("a1", &folds, &here, &HELD, &failed, 100 + RETRY.as_secs(), EVERY);
    assert!(matches!(&again[..], [Step::Move { .. }]), "{again:?}");
}

fn taken(id: &str, actor: &str, over: &str, from: &str) -> Value {
    op(
        id,
        actor,
        json!({ "kind": "taken", "app": "mine", "card": "v1", "over": over, "from": from, "sum": "5e2", "agreed": true }),
    )
}

fn held(group: &str, act: &str, copied: bool) -> BTreeMap<String, Held> {
    let h = Held {
        group: group.into(),
        act: act.into(),
        copied,
    };
    BTreeMap::from([("mine".to_string(), h)])
}

/// A copy that did not start here is not tried again on its own: the
/// record says so, and a person chooses the copy before it (GIVE.md §3
/// scene 4). Running, it is stopped with no copy of its own.
#[test]
fn a_copy_that_did_not_start_is_not_tried_again() {
    let kept = fold(&[
        ready("r1", "a1"),
        ready("r2", "b1"),
        started("s1", "a1", "mine"),
        taken("t1", "b1", "s1", "give/mine/a1/1"),
        op("f1", "b1", json!({ "kind": "failed", "app": "mine", "over": "t1" })),
    ]);
    let folds = [("g1".to_string(), kept)];
    let steps = plan("b1", &folds, &BTreeMap::new(), &HELD, &NONE, 1000, EVERY);
    assert!(
        matches!(&steps[..], [Step::Refuse { why, .. }] if why.contains("did not start")),
        "{steps:?}"
    );
    let here = BTreeMap::from([("mine".to_string(), running("g1", "t1", 100))]);
    assert_eq!(
        plan("b1", &folds, &here, &HELD, &NONE, 101, EVERY),
        [Step::Stop {
            app: "mine".into(),
            why: "the copy it started from did not start, so the group chooses another".into(),
            copy: false
        }]
    );
}

/// For a minute after its start, an app is checked each round: one that
/// exits then did not start.
#[test]
fn an_app_is_watched_for_a_minute_after_its_start() {
    let kept = fold(&[ready("r1", "a1"), started("s1", "a1", "mine")]);
    let folds = [("g1".to_string(), kept)];
    let mut r = running("g1", "s1", 100);
    r.began = 100;
    let here = BTreeMap::from([("mine".to_string(), r)]);
    assert_eq!(
        plan("a1", &folds, &here, &HELD, &NONE, 100 + EARLY.as_secs() - 1, EVERY),
        [Step::Check { app: "mine".into() }]
    );
    assert_eq!(
        plan("a1", &folds, &here, &HELD, &NONE, 100 + EARLY.as_secs(), EVERY),
        []
    );
}

/// Back after another computer took the app over, this computer copies
/// the state it held, as it had it, and once only (GIVE.md §3 scene 5).
/// State a clean stop copied already is not copied again.
#[test]
fn back_after_a_takeover_it_copies_the_state_it_had() {
    let kept = fold(&[
        ready("r1", "a1"),
        ready("r2", "b1"),
        started("s1", "a1", "mine"),
        taken("t1", "b1", "s1", "give/mine/a1/1"),
    ]);
    let folds = [("g1".to_string(), kept)];
    assert_eq!(
        plan(
            "a1",
            &folds,
            &BTreeMap::new(),
            &held("g1", "s1", false),
            &NONE,
            1000,
            EVERY
        ),
        [Step::Had {
            group: "g1".into(),
            app: "mine".into(),
            act: "s1".into()
        }]
    );
    assert_eq!(
        plan(
            "a1",
            &folds,
            &BTreeMap::new(),
            &held("g1", "s1", true),
            &NONE,
            1000,
            EVERY
        ),
        []
    );
    // Still named for it: a restart, from the state it holds.
    let mine = fold(&[ready("r1", "a1"), started("s1", "a1", "mine")]);
    let steps = plan(
        "a1",
        &[("g1".into(), mine)],
        &BTreeMap::new(),
        &held("g1", "s1", false),
        &NONE,
        1000,
        EVERY,
    );
    assert!(matches!(&steps[..], [Step::Start { .. }]), "{steps:?}");
}

/// Taking over from a copy, the state held here from an older keeping is
/// copied first, as this computer had it, then replaced.
#[test]
fn what_it_held_is_copied_before_a_takeover_replaces_it() {
    let kept = fold(&[
        ready("r1", "a1"),
        ready("r2", "b1"),
        started("s1", "a1", "mine"),
        taken("t1", "b1", "s1", "give/mine/a1/1"),
        taken("t2", "a1", "t1", "give/mine/b1/2"),
    ]);
    let steps = plan(
        "a1",
        &[("g1".into(), kept)],
        &BTreeMap::new(),
        &held("g1", "s1", false),
        &NONE,
        1000,
        EVERY,
    );
    assert!(
        matches!(&steps[..], [Step::Had { act, .. }, Step::Start { keeping, .. }] if act == "s1" && keeping.act == "t2"),
        "{steps:?}"
    );
}

/// Two takeovers at once: the later's computer, having started it, stops
/// it and keeps its state as it had it, the first standing.
#[test]
fn the_later_of_two_takeovers_stops_with_a_copy_of_what_it_had() {
    let kept = fold(&[
        ready("r1", "a1"),
        ready("r2", "b1"),
        ready("r3", "c1"),
        started("s1", "a1", "mine"),
        taken("t1", "c1", "s1", "give/mine/a1/1"),
        taken("t2", "b1", "s1", "give/mine/a1/1"),
    ]);
    let here = BTreeMap::from([("mine".to_string(), running("g1", "t2", 100))]);
    let steps = plan("b1", &[("g1".into(), kept)], &here, &HELD, &NONE, 1000, EVERY);
    assert!(matches!(&steps[..], [Step::Stop { copy: true, .. }]), "{steps:?}");
}

/// What a computer can't run here is said again only when it changes
/// (give-install (c)): its latest `ready` is what the page reads, an older
/// one with nothing said being nothing it can't run. Planted, watched
/// failing: the latest `ready` read as the first.
#[test]
fn what_a_computer_cannot_run_is_said_again_only_when_it_changes() {
    let ready = |actor: &str, cannot: Option<Value>| {
        let mut payload = json!({ "kind": "ready" });
        if let Some(c) = cannot {
            payload["cannot"] = c;
        }
        json!({ "actor": actor, "payload": payload })
    };
    let off = json!({ "factorio": "rosetta" });
    let acts = [ready("a1", None), ready("b1", Some(off.clone()))];
    assert!(!resay(&acts, "a1", &json!({})), "nothing said is nothing it can't run");
    assert!(resay(&acts, "a1", &off));
    assert!(!resay(&acts, "b1", &off));
    let later = [ready("b1", Some(off.clone())), ready("b1", Some(json!({})))];
    assert!(resay(&later, "b1", &off), "its latest said it can run it");
    assert!(!resay(&later, "b1", &json!({})));
}
