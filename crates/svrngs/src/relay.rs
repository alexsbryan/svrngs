// SPDX-License-Identifier: AGPL-3.0-or-later
//! `svrngs relay` (relay-verb; DESIGN §3, "A relay anyone runs"): iroh's
//! own relay server, iroh-relay 1.4.2, the one the computers that dial it
//! are built with, so the relay svrngs runs for everyone is the one anyone
//! runs for themselves (ralph/next/relay/CHARTER.md).
//!
//! It is reached at a name, on the ports every iroh dials a relay at
//! (iroh-relay `defaults.rs`): HTTPS on TCP 443, HTTP on 80, and QUIC
//! address discovery on UDP 7842. Its certificate is Let's Encrypt's, asked
//! and renewed by the relay itself (`CertConfig::LetsEncrypt`) and kept
//! under `<data dir>/relay`. Never HTTPS at a bare address: an address's
//! certificate needs Let's Encrypt's `shortlived` profile, and
//! tokio-rustls-acme 0.9.1, iroh-relay's ACME client, orders names alone —
//! its `Identifier` is `Dns` only and its order names no profile
//! (`acme.rs:129-130`, `:301-303`); PLAN §2 keeps the row. At an address
//! it serves plain HTTP alone, [`At::Here`].
//!
//! Who it lets through is decided per connection
//! (`AccessControl::on_connect`): by default a computer or phone standing
//! in a group this computer holds — the register's one answer,
//! [`RingRail::standing_anywhere`], read from the record on disk as svrngs
//! running beside it writes it — and a lend's worker acting for such a
//! person ([`pass_holds`], self-host): its key stands in no group, and
//! the token it presents is its pass and the holder's link naming that
//! key, judged as the lending computer's door judges them; with `--open`,
//! everyone, each held to the share, [`SHARE_RATE`] with a
//! [`SHARE_BURST`]. Every decision traces at debug.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iroh_relay::defaults::{
    DEFAULT_HTTPS_PORT, DEFAULT_HTTP_PORT, DEFAULT_METRICS_PORT, DEFAULT_RELAY_QUIC_PORT,
};
use iroh_relay::server::{
    Access, AccessControl, AcmeConfig, CertConfig, ClientRateLimit, ClientRequest, QuicConfig,
    RelayConfig, Server, ServerConfig, TlsConfig,
};
use svrngs_node::{authority::resolve, pass::presented, register::Member};
use svrngs_node::{namespaces_in, RailError, RingRail, SigningKey};
use tokio::task::JoinError;

use crate::cli::{Refusal, Refused, Unjudged};
use crate::identity;

/// What an `--open` relay holds each computer or phone to, in bytes a
/// second: about one megabit. The numbers come from measurement: an AI
/// answer passes whole (0.057 Mbit/s), a first open fits the burst beside
/// it (Jellyfin's 2.11 MB brotli, 5.66 MB decoded; CHARTER, "Direction"),
/// and a film (3.73 Mbit/s) is held once the burst is spent.
pub const SHARE_RATE: NonZeroU32 = NonZeroU32::new(125_000).unwrap();

/// The most one may send before [`SHARE_RATE`] holds it: an 8 MiB burst,
/// which carries a first open whole.
pub const SHARE_BURST: NonZeroU32 = NonZeroU32::new(8 * 1024 * 1024).unwrap();

/// What `svrngs relay` logs with no `RUST_LOG` set: warnings, the relay's
/// own and iroh-relay's, so a host's capped journal keeps what went wrong
/// and never the debug decisions, which carry keys and addresses
/// (RELAY_COMMONS design 3, "The host's basics").
pub const FILTER: &str = "relay=warn,iroh_relay=warn";

/// What `svrngs relay` was asked.
#[derive(Debug, PartialEq, Eq)]
pub struct Relay {
    pub at: At,
    /// `--open`: everyone, each held to the share, [`SHARE_RATE`] with a
    /// [`SHARE_BURST`].
    pub open: bool,
}

