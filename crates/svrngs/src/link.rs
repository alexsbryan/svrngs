// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-join-vocab/src/deep_link.rs@e1a43bcd2
//! A group's link (DESIGN §8, U2's inventory, the deep_link.rs row; §10,
//! 2026-10-04): upstream's https guest link, everything it carries in its
//! fragment, and a parser that never reads left of the `#`. Its edits: the
//! fragment carries the group's seed, the link's secret key, the id of the
//! `Invite` that names its key and its issuer's dial, in place of
//! upstream's bearer token, lapse, summary,
//! marks and door route; the base is [`Origin`], by default
//! [`DEFAULT_ORIGIN`], and a link printed on any origin parses the same,
//! since everything a computer needs is in the fragment. A computer never
//! fetches the origin: it dials the issuer by key. A phone (U4) loads the
//! page there, which never sees the fragment, so the origin can be anyone's.
//!
//! The lapse is not in the link: the issuer's `Invite` says it, and the
//! issuer's node is the one that decides a redemption (DESIGN §6).

use svrngs_node::iroh::{format_dial_string, parse_dial_string, EndpointAddr};
use svrngs_node::{actor_of, OpId, SigningKey};

use crate::group::Seed;

/// Where a link points when its founder names no origin of their own
/// (operator, 2026-10-04; upstream pins it at
/// `sovereign/crates/sovereign-contracts/src/guest_pages.rs:270`).
pub const DEFAULT_ORIGIN: &str = "https://svrnme.sh/ring/";

/// A link's base: an http(s) address with no fragment of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin(String);

impl Origin {
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let ok = (raw.starts_with("https://") || raw.starts_with("http://")) && !raw.contains('#');
        if !ok {
            tracing::debug!(target: "link", origin = raw, "link: not an origin — not http(s), or carrying a '#'");
        }
        ok.then(|| Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Origin {
    fn default() -> Self {
        Self(DEFAULT_ORIGIN.to_string())
    }
}

/// How the issuer's computer is dialled: an iroh dial string
/// (`<hex-key>@<relay-or-addr>[,…]`, `svrngs_node::iroh::format_dial_string`)
/// with at least one target, held parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dial {
    addr: EndpointAddr,
    text: String,
}

impl Dial {
    /// An endpoint's dial, or `None` while it has no address yet.
    pub fn of(addr: &EndpointAddr) -> Option<Self> {
        let text = format_dial_string(addr)?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Option<Self> {
        match parse_dial_string(text) {
            Ok(addr) => Some(Self {
                addr,
                text: text.trim().to_string(),
            }),
            Err(why) => {
                tracing::debug!(target: "link", dial = text, why, "link: not a dial string");
                None
            }
        }
    }

    pub fn addr(&self) -> &EndpointAddr {
        &self.addr
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// A parsed link.
#[derive(Debug, Clone)]
pub struct Link {
    /// The group it admits to; the group's id is the seed's hash.
    pub seed: Seed,
    /// The link's secret half. Its public half is the key the issuer's
    /// `Invite` names; it is in no record.
    pub secret: SigningKey,
    /// The id of the issuer's `Invite` that names the link's key: what a
    /// joiner's consent signs (`Redemption::sign`), so it rides the link.
    pub invite: OpId,
    /// The issuer's computer: the one that redeems this link.
    pub issuer: Dial,
}

impl Link {
    /// The link's public key, as the issuer's `Invite` names it.
    pub fn key(&self) -> String {
        actor_of(&self.secret)
    }
}

/// Build the https form of a link:
/// `<origin>#name=…&by=…&founder=…&salt=…&secret=…&invite=…&iroh=…`.
///
/// Every field rides the URL FRAGMENT, never the path or query: a browser
/// does not send the fragment to the server, so the secret stays out of
/// access logs and `Referer` headers, and only the page's own script reads
/// it. `origin` is kept verbatim — it is the page's address.
pub fn build_https_link(link: &Link, origin: &Origin) -> String {
    let seed = &link.seed;
    let params = [
        format!("name={}", percent_encode(seed.name().as_str())),
        format!("by={}", percent_encode(seed.founder().as_str())),
        format!("founder={}", seed.founder_actor()),
        format!("salt={}", seed.salt_hex()),
        format!("secret={}", hex::encode(link.secret.to_bytes())),
        format!("invite={}", link.invite),
        format!("iroh={}", percent_encode(link.issuer.as_str())),
    ];
    format!("{}#{}", origin.as_str(), params.join("&"))
}

/// Parse the https form built by [`build_https_link`], from any origin.
/// Every field is required; one rule covers them all — anywhere but the
/// fragment is not read, because the parser never looks left of the `#`.
pub fn parse_https_link(link: &str) -> Option<Link> {
    let Some((base, fragment)) = link.trim().split_once('#') else {
        tracing::debug!(target: "link", "link: refused — no fragment");
        return None;
    };
    Origin::parse(base)?;
    let params = parse_query_params(Some(fragment));
    let field = |name: &str| {
        let value = params.get(name).map(|v| v.trim()).filter(|v| !v.is_empty());
        if value.is_none() {
            tracing::debug!(target: "link", field = name, "link: refused — a field is missing from the fragment");
        }
        value
    };
    let seed = Seed::parse(
        field("name")?,
        field("by")?,
        field("founder")?,
        field("salt")?,
    );
    let secret: Option<[u8; 32]> = hex::decode(field("secret")?)
        .ok()
        .and_then(|b| b.try_into().ok());
    let invite = OpId::from_hex(field("invite")?);
    let issuer = Dial::parse(field("iroh")?);
    match (seed, secret, invite, issuer) {
        (Some(seed), Some(secret), Some(invite), Some(issuer)) => {
            tracing::debug!(target: "link", group = %seed.id(), "link: parsed");
            Some(Link {
                seed,
                secret: SigningKey::from_bytes(&secret),
                invite,
                issuer,
            })
        }
        _ => {
            tracing::debug!(target: "link", "link: refused — the seed, the secret, the invite or the dial is malformed");
            None
        }
    }
}

fn parse_query_params(query: Option<&str>) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                map.insert(percent_decode(key), percent_decode(value));
            }
        }
    }
    map
}

