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

/// Where the script goes, lend.js's cases (`web/lend.js:338-340`): after the
/// first `<head…>`, else `<html…>`, else `<!doctype…>`, else at the start;
/// and nothing decided while a `<head…>` may yet come.
#[test]
fn the_script_goes_where_lend_js_puts_its_shim() {
    let head = "<!DOCTYPE html><html lang=\"en\"><HEAD data-x=\"1\"><script defer src=\"a.js\"></script></head>";
    for (page, all, at) in [
        (head, false, head.find("<script").map(|i| i as isize)),
        ("<html><body>x", true, Some(6)),
        ("<!doctype html><p>x", true, Some(15)),
        ("<p>x</p>", true, Some(0)),
        ("", true, Some(0)),
        ("<!doctype html><html><he", false, None),
        ("<!doctype html><html><head", false, None),
    ] {
        let at = at.map(|a| a as usize);
        assert_eq!(first_in(page.as_bytes(), all), at, "{page:?}");
    }
}

/// The script carries the policy's nonce when it names one, as lend.js's
/// shim does, and the door says when a policy will not run it.
#[test]
fn the_script_carries_the_policy_s_nonce_and_says_when_it_will_not_run() {
    let canary = "default-src 'self'; script-src 'nonce-Ab+9/=' 'strict-dynamic'";
    for (policy, want, runs) in [
        ("", "<script>S</script>", true),
        (canary, "<script nonce=\"Ab+9/=\">S</script>", true),
        ("img-src *", "<script>S</script>", true),
        ("default-src 'self'", "<script>S</script>", false),
        (
            "Default-Src 'self'; Script-Src 'self' 'Unsafe-Inline'",
            "<script>S</script>",
            true,
        ),
        (
            "script-src 'self'; default-src 'unsafe-inline'",
            "<script>S</script>",
            false,
        ),
    ] {
        let (got, ran) = tag("S", policy);
        assert_eq!(
            (String::from_utf8(got).unwrap().as_str(), ran),
            (want, runs),
            "{policy:?}"
        );
    }
}

/// A page's script is put first while the page streams, a `<head>` cut
/// across chunks included, and the rest is swapped as before.
#[tokio::test]
async fn a_streamed_page_gets_its_script_first_and_its_secrets_swapped() {
    let page = format!("<!doctype html><html><head><script>own('{SECRET}')</script></head>");
    let script = b"<script>mine()</script>".to_vec();
    let want = page
        .replace("<head>", "<head><script>mine()</script>")
        .replace(SECRET, STAND_IN);
    for cut in 0..=page.len() {
        let chunks = [&page[..cut], &page[cut..]]
            .map(|c| Ok::<_, std::convert::Infallible>(Frame::data(Bytes::from(c.to_string()))));
        let inner = http_body_util::StreamBody::new(n0_future::stream::iter(chunks));
        let secret = Answer::parse(
            serde_json::json!({"secret": SECRET, "headers": {}, "script": null})
                .to_string()
                .as_bytes(),
        )
        .unwrap()
        .secret;
        let out = Swapping::new(inner, vec![secret], Some(script.clone()))
            .collect()
            .await
            .unwrap()
            .to_bytes();
        assert_eq!(String::from_utf8_lossy(&out), want, "cut at {cut}");
    }
}
