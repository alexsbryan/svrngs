// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_sync.rs@e1a43bcd2
//! The ring journal's own replication loop — slower than gossip, and by
//! digest rather than by snapshot.
//!
//! Moved here from sovereign-mesh (phase-b pb-rails-parity). cw-rails runs
//! the round over its own journals (upstream's `RailsDaemon` is a
//! `RingSyncHost`) and answers `/internal/ring/sync` with [`answer`]. The
//! daemon's copy retired with pb-mesh-exit-transport: this is the one round.
//! (svrngs: forked in U1 without `RailsDaemon`; the round runs in the node,
//! [`crate::node`], and reads its [`Host`] directly — with one host, the
//! `RingSyncHost` seam went, DESIGN §8.)
//!
//! # Why this is a digest and not a snapshot
//!
//! `gossip.rs` used to carry a fourth step that shipped a **full mesh-store
//! snapshot to every online peer every ten seconds** — 8,640 rounds a day. A
//! household writes on the order of 3,500 journal ops a year, call it 1.5 MB,
//! so riding that push would have cost roughly **246 GB/day of egress per
//! node** and taxed every other namespace on the same body forever. Bandwidth
//! is the binding constraint for this feature and it binds on day one.
//!
//! So: a sixty-second cadence (ample for money), and an exchange whose
//! request is a ~600-byte digest rather than the journal.
//!
//! At cw-lift rung 2e that argument stopped being a comparison and became the
//! whole story: the push, its `/internal/app/state` route and the
//! event-driven `broadcast_now` beside it are deleted, `MeshStore` is a
//! projection of these journals on both sides of the wire, and this is the ONE
//! sender of replicated state in the workspace. The nudge is what keeps that
//! affordable for a latency-sensitive writer — `RailsDaemon::ring_nudge`
//! starts a round when the append door signs onto a journal or a member comes
//! back Online, so the sixty seconds
//! is a ceiling on idleness rather than on a write. (svrngs: the record
//! API's append door nudges, [`append`]; the redeem route once it writes
//! an `Admit`; and the member door when a pass's signer is unknown to it,
//! [`AskRound`], waiting on the round it starts.)
//!
//! # The exchange
//!
//! Two calls per chunk per peer per namespace, both idempotent:
//!
//! 1. **`{digest_mine, ops: []}`** → the peer ingests nothing, answers with
//!    its own digest and one budget's worth of the ops our digest says we
//!    lack. We ingest those.
//! 2. **`{digest_mine', ops: one budget of what_they_lack}`** → computed from
//!    the digest they just gave us. They ingest; we read the count back.
//!
//! A dropped call costs one round of convergence and never a duplicate entry,
//! because ingest is keyed on the content-addressed op id.
//!
//! # …and it repeats, because one body is not the unit of convergence
//!
//! Both `ops` arrays are stopped at
//! `RING_SYNC_OPS_BUDGET_BYTES`,
//! and [`exchange`] repeats the pair until neither side moves. Nothing on the
//! wire changed shape for that: the exchange was always idempotent, so a
//! partial one is safe and the second half is just the next call.
//!
//! Before the budget, one exchange carried the whole selection and the
//! receiver's `DefaultBodyLimit` refused it at ~9,599 ops of the measured
//! 594-byte fixture. The refusal was answered at the extractor, so the
//! handler never ran and its gauge could not fire; this loop mapped the 413
//! to a string, logged it at DEBUG and counted the peer as UNREACHABLE. A
//! peer that had been refused the journal then reported zero ops, zero gaps
//! and a complete ring.
//!
//! **Why repeating terminates** (and it must, or a ceiling that was at least
//! measurable becomes a silent spin): a chunk's first op is always one the
//! receiver provably lacks — a contiguous mark of `n` means they do not hold
//! `n + 1`, and the selection is ordered and filtered so `n + 1` is the
//! lowest element it can yield. Every non-empty chunk therefore moves the
//! receiver's mark. Holdings are finite, so the loop runs out of work; and
//! [`MAX_CHUNKS_PER_EXCHANGE`] bounds it anyway, because a bound you can name
//! beats an argument you have to trust.
//!
//! # Everyone republishes everything they hold
//!
//! Call 2 sends what the PEER lacks out of everything WE hold, with no filter
//! on who authored it. Three failure modes die at once: the author's node
//! dying before anyone else came online, a peer restart wiping in-memory
//! buffers, and a housemate leaving the ring with half the journal. It is also
//! why there is no own-origin skip to get wrong here — the mesh store's
//! `origin` names the last republisher rather than the author, and this path
//! has no origin field at all because the op carries its author in a
//! signature.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::peer_wire::{RingSyncRequest, RingSyncResponse, RING_SYNC_OPS_BUDGET_BYTES};
use crate::{RailError, RingRail};
use n0_future::task::AbortOnDropHandle;
use tokio::sync::{watch, Notify};
use tracing::{debug, info, warn};

