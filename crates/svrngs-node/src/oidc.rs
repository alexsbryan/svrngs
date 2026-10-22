// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sign in with svrngs: the lender's signing key, the JWKS from it, and the
//! tokens the door's provider signs (SIGN_IN_WITH_SVRNGS §5, §6, §8 3).
//!
//! **One key per lender's computer**, at `<data dir>/oidc/key.pk8`, 0600,
//! made on first use and kept across a restart so an app's cached keys
//! hold. It is never in the record and never synced: it lives in the data
//! dir, which no group's folder reaches. It is ES256 through `ring`, which
//! the lock already holds through rustls.
//!
//! The public half is the JWKS ([`Key::jwks`]), read from the same key pair
//! that signs ([`Key::token`]), so the key a stock client verifies is the
//! key that signed. Every decision is traced at debug, never the key.

use std::path::{Path, PathBuf};

use ring::rand::SystemRandom;
use ring::signature::{
    EcdsaKeyPair, KeyPair, UnparsedPublicKey, ECDSA_P256_SHA256_FIXED,
    ECDSA_P256_SHA256_FIXED_SIGNING,
};
use svrngs_core::token::{Claims, Token};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;

/// The key's file under [`key_dir`], an unencrypted PKCS#8 document.
pub const KEY_FILE: &str = "key.pk8";

/// Where this computer keeps its sign-in key: `<data dir>/oidc`.
pub fn key_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("oidc")
}

/// Why a token could not be made.
#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("cannot keep the sign-in key at {0}: {1}")]
    Store(PathBuf, std::io::Error),
    #[error("the sign-in key at {0} is not an ES256 key: {1}")]
    Rejected(PathBuf, ring::error::KeyRejected),
    #[error("cannot draw the sign-in key's randomness: {0}")]
    Random(ring::error::Unspecified),
    #[error("cannot sign a token: {0}")]
    Sign(ring::error::Unspecified),
}

/// The lender's ES256 key pair, its public point and the `kid` naming it.
pub struct Key {
    pair: EcdsaKeyPair,
    /// The uncompressed point `04 || x[32] || y[32]`, from the same pair.
    point: Vec<u8>,
    kid: String,
    rng: SystemRandom,
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Key").field("kid", &self.kid).finish()
    }
}

/// The key at `data_dir`, made and kept owner-only if this computer has
/// none yet, else the one it kept; the same across a restart.
pub fn load_or_create(data_dir: &Path) -> Result<Key, KeyError> {
    let dir = key_dir(data_dir);
    let path = dir.join(KEY_FILE);
    let rng = SystemRandom::new();
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => {
            tracing::debug!(target: "oidc", path = %path.display(), "oidc: the sign-in key this computer kept, read");
            bytes
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let document = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
                .map_err(KeyError::Random)?;
            crate::declared::secret_dir(&dir).map_err(|e| KeyError::Store(dir.clone(), e))?;
            crate::declared::write_secret(&path, document.as_ref())
                .map_err(|(at, e)| KeyError::Store(at, e))?;
            tracing::debug!(target: "oidc", path = %path.display(), "oidc: made one sign-in key for this computer, once, owner-only");
            document.as_ref().to_vec()
        }
        Err(e) => return Err(KeyError::Store(path, e)),
    };
    let pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &bytes, &rng)
        .map_err(|e| KeyError::Rejected(path.clone(), e))?;
    let point = pair.public_key().as_ref().to_vec();
    let kid = kid_of(&point);
    tracing::debug!(target: "oidc", kid, "oidc: the sign-in key is ready");
    Ok(Key {
        pair,
        point,
        kid,
        rng,
    })
}

impl Key {
    /// The `kid` in the token's header and in the JWKS: the first 16 bytes
    /// of the public point's SHA-256, in hex, so one string names the key.
    pub fn kid(&self) -> &str {
        &self.kid
    }

    /// The token the core spells, signed with this key.
    pub fn token(&self, claims: Claims) -> Result<String, KeyError> {
        let token = Token::new(claims, &self.kid);
        let signature = self
            .pair
            .sign(&self.rng, token.signing_input().as_bytes())
            .map_err(KeyError::Sign)?;
        let signed = token.signed(signature.as_ref());
        tracing::debug!(target: "oidc", kid = %self.kid, sub = %token.claims.sub, aud = %token.claims.aud, "oidc: signed a token");
        Ok(signed)
    }

    /// The claims of `token` when this key signed it, for `issuer` and
    /// `client`, and it is not expired at `at`; or the fact that refused it.
    /// The access token is the signed token itself, so this is userinfo's
    /// one check.
    pub fn verified(
        &self,
        token: &str,
        issuer: &str,
        client: &str,
        at: i64,
    ) -> Result<serde_json::Value, &'static str> {
        let (input, signature) = token
            .rsplit_once('.')
            .ok_or("the token is not three parts")?;
        let signature = URL_SAFE_NO_PAD
            .decode(signature)
            .map_err(|_| "the token's signature is not base64url")?;
        UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, &self.point)
            .verify(input.as_bytes(), &signature)
            .map_err(|_| "another key signed the token")?;
        let (_, payload) = input
            .split_once('.')
            .ok_or("the token is not three parts")?;
        let payload = URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| "the token's claims are not base64url")?;
        let claims: serde_json::Value =
            serde_json::from_slice(&payload).map_err(|_| "the token's claims are not JSON")?;
        if claims["iss"] != issuer {
            return Err("the token is another issuer's");
        }
        if claims["aud"] != client {
            return Err("the token is another client's");
        }
        if claims["exp"].as_i64().is_none_or(|exp| exp <= at) {
            return Err("the token has expired");
        }
        tracing::debug!(target: "oidc", kid = %self.kid, "oidc: a token verified under this computer's key");
        Ok(claims)
    }

    /// The JWKS an app fetches: this key alone, `EC` on `P-256`, its `x` and
    /// `y` the public point's halves, and the `kid` the header names.
    pub fn jwks(&self) -> String {
        let x = url(self.point.get(1..33).expect("an uncompressed point's x"));
        let y = url(self.point.get(33..65).expect("an uncompressed point's y"));
        let jwks = format!(
            r#"{{"keys":[{{"kid":"{}","kty":"EC","crv":"P-256","x":"{x}","y":"{y}"}}]}}"#,
            self.kid
        );
        tracing::debug!(target: "oidc", kid = %self.kid, "oidc: the JWKS a client reads");
        jwks
    }
}

/// `point`'s `kid`: the first 16 bytes of its SHA-256, in lowercase hex.
fn kid_of(point: &[u8]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, point);
    hex::encode(&digest.as_ref()[..16])
}

/// `bytes` base64url, unpadded: how a JWK spells `x` and `y`.
fn url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub(crate) mod provider;

pub(crate) use provider::Code;
pub use provider::Provider;

#[cfg(test)]
#[path = "oidc/tests.rs"]
mod tests;
