// SPDX-License-Identifier: AGPL-3.0-or-later
//! Where this computer meets its groups, in tiers (relay fallbacks; svrn-docs
//! `rings/current/GROUP_RELAYS.md` §7, the operator's "Tiers", 2026-11-02):
//! its groups' own relays while any of them answers, and the build's relay
//! only when none does, or while a group it holds names none of its own,
//! since one endpoint, so one home, serves every group. svrngs-core's
//! `Meeting::decide` is the one decision; [`Host::decide`] is the one place
//! it changes the endpoint's relays (iroh-1.0.2 `Endpoint::insert_relay`,
//! `remove_relay`, endpoint.rs:982, :996).
//!
//! Why svrngs asks the own relays itself ([`Host::check`]): an
//! `svrngs relay --at` relay serves no QUIC address discovery, and iroh's
//! report stops its other probes once a relay's QUIC probe answers
//! (iroh-1.0.2 `net_report/probes.rs:83-89`), so with the build's relay
//! held iroh never sees the own relays come back. A dial still reaches
//! both tiers, whichever this computer holds: iroh dials a member at a relay
//! its endpoint does not hold (`tests/relay_tiers.rs`).

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use svrngs_core::relay::{Meeting, Tier, Url};

use crate::iroh::RelayUrl;
use crate::node::Host;

/// How often svrngs asks its groups' own relays whether they answer: half
/// iroh's own re-check, every 20 to 26 s (iroh-1.0.2 socket.rs:2004-2018),
/// so a tier is decided before iroh's next report, and a move either way
/// waits at most one ask and that report. One connection per own relay
/// each time, on the local network the hack night's.
pub const ASKED_EVERY: Duration = Duration::from_secs(10);

/// How long one relay has to answer: iroh's own probe timeout,
/// `PROBES_TIMEOUT` (iroh-1.0.2 net_report/defaults.rs:23), so a relay
/// svrngs calls silent is one iroh's report would too.
pub const ANSWERS_WITHIN: Duration = Duration::from_secs(3);

/// What this computer's endpoint holds now, and from what: where its
/// groups meet, as last read from their records ([`Host::refresh`]), the
/// tier, the relays, whether the own tier answered at the last ask, and the
/// relays a `new` or `join` asked for before its group is kept.
#[derive(Debug, Clone)]
pub(crate) struct Held {
    meeting: Meeting,
    tier: Tier,
    relays: BTreeSet<Url>,
    answers: bool,
    asked: BTreeSet<Url>,
}

impl Held {
    /// What an endpoint built for `meeting`'s groups holds, its own tier
    /// taken to answer until the first ask.
    pub(crate) fn built(meeting: Meeting) -> (Self, BTreeSet<Url>) {
        let (tier, relays) = meeting.decide(true);
        let asked = BTreeSet::new();
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
    /// tier needs and relieved of the rest, given whether the own tier
    /// `answers` (the last ask's, when `None`). Each change of tier said.
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

    /// Asks the own tier whether it answers, every [`ASKED_EVERY`], and
    /// decides on each answer, until the node stops.
    pub(crate) async fn check(self: Arc<Self>) {
        let mut every = tokio::time::interval(ASKED_EVERY);
        loop {
            every.tick().await;
            let own = self.held.borrow().meeting.own().clone();
            let mut asks = tokio::task::JoinSet::new();
            for relay in own.clone() {
                asks.spawn(async move { answers(&relay).await });
            }
            // With no own tier, nothing to answer: the tier is the build's
            // whatever, and a group asked for next starts at its own.
            let answered = own.is_empty() || asks.join_all().await.into_iter().any(|a| a);
            tracing::debug!(target: "tiers", own = own.len(), answered, "tiers: whether any of the groups' own relays answers");
            self.decide(Some(answered)).await;
        }
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

    /// The tier this computer meets `group` at, when it is the build's and
    /// the group names relays of its own that this computer's home is none
    /// of: the home page says why (relay fallbacks). `None` otherwise.
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
    /// the groups it meets at their fallback; then said in the record, so a
    /// member dials it there ([`Self::say_relay`]).
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
            self.say_relay();
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
