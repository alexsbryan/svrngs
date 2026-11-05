# The relay-fallback queue's final audit

Unit `REVIEW-audit-relay-fallback` (ralph/next/relay-fallback/STATE.md:26),
audited 2026-11-05 at 98f12717 from baseline
f8a57035206be60212d283493ffd3a944976fbdd. The product delta is
f8a57035..44453bc7: 22 commits, 37 files, +2027/-284. 98f12717 (after the
landing) changes only `ralph/next/relay-fallback/PROMPT.addendum.md`: the
operator's 450 test allowance and the direction not to probe outside this
worktree; no product or doc file. Every count below was measured at HEAD by
this audit with the repository's own tools; the recorded red/green runs are
the ones each commit body cites, on this tree.

## Verdicts, kept apart

- **Automated: passed.** The unchanged full gate's second run (2026-11-05,
  `ralph-result await 6h` from db292099) exited 0 in 24m01s (1441 s): every
  step passed, no never-ran, no could-not-judge — the browser suite
  (214 s), `join_page` including the published-policy cold-start join and
  the lent `shelf` opening (96 s), the fourteen scenes (292 s), the
  conformance canary (141 s), the ceiling canary at 0.0268 s against 0.1 s.
  `GATE.md` beside this file records both runs and their step tables. The
  first run (2026-11-05, exit 1) found one real regression the delta had
  caused (`a_phone_s_socket_opens_again_once_its_lender_answers_again::webkit`,
  fixed in 96a40bc7 after two preserving cuts, 98c06cad and 927057a2), a
  cold release profile at the ceiling canary, and the podman machine down
  (give's tests and three scenes never ran); all repaired before run 2. A
  gate that did not exit 0 is a repair owed, not a pass.
- **Real device: never-ran.** `HUMAN-relay-fallback-phone` (STATE.md:27) is
  the operator's physical scene: a phone joins from the shared link with the
  Pi's HTTP-only primary and the public fallback online, opens the mesh-side
  model, and native mesh computers sync a fresh contribution after the uplink
  cut. No worker may tick that row or publish a build to obtain it, and none
  did. An HTTP-only local relay cannot carry the phone after the cut; nothing
  here claims offline mobile support.

## Budget, measured at HEAD

Counted as lean_check.py counts: nonblank, noncomment.

| what | allowance | landed |
| --- | --- | --- |
| product, all | +150 | **+79** |
| — core Rust (`crates/svrngs-core`) | ≤ +10 | **+5** (`src/relay.rs`) |
| — native Rust (`crates/svrngs-node` product) | (in +120) | **+0** (`tiers.rs`, `dial.rs`, `home/page/group.js`) |
| — browser Rust (`web/src/lib.rs`) | (in +120, ≤ +80 slice) | **+74** |
| — JavaScript product | ≤ +30 | **+0** (`authority.js`, `lend.js`, `hand.js`, `visit.js`, `apps/…/other.js`) |
| tests, all | ≤ +450 (operator, 2026-11-05) | **+436** |
| — `crates/svrngs-node/tests/relay_fallback.rs` | | +179 |
| — `crates/svrngs/tests/join_page.rs` | | +243 |
| — `crates/svrngs-node/tests/relay_tiers.rs` | | +3 |
| — `crates/svrngs-core/src/relay_tests.rs` | | +7 |
| — `web/tests/join.mjs`, `web/tests/page.mjs` | | +2, +2 |
| product docs (`docs/DESIGN.md`, `docs/PLAN.md`) | ≤ +30 | **+16** (+8, +8) |

`python3 scripts/lean_check.py` exit 0 at HEAD: core 3491/3494,
`. ` 19911/19913, js 8187/8188, swift 0/1 (baseline 3486/3494, 19832/19833,
8187/8188, 0/1). The one `lean_check.py` change in the delta is the operator's
cap decision (fb0d5fb7): `.` 19833 → 19913, one past where the landing then
sat; the commit gives the reason and the queue's test allowance rose 150 → 450
after the measured 353. No other cap changed; no cap was pre-raised. The
gate's browser failure cost 3 lines to repair; two preserving cuts first
gave back 4 (98c06cad, 927057a2), so the repaired landing sits 2 under the
cap and the cap is unchanged.

## Plant ledger: every new check, its plant, its red and green

All eight new checks name their plant in the test's own doc comment; each red
run is the one the landing commit pasted.

| check (file) | plant, watched red | green |
| --- | --- | --- |
| `relay_fallback.rs::a_fallback_home_reaches_the_issuer_and_the_local_primary_takes_over` | fallback removed from Sam before the join: `Err(IssuerAway)` (53c290f9) | 20.5 s |
| `relay_tiers.rs::a_group_meets_at_the_builds_fallback_while_it_answers` | own tier preferred while the build's answers: "not homed at … within 56 s" (b6db94fd) | 24.1 s |
| `relay_tests.rs::a_group_meets_at_the_builds_fallback_while_it_answers` | `decide`'s arms swapped: first row failed (b6db94fd) | 7 core relay tests |
| `join_page.rs::a_browser_under_the_published_policy_joins_through_the_public_fallback` | `this_page_takes` true whatever the policy: HTTP relay bound, lend did not open (a44164cd) | 29.6 s |
| `join_page.rs::a_route_that_answers_but_cannot_reach_the_issuer_does_not_win` | `reach` dialling `routes.first()` alone: browser exited 1 (adc262fe) | 34.7 s, all four |
| `join_page.rs::a_stalled_eligible_primary_does_not_delay_the_fallback` | same plant: failed after 56.20 s (097f0986; re-watched in bf90c408) | 55.43 s |
| `relay_fallback.rs::a_fresh_contribution_syncs_after_the_public_reach_is_cut` | `decide` kept `Fallback` when it does not answer: "not homed at A within 56 s" (73bc8bc5) | 23.36 s, 2 passed |
| `join_page.rs::every_route_down_fails_bounded_and_closes_the_endpoint` | the unbounded `ep.online()` wait: 60315 ms (PATIENCE), no refusal (bf90c408) | 10269 ms, refusal line |

Pre-existing checks the row names, still green in the landings' runs: the name
refusal asked again, exactly one `Admit` on a join and none on reopening
(`a_phone_joins_by_the_link_sam_printed_and_is_in_again_without_asking`), and
the pages' laws (`web/tests/fallback.test.mjs`). The gate's own first run
found one more, red at c2248833 and green at 96a40bc7: the pre-existing
`a_phone_s_socket_opens_again_once_its_lender_answers_again::webkit`
(failed: the socket opened while the lid was shut, 33.59 s; green 78.26 s).
Its plant is `reach` reading the page's policy in a lend's worker, where
there is no document: the named relay was refused and `bound.relays` put in
its place (GATE.md).

## The incident reproduction, synthetic

c7ac2460, `demo/runs/relay-fallback/2026-11-05/`: a plain-HTTP relay on
127.0.0.1 (the Pi's stand-in) and a cold keeper; the built page served under
the published policy (`web/index.html:51`, `connect-src 'self' wss: https:`)
records 18 CSP refusals at `http://127.0.0.1:<port>/ping`, no name form, and
"The computer that sent this link did not answer. Try again later."
(`data/repro.json`). The control, the same page with the tests' loopback
allowance alone, reaches "Join house": the failure is the policy, not the
keeper, and the control cannot prove the correction. Only synthetic, redacted
evidence is committed (`keeper.out` carries `<the token>`; the relay is
loopback).

## Invariants

- **One policy owner.** The build's relay set is `cli::SHIPPED`
  (crates/svrngs/src/cli.rs:281), made into a `Meeting` with the record's own
  relays (`Meeting::of`, crates/svrngs-core/src/relay.rs:103); `decide`
  (:149) is the one decision and `tiers::Host::decide` the one place it moves
  the endpoint. The browser carries routes only from the link's token
  (`bind_for`, web/src/lib.rs) and hardcodes no relay list — no n0 name
  appears in `web/` outside comments. The token format is unchanged;
  `Meeting::carried` (:166) puts the issuer's home first, then the group's
  own, at most two.
- **Keeper fallback readiness.** `tiers::check` asks the build's fallback
  every `ASKED_EVERY` (10 s), each within `ANSWERS_WITHIN` (3 s), and decides
  on the answer; `Held::built` starts at the fallback; a build with no relay
  answers nothing (`decide`'s guard), so a sovereign node stays on its own.
  `at_fallback` still tells the home page why, in the new words
  (`home/page/group.js`).
- **Browser compatibility, deadlines, cleanup.** `this_page_takes` skips a
  plain-HTTP route on the published page before any request is issued (the
  production test asserts no CSP/mixed-content line and no line naming the
  HTTP relay); `bind` waits `dial::LINK_WITHIN` (10 s, the existing constant)
  for any relay, closes the endpoint, and refuses in one traced line; `reach`
  races the taken routes in a `FuturesUnordered` and returns only a
  connection to the expected key, dropping the losing tries with the set;
  `Dialled::close` closes the endpoint it bound or its connection.
- **Holder handoff.** `relays_of` (web/src/lib.rs) hands a pass's holder
  every taken route; `hand.js` puts them in the hand-off, `authority.js`
  carries the field (`OPEN_FIELDS`), `lend.js` binds the worker with them, so
  an app's worker dials the fallback and not a route the page refuses;
  `visit.js` keeps the first taken route, never the refused one.
- **Identity and refusal.** A dial carries the expected computer's key (the
  link's issuer, or the named computer) and iroh authenticates the
  connection as that key; a route that answers but cannot carry the issuer
  does not win (its own test). Refusal, name and expiry paths are untouched
  by the delta and their tests stay green.
- **One endpoint per key; no duplicate semantic writes.** One `bind` per key
  (page memoized, holder per worker, `RelayMode::custom(relays)` on one
  endpoint); the winner alone carries the join, and the pre-existing test
  asserts one `Admit` and none on reopening.

## Preserving cuts, in their own commits

7efc3e55 (the economizing survey; every remaining item live, callers listed),
ab783c5e (`relay_fallback.rs` −22: shared plumbing, control round removed;
check, plant and assertion unchanged), b0ba7e8c (`join_page.rs` −104: the
shared `phone`/`finished`/`phone_joining` drivers), 38d4bc0c (`join_page.rs`
−23: shared `sam_meeting_at`), d6aecb58 and c4e500c1 (`web/src/lib.rs` −5, −2:
refusals in rustfmt's own form, same words and trace), f6211c5b (rustfmt
only). Each body records its green rerun. One watched item, not a cut: the
sync regression's `homed_at` bound is 30 s again (ab783c5e; measured 15.3 s
for the whole test, green 23.39 s) — the gate reruns it.

## The gate

Command, under the shared lock and `CARGO_INCREMENTAL=0`:

```
env CARGO_INCREMENTAL=0 lockf -k /tmp/svrngs-gate.lock scripts/check.sh
```

Run 1 (2026-11-05, the unit's first `ralph-result await 6h`) exited 1; the
repair is in the three commits above and its evidence is in `GATE.md`
beside this file. Run 2 (2026-11-05, the unit's second `ralph-result await
6h`, the same command and lock, tree db292099) exited 0 in 1441 s; every
step passed and its step table and notes are in `GATE.md`. The audit's
automated verdict is passed on run 2; the real-device verdict stays
never-ran (`HUMAN-relay-fallback-phone`).
