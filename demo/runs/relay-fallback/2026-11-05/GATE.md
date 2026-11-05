# The relay-fallback gate

Command, under the shared lock and `CARGO_INCREMENTAL=0`:

```
env CARGO_INCREMENTAL=0 lockf -k /tmp/svrngs-gate.lock scripts/check.sh
```

## Run 1 — 2026-11-05 at c2248833: exit 1 (26m07s)

Log: `ralph/next/relay-fallback/ctl/log/sessions/REVIEW-audit-relay-fallback-2.await.log`
(the unit's `ralph-result await 6h` run). Steps before `tests` passed
(principles, lean, trust floor, advisories, app nouns, lint, format, page
laws, rustfmt, clippy, canaries, web rustfmt/clippy, types, typed lint,
joining, pages, scene doc/events, the other scenes, conformance, app laws).

| step | verdict | why |
| --- | --- | --- |
| tests | failed (211 s) | 7 of `apps/give/tests/check.rs` never ran: `podman info` exited 125, the podman machine was down. Not a code failure. |
| ceiling canary | failed | still running at its 300 s limit: the release profile was cold. |
| browser | failed (250 s) | `a_phone_s_socket_opens_again_once_its_lender_answers_again::webkit`: a real regression in the delta, below. |
| scenes | 3 never-ran | `give`, `give_gone`, `give_move` (chromium): podman, as above. The rest passed. |
| rate | printed | WebKit seeks back to back: `wildcard sam` missed a rate; not a gate failure (the run's note: 2 of 4 ended above 12 cores). |

### The repair

1. Host: `podman machine start` (the machine had been down 4 days; no code
   change). `cargo test -p give --test check` then passed 8/8 in 74.06 s;
   the three give scenes passed in 100.79 s.
2. Host: the release profile built warm before the rerun
   (`cargo test --release -p svrngs-node --test scale --no-run`, 2m13s);
   the canary then read 0.0297 s of processor and passed in 4.34 s.
3. Code, 96a40bc7: the regression. Watched red at both ends: the test at
   baseline f8a57035 passed (41.86 s), at HEAD it failed twice (33.59 s
   rerun); taking the named relay again in `reach` made it pass (43.67 s).
   Cause: `reach` read the page's policy in the lend's worker, where there
   is no document, so every HTTP relay was refused and `bound.relays`
   silently substituted for the lender's own relay — the route the test's
   lid does not cut. Now a pass's holder takes the relay the page handed
   it, and the page filters what it hands per lender (`relay_taken`,
   `hand.js` `passLends`). Green: the socket test 78.26 s; the
   published-policy join 74.95 s; all six `join_page` tests 46.89 s;
   `relay_fallback` and `relay_tiers` 2+2; the whole browser suite 98
   passed, 2 ignored, 250.07 s. Two preserving cuts landed first, in their
   own commits (98c06cad, 927057a2), so the repair sits inside the cap:
   `lean_check.py` exit 0 at `.` 19911/19913.

## Run 2 — pending

The unchanged command above is started as this unit's `ralph-result await`
run after the repair commits; the loop owns its lifetime. Its exit and step
table land here when it ends; this file and `AUDIT.md`'s "Automated" line
are completed with them.
