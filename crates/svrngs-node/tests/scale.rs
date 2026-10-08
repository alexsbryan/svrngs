//! The scale instrument: how one group's cost grows with its members, its
//! journal and its asleep computers. An instrument, not a check: it is
//! `#[ignore]`d, so `scripts/check.sh`'s pass of every test never runs it,
//! and it asserts only what makes a reading meaningful (the group
//! converged, every member was reached, an idle round moved nothing). What
//! `scripts/check.sh` does run here is the account's hand count, below, and
//! by name, in release, the ceiling's canary,
//! `an_idle_round_is_under_the_ceiling`: at 25 members and 10,000 acts an
//! idle round under 0.1 s of processor (DESIGN §8 :1083).
//!
//! It stands up real nodes — the record, the round, iroh — as `canaries.rs`
//! does: in one process, no relay, peers reached by seeded loopback
//! addresses. Every member writes an equal share of the journal, so each
//! act carries a view naming every member (`svrngs-core`
//! `SignedOp::view`), as it does in a group where everyone has written.
//! Each node is started with a round every day (`Node::start_every`), so no
//! node's own round runs inside one being timed.
//!
//! Run, one point at a time, in release:
//!
//! ```sh
//! SVRNGS_SCALE_MEMBERS=25 SVRNGS_SCALE_ACTS=10000 SVRNGS_SCALE_ASLEEP=0 \
//!   cargo test --release -p svrngs-node --test scale -- --ignored --nocapture
//! ```
//!
//! It prints one line, `SCALE {json}`:
//! - `idle_round_cpu_s` — the process's processor time for one computer's
//!   round with nothing to move. In one process it includes the answering
//!   side of every exchange, which is the per-computer average when every
//!   member rounds.
//! - `idle_round_wall_s` — how long that round took.
//! - `push_round_s` — the round after five appends: one write reaching
//!   everyone.
//! - `asleep_round` — with `SVRNGS_SCALE_ASLEEP` computers shut, how long
//!   one round took and how many it could not reach.
//! - `quiet_cpu_per_s` — the process's processor time a second over two
//!   seconds with no round running, after the idle rounds: what every node
//!   costs sitting idle, which a round's own reading includes for its wall
//!   time.
//! - `bytes_per_act`, `digest_pass_s`, `append_s`, and the journal's size.
//! - `rounds` — each measured round's own account of what it cost (`push`,
//!   the last `idle`, `asleep`): members `dialled`, `reached`, `asleep`
//!   (dialled and not reached) and `refused`; `acts_moved`, pulled and
//!   pushed; `cpu_s` and `wall_s`; and `folds`, each fold of a group's
//!   journal the round paid for, on either side of every exchange, counted
//!   `kept` (caught up from the journal's tail) or `rebuilt` (read from the
//!   top). A fold is counted from the debug line it traces ([`FOLDS`]), so
//!   one that traces nothing is not counted, and the hand count fails.
//!
//! The account's hand count, `the_account_matches_a_hand_count`, is not
//! ignored: a small round, one member killed after the round has listed it
//! and before it is dialled, whose every number is counted from the code by
//! hand beside it.
//!
//! Its record: first run 2026-10-05 at `15a9de7` (svrn-docs
//! `rings/runs/size-2026-10-05/`). Fitted on points up to 100 members, its
//! cost line predicted the 150-member idle round at 188 CPU-seconds before
//! it ran; it took 166. One run is not a rate (AGENTS.md 7): repeat a point
//! before reading a difference smaller than about 5% (the size run's two
//! readings of one point).
//!
//! ceiling-2, the digest kept (17f4611), predicted 2026-10-08 02:58 PDT,
//! before its runs, all in release on this computer:
//! - at 25 members and 2,000 acts, the idle round's processor about 15 s
//!   at 4d50557 (the instrument, promoted, read 15.09 s at that point,
//!   THE_CEILING.md "Order of work") and 0.5 to 2 s at 17f4611: the
//!   register, read whole once per member and twice by node 0, is what is
//!   left.
//! - at 25 and 10,000, the idle round's processor 3 to 10 s at 17f4611
//!   (73.1 s at `15a9de7`; the prototype's digest kept, 6.4 s), its
//!   account: digest kept 96 and rebuilt 0, offer kept 48 and rebuilt 0,
//!   write index kept 96, register rebuilt 26 (four digest catch-ups, two
//!   offers and one register read per member, as ceiling-1's 5-member
//!   idle round counted its register); and the write reaching everyone
//!   (`push_round_s`) in 25 to 60 s (206 s at `15a9de7`; 76 s with the
//!   prototype's digest kept).
//!
//! Measured 2026-10-08, after the predictions:
//! - 25 x 2,000, one run each: 14.12 s at 4d50557 and 1.13 s at 17f4611,
//!   confirmed; the idle account at 4d50557 digest rebuilt 48, offer
//!   rebuilt 48, register rebuilt 26.
//! - 25 x 10,000 at 17f4611, three runs: the idle round's processor 5.70,
//!   5.68 and 6.52 s (median 5.70), confirmed, its account in each exactly
//!   the one predicted; the write reaching everyone 54.2, 92.0 and 58.0 s,
//!   the median inside the 25 to 60 predicted and one of three above it:
//!   not confirmed. At 4d50557, one run: 67.1 s idle (73.1 s at
//!   `15a9de7`), its account digest rebuilt 48, offer rebuilt 48, register
//!   rebuilt 26, and 188 s for the write. What the idle round still pays
//!   for is the register, read whole 26 times: ceiling-4's fold.
//!
//! ceiling-3, each signature checked once (1f22ee6), predicted 2026-10-08
//! 03:26 PDT, before its runs, at 25 members and 10,000 acts in release,
//! three runs at 5a32d9e (the commit before) and three at 1f22ee6:
//! - the write reaching everyone (`push_round_s`), median 45 to 70 s at
//!   5a32d9e (54.2, 92.0, 58.0 at 17f4611, the same offer) and 28 to 45 s
//!   at 1f22ee6; the push round's own `cpu_s` lower by 12 to 22 s. Why:
//!   each of the 24 exchanges reads the log whole twice, an offer on each
//!   side, and each read verified its 10,054 lines, about 482,000
//!   signatures at 25 to 45 us.
//! - the idle round's processor unchanged, each median within 15% of the
//!   other (5.70 s at 17f4611): its 48 offers are answered unread and
//!   check no signature, as the prototype's idle round stayed at 6.4 s
//!   (THE_CEILING.md, "+ signatures once").
//! - every fold count in both rounds' accounts the same at both commits,
//!   and `append_s` within 15%: `commit` still checks every signature.
//!
//! Measured 2026-10-08, after the predictions, three runs each, the two
//! commits alternated, the computer's load average 31 to 50 throughout:
//! - the write reaching everyone 53.28, 53.10, 54.93 s at 5a32d9e (median
//!   53.28) and 37.51, 37.64, 37.82 s at 1f22ee6 (median 37.64), both
//!   confirmed; the push round's `cpu_s` medians 56.44 and 39.75 s, 16.7 s
//!   lower, confirmed. The effect, 15.6 s, is over three times the wider
//!   spread (1.8 s).
//! - the idle round's processor 5.53, 6.20, 5.68 s (median 5.68) and
//!   5.50, 5.55, 5.67 s (median 5.55), 2% apart, confirmed.
//! - every fold count the same in all six runs, confirmed: push digest kept
//!   264, offer kept 72 and rebuilt 48, register rebuilt 98, write index
//!   kept 336; idle as ceiling-2's. `append_s` medians 0.740 and 0.734 s,
//!   confirmed. What the write still pays for is the register, read whole
//!   98 times (ceiling-4's fold), and the 48 offers read whole, each line
//!   parsed and its id derived, no longer verified.
//!
//! ceiling-4, the register kept (8991334), predicted 2026-10-08 04:01 PDT,
//! before its runs, at 25 members and 10,000 acts in release, three runs at
//! 61de5bc (the commit before, the register read whole) and three at
//! 8991334, alternated:
//! - the idle round's processor, median 4.5 to 7 s at 61de5bc (5.55 s at
//!   1f22ee6, whose idle round is the same code) and 0.005 to 0.06 s at
//!   8991334, under the bar of 0.1 s (the prototype's register kept,
//!   0.010 s, THE_CEILING.md). Why: what the idle round paid for was 26
//!   register reads of the whole log, about 0.2 s each; kept, each costs a
//!   catch-up that reads no line and the seed's few hundred bytes.
//! - its account at 8991334: register kept 26 and rebuilt 0 (the push
//!   round before it folded every register once, and moved only records),
//!   digest kept 96, offer kept 48, write index kept 122 (96 and the 26
//!   register reads); at 61de5bc register rebuilt 26, write index kept 96.
//! - the write reaching everyone (`push_round_s`), median 12 to 25 s at
//!   8991334 (37.64 s at 1f22ee6): its 98 register reads, about 20 s of
//!   one exchange after another, kept; its account register rebuilt 25
//!   (each node's first read) and kept 73, write index kept 434.
//!
//! Measured 2026-10-08, after the predictions, three runs each, the two
//! commits alternated, each built in a target of its own, load average 3 to
//! 9 throughout (`uptime` before each):
//! - the idle round's processor 5.66, 5.58, 6.14 s at 61de5bc (median
//!   5.66) and 0.0101, 0.0098, 0.0096 s at 8991334 (median 0.0098), both
//!   confirmed, under the bar by ten times; the prototype's, 0.010 s.
//! - every account as predicted, in all six: idle register kept 26 and
//!   rebuilt 0, write index kept 122, at 8991334; register rebuilt 26 and
//!   write index kept 96 at 61de5bc; digest kept 96 and offer kept 48 at
//!   both. Confirmed.
//! - the write reaching everyone 38.33, 37.74, 39.68 s at 61de5bc (median
//!   38.33) and 16.82, 16.92, 16.92 s at 8991334 (median 16.92), confirmed;
//!   its account register rebuilt 25 and kept 73, write index kept 434.
//!   What it still pays for is the 48 offers read whole.
//! - void, and not counted: three earlier runs labelled 8991334 built in
//!   the plant's target, whose fingerprints named the worktree's files, so
//!   cargo ran the plant's binary; their accounts read register rebuilt 26,
//!   the plant's.
//!
//! Then the line, as the size run's was: fitted on 10, 25 and 50 members at
//! 10,000 acts, three runs each, an idle round's processor as (N - 1)(a +
//! b N), THE_CEILING.md's model term; its prediction for 100 members at
//! 2,000 acts, a point it was not fitted on in either variable, written and
//! timestamped in a commit before that point runs.
//!
//! The fit, 2026-10-08, all at e851793 (8991334's folds), load average 6 to
//! 8: the idle round's processor at 10 x 10,000 0.0030, 0.0031, 0.0028 s
//! (median 0.0030) and at 50 x 10,000 0.0284, 0.0271, 0.0277 s (median
//! 0.0277), with 25 x 10,000's median above, 0.0098. Least squares on each
//! exchange's share, c / (N - 1) = a + b N: a = 0.270 ms, b = 5.85 us, so
//! c(N) = (N - 1)(0.270 ms + 5.85 us N), residuals under 0.2 ms at all
//! three. (THE_CEILING.md's prototype, (N - 1)(0.18 + 0.009 N) ms.)
//!
//! Its prediction, 2026-10-08 04:22 PDT, before any run at that point: at
//! 100 members and 2,000 acts, a point the line was not fitted on in either
//! variable, the idle round's processor median 0.085 s, three runs,
//! confirmed if the median is within 0.064 to 0.106 s (25% either side;
//! the size run's line missed its unfitted point by 12%) and refuted
//! otherwise. That 2,000 acts rather than 10,000 changes nothing is the
//! structural claim under it: a kept fold reads no act. Its account: digest
//! kept 396, offer kept 198, register kept 101, write index kept 497, each
//! rebuilt 0 (4 (N - 1), 2 (N - 1), N + 1 and their sum, as at 10, 25 and
//! 50).
//!
//! Measured 2026-10-08, after the prediction, three runs at ae45c71, load
//! average 8 to 12: 0.1341, 0.0964, 0.1245 s (median 0.1245). Refuted: 47%
//! over the line, outside its band. The account was the one predicted in
//! all three, so every exchange answered from kept folds, as at the fitted
//! points: what the line missed is not a fold read whole.
//!
//! The hypothesis, 2026-10-08 04:27 PDT (its commit, ebaad9b, at 04:27:57;
//! that commit's subject says 04:41, a stamp not read off the clock), before
//! any run that reads it: a
//! round's processor includes the whole process's for its wall time, and
//! N idle nodes cost about N times one, a term (N q)(wall) the line lacks.
//! `quiet_cpu_per_s` at 100 members between 0.2 and 0.6 s a second, enough
//! to cover half or more of the 0.04 s the line missed over the round's
//! 0.07 to 0.11 s of wall; at 10 members under a tenth of that. Refuted if
//! it is under 0.2 at 100.
//!
//! Measured after it, one run each at ebaad9b, load average 10 to 15:
//! `quiet_cpu_per_s` 0.0001 at 10 members and 0.002 at 100. Refuted: idle
//! nodes cost the reading nothing to speak of. (Those runs' idle rounds,
//! 0.0155 s at 10 x 10,000 and 0.1184 s at 100 x 2,000.) What the line
//! missed is in the exchange: each one's share, c / (N - 1), is 0.333,
//! 0.408 and 0.565 ms at 10, 25 and 50 and 1.26 ms at 100, its slope 5.0,
//! 6.3 and then 13.8 us a member, steeper as N grows; what in an exchange
//! grows faster than N is not yet found.
//!
//! The line refitted, 2026-10-08 04:31 PDT, with that term: c / (N - 1) =
//! a + b N + c N^2 by least squares on the four medians (10, 25, 50 at
//! 10,000 acts; 100 at 2,000): a = 0.326 ms, b = 0.575 us, c = 87.3 ns,
//! within 3.3% of all four. Its prediction, before any run there: 75
//! members at 5,000 acts, a point it was not fitted on in either variable,
//! the idle round's processor median 0.064 s over three runs, confirmed if
//! within 0.048 to 0.080 s (25% either side), refuted otherwise; the
//! account digest kept 296, offer kept 148, register kept 76, write index
//! kept 372, rebuilt 0 each.
//!
//! Measured 2026-10-08, after it, three runs at cb396e7, load average 25
//! to 76 (other work on this computer): 0.0864, 0.0587, 0.0586 s, median
//! 0.0587, 8% under the line: confirmed, the account the one predicted in
//! all three. So the line that holds is the refitted one,
//! (N - 1)(0.326 ms + 0.575 us N + 87.3 ns N^2); the first, linear in its
//! share, was refuted at 100 members, and the term that refuted it is in
//! the exchange and unexplained. At 150 members the refitted line reads 0.35 s, a reading
//! for the next ring to test, not a result.
//!
//! It needs the process's own processor time, which std does not give;
//! `getrusage` through `libc` is the one way the lock already holds, and the
//! one `unsafe` block below is that call. It counts folds with a
//! `tracing-subscriber` layer of its own, interested in the folds' and the
//! round's targets alone, so the rest of the debug lines cost nothing.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::collections::BTreeMap;
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use tracing::field::{Field, Visit};
use tracing::subscriber::Interest;
use tracing::{Event, Metadata};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::Layer;

