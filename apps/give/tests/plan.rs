//! What the keeper program does each round, decided apart from doing it
//! (keep.rs `plan`): pure over the groups' folds and what runs here, so
//! each choice is tested without podman or svrngs.

use std::collections::BTreeMap;
use std::time::Duration;

use give::keep::{EVERY, RETRY, Running, Step, plan};
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
    }
}

const NONE: BTreeMap<String, (String, u64)> = BTreeMap::new();

#[test]
fn a_computer_that_has_not_said_it_can_keep_says_so_and_starts_nothing() {
    let folds = [("g1".to_string(), fold(&[]))];
    assert_eq!(
        plan("a1", &folds, &BTreeMap::new(), &NONE, 100, EVERY),
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
    assert_eq!(plan("a1", &folds, &here, &NONE, 399, every), []);
    assert_eq!(
        plan("a1", &folds, &here, &NONE, 400, every),
        [Step::Snapshot { app: "mine".into() }]
    );
}

#[test]
fn it_stops_what_another_computer_keeps_now() {
    let taken = op(
        "t1",
        "b1",
        json!({ "kind": "taken", "app": "mine", "card": "v1", "over": "s1", "from": "give/mine/a1/1", "agreed": true }),
    );
    let kept = fold(&[ready("r1", "a1"), ready("r2", "b1"), started("s1", "a1", "mine"), taken]);
    let here = BTreeMap::from([("mine".to_string(), running("g1", "s1", 100))]);
    let steps = plan("a1", &[("g1".into(), kept)], &here, &NONE, 101, EVERY);
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
            &failed,
            100 + RETRY.as_secs() - 1,
            EVERY
        ),
        []
    );
    let again = plan("a1", &folds, &BTreeMap::new(), &failed, 100 + RETRY.as_secs(), EVERY);
    assert!(matches!(&again[..], [Step::Start { .. }]), "{again:?}");
}
