// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-reach/src/guest.rs@e1a43bcd2
//! The guest dialer: how a NON-member reaches a lender's mesh over iroh.
//!
//! A borrower is a client of someone else's mesh, like the phone (phase-b-29
//! Q5): it holds a dial string and a bearer, never a membership. So the dial
//! lives here, in the leaf svrn and serve both link, and not in
//! `commonwealth-transport`, which neither may link. (svrngs: what is left
//! is how an endpoint is built. Upstream's one HTTP bridge, a loopback port
//! per peer that every iroh tunnel rode, went in arch-peer (d): any program
//! on the computer reached a friend's door through it as this computer's
//! person, `tests/peer.rs`. A request rides the stream itself,
//! [`crate::iroh::IrohTransport::send`].)

use std::net::SocketAddr;
use std::sync::Arc;

use iroh::endpoint::presets;
use iroh::endpoint::Builder as EndpointBuilder;
use iroh::{Endpoint, RelayUrl, SecretKey};

use crate::relay::{self, DotlessLookup};

#[cfg(test)]
mod tests;

/// The rustls crypto provider for `EndpointBuilder::crypto_provider`.
/// iroh's `Builder::empty()` deliberately sets no provider (only
/// presets choose one), and `bind()` errors without it — pass this.
/// Ring, matching the rest of the workspace's rustls usage.
pub fn ring_crypto_provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// How an endpoint reaches the wider network — the sovereignty knob
/// (H1 of the enterprise-hardening plan). Bundled into one struct so
/// adding a relay/discovery option later is not another signature
/// change across every constructor call site.
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// Custom relay URLs. Empty = the default for the chosen discovery
    /// posture (n0's public relays under `n0_services`, or NO relay —
    /// direct-addr only — without it).
    pub relay_urls: Vec<String>,
    /// Use n0's public infrastructure — BOTH the public relays AND the
    /// n0 DNS/pkarr address-lookup (`iroh.link`). `true` is the
    /// bootstrap default. `false` severs ALL n0 contact: the endpoint
    /// builds from `presets::Minimal` (crypto only — no n0 relay, no n0
    /// DNS), so peers are reached ONLY via gossiped `iroh_direct_addrs`
    /// (a flat LAN/VPC) and/or the `relay_urls` above (a self-hosted
    /// relay for cross-subnet NAT traversal). This is the knob a
    /// sovereignty- or air-gap-focused netops team requires — setting
    /// `relay_urls` alone does NOT stop the n0 DNS lookup.
    pub n0_services: bool,
    /// Listen on 127.0.0.1 alone (svrngs): no other computer can reach the
    /// endpoint, and it learns no other address of this computer to offer
    /// a peer. Nodes sharing one computer in a test use it, so iroh cannot
    /// move their connection onto the computer's private address, which the
    /// door guesses is home (ralph/REVIEW_FINDINGS.md #33). `false` listens
    /// on every interface, as iroh does.
    pub loopback_only: bool,
}

impl Default for RelayConfig {
    fn default() -> Self {
        // Bootstrap posture: full n0 (relays + DNS). Callers that want
        // sovereignty opt out with `n0_services: false`.
        Self {
            relay_urls: Vec::new(),
            n0_services: true,
            loopback_only: false,
        }
    }
}

/// Build an iroh endpoint from a node identity, serving `alpns`, per a
/// [`RelayConfig`]. With `n0_services` it starts from `presets::N0`
/// (n0 relays + n0 DNS/pkarr lookup); without it, from `presets::Minimal`
/// (crypto only — no n0 anything), so a sovereign/air-gapped node reaches
/// peers by gossiped direct addrs and/or self-hosted `relay_urls` alone.
/// Non-empty `relay_urls` overrides the relay set in either mode; an
/// empty list under Minimal means relays disabled (direct-addr only).
/// Always calls [`EndpointBuilder::proxy_from_env`], so a corporate
/// `HTTP_PROXY`/`HTTPS_PROXY` (incl. Basic auth) is honored for the
/// relay's WebSocket-over-TLS/443 connection — the path that carries the
/// mesh when UDP is blocked.
///
/// One constructor so every production caller — the mobile host
/// ([`crate`] consumers like `sovereign-server::iroh_access`) and the
/// mesh daemon — binds identically and any iroh-API churn stays here.
/// Hermetic tests still use `EndpointBuilder::empty()` directly.
pub async fn build_relayed_endpoint(
    secret_key: SecretKey,
    alpns: Vec<Vec<u8>>,
    cfg: &RelayConfig,
) -> Result<Endpoint, String> {
    relayed_endpoint_builder(secret_key, alpns, cfg)?
        .bind()
        .await
        .map_err(|e| format!("iroh endpoint bind failed: {e}"))
}

