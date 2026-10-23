//! The demo scenes, played (capability-scenes; the operator, 2026-10-23:
//! "Part of what keeps the platform accountable is that it keeps
//! supporting these use cases with hi fidelity"): each scene of `demo/`
//! end to end, every person a `svrngs` of this build with a `HOME` of
//! their own ([`people`]), in the scene's group, through the real door,
//! relay and record, in Playwright's Chromium and in its WebKit as an
//! iPhone. Each scene says, in its own words, what each person sees, and
//! the one invariant it relies on. `scripts/check.sh` runs this binary
//! in its "scenes" step, each scene's verdict on its own line.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
#[path = "../../../svrngs-node/tests/browser/drive.rs"]
mod drive;
mod people;

// Money (apps/expenses): a split, a settle, another's correction.
mod money;
// Events (demo/events.md): a dinner for two groups, each its own answers.
mod events;
// House rules (demo/house-rules.md): a rule agreed holds after one leaves.
mod house_rules;
// Grapevine (demo/grapevine.md): an ask carried a ring away, sealed replies.
mod grapevine;
