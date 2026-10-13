// SPDX-License-Identifier: AGPL-3.0-or-later
//! `svrngs relay` (relay-verb; DESIGN §3, "A relay anyone runs"): iroh's
//! own relay server, iroh-relay 1.0.2, the one the computers that dial it
//! are built with, so the relay svrngs runs for everyone is the one anyone
//! runs for themselves (ralph/next/relay/CHARTER.md).
//!
//! It is reached at a name, on the ports every iroh dials a relay at
//! (iroh-relay `defaults.rs`): HTTPS on TCP 443, HTTP on 80, and QUIC
//! address discovery on UDP 7842. Its certificate is Let's Encrypt's, asked
//! and renewed by the relay itself (`CertConfig::LetsEncrypt`) and kept
//! under `<data dir>/relay`. Never at a bare address: an address's
//! certificate needs Let's Encrypt's `shortlived` profile, and
//! tokio-rustls-acme 0.9.1, iroh-relay's ACME client, orders names alone —
//! its `Identifier` is `Dns` only and its order names no profile
//! (`acme.rs:129-130`, `:301-303`); DESIGN §9 keeps the row.
//!
//! Who it lets through is decided per connection
//! (`AccessControl::on_connect`): by default a computer or phone standing
//! in a group this computer holds — the register's one answer,
//! [`RingRail::standing_anywhere`], read from the record on disk as svrngs
//! running beside it writes it; with `--open`, everyone, each sending at
//! most [`OPEN_RATE`]. Every decision traces at debug.

use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iroh_relay::defaults::{DEFAULT_HTTPS_PORT, DEFAULT_HTTP_PORT, DEFAULT_RELAY_QUIC_PORT};
use iroh_relay::server::{
    Access, AccessControl, AcmeConfig, CertConfig, ClientRateLimit, ClientRequest, QuicConfig,
    RelayConfig, Server, ServerConfig, TlsConfig,
};
use svrngs_node::{namespaces_in, RingRail, SigningKey};

use crate::cli::Refusal;
use crate::identity;

/// What an `--open` relay lets each computer or phone send it, in bytes a
/// second: the most a phone's film at Demo A's bitrate drew through a
/// relay in any one second, 8,799,728 of three takes, rounded up
/// (`demo/runs/relay-rate/2026-10-12/`).
pub const OPEN_RATE: NonZeroU32 = NonZeroU32::new(8_800_000).unwrap();

/// What `svrngs relay` was asked.
#[derive(Debug, PartialEq, Eq)]
pub struct Relay {
    pub at: At,
    /// `--open`: everyone, each held to [`OPEN_RATE`].
    pub open: bool,
}

/// Where it is reached.
#[derive(Debug, PartialEq, Eq)]
pub enum At {
    /// `https://<name>/`, its certificate from Let's Encrypt, from its
    /// staging service with `--staging`.
    Name { name: String, staging: bool },
    /// Plain HTTP at an address of this computer's, no certificate, as
    /// iroh-relay's `--dev`: what the tests serve. No flag says it.
    Here(SocketAddr),
}

/// `raw` as the name a relay is reached at, lower case; an address is
/// refused, saying why.
pub fn name(raw: &str) -> Result<String, String> {
    if raw.trim_matches(['[', ']']).parse::<IpAddr>().is_ok() {
        return Err(format!("`{raw}` is an address, and a relay's certificate is for a name: give it a name that leads to this computer"));
    }
    let label = |l: &str| !l.is_empty() && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    (raw.contains('.') && raw.split('.').all(label))
        .then(|| raw.to_ascii_lowercase())
        .ok_or_else(|| {
            format!("`{raw}` is not a name a relay is reached at, as relay.example.org is")
        })
}

/// Who the relay lets through: everyone, or whoever stands in a group
/// held under the data dir, read with this computer's key.
#[derive(Debug)]
struct Admits(Option<(PathBuf, Arc<SigningKey>)>);

