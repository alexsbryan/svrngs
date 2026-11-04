// SPDX-License-Identifier: AGPL-3.0-or-later
//! A signature for an app (DESIGN §3, "What an app gets"; svrn-docs
//! `rings/current/ON_AIR.md` §4.2; onair-sign). New: upstream signed only
//! its own acts and bindings. An app on a computer has the computer's own
//! key — the one that signs its acts — sign bytes it chose, under a domain
//! of their own: `"svrngs-app:" || app || ":" || bytes`. Whoever holds the
//! key, the app's name and the bytes checks it with Ed25519 alone, outside
//! the group too. The app's name is a closed form without `:`, so the `:`
//! after it ends it, and no two (app, bytes) sign the same message; the
//! domain begins no other ([`crate::sig`]'s home of domains), so an app's
//! signature is never an act's, a link's, a pass's or a consent, nor any of
//! those an app's. Pure: the key is an argument.

use alloc::string::String;
use alloc::vec::Vec;

use crate::{sign_message, verify_hex, SigningKey};

/// An app's name, as it is signed under: 1 to 64 of `a`-`z`, `0`-`9` and
/// `-`. Parsed once, where it arrives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App(String);
crate::text_newtype!(App);

/// Why some text is not an [`App`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("not an app's name: 1 to 64 of a-z, 0-9 and -")]
pub struct NotAnApp;

impl App {
    pub fn parse(raw: &str) -> Result<Self, NotAnApp> {
        let closed = (raw.bytes()).all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-'));
        ((1..=64).contains(&raw.len()) && closed)
            .then(|| Self(raw.into()))
            .ok_or(NotAnApp)
    }

    /// What is signed for this app over `bytes`:
    /// `"svrngs-app:" || app || ":" || bytes`.
    pub fn message(&self, bytes: &[u8]) -> Vec<u8> {
        [crate::sig::APP_DOMAIN, self.0.as_bytes(), b":", bytes].concat()
    }

    /// `key`'s signature over `bytes` for this app, in hex.
    pub fn sign(&self, key: &SigningKey, bytes: &[u8]) -> String {
        sign_message(key, &self.message(bytes))
    }

    /// Whether `sig` is `key`'s, as the record spells an actor, over
    /// `bytes` for this app.
    pub fn check(&self, key: &str, bytes: &[u8], sig: &str) -> bool {
        verify_hex(key, &self.message(bytes), sig)
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
