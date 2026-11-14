// SPDX-License-Identifier: AGPL-3.0-or-later
//! Where this computer meets its groups, in tiers (relay fallbacks; svrn-docs
//! `rings/current/GROUP_RELAYS.md` §7, the operator's "Tiers", 2026-11-02,
//! corrected by the relay-fallback charter: a browser may reach this
//! computer only through the build's fallback while it answers, so that is
//! the tier held): the build's fallback alone while it answers, and the
//! groups' own relays alone when it does not, one endpoint, so one home,
//! serving every group; the build's whatever answers only when no group
//! names one. svrngs-core's `Meeting::decide` is the one decision;
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

/// How many asks in a row find this endpoint unregistered at the build's
/// tier before it is left, once reached: a loss shorter than two asks and
/// their waits, 25 s, is kept, longer than iroh's cap on its wait to dial a
/// relay again, 16 s (iroh-1.0.2 socket/transports/relay/actor.rs:353),
/// though its jitter may double one wait (backon-1.6.0
/// backoff/exponential.rs:233). One home, read by the tests (review F2).
pub const LEFT_AFTER: u32 = 3;

/// The most asks a move to the build's tier is held off for after one that
/// never registered there: 30, five minutes of [`ASKED_EVERY`]. The hold
/// doubles, 1, 2, 4 and on to this, from each such move, and ends at the
/// first registration, so a relay that answers the ask and registers no
/// one, as a captive portal does, is not gone back to every 20 s (review
/// F1's residual: at its own relay 18 s of 60).
pub const HELD_AT_MOST: u32 = 30;

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
    pub(crate) asked: BTreeSet<Url>,
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
        let add: Vec<_> = add.into_iter().map(RelayUrl::from).collect();
        let drop: Vec<_> = drop.into_iter().map(RelayUrl::from).collect();
        tracing::debug!(target: "tiers", ?add, ?drop, ?tier, "tiers: relays given to this computer's endpoint, and taken from it");
        self.transport.relays(add, drop).await;
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
    /// answer, not the own tier's, moves the meeting. At the build's tier, the
    /// answer is whether this endpoint is [`Self::registered`] there, since
    /// a relay may take a connection and register nobody (review F1), and
    /// one reached is left only after [`LEFT_AFTER`] misses (review F2).
    pub(crate) async fn check(self: Arc<Self>) {
        let (mut every, mut misses, mut reached) = (tokio::time::interval(ASKED_EVERY), 0, false);
        let (mut wait, mut held_for) = (0, 0);
        loop {
            every.tick().await;
            let Held { meeting, tier, .. } = self.held.borrow().clone();
            let build = meeting.build().clone();
            let holding = tier == Tier::Own && held_for > 0;
            if holding {
                held_for -= 1;
                tracing::debug!(target: "tiers", misses, wait, left = held_for, "tiers: the build's fallback not asked — a move there registered nowhere, so moves are held off");
            }
            let ask = tier == Tier::Own && !holding;
            // With no build tier, nothing to answer: the tier is the
            // groups' own whatever, and a group asked for next starts at
            // the build's until the first ask says otherwise.
            let answered = build.is_empty()
                || match tier {
                    Tier::Own => ask && any_answers(&build).await,
                    _ => self.registered(&build).await,
                };
            misses = if answered { 0 } else { misses + 1 };
            // A move to the build's that never registered holds the next
            // off, twice as long each time; a registration ends the hold.
            if tier != Tier::Own && !answered && !reached {
                wait = (wait * 2).clamp(1, HELD_AT_MOST);
                held_for = wait;
            } else if tier != Tier::Own && answered {
                wait = 0;
            }
            // Kept only once reached: a fallback this computer reached and
            // lost for a moment is held; one it never reached is left at once,
            // as is one a person's press left while this ask ran.
            let pressed = self.held.borrow().tier != tier;
            let kept = !pressed && tier != Tier::Own && !answered && reached && misses < LEFT_AFTER;
            reached = tier != Tier::Own && (answered || kept);
            tracing::debug!(target: "tiers", build = build.len(), ?tier, answered, misses, reached, pressed, kept, wait, "tiers: whether the build's fallback answers, at the build's whether this endpoint is registered there; a miss short of LEFT_AFTER in a row keeps one this computer reached");
            self.decide(Some(answered || kept)).await;
        }
    }

    /// The home page's button (KEEP_GOING_AT_HOME 7), for someone who cannot
    /// wait out [`LEFT_AFTER`]: the build's fallback asked once now and, when
    /// it does not answer, left at once for the groups' own, and said home
    /// only once this endpoint is [`Self::registered`] there. What happened,
    /// as the page's address says it. An answer moves nobody: a blip decides
    /// nothing here, and [`Self::check`] goes on as it was.
    pub(crate) async fn check_now(&self) -> &'static str {
        let meeting = self.held.borrow().meeting.clone();
        let (build, own) = (meeting.build(), meeting.own());
        let answered = build.is_empty() || any_answers(build).await;
        self.decide(Some(answered).filter(|a| !a)).await;
        let moved = !answered && self.registered(own).await;
        tracing::debug!(target: "tiers", answered, moved, "tiers: a person asked now whether the build's fallback answers, and left it if not");
        match (answered, moved) {
            (true, _) => "stayed",
            (_, true) => "moved",
            _ => "nowhere",
        }
    }

    /// Whether this endpoint's home relay is one of `relays` and connected,
    /// waited for up to [`REGISTERED_WITHIN`]: registered there, its
    /// handshake done, not merely chosen (iroh-1.0.2
    /// `Endpoint::home_relay_status`, endpoint.rs:1384, and
    /// `RelayStatus::is_connected`, :1906, what `Endpoint::online` reads).
    async fn registered(&self, relays: &BTreeSet<Url>) -> bool {
        use n0_future::StreamExt as _;
        let mut homes = self.transport.homes();
        let at = |h: &iroh::endpoint::RelayStatus| {
            h.is_connected() && relays.contains(&h.url().clone().into())
        };
        let registered =
            tokio::time::timeout(REGISTERED_WITHIN, homes.any(|homes| homes.iter().any(at))).await;
        tracing::debug!(target: "tiers", ?registered, "tiers: whether this endpoint is registered at the relays asked of");
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
    pub fn home(&self) -> Option<Url> {
        self.transport
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
        let mut addrs = self.transport.addrs();
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

/// Whether any of `relays` answers ([`answers`]), all asked at once.
async fn any_answers(relays: &BTreeSet<Url>) -> bool {
    let mut asks = tokio::task::JoinSet::new();
    for relay in relays.clone() {
        asks.spawn(async move { answers(&relay).await });
    }
    asks.join_all().await.contains(&true)
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