mod journal;
use journal::OnlinePeer;
pub use journal::RoundMembers;

use crate::iroh::key_actor;
use crate::node::Host;
use axum::http::StatusCode;

/// Money does not need ten-second convergence, and the bandwidth argument in
/// the module docs says it must not have it.
pub const DEFAULT_RING_SYNC_INTERVAL: Duration = Duration::from_secs(60);

/// How many chunked round-trips one [`exchange`] makes before it hands the
/// round back.
///
/// A **safety valve, not a throughput knob.** The termination argument in the
/// module docs is the real reason the loop stops; this is what makes the loop
/// stop even if that argument is one day wrong, which is the whole difference
/// between a ceiling that is measurable and a sync loop that silently spins.
///
/// Sixteen chunks of the four-megabyte budget is 64 MiB per peer per
/// namespace per round — about 67,800 ops of the 594-byte fixture the ceiling
/// was measured against (~990 B/op since svrngs's U2 lengthened the line;
/// 76,800 at upstream's ~873), or nineteen years of the ~3,500 ops/year household
/// journal the module docs price, moved in one sixty-second round. Tripping
/// it costs nothing but time: every op ingested is already on disk, so the
/// next round resumes where this one stopped.
pub const MAX_CHUNKS_PER_EXCHANGE: usize = 16;

/// Handle to the spawned loop. Aborts the task on drop, matching
/// the daemon's `GossipHandle` so the daemon tears both
/// down the same way.
pub struct RingSyncHandle {
    _task: AbortOnDropHandle<()>,
}

/// What one round moved. Returned rather than only logged so a test can
/// assert convergence instead of asserting on log lines.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RoundOutcome {
    pub namespaces: usize,
    pub peers_reached: usize,
    pub peers_unreachable: usize,
    /// Peers that ANSWERED and refused our body as too large.
    ///
    /// Counted apart from `peers_unreachable` because they are different
    /// facts, and collapsing them is what kept the convergence ceiling
    /// invisible: a peer saying "that body is over my limit" is a peer that
    /// received the request, and filing it as one that could not be dialled
    /// reports the wrong problem to whoever goes looking (ARCH §18.3).
    ///
    /// With the exchange budgeted against the same limit, this reads zero
    /// against peers on this build. A non-zero count means a peer's limit is
    /// lower than this build's budget — which is worth seeing, and used not
    /// to be visible at any level.
    pub peers_refused: usize,
    pub ops_pulled: usize,
    pub ops_pushed: usize,
}

/// How many rounds have begun and how many have ended, and how many of
/// their exchanges a member has answered, written by the loop alone: rounds
/// run one at a time, so the last ended is the `ended`th.
#[derive(Debug, Default, Clone, Copy)]
pub struct Progress {
    pub begun: u64,
    pub ended: u64,
    pub answered: u64,
}

/// The loop's [`Progress`], for a door to wait on.
pub type Rounds = watch::Sender<Progress>;

/// A round asked for now and waited on (mnp-1): what a door holds to ask
/// the group's ring once before it answers a signer it does not know.
/// Asks while a round runs share the next round, as nudges do.
#[derive(Clone)]
pub struct AskRound {
    nudge: Arc<Notify>,
    rounds: watch::Receiver<Progress>,
}

impl AskRound {
    pub fn new(nudge: Arc<Notify>, rounds: &Rounds) -> Self {
        let rounds = rounds.subscribe();
        Self { nudge, rounds }
    }

