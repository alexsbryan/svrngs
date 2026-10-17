// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::sig::{
    verify_ring_op, ASK_DOMAIN, LABEL_DOMAIN, PASS_DOMAIN, REDEMPTION_DOMAIN, REQUEST_DOMAIN,
    RING_OP_DOMAIN,
};
use crate::tests_support::key;
use crate::{act_id, actor_of, sign_ring_op};

fn app(name: &str) -> App {
    App::parse(name).unwrap()
}

/// What an app's name may be: the closed form, nothing else, `:` least.
#[test]
fn an_apps_name_is_parsed_once_in_a_closed_form() {
    for ok in ["notes", "a", "x-2", &"a".repeat(64)] {
        assert_eq!(App::parse(ok).map(|a| a.to_string()), Ok(ok.to_string()));
    }
    for not in [
        "",
        "Notes",
        "no tes",
        "no:tes",
        "nötes",
        "no/tes",
        &"a".repeat(65),
    ] {
        assert_eq!(App::parse(not), Err(NotAnApp), "{not:?}");
    }
}

/// A signature an app was given checks for that app, those bytes and that
/// key only — `notes` over `sx` is not `note` over `ssx`, which signs the
/// same bytes without the `:` that ends an app's name.
#[test]
fn an_apps_signature_checks_for_that_app_and_those_bytes_only() {
    let (k, notes) = (key(1), app("notes"));
    let sig = notes.sign(&k, b"sx");
    assert!(notes.check(&actor_of(&k), b"sx", &sig));
    assert!(!notes.check(&actor_of(&k), b"sy", &sig), "other bytes");
    assert!(!notes.check(&actor_of(&key(2)), b"sx", &sig), "another key");
    assert!(
        !app("other").check(&actor_of(&k), b"sx", &sig),
        "another app"
    );
    assert!(
        !app("note").check(&actor_of(&k), b"ssx", &sig),
        "another app, the same bytes run on"
    );
}

/// An app's signature fails as an act's, a link's request or ask, a
/// redemption's, a pass's or a label's, though the app is named after the
/// domain and its bytes are the rest of a message under it: without its own
/// domain, `svrngs-pass` signing `rest` would sign `svrngs-pass:` || rest.
#[test]
fn an_apps_signature_is_no_act_link_pass_or_label() {
    let k = key(1);
    let id = act_id("house", &actor_of(&k), 1, None, 100, r#"{"op":"record"}"#);
    let as_act = app("svrngs-act-id-binding").sign(&k, id.as_bytes());
    assert!(!verify_ring_op(&actor_of(&k), &id, &as_act), "an act's");
    for domain in [
        RING_OP_DOMAIN,
        REDEMPTION_DOMAIN,
        REQUEST_DOMAIN,
        ASK_DOMAIN,
        PASS_DOMAIN,
        LABEL_DOMAIN,
    ] {
        let name = core::str::from_utf8(&domain[..domain.len() - 1]).unwrap();
        let sig = app(name).sign(&k, b"rest of a message");
        let under = [domain, b"rest of a message"].concat();
        assert!(!verify_hex(&actor_of(&k), &under, &sig), "{name}'s");
        assert!(!app(name).message(b"rest").starts_with(domain), "{name}'s");
    }
}

/// An act's signature fails as an app's, for the app named after its
/// domain over its id: what an act signs is never an app's message.
#[test]
fn an_acts_signature_is_no_apps() {
    let k = key(1);
    let id = act_id("house", &actor_of(&k), 1, None, 100, r#"{"op":"record"}"#);
    let sig = sign_ring_op(&k, &id);
    assert!(verify_ring_op(&actor_of(&k), &id, &sig));
    assert!(!app("svrngs-act-id-binding").check(&actor_of(&k), id.as_bytes(), &sig));
}

const KEY_1: &str = "8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c";
const SIG_812: &str = "6296480dc80a73a3f3872dac92fb777835e837829b20c7f818ab7ea922abdf401452e404d04f7a39cf613161bdb80a89ba55bfc43700b0a678f63b5c4110440f";

/// The vector a checker in another language reproduces, frozen from this
/// implementation: key(1) signs `line 812` for `notes`. `apps/on-air/
/// laws.test.mjs` checks the same three with the page's own code.
#[test]
fn a_frozen_vector_for_a_stranger() {
    let (k, notes) = (key(1), app("notes"));
    assert_eq!(notes.message(b"line 812"), b"svrngs-app:notes:line 812");
    assert_eq!(actor_of(&k), KEY_1);
    assert_eq!(notes.sign(&k, b"line 812"), SIG_812);
    assert!(notes.check(KEY_1, b"line 812", SIG_812));
}
