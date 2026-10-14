// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-reach/src/guest.rs@e1a43bcd2
//! The guest dialer: how a NON-member reaches a lender's mesh over iroh, and
//! the one HTTP bridge every iroh tunnel rides.
//!
//! A borrower is a client of someone else's mesh, like the phone (phase-b-29
//! Q5): it holds a dial string and a bearer, never a membership. So the dial
//! lives here, in the leaf svrn and serve both link, and not in
//! `commonwealth-transport`, which neither may link. In svrngs both halves
//! sit in `svrngs-node`: [`crate::iroh`] dials through [`HttpBridge`] and its
//! acceptor splices through the same [`pump`], so there is still one bridge
//! (phase-b-35).

use std::net::SocketAddr;
use std::sync::Arc;

use iroh::endpoint::presets;
use iroh::endpoint::Builder as EndpointBuilder;
use iroh::{Endpoint, EndpointAddr, RelayUrl, SecretKey};

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
    // Glassbox the egress posture so netops can confirm — from the log,
    // not by inference — which relays this node uses and whether a proxy
    // is engaged. Credentials are redacted.
    log_egress_posture(cfg);
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

/// Read the HTTP(S) proxy from the environment iroh's `proxy_from_env`
/// consults, with any `user:pass@` userinfo redacted. `None` when no
/// proxy is set. Public so the daemon's `doctor` can report the same
/// value it logs.
pub fn configured_proxy_redacted() -> Option<String> {
    // iroh checks HTTPS_PROXY then HTTP_PROXY (and lowercase). Mirror
    // that precedence so what we log is what it will actually use.
    for var in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy"] {
        if let Ok(v) = std::env::var(var) {
            if !v.trim().is_empty() {
                return Some(redact_userinfo(&v));
            }
        }
    }
    None
}

/// Replace a URL's `user:pass@` with `***@` for safe logging. Falls
/// back to the raw string if there's no userinfo to redact.
fn redact_userinfo(url: &str) -> String {
    match (url.find("://"), url.find('@')) {
        (Some(scheme_end), Some(at)) if at > scheme_end + 3 => {
            format!("{}***@{}", &url[..scheme_end + 3], &url[at + 1..])
        }
        _ => url.to_string(),
    }
}