    /// Wake the round now, waiting on nothing: a write that must reach the
    /// group before the next interval (U7, a removal and a hand-on).
    pub fn wake(&self) {
        self.nudge.notify_one();
    }

    /// Wake the round now and wait, at most `within`, until `known` holds,
    /// asked again as each member answers, or a round begun after this ask
    /// ends: whether either came in time. A member that answers ends the
    /// wait; one asleep holds only its own dial (asleep-1-none-held).
    pub async fn until(&self, within: Duration, known: impl Fn() -> bool) -> bool {
        let mut rounds = self.rounds.clone();
        let after = rounds.borrow_and_update().begun + 1;
        self.nudge.notify_one();
        let waited = tokio::time::timeout(within, async {
            while !known() && rounds.borrow_and_update().ended < after {
                if rounds.changed().await.is_err() {
                    return false;
                }
            }
            true
        });
        waited.await.unwrap_or(false)
    }
}

/// Spawn the periodic ring-sync task. Call once per daemon start.
///
/// Runs one round **immediately** before entering the interval, because the
/// first thing a node that has been offline owes its ring is everything it
/// holds — waiting a full minute to boot-republish would leave a freshly
/// restarted peer confidently reporting a total over a subset for that whole
/// minute.
///
/// `nudge` is the wake-up a local writer fires after it signs onto a journal
/// (`POST /v1/rail/append`, the donor loop, a peer coming back). A mesh-store
/// write reaches its journal on cw-rails' pump tick and waits for the next
/// round; nothing nudges for it (fp-83). Sixty seconds is the right cadence for anti-entropy
/// and the wrong one for "I just wrote something", and the two do not have to
/// be the same number: the nudge starts a round now, and the round is the same
/// round. **Nothing about the sender census changes** — the pump does not talk
/// to a peer, it asks this loop to. (svrngs: of those writers, the record
/// API's append door, [`append`]; beside it the member door asks one,
/// [`AskRound`], and `rounds` counts them begun and ended for it.)
pub fn spawn_ring_sync_loop(
    host: Arc<Host>,
    interval: Duration,
    nudge: Arc<Notify>,
    rounds: Rounds,
) -> RingSyncHandle {
    let task = tokio::spawn(async move {
        info!(
            interval_secs = interval.as_secs(),
            "ring sync: loop started"
        );
        let mut told = Told::new();
        loop {
            #[expect(
                clippy::disallowed_methods,
                reason = "times a round for its trace; no decision reads it"
            )]
            let started = Instant::now();
            rounds.send_modify(|p| p.begun += 1);
            let answered = || rounds.send_modify(|p| p.answered += 1);
            let outcome = run_one_round(host.as_ref(), &answered, &mut told).await;
            rounds.send_modify(|p| p.ended = p.begun);
            if outcome.namespaces > 0 {
                debug!(
                    namespaces = outcome.namespaces,
                    peers_reached = outcome.peers_reached,
                    peers_unreachable = outcome.peers_unreachable,
                    peers_refused = outcome.peers_refused,
                    ops_pulled = outcome.ops_pulled,
                    ops_pushed = outcome.ops_pushed,
                    round_ms = started.elapsed().as_millis() as u64,
                    "ring sync: round"
                );
            }
            // `Notify::notify_one` stores one permit when nobody is waiting, so
            // a write that lands mid-round wakes the NEXT sleep rather than
            // being lost — which is exactly the write that most needs to go.
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = nudge.notified() => {
                    debug!("ring sync: woken by a local write rather than the interval");
                }
            }
        }
    });
    RingSyncHandle {
        _task: AbortOnDropHandle::new(task),
    }
}

