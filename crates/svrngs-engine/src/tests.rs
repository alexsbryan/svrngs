use serde_json::{Value, json};

use crate::{Breach, Engine, EngineError, Limits, QuickJs, Segment};

/// A counter app: the smallest module with every export.
const COUNTER: &str = r#"
export function init() { return { n: 0, seen: [] }; }
export function reduce(state, act) { state.n += act.add; state.seen.push(act.id); return state; }
export function propose(state, input) { return { op: { add: input.add } }; }
export function say(state, view) { return { text: `${view.who} sees ${state.n}` }; }
export function due(state, now) { return now > 100 ? [{ id: 'late', n: state.n }] : []; }
"#;

fn engine(src: &str) -> QuickJs {
    QuickJs::new(src, Limits::E0).unwrap()
}

/// `src` with `reduce` replaced.
fn with_reduce(body: &str) -> String {
    format!("export function init() {{ return {{ n: 0 }}; }}\nexport function reduce(state, act) {{ {body} }}")
}

fn fold(src: &str, acts: &[Value]) -> Segment {
    engine(src).fold_segment(None, acts).unwrap()
}

#[test]
fn every_export_runs_on_json() {
    let e = engine(COUNTER);
    let acts = [json!({"id": "a", "add": 2}), json!({"id": "b", "add": 3})];
    let Segment::Complete(state) = e.fold_segment(None, &acts).unwrap() else {
        panic!()
    };
    assert_eq!(state, json!({"n": 5, "seen": ["a", "b"]}));
    let Segment::Complete(more) = e.fold_segment(Some(&state), &[json!({"id": "c", "add": 1})]).unwrap() else {
        panic!()
    };
    assert_eq!(more["n"], 6, "a seed is folded onto, not replaced by init");
    assert_eq!(
        e.propose(&state, &json!({"add": 7})).unwrap(),
        json!({"op": {"add": 7}})
    );
    assert_eq!(
        e.say(&state, &json!({"who": "Ama"})).unwrap(),
        json!({"text": "Ama sees 5"})
    );
    assert_eq!(e.due(&state, 101).unwrap(), json!([{"id": "late", "n": 5}]));
    assert_eq!(e.due(&state, 99).unwrap(), json!([]));
}

#[test]
fn what_lets_copies_disagree_is_not_there() {
    let probe = with_reduce(
        "return { date: typeof Date, intl: typeof Intl, weak: typeof WeakRef, fin: typeof FinalizationRegistry, \
         random: typeof Math.random, locale: typeof ''.localeCompare, upper: typeof ''.toLocaleUpperCase, \
         num: typeof (1).toLocaleString, arr: typeof [].toLocaleString, typed: typeof Uint8Array };",
    );
    let Segment::Complete(state) = fold(&probe, &[json!({})]) else {
        panic!()
    };
    for (name, kind) in state.as_object().unwrap() {
        assert_eq!(kind, "undefined", "{name} is still reachable");
    }
    let read_the_clock = fold(&with_reduce("state.at = Date.now(); return state;"), &[json!({})]);
    assert!(
        matches!(&read_the_clock, Segment::Incomplete(Breach::Threw(m)) if m.contains("Date is not defined")),
        "{read_the_clock:?}"
    );
}

#[test]
fn a_module_global_does_not_outlive_its_call() {
    let src = "let calls = 0;\nexport function init() { return { calls: 0 }; }\n\
               export function reduce(state, act) { calls += 1; state.calls = calls; return state; }";
    let e = engine(src);
    let once = e.fold_segment(None, &[json!({})]).unwrap();
    assert_eq!(e.fold_segment(None, &[json!({})]).unwrap(), once);
}

#[test]
fn a_state_that_is_not_exactly_json_is_a_breach() {
    for (body, why) in [
        ("state.s = new Set(); return state;", "the state.s is a Set"),
        ("state.z = -0; return state;", "the state.z is -0"),
        ("state.x = 0 / 0; return state;", "the state.x is NaN"),
        ("state.x = 1 / 0; return state;", "the state.x is Infinity"),
        ("state.u = undefined; return state;", "the state.u is undefined"),
        ("state.f = () => 1; return state;", "the state.f is function"),
        ("state.h = [1, , 3]; return state;", "the state.h[1] is a hole"),
        ("return undefined;", "the state is undefined"),
    ] {
        assert_eq!(
            fold(&with_reduce(body), &[json!({})]),
            Segment::Incomplete(Breach::NotJson(why.into())),
            "{body}"
        );
    }
    let Segment::Complete(state) = fold(
        &with_reduce("state.r = 0.5; state.t = 'x'; return state;"),
        &[json!({})],
    ) else {
        panic!()
    };
    assert_eq!(state, json!({"n": 0, "r": 0.5, "t": "x"}));
}

#[test]
fn each_budget_halts_the_fold() {
    assert_eq!(
        fold(&with_reduce("for (;;) {}"), &[json!({})]),
        Segment::Incomplete(Breach::Fuel)
    );
    assert_eq!(
        fold(
            &with_reduce("try { for (;;) {} } catch (e) { return state; }"),
            &[json!({})]
        ),
        Segment::Incomplete(Breach::Fuel),
        "a reducer cannot catch running out of fuel",
    );
    assert_eq!(
        fold(
            &with_reduce("const deep = (n) => deep(n + 1) + 1; return deep(0);"),
            &[json!({})]
        ),
        Segment::Incomplete(Breach::Stack),
    );
    assert_eq!(
        fold(
            &with_reduce("const a = []; for (;;) a.push('x'.repeat(1 << 16)); "),
            &[json!({})]
        ),
        Segment::Incomplete(Breach::Memory),
    );
    assert_eq!(
        fold(
            &with_reduce("try { const a = []; for (;;) a.push('x'.repeat(1 << 16)); } catch (e) { return state; }"),
            &[json!({})]
        ),
        Segment::Incomplete(Breach::Memory),
        "a reducer cannot catch running out of memory",
    );
}

#[test]
fn a_bundle_that_does_not_load_is_refused_at_load() {
    assert!(matches!(
        QuickJs::new("export function (", Limits::E0),
        Err(EngineError::Load(_))
    ));
    assert!(matches!(
        QuickJs::new("import x from 'node:fs';", Limits::E0),
        Err(EngineError::Load(_))
    ));
    let no_say = engine("export function init() { return {}; }\nexport function reduce(s) { return s; }");
    assert_eq!(no_say.say(&json!({}), &json!({})), Err(EngineError::Missing("say")));
    let no_reduce = engine("export function init() { return {}; }");
    assert_eq!(no_reduce.fold_segment(None, &[]), Err(EngineError::Missing("reduce")));
}

#[test]
fn a_handler_that_throws_says_which_call() {
    let e = engine("export function propose() { throw new Error('no amount'); }");
    let err = e.propose(&json!({}), &json!({})).unwrap_err();
    assert!(
        matches!(&err, EngineError::Breach { call: "propose", breach: Breach::Threw(m) } if m.contains("no amount")),
        "{err:?}"
    );
}
