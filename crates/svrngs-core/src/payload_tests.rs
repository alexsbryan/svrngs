// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/payload.rs@e1a43bcd2
use super::*;

fn json(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

/// **The property the type exists for.** Two spellings of one act must
/// serialize to identical bytes, or two nodes sign different messages for
/// the same fact.
#[test]
fn key_order_does_not_survive_construction() {
    let a = Payload::new(json(r#"{"b":1,"a":2}"#)).unwrap();
    let b = Payload::new(json(r#"{"a":2,"b":1}"#)).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
    assert_eq!(serde_json::to_string(&a).unwrap(), r#"{"a":2,"b":1}"#);
}

#[test]
fn nesting_is_canonical_all_the_way_down() {
    let p = Payload::new(json(r#"{"z":{"y":1,"x":[{"n":1,"m":2}]}}"#)).unwrap();
    assert_eq!(
        serde_json::to_string(&p).unwrap(),
        r#"{"z":{"x":[{"m":2,"n":1}],"y":1}}"#
    );
}

/// Array order is data, not spelling — reordering it would silently
/// change what the app said.
#[test]
fn array_order_is_left_alone() {
    let p = Payload::new(json(r#"{"who":["cy","alex","bo"]}"#)).unwrap();
    assert_eq!(
        serde_json::to_string(&p).unwrap(),
        r#"{"who":["cy","alex","bo"]}"#
    );
}

/// A round trip through the wire must be a fixed point, or a journal line
/// re-read does not verify against the signature it was written with.
#[test]
fn a_payload_read_back_is_the_payload_that_was_written() {
    let p = Payload::new(json(r#"{"b":1,"a":{"d":4,"c":3}}"#)).unwrap();
    let wire = serde_json::to_string(&p).unwrap();
    let back: Payload = serde_json::from_str(&wire).unwrap();
    assert_eq!(back, p);
    assert_eq!(serde_json::to_string(&back).unwrap(), wire);
}

#[test]
fn a_fraction_is_refused_and_the_refusal_says_what_to_write_instead() {
    let e = Payload::new(json(r#"{"amount":3.5}"#)).unwrap_err();
    assert_eq!(e, PayloadError::Fractional("3.5".into()));
    assert!(e.to_string().contains("whole number"), "{e}");
    // Nested, too — a check that only looked at the top level would let
    // this through.
    assert!(matches!(
        Payload::new(json(r#"{"a":{"b":[1,2.5]}}"#)).unwrap_err(),
        PayloadError::Fractional(_)
    ));
}

#[test]
fn whole_numbers_including_negative_and_large_are_fine() {
    assert!(Payload::new(json(r#"{"a":-6000,"b":9007199254740991}"#)).is_ok());
}

#[test]
fn a_bare_scalar_is_not_an_act() {
    assert_eq!(
        Payload::new(json("42")).unwrap_err(),
        PayloadError::NotAnObject
    );
    assert_eq!(
        Payload::new(json("[1,2]")).unwrap_err(),
        PayloadError::NotAnObject
    );
}

#[test]
fn a_payload_larger_than_the_cap_is_refused() {
    let big = "x".repeat(MAX_PAYLOAD_BYTES);
    let v = serde_json::json!({ "note": big });
    assert!(matches!(
        Payload::new(v).unwrap_err(),
        PayloadError::TooLarge(_)
    ));
}

/// The rail has no opinion about meaning. A payload that is nonsense to
/// the app is still a well-formed payload here.
#[test]
fn the_rail_does_not_judge_what_a_payload_says() {
    assert!(Payload::new(json(r#"{"kind":"expense","amount_cents":-1}"#)).is_ok());
}
