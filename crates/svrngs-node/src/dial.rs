// SPDX-License-Identifier: AGPL-3.0-or-later
//! How long a dial to a lend's computer waits to be answered (events-asleep):
//! one constant for both dials, the transport's bridges a running computer
//! dials every member's through (`iroh.rs`, `IrohTransport::base_url`) and a
//! phone's page's dial to a lender (`web/src/lib.rs` `Endpoint::dial` and
//! `dial_lend`; the page compiles this file by `#[path]`). An asleep
//! computer answers no dial, and iroh gave one up after 30 s on both sides,
//! so a person was told only then (`demo/runs/asleep/2026-10-14/README.md`,
//! 3 and 4). A joiner's dial to the computer its link names is not capped:
//! the measurement did not cover it, and a computer just started answered
//! its first in 3.03 s (`crates/svrngs/tests/home_group.rs`, at 883fca8).
//! A member's page's dial to that computer is, by [`LINK_WITHIN`].
//!
//! **4 s, from the measurements, by a rule written before them**
//! (`demo/runs/asleep/2026-10-14/PREDICTIONS.md`, "The cap"): the slowest
//! working dial on this host, times 3, rounded up to a whole second. The
//! measurement's slowest was 347 ms (a computer's first dial, before iroh had
//! chosen a path; a phone's through n0's relay took 66–103 ms), which gave
//! 2 s; the capped take's was 1,013 ms, a computer's first dial to one just
//! started, under a load of 12–13.6 on 12 cores (that README, "What the
//! capped take found", 1), which gives 4 s (the operator, 2026-10-14: "4 s,
//! by the same rule"). The margin is for a slower path than this host's: a
//! relay farther than the one n0 gives it, a busier one, a loaded computer.
//! It is not for a phone on cellular, a second host or a release build, none
//! of which was measured: those are the operator's, and a dial slower than
//! this is said away when it would have answered.

/// How long a dial to a lend's computer waits for its handshake before the
/// computer is taken as asleep.
pub const DIAL_WITHIN: std::time::Duration = std::time::Duration::from_secs(4);

/// How long a phone's page waits for the computer that made its link to
/// answer its dial as a member (v1ui-reach (b); `web/src/lib.rs`
/// `connect`), before it reaches the group through another person's.
/// **10 s, by [`DIAL_WITHIN`]'s rule, written before the measurement**
/// (`demo/runs/reach/2026-10-14/`, PREDICTIONS.md, "The cap"): the slowest
/// working dial, 3,057 ms, times 3, rounded up to a whole second; the
/// constant above did not fit. That dial was to a computer just woken,
/// through a relay on this host: all five such took 3,047-3,057 ms, where a
/// warm one took 9-24 ms there and 73-187 ms through n0's; asleep, iroh gave
/// one up at 30 s (that README). Cellular, a second host and a release build
/// were not measured, and are the operator's. The joiner's redeem dial stays
/// uncapped (5c097eb).
pub const LINK_WITHIN: std::time::Duration = std::time::Duration::from_secs(10);

/// How a phone's page holds its dial to a lender (events-wait (b)): given up
/// once the computer has sent nothing for [`DIAL_WITHIN`], as a dial it does
/// not answer is, with a keep-alive every quarter of that, so a computer that
/// answers is never that silent: three keep-alives may go unanswered and the
/// fourth still keep it. iroh's own are 30 s and 5 s, so a phone holding a
/// dial to a computer that froze, as a lid closing, was told only after 30 s
/// (`demo/runs/wait/2026-10-14/`, the plant take). The page's alone: a
/// computer's bridge dials anew for each request, so the cap bounds it.
pub fn held() -> iroh::endpoint::QuicTransportConfig {
    let idle = iroh::endpoint::VarInt::from_u32(DIAL_WITHIN.as_millis() as u32);
    let held = iroh::endpoint::QuicTransportConfig::builder().max_idle_timeout(Some(idle.into()));
    held.keep_alive_interval(DIAL_WITHIN / 4).build()
}
