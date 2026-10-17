# What the product carries that it does not need — start-economize's survey, 2026-10-13

Read-only, at main `9267bc1`. Every count is `scripts/lean_check.py`'s
rule (a line not blank and not starting `//`, `#`, `/*` or `*`; tests,
`apps/`, `node_modules/`, `demo/runs/` and `target/` apart), per file by
`scripts/count.py` here, agreeing with its total: **16,036 / 16,040**, and
svrngs-core **3,121 / 3,130**. The operator's line (2026-10-13, "Back
under 15,800, then ratchet") is 15,799: **237 lines** to find.

Found by the compiler, by the clone report and by the open findings, as
`ralph/next/lean/SURVEY.md` found them, not by grep — grep only checks
what they name.

## Where the 16,036 are

| Part | Lines |
|---|---:|
| crates/svrngs-node | 7,732 |
| crates/svrngs-core | 3,121 |
| crates/svrngs | 1,882 |
| web (`*.js`, `src/*.rs`, `install.sh`) | 1,702 |
| crates/svrngs-runtime | 1,076 |
| scripts | 523 |

Since `eaa401e`, the start queue's first commit's parent, the repository
went from 15,797 to 16,036 (ralph/REVIEW_FINDINGS.md, REVIEW-audit-start-auto-1,
"The size budget"; `python3 scripts/lean_check.py` at HEAD): +239, the
whole of 2026-10-13's three raises.

## 1. The compiler — what nothing but a test reaches: 29

In a scratch clone (`git clone` of `9267bc1` into /tmp/econ-scratch),
every `pub` in every non-test source of the four crates and `web/src`
was made `pub(crate)` (perl, `s/\bpub\b(?!\s*\()/pub(crate)/g`), and each
`pub use a::{b, c}` split to one name a line, so a re-export is kept
or dropped name by name. The two glob re-exports that make a crate's API
(`svrngs-runtime` `pub use svrngs_core::*`, `svrngs-node` `pub use
svrngs_runtime::*`) and the page's `#[wasm_bindgen]` items, which
JavaScript calls, stayed `pub`. Then `scripts/loop.sh` built the product
(`cargo build --workspace --offline`, no tests) and `web/` for wasm32
(`cargo build --release --target wasm32-unknown-unknown`), and
`scripts/repub.py` read each privacy error (E0603, E0616, E0624, E0451,
"private type") and each name a downstream crate could no longer resolve,
and made its definition `pub` again, round after round, until the
product and the page both built (`build-finished success: True`). What still
draws a warning is what no other crate's product code needs
(`data/warnings-prod.txt`, `data/warnings-web.txt`; the whole scratch
diff, `data/demoted.diff`). Each was then read where it is used:

| Item at `9267bc1` | Warning | Who uses it | Lines | Cut |
|---|---|---|---:|---|
| crates/svrngs/src/cli.rs:1704-1715 `new_link` | never used | 14 calls in tests/new.rs and tests/join_by_link.rs; it is `link_to(.., false)` with the person dropped | 9 | the tests call `link_to`, its doc folded into `link_to`'s |
| crates/svrngs-node/src/guest.rs:75-84 `RelayConfig::on_loopback_only` | never used | 12 test sites, each `RelayConfig { .., loopback_only: false }.on_loopback_only()` | 8 | each writes `loopback_only: true` |
| crates/svrngs-runtime/src/register.rs:95-105 `standing_in`, `#[cfg(test)]` in a counted file | (cfg(test): not built) | register_tests.rs:108, :140, :234 | 8 | moved into register_tests.rs |
| crates/svrngs-node/src/node.rs:453-457 `Node::say_relay`, a wrapper of `Host::say_relay` | never used | tests/relay.rs:142 | 3 | `Host::say_relay` `pub`; the test asks `sam.host()` |
| crates/svrngs/src/cli.rs:216-217 `pub use svrngs_node::invite::LINK_LIFE_SECS` | unused import | tests/new.rs:7, :115 | 1 | the test imports it from `svrngs_node::invite` |
| crates/svrngs/src/relay.rs:57-63 `At::Here` | never constructed | tests/relay.rs:28, tests/n0_off.rs:99: a relay on plain HTTP is what the tests serve | — | not a cut: the variant is how a test runs `svrngs relay`; nothing else could stand in for it |
| crates/svrngs-node/src/iroh.rs:61 `pub use iroh::address_lookup::MemoryLookup` | unused import | five test files, two of them the command's | — | not a cut: its comment's reason, "the version pin in exactly one place", is the command's tests' |
| crates/svrngs-core/src/lib.rs:117-133, 13 names re-exported that only tests read (`Reads`, `membership`, `PayloadError`, `MAX_PAYLOAD_BYTES`, `Uncaused`, `ProcessManager`, `act_id`, `redemption_message`, `ring_op_message`, `sign_ring_op`, `NO_BUDGET`, `NotASyncPath`, `DIGEST_V`) | unused import | the other crates' tests, through modules that are private (`mod sig;`, `mod sync;`) | — | not a cut: dropping them saves at most a line of a wrapped list, and each test would need the module made public instead |

Nothing in the page's Rust drew a warning of its own. Two suppressions
hide nothing: `#[expect(dead_code)]` at door/choices.rs:76 is a field
read by serde's `deny_unknown_fields`, and `#[allow(dead_code)]` at
web/src/lib.rs:31 is the node's relay.rs compiled into the page, whose n0
half the page never dials. A scan of every counted source for a test
cfg ahead of an item other than `mod x;` found `standing_in` alone
(the row above). The page's JavaScript has no compiler: every top-level
`function` and `const` of `web/*.js` and `home/*.js` is named somewhere
else in them (checked by name, the one check a page without a compiler
allows; none is unreferenced).

## 2. What no check, test or person runs: 21

`scripts/ralph-mark.sh` (21 lines) marked a row done when a worker ran
it. The loop marks rows now: commonwealth-ai `scripts/ralph.py:478`
(`mark_done`) and `:2318-2326` (`ralph: <unit> done`), and the worker's
prompt says never to mark a row. Nothing in this repository runs it
(`git grep -n ralph-mark` outside `demo/runs/` and `ralph/`:
ARCHITECTURE.md:686, :692, which describe it, and
ralph/claude-settings.json:14, which permits it). Every other script is
run: check.sh, lean_check.py, vocab_check.py, principles_check.py by
check.sh; build-web.sh by check.sh and build-release.sh; ports.py by
DESIGN §4's routine; relay-answers.sh by tests/relay.rs:201 and both
deploy scripts; deploy-*.sh and build-release.sh by the operator.

## 3. The clone report

`~/dev/commonwealth-ai/target/debug/sovereign-cli code dry-report
--scope <dir>`, over the corpus rebuilt at `9267bc1` (`sovereign-cli
refresh --rebuild-index`: 3,936 chunks in 729 s; the index it replaced
was four days old and named a file, core attest.rs, that no longer
exists), for each crate, `web/` and `scripts/`, at the default 0.95 and
at 0.90 as the audits run it (`data/dry-<scope>-<threshold>.md`):

| Scope | Exact | Near 0.95 | Near 0.90 |
|---|---:|---:|---:|
| crates/svrngs-core | 5 | 0 | 12 |
| crates/svrngs-runtime | 1 | 0 | 0 |
| crates/svrngs-node | 1 | 0 | 8 |
| crates/svrngs | 0 | 0 | 0 |
| web | could-not-judge (I4: no SCIP for JavaScript) | 0 | 1 |
| scripts | 0 | 0 | 0 |

Read one by one, none is a cut:

- **The exact groups** are three-line accessors on different types:
  command.rs:87/:105 `lines` (`Authored`, `Recorded`), joining.rs:63/:155
  `signed_by_link` (an ask, a redemption), link.rs:186/synced.rs:67
  `as_str`, act_set.rs:40/command.rs:109 `is_empty`, api.rs:212/:228
  `addr` (`Serving`, `Bound`); one home for each would be a trait or a
  shared type costing more lines than it saves. runtime's `open` and
  core's `key` are in test files.
- **Near, in test files** (core's removal_tests, register_tests,
  membership_tests, membership_invite_tests, tests_support; node's
  ring_sync/tests): no budget line.
- **Near, in counted code**, each two neighbours the embedder reads as
  alike: core lib.rs:870/:873 `name`/`verify`, :826/:829 `actor`/`sign`,
  op.rs:82/:85, record.rs:53 `decide` (a fixture's in tests_support);
  node sign_in.rs:336/:353 `of`/`forget`, synced.rs:336/:350
  `kept_files`/`kept_at` (one calls the other), redeem.rs:124/:131,
  choices.rs:79/:88 `load`/`read` (one calls the other), rail.rs:606/:615
  `live_push`/`live_drain`, lend.rs:153/:157 `from_mbit`/`bps`; web
  lend.js:174/:177 `cookieFor`/`keepJar`. Different bodies, nothing said
  twice.
- **origin.rs:114-153 `kept_to_reads` and surface.rs:234-271 `reads`**
  (0.912) are two deciders of one question: may the page of the lend
  `app` reach the lend `label`. They answer an absence differently:
  `kept_to_reads` asks the two lenders and refuses a silent one as
  could-not-judge (`away`), where `reads` asks every lender of the
  shared groups and lists a silent one as `unanswered`. One decider would
  change what a page is refused, and how: a follow-up, not a cut here
  (NEEDS_HUMAN.md).

## 4. Said twice — the open findings

The open rows of `ralph/REVIEW_FINDINGS.md` that name two readers or
two deciders in counted code, each read at `9267bc1`:

| # | Site | Lines a fix saves | Why not here |
|---|---|---:|---|
| 136 | cli.rs:891 `ask_running` reads the API's file by hand beside api.rs:58 `kept_address` | 0 | asking `kept_address` costs the lines it saves, and a file svrngs cannot read would be said in other words: a change in what a person is told |
| 135 | cli.rs:1542-1545 `join_group` infers the posture `SHIPPED` decides | 0 | a change of signature (17 callers), no line saved |
| 113 | home/home.js `uses` and web/app.js `uses`, the walk around question.js's `uses` | ~5 | each fetches by its own transport (the page's own `get`, a dial), and the "not lent" line differs ("groups", "group"): one wording changes what a person sees |
| 42, 48, 63, 89 | issuer.rs / web/src/lib.rs; the 32-byte key's reads in web/src; lend.js / shim.js `PAGE_COOKIES`; sign_in.rs / lend.js | — | lean-4 (a) said why not (f22f7cb): copies that decide differently, or two runtimes with no crate both link |
| 137, 7, 8, 62, 68, 70, 71 | test code and `apps/` | — | no budget line |

The near-clone pair above, `kept_to_reads` and `reads`, is the one more
"two deciders" the survey found; it has no number yet.

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §1, the compiler: `new_link`, `on_loopback_only`, `standing_in`, `Node::say_relay`, `LINK_LIFE_SECS`'s re-export | 29 | 0 |
| §2, `scripts/ralph-mark.sh` | 21 | 0 |
| §3, the clone report | 0 | 0 |
| §4, the open findings | 0 | 0 |
| **Total** | **50** | **0** |

16,036 − 50 = **15,986**, 187 past 15,799. svrngs-core stays 3,121:
nothing in it is unreached or written twice by these instruments. The
cuts are measured again in each cut's commit; the shortfall, and what
could close it with a change or a decision, is NEEDS_HUMAN.md.
