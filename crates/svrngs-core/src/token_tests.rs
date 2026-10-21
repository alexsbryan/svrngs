// SPDX-License-Identifier: AGPL-3.0-or-later
//! The token's claims are exactly the door's headers (SIGN_IN_WITH_SVRNGS
//! §5; §8 1, 4), and a guest is a guest (§8 6).
use super::*;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;

/// The person's key as the record spells an actor: 64 lowercase hex.
const SUB: &str = "9f2c0a1b3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8";
const ISSUER: &str = "http://127.0.0.1:51234/oidc/jellyfin";
const CLIENT: &str = "svrngs-jellyfin";

fn claims(group: Group) -> Claims {
    Claims::new(
        ISSUER,
        SUB,
        CLIENT,
        "Ana",
        "Ana-9f2c0a1b3d4e5f60",
        group,
        1_767_000_000,
        600,
    )
}

fn decoded(part: &str) -> serde_json::Value {
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(part).unwrap()).unwrap()
}

/// Given a person's key and name, their account, the lender's issuer, a
/// client and a time, when the claims make a token, then the header names
/// ES256 and the key, and the claims are the door's headers and the token's
/// own fields — no email, and group membership as sent.
#[test]
fn the_claims_are_what_the_door_already_sends() {
    let token = Token::new(claims(Group::Member), "a-kid");
    let input = token.signing_input();
    let (header, payload) = input.split_once('.').expect("header.payload");

    let header = decoded(header);
    assert_eq!(header["alg"], "ES256");
    assert_eq!(header["typ"], "JWT");
    assert_eq!(header["kid"], "a-kid");

    let payload = decoded(payload);
    assert_eq!(payload["iss"], ISSUER, "one issuer string, exact");
    assert_eq!(payload["sub"], SUB, "sub is the person's key");
    assert_eq!(payload["aud"], CLIENT);
    assert_eq!(payload["iat"], 1_767_000_000);
    assert_eq!(payload["exp"], 1_767_000_600);
    assert_eq!(payload["name"], "Ana");
    assert_eq!(payload["preferred_username"], "Ana-9f2c0a1b3d4e5f60");
    assert_eq!(payload["groups"], serde_json::json!(["member"]));
    assert!(payload.get("email").is_none(), "svrngs has no email");

    let mut keys: Vec<&str> = payload.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "aud", "exp", "groups", "iat", "iss", "name", "preferred_username", "sub"
        ],
        "the claims are exactly what the door already sends, and nothing it does not"
    );
}

/// A guest's token names the guest, never their host.
#[test]
fn a_guest_is_a_guest() {
    let token = Token::new(claims(Group::Guest), "a-kid");
    let (_, payload) = token.signing_input().split_once('.').unwrap();
    let payload = decoded(payload);
    assert_eq!(payload["groups"], serde_json::json!(["guest"]));
    assert_eq!(payload["sub"], SUB);
}

/// The signature is the third base64url part of the input it signed.
#[test]
fn the_signature_completes_the_signing_input() {
    let token = Token::new(claims(Group::Member), "a-kid");
    let input = token.signing_input();
    let signature = [7u8; 64];
    let signed = token.signed(&signature);
    let (head, got) = signed.rsplit_once('.').unwrap();
    assert_eq!(head, input);
    assert_eq!(URL_SAFE_NO_PAD.decode(got).unwrap(), signature);
}
