# The test audit of 2026-11-16

The operator asked for an audit of all tests: quick, robust and comprehensive
at every level, with the functional over the trivial, so the code can be
refactored with freedom. HEAD d39afd3a (the invite flow's work, gate exit 0
at bcc9126f, `../invite/2026-11-16/check.txt`: 1,257 s warm, every step
passed, nothing could-not-judge).

Every number below was measured or recounted today unless it names its own
date. Commands and exits: `node web/tests/run.mjs unit` exit 0 (17.0 s wall,
351 tests, 56 files); `cargo test -p svrngs-core -p svrngs-runtime --quiet`
exit 0 (15.1 s wall warm, the tests themselves 0.27 s); per-file timings by
`node --test <file>` over `run.mjs unit --list`, exits 0.

## What stands (the inventory)

| Tier | What it is | Count | Time today | Verdict |
|---|---|---|---|---|
| Static (`check.sh --fast`) | pins, lean/RUST_ARCH, trust floor, lint, format, app nouns | 9 steps | < 1 min | quick, robust |
| Isolated Node laws (`page laws`) | page words and decisions over the twin, apps' handlers, resolver contracts | 351 tests / 56 files | 17-19 s | quick; functional |
| Pure Rust units (core, runtime, node's src) | admission, sealing, authority, journal, oplog | 896 gate Rust tests minus the seam ones below | `tests` step 387-505 s incl. build; `-p svrngs-core -p svrngs-runtime` alone 15 s warm | comprehensive (see gaps) |
| Wire tests | nodes on `Wire`/`InProcess` harnesses, in the `tests` step | 720 of 896 deterministic (80.4%, recount `../../flakes/2026-11-13/recount-e09e8462.out`) | inside `tests` | the refactoring guard |
| Node integration (`page integration`) | real Redis/Lua lifecycle, WebKit CSS | 2 files, 13 tests' worth of Redis rows (11 kept by design, CHECKS.md) | 7-8 s | robust, contained D |
| Browser (`browser`, `join_page`, `pages`) | seam B contained: one engine per page family | 37+6+3 fragile rows | 114+49+21 s | contained by design |
| Scenes (`scenes`) | 19 e2e journeys, every person this build's svrngs | 20 fragile rows | 364 s at 2 threads | e2e, contained |
| Playback / conformance / n0 | seam M and R contained | 8+7+1 fragile rows | 175+48+13 s | contained |
| Apps (`app laws`) | 31 files, handler-level laws | 161 calls | 16-17 s | functional |

The architecture is already the one the operator's goal asks for: each
behavior claim lives at the smallest boundary that can judge it (CHECKS.md
"Check the contract at its smallest real boundary"), the seams that vary with
the host (browser, decode, relay, socket, clock, daemon) are each contained
in one named place, and every step reports four verdicts. Nothing found in
this audit argues for a new tier or a rewrite; the findings are distances to
the architecture's own targets, plus two robustness gaps and one doc debt,
all but one addressed in this audit's commits.

## Findings, ranked

1. **Fragile share 80.4%, the target 95%** (`recount-e09e8462.out`: 720 of
   896 deterministic, 176 fragile; target "at most about 51", CHECKS.md
   "Where it stands"). The remaining mass by target (seams.tsv fragile rows):
   browser 37, scenes 20, resolver-Redis 13 (kept by design — the Lua is the
   decider), home 10, svrngs_node 9, conformance 8, tcp 7, install 7, udp 6,
   share 6, join_page 6, then 2-5 each. The roadmap exists and is proven
   (46.2% → 80.4% in four days): harnesses 6 (a probe passed into
   `share::listening`), 7 (`Net::now` for carried's budget clock) and 9 (the
   room's listener) are owed in CHECKS.md with designs and line costs, and
   they carry the home/tcp/udp/install/share mass. Not acted on here: each is
   a product change at the cap, the operator's to fund.

2. **The edit loop can hang forever on one test.** `node --test` has no
   default timeout; `web/tests/run.mjs` passed none, and CHECKS.md names
   `node web/tests/run.mjs unit` as the edit loop. A law that awaits nothing
   would hold a session indefinitely (the gate would cut it at its step
   limit, 300 s, but the loop has no such limit). Fixed in this audit:
   `run.mjs` now passes `--test-timeout=120000`, planted and watched
   (below). 120 s is 60x the slowest law observed today and 15x the whole
   apps step; it ends a hang, never races real work.

3. **`ActSet` had no direct law.** Every convergence test builds one, but
   nothing asserted the type's own two promises (order and repetition cannot
   be written down; two copies merge by union, `insert` false only on the
   exact same line). The pure heart's merge semantics deserved their own
   pinned law, in the house style, with a plant. Landed in this audit.

4. **Doc debt: CHECKS.md's "Where it stands" stopped at 18cd2afb (72.3%).**
   Two newer recounts existed in the record
   (`../../flakes/2026-11-13/README.md`: 6c30c2aa 68.7%, e09e8462 80.4%) but
   the doc's chain did not carry them, so its distance-to-target read
   stale. Fixed in this audit: both recount paragraphs added.

5. **The isolated Node laws' wall time is one file's.** `node --test`
   parallelizes by file; the tier's 17 s is `pages/home.test.mjs` (16.2 s,
   ~30 laws; each `open()` loads the page's modules anew and settles by
   polling — deliberate isolation, twin.mjs). The tier grew 0.652 s
   (2026-11-11, 309 tests) → 17 s (351 tests) as laws moved in, and it will
   keep growing with the pages. Still comfortably inside an edit loop. If it
   outgrows it, the lever is splitting `home.test.mjs`'s laws across files
   (they are independent), not weakening the per-law isolation. Recorded
   here so the next reader measures before cutting.

