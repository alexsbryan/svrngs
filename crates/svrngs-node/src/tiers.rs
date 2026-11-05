// SPDX-License-Identifier: AGPL-3.0-or-later
//! Where this computer meets its groups, in tiers (relay fallbacks; svrn-docs
//! `rings/current/GROUP_RELAYS.md` §7, the operator's "Tiers", 2026-11-02,
//! corrected by the relay-fallback charter: a browser may reach this
//! computer only through the build's fallback while it answers, so that is
//! the tier held): the build's fallback alone while it answers, and the
//! groups' own relays alone when it does not, or the build's while a group
//! it holds names none of its own, since one endpoint, so one home, serves
//! every group. svrngs-core's `Meeting::decide` is the one decision;
//! [`Host::decide`] is the one place it changes the endpoint's relays
//! (iroh-1.0.2 `Endpoint::insert_relay`, `remove_relay`, endpoint.rs:982,
//! :996).
//!
//! Why svrngs asks the fallback itself ([`Host::check`]): iroh's report
//! stops probing once a relay answers (iroh-1.0.2 `net_report/probes.rs`),
//! so a relay that went quiet and came back is never noticed by iroh alone;
//! and the tier must follow the answer, not iroh's own latency choice,
//! since the link's routes are drawn from it.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use svrngs_core::relay::{Meeting, Tier, Url};

use crate::iroh::RelayUrl;
use crate::node::Host;

/// How often svrngs asks the build's fallback whether it answers: half
/// iroh's own re-check, every 20 to 26 s (iroh-1.0.2 socket.rs:2004-2018),
/// so a tier is decided before iroh's next report, and a move either way
/// waits at most one ask and that report. One connection per relay each
/// time, on the network the fallback's.
pub const ASKED_EVERY: Duration = Duration::from_secs(10);

/// How long one relay has to answer: iroh's own probe timeout,
/// `PROBES_TIMEOUT` (iroh-1.0.2 net_report/defaults.rs:23), so a relay
/// svrngs calls silent is one iroh's report would too.
pub const ANSWERS_WITHIN: Duration = Duration::from_secs(3);

/// How long this endpoint has, at each ask, to be registered at the
/// build's fallback it holds: iroh's own report timeout,
/// `NET_REPORT_TIMEOUT` (iroh-1.0.2 lib.rs:292, net_report/defaults.rs:14),
/// which iroh advises waiting `online` for (endpoint.rs:1316-1319).
pub const REGISTERED_WITHIN: Duration = Duration::from_secs(iroh::NET_REPORT_TIMEOUT);

/// What this computer's endpoint holds now, and from what: where its
/// groups meet, as last read from their records ([`Host::refresh`]), the
/// tier, the relays, whether the build's fallback answered at the last ask,
/// and the relays a `new` or `join` asked for before its group is kept.
#[derive(Debug, Clone)]
pub(crate) struct Held {
    meeting: Meeting,
    tier: Tier,
    relays: BTreeSet<Url>,
    answers: bool,
    asked: BTreeSet<Url>,
}

impl Held {
    /// What an endpoint built for `meeting`'s groups and the relays
    /// `asked` holds, the build's fallback taken to answer until the first
    /// ask.
    pub(crate) fn built(meeting: Meeting, asked: BTreeSet<Url>) -> (Self, BTreeSet<Url>) {
        let (tier, relays) = meeting.decide(true);
        let held = Self {
            meeting,
            tier,
            relays: relays.clone(),
            answers: true,
            asked,
        };
        (held, relays)
    }
}

impl Host {
    /// Where every group held meets read again from their records, joined
    /// with the relays a `new` or `join` asked for: what the one decision
    /// decides over until the next read. Read at each round's start and as
    /// a group is kept, never on an ask, which reads nothing.
    pub(crate) fn refresh(&self) {
        let groups = self.rail.namespaces().inspect_err(|e| tracing::warn!(target: "tiers", error = %e, "tiers: the groups held cannot be listed, so this computer meets at the relays asked, else the build's"));
        let groups = groups.unwrap_or_default();
        self.held.send_modify(|h| {
            h.meeting = everywhere(&self.rail, &groups, &self.build, &h.asked);
        });
    }

