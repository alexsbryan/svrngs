// SPDX-License-Identifier: AGPL-3.0-or-later
//! A sign-in token: what an app is told about the person the door already
//! knows (SIGN_IN_WITH_SVRNGS §5, "The person's details the app is told").
//!
//! The claims are the door's identity headers and nothing else: `sub` the
//! person's key, `preferred_username` their account, `name` their name,
//! `groups` `["member"]` or `["guest"]`, and the token's own `iss`, `aud`,
//! `exp` and `iat`. There is no `email`: svrngs has none (§8 4).
//!
//! Pure and synchronous (RUST_ARCH §2): this module spells the header, the
//! claims and the signing input. The node signs the input with the
//! lender's ES256 key ([`crate::token::Token::signed`]).

#[cfg(test)]
#[path = "token_tests.rs"]
mod tests;
