// SPDX-License-Identifier: AGPL-3.0-or-later
//! A sign-in token: what an app is told about the person the door already
//! knows (SIGN_IN_WITH_SVRNGS §5, "The person's details the app is told").
//!
//! The claims are the door's identity headers and nothing else: `sub` the
//! person's key, `preferred_username` their account, `name` their name,
//! `groups` `["member"]` or `["guest"]`, and the token's own `iss`, `aud`,
//! `exp` and `iat`. There is no `email`: svrngs has none (§8 4). The
//! issuer is handed in as one string, so every token of a lender's is
//! byte-equal in `iss` to discovery (§5a, "One issuer string, exact").
//!
//! Pure and synchronous (RUST_ARCH §2): this module spells the header, the
//! claims and the signing input. The node signs the input with the
//! lender's ES256 key ([`Token::signed`]), and the core links no signature
//! crate.

use alloc::string::String;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde::Serialize;

/// The group a person's token names: the door's closed set
/// (SIGN_IN_WITH_SVRNGS §8 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    /// A person of the group.
    Member,
    /// A visit's guest, named as the guest and never as their host.
    Guest,
}

/// What a sign-in token says about the person, by the claims' own names.
/// Built only by [`Claims::new`], which takes the door's `Asker` fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Claims {
    /// The lender's one issuer string, exact.
    pub iss: String,
    /// The person's key, the same from every device: `x-svrngs-key`.
    pub sub: String,
    /// The lend's client id.
    pub aud: String,
    /// Seconds since the epoch, after which the token is refused.
    pub exp: i64,
    /// Seconds since the epoch when it was made.
    pub iat: i64,
    /// The person's name: `x-svrngs-person`.
    pub name: String,
    /// Their account, unique by key: `x-svrngs-user`.
    pub preferred_username: String,
    /// One group: `["member"]` or `["guest"]`.
    pub groups: [Group; 1],
}

impl Claims {
    /// The claims for the person `subject`, account `username` and name
    /// `name`, made by `issuer` for `audience` at `issued_at`, live for
    /// `lifetime` seconds. Nothing else is added, so an app is told no more
    /// than the door already sends it.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        issuer: &str,
        subject: &str,
        audience: &str,
        name: &str,
        username: &str,
        group: Group,
        issued_at: i64,
        lifetime: i64,
    ) -> Self {
        tracing::debug!(target: "oidc", iss = issuer, sub = subject, aud = audience, ?group, iat = issued_at, exp = issued_at.saturating_add(lifetime), "oidc: the claims a token will carry");
        Self {
            iss: issuer.into(),
            sub: subject.into(),
            aud: audience.into(),
            exp: issued_at.saturating_add(lifetime),
            iat: issued_at,
            name: name.into(),
            preferred_username: username.into(),
            groups: [group],
        }
    }
}

/// The JOSE header's fixed fields: ES256, a JWT, and the lender's key id.
#[derive(Serialize)]
struct Header<'a> {
    alg: &'static str,
    typ: &'static str,
    kid: &'a str,
}

/// A token before its signature: the claims, and the key id that will
/// verify it. [`Self::signing_input`] is the bytes the node's ES256 key
/// signs, and [`Self::signed`] is the token an app receives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// What the app is told about the person.
    pub claims: Claims,
    kid: String,
}

impl Token {
    /// The claims, to be signed by the key `kid` names.
    pub fn new(claims: Claims, kid: &str) -> Self {
        tracing::debug!(target: "oidc", kid, sub = %claims.sub, aud = %claims.aud, "oidc: a token, before its signature");
        Self {
            claims,
            kid: kid.into(),
        }
    }

    /// `base64url(header) || "." || base64url(claims)`: what the key signs.
    /// The header is `{"alg":"ES256","typ":"JWT","kid":…}`, In declaration
    /// order, so a client sees the key the JWKS lists.
    pub fn signing_input(&self) -> String {
        let header = Header {
            alg: "ES256",
            typ: "JWT",
            kid: &self.kid,
        };
        let header = URL_SAFE_NO_PAD
            .encode(serde_json::to_string(&header).expect("a header of strings serializes"));
        let payload = URL_SAFE_NO_PAD.encode(
            serde_json::to_string(&self.claims).expect("claims of strings and integers serialize"),
        );
        alloc::format!("{header}.{payload}")
    }

    /// The token, with the ES256 signature over [`Self::signing_input`].
    pub fn signed(&self, signature: &[u8]) -> String {
        let input = self.signing_input();
        alloc::format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature))
    }
}
