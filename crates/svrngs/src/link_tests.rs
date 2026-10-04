// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-join-vocab/src/deep_link.rs@e1a43bcd2
use super::*;

#[test]
fn percent_decode_passes_through_invalid_escapes() {
    // A lone `%` with no two hex digits shouldn't panic or drop
    // characters — we'd rather hand back a partial value than
    // reject the whole link. Guard for accidental truncation.
    assert_eq!(percent_decode("abc%"), "abc%");
    assert_eq!(percent_decode("abc%ZZ"), "abc%ZZ");
}

#[test]
fn percent_encode_round_trips_dial_string() {
    // The dial string carries every reserved char that breaks a
    // naive query value: `@ : / , .`. Encode → decode must be
    // identity, or the joiner dials a corrupt founder address.
    let dial = "aabbccddeeff00112233@https://relay.example./,10.0.0.5:9742,[fd7a::1]:9742";
    assert_eq!(percent_decode(&percent_encode(dial)), dial);
}

fn a_link(name: &str, origin: Origin) -> Link {
    let founder = SigningKey::from_bytes(&[1u8; 32]);
    let seed = Seed::from_parts(
        crate::group::Name::parse(name).unwrap(),
        crate::group::Name::parse("Sam").unwrap(),
        founder.verifying_key(),
        [9u8; 16],
    );
    let issuer =
        svrngs_node::iroh::PublicKey::from_bytes(&founder.verifying_key().to_bytes()).unwrap();
    let addr = EndpointAddr::new(issuer)
        .with_ip_addr("127.0.0.1:4242".parse().unwrap())
        .with_relay_url("https://relay.example./".parse().unwrap());
    Link {
        seed,
        secret: SigningKey::from_bytes(&[7u8; 32]),
        invite: OpId::from_hex(&"ab".repeat(32)).unwrap(),
        issuer: Dial::of(&addr).unwrap(),
        origin,
    }
}

fn same(a: &Link, b: &Link) {
    assert_eq!(a.seed, b.seed);
    assert_eq!(a.seed.id(), b.seed.id());
    assert_eq!(a.secret.to_bytes(), b.secret.to_bytes());
    assert_eq!(a.invite, b.invite);
    assert_eq!(a.issuer, b.issuer);
}

/// A link round-trips through build and parse: a group name with a
/// space, an apostrophe and an ampersand, the seed and its id, the
/// secret, and the dial with its relay and address. The secret is only
/// in the fragment.
#[test]
fn a_link_round_trips_through_build_and_parse() {
    let link = a_link("Sam's house & yard", Origin::default());
    let url = build_https_link(&link);
    let (before, fragment) = url.split_once('#').unwrap();
    assert_eq!(before, DEFAULT_ORIGIN);
    let secret = hex::encode(link.secret.to_bytes());
    assert!(
        !before.contains(&secret) && fragment.contains(&secret),
        "{url}"
    );
    let parsed = parse_https_link(&url).expect("the link parses");
    same(&parsed, &link);
    assert_eq!(parsed.origin, link.origin);
}

/// The parser never reads left of the `#`: the secret moved into the
/// query, the part a browser sends to the server, is no link at all;
/// and every field moved there, with no fragment, is none either.
#[test]
fn a_link_whose_secret_sits_left_of_the_hash_parses_as_nothing() {
    let link = a_link("house", Origin::default());
    let url = build_https_link(&link);
    let (base, fragment) = url.split_once('#').unwrap();
    let secret = format!("secret={}", hex::encode(link.secret.to_bytes()));
    let rest: Vec<&str> = fragment.split('&').filter(|p| *p != secret).collect();
    let moved = format!("{base}?{secret}#{}", rest.join("&"));
    assert!(parse_https_link(&moved).is_none(), "{moved}");
    assert!(parse_https_link(&format!("{base}?{fragment}")).is_none());
}

/// A link printed on an origin of the founder's own parses to the same
/// link as one on the default, and keeps the origin it was printed on.
#[test]
fn a_link_printed_on_another_origin_parses_the_same() {
    let theirs = Origin::parse("https://links.example.org/our/house/").unwrap();
    let link = a_link("house", theirs.clone());
    let url = build_https_link(&link);
    assert!(
        url.starts_with("https://links.example.org/our/house/#"),
        "{url}"
    );
    let parsed = parse_https_link(&url).expect("a link on another origin parses");
    same(&parsed, &link);
    assert_eq!(parsed.origin, theirs);
    let on_default = a_link("house", Origin::default());
    same(
        &parsed,
        &parse_https_link(&build_https_link(&on_default)).unwrap(),
    );
}

#[test]
fn an_origin_is_http_with_no_fragment() {
    assert!(Origin::parse("https://svrnme.sh/ring/").is_some());
    assert!(Origin::parse("svrnme.sh/ring/").is_none());
    assert!(Origin::parse("https://h/#x").is_none());
}
