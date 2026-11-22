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
//! `RingSyncHost` seam went, PLAN §4, U1's inventory.)
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
//! is a ceiling on idleness rather than on a write. (svrngs: node startup
//! installs one [`RingRail::on_write`] callback for folder application and
//! the round, for local writes and new peer lines alike. The member door
//! can also ask a round explicitly, [`AskRound`].)
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
use n0_future::StreamExt as _;
use tokio::sync::{watch, Notify};
use tracing::{debug, info, warn};

use crate::iroh::{key_actor, Contact};
use crate::node::Host;
use axum::http::StatusCode;

/// A member of a round, as the round sees it: the key and relay it dials
/// ([`Contact`]), and the name its glassbox line prints. With no presence
/// (PLAN §4, U1's "No presence") none is skipped as offline: an offline
/// member costs its dial.
pub(crate) struct OnlinePeer {
    pub(crate) contact: Contact,
    pub(crate) name: String,
}

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
/// `nudge` is shared by the node's journal-change callback and explicit
/// refresh requests ([`AskRound`]). A change during a round leaves a permit
/// for the next round. No-op commits and duplicate ingestion notify nothing,
/// so a converged exchange returns to the interval. `rounds` counts rounds
/// begun and ended for a waiting door. The loop is aborted when the
/// handle returned drops.
pub fn spawn_ring_sync_loop(
    folders: crate::folder::Folders,
    interval: Duration,
    nudge: Arc<Notify>,
    rounds: Rounds,
) -> AbortOnDropHandle<()> {
    let task = tokio::spawn(async move {
        info!(
            interval_secs = interval.as_secs(),
            "ring sync: loop started"
        );
        let mut told = LeavesTold::new();
        loop {
            #[expect(
                clippy::disallowed_methods,
                reason = "times a round for its trace; no decision reads it"
            )]
            let started = Instant::now();
            rounds.send_modify(|p| p.begun += 1);
            let answered = || rounds.send_modify(|p| p.answered += 1);
            let outcome = run_one_round(&folders, &answered, &mut told).await;
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
                    debug!("ring sync: woken by a journal change or explicit ask rather than the interval");
                }
            }
        }
    });
    AbortOnDropHandle::new(task)
}

