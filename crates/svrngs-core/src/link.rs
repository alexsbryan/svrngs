// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-join-vocab/src/deep_link.rs@e1a43bcd2
//! A group's link (DESIGN §8, U2's inventory, the deep_link.rs row; §10,
//! 2026-10-04): short and opaque (u2-8). It carries only what the joiner
//! cannot ask the issuer for, as one base64url [`Token`]: a [`Format`]
//! byte, the issuer's key — the computer that redeems the link, dialled by
//! key — and 16 secret bytes the link's signing key is derived from
//! ([`Token::key`]); and, from a computer run without address lookup, the
//! issuer's relay (DESIGN §11, risk 5), since a bare key is dialled through
//! lookup otherwise (§8, "A bare key is dialled"). The group's seed and the
//! `Invite`'s id are the issuer's answer to the link's key
//! (`svrngs_node::redeem::ask`).
//!
//! Two wrappers carry the one token, and neither depends on a hosted URL:
//! the client's link, `svrngs://join/<token>`, upstream's
//! `sovereign://join/<key>` shape (`build_join_link`, `:421-436`), which
//! `new` and `link` always print; and `<origin>#<token>`, upstream's https
//! form with the token in its fragment, printed beside it only for a group
//! that keeps a page origin (u2-7, `group::write_origin`). A browser does
//! not send the fragment to the server, so the origin's page sees the
//! token only through its own script, and a computer never fetches the
//! origin. `svrngs join` takes either, or the bare token. Upstream's
//! fields, percent helpers and lapse are gone: the lapse is the issuer's
//! `Invite`'s, and the issuer's node decides a redemption (DESIGN §6).
//!
//! In svrngs-core since u4-2 (DESIGN §10, call 5), so the command and a
//! browser's page parse a link with one body. The core has no iroh, so the
//! two types it named are the ones iroh's wrap, imported under iroh's
//! names: the issuer's key is the ed25519 key iroh's `PublicKey::from_bytes`
//! checks (`iroh-base-1.0.2/src/key.rs:122-127`), and its relay the `Url`
//! iroh's `RelayUrl::from_str` parses (`relay_url.rs:38-45`), so a token
//! parses here as it did through iroh. Drawing a secret and dialling the issuer
//! stay the command's (`svrngs::issuer`): the core draws nothing and dials
//! nothing.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use ed25519_dalek::{SigningKey, VerifyingKey as PublicKey};
use url::Url as RelayUrl;

/// The client's link's prefix, before the token.
pub const CLIENT_PREFIX: &str = "svrngs://join/";

/// The words DESIGN §1 keeps from people. The command's `mint` draws a
/// link's secret again while its token spells one, in any case.
pub const KEPT_WORDS: [&str; 7] = ["ring", "member", "key", "sync", "journal", "node", "relay"];

/// BLAKE3 `derive_key` context for a link's signing key from its secret.
const LINK_KEY_CONTEXT: &str = "svrngs 2026-10-04 link signing key from its secret";

/// A link's secret: 128 bits.
pub const SECRET_LEN: usize = 16;

/// What a token's bytes after its format byte hold: a closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// The issuer's key and the secret: the issuer is found by lookup.
    Bare,
    /// The issuer's key, the secret and its relay's URL, for an issuer run
    /// without address lookup.
    Relayed,
}

impl Format {
    const fn byte(self) -> u8 {
        match self {
            Format::Bare => 1,
            Format::Relayed => 2,
        }
    }

    fn of(byte: u8) -> Option<Self> {
        [Format::Bare, Format::Relayed]
            .into_iter()
            .find(|f| f.byte() == byte)
    }
}

/// A link's one token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The issuer's computer: the one that redeems this link.
    pub issuer: PublicKey,
    /// The secret the link's signing key is derived from.
    pub secret: [u8; SECRET_LEN],
    /// The issuer's relay, carried only from a computer without lookup.
    pub relay: Option<RelayUrl>,
}