impl AccessControl for Admits {
    async fn on_connect(&self, request: &ClientRequest) -> Access {
        let key = hex::encode(request.endpoint_id().as_bytes());
        let Some((root, signer)) = self.0.clone() else {
            tracing::debug!(target: "relay", key, "relay: admitted — the relay is open");
            return Access::Allow;
        };
        let asked = key.clone();
        // The record is read from disk, off the relay's own tasks.
        let stands = tokio::task::spawn_blocking(move || {
            RingRail::new(root, signer).standing_anywhere(&asked)
        });
        if let Some((group, member)) = stands.await.ok().flatten() {
            tracing::debug!(target: "relay", key, group, person = %member.person, "relay: admitted — the key stands in a group held here");
            return Access::Allow;
        }
        tracing::debug!(target: "relay", key, "relay: refused — the key stands in no group held here");
        Access::Deny { reason: None }
    }
}

/// The relay `relay` asks for, serving over the data dir `root`, until the
/// [`Server`] is shut down. Refused for its groups when this computer is in
/// none, so it would let nobody through.
pub async fn spawn(relay: &Relay, root: &Path) -> Result<Server, Refusal> {
    let admits = if relay.open {
        Admits(None)
    } else if namespaces_in(root).is_ok_and(|n| !n.is_empty()) {
        let key = identity::load_or_generate_node_key(root)?;
        Admits(Some((root.to_path_buf(), Arc::new(key))))
    } else {
        tracing::debug!(target: "relay", root = %root.display(), "relay: refused — this computer holds no group");
        return Err(Refusal::RelayForNobody);
    };
    let any = |port| SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));
    let (http, tls, quic) = match &relay.at {
        At::Here(addr) => (*addr, None, None),
        At::Name { name, staging } => {
            let builder = rustls::ServerConfig::builder_with_provider(Arc::new(
                rustls::crypto::ring::default_provider(),
            ))
            .with_safe_default_protocol_versions()
            .expect("ring's protocols")
            .with_no_client_auth();
            let acme = AcmeConfig::letsencrypt(!staging)
                .domains(vec![name.clone()])
                .cache_path(root.join("relay"));
            let cert = CertConfig::LetsEncrypt {
                acme_config: acme,
                server_config_builder: builder,
            };
            (
                any(DEFAULT_HTTP_PORT),
                Some(TlsConfig::new(any(DEFAULT_HTTPS_PORT), cert)),
                Some(QuicConfig::new(any(DEFAULT_RELAY_QUIC_PORT))),
            )
        }
    };
    let mut config = RelayConfig::new(http);
    config.tls = tls;
    if relay.open {
        config.limits.client_rx = Some(ClientRateLimit::new(OPEN_RATE));
    }
    tracing::debug!(target: "relay", ?relay, ?admits, "relay: starting");
    config.access = Arc::new(admits);
    let mut server = ServerConfig::default();
    server.relay = Some(config);
    server.quic = quic;
    Server::spawn(server).await.map_err(|e| {
        tracing::debug!(target: "relay", error = %e, "relay: refused — it did not start");
        Refusal::RelayStart(e.to_string())
    })
}

/// `svrngs relay`: start it, say so, and run until Ctrl-C.
pub async fn run(relay: Relay, root: &Path) -> Result<(), Refusal> {
    let mut server = spawn(&relay, root).await?;
    let at = match &relay.at {
        At::Name { name, .. } => format!("https://{name}/"),
        At::Here(_) => format!("http://{}/", server.http_addr().expect("it serves HTTP")),
    };
    let whom = if relay.open {
        format!(
            "every computer and phone that reaches it, each sending at most {} Mbit/s",
            OPEN_RATE.get() * 8 / 1_000_000
        )
    } else {
        "the computers and phones of the groups this computer is in".to_string()
    };
    println!("Relaying at {at} for {whom}. Leave this running.");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        ended = server.join() => tracing::debug!(target: "relay", ?ended, "relay: it stopped"),
    }
    if let Err(e) = server.shutdown().await {
        tracing::debug!(target: "relay", error = %e, "relay: it did not stop cleanly");
    }
    Ok(())
}
