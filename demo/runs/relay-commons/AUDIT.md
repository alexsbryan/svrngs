# REVIEW-audit-relay-commons: the ledger

The whole delta from `d59c8246` (CHARTER.md; STATE.md's last row). Counts
are `scripts/lean_check.py` run on `git archive` of each commit, so each
row's figure is its own commits' sum, not a later reading.

## Verdicts, automated and real-device apart

| Check | Verdict | Where |
|---|---|---|
| Full `scripts/check.sh`, svrngs-gate container, plain clone | passed at ab15a9ea (failed at b170fc6e; below, "The gate") | |
| Each row's named checks, planted red then green | passed | "Plants", below |
| HUMAN-relay-box: the box, `deploy-relay.sh`, load to drop, a phone on cellular | never-ran | the operator's |
| HUMAN-phone-home: a real phone goes direct, falls back mid-stream, stays relayed on a network not enabled | never-ran | the operator's |
| HUMAN-switch: `SHIPPED` to ours | never-ran | the operator's |

## What the audit repaired

| Commit | What |
|---|---|
| a0e6afc1 | The wire's per-worker thread-locals gone: each direct attempt owns its send queue, port and sender (the director's finding 2). webrtc_page 6 passed, four runs. |
| ce784518 | n0's held while none of ours answers, a group's own included (RELAY_COMMONS 2): it never was for a group naming its own. Reproduced red first. Its body's size line is wrong: it is core +8, non-JS +29, as measured here. |
| ef50e3bf | `docs/seams.tsv`: 17 rows for the queue's tests, which had none. |
| 4ed6db32 | Trust floor: three `listen` descriptions brought to their code (the offer's admission and cap; the relay's counters on loopback; a stale line cite). |
| cc6fed0a | `Direct::go`'s bare 3 s is `paths::PATH_IDLE`. |
| f3fe30b0 | DESIGN back within its +60 net (was +73); the direct path's "nothing past it" rephrased per the UDP decision. |
| af24c051 | The gate's `new_relay`: `Host::check` decided an answer about the meeting it had snapshotted, so a `meet_at` during the 5 s ask read the group's own as silent and brought n0's in (since ce784518). An answer about a changed meeting now decides nothing. Reproduced red first. |
| 66c37715 | The gate's `surface`: two rows' expected `/v1/groups` lacked the `share` 1b08385c added. |
| a2661814 | The gate's `types` (12) and `typed lint` (7): the queue's JS typed. One guard read right: `!handed.key`, never true, is `!held`. JS at its cap, 8,674. |

## Counts against each allowance

Product lines, `lean_check` (all code but JavaScript, core within it):

| Row | Allowance | Measured | Note |
|---|---|---|---|
| relay-survey | no feature code | 0 | its pass cut nothing (SURVEY.md:123) |
| iroh-1-4 | +30 | 0 | |
| relay-share | +20 | +17 | |
| n0-emergency | +50 (core ≤15) | +41 (core +11), JS +12 | 53 if "product" counts JS; landed before the `Size:` rule (230cc277) |
| share-refusal | +70 Rust, +40 JS | +68, JS +38 | |
| webrtc-node | +400 | +375 (core +2) | 0ef57b10 +141, 95f58944 +240, 1eacc58e +8, 01507e33 −14 |
| webrtc-network | +70 Rust, +20 JS | +141, JS +17 | `Size:` 6807cb53 |
| deploy-relay | 140 script | +59 | |
| self-host | +50 (core ≤15) | +95 (core +12), JS +7 | `Size:` b3ae73f2, 744a41e2 |
| webrtc-page | +120 browser Rust, +60 JS | browser +226, node +37, JS +55 | `Size:` 36ebb09a, 47462fb6; JS cap the operator's (e46447fe, 80d270f2) |
| the audit | — | +28 (core +8), JS +1 | a0e6afc1 −8, ce784518 +29, af24c051 +5, a2661814 +2, JS +1 (`Size:` there) |

Rows past their estimate land under the operator's rule of 2026-11-19
("a row's allowance is its estimate; the caps decide"), each with its
`Size:` paragraph. Tests are not budgeted (the operator, 2026-11-19).

Totals, d59c8246 to the audit: core 3,499 → 3,532 of 3,539; all code
but JavaScript 21,474 → 22,569 of 23,101; JavaScript 8,544 → 8,674 of
8,674. Core and non-JS caps unchanged; JavaScript's raised only by the
operator. No standalone cut was made, so none was due a ratchet.

DESIGN: +106 / −46, **+60 net** of 60.

## One decider