/// The shared builder behind [`build_relayed_endpoint`] — preset/relay/proxy
/// policy lives exactly once.
fn relayed_endpoint_builder(
    secret_key: SecretKey,
    alpns: Vec<Vec<u8>>,
    cfg: &RelayConfig,
) -> Result<EndpointBuilder, String> {
    let preset_builder = if cfg.n0_services {
        // svrngs: the preset's two lookups, the resolver's answers read
        // dotless (crate::relay, u4-9); `presets::N0` adds these two
        // (iroh-1.0.2/src/endpoint/presets.rs:116-136).
        EndpointBuilder::new(presets::N0)
            .clear_address_lookup()
            .address_lookup(iroh::address_lookup::PkarrPublisher::n0_dns())
            .address_lookup(DotlessLookup(
                iroh::address_lookup::DnsAddressLookup::n0_dns(),
            ))
    } else {
        // Minimal = crypto only: no n0 relay, no n0 DNS. Nothing this
        // endpoint does will contact n0 infrastructure.
        EndpointBuilder::new(presets::Minimal)
    };
    let mut builder = preset_builder
        .crypto_provider(ring_crypto_provider())
        .secret_key(secret_key)
        .alpns(alpns)
        // Honor corporate HTTP(S) proxies on the relay dial. No-op when
        // the env vars are unset, so it's safe on every deployment.
        .proxy_from_env();
    // Glassbox the egress posture: which relays this node uses, from the log,
    // not by inference. (svrngs: upstream's proxy reading went, arch-peer.)
    tracing::info!(target: "transport", n0_services = cfg.n0_services, loopback_only = cfg.loopback_only, relays = ?cfg.relay_urls,
        "iroh egress posture (no relay named is n0's with n0_services, none without it; n0_services=false severs all n0 contact)");
    match parse_relay_mode(&cfg.relay_urls) {
        Some(mode) => builder = builder.relay_mode(mode),
        None if !cfg.n0_services => {
            // Sovereign mode, no custom relay: disable relays entirely.
            // Peers are reached by gossiped direct addrs only (flat
            // LAN/VPC). Minimal carries no relay by default, but be
            // explicit so intent is unmistakable.
            builder = builder.relay_mode(iroh::RelayMode::Disabled);
        }
        // n0_services + no custom relays → n0's default relays; svrngs:
        // dotless, so the relay this computer advertises is one WebKit
        // reaches (crate::relay, u4-9).
        None => builder = builder.relay_mode(iroh::RelayMode::Custom(relay::n0_relays())),
    }
    if cfg.loopback_only {
        builder = builder
            .clear_ip_transports()
            .bind_addr(SocketAddr::from(([127, 0, 0, 1], 0)))
            .map_err(|e| format!("iroh loopback address refused: {e}"))?;
    }
    Ok(builder)
}

/// Turn operator-configured relay URL strings into a custom
/// [`RelayMode`]. `None` (empty input, or every entry unparseable)
/// leaves the caller on the preset default. Unparseable entries are
/// logged and skipped rather than aborting the bind — a fat-fingered
/// relay URL must not take a node offline when the default relays
/// would still work.
fn parse_relay_mode(relay_urls: &[String]) -> Option<iroh::RelayMode> {
    if relay_urls.is_empty() {
        return None;
    }
    let parsed: Vec<RelayUrl> = relay_urls
        .iter()
        .filter_map(|u| match relay::parse(u) {
            Ok(url) => Some(url),
            Err(e) => {
                tracing::warn!(
                    target: "transport",
                    url = %u,
                    error = %e,
                    "iroh: ignoring unparseable relay_url — falling back to remaining/default relays"
                );
                None
            }
        })
        .collect();
    if parsed.is_empty() {
        tracing::warn!(
            target: "transport",
            "iroh: all configured relay_urls were unparseable — using default relays"
        );
        return None;
    }
    Some(iroh::RelayMode::custom(parsed))
}