impl Token {
    /// The link's signing key. Its public half is the key the issuer's
    /// `Invite` names; it is in no record.
    pub fn key(&self) -> SigningKey {
        SigningKey::from_bytes(&blake3::derive_key(LINK_KEY_CONTEXT, &self.secret))
    }

    /// `format || issuer[32] || secret[16] || relay?`, base64url unpadded.
    pub fn encode(&self) -> String {
        let format = match self.relay {
            Some(_) => Format::Relayed,
            None => Format::Bare,
        };
        let mut bytes = vec![format.byte()];
        bytes.extend_from_slice(self.issuer.as_bytes());
        bytes.extend_from_slice(&self.secret);
        if let Some(relay) = &self.relay {
            bytes.extend_from_slice(relay.to_string().as_bytes());
        }
        URL_SAFE_NO_PAD.encode(bytes)
    }

    /// The token [`Self::encode`] wrote, or `None`: not base64url, an
    /// unknown format byte, cut short, a key that is no key, or a relay
    /// that is no URL.
    pub fn decode(text: &str) -> Option<Self> {
        let Ok(bytes) = URL_SAFE_NO_PAD.decode(text.trim()) else {
            tracing::debug!(target: "link", "link: refused — the token is not base64url");
            return None;
        };
        let Some(format) = bytes.first().copied().and_then(Format::of) else {
            tracing::debug!(target: "link", first = ?bytes.first(), "link: refused — no format byte this build reads");
            return None;
        };
        const FIXED: usize = 1 + 32 + SECRET_LEN;
        let rest = bytes.get(FIXED..);
        let relay = match (format, rest) {
            (Format::Bare, Some([])) => None,
            (Format::Relayed, Some(rest)) if !rest.is_empty() => {
                let parsed = core::str::from_utf8(rest)
                    .ok()
                    .and_then(|s| s.parse::<RelayUrl>().ok());
                if parsed.is_none() {
                    tracing::debug!(target: "link", "link: refused — the relay is not a URL");
                    return None;
                }
                parsed
            }
            _ => {
                tracing::debug!(target: "link", ?format, len = bytes.len(), "link: refused — the token is not its format's length");
                return None;
            }
        };
        let issuer: [u8; 32] = bytes[1..33].try_into().expect("checked above");
        let Ok(issuer) = PublicKey::from_bytes(&issuer) else {
            tracing::debug!(target: "link", "link: refused — the issuer's key is no key");
            return None;
        };
        let secret = bytes[33..FIXED].try_into().expect("checked above");
        Some(Self {
            issuer,
            secret,
            relay,
        })
    }

    /// Whether the token spells a word DESIGN §1 keeps from people.
    pub fn spells_a_kept_word(&self) -> bool {
        let lower = self.encode().to_lowercase();
        KEPT_WORDS.iter().any(|w| lower.contains(w))
    }
}

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

/// A parsed link: its token, and the origin it was printed on when it was
/// given in the https form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub token: Token,
    pub origin: Option<Origin>,
}

impl Link {
    /// A link in any of its three forms: `svrngs://join/<token>`,
    /// `<origin>#<token>` — the part left of the `#` read for nothing but
    /// the origin — or the bare token.
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let (origin, token) = if let Some(token) = raw.strip_prefix(CLIENT_PREFIX) {
            (None, token)
        } else if let Some((base, token)) = raw.split_once('#') {
            (Some(Origin::parse(base)?), token)
        } else {
            (None, raw)
        };
        let token = Token::decode(token)?;
        tracing::debug!(target: "link", https = origin.is_some(), "link: parsed");
        Some(Self { token, origin })
    }
}

/// The client's link to `token`.
pub fn client_link(token: &Token) -> String {
    format!("{CLIENT_PREFIX}{}", token.encode())
}

/// The https link to `token` on `origin`: the token in the fragment.
pub fn https_link(origin: &Origin, token: &Token) -> String {
    format!("{}#{}", origin.as_str(), token.encode())
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
