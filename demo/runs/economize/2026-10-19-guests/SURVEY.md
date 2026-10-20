# What the product carries that it does not need — guests-economize's survey, 2026-10-19

Read-only, at main `1eeb092b`. Every count is `scripts/lean_check.py`'s
rule, per file by `scripts/count.py` here (groups-economize's, unchanged),
agreeing with its total: **16,582 / 16,583**, and svrngs-core **3,008 /
3,009** (`python3 scripts/lean_check.py` at 1eeb092b). The guests queue's
budget (the operator, 2026-10-18, "+150 repo, +60 core") is over the caps
this pass leaves.

The instruments are canon-economize's (`../2026-10-16-canon/SURVEY.md`):
the compiler with every `pub` demoted, the clone report, and the open
findings that name a thing said twice. Grep only checks what they name.
The scripts here are that survey's, copied unchanged but for loop.sh's
scratch paths (/tmp/guests-econ-*). The builds ran with
`TD=/Users/alexsbryan/dev/svrngs/target` (and `web/target` for the page),
so the scratch clone reused the checkout's built dependencies rather than
fill the disk with a second copy (18 GB free at the start, against a floor
of 15).

## What changed since canon-economize's survey

`data/delta-a0129152.txt` (`scripts/delta.py` over `count.py` at
`a0129152` and at `1eeb092b`): 16,419 → 16,582, **+163**. canon-economize
cut 6 after its count (8515859f, `read_origin`: group.rs −6); the screen's
pass cut 3 and the musl fix's pass 2 net. The rest is the canon queue's
raises: scripts/trust_check.py +58 (canon-floor), carried.rs +56
(canon-pin), synced.rs +26 (canon-pin's `Runs`), home.js +19 (the screen),
question.js +8, folder.rs +4, web/app.js +3, core lib.rs +3, and smaller;
membership.rs −7 and admit.rs −6 (canon-spans' pass), door.rs −3 (the musl
fix's one dialer guard).

## 1. The compiler: nothing new

In a scratch clone of `1eeb092b` (/tmp/guests-econ-scratch),
`scripts/demote.py` made every `pub` in every non-test source of the four
crates and `web/src` `pub(crate)` (69 files). Then `scripts/loop.sh` built
the product (`cargo build --workspace --offline`) and `scripts/repub.py`
made `pub` again each item a downstream crate was refused, round after
round. The two refusals repub.py does not parse were made `pub` by hand,
as before: `NotASyncPath` (synced.rs:37; E0446) and `Refusal::exit_code`
(cli.rs:633). The product ended `build-finished success: True` (its last
round `errors 0 fixed 0`), and `web/` for `wasm32-unknown-unknown`
(`DIR=web ARGS="--target wasm32-unknown-unknown"`) on its first round. The
scratch diff is `data/demoted.diff`; what still warns is
`data/warnings-prod.txt` and `data/warnings-web.txt`.

Against canon-economize's, line for line (paths and lines set aside): its
one cut is gone from the list (`group::read_origin`), and nothing is new.
The page's list differs only in naming each file by its absolute path.
The rest is what the earlier surveys read and kept: `At::Here`
(relay.rs:63, how a test runs `svrngs relay`), `MemoryLookup` (iroh.rs:64,
the version pin in one place), `LINK_WITHIN` (dial.rs:46, which the page
compiles), `iroh::ALPN`'s re-export (iroh.rs:48, sharing its line with
`Alpn`), the core's 9 re-exports only tests read (lib.rs:130-182, each
test would need a private module made public), and the node lib.rs:41
glob shadow, the demotion's own effect.

`scripts/cfgtest.py` finds node.rs:202-203 and :388-389 (`Node`'s
`routes`) and :953-954 (`pub use tests_support::splice`, #7) as before, 3
lines, and nothing new.

`scripts/jsnames.py` over the page's JavaScript (12 files,
`data/jsnames.txt`): every top-level name is said again. Its plant, a
function appended to web/app.js in the scratch clone and never called,
went red: `web/app.js: plantedNeverCalled said 1`; with the file restored,
green (`data/jsnames-plant.txt`).

## 2. What no check, test or person runs: nothing

`git diff --name-status a0129152 1eeb092b -- scripts` adds
scripts/trust_check.py and scripts/trust_floor.txt, which check.sh runs
(check.sh:24, `step "trust floor"`), and modifies five that it or a
release runs.

## 3. The clone report: nothing to cut

A fresh corpus, since canon-economize's predates 94 changed files (`git
diff --stat a0129152 1eeb092b -- crates web scripts Cargo.toml`): `git
archive 1eeb092b crates web scripts Cargo.toml Cargo.lock` into
/tmp/guests-econ-corpus (234 files; apps/ left out, as it is not
counted), `sovereign-cli code index . --corpus-id guests-1eeb092b` (3196
chunks in 588 s, `data/index.out`), then `sovereign-cli refresh` there
(rust 17818 symbols, 103437 references; typescript and python "skipped
(export failed)", I4; `data/refresh.out`). The refresh named its corpus by
the directory, `guests-econ-corpus`, and built the index again under that
name with the call graph beside it, so the reports read that one. Then
`~/dev/commonwealth-ai/target/debug/sovereign-cli code dry-report
--corpus-id guests-econ-corpus --scope <dir> --threshold <t>` for each
crate, `web` and `scripts`, at 0.95 and 0.90 (`data/dry-<scope>-<t>.md`).

| Scope | Exact | Near 0.95 | Near 0.90 |
|---|---:|---:|---:|
| crates/svrngs-core | 4 | 0 | 11 |
| crates/svrngs-runtime | 1 | 0 | 0 |
| crates/svrngs-node | 1 | 1 | 9 |
| crates/svrngs | 0 | 0 | 0 |
| web | could-not-judge (I4) | 0 | 2 |
| scripts | could-not-judge (I4) | 0 | 0 |

Against canon-economize's reports, member by member with the lines set
aside: the same clusters, and one gone (question.js `door`/`judge`, which
canon-fix rewrote). Those clusters were read there: three-line accessors
on different types, test code, and overlapping spans, one function inside
its neighbour (I32). None is a cut.

## 4. Said twice: the open findings

No row under principle 8 opened since canon-economize's survey is open:
#206 and #208 are fixed (5eb3d563, aa70a97c), and #205, #207, #209, #210
are under other principles, each fixed or closed. The open ones before
them are as that survey read them: #185 (a helper costing 2 more than it
saves), #186 and #150 (apps/, not counted), #202 (the operator's), #204
(with #107, the operator's).

## 5. What this queue's rows will touch

So no cut is written twice. Each was read at 1eeb092b.

- **The join link's redemption** (guests-link reuses it for the visit
  link). `redeem.rs` `answer` (redeem.rs:153-187) answers each
  decision in one of seven arms. Its last two, `Decided(why)` (:176-180)
  and any other `why` (:181-185), are the same answer, 403 with
  `REFUSAL`, each traced at debug with `why`: two arms for one decision.
  **A cut of 5**: the second arm alone answers both, its trace still
  naming the refusal by `why` (`Decided(Fault(Lapsed))`,
  `Decided(NotOwn)` and the like), which is what tells them apart.
  The visit link's refusals land in this match, so the arm a refusal falls
  to is one.
- **Who a commit refused, said as the rail's error.** rail.rs:225-229
  (`append_act`) and redeem.rs:274-280 (`serve_redeem`) each turn
  `Refused::NoStanding` into `RailError::NotInRoster { actor, namespace }`
  and `Refused::Unsettled` into `RailError::Unsettled`, by hand. One
  helper beside `RingRail::commit` is about 15 counted lines after rustfmt
  and saves 12 at these two callers: it costs 3, so it is not a cut. The
  other callers (invite.rs:135-147, removal.rs:171-178 and :275-281,
  folder.rs:447-454, node.rs:493-500) answer each refusal their own way.
  Left for guests-link, whose `Visit` is a third caller of the same shape:
  if it is, the helper pays then.
- **The membership fold** (guests-visit folds visits beside it,
  membership.rs). canon-spans' pass put its traces on one line each
  (membership.rs −7 since a0129152); its collections are what rustfmt
  makes of them, and every method a downstream crate reads (§1). Nothing
  to cut.
- **The door's holders' path** (guests-door, door.rs `holder_of`,
  :932-992, `Who`, `PersonKey::resolved`). Each refusal is said once by
  the `refused` closure; the two `standing.is_none()` steps are two
  decisions (the cut key told at once, the ring asked first), each traced.
  Nothing to cut.
- **The home page's lists** (guests-pages, home.js `show` and `screen`).
  Both read `/v1/rail/people` and `/v1/lends`, and both draw the people
  through question.js `door`, already one. The clone report's one home.js
  pair, `running`/`look`, is one function inside the other. Nothing to cut.

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §1, §2, §3, §4 | 0 | 0 |
| §5, redeem.rs `answer`'s two arms of one refusal | 5 | 0 |
| §5, the commit's refusals said as the rail's error | costs 3, not a cut; for guests-link | 0 |
| **Total cut here** | **5** | **0** |

The cut would leave the repository at **16,577**, its cap one past at
**16,578**, and this queue's ceiling **16,728**. svrngs-core stays
**3,008 / 3,009**, its ceiling **3,069**.