/// One anti-entropy pass over every namespace this node holds, against every
/// online peer **standing in its register** (svrngs: upstream, its roster),
/// every pair dialled at once and `answered` told as each member answers.
///
/// (svrngs, asleep-1-none-held: upstream dialled one pair after another, so
/// an asleep member held every exchange after it to its dial's ten-second
/// timeout, and a door waiting on the round refused a newcomer another
/// member had already let in, `demo/runs/mnp/2026-10-09/README.md`, "What
/// no prediction named" 1. Each exchange is idempotent and keyed on the op
/// id, so two at once with one journal duplicate nothing.)
pub async fn run_one_round(
    host: &Host,
    answered: &(impl Fn() + Sync),
    told: &mut Told,
) -> RoundOutcome {
    let mut outcome = RoundOutcome::default();
    let rail = &host.rail;
    let namespaces = match rail.namespaces() {
        Ok(n) => n,
        Err(e) => {
            warn!(error = %e, "ring sync: cannot enumerate namespaces");
            return outcome;
        }
    };
    if namespaces.is_empty() {
        return outcome;
    }
    // Where this computer is reached, said before it dials, so a move goes
    // out in this round (relay-in-record).
    host.say_relay();
    let leaves = tell_leaves(host, &namespaces, told);

    let Some(RoundMembers {
        peers,
        exchanged_with,
        skipped_offline,
    }) = host.members().await
    else {
        debug!("ring sync: this node has no membership to round with");
        leaves.await;
        return outcome;
    };
    // GLASSBOX: which members this round carried and which it passed over. A
    // round that skips the only peer looks exactly like a round that had
    // nothing to send, and room run 2 (2026-09-20) cost 85 s to a difference
    // that was invisible in the log. Logged BEFORE the empty-peer return, so
    // the skipped-everyone round is the one that says so.
    info!(
        exchanged_with = ?exchanged_with,
        skipped_offline = ?skipped_offline,
        "ring sync: round membership"
    );
    if peers.is_empty() {
        leaves.await;
        return outcome;
    }
    let transport = &host.transport;

    let mut dials = Vec::new();
    for namespace in &namespaces {
        outcome.namespaces += 1;
        // ── A ring is OFFERED to its register, and to nobody else.
        //
        // Per NAMESPACE and not per round: one peer may be on one ring and not
        // another, so a round-level peer set cannot answer this. Standing is
        // read through the rail's ONE accessor (`RingRail::standing`), the
        // answer the serving route asks too: the keys standing in the ring's
        // seed and its `Admit` and `Remove` acts. (svrngs: upstream read the
        // roster — its seed, or one derived from the mesh — DESIGN §8, U1.)
        //
        // A ring with no seed, or an UNREADABLE register, offers nothing for
        // this namespace. Under-share, never over-share.
        let standing = match rail.standing(namespace) {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    namespace,
                    error = %e,
                    "ring sync: this namespace's register is unreadable, so it is \
                     offered to nobody this round"
                );
                continue;
            }
        };
        let (offered_to, skipped_not_standing): (Vec<&OnlinePeer>, Vec<&str>) = {
            let mut on = Vec::new();
            let mut off = Vec::new();
            for p in &peers {
                if p.pubkey.is_some_and(|k| standing.names(&key_actor(&k))) {
                    on.push(p);
                } else {
                    off.push(p.name.as_str());
                }
            }
            (on, off)
        };
        // GLASSBOX: the third list beside exchanged and offline. A namespace
        // that reaches nobody because nobody stands in it looks exactly
        // like a namespace with nothing to send.
        debug!(
            namespace,
            offered_to = ?offered_to.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            skipped_not_standing = ?skipped_not_standing,
            "ring sync: namespace register"
        );

        for OnlinePeer { contact, .. } in offered_to {
            dials.push(async move {
                let ended = exchange_with((transport, contact), rail, namespace).await;
                if matches!(ended.2, Reach::Reached) {
                    answered();
                }
                ended
            });
        }
    }
    let ((), ended) = tokio::join!(leaves, n0_future::join_all(dials));
    for (pulled, pushed, reach) in ended {
        // Counted BEFORE the verdict is read: see `exchange_with`.
        outcome.ops_pulled += pulled;
        outcome.ops_pushed += pushed;
        match reach {
            Reach::Reached => outcome.peers_reached += 1,
            Reach::Refused => outcome.peers_refused += 1,
            Reach::Unreachable => outcome.peers_unreachable += 1,
        }
    }
    outcome
}

/// Each group this computer left and each computer told its leave there
/// that answered: the round's loop owns it, so a computer is told again
/// only until it answers, and once more after a restart (#190).
pub type Told = std::collections::BTreeSet<(String, String)>;

