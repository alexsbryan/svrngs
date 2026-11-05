<!-- section: relay-fallback after=intro -->
Read this queue's CHARTER.md and STATE.md before any work. The operator approved
the simplified connect-first scope and asked to start this loop. Keep marching
through the assigned unit; another step boundary is not a new approval gate.
The one technical stop is transport feasibility; a budget/cap decision remains
the operator's under CHARTER, not a reason to invent a workaround.

This is a dedicated clean worktree based on the demo candidate. Do not import
main's unrelated WIP or checkpoint another worktree's dirty files. Worker scope
is this worktree plus synthetic fixtures. Do not stop/restart the live keeper,
touch its data, operate the Pi, or use public deployment credentials. No push,
main merge, dependency, new transport stack, CSP/trust-floor widening or release.

Keep the source estimate honest: at most +150 product lines (120 Rust, including
<=10 core, and 30 JS) and <=150 added test code lines; no scope hidden in demos
or generated helpers. Inventory before new modules; preserve cuts in their own
commits and lower caps. Do not pre-raise caps to the allowance. If an actual
post-pass landing needs a cap change, present measured counts and a ready diff
through ralph-result needs-human rather than changing the cap yourself.

The known failure is already reproduced against the real public page. Read its
private artifact if useful, but commit only synthetic/redacted evidence. A test
that relaxes CSP to admit loopback HTTP cannot prove this correction. A test
that checks only relay /ping cannot prove issuer reachability. Do not hardcode
another list of n0 servers in the browser. Keep one endpoint per key; cancel
underlying losing work, not merely the JavaScript wait for it.

Use CARGO_INCREMENTAL=0. Run every browser test and the final full gate under
lockf -k /tmp/svrngs-gate.lock. There is currently about 49 GiB free on this Mac;
inspect build-cache reuse before expensive builds and never remove another
worktree's artifacts. The final audit owns the unchanged full gate; intermediate
units use their assigned focused checks, with no weaker final acceptance.

<!-- section: done -->
For fallback-proof and fallback-connect, done means the assigned focused checks,
red/green evidence, counts and commits are complete, within CHARTER. Do not claim
the final product gate passed at an intermediate close. REVIEW-audit-relay-fallback
must run the unchanged full scripts/check.sh under the shared lock, exit 0.
Then run ralph-result done as its own shell command. Long checks use
ralph-result await <budget> -- <command>; the loop owns their lifetime. An
operator-only blocker gets a committed package and ralph-result needs-human.