    /// The one decision applied (§7): the endpoint given the relays the
    /// tier needs and relieved of the rest, given whether the build's
    /// fallback `answers` (the last ask's, when `None`). Each change of
    /// tier said.
    pub(crate) async fn decide(&self, answers: Option<bool>) {
        let (mut add, mut drop, mut was) = (Vec::new(), Vec::new(), Tier::Own);
        self.held.send_modify(|held| {
            let answers = answers.unwrap_or(held.answers);
            let (tier, relays) = held.meeting.decide(answers);
            add.extend(relays.difference(&held.relays).cloned());
            drop.extend(held.relays.difference(&relays).cloned());
            was = held.tier;
            (held.tier, held.relays, held.answers) = (tier, relays, answers);
        });
        let tier = self.held.borrow().tier;
        if tier != was {
            tracing::debug!(target: "tiers", from = ?was, to = ?tier, "tiers: this computer meets its groups at another tier now");
        }
        for relay in add.into_iter().map(RelayUrl::from) {
            tracing::debug!(target: "tiers", %relay, ?tier, "tiers: a relay given to this computer's endpoint");
            self.endpoint
                .insert_relay(relay.clone(), Arc::new(relay.into()))
                .await;
        }
        for relay in drop.into_iter().map(RelayUrl::from) {
            tracing::debug!(target: "tiers", %relay, ?tier, "tiers: a relay taken from this computer's endpoint");
            self.endpoint.remove_relay(&relay).await;
        }
    }

    /// The relays `new --relay` or a link asks for, held as a group's own
    /// before the group is kept, and decided at once.
    pub async fn meet_at(&self, own: &[Url]) {
        self.held
            .send_modify(|held| held.asked.extend(own.iter().cloned()));
        self.refresh();
        self.decide(None).await;
    }

    /// Asks the build's fallback whether it answers, every [`ASKED_EVERY`],
    /// and decides on each answer, until the node stops: the fallback is
    /// the one tier a browser may reach this computer through, so its
    /// answer, not the own tier's, moves the meeting. At the fallback, the
    /// answer is whether this endpoint is [`Self::registered`] there, since
    /// a relay may take a connection and register nobody (review F1).
    pub(crate) async fn check(self: Arc<Self>) {
        let mut every = tokio::time::interval(ASKED_EVERY);
        loop {
            every.tick().await;
            let Held { meeting, tier, .. } = self.held.borrow().clone();
            let build = meeting.build().clone();
            let mut asks = tokio::task::JoinSet::new();
            for relay in build.clone().into_iter().filter(|_| tier != Tier::Fallback) {
                asks.spawn(async move { answers(&relay).await });
            }
            // With no build tier, nothing to answer: the tier is the
            // groups' own whatever, and a group asked for next starts at
            // the build's until the first ask says otherwise.
            let answered = match tier {
                Tier::Fallback => self.registered(&build).await,
                _ => build.is_empty() || asks.join_all().await.into_iter().any(|a| a),
            };
            tracing::debug!(target: "tiers", build = build.len(), ?tier, answered, "tiers: whether the build's fallback answers, at the fallback whether this endpoint is registered there");
            self.decide(Some(answered)).await;
        }
    }

    /// Whether this endpoint's home relay is one of `build`'s and connected,
    /// waited for up to [`REGISTERED_WITHIN`]: registered there, its
    /// handshake done, not merely chosen (iroh-1.0.2
    /// `Endpoint::home_relay_status`, endpoint.rs:1384, and
    /// `RelayStatus::is_connected`, :1906, what `Endpoint::online` reads).
    async fn registered(&self, build: &BTreeSet<Url>) -> bool {
        use n0_future::StreamExt as _;
        let mut homes = iroh::Watcher::stream(self.endpoint.home_relay_status());
        let at = |h: &iroh::endpoint::RelayStatus| {
            h.is_connected() && build.contains(&h.url().clone().into())
        };
        let registered =
            tokio::time::timeout(REGISTERED_WITHIN, homes.any(|homes| homes.iter().any(at))).await;
        tracing::debug!(target: "tiers", ?registered, "tiers: whether this endpoint is registered at the build's fallback");
        registered.unwrap_or(false)
    }

    /// Where a group made here with no relay named meets: where this
    /// computer is met now, else, before it has reached a relay, the own
    /// relays its endpoint holds, as the first relay configured once was;
    /// `Host::keep_group` leaves the build's out.
    pub fn made_at(&self) -> Vec<Url> {
        let held = self.held.borrow();
        let own = held
            .relays
            .iter()
            .filter(|r| held.meeting.own().contains(*r));
        let at = self
            .home()
            .map_or_else(|| own.cloned().collect(), |h| vec![h]);
        tracing::debug!(target: "tiers", ?at, "tiers: where a group made here meets, where this computer is met now, else the own relays it holds");
        at
    }