/// One anti-entropy pass over every namespace this node holds, against every
/// online peer **standing in its register** (svrngs: upstream, its roster),
/// every pair dialled at once and `answered` told as each member answers;
/// `folders`' host's, with the app databases `folders` says it keeps.
///
/// (svrngs, asleep-1-none-held: upstream dialled one pair after another, so
/// an asleep member held every exchange after it to its dial's ten-second
/// timeout, and a door waiting on the round refused a newcomer another
/// member had already let in, `demo/runs/mnp/2026-10-09/README.md`, "What
/// no prediction named" 1. Each exchange is idempotent and keyed on the op
/// id, so two at once with one journal duplicate nothing.)
pub async fn run_one_round(
    folders: &crate::folder::Folders,
    answered: &(impl Fn() + Sync),
    told: &mut LeavesTold,
) -> RoundOutcome {
    let (host, mut outcome) = (folders.host.as_ref(), RoundOutcome::default());
    let rail = &host.rail;
    let Ok(namespaces) = rail.namespaces().inspect_err(|e| {
        warn!(error = %e, "ring sync: cannot enumerate namespaces");
    }) else {
        return outcome;
    };
    if namespaces.is_empty() {
        return outcome;
    }
    // Where this computer is reached, said before it dials, so a move goes
    // out in this round (relay-in-record); and where its groups meet read
    // again, so a keeper's new list is met from this round on (relay
    // fallbacks).
    host.say_relay(host.home());
    host.refresh();
    host.decide(None).await;

    // With no members, no group is offered, and a group this computer left
    // is still read below, for its leave (#190).
    let members = host.members().await;
    if members.is_none() {
        debug!("ring sync: this node has no membership to round with");
    }
    let peers = members.unwrap_or_default();
    // GLASSBOX: which members this round carries. A round with nobody looks
    // exactly like a round that had nothing to send, and room run 2
    // (2026-09-20) cost 85 s to a difference that was invisible in the log.
    // Logged before any group is offered, so the empty round says so.
    info!(
        exchanged_with = ?peers.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        "ring sync: round membership"
    );
    let transport: &dyn crate::iroh::Fetch = &*host.transport;

    let (mut dials, mut leaves) = (Vec::new(), Vec::new());
    for namespace in &namespaces {
        outcome.namespaces += 1;
        // ── A ring is OFFERED to its register, and to nobody else.
        //
        // Per NAMESPACE and not per round: one peer may be on one ring and not
        // another, so a round-level peer set cannot answer this. Standing is
        // read through the rail's ONE accessor (`RingRail::standing`), the
        // answer the serving route asks too: the keys standing in the ring's
        // seed and its `Admit` and `Remove` acts. (svrngs: upstream read the
        // roster — its seed, or one derived from the mesh — PLAN §4, U1.)
        //
        // A ring with no seed, or an UNREADABLE register, offers nothing for
        // this namespace. Under-share, never over-share.
        let standing = match rail.standing(namespace) {
            Ok(crate::register::Standing::Left(stands)) => {
                // Its app databases closed, their files kept, as the group's
                // other files are: nothing of a group left is live here.
                host.states.close(namespace);
                leaves.extend(leave_told(host, namespace, stands, told));
                continue;
            }
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
        let (offered_to, off): (Vec<&OnlinePeer>, Vec<_>) =
            (peers.iter()).partition(|p| standing.names(&key_actor(&p.contact.key)));
        // GLASSBOX: the third list beside exchanged and offline. A namespace
        // that reaches nobody because nobody stands in it looks exactly
        // like a namespace with nothing to send.
        debug!(
            namespace,
            offered_to = ?offered_to.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            skipped_not_standing = ?off.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            "ring sync: namespace register"
        );

        // The app databases this computer keeps, opened before any member is
        // dialled, so a member's call this round finds each one here too.
        let kept = Arc::new(folders.kept(namespace).await);
        for OnlinePeer { contact, .. } in offered_to {
            let kept = kept.clone();
            dials.push(async move {
                let ended = exchange_with((transport, contact), rail, namespace).await;
                if ended.stop.is_none() {
                    answered();
                    states((transport, contact), host, namespace, &kept).await;
                }
                ended
            });
        }
    }
    let (answered_leaves, ended) =
        tokio::join!(n0_future::join_all(leaves), n0_future::join_all(dials));
    told.extend(answered_leaves.into_iter().flatten());
    for ex in ended {
        // Counted BEFORE the verdict is read: see `exchange_with`. It
        // answered, it answered and refused, or no address of it answered.
        outcome.ops_pulled += ex.pulled;
        outcome.ops_pushed += ex.pushed;
        match ex.stop {
            None => outcome.peers_reached += 1,
            Some(ExchangeStop::Refused { .. }) => outcome.peers_refused += 1,
            Some(ExchangeStop::Failed(_)) => outcome.peers_unreachable += 1,
        }
    }
    outcome
}

/// Each group this computer left and each computer told its leave there
/// that answered: the round's loop owns it, so a computer is told again
/// only until it answers, and once more after a restart (#190).
pub type LeavesTold = std::collections::BTreeSet<(String, String)>;

/// A group this computer left stands for nobody here, so no exchange
/// carries its leave: told, each round, from the standing the round read,
/// to each computer the copy says stands and has not answered (#190), at
/// once with the round's dials, with the leaver's acts before it that each
/// lacks (`removal.rs` `tell`, #211); `None` with nobody, or nothing, to
/// tell. A computer is answered once its digest covers the leave, or it
/// refuses this one, so a chain past one exchange's budget is told again
/// until it all arrives, never taken for told.
fn leave_told<'a>(
    host: &'a Host,
    ns: &'a str,
    stands: std::collections::BTreeMap<String, crate::register::Member>,
    told: &LeavesTold,
) -> Option<impl std::future::Future<Output = Vec<(String, String)>> + 'a> {
    let computers: Vec<_> = (stands.into_iter())
        .filter(|(k, m)| {
            m.device == crate::Device::Computer && !told.contains(&(ns.to_string(), k.clone()))
        })
        .map(|(k, m)| (k, m.relay))
        .collect();
    let lines = host.rail.own_removes(ns).unwrap_or_else(|e| {
        warn!(namespace = ns, error = %e, "ring sync: a group left, whose leave cannot be read, so it is not told");
        Vec::new()
    });
    debug!(namespace = ns, untold = computers.len(), lines = lines.len(), "ring sync: a group this computer left — its leave told to each computer standing that has not answered; none when it was removed");
    let due = !computers.is_empty() && !lines.is_empty();
    due.then_some(async move {
        let answered = crate::removal::tell(host, ns, computers).await;
        answered
            .into_iter()
            .map(|actor| (ns.to_string(), actor))
            .collect()
    })
}