/// A group this computer left stands for nobody here, so no exchange
/// carries its leave: told, each round, to each computer the copy says
/// stands and has not answered (#190), at once with the round's dials.
async fn tell_leaves(host: &Host, namespaces: &[String], told: &mut Told) {
    for ns in namespaces {
        let Ok(crate::register::Standing::Left(stands)) = host.rail.standing(ns) else {
            continue;
        };
        let computers: Vec<_> = (stands.into_iter())
            .filter(|(k, m)| {
                m.device == crate::Device::Computer && !told.contains(&(ns.clone(), k.clone()))
            })
            .map(|(k, m)| (k, m.relay))
            .collect();
        let lines = host.rail.own_removes(ns).unwrap_or_else(|e| {
            warn!(namespace = ns, error = %e, "ring sync: a group left, whose leave cannot be read, so it is not told");
            Vec::new()
        });
        debug!(namespace = ns, untold = computers.len(), lines = lines.len(), "ring sync: a group this computer left — its leave told to each computer standing that has not answered; none when it was removed");
        if computers.is_empty() || lines.is_empty() {
            continue;
        }
        for actor in crate::removal::tell(host, ns, computers, &lines).await {
            told.insert((ns.clone(), actor));
        }
    }
}

/// How a round's exchange with one member ended: it answered, it answered
/// and refused, or no address of it answered.
enum Reach {
    Reached,
    Refused,
    Unreachable,
}

/// One round's exchange of `namespace` with the member at `to`: the ops
/// pulled and pushed, and how it ended. (svrngs, arch-peer (c): upstream
/// tried each address its transport answered until one worked; a member is
/// asked by its key now, so there is one.)
async fn exchange_with(to: To<'_>, rail: &RingRail, namespace: &str) -> (usize, usize, Reach) {
    let ex = exchange(to, rail, namespace).await;
    // `ex.pulled` is progress whether or not a later call in the same
    // exchange failed: `ingest_all` has already written those lines to disk.
    let reach = match ex.stop {
        None => Reach::Reached,
        Some(ExchangeStop::Refused { sent_bytes, status }) => {
            warn!(
                peer = %to.1.key,
                sent_bytes,
                status,
                budget_bytes = RING_SYNC_OPS_BUDGET_BYTES,
                "ring sync: peer REFUSED — it answered, so this is \
                 not an unreachable peer. 413: its body limit is \
                 below this build's exchange budget. 403: it does \
                 not serve this ring to us — we hold no standing \
                 in its register there, or it holds no such ring"
            );
            Reach::Refused
        }
        Some(ExchangeStop::Failed(detail)) => {
            debug!(peer = %to.1.key, detail, "ring sync: exchange failed, so the member is unreachable this round");
            Reach::Unreachable
        }
    };
    (ex.pulled, ex.pushed, reach)
}

/// Where an exchange goes: a member, by its contact, through the
/// transport that dials it.
pub type To<'a> = (&'a crate::iroh::IrohTransport, &'a crate::iroh::Contact);

/// Why an exchange with one peer address stopped.
#[derive(Debug)]
pub enum ExchangeStop {
    /// The peer **answered**, and refused. Two statuses arrive here and both
    /// are answers rather than reachability: `413`, our body was too large,
    /// and `403`, this ring is not served to us.
    ///
    /// Its own variant because the alternative is the collapse this rung
    /// exists to undo: a 413 came back as `Err("HTTP 413")`, indistinguishable
    /// from a dead socket, and the round filed a reachable peer under
    /// `peers_unreachable` at DEBUG. The 403 had exactly the same shape until
    /// `tg-2-strangers-are-refused`, and it is the status the new internal-port
    /// gate and `roster_refusal` both answer with — so a round refused for
    /// identity would have read as a partitioned mesh.
    ///
    /// `status` is carried because the two need different sentences: a 413
    /// names a body limit below this build's budget, a 403 names a refusal to
    /// serve. `sent_bytes` is honest for both — it is what THIS side put on
    /// the wire, measured before the send.
    Refused { sent_bytes: usize, status: u16 },
    /// Everything else this address could stop on: no route, a timeout, a
    /// 5xx, an answer this build could not read, or **this node's own journal
    /// refusing to read**. They are one variant because the round does the
    /// same thing with all of them — try the next address, then count the
    /// peer unreachable — and the local case is spelled `local journal: …` in
    /// the detail so the log still says whose fault it was.
    Failed(String),
}