    /// This computer's home relay now, if it has reached one.
    pub(crate) fn home(&self) -> Option<Url> {
        self.endpoint
            .addr()
            .relay_urls()
            .next()
            .cloned()
            .map(Into::into)
    }

    /// The tier this computer meets `group` at, when it is the build's
    /// fallback and the group names relays of its own: the home page says
    /// why (relay fallbacks). `None` otherwise.
    pub(crate) fn at_fallback(&self, group: &str) -> Option<Tier> {
        let (home, tier) = (self.home(), self.held.borrow().tier);
        let own = self.meeting(group).own().clone();
        let at = home
            .as_ref()
            .is_some_and(|h| tier != Tier::Own && !own.is_empty() && !own.contains(h));
        tracing::debug!(target: "tiers", group, home = ?home.as_ref().map(Url::as_str), ?tier, at, "tiers: whether this computer meets the group at its fallback");
        at.then_some(tier)
    }

    /// Each move of this computer's home said, until the node stops, with
    /// the groups it meets at their fallback. The record hears of it at the
    /// next round's start ([`Self::say_relay`]); a member dials every relay
    /// of both tiers meanwhile. Saying it here wrote the keeper's cutovers
    /// as a node first came online, before acts a test held for them
    /// (browser/carried.rs `an_entry_another_s_computer_wrote_says_whose`).
    pub(crate) async fn moved(self: Arc<Self>) {
        use n0_future::StreamExt as _;
        let mut addrs = iroh::Watcher::stream(self.endpoint.watch_addr());
        let mut home = None;
        while let Some(addr) = addrs.next().await {
            let now = addr.relay_urls().next().cloned().map(Url::from);
            if now == home {
                continue;
            }
            let held = self.rail.namespaces().unwrap_or_default();
            let fallback: Vec<&String> = held
                .iter()
                .filter(|g| self.at_fallback(g).is_some())
                .collect();
            tracing::debug!(target: "tiers", from = ?home.as_ref().map(Url::as_str), to = ?now.as_ref().map(Url::as_str), ?fallback, "tiers: this computer meets its groups at another relay now, the groups it meets at their fallback named");
            home = now;
        }
    }
}

/// Where `groups` held under `rail` meet, joined, with a meeting of the
/// relays `asked` when any; with neither, the build's tier alone, as a
/// group naming none of its own.
pub(crate) fn everywhere(
    rail: &crate::RingRail,
    groups: &[String],
    build: &[Url],
    asked: &BTreeSet<Url>,
) -> Meeting {
    let asked = (!asked.is_empty()).then(|| Meeting::of(asked.iter().cloned(), build));
    let each = groups.iter().map(|g| crate::group::meeting(rail, g, build));
    each.chain(asked)
        .reduce(Meeting::join)
        .unwrap_or_else(|| Meeting::of([], build))
}

/// Whether `relay` answers within [`ANSWERS_WITHIN`]: a plain-HTTP relay,
/// as `svrngs relay --at` serves, its probe `GET /ping` with 200
/// (iroh-relay-1.0.2 http.rs:15, server.rs:738); one over HTTPS, a
/// connection taken at its address, since the node holds no TLS client of
/// its own to ask the probe through.
async fn answers(relay: &Url) -> bool {
    let host = relay
        .host_str()
        .unwrap_or_default()
        .trim_matches(['[', ']'])
        .to_string();
    let asked = async {
        let mut at =
            tokio::net::TcpStream::connect((host.as_str(), relay.port_or_known_default()?))
                .await
                .ok()?;
        if relay.scheme() != "http" {
            return Some(true);
        }
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        let ask = format!("GET /ping HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
        at.write_all(ask.as_bytes()).await.ok()?;
        let mut said = [0u8; 12];
        at.read_exact(&mut said).await.ok()?;
        Some(said.ends_with(b" 200"))
    };
    let said = tokio::time::timeout(ANSWERS_WITHIN, asked).await;
    tracing::debug!(target: "tiers", %relay, ?said, "tiers: whether a group's own relay answers");
    matches!(said, Ok(Some(true)))
}
