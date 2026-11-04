# What the product carries that it does not need — leave-economize's survey, 2026-10-21

Read-only, at main `39f39a59`. Every count is `scripts/lean_check.py`'s
rule, per file by `scripts/count.py` here (guests-economize's, unchanged),
agreeing with its total: **16,906 / 16,907**, and svrngs-core **3,075 /
3,076** (`python3 scripts/lean_check.py` at 39f39a59). The leave queue's
budget (the operator, 2026-10-21, "Pass first, then raise") is at most +25
over the cap this pass leaves, none in svrngs-core.

The instruments are guests-economize's (`../2026-10-19-guests/SURVEY.md`):
the compiler with every `pub` demoted, the clone report, and the open
findings that name a thing said twice. The scripts here are that survey's,
copied unchanged but for loop.sh's scratch paths (/tmp/leave-econ-*). The
builds ran with `TD=/Users/alexsbryan/dev/svrngs/target` (and `web/target`
for the page), so the scratch clone reused the checkout's built
dependencies: the disk had 15 GB free at the start, the floor.

## What changed since guests-economize's survey

`data/delta-1eeb092b.txt` (`scripts/delta.py` over `count.py` at
`1eeb092b` and at `39f39a59`): 16,582 → 16,906, **+324**, every line of it
the guests rows' and V1.1.3's raises, which REVIEW-audit-guests-auto-1
counted row by row (`ralph/REVIEW_FINDINGS.md`, its "(1)" tables):
door.rs +54, membership.rs +50, redeem.rs +47, invite.rs +32, home.rs +24,
web/app.js +23, runtime register.rs and lib.rs +15 each, and smaller.
Since that audit's range end, `beb20b68`, no counted file changed: `git
diff --stat beb20b68 39f39a59 -- crates web scripts Cargo.toml Cargo.lock`
is one test file, crates/svrngs-node/tests/browser/main.rs (+18 −28,
c91069af's `seeded`).

## 1. The compiler: one item only tests reach

In a scratch clone of `39f39a59` (/tmp/leave-econ-scratch),
`scripts/demote.py` made every `pub` in every non-test source of the four
crates and `web/src` `pub(crate)` (69 files). `scripts/loop.sh` then built
the product (`cargo build --workspace --offline`) and `scripts/repub.py`
made `pub` again each item a downstream crate was refused, round after
round. Three refusals repub.py does not parse were made `pub` by hand: the
two the last survey named, `NotASyncPath` (svrngs-core synced.rs:37;
E0446) and `Refusal::exit_code` (cli.rs:633), and one new,
`standing_reads` (membership.rs:794; E0603 through `pub use
membership::*`, which repub.py does not follow), which svrngs-runtime
reads (journal.rs:113, register.rs:219). The product ended
`build-finished success: True` (its last round `errors 0 fixed 0`), and
`web/` for `wasm32-unknown-unknown` (`DIR=web ARGS="--target
wasm32-unknown-unknown"`) on its first round. The scratch diff is
`data/demoted.diff`; what still warns is `data/warnings-prod.txt` and
`data/warnings-web.txt`.

Against guests-economize's lists, paths and lines set aside (`diff` of
each list with `sed -E 's#^(/private)?/tmp/[^/]+/##; s#:[0-9]+:#:#' |
sort -u`), four lines are new, the same four in both:

- **`method visiting is never used`** (membership.rs:388,
  `Membership::visiting`, guests-visit's). Only `visit_tests.rs:44` calls
  it. The product asks who is visiting through svrngs-runtime's
  `Register`, which copies the fold's `visits` and `cut`
  (runtime register.rs:54, :165) and is no `Membership`: rail.rs:195
  filters `r.visits` by `Guest::live` itself, and home.rs:439,
  redeem.rs:220 and runtime register.rs:347 read `visits` without it.
  **A cut of 3**, svrngs-core's: `Guest::live` stays the one decider of
  whether a visit is on, and the test asks it as rail.rs does. The three
  docs naming `Membership::visiting` (membership.rs:98, :235, lib.rs:431)
  and DESIGN §6 (docs/DESIGN.md:1114) name `Guest::live` instead.
- `InviteFault` more private than `RailGap::RefusedRedemption::fault`
  (admit.rs:158) and than `RedeemRefusal::Fault::0` (register.rs:42), and
  `Reads` more private than `<Visit as Command>::READS` (register.rs:146):
  the demotion's own effect on types a public item names, as the list's
  other `more private` lines are. Nothing to cut.

The rest is what the earlier surveys read and kept: `At::Here`
(relay.rs:63), `MemoryLookup` (iroh.rs:64), `LINK_WITHIN` (dial.rs:46),
`iroh::ALPN`'s re-export (iroh.rs:48), the core's re-exports only tests
read (lib.rs:130-176), and the node lib.rs glob shadow.

`scripts/cfgtest.py` finds node.rs:202-203 and :388-389 (`Node`'s
`routes`) and :958-959 (`pub use tests_support::splice`, #7) as before, 3
lines, and nothing new.

`scripts/jsnames.py` over the page's JavaScript (13 files,
`data/jsnames.txt`): every top-level name is said again. Its plant, a
function appended to web/app.js in the scratch clone and never called,
went red: `web/app.js: plantedNeverCalled said 1`; with the file restored,
green (`data/jsnames-plant.txt`).

## 2. What no check, test or person runs: nothing

`git diff --name-status 1eeb092b 39f39a59 -- scripts` modifies two,
scripts/build-web.sh and scripts/lean_check.py, which a release and
check.sh run. Nothing is added.

## 3. The clone report: nothing to cut

REVIEW-audit-guests-auto-1 ran this instrument over `beb20b68`'s crates/,
web/ and scripts/ the day before (`ralph/REVIEW_FINDINGS.md`, its "(2)
Clones"): `code index` over a `git archive` of beb20b68 (4147 chunks),
`refresh` (rust 18452 symbols; typescript and python "skipped (export
failed)", I4), and `code dry-report` for each crate, web/ and scripts/ at
0.95 and 0.90. The counted product at 39f39a59 is beb20b68's, line for
line (above), so a second index would read the same files; it was not
run, and the disk's 15 GB was kept for the builds. Its product clusters:
svrngs-core 4 exact, 11 near 0.90; svrngs-runtime 1 exact; svrngs-node 1
exact, 1 near 0.95, 8 near 0.90; web 2 near 0.90. Every side blames to a
commit before 34d6e287 (its `cluster-sides.txt`), the clusters
guests-economize read and kept: accessors on different types, test code,
and one function inside its neighbour (I32). Its line-window pass's six
new spans are all test code (#217, and c91069af's fix). None is a cut.

## 4. Said twice: the open findings

Open since guests-economize's survey: #211-#213 and #215-#217
(`ralph/REVIEW_FINDINGS.md`). Only #217 is principle 8, and it names test
code alone (tests/browser/main.rs, tests/browser/guests.rs,
tests/join_page.rs, web/tests/), which the budget does not count: not
product, so not a cut here. #211 is this queue's; #212, #213, #215, #216
change behaviour, so are no pass's. The older ones stand as the last
survey read them: #185 (one helper for how a day is said costs 2 more
than it saves), #186 and #150 (apps/, not counted), #202 and #204 (with
#107), the operator's.

## 5. What the next row will touch

So no cut is written twice. Each was read at 39f39a59.

- **`told_cut`'s selection** (ring_routes.rs:140-158). Its four lines
  that choose what a cut is told with (:146-149: the first `Remove`
  naming the asker, its signer and seq, and `retain` of the signer's
  acts before it and the hand-ons) are what leave-told moves into the one
  shared function, both tells calling it (the charter's "One rule"). A
  reshaping now would be written again there. Nothing to cut ahead of it.
- **`removal.rs` `tell`** (:234-265). It reads this copy's digest once
  and posts one `RingSyncRequest` per computer through
  `ring_sync::post`, the one poster (ring_sync.rs:661), already shared
  with the round. leave-told rewrites its body (a digest asked, then the
  selection sent until covered). Its trace and its `answered` filter
  are the row's to keep or change. Nothing to cut ahead of it.
- **`leave_told`** (ring_sync.rs:420-445) reads the leave through
  `RingRail::own_removes` (runtime lib.rs:251-253), its one caller,
  and hands `tell` the lines. Whether the row still needs the lines, or
  only whether there is a leave to tell, is the row's to decide: cutting
  `own_removes` now would decide it for the row. Nothing to cut ahead of
  it.

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §1, `Membership::visiting`, which only a test calls | 3 | 3 |
| §2, §3, §4 | 0 | 0 |
| §5, the code leave-told rewrites | 0, left to the row | 0 |
| **Total cut here** | **3** | **3** |

The cut would leave svrngs-core at **3,072**, its cap one past at
**3,073**, and the repository at **16,903**, its cap one past at
**16,904**; this queue's ceiling **16,929** (+25), svrngs-core's
**3,073**.
