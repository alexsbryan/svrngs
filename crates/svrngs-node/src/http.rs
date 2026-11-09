// SPDX-License-Identifier: AGPL-3.0-or-later
//! The one HTTP client the node and the command ask a page with: the
//! page's host when a group's origin is checked (`svrngs`'s
//! `checked_page`) and when a new svrngs is fetched ([`crate::update`]).
//! iroh's reqwest 0.13, already in the lock: its `rustls-no-provider`
//! brings no TLS provider, and a client built without one panics, so its
//! TLS is given ring and this computer's verifier here.

/// The client both callers use, or why this computer cannot make one.
pub fn client() -> Result<reqwest13::Client, String> {
    builder()?.build().map_err(|e| e.to_string())
}

pub(crate) fn builder() -> Result<reqwest13::ClientBuilder, String> {
    // reqwest's `rustls-no-provider` asks the process for one when it
    // builds; a command that only checks a page may not have installed one.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let ring = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    rustls::ClientConfig::builder_with_provider(ring)
        .with_safe_default_protocol_versions()
        .and_then(rustls_platform_verifier::BuilderVerifierExt::with_platform_verifier)
        .map(|tls| {
            reqwest13::Client::builder().tls_backend_preconfigured(tls.with_no_client_auth())
        })
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