/// What one exchange moved, and why it stopped.
///
/// **Progress and failure are reported together, and that is the point.**
/// `ingest_all` writes to the journal, so ops pulled in call 1 are durably
/// held whether or not a later call fails. The old signature was
/// `Result<(usize, usize), String>`, so a failing call 2 returned `Err` and
/// threw the pulled count away — `ops_pulled` undercounted exactly in the
/// failure case, which is the case anyone reading the metric is looking for.
#[derive(Debug, Default)]
pub struct ExchangeOutcome {
    pub pulled: usize,
    pub pushed: usize,
    pub stop: Option<ExchangeStop>,
}

impl ExchangeOutcome {
    /// Keep what moved; record why it stopped.
    fn stopped(mut self, stop: ExchangeStop) -> Self {
        self.stop = Some(stop);
        self
    }
}

/// The chunked exchange with one member: two calls per chunk, repeated
/// until neither side moves or [`MAX_CHUNKS_PER_EXCHANGE`] is spent.
///
/// See the module docs for why repeating terminates. The two stopping
/// conditions below are the honest ones and neither is silent: *converged*
/// (nothing came, nothing left to send) returns quietly, and *stalled*
/// (something outstanding, nothing moved anywhere) warns — repeating that
/// would repeat verbatim.
pub async fn exchange(to: To<'_>, rail: &RingRail, namespace: &str) -> ExchangeOutcome {
    let peer = to.1.key;
    let mut out = ExchangeOutcome::default();
    let mut last_peer_digest: Option<crate::Digest> = None;

    for chunk in 1..=MAX_CHUNKS_PER_EXCHANGE {
        // ── Call 1 — learn what they have, take one chunk of what we lack.
        let mine = match rail.digest(namespace) {
            Ok(d) => d,
            Err(e) => return out.stopped(ExchangeStop::Failed(format!("local journal: {e}"))),
        };
        let first = match post(
            to,
            &RingSyncRequest {
                namespace: namespace.to_string(),
                digest: mine,
                ops: Vec::new(),
            },
        )
        .await
        {
            Ok(r) => r,
            Err(stop) => return out.stopped(stop),
        };
        let pulled = match rail.ingest_all(namespace, &first.ops) {
            Ok(n) => n,
            Err(e) => return out.stopped(ExchangeStop::Failed(format!("local journal: {e}"))),
        };
        out.pulled += pulled;

        // The peer's OWN report of what it holds — the one progress signal
        // that cannot be faked by a count an older build did not send.
        let peer_digest_moved = last_peer_digest.as_ref() != Some(&first.digest);
        last_peer_digest = Some(first.digest.clone());

        // ── Call 2 — give them one chunk of what they lack, out of
        // everything we now hold.
        let (for_peer, more_for_peer) = match rail.ops_missing_from_within(
            namespace,
            &first.digest,
            RING_SYNC_OPS_BUDGET_BYTES,
        ) {
            Ok(v) => v,
            Err(e) => return out.stopped(ExchangeStop::Failed(format!("local journal: {e}"))),
        };
        let offered = for_peer.len();
        let pushed = if for_peer.is_empty() {
            0
        } else {
            let refreshed = match rail.digest(namespace) {
                Ok(d) => d,
                Err(e) => return out.stopped(ExchangeStop::Failed(format!("local journal: {e}"))),
            };
            match post(
                to,
                &RingSyncRequest {
                    namespace: namespace.to_string(),
                    digest: refreshed,
                    ops: for_peer,
                },
            )
            .await
            {
                Ok(second) => second.ingested,
                // `out.pulled` is KEPT: call 1's ops are on disk already.
                Err(stop) => return out.stopped(stop),
            }
        };
        out.pushed += pushed;

        debug!(
            namespace,
            %peer,
            chunk,
            pulled,
            pushed,
            peer_sent = first.ops.len(),
            offered,
            more_for_peer,
            "ring sync: chunk"
        );

        if pulled == 0 && offered == 0 {
            return out; // converged
        }
        if pulled == 0 && pushed == 0 && !peer_digest_moved {
            warn!(
                namespace,
                %peer,
                chunk,
                offered,
                "ring sync: exchange stalled — the peer's digest did not move \
                 and nothing was ingested, so repeating would repeat verbatim; \
                 leaving the rest to the next round"
            );
            return out;
        }
    }
    warn!(
        namespace,
        %peer,
        chunks = MAX_CHUNKS_PER_EXCHANGE,
        pulled = out.pulled,
        pushed = out.pushed,
        "ring sync: exchange hit its chunk bound with work outstanding — every \
         op moved is already on disk, so the next round resumes here"
    );
    out
}

