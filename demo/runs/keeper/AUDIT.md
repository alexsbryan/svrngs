# REVIEW-audit-keeper: the ledger

The whole delta from `e5d9813b` (CHARTER.md "Baseline") to `8bddc92f`.

| Check | Verdict | Where |
|---|---|---|
| Full `scripts/check.sh`, svrngs-gate container | passed at 8bddc92f | "The gate", below |
| Ported-from lines listed by `scripts/ports.py` | passed | both forked files, below |
| Trust floor unchanged | passed | `git diff e5d9813b 8bddc92f -- scripts/trust_floor.txt`: empty |
| CSP unchanged | passed | no `+`/`-` line of `git diff e5d9813b 8bddc92f` names `Content-Security-Policy` or a `*-src` |
| Counts within each allowance | passed | "Counts", below |

## The gate

**1. Failed, and not on the code.** The worktree's `.git` points into
`../svrngs/.git`, which was outside the container's `/work` mount. Git in
the container said "not a git repository", so the lean check counted
untracked build output (24,665 of 24,655) and the trust floor took its own
file for uncommitted. That failed lean and RUST_ARCH, the trust floor, page
laws and script laws. The repair was to the mount: the main `.git` was
mounted at its own path (`/tmp/keeper-audit/gate.sh`). No code changed.

**2. 8bddc92f: passed**, exit 0 after 2,143 s, on a clean tree
(ctl/log/sessions/REVIEW-audit-keeper-3.await.log, sha256 21a8b063…b011).
`grep -cE '^(failed|could-not|never)'` on the log gives 0: every step
passed, including tests (1,078 s), browser and the 18 scenes. The
conformance canary's two could-not-judge rows ("its worker, after svrngs's
restarts", log lines 78 and 93) are a WebKit limit, on both origins. Each
sits beside a passed row of the same check. The prefetch printed "the
cr-sqlite source or its crates did not fetch". It runs before
`check.sh`, which built and passed the tests anyway.

## Ported from commonwealth-ai

`scripts/ports.py`, exit 0:

    apps/svrngs-keeper/src/systemd/tests.rs:1  <-  shared/crates/host-kit/src/service/tests.rs@8ed8d99b0  unchanged; here: changed in 1 commit(s)
    apps/svrngs-keeper/src/systemd.rs:2  <-  shared/crates/host-kit/src/service.rs@8ed8d99b0  unchanged; here: changed in 3 commit(s)

These are the only `ported from commonwealth-ai` lines the branch adds.

## Counts

| | Baseline | After the pass | Now (cap) | Allowance |
|---|---|---|---|---|
| core | 3,465 | | 3,465 (3,539) | unchanged |
| all code but JavaScript | 24,517 | 24,475 | 24,655 (24,655) | +180 over the pass: +180 |
| JavaScript | 8,693 | 8,682 | 8,688 (8,688) | +30 over the pass: +6 |

The pass and the renewal raise are recorded in `scripts/lean_check.py`'s
`BUDGET` comment.