use svrngs_core::tests_support::Batch;
use svrngs_node::iroh::{PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Digest, Payload, Person, RailAct, Roster, SigningKey};

const GROUP: &str = "scale";

/// No node's own round inside a measured one.
const A_DAY: Duration = Duration::from_secs(86_400);

fn setting(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from_bytes(&signing(seed).verifying_key().to_bytes()).unwrap()
}

/// This process's processor time so far, user and system, and its peak
/// resident memory in bytes as the platform reports it (bytes on macOS,
/// kilobytes on Linux).
fn usage() -> (f64, i64) {
    // SAFETY: `getrusage` writes one `rusage` it is given a valid pointer
    // to; a zeroed `rusage` is a valid value of that plain C struct.
    let ru = unsafe {
        let mut ru: libc::rusage = std::mem::zeroed();
        libc::getrusage(libc::RUSAGE_SELF, &mut ru);
        ru
    };
    let secs = |tv: libc::timeval| tv.tv_sec as f64 + tv.tv_usec as f64 / 1e6;
    (secs(ru.ru_utime) + secs(ru.ru_stime), ru.ru_maxrss as i64)
}

fn cpu_secs() -> f64 {
    usage().0
}

fn dir_bytes(p: &std::path::Path) -> u64 {
    let mut n = 0;
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let m = e.metadata().unwrap();
            n += if m.is_dir() {
                dir_bytes(&e.path())
            } else {
                m.len()
            };
        }
    }
    n
}

