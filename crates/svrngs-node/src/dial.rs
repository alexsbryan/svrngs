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
//!
//! **2 s, from the measurement, by a rule written before it**
//! (`demo/runs/asleep/2026-10-14/PREDICTIONS.md`, "The cap"): the slowest
//! working dial on this host, 347 ms (a computer's first dial, before iroh
//! had chosen a path; a phone's through n0's relay took 66–103 ms), times 3,
//! rounded up to a whole second. The margin is for a slower path than this
//! host's: a relay farther than the one n0 gives it, a busier one, a loaded
//! computer. Cellular was not measured, and is the operator's: a dial
//! slower than this is said away when it would have answered.

/// How long a dial to a lend's computer waits for its handshake before the
/// computer is taken as asleep.
pub const DIAL_WITHIN: std::time::Duration = std::time::Duration::from_secs(2);