/// One-line, info-level egress summary at endpoint construction:
/// discovery posture, relay set, and proxy (redacted). This is the
/// audit surface a netops team greps to confirm what the node touches.
fn log_egress_posture(cfg: &RelayConfig) {
    let relays = if cfg.relay_urls.is_empty() {
        if cfg.n0_services {
            "n0-default".to_string()
        } else {
            "none (direct-addr only)".to_string()
        }
    } else {
        cfg.relay_urls.join(",")
    };
    tracing::info!(
        target: "transport",
        n0_services = cfg.n0_services,
        loopback_only = cfg.loopback_only,
        relays = %relays,
        proxy = configured_proxy_redacted().as_deref().unwrap_or("none"),
        "iroh egress posture (n0_services=false severs all n0 contact; \
         proxy honored for relay TCP:443, Basic auth only)"
    );
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

/// One client-side tunnel: a localhost `TcpListener` whose accepted
/// connections each become an iroh bi-stream to a fixed peer. Point
/// any plain-TCP client (reqwest, tungstenite) at
/// `http://{local_addr()}` / `ws://{local_addr()}` and it transparently
/// rides QUIC dialed by the peer's Ed25519 key. Dropping the bridge
/// aborts the accept loop.
#[derive(Debug)]
pub struct HttpBridge {
    local_addr: SocketAddr,
    /// Where accepted connections dial, read fresh per connection. The
    /// loopback listener is the bridge's IDENTITY; the peer address it
    /// tunnels to is a mutable attribute — see [`retarget`](Self::retarget).
    #[expect(clippy::disallowed_types, reason = "the bridge's retarget point")]
    target: Arc<std::sync::Mutex<EndpointAddr>>,
    task: tokio::task::JoinHandle<()>,
}

/// How many accept() errors in a row before the loop gives up. Transient
/// errors are the norm; a listener that fails this many times consecutively
/// is broken, and spinning on it would burn a core silently.
const MAX_CONSECUTIVE_ACCEPT_FAILURES: u32 = 32;

impl Drop for HttpBridge {
    fn drop(&mut self) {
        // Traced because dropping this is what makes a live local port stop
        // answering, and callers cache that port's URL. Without this line the
        // only evidence is a connection refused on an address that was
        // serving a minute ago — which reads as the far end's fault.
        tracing::debug!(
            target: "transport",
            addr = %self.local_addr,
            "iroh bridge: dropped — its local port stops answering now"
        );
        self.task.abort();
    }
}

impl HttpBridge {
    /// Bind the localhost listener and start tunneling to `target`
    /// over `alpn`. Dialing is lazy (per accepted TCP connection) and
    /// IS key verification — the QUIC handshake fails unless the
    /// responder holds the private key for `target.id`. Each dial waits
    /// as long as iroh does: a joiner's first dial, the one this serves,
    /// was answered after 3 s (svrngs, events-asleep).
    pub async fn spawn(
        endpoint: Endpoint,
        target: EndpointAddr,
        alpn: &'static [u8],
    ) -> std::io::Result<Self> {
        Self::spawn_preferring(endpoint, target, alpn, None, None).await
    }

    /// [`spawn`](Self::spawn), asking for `preferred` first. A bridge the
    /// transport mints for a peer prefers [`preferred_bridge_port`], so the
    /// same peer answers at the same loopback URL across daemon restarts and a
    /// player or a shim can hold it with no mesh state of its own. When that
    /// port is taken the bind falls back to an ephemeral one and SAYS so —
    /// a substitution named at warn, never a silent different URL (§18.3).
    /// A dial not answered `within`, when given, is given up and its
    /// connection closed, so the caller is told at once (svrngs,
    /// events-asleep: the transport's, [`crate::dial::DIAL_WITHIN`]).
    #[expect(clippy::disallowed_types, reason = "the bridge's retarget point")]
    pub async fn spawn_preferring(
        endpoint: Endpoint,
        target: EndpointAddr,
        alpn: &'static [u8],
        preferred: Option<u16>,
        within: Option<std::time::Duration>,
    ) -> std::io::Result<Self> {
        let listener = match preferred {
            None => tokio::net::TcpListener::bind("127.0.0.1:0").await?,
            Some(port) => match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::warn!(
                        target: "transport",
                        preferred = port,
                        peer = %target.id,
                        error = %e,
                        "iroh bridge: the port derived from this peer's key is taken — \
                         substituting an ephemeral one, so this peer's URL differs from its usual"
                    );
                    tokio::net::TcpListener::bind("127.0.0.1:0").await?
                }
            },
        };
        let local_addr = listener.local_addr()?;
        let peer_label = target.id.to_string();
        let target = Arc::new(std::sync::Mutex::new(target));
        let target_slot = Arc::clone(&target);
        let task = tokio::spawn(async move {
            // Consecutive accept failures. An accept error is almost always
            // transient (ECONNABORTED on a peer that hung up mid-handshake,
            // EMFILE under fd pressure, EINTR), and the old code treated
            // EVERY one as terminal: `let Ok(..) = accept().await else
            // { break }`, with no tracing on the branch. A single transient
            // error therefore killed the port permanently and SILENTLY.
            //
            // Observed live 2026-08-28: a guest tunnel opened at 21:11:10,
            // served a `/v1/models` listing, and by 21:12:39 the port was
            // refusing connections with nothing in the log to say why or
            // when. Every surface above it kept advertising the dead address
            // (`StoredGuestLink` caches the base URL), so the failure
            // presented as "the lender revoked your grant".
            let mut consecutive_failures: u32 = 0;
            loop {
                let (tcp, _) = match listener.accept().await {
                    Ok(accepted) => {
                        consecutive_failures = 0;
                        accepted
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        tracing::warn!(
                            target: "transport",
                            peer = %peer_label,
                            addr = %local_addr,
                            error = %e,
                            kind = ?e.kind(),
                            consecutive_failures,
                            "iroh bridge: accept failed — the tunnel stays open"
                        );
                        // Bounded, so a genuinely broken listener cannot spin
                        // a core forever. The bound is loud when it fires:
                        // this port going quiet is what the caller sees, and
                        // it must never be something they have to infer.
                        if consecutive_failures >= MAX_CONSECUTIVE_ACCEPT_FAILURES {
                            tracing::error!(
                                target: "transport",
                                peer = %peer_label,
                                addr = %local_addr,
                                error = %e,
                                consecutive_failures,
                                "iroh bridge: accept loop EXITING after repeated failures — \
                                 this local port stops answering and any cached base URL \
                                 pointing at it is now dead"
                            );
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        continue;
                    }
                };
                // Nagle on this loopback hop stacks with the peer's delayed
                // ACK into ~40 ms stalls per direction on request/response
                // traffic (measured: 82 ms added to a 16 KB round-trip).
                // The tunnel must be latency-transparent — disable it.
                tcp.set_nodelay(true).ok();
                let endpoint = endpoint.clone();
                // Read the CURRENT target per connection: a retarget between
                // accepts must take effect on the very next dial.
                let target = target_slot
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clone();
                let peer_label = peer_label.clone();
                tokio::spawn(async move {
                    let connect = endpoint.connect(target, alpn);
                    let dialled = match within {
                        Some(within) => tokio::time::timeout(within, connect).await,
                        None => Ok(connect.await),
                    };
                    let conn = match dialled {
                        Ok(Ok(c)) => c,
                        Err(_) => {
                            // svrngs (events-asleep): an asleep computer never
                            // answers, and iroh gave up only after 30 s.
                            tracing::debug!(target: "transport", peer = %peer_label, ?within,
                                "iroh bridge: the dial was not answered in time, so the computer is taken as asleep");
                            return;
                        }
                        Ok(Err(e)) => {
                            // The ALPN is the whole diagnosis for an "error 120:
                            // peer doesn't support any known protocol" reject —
                            // without it you cannot tell a peer that lacks a
                            // ggml rpc-server (no cwth/rpc/0) from one that is
                            // unreachable, and both surface as "dial failed".
                            tracing::warn!(
                                target: "transport",
                                peer = %peer_label,
                                alpn = %String::from_utf8_lossy(alpn),
                                error = %e,
                                "iroh bridge: dial failed"
                            );
                            return;
                        }
                    };
                    let (send, recv) = match conn.open_bi().await {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::warn!(
                                target: "transport",
                                peer = %peer_label,
                                error = %e,
                                "iroh bridge: open_bi failed"
                            );
                            return;
                        }
                    };
                    pump(tcp, send, recv, PumpSide::Bridge, peer_label.clone(), alpn).await;
                });
            }
        });
        Ok(Self {
            local_addr,
            target,
            task,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Point subsequent dials at `target` WITHOUT rebinding the loopback
    /// listener, so the local port survives a peer's dial-info change.
    ///
    /// That port is not an implementation detail: it is the address handed
    /// to plain-TCP clients that cannot be told to re-resolve — most
    /// sharply ggml's rpc-server list, which the mesh's RPC discovery
    /// advertises as the worker's endpoint STRING. Rebuilding the bridge
    /// minted a new ephemeral port on every gossiped address change, so a
    /// stable peer read downstream as a stream of different workers
    /// (observed 2026-07-25: 34207 → 40043 → 39419 → 34133 → 40021 for one
    /// unmoved Mac). Identity is the bridge; the peer address is an
    /// attribute of it.
    pub fn retarget(&self, target: EndpointAddr) {
        *self.target.lock().unwrap_or_else(|e| e.into_inner()) = target;
    }
}

/// Copy with a byte count that survives an error on the write side:
/// `tokio::io::copy` returns `Err` with the partial count lost, and a far
/// end that RESETS the stream after reading would otherwise look like
/// "the client sent nothing" — exactly the case the zero-answer warn
/// exists to name.
async fn copy_count<R, W>(mut r: R, mut w: W) -> u64
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut buf = [0u8; 8192];
    let mut total: u64 = 0;
    loop {
        match r.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                if w.write_all(&buf[..n]).await.is_err() {
                    break;
                }
                total += n as u64;
            }
            Err(_) => break,
        }
    }
    let _ = w.shutdown().await;
    total
}