/// An expense-sized act: about two hundred bytes of payload.
fn act(i: usize) -> RailAct {
    RailAct::Record {
        payload: Payload::new(serde_json::json!({
            "kind": "scale",
            "i": i,
            "amount_cents": 4720,
            "note": "propane for the house, split four ways, with a sentence of context"
        }))
        .unwrap(),
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// Every act any node holds, ingested by every node: a converged group
/// without paying a round per member to reach it. The rounds measured
/// afterwards are the steady state, not the catch-up.
async fn converge(nodes: &[Node]) {
    let mut all = Vec::new();
    for n in nodes {
        let (ops, more) = n
            .rail()
            .ops_missing_from_within(GROUP, &Digest::new(), usize::MAX)
            .unwrap();
        assert!(!more, "an unbounded budget leaves nothing behind");
        all.extend(ops);
    }
    let all = Arc::new(all);
    let mut ingests = Vec::new();
    for n in nodes {
        let rail = n.rail().clone();
        let all = all.clone();
        ingests.push(tokio::task::spawn_blocking(move || {
            rail.ingest_all(GROUP, &all).unwrap();
        }));
    }
    for i in ingests {
        i.await.unwrap();
    }
}

/// What a fold's debug line says of it.
#[derive(Clone, Copy)]
enum Says {
    /// Its words name a read of the whole journal.
    Rebuilt,
    /// Its words name an answer from a kept fold, the journal unread.
    Kept,
    /// It carries `restarted`: true when the fold began again from the top.
    Restarted,
}

/// Each fold of a group's journal a round pays for, by the line it traces
/// in svrngs-runtime: the write index (journal.rs `Held::catch_up`, which
/// the digest is now caught up by too, so it counts every catch-up,
/// whatever asked for it), the digest (`RingJournal::digest`, and the
/// offer's own catch-up of it), the offer
/// (`RingJournal::ops_missing_from_within`, its two lines: answered from
/// the whole log, or unread) and the register (register.rs
/// `RingRail::standing`, its two lines: folded from the acts that move
/// membership, or answered from the standing kept since).
const FOLDS: [(&str, &str, Says); 6] = [
    (
        "write_index",
        "ring rail: write index caught up with the log",
        Says::Restarted,
    ),
    (
        "digest",
        "ring rail: digest caught up with the log",
        Says::Restarted,
    ),
    (
        "offer",
        "ring rail: offer folded from the whole log",
        Says::Rebuilt,
    ),
    (
        "offer",
        "ring rail: offer answered unread, the peer's digest is ours",
        Says::Kept,
    ),
    (
        "register",
        "register: standing read from the seed and the group's acts",
        Says::Rebuilt,
    ),
    (
        "register",
        "register: standing answered from the kept fold, no act that moves it since",
        Says::Kept,
    ),
];

/// The targets those lines and the round's membership line are traced at.
const TARGETS: [&str; 3] = [
    "svrngs_runtime::journal",
    "register",
    "svrngs_node::ring_sync",
];

/// The line the round traces once it has listed its members and before it
/// dials any (ring_sync.rs `run_one_round`), where the hand count kills one.
const LISTED: &str = "ring sync: round membership";

/// Kept and rebuilt, per fold, since the last [`take_folds`].
static FOLDED: Mutex<BTreeMap<&str, (usize, usize)>> = Mutex::new(BTreeMap::new());

/// A round to hold at [`LISTED`]: it says so on `listed`, then waits for
/// `resume`.
struct Hold {
    listed: mpsc::Sender<()>,
    resume: mpsc::Receiver<()>,
}

static HOLD: Mutex<Option<Hold>> = Mutex::new(None);

/// One instrument reading at a time: the counts are the process's.
static ONE_AT_A_TIME: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Default)]
struct Line {
    message: String,
    restarted: Option<bool>,
}