/// One round's exchange of `namespace` with the member at `to`: the ops
/// pulled and pushed, and how it ended, traced. (svrngs, arch-peer (c):
/// upstream tried each address its transport answered until one worked; a
/// member is asked by its key now, so there is one.)
async fn exchange_with(to: To<'_>, rail: &RingRail, namespace: &str) -> ExchangeOutcome {
    let ex = exchange(to, rail, namespace).await;
    // `ex.pulled` is progress whether or not a later call in the same
    // exchange failed: `ingest_all` has already written those lines to disk.
    match &ex.stop {
        None => {}
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
        }
        Some(ExchangeStop::Failed(detail)) => {
            debug!(peer = %to.1.key, detail, "ring sync: exchange failed, so the member is unreachable this round");
        }
    }
    ex
}

/// Where an exchange goes: a member, by its contact, through the
/// transport that dials it.
pub type To<'a> = (&'a dyn crate::iroh::Fetch, &'a crate::iroh::Contact);

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

/// The chunked exchange with one member: two calls per chunk, repeated
/// until neither side moves or [`MAX_CHUNKS_PER_EXCHANGE`] is spent.
///
/// See the module docs for why repeating terminates. The two stopping
/// conditions below are the honest ones and neither is silent: *converged*
/// (nothing came, nothing left to send) returns quietly, and *stalled*
/// (something outstanding, nothing moved anywhere) warns — repeating that
/// would repeat verbatim.
pub async fn exchange(to: To<'_>, rail: &RingRail, namespace: &str) -> ExchangeOutcome {
    let mut out = ExchangeOutcome::default();
    if let Err(stop) = chunks(to, rail, namespace, &mut out).await {
        out.stop = Some(stop);
    }
    out
}