/// Where it is reached.
#[derive(Debug, PartialEq, Eq)]
pub enum At {
    /// `https://<name>/`, its certificate from Let's Encrypt, from its
    /// staging service with `--staging`.
    Name { name: String, staging: bool },
    /// Plain HTTP at an address of this computer's, no certificate, as
    /// iroh-relay's `--dev`: what the tests serve, and `--at <address>`, a
    /// flag `svrngs help` does not list, for a relay on a local network
    /// with no internet, where Let's Encrypt reaches no name. What crosses
    /// it is iroh's QUIC, encrypted end to end, as at a name; whom it lets
    /// through is decided as at a name.
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
/// held under the data dir, read with this computer's key, and a key a
/// pass of theirs and its holder's link name (self-host).
#[derive(Debug)]
struct Admits(Option<(PathBuf, Arc<SigningKey>)>);

impl AccessControl for Admits {
    async fn on_connect(&self, request: &ClientRequest) -> Access {
        let key = hex::encode(request.endpoint_id().as_bytes());
        let Some((root, signer)) = self.0.clone() else {
            tracing::debug!(target: "relay", key, "relay: admitted — the relay is open");
            return Access::Allow;
        };
        // A lend's worker (self-host) dials from a key of its run, which
        // stands in no group and which the pass's holder names in a link
        // (web/lend.js `opened`): the pass its person signed and that link
        // are what a group's own relay admits it by, as the door admits
        // them for the lends the pass names; then the person stands here
        // or not, read as their own key's is. A token that holds nothing
        // is refused.
        let person = match request.auth_token().map(|token| pass_holds(&key, &token)) {
            Some(None) => return Access::Deny { reason: None },
            held => held.flatten(),
        };
        let asked = person.clone().unwrap_or_else(|| key.clone());
        // The record is read from disk, off the relay's own tasks.
        let stands = tokio::task::spawn_blocking(move || {
            RingRail::new(root, signer).standing_anywhere(&asked)
        });
        verdict(&key, person.as_deref(), stands.await)
    }
}

/// The person the dialer `key`, presenting `token`, acts for: the token
/// read as a pass and the link carrying it ([`presented`]), judged as the
/// door judges the pair (door.rs `holder_of`), by [`resolve`] — the pass
/// signed by the person it names, live, and held by `key` or by the holder
/// whose link, live and no wider, names `key`. `None` for any other
/// token, each branch traced.
fn pass_holds(key: &str, token: &str) -> Option<String> {
    let (read, clock) = (presented(token), svrngs_node::door::unix_now());
    // Nothing to judge, the trace saying which: no pass and link that
    // read, or no clock to judge them on.
    let (Some((pass, link)), Some(now)) = (&read, clock) else {
        tracing::debug!(target: "relay", key, token_reads = read.is_some(), clock_reads = clock.is_some(), "relay: refused — the token is not a pass, nor a pass and its link, that reads, or this computer's clock cannot be read");
        return None;
    };
    // The lend asked is the pass's own app: a relay opens no lend, and a
    // pair the door takes for any lend the pass names it takes for its app
    // (`resolve`'s `Wider`: a link names the pass's app).
    if let Err(why) = resolve(pass, link.as_ref(), key, &pass.app, now, || ()) {
        tracing::debug!(target: "relay", key, person = %pass.person, %why, "relay: refused — a pass that does not admit this dialer");
        return None;
    }
    Some(pass.person.clone())
}

/// Whether `key` is let through, by the read of where it — or `person`,
/// the signer a pass names (self-host) — stands, made off the relay's own
/// tasks. A read that failed — it panicked, or was cancelled — refuses,
/// traced as that, never as the key standing in no group (#129).
fn verdict(
    key: &str,
    person: Option<&str>,
    read: Result<Option<(String, Member)>, JoinError>,
) -> Access {
    match (read, person) {
        (Ok(Some((group, member))), None) => {
            tracing::debug!(target: "relay", key, group, person = %member.person, "relay: admitted — the key stands in a group held here");
            return Access::Allow;
        }
        (Ok(Some((group, _))), Some(person)) => {
            tracing::debug!(target: "relay", key, person, group, "relay: admitted — a key acting for a pass's person, who stands in a group held here");
            return Access::Allow;
        }
        (Ok(None), None) => {
            tracing::debug!(target: "relay", key, "relay: refused — the key stands in no group held here")
        }
        (Ok(None), Some(person)) => {
            tracing::debug!(target: "relay", key, person, "relay: refused — the person a pass names stands in no group held here")
        }
        (Err(e), _) => {
            tracing::warn!(target: "relay", key, error = %e, "relay: refused — the record could not be read")
        }
    }
    Access::Deny { reason: None }
}

/// The relay `relay` asks for, serving over the data dir `root`, until the
/// [`Server`] is shut down. Refused for its groups when this computer is in
/// none, so it would let nobody through, and when they cannot be read,
/// said as that, never as none (#128).
pub async fn spawn(relay: &Relay, root: &Path) -> Result<Server, Refusal> {
    let unread = |e: RailError| {
        tracing::debug!(target: "relay", root = %root.display(), error = %e, "relay: refused — the groups this computer holds cannot be read");
        Unjudged::GroupsUnread(e.to_string())
    };
    let admits = if relay.open {
        Admits(None)
    } else if namespaces_in(root).map_err(unread)?.is_empty() {
        tracing::debug!(target: "relay", root = %root.display(), "relay: refused — this computer holds no group");
        return Err(Refused::RelayForNobody.into());
    } else {
        let key = identity::load_or_generate_node_key(root)?;
        Admits(Some((root.to_path_buf(), Arc::new(key))))
    };
    let any = |port| SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));
    let (http, tls, quic) = match &relay.at {
        At::Here(addr) => {
            tracing::debug!(target: "relay", %addr, "relay: plain HTTP at the address given, with no certificate");
            (*addr, None, None)
        }
        At::Name { name, staging } => {
            tracing::debug!(target: "relay", name, staging, "relay: HTTPS at the name, its certificate Let's Encrypt's");
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
        let mut share = ClientRateLimit::new(SHARE_RATE);
        share.max_burst_bytes = Some(SHARE_BURST);
        config.limits.client_rx = Some(share);
    }
    tracing::debug!(target: "relay", ?relay, ?admits, "relay: starting");
    config.access = Arc::new(admits);
    let mut server = ServerConfig::default();
    server.relay = Some(config);
    server.quic = quic;
    if matches!(relay.at, At::Name { .. }) {
        // The counters an operator watches, on loopback, where the box
        // alone reads them: two `--at` relays on one computer (the tests',
        // a local network's) would collide on the one port.
        let metrics = SocketAddr::from((Ipv4Addr::LOCALHOST, DEFAULT_METRICS_PORT));
        server.metrics_addr = Some(metrics);
    }
    Server::spawn(server).await.map_err(|e| {
        tracing::debug!(target: "relay", error = %e, "relay: refused — it did not start");
        Unjudged::RelayStart(e.to_string()).into()
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
            "every computer and phone that reaches it, each held to {} Mbit/s, in {} MiB bursts",
            SHARE_RATE.get() * 8 / 1_000_000,
            SHARE_BURST.get() / (1024 * 1024)
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

#[cfg(test)]
#[path = "relay_tests.rs"]
mod tests;