- The relay's share: `SHARE_RATE`, `SHARE_BURST` (crates/svrngs/src/relay.rs:52, :56) alone; tests read them.
- The door's "large" line, `OVER_SHARE` (door.rs), is the door's own by CHARTER; it equals the burst and cites it. svrngs-node cannot read svrngs's constant.
- The path timing, `PATH_IDLE` and `KEEP_ALIVE` (webrtc/paths.rs), compiled into the node and the page by `#[path]`; the page's wait now reads it. The shim's ICE-gathering 3,000 ms is its own bound, the operator's.
- n0's relays: `relay::n0_relays` and `tiers::behind`; no URL of n0's in product code.
- The offer's cap `MAX_ATTEMPTS`, `SETUP`, `IDLE`: webrtc.rs alone.
- The tier decision: `Meeting::decide`, its input now the closed `Answer`.

## Plants (each red, then green, in its commit body)

| Row | Checks | Plants |
|---|---|---|
| relay-share | burst held to the rate; RateLimited seen; warn filter | 8ea1b050 |
| n0-emergency | n0's while ours is down, back after; #135 | effc46bc, 592c93d7, 001db3c6, 401ef4dd, d386b0b8 |
| share-refusal | tiny-rate relay throttles; range gets 429 share and words; small passes; direct/own never refused; stays spent | 1b08385c, 0089b3b1, f80942db, b77b6233, 8fe3c5b5, a5850dae, re-watched a17b5aa2 |
| webrtc-node | native peer over the wire, fallback ≤ 4 s; no custom address; relay stopped | 95f58944, 69dd09e0, 85cbdf2f |
| webrtc-network | unapproved network; unread network; network change; cap; enabled answers | 6807cb53 |
| deploy-relay | shellcheck; `--dry-run` steps | c8399de5 |
| self-host | own HTTPS first; group-only relay admits a pass's holder; no-group key refused | b3ae73f2, 744a41e2 |
| webrtc-page | film on webrtc; withheld stays relayed; closed mid-film ≤ 4 s; the offer a pass's; shim timeout | 536cd2fc, 36ebb09a, 47462fb6 |
| the audit | own relay and ours down → n0's, said on the page | ce784518 |

## CSP, trust floor, dependencies

- CSP: no policy line changed (`git diff d59c8246 -- scripts/trust_floor.txt` touches no `page` line; web/vercel.json unchanged).
- Trust floor: two listeners added, each the operator's (the WebRTC socket, 95f58944 and the UDP decision; the data endpoint's no-socket line), and three descriptions brought to their code (13ffd41e, 4ed6db32). `python3 scripts/trust_check.py`: 13 policies, 13 listeners, 10 gates.
- For the operator: the WebRTC line's last clause ("so a phone on the same WiFi reaches its lender and nothing past it") is the approved wording of 95f58944, kept. The UDP decision says nothing may claim the port is kept "on this Wi-Fi"; DESIGN no longer does. The floor's clause is about what host candidates offer, and the audit leaves it as approved.
- Dependencies: workspace lock +38 crates, all in the closure of iroh 1.4.2 and str0m 0.25 (`cargo tree -i`). `aws-lc-rs`, `aws-lc-sys` and `cmake` come in through `dimpl`. str0m-rust-crypto 0.7.0 itself turns on `dimpl/rcgen`, which enables `aws-lc-rs` (its Cargo.toml:42-46), so svrngs's features cannot remove them. The web lock gained none.

## The gate

Run from a plain clone of this branch's head mounted as `/work`
(scripts/gate/README.md; PROMPT.addendum), by `target/audit-gate.sh`.

**1. b170fc6e: failed** (1,559 s; ctl/log/sessions/REVIEW-audit-relay-commons-1.await.log).
Every step passed but four:
- tests: `new_relay` and `surface` (af24c051, 66c37715).
- types and typed lint (a2661814).
- browser: 33 of 45 rows. The first fault was not svrngs's. Playwright
  1.59.1's server died on its own assertion (`WKPage._onTargetCreated`,
  playwright-core/lib/server/webkit/wkPage.js:331, a second provisional
  page while one was pending) in `navigations::…::webkit`, a chain of
  cross-origin redirects. Every later row lost its browser (`ECONNREFUSED`
  to the browser server, the proxy refused). Not reproduced in the
  container at a2661814: the browser step 42 passed, and
  `navigations::` 8 runs of 8, 4 passed each. One crash in 9 runs is not
  a rate; the next gate run is the judge.

**2. ab15a9ea: passed**, exit 0 (1,214 s; ctl/log/sessions/REVIEW-audit-relay-commons-2.await.log,
sha256 4ddf2534…72ad). All 26 steps passed, the browser step and 19
scenes among them; no step failed or never ran. The conformance canary's
two could-not-judge rows ("its worker, after svrngs's restarts", WebKit
stops no worker on ask) are WebKit's, on both origins, each beside a
passed row of the same check. The Playwright crash of run 1 did not recur: 1 in 10 runs.
Only this ledger changed after ab15a9ea.