impl Visit for Line {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "restarted" {
            self.restarted = Some(value);
        }
    }
}

struct Counter;

impl<S: tracing::Subscriber> Layer<S> for Counter {
    fn register_callsite(&self, meta: &'static Metadata<'static>) -> Interest {
        if meta.is_event() && TARGETS.contains(&meta.target()) {
            Interest::always()
        } else {
            Interest::never()
        }
    }

    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let mut line = Line::default();
        event.record(&mut line);
        if line.message == LISTED {
            if let Some(hold) = HOLD.lock().unwrap().take() {
                hold.listed.send(()).unwrap();
                hold.resume
                    .recv_timeout(Duration::from_secs(60))
                    .expect("the held round is resumed within a minute");
            }
            return;
        }
        let Some((fold, _, says)) = FOLDS.iter().find(|(_, m, _)| *m == line.message) else {
            return;
        };
        let kept = match says {
            Says::Rebuilt => false,
            Says::Kept => true,
            Says::Restarted => !line
                .restarted
                .expect("a caught-up fold's line carries `restarted`"),
        };
        let mut folded = FOLDED.lock().unwrap();
        let n = folded.entry(fold).or_default();
        if kept {
            n.0 += 1;
        } else {
            n.1 += 1;
        }
    }
}

fn count_folds() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        tracing::subscriber::set_global_default(tracing_subscriber::registry().with(Counter))
            .expect("the instrument's counter is the process's one subscriber");
    });
}

