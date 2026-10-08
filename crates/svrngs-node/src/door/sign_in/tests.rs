// SPDX-License-Identifier: AGPL-3.0-or-later
//! A sign-in's answer read, and the swap, alone; through the door in
//! `tests/door/sign_in.rs`.
use super::*;

const SECRET: &str = "a-person-s-secret-0123";

fn answer(json: serde_json::Value) -> Result<Answer, Refusal> {
    Answer::parse(json.to_string().as_bytes())
}

/// Every field required, none other; a secret of its form; no header the
/// door sets or a hop's; a script holding neither the secret nor its end.
#[test]
fn an_answer_is_read_whole_or_refused() {
    let ok = answer(serde_json::json!({
        "secret": SECRET, "headers": {"authorization": "Bearer x"}, "script": null,
    }))
    .unwrap();
    assert_eq!(ok.headers.len(), 1);
    assert!(ok.script.is_none());
    for (bad, why) in [
        (
            serde_json::json!({"secret": SECRET, "headers": {}}),
            "script missing",
        ),
        (
            serde_json::json!({"secret": SECRET, "headers": {}, "script": null, "x": 1}),
            "unknown field",
        ),
        (
            serde_json::json!({"secret": "short", "headers": {}, "script": null}),
            "short secret",
        ),
        (
            serde_json::json!({"secret": "has space in it ok", "headers": {}, "script": null}),
            "a space",
        ),
        (
            serde_json::json!({"secret": SECRET, "headers": {"X-Svrngs-Key": "k"}, "script": null}),
            "the door's",
        ),
        (
            serde_json::json!({"secret": SECRET, "headers": {"connection": "close"}, "script": null}),
            "a hop's",
        ),
        (
            serde_json::json!({"secret": SECRET, "headers": {"Host": "h"}, "script": null}),
            "host",
        ),
        (
            serde_json::json!({"secret": SECRET, "headers": {}, "script": format!("t='{SECRET}'")}),
            "script holds it",
        ),
        (
            serde_json::json!({"secret": SECRET, "headers": {}, "script": "</SCRIPT>"}),
            "script's end",
        ),
    ] {
        assert!(answer(bad).is_err(), "{why}");
    }
    assert!(Answer::parse(b"not json").is_err());
}

/// A secret split across any two chunks is swapped whole, and nothing of it
/// is let out before its end arrives. Planted: no holding back.
#[test]
fn a_secret_split_anywhere_is_swapped() {
    let body = format!("{{\"url\":\"/x?k={SECRET}\",\"again\":\"{SECRET}\"}}");
    let want = body.replace(SECRET, STAND_IN);
    for cut in 0..=body.len() {
        let (mut held, mut out) = (Vec::new(), Vec::new());
        for (chunk, end) in [(&body[..cut], false), (&body[cut..], true)] {
            held.extend_from_slice(chunk.as_bytes());
            let (o, used) = swapped(&held, &[SECRET.as_bytes()], STAND_IN.as_bytes(), end);
            held.drain(..used);
            out.extend(o);
        }
        assert_eq!(String::from_utf8(out).unwrap(), want, "cut at {cut}");
    }
}