/// [`exchange`]'s chunks, each call's move kept in `out` as it lands; why a
/// call stopped them, if one did.
async fn chunks(
    to: To<'_>,
    rail: &RingRail,
    namespace: &str,
    out: &mut ExchangeOutcome,
) -> Result<(), ExchangeStop> {
    let peer = to.1.key;
    let local = |e: RailError| ExchangeStop::Failed(format!("local journal: {e}"));
    let call = |digest, ops| async move {
        let body = RingSyncRequest {
            namespace: namespace.to_string(),
            digest,
            ops,
        };
        post(to, &body).await
    };
    let mut last_peer_digest: Option<crate::Digest> = None;

    for chunk in 1..=MAX_CHUNKS_PER_EXCHANGE {
        // ── Call 1 — learn what they have, take one chunk of what we lack.
        let mine = rail.digest(namespace).map_err(local)?;
        let first = call(mine, Vec::new()).await?;
        let pulled = rail.ingest_all(namespace, &first.ops).map_err(local)?;
        out.pulled += pulled;

        // The peer's OWN report of what it holds — the one progress signal
        // that cannot be faked by a count an older build did not send.
        let peer_digest_moved = last_peer_digest.as_ref() != Some(&first.digest);
        last_peer_digest = Some(first.digest.clone());

        // ── Call 2 — give them one chunk of what they lack, out of
        // everything we now hold.
        let (for_peer, more_for_peer) = rail
            .ops_missing_from_within(namespace, &first.digest, RING_SYNC_OPS_BUDGET_BYTES)
            .map_err(local)?;
        let offered = for_peer.len();
        let pushed = if for_peer.is_empty() {
            0
        } else {
            let refreshed = rail.digest(namespace).map_err(local)?;
            // Stopped here, `out.pulled` is KEPT: call 1's ops are on disk already.
            call(refreshed, for_peer).await?.ingested
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
            return Ok(()); // converged
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
            return Ok(());
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
    Ok(())
}

pub(crate) async fn post(
    to: To<'_>,
    body: &RingSyncRequest,
) -> Result<RingSyncResponse, ExchangeStop> {
    // Serialised here rather than handed to `.json(body)` so the byte count
    // the receiver's limit judges is a number THIS side can name in a log.
    let payload =
        serde_json::to_vec(body).map_err(|e| ExchangeStop::Failed(format!("local encode: {e}")))?;
    let sent_bytes = payload.len();
    let request = crate::iroh::posted(SYNC_ROUTE, payload);
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

/// Where a member's app databases are exchanged, and served
/// ([`crate::ring_routes::ring_state`]).
pub const STATE_ROUTE: &str = "/internal/ring/state";

/// The keys `group`'s record binds, as a set's device is judged: standing
/// now, or cut since, as a removed device's sets are taken like any (svrn-docs
/// APP_STATE §4, "A removed device"), each with its share of the room an
/// app database's ask has ([`crate::peer_wire::ASK_ROOM_BYTES`]). An
/// unreadable record binds none, so sets wait, said at warn.
pub(crate) fn bound(rail: &RingRail, group: &str) -> svrngs_db::Binds {
    let read = rail.register(group).inspect_err(|e| warn!(namespace = group, error = %e, "ring state: the record is unreadable, so every set waits"));
    let read = read.unwrap_or_default();
    let keys = read.members.into_keys().chain(read.removed.into_keys());
    svrngs_db::Binds::new(keys, crate::peer_wire::ASK_ROOM_BYTES)
}

/// Each app database this computer keeps in `group`
/// ([`crate::folder::Folders::kept`]), exchanged with the member at `to`,
/// after the record's in the same round (the app-state
/// spike's `exchange`, svrn-docs `rings/current/app-state-spike/spike.patch:1355-1401`):
/// its waiting sets judged again by the record, then a call a chunk, each
/// giving what the member's last ask lacks and taking what the member's
/// answer gives, until neither moves or the chunks are spent. Whether the
/// member answered each call of each one, so holds what this copy sent.
async fn states(to: To<'_>, host: &Host, group: &str, kept: &[Arc<svrngs_db::Db>]) -> bool {
    let bound = Arc::new(bound(&host.rail, group));
    // The member called, the computer its sets are signed by, which a full
    // answer's marks for its own sites are taken on the word of.
    let from = svrngs_db::Device::parse(&key_actor(&to.1.key));
    let mut answered = true;
    for db in kept {
        let judge = bound.clone();
        let again = off(&db, move |db| db.rejudge(&*judge)).await;
        let mut theirs: Option<svrngs_db::Ask> = None;
        for chunk in 1..=MAX_CHUNKS_PER_EXCHANGE {
            let asked = theirs.take();
            let ours = off(&db, move |db| match asked {
                Some(ask) => db.missing(&ask, RING_SYNC_OPS_BUDGET_BYTES),
                None => db.ask().map(|ask| svrngs_db::Call {
                    ask,
                    ..Default::default()
                }),
            });
            let Some(Ok(ours)) = ours.await else {
                answered = false;
                break;
            };
            let body = postcard::to_allocvec(&((group, &db.app), &ours))
                .expect("a call is written in memory");
            let answer =
                to.0.fetch(to.1, crate::iroh::posted(STATE_ROUTE, body))
                    .await;
            let answer = answer.ok().filter(|a| a.status().is_success());
            let answer =
                answer.and_then(|a| postcard::from_bytes::<svrngs_db::Call>(a.body()).ok());
            let (judge, from) = (bound.clone(), from.clone());
            let ingested = match answer {
                Some(a) => off(&db, move |db| (db.ingest(&a, &*judge, from.as_ref()), a)).await,
                None => None,
            };
            let (took, answer) = ingested.unzip();
            debug!(namespace = group, app = db.app, chunk, ?again, sent = ours.sets.len(), ?took, "ring state: a chunk of an app database's exchange with a member, or none, as it didn't answer");
            let moved = took.as_ref().is_some_and(|t| {
                t.as_ref().is_ok_and(|t| {
                    t.iter().any(|(_, t)| {
                        !matches!(
                            t,
                            svrngs_db::Took::Held | svrngs_db::Took::Fault | svrngs_db::Took::Over
                        )
                    })
                })
            });
            match answer {
                Some(answer) if chunk == 1 || moved || !ours.sets.is_empty() => {
                    theirs = Some(answer.ask)
                }
                ended => {
                    answered &= ended.is_some();
                    break;
                }
            }
        }
    }
    answered
}

/// Before this computer sleeps, as a lid closes: each app database it keeps
/// in each group it stands in pushed to an awake member, by a full call by
/// that member's marks ([`states`]), so a closing lid strands no write while
/// another computer of the group is awake (svrn-docs APP_STATE §11, W2;
/// MAGNETS_BUILD §3, "Slice B": under §4's marks a full call is safe). Every
/// member of a group is dialled at once, and the group is done once one has
/// answered each call, so an asleep one costs only its own dial. The
/// platform's own hook asks it before it sleeps (`carried::SLEEPS`):
/// nothing here knows how a platform sleeps. Of the groups it keeps a
/// database in, how many an awake member took, and how many there are.
pub(crate) async fn before_sleep(host: &Host) -> (usize, usize) {
    let peers = host.members().await.unwrap_or_default();
    let (peers, transport): (&[OnlinePeer], &dyn crate::iroh::Fetch) = (&peers, &*host.transport);
    let groups = host.rail.standings().unwrap_or_default();
    let kept = (groups.iter()).map(|(g, stands)| (g, stands, host.states.held(g)));
    let pushes = kept
        .filter(|(.., k)| !k.is_empty())
        .map(|(group, stands, kept)| async move {
            let to = peers
                .iter()
                .filter(|p| stands.contains_key(&key_actor(&p.contact.key)));
            let dials = to.map(|p| states((transport, &p.contact), host, group, &kept));
            let mut dials: n0_future::FuturesUnordered<_> = dials.collect();
            dials.any(|took| took).await
        });
    let reached = n0_future::join_all(pushes).await;
    let took = reached.iter().filter(|r| **r).count();
    info!(groups = reached.len(), took, "ring state: before this computer sleeps, each group's app databases pushed to an awake member, or none answered");
    (took, reached.len())
}

/// `f` on `db`, off the runtime's threads, as a database's work is SQLite's
/// and its signature checks', the page's reads' too (`carried.rs`), so one
/// member's exchange holds no thread every group's round needs; none where
/// its task didn't end.
async fn off<T: Send + 'static>(
    db: &Arc<svrngs_db::Db>,
    f: impl FnOnce(&svrngs_db::Db) -> T + Send + 'static,
) -> Option<T> {
    let db = db.clone();
    tokio::task::spawn_blocking(move || f(&db)).await.ok()
}

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

#[cfg(test)]
pub(crate) mod tests;