pub(crate) async fn post(
    to: To<'_>,
    body: &RingSyncRequest,
) -> Result<RingSyncResponse, ExchangeStop> {
    // Serialised here rather than handed to `.json(body)` so the byte count
    // the receiver's limit judges is a number THIS side can name in a log.
    let payload = match serde_json::to_vec(body) {
        Ok(p) => p,
        Err(e) => return Err(ExchangeStop::Failed(format!("local encode: {e}"))),
    };
    let sent_bytes = payload.len();
    let request = axum::http::Request::post(SYNC_ROUTE)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(payload))
        .expect("a fixed route and header");
    let resp = to.0.fetch(to.1, request).await;
    let resp = resp.map_err(|e| ExchangeStop::Failed(e.to_string()))?;
    let status = resp.status();
    if status == StatusCode::PAYLOAD_TOO_LARGE || status == StatusCode::FORBIDDEN {
        return Err(ExchangeStop::Refused {
            sent_bytes,
            status: status.as_u16(),
        });
    }
    if !status.is_success() {
        return Err(ExchangeStop::Failed(format!("HTTP {status}")));
    }
    serde_json::from_slice(resp.body()).map_err(|e| ExchangeStop::Failed(e.to_string()))
}

/// Where a member's exchange is asked, and served (`ring_routes.rs`).
pub const SYNC_ROUTE: &str = "/internal/ring/sync";

/// The responder's half of one exchange: ingest what the caller offered,
/// then answer with this node's digest and one budget of what the caller's
/// digest says it lacks. The second value says whether more remains.
///
/// Budgeted in BOTH directions — the response was the unbounded half, and an
/// unbounded body is also an unbounded allocation on a route any member may
/// call. Called after the asker's roster check, never before: a refusal that
/// ran after the ingest would have taken the asker's ops onto this disk on its
/// way to saying no.
pub fn answer(
    rail: &RingRail,
    req: &RingSyncRequest,
) -> Result<(RingSyncResponse, bool), RailError> {
    let ingested = rail.ingest_all(&req.namespace, &req.ops)?;
    let (ops, more_for_caller) =
        rail.ops_missing_from_within(&req.namespace, &req.digest, RING_SYNC_OPS_BUDGET_BYTES)?;
    let digest = rail.digest(&req.namespace)?;
    Ok((
        RingSyncResponse {
            namespace: req.namespace.clone(),
            digest,
            ops,
            ingested,
        },
        more_for_caller,
    ))
}

/// `POST /v1/rail/append`, then wake the round: an act this node just signed
/// is the one that most needs to go, and sixty seconds is the cadence for
/// anti-entropy, not for a write (the daemon's `ring_write_nudge`, here).
/// A refused append wakes nothing. (svrngs: the nudge is the one the node
/// builds for its round, held by the record API's state, `RailApi`, where
/// upstream's was `RailsDaemon::ring_nudge`.)
pub async fn append(
    axum::extract::State(api): axum::extract::State<crate::rail::RailApi>,
    query: axum::extract::Query<crate::rail::RailQuery>,
    body: axum::Json<serde_json::Value>,
) -> axum::response::Response {
    let nudge = api.nudge.clone();
    let answer = crate::rail::append(axum::extract::State(api), query, body)
        .await
        .unwrap_or_else(|refused| refused);
    if answer.status().is_success() {
        nudge.notify_one();
        info!(target: "rails", "ring sync: an append woke the round");
    }
    answer
}

#[cfg(test)]
pub(crate) mod tests;