6. **Trivial unit tests: none found worth cutting.** Sampling the Rust unit
   tier (`tests.rs`, `payload_tests.rs`, the pages laws) finds laws named for
   their failure mode, asserting user-visible contracts (words drawn, routes
   asked, refusals said), golden copy pinned in one home. The extraction
   ledger (CHECKS.md) has already retired the duplicated/trivial kind
   (252 rows `moved`). The audit found no test asserting an implementation
   detail that would block a refactor.

7. **Apps with no test file of their own — each an acceptance, named:** tap
   (a static example page, read by pages/conformance), web-conformance (the
   conformance canary is its test), taste (llama-server by hand, CHECKS.md
   unit 6), jellyfin and holyarrs (the films scene, the holyarrs scene and
   `apps/holyarrs/check/`'s xenia worlds). Absence reported per rule 6; none
   is a gap today, and holyarrs's own check scripts carry plants
   (`apps/holyarrs/check/plants.py`).

## What this audit changed

Commits 1cc00b07 (the `of` fold, behaviour kept), d893b559 (the ActSet
laws), 071ffb80 (the tier timeout), 24205d87 (the recounts), 46f002f3
(this record). After them all: `scripts/check.sh` exit 0 in 1,449 s,
45 steps passed, nothing could-not-judge (`gate.txt`).

- `web/tests/run.mjs` passes `--test-timeout=120000` to every tier. Plant:
  a law holding a timer that no one clears. Before, the tier never ended on
  its own — every other law done, the run killed only by the shell's 40 s
  cut (`plant-timeout-before.log`, exit 124). After, the tier failed in
  121 s naming the test, `test timed out after 120000ms`
  (`plant-timeout.log`), all 351 real laws passing; plant removed, the tier
  green in 18.8 s. A bare promise that nothing resolves does not hang
  (the loop drains and node:test says `cancelledByParent`); a held handle
  does — the plant holds one, as a stuck socket or timer would.
- `crates/svrngs-core/src/act_set_tests.rs`: three laws (order and
  repetition unwritable; union of two copies; `insert` false only on the
  exact line). Plant: keying the set by a constant (so every line collapses
  onto one) failed the first law (`plant-act-set.log`); reverted, all
  `cargo test -p svrngs-core` green.
- `docs/CHECKS.md` "Where it stands" carries the 6c30c2aa and e09e8462
  recounts, citing the record.

## Owed by this audit (not done here)

- Harnesses 6, 7, 9 (CHECKS.md "Harnesses missing"), the operator's to
  fund: they move the remaining home/tcp/udp/share/install fragile mass and
  carry the 80.4% toward 95%.
- The scenes step (364-434 s) is half browser wait by the operator's "2 at
  a time" choice; no change recommended.
- `tests` step 387-505 s is build-dominated; the touched tier already
  covers the edit loop (172 s warm, CHECKS.md), and the whole gate remains
  the gate before a push.