fn take_folds() -> BTreeMap<&'static str, (usize, usize)> {
    std::mem::take(&mut *FOLDED.lock().unwrap())
}

/// One round's own account of what it cost.
struct Cost {
    wall_s: f64,
    cpu_s: f64,
    out: svrngs_node::ring_sync::RoundOutcome,
    folds: BTreeMap<&'static str, (usize, usize)>,
}

impl Cost {
    /// Every member the round offered the group to is dialled, and lands in
    /// exactly one of reached, refused and unreachable (ring_sync.rs
    /// `run_one_round`, the end of its peer loop); unreachable is asleep.
    fn account(&self) -> serde_json::Value {
        let o = &self.out;
        serde_json::json!({
            "dialled": o.peers_reached + o.peers_refused + o.peers_unreachable,
            "reached": o.peers_reached,
            "asleep": o.peers_unreachable,
            "refused": o.peers_refused,
            "acts_moved": o.ops_pulled + o.ops_pushed,
            "cpu_s": self.cpu_s,
            "wall_s": self.wall_s,
            // Every fold, its lines counted: a fold that traced no line in
            // this round reads kept 0, rebuilt 0. A fold with two lines is
            // named twice, and answers the same both times.
            "folds": FOLDS.iter()
                .map(|(f, _, _)| {
                    let (kept, rebuilt) = self.folds.get(f).copied().unwrap_or_default();
                    (f.to_string(), serde_json::json!({"kept": kept, "rebuilt": rebuilt}))
                })
                .collect::<serde_json::Map<_, _>>(),
        })
    }
}