/// Best-effort percent-decoder for query values.
///
/// Browsers and most clipboard-capable chat clients round-trip URLs
/// through `URL`/`encodeURIComponent`, which percent-escapes reserved
/// characters the builder didn't bother to encode — most importantly
/// the `:` in `relay=100.64.0.2:9742` (becomes `%3A`) and `'` in
/// mesh names (becomes `%27`). Without this decode the parser treats
/// `100.64.0.2%3A9742` as a DNS hostname, the join handshake
/// fails, and the user sees a "no peer at that address" error that
/// looks like a networking problem but is actually an encoding one.
///
/// Scope: handles `%XX` sequences and translates `+` to space (the
/// traditional HTML form-encoding convention — existing code already
/// does this for the `name` param). Invalid escapes pass through as
/// literals rather than erroring, because our input is user-pasted
/// and a partial-success link is better than a hard reject.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push(((h << 4) | l) as u8);
                    i += 3;
                    continue;
                }
                out.push(b'%');
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    // Decoded bytes are only valid UTF-8 if the original encoder
    // produced valid UTF-8 — true for anything `encodeURIComponent`
    // emits. On malformed input, fall back to lossy conversion so we
    // still return something the caller can pattern-match.
    String::from_utf8(out).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

/// Percent-encode a query value — the inverse of [`percent_decode`].
/// Encodes everything outside the unreserved set (`ALPHA DIGIT - . _ ~`),
/// so the iroh dial string's `@ , : /` are escaped on the way out and
/// `parse_query_params` decodes them back verbatim.
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
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

    fn a_link(name: &str) -> Link {
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
        let link = a_link("Sam's house & yard");
        let url = build_https_link(&link, &Origin::default());
        let (before, fragment) = url.split_once('#').unwrap();
        assert_eq!(before, DEFAULT_ORIGIN);
        let secret = hex::encode(link.secret.to_bytes());
        assert!(
            !before.contains(&secret) && fragment.contains(&secret),
            "{url}"
        );
        same(&parse_https_link(&url).expect("the link parses"), &link);
    }

    /// The parser never reads left of the `#`: the secret moved into the
    /// query, the part a browser sends to the server, is no link at all;
    /// and every field moved there, with no fragment, is none either.
    #[test]
    fn a_link_whose_secret_sits_left_of_the_hash_parses_as_nothing() {
        let link = a_link("house");
        let url = build_https_link(&link, &Origin::default());
        let (base, fragment) = url.split_once('#').unwrap();
        let secret = format!("secret={}", hex::encode(link.secret.to_bytes()));
        let rest: Vec<&str> = fragment.split('&').filter(|p| *p != secret).collect();
        let moved = format!("{base}?{secret}#{}", rest.join("&"));
        assert!(parse_https_link(&moved).is_none(), "{moved}");
        assert!(parse_https_link(&format!("{base}?{fragment}")).is_none());
    }

    /// A link printed on an origin of the founder's own parses to the same
    /// link as one on the default.
    #[test]
    fn a_link_printed_on_another_origin_parses_the_same() {
        let link = a_link("house");
        let theirs = Origin::parse("https://links.example.org/our/house/").unwrap();
        let url = build_https_link(&link, &theirs);
        assert!(
            url.starts_with("https://links.example.org/our/house/#"),
            "{url}"
        );
        let parsed = parse_https_link(&url).expect("a link on another origin parses");
        same(&parsed, &link);
        same(
            &parsed,
            &parse_https_link(&build_https_link(&link, &Origin::default())).unwrap(),
        );
    }

    #[test]
    fn an_origin_is_http_with_no_fragment() {
        assert!(Origin::parse("https://svrnme.sh/ring/").is_some());
        assert!(Origin::parse("svrnme.sh/ring/").is_none());
        assert!(Origin::parse("https://h/#x").is_none());
    }
}
