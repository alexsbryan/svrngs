// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-join-vocab/src/deep_link.rs@e1a43bcd2
use super::*;

fn issuer() -> PublicKey {
    let key = SigningKey::from_bytes(&[1u8; 32]);
    PublicKey::from_bytes(&key.verifying_key().to_bytes()).unwrap()
}

fn a_token(relay: Option<&str>) -> Token {
    Token {
        issuer: issuer(),
        secret: [7u8; SECRET_LEN],
        relay: relay.map(|r| r.parse().unwrap()),
    }
}

/// A token round-trips through each of its three forms, with and without
/// a relay, and the https form keeps its origin.
#[test]
fn a_link_round_trips_in_every_form() {
    let theirs = Origin::parse("https://links.example.org/our/house/").unwrap();
    for token in [a_token(None), a_token(Some("https://relay.example./"))] {
        let client = client_link(&token);
        assert!(client.starts_with(CLIENT_PREFIX), "{client}");
        let https = https_link(&theirs, &token);
        assert!(
            https.starts_with("https://links.example.org/our/house/#"),
            "{https}"
        );
        for (raw, origin) in [
            (client.as_str(), None),
            (https.as_str(), Some(theirs.clone())),
            (&token.encode(), None),
        ] {
            let link = Link::parse(raw).unwrap_or_else(|| panic!("{raw} parses"));
            assert_eq!(link.token, token, "{raw}");
            assert_eq!(link.origin, origin, "{raw}");
        }
    }
}

/// The link's signing key is derived from the secret alone, by a named
/// context: one secret, one key; another secret, another.
#[test]
fn the_links_key_is_derived_from_its_secret() {
    let a = a_token(None);
    assert_eq!(a.key().to_bytes(), a_token(None).key().to_bytes());
    let b = Token {
        secret: [8u8; SECRET_LEN],
        ..a_token(None)
    };
    assert_ne!(a.key().to_bytes(), b.key().to_bytes());
    assert_ne!(a.key().to_bytes()[..SECRET_LEN], a.secret);
}

/// A token with an unknown format byte, or cut short, or with a byte too
/// many, or a relayed one with no relay, parses as nothing.
#[test]
fn a_token_with_an_unknown_format_or_cut_short_parses_as_nothing() {
    let bytes = |t: &Token| URL_SAFE_NO_PAD.decode(t.encode()).unwrap();
    let text = |b: &[u8]| URL_SAFE_NO_PAD.encode(b);
    let bare = bytes(&a_token(None));
    let relayed = bytes(&a_token(Some("https://relay.example./")));
    assert!(Token::decode(&text(&bare)).is_some());
    for format in [0u8, 3, 0xFF] {
        let mut b = bare.clone();
        b[0] = format;
        assert!(Token::decode(&text(&b)).is_none(), "format byte {format}");
    }
    for cut in [0, 1, 33, bare.len() - 1] {
        assert!(Token::decode(&text(&bare[..cut])).is_none(), "cut at {cut}");
    }
    assert!(Token::decode(&text(&[bare.as_slice(), &[0]].concat())).is_none());
    assert!(Token::decode(&text(&relayed[..bare.len()])).is_none());
    let mut r = relayed.clone();
    r[0] = Format::Bare.byte();
    assert!(
        Token::decode(&text(&r)).is_none(),
        "a bare token with a tail"
    );
    let client = client_link(&a_token(None));
    assert!(Link::parse(&client[..client.len() - 4]).is_none());
    assert!(Link::parse("svrngs://join/").is_none());
    assert!(Link::parse("not a link").is_none());
}

/// A client's link with no relay is short: at most 90 characters.
#[test]
fn a_clients_link_with_no_relay_is_at_most_90_characters() {
    let link = client_link(&mint(issuer(), None));
    eprintln!("a client's link, {} characters: {link}", link.len());
    assert!(link.len() <= 90, "{} characters: {link}", link.len());
}

/// A minted token spells no word DESIGN §1 keeps from people, in any case,
/// draw after draw.
#[test]
fn a_minted_token_spells_no_kept_word() {
    for _ in 0..10_000 {
        let token = mint(issuer(), None);
        assert!(!token.spells_a_kept_word(), "{}", token.encode());
    }
}

#[test]
fn an_origin_is_http_with_no_fragment() {
    assert!(Origin::parse("https://svrnme.sh/ring/").is_some());
    assert!(Origin::parse("svrnme.sh/ring/").is_none());
    assert!(Origin::parse("https://h/#x").is_none());
}