async fn timed_round(node: &Node) -> Cost {
    take_folds();
    let (c, t) = (cpu_secs(), Instant::now());
    let out = tokio::time::timeout(Duration::from_secs(3600), node.round())
        .await
        .expect("a round finishes within the hour");
    let (wall_s, cpu_s) = (t.elapsed().as_secs_f64(), cpu_secs() - c);
    Cost {
        wall_s,
        cpu_s,
        out,
        folds: take_folds(),
    }
}

/// A converged group of `members` real nodes, `acts` record acts written
/// in equal shares, each node rounding once a day.
async fn stand_up(members: usize, acts: usize) -> (Vec<tempfile::TempDir>, Vec<Node>) {
    let dirs: Vec<_> = (0..members).map(|_| tempfile::tempdir().unwrap()).collect();
    let no_relay = RelayConfig::from_parts(Vec::new(), Some("none")).unwrap();
    let mut nodes = Vec::new();
    for (i, d) in dirs.iter().enumerate() {
        nodes.push(
            Node::start_every(
                d.path().to_path_buf(),
                signing(i as u8 + 1),
                &no_relay,
                |_| Ok(None),
                "127.0.0.1:0".parse().unwrap(),
                A_DAY,
            )
            .await
            .expect("a node starts"),
        );
    }
    let founded = Roster::new(
        [(Person::parse("p1").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    for node in &nodes {
        node.rail().set_seed(GROUP, &founded).unwrap();
    }
    let admits: Vec<RailAct> = (2..=members)
        .map(|i| RailAct::Admit {
            person: Person::parse(&format!("p{i}")).unwrap(),
            key: actor_of(&signing(i as u8)),
            invite: None,
        })
        .collect();
    nodes[0]
        .rail()
        .commit(GROUP, &Batch(admits))
        .unwrap()
        .unwrap();
    for (i, a) in nodes.iter().enumerate() {
        for (k, b) in nodes.iter().enumerate() {
            if i != k {
                a.transport()
                    .add_known_peer(key(k as u8 + 1), loopback_sockets(b.endpoint()));
            }
        }
    }
    converge(&nodes).await;
    // One act each, converged, so every later act's view names everyone.
    for (i, node) in nodes.iter().enumerate() {
        node.rail()
            .commit(GROUP, &Batch(vec![act(i)]))
            .unwrap()
            .unwrap();
    }
    converge(&nodes).await;
    let share = acts / members;
    let mut writes = Vec::new();
    for node in &nodes {
        let rail = node.rail().clone();
        writes.push(tokio::task::spawn_blocking(move || {
            let mut left = share;
            while left > 0 {
                // One commit per five hundred: a batch shares one stamp, so
                // writing fast does not run the clock ahead (DESIGN §6).
                let k = left.min(500);
                rail.commit(GROUP, &Batch((0..k).map(act).collect()))
                    .unwrap()
                    .unwrap();
                left -= k;
            }
        }));
    }
    for w in writes {
        w.await.unwrap();
    }
    converge(&nodes).await;
    let digest = nodes[0].rail().digest(GROUP).unwrap();
    for node in &nodes[1..] {
        assert_eq!(
            node.rail().digest(GROUP).unwrap(),
            digest,
            "every copy holds the same journal before anything is measured"
        );
    }
    (dirs, nodes)
}

/// Three rounds of node 0 with nothing to move: their wall and processor
/// times, and the last one's cost. A round that did not reach every member,
/// or moved an act, is not an idle round, so it is no reading.
async fn idle_rounds(nodes: &[Node]) -> (Vec<f64>, Vec<f64>, Cost) {
    let (mut wall, mut cpu, mut last) = (Vec::new(), Vec::new(), None);
    for _ in 0..3 {
        let cost = timed_round(&nodes[0]).await;
        let out = &cost.out;
        assert_eq!(
            out.peers_reached,
            nodes.len() - 1,
            "could not judge: every member reached: {out:?}"
        );
        assert_eq!(
            out.ops_pulled + out.ops_pushed,
            0,
            "could not judge: an idle round moves nothing: {out:?}"
        );
        wall.push(cost.wall_s);
        cpu.push(cost.cpu_s);
        last = Some(cost);
    }
    (wall, cpu, last.expect("three idle rounds ran"))
}

/// The ceiling (DESIGN §8 :1083): at 25 members and 10,000 acts an idle
/// round costs under this much processor, the median of three.
const CEILING_CPU_S: f64 = 0.1;

/// The ceiling's canary (ceiling-4), the one scale number
/// `scripts/check.sh` holds, run there by name, in release. Processor, not
/// wall time: an asleep member's ten-second timeout is DESIGN §9's
/// dial-at-once row, not this bar. A round that could not be read panics in
/// [`idle_rounds`] as could-not-judge, never a pass with a smaller number.
#[tokio::test(flavor = "multi_thread", worker_threads = 12)]
#[ignore = "the ceiling's canary: run by name, in release, by scripts/check.sh"]
async fn an_idle_round_is_under_the_ceiling() {
    let _alone = ONE_AT_A_TIME.lock().await;
    count_folds();
    let (_dirs, nodes) = stand_up(25, 10_000).await;
    let (_, cpu, last) = idle_rounds(&nodes).await;
    let idle = median(cpu.clone());
    println!(
        "CEILING idle round at 25 x 10,000: {idle:.4} s of processor, the median of {cpu:?}, \
         against {CEILING_CPU_S} s; the last round's account {}",
        last.account()
    );
    assert!(
        idle < CEILING_CPU_S,
        "an idle round at 25 x 10,000 cost {idle} s of processor, over the ceiling of \
         {CEILING_CPU_S} s: {cpu:?}"
    );
    for node in &nodes {
        node.endpoint().close().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 12)]
#[ignore = "an instrument, not a check: run by hand, in release (see the module docs)"]
async fn scale() {
    let _alone = ONE_AT_A_TIME.lock().await;
    count_folds();
    let members = setting("SVRNGS_SCALE_MEMBERS", 5);
    let acts = setting("SVRNGS_SCALE_ACTS", 1000);
    let asleep = setting("SVRNGS_SCALE_ASLEEP", 0).min(members - 1);
    assert!(
        (2..=250).contains(&members),
        "a key per member is seeded from one byte"
    );
    let started = Instant::now();
    let (_dirs, nodes) = stand_up(members, acts).await;
    let digest = nodes[0].rail().digest(GROUP).unwrap();
    let setup_s = started.elapsed().as_secs_f64();

    let (held, _) = nodes[0]
        .rail()
        .ops_missing_from_within(GROUP, &Digest::new(), usize::MAX)
        .unwrap();
    let journal_acts = held.len();
    let journal_bytes = dir_bytes(&nodes[0].rail().group_dir(GROUP).unwrap());
    let digest_bytes = serde_json::to_vec(&digest).unwrap().len();
    let digest_pass_s = median(
        (0..5)
            .map(|_| {
                let t = Instant::now();
                nodes[0].rail().digest(GROUP).unwrap();
                t.elapsed().as_secs_f64()
            })
            .collect(),
    );
    let append_s = median(
        (0..5)
            .map(|i| {
                let t = Instant::now();
                nodes[0]
                    .rail()
                    .commit(GROUP, &Batch(vec![act(i)]))
                    .unwrap()
                    .unwrap();
                t.elapsed().as_secs_f64()
            })
            .collect(),
    );
    // The five appends put node 0 ahead: its next round is one write
    // reaching everyone.
    let push = timed_round(&nodes[0]).await;

    let (idle_wall, idle_cpu, idle) = idle_rounds(&nodes).await;
    // What the process spends with no round running, every node idle: a
    // round's processor reading includes it for as long as the round takes.
    let c = cpu_secs();
    tokio::time::sleep(Duration::from_secs(2)).await;
    let quiet_cpu_per_s = (cpu_secs() - c) / 2.0;

    let mut asleep_round = None;
    let mut rounds = serde_json::json!({
        "push": push.account(),
        "idle": idle.account(),
    });
    if asleep > 0 {
        for node in &nodes[members - asleep..] {
            node.endpoint().close().await;
        }
        let cost = timed_round(&nodes[0]).await;
        asleep_round = Some((cost.wall_s, cost.out.peers_unreachable));
        rounds["asleep"] = cost.account();
    }

    println!(
        "SCALE {}",
        serde_json::json!({
            "members": members, "acts": acts, "asleep": asleep,
            "setup_s": setup_s,
            "journal_acts": journal_acts,
            "journal_bytes": journal_bytes,
            "bytes_per_act": journal_bytes as f64 / journal_acts as f64,
            "digest_bytes": digest_bytes,
            "digest_pass_s": digest_pass_s,
            "append_s": append_s,
            "push_round_s": push.wall_s,
            "push_pushed": push.out.ops_pushed,
            "idle_round_wall_s": median(idle_wall),
            "idle_round_cpu_s": median(idle_cpu),
            "quiet_cpu_per_s": quiet_cpu_per_s,
            "asleep_round": asleep_round,
            "rounds": rounds,
            "max_rss": usage().1,
        })
    );
    for node in &nodes[..members - asleep] {
        node.endpoint().close().await;
    }
}

/// The account against a count made by hand from the code, on a round
/// small enough to count: four members, node 0 two acts ahead, and member
/// 3 killed after the round has listed it and before anything is dialled,
/// so the round dials a member that was awake when it began.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_account_matches_a_hand_count() {
    let _alone = ONE_AT_A_TIME.lock().await;
    count_folds();
    let (_dirs, nodes) = stand_up(4, 8).await;
    nodes[0]
        .rail()
        .commit(GROUP, &Batch(vec![act(100), act(101)]))
        .unwrap()
        .unwrap();

    // The killer runs on a worker; the round runs here and is held at its
    // membership line until member 3's computer is shut.
    let (listed_tx, listed) = mpsc::channel();
    let (resume, resume_rx) = mpsc::channel();
    *HOLD.lock().unwrap() = Some(Hold {
        listed: listed_tx,
        resume: resume_rx,
    });
    let doomed = nodes[3].endpoint().clone();
    let killer = tokio::spawn(async move {
        tokio::task::spawn_blocking(move || listed.recv())
            .await
            .unwrap()
            .expect("the round lists its members");
        doomed.close().await;
        resume.send(()).unwrap();
    });
    let cost = timed_round(&nodes[0]).await;
    killer.await.unwrap();
    assert!(
        HOLD.lock().unwrap().is_none(),
        "the round passed its membership line, so member 3 was killed mid-round"
    );

    let mut account = cost.account();
    let (cpu_s, wall_s) = (account["cpu_s"].take(), account["wall_s"].take());
    assert!(
        cpu_s.as_f64().unwrap() > 0.0 && wall_s.as_f64().unwrap() > 0.0,
        "{account}"
    );
    // Counted by hand. Each reached member (1 and 2) takes two chunks
    // (ring_sync.rs `exchange`): in chunk 1 it answers with node 0's six
    // acts it holds, node 0's window travelling whole because node 0's head
    // is past its mark (svrngs-core sync.rs `ops_missing_from_within`, "a
    // mismatching window travels as one atomic per-actor unit"), which node
    // 0 already holds, and node 0 pushes its two new acts; chunk 2 moves
    // nothing and converges. Member 3 fails at chunk 1's first post.
    // Every fold below is kept: each node's journal index was caught up by
    // `stand_up`'s last digests, node 0's again after its two acts, and
    // nothing replaces a file. An empty ingest returns before it folds.
    // - digest: node 0 asks it for each chunk's first call, and again
    //   before chunk 1's push, 3 per reached member, 1 for member 3; each
    //   member answers each of its 3 posts with one (`answer`); and every
    //   offer below catches it up first (journal.rs
    //   `ops_missing_from_within`): 3 x 2 + 1 + 3 x 2 + 10 = 23.
    // - offer: node 0 once a chunk, 2 per reached member; each member once
    //   per post, 3. Unread when the peer's digest is ours: node 0's in
    //   chunk 2, and the member's at posts 2 and 3, once node 0's two acts
    //   landed; read whole at chunk 1, the digests apart: kept 3 x 2,
    //   rebuilt 2 x 2.
    // - write_index: every digest catch-up above, 23, the ingests that
    //   fold: node 0 finds the six acts held, once per reached member
    //   (`ingest_all`); each reached member catches up before and after
    //   landing the two (`ingest_all`, `Held::after_write`): 23 + 2 + 2 x 2
    //   = 29; and every register read below catches up first
    //   (journal.rs `RingJournal::standing`), 10: 39.
    // - register: node 0 lists its members (node.rs `standings`) and then
    //   offers the group to them (`run_one_round`); each member admits node
    //   0's one connection (node.rs `admit_dialer`) and checks each of 3
    //   posts (ring_routes.rs `standing_refusal`): 2 + (1 + 3) x 2 = 10.
    //   Each node's first read folds (nothing asked its register before
    //   the round) and the rest are kept: the two acts node 0 pushes are
    //   records, which move no membership: kept 1 + 3 x 2, rebuilt 3.
    let hand = serde_json::json!({
        "dialled": 3, "reached": 2, "asleep": 1, "refused": 0,
        "acts_moved": 4,
        "cpu_s": null, "wall_s": null,
        "folds": {
            "write_index": {"kept": 39, "rebuilt": 0},
            "digest": {"kept": 23, "rebuilt": 0},
            "offer": {"kept": 6, "rebuilt": 4},
            "register": {"kept": 7, "rebuilt": 3},
        },
    });
    assert_eq!(account, hand, "the round's account against the hand count");
    for node in &nodes[..3] {
        node.endpoint().close().await;
    }
}