/// Which end of a tunnel `pump` is splicing — the warn's direction and
/// wording depend on it. On the BRIDGE side the client's request flows
/// tcp→iroh and the answer iroh→tcp. (Upstream's ACCEPTOR side, the
/// mirror, went in lean-3: svrngs's acceptor serves a router, not a splice.)
#[derive(Clone, Copy)]
pub enum PumpSide {
    Bridge,
}

/// Splice one TCP connection and one iroh bi-stream, both ways. The bridge
/// calls this, and the bridge's tests splice their far end with it.
pub async fn pump(
    tcp: tokio::net::TcpStream,
    mut send: iroh::endpoint::SendStream,
    mut recv: iroh::endpoint::RecvStream,
    side: PumpSide,
    peer: String,
    alpn: &[u8],
) {
    let (mut tcp_r, mut tcp_w) = tcp.into_split();
    // Bytes counted per direction so a tunnel that swallows a request can be
    // NAMED. A far side that accepts the stream, reads the bytes, and closes
    // (or resets) without answering is the silent signature of a down origin
    // (observed 2026-09-22: a media bridge served every caller "empty reply
    // from server" with zero log lines on EITHER side — the dial succeeds,
    // so the existing dial/open_bi warns never fire).
    let (request, answer) =
        tokio::join!(async { copy_count(&mut tcp_r, &mut send).await }, async {
            copy_count(&mut recv, &mut tcp_w).await
        },);
    match side {
        PumpSide::Bridge => {
            // request = client→peer, answer = peer→client.
            if request > 0 && answer == 0 {
                tracing::warn!(
                    target: "transport",
                    peer = %peer,
                    alpn = %String::from_utf8_lossy(alpn),
                    request_bytes = request,
                    "iroh bridge: the peer accepted the stream but answered nothing — \
                     its origin or forward is down on the far side"
                );
            }
        }
    }
}
